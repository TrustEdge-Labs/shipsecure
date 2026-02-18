pub mod api;
pub mod db;
pub mod email;
pub mod metrics;
pub mod models;
pub mod orchestrator;
pub mod rate_limit;
pub mod scanners;
pub mod ssrf;

// RequestId newtype for Axum Extension
#[derive(Clone, Debug)]
pub struct RequestId(pub uuid::Uuid);
