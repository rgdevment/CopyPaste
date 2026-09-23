import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach, vi } from "vitest";

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: vi.fn(() => Promise.resolve()),
  revealItemInDir: vi.fn(() => Promise.resolve()),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((what: string, args?: { config?: unknown }) => {
    if (what === "settings") {
      return Promise.resolve({
        locale: null,
        theme: "system",
        "wakes-with-session": true,
        shortcut: "Ctrl+Alt+V",
        "hides-when-left": true,
        "keeps-days": 30,
        "images-quota-mb": null,
      });
    }
    if (what === "keep") return Promise.resolve(args?.config);
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
