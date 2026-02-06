use crate::models::finding::{Finding, Severity};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;
use url::Url;
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SslLabsResponse {
    status: String,
    endpoints: Option<Vec<Endpoint>>,
    #[serde(default)]
    status_message: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Endpoint {
    #[serde(default)]
    grade: Option<String>,
    #[serde(default)]
    details: Option<EndpointDetails>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct EndpointDetails {
    #[serde(default)]
    protocols: Vec<Protocol>,
    #[serde(default)]
    cert: Option<Certificate>,
    #[serde(default)]
    heartbleed: bool,
    #[serde(default)]
    poodle: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Protocol {
    name: String,
    version: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Certificate {
    #[serde(default)]
    not_after: Option<i64>, // Unix timestamp in milliseconds
}

/// Scan a URL for TLS/SSL configuration issues via SSL Labs API
pub async fn scan_tls(url: &str) -> Result<Vec<Finding>, ScannerError> {
    // Extract hostname from URL
    let parsed_url = Url::parse(url).map_err(|e| {
        ScannerError::Other(format!("Invalid URL: {}", e))
    })?;

    let hostname = parsed_url.host_str().ok_or_else(|| {
        ScannerError::Other("No hostname found in URL".to_string())
    })?;

    // Create HTTP client
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("TrustEdge-Scanner/0.1")
        .build()
        .map_err(|e| ScannerError::Other(format!("Failed to build client: {}", e)))?;

    // Start SSL Labs assessment
    let start_url = format!(
        "https://api.ssllabs.com/api/v4/analyze?host={}&startNew=on",
        hostname
    );

    let start_response = client.get(&start_url).send().await?;
    let _rate_limit_info = extract_rate_limits(&start_response);

    if !start_response.status().is_success() {
        let status_code = start_response.status();
        if status_code.as_u16() == 429 || status_code.as_u16() == 529 {
            return Ok(vec![create_informational_finding(
                "TLS analysis unavailable due to SSL Labs API rate limiting. Please try again later.",
            )]);
        }
        return Err(ScannerError::Other(format!(
            "SSL Labs API returned status: {}",
            status_code
        )));
    }

    // Poll for completion
    let poll_url = format!(
        "https://api.ssllabs.com/api/v4/analyze?host={}",
        hostname
    );

    const MAX_POLLS: u32 = 30; // 5 minutes at 10s intervals
    const POLL_INTERVAL_SECS: u64 = 10;

    for attempt in 0..MAX_POLLS {
        // Wait before polling (except first attempt)
        if attempt > 0 {
            tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
        }

        let poll_response = match client.get(&poll_url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                if e.is_timeout() {
                    continue; // Retry on timeout
                }
                return Err(e.into());
            }
        };

        // Check rate limits
        let current_rate_limit = extract_rate_limits(&poll_response);
        if let (Some(current), Some(max)) = (current_rate_limit.current, current_rate_limit.max) {
            if current >= max {
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        }

        // Handle HTTP errors
        let status = poll_response.status();
        if status.as_u16() == 429 {
            tokio::time::sleep(Duration::from_secs(60)).await;
            continue;
        }
        if status.as_u16() == 529 {
            tokio::time::sleep(Duration::from_secs(30)).await;
            continue;
        }
        if !status.is_success() {
            return Err(ScannerError::Other(format!(
                "SSL Labs API returned status: {}",
                status
            )));
        }

        // Parse response
        let ssl_response: SslLabsResponse = match poll_response.json().await {
            Ok(r) => r,
            Err(e) => {
                return Err(ScannerError::Other(format!(
                    "Failed to parse SSL Labs response: {}",
                    e
                )));
            }
        };

        // Check status
        match ssl_response.status.as_str() {
            "DNS" | "IN_PROGRESS" => {
                // Keep polling
                continue;
            }
            "READY" => {
                // Assessment complete, generate findings
                return Ok(generate_findings_from_response(&ssl_response, hostname));
            }
            "ERROR" => {
                let msg = ssl_response
                    .status_message
                    .unwrap_or_else(|| "SSL Labs analysis failed".to_string());
                return Ok(vec![create_informational_finding(&format!(
                    "TLS analysis could not be completed: {}",
                    msg
                ))]);
            }
            other => {
                return Err(ScannerError::Other(format!(
                    "Unexpected SSL Labs status: {}",
                    other
                )));
            }
        }
    }

    // Timeout after max polls
    Ok(vec![create_informational_finding(
        "TLS analysis timed out after 5 minutes. The SSL Labs API may be overloaded.",
    )])
}

struct RateLimitInfo {
    current: Option<u32>,
    max: Option<u32>,
}

fn extract_rate_limits(response: &reqwest::Response) -> RateLimitInfo {
    let current = response
        .headers()
        .get("X-Current-Assessments")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok());

    let max = response
        .headers()
        .get("X-Max-Assessments")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok());

    RateLimitInfo { current, max }
}

fn generate_findings_from_response(response: &SslLabsResponse, hostname: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let now = Utc::now().naive_utc();

    let endpoints = match &response.endpoints {
        Some(eps) if !eps.is_empty() => eps,
        _ => return findings, // No endpoints to analyze
    };

    for endpoint in endpoints {
        // Check grade
        if let Some(grade) = &endpoint.grade {
            let grade_upper = grade.to_uppercase();
            match grade_upper.as_str() {
                "F" | "T" => {
                    findings.push(Finding {
                        id: Uuid::new_v4(),
                        scan_id: Uuid::nil(),
                        scanner_name: "tls".to_string(),
                        title: format!("Poor TLS Configuration (Grade: {})", grade_upper),
                        description: format!(
                            "SSL Labs assigned a grade of {} to {}, indicating serious TLS/SSL configuration problems. This typically means the server uses weak cipher suites, outdated protocols, or has certificate issues.",
                            grade_upper, hostname
                        ),
                        severity: Severity::Critical,
                        remediation: "Review SSL Labs detailed report and update your TLS configuration. Disable weak ciphers and outdated protocols. Ensure your certificate is valid and properly configured.".to_string(),
                        raw_evidence: Some(format!("SSL Labs Grade: {}", grade_upper)),
                        vibe_code: false,
                        created_at: now,
                    });
                }
                "C" | "D" => {
                    findings.push(Finding {
                        id: Uuid::new_v4(),
                        scan_id: Uuid::nil(),
                        scanner_name: "tls".to_string(),
                        title: format!("Weak TLS Configuration (Grade: {})", grade_upper),
                        description: format!(
                            "SSL Labs assigned a grade of {} to {}, indicating significant TLS/SSL configuration weaknesses. Your server may support outdated protocols or weak cipher suites.",
                            grade_upper, hostname
                        ),
                        severity: Severity::High,
                        remediation: "Update your TLS configuration to use only TLS 1.2 and 1.3 with strong cipher suites. For Nginx: ssl_protocols TLSv1.2 TLSv1.3; ssl_ciphers 'TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-RSA-AES256-GCM-SHA384';".to_string(),
                        raw_evidence: Some(format!("SSL Labs Grade: {}", grade_upper)),
                        vibe_code: false,
                        created_at: now,
                    });
                }
                "B" => {
                    findings.push(Finding {
                        id: Uuid::new_v4(),
                        scan_id: Uuid::nil(),
                        scanner_name: "tls".to_string(),
                        title: format!("TLS Configuration Needs Improvement (Grade: {})", grade_upper),
                        description: format!(
                            "SSL Labs assigned a grade of {} to {}. While functional, there are improvements that can be made to your TLS configuration.",
                            grade_upper, hostname
                        ),
                        severity: Severity::Medium,
                        remediation: "Review SSL Labs recommendations and update your TLS configuration to achieve an A grade. This typically involves enabling TLS 1.3, using strong cipher suites, and enabling features like HSTS.".to_string(),
                        raw_evidence: Some(format!("SSL Labs Grade: {}", grade_upper)),
                        vibe_code: false,
                        created_at: now,
                    });
                }
                _ => {
                    // A-, A, A+ - no finding needed for grades
                }
            }
        }

        // Check endpoint details
        if let Some(details) = &endpoint.details {
            // Check for deprecated protocols
            for protocol in &details.protocols {
                let proto_name = format!("{} {}", protocol.name, protocol.version);

                if protocol.name.eq_ignore_ascii_case("SSL") {
                    // SSLv2 or SSLv3
                    findings.push(Finding {
                        id: Uuid::new_v4(),
                        scan_id: Uuid::nil(),
                        scanner_name: "tls".to_string(),
                        title: format!("Deprecated Protocol Enabled: {}", proto_name),
                        description: format!(
                            "The server supports {}, which is cryptographically broken and should never be used. This exposes communications to downgrade attacks.",
                            proto_name
                        ),
                        severity: Severity::Critical,
                        remediation: "Disable all SSL protocols in your server configuration. For Nginx: ssl_protocols TLSv1.2 TLSv1.3; For Apache: SSLProtocol -all +TLSv1.2 +TLSv1.3".to_string(),
                        raw_evidence: Some(format!("Supported protocol: {}", proto_name)),
                        vibe_code: false,
                        created_at: now,
                    });
                } else if protocol.name.eq_ignore_ascii_case("TLS")
                    && (protocol.version == "1.0" || protocol.version == "1.1") {
                    findings.push(Finding {
                        id: Uuid::new_v4(),
                        scan_id: Uuid::nil(),
                        scanner_name: "tls".to_string(),
                        title: format!("Outdated Protocol Enabled: TLS {}", protocol.version),
                        description: format!(
                            "The server supports TLS {}, which is deprecated and should be disabled. Modern browsers and security standards require TLS 1.2 or higher.",
                            protocol.version
                        ),
                        severity: Severity::High,
                        remediation: "Disable TLS 1.0 and 1.1 in your server configuration. For Nginx: ssl_protocols TLSv1.2 TLSv1.3; For Apache: SSLProtocol -all +TLSv1.2 +TLSv1.3".to_string(),
                        raw_evidence: Some(format!("Supported protocol: TLS {}", protocol.version)),
                        vibe_code: false,
                        created_at: now,
                    });
                }
            }

            // Check certificate expiry
            if let Some(cert) = &details.cert {
                if let Some(not_after_ms) = cert.not_after {
                    let not_after_secs = not_after_ms / 1000;
                    let now_secs = Utc::now().timestamp();
                    let days_until_expiry = (not_after_secs - now_secs) / 86400;

                    if days_until_expiry < 0 {
                        findings.push(Finding {
                            id: Uuid::new_v4(),
                            scan_id: Uuid::nil(),
                            scanner_name: "tls".to_string(),
                            title: "Expired TLS Certificate".to_string(),
                            description: format!(
                                "The TLS certificate for {} has expired. Browsers will show security warnings and users will not be able to access your site securely.",
                                hostname
                            ),
                            severity: Severity::Critical,
                            remediation: "Renew your TLS certificate immediately. If using Let's Encrypt, run: certbot renew. If using a commercial CA, purchase and install a new certificate.".to_string(),
                            raw_evidence: Some(format!("Certificate expired {} days ago", -days_until_expiry)),
                            vibe_code: false,
                            created_at: now,
                        });
                    } else if days_until_expiry < 30 {
                        findings.push(Finding {
                            id: Uuid::new_v4(),
                            scan_id: Uuid::nil(),
                            scanner_name: "tls".to_string(),
                            title: "TLS Certificate Expiring Soon".to_string(),
                            description: format!(
                                "The TLS certificate for {} will expire in {} days. Renew it soon to avoid service disruption.",
                                hostname, days_until_expiry
                            ),
                            severity: Severity::High,
                            remediation: "Renew your TLS certificate before it expires. If using Let's Encrypt, run: certbot renew. Set up automatic renewal to prevent future expirations.".to_string(),
                            raw_evidence: Some(format!("Certificate expires in {} days", days_until_expiry)),
                            vibe_code: false,
                            created_at: now,
                        });
                    }
                }
            }

            // Check for Heartbleed vulnerability
            if details.heartbleed {
                findings.push(Finding {
                    id: Uuid::new_v4(),
                    scan_id: Uuid::nil(),
                    scanner_name: "tls".to_string(),
                    title: "Heartbleed Vulnerability Detected".to_string(),
                    description: format!(
                        "The server at {} is vulnerable to the Heartbleed bug (CVE-2014-0160), which allows attackers to read sensitive data from server memory including private keys, passwords, and user data.",
                        hostname
                    ),
                    severity: Severity::Critical,
                    remediation: "Update OpenSSL to version 1.0.1g or later immediately. After updating, regenerate all private keys and certificates, and force users to reset passwords.".to_string(),
                    raw_evidence: Some("Heartbleed vulnerability: true".to_string()),
                    vibe_code: false,
                    created_at: now,
                });
            }

            // Check for POODLE vulnerability
            if details.poodle {
                findings.push(Finding {
                    id: Uuid::new_v4(),
                    scan_id: Uuid::nil(),
                    scanner_name: "tls".to_string(),
                    title: "POODLE Vulnerability Detected".to_string(),
                    description: format!(
                        "The server at {} is vulnerable to POODLE (Padding Oracle On Downgraded Legacy Encryption), which allows attackers to decrypt secure connections.",
                        hostname
                    ),
                    severity: Severity::High,
                    remediation: "Disable SSL 3.0 and TLS 1.0 in your server configuration. For Nginx: ssl_protocols TLSv1.2 TLSv1.3; For Apache: SSLProtocol -all +TLSv1.2 +TLSv1.3".to_string(),
                    raw_evidence: Some("POODLE vulnerability: true".to_string()),
                    vibe_code: false,
                    created_at: now,
                });
            }
        }
    }

    findings
}

fn create_informational_finding(message: &str) -> Finding {
    Finding {
        id: Uuid::new_v4(),
        scan_id: Uuid::nil(),
        scanner_name: "tls".to_string(),
        title: "TLS Analysis Information".to_string(),
        description: message.to_string(),
        severity: Severity::Low,
        remediation: "No action required. This is an informational message about the TLS scan.".to_string(),
        raw_evidence: None,
        vibe_code: false,
        created_at: Utc::now().naive_utc(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_hostname() {
        let url = "https://example.com/path?query=1";
        let parsed = Url::parse(url).unwrap();
        assert_eq!(parsed.host_str().unwrap(), "example.com");
    }

    #[test]
    fn test_rate_limit_info() {
        // This would require mocking reqwest::Response, so it's a minimal test
        let info = RateLimitInfo {
            current: Some(2),
            max: Some(5),
        };
        assert_eq!(info.current, Some(2));
        assert_eq!(info.max, Some(5));
    }
}
