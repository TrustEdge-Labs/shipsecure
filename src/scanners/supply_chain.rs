// Supply chain scan orchestrator.
//
// Wires the lockfile parser -> OSV.dev client -> categorization logic
// to produce a fully categorized SupplyChainScanResult.

use crate::scanners::lockfile_parser::{
    DepFinding, DepSource, DepTier, ParsedDep, SupplyChainError, SupplyChainScanResult,
};
use crate::scanners::osv_client::{OsvClient, OsvVulnDetail};
use std::collections::HashMap;

/// Scan a lockfile string end-to-end: parse -> query OSV -> categorize -> result.
///
/// Uses a default OsvClient targeting production OSV.dev.
pub async fn scan_lockfile(content: &str) -> Result<SupplyChainScanResult, SupplyChainError> {
    let client = OsvClient::new();
    scan_lockfile_with_client(content, &client).await
}

/// Scan a lockfile with a provided OsvClient (for testing).
pub async fn scan_lockfile_with_client(
    content: &str,
    osv_client: &OsvClient,
) -> Result<SupplyChainScanResult, SupplyChainError> {
    // Step 1: Parse lockfile
    let all_deps = crate::scanners::lockfile_parser::parse(content)?;
    let total_deps = all_deps.len();

    // Step 2: Partition into registry vs non-registry
    let (registry_deps, non_registry_deps): (Vec<ParsedDep>, Vec<ParsedDep>) = all_deps
        .into_iter()
        .partition(|d| d.source == DepSource::Registry);

    // Step 3: Query OSV for registry deps
    let vuln_matches = osv_client.query_batch(&registry_deps).await?;

    // Step 4: Collect non-MAL vuln IDs that need hydration
    let ids_to_hydrate: Vec<String> = vuln_matches
        .iter()
        .flat_map(|m| {
            m.vuln_ids
                .iter()
                .filter(|id| !id.starts_with("MAL-"))
                .cloned()
        })
        .collect();

    // Step 5: Hydrate non-MAL vulns
    let hydrated = if ids_to_hydrate.is_empty() {
        vec![]
    } else {
        osv_client.hydrate_vulns(&ids_to_hydrate).await?
    };

    // Build lookup map
    let detail_map: HashMap<String, OsvVulnDetail> =
        hydrated.into_iter().map(|d| (d.id.clone(), d)).collect();

    // Step 6: Categorize each dep
    let mut infected = Vec::new();
    let mut vulnerable = Vec::new();
    let mut advisory = Vec::new();
    let mut no_known_issues = Vec::new();

    for dep_match in &vuln_matches {
        if dep_match.vuln_ids.is_empty() {
            // No vulnerabilities found
            no_known_issues.push(format!("{}@{}", dep_match.dep.name, dep_match.dep.version));
            continue;
        }

        for vuln_id in &dep_match.vuln_ids {
            let detail = detail_map.get(vuln_id);
            let tier = categorize_finding(vuln_id, detail);

            let description = detail.and_then(|d| d.summary.clone()).unwrap_or_default();

            let finding = DepFinding {
                name: dep_match.dep.name.clone(),
                version: dep_match.dep.version.clone(),
                osv_id: vuln_id.clone(),
                description,
                tier: tier.clone(),
            };

            match tier {
                DepTier::Infected => infected.push(finding),
                DepTier::Vulnerable => vulnerable.push(finding),
                DepTier::Advisory => advisory.push(finding),
                _ => {}
            }
        }
    }

    // Step 7: Non-registry deps go to unscanned
    let unscanned = non_registry_deps;

    Ok(SupplyChainScanResult {
        total_deps,
        infected,
        vulnerable,
        advisory,
        no_known_issues,
        unscanned,
        scanned_at: chrono::Utc::now().naive_utc(),
    })
}

/// Categorize a vulnerability finding into a tier.
///
/// Order of precedence:
/// 1. MAL- prefix -> Infected (no hydration data needed)
/// 2. database_specific.severity HIGH/CRITICAL -> Vulnerable
/// 3. Any other OSV match -> Advisory
///
/// Note: CVSS vector parsing is not implemented (no cvss crate).
/// We rely on database_specific.severity which is reliably present
/// for GHSA entries (the majority for npm).
pub fn categorize_finding(vuln_id: &str, detail: Option<&OsvVulnDetail>) -> DepTier {
    // Rule 1: MAL- prefix = malware = Infected
    if vuln_id.starts_with("MAL-") {
        return DepTier::Infected;
    }

    // Rule 2: Check database_specific.severity (normalized to uppercase)
    if let Some(detail) = detail
        && let Some(db_specific) = &detail.database_specific
        && let Some(severity_str) = db_specific.get("severity").and_then(|s| s.as_str())
    {
        let normalized = severity_str.to_uppercase();
        if normalized == "CRITICAL" || normalized == "HIGH" {
            return DepTier::Vulnerable;
        }
    }

    // Rule 3: Any other OSV match = Advisory
    DepTier::Advisory
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanners::osv_client::OsvSeverity;

    fn make_detail(
        id: &str,
        summary: &str,
        severity: Vec<OsvSeverity>,
        db_severity: Option<&str>,
    ) -> OsvVulnDetail {
        let database_specific = db_severity.map(|s| serde_json::json!({ "severity": s }));
        OsvVulnDetail {
            id: id.to_string(),
            summary: Some(summary.to_string()),
            severity,
            database_specific,
        }
    }

    // -- categorize_finding tests --

    #[test]
    fn categorize_mal_prefix_is_infected() {
        let tier = categorize_finding("MAL-2025-1234", None);
        assert_eq!(tier, DepTier::Infected);
    }

    #[test]
    fn categorize_mal_prefix_with_detail_still_infected() {
        let detail = make_detail("MAL-2025-1234", "Malware", vec![], Some("LOW"));
        let tier = categorize_finding("MAL-2025-1234", Some(&detail));
        assert_eq!(tier, DepTier::Infected);
    }

    #[test]
    fn categorize_db_severity_critical_is_vulnerable() {
        let detail = make_detail("GHSA-1234", "Critical vuln", vec![], Some("CRITICAL"));
        let tier = categorize_finding("GHSA-1234", Some(&detail));
        assert_eq!(tier, DepTier::Vulnerable);
    }

    #[test]
    fn categorize_db_severity_high_is_vulnerable() {
        let detail = make_detail("GHSA-5678", "High vuln", vec![], Some("HIGH"));
        let tier = categorize_finding("GHSA-5678", Some(&detail));
        assert_eq!(tier, DepTier::Vulnerable);
    }

    #[test]
    fn categorize_db_severity_high_lowercase_is_vulnerable() {
        let detail = make_detail("GHSA-9999", "High vuln", vec![], Some("high"));
        let tier = categorize_finding("GHSA-9999", Some(&detail));
        assert_eq!(tier, DepTier::Vulnerable);
    }

    #[test]
    fn categorize_db_severity_moderate_is_advisory() {
        let detail = make_detail("GHSA-4444", "Moderate vuln", vec![], Some("MODERATE"));
        let tier = categorize_finding("GHSA-4444", Some(&detail));
        assert_eq!(tier, DepTier::Advisory);
    }

    #[test]
    fn categorize_db_severity_low_is_advisory() {
        let detail = make_detail("GHSA-5555", "Low vuln", vec![], Some("LOW"));
        let tier = categorize_finding("GHSA-5555", Some(&detail));
        assert_eq!(tier, DepTier::Advisory);
    }

    #[test]
    fn categorize_no_severity_data_is_advisory() {
        let detail = OsvVulnDetail {
            id: "GHSA-0000".to_string(),
            summary: Some("Unknown severity".to_string()),
            severity: vec![],
            database_specific: None,
        };
        let tier = categorize_finding("GHSA-0000", Some(&detail));
        assert_eq!(tier, DepTier::Advisory);
    }

    #[test]
    fn categorize_no_detail_at_all_is_advisory() {
        let tier = categorize_finding("GHSA-XXXX", None);
        assert_eq!(tier, DepTier::Advisory);
    }

    // -- Integration-style test for result structure --

    #[tokio::test]
    async fn scan_lockfile_result_structure() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;

        // Mock querybatch: 2 registry deps, first has MAL- vuln, second clean
        Mock::given(method("POST"))
            .and(path("/v1/querybatch"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [
                    { "vulns": [{ "id": "MAL-2025-001", "modified": null }] },
                    { "vulns": [] }
                ]
            })))
            .mount(&server)
            .await;

        // No hydration calls expected (MAL- prefix skips hydration, other dep is clean)

        let lockfile = serde_json::json!({
            "lockfileVersion": 3,
            "packages": {
                "": { "name": "my-app", "version": "1.0.0" },
                "node_modules/evil-pkg": {
                    "version": "1.0.0",
                    "resolved": "https://registry.npmjs.org/evil-pkg/-/evil-pkg-1.0.0.tgz"
                },
                "node_modules/safe-pkg": {
                    "version": "2.0.0",
                    "resolved": "https://registry.npmjs.org/safe-pkg/-/safe-pkg-2.0.0.tgz"
                },
                "node_modules/git-dep": {
                    "version": "3.0.0",
                    "resolved": "git+https://github.com/user/repo.git#abc"
                }
            }
        });

        let client = OsvClient::with_base_url(&server.uri());
        let result = scan_lockfile_with_client(&lockfile.to_string(), &client)
            .await
            .unwrap();

        assert_eq!(result.total_deps, 3);
        assert_eq!(result.infected.len(), 1);
        assert_eq!(result.infected[0].osv_id, "MAL-2025-001");
        assert_eq!(result.infected[0].tier, DepTier::Infected);
        assert_eq!(result.vulnerable.len(), 0);
        assert_eq!(result.advisory.len(), 0);
        assert_eq!(result.no_known_issues.len(), 1);
        assert_eq!(result.no_known_issues[0], "safe-pkg@2.0.0");
        assert_eq!(result.unscanned.len(), 1);
        assert_eq!(result.unscanned[0].name, "git-dep");
    }

    #[tokio::test]
    async fn scan_lockfile_with_hydration() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;

        // querybatch returns a non-MAL vuln
        Mock::given(method("POST"))
            .and(path("/v1/querybatch"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [
                    { "vulns": [{ "id": "GHSA-HIGH-001", "modified": null }] }
                ]
            })))
            .mount(&server)
            .await;

        // Hydration returns HIGH severity
        Mock::given(method("GET"))
            .and(path("/v1/vulns/GHSA-HIGH-001"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "GHSA-HIGH-001",
                "summary": "A high severity vulnerability",
                "severity": [],
                "database_specific": { "severity": "HIGH" }
            })))
            .mount(&server)
            .await;

        let lockfile = serde_json::json!({
            "lockfileVersion": 3,
            "packages": {
                "": { "name": "app", "version": "1.0.0" },
                "node_modules/vuln-pkg": {
                    "version": "1.0.0",
                    "resolved": "https://registry.npmjs.org/vuln-pkg/-/vuln-pkg-1.0.0.tgz"
                }
            }
        });

        let client = OsvClient::with_base_url(&server.uri());
        let result = scan_lockfile_with_client(&lockfile.to_string(), &client)
            .await
            .unwrap();

        assert_eq!(result.total_deps, 1);
        assert_eq!(result.vulnerable.len(), 1);
        assert_eq!(result.vulnerable[0].osv_id, "GHSA-HIGH-001");
        assert_eq!(result.vulnerable[0].tier, DepTier::Vulnerable);
        assert_eq!(
            result.vulnerable[0].description,
            "A high severity vulnerability"
        );
    }

    #[test]
    fn result_serializes_to_json() {
        let result = SupplyChainScanResult {
            total_deps: 2,
            infected: vec![DepFinding {
                name: "evil".into(),
                version: "1.0.0".into(),
                osv_id: "MAL-001".into(),
                description: "Malware".into(),
                tier: DepTier::Infected,
            }],
            vulnerable: vec![],
            advisory: vec![],
            no_known_issues: vec!["safe@1.0.0".into()],
            unscanned: vec![],
            scanned_at: chrono::NaiveDateTime::parse_from_str(
                "2026-01-01 00:00:00",
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap(),
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["total_deps"], 2);
        assert_eq!(json["infected"][0]["osv_id"], "MAL-001");
        assert_eq!(json["no_known_issues"][0], "safe@1.0.0");
    }
}
