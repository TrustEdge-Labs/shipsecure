use crate::models::finding::Finding;
use std::collections::HashMap;

/// Compute a letter grade (A+ to F) based on total severity score
pub fn compute_score(findings: &[Finding]) -> String {
    let total_score: i32 = findings.iter().map(|f| f.severity.score_weight()).sum();

    match total_score {
        0 => "A+".to_string(),
        1..=5 => "A".to_string(),
        6..=10 => "B".to_string(),
        11..=20 => "C".to_string(),
        21..=40 => "D".to_string(),
        _ => "F".to_string(),
    }
}

/// Deduplicate findings by title, keeping the highest severity and combining scanner names
pub fn deduplicate_findings(findings: Vec<Finding>) -> Vec<Finding> {
    let mut dedup_map: HashMap<String, Finding> = HashMap::new();

    for finding in findings {
        let title = finding.title.clone();

        dedup_map
            .entry(title.clone())
            .and_modify(|existing| {
                // Keep the highest severity
                if finding.severity > existing.severity {
                    existing.severity = finding.severity.clone();
                }

                // Combine scanner names in raw_evidence
                if finding.scanner_name != existing.scanner_name {
                    let combined_scanners = format!(
                        "Also found by: {}",
                        finding.scanner_name
                    );

                    if let Some(ref mut evidence) = existing.raw_evidence {
                        if !evidence.contains(&finding.scanner_name) {
                            evidence.push_str(&format!("\n{}", combined_scanners));
                        }
                    } else {
                        existing.raw_evidence = Some(combined_scanners);
                    }
                }
            })
            .or_insert(finding);
    }

    let mut result: Vec<Finding> = dedup_map.into_values().collect();

    // Sort by severity (highest first) for consistent ordering
    result.sort_by(|a, b| b.severity.cmp(&a.severity));

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::finding::Severity;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_finding(title: &str, severity: Severity, scanner: &str) -> Finding {
        Finding {
            id: Uuid::new_v4(),
            scan_id: Uuid::nil(),
            scanner_name: scanner.to_string(),
            title: title.to_string(),
            description: "Test description".to_string(),
            severity,
            remediation: "Test remediation".to_string(),
            raw_evidence: Some("Test evidence".to_string()),
            vibe_code: false,
            created_at: Utc::now().naive_utc(),
        }
    }

    #[test]
    fn test_compute_score_empty() {
        let findings = vec![];
        assert_eq!(compute_score(&findings), "A+");
    }

    #[test]
    fn test_compute_score_two_high() {
        let findings = vec![
            create_test_finding("Finding 1", Severity::High, "scanner1"),
            create_test_finding("Finding 2", Severity::High, "scanner1"),
        ];
        // 2 High = 2 * 5 = 10 points -> B
        assert_eq!(compute_score(&findings), "B");
    }

    #[test]
    fn test_compute_score_critical_and_high() {
        let findings = vec![
            create_test_finding("Finding 1", Severity::Critical, "scanner1"),
            create_test_finding("Finding 2", Severity::High, "scanner1"),
        ];
        // Critical (10) + High (5) = 15 points -> C
        assert_eq!(compute_score(&findings), "C");
    }

    #[test]
    fn test_compute_score_boundary_cases() {
        // Test exact boundaries
        let findings_1pt = vec![create_test_finding("F1", Severity::Low, "s1")];
        assert_eq!(compute_score(&findings_1pt), "A");

        let findings_5pt = vec![create_test_finding("F1", Severity::High, "s1")];
        assert_eq!(compute_score(&findings_5pt), "A");

        let findings_6pt = vec![
            create_test_finding("F1", Severity::High, "s1"),
            create_test_finding("F2", Severity::Low, "s1"),
        ];
        assert_eq!(compute_score(&findings_6pt), "B");

        let findings_41pt = vec![
            create_test_finding("F1", Severity::Critical, "s1"),
            create_test_finding("F2", Severity::Critical, "s1"),
            create_test_finding("F3", Severity::Critical, "s1"),
            create_test_finding("F4", Severity::Critical, "s1"),
            create_test_finding("F5", Severity::Low, "s1"),
        ];
        assert_eq!(compute_score(&findings_41pt), "F");
    }

    #[test]
    fn test_deduplicate_same_title() {
        let findings = vec![
            create_test_finding("Missing CSP", Severity::High, "scanner1"),
            create_test_finding("Missing CSP", Severity::Medium, "scanner2"),
        ];

        let deduped = deduplicate_findings(findings);

        // Should have only one finding
        assert_eq!(deduped.len(), 1);

        // Should keep the higher severity
        assert_eq!(deduped[0].severity, Severity::High);

        // Should note the other scanner
        assert!(deduped[0].raw_evidence.as_ref().unwrap().contains("scanner2"));
    }

    #[test]
    fn test_deduplicate_different_titles() {
        let findings = vec![
            create_test_finding("Missing CSP", Severity::High, "scanner1"),
            create_test_finding("Missing HSTS", Severity::High, "scanner1"),
        ];

        let deduped = deduplicate_findings(findings);

        // Should keep both findings
        assert_eq!(deduped.len(), 2);
    }

    #[test]
    fn test_deduplicate_empty() {
        let findings = vec![];
        let deduped = deduplicate_findings(findings);
        assert_eq!(deduped.len(), 0);
    }
}
