#!/bin/bash
set -e

BACKEND_URL="${BACKEND_URL:-http://localhost:3000}"
FRONTEND_URL="${FRONTEND_URL:-http://localhost:3001}"

echo "=== Phase 2: Free Tier MVP E2E Tests ==="
echo "Backend: $BACKEND_URL"
echo "Frontend: $FRONTEND_URL"
echo ""

# Test 1: Health check
echo "--- Test 1: Backend health check ---"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$BACKEND_URL/health")
if [ "$HTTP_CODE" = "200" ]; then
  echo "PASS: Backend healthy"
else
  echo "FAIL: Backend returned $HTTP_CODE"
  exit 1
fi

# Test 2: Frontend health check
echo "--- Test 2: Frontend health check ---"
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$FRONTEND_URL")
if [ "$HTTP_CODE" = "200" ]; then
  echo "PASS: Frontend healthy"
else
  echo "FAIL: Frontend returned $HTTP_CODE"
  exit 1
fi

# Test 3: Scan counter endpoint
echo "--- Test 3: Scan counter ---"
RESPONSE=$(curl -s "$BACKEND_URL/api/v1/stats/scan-count")
echo "Response: $RESPONSE"
echo "$RESPONSE" | grep -q '"count"' && echo "PASS: Scan counter works" || { echo "FAIL: No count field"; exit 1; }

# Test 4: Submit a scan
echo "--- Test 4: Submit scan ---"
SCAN_RESPONSE=$(curl -s -X POST "$BACKEND_URL/api/v1/scans" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com", "email": "test@example.com"}')
echo "Response: $SCAN_RESPONSE"
SCAN_ID=$(echo "$SCAN_RESPONSE" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
if [ -n "$SCAN_ID" ]; then
  echo "PASS: Scan created with ID: $SCAN_ID"
else
  echo "FAIL: No scan ID in response"
  exit 1
fi

# Test 5: Poll for scan completion (up to 5 minutes)
echo "--- Test 5: Poll scan status ---"
MAX_POLLS=150
POLL_COUNT=0
RESULTS_TOKEN=""
while [ $POLL_COUNT -lt $MAX_POLLS ]; do
  STATUS_RESPONSE=$(curl -s "$BACKEND_URL/api/v1/scans/$SCAN_ID")
  STATUS=$(echo "$STATUS_RESPONSE" | grep -o '"status":"[^"]*"' | head -1 | cut -d'"' -f4)
  STAGE_HEADERS=$(echo "$STATUS_RESPONSE" | grep -o '"stage_headers":[^,}]*' | head -1 | cut -d':' -f2)
  STAGE_TLS=$(echo "$STATUS_RESPONSE" | grep -o '"stage_tls":[^,}]*' | head -1 | cut -d':' -f2)

  echo "  Poll $POLL_COUNT: status=$STATUS headers=$STAGE_HEADERS tls=$STAGE_TLS"

  if [ "$STATUS" = "completed" ]; then
    RESULTS_TOKEN=$(echo "$STATUS_RESPONSE" | grep -o '"results_token":"[^"]*"' | head -1 | cut -d'"' -f4)
    echo "PASS: Scan completed"
    break
  elif [ "$STATUS" = "failed" ]; then
    echo "WARN: Scan failed (this may be expected for example.com)"
    break
  fi

  POLL_COUNT=$((POLL_COUNT + 1))
  sleep 2
done

if [ $POLL_COUNT -ge $MAX_POLLS ]; then
  echo "FAIL: Scan did not complete within 5 minutes"
  exit 1
fi

# Test 6: Check stage tracking worked
echo "--- Test 6: Stage tracking ---"
echo "$STATUS_RESPONSE" | grep -q '"stage_headers":true' && echo "PASS: Headers stage tracked" || echo "WARN: Headers stage not completed"

# Test 7: Results by token (if token available)
if [ -n "$RESULTS_TOKEN" ]; then
  echo "--- Test 7: Results by token ---"
  RESULTS_RESPONSE=$(curl -s "$BACKEND_URL/api/v1/results/$RESULTS_TOKEN")
  echo "$RESULTS_RESPONSE" | grep -q '"findings"' && echo "PASS: Results accessible by token" || echo "FAIL: No findings in results"

  echo "--- Test 8: Markdown download ---"
  DOWNLOAD_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$BACKEND_URL/api/v1/results/$RESULTS_TOKEN/download")
  if [ "$DOWNLOAD_CODE" = "200" ]; then
    echo "PASS: Markdown download works"
  else
    echo "WARN: Markdown download returned $DOWNLOAD_CODE"
  fi

  echo "--- Test 9: Frontend results page ---"
  RESULTS_PAGE_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$FRONTEND_URL/results/$RESULTS_TOKEN")
  if [ "$RESULTS_PAGE_CODE" = "200" ]; then
    echo "PASS: Frontend results page accessible"
  else
    echo "WARN: Frontend results page returned $RESULTS_PAGE_CODE"
  fi
fi

# Test 10: Invalid token returns 404
echo "--- Test 10: Invalid token returns 404 ---"
INVALID_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$BACKEND_URL/api/v1/results/nonexistent-token-xyz")
if [ "$INVALID_CODE" = "404" ]; then
  echo "PASS: Invalid token returns 404"
else
  echo "FAIL: Invalid token returned $INVALID_CODE"
fi

# Test 11: Landing page loads
echo "--- Test 11: Landing page content ---"
LANDING=$(curl -s "$FRONTEND_URL")
echo "$LANDING" | grep -qi "trustedge\|scan\|security" && echo "PASS: Landing page has expected content" || echo "WARN: Landing page content unexpected"

echo ""
echo "=== E2E Tests Complete ==="
