import { useState } from "react";
import { useKept, useTrouble } from "./core";
import { fill, t } from "./locales";
import About from "./ui/About";
import Backup from "./ui/Backup";
import Chrome from "./ui/Chrome";
import General from "./ui/General";
import History from "./ui/History";
import { Clock, Gear, Info, Vault } from "./ui/Icons";

const WHERE = [
  { key: "general", says: "railGeneral", icon: Gear },
  { key: "history", says: "railHistory", icon: Clock },
  { key: "backup", says: "railBackup", icon: Vault },
] as const;

type Where = (typeof WHERE)[number]["key"] | "about";

export default function App() {
  const [where, setWhere] = useState<Where>("general");
  const { kept, trouble, change, look } = useKept();
  const panelTrouble = useTrouble();

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
          {panelTrouble && <p className="alarm">{fill("panelTrouble", panelTrouble)}</p>}
          {trouble && (
            <p className="alarm">
              {trouble}
              <button type="button" className="mild" onClick={look}>
                {t("tryAgain")}
              </button>
            </p>
          )}
          {kept && where === "general" && <General kept={kept} change={change} />}
          {kept && where === "history" && <History kept={kept} change={change} />}
          {where === "backup" && <Backup />}
          {where === "about" && <About />}
        </main>
      </div>
    </>
  );
}
