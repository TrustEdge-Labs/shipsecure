use axum::http::Method;
use axum::routing::{get, post};
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;
use tracing_panic::panic_hook;

// Import from lib
use trustedge_audit::api::scans::{self, AppState};
use trustedge_audit::api::{checkout, results, stats, webhooks};
use trustedge_audit::orchestrator::ScanOrchestrator;

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

#[tokio::main]
async fn main() {
    // Load .env
    dotenvy::dotenv().ok();

    // Validate required env vars before anything else
    validate_required_env_vars(&[
        "DATABASE_URL",
        "PORT",
        "TRUSTEDGE_BASE_URL",
        "FRONTEND_URL",
        "MAX_CONCURRENT_SCANS",
    ]).expect("Configuration error");

    // Init logging with format switching
    let (log_format, filter_description) = init_logging();

    // Install panic hook for structured panic logging
    std::panic::set_hook(Box::new(panic_hook));

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
        "TrustEdge Audit API starting"
    );

    // Parse max concurrent scans from env var
    let max_concurrent: usize = std::env::var("MAX_CONCURRENT_SCANS")
        .expect("MAX_CONCURRENT_SCANS must be set")
        .parse()
        .expect("MAX_CONCURRENT_SCANS must be a valid number");

    // Create orchestrator with configurable concurrency
    let orchestrator = Arc::new(ScanOrchestrator::new(pool.clone(), max_concurrent));

    // App state
    let state = AppState {
        pool: pool.clone(),
        orchestrator,
    };

    // CORS middleware for frontend communication
    let cors = CorsLayer::new()
        .allow_origin(Any) // For development; restrict in production
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    // Router
    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/api/v1/scans", post(scans::create_scan))
        .route("/api/v1/scans/{id}", get(scans::get_scan))
        .route("/api/v1/results/{token}", get(results::get_results_by_token))
        .route(
            "/api/v1/results/{token}/download",
            get(results::download_results_markdown),
        )
        .route("/api/v1/stats/scan-count", get(stats::get_scan_count))
        .route("/api/v1/checkout", post(checkout::create_checkout))
        .route("/api/v1/webhooks/stripe", post(webhooks::handle_stripe_webhook))
        .layer(cors)
        .with_state(state)
        .into_make_service_with_connect_info::<SocketAddr>();

    // Bind and serve
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind");

    tracing::info!(addr = %addr, "Listening");

    axum::serve(listener, app).await.expect("Server error");
}
