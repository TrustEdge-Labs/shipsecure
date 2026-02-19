use axum::http::Method;
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use axum_jwt_auth::RemoteJwksDecoder;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::Span;
use tracing_subscriber::EnvFilter;
use tracing_panic::panic_hook;

// Import from lib
use shipsecure::api::auth::ClerkClaims;
use shipsecure::api::scans::{self, AppState};
use shipsecure::api::{domains, health, results, stats, users, webhooks};
use shipsecure::metrics;
use shipsecure::api::metrics as api_metrics;
use shipsecure::orchestrator::ScanOrchestrator;
use shipsecure::RequestId;

fn validate_required_env_vars(vars: &[&str]) -> Result<(), String> {
    let mut missing = Vec::new();
    for var in vars {
        if std::env::var(var).is_err() {
            missing.push(*var);
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "Missing required environment variables:\n  - {}\n\nSee .env.example for configuration.",
            missing.join("\n  - ")
        ));
    }
    Ok(())
}

fn build_env_filter() -> EnvFilter {
    // RUST_LOG overrides everything when set
    if std::env::var("RUST_LOG").is_ok() {
        return EnvFilter::from_default_env();
    }
    // Sensible defaults based on build profile
    let defaults = if cfg!(debug_assertions) {
        "debug,hyper=info,sqlx=info,tower=info,tower_http=info,reqwest=info,h2=info"
    } else {
        "info,hyper=warn,sqlx=warn,tower=warn,tower_http=warn,reqwest=warn,h2=warn"
    };
    EnvFilter::new(defaults)
}

fn init_logging() -> (String, String) {
    let log_format = std::env::var("LOG_FORMAT").unwrap_or_default();
    let filter_description = if std::env::var("RUST_LOG").is_ok() {
        "RUST_LOG".to_string()
    } else if cfg!(debug_assertions) {
        "defaults (debug)".to_string()
    } else {
        "defaults (release)".to_string()
    };

    if log_format == "json" {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(build_env_filter())
            .with_target(true)
            .with_thread_ids(false)
            .with_thread_names(false)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_ansi(false)
            .with_env_filter(build_env_filter())
            .with_target(true)
            .init();
    }

    (
        if log_format == "json" { "json".to_string() } else { "text".to_string() },
        filter_description
    )
}

async fn inject_request_id(
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let request_id = uuid::Uuid::new_v4();
    request.extensions_mut().insert(RequestId(request_id));
    next.run(request).await
}

async fn reject_scans_during_shutdown(
    axum::extract::State(state): axum::extract::State<AppState>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // Only reject scan creation endpoints during shutdown
    // Per user decision: scan creation returns 503, other endpoints keep working
    let is_scan_creation = request.method() == Method::POST
        && request.uri().path() == "/api/v1/scans";

    if is_scan_creation && state.shutdown_token.is_cancelled() {
        let error_body = serde_json::json!({"error": "Service shutting down"});
        return (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(error_body),
        ).into_response();
    }

    next.run(request).await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received SIGINT, initiating graceful shutdown");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, initiating graceful shutdown");
        },
    }
}

fn parse_shutdown_timeout() -> Duration {
    let timeout_secs: u64 = std::env::var("SHUTDOWN_TIMEOUT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(90);
    Duration::from_secs(timeout_secs)
}

#[tokio::main]
async fn main() {
    // Load .env
    dotenvy::dotenv().ok();

    // Validate required env vars before anything else
    validate_required_env_vars(&[
        "DATABASE_URL",
        "PORT",
        "SHIPSECURE_BASE_URL",
        "FRONTEND_URL",
        "MAX_CONCURRENT_SCANS",
        "CLERK_JWKS_URL",
    ]).expect("Configuration error");

    // Init logging with format switching
    let (log_format, filter_description) = init_logging();

    // Install panic hook for structured panic logging
    std::panic::set_hook(Box::new(panic_hook));

    // Install metrics recorder
    let metrics_handle = metrics::install_metrics_recorder();

    // Database pool
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Parse port and max concurrent scans
    let port: u16 = std::env::var("PORT")
        .expect("PORT must be set")
        .parse()
        .expect("PORT must be a valid number");

    // Log startup banner with key configuration
    tracing::info!(
        log_format = %log_format,
        log_filter = %filter_description,
        port = port,
        db_connected = true,
        "ShipSecure API starting"
    );

    // Parse max concurrent scans from env var
    let max_concurrent: usize = std::env::var("MAX_CONCURRENT_SCANS")
        .expect("MAX_CONCURRENT_SCANS must be set")
        .parse()
        .expect("MAX_CONCURRENT_SCANS must be a valid number");

    // Parse shutdown timeout configuration
    let shutdown_timeout = parse_shutdown_timeout();

    // Create shutdown coordination primitives
    let task_tracker = TaskTracker::new();
    let shutdown_token = CancellationToken::new();

    // Spawn retention cleanup task (hourly DELETE of expired scans)
    // Must be spawned before task_tracker moves into ScanOrchestrator::new()
    shipsecure::cleanup::spawn_cleanup_task(
        pool.clone(),
        &task_tracker,
        shutdown_token.clone(),
    );

    // Create orchestrator with configurable concurrency
    let orchestrator = Arc::new(ScanOrchestrator::new(pool.clone(), max_concurrent, task_tracker, shutdown_token.clone()));

    // Create health cache
    let health_cache = health::HealthCache::new();

    // Initialize JWKS decoder for Clerk JWT verification
    // Fetches public keys from Clerk's JWKS endpoint and caches them with background refresh.
    let jwks_url = std::env::var("CLERK_JWKS_URL")
        .expect("CLERK_JWKS_URL must be set");
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);
    // Clerk session tokens do not include an aud claim by default
    validation.validate_aud = false;
    let jwks_decoder = RemoteJwksDecoder::builder()
        .jwks_url(jwks_url)
        .validation(validation)
        .build()
        .expect("Failed to build JWKS decoder");
    let jwks_decoder = Arc::new(jwks_decoder);
    let _jwks_shutdown = jwks_decoder
        .initialize()
        .await
        .expect("Failed to initialize JWKS decoder");
    // Cast to Decoder<ClerkClaims> = Arc<dyn JwtDecoder<ClerkClaims> + Send + Sync>
    let jwt_decoder: axum_jwt_auth::Decoder<ClerkClaims> = jwks_decoder;

    // App state
    let state = AppState {
        pool: pool.clone(),
        orchestrator: orchestrator.clone(),
        health_cache,
        metrics_handle: metrics_handle.clone(),
        shutdown_token: shutdown_token.clone(),
        jwt_decoder,
    };

    // CORS middleware — restrict origin to configured frontend URL
    let frontend_url: axum::http::HeaderValue = std::env::var("FRONTEND_URL")
        .expect("FRONTEND_URL must be set")
        .parse()
        .expect("FRONTEND_URL must be a valid header value");
    let cors = CorsLayer::new()
        .allow_origin(frontend_url)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::AUTHORIZATION]);

    // TraceLayer middleware for request tracing
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &axum::http::Request<axum::body::Body>| {
            let request_id = request.extensions()
                .get::<RequestId>()
                .map(|r| r.0)
                .unwrap_or_else(uuid::Uuid::new_v4);
            tracing::info_span!(
                "http_request",
                request_id = %request_id,
                method = %request.method(),
                uri = %request.uri().path(),
                status_code = tracing::field::Empty,
                latency_ms = tracing::field::Empty,
            )
        })
        .on_response(|response: &axum::http::Response<_>, latency: Duration, span: &Span| {
            let status = response.status();
            let latency_ms = latency.as_millis() as u64;
            span.record("status_code", status.as_u16());
            span.record("latency_ms", latency_ms);
            if status.is_client_error() || status.is_server_error() {
                tracing::info!("http_response");
            } else {
                tracing::debug!("http_response");
            }
        });

    // Health check router — separate from traced routes
    let health_router = Router::new()
        .route("/health", get(health::health_liveness))
        .route("/health/ready", get(health::health_readiness))
        .with_state(state.clone());

    // Metrics router — separate from traced routes
    let metrics_router = Router::new()
        .route("/metrics", get(api_metrics::metrics_handler))
        .with_state(state.clone());

    // Router
    let app = Router::new()
        // API routes — these get traced
        .route("/api/v1/scans", post(scans::create_scan))
        .route("/api/v1/scans/{id}", get(scans::get_scan))
        .route("/api/v1/results/{token}", get(results::get_results_by_token))
        .route(
            "/api/v1/results/{token}/download",
            get(results::download_results_markdown),
        )
        .route("/api/v1/quota", get(scans::get_quota))
        .route("/api/v1/stats/scan-count", get(stats::get_scan_count))
        .route("/api/v1/webhooks/clerk", post(webhooks::handle_clerk_webhook))
        // Domain verification routes
        .route("/api/v1/domains/verify-start", post(domains::verify_start))
        .route("/api/v1/domains/verify-confirm", post(domains::verify_confirm))
        .route("/api/v1/domains/verify-check", post(domains::verify_check))
        .route("/api/v1/domains", get(domains::list_domains))
        .route("/api/v1/users/me/scans", get(users::get_user_scans))
        .layer(axum::middleware::from_fn(metrics::middleware::track_http_metrics))
        .layer(cors)
        .layer(trace_layer)
        .layer(middleware::from_fn(inject_request_id))
        .layer(axum::middleware::from_fn_with_state(state.clone(), reject_scans_during_shutdown))
        .with_state(state)
        // Health checks and metrics — merged AFTER layers, bypass tracing
        .merge(health_router)
        .merge(metrics_router)
        .into_make_service_with_connect_info::<SocketAddr>();

    // Bind and serve
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind");

    // Log startup with shutdown config
    tracing::info!(
        addr = %addr,
        shutdown_timeout_seconds = shutdown_timeout.as_secs(),
        "Listening"
    );

    // Serve with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server error");

    // -- At this point, shutdown signal was received and HTTP server stopped accepting --

    // Initiate orchestrator shutdown (close tracker + cancel token)
    tracing::info!(
        timeout_seconds = shutdown_timeout.as_secs(),
        "Graceful shutdown initiated, draining in-flight scans"
    );
    orchestrator.initiate_shutdown();

    // Periodic logging + wait with timeout
    let start = std::time::Instant::now();
    let log_interval = Duration::from_secs(5);

    let drain_future = orchestrator.wait_for_drain();
    tokio::pin!(drain_future);

    loop {
        let remaining = shutdown_timeout.checked_sub(start.elapsed()).unwrap_or(Duration::ZERO);
        if remaining.is_zero() {
            // Timeout expired
            let (active, _max) = orchestrator.get_capacity();
            tracing::warn!(
                active_scans = active,
                elapsed_seconds = start.elapsed().as_secs(),
                timeout_seconds = shutdown_timeout.as_secs(),
                "Shutdown forced: {} scans remaining after {}s",
                active,
                start.elapsed().as_secs()
            );
            break;
        }

        // Wait for either: all tasks done, or next log interval
        let wait_duration = remaining.min(log_interval);
        tokio::select! {
            _ = &mut drain_future => {
                // All tasks completed gracefully -- no summary log per user decision
                break;
            }
            _ = tokio::time::sleep(wait_duration) => {
                // Log progress
                let (active, _max) = orchestrator.get_capacity();
                if active == 0 {
                    break; // Clean exit
                }
                tracing::info!(
                    active_scans = active,
                    elapsed_seconds = start.elapsed().as_secs(),
                    timeout_seconds = shutdown_timeout.as_secs(),
                    "shutdown_progress"
                );
            }
        }
    }

    // Exit with code 0 -- per user decision, clean exit prevents systemd restart
    std::process::exit(0);
}
