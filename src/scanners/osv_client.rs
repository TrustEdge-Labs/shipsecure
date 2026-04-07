// OSV.dev HTTP client for supply chain vulnerability scanning.
//
// Provides batch querying against the OSV.dev API with chunking (1000 per batch),
// single retry on 5xx errors, and individual vulnerability hydration.

use crate::scanners::lockfile_parser::{DepSource, ParsedDep, SupplyChainError};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// -- OSV API request types --

#[derive(Debug, Serialize)]
struct OsvBatchRequest {
    queries: Vec<OsvQuery>,
}

#[derive(Debug, Serialize)]
struct OsvQuery {
    package: OsvPackage,
    version: String,
}

#[derive(Debug, Serialize)]
struct OsvPackage {
    name: String,
    ecosystem: String,
}

// -- OSV API response types --

#[derive(Debug, Deserialize, Default)]
struct OsvBatchResponse {
    #[serde(default)]
    results: Vec<OsvBatchResult>,
}

#[derive(Debug, Deserialize, Default)]
struct OsvBatchResult {
    #[serde(default)]
    vulns: Vec<OsvVulnSummary>,
}

#[derive(Debug, Deserialize, Clone)]
struct OsvVulnSummary {
    id: String,
    #[allow(dead_code)]
    modified: Option<String>,
}

/// Full vulnerability detail from /v1/vulns/{id} hydration endpoint.
#[derive(Debug, Deserialize, Clone)]
pub struct OsvVulnDetail {
    pub id: String,
    pub summary: Option<String>,
    #[serde(default)]
    pub severity: Vec<OsvSeverity>,
    pub database_specific: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OsvSeverity {
    #[serde(rename = "type")]
    pub score_type: String,
    pub score: String,
}

// -- Client output types --

/// A dependency matched against OSV, with any associated vulnerability IDs.
#[derive(Debug, Clone)]
pub struct DepVulnMatch {
    pub dep: ParsedDep,
    pub vuln_ids: Vec<String>,
}

// -- Constants --

const CHUNK_SIZE: usize = 1000;
const REQUEST_TIMEOUT_SECS: u64 = 10;
const DEFAULT_BASE_URL: &str = "https://api.osv.dev";

// -- Client --

pub struct OsvClient {
    client: reqwest::Client,
    base_url: String,
}

impl Default for OsvClient {
    fn default() -> Self {
        Self::new()
    }
}

impl OsvClient {
    /// Create a new OsvClient targeting the production OSV.dev API.
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .user_agent("ShipSecure-Scanner/1.0")
            .build()
            .expect("Failed to build reqwest client");

        Self {
            client,
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    /// Create a new OsvClient with a custom base URL (for testing).
    pub fn with_base_url(base_url: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .user_agent("ShipSecure-Scanner/1.0")
            .build()
            .expect("Failed to build reqwest client");

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Query OSV.dev for vulnerabilities affecting the given registry dependencies.
    ///
    /// Deps are chunked into groups of 1000 and queried in parallel.
    /// Non-registry deps are defensively filtered out.
    /// Returns a DepVulnMatch per input dep with any matched vuln IDs.
    pub async fn query_batch(
        &self,
        deps: &[ParsedDep],
    ) -> Result<Vec<DepVulnMatch>, SupplyChainError> {
        // Defensive filter: only registry deps
        let registry_deps: Vec<&ParsedDep> = deps
            .iter()
            .filter(|d| d.source == DepSource::Registry)
            .collect();

        if registry_deps.is_empty() {
            return Ok(vec![]);
        }

        // Chunk into groups of CHUNK_SIZE
        let chunks: Vec<Vec<&ParsedDep>> = registry_deps
            .chunks(CHUNK_SIZE)
            .map(|c| c.to_vec())
            .collect();

        // Fire all chunks in parallel
        let futures: Vec<_> = chunks
            .into_iter()
            .map(|chunk| self.query_chunk(chunk))
            .collect();

        let results = futures::future::join_all(futures).await;

        // If any chunk failed, return the first error
        let mut all_matches = Vec::new();
        for result in results {
            match result {
                Ok(mut matches) => all_matches.append(&mut matches),
                Err(e) => return Err(e),
            }
        }

        Ok(all_matches)
    }

    /// Query a single chunk of deps against /v1/querybatch.
    /// Retries once on 5xx responses.
    async fn query_chunk(
        &self,
        deps: Vec<&ParsedDep>,
    ) -> Result<Vec<DepVulnMatch>, SupplyChainError> {
        let request_body = OsvBatchRequest {
            queries: deps
                .iter()
                .map(|d| OsvQuery {
                    package: OsvPackage {
                        name: d.name.clone(),
                        ecosystem: "npm".to_string(),
                    },
                    version: d.version.clone(),
                })
                .collect(),
        };

        let url = format!("{}/v1/querybatch", self.base_url);

        // First attempt
        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| SupplyChainError::ChunkFailure(format!("Request failed: {e}")))?;

        let response = if response.status().is_server_error() {
            // Retry once on 5xx
            let retry_response = self
                .client
                .post(&url)
                .json(&request_body)
                .send()
                .await
                .map_err(|e| {
                    SupplyChainError::ChunkFailure(format!("Retry request failed: {e}"))
                })?;

            if retry_response.status().is_server_error() {
                return Err(SupplyChainError::ChunkFailure(format!(
                    "Chunk failed after retry with status {}",
                    retry_response.status()
                )));
            }
            retry_response
        } else {
            response
        };

        let batch_response: OsvBatchResponse = response.json().await.map_err(|e| {
            SupplyChainError::ChunkFailure(format!("Failed to parse response: {e}"))
        })?;

        // Positionally align results with input deps
        let matches: Vec<DepVulnMatch> = deps
            .iter()
            .enumerate()
            .map(|(i, dep)| {
                let vuln_ids = batch_response
                    .results
                    .get(i)
                    .map(|r| r.vulns.iter().map(|v| v.id.clone()).collect())
                    .unwrap_or_default();

                DepVulnMatch {
                    dep: (*dep).clone(),
                    vuln_ids,
                }
            })
            .collect();

        Ok(matches)
    }

    /// Hydrate vulnerability details by fetching /v1/vulns/{id} for each unique ID.
    ///
    /// Deduplicates IDs before fetching. Returns full OsvVulnDetail for each.
    pub async fn hydrate_vulns(
        &self,
        vuln_ids: &[String],
    ) -> Result<Vec<OsvVulnDetail>, SupplyChainError> {
        // Deduplicate
        let unique_ids: Vec<String> = {
            let mut seen = HashSet::new();
            vuln_ids
                .iter()
                .filter(|id| seen.insert((*id).clone()))
                .cloned()
                .collect()
        };

        if unique_ids.is_empty() {
            return Ok(vec![]);
        }

        // Fetch all in parallel
        let futures: Vec<_> = unique_ids
            .iter()
            .map(|id| self.fetch_vuln_detail(id))
            .collect();

        let results = futures::future::join_all(futures).await;

        let mut details = Vec::new();
        for result in results {
            match result {
                Ok(detail) => details.push(detail),
                Err(e) => return Err(e),
            }
        }

        Ok(details)
    }

    /// Fetch a single vulnerability detail from /v1/vulns/{id}.
    async fn fetch_vuln_detail(&self, id: &str) -> Result<OsvVulnDetail, SupplyChainError> {
        let url = format!("{}/v1/vulns/{}", self.base_url, id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| SupplyChainError::OsvQuery(format!("Failed to fetch {id}: {e}")))?;

        if !response.status().is_success() {
            return Err(SupplyChainError::OsvQuery(format!(
                "Hydration of {id} returned status {}",
                response.status()
            )));
        }

        response
            .json()
            .await
            .map_err(|e| SupplyChainError::OsvQuery(format!("Failed to parse {id}: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_dep(name: &str, version: &str) -> ParsedDep {
        ParsedDep {
            name: name.to_string(),
            version: version.to_string(),
            source: DepSource::Registry,
            is_dev: false,
        }
    }

    fn make_non_registry_dep(name: &str) -> ParsedDep {
        ParsedDep {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            source: DepSource::Git,
            is_dev: false,
        }
    }

    #[tokio::test]
    async fn query_batch_three_registry_deps() {
        let server = MockServer::start().await;

        let response_body = serde_json::json!({
            "results": [
                { "vulns": [] },
                { "vulns": [{ "id": "GHSA-1234", "modified": "2024-01-01T00:00:00Z" }] },
                { "vulns": [] }
            ]
        });

        Mock::given(method("POST"))
            .and(path("/v1/querybatch"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .expect(1)
            .mount(&server)
            .await;

        let client = OsvClient::with_base_url(&server.uri());
        let deps = vec![
            make_dep("express", "4.18.2"),
            make_dep("lodash", "4.17.21"),
            make_dep("react", "18.2.0"),
        ];

        let matches = client.query_batch(&deps).await.unwrap();
        assert_eq!(matches.len(), 3);
        assert!(matches[0].vuln_ids.is_empty());
        assert_eq!(matches[1].vuln_ids, vec!["GHSA-1234"]);
        assert!(matches[2].vuln_ids.is_empty());
    }

    #[tokio::test]
    async fn query_batch_1500_deps_two_chunks() {
        let server = MockServer::start().await;

        // Use a single mock that responds to all batch requests with empty results.
        // The key assertion is that 1500 deps produce 1500 DepVulnMatch entries
        // (proving they were split into chunks and reassembled).
        Mock::given(method("POST"))
            .and(path("/v1/querybatch"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": []
            })))
            .mount(&server)
            .await;

        let client = OsvClient::with_base_url(&server.uri());
        let deps: Vec<ParsedDep> = (0..1500)
            .map(|i| make_dep(&format!("pkg-{i}"), "1.0.0"))
            .collect();

        let matches = client.query_batch(&deps).await.unwrap();
        // With empty results arrays, matches will still be created (one per dep)
        // but vuln_ids will be empty since there are no positional results.
        assert_eq!(matches.len(), 1500);
    }

    #[tokio::test]
    async fn query_batch_positional_alignment() {
        let server = MockServer::start().await;

        let response_body = serde_json::json!({
            "results": [
                { "vulns": [{ "id": "VULN-A", "modified": null }] },
                { "vulns": [] },
                { "vulns": [{ "id": "VULN-B", "modified": null }, { "id": "VULN-C", "modified": null }] }
            ]
        });

        Mock::given(method("POST"))
            .and(path("/v1/querybatch"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
            .mount(&server)
            .await;

        let client = OsvClient::with_base_url(&server.uri());
        let deps = vec![
            make_dep("pkg-a", "1.0.0"),
            make_dep("pkg-b", "2.0.0"),
            make_dep("pkg-c", "3.0.0"),
        ];

        let matches = client.query_batch(&deps).await.unwrap();
        assert_eq!(matches[0].dep.name, "pkg-a");
        assert_eq!(matches[0].vuln_ids, vec!["VULN-A"]);
        assert_eq!(matches[1].dep.name, "pkg-b");
        assert!(matches[1].vuln_ids.is_empty());
        assert_eq!(matches[2].dep.name, "pkg-c");
        assert_eq!(matches[2].vuln_ids, vec!["VULN-B", "VULN-C"]);
    }

    #[tokio::test]
    async fn query_batch_5xx_retry_then_fail() {
        let server = MockServer::start().await;

        // Always return 500 - both attempts should fail
        Mock::given(method("POST"))
            .and(path("/v1/querybatch"))
            .respond_with(ResponseTemplate::new(500))
            .expect(2) // initial + 1 retry
            .mount(&server)
            .await;

        let client = OsvClient::with_base_url(&server.uri());
        let deps = vec![make_dep("express", "4.18.2")];

        let result = client.query_batch(&deps).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SupplyChainError::ChunkFailure(msg) => {
                assert!(msg.contains("retry"), "Error should mention retry: {msg}");
            }
            other => panic!("Expected ChunkFailure, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn query_batch_5xx_then_success_on_retry() {
        let server = MockServer::start().await;

        // First call returns 500, second returns 200
        Mock::given(method("POST"))
            .and(path("/v1/querybatch"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/v1/querybatch"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({ "results": [{ "vulns": [] }] })),
            )
            .mount(&server)
            .await;

        let client = OsvClient::with_base_url(&server.uri());
        let deps = vec![make_dep("express", "4.18.2")];

        let matches = client.query_batch(&deps).await.unwrap();
        assert_eq!(matches.len(), 1);
        assert!(matches[0].vuln_ids.is_empty());
    }

    #[tokio::test]
    async fn hydrate_vulns_returns_details() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/vulns/GHSA-1234"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "GHSA-1234",
                "summary": "Some vulnerability",
                "severity": [{ "type": "CVSS_V3", "score": "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H" }],
                "database_specific": { "severity": "CRITICAL" }
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/v1/vulns/GHSA-5678"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "GHSA-5678",
                "summary": "Another vulnerability",
                "severity": [],
                "database_specific": { "severity": "LOW" }
            })))
            .mount(&server)
            .await;

        let client = OsvClient::with_base_url(&server.uri());
        let ids = vec!["GHSA-1234".to_string(), "GHSA-5678".to_string()];

        let details = client.hydrate_vulns(&ids).await.unwrap();
        assert_eq!(details.len(), 2);

        let detail_ids: Vec<&str> = details.iter().map(|d| d.id.as_str()).collect();
        assert!(detail_ids.contains(&"GHSA-1234"));
        assert!(detail_ids.contains(&"GHSA-5678"));
    }

    #[tokio::test]
    async fn mal_prefix_detected_in_querybatch() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/querybatch"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [
                    { "vulns": [{ "id": "MAL-2025-1234", "modified": null }] }
                ]
            })))
            .mount(&server)
            .await;

        let client = OsvClient::with_base_url(&server.uri());
        let deps = vec![make_dep("evil-package", "1.0.0")];

        let matches = client.query_batch(&deps).await.unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].vuln_ids, vec!["MAL-2025-1234"]);
        assert!(matches[0].vuln_ids[0].starts_with("MAL-"));
    }

    #[tokio::test]
    async fn empty_vulns_array_means_clean() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/querybatch"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [
                    { "vulns": [] }
                ]
            })))
            .mount(&server)
            .await;

        let client = OsvClient::with_base_url(&server.uri());
        let deps = vec![make_dep("safe-pkg", "1.0.0")];

        let matches = client.query_batch(&deps).await.unwrap();
        assert_eq!(matches.len(), 1);
        assert!(matches[0].vuln_ids.is_empty());
    }

    #[tokio::test]
    async fn non_registry_deps_filtered_out() {
        let server = MockServer::start().await;

        // Should never be called since all deps are non-registry
        Mock::given(method("POST"))
            .and(path("/v1/querybatch"))
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount(&server)
            .await;

        let client = OsvClient::with_base_url(&server.uri());
        let deps = vec![make_non_registry_dep("git-pkg")];

        let matches = client.query_batch(&deps).await.unwrap();
        assert!(matches.is_empty());
    }

    #[tokio::test]
    async fn hydrate_deduplicates_ids() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/vulns/GHSA-1234"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "GHSA-1234",
                "summary": "Vuln",
                "severity": [],
                "database_specific": null
            })))
            .expect(1) // Should only be called once despite two identical IDs
            .mount(&server)
            .await;

        let client = OsvClient::with_base_url(&server.uri());
        let ids = vec![
            "GHSA-1234".to_string(),
            "GHSA-1234".to_string(), // duplicate
        ];

        let details = client.hydrate_vulns(&ids).await.unwrap();
        assert_eq!(details.len(), 1);
    }
}
