// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod dns_tester;
mod speed_tester;

#[tauri::command]
async fn run_dns_benchmark(
    domain_or_ip: String,
    samples: Option<u32>,
    timeout_secs: Option<u64>,
    custom_servers: Option<Vec<String>>,
) -> Vec<dns_tester::DnsTestResult> {
    dns_tester::perform_dns_benchmark(domain_or_ip, custom_servers, timeout_secs, samples).await
}

fn main() {
    // Ensure default configs are initialized (servers + TLS host map)
    tauri::async_runtime::block_on(dns_tester::init_configs());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![run_dns_benchmark, speed_tester::perform_download_speed_test])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
