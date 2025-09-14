use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;

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

        // DNS-over-TLS (DoT)
        "tls://cloudflare-dns.com:853",
        "tls://dns.google:853",
        "tls://dns.quad9.net:853",
        "tls://dns.adguard.com:853",
        "tls://max.rethinkdns.com:853",
        "tls://dns.alidns.com:853",
        
        // DNS-over-HTTPS (DoH)
        "https://cloudflare-dns.com/dns-query",
        "https://security.cloudflare-dns.com/dns-query",
        "https://dns.google/dns-query",
        "https://dns.quad9.net/dns-query",
        "https://doh.dns.sb/dns-query",
        "https://doh.cleanbrowsing.org/doh/family-filter/",
        "https://dns.adguard-dns.com/dns-query",
        "https://dns-family.adguard-dns.com/dns-query",
        "https://dns-unfiltered.adguard-dns.com/dns-query",
        "https://doh.opendns.com/dns-query",
        "https://freedns.controld.com/x-goodbyeads",
        "https://blitz.ahadns.com/1:17",
        "https://doh.blahdns.com/dns-query",
        "https://doh.uncensoreddns.org/dns-query",
        "https://dns.fdn.org/dns-query",
        "https://doh.dns.watch/dns-query",
        "https://sky.rethinkdns.com/dns-query",
        "https://dns.alidns.com/dns-query",
        "https://doh.libredns.gr/dns-query",
        "https://doh.tiar.app/dns-query",
        "https://dns.aa.net.uk/dns-query",
        "https://dnsforge.de/dns-query",
        
        // DNS-over-QUIC (DoQ)
        "quic://dns.adguard.com",
        "quic://family.adguard-dns.com",
        "quic://unfiltered.adguard-dns.com",
        "quic://x-goodbyeads.freedns.controld.com",
];

pub static DNS_SERVERS: Lazy<Arc<RwLock<Vec<String>>>> = Lazy::new(|| {
    Arc::new(RwLock::new(
        DEFAULT_DNS_SERVERS.iter().map(|s| s.to_string()).collect(),
    ))
});

pub fn init_servers() {
    Lazy::force(&DNS_SERVERS);
}

pub async fn get_servers() -> Vec<String> {
    DNS_SERVERS.read().await.clone()
}

pub async fn set_servers(new_servers: Vec<String>) {
    let filtered: Vec<String> = new_servers
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let mut servers_guard = DNS_SERVERS.write().await;
    *servers_guard = filtered;
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
        let mut servers_guard = DNS_SERVERS.write().await;
        *servers_guard = new_servers;
        println!(
            "Successfully updated DNS servers. Total count: {}",
            servers_guard.len()
        );
    } else {
        println!("No new servers found in the response.");
    }

    Ok(())
}
