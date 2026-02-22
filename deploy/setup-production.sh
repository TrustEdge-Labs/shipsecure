#!/bin/bash
set -euo pipefail

# ShipSecure Production Setup Script
# Run on the server as the deploy user: bash /opt/shipsecure/setup-production.sh
# Idempotent — safe to run multiple times.

APP_DIR="/opt/shipsecure"
SERVICE_FILE="/etc/systemd/system/shipsecure.service"

echo "=== ShipSecure Production Setup ==="
echo ""

# -------------------------------------------------------------------
# Step 1: Stop everything
# -------------------------------------------------------------------
echo "[1/8] Stopping all services..."
sudo systemctl stop shipsecure 2>/dev/null || true
docker stop $(docker ps -aq) 2>/dev/null || true
docker rm $(docker ps -aq) 2>/dev/null || true
docker network prune -f 2>/dev/null || true
echo "  Done — all containers stopped and removed."

# -------------------------------------------------------------------
# Step 2: Verify ports are free
# -------------------------------------------------------------------
echo "[2/8] Verifying ports 3000 and 3001 are free..."
if sudo lsof -i :3000 -i :3001 2>/dev/null | grep LISTEN; then
    echo "  ERROR: Ports still in use. Kill the processes above and re-run."
    exit 1
fi
echo "  Done — ports are free."

# -------------------------------------------------------------------
# Step 3: Write docker-compose.prod.yml (standalone — no dev merge)
# -------------------------------------------------------------------
echo "[3/8] Writing docker-compose.prod.yml..."
cat > "$APP_DIR/docker-compose.prod.yml" << 'EOF'
services:
  backend:
    image: ghcr.io/trustedge-labs/shipsecure-backend:latest
    pull_policy: always
    ports:
      - "127.0.0.1:3000:3000"
    environment:
      DATABASE_URL: ${DATABASE_URL}
      PORT: ${PORT:-3000}
      SHIPSECURE_BASE_URL: ${SHIPSECURE_BASE_URL}
      FRONTEND_URL: ${FRONTEND_URL}
      MAX_CONCURRENT_SCANS: ${MAX_CONCURRENT_SCANS:-10}
      CLERK_JWKS_URL: ${CLERK_JWKS_URL}
      RESEND_API_KEY: ${RESEND_API_KEY:-}
      RUST_LOG: ${RUST_LOG:-info,shipsecure=info}
      LOG_FORMAT: ${LOG_FORMAT:-json}
      SHUTDOWN_TIMEOUT: ${SHUTDOWN_TIMEOUT:-90}
      HEALTH_DB_LATENCY_THRESHOLD_MS: ${HEALTH_DB_LATENCY_THRESHOLD_MS:-200}
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 2G
    restart: unless-stopped
    stop_grace_period: 95s
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"

  frontend:
    image: ghcr.io/trustedge-labs/shipsecure-frontend:latest
    pull_policy: always
    ports:
      - "127.0.0.1:3001:3001"
    environment:
      HOSTNAME: 0.0.0.0
      BACKEND_URL: http://backend:3000
      NEXT_PUBLIC_BACKEND_URL: ${NEXT_PUBLIC_BACKEND_URL}
      CLERK_SECRET_KEY: ${CLERK_SECRET_KEY}
    depends_on:
      backend:
        condition: service_started
    deploy:
      resources:
        limits:
          cpus: '1.0'
          memory: 1G
    restart: unless-stopped
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
EOF
echo "  Done."

# -------------------------------------------------------------------
# Step 4: Clean up .env (remove stale COMPOSE_FILE if present)
# -------------------------------------------------------------------
echo "[4/8] Cleaning .env..."
sed -i '/^COMPOSE_FILE=/d' "$APP_DIR/.env" 2>/dev/null || true
echo "  Done."

# -------------------------------------------------------------------
# Step 5: Validate required env vars
# -------------------------------------------------------------------
echo "[5/8] Validating required environment variables..."
MISSING=""
source "$APP_DIR/.env"
for var in DATABASE_URL SHIPSECURE_BASE_URL FRONTEND_URL CLERK_JWKS_URL CLERK_SECRET_KEY NEXT_PUBLIC_BACKEND_URL; do
    if [ -z "${!var:-}" ]; then
        MISSING="$MISSING  - $var\n"
    fi
done
if [ -n "$MISSING" ]; then
    echo "  ERROR: Missing required variables in $APP_DIR/.env:"
    echo -e "$MISSING"
    echo "  Add them to $APP_DIR/.env and re-run."
    exit 1
fi
echo "  Done — all required variables present."

# -------------------------------------------------------------------
# Step 6: Install systemd service
# -------------------------------------------------------------------
echo "[6/8] Installing systemd service..."
sudo tee "$SERVICE_FILE" > /dev/null << 'EOF'
[Unit]
Description=ShipSecure Application
Requires=docker.service
After=docker.service network-online.target
Wants=network-online.target

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=/opt/shipsecure
EnvironmentFile=/opt/shipsecure/.env
ExecStart=/bin/bash -c '/usr/bin/docker compose -f docker-compose.prod.yml pull && /usr/bin/docker compose -f docker-compose.prod.yml up -d'
ExecStop=/usr/bin/docker compose -f docker-compose.prod.yml down
TimeoutStartSec=120
TimeoutStopSec=95s
Restart=on-failure
RestartSec=15
User=deploy
Group=deploy

[Install]
WantedBy=multi-user.target
EOF
sudo systemctl daemon-reload
sudo systemctl enable shipsecure
echo "  Done."

# -------------------------------------------------------------------
# Step 7: Pull images and start
# -------------------------------------------------------------------
echo "[7/8] Pulling images and starting services..."
cd "$APP_DIR"
docker compose -f docker-compose.prod.yml pull
docker compose -f docker-compose.prod.yml up -d
echo "  Done."

# -------------------------------------------------------------------
# Step 8: Health check
# -------------------------------------------------------------------
echo "[8/8] Waiting for services to start..."
sleep 10

echo ""
echo "=== Service Status ==="
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
echo ""

# Check backend
BACKEND_STATUS=$(docker inspect --format='{{.State.Status}}' shipsecure-backend-1 2>/dev/null || echo "missing")
if [ "$BACKEND_STATUS" != "running" ]; then
    echo "BACKEND: FAILED (status: $BACKEND_STATUS)"
    echo "Logs:"
    docker logs shipsecure-backend-1 --tail 10 2>&1
    echo ""
else
    echo "BACKEND: OK"
fi

# Check frontend
FRONTEND_STATUS=$(docker inspect --format='{{.State.Status}}' shipsecure-frontend-1 2>/dev/null || echo "missing")
if [ "$FRONTEND_STATUS" != "running" ]; then
    echo "FRONTEND: FAILED (status: $FRONTEND_STATUS)"
    echo "Logs:"
    docker logs shipsecure-frontend-1 --tail 10 2>&1
    echo ""
else
    echo "FRONTEND: OK"
fi

# Check HTTP
HTTP_CODE=$(curl -sS -o /dev/null -w "%{http_code}" --max-time 10 http://localhost:3001 2>/dev/null || echo "000")
echo "HTTP localhost:3001: $HTTP_CODE"

if [ "$BACKEND_STATUS" = "running" ] && [ "$FRONTEND_STATUS" = "running" ]; then
    echo ""
    echo "=== SUCCESS ==="
    echo "Both services are running."
    echo "Check https://shipsecure.ai to verify."
else
    echo ""
    echo "=== ISSUES DETECTED ==="
    echo "Review the logs above and fix before continuing."
    exit 1
fi
