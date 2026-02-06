use axum::http::Method;
use axum::routing::{get, post};
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

// Import from lib
use trustedge_audit::api::scans::{self, AppState};
use trustedge_audit::api::{checkout, results, stats, webhooks};
use trustedge_audit::orchestrator::ScanOrchestrator;

#[tokio::main]
async fn main() {
    // Load .env
    dotenvy::dotenv().ok();

    // Init tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

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

    // Create orchestrator (5 concurrent scans max)
    let orchestrator = Arc::new(ScanOrchestrator::new(pool.clone(), 5));

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
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a number");
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("TrustEdge Audit API listening on {}", addr);

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}
