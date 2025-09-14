use crate::dns_tester::{build_resolver_for_server, get_servers};
use futures::{stream, StreamExt};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::time::Instant;
use tokio::time::timeout;
use tracing::info;
use url::Url;
use tokio::runtime::Builder as TokioRtBuilder; // isolated DNS resolve runtime with larger stack

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DownloadTestResult {
    pub server_address: String,
    pub resolved_ip: Option<String>,
    pub duration_ms: u128,
    pub bytes_read: u64,
    pub bandwidth_mbps: f64,
    pub query_successful: bool,
    pub http_status: Option<u16>,
    pub error_msg: Option<String>,
}

#[derive(Deserialize)]
pub struct DownloadSpeedArgs {
    pub url: String,
    #[serde(alias = "durationSecs")] 
    pub duration_secs: Option<u64>,
    #[serde(alias = "timeoutSecs")] 
    pub timeout_secs: Option<u64>,
    #[serde(alias = "customServers")] 
    pub custom_servers: Option<Vec<String>>,
}

#[tauri::command]
pub async fn perform_download_speed_test(args: DownloadSpeedArgs) -> Vec<DownloadTestResult> {
    let test_duration = args.duration_secs.unwrap_or(10); // default 10s
    let timeout = args.timeout_secs.unwrap_or(15).max(test_duration + 5);

    let parsed = match Url::parse(&args.url) {
        Ok(u) => u,
        Err(e) => {
            return vec![DownloadTestResult {
                server_address: "invalid_url".to_string(),
                resolved_ip: None,
                duration_ms: 0,
                bytes_read: 0,
                bandwidth_mbps: 0.0,
                query_successful: false,
                http_status: None,
                error_msg: Some(format!("Invalid URL: {}", e)),
            }]
        }
    };

    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return vec![DownloadTestResult {
            server_address: "unsupported_scheme".to_string(),
            resolved_ip: None,
            duration_ms: 0,
            bytes_read: 0,
            bandwidth_mbps: 0.0,
            query_successful: false,
            http_status: None,
            error_msg: Some("Only http and https are supported".to_string()),
        }];
    }

    let host = match parsed.host_str() {
        Some(h) => h.to_string(),
        None => {
            return vec![DownloadTestResult {
                server_address: "invalid_url".to_string(),
                resolved_ip: None,
                duration_ms: 0,
                bytes_read: 0,
                bandwidth_mbps: 0.0,
                query_successful: false,
                http_status: None,
                error_msg: Some("URL missing host".to_string()),
            }];
        }
    };
    let port = parsed.port().unwrap_or(if scheme == "https" { 443 } else { 80 });

    let mut servers_list = match args.custom_servers {
        Some(s) => s,
        None => get_servers().await,
    };
    // Avoid extremely long runs if the list is huge (join_all waits for all tasks)
    // Keep it conservative by default; can be made configurable from UI later.
    const MAX_SERVERS: usize = 40;
    if servers_list.len() > MAX_SERVERS {
        servers_list.truncate(MAX_SERVERS);
    }

    // Process servers with bounded concurrency without extra task spawning.
    const CONCURRENCY: usize = 6;
    stream::iter(servers_list.into_iter().map(|server| {
        let url_clone = args.url.clone();
        let host_clone = host.clone();
        async move { download_via_dns_server(&server, &host_clone, port, &url_clone, test_duration, timeout).await }
    }))
    .buffer_unordered(CONCURRENCY)
    .collect()
    .await
}

async fn download_via_dns_server(
    server_address: &str,
    host: &str,
    port: u16,
    url: &str,
    test_duration_secs: u64,
    timeout_secs: u64,
) -> DownloadTestResult {
    info!("Speed test via {} for {}", server_address, host);

    // Resolve A/AAAA in an isolated runtime with a larger thread stack to avoid worker overflow.
    let resolved_ip: Result<IpAddr, String> = tokio::task::spawn_blocking({
        let server = server_address.to_string();
        let host = host.to_string();
        move || resolve_ip_in_isolated_rt(server, host, timeout_secs)
    })
    .await
    .unwrap_or_else(|e| Err(format!("Resolve task error: {}", e)));

    let ip = match resolved_ip {
        Ok(ip) => ip,
        Err(msg) => {
            return DownloadTestResult {
                server_address: server_address.to_string(),
                resolved_ip: None,
                duration_ms: 0,
                bytes_read: 0,
                bandwidth_mbps: 0.0,
                query_successful: false,
                http_status: None,
                error_msg: Some(msg),
            };
        }
    };
    let socket = SocketAddr::new(ip, port);

    // Build reqwest client that connects to the chosen IP but uses host for SNI
    let client = match reqwest::Client::builder()
        .resolve(host, socket)
        .connect_timeout(std::time::Duration::from_secs(timeout_secs))
        .timeout(std::time::Duration::from_secs(timeout_secs))
        // Prefer HTTP/1.1 for streaming stability across diverse CDNs
        .http1_only()
        // Avoid env proxies interfering with direct IP connections
        .no_proxy()
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return DownloadTestResult {
                server_address: server_address.to_string(),
                resolved_ip: Some(ip.to_string()),
                duration_ms: 0,
                bytes_read: 0,
                bandwidth_mbps: 0.0,
                query_successful: false,
                http_status: None,
                error_msg: Some(format!("HTTP client build error: {}", e)),
            };
        }
    };

    let start = Instant::now();
    let mut total_bytes: u64 = 0;
    let mut status: Option<u16> = None;
    let mut last_err: Option<String> = None;

    // Single streaming GET; if it ends before time, we finish.
    match client.get(url).send().await {
        Ok(resp) => {
            status = Some(resp.status().as_u16());
            let mut stream = resp.bytes_stream();
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        total_bytes += bytes.len() as u64;
                    }
                    Err(e) => {
                        last_err = Some(format!("Read error: {}", e));
                        break;
                    }
                }
                if start.elapsed().as_secs() >= test_duration_secs {
                    break;
                }
            }
        }
        Err(e) => {
            last_err = Some(format!("Request error: {}", e));
        }
    }

    let elapsed_ms = start.elapsed().as_millis();
    let secs = (elapsed_ms as f64) / 1000.0;
    let mbps = if secs > 0.0 { (total_bytes as f64) * 8.0 / 1_000_000.0 / secs } else { 0.0 };

    DownloadTestResult {
        server_address: server_address.to_string(),
        resolved_ip: Some(ip.to_string()),
        duration_ms: elapsed_ms,
        bytes_read: total_bytes,
        bandwidth_mbps: mbps,
        query_successful: total_bytes > 0 && last_err.is_none(),
        http_status: status,
        error_msg: last_err,
    }
}

// Resolve a single IP for `host` using a resolver configured for `server_address`,
// executing in a dedicated runtime with larger thread stack.
fn resolve_ip_in_isolated_rt(server_address: String, host: String, timeout_secs: u64) -> Result<IpAddr, String> {
    let rt = TokioRtBuilder::new_current_thread()
        .enable_all()
        .thread_stack_size(4 * 1024 * 1024)
        .build()
        .map_err(|e| format!("Runtime build error: {}", e))?;

    rt.block_on(async move {
        let resolver = build_resolver_for_server(&server_address, timeout_secs, true)
            .await
            .map_err(|e| format!("Resolver error: {}", e))?;

        match timeout(std::time::Duration::from_secs(timeout_secs), resolver.lookup_ip(&host)).await {
            Ok(Ok(lookup)) => {
                // Prefer IPv4 for download sockets to avoid environments where IPv6 is present
                // in DNS but not actually reachable, which would yield 0 bytes read.
                let mut first: Option<IpAddr> = None;
                let mut v4_choice: Option<IpAddr> = None;
                for ip in lookup.iter() {
                    if first.is_none() { first = Some(ip); }
                    if ip.is_ipv4() { v4_choice = Some(ip); break; }
                }
                v4_choice.or(first).ok_or_else(|| "No A/AAAA records found".to_string())
            }
            Ok(Err(e)) => Err(format!("DNS resolve error: {}", e)),
            Err(_) => Err("DNS resolve timeout".to_string()),
        }
    })
}
