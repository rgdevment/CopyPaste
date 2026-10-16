import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";
import { caps, onMac, useKept } from "../core";
import { inEnglish, sized, t } from "../locales";
import panda from "../panda.svg";

export const LASTS_MS = 10000;

export type Glance = { items: number; today: number; pinned: number; bytes: number };

export function midnight(now = new Date()): number {
  const day = new Date(now);
  day.setHours(0, 0, 0, 0);
  return day.getTime();
}

function leave() {
  void getCurrentWindow().close();
}

function counted(value: number) {
  return value.toLocaleString(inEnglish() ? "en" : "es");
}

export default function Ready() {
  const { kept } = useKept();
  const [glance, setGlance] = useState<Glance | null>(null);
  const [held, setHeld] = useState(false);
  const left = useRef(LASTS_MS);
  const mac = onMac();

  useEffect(() => {
    invoke<Glance>("glance", { since: midnight() })
      .then(setGlance)
      .catch(() => setGlance({ items: 0, today: 0, pinned: 0, bytes: 0 }));
    const heard = listen("panel-shown", leave);
    return () => {
      void heard.then((drop) => drop());
    };
  }, []);

  useEffect(() => {
    const page = document.documentElement;
    const hold = () => setHeld(true);
    const release = () => setHeld(false);
    page.addEventListener("mouseenter", hold);
    page.addEventListener("mouseleave", release);
    return () => {
      page.removeEventListener("mouseenter", hold);
      page.removeEventListener("mouseleave", release);
    };
  }, []);

  useEffect(() => {
    if (held || glance === null) {
      return;
    }
    const started = Date.now();
    const timer = setTimeout(leave, left.current);
    return () => {
      clearTimeout(timer);
      left.current -= Date.now() - started;
    };
  }, [held, glance]);

  if (!kept || !glance) {
    return null;
  }

  return (
    <div className={`ready${held ? " held" : ""}`}>
      <div className="ready-top">
        <img className="ready-mark" src={panda} alt="" />
        <div className="ready-who">
          <b>{t("readyTitle")}</b>
          <span>
            <i className="ready-dot" />
            {t("readyWatching")}
          </span>
        </div>
        <button
          type="button"
          className="ready-knob"
          aria-label={t("readySettings")}
          title={t("readySettings")}
          onClick={() => {
            invoke("open_settings", { rail: "general" }).catch(() => {});
            leave();
          }}
        >
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <circle cx="12" cy="12" r="3" />
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33 1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82 1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" />
          </svg>
        </button>
        <button
          type="button"
          className="ready-knob"
          aria-label={t("chromeClose")}
          title={t("chromeClose")}
          onClick={leave}
        >
          <svg viewBox="0 0 10 10" aria-hidden="true">
            <path d="M2.5 2.5l5 5M7.5 2.5l-5 5" />
          </svg>
        </button>
      </div>
      {glance.items > 0 ? (
        <dl className="ready-stats">
          {(
            [
              [counted(glance.items), t("readySaved")],
              [counted(glance.today), t("readyToday")],
              [counted(glance.pinned), t("readyPinned")],
              [sized(glance.bytes), t("readyOnDisk")],
            ] as const
          ).map(([value, label]) => (
            <div key={label}>
              <dt>{value}</dt>
              <dd>{label}</dd>
            </div>
          ))}
        </dl>
      ) : (
        <p className="ready-empty">{t("readyEmpty")}</p>
      )}
      <div className="ready-foot">
        <span>{t("readyOpenWith")}</span>
        {caps(kept.shortcut, mac).map((one, index) => (
          <span key={one} className="ready-cap">
            {index > 0 && !mac && <span className="ready-plus">+</span>}
            <kbd>{one}</kbd>
          </span>
        ))}
        <button
          type="button"
          className="strong ready-show"
          onClick={() => {
            invoke("show_panel").catch(() => {});
          }}
        >
          {t("readyShow")}
        </button>
      </div>
      <div className="ready-bar">
        <i style={{ animationDuration: `${LASTS_MS}ms` }} />
      </div>
    </div>
  );
}
