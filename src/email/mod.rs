pub mod templates;

use std::fmt;

#[derive(Debug, Clone)]
pub struct FindingsSummary {
    pub critical: i64,
    pub high: i64,
    pub medium: i64,
    pub low: i64,
    pub total: i64,
}

#[derive(Debug)]
pub enum EmailError {
    ApiKeyMissing,
    SendFailed(String),
    HttpError(reqwest::Error),
}

impl fmt::Display for EmailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmailError::ApiKeyMissing => write!(f, "RESEND_API_KEY environment variable not set"),
            EmailError::SendFailed(msg) => write!(f, "Email send failed: {}", msg),
            EmailError::HttpError(e) => write!(f, "HTTP error: {}", e),
        }
    }
}

impl std::error::Error for EmailError {}

impl From<reqwest::Error> for EmailError {
    fn from(e: reqwest::Error) -> Self {
        EmailError::HttpError(e)
    }
}

/// Send a scan completion email via Resend API
pub async fn send_scan_complete_email(
    to: &str,
    target_url: &str,
    grade: &str,
    findings_summary: &FindingsSummary,
    results_token: &str,
    base_url: &str,
) -> Result<(), EmailError> {
    // Check for API key
    let api_key = match std::env::var("RESEND_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            tracing::warn!("RESEND_API_KEY not set, skipping email send (development mode)");
            return Err(EmailError::ApiKeyMissing);
        }
    };

    // Generate email content
    let results_url = format!("{}/results/{}", base_url, results_token);
    let expires_at = chrono::Utc::now() + chrono::Duration::days(3);
    let expires_formatted = expires_at.format("%B %d, %Y").to_string();

    let subject = format!("Scan Complete: {} Grade for {}", grade, target_url);
    let html_body = templates::scan_complete_html(
        target_url,
        grade,
        findings_summary,
        &results_url,
        &expires_formatted,
    );

    // Build request body
    let request_body = serde_json::json!({
        "from": "TrustEdge Audit <scans@trustedgeaudit.com>",
        "to": [to],
        "subject": subject,
        "html": html_body,
    });

    // Send via Resend API
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.resend.com/emails")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(EmailError::SendFailed(format!(
            "Resend API returned {}: {}",
            status,
            error_body
        )));
    }

    tracing::info!("Successfully sent completion email to {} for scan of {}", to, target_url);
    Ok(())
}
