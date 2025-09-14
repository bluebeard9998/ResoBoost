import { DnsTestResult, DownloadTestResult } from "../types";

export async function saveJsonAs(data: unknown, suggestedName: string): Promise<boolean> {
  try {
    const json = typeof data === "string" ? data : JSON.stringify(data, null, 2);
    const blob = new Blob([json], { type: "application/json" });

    // Try the File System Access API if available (Chromium/WebView2).
    const w: any = window as any;
    if (typeof w.showSaveFilePicker === "function") {
      try {
        const handle = await w.showSaveFilePicker({
          suggestedName,
          types: [
            {
              description: "JSON Files",
              accept: { "application/json": [".json"] },
            },
          ],
        });
        const writable = await handle.createWritable();
        await writable.write(blob);
        await writable.close();
        return true;
      } catch (e) {
        // fall through to anchor-based download
        console.error("showSaveFilePicker failed", e);
      }
    }

    // Fallback: anchor-based download
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = suggestedName;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    return true;
  } catch (err) {
    console.error("Failed to export JSON:", err);
    return false;
  }
}

export function toSafeName(input: string, fallback = "data") {
  const base = (input || fallback).trim();
  return base.replace(/[^a-zA-Z0-9._-]/g, "_");
}

export async function saveCsvAs(csv: string, suggestedName: string): Promise<boolean> {
  try {
    const blob = new Blob([csv], { type: "text/csv" });
    const w: any = window as any;
    if (typeof w.showSaveFilePicker === "function") {
      try {
        const handle = await w.showSaveFilePicker({
          suggestedName,
          types: [
            {
              description: "CSV Files",
              accept: { "text/csv": [".csv"] },
            },
          ],
        });
        const writable = await handle.createWritable();
        await writable.write(blob);
        await writable.close();
        return true;
      } catch (e) {
        console.error("showSaveFilePicker failed", e);
      }
    }

    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = suggestedName;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    return true;
  } catch (err) {
    console.error("Failed to export CSV:", err);
    return false;
  }
}

function csvEscape(value: string): string {
  if (value == null) return "";
  const needsQuotes = /[",\n\r]/.test(value);
  let out = value.replace(/"/g, '""');
  return needsQuotes ? `"${out}"` : out;
}

export function toDnsCsv(results: DnsTestResult[]): string {
  const headers = [
    "server_address",
    "query_successful",
    "success_percent",
    "latency_avg_ms",
    "jitter_avg_ms",
    "resolution_time_ms",
    "avg_time",
    "dnssec_validated",
    "ipv4_ips",
    "ipv6_ips",
    "error_msg",
  ];
  const lines: string[] = [];
  lines.push(headers.join(","));
  for (const r of results) {
    const row = [
      r.server_address ?? "",
      String(!!r.query_successful),
      r.success_percent != null ? String(r.success_percent) : "",
      r.latency_avg_ms != null ? String(r.latency_avg_ms) : "",
      r.jitter_avg_ms != null ? String(r.jitter_avg_ms) : "",
      r.resolution_time_ms != null ? String(r.resolution_time_ms) : "",
      r.avg_time != null ? String(r.avg_time) : "",
      String(!!r.dnssec_validated),
      (r.ipv4_ips || []).join(";"),
      (r.ipv6_ips || []).join(";"),
      r.error_msg ?? "",
    ].map((v) => csvEscape(v as string));
    lines.push(row.join(","));
  }
  return lines.join("\n");
}

export function toDownloadCsv(results: DownloadTestResult[]): string {
  const headers = [
    "server_address",
    "resolved_ip",
    "query_successful",
    "http_status",
    "duration_ms",
    "bytes_read",
    "bandwidth_mbps",
    "error_msg",
  ];
  const lines: string[] = [];
  lines.push(headers.join(","));
  for (const r of results) {
    const row = [
      r.server_address ?? "",
      r.resolved_ip ?? "",
      String(!!r.query_successful),
      r.http_status != null ? String(r.http_status) : "",
      r.duration_ms != null ? String(r.duration_ms) : "",
      r.bytes_read != null ? String(r.bytes_read) : "",
      r.bandwidth_mbps != null ? String(r.bandwidth_mbps) : "",
      r.error_msg ?? "",
    ].map((v) => csvEscape(v as string));
    lines.push(row.join(","));
  }
  return lines.join("\n");
}
