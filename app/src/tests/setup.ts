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
        "images-quota-mb": null,
      });
    }
    if (what === "keep") return Promise.resolve(args?.config);
    if (what === "relabel") return Promise.resolve(null);
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

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    minimize: vi.fn(() => Promise.resolve()),
    close: vi.fn(() => Promise.resolve()),
  }),
}));

afterEach(cleanup);
