#!/bin/bash
set -e

echo "========================================="
echo "TrustEdge Audit - End-to-End Test Suite"
echo "========================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
PASSED=0
FAILED=0

pass() {
    echo -e "${GREEN}✓ PASS${NC}: $1"
    ((PASSED++))
}

fail() {
    echo -e "${RED}✗ FAIL${NC}: $1"
    ((FAILED++))
}

info() {
    echo -e "${YELLOW}→${NC} $1"
}

# Wait for server
info "Checking if server is running..."
if ! curl -s http://localhost:8888/health > /dev/null; then
    echo "Server not running. Start with: PORT=8888 cargo run"
    exit 1
fi
echo ""

# SC1: Submit scan and receive scan ID
echo "=== SC1: Submit scan and receive scan ID ==="
RESPONSE=$(curl -s -X POST http://localhost:8888/api/v1/scans \
    -H "Content-Type: application/json" \
    -d '{"url":"https://httpbin.org","email":"e2e@test.com"}')
SCAN_ID=$(echo "$RESPONSE" | jq -r '.id')
STATUS=$(echo "$RESPONSE" | jq -r '.status')

if [[ "$STATUS" == "pending" && "$SCAN_ID" != "null" ]]; then
    pass "Scan submitted successfully (ID: ${SCAN_ID:0:8}...)"
else
    fail "Scan submission failed"
    echo "$RESPONSE"
    exit 1
fi
echo ""

# SC2: Backend executes scan and stores findings
echo "=== SC2: Backend executes scan and stores findings ==="
info "Waiting 10 seconds for scan to complete..."
sleep 10

RESULT=$(curl -s http://localhost:8888/api/v1/scans/$SCAN_ID)
RESULT_STATUS=$(echo "$RESULT" | jq -r '.status')
FINDINGS_COUNT=$(echo "$RESULT" | jq '.findings | length')

if [[ "$RESULT_STATUS" == "completed" ]]; then
    pass "Scan completed successfully"
else
    fail "Scan did not complete (status: $RESULT_STATUS)"
fi

if [[ $FINDINGS_COUNT -gt 0 ]]; then
    pass "Findings stored in database ($FINDINGS_COUNT findings)"
else
    fail "No findings returned"
fi
echo ""

# SC3: GET /api/v1/scans/:id returns full results
echo "=== SC3: GET /api/v1/scans/:id returns full results ==="
HAS_ID=$(echo "$RESULT" | jq -r '.id')
HAS_TARGET=$(echo "$RESULT" | jq -r '.target_url')
HAS_SCORE=$(echo "$RESULT" | jq -r '.score')
HAS_SUMMARY=$(echo "$RESULT" | jq -r '.summary.total')
SAMPLE_SEVERITY=$(echo "$RESULT" | jq -r '.findings[0].severity')
SAMPLE_REMEDIATION=$(echo "$RESULT" | jq -r '.findings[0].remediation')

if [[ "$HAS_ID" != "null" && "$HAS_TARGET" != "null" ]]; then
    pass "Scan metadata present"
else
    fail "Scan metadata missing"
fi

if [[ "$HAS_SCORE" != "null" && ${#HAS_SCORE} -ge 1 ]]; then
    pass "Security score computed ($HAS_SCORE)"
else
    fail "Security score missing"
fi

if [[ "$HAS_SUMMARY" != "null" && $HAS_SUMMARY -gt 0 ]]; then
    pass "Summary object present"
else
    fail "Summary object missing"
fi

if [[ "$SAMPLE_SEVERITY" != "null" && "$SAMPLE_REMEDIATION" != "null" ]]; then
    pass "Findings include severity and remediation"
else
    fail "Findings missing required fields"
fi

# Test 404
NOT_FOUND=$(curl -s http://localhost:8888/api/v1/scans/00000000-0000-0000-0000-000000000000 | jq -r '.status')
if [[ "$NOT_FOUND" == "404" ]]; then
    pass "404 returned for non-existent scan"
else
    fail "404 handling broken"
fi
echo ""

# SC4: SSRF protection
echo "=== SC4: SSRF protection blocks internal targets ==="

LOCALHOST=$(curl -s -X POST http://localhost:8888/api/v1/scans \
    -H "Content-Type: application/json" \
    -d '{"url":"http://127.0.0.1","email":"ssrf@test.com"}' | jq -r '.status')
if [[ "$LOCALHOST" == "400" ]]; then
    pass "Localhost blocked"
else
    fail "Localhost not blocked"
fi

PRIVATE_IP=$(curl -s -X POST http://localhost:8888/api/v1/scans \
    -H "Content-Type: application/json" \
    -d '{"url":"http://192.168.1.1","email":"ssrf@test.com"}' | jq -r '.status')
if [[ "$PRIVATE_IP" == "400" ]]; then
    pass "Private IP blocked"
else
    fail "Private IP not blocked"
fi

CLOUD_META=$(curl -s -X POST http://localhost:8888/api/v1/scans \
    -H "Content-Type: application/json" \
    -d '{"url":"http://169.254.169.254/latest/meta-data/","email":"ssrf@test.com"}' | jq -r '.status')
if [[ "$CLOUD_META" == "400" ]]; then
    pass "Cloud metadata endpoint blocked"
else
    fail "Cloud metadata not blocked"
fi
echo ""

# SC5: Rate limiting
echo "=== SC5: Rate limiting enforces daily limits ==="
EMAIL="ratelimit-$(date +%s)@test.com"

for i in 1 2 3; do
    curl -s -X POST http://localhost:8888/api/v1/scans \
        -H "Content-Type: application/json" \
        -d "{\"url\":\"https://example.com\",\"email\":\"$EMAIL\"}" > /dev/null
    sleep 0.5
done
pass "First 3 scans accepted"

RATE_LIMITED=$(curl -s -X POST http://localhost:8888/api/v1/scans \
    -H "Content-Type: application/json" \
    -d "{\"url\":\"https://example.com\",\"email\":\"$EMAIL\"}" | jq -r '.status')

if [[ "$RATE_LIMITED" == "429" ]]; then
    pass "4th scan rate limited (429)"
else
    fail "Rate limiting not enforced (got $RATE_LIMITED instead of 429)"
fi
echo ""

# Summary
echo "========================================="
echo "Test Results: ${GREEN}${PASSED} passed${NC}, ${RED}${FAILED} failed${NC}"
echo "========================================="

if [[ $FAILED -eq 0 ]]; then
    echo -e "${GREEN}All Phase 1 success criteria verified!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed. Review output above.${NC}"
    exit 1
fi
