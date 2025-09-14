import { DnsTestResult } from "../types";

function copy(text: string) {
  if (navigator?.clipboard?.writeText) {
    navigator.clipboard.writeText(text).catch(() => {});
  } else {
    const ta = document.createElement("textarea");
    ta.value = text;
    document.body.appendChild(ta);
    ta.select();
    document.execCommand("copy");
    document.body.removeChild(ta);
  }
}

export default function DnsCard({ r }: { r: DnsTestResult }) {
  const num = (v: any): number | undefined => {
    if (v == null) return undefined;
    const n = Number(v);
    return Number.isFinite(n) ? n : undefined;
  };

  const latency = num(r.latency_avg_ms) ?? num(r.avg_time) ?? num(r.resolution_time_ms);
  const jitter = num(r.jitter_avg_ms);
  const fmtMs = (v?: number) => (v == null ? "–" : (v < 1 ? `${v.toFixed(1)} ms` : `${Math.round(v)} ms`));
  const ok = r.query_successful && (r.success_percent ?? 0) > 0;

  return (
    <div className={["card flex items-center justify-between gap-4", ok ? "border-emerald-500/30" : "border-red-500/30"].join(" ") }>
      <div className="flex items-center gap-3 min-w-0">
        <button onClick={() => copy(r.server_address)} className="btn !px-2 !py-1 bg-white/10 hover:bg-white/20 text-white" title="Copy server">
          Copy
        </button>
        <div className="truncate">
          <div className="font-semibold truncate">{r.server_address}</div>
          <div className="text-xs text-[var(--muted)] truncate max-w-[52ch]">
            {r.error_msg ? r.error_msg : (r.ipv4_ips?.length || r.ipv6_ips?.length ? [...r.ipv4_ips, ...r.ipv6_ips].slice(0,3).join(", ") : "")}
          </div>
        </div>
      </div>
      <div className="grid grid-cols-4 gap-3 text-sm text-right">
        <div>
          <div className="text-xs text-[var(--muted)]">Latency</div>
          <div className="font-medium">{fmtMs(latency)}</div>
        </div>
        <div>
          <div className="text-xs text-[var(--muted)]">Jitter</div>
          <div className="font-medium">{fmtMs(jitter)}</div>
        </div>
        <div>
          <div className="text-xs text-[var(--muted)]">Success rate</div>
          <div className="font-medium">{`${Math.round(r.success_percent)}%`}</div>
        </div>
        <div className="text-right">
          <div className="text-xs text-[var(--muted)]">DNSSEC</div>
          {r.dnssec_enabled === false ? (
            <div className={"inline-flex items-center gap-1 badge bg-yellow-500/20 text-yellow-400"}>
              <span className={"h-2.5 w-2.5 rounded-full bg-yellow-500"}/>
              Disabled
            </div>
          ) : (
            <div className={["inline-flex items-center gap-1 badge", r.dnssec_validated ? "bg-emerald-500/20 text-emerald-400" : "bg-red-500/20 text-red-400"].join(" ") }>
              <span className={["h-2.5 w-2.5 rounded-full", r.dnssec_validated ? "bg-emerald-500" : "bg-red-500"].join(" ")}/>
              {r.dnssec_validated ? "Validated" : "No"}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}




