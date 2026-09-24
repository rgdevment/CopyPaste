import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach, vi } from "vitest";

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: vi.fn(() => Promise.resolve()),
  revealItemInDir: vi.fn(() => Promise.resolve()),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((what: string, args?: { config?: unknown; wanted?: boolean }) => {
    if (what === "settings") {
      return Promise.resolve({
        locale: "es",
        theme: "system",
        shortcut: "Ctrl+Alt+V",
        "hides-when-left": true,
        "keeps-days": 30,
        "images-quota-mb": 0,
      });
    }
    if (what === "keep") {
      const config = args?.config as Record<string, unknown>;
      for (const key of ["keeps-days", "images-quota-mb"]) {
        if (typeof config?.[key] !== "number") {
          return Promise.reject(new Error(`invalid type for ${key}: expected a number`));
        }
      }
      return Promise.resolve(config);
    }
    if (what === "relabel") return Promise.resolve(null);
    if (what === "keys") {
      return Promise.resolve({ wanted: "Ctrl+Alt+V", bound: true });
    }
    if (what === "waking") {
      return Promise.resolve({ offered: true, wakes: false, theirs: false });
    }
    if (what === "wake") {
      return Promise.resolve({ offered: true, wakes: Boolean(args?.wanted), theirs: false });
    }
    if (what === "former") {
      return Promise.resolve({
        path: "C:UsersquienAppDataLocalCopyPasteclipboard.db",
        bytes: 224395264,
      });
    }
    if (what === "where_it_lives") return Promise.resolve("C:UsersquienAppDataLocalCopyPaste");
    return Promise.reject(new Error(`sin simular: ${what}`));
  }),
}));

vi.mock("@tauri-apps/api/app", () => ({
  getVersion: vi.fn(() => Promise.resolve("3.0.0")),
}));

export const theWindow = {
  minimize: vi.fn(() => Promise.resolve()),
  close: vi.fn(() => Promise.resolve()),
};

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => theWindow,
}));

afterEach(cleanup);
