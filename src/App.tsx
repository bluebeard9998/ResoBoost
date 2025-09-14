import { useState } from "react";
import Header from "./components/Header";
import Sidebar from "./components/Sidebar";
import DnsBenchmark from "./pages/DnsBenchmark";
import SpeedTest from "./pages/SpeedTest";
import "./style.css";
import ServersModal from "./components/ServersModal";

type Tab = "dns" | "speed";

export default function App() {
  const [tab, setTab] = useState<Tab>("dns");
  const [editOpen, setEditOpen] = useState(false);

  return (
    <div className="min-h-full flex flex-col">
      <Header onEditServers={() => setEditOpen(true)} />
      <div className="flex gap-6 px-4 md:px-[8vw] pb-10 items-stretch min-h-[calc(100vh-16rem)] flex-col md:flex-row">
        <div className="hidden md:block">
          <Sidebar current={tab} onChange={setTab} />
        </div>
        <main className="flex-1">
          {/* Mobile top tabs */}
          <div className="md:hidden mb-4">
            <div className="inline-flex rounded-xl overflow-hidden border border-white/10 bg-[var(--panel)]">
              <button
                className={`px-3 py-2 text-sm ${tab === "dns" ? "bg-blue-600 text-white" : "text-gray-300 hover:bg-white/5"}`}
                onClick={() => setTab("dns")}
              >
                DNS Benchmark
              </button>
              <button
                className={`px-3 py-2 text-sm ${tab === "speed" ? "bg-blue-600 text-white" : "text-gray-300 hover:bg-white/5"}`}
                onClick={() => setTab("speed")}
              >
                Download Speed
              </button>
            </div>
          </div>
          {tab === "dns" ? <DnsBenchmark /> : <SpeedTest />}
        </main>
      </div>
      <ServersModal open={editOpen} onClose={() => setEditOpen(false)} />
    </div>
  );
}
