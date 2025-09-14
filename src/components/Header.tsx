import Logo from "../assets/Logo.png";

export default function Header({ onEditServers }: { onEditServers?: () => void }) {
  return (
    <header className="w-full flex items-center justify-between px-4 md:px-[8vw] py-6 md:py-8">
      <div className="flex items-center gap-4 md:gap-6 select-none">
        <img src={Logo} alt="Resoboost" className="h-12 w-12 md:h-16 md:w-16 rounded-lg" />
        <div>
          <div className="text-2xl md:text-3xl font-semibold leading-tight">Resoboost</div>
          <div className="text-sm md:text-base text-[var(--muted)] -mt-0.5">DNS Benchmark & Speed Analysis</div>
        </div>
      </div>
      <div>
        <button className="btn" onClick={onEditServers} title="Edit DNS servers">✏️ Edit DNS Servers</button>
      </div>
    </header>
  );
}
