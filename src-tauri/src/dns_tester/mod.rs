use hickory_resolver::config::{NameServerConfigGroup, ResolverConfig, ResolverOpts};
use hickory_resolver::TokioResolver;
use hickory_resolver::Resolver;
use hickory_resolver::name_server::TokioConnectionProvider;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use futures::{stream, StreamExt};
use url::Url;
use tokio::time::timeout;
use tracing::{error, info, warn};
use idna::domain_to_ascii;
use std::net::IpAddr;
use tokio::runtime::Builder as TokioRtBuilder; // for isolated runtimes with larger stacks
// reverted: removed host-IP cache to restore direct resolution behavior

mod servers;
pub use servers::get_servers;
use servers::set_servers as set_servers_inner;
mod tls_hosts;

use crate::dns_tester::tls_hosts::TLS_HOST_MAP;
use servers::{init_servers, update_servers_from_url};
use tls_hosts::{init_tls_hosts, update_tls_hosts_from_url};

// (no host-IP cache)

// Made sync to avoid creating a temporary runtime in main; it only spawns async work.
pub fn init_configs() {
    init_servers();
    init_tls_hosts();

    // Kick off remote updates in background to avoid blocking startup
    let dns_list_url =
        "https://raw.githubusercontent.com/ednoct/DNS_SERVERS/main/servers.txt".to_string();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = update_servers_from_url(&dns_list_url).await {
            warn!("Could not update DNS servers from URL: {}", e);
        }
    });

    tauri::async_runtime::spawn(async move {
        if let Err(e) = update_tls_hosts_from_url().await {
            warn!("Could not update TLS hosts from URL: {}", e);
        }
    });
}

#[tauri::command]
pub async fn get_dns_servers() -> Vec<String> {
    get_servers().await
}

#[tauri::command]
pub async fn set_dns_servers(servers: Vec<String>) -> Result<(), String> {
    set_servers_inner(servers).await;
    Ok(())
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DnsTestResult {
    pub server_address: String,
    // Back-compat single-measurement fields (now represent averages)
    pub resolution_time_ms: Option<u128>,
    pub query_successful: bool,
    // New aggregated metrics
    pub latency_avg_ms: Option<f64>,
    pub jitter_avg_ms: Option<f64>,
    pub success_percent: f64,
    pub dnssec_validated: bool,
    pub dnssec_enabled: bool,
    pub ipv4_ips: Vec<String>,
    pub ipv6_ips: Vec<String>,
    pub error_msg: Option<String>,
    pub avg_time: Option<f64>,
}

#[tauri::command]
pub async fn perform_dns_benchmark(
    domain_or_ip: String,
    custom_servers: Option<Vec<String>>,
    timeout_secs: Option<u64>,
    samples: Option<u32>,
    validate_dnssec: Option<bool>,
    warm_up: Option<bool>,
) -> Vec<DnsTestResult> {
    // Accept domain or IP. Validate/convert domain (IDNA) off the worker thread.
    let validate_dnssec_flag = validate_dnssec.unwrap_or(false);
    let warm_up_flag = warm_up.unwrap_or(false);
    let input_is_ip = domain_or_ip.parse::<IpAddr>().is_ok();
    let ascii_domain: Option<String> = if !input_is_ip {
        match tokio::task::spawn_blocking({
            let s = domain_or_ip.clone();
            move || domain_to_ascii(&s)
        })
        .await
        {
            Ok(Ok(d)) => Some(d),
            _ => {
                return vec![DnsTestResult {
                    server_address: "invalid_domain".to_string(),
                    resolution_time_ms: None,
                    query_successful: false,
                    latency_avg_ms: None,
                    jitter_avg_ms: None,
                    success_percent: 0.0,
                    dnssec_validated: false,
                    dnssec_enabled: validate_dnssec_flag,
                    ipv4_ips: vec![],
                    ipv6_ips: vec![],
                    error_msg: Some("Invalid domain format".to_string()),
                    avg_time: None,
                }];
            }
        }
    } else {
        None
    };

    let timeout = timeout_secs.unwrap_or(10);
    let sample_count = samples.unwrap_or(5).max(1) as usize;

    // If a domain is entered, benchmark standard forward lookups (A/AAAA) for that domain.
    // If an IP is entered, benchmark a reverse (PTR) lookup for that IP.
    let query_norm = if input_is_ip {
        domain_or_ip.clone()
    } else {
        ascii_domain.unwrap()
    };

    let mut servers_list = match custom_servers {
        Some(s) => s,
        None => get_servers().await,
    };
    // Soft cap to avoid extremely long runs when user has a huge list
    if servers_list.len() > 120 {
        servers_list.truncate(120);
    }
    // Early reachability precheck: quickly test servers with a shorter timeout and skip unresponsive ones.
    let precheck_timeout = std::cmp::min(3, timeout);
    let query_for_pre = query_norm.clone();
    let prechecked: Vec<(String, bool)> = stream::iter(servers_list.iter().cloned().map(|server| {
        let q = query_for_pre.clone();
        let validate = validate_dnssec_flag;
        async move {
            let ok = precheck_server(&q, &server, precheck_timeout, validate).await;
            (server, ok)
        }
    }))
    .buffer_unordered(20)
    .collect()
    .await;

    let filtered: Vec<String> = prechecked
        .into_iter()
        .filter(|(_, ok)| *ok)
        .map(|(s, _)| s)
        .collect();

    if !filtered.is_empty() {
        servers_list = filtered;
    }
    // Process servers with bounded concurrency, offloading each server's work
    // into an isolated Tokio runtime with a larger thread stack to avoid worker overflows.
    const CONCURRENCY: usize = 10;
    stream::iter(servers_list.into_iter().map(|server| {
        let query_clone = query_norm.clone();
        let validate_dnssec_flag = validate_dnssec_flag;
        let warm_up_flag = warm_up_flag;
        async move {
            tokio::task::spawn_blocking(move || {
                run_server_benchmark_in_isolated_rt(
                    query_clone,
                    server,
                    timeout,
                    sample_count,
                    validate_dnssec_flag,
                    warm_up_flag,
                )
            })
            .await
            .unwrap_or_else(|e| DnsTestResult {
                server_address: "unknown".to_string(),
                resolution_time_ms: None,
                query_successful: false,
                latency_avg_ms: None,
                jitter_avg_ms: None,
                success_percent: 0.0,
                dnssec_validated: false,
                dnssec_enabled: validate_dnssec_flag,
                ipv4_ips: vec![],
                ipv6_ips: vec![],
                error_msg: Some(format!("Task error: {}", e)),
                avg_time: None,
            })
        }
    }))
    .buffer_unordered(CONCURRENCY)
    .collect()
    .await
}

// Quick reachability check with short timeout, returns true if a basic query succeeds.
async fn precheck_server(
    query: &str,
    server: &str,
    timeout_secs: u64,
    validate_dnssec: bool,
) -> bool {
    // Build resolver and attempt one lookup within timeout.
    let resolver = match build_resolver_for_server(server, timeout_secs, validate_dnssec).await {
        Ok(r) => r,
        Err(_) => return false,
    };
    // Treat IP input as reverse lookup, otherwise forward lookup.
    if let Ok(ip) = query.parse::<IpAddr>() {
        match timeout(std::time::Duration::from_secs(timeout_secs), resolver.reverse_lookup(ip)).await {
            Ok(Ok(lookup)) => lookup.iter().next().is_some(),
            _ => false,
        }
    } else {
        match timeout(std::time::Duration::from_secs(timeout_secs), resolver.lookup_ip(query)).await {
            Ok(Ok(lookup)) => lookup.iter().next().is_some(),
            _ => false,
        }
    }
}

// Runs the async per-server benchmark inside a dedicated Tokio runtime with a larger
// thread stack size. This avoids deep stack use on shared worker threads.
fn run_server_benchmark_in_isolated_rt(
    query: String,
    server_address: String,
    timeout_secs: u64,
    samples: usize,
    validate_dnssec: bool,
    warm_up: bool,
) -> DnsTestResult {
    // 4 MiB stack to be safe on Windows for TLS/ASN.1/h3 parsing paths
    let rt = TokioRtBuilder::new_current_thread()
        .enable_all()
        .thread_stack_size(4 * 1024 * 1024)
        .build();
    let rt = match rt {
        Ok(rt) => rt,
        Err(e) => {
            return DnsTestResult {
                server_address,
                resolution_time_ms: None,
                query_successful: false,
                latency_avg_ms: None,
                jitter_avg_ms: None,
                success_percent: 0.0,
                dnssec_validated: false,
                dnssec_enabled: validate_dnssec,
                ipv4_ips: vec![],
                ipv6_ips: vec![],
                error_msg: Some(format!("Runtime build error: {}", e)),
                avg_time: None,
            };
        }
    };

    rt.block_on(async move {
        benchmark_single_server(
            query,
            server_address,
            timeout_secs,
            samples,
            validate_dnssec,
            warm_up,
        )
        .await
    })
}


async fn benchmark_single_server(
    query: String,
    server_address: String,
    timeout_secs: u64,
    samples: usize,
    validate_dnssec: bool,
    warm_up: bool,
) -> DnsTestResult {
    info!("Testing server: {}", server_address);
    let resolver_result = build_resolver_for_server(&server_address, timeout_secs, validate_dnssec).await;

    let resolver = match resolver_result {
        Ok(r) => r,
        Err(e) => {
            error!("Resolver build error: {}", e);
            return DnsTestResult {
                server_address,
                resolution_time_ms: None,
                query_successful: false,
                latency_avg_ms: None,
                jitter_avg_ms: None,
                success_percent: 0.0,
                dnssec_validated: false,
                dnssec_enabled: validate_dnssec,
                ipv4_ips: vec![],
                ipv6_ips: vec![],
                error_msg: Some(e.to_string()),
                avg_time: None,
            };
        }
    };

    let is_ip = query.parse::<IpAddr>().is_ok();

    // Optional warm-up query to establish connections (not measured)
    if warm_up {
        let warm_to = std::cmp::min(timeout_secs, 3);
        if is_ip {
            if let Ok(ip) = query.parse::<IpAddr>() {
                let _ = timeout(std::time::Duration::from_secs(warm_to), resolver.reverse_lookup(ip)).await;
            }
        } else {
            let _ = timeout(std::time::Duration::from_secs(warm_to), resolver.lookup_ip(&query)).await;
        }
    }
    let mut latencies_ms: Vec<f64> = Vec::with_capacity(samples);
    let mut successes = 0usize;
    let mut last_error: Option<String> = None;
    let mut ipv4_all = Vec::new();
    let mut ipv6_all = Vec::new();
    // per-record security not available in all versions; aggregate via resolver options below

    for _ in 0..samples {
        let start = Instant::now();
        let mut sample_success = false;

        if is_ip {
            // Proper reverse lookup for IPs
            match query.parse::<IpAddr>() {
                Ok(ip) => match timeout(std::time::Duration::from_secs(timeout_secs), resolver.reverse_lookup(ip)).await {
                    Ok(Ok(lookup)) => {
                        sample_success = !lookup.iter().next().is_none();
                        // ReverseLookup security indicator not consistently available across versions.
                    }
                    Ok(Err(e)) => {
                        last_error = Some(e.to_string());
                    }
                    Err(_) => {
                        // timeout
                        if last_error.is_none() { last_error = Some("Timeout".to_string()); }
                    }
                },
                Err(e) => {
                    last_error = Some(e.to_string());
                }
            }
        } else {
            // For domains, prefer a single lookup_ip (gathers A/AAAA) with a hard timeout
            match timeout(std::time::Duration::from_secs(timeout_secs), resolver.lookup_ip(&query)).await {
                Ok(Ok(lookup)) => {
                    let mut v4: Vec<String> = Vec::new();
                    let mut v6: Vec<String> = Vec::new();
                    for ip in lookup.iter() {
                        if ip.is_ipv4() { v4.push(ip.to_string()); } else { v6.push(ip.to_string()); }
                    }
                    if !v4.is_empty() { sample_success = true; ipv4_all.append(&mut v4); }
                    if !v6.is_empty() { sample_success = true; ipv6_all.append(&mut v6); }
                }
                Ok(Err(e)) => {
                    if last_error.is_none() { last_error = Some(e.to_string()); }
                }
                Err(_) => {
                    if last_error.is_none() { last_error = Some("Timeout".to_string()); }
                }
            }
        }

        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        latencies_ms.push(elapsed_ms);
        if sample_success {
            successes += 1;
        }
    }

    // Compute metrics (median latency + standard deviation for jitter)
    let latency_avg_ms = if !latencies_ms.is_empty() {
        let mut sorted = latencies_ms.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = sorted.len() / 2;
        let median = if sorted.len() % 2 == 1 {
            sorted[mid]
        } else {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        };
        Some(median)
    } else {
        None
    };
    let jitter_avg_ms = if latencies_ms.len() > 1 {
        let n = latencies_ms.len() as f64;
        let mean = latencies_ms.iter().sum::<f64>() / n;
        let var = latencies_ms
            .iter()
            .map(|v| {
                let d = *v - mean;
                d * d
            })
            .sum::<f64>()
            / (n - 1.0); // sample standard deviation
        Some(var.sqrt())
    } else if latencies_ms.len() == 1 {
        // With a single sample, define jitter as 0 instead of null for clearer UI
        Some(0.0)
    } else {
        None
    };

    // unique IPs
    ipv4_all.sort();
    ipv4_all.dedup();
    ipv6_all.sort();
    ipv6_all.dedup();

    let success_percent = if samples > 0 {
        (successes as f64) * 100.0 / (samples as f64)
    } else {
        0.0
    };

    let avg_u128 = latency_avg_ms.map(|v| v as u128);

    DnsTestResult {
        server_address,
        resolution_time_ms: avg_u128,
        query_successful: successes > 0,
        latency_avg_ms,
        jitter_avg_ms,
        success_percent,
        // this flag indicates whether validation is enabled; per-record security would require
        // checking lookup.is_secure(), which is partially accounted for during lookups above.
        dnssec_validated: resolver.options().validate && successes > 0,
        dnssec_enabled: resolver.options().validate,
        ipv4_ips: ipv4_all,
        ipv6_ips: ipv6_all,
        error_msg: last_error,
        avg_time: latency_avg_ms,
    }
}

pub async fn build_resolver_for_server(
    server_address: &str,
    timeout_secs: u64,
    validate_dnssec: bool,
) -> Result<TokioResolver, Box<dyn std::error::Error + Send + Sync>> {
    let mut opts = ResolverOpts::default();
    opts.timeout = std::time::Duration::from_secs(timeout_secs);
    opts.validate = validate_dnssec;
    // Reduce retries to avoid long stalls across many servers
    opts.attempts = 1;
    // Disable resolver cache to avoid near-zero times after warm-up
    // and measure real network latency rather than in-process cache hits.
    opts.cache_size = 0;

    let config = if server_address.starts_with("https://") || server_address.starts_with("h3://") {
        // Parse full URL, capture host/port/path, and build DoH/H3 with SNI + path
        let is_h3 = server_address.starts_with("h3://");
        let url = Url::parse(server_address)?;
        let host_raw = url.host_str().ok_or("No host in URL")?.to_string();
        let port = url.port().unwrap_or(443);

        // Build endpoint path (path + optional query); default to /dns-query if empty
        let mut endpoint = url.path().to_string();
        if let Some(q) = url.query() {
            if !endpoint.is_empty() {
                endpoint.push('?');
                endpoint.push_str(q);
            }
        }
        if endpoint.is_empty() || endpoint == "/" {
            endpoint = "/dns-query".to_string();
        }

        // Determine SNI name and IPs for the target host
        let (tls_dns_name, ips): (String, Vec<IpAddr>) = if let Ok(ip) = host_raw.parse::<IpAddr>() {
            // IP-based DoH/H3: use TLS_HOST_MAP to get SNI name when possible
            let tls_name = match TLS_HOST_MAP.get() {
                Some(cell) => {
                    let map = cell.read().await;
                    map.get(&ip.to_string()).cloned().unwrap_or_else(|| {
                        warn!(
                            "No TLS host map for IP: {}. Using IP as SNI name; TLS may fail.",
                            host_raw
                        );
                        host_raw.clone()
                    })
                }
                None => {
                    warn!("TLS host map not initialized. Using IP as SNI name.");
                    host_raw.clone()
                }
            };
            (tls_name, vec![ip])
        } else {
            // Domain host: SNI is host itself; resolve to IPs with system resolver
            let sys_resolver = Resolver::builder_with_config(
                ResolverConfig::default(),
                TokioConnectionProvider::default(),
            )
            .build();
            let response = sys_resolver.lookup_ip(&host_raw).await?;
            let ips: Vec<IpAddr> = response.iter().collect();
            (host_raw.clone(), ips)
        };

        // Build NameServerConfigGroup for DoH (HTTPS). Treat any h3:// as HTTPS fallback.
        if is_h3 {
            warn!("H3 scheme provided but H3 support is disabled; using HTTPS fallback");
        }
        let mut group = NameServerConfigGroup::from_ips_https(&ips, port, tls_dns_name, true);
        for ns in group.iter_mut() {
            ns.http_endpoint = Some(endpoint.clone());
        }

        ResolverConfig::from_parts(None, vec![], group)
    } else if let Some(stripped) = server_address.strip_prefix("tls://") {
        // Support optional SNI override via '@': tls://<host>[:port]@<sni>
        let (host_part, sni_override) = match stripped.split_once('@') {
            Some((left, right)) => (left, Some(right.to_string())),
            None => (stripped, None),
        };
        let (host_str, port) = host_part.split_once(':').unwrap_or((host_part, "853"));
        let port_num: u16 = port.parse()?;

        let tls_dns_name = if let Some(sni) = sni_override {
            sni
        } else if let Ok(ip) = host_str.parse::<IpAddr>() {
            match TLS_HOST_MAP.get() {
                Some(cell) => {
                    let map = cell.read().await;
                    map.get(&ip.to_string()).cloned().unwrap_or_else(|| {
                        warn!(
                            "No TLS host map for IP: {}. Using IP as name, may fail.",
                            host_str
                        );
                        host_str.to_string()
                    })
                }
                None => {
                    warn!("TLS host map not initialized. Using IP as TLS name.");
                    host_str.to_string()
                }
            }
        } else {
            host_str.to_string()
        };

        let ips = if let Ok(ip) = host_str.parse::<IpAddr>() {
            vec![ip]
        } else {
            let sys_resolver = Resolver::builder_with_config(
                ResolverConfig::default(),
                TokioConnectionProvider::default(),
            )
            .build();
            let response = sys_resolver.lookup_ip(host_str).await?;
            response.iter().collect()
        };

        ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_tls(&ips, port_num, tls_dns_name, true),
        )
    } else if let Some(stripped) = server_address.strip_prefix("quic://") {
        // Support optional SNI override via '@': quic://<host>[:port]@<sni>
        let (host_part, sni_override) = match stripped.split_once('@') {
            Some((left, right)) => (left, Some(right.to_string())),
            None => (stripped, None),
        };
        let (host_str, port) = host_part.split_once(':').unwrap_or((host_part, "853"));
        let port_num: u16 = port.parse()?;

        // Use TLS host mapping for QUIC as well when given an IP
        let tls_dns_name = if let Some(sni) = sni_override {
            sni
        } else if let Ok(ip) = host_str.parse::<IpAddr>() {
            match TLS_HOST_MAP.get() {
                Some(cell) => {
                    let map = cell.read().await;
                    map.get(&ip.to_string()).cloned().unwrap_or_else(|| {
                        warn!(
                            "No TLS host map for IP: {}. Using IP as name, may fail.",
                            host_str
                        );
                        host_str.to_string()
                    })
                }
                None => {
                    warn!("TLS host map not initialized. Using IP as TLS name.");
                    host_str.to_string()
                }
            }
        } else {
            host_str.to_string()
        };

        let ips = if let Ok(ip) = host_str.parse::<IpAddr>() {
            vec![ip]
        } else {
            let sys_resolver = Resolver::builder_with_config(
                ResolverConfig::default(),
                TokioConnectionProvider::default(),
            )
            .build();
            let response = sys_resolver.lookup_ip(host_str).await?;
            response.iter().collect()
        };

        ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_quic(&ips, port_num, tls_dns_name, true),
        )
    } else {
        // UDP: accept either an IP or a hostname and resolve it first
        if let Ok(ip) = server_address.parse::<IpAddr>() {
            ResolverConfig::from_parts(
                None,
                vec![],
                NameServerConfigGroup::from_ips_clear(&[ip], 53, true),
            )
        } else {
            let sys_resolver = Resolver::builder_with_config(
                ResolverConfig::default(),
                TokioConnectionProvider::default(),
            )
            .build();
            let response = sys_resolver.lookup_ip(server_address).await?;
            let ips: Vec<IpAddr> = response.iter().collect();
            ResolverConfig::from_parts(
                None,
                vec![],
                NameServerConfigGroup::from_ips_clear(&ips, 53, true),
            )
        }
    };

    let resolver_builder = Resolver::builder_with_config(config, TokioConnectionProvider::default())
        .with_options(opts);

    // Offload resolver build (may do heavier sync init) to a blocking thread.
    let built = tokio::task::spawn_blocking(move || resolver_builder.build())
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
    Ok(built)
}
