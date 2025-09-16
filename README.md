# ResoBoost – DNS & Download Benchmark App
<div align="center">
    <img src="public/Logo-github.png" alt="ResoBoost Logo" width="231" height="231">

  [![Release](https://img.shields.io/github/v/release/ednoct/ResoBoost)](https://github.com/ednoct/ResoBoost/releases)
  ![License MIT](https://img.shields.io/badge/license-MIT-blue.svg)  
  [![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue.svg)](https://github.com/ednoct/ResoBoost/releases)
  [![Tauri](https://img.shields.io/badge/Tauri-2.0-blue.svg)](https://tauri.app/)
  [![React](https://img.shields.io/badge/React-19.1-blue.svg)](https://reactjs.org/)
  [![Rust](https://img.shields.io/badge/Rust-1.89.0-blue.svg)](https://www.rust-lang.org/)
  
  </div>
  
## ℹ️ Overview
**ResoBoost** is a cross-platform network performance tool built with **Tauri** (Rust back-end) and a **React + TypeScript + Vite + Tailwind CSS** front-end. It benchmarks DNS resolvers and download speeds, producing metrics such as average latency, jitter, success rate, DNSSEC validation, and per-server bandwidth.

---

## ❓ Why ResoBoost

Most “speed test” websites only check your current resolver or CDN connection. **ResoBoost** lets you compare many DNS providers and measure download throughput via those servers on your own machine:

- **DNS metrics beyond ping:** median latency, jitter, success rate, DNSSEC validation, and IP responses per resolver.
- **Real download bandwidth:** resolves a host via each DNS server, streams data from a URL, and reports bytes read, duration, and Mbps.
- **Protocol diversity:** work with classic UDP/TCP resolvers and modern DoH/DoT/DoQ lists; load defaults or your own lists.
- **Customisable and private:** edit/import resolver lists via the UI; export results to CSV.

---

## 🎯 Features

- 🚀 **Cross-platform desktop app** via Tauri (small footprint, fast startup).
- 📊 **DNS benchmarking:** latency (median), jitter, success rate, DNSSEC status, and resolved IPs.
- 📥 **Download speed testing:** per-DNS bandwidth measurement for any HTTP/HTTPS URL.
- 📝 **CSV export:** save DNS and download results with one click.
- 🧠 **Dynamic resolver lists:** load default UDP/TCP/DoH/DoT/DoQ or regional lists, import from URL, or edit manually.
- ⚙️ **Configurable tests:** domain/IP, sample count, timeouts, DNSSEC, warm-up, per-DNS duration, etc.
- 💻 **Modern tech stack:** React 19, TypeScript, Vite 7, Tailwind CSS 4, Tauri 2; Rust libs include `hickory-resolver`, `tokio`, `reqwest`, `serde`.

---

## 📷 Screenshots

| DNS Benchmark | Download Speed | Dns Servers |
|-------------|----------------|-----------------|
| ![DNS Benchmark](Screenshots/DnsBenchmark.png) | ![Download Speed](Screenshots/SpeedTest.png) | ![Dns Servers](Screenshots/ServersModal.png) |

---

## 🛠️ Architecture

- The UI uses @tauri-apps/api to invoke commands in the Rust layer.

- The DNS tester initialises resolver and TLS host lists, can refresh them from remote files, then runs concurrent lookups.

- The speed tester resolves the target host per DNS server and performs streaming downloads to calculate bandwidth.

---

## 🚀 Quickstart (TTFS ≤ 5 minutes)
### Prerequisites
- Rust (stable)
- Node.js ≥ 16 (v18+ recommended)
- bun (bundled) or your preferred package manager
> Windows: install MSVC build tools.
> Linux: ensure Tauri deps like libwebkit2gtk and openssl dev headers are installed (see Tauri docs).
> macOS: Xcode command-line tools.
### Clone & Install
   ```bash
# clone this repo
git clone https://github.com/ednoct/ResoBoost.git
cd ResoBoost

# install JS dependencies
bun install
   ```
### Run in Development
   ```bash
# start the UI + Rust back-end in dev mode
bun run tauri dev
   ```
## ⚙️ Configuration
Most options are set via the UI, but the following environment variable can be useful during development:
Custom DNS server lists are fetched from the DNS_SERVERS repository on start and can be refreshed or edited via Server Lists → Edit. You can load default sets for UDP/TCP, DoH, DoT, DoQ, or region-specific lists, or paste your own.

## 🖇️ Integrations & Compatibility
- OS: Windows, macOS, Linux, android (Tauri)
- Front-end: React 19, TypeScript, Vite 7, Tailwind CSS 4
- Rust: Tauri 2, hickory-resolver, reqwest, tokio, serde
- Extensibility: strongly-typed result objects; easy to integrate new pages/components or expose additional Tauri commands.

## 🤔 Troubleshooting / FAQ
- “Network unavailable” errors → check firewall/VPN; the app needs outbound DNS/HTTP.
- DNSSEC fails → not all resolvers support DNSSEC; try disabling it or use a DNSSEC-enabled resolver.
- Custom servers won’t save → one server per line; supported forms include:
     - `8.8.8.8` (UDP/TCP)
     - `tls://1.1.1.1@cloudflare-dns.com` (DoT)
     - `https://dns.quad9.net/dns-query` (DoH)
     - `quic://1.1.1.1:784@dns.cloudflare.com` (DoQ)
- Low download speed → increase per-DNS test duration, pick a closer mirror, or verify your network path.

## 💡 Roadmap
-  Pre-built installers and auto-update support
-  CLI mode for headless benchmarking / CI integration
-  Additional metrics (packet loss, jitter distributions, upstream tests)
- Tagging/favourites and better server list management
- Built-in charts and historical comparisons

## 🛡️ Security & Responsible Disclosure
If you discover a security vulnerability (for example in the DNS resolution logic or Tauri packaging), please do not open a public issue. Instead, email the maintainer (see GitHub profile) with details. We appreciate responsible disclosure and will respond quickly.
# ❤️ Support This Project

If this project has helped you, consider a crypto donation.  
Every bit keeps the lights on and the code humming.

## Crypto

| Coin | Address |
|---|---|
| **Bitcoin (BTC)** | `1HDsbDpY9dJXrrvLy3rJMm2BZd7zvF9Erw` |
| **Ethereum (ETH)** | `0x66792127017a21C89Df9D0Eab43FFA625171edc7` |
| **USDT (ERC-20 ETH)** | `0x66792127017a21C89Df9D0Eab43FFA625171edc7` |
| **USDT (TRC-20 TRX)** | `TSXAWSiTaB55ncW6ettK32LBEc859Emsv2` |
| **Tron (TRX)** | `TSXAWSiTaB55ncW6ettK32LBEc859Emsv2` |
| **TON (TON)** | `EQBif0KSURTfRhyFtuglbstHO1GS_A2Pb-HPu--vn4VSRwU1` |
| **Solana (SOL)** | `idRY4iwQQGZgzqKSzkmBGcsQjqDiy2ZcJyD1vg5TyVH` |
| **Litecoin (LTC)** | `LYVzCp7jGazsScq6Mie3omFvj8pJqjzwTB` |
| **Dogecoin (DOGE)** | `DFRVfM4AWYtaYTNg2TNfXLe2aHShnpL6CN` |

### Notes
- **No private keys—ever.** Commit only public receive addresses.
- Prefer **self-custody** wallets over exchanges.
- If you rotate addresses, update this file so supporters aren’t sending into the void.
- Optional: use an ENS name (e.g., `yourname.eth`) to make ETH donations friendlier.

## Thanks
Your support keeps development steady and true. 🙏

## 🤝 Acknowledgements
- [Tauri](https://tauri.app/), for providing a lightweight, secure application framework.  
- [`hickory-resolver`](https://crates.io/crates/hickory-resolver) and [`reqwest`](https://crates.io/crates/reqwest), for enabling async DNS and HTTP operations.  
- [Tailwind CSS](https://tailwindcss.com/) and [Vite](https://vitejs.dev/), for powering a modern, fast React front-end.  
- [`DNS_SERVERS` repository](https://github.com/ednoct/DNS_SERVERS), which supplies the resolver lists.  
