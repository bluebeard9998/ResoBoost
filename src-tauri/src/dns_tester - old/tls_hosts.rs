use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::Mutex;
use reqwest::Client;

pub static TLS_HOST_MAP: OnceCell<Mutex<HashMap<String, String>>> = OnceCell::new();

pub fn init_tls_hosts() {
    let default = DEFAULT_TLS_HOST_MAP
        .iter()
        .map(|(host, ip)| (host.to_string(), ip.to_string()))
        .collect::<HashMap<_, _>>();
    TLS_HOST_MAP.set(Mutex::new(default)).ok();
}

pub async fn update_tls_hosts_from_url() -> Result<(), reqwest::Error> {
    let url = "https://raw.githubusercontent.com/bluebeard9998/DNS_SERVERS/refs/heads/main/tls-host-map.txt";
    let client = Client::new();
    let response = client.get(url).send().await?.text().await?;

    let mut new_map = HashMap::new();
    for line in response.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() == 2 {
            new_map.insert(parts[0].to_string(), parts[1].to_string());
        }
    }

    if let Some(map) = TLS_HOST_MAP.get() {
        let mut locked = map.lock().unwrap();
        *locked = new_map;
    }

    Ok(())
}

pub const DEFAULT_TLS_HOST_MAP: &[(&str, &str)] = &[
    ("cloudflare-dns.com", "1.1.1.1"),
    ("cloudflare-dns.com", "1.0.0.1"),
    ("dns.google", "8.8.8.8"),
    ("dns.google", "8.8.4.4"),
    ("dns.quad9.net", "9.9.9.9"),
    ("dns.quad9.net", "149.112.112.112"),
    ("cdns.comodo.com", "8.26.56.26"),
    ("max.rethinkdns.com", "137.66.7.89"),

];
