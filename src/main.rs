use axum::{routing::get, Router};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Load .env file
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trustedge_audit=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get database URL and port
    let database_url = env::var("DATABASE_URL").ok();
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    // Create database pool if DATABASE_URL is set
    let _pool = if let Some(url) = database_url {
        match PgPoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
        {
            Ok(pool) => {
                tracing::info!("Database connection established");

                // Run migrations (handle gracefully if it fails)
                if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
                    tracing::warn!("Failed to run migrations: {}. Continuing without migrations.", e);
                }

                Some(pool)
            }
            Err(e) => {
                tracing::warn!("Failed to connect to database: {}. Starting without database.", e);
                None
            }
        }
    } else {
        tracing::warn!("DATABASE_URL not set. Starting without database.");
        None
    };

    // Build router
    let app = Router::new().route("/health", get(health_check));

    // Bind server
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind server");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}

async fn health_check() -> &'static str {
    "ok"
}
