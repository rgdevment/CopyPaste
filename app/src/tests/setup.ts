import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach, vi } from "vitest";

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: vi.fn(() => Promise.resolve("/donde/quiera/CopyPaste.cpbackup")),
  open: vi.fn(() => Promise.resolve("/donde/quiera/CopyPaste.cpbackup")),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

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
    if (what === "empty") return Promise.resolve(null);
    if (what === "trouble") return Promise.resolve(null);
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
        items: 1200,
        pictures: 40,
        picturesGone: 3,
        pinned: 12,
        labelled: 5,
        withStyles: 260,
        beyondKeep: 860,
        unreadable: null,
      });
    }
    if (what === "bring_former") {
      return Promise.resolve({
        added: 1180,
        already: 0,
        refused: 0,
        withoutTheirPicture: 3,
        swept: 860,
        crowded: 0,
      });
    }
    if (what === "where_it_lives") return Promise.resolve("C:UsersquienAppDataLocalCopyPaste");
    if (what === "trust") {
      return Promise.resolve({
        offered: false,
        pastes: false,
        secureInput: false,
      });
    }
    if (what === "update_ready") {
      return Promise.resolve({ route: "download", looked: true, ready: null });
    }
    if (what === "update_install") {
      return Promise.resolve(null);
    }
    if (what === "save_backup") {
      return Promise.resolve({ path: "/donde/quiera/CopyPaste.cpbackup", items: 3, bytes: 2048 });
    }
    if (what === "spare") {
      return Promise.resolve(["Ctrl+Shift+V", "Ctrl+Alt+C"]);
    }
    if (what === "load_backup") {
      return Promise.resolve({ added: 2, already: 1 });
    }
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
