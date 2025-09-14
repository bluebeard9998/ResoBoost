import { useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import DnsCard from "../components/DnsCard";
import Stepper from "../components/Stepper";
import ProgressBar from "../components/ProgressBar";
import { DnsBenchmarkParams, DnsTestResult } from "../types";
import { saveCsvAs, toSafeName, toDnsCsv } from "../utils/export";

export default function DnsBenchmark() {
  const [domain, setDomain] = useState("flutter.dev");
  const [samples, setSamples] = useState(3);
  const [timeout, setTimeout] = useState(11);
  const [dnssec, setDnssec] = useState(false);
  const [warmUp, setWarmUp] = useState(false);
  const [loading, setLoading] = useState(false);
  const [results, setResults] = useState<DnsTestResult[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const runCounter = useRef(0);
  const activeRunId = useRef<number | null>(null);

  const usable = useMemo(() => (results || []).filter(r => r.query_successful && r.success_percent > 0), [results]);

  const sortedUsable = useMemo(() => {
    return [...usable].sort((a, b) => (
      Number(a.latency_avg_ms ?? a.avg_time ?? a.resolution_time_ms ?? Infinity) -
      Number(b.latency_avg_ms ?? b.avg_time ?? b.resolution_time_ms ?? Infinity)
    ));
  }, [usable]);

  async function run() {
    const id = ++runCounter.current;
    activeRunId.current = id;
    setLoading(true); setError(null); setResults(null);
    try {
      const params: DnsBenchmarkParams = {
        domainOrIp: domain.trim(),
        samples: Math.max(1, samples),
        timeoutSecs: Math.max(1, timeout),
        validateDnssec: dnssec,
        warmUp,
      };
      const res = await invoke<DnsTestResult[]>("run_dns_benchmark", { args: params });
      if (activeRunId.current !== id) return; // canceled or superseded
      setResults(res);
    } catch (e: any) {
      if (activeRunId.current !== id) return; // canceled
      setError(String(e?.message ?? e));
    } finally {
      if (activeRunId.current === id) setLoading(false);
    }
  }

  function stop() {
    activeRunId.current = null;
    setLoading(false);
  }

  return (
    <div className="flex-1 p-4">
      <div className="glass rounded-xl p-4 mb-4">
        <div className="flex flex-wrap items-center gap-3">
          <input
            className="input"
            placeholder="Domain or IP (e.g. google.com or 65.49.2.178)"
            onChange={e => setDomain(e.target.value)}
          />
        </div>
        <div className="mt-4 grid grid-cols-1 md:grid-cols-3 items-center gap-3">
          <div className="flex items-center gap-3">
            <span className="text-sm text-[var(--muted)]">Samples</span>
            <Stepper value={samples} onChange={setSamples} min={1} aria-label="Samples" />
          </div>
          <div className="flex justify-center">
            <button className={loading ? "btn-danger" : "btn"} onClick={loading ? stop : run}>
              {loading ? "Stop" : "Benchmark DNS Servers"}
            </button>
          </div>
          <div className="flex items-center gap-3 justify-end">
            <span className="text-sm text-[var(--muted)]">Timeout (s)</span>
            <Stepper value={timeout} onChange={setTimeout} min={1} aria-label="Timeout seconds" />
          </div>
        </div>

        {/* Options */}
        <div className="mt-3 flex flex-wrap items-center justify-center gap-4 md:gap-6 text-sm">
          <label className="flex items-center gap-2 cursor-pointer select-none">
            <input
              type="checkbox"
              checked={dnssec}
              onChange={e => setDnssec(e.target.checked)}
            />
            <span className="text-sm">Validate DNSSEC (slower benchmark but accurate)</span>
          </label>
          <label className="flex items-center gap-2 cursor-pointer select-none">
            <input
              type="checkbox"
              checked={warmUp}
              onChange={e => setWarmUp(e.target.checked)}
            />
            <span className="text-sm">Warm-up (don’t measure first lookup)</span>
          </label>
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
              <div><span className="font-medium text-white">Latency (median) :</span> DNS response time - lower is better.</div>
              <div><span className="font-medium text-white">Jitter:</span> variability of response times (Samples must be over 1) - lower is steadier.</div>
              <div><span className="font-medium text-white">Success rate:</span> percentage of successful queries (No Packet Loss) - higher is better.</div>
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

          {/* Export button moved to end, centered */}
          <div className="flex justify-center">
            <button
              className="btn"
              onClick={async () => {
                const safeDomain = toSafeName(domain || "query", "query");
                const ts = new Date().toISOString().replace(/[:.]/g, "-");
                const name = `dns-benchmark-${safeDomain}-${ts}.csv`;
                const csv = toDnsCsv(results || []);
                await saveCsvAs(csv, name);
              }}
              title="Export all results (CSV)"
            >
              Export CSV
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
