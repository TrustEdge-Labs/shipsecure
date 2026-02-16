use crate::models::finding::{Finding, Severity};
use chrono::Utc;
use regex::Regex;
use std::collections::HashSet;
use url::Url;
use uuid::Uuid;

#[derive(Debug)]
pub enum ScannerError {
    HttpError(String),
    ParseError(String),
    Timeout,
}

impl std::fmt::Display for ScannerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::Timeout => write!(f, "Scanner timeout"),
        }
    }
}

impl std::error::Error for ScannerError {}

/// Pattern definition for secret detection
struct SecretPattern {
    name: &'static str,
    regex: Regex,
    severity: Severity,
    confidence: &'static str,
    advice: &'static str,
}

impl SecretPattern {
    fn new(name: &'static str, pattern: &str, severity: Severity, confidence: &'static str, advice: &'static str) -> Self {
        Self {
            name,
            regex: Regex::new(pattern).unwrap(),
            severity,
            confidence,
            advice,
        }
    }
}

lazy_static::lazy_static! {
    static ref SECRET_PATTERNS: Vec<SecretPattern> = vec![
        // HIGH confidence (format-validated)
        SecretPattern::new(
            "AWS Access Key",
            r"AKIA[0-9A-Z]{16}",
            Severity::Critical,
            "HIGH",
            "Rotate this key immediately in the AWS IAM console and use AWS Cognito or a backend proxy for client-side access."
        ),
        SecretPattern::new(
            "Stripe Live Secret Key",
            r"sk_live_[a-zA-Z0-9]{24,}",
            Severity::Critical,
            "HIGH",
            "Rotate this key immediately in the Stripe dashboard. Never expose secret keys in client-side code. Use publishable keys for client-side and keep secret keys on the server."
        ),
        SecretPattern::new(
            "Stripe Live Publishable Key",
            r"pk_live_[a-zA-Z0-9]{24,}",
            Severity::Medium,
            "HIGH",
            "Publishable keys are meant to be public, but verify this is intentional. If this site should use test mode, switch to pk_test_ keys."
        ),
        SecretPattern::new(
            "GitHub Token (Personal)",
            r"ghp_[a-zA-Z0-9]{36}",
            Severity::Critical,
            "HIGH",
            "Revoke this token immediately at https://github.com/settings/tokens and create a new one. Never commit tokens to client-side code."
        ),
        SecretPattern::new(
            "GitHub Token (OAuth)",
            r"gho_[a-zA-Z0-9]{36}",
            Severity::Critical,
            "HIGH",
            "Revoke this OAuth token immediately and rotate credentials. Use GitHub Apps or OAuth flows properly."
        ),
        SecretPattern::new(
            "Slack Token",
            r"xox[bprs]-[a-zA-Z0-9-]+",
            Severity::High,
            "HIGH",
            "Rotate this token immediately in your Slack workspace settings. Use proper OAuth flows for client-side integration."
        ),
        SecretPattern::new(
            "Twilio API Key",
            r"SK[a-f0-9]{32}",
            Severity::High,
            "HIGH",
            "Rotate this API key immediately in the Twilio console. Keep API keys server-side only."
        ),

        // MEDIUM confidence (pattern + context)
        SecretPattern::new(
            "Supabase Anon Key",
            r"eyJ[a-zA-Z0-9_-]+\.eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+",
            Severity::Medium,
            "MEDIUM",
            "If this is a Supabase anon key, verify your Row Level Security (RLS) policies are properly configured. Anon keys are meant to be public but rely on RLS for security."
        ),
        SecretPattern::new(
            "Firebase API Key",
            r#"apiKey\s*[:=]\s*["']([A-Za-z0-9_-]{39})["']"#,
            Severity::Medium,
            "MEDIUM",
            "Firebase API keys in client-side code are normal, but ensure Firebase Security Rules are properly configured to restrict access."
        ),
        SecretPattern::new(
            "Generic API Key",
            r#"(?:api[_-]?key|apikey)\s*[:=]\s*["']([a-zA-Z0-9_-]{20,})["']"#,
            Severity::Medium,
            "MEDIUM",
            "Review if this API key should be exposed client-side. If not, move it to server-side environment variables."
        ),
    ];

    static ref SCRIPT_TAG_REGEX: Regex = Regex::new(r#"<script[^>]+src=["']([^"']+\.js[^"']*)["']"#).unwrap();

    static ref TEST_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?i)test").unwrap(),
        Regex::new(r"(?i)example").unwrap(),
        Regex::new(r"(?i)placeholder").unwrap(),
        Regex::new(r"(?i)your[_-]?key[_-]?here").unwrap(),
        Regex::new(r"(?i)change[_-]?me").unwrap(),
        Regex::new(r"(?i)todo").unwrap(),
        Regex::new(r"xxx+").unwrap(),
        Regex::new(r"000+").unwrap(),
        Regex::new(r"123+").unwrap(),
        Regex::new(r"^(.)\1+$").ok().unwrap_or_else(|| Regex::new(r"^(a{2,}|b{2,}|c{2,}|d{2,}|e{2,}|f{2,}|0{2,}|1{2,}|x{2,}|X{2,})$").unwrap()), // All same character (backrefs unsupported, match common placeholders)
    ];
}

/// Scan a target URL for hardcoded secrets in JavaScript bundles
///
/// # Arguments
/// * `url` - The target URL to scan
/// * `max_files` - Maximum number of JS files to scan (e.g., 20 for free, 50 for paid)
pub async fn scan_js_secrets(url: &str, max_files: usize) -> Result<Vec<Finding>, ScannerError> {
    let mut findings = Vec::new();

    // Discover JavaScript URLs
    let js_urls = discover_js_urls(url).await?;

    // Limit to max_files
    let js_urls: Vec<String> = js_urls.into_iter().take(max_files).collect();

    // Fetch and scan each JS file concurrently
    let scan_tasks: Vec<_> = js_urls.iter()
        .map(|js_url| scan_single_js_file(js_url.clone()))
        .collect();

    let results = futures::future::join_all(scan_tasks).await;

    for result in results {
        if let Ok(mut file_findings) = result {
            findings.append(&mut file_findings);
        }
        // Silently skip failed fetches (404s, timeouts, etc.)
    }

    Ok(findings)
}

/// Discover JavaScript URLs from the target page
async fn discover_js_urls(url: &str) -> Result<HashSet<String>, ScannerError> {
    let mut js_urls = HashSet::new();

    let base_url = Url::parse(url)
        .map_err(|e| ScannerError::ParseError(format!("Invalid URL: {}", e)))?;

    // Fetch the HTML page
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("ShipSecure-Scanner/1.0")
        .build()
        .map_err(|e| ScannerError::HttpError(e.to_string()))?;

    let response = client.get(url)
        .send()
        .await
        .map_err(|e| ScannerError::HttpError(e.to_string()))?;

    let html = response.text()
        .await
        .map_err(|e| ScannerError::HttpError(e.to_string()))?;

    // Parse script tags
    for cap in SCRIPT_TAG_REGEX.captures_iter(&html) {
        if let Some(src) = cap.get(1) {
            let src_str = src.as_str();
            if let Ok(absolute_url) = base_url.join(src_str) {
                js_urls.insert(absolute_url.to_string());
            }
        }
    }

    // Add common bundle paths
    let common_paths = vec![
        "/main.js",
        "/app.js",
        "/bundle.js",
        "/static/js/main.js",
        "/_next/static/chunks/main.js",
    ];

    for path in common_paths {
        if let Ok(absolute_url) = base_url.join(path) {
            js_urls.insert(absolute_url.to_string());
        }
    }

    Ok(js_urls)
}

/// Scan a single JavaScript file for secrets
async fn scan_single_js_file(js_url: String) -> Result<Vec<Finding>, ScannerError> {
    let mut findings = Vec::new();

    // Fetch JS content with timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("ShipSecure-Scanner/1.0")
        .build()
        .map_err(|e| ScannerError::HttpError(e.to_string()))?;

    let response = client.get(&js_url)
        .send()
        .await
        .map_err(|e| ScannerError::HttpError(e.to_string()))?;

    // Only scan successful responses
    if !response.status().is_success() {
        return Ok(findings);
    }

    // Limit to first 2MB
    let bytes = response.bytes()
        .await
        .map_err(|e| ScannerError::HttpError(e.to_string()))?;

    let content = if bytes.len() > 2 * 1024 * 1024 {
        String::from_utf8_lossy(&bytes[..2 * 1024 * 1024]).to_string()
    } else {
        String::from_utf8_lossy(&bytes).to_string()
    };

    // Scan for each pattern
    for pattern in SECRET_PATTERNS.iter() {
        for mat in pattern.regex.find_iter(&content) {
            let matched_text = mat.as_str();

            // Apply false positive filtering
            if is_false_positive(matched_text, &content[mat.start()..]) {
                continue;
            }

            // Check for Stripe test keys
            if matched_text.contains("sk_test_") || matched_text.contains("pk_test_") {
                continue;
            }

            // Generate finding
            let redacted = redact_secret(matched_text);
            let evidence = extract_evidence(&content, mat.start(), mat.end());

            findings.push(Finding {
                id: Uuid::new_v4(),
                scan_id: Uuid::nil(), // Placeholder, will be set by caller
                scanner_name: "js_secrets".to_string(),
                severity: pattern.severity.clone(),
                title: format!("Hardcoded {} in JavaScript", pattern.name),
                description: format!(
                    "A hardcoded {} was found in {}. Hardcoded secrets in client-side JavaScript are visible to anyone who views your site's source code. This could allow unauthorized access to your {} account or services.",
                    pattern.name,
                    js_url,
                    pattern.name
                ),
                remediation: format!(
                    "Remove the secret from your JavaScript code. Use environment variables on the server side. For {}: {}",
                    pattern.name,
                    pattern.advice
                ),
                raw_evidence: Some(format!("Found in {}: {}\nContext: {}", js_url, redacted, evidence)),
                vibe_code: false,
                created_at: Utc::now().naive_utc(),
            });
        }
    }

    Ok(findings)
}

/// Check if a matched string is a false positive
fn is_false_positive(matched_text: &str, context: &str) -> bool {
    let check_text = matched_text.to_lowercase();

    // Check against test patterns
    for pattern in TEST_PATTERNS.iter() {
        if pattern.is_match(&check_text) {
            return true;
        }
    }

    // Check for known example values (take context sample)
    let context_sample = &context[..context.len().min(200)].to_lowercase();
    if context_sample.contains("example") || context_sample.contains("demo") {
        return true;
    }

    false
}

/// Redact middle of secret, showing only first 8 and last 4 characters
fn redact_secret(secret: &str) -> String {
    if secret.len() <= 12 {
        return "[REDACTED]".to_string();
    }

    let first = &secret[..8.min(secret.len())];
    let last = &secret[secret.len().saturating_sub(4)..];

    format!("{}...{}", first, last)
}

/// Extract evidence context around the match (up to 100 chars)
fn extract_evidence(content: &str, start: usize, end: usize) -> String {
    let context_start = start.saturating_sub(50);
    let context_end = (end + 50).min(content.len());

    let evidence = &content[context_start..context_end];

    // Clean up whitespace and newlines for readability
    evidence
        .replace('\n', " ")
        .replace('\r', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(100)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_secret() {
        assert_eq!(redact_secret("AKIAIOSFODNN7EXAMPLE"), "AKIAIOSF...MPLE");
        assert_eq!(redact_secret("short"), "[REDACTED]");
    }

    #[test]
    fn test_false_positive_detection() {
        assert!(is_false_positive("test_key_12345", "context"));
        assert!(is_false_positive("example_api_key", "context"));
        assert!(is_false_positive("YOUR_KEY_HERE", "context"));
        assert!(!is_false_positive("AKIAIOSFODNN7EXAMPLE", "production code"));
    }
}
