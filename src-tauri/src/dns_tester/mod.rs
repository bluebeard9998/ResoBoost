use hickory_resolver::config::{NameServerConfigGroup, ResolverConfig, ResolverOpts};
use hickory_resolver::TokioResolver;
use hickory_resolver::Resolver;
use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_resolver::proto::rr::RecordType;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use futures::future::join_all;
use url::Url;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};
use idna::domain_to_ascii;
use std::net::IpAddr;
use std::sync::Arc as StdArc;

mod servers;
pub use servers::get_servers;
mod tls_hosts;

use crate::dns_tester::tls_hosts::TLS_HOST_MAP;
use servers::{init_servers, update_servers_from_url};
use tls_hosts::{init_tls_hosts, update_tls_hosts_from_url};

pub async fn init_configs() {
    init_servers();
    init_tls_hosts();

    let dns_list_url =
        "https://raw.githubusercontent.com/bluebeard9998/DNS_SERVERS/main/servers.txt";
    if let Err(e) = update_servers_from_url(dns_list_url).await {
        warn!("Could not update DNS servers from URL: {}", e);
    }
    if let Err(e) = update_tls_hosts_from_url().await {
        warn!("Could not update TLS hosts from URL: {}", e);
    }
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
) -> Vec<DnsTestResult> {
    // Accept domain or IP. Validate domain if not an IP.
    let is_ip = domain_or_ip.parse::<IpAddr>().is_ok();
    if !is_ip && domain_to_ascii(&domain_or_ip).is_err() {
        return vec![DnsTestResult {
            server_address: "invalid_domain".to_string(),
            resolution_time_ms: None,
            query_successful: false,
            latency_avg_ms: None,
            jitter_avg_ms: None,
            success_percent: 0.0,
            dnssec_validated: false,
            ipv4_ips: vec![],
            ipv6_ips: vec![],
            error_msg: Some("Invalid domain format".to_string()),
            avg_time: None,
        }];
    }

    let timeout = timeout_secs.unwrap_or(10);
    let sample_count = samples.unwrap_or(5).max(1) as usize;

    // Normalize domain using IDNA (punycode) so non-ASCII domains resolve correctly
    let query_norm = if is_ip {
        domain_or_ip.clone()
    } else {
        // safe unwrap; validated above
        domain_to_ascii(&domain_or_ip).unwrap()
    };

    let servers_list = match custom_servers {
        Some(s) => s,
        None => get_servers().await,
    };
    let mut tasks = Vec::new();
    let sem = StdArc::new(Semaphore::new(10));

    for server in servers_list.iter() {
        let permit = sem.clone().acquire_owned().await.unwrap();
        let query_clone = query_norm.clone();
        let server_clone = server.clone();
        tasks.push(tokio::spawn(async move {
            let res = benchmark_single_server(query_clone, server_clone, timeout, sample_count).await;
            drop(permit);
            res
        }));
    }

    let results: Vec<DnsTestResult> = join_all(tasks)
        .await
        .into_iter()
        .map(|res| {
            res.unwrap_or_else(|e| DnsTestResult {
                server_address: "unknown".to_string(),
                resolution_time_ms: None,
                query_successful: false,
                latency_avg_ms: None,
                jitter_avg_ms: None,
                success_percent: 0.0,
                dnssec_validated: false,
                ipv4_ips: vec![],
                ipv6_ips: vec![],
                error_msg: Some(format!("Task error: {}", e)),
                avg_time: None,
            })
        })
        .collect();

    results
}

async fn benchmark_single_server(
    query: String,
    server_address: String,
    timeout_secs: u64,
    samples: usize,
) -> DnsTestResult {
    info!("Testing server: {}", server_address);
    let resolver_result = build_resolver_for_server(&server_address, timeout_secs).await;

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
                ipv4_ips: vec![],
                ipv6_ips: vec![],
                error_msg: Some(e.to_string()),
                avg_time: None,
            };
        }
    };

    let is_ip = query.parse::<IpAddr>().is_ok();
    let mut latencies_ms: Vec<f64> = Vec::with_capacity(samples);
    let mut successes = 0usize;
    let mut last_error: Option<String> = None;
    let mut ipv4_all = Vec::new();
    let mut ipv6_all = Vec::new();
    let mut dnssec_any_secure = false;

    for _ in 0..samples {
        let start = Instant::now();
        let mut sample_success = false;

        if is_ip {
            // Proper reverse lookup for IPs
            match query.parse::<IpAddr>() {
                Ok(ip) => match resolver.reverse_lookup(ip).await {
                    Ok(lookup) => {
                        sample_success = !lookup.iter().next().is_none();
                        #[allow(unused_variables)]
                        let _ = {
                            // ReverseLookup security indicator is not guaranteed in all versions,
                            // but if available, prefer it
                            #[cfg(any())]
                            if lookup.is_secure() { dnssec_any_secure = true; }
                        };
                    }
                    Err(e) => {
                        last_error = Some(e.to_string());
                    }
                },
                Err(e) => {
                    last_error = Some(e.to_string());
                }
            }
        } else {
            // For domains, query A and AAAA
            match resolver.lookup(&query, RecordType::A).await {
                Ok(lookup) => {
                    let mut v4: Vec<String> = lookup
                        .record_iter()
                        .filter_map(|r| r.data().as_a().map(|ip| ip.to_string()))
                        .collect();
                    if !v4.is_empty() {
                        sample_success = true;
                        ipv4_all.append(&mut v4);
                    }
                    // Count as DNSSEC validated if this lookup was secure (when feature enabled)
                    #[allow(unused_variables)]
                    let _ = {
                        #[cfg(any())]
                        if lookup.is_secure() { dnssec_any_secure = true; }
                    };
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                }
            }

            match resolver.lookup(&query, RecordType::AAAA).await {
                Ok(lookup) => {
                    let mut v6: Vec<String> = lookup
                        .record_iter()
                        .filter_map(|r| r.data().as_aaaa().map(|ip| ip.to_string()))
                        .collect();
                    if !v6.is_empty() {
                        sample_success = true;
                        ipv6_all.append(&mut v6);
                    }
                    #[allow(unused_variables)]
                    let _ = {
                        #[cfg(any())]
                        if lookup.is_secure() { dnssec_any_secure = true; }
                    };
                }
                Err(e) => {
                    // preserve first error if any
                    if last_error.is_none() {
                        last_error = Some(e.to_string());
                    }
                }
            }
        }

        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
        latencies_ms.push(elapsed_ms);
        if sample_success {
            successes += 1;
        }
    }

    // Compute metrics
    let latency_avg_ms = if !latencies_ms.is_empty() {
        Some(latencies_ms.iter().sum::<f64>() / latencies_ms.len() as f64)
    } else {
        None
    };
    let jitter_avg_ms = if latencies_ms.len() > 1 {
        let mean = latency_avg_ms.unwrap_or(0.0);
        let mad = latencies_ms
            .iter()
            .map(|v| (v - mean).abs())
            .sum::<f64>()
            / latencies_ms.len() as f64;
        Some(mad)
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
        ipv4_ips: ipv4_all,
        ipv6_ips: ipv6_all,
        error_msg: last_error,
        avg_time: latency_avg_ms,
    }
}

pub async fn build_resolver_for_server(
    server_address: &str,
    timeout_secs: u64,
) -> Result<TokioResolver, Box<dyn std::error::Error + Send + Sync>> {
    let mut opts = ResolverOpts::default();
    opts.timeout = std::time::Duration::from_secs(timeout_secs);
    opts.validate = true;
    opts.attempts = 3;
    opts.cache_size = 512;

    let config = if server_address.starts_with("https://") || server_address.starts_with("h3://") {
        let is_h3 = server_address.starts_with("h3://");
        // Treat h3 servers as DoH endpoints; strip the scheme for URL parsing
        let url_str = if is_h3 { &server_address[5..] } else { server_address };
        let url = Url::parse(url_str)?;
        let host = url.host_str().ok_or("No host in URL")?.to_string();
        let port = url.port().unwrap_or(443);
        let sys_resolver = Resolver::builder_with_config(
            ResolverConfig::default(),
            TokioConnectionProvider::default(),
        )
        .build();
        let response = sys_resolver.lookup_ip(&host).await?;
        let ips: Vec<IpAddr> = response.iter().collect();
        if is_h3 {
            warn!("H3 treated as HTTPS; ensure h3 feature enabled if needed.");
        }
        ResolverConfig::from_parts(
            None,
            vec![],
            NameServerConfigGroup::from_ips_https(&ips, port, host, true),
        )
    } else if let Some(stripped) = server_address.strip_prefix("tls://") {
        let (host_str, port) = stripped.split_once(':').unwrap_or((stripped, "853"));
        let port_num: u16 = port.parse()?;

        let tls_dns_name = if let Ok(ip) = host_str.parse::<IpAddr>() {
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
        let (host_str, port) = stripped.split_once(':').unwrap_or((stripped, "853"));
        let port_num: u16 = port.parse()?;

        let tls_dns_name = host_str.to_string();

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

    let resolver = Resolver::builder_with_config(config, TokioConnectionProvider::default())
        .with_options(opts)
        .build();
    Ok(resolver)
}
