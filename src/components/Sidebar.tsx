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

  const GitHubIcon = ({ className = "" }: { className?: string }) => (
    <svg viewBox="0 0 24 24" aria-hidden="true" className={className}>
      <path fill="currentColor" d="M12 2C6.48 2 2 6.58 2 12.26c0 4.52 2.87 8.35 6.85 9.7.5.1.68-.22.68-.49 0-.24-.01-.88-.01-1.73-2.78.62-3.37-1.2-3.37-1.2-.45-1.18-1.1-1.5-1.1-1.5-.9-.63.07-.62.07-.62 1 .07 1.53 1.05 1.53 1.05.9 1.57 2.36 1.12 2.94.86.09-.67.35-1.12.63-1.38-2.22-.26-4.56-1.14-4.56-5.07 0-1.12.39-2.03 1.03-2.75-.1-.26-.45-1.3.1-2.7 0 0 .85-.28 2.8 1.05.81-.23 1.68-.35 2.54-.36.86 0 1.73.12 2.54.36 1.95-1.33 2.8-1.05 2.8-1.05.55 1.4.21 2.44.1 2.7.64.72 1.03 1.63 1.03 2.75 0 3.94-2.34 4.8-4.57 5.05.36.31.68.92.68 1.86 0 1.35-.01 2.43-.01 2.76 0 .27.18.6.68.49A10.02 10.02 0 0 0 22 12.26C22 6.58 17.52 2 12 2z"/>
    </svg>
  );

  return (
    <aside className="w-64 shrink-0 p-4 rounded-xl bg-[var(--sidebar-bg)] self-stretch min-h-[calc(100vh-16rem)] flex flex-col">
      <div>
        <div className="text-xl font-semibold text-center mb-4">Services</div>
        {item("dns", "DNS Benchmark", "🔎")}
        {item("speed", "Download Speed", "⤵️")}
      </div>
      <div className="mt-auto pt-4 flex justify-center">
        <a
          href="https://github.com/ednoct"
          target="_blank"
          rel="noreferrer"
          className="group inline-flex items-center gap-2 px-3 py-2 rounded-full border border-white/10 bg-white/5 hover:bg-white/10 text-white shadow-sm hover:shadow-md transition"
          title="Visit GitHub profile"
        >
          <span className="relative inline-flex items-center justify-center w-8 h-8 rounded-full bg-white/10 group-hover:bg-white/20 transition">
            <GitHubIcon className="w-5 h-5" />
          </span>
          <span className="text-sm font-medium">GitHub</span>
        </a>
      </div>
    </aside>
  );
}

