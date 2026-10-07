import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { useKept, useTrouble } from "./core";
import { panelTroubleSaid, t } from "./locales";
import About from "./ui/About";
import Backup from "./ui/Backup";
import Chrome from "./ui/Chrome";
import General from "./ui/General";
import History from "./ui/History";
import { Clock, Gear, Info, Key, Vault } from "./ui/Icons";
import Keys from "./ui/Keys";

const WHERE = [
  { key: "general", says: "railGeneral", icon: Gear },
  { key: "keys", says: "railKeys", icon: Key },
  { key: "history", says: "railHistory", icon: Clock },
  { key: "backup", says: "railBackup", icon: Vault },
] as const;

type Where = (typeof WHERE)[number]["key"] | "about";

export function landing(hash: string): Where {
  const asked = hash.replace(/^#/, "");
  return WHERE.some((one) => one.key === asked) || asked === "about" ? (asked as Where) : "general";
}

export default function App() {
  const [where, setWhere] = useState<Where>(() => landing(window.location.hash));
  const { kept, trouble, change, look } = useKept();
  const panelTrouble = useTrouble();

  useEffect(() => {
    const asked = listen<string>("rail", (event) => {
      if (WHERE.some((one) => one.key === event.payload)) {
        setWhere(event.payload as Where);
      }
    });
    return () => {
      void asked.then((drop) => drop());
    };
  }, []);

  return (
    <>
      <Chrome />
      <div className="shell">
        <nav className="rail" aria-label={t("railSections")}>
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
                {t(one.says)}
              </button>
            );
          })}
          <span className="spacer" />
          <button type="button" aria-current={where === "about"} onClick={() => setWhere("about")}>
            <Info />
            {t("railAbout")}
          </button>
        </nav>

        <main className="pane">
          {panelTrouble && <p className="alarm">{panelTroubleSaid(panelTrouble)}</p>}
          {trouble && (
            <p className="alarm">
              {trouble}
              <button type="button" className="mild" onClick={look}>
                {t("tryAgain")}
              </button>
            </p>
          )}
          {kept && where === "general" && <General kept={kept} change={change} />}
          {kept && where === "keys" && <Keys kept={kept} change={change} />}
          {kept && where === "history" && <History kept={kept} change={change} />}
          {where === "backup" && <Backup />}
          {where === "about" && <About />}
        </main>
      </div>
    </>
  );
}
