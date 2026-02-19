---
phase: 05-codebase-preparation
plan: 01
subsystem: scanning-infrastructure
tags: [rust, scanners, subprocess, nuclei, testssl, native-binaries]
requires:
  - v1.0 scanner implementation (Docker-based)
provides:
  - Native binary subprocess execution for Nuclei and testssl.sh
  - Configurable scanner binary paths via environment variables
  - Graceful degradation when scanner binaries not found
affects:
  - 05-02: Environment configuration (needs NUCLEI_BINARY_PATH, TESTSSL_BINARY_PATH)
  - 06-01: Server provisioning (needs to install Nuclei and testssl.sh binaries)
  - 06-02: Docker setup (no longer needs Docker socket access for scanners)
tech-stack:
  added:
    - which: "7.0.3 - Binary path resolution via PATH lookup"
    - tempfile: "3.24.0 - Temporary file creation for JSON output"
  patterns:
    - Native subprocess execution with tokio::process::Command
    - Environment variable fallback to PATH lookup for binary resolution
    - Temp file output capture to avoid stdout buffering issues
    - Graceful feature detection (skip scanner if binary not found)
key-files:
  created: []
  modified:
    - Cargo.toml: "Added which and tempfile dependencies"
    - src/scanners/container.rs: "Refactored from Docker to native binary execution"
    - src/scanners/vibecode.rs: "Refactored to use native Nuclei with local templates"
key-decisions:
  - decision: "Binary path resolution via NUCLEI_BINARY_PATH/TESTSSL_BINARY_PATH env vars with PATH fallback"
    rationale: "Allows explicit override for production while still working in dev environments via PATH"
    alternatives: "Hardcoded paths (not flexible), PATH-only (no override), config file (adds complexity)"
  - decision: "Temp file output capture instead of stdout"
    rationale: "Avoids stdout buffering deadlocks for large JSON output (Nuclei can produce >64KB)"
    alternatives: "Stdout streaming (complex), chunked reading (error-prone), stdout with large buffer (still risks deadlock)"
  - decision: "Graceful degradation when binaries not found"
    rationale: "Dev environments may not have scanners installed, app should still start and serve other features"
    alternatives: "Hard fail (breaks dev experience), stub scanners (misleading results)"
  - decision: "Renamed error variants to generic names (BinaryNotFound vs DockerNotAvailable)"
    rationale: "Error names now reflect native execution model, more accurate for troubleshooting"
    alternatives: "Keep Docker names (misleading), add new errors (duplicates similar cases)"
duration: "276 seconds (4.6 minutes)"
completed: 2026-02-07
---

# Phase 05 Plan 01: Scanner Native Binary Execution Summary

**One-liner:** Refactored Nuclei and testssl.sh from Docker container execution to native binary subprocesses with configurable paths and temp file output.

---

## Performance

**Execution time:** 276 seconds (4.6 minutes)
**Start:** 2026-02-07T01:58:05Z
**End:** 2026-02-07T02:02:41Z

**Tasks completed:** 2/2
**Files modified:** 3
**Test suites passed:** 2 (container + vibecode scanners)

---

## Accomplishments

### Core Functionality

**Native binary execution model:**
- Replaced all Docker container execution (`docker run ...`) with direct native binary subprocess invocation (`Command::new(nuclei_binary)`)
- Removed all Docker-specific code (hardened container args, volume mounts, Docker availability checks)
- No Docker daemon dependency required for scanner execution

**Binary path resolution:**
- Created `resolve_nuclei_binary()` and `resolve_testssl_binary()` functions
- Resolution priority: env var (NUCLEI_BINARY_PATH) → PATH lookup → common install paths (/usr/local/bin, /usr/bin, /opt)
- Logs warnings when env var path doesn't exist but continues to fallback

**Temp file output capture:**
- Uses `tempfile::NamedTempFile` for safe cross-platform temp file creation
- Nuclei outputs to file via `-o` flag instead of stdout
- testssl.sh outputs to file via `--jsonfile-pretty` flag
- Temp files auto-cleaned via RAII when going out of scope

**Graceful degradation:**
- If Nuclei binary not found: logs warning, returns empty findings (doesn't fail scan)
- If testssl.sh binary not found: logs warning, returns empty findings
- Other scanners (JS secrets) continue to work when binary scanners unavailable

**Error handling improvements:**
- Renamed error variants to reflect native execution:
  - `DockerNotAvailable` → `BinaryNotFound`
  - `ContainerTimeout` → `ScanTimeout`
  - `ContainerError` → `ExecutionError`
- Errors now accurately describe native subprocess failures

### Code Quality

**Removed duplicates:**
- Deleted `is_docker_available()` duplicate in vibecode.rs
- Deleted `run_docker_container()` duplicates in both container.rs and vibecode.rs
- Shared `resolve_nuclei_binary()` function used by both scanner modules

**Template path resolution:**
- Updated `select_templates()` in vibecode.rs to accept `templates_dir` parameter
- Template paths resolved from local filesystem instead of Docker container paths
- Changed from `/templates/foo.yaml` to `{templates_dir}/foo.yaml` pattern

**Test coverage:**
- Updated `test_docker_availability` → `test_binary_resolution` (tests graceful handling of missing binaries)
- Updated vibecode template tests to pass `templates_dir` parameter
- All existing parser tests still pass (no changes to JSON parsing logic)

---

## Task Commits

| Task | Description | Commit | Files Modified |
|------|-------------|--------|----------------|
| 1 | Add dependencies and refactor container.rs | 203c268 | Cargo.toml, src/scanners/container.rs |
| 2 | Refactor vibecode.rs to native Nuclei execution | cfc5b7e | src/scanners/vibecode.rs |

**Commit details:**

**Task 1 (203c268):**
- Added `which = "7"` and `tempfile = "3"` dependencies
- Created `resolve_nuclei_binary()` and `resolve_testssl_binary()`
- Refactored `run_nuclei()` to execute native binary with temp file output
- Refactored `run_testssl()` to execute native binary with temp file output
- Removed `run_docker_container()` function
- Renamed error enum variants
- Updated tests

**Task 2 (cfc5b7e):**
- Refactored `scan_vibecode()` to call `resolve_nuclei_binary()` from container.rs
- Updated to execute Nuclei as native binary with local template paths
- Modified `select_templates()` to accept `templates_dir` and resolve filesystem paths
- Removed duplicate `is_docker_available()` and `run_docker_container()` functions
- Updated all tests to pass `templates_dir` parameter

---

## Files Created

None. This plan modified existing scanner modules only.

---

## Files Modified

**Cargo.toml:**
- Added `which = "7"` for binary path resolution
- Added `tempfile = "3"` for temp file creation

**src/scanners/container.rs:**
- Added imports: `std::path::PathBuf`
- Created `resolve_nuclei_binary()` function (public, for sharing with vibecode.rs)
- Created `resolve_testssl_binary()` function (public)
- Refactored `run_nuclei()`: native binary execution with temp file output
- Refactored `run_testssl()`: native binary execution with temp file output
- Removed `is_docker_available()` function
- Removed `run_docker_container()` function
- Renamed error variants in `ScannerError` enum
- Updated `test_docker_availability` → `test_binary_resolution`

**src/scanners/vibecode.rs:**
- Refactored `scan_vibecode()`: native Nuclei execution with temp file output
- Updated `select_templates()` to accept `templates_dir: &PathBuf` parameter
- Changed all template paths from `/templates/foo.yaml` to `{templates_dir}/foo.yaml`
- Removed duplicate `is_docker_available()` function
- Removed duplicate `run_docker_container()` function
- Updated all tests to pass `PathBuf::from("/templates")` to `select_templates()`

---

## Decisions Made

**1. Binary path resolution strategy**

**Decision:** Use environment variables with fallback to PATH lookup and common install paths.

**Context:** Production deployments need explicit control over binary locations, but dev environments should "just work" if binaries are in PATH.

**Implementation:**
```rust
pub fn resolve_nuclei_binary() -> Option<PathBuf> {
    // 1. Check NUCLEI_BINARY_PATH env var
    // 2. Check PATH via which::which("nuclei")
    // 3. Check /usr/local/bin/nuclei, /usr/bin/nuclei, /opt/nuclei/bin/nuclei
    // 4. Return None (triggers graceful skip)
}
```

**Impact:** Deployment flexibility increased. Dev experience improved (no config required if binary in PATH).

---

**2. Temp file output capture**

**Decision:** Use temp files for JSON output instead of capturing stdout.

**Context:** Large Nuclei scans can produce >64KB of JSON, exceeding typical subprocess stdout buffer size (64KB). Reading stdout can deadlock if buffer fills while process is still writing.

**Implementation:**
- Create `NamedTempFile` before execution
- Pass path to binary via `-o` flag (Nuclei) or `--jsonfile-pretty` flag (testssl.sh)
- Read file contents after process completes
- Temp file auto-deleted via RAII

**Alternative considered:** Async stdout streaming with chunked reading. Rejected due to complexity and error-proneness.

**Impact:** Eliminates stdout buffering deadlocks. Slightly slower (disk I/O) but more reliable.

---

**3. Graceful degradation when binaries not found**

**Decision:** Return empty findings instead of failing the entire scan.

**Context:** Dev environments may not have Nuclei/testssl.sh installed. App should still start and serve other features (JS secrets scanner, URL analysis, database queries).

**Implementation:**
```rust
let nuclei_binary = match resolve_nuclei_binary() {
    Some(path) => path,
    None => {
        tracing::warn!("Nuclei binary not found, skipping Nuclei scan");
        return Ok(Vec::new());
    }
};
```

**Alternative considered:** Hard fail with error. Rejected because it breaks dev experience and prevents partial scans.

**Impact:** Dev-friendliness increased. Production requires explicit verification that binaries are installed.

---

**4. Error variant renaming**

**Decision:** Rename error variants from Docker-specific to generic execution terms.

**Context:** Error names like `DockerNotAvailable` are now inaccurate since we don't use Docker. Generic names like `BinaryNotFound` are more accurate.

**Implementation:**
- `DockerNotAvailable` → `BinaryNotFound`
- `ContainerTimeout` → `ScanTimeout`
- `ContainerError` → `ExecutionError`

**Alternative considered:** Keep Docker names for backward compatibility. Rejected because no external API consumers exist (all internal).

**Impact:** Error messages now match actual failure mode. Easier troubleshooting.

---

## Deviations from Plan

None. Plan executed exactly as written.

---

## Issues Encountered

**Issue 1: vibecode.rs used duplicate error variants**

**Context:** After renaming error variants in container.rs, vibecode.rs still referenced old names (`ContainerError`, `ContainerTimeout`).

**Resolution:** Updated vibecode.rs to use new names (`ExecutionError`, `ScanTimeout`) as part of Task 2.

**Impact:** Caught by compiler, fixed immediately. No manual intervention needed.

---

## Next Phase Readiness

**Phase 05-02 (Environment Configuration) readiness:**

**Ready to proceed:**
- Scanner binary path resolution implemented
- Environment variable pattern established (NUCLEI_BINARY_PATH, TESTSSL_BINARY_PATH)
- Graceful degradation tested

**Requires in 05-02:**
- Add NUCLEI_BINARY_PATH and TESTSSL_BINARY_PATH to .env.example
- Document installation instructions for Nuclei and testssl.sh
- Add TRUSTEDGE_TEMPLATES_DIR to .env.example (already implemented in code)

**Handoff notes:**
- Binary resolution functions are public and tested
- Temp file pattern is established and working
- No Docker socket access needed for scanners (simplifies Docker Compose setup)

---

**Phase 06-01 (Server Provisioning) readiness:**

**Requires in 06-01:**
- Install Nuclei binary on droplet (`curl -L https://github.com/projectdiscovery/nuclei/releases/... | tar xz`)
- Install testssl.sh binary on droplet (`git clone https://github.com/drwetter/testssl.sh /opt/testssl`)
- Verify binaries are in PATH or set env vars explicitly
- Copy templates directory to droplet filesystem

**Alternative paths:**
- If testssl.sh native install is problematic, can revert to Docker for testssl.sh only
- If Nuclei binary not available for platform, can revert to Docker for Nuclei only

**No blockers:** All infrastructure changes in this plan are self-contained.

---

## Self-Check: PASSED

**Files created:** None (expected for this plan)

**Files modified verification:**
- [FOUND] Cargo.toml
- [FOUND] src/scanners/container.rs
- [FOUND] src/scanners/vibecode.rs

**Commits verification:**
- [FOUND] 203c268
- [FOUND] cfc5b7e

**Compilation check:** PASSED (cargo check)

**Test suites:** PASSED (3 container tests + 9 vibecode tests)

**Docker references:** NONE (grep -i docker returned no matches)

**Dependencies:** VERIFIED (which and tempfile in Cargo.toml)

All verification checks passed. Plan completed successfully.
