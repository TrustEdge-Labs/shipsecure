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

/// Send a paid audit email with PDF attachment via Resend API
pub async fn send_paid_audit_email(
    to: &str,
    target_url: &str,
    grade: &str,
    findings_summary: &FindingsSummary,
    results_token: &str,
    base_url: &str,
    pdf_bytes: Vec<u8>,
    scan_id: &str,
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

    let subject = format!("Your TrustEdge Deep Audit Report: {} Grade for {}", grade, target_url);

    // Build HTML body for paid audit email
    let html_body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>TrustEdge Deep Audit Report</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; border-radius: 8px; margin-bottom: 30px;">
        <h1 style="margin: 0 0 10px 0; font-size: 28px;">TrustEdge Deep Audit Complete</h1>
        <p style="margin: 0; font-size: 16px; opacity: 0.9;">Your comprehensive security analysis is ready</p>
    </div>

    <div style="background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 20px;">
        <h2 style="margin: 0 0 15px 0; font-size: 20px;">Overall Grade: <span style="color: #667eea; font-weight: bold;">{}</span></h2>
        <p style="margin: 0 0 10px 0;"><strong>Target:</strong> {}</p>
        <p style="margin: 0;"><strong>Findings:</strong> {} Critical, {} High, {} Medium, {} Low</p>
    </div>

    <div style="margin-bottom: 30px;">
        <h3 style="color: #667eea; margin-bottom: 15px;">Your Professional Report</h3>
        <p>Your detailed security audit report is attached to this email as a PDF. The report includes:</p>
        <ul style="line-height: 1.8;">
            <li>Executive summary with overall security grade</li>
            <li>Complete findings organized by severity</li>
            <li>Detailed remediation guidance for each issue</li>
            <li>Framework and platform-specific insights</li>
            <li>Prioritized remediation roadmap</li>
        </ul>
        <p>You can also view your results online:</p>
        <p style="text-align: center; margin: 25px 0;">
            <a href="{}" style="display: inline-block; background: #667eea; color: white; padding: 14px 30px; text-decoration: none; border-radius: 6px; font-weight: 600;">View Results Online</a>
        </p>
    </div>

    <div style="border-top: 2px solid #e9ecef; padding-top: 20px; color: #6c757d; font-size: 14px;">
        <p>Questions about your report? Reply to this email and we'll help you understand and prioritize the findings.</p>
        <p style="margin-bottom: 0;">— The TrustEdge Team</p>
    </div>
</body>
</html>"#,
        grade,
        target_url,
        findings_summary.critical,
        findings_summary.high,
        findings_summary.medium,
        findings_summary.low,
        results_url
    );

    // Base64-encode PDF
    let pdf_base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &pdf_bytes);

    // Get short scan ID for filename (first 8 chars)
    let scan_id_short = if scan_id.len() >= 8 {
        &scan_id[..8]
    } else {
        scan_id
    };

    // Build request body with attachment
    let request_body = serde_json::json!({
        "from": "TrustEdge Audit <scans@trustedgeaudit.com>",
        "to": [to],
        "subject": subject,
        "html": html_body,
        "attachments": [{
            "filename": format!("trustedge-deep-audit-{}.pdf", scan_id_short),
            "content": pdf_base64,
            "content_type": "application/pdf"
        }]
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

    tracing::info!("Successfully sent paid audit email with PDF attachment to {} for scan of {}", to, target_url);
    Ok(())
}
