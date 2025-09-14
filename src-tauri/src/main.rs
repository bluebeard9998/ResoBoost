// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod dns_tester;
mod speed_tester;
use serde::Deserialize;

#[derive(Deserialize)]
struct DnsBenchmarkArgs {
    #[serde(alias = "domainOrIp")]
    domain_or_ip: String,
    samples: Option<u32>,
    #[serde(alias = "timeoutSecs")]
    timeout_secs: Option<u64>,
    #[serde(alias = "customServers")]
    custom_servers: Option<Vec<String>>,
    #[serde(alias = "validateDnssec")]
    validate_dnssec: Option<bool>,
    #[serde(alias = "warmUp")]
    warm_up: Option<bool>,
}

#[tauri::command]
async fn run_dns_benchmark(args: DnsBenchmarkArgs) -> Vec<dns_tester::DnsTestResult> {
    dns_tester::perform_dns_benchmark(
        args.domain_or_ip,
        args.custom_servers,
        args.timeout_secs,
        args.samples,
        args.validate_dnssec,
        args.warm_up,
    )
    .await
}

fn main() {
    // Increase minimum stack size for threads created by std where applicable (Windows safety net)
    #[cfg(target_os = "windows")]
    {
        std::env::set_var("RUST_MIN_STACK", (4 * 1024 * 1024).to_string());
    }

    // Ensure default configs are initialized (servers + TLS host map)
    // Avoid creating a temporary Tokio runtime here; init is sync and spawns its own async work.
    dns_tester::init_configs();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            run_dns_benchmark,
            speed_tester::perform_download_speed_test,
            dns_tester::get_dns_servers,
            dns_tester::set_dns_servers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

