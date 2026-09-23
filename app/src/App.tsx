import { useState } from "react";
import { useKept } from "./core";
import About from "./ui/About";
import Backup from "./ui/Backup";
import Chrome from "./ui/Chrome";
import General from "./ui/General";
import History from "./ui/History";
import { Clock, Gear, Info, Vault } from "./ui/Icons";

const WHERE = [
  { key: "general", says: "General", icon: Gear },
  { key: "history", says: "Historial", icon: Clock },
  { key: "backup", says: "Copia de seguridad", icon: Vault },
] as const;

type Where = (typeof WHERE)[number]["key"] | "about";

export default function App() {
  const [where, setWhere] = useState<Where>("general");
  const { kept, trouble, change } = useKept();

  return (
    <>
      <Chrome />
      <div className="shell">
        <nav className="rail" aria-label="Secciones">
          {WHERE.map((one) => {
            const Icon = one.icon;
            return (
              <button
                key={one.key}
                type="button"
                aria-current={where === one.key}
                onClick={() => setWhere(one.key)}
              >
                <Icon />
                {one.says}
              </button>
            );
          })}
          <span className="spacer" />
          <button type="button" aria-current={where === "about"} onClick={() => setWhere("about")}>
            <Info />
            Acerca de
          </button>
        </nav>

        <main className="pane">
          {trouble && <p className="alarm">{trouble}</p>}
          {kept && where === "general" && <General kept={kept} change={change} />}
          {kept && where === "history" && <History kept={kept} change={change} />}
          {where === "backup" && <Backup />}
          {where === "about" && <About />}
        </main>
      </div>
    </>
  );
}
