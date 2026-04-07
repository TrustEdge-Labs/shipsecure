// Supply chain scanning: lockfile parser and shared types
//
// This module defines the shared types used by all supply chain modules
// (lockfile_parser, osv_client, supply_chain) and implements the lockfile
// parser that extracts dependencies from package-lock.json v1/v2/v3.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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

// -- Parser --

/// Classify the source of a dependency based on its `resolved` and `link` fields.
fn classify_source(pkg: &serde_json::Value) -> DepSource {
    // Check for workspace link packages first
    if pkg.get("link").and_then(|v| v.as_bool()).unwrap_or(false) {
        return DepSource::Link;
    }
    match pkg.get("resolved").and_then(|r| r.as_str()) {
        Some(r) if r.starts_with("git+") => DepSource::Git,
        Some(r) if r.starts_with("file:") => DepSource::File,
        Some(r) if r.starts_with("link:") => DepSource::Link,
        Some(_) => DepSource::Registry,
        None => DepSource::Tarball,
    }
}

/// Parse a package-lock.json string into a list of dependencies.
///
/// Supports lockfileVersion 1, 2, and 3. Returns an error for unsupported
/// versions or malformed JSON.
pub fn parse(content: &str) -> Result<Vec<ParsedDep>, SupplyChainError> {
    let root: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| SupplyChainError::LockfileParse(e.to_string()))?;

    let version = root
        .get("lockfileVersion")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);

    match version {
        3 => parse_v3(&root),
        1 | 2 => {
            // v2 may have a `packages` key (same structure as v3)
            if root.get("packages").and_then(|p| p.as_object()).is_some() {
                parse_v3(&root)
            } else {
                parse_v1_v2_nested(&root)
            }
        }
        other => Err(SupplyChainError::LockfileParse(format!(
            "Unsupported lockfileVersion: {other}"
        ))),
    }
}

/// Parse v3-style lockfile (and v2 with `packages` key).
/// The `packages` object maps node_modules paths to package metadata.
fn parse_v3(root: &serde_json::Value) -> Result<Vec<ParsedDep>, SupplyChainError> {
    let packages = root
        .get("packages")
        .and_then(|p| p.as_object())
        .ok_or_else(|| {
            SupplyChainError::LockfileParse("Missing 'packages' key for v3 lockfile".into())
        })?;

    let mut seen = HashSet::new();
    let mut deps = Vec::new();

    for (key, pkg) in packages {
        // Skip root entry
        if key.is_empty() {
            continue;
        }

        // Extract package name: strip everything up to and including the LAST
        // `node_modules/` occurrence. Handles both:
        //   node_modules/@babel/core -> @babel/core
        //   node_modules/parent/node_modules/@babel/core -> @babel/core
        let name = match key.rfind("node_modules/") {
            Some(idx) => &key[idx + "node_modules/".len()..],
            None => key.as_str(),
        };

        // Skip entries with empty or missing version
        let version = match pkg.get("version").and_then(|v| v.as_str()) {
            Some(v) if !v.is_empty() => v.to_string(),
            _ => continue,
        };

        // Dedup by (name, version)
        let dedup_key = (name.to_string(), version.clone());
        if !seen.insert(dedup_key) {
            continue;
        }

        let source = classify_source(pkg);
        let is_dev = pkg.get("dev").and_then(|v| v.as_bool()).unwrap_or(false);

        deps.push(ParsedDep {
            name: name.to_string(),
            version,
            source,
            is_dev,
        });
    }

    Ok(deps)
}

/// Parse v1/v2-style lockfile with nested `dependencies` objects.
fn parse_v1_v2_nested(root: &serde_json::Value) -> Result<Vec<ParsedDep>, SupplyChainError> {
    let dependencies = root
        .get("dependencies")
        .and_then(|d| d.as_object())
        .ok_or_else(|| {
            SupplyChainError::LockfileParse("Missing 'dependencies' key for v1 lockfile".into())
        })?;

    let mut seen = HashSet::new();
    let mut deps = Vec::new();

    walk_nested_deps(dependencies, &mut seen, &mut deps);

    Ok(deps)
}

/// Recursively walk nested dependency objects (v1/v2 format).
fn walk_nested_deps(
    dependencies: &serde_json::Map<String, serde_json::Value>,
    seen: &mut HashSet<(String, String)>,
    deps: &mut Vec<ParsedDep>,
) {
    for (name, pkg) in dependencies {
        // Skip entries with empty or missing version
        let version = match pkg.get("version").and_then(|v| v.as_str()) {
            Some(v) if !v.is_empty() => v.to_string(),
            _ => continue,
        };

        // Dedup by (name, version)
        let dedup_key = (name.clone(), version.clone());
        if !seen.insert(dedup_key) {
            continue;
        }

        let source = classify_source(pkg);
        let is_dev = pkg.get("dev").and_then(|v| v.as_bool()).unwrap_or(false);

        deps.push(ParsedDep {
            name: name.clone(),
            version,
            source,
            is_dev,
        });

        // Recurse into nested dependencies
        if let Some(nested) = pkg.get("dependencies").and_then(|d| d.as_object()) {
            walk_nested_deps(nested, seen, deps);
        }
    }
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
        let variants = [
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
        assert!(
            json.is_ok(),
            "SupplyChainScanResult should serialize to JSON"
        );
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

    // -- Parser tests --

    #[test]
    fn parse_v3() {
        let json = r#"{
            "lockfileVersion": 3,
            "packages": {
                "": { "name": "my-app", "version": "1.0.0" },
                "node_modules/express": { "version": "4.18.2", "resolved": "https://registry.npmjs.org/express/-/express-4.18.2.tgz" },
                "node_modules/lodash": { "version": "4.17.21", "resolved": "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz" },
                "node_modules/@babel/core": { "version": "7.24.0", "resolved": "https://registry.npmjs.org/@babel/core/-/core-7.24.0.tgz" }
            }
        }"#;
        let deps = parse(json).unwrap();
        assert_eq!(deps.len(), 3);
        let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"express"));
        assert!(names.contains(&"lodash"));
        assert!(names.contains(&"@babel/core"));
    }

    #[test]
    fn parse_v3_skips_root() {
        let json = r#"{
            "lockfileVersion": 3,
            "packages": {
                "": { "name": "root", "version": "1.0.0" },
                "node_modules/foo": { "version": "1.0.0", "resolved": "https://registry.npmjs.org/foo/-/foo-1.0.0.tgz" }
            }
        }"#;
        let deps = parse(json).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "foo");
    }

    #[test]
    fn parse_v3_dedup() {
        let json = r#"{
            "lockfileVersion": 3,
            "packages": {
                "node_modules/foo": { "version": "1.0.0", "resolved": "https://registry.npmjs.org/foo/-/foo-1.0.0.tgz" },
                "node_modules/bar/node_modules/foo": { "version": "1.0.0", "resolved": "https://registry.npmjs.org/foo/-/foo-1.0.0.tgz" }
            }
        }"#;
        let deps = parse(json).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "foo");
    }

    #[test]
    fn parse_v3_nested_node_modules() {
        let json = r#"{
            "lockfileVersion": 3,
            "packages": {
                "node_modules/parent/node_modules/@babel/core": { "version": "7.24.0", "resolved": "https://registry.npmjs.org/@babel/core/-/core-7.24.0.tgz" }
            }
        }"#;
        let deps = parse(json).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "@babel/core");
    }

    #[test]
    fn parse_v1() {
        let json = r#"{
            "lockfileVersion": 1,
            "dependencies": {
                "express": {
                    "version": "4.18.2",
                    "resolved": "https://registry.npmjs.org/express/-/express-4.18.2.tgz",
                    "dependencies": {
                        "accepts": {
                            "version": "1.3.8",
                            "resolved": "https://registry.npmjs.org/accepts/-/accepts-1.3.8.tgz"
                        }
                    }
                }
            }
        }"#;
        let deps = parse(json).unwrap();
        assert_eq!(deps.len(), 2);
        let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"express"));
        assert!(names.contains(&"accepts"));
    }

    #[test]
    fn parse_v2_with_packages() {
        let json = r#"{
            "lockfileVersion": 2,
            "packages": {
                "": { "name": "my-app", "version": "1.0.0" },
                "node_modules/express": { "version": "4.18.2", "resolved": "https://registry.npmjs.org/express/-/express-4.18.2.tgz" }
            },
            "dependencies": {
                "express": { "version": "4.18.2", "resolved": "https://registry.npmjs.org/express/-/express-4.18.2.tgz" }
            }
        }"#;
        let deps = parse(json).unwrap();
        // Should use packages key (v3 path), not dependencies
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "express");
    }

    #[test]
    fn parse_non_registry_sources() {
        let json = r#"{
            "lockfileVersion": 3,
            "packages": {
                "node_modules/git-dep": { "version": "1.0.0", "resolved": "git+https://github.com/user/repo.git#abc123" },
                "node_modules/file-dep": { "version": "2.0.0", "resolved": "file:../local-pkg" },
                "node_modules/link-dep": { "version": "3.0.0", "resolved": "link:../workspace-pkg" },
                "node_modules/no-resolved": { "version": "4.0.0" }
            }
        }"#;
        let deps = parse(json).unwrap();
        assert_eq!(deps.len(), 4);

        let git_dep = deps.iter().find(|d| d.name == "git-dep").unwrap();
        assert_eq!(git_dep.source, DepSource::Git);

        let file_dep = deps.iter().find(|d| d.name == "file-dep").unwrap();
        assert_eq!(file_dep.source, DepSource::File);

        let link_dep = deps.iter().find(|d| d.name == "link-dep").unwrap();
        assert_eq!(link_dep.source, DepSource::Link);

        let no_resolved = deps.iter().find(|d| d.name == "no-resolved").unwrap();
        assert_eq!(no_resolved.source, DepSource::Tarball);
    }

    #[test]
    fn parse_dev_deps() {
        let json = r#"{
            "lockfileVersion": 3,
            "packages": {
                "node_modules/prod-dep": { "version": "1.0.0", "resolved": "https://registry.npmjs.org/prod/-/prod-1.0.0.tgz" },
                "node_modules/dev-dep": { "version": "2.0.0", "resolved": "https://registry.npmjs.org/dev/-/dev-2.0.0.tgz", "dev": true }
            }
        }"#;
        let deps = parse(json).unwrap();
        let prod = deps.iter().find(|d| d.name == "prod-dep").unwrap();
        assert!(!prod.is_dev);
        let dev = deps.iter().find(|d| d.name == "dev-dep").unwrap();
        assert!(dev.is_dev);
    }

    #[test]
    fn parse_empty_version_skipped() {
        let json = r#"{
            "lockfileVersion": 3,
            "packages": {
                "node_modules/no-version": { "resolved": "https://registry.npmjs.org/nv/-/nv-1.0.0.tgz" },
                "node_modules/empty-version": { "version": "", "resolved": "https://registry.npmjs.org/ev/-/ev-1.0.0.tgz" },
                "node_modules/good": { "version": "1.0.0", "resolved": "https://registry.npmjs.org/good/-/good-1.0.0.tgz" }
            }
        }"#;
        let deps = parse(json).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "good");
    }

    #[test]
    fn parse_invalid_json() {
        let result = parse("not json at all");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Lockfile parse error"),
            "Expected LockfileParse error, got: {err}"
        );
    }

    #[test]
    fn parse_missing_packages_key() {
        let json = r#"{ "lockfileVersion": 3 }"#;
        let result = parse(json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Missing 'packages'"),
            "Expected missing packages error, got: {err}"
        );
    }

    #[test]
    fn parse_unsupported_version() {
        let json = r#"{ "lockfileVersion": 99 }"#;
        let result = parse(json);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Unsupported lockfileVersion: 99"),
            "Expected unsupported version error, got: {err}"
        );
    }

    #[test]
    fn parse_workspace_link_package() {
        let json = r#"{
            "lockfileVersion": 3,
            "packages": {
                "node_modules/workspace-pkg": { "version": "1.0.0", "link": true }
            }
        }"#;
        let deps = parse(json).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].source, DepSource::Link);
    }
}
