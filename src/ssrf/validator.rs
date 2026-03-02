use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use tokio::net::lookup_host;
use url::Url;

#[derive(Debug)]
pub enum SsrfError {
    InvalidUrl,
    PrivateIp,
    LoopbackIp,
    LinkLocalIp,
    CloudMetadata,
    DnsResolutionFailed,
    BlockedScheme,
}

impl fmt::Display for SsrfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SsrfError::InvalidUrl => write!(f, "Invalid URL format"),
            SsrfError::PrivateIp => write!(f, "Private IP address not allowed"),
            SsrfError::LoopbackIp => write!(f, "Loopback IP address not allowed"),
            SsrfError::LinkLocalIp => write!(f, "Link-local IP address not allowed"),
            SsrfError::CloudMetadata => write!(f, "Cloud metadata IP address not allowed"),
            SsrfError::DnsResolutionFailed => write!(f, "DNS resolution failed"),
            SsrfError::BlockedScheme => write!(f, "Only http and https schemes are allowed"),
        }
    }
}

impl std::error::Error for SsrfError {}

/// Validate a URL for SSRF protection by checking scheme and resolving DNS to block private IPs
pub async fn validate_scan_target(url: &str) -> Result<String, SsrfError> {
    // Parse the URL
    let parsed = Url::parse(url).map_err(|_| SsrfError::InvalidUrl)?;

    // Check scheme - only http/https allowed
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(SsrfError::BlockedScheme);
    }

    // Get host
    let host = parsed.host_str().ok_or(SsrfError::InvalidUrl)?;

    // If the host is an IP literal, check it directly
    if let Ok(ip) = host.parse::<IpAddr>() {
        check_ip_blocked(&ip)?;
        return Ok(url.to_string());
    }

    // For hostnames, resolve DNS and check all resolved IPs
    let addr = format!(
        "{}:{}",
        host,
        parsed
            .port()
            .unwrap_or(if scheme == "https" { 443 } else { 80 })
    );
    let addrs = lookup_host(&addr)
        .await
        .map_err(|_| SsrfError::DnsResolutionFailed)?;

    // Check all resolved IPs
    for socket_addr in addrs {
        check_ip_blocked(&socket_addr.ip())?;
    }

    Ok(url.to_string())
}

fn check_ip_blocked(ip: &IpAddr) -> Result<(), SsrfError> {
    match ip {
        IpAddr::V4(ipv4) => check_ipv4_blocked(ipv4),
        IpAddr::V6(ipv6) => check_ipv6_blocked(ipv6),
    }
}

fn check_ipv4_blocked(ip: &Ipv4Addr) -> Result<(), SsrfError> {
    // Check for cloud metadata IPs explicitly first (before more general checks)
    // AWS/GCP metadata: 169.254.169.254
    if ip == &Ipv4Addr::new(169, 254, 169, 254) {
        return Err(SsrfError::CloudMetadata);
    }

    // Alibaba Cloud metadata: 100.100.100.200
    if ip == &Ipv4Addr::new(100, 100, 100, 200) {
        return Err(SsrfError::CloudMetadata);
    }

    // Check for loopback (127.0.0.0/8)
    if ip.is_loopback() {
        return Err(SsrfError::LoopbackIp);
    }

    // Check for private IP ranges
    if ip.is_private() {
        return Err(SsrfError::PrivateIp);
    }

    // Check for link-local (169.254.0.0/16)
    if ip.is_link_local() {
        return Err(SsrfError::LinkLocalIp);
    }

    // Check for unspecified (0.0.0.0)
    if ip.is_unspecified() {
        return Err(SsrfError::PrivateIp);
    }

    // Check for multicast
    if ip.is_multicast() {
        return Err(SsrfError::PrivateIp);
    }

    Ok(())
}

fn check_ipv6_blocked(ip: &Ipv6Addr) -> Result<(), SsrfError> {
    // Check for loopback (::1)
    if ip.is_loopback() {
        return Err(SsrfError::LoopbackIp);
    }

    // Check for unspecified (::)
    if ip.is_unspecified() {
        return Err(SsrfError::PrivateIp);
    }

    // Check for multicast
    if ip.is_multicast() {
        return Err(SsrfError::PrivateIp);
    }

    // Check for AWS EC2 metadata (fd00:ec2::254)
    let segments = ip.segments();
    if segments[0] == 0xfd00 && segments[1] == 0xec2 && segments[7] == 0x254 {
        return Err(SsrfError::CloudMetadata);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_loopback_blocked() {
        let result = validate_scan_target("http://127.0.0.1").await;
        assert!(matches!(result, Err(SsrfError::LoopbackIp)));
    }

    #[tokio::test]
    async fn test_private_ip_blocked() {
        let result = validate_scan_target("http://192.168.1.1").await;
        assert!(matches!(result, Err(SsrfError::PrivateIp)));

        let result = validate_scan_target("http://10.0.0.1").await;
        assert!(matches!(result, Err(SsrfError::PrivateIp)));
    }

    #[tokio::test]
    async fn test_cloud_metadata_blocked() {
        let result = validate_scan_target("http://169.254.169.254").await;
        assert!(matches!(result, Err(SsrfError::CloudMetadata)));
    }

    #[tokio::test]
    async fn test_blocked_scheme() {
        let result = validate_scan_target("ftp://example.com").await;
        assert!(matches!(result, Err(SsrfError::BlockedScheme)));
    }

    #[tokio::test]
    async fn test_invalid_url() {
        let result = validate_scan_target("not-a-url").await;
        assert!(matches!(result, Err(SsrfError::InvalidUrl)));
    }

    #[tokio::test]
    async fn test_public_url_allowed() {
        // This test uses a real domain that should resolve to public IPs
        let result = validate_scan_target("https://example.com").await;
        // Should succeed if DNS resolution works and example.com resolves to public IPs
        assert!(result.is_ok() || matches!(result, Err(SsrfError::DnsResolutionFailed)));
    }
}
