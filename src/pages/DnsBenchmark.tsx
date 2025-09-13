import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import DnsCard from "../components/DnsCard";
import Stepper from "../components/Stepper";
import ProgressBar from "../components/ProgressBar";
import { DnsBenchmarkParams, DnsTestResult } from "../types";

export default function DnsBenchmark() {
  const [domain, setDomain] = useState("google.com");
  const [samples, setSamples] = useState(5);
  const [timeout, setTimeout] = useState(10);
  const [loading, setLoading] = useState(false);
  const [results, setResults] = useState<DnsTestResult[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  const usable = useMemo(() => (results || []).filter(r => r.query_successful && r.success_percent > 0), [results]);

  const sortedUsable = useMemo(() => {
    return [...usable].sort((a, b) => (
      Number(a.latency_avg_ms ?? a.avg_time ?? a.resolution_time_ms ?? Infinity) -
      Number(b.latency_avg_ms ?? b.avg_time ?? b.resolution_time_ms ?? Infinity)
    ));
  }, [usable]);

  async function run() {
    setLoading(true); setError(null); setResults(null);
    try {
      const params: DnsBenchmarkParams = {
        domainOrIp: domain.trim(),
        samples: Math.max(1, samples),
        timeoutSecs: Math.max(1, timeout),
      };
      const res = await invoke<DnsTestResult[]>("run_dns_benchmark", params);
      setResults(res);
    } catch (e: any) {
      setError(String(e?.message ?? e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="flex-1 p-4">
      <div className="glass rounded-xl p-4 mb-4">
        <div className="flex flex-wrap items-center gap-3">
          <input
            className="input"
            placeholder="Domain or IP (e.g. example.com)"
            value={domain}
            onChange={e => setDomain(e.target.value)}
          />
        </div>
        <div className="mt-4 grid grid-cols-3 items-center">
          <div className="flex items-center gap-3">
            <span className="text-sm text-[var(--muted)]">Samples</span>
            <Stepper value={samples} onChange={setSamples} min={1} aria-label="Samples" />
          </div>
          <div className="flex justify-center">
            <button className="btn" onClick={run} disabled={loading}>
              {loading ? "Running…" : "Benchmark DNS Servers"}
            </button>
          </div>
          <div className="flex items-center gap-3 justify-end">
            <span className="text-sm text-[var(--muted)]">Timeout (s)</span>
            <Stepper value={timeout} onChange={setTimeout} min={1} aria-label="Timeout seconds" />
          </div>
        </div>
      </div>

      {error && <div className="card border-red-500/50 text-red-300">{error}</div>}

      {loading && (
        <div className="card">
          <ProgressBar />
        </div>
      )}

      {!loading && results && (
        <div className="space-y-4">
          {/* Factors explanation */}
          <div className="card">
            <div className="text-sm text-[var(--muted)] space-y-1">
              <div><span className="font-medium text-white">Latency:</span> average DNS response time - lower is better.</div>
              <div><span className="font-medium text-white">Jitter:</span> variability of response times - lower is steadier.</div>
              <div><span className="font-medium text-white">Success rate:</span> percentage of successful queries - higher is better.</div>
              <div><span className="font-medium text-white">DNSSEC:</span> cryptographic validation support - prefer validated.</div>
            </div>
          </div>

          {/* Results list (usable) */}
          <section className="space-y-3">
            {sortedUsable.map((r) => <DnsCard key={r.server_address} r={r} />)}
            {sortedUsable.length === 0 && (
              <div className="text-sm text-[var(--muted)]">No usable results.</div>
            )}
          </section>
        </div>
      )}
    </div>
  );
}

