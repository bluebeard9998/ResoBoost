import Logo from "../assets/Logo.png";

export default function Header({ onEditServers }: { onEditServers?: () => void }) {
  return (
    <header className="w-full flex items-center justify-between px-[8vw] py-8">
      <div className="flex items-center gap-6 select-none">
        <img src={Logo} alt="Resoboost" className="h-16 w-16 rounded-lg" />
        <div>
          <div className="text-3xl font-semibold leading-tight">Resoboost</div>
          <div className="text-base text-[var(--muted)] -mt-0.5">DNS Benchmark & Speed Analysis</div>
        </div>
      </div>
      <div>
        <button className="btn" onClick={onEditServers} title="Edit DNS servers">✏️ Edit DNS Servers</button>
      </div>
    </header>
  );
}
