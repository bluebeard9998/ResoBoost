use once_cell::sync::OnceCell;
use std::collections::HashMap;
use tokio::sync::RwLock;
use reqwest::Client;

// Maps IP -> TLS DNS name (hostname for SNI)
pub static TLS_HOST_MAP: OnceCell<RwLock<HashMap<String, String>>> = OnceCell::new();

pub fn init_tls_hosts() {
    let default = DEFAULT_TLS_HOST_MAP
        .iter()
        .map(|(ip, host)| (ip.to_string(), host.to_string()))
        .collect::<HashMap<_, _>>();
    TLS_HOST_MAP.set(RwLock::new(default)).ok();
}

pub async fn update_tls_hosts_from_url() -> Result<(), reqwest::Error> {
    // Remote file format: "host ip" per line.
    let url = "https://raw.githubusercontent.com/bluebeard9998/DNS_SERVERS/main/tls-host-map.txt";
    let client = Client::new();
    let response = client.get(url).send().await?.text().await?;

    let mut new_map = HashMap::new();
    for line in response.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2 {
            // store as IP -> host
            let host = parts[0].to_string();
            let ip = parts[1].to_string();
            new_map.insert(ip, host);
        }
    }

    if let Some(map) = TLS_HOST_MAP.get() {
        let mut locked = map.write().await;
        *locked = new_map;
    }

    Ok(())
}

// Default pairs in IP -> host form
pub const DEFAULT_TLS_HOST_MAP: &[(&str, &str)] = &[
    ("1.1.1.1", "cloudflare-dns.com"),
    ("1.0.0.1", "cloudflare-dns.com"),
    ("8.8.8.8", "dns.google"),
    ("8.8.4.4", "dns.google"),
    ("9.9.9.9", "dns.quad9.net"),
    ("149.112.112.112", "dns.quad9.net"),
    ("8.26.56.26", "cdns.comodo.com"),
    ("137.66.7.89", "max.rethinkdns.com"),
];
