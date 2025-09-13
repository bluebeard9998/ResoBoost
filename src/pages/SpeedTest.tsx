import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { DownloadSpeedParams, DownloadTestResult } from "../types";
import Stepper from "../components/Stepper";
import ProgressBar from "../components/ProgressBar";

function copy(text: string) {
  if (navigator?.clipboard?.writeText) {
    navigator.clipboard.writeText(text).catch(() => {});
  }
}

export default function SpeedTest() {
  const [url, setUrl] = useState("https://cachefly.cachefly.net/1mb.test");
  const [duration, setDuration] = useState(7);
  const [timeout, setTimeout] = useState(10);
  const [loading, setLoading] = useState(false);
  const [results, setResults] = useState<DownloadTestResult[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  const sorted = useMemo(() => {
    return [...(results || [])].sort((a, b) => b.bandwidth_mbps - a.bandwidth_mbps);
  }, [results]);

  async function run() {
    setLoading(true);
    setError(null);
    setResults(null);
    try {
      const params: DownloadSpeedParams = {
        url: url.trim(),
        durationSecs: Math.max(1, duration),
        timeoutSecs: Math.max(1, timeout),
      };
      const res = await invoke<DownloadTestResult[]>("perform_download_speed_test", params);
      setResults(res);
    } catch (e: any) {
      setError(String(e?.message ?? e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="flex-1 p-4">
      <h2 className="text-2xl font-semibold mb-3">Download File Address</h2>
      <div className="glass rounded-xl p-4 mb-4">
        <div className="flex flex-wrap items-center gap-3">
          <input
            className="input"
            placeholder="HTTP/HTTPS file url"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
          />
        </div>
        <div className="mt-4 grid grid-cols-3 items-center">
          <div className="flex items-center gap-3">
            <span className="text-sm text-[var(--muted)]">Per-DNS Duration (s)</span>
            <Stepper value={duration} onChange={setDuration} min={1} aria-label="Duration seconds" />
          </div>
          <div className="flex justify-center">
            <button className="btn" onClick={run} disabled={loading}>
              {loading ? "Running…" : "Test Download Speed"}
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
        <div className="space-y-3">
          {sorted.map((r) => (
            <div key={r.server_address} className="card flex items-center justify-between">
              <div className="flex items-center gap-3 min-w-0">
                <button
                  onClick={() => copy(r.server_address)}
                  className="btn !px-2 !py-1 bg-white/10 hover:bg-white/20 text-white"
                  title="Copy server"
                >
                  Copy
                </button>
                <div className="truncate">
                  <div className="font-semibold truncate">{r.server_address}</div>
                  <div className="text-xs text-[var(--muted)] truncate">{r.resolved_ip || ""}</div>
                </div>
              </div>
              <div className="text-right">
                <div className="text-xs text-[var(--muted)]">Bandwidth</div>
                <div className="font-semibold">{r.bandwidth_mbps.toFixed(2)} Mbps</div>
              </div>
            </div>
          ))}
          {sorted.length === 0 && <div className="text-sm text-[var(--muted)]">No results.</div>}
        </div>
      )}
    </div>
  );
}

