use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

use crate::ssrf::SsrfError;

#[derive(Debug)]
pub enum ApiError {
    ValidationError(String),
    SsrfBlocked(String),
    RateLimited(String),
    RateLimitedWithReset {
        message: String,
        resets_at: chrono::DateTime<chrono::Utc>,
    },
    NotFound,
    Unauthorized,
    InternalError(String),
    Custom {
        status: StatusCode,
        error_type: String,
        title: String,
        detail: String,
    },
}

#[derive(Serialize, Deserialize)]
struct ProblemDetails {
    #[serde(rename = "type")]
    problem_type: String,
    title: String,
    status: u16,
    detail: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // RateLimitedWithReset needs a custom JSON body with resets_at — handle before standard ProblemDetails
        if let ApiError::RateLimitedWithReset { message, resets_at } = self {
            let body = serde_json::json!({
                "type": "https://shipsecure.ai/errors/rate-limited",
                "title": "Rate Limit Exceeded",
                "status": 429,
                "detail": message,
                "resets_at": resets_at.to_rfc3339(),
            });
            return (
                StatusCode::TOO_MANY_REQUESTS,
                [(axum::http::header::CONTENT_TYPE, "application/problem+json")],
                body.to_string(),
            )
                .into_response();
        }

        let (problem_type, title, status, detail) = match self {
            ApiError::ValidationError(msg) => (
                "about:blank".to_string(),
                "Validation Error".to_string(),
                StatusCode::BAD_REQUEST,
                msg,
            ),
            ApiError::SsrfBlocked(msg) => (
                "https://shipsecure.ai/errors/ssrf-blocked".to_string(),
                "Target URL Not Allowed".to_string(),
                StatusCode::BAD_REQUEST,
                msg,
            ),
            ApiError::RateLimited(msg) => (
                "https://shipsecure.ai/errors/rate-limited".to_string(),
                "Rate Limit Exceeded".to_string(),
                StatusCode::TOO_MANY_REQUESTS,
                msg,
            ),
            ApiError::RateLimitedWithReset { .. } => unreachable!("handled above"),
            ApiError::NotFound => (
                "about:blank".to_string(),
                "Not Found".to_string(),
                StatusCode::NOT_FOUND,
                "The requested scan was not found.".to_string(),
            ),
            ApiError::Unauthorized => (
                "about:blank".to_string(),
                "Unauthorized".to_string(),
                StatusCode::UNAUTHORIZED,
                "Authentication required".to_string(),
            ),
            ApiError::InternalError(msg) => {
                // Log the actual error but don't expose it to the client
                tracing::error!("Internal error: {}", msg);
                (
                    "about:blank".to_string(),
                    "Internal Server Error".to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An unexpected error occurred. Please try again later.".to_string(),
                )
            }
            ApiError::Custom {
                status,
                error_type,
                title,
                detail,
            } => (error_type, title, status, detail),
        };

        let problem = ProblemDetails {
            problem_type,
            title,
            status: status.as_u16(),
            detail,
        };

        let body = serde_json::to_string(&problem).unwrap_or_else(|_| {
            r#"{"type":"about:blank","title":"Internal Server Error","status":500,"detail":"Failed to serialize error response"}"#.to_string()
        });

        (
            status,
            [(axum::http::header::CONTENT_TYPE, "application/problem+json")],
            body,
        )
            .into_response()
    }
}

impl From<SsrfError> for ApiError {
    fn from(err: SsrfError) -> Self {
        ApiError::SsrfBlocked(err.to_string())
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        ApiError::InternalError(format!("Database error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::header::CONTENT_TYPE;
    use http_body_util::BodyExt;

    async fn extract_response_parts(response: Response) -> (StatusCode, String, String) {
        let status = response.status();
        let content_type = response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body = String::from_utf8(body_bytes.to_vec()).unwrap();

        (status, content_type, body)
    }

    #[tokio::test]
    async fn test_validation_error() {
        let error = ApiError::ValidationError("Invalid email".to_string());
        let response = error.into_response();
        let (status, content_type, body) = extract_response_parts(response).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(content_type, "application/problem+json");

        let problem: ProblemDetails = serde_json::from_str(&body).unwrap();
        assert_eq!(problem.problem_type, "about:blank");
        assert_eq!(problem.title, "Validation Error");
        assert_eq!(problem.status, 400);
        assert_eq!(problem.detail, "Invalid email");
    }

    #[tokio::test]
    async fn test_ssrf_blocked() {
        let error = ApiError::SsrfBlocked("Private IP blocked".to_string());
        let response = error.into_response();
        let (status, content_type, body) = extract_response_parts(response).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(content_type, "application/problem+json");

        let problem: ProblemDetails = serde_json::from_str(&body).unwrap();
        assert_eq!(
            problem.problem_type,
            "https://shipsecure.ai/errors/ssrf-blocked"
        );
        assert_eq!(problem.title, "Target URL Not Allowed");
        assert_eq!(problem.status, 400);
    }

    #[tokio::test]
    async fn test_rate_limited() {
        let error = ApiError::RateLimited("Too many requests".to_string());
        let response = error.into_response();
        let (status, content_type, body) = extract_response_parts(response).await;

        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(content_type, "application/problem+json");

        let problem: ProblemDetails = serde_json::from_str(&body).unwrap();
        assert_eq!(
            problem.problem_type,
            "https://shipsecure.ai/errors/rate-limited"
        );
        assert_eq!(problem.title, "Rate Limit Exceeded");
        assert_eq!(problem.status, 429);
    }

    #[tokio::test]
    async fn test_not_found() {
        let error = ApiError::NotFound;
        let response = error.into_response();
        let (status, content_type, body) = extract_response_parts(response).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(content_type, "application/problem+json");

        let problem: ProblemDetails = serde_json::from_str(&body).unwrap();
        assert_eq!(problem.status, 404);
    }

    #[tokio::test]
    async fn test_internal_error() {
        let error = ApiError::InternalError("Database connection failed".to_string());
        let response = error.into_response();
        let (status, content_type, body) = extract_response_parts(response).await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(content_type, "application/problem+json");

        let problem: ProblemDetails = serde_json::from_str(&body).unwrap();
        assert_eq!(problem.status, 500);
        // Should not leak internal details
        assert!(!problem.detail.contains("Database connection failed"));
    }
}
