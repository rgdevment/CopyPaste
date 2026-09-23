import { getCurrentWindow } from "@tauri-apps/api/window";
import { useRef } from "react";
import { t } from "../locales";

export default function Chrome() {
  const held = useRef<ReturnType<typeof getCurrentWindow>>(null);
  held.current ??= getCurrentWindow();
  const win = held.current;

  return (
    <div className="chrome" data-tauri-drag-region>
      <span className="chrome-who" data-tauri-drag-region>
        CopyPaste
      </span>
      <div className="chrome-does">
        <button
          type="button"
          className="chrome-knob"
          aria-label={t("chromeMinimise")}
          onClick={() => {
            void win.minimize();
          }}
        >
          <svg viewBox="0 0 10 10" aria-hidden="true">
            <path d="M2.4 5h5.2" />
          </svg>
        </button>
        <button
          type="button"
          className="chrome-knob shut"
          aria-label={t("chromeClose")}
          onClick={() => {
            void win.close();
          }}
        >
          <svg viewBox="0 0 10 10" aria-hidden="true">
            <path d="M3.1 3.1l3.8 3.8M6.9 3.1L3.1 6.9" />
          </svg>
        </button>
      </div>
    </div>
  );
}
