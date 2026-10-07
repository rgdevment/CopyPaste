import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach, vi } from "vitest";

const asked = vi.hoisted(() => ({
  greeting: { kind: "tour", former: true } as unknown,
  bound: true,
  locale: "es",
  trust: { offered: false, pastes: false, secureInput: false, clipboard: "allowed" },
}));

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
        locale: asked.locale,
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
      return Promise.resolve({ wanted: "Ctrl+Alt+V", bound: asked.bound });
    }
    if (what === "waking") {
      return Promise.resolve({ offered: true, wakes: false, theirs: false, managed: false });
    }
    if (what === "wake") {
      return Promise.resolve({
        offered: true,
        wakes: Boolean(args?.wanted),
        theirs: false,
        managed: false,
      });
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
        came: 0,
        cameStill: 0,
        cameAt: null,
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
    if (what === "trust" || what === "ask_trust") {
      return Promise.resolve({ ...asked.trust });
    }
    if (what === "update_ready") {
      return Promise.resolve({ route: "download", looked: true, ready: null });
    }
    if (what === "install_route") {
      return Promise.resolve("download");
    }
    if (what === "update_install") {
      return Promise.resolve(null);
    }
    if (what === "save_backup") {
      return Promise.resolve({ path: "/donde/quiera/CopyPaste.cpbackup", items: 3, bytes: 2048 });
    }
    if (what === "spare") {
      return Promise.resolve(["Alt+Shift+V", "Ctrl+Alt+Shift+V"]);
    }
    if (what === "load_backup") {
      return Promise.resolve({ added: 2, already: 1 });
    }
    if (what === "linkunbound_here") {
      return Promise.resolve(false);
    }
    if (what === "open_web") {
      return Promise.resolve();
    }
    if (what === "greeting") return Promise.resolve(asked.greeting);
    if (what === "open_settings" || what === "tour") return Promise.resolve(null);
    return Promise.reject(new Error(`sin simular: ${what}`));
  }),
}));

vi.mock("@tauri-apps/api/app", () => ({
  getVersion: vi.fn(() => Promise.resolve("3.0.0")),
}));

export const scene = asked;

export const theWindow = {
  minimize: vi.fn(() => Promise.resolve()),
  close: vi.fn(() => Promise.resolve()),
};

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => theWindow,
}));

afterEach(() => {
  cleanup();
  asked.greeting = { kind: "tour", former: true };
  asked.bound = true;
  asked.locale = "es";
  asked.trust = { offered: false, pastes: false, secureInput: false, clipboard: "allowed" };
});
