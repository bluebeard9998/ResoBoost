use hickory_resolver::config::{ResolverConfig, ResolverOpts, NameServerConfigGroup};
use hickory_resolver::TokioResolver;
use hickory_resolver::proto::rr::RecordType;
use serde::{Serialize, Deserialize};
use std::time::Instant;
use futures::future::join_all;
use url::Url;
use tokio::sync::{Semaphore, RwLock};
use tracing::{error, info, warn};
use idna::domain_to_ascii;
use std::sync::Arc as StdArc;
use std::net::IpAddr;
use std::collections::HashMap;
use once_cell::sync::Lazy;

mod servers;
mod tls_hosts;

use servers::{init_servers, update_servers_from_url};
use tls_hosts::{init_tls_hosts, update_tls_hosts_from_url};

pub async fn init_configs() {
    init_servers();
    init_tls_hosts();

    let _ = update_servers_from_url().await;
    let _ = update_tls_hosts_from_url().await;
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DnsTestResult {
    pub server_address: String,
    pub resolution_time_ms: Option<u128>,
    pub query_successful: bool,
    pub dnssec_validated: bool,
    pub ipv4_ips: Vec<String>,
    pub ipv6_ips: Vec<String>,
    pub error_msg: Option<String>,
    pub avg_time: Option<f64>
}

#[tauri::command]
pub async fn perform_dns_benchmark(domain: String, custom_servers: Option<Vec<String>>, timeout_secs: Option<u64>) -> Vec<DnsTestResult> {
    if domain_to_ascii(&domain).is_err() {
        return vec![DnsTestResult {
            server_address: "invalid_domain".to_string(),
            resolution_time_ms: None,
            query_successful: false,
            dnssec_validated: false,
            ipv4_ips: vec![],
            ipv6_ips: vec![],
            error_msg: Some("Invalid domain format".to_string()),
            avg_time: None
        }];
    }

    let timeout = timeout_secs.unwrap_or(10);

    let servers_list = custom_servers.unwrap_or_else(|| servers::DNS_SERVERS.iter().map(|s| s.to_string()).collect());
    let mut tasks = Vec::new();
    let sem = StdArc::new(Semaphore::new(10));

    for server in servers_list.iter() {
        let permit = sem.clone().acquire_owned().await.unwrap();
        let domain_clone = domain.clone();
        let server_clone = server.clone();
        tasks.push(tokio::spawn(async move {
            let res = test_single_server(domain_clone, server_clone, timeout).await;
            drop(permit);
            res
        }));
    }

    let results: Vec<DnsTestResult> = join_all(tasks)
        .await
        .into_iter()
        .map(|res| res.unwrap_or_else(|e| DnsTestResult {
            server_address: "unknown".to_string(),
            resolution_time_ms: None,
            query_successful: false,
            dnssec_validated: false,
            ipv4_ips: vec![],
            ipv6_ips: vec![],
            error_msg: Some(format!("Task error: {}", e)),
            avg_time: None
        }))
        .collect();

    results
}

async fn test_single_server(domain: String, server_address: String, timeout_secs: u64) -> DnsTestResult {
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
                dnssec_validated: false,
                ipv4_ips: vec![],
                ipv6_ips: vec![],
                error_msg: Some(e.to_string()),
                avg_time: None
            };
        }
    };
    
    let start_time = Instant::now();
    let a_response = resolver.lookup(&domain, RecordType::A).await;
    let aaaa_response = resolver.lookup(&domain, RecordType::AAAA).await;

    let mut ipv4 = vec![];
    let mut ipv6 = vec![]
    let mut dnssec_ok = false;
    let mut successful = false;
    let mut error_msg = None;

    match a_response {
        Ok(lookup) => {
            ipv4 = lookup.record_iter().filter_map(|r| r.data().as_a().map(|ip| ip.to_string())).collect();
            successful = !ipv4.is_empty();
            dnssec_ok = successful && resolver.options().validate;
        }
        Err(e) => {
            error_msg = Some(e.to_string());
        }
    }

    match aaaa_response {
        Ok(lookup) => {
            ipv6 = lookup.record_iter().filter_map(|r| r.data().as_aaaa().map(|ip| ip.to_string())).collect();
            successful = successful || !ipv6.is_empty();
            dnssec_ok = dnssec_ok || (successful && resolver.options().validate);
        }
        Err(e) => {
            if error_msg.is_none() {
                error_msg = Some(e.to_string());
            }
        }
    }

    let duration = start_time.elapsed();

    DnsTestResult {
        server_address,
        resolution_time_ms: Some(duration.as_millis()),
        query_successful: successful,
        dnssec_validated: dnssec_ok,
        ipv4_ips: ipv4,
        ipv6_ips: ipv6,
        error_msg,
        avg_time: Some(duration.as_millis() as f64)
    }
}

async fn build_resolver_for_server(server_address: &str, timeout_secs: u64) -> Result<TokioResolver, Box<dyn std::error::Error + Send + Sync>> {
    let mut opts = ResolverOpts::default();
    opts.timeout = std::time::Duration::from_secs(timeout_secs);
    opts.validate = true;
    opts.attempts = 3;
    opts.cache_size = 512;

    let config = if server_address.starts_with("https://") || server_address.starts_with("h3://") {
        let is_h3 = server_address.starts_with("h3://");
        let url_str = if is_h3 { &server_address[4..] } else { server_address };
        let url = Url::parse(url_str)?;
        let host = url.host_str().ok_or("No host in URL")?.to_string();
        let port = url.port().unwrap_or(443);
        let sys_resolver = TokioResolver::tokio(ResolverConfig::default(), ResolverOpts::default())?;
        let response = sys_resolver.lookup_ip(&host).await?;
        let ips: Vec<IpAddr> = response.iter().collect();
        if is_h3 {
            warn!("H3 treated as HTTPS; ensure h3 feature enabled if needed.");
        }
        ResolverConfig::from_parts(None, vec![], NameServerConfigGroup::from_ips_https(&ips, port, host, true))
    } else if let Some(stripped) = server_address.strip_prefix("tls://") {
        let (host_str, port) = stripped.split_once(':').unwrap_or((stripped, "853"));
        let port_num: u16 = port.parse()?;
        
        // --- MODIFIED: Use the global, dynamic map instead of a hardcoded one ---
        let tls_dns_name = if let Ok(ip) = host_str.parse::<IpAddr>() {
            let map = TLS_HOST_MAP.read().await;
            map.get(&ip.to_string()).map_or_else(
                || {
                    warn!("No TLS host map for IP: {}. Using IP as name, may fail.", host_str);
                    host_str.to_string()
                },
                |name| name.clone(),
            )
        } else {
            host_str.to_string()
        };
        
        let ips = if let Ok(ip) = host_str.parse::<IpAddr>() {
            vec![ip]
        } else {
            let sys_resolver = TokioResolver::tokio(ResolverConfig::default(), ResolverOpts::default())?;
            let response = sys_resolver.lookup_ip(host_str).await?;
            response.iter().collect()
        };

        ResolverConfig::from_parts(None, vec![], NameServerConfigGroup::from_ips_tls(&ips, port_num, tls_dns_name, true))
    } else if let Some(stripped) = server_address.strip_prefix("quic://") {
        let (host_str, port) = stripped.split_once(':').unwrap_or((stripped, "853"));
        let port_num: u16 = port.parse()?;
        
        let tls_dns_name = if let Ok(ip) = host_str.parse::<IpAddr>() {
            host_str.to_string()
        } else {
            host_str.to_string()
        };
        
        let ips = if let Ok(ip) = host_str.parse::<IpAddr>() {
            vec![ip]
        } else {
            let sys_resolver = TokioResolver::tokio(ResolverConfig::default(), ResolverOpts::default())?;
            let response = sys_resolver.lookup_ip(host_str).await?;
            response.iter().collect()
        };

        ResolverConfig::from_parts(None, vec![], NameServerConfigGroup::from_ips_quic(&ips, port_num, tls_dns_name, true))
    } else {
        let ip: IpAddr = server_address.parse()?;
        ResolverConfig::from_parts(None, vec![], NameServerConfigGroup::from_ips_clear(&[ip], 53, true))
    };

    Ok(TokioResolver::tokio(config, opts)?)
}

