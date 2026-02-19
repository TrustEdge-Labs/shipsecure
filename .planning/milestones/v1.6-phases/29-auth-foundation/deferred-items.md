# Phase 29 Deferred Items

## Pre-existing Test Failures (Out of Scope)

### scanners::js_secrets::tests::test_false_positive_detection

- **Found during:** Task 1 (cargo test)
- **Status:** Pre-existing failure — confirmed by stash test before our changes
- **Location:** `src/scanners/js_secrets.rs:367`
- **Issue:** `is_false_positive("AKIAIOSFODNN7EXAMPLE", "production code")` returns `false` but test asserts it should return `true` (i.e., "AKIAIOSFODNN7EXAMPLE" should be recognized as a test/example value)
- **Impact:** 1 test failure; 62 other tests pass
- **Action required:** Fix false-positive detection logic in `src/scanners/js_secrets.rs` to recognize "EXAMPLE" suffix as a false positive indicator
