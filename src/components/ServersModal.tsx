import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function ServersModal({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [text, setText] = useState("");
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      setLoading(true); setError(null);
      invoke<string[]>("get_dns_servers")
        .then((list) => setText((list || []).join("\n")))
        .catch((e: any) => setError(String(e?.message ?? e)))
        .finally(() => setLoading(false));
    }
  }, [open]);

  async function save() {
    setSaving(true); setError(null);
    const servers = text.split(/\r?\n/).map((s) => s.trim()).filter(Boolean);
    try {
      await invoke("set_dns_servers", { servers });
      onClose();
    } catch (e: any) {
      setError(String(e?.message ?? e));
    } finally { setSaving(false); }
  }

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/60" onClick={onClose} />
      <div className="relative w-[720px] max-w-[92vw] rounded-xl bg-[var(--panel-2)] border border-white/10 p-5">
        <div className="flex items-center justify-between mb-3">
          <div className="text-lg font-semibold">Edit DNS Servers</div>
          <button className="btn" onClick={onClose}>Close</button>
        </div>
        {error && <div className="card border-red-500/50 text-red-300 mb-3">{error}</div>}
        <textarea
          className="w-full h-80 rounded-md bg-[var(--panel)] border border-white/10 p-3 text-sm font-mono"
          value={text}
          onChange={(e) => setText(e.target.value)}
          placeholder={`Enter one server per line.\nExamples:\n8.8.8.8\ntls://dns.google:853\nhttps://cloudflare-dns.com/dns-query`}
        />
        <div className="mt-4 flex items-center justify-end gap-3">
          <button className="btn bg-white/10 hover:bg-white/20" onClick={onClose}>Cancel</button>
          <button className="btn" onClick={save} disabled={saving || loading}>{saving ? "Saving…" : "Save"}</button>
        </div>
      </div>
    </div>
  );
}

