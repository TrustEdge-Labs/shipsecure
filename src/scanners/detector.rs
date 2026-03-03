use crate::models::detection::{DetectionResult, Framework, Platform};
use reqwest::header::HeaderMap;
use scraper::{Html, Selector};
use std::net::SocketAddr;
use std::time::Duration;

const HIGH_CONFIDENCE_THRESHOLD: u8 = 60;

#[derive(Debug)]
pub enum ScannerError {
    HttpError(reqwest::Error),
    Timeout,
    Other(String),
}

impl std::fmt::Display for ScannerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

/// Detect framework and platform stack from target URL
pub async fn detect_stack(
    target_url: &str,
    hostname: &str,
    resolved_addrs: &[SocketAddr],
) -> Result<DetectionResult, ScannerError> {
    let client = crate::ssrf::safe_client_builder(hostname, resolved_addrs)
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent("ShipSecure-Scanner/1.0")
        .build()
        .map_err(|e| ScannerError::Other(format!("Failed to build client: {}", e)))?;

    let response = client.get(target_url).send().await?;
    let headers = response.headers().clone();
    let html = response.text().await?;

    // Detect framework and platform
    let (framework, framework_confidence, mut framework_signals) =
        detect_framework(&headers, &html);
    let (platform, platform_confidence, platform_signals) = detect_platform(&headers);

    // Combine all signals
    let mut signals = Vec::new();
    signals.append(&mut framework_signals);
    signals.extend(platform_signals);

    Ok(DetectionResult {
        framework,
        platform,
        framework_confidence,
        platform_confidence,
        signals,
    })
}

/// Detect framework from headers and HTML using weighted scoring
fn detect_framework(headers: &HeaderMap, html: &str) -> (Option<Framework>, u8, Vec<String>) {
    let document = Html::parse_document(html);

    // Score each framework
    let nextjs_score = score_nextjs(&document, headers, html);
    let vite_react_score = score_vite_react(&document, html, nextjs_score.0);
    let sveltekit_score = score_sveltekit(&document, headers, html);
    let nuxt_score = score_nuxt(&document, headers, html);

    // Find highest scoring framework
    let scores = [
        (Framework::NextJs, nextjs_score.0, nextjs_score.1),
        (Framework::ViteReact, vite_react_score.0, vite_react_score.1),
        (Framework::SvelteKit, sveltekit_score.0, sveltekit_score.1),
        (Framework::Nuxt, nuxt_score.0, nuxt_score.1),
    ];

    let (framework, score, signals) = scores.iter().max_by_key(|(_, score, _)| *score).unwrap();

    if *score >= HIGH_CONFIDENCE_THRESHOLD {
        (Some(framework.clone()), *score, signals.clone())
    } else {
        (None, *score, signals.clone())
    }
}

/// Score Next.js framework detection
fn score_nextjs(document: &Html, headers: &HeaderMap, _html: &str) -> (u8, Vec<String>) {
    let mut score = 0u8;
    let mut signals = Vec::new();

    // STRONG signal: __NEXT_DATA__ script with valid JSON containing buildId (40 points)
    if let Ok(selector) = Selector::parse("script#__NEXT_DATA__")
        && let Some(element) = document.select(&selector).next()
    {
        let content = element.text().collect::<String>();
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content)
            && json.get("buildId").is_some()
        {
            score += 40;
            signals.push("Next.js: __NEXT_DATA__ script with buildId".to_string());
        }
    }

    // MEDIUM signal: /_next/static assets (30 points)
    if let Ok(selector) = Selector::parse("script[src]") {
        for element in document.select(&selector) {
            if let Some(src) = element.value().attr("src")
                && src.contains("/_next/static")
            {
                score += 30;
                signals.push("Next.js: /_next/static assets detected".to_string());
                break;
            }
        }
    }

    // LOW signal: x-powered-by header with Next.js (20 points, often stripped)
    if let Some(powered_by) = headers.get("x-powered-by")
        && let Ok(value) = powered_by.to_str()
        && value.contains("Next.js")
    {
        score += 20;
        signals.push("Next.js: x-powered-by header".to_string());
    }

    // LOW signal: meta generator tag (10 points)
    if let Ok(selector) = Selector::parse("meta[name='generator']")
        && let Some(element) = document.select(&selector).next()
        && let Some(content) = element.value().attr("content")
        && content.contains("Next.js")
    {
        score += 10;
        signals.push("Next.js: meta generator tag".to_string());
    }

    (score, signals)
}

/// Score Vite/React framework detection
fn score_vite_react(document: &Html, html: &str, nextjs_score: u8) -> (u8, Vec<String>) {
    // If Next.js scored above threshold, don't detect Vite/React
    // (Next.js uses React internally)
    if nextjs_score >= HIGH_CONFIDENCE_THRESHOLD {
        return (0, Vec::new());
    }

    let mut score = 0u8;
    let mut signals = Vec::new();

    // MEDIUM signal: Vite-specific module patterns (30 points)
    if let Ok(selector) = Selector::parse("script[type='module'][src]") {
        for element in document.select(&selector) {
            if let Some(src) = element.value().attr("src")
                && (src.contains("/.vite/") || (src.contains("/assets/") && src.contains("-")))
            {
                score += 30;
                signals.push("Vite/React: Vite module pattern detected".to_string());
                break;
            }
        }
    }

    // MEDIUM signal: React mount points (20 points)
    if let Ok(selector) = Selector::parse("div#root, div#app")
        && document.select(&selector).next().is_some()
    {
        score += 20;
        signals.push("Vite/React: React mount point (div#root or div#app)".to_string());
    }

    // MEDIUM signal: import.meta in scripts (20 points)
    if html.contains("import.meta") {
        score += 20;
        signals.push("Vite/React: import.meta detected".to_string());
    }

    // LOW signal: React indicators without Next.js (10 points)
    if html.contains("data-reactroot") || html.contains("__REACT") {
        score += 10;
        signals.push("Vite/React: React indicators detected".to_string());
    }

    (score, signals)
}

/// Score SvelteKit framework detection
fn score_sveltekit(document: &Html, _headers: &HeaderMap, html: &str) -> (u8, Vec<String>) {
    let mut score = 0u8;
    let mut signals = Vec::new();

    // STRONG signal: __sveltekit or _app patterns (40 points)
    if let Ok(selector) = Selector::parse("script[src]") {
        for element in document.select(&selector) {
            if let Some(src) = element.value().attr("src")
                && (src.contains("/__sveltekit/") || src.contains("/_app/"))
            {
                score += 40;
                signals.push("SvelteKit: __sveltekit or _app assets".to_string());
                break;
            }
        }
    }

    // MEDIUM signal: data-sveltekit attributes (30 points)
    if html.contains("data-sveltekit") {
        score += 30;
        signals.push("SvelteKit: data-sveltekit attributes".to_string());
    }

    // LOW signal: meta generator tag (10 points)
    if let Ok(selector) = Selector::parse("meta[name='generator']")
        && let Some(element) = document.select(&selector).next()
        && let Some(content) = element.value().attr("content")
        && content.contains("SvelteKit")
    {
        score += 10;
        signals.push("SvelteKit: meta generator tag".to_string());
    }

    (score, signals)
}

/// Score Nuxt framework detection
fn score_nuxt(document: &Html, headers: &HeaderMap, _html: &str) -> (u8, Vec<String>) {
    let mut score = 0u8;
    let mut signals = Vec::new();

    // STRONG signal: __NUXT__ or __NUXT_DATA__ (40 points)
    if let Ok(selector) = Selector::parse("script#__NUXT__, script#__NUXT_DATA__")
        && document.select(&selector).next().is_some()
    {
        score += 40;
        signals.push("Nuxt: __NUXT__ or __NUXT_DATA__ script".to_string());
    }

    // MEDIUM signal: /_nuxt/ assets (30 points)
    if let Ok(selector) = Selector::parse("script[src]") {
        for element in document.select(&selector) {
            if let Some(src) = element.value().attr("src")
                && src.contains("/_nuxt/")
            {
                score += 30;
                signals.push("Nuxt: /_nuxt/ assets detected".to_string());
                break;
            }
        }
    }

    // LOW signal: x-powered-by header with Nuxt (20 points)
    if let Some(powered_by) = headers.get("x-powered-by")
        && let Ok(value) = powered_by.to_str()
        && value.contains("Nuxt")
    {
        score += 20;
        signals.push("Nuxt: x-powered-by header".to_string());
    }

    (score, signals)
}

/// Detect platform from response headers
fn detect_platform(headers: &HeaderMap) -> (Option<Platform>, u8, Vec<String>) {
    let mut signals = Vec::new();

    // Definitive platform headers (100 confidence)
    if headers.get("x-vercel-id").is_some() {
        signals.push("Platform: x-vercel-id header".to_string());
        return (Some(Platform::Vercel), 100, signals);
    }

    if headers.get("x-nf-request-id").is_some() {
        signals.push("Platform: x-nf-request-id header".to_string());
        return (Some(Platform::Netlify), 100, signals);
    }

    if headers.get("x-railway-request-id").is_some() {
        signals.push("Platform: x-railway-request-id header".to_string());
        return (Some(Platform::Railway), 100, signals);
    }

    // Fallback: server header contains platform name (80 confidence)
    if let Some(server) = headers.get("server")
        && let Ok(value) = server.to_str()
    {
        let lower = value.to_lowercase();
        if lower.contains("vercel") {
            signals.push("Platform: server header contains Vercel".to_string());
            return (Some(Platform::Vercel), 80, signals);
        }
        if lower.contains("netlify") {
            signals.push("Platform: server header contains Netlify".to_string());
            return (Some(Platform::Netlify), 80, signals);
        }
    }

    (None, 0, signals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nextjs_detection_with_next_data_and_assets() {
        let html = r#"
            <html>
                <head>
                    <script id="__NEXT_DATA__" type="application/json">{"buildId":"test123"}</script>
                    <script src="/_next/static/chunks/main.js"></script>
                </head>
                <body>
                    <div id="__next"></div>
                </body>
            </html>
        "#;
        let headers = HeaderMap::new();
        let (framework, confidence, signals) = detect_framework(&headers, html);

        assert_eq!(framework, Some(Framework::NextJs));
        assert!(
            confidence >= HIGH_CONFIDENCE_THRESHOLD,
            "confidence should be >= 60, got {}",
            confidence
        );
        assert!(signals.iter().any(|s| s.contains("__NEXT_DATA__")));
    }

    #[test]
    fn test_nextjs_single_signal_below_threshold() {
        let html = r#"
            <html>
                <head>
                    <script id="__NEXT_DATA__" type="application/json">{"buildId":"test123"}</script>
                </head>
                <body>
                    <div id="__next"></div>
                </body>
            </html>
        "#;
        let headers = HeaderMap::new();
        let (framework, confidence, _) = detect_framework(&headers, html);

        assert_eq!(confidence, 40);
        assert_eq!(framework, None, "should not detect with only 40 points");
    }

    #[test]
    fn test_vercel_platform_detection() {
        let mut headers = HeaderMap::new();
        headers.insert("x-vercel-id", "test-id".parse().unwrap());

        let (platform, confidence, signals) = detect_platform(&headers);

        assert_eq!(platform, Some(Platform::Vercel));
        assert_eq!(confidence, 100);
        assert!(signals.iter().any(|s| s.contains("x-vercel-id")));
    }

    #[test]
    fn test_no_detection_empty_page() {
        let html = "<html><body>Hello World</body></html>";
        let headers = HeaderMap::new();

        let (framework, _, _) = detect_framework(&headers, html);
        let (platform, _, _) = detect_platform(&headers);

        assert_eq!(framework, None);
        assert_eq!(platform, None);
    }

    #[test]
    fn test_vite_react_not_detected_when_nextjs_present() {
        // HTML with both React indicators and Next.js indicators
        let html = r#"
            <html>
                <head>
                    <script id="__NEXT_DATA__" type="application/json">{"buildId":"test123"}</script>
                    <script src="/_next/static/chunks/main.js"></script>
                </head>
                <body>
                    <div id="root"></div>
                </body>
            </html>
        "#;
        let headers = HeaderMap::new();
        let (framework, _, _) = detect_framework(&headers, html);

        // Should detect Next.js, not Vite/React
        assert_eq!(framework, Some(Framework::NextJs));
    }
}
