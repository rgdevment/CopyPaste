import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { fill, items, t } from "../locales";
import { Band, Line } from "./Bits";

type Former = { path: string; bytes: number };
type Kept = { path: string; items: number; bytes: number };
type Brought = { added: number; already: number };

const EXTENSION = "cpbackup";

function weighed(bytes: number) {
  const mb = bytes / (1024 * 1024);
  return mb >= 1 ? `${mb.toFixed(1)} MB` : `${Math.round(bytes / 1024)} KB`;
}

function named() {
  const day = new Date().toISOString().slice(0, 10);
  return `CopyPaste-${day}.${EXTENSION}`;
}

export default function Backup() {
  const [former, setFormer] = useState<Former | null>(null);
  const [trouble, setTrouble] = useState<string | null>(null);
  const [busy, setBusy] = useState<"out" | "in" | null>(null);
  const [said, setSaid] = useState<string | null>(null);
  const [came, setCame] = useState<string | null>(null);

  useEffect(() => {
    invoke<Former | null>("former")
      .then(setFormer)
      .catch((why) => setTrouble(String(why)));
  }, []);

  const out = async () => {
    setSaid(null);
    const where = await save({
      defaultPath: named(),
      filters: [{ name: "CopyPaste", extensions: [EXTENSION] }],
    }).catch(() => null);
    if (!where) {
      return;
    }
    setBusy("out");
    try {
      const kept = await invoke<Kept>("save_backup", { path: where, at: Date.now() });
      setSaid(fill("outDone", items(kept.items)));
    } catch (why) {
      setSaid(String(why));
    } finally {
      setBusy(null);
    }
  };

  const bring = async () => {
    setCame(null);
    const chosen = await open({
      multiple: false,
      filters: [{ name: "CopyPaste", extensions: [EXTENSION] }],
    }).catch(() => null);
    if (typeof chosen !== "string") {
      return;
    }
    setBusy("in");
    try {
      const brought = await invoke<Brought>("load_backup", { path: chosen, at: Date.now() });
      if (brought.added === 0) {
        setCame(t("inNothing"));
      } else {
        const more = brought.already > 0 ? ` · ${fill("inAlready", items(brought.already))}` : "";
        setCame(`${fill("inDone", items(brought.added))}${more}`);
      }
    } catch (why) {
      setCame(String(why));
    } finally {
      setBusy(null);
    }
  };

  return (
    <>
      <h1>{t("railBackup")}</h1>

      <Band says={t("bandCopies")} />

      <Line says={t("out")} why={said ?? t("outWhy")}>
        <button type="button" className="strong" disabled={busy !== null} onClick={out}>
          {busy === "out" ? t("busy") : t("outDo")}
        </button>
      </Line>

      <Line says={t("in")} why={came ?? t("inWhy")}>
        <button type="button" className="mild" disabled={busy !== null} onClick={bring}>
          {busy === "in" ? t("busy") : t("inDo")}
        </button>
      </Line>

      <Band says={t("bandFormer")} />

      {former ? (
        <Line
          says={t("former")}
          why={<span className="path">{`${former.path} · ${weighed(former.bytes)}`}</span>}
          more={<div className="said">{t("formerLoses")}</div>}
        >
          <button type="button" className="mild" disabled>
            {t("formerBring")}
          </button>
          <button type="button" className="grave" disabled>
            {t("formerDrop")}
          </button>
          <span className="soon">{t("soon")}</span>
        </Line>
      ) : (
        <Line says={t("former")} why={trouble ?? t("formerNone")} />
      )}
    </>
  );
}
