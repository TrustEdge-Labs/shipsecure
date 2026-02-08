# Pitfalls Research

**Domain:** DigitalOcean Single-Droplet Deployment (Rust/Next.js/PostgreSQL/Docker)
**Researched:** 2026-02-06
**Confidence:** HIGH

## Critical Pitfalls

### Pitfall 1: Docker Socket Security — Root-Equivalent Access

**What goes wrong:**
Mounting `/var/run/docker.sock` into your Rust backend container to enable `docker run` for Nuclei scans gives that container **root-equivalent access** to the host. A compromised backend container can escape isolation, start privileged containers, mount the host filesystem (`/`), and completely own the droplet. With user-submitted URLs being scanned, this is a direct attack vector.

**Why it happens:**
The Docker socket is the Docker API entry point. Any process with access to it can execute arbitrary Docker commands as root. Developers mount it into containers for convenience ("need Docker-in-Docker") without understanding the security implications.

**How to avoid:**
1. **Run Nuclei natively as subprocess** — Install Nuclei binary directly on the host, invoke via `std::process::Command` from Rust backend. No Docker socket access needed.
2. **If Docker socket required:** Use Docker socket proxy (tecnativa/docker-socket-proxy) with read-only access and whitelist only necessary API endpoints (`containers.create`, `containers.start`, `containers.wait`).
3. **Harden spawned containers:** Already doing this (8 CIS flags: `--read-only`, `--cap-drop=ALL`, `--security-opt=no-new-privileges`, `--network=none`, `--pids-limit=64`, `--memory=512m`, `--cpus=1.0`, `--user=nonroot`). Continue this.
4. **Network isolation:** Nuclei containers use `--network=none`. Keep this for external scans to prevent SSRF.

**Warning signs:**
- Seeing `docker.sock` mounted in `docker-compose.yml` or systemd service
- Backend container running with `privileged: true`
- Docker socket mounted read-write instead of read-only

**Phase to address:**
**Phase 1 (Infrastructure Setup)** — Decide architecture: native Nuclei binary vs. containerized with socket proxy. Native is simpler and safer for single-droplet MVP.

---

### Pitfall 2: UFW Firewall Bypass — Exposed Ports Despite Rules

**What goes wrong:**
UFW (Uncomplicated Firewall) **does not protect Docker-published ports**. Docker manipulates iptables directly in the `nat` table, bypassing UFW's INPUT/OUTPUT chains entirely. Publishing ports with `-p 5432:5432` (PostgreSQL) or `-p 3000:3000` (backend) exposes them to the internet **even if UFW denies them**. Your database becomes publicly accessible.

**Why it happens:**
Docker inserts early iptables ACCEPT rules before UFW sees the packets. UFW sits later in the packet pipeline. Both modify the same iptables config, causing misconfigurations. This is a well-known security flaw that still exists in 2026.

**How to avoid:**
1. **Bind ports to localhost:** Use `127.0.0.1:5432:5432` instead of `5432:5432` to prevent external access. Nginx on host can still proxy to `localhost:3000`.
2. **Don't publish unnecessary ports:** PostgreSQL and backend should NOT publish ports if only accessed via Docker network or localhost.
3. **Use ufw-docker tool:** Install [chaifeng/ufw-docker](https://github.com/chaifeng/ufw-docker) to link UFW chains into `DOCKER-USER` chain.
4. **Verify with nmap:** After deployment, run `nmap -p 1-65535 <droplet-ip>` from external machine to verify only 80/443 are exposed.

**Warning signs:**
- `docker ps` shows `0.0.0.0:5432->5432/tcp` instead of `127.0.0.1:5432->5432/tcp`
- Running `ufw status` shows port blocked, but `nmap` from outside shows it open
- PostgreSQL logs showing connection attempts from unknown IPs

**Phase to address:**
**Phase 2 (Docker Compose Production Config)** — Rewrite port bindings to use `127.0.0.1:` prefix. Verify with external nmap scan.

---

### Pitfall 3: PostgreSQL Data Loss — Ephemeral Containers Without Persistent Volumes

**What goes wrong:**
Docker containers are ephemeral. Without a persistent volume, **all PostgreSQL data is lost** when the container stops, is removed, or crashes. `docker-compose down` wipes the database. Redeployment after a security patch deletes all scan history and user emails.

**Why it happens:**
Developers forget to mount volumes or use anonymous volumes that Docker deletes. Testing in dev doesn't catch this because "it works on my machine" until production container restarts.

**How to avoid:**
1. **Use named volumes:** Already have `pgdata:/var/lib/postgresql/data` in docker-compose.yml. Keep this.
2. **Verify volume persistence:** `docker volume ls` should show `trustedge_pgdata`. Run `docker-compose down && docker-compose up` and verify data survives.
3. **Backup automation:** Set up daily pg_dump to external storage (DigitalOcean Spaces or Dropbox). Use cron job: `docker exec postgres pg_dump -U trustedge trustedge_prod > /backup/$(date +\%Y\%m\%d).sql`
4. **Snapshot the volume directory:** DigitalOcean droplet backups snapshot entire disk, but configure weekly snapshots separately for `/var/lib/docker/volumes/`.

**Warning signs:**
- No volumes listed in `docker-compose.yml` for `db` service
- Running `docker volume ls` shows no volumes with project prefix
- PostgreSQL data directory is `/var/lib/postgresql/data` without `:volume` mapping

**Phase to address:**
**Phase 2 (Docker Compose Production Config)** — Verify volume exists and persists.
**Phase 4 (Backup & Monitoring)** — Implement automated pg_dump backups and test restore.

---

### Pitfall 4: Memory Exhaustion from Concurrent Scans — OOM Killer Murdering Containers

**What goes wrong:**
Each Nuclei scan spawns an ephemeral container with 512MB memory limit. With 5 concurrent scans (your semaphore limit), that's **2.5GB for scans alone**, plus backend (256MB?), frontend (256MB?), PostgreSQL (512MB?), Nginx (32MB?) = **~3.5GB total**. On a $12/month droplet (2GB RAM), the Linux OOM killer **randomly murders containers** when memory pressure occurs. PostgreSQL or backend could die mid-scan.

**Why it happens:**
Developers size containers for "typical" workload, not burst workload. Five concurrent scans hitting maximum memory simultaneously triggers OOM. Kernel kills highest memory consumer (often PostgreSQL).

**How to avoid:**
1. **Right-size droplet:** For 5 concurrent scans at 512MB each, minimum **4GB RAM droplet** ($24/month). Calculate: `(5 × 512MB) + backend + frontend + PostgreSQL + Nginx + OS overhead (512MB) = 4GB`.
2. **Add swap:** Configure 2GB swap as safety buffer for short bursts. `fallocate -l 2G /swapfile && chmod 600 /swapfile && mkswap /swapfile && swapon /swapfile`.
3. **Set memory limits on all containers:** Explicitly set `mem_limit: 256m` for backend, `mem_limit: 256m` for frontend, `mem_limit: 512m` for PostgreSQL in docker-compose.yml.
4. **Monitor with alerts:** Install and configure monitoring (Prometheus + Grafana or DigitalOcean monitoring) to alert when memory >85%.
5. **Reduce concurrency if needed:** If budget-constrained, reduce semaphore from 5 to 3 concurrent scans (1.5GB for scans, fits on 2GB droplet with swap).

**Warning signs:**
- `dmesg | grep -i oom` shows "Out of memory: Killed process" messages
- Container status shows "Exited (137)" (137 = 128 + 9 = killed by SIGKILL from OOM)
- Scans failing randomly with "container not running" errors
- PostgreSQL connection errors: "could not connect to server"

**Phase to address:**
**Phase 1 (Infrastructure Setup)** — Choose droplet size based on memory calculation.
**Phase 2 (Docker Compose Production Config)** — Set explicit memory limits on all services.
**Phase 4 (Monitoring)** — Set up memory alerts and OOM detection.

---

### Pitfall 5: Disk Space Exhaustion — Logs and Images Filling Droplet

**What goes wrong:**
Docker logs are **not rotated by default**. Each container writes unbounded logs to `/var/lib/docker/containers/*/`. Over weeks, log files grow to gigabytes, filling the 50GB droplet disk. When disk hits 100%, PostgreSQL cannot write WAL files, scans fail, and the entire app crashes. Orphaned Docker images (from redeployments) accumulate, consuming 5-10GB each.

**Why it happens:**
Docker doesn't configure log rotation out of the box. Developers don't notice in dev because logs are small. In production, a single verbose container can generate 1GB/week.

**How to avoid:**
1. **Configure log rotation in docker-compose.yml:**
   ```yaml
   services:
     backend:
       logging:
         driver: "json-file"
         options:
           max-size: "10m"
           max-file: "3"
   ```
   This limits each container to 30MB of logs (10MB × 3 files).

2. **Prune old images weekly:** Set up cron job: `0 2 * * 0 docker system prune -af --volumes` (Sundays at 2am, removes unused images/volumes).

3. **Monitor disk usage:** Add to monitoring: `df -h /` should alert at >80% disk usage.

4. **Use external logging (optional):** For production visibility, ship logs to external service (Papertrail, Logtail, or DigitalOcean managed logging).

**Warning signs:**
- `df -h` shows `/` at >90% usage
- `du -sh /var/lib/docker/containers` shows multi-GB sizes
- `docker system df` shows large "Build Cache" or "Images" sizes
- Errors like "no space left on device" in logs

**Phase to address:**
**Phase 2 (Docker Compose Production Config)** — Add log rotation to all services.
**Phase 4 (Monitoring)** — Set up disk space alerts and automated pruning.

---

### Pitfall 6: Let's Encrypt Certificate Renewal Failure — Silent SSL Expiration

**What goes wrong:**
Let's Encrypt certificates expire after 90 days. Certbot auto-renewal **silently fails** when Nginx is misconfigured (wrong web root, IPv6 disabled, stale config). Three months later, your site shows "Certificate Expired" warnings, blocking all users. No monitoring = no alert until users complain.

**Why it happens:**
Certbot renewal relies on HTTP-01 challenge (writes file to `.well-known/acme-challenge/`), but Nginx config doesn't serve that directory, or certbot uses wrong web root path. IPv6 AAAA records exist, but Nginx only listens on IPv4. Renewal cron job runs, fails silently, logs ignored.

**How to avoid:**
1. **Test renewal immediately after setup:** `certbot renew --dry-run` to verify renewal works before 90 days pass.
2. **Configure Nginx for ACME challenge:**
   ```nginx
   location /.well-known/acme-challenge/ {
       root /var/www/certbot;
   }
   ```
3. **Listen on IPv6:** Ensure Nginx listens on both IPv4 and IPv6:
   ```nginx
   listen 443 ssl;
   listen [::]:443 ssl;
   ```
4. **Monitor renewal:** Add certbot logs to monitoring, or use external SSL monitoring (SSL Labs Scan, UptimeRobot SSL expiry checks).
5. **Set up renewal cron job explicitly:** `0 3 * * * certbot renew --quiet && systemctl reload nginx`

**Warning signs:**
- `certbot renew --dry-run` fails with "Connection refused" or "Failed authorization procedure"
- Certbot logs at `/var/log/letsencrypt/letsencrypt.log` show errors
- `openssl s_client -connect yourdomain.com:443 -servername yourdomain.com` shows expiration date <30 days away
- Certificate not renewed after 60 days (Let's Encrypt should renew at 30 days remaining)

**Phase to address:**
**Phase 3 (SSL Configuration)** — Set up certbot with proper Nginx config, test dry run.
**Phase 4 (Monitoring)** — Add SSL expiry monitoring and renewal failure alerts.

---

### Pitfall 7: Systemd Restart Loops — Crash Loops from Missing Environment Variables

**What goes wrong:**
Systemd service configured with `Restart=always` encounters a missing environment variable (e.g., `STRIPE_SECRET_KEY` or `DATABASE_URL`). Backend crashes on startup. Systemd immediately restarts it. It crashes again. **Infinite restart loop** floods logs, burns CPU, and never reaches healthy state. Application appears "running" in systemd but is non-functional.

**Why it happens:**
Environment variables not loaded in systemd service file. Developer tests with `docker-compose` (which loads `.env` file) but systemd doesn't. Backend starts, reads config, panics on missing env var, exits. Systemd sees exit, restarts per policy.

**How to avoid:**
1. **Use systemd EnvironmentFile:**
   ```ini
   [Service]
   EnvironmentFile=/home/deploy/trustedge/.env.production
   ExecStart=/usr/local/bin/docker-compose up
   Restart=on-failure
   RestartSec=10s
   StartLimitBurst=3
   StartLimitIntervalSec=300
   ```
2. **Use `on-failure` instead of `always`:** Only restart on actual failures, not manual stops.
3. **Add StartLimitBurst:** Limit to 3 restart attempts within 5 minutes before giving up. Prevents infinite loops.
4. **Validate environment on startup:** Backend should log "Missing required env var: STRIPE_SECRET_KEY" before panic, making diagnosis obvious.
5. **Use systemd `ConditionPathExists`:** Ensure `.env.production` exists before starting.

**Warning signs:**
- `systemctl status trustedge-backend` shows "Active (running)" but uptime keeps resetting to <10s
- `journalctl -u trustedge-backend -n 100` shows repeated startup/crash cycles
- CPU spiking due to rapid restart attempts
- Backend logs show repeated initialization but no requests processed

**Phase to address:**
**Phase 3 (Systemd Service Setup)** — Configure systemd with EnvironmentFile, restart policies, and start limits.

---

### Pitfall 8: Nginx Reverse Proxy Connection Refused — Wrong Backend Address

**What goes wrong:**
Nginx on the **host** tries to proxy to `http://backend:3000` (Docker service name), but that hostname doesn't exist on the host network. Or Nginx uses `http://localhost:3000`, which works in dev but fails in production because backend isn't on localhost if it's in a container. Result: 502 Bad Gateway, app unreachable.

**Why it happens:**
Confusion between Docker network namespaces and host network. `backend` hostname only resolves inside Docker network, not on host. `localhost` refers to Nginx container's localhost, not the backend container's localhost.

**How to avoid:**
1. **If Nginx on host:** Use `http://127.0.0.1:3000` after binding backend port to host with `127.0.0.1:3000:3000` in docker-compose.yml.
2. **If Nginx in container:** Use Docker service name `http://backend:3000` and put Nginx in same Docker network.
3. **For this project (Nginx on host):** Backend and frontend should publish to `127.0.0.1:PORT`, then Nginx proxies to `http://127.0.0.1:3000` (backend) and `http://127.0.0.1:3001` (frontend).
4. **Test before SSL:** Verify `curl http://127.0.0.1:3000/health` from host returns 200 before configuring Nginx.

**Warning signs:**
- Nginx logs show `connect() failed (111: Connection refused) while connecting to upstream`
- 502 Bad Gateway error when accessing frontend
- `curl localhost:3000` works, but Nginx proxy returns 502
- Nginx config has `proxy_pass http://backend:3000` but Nginx is on host

**Phase to address:**
**Phase 2 (Docker Compose Production Config)** — Configure port bindings to 127.0.0.1.
**Phase 3 (Nginx Configuration)** — Configure Nginx to proxy to 127.0.0.1:3000/3001.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Running PostgreSQL in Docker on single droplet | Simple, no managed DB cost ($15/mo) | No automatic backups, no HA, manual scaling, backup/restore complexity | **Acceptable for MVP** — Upgrade to managed DB after validating product-market fit |
| Using `.env` files for secrets | Simple configuration, works with docker-compose | Secrets in plain text on filesystem, visible in process list, shared insecurely | **Acceptable for MVP** — Migrate to Docker Secrets or Vault for multi-droplet/team |
| Storing scans in PostgreSQL (no S3/object storage) | Simple, one less service, no external cost | Database bloat from large scan results, expensive to backup, difficult to archive old scans | **Acceptable for MVP** — Move to S3 after 10K scans or when DB >5GB |
| Manual deployment via SSH | Simple, no CI/CD complexity | Deployment errors, no rollback, downtime during updates | **Acceptable for solo MVP** — Add GitHub Actions after first paying customer |
| Single droplet (no load balancer) | Low cost ($24/mo vs $120+), simple architecture | No zero-downtime deployments, single point of failure, can't scale horizontally | **Acceptable until 1K active users** — Add load balancer when revenue >$1K/mo |
| Polling for scan status (no WebSockets) | Simple implementation, works with Nginx | Higher latency (2-5s updates), more backend load | **Acceptable permanently** — Polling is fine for 3-5min scan duration |

---

## Integration Gotchas

Common mistakes when connecting to external services.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Stripe Webhook | Using localhost URL in Stripe dashboard, webhooks never arrive | Use Stripe CLI for local dev (`stripe listen --forward-to localhost:3000/webhooks`), use real domain for production |
| Resend Email API | Not handling rate limits (10 emails/sec), bulk sends fail | Implement exponential backoff, queue emails if >10/sec, handle 429 responses |
| SSL Labs API | Not respecting rate limits (1 request/scan, max 25/hour), getting banned | Cache results for 24h, implement rate limiter in backend, handle 429/503 responses |
| Nuclei Templates | Using outdated templates, missing new checks | Run `nuclei -update-templates` daily via cron, pin template version for reproducibility |
| Docker Hub Rate Limits | Anonymous pulls limited to 100/6h, hitting limit during deployments | Authenticate to Docker Hub (200/6h), cache images locally, use DigitalOcean Container Registry |
| DigitalOcean Firewall | Enabling DO cloud firewall + UFW causes double firewall complexity | Choose one: UFW with ufw-docker for single droplet OR DO cloud firewall for multi-droplet |

---

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| PostgreSQL connection pool exhaustion | "Connection pool exhausted" errors, 500s during traffic spikes | Right-size pool: `max_connections = (RAM in GB × 100)`, use PgBouncer for >100 concurrent requests | >50 concurrent scans or >200 req/s |
| Database-as-queue for scans | Scans stuck in "pending" when workers crash, no retry logic | Add `status` index, implement dead letter queue for failed scans after 3 attempts, add scan timeout (15min) | >500 scans/day or >10 concurrent |
| In-memory PDF generation blocking requests | PDF generation (5-10s) blocks Axum worker thread, response times spike | Move PDF generation to background task queue (tokio::spawn), return scan ID immediately, poll for PDF | PDFs >2MB or >20 PDFs/hour |
| N+1 queries for scan results | Loading scan + findings + remediation = 3 queries per scan, slow dashboard | Use SQL JOINs or eager loading, fetch all scan data in single query | >100 scans in database |
| Docker image layers not cached | Rebuilding images from scratch on every deployment, 5-10min downtime | Use multi-stage builds, `.dockerignore` for node_modules, cache layers in registry | Any production deployment |
| Nginx serving frontend static files from Docker | Slow file serving, high memory usage, Nginx proxies to Next.js for every static asset | Use Next.js `output: 'standalone'` + copy static assets to Nginx directory, serve directly | >100 concurrent users |

---

## Security Mistakes

Domain-specific security issues beyond general web security.

| Mistake | Risk | Prevention |
|---------|------|------------|
| Not validating user-submitted URLs before scanning | SSRF attacks: scan internal network (192.168.x.x, 10.x.x.x, 169.254.x.x), scan localhost services, scan cloud metadata endpoints (169.254.169.254) | Already implemented: validate URL scheme (http/https only), block private IP ranges, block localhost, block cloud metadata IPs |
| Exposing PostgreSQL port to internet (0.0.0.0:5432) | Direct database access for attackers, brute force attacks on postgres user, data exfiltration | Bind PostgreSQL to 127.0.0.1:5432 only, use strong passwords (32+ chars random), rotate credentials quarterly |
| Storing Stripe secret keys in .env with weak permissions | Keys readable by all users on droplet, keys in bash history, keys in process list | Set .env file permissions to 600 (owner read/write only), never echo keys, use Docker secrets for multi-container |
| Not sanitizing Nuclei output before display | XSS via malicious scan targets returning crafted HTML in error messages | Sanitize all Nuclei output, escape HTML in frontend, use `dangerouslySetInnerHTML` never |
| Running backend as root in container | Container escape = root on host, privilege escalation attacks | Already using `--user=nonroot` for Nuclei containers, verify backend Dockerfile uses `USER nonroot` |
| No rate limiting on free tier scans | Abuse: scanning entire internet, DoS by exhausting scan queue, resource exhaustion | Rate limit: 5 scans/hour per IP, 10 scans/day per email, use Redis for distributed rate limiting later |

---

## UX Pitfalls

Common user experience mistakes in this domain.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| No scan progress visibility | User doesn't know if scan is working, abandons page after 30s, unclear how long to wait | Show scan stages: "Fetching site... Checking SSL... Running Nuclei... Generating report..." |
| PDF generation blocks scan results | User waits 10s extra for PDF before seeing results, unnecessary latency | Show results immediately, generate PDF in background, add "Download PDF" button when ready |
| Generic error messages | "Scan failed" with no context, user doesn't know if it's their fault or a bug | Specific errors: "Site unreachable - check URL", "Scan timeout - site too slow", "Rate limit - try again in 1h" |
| No email notification when paid scan completes | User paid $49, starts scan, closes tab, forgets, never gets results | Send email with PDF attachment + results link when complete (already planned) |
| Scan results overwhelming | 50+ findings with no prioritization, user doesn't know where to start | Group by severity (Critical → High → Medium → Low), collapse Medium/Low by default, show "Fix these 3 first" |
| Remediation code not framework-specific | Generic "add CSP header" advice, user doesn't know where to put it in Next.js | Detect framework, show Next.js-specific fix: "Add to next.config.js: headers: [...]" |

---

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **PostgreSQL backups:** Container has persistent volume, but no automated backups — verify daily pg_dump cron job exists and test restore
- [ ] **SSL certificate renewal:** Certbot installed and cert works, but renewal might fail — verify `certbot renew --dry-run` succeeds
- [ ] **Docker log rotation:** Containers running fine, but logs grow unbounded — verify `logging.options.max-size` set in docker-compose.yml
- [ ] **Firewall rules:** UFW enabled and shows blocked ports, but Docker bypasses it — verify `nmap` from external machine shows only 80/443 open
- [ ] **Memory limits:** Containers start successfully, but no memory limits set — verify `docker stats` shows memory limits for all containers
- [ ] **Restart policies:** Containers restart after crashes, but no restart limits — verify systemd/docker-compose has `on-failure` + `StartLimitBurst`
- [ ] **Environment variables:** App works in dev with .env file, but systemd doesn't load it — verify systemd EnvironmentFile directive exists
- [ ] **Health checks:** Containers show "running" status, but app is actually crashed in restart loop — verify `docker ps` shows "healthy" status
- [ ] **Disk space monitoring:** Droplet has 50GB, feels like plenty, but Docker images consume 20GB — verify `df -h` and set up alerts at 80%
- [ ] **Connection pool sizing:** Database accepts connections now, but will exhaust pool at scale — verify `max_connections` calculated based on RAM

---

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Docker socket compromised | **HIGH** — assume full host compromise | 1. Take droplet snapshot. 2. Shut down droplet. 3. Analyze snapshot for persistence. 4. Provision new droplet. 5. Deploy from clean source. 6. Rotate all secrets. |
| PostgreSQL data loss (no backups) | **HIGH** — data unrecoverable | 1. Accept data loss. 2. Notify affected users. 3. Implement daily pg_dump backups. 4. Test restore monthly. |
| SSL certificate expired | **LOW** — fixable in <1h | 1. Check certbot logs for failure reason. 2. Fix Nginx config (web root, IPv6). 3. Run `certbot renew --force-renewal`. 4. Test renewal: `certbot renew --dry-run`. |
| OOM killer murdering containers | **MEDIUM** — requires droplet resize | 1. Add 2GB swap immediately: `fallocate -l 2G /swapfile && swapon /swapfile`. 2. Reduce scan concurrency to 3. 3. Resize to 4GB droplet. 4. Set memory limits. |
| Disk full (logs or images) | **LOW** — fixable in <30min | 1. Stop containers: `docker-compose stop`. 2. Prune system: `docker system prune -af --volumes`. 3. Delete old logs: `journalctl --vacuum-time=7d`. 4. Restart. 5. Configure log rotation. |
| UFW bypass exposing database | **MEDIUM** — requires config change + verify no breach | 1. Stop PostgreSQL immediately. 2. Change docker-compose to `127.0.0.1:5432:5432`. 3. Restart containers. 4. Check PostgreSQL logs for unauthorized access. 5. Rotate DB password if suspicious IPs found. |
| Systemd restart loop | **LOW** — fixable in <15min | 1. Stop service: `systemctl stop trustedge-backend`. 2. Check logs: `journalctl -u trustedge-backend -n 100`. 3. Fix missing env var or config. 4. Test manually: `docker-compose up` and verify starts. 5. Restart service. |
| Nginx reverse proxy broken | **LOW** — fixable in <15min | 1. Test backend directly: `curl http://127.0.0.1:3000/health`. 2. If works, fix Nginx proxy_pass address. 3. Test Nginx config: `nginx -t`. 4. Reload Nginx: `systemctl reload nginx`. |

---

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Docker socket security | Phase 1 (Infrastructure Setup) | Nuclei runs as native binary OR socket proxy configured, no raw socket mount |
| UFW firewall bypass | Phase 2 (Docker Compose Config) | `nmap <droplet-ip>` from external machine shows only 80/443 open |
| PostgreSQL data loss | Phase 2 (Docker Compose Config) + Phase 4 (Backups) | `docker volume ls` shows named volume, `crontab -l` shows pg_dump job, test restore successful |
| Memory exhaustion (OOM) | Phase 1 (Droplet Sizing) + Phase 2 (Memory Limits) | `docker stats` shows memory limits, no OOM entries in `dmesg`, 4GB+ droplet |
| Disk space exhaustion | Phase 2 (Log Rotation) + Phase 4 (Monitoring) | All services have max-size/max-file in docker-compose.yml, cron prunes weekly, alerts at 80% |
| SSL renewal failure | Phase 3 (SSL Setup) | `certbot renew --dry-run` succeeds, cron job configured, expiry monitoring active |
| Systemd restart loops | Phase 3 (Systemd Services) | Service uses EnvironmentFile, `on-failure` policy, StartLimitBurst=3, manual test successful |
| Nginx proxy connection refused | Phase 2 (Port Bindings) + Phase 3 (Nginx Config) | Ports bound to 127.0.0.1, Nginx proxies to 127.0.0.1:3000/3001, manual curl test successful |
| Connection pool exhaustion | Phase 2 (PostgreSQL Config) | `max_connections` sized for workload, PgBouncer configured if >100 concurrent requests |
| Secrets in plain text .env | Phase 2 (Secrets Management) | .env file permissions = 600, file not in git, never echoed in logs |

---

## Sources

### Docker Socket Security
- [OWASP Docker Security Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Docker_Security_Cheat_Sheet.html)
- [Why Exposing Docker Socket is Bad - Quarkslab](https://blog.quarkslab.com/why-is-exposing-the-docker-socket-a-really-bad-idea.html)
- [Docker Socket Security Guide - Medium](https://medium.com/@instatunnel/docker-socket-security-a-critical-vulnerability-guide-76f4137a68c5)
- [Docker Security Documentation](https://docs.docker.com/engine/security/)

### UFW Firewall Bypass
- [chaifeng/ufw-docker GitHub](https://github.com/chaifeng/ufw-docker)
- [Docker UFW Bypass Deep Dive](https://blogs.srikanthkarthi.tech/blog/docker-ufw-firewall-bypass)
- [Docker + UFW Security Risk](https://lukasrotermund.de/posts/docker-and-ufw_when-convenience-turns-into-a-security-risk/)
- [Docker Network Packet Filtering Docs](https://docs.docker.com/engine/network/packet-filtering-firewalls/)

### PostgreSQL Data Loss & Backups
- [PostgreSQL Docker Backup/Restore Guide](https://simplebackups.com/blog/docker-postgres-backup-restore-guide-with-examples)
- [Docker Volumes for Persistent Data](https://oneuptime.com/blog/post/2026-02-02-docker-volumes-persistent-data/view)
- [PostgreSQL Docker Persistence Guide](https://oneuptime.com/blog/post/2026-01-17-postgresql-docker-persistence/view)
- [Prevent Data Loss in Postgres Containers - DEV](https://dev.to/ndohjapan/how-to-prevent-data-loss-when-a-postgres-container-is-killed-or-shut-down-p8d)

### Memory Management & OOM
- [Docker Resource Constraints Docs](https://docs.docker.com/engine/containers/resource_constraints/)
- [OOMKilled Kubernetes Troubleshooting](https://komodor.com/learn/how-to-fix-oomkilled-exit-code-137/)
- [Managing RAM and OOM Killer on VPS](https://www.dchost.com/blog/en/managing-ram-swap-and-the-oom-killer-on-vps-servers/)
- [Docker Restart Policies Guide](https://oneuptime.com/blog/post/2026-01-16-docker-restart-policies/view)

### Disk Space & Log Management
- [Docker Disk Usage Cleanup Guide](https://oneuptime.com/blog/post/2026-01-06-docker-disk-usage-cleanup/view)
- [Managing Docker Logs to Prevent Overflow - DEV](https://dev.to/tejastn10/managing-docker-logs-and-preventing-log-overflows-on-servers-2o3p)
- [Docker Log Rotation - Red Hat](https://access.redhat.com/solutions/2334181)
- [Doku - Docker Disk Usage Dashboard](https://docker-disk.space/)

### SSL Certificate Renewal
- [Common Certbot Errors - Webdock](https://webdock.io/en/docs/webdock-control-panel/ssl-certificate-guides/common-certbot-errors)
- [Free SSL for Nginx with Let's Encrypt 2026](https://www.getpagespeed.com/server-setup/nginx/nginx-ssl-certificate-letsencrypt)
- [Let's Encrypt Community: Certificate Renewal Failures](https://community.letsencrypt.org/t/failed-certificate-renewal-nginx/204507)

### Systemd & Container Restarts
- [Solving Docker Container Restart Loops](https://mindfulchase.com/explore/troubleshooting-tips/devops-tools/solving-docker-container-restart-loops-in-production-environments.html)
- [Docker Restart Policies Guide](https://oneuptime.com/blog/post/2026-01-16-docker-restart-policies/view)
- [Docker Container Restart Loops Troubleshooting 2026](https://copyprogramming.com/howto/how-to-fix-a-restarting-docker-container)
- [Start Containers Automatically - Docker Docs](https://docs.docker.com/engine/containers/start-containers-automatically/)

### Nginx Reverse Proxy
- [Nginx Reverse Proxy Connection Refused - DigitalOcean](https://www.digitalocean.com/community/questions/nginx-reverse-proxy-connection-refused)
- [Nginx Reverse Proxy Complete Guide 2026](https://www.getpagespeed.com/server-setup/nginx/nginx-reverse-proxy)
- [Connection Refused Issues - Docker Forums](https://forums.docker.com/t/nginx-error-connect-failed-111-connection-refused-while-connecting-to-upstream/125697)

### Secrets Management
- [Docker Secrets Documentation](https://docs.docker.com/engine/swarm/secrets/)
- [4 Ways to Securely Store Secrets in Docker](https://blog.gitguardian.com/how-to-handle-secrets-in-docker/)
- [Docker Secrets Management Guide](https://oneuptime.com/blog/post/2026-01-30-docker-secrets-management/view)
- [Docker Secrets Security - Wiz](https://www.wiz.io/academy/container-security/docker-secrets)

### Connection Pool Management
- [PostgreSQL Connection Pool Exhaustion](https://www.c-sharpcorner.com/article/postgresql-connection-pool-exhaustion-lessons-from-a-production-outage/)
- [How to Fix Connection Pool Exhausted Errors](https://oneuptime.com/blog/post/2026-01-24-connection-pool-exhausted-errors/view)
- [PgBouncer Connection Pooling Guide](https://oneuptime.com/blog/post/2026-01-26-pgbouncer-connection-pooling/view)

---

*Pitfalls research for: TrustEdge Audit DigitalOcean Single-Droplet Deployment*
*Researched: 2026-02-06*
