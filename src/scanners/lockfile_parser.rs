// Supply chain scanning: lockfile parser and shared types
//
// This module defines the shared types used by all supply chain modules
// (lockfile_parser, osv_client, supply_chain) and implements the lockfile
// parser that extracts dependencies from package-lock.json v1/v2/v3.

use serde::{Deserialize, Serialize};
use std::fmt;

// -- Error type --

#[derive(Debug)]
pub enum SupplyChainError {
    LockfileParse(String),
    OsvQuery(String),
    GitHubFetch(String),
    ChunkFailure(String),
    DepCountExceeded(usize),
    Timeout,
}

impl fmt::Display for SupplyChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SupplyChainError::LockfileParse(msg) => write!(f, "Lockfile parse error: {msg}"),
            SupplyChainError::OsvQuery(msg) => write!(f, "OSV query error: {msg}"),
            SupplyChainError::GitHubFetch(msg) => write!(f, "GitHub fetch error: {msg}"),
            SupplyChainError::ChunkFailure(msg) => write!(f, "Chunk failure: {msg}"),
            SupplyChainError::DepCountExceeded(n) => {
                write!(f, "Dependency count exceeded: {n}")
            }
            SupplyChainError::Timeout => write!(f, "Operation timed out"),
        }
    }
}

impl std::error::Error for SupplyChainError {}

// -- Dependency types --

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DepSource {
    Registry,
    Git,
    File,
    Link,
    Tarball,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDep {
    pub name: String,
    pub version: String,
    pub source: DepSource,
    pub is_dev: bool,
}

// -- Result types --

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DepTier {
    Infected,
    Vulnerable,
    Advisory,
    NoKnownIssues,
    Unscanned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepFinding {
    pub name: String,
    pub version: String,
    pub osv_id: String,
    pub description: String,
    pub tier: DepTier,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SupplyChainScanResult {
    pub total_deps: usize,
    pub infected: Vec<DepFinding>,
    pub vulnerable: Vec<DepFinding>,
    pub advisory: Vec<DepFinding>,
    pub no_known_issues: Vec<String>,
    pub unscanned: Vec<ParsedDep>,
    pub scanned_at: chrono::NaiveDateTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lockfile_parse_error_display() {
        let err = SupplyChainError::LockfileParse("bad json".into());
        assert_eq!(err.to_string(), "Lockfile parse error: bad json");
    }

    #[test]
    fn chunk_failure_error_display() {
        let err = SupplyChainError::ChunkFailure("timeout".into());
        assert_eq!(err.to_string(), "Chunk failure: timeout");
    }

    #[test]
    fn osv_query_error_display() {
        let err = SupplyChainError::OsvQuery("connection refused".into());
        assert_eq!(err.to_string(), "OSV query error: connection refused");
    }

    #[test]
    fn github_fetch_error_display() {
        let err = SupplyChainError::GitHubFetch("404".into());
        assert_eq!(err.to_string(), "GitHub fetch error: 404");
    }

    #[test]
    fn dep_count_exceeded_display() {
        let err = SupplyChainError::DepCountExceeded(5001);
        assert_eq!(err.to_string(), "Dependency count exceeded: 5001");
    }

    #[test]
    fn timeout_error_display() {
        let err = SupplyChainError::Timeout;
        assert_eq!(err.to_string(), "Operation timed out");
    }

    #[test]
    fn dep_source_variants_are_distinct() {
        let variants = vec![
            DepSource::Registry,
            DepSource::Git,
            DepSource::File,
            DepSource::Link,
            DepSource::Tarball,
        ];
        for (i, a) in variants.iter().enumerate() {
            for (j, b) in variants.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn parsed_dep_constructible() {
        let dep = ParsedDep {
            name: "express".into(),
            version: "4.18.2".into(),
            source: DepSource::Registry,
            is_dev: false,
        };
        assert_eq!(dep.name, "express");
        assert_eq!(dep.version, "4.18.2");
        assert_eq!(dep.source, DepSource::Registry);
        assert!(!dep.is_dev);
    }

    #[test]
    fn supply_chain_scan_result_serializes() {
        let result = SupplyChainScanResult {
            total_deps: 0,
            infected: vec![],
            vulnerable: vec![],
            advisory: vec![],
            no_known_issues: vec![],
            unscanned: vec![],
            scanned_at: chrono::NaiveDateTime::parse_from_str(
                "2026-01-01 00:00:00",
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap(),
        };
        let json = serde_json::to_string(&result);
        assert!(json.is_ok(), "SupplyChainScanResult should serialize to JSON");
    }

    #[test]
    fn dep_tier_serializes_correctly() {
        assert_eq!(
            serde_json::to_string(&DepTier::Infected).unwrap(),
            "\"Infected\""
        );
        assert_eq!(
            serde_json::to_string(&DepTier::Vulnerable).unwrap(),
            "\"Vulnerable\""
        );
        assert_eq!(
            serde_json::to_string(&DepTier::Advisory).unwrap(),
            "\"Advisory\""
        );
        assert_eq!(
            serde_json::to_string(&DepTier::NoKnownIssues).unwrap(),
            "\"NoKnownIssues\""
        );
        assert_eq!(
            serde_json::to_string(&DepTier::Unscanned).unwrap(),
            "\"Unscanned\""
        );
    }
}
