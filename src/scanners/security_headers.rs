use crate::models::finding::{Finding, Severity};
use chrono::Utc;
use reqwest::header::{HeaderMap, HeaderName};
use std::fmt;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug)]
pub enum ScannerError {
    HttpError(reqwest::Error),
    Timeout,
    Other(String),
}

impl fmt::Display for ScannerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScannerError::HttpError(e) => write!(f, "HTTP error: {}", e),
            ScannerError::Timeout => write!(f, "Request timeout"),
            ScannerError::Other(msg) => write!(f, "Scanner error: {}", msg),
        }
    }
}

impl std::error::Error for ScannerError {}

impl From<reqwest::Error> for ScannerError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            ScannerError::Timeout
        } else {
            ScannerError::HttpError(e)
        }
    }
}

/// Scan a URL for security headers and return findings for missing headers
pub async fn scan_security_headers(url: &str) -> Result<Vec<Finding>, ScannerError> {
    // Create HTTP client with configuration
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("ShipSecure-Scanner/1.0")
        .build()
        .map_err(|e| ScannerError::Other(format!("Failed to build client: {}", e)))?;

    // Make GET request
    let response = client.get(url).send().await?;
    let headers = response.headers();

    // Collect findings for missing headers
    let mut findings = Vec::new();
    let raw_evidence = format_headers(headers);

    // Check each security header
    check_header(
        &mut findings,
        headers,
        "content-security-policy",
        "Missing Content-Security-Policy",
        "The Content-Security-Policy (CSP) header helps prevent cross-site scripting (XSS), clickjacking, and other code injection attacks by specifying approved sources of content.",
        "Add a Content-Security-Policy header with directives that match your application's needs. Example: Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'",
        Severity::High,
        &raw_evidence,
    );

    check_header(
        &mut findings,
        headers,
        "strict-transport-security",
        "Missing Strict-Transport-Security",
        "The Strict-Transport-Security (HSTS) header forces browsers to use HTTPS connections, protecting against man-in-the-middle attacks and cookie hijacking.",
        "Add a Strict-Transport-Security header to enforce HTTPS. Example: Strict-Transport-Security: max-age=31536000; includeSubDomains; preload",
        Severity::High,
        &raw_evidence,
    );

    check_header(
        &mut findings,
        headers,
        "x-frame-options",
        "Missing X-Frame-Options",
        "The X-Frame-Options header prevents clickjacking attacks by controlling whether your site can be embedded in frames or iframes.",
        "Add an X-Frame-Options header to prevent clickjacking. Example: X-Frame-Options: DENY or X-Frame-Options: SAMEORIGIN",
        Severity::Medium,
        &raw_evidence,
    );

    check_header(
        &mut findings,
        headers,
        "x-content-type-options",
        "Missing X-Content-Type-Options",
        "The X-Content-Type-Options header prevents MIME-sniffing attacks by instructing browsers to follow declared content types.",
        "Add the X-Content-Type-Options header. Example: X-Content-Type-Options: nosniff",
        Severity::Medium,
        &raw_evidence,
    );

    check_header(
        &mut findings,
        headers,
        "referrer-policy",
        "Missing Referrer-Policy",
        "The Referrer-Policy header controls how much referrer information is included with requests, protecting user privacy and preventing information leakage.",
        "Add a Referrer-Policy header to control referrer information. Example: Referrer-Policy: strict-origin-when-cross-origin or Referrer-Policy: no-referrer",
        Severity::Low,
        &raw_evidence,
    );

    check_header(
        &mut findings,
        headers,
        "permissions-policy",
        "Missing Permissions-Policy",
        "The Permissions-Policy header allows you to control which browser features and APIs can be used, reducing the attack surface.",
        "Add a Permissions-Policy header to restrict browser features. Example: Permissions-Policy: geolocation=(), camera=(), microphone=()",
        Severity::Low,
        &raw_evidence,
    );

    Ok(findings)
}

fn check_header(
    findings: &mut Vec<Finding>,
    headers: &HeaderMap,
    header_name: &str,
    title: &str,
    description: &str,
    remediation: &str,
    severity: Severity,
    raw_evidence: &str,
) {
    // Parse header name (this should always succeed for valid header names)
    let header_key = match HeaderName::try_from(header_name) {
        Ok(key) => key,
        Err(_) => return, // Skip invalid header names
    };

    // Check if header is present
    if !headers.contains_key(&header_key) {
        findings.push(Finding {
            id: Uuid::new_v4(),
            scan_id: Uuid::nil(), // Placeholder, will be set by caller
            scanner_name: "security_headers".to_string(),
            title: title.to_string(),
            description: description.to_string(),
            severity,
            remediation: remediation.to_string(),
            raw_evidence: Some(raw_evidence.to_string()),
            vibe_code: false,
            created_at: Utc::now().naive_utc(),
        });
    }
}

fn format_headers(headers: &HeaderMap) -> String {
    let mut lines = vec!["Response Headers:".to_string()];
    for (name, value) in headers.iter() {
        if let Ok(val_str) = value.to_str() {
            lines.push(format!("{}: {}", name, val_str));
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

    #[test]
    fn test_check_header_logic() {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        );

        let mut findings = Vec::new();
        let evidence = format_headers(&headers);

        // Should NOT create finding for present header
        check_header(
            &mut findings,
            &headers,
            "x-frame-options",
            "Missing X-Frame-Options",
            "Description",
            "Remediation",
            Severity::Medium,
            &evidence,
        );
        assert_eq!(findings.len(), 0);

        // Should create finding for missing header
        check_header(
            &mut findings,
            &headers,
            "content-security-policy",
            "Missing Content-Security-Policy",
            "Description",
            "Remediation",
            Severity::High,
            &evidence,
        );
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].title, "Missing Content-Security-Policy");
        assert_eq!(findings[0].severity, Severity::High);
        assert_eq!(findings[0].scanner_name, "security_headers");
    }

    #[test]
    fn test_format_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        );
        headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("text/html"),
        );

        let formatted = format_headers(&headers);
        assert!(formatted.contains("Response Headers:"));
        assert!(formatted.contains("x-frame-options: DENY"));
        assert!(formatted.contains("content-type: text/html"));
    }
}
