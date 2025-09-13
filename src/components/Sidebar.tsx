type NavKey = "dns" | "speed";

export default function Sidebar({ current, onChange }: { current: NavKey; onChange: (k: NavKey) => void; }) {
  const item = (key: NavKey, label: string, icon?: string) => {
    const active = current === key;
    return (
      <button
        key={key}
        onClick={() => onChange(key)}
        className={[
          "w-full flex items-center gap-2 px-3 py-2 rounded-md mb-2",
          active ? "bg-blue-600 text-white" : "bg-[var(--panel)] hover:bg-white/5",
        ].join(" ")}
      >
        {icon && <span className="text-lg">{icon}</span>}
        <span className="text-sm font-medium">{label}</span>
      </button>
    );
  };

  return (
    <aside className="w-64 shrink-0 p-4 rounded-xl bg-[var(--sidebar-bg)] self-stretch min-h-[calc(100vh-16rem)]">
      <div className="text-xl font-semibold text-center mb-4">Services</div>
      {item("dns", "DNS Benchmark", "🌐")}
      {item("speed", "Download Speed", "⬇️")}
    </aside>
  );
}

