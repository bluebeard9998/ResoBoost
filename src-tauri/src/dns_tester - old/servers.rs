use tokio::sync::{OnceCell, RwLock};
use std::sync::Arc;

const DEFAULT_DNS_SERVERS: &[&'static str] = &[
        // Standard DNS (UDP/53)
        "8.8.8.8",          // Google Public DNS (General Purpose)
        "8.8.4.4",          // Google Public DNS (General Purpose)
        "1.1.1.1",          // Cloudflare DNS (Unfiltered, Privacy-Focused)
        "1.0.0.1",          // Cloudflare DNS (Unfiltered, Privacy-Focused)
        "208.67.222.222", // OpenDNS Home (Phishing Protection)
        "208.67.220.220", // OpenDNS Home (Phishing Protection)
        "208.67.220.2",   // OpenDNS Sandbox (Unfiltered)
        "208.67.222.2",   // OpenDNS Sandbox (Unfiltered)
        "9.9.9.9",          // Quad9 (Malware Blocking, DNSSEC Validation)
        "149.112.112.112", // Quad9 (Malware Blocking, DNSSEC Validation)
        "9.9.9.11",         // Quad9 (Malware Blocking, DNSSEC Validation, ECS Enabled)
        "149.112.112.11", // Quad9 (Malware Blocking, DNSSEC Validation, ECS Enabled)
        "9.9.9.10",         // Quad9 (Unsecured - No Malware Blocking, No DNSSEC Validation)
        "149.112.112.10", // Quad9 (Unsecured - No Malware Blocking, No DNSSEC Validation)
        "94.140.14.14", // AdGuard DNS (Ads, Trackers, Malware, Phishing Blocking)
        "94.140.15.15", // AdGuard DNS (Ads, Trackers, Malware, Phishing Blocking)
        "94.140.14.140", // AdGuard DNS (Non-filtering)
        "94.140.14.141", // AdGuard DNS (Non-filtering)
        "77.88.8.8",        // Yandex DNS Basic (General Purpose)
        "77.88.8.1",        // Yandex DNS Basic (General Purpose)
        "77.88.8.88",   // Yandex DNS Safe (Protection from Dangerous Websites)
        "77.88.8.2",        // Yandex DNS Safe (Protection from Dangerous Websites)
        "185.228.168.9", // CleanBrowse Security Filter (Malware, Phishing, Spam Blocking)
        "185.228.169.9", // CleanBrowse Security Filter (Malware, Phishing, Spam Blocking)
        "76.76.2.0",        // Control D (General Purpose / Customizable)
        "76.76.10.0",   // Control D (General Purpose / Customizable)
        "138.197.140.189", // OpenNIC (Community-run, Neutral)
        "168.235.111.72", // OpenNIC (Community-run, Neutral)
        "76.76.19.19",  // Alternate DNS (Ad-blocking)
        "76.223.122.150", // Alternate DNS (Ad-blocking)
        "216.146.35.35", // Dyn (General Purpose)
        "216.146.36.36", // Dyn (General Purpose)
        "74.82.42.42",  // Hurricane Electric (General Purpose)
        "149.112.121.10", // CIRA Canadian Shield (Malware and Phishing Protection)
        "149.112.122.10", // CIRA Canadian Shield (Malware and Phishing Protection)
        "8.26.56.26",   // Comodo Secure DNS (Security, Fraudulent Website Protection)
        "8.20.247.20",  // Comodo Secure DNS (Security, Fraudulent Website Protection)
        "205.171.3.65", // CenturyLink (Level3) (General Purpose)
        "205.171.2.65", // CenturyLink (Level3) (General Purpose)
        "223.5.5.5",        // AliDNS
        "223.6.6.6",        // AliDNS
        "185.222.222.222", // DNS.SB
        "45.11.45.11",  // DNS.SB
        "119.29.29.29", // DNSPod
        "182.254.116.116", // DNSPod
        "194.242.2.2",  // Mullvad
        "194.242.2.4",  // Mullvad Base
        "45.90.28.0",   // NextDNS
        "45.90.30.0",   // NextDNS
        "146.112.41.2", // OpenBLD
        "146.112.41.102", // OpenBLD
        "193.110.81.9", // DNS0.EU
        "185.253.5.9",  // DNS0.EU
        "101.226.4.6",  // 360
        "180.163.224.54", // 360
        "185.95.218.42", // Digitale Gesellschaft
        "185.95.218.43", // Digitale Gesellschaft
        "158.64.1.29",  // Restena
        "203.180.164.45", // IIJ
        "203.180.166.45", // IIJ
        "116.202.176.26", // LibreDNS
        "147.135.76.183", // LibreDNS
        "130.59.31.248", // Switch
        "130.59.31.251", // Switch
        "146.255.56.98", // Foundation for Applied Privacy
        "91.239.100.100", // UncensoredDNS
        "89.233.43.71", // UncensoredDNS
        "104.21.83.62", // RethinkDNS
        "172.67.214.246", // RethinkDNS
        "217.218.155.155",
        "217.218.127.127",
        "80.191.40.41",
        "2.188.21.130",
        "2.188.21.131",
        "2.188.21.132",
        "2.189.44.44",
        "194.225.152.10",
        "217.219.157.2",
        "217.219.103.5",
        "217.219.132.88",
        "217.219.133.21",
        "217.219.72.194",
        "217.219.187.3",
        "217.218.234.221",
        "80.191.209.105",
        "80.191.233.17",
        "80.191.233.33",
        "85.15.1.14",
        "85.15.1.15",
        "37.156.29.27",
        "188.213.72.84",
        "188.213.72.85",
        "91.99.101.12",
        "91.99.96.158",
        "92.42.49.43",
        "185.15.1.100",
        "37.156.145.21",
        "37.156.145.229",
        "185.98.113.113",
        "185.98.113.141",
        "185.98.113.142",
        "185.98.114.114",
        "37.156.145.18",
        "194.36.174.161",
        "185.98.115.135",
        "172.19.190.190",
        "172.28.195.195",
        "46.224.1.220",
        "178.22.122.100",
        "185.51.200.2",
        "185.55.226.26",
        "185.55.225.25",
        "78.157.42.100",
        "78.157.42.101",
        "78.157.40.158",
        "78.157.40.157",
        "10.202.10.10",
        "10.202.10.11",
        "5.202.100.101",
        "5.202.100.100",
        "5.202.100.102",
        "5.202.100.99",
        "5.202.122.222",
        "185.97.117.187",
        "185.231.182.126",
        "185.113.59.253",
        "185.187.84.15",
        "194.225.73.141",
        "213.176.123.5",
        "185.51.200.10",
        "185.51.200.50",
        "185.51.200.6",
        "91.245.229.1",
        "91.245.229.2",
        "185.161.112.33",
        "185.161.112.34",
        "185.161.112.38",
        "194.225.62.80",
        "46.224.1.42",
        "185.143.235.253",
        "78.38.122.12",
        "81.163.3.1",
        "81.163.3.2",
        "31.47.37.35",
        "5.145.112.39",
        "5.145.112.38",
        "185.164.73.148",
        "95.80.184.184",
        "81.91.144.190",
        "80.75.5.100",
        "85.185.6.3",
        "85.185.67.235",
        "2.185.239.137",
        "185.53.143.3",
        "185.128.139.139",
        "185.128.139.128",
        "212.80.20.243",
        "212.80.20.244",
        "2.185.239.133",
        "85.185.85.6",
        "185.109.74.85",
        "2.185.239.134",
        "2.185.239.139",
        "2.185.239.136",
        "37.19.90.65",
        "37.19.90.62",
        "2.185.239.138",
        "93.115.231.100",
        "185.164.73.180",
        "217.219.250.200",
        "217.219.250.201",
        "217.219.250.202",
        "185.64.179.89",
        "194.60.210.66",
        "176.221.23.252",
        "89.144.144.144",
        "5.200.200.200",
        "185.186.242.161",
        "78.39.101.186",
        "185.229.29.214",
        "185.229.29.215",
        "185.23.131.73",
        "31.130.180.120",
        "31.47.37.92",
        "79.175.176.42",
        "78.38.23.216",
        "31.24.234.34",
        "31.24.234.35",
        "31.24.234.37",
        "94.139.190.190",
        "45.159.151.220",
        "82.99.202.164",
        "94.183.42.232",
        "188.158.158.158",
        "188.159.159.159",
        "185.20.163.2",
        "95.38.61.50",
        "2.188.166.22",
        "5.160.211.66",
        "77.238.109.196",
        "31.24.200.1",
        "31.24.200.2",
        "31.24.200.3",
        "31.24.200.4",
        "178.215.3.142",
        "78.38.117.206",
        "171.22.26.14",
        "185.8.173.236",
        "82.99.242.155",
        "194.225.125.12",
        "185.11.70.174",
        "185.83.197.154",
        "85.185.157.2",
        "dns.google.com",
        "cloudflare-dns.com",

        // DNS-over-TLS (DoT)
        "tls://cloudflare-dns.com:853",
        "tls://dns.google:853",
        "tls://dns.quad9.net:853",
        "tls://cloudflare-dns.com:853",
        "tls://dns.google:853",
        "tls://cdns.comodo.com:853",
        "tls://8.8.4.4:853",
        "tls://max.rethinkdns.com:853",

        // DNS-over-HTTPS (DoH)
        "https://cloudflare-dns.com/dns-query",
        "https://dns.google/dns-query",
        "https://dns.quad9.net/dns-query",
        "https://dns.google/resolve",
		"https://doh.dns.sb/dns-query",
        "https://doh.cleanbrowsing.org/doh/family-filter/",
        "https://dns.adguard.com/dns-query",
        "https://doh.opendns.com/dns-query",
        "https://doh.umbrella.com/dns-query",
        "https://dns.nextdns.io",
        "https://doh.digitalocean.com/dns-query",
        "https://dns.mozilla.org/dns-query",
        "https://doh.powerdns.org",
        "https://doh.surfshark.com",
        "https://dns4.nextdns.io/dns-query",
        "https://doh.blahdns.com/dns-query",
        "https://doh.uncensoreddns.org/dns-query",
        "https://dns.fdn.org/dns-query",
        "https://doh.neustar.biz/dns-query",
        "https://doh.dns.watch/dns-query",
        "https://sky.rethinkdns.com/1:EAACAA==",
        "https://max.rethinkdns.com/1:EAACAA==",
		"https://sky.rethinkdns.com/1:gAAAAQ==",
        "https://max.rethinkdns.com/1:gAAAAQ==",
		"https://blitz.ahadns.com/1:17",
		"https://freedns.controld.com/x-goodbyeads",
		"https://blitz.ahadns.com/1:17",
        "https://xmission-slc-1.edge.nextdns.io/dns-query",
        "https://ipv4-zepto-mci-1.edge.nextdns.io/dns-query",
        "https://dns.controld.com/",
        "https://170.176.145.150/",
        "https://zepto-sto-1.edge.nextdns.io",
        "https://nsc.torgues.net/dns-query",
        "https://jp-kix2.doh.sb/",
        "https://xtom-osa-1.edge.nextdns.io/dns-query",
        "https://dns.aa.net.uk/dns-query",
        "https://res-acst3.absolight.net/",
        "https://dns.melalandia.tk/dns-query",
        "https://dns.rafn.is/dns-query",
        "https://9.9.9.13/dns-query",
        "https://9.9.9.12/dns-query",
        "https://dns.adguard-dns.com/dns-query",
        "https://dns.nas-server.ru/dns-query",
        "https://sky.rethinkdns.com/dns-query",
        "https://8.8.8.8/dns-query",
        "https://9.9.9.9/dns-query",
        "https://94.140.14.14/dns-query",
        "https://94.140.15.15/dns-query",
        "https://223.5.5.5/dns-query",
        "https://223.6.6.6/dns-query",
        "https://1.1.1.1/dns-query",
        "https://1.0.0.1/dns-query",
        "https://120.53.53.53/dns-query",
        "https://8.8.4.4/dns-query",
        "https://yovbak.com/dns-query",
        "https://208.67.222.222/dns-query",
        "https://dns.kernel-error.de/dns-query",
        "https://security.cloudflare-dns.com/dns-query",
        "https://doh.dns4all.eu/dns-query",
        "https://8.26.56.26/dns-query",
		
        // DNS-over-HTTPS3 (h3)
        "h3://cloudflare-dns.com/dns-query",
        "h3://dns.google/dns-query",
        "h3://dns.alidns.com/dns-query",
		
        // DNS-over-QUIC (DoQ)
        "quic://dns.adguard.com",
        "quic://dns.google",
        "quic://dns.adguard-dns.com",
        "quic://family.adguard-dns.com",
        "quic://unfiltered.adguard-dns.com",
        "quic://dns.futuredns.me",
        "quic://doh.tiar.app",
        "quic://dandelionsprout.asuscomm.com:48582",
        "quic://x-goodbyeads.freedns.controld.com",
		"quic://dns.alidns.com",
];

pub async fn get_servers() -> Vec<String> {
let servers_cell = get_servers_cell().await;
let servers_guard = servers_cell.read().await;
servers_guard.clone()
}


pub async fn update_servers_from_url(url: &str) -> Result<(), reqwest::Error> {
println!("Updating DNS servers from: {}", url);
let response_text = reqwest::get(url).await?.text().await?;
let new_servers: Vec<String> = response_text
.lines()
.map(|line| line.trim().to_string())
.filter(|line| !line.is_empty())
.collect();


if !new_servers.is_empty() {
let servers_cell = get_servers_cell().await;
let mut servers_guard = servers_cell.write().await;
*servers_guard = new_servers;
println!("Successfully updated DNS servers. Total count: {}", servers_guard.len());
} else {
println!("No new servers found in the response.");
}


Ok(())
}


#[tokio::main]
async fn main() {
let dns_list_url = "https://raw.githubusercontent.com/bluebeard9998/DNS_SERVERS/refs/heads/main/servers.txt";


println!("--- Initial Servers ---");
let initial_servers = get_servers().await;
println!("Count: {}", initial_servers.len());
initial_servers.iter().take(5).for_each(|s| println!(" - {}", s));


println!("\n--- Updating from URL ---");
match update_servers_from_url(dns_list_url).await {
Ok(_) => {
println!("\n--- Updated Servers ---");
let updated_servers = get_servers().await;
println!("Count: {}", updated_servers.len());
updated_servers.iter().take(5).for_each(|s| println!(" - {}", s));
}
Err(e) => {
eprintln!("Failed to update servers: {}", e);
}
}
}