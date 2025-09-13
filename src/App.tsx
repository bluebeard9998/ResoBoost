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
      <div className="flex gap-6 px-[8vw] pb-10 items-stretch min-h-[calc(100vh-16rem)]">
        <Sidebar current={tab} onChange={setTab} />
        <main className="flex-1">
          {tab === "dns" ? <DnsBenchmark /> : <SpeedTest />}
        </main>
      </div>
      <ServersModal open={editOpen} onClose={() => setEditOpen(false)} />
    </div>
  );
}
