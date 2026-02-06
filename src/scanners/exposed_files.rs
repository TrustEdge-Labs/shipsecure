use crate::models::finding::{Finding, Severity};
use chrono::Utc;
use reqwest::Client;
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

#[derive(Clone)]
struct ProbeTarget {
    path: &'static str,
    title: &'static str,
    severity: Severity,
    description: &'static str,
    remediation: &'static str,
    validator: fn(&ProbeResult) -> bool,
}

struct ProbeResult {
    status_code: u16,
    body: Option<String>,
    content_type: Option<String>,
}

/// Scan a URL for exposed files and directories
pub async fn scan_exposed_files(url: &str) -> Result<Vec<Finding>, ScannerError> {
    // Parse and normalize base URL
    let parsed_url = Url::parse(url).map_err(|e| {
        ScannerError::Other(format!("Invalid URL: {}", e))
    })?;

    let base_url = format!(
        "{}://{}{}",
        parsed_url.scheme(),
        parsed_url.host_str().ok_or_else(|| {
            ScannerError::Other("No hostname found in URL".to_string())
        })?,
        if let Some(port) = parsed_url.port() {
            format!(":{}", port)
        } else {
            String::new()
        }
    );

    // Create HTTP client
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(5))
        .user_agent("TrustEdge-Scanner/0.1")
        .build()
        .map_err(|e| ScannerError::Other(format!("Failed to build client: {}", e)))?;

    // Define probe targets
    let targets = vec![
        ProbeTarget {
            path: "/.env",
            title: "Exposed Environment File",
            severity: Severity::Critical,
            description: ".env files often contain database credentials, API keys, and secrets. This file is publicly accessible.",
            remediation: "Block access to .env files in your web server configuration. For Nginx: location ~ /\\.env { deny all; return 404; } For Apache: <FilesMatch \"^\\.env\"> Require all denied </FilesMatch>",
            validator: is_env_file,
        },
        ProbeTarget {
            path: "/.env.local",
            title: "Exposed Environment File",
            severity: Severity::Critical,
            description: ".env.local files often contain database credentials, API keys, and secrets. This file is publicly accessible.",
            remediation: "Block access to .env files in your web server configuration. For Nginx: location ~ /\\.env { deny all; return 404; } For Apache: <FilesMatch \"^\\.env\"> Require all denied </FilesMatch>",
            validator: is_env_file,
        },
        ProbeTarget {
            path: "/.env.production",
            title: "Exposed Environment File",
            severity: Severity::Critical,
            description: ".env.production files often contain database credentials, API keys, and secrets. This file is publicly accessible.",
            remediation: "Block access to .env files in your web server configuration. For Nginx: location ~ /\\.env { deny all; return 404; } For Apache: <FilesMatch \"^\\.env\"> Require all denied </FilesMatch>",
            validator: is_env_file,
        },
        ProbeTarget {
            path: "/.git/config",
            title: "Exposed Git Repository",
            severity: Severity::Critical,
            description: "The .git directory is accessible, potentially exposing source code, commit history, and credentials.",
            remediation: "Block access to .git directory in your web server configuration. For Nginx: location ~ /\\.git { deny all; return 404; } For Apache: <DirectoryMatch \"^\\.git\"> Require all denied </DirectoryMatch>",
            validator: is_git_config,
        },
        ProbeTarget {
            path: "/.git/HEAD",
            title: "Exposed Git Repository",
            severity: Severity::Critical,
            description: "The .git directory is accessible, potentially exposing source code, commit history, and credentials.",
            remediation: "Block access to .git directory in your web server configuration. For Nginx: location ~ /\\.git { deny all; return 404; } For Apache: <DirectoryMatch \"^\\.git\"> Require all denied </DirectoryMatch>",
            validator: is_git_file,
        },
        ProbeTarget {
            path: "/wp-admin/",
            title: "Exposed Admin Panel",
            severity: Severity::High,
            description: "A WordPress admin panel is publicly accessible without additional protection.",
            remediation: "Add IP allowlisting or HTTP authentication to your admin panel. For Nginx: location /wp-admin/ { allow YOUR_IP; deny all; auth_basic \"Admin\"; auth_basic_user_file /etc/nginx/.htpasswd; }",
            validator: is_accessible,
        },
        ProbeTarget {
            path: "/admin/",
            title: "Exposed Admin Panel",
            severity: Severity::High,
            description: "An admin panel is publicly accessible without additional protection.",
            remediation: "Add IP allowlisting or HTTP authentication to your admin panel. For Nginx: location /admin/ { allow YOUR_IP; deny all; auth_basic \"Admin\"; auth_basic_user_file /etc/nginx/.htpasswd; }",
            validator: is_accessible,
        },
        ProbeTarget {
            path: "/debug/",
            title: "Exposed Debug Endpoint",
            severity: Severity::High,
            description: "Debug endpoints can expose internal application state, stack traces, and configuration.",
            remediation: "Remove or disable debug endpoints in production. If needed for diagnostics, protect with authentication and IP allowlisting.",
            validator: is_accessible,
        },
        ProbeTarget {
            path: "/_debug/",
            title: "Exposed Debug Endpoint",
            severity: Severity::High,
            description: "Debug endpoints can expose internal application state, stack traces, and configuration.",
            remediation: "Remove or disable debug endpoints in production. If needed for diagnostics, protect with authentication and IP allowlisting.",
            validator: is_accessible,
        },
        ProbeTarget {
            path: "/api/debug",
            title: "Exposed Debug Endpoint",
            severity: Severity::High,
            description: "Debug endpoints can expose internal application state, stack traces, and configuration.",
            remediation: "Remove or disable debug endpoints in production. If needed for diagnostics, protect with authentication and IP allowlisting.",
            validator: is_accessible,
        },
        ProbeTarget {
            path: "/server-status",
            title: "Exposed Server Status",
            severity: Severity::High,
            description: "Apache server-status page is publicly accessible, revealing active connections and server configuration.",
            remediation: "Restrict access to server-status. For Apache: <Location /server-status> SetHandler server-status Require ip YOUR_IP </Location>",
            validator: is_accessible,
        },
        ProbeTarget {
            path: "/phpinfo.php",
            title: "Exposed PHP Info",
            severity: Severity::High,
            description: "phpinfo() output is accessible, revealing server configuration, PHP version, and loaded modules.",
            remediation: "Delete phpinfo.php from your web root immediately. Never leave diagnostic files in production.",
            validator: is_php_info,
        },
        ProbeTarget {
            path: "/main.js.map",
            title: "Exposed Source Maps",
            severity: Severity::Medium,
            description: "JavaScript source maps are publicly accessible, revealing your application's source code structure and potentially sensitive logic.",
            remediation: "Disable source map generation in production builds or configure your web server to block .map files. For webpack: devtool: false in production config. For Nginx: location ~ \\.map$ { deny all; return 404; }",
            validator: is_source_map,
        },
        ProbeTarget {
            path: "/app.js.map",
            title: "Exposed Source Maps",
            severity: Severity::Medium,
            description: "JavaScript source maps are publicly accessible, revealing your application's source code structure and potentially sensitive logic.",
            remediation: "Disable source map generation in production builds or configure your web server to block .map files. For webpack: devtool: false in production config. For Nginx: location ~ \\.map$ { deny all; return 404; }",
            validator: is_source_map,
        },
        ProbeTarget {
            path: "/bundle.js.map",
            title: "Exposed Source Maps",
            severity: Severity::Medium,
            description: "JavaScript source maps are publicly accessible, revealing your application's source code structure and potentially sensitive logic.",
            remediation: "Disable source map generation in production builds or configure your web server to block .map files. For webpack: devtool: false in production config. For Nginx: location ~ \\.map$ { deny all; return 404; }",
            validator: is_source_map,
        },
        ProbeTarget {
            path: "/robots.txt",
            title: "Informational: robots.txt Found",
            severity: Severity::Low,
            description: "robots.txt is accessible. Review it to ensure it doesn't reveal sensitive paths via Disallow directives.",
            remediation: "Review your robots.txt file and ensure Disallow directives don't point to sensitive endpoints. Consider using meta tags instead for sensitive pages.",
            validator: is_interesting_robots,
        },
        ProbeTarget {
            path: "/sitemap.xml",
            title: "Informational: sitemap.xml Found",
            severity: Severity::Low,
            description: "sitemap.xml is accessible. Verify it doesn't expose internal or staging URLs.",
            remediation: "Review your sitemap.xml and ensure it only contains public, production URLs. Exclude admin, staging, and internal paths.",
            validator: is_accessible,
        },
    ];

    // Probe all targets concurrently
    let mut probe_tasks = Vec::new();
    for target in targets.iter() {
        let client = client.clone();
        let url = format!("{}{}", base_url, target.path);
        let target = target.clone();

        let task = tokio::spawn(async move {
            probe_path(&client, &url, target).await
        });

        probe_tasks.push(task);
    }

    // Check for missing security.txt
    let client_clone = client.clone();
    let security_txt_url = format!("{}/.well-known/security.txt", base_url);
    let security_txt_task = tokio::spawn(async move {
        check_security_txt(&client_clone, &security_txt_url).await
    });
    probe_tasks.push(security_txt_task);

    // Collect all results
    let mut findings = Vec::new();
    let mut successful_probes = 0;

    for task in probe_tasks {
        match task.await {
            Ok(Some(finding)) => {
                findings.push(finding);
                successful_probes += 1;
            }
            Ok(None) => {
                successful_probes += 1;
            }
            Err(_) => {
                // Task panicked or was cancelled - skip it
            }
        }
    }

    // If no probes succeeded, host might be unreachable
    if successful_probes == 0 {
        return Err(ScannerError::Other(
            "Failed to connect to host - all probes failed".to_string()
        ));
    }

    // Deduplicate findings with same title
    deduplicate_findings(&mut findings);

    Ok(findings)
}

async fn probe_path(
    client: &Client,
    url: &str,
    target: ProbeTarget,
) -> Option<Finding> {
    // Try HEAD request first
    let result = match client.head(url).send().await {
        Ok(resp) if resp.status().as_u16() == 405 => {
            // HEAD not allowed, try GET
            match client.get(url).send().await {
                Ok(resp) => Some(resp),
                Err(_) => return None, // Connection failed, skip
            }
        }
        Ok(resp) => Some(resp),
        Err(_) => return None, // Connection failed, skip
    };

    let response = result?;
    let status_code = response.status().as_u16();

    // Get content type
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_lowercase());

    // For 200 responses, fetch body for validation
    let body = if status_code == 200 {
        match response.text().await {
            Ok(text) if text.len() <= 10_000 => Some(text), // Limit body size
            Ok(_) => Some(String::new()), // Body too large, skip
            Err(_) => None,
        }
    } else {
        None
    };

    let probe_result = ProbeResult {
        status_code,
        body,
        content_type,
    };

    // Run validator
    if (target.validator)(&probe_result) {
        let now = Utc::now().naive_utc();
        Some(Finding {
            id: Uuid::new_v4(),
            scan_id: Uuid::nil(),
            scanner_name: "exposed_files".to_string(),
            title: target.title.to_string(),
            description: target.description.to_string(),
            severity: target.severity,
            remediation: target.remediation.to_string(),
            raw_evidence: Some(format!(
                "URL: {}\nStatus: {}\nContent-Type: {}",
                url,
                status_code,
                probe_result.content_type.as_deref().unwrap_or("unknown")
            )),
            vibe_code: false,
            created_at: now,
        })
    } else {
        None
    }
}

async fn check_security_txt(client: &Client, url: &str) -> Option<Finding> {
    match client.head(url).send().await {
        Ok(resp) if resp.status().as_u16() == 200 => None, // security.txt exists, good
        Ok(resp) if resp.status().as_u16() == 404 => {
            // Missing security.txt
            let now = Utc::now().naive_utc();
            Some(Finding {
                id: Uuid::new_v4(),
                scan_id: Uuid::nil(),
                scanner_name: "exposed_files".to_string(),
                title: "No security.txt Found".to_string(),
                description: "No security.txt file found at /.well-known/security.txt. This file provides security researchers with a way to report vulnerabilities responsibly.".to_string(),
                severity: Severity::Low,
                remediation: "Create a security.txt file at /.well-known/security.txt following RFC 9116. Include Contact, Expires, and optionally Acknowledgments and Preferred-Languages fields. Example: Contact: mailto:security@example.com\\nExpires: 2025-12-31T23:59:59z".to_string(),
                raw_evidence: Some(format!("URL: {}\nStatus: 404 Not Found", url)),
                vibe_code: false,
                created_at: now,
            })
        }
        _ => None, // Other status or error, skip
    }
}

// Validator functions

fn is_accessible(result: &ProbeResult) -> bool {
    result.status_code == 200
}

fn is_env_file(result: &ProbeResult) -> bool {
    if result.status_code != 200 {
        return false;
    }

    // Check if body contains common env file patterns
    if let Some(body) = &result.body {
        // Check for common env var patterns
        let has_env_pattern = body.contains("DB_")
            || body.contains("API_KEY")
            || body.contains("SECRET")
            || body.contains("PASSWORD")
            || body.contains("DATABASE_URL")
            || body.contains("MONGO")
            || body.contains("REDIS")
            || body.contains("AWS_")
            || body.contains("_KEY=")
            || body.contains("_SECRET=");

        // Check it's not HTML (framework catch-all)
        let is_html = body.trim_start().starts_with("<!DOCTYPE")
            || body.trim_start().starts_with("<html")
            || body.trim_start().starts_with("<HTML");

        return has_env_pattern && !is_html;
    }

    false
}

fn is_git_config(result: &ProbeResult) -> bool {
    if result.status_code != 200 {
        return false;
    }

    if let Some(body) = &result.body {
        return body.contains("[core]") || body.contains("[remote");
    }

    false
}

fn is_git_file(result: &ProbeResult) -> bool {
    if result.status_code != 200 {
        return false;
    }

    if let Some(body) = &result.body {
        // .git/HEAD typically contains "ref: refs/heads/main" or a commit hash
        return body.starts_with("ref: refs/") || body.len() == 40 || body.len() == 41;
    }

    false
}

fn is_php_info(result: &ProbeResult) -> bool {
    if result.status_code != 200 {
        return false;
    }

    if let Some(body) = &result.body {
        return body.contains("phpinfo()") || body.contains("PHP Version");
    }

    false
}

fn is_source_map(result: &ProbeResult) -> bool {
    if result.status_code != 200 {
        return false;
    }

    // Check Content-Type
    if let Some(ct) = &result.content_type {
        if ct.contains("application/json") {
            if let Some(body) = &result.body {
                return body.contains("\"version\":3") || body.contains("\"mappings\"");
            }
        }
    }

    // Also check body starts with source map signature
    if let Some(body) = &result.body {
        return body.trim_start().starts_with("{\"version\":3");
    }

    false
}

fn is_interesting_robots(result: &ProbeResult) -> bool {
    if result.status_code != 200 {
        return false;
    }

    if let Some(body) = &result.body {
        // Only flag if it has Disallow directives (not just empty or "allow all")
        let has_disallow = body.contains("Disallow:");

        // Check it's not just "Disallow: " (empty - allows all)
        let has_specific_disallow = body.lines().any(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("Disallow:")
                && trimmed.len() > "Disallow:".len()
                && !trimmed.ends_with("Disallow:")
        });

        return has_disallow && has_specific_disallow;
    }

    false
}

fn deduplicate_findings(findings: &mut Vec<Finding>) {
    // Use a simple deduplication by title
    let mut seen = std::collections::HashSet::new();
    findings.retain(|f| seen.insert(f.title.clone()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_env_file_validator() {
        let result = ProbeResult {
            status_code: 200,
            body: Some("DB_PASSWORD=secret\nAPI_KEY=test123".to_string()),
            content_type: Some("text/plain".to_string()),
        };
        assert!(is_env_file(&result));

        let html_result = ProbeResult {
            status_code: 200,
            body: Some("<!DOCTYPE html><html>DB_PASSWORD</html>".to_string()),
            content_type: Some("text/html".to_string()),
        };
        assert!(!is_env_file(&html_result));
    }

    #[test]
    fn test_is_git_config_validator() {
        let result = ProbeResult {
            status_code: 200,
            body: Some("[core]\n\trepositoryformatversion = 0".to_string()),
            content_type: Some("text/plain".to_string()),
        };
        assert!(is_git_config(&result));
    }

    #[test]
    fn test_is_source_map_validator() {
        let result = ProbeResult {
            status_code: 200,
            body: Some("{\"version\":3,\"file\":\"app.js\",\"mappings\":\"AAAA\"}".to_string()),
            content_type: Some("application/json".to_string()),
        };
        assert!(is_source_map(&result));
    }

    #[test]
    fn test_is_interesting_robots_validator() {
        let result = ProbeResult {
            status_code: 200,
            body: Some("User-agent: *\nDisallow: /admin/\nDisallow: /api/internal/".to_string()),
            content_type: Some("text/plain".to_string()),
        };
        assert!(is_interesting_robots(&result));

        let empty_result = ProbeResult {
            status_code: 200,
            body: Some("User-agent: *\nDisallow:".to_string()),
            content_type: Some("text/plain".to_string()),
        };
        assert!(!is_interesting_robots(&empty_result));
    }
}
