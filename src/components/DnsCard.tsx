import { DnsTestResult } from "../types";

function copy(text: string) {
  if (navigator?.clipboard?.writeText) {
    navigator.clipboard.writeText(text).catch(() => {});
  } else {
    const ta = document.createElement("textarea");
    ta.value = text; document.body.appendChild(ta); ta.select();
    document.execCommand("copy"); document.body.removeChild(ta);
  }
}

export default function DnsCard({ r }: { r: DnsTestResult }) {
  const latency = r.latency_avg_ms ?? r.avg_time ?? (r.resolution_time_ms ?? undefined);
  const ok = r.query_successful && (r.success_percent ?? 0) > 0;
  return (
    <div className={["card flex items-center justify-between gap-4", ok ? "border-emerald-500/30" : "border-red-500/30"].join(" ") }>
      <div className="flex items-center gap-3 min-w-0">
        <button onClick={() => copy(r.server_address)} className="btn !px-2 !py-1 bg-white/10 hover:bg-white/20 text-white" title="Copy server">
          ⧉
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
          <div className="text-xs text-[var(--muted)]">Latency avg</div>
          <div className="font-medium">{latency !== undefined ? `${Math.round(Number(latency))} ms` : "—"}</div>
        </div>
        <div>
          <div className="text-xs text-[var(--muted)]">Jitter avg</div>
          <div className="font-medium">{r.jitter_avg_ms != null ? `${Math.round(r.jitter_avg_ms)} ms` : "—"}</div>
        </div>
        <div>
          <div className="text-xs text-[var(--muted)]">Success rate</div>
          <div className="font-medium">{`${Math.round(r.success_percent)}%`}</div>
        </div>
        <div className="text-right">
          <div className="text-xs text-[var(--muted)]">DNSSEC</div>
          <div className={["inline-flex items-center gap-1 badge", r.dnssec_validated ? "bg-emerald-500/20 text-emerald-400" : "bg-red-500/20 text-red-400"].join(" ") }>
            <span className={["h-2.5 w-2.5 rounded-full", r.dnssec_validated ? "bg-emerald-500" : "bg-red-500"].join(" ")}/>
            {r.dnssec_validated ? "Validated" : "No"}
          </div>
        </div>
      </div>
    </div>
  );
}

