import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { fill, items, t } from "../locales";
import { Band, Line } from "./Bits";

type Former = {
  path: string;
  bytes: number;
  items: number;
  pictures: number;
  picturesGone: number;
  pinned: number;
  labelled: number;
  withStyles: number;
  beyondKeep: number;
  unreadable: string | null;
};

type Underway = { done: number; total: number };

type Crossed = {
  added: number;
  already: number;
  refused: number;
  withoutTheirPicture: number;
  swept: number;
  crowded: number;
};
type Saved = { path: string; items: number; missing: number; bytes: number };
type Brought = { added: number; already: number; refused: number; fromElsewhere: boolean };

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
  const [busy, setBusy] = useState<"out" | "in" | "former" | null>(null);
  const [said, setSaid] = useState<string | null>(null);
  const [came, setCame] = useState<string | null>(null);
  const [crossed, setCrossed] = useState<string | null>(null);
  const [crossing, setCrossing] = useState<Underway | null>(null);
  const [sure, setSure] = useState(false);

  useEffect(() => {
    invoke<Former | null>("former", { at: Date.now() })
      .then(setFormer)
      .catch((why) => setTrouble(String(why)));
  }, []);

  useEffect(() => {
    const going = listen<Underway>("crossing", (event) => {
      setBusy("former");
      setCrossing(event.payload);
    });
    const gone = listen("crossed", () => {
      setCrossing(null);
      setBusy(null);
      invoke<Former | null>("former", { at: Date.now() })
        .then(setFormer)
        .catch(() => {});
    });
    return () => {
      going.then((off) => off()).catch(() => {});
      gone.then((off) => off()).catch(() => {});
    };
  }, []);

  const out = async () => {
    setSaid(null);
    setBusy("out");
    const where = await save({
      defaultPath: named(),
      filters: [{ name: "CopyPaste", extensions: [EXTENSION] }],
    }).catch(() => null);
    if (!where) {
      setBusy(null);
      return;
    }
    try {
      const kept = await invoke<Saved>("save_backup", { path: where, at: Date.now() });
      const lost = kept.missing > 0 ? ` · ${fill("outMissing", items(kept.missing))}` : "";
      setSaid(`${fill("outDone", items(kept.items))}${lost}`);
    } catch (why) {
      setSaid(String(why));
    } finally {
      setBusy(null);
    }
  };

  const bring = async () => {
    setCame(null);
    setBusy("in");
    const chosen = await open({
      multiple: false,
      filters: [{ name: "CopyPaste", extensions: [EXTENSION] }],
    }).catch(() => null);
    if (typeof chosen !== "string") {
      setBusy(null);
      return;
    }
    try {
      const brought = await invoke<Brought>("load_backup", { path: chosen, at: Date.now() });
      const notes = [
        brought.already > 0 ? fill("inAlready", items(brought.already)) : null,
        brought.refused > 0 ? fill("inRefused", items(brought.refused)) : null,
        brought.fromElsewhere ? t("inElsewhere") : null,
      ].filter(Boolean);
      const tail = notes.length > 0 ? ` · ${notes.join(" · ")}` : "";
      setCame(
        brought.added === 0 && notes.length === 0
          ? t("inNothing")
          : `${brought.added === 0 ? t("inNothing") : fill("inDone", items(brought.added))}${tail}`,
      );
    } catch (why) {
      setCame(String(why));
    } finally {
      setBusy(null);
    }
  };

  useEffect(() => {
    if (!sure) {
      return;
    }
    const forgets = setTimeout(() => setSure(false), 4_000);
    return () => clearTimeout(forgets);
  }, [sure]);

  const drop = async () => {
    if (!sure) {
      setSure(true);
      return;
    }
    setSure(false);
    setCrossed(null);
    setBusy("former");
    try {
      const swept = await invoke<{ files: number; bytes: number }>("drop_former");
      setCrossed(fill("formerDropGone", fill("formerDropFiles", String(swept.files))));
      setFormer(null);
    } catch (why) {
      setCrossed(String(why));
    } finally {
      setBusy(null);
    }
  };

  const cross = async () => {
    setCrossed(null);
    setBusy("former");
    try {
      const said = await invoke<Crossed>("bring_former", { at: Date.now() });
      const notes = [
        said.already > 0 ? fill("formerCameAlready", items(said.already)) : null,
        said.refused > 0 ? fill("formerCameRefused", items(said.refused)) : null,
        said.withoutTheirPicture > 0
          ? fill("formerCameFlat", items(said.withoutTheirPicture))
          : null,
        said.swept > 0 ? fill("formerCameSwept", items(said.swept)) : null,
        said.crowded > 0 ? fill("formerCameCrowded", items(said.crowded)) : null,
      ].filter(Boolean);
      const tail = notes.length > 0 ? ` · ${notes.join(" · ")}` : "";
      setCrossed(`${fill("formerCame", items(said.added))}${tail}`);
    } catch (why) {
      setCrossed(String(why));
    } finally {
      setCrossing(null);
      setBusy(null);
    }
  };

  return (
    <>
      <h1>{t("railBackup")}</h1>

      <Band says={t("bandCopies")} />

      <Line
        says={t("out")}
        why={said ?? t("outWhy")}
        more={<div className="said">{t("outPlain")}</div>}
      >
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
          why={
            crossed ?? <span className="path">{`${former.path} · ${weighed(former.bytes)}`}</span>
          }
          more={
            former.unreadable ? (
              <div className="alarm">{fill("formerUnreadable", former.unreadable)}</div>
            ) : (
              <>
                <ul className="facts">
                  <li>{fill("formerHas", items(former.items))}</li>
                  <li>{t("formerKeeps")}</li>
                  {former.withStyles > 0 ? (
                    <li>{fill("formerKeepsStyles", String(former.withStyles))}</li>
                  ) : null}
                  <li>{t("formerLosesPlain")}</li>
                  {former.picturesGone > 0 && (
                    <li>{fill("formerLosesPictures", String(former.picturesGone))}</li>
                  )}
                  <li>{t("formerKeepsSecrets")}</li>
                  <li>{t("formerPanelRests")}</li>
                  <li>{t("formerStays")}</li>
                </ul>
                {former.beyondKeep > 0 && (
                  <div className="alarm">{fill("formerLosesKept", items(former.beyondKeep))}</div>
                )}
                {sure && <div className="alarm">{t("formerDropWhy")}</div>}
              </>
            )
          }
        >
          <button
            type="button"
            className="strong"
            disabled={busy !== null || former.unreadable !== null || former.items === 0}
            onClick={cross}
          >
            {busy !== "former"
              ? t("formerDo")
              : crossing && crossing.total > 0
                ? fill("formerCrossing", `${crossing.done}/${crossing.total}`)
                : t("formerBringing")}
          </button>
          <button type="button" className="grave" disabled={busy !== null} onClick={drop}>
            {sure ? t("formerDropSure") : t("formerDrop")}
          </button>
        </Line>
      ) : (
        <Line says={t("former")} why={crossed ?? trouble ?? t("formerNone")} />
      )}
    </>
  );
}
