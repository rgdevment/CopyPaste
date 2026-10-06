import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { adopt, type Binding, panelKeys } from "../locales";

function ui(name: string) {
  return readFileSync(join(__dirname, "..", "..", "..", "crates", "cp-panel", "ui", name), "utf8");
}

const PANEL = ui("panel.slint");

// the sheet answers first and bails out with a reject, so the table's keys are read after it
const AFTER_THE_SHEET = PANEL.slice(PANEL.indexOf("return reject;"));

// past that point every binding closes its branch with one, so each piece is a single condition
const BRANCHES = AFTER_THE_SHEET.split("return accept;").slice(0, -1);

const APPLE = "Theme.apple";

// what the branch asks for before it runs, which is where a modifier is required rather than read
function conditionOf(branch: string) {
  const at = branch.lastIndexOf("if ");
  const said = at === -1 ? branch : branch.slice(at);
  const opens = said.indexOf("{");
  return opens === -1 ? said : said.slice(0, opens);
}

// a condition that refuses a chord reads the same as one that answers it, so drop what it negates,
// and on Windows drop the alternatives that only a Mac ever reaches
function liveOn(said: string, mac: boolean) {
  const kept = said.replaceAll(/![(][^)]*[)]/g, "");
  return mac ? kept : kept.replaceAll(/[(]Theme[.]apple[^)]*[)]/g, "");
}

// a key promised alone has to be read alone: a branch that demands a modifier alongside it
// answers a different chord, which is how Enter kept its promise from the «paste as» branch
function readsItAlone(branch: string, key: string, mac: boolean) {
  return liveOn(conditionOf(branch), mac)
    .split("||")
    .filter((one) => one.includes(key))
    .some((one) => !one.includes("modifiers."));
}

function answers(branch: string, needs: string[], mac: boolean) {
  if (!needs.every((one) => liveOn(branch, mac).includes(one))) {
    return false;
  }
  return needs.length > 1 || readsItAlone(branch, needs[0], mac);
}

const LOOKED_FOR: Record<string, string> = {
  enter: "event.text == Key.Return",
  "⏎": "event.text == Key.Return",
  esc: "event.text == Key.Escape",
  retroceso: "event.text == Key.Backspace",
  backspace: "event.text == Key.Backspace",
  "⌫": "event.text == Key.Backspace",
  supr: "event.text == Key.Delete",
  delete: "event.text == Key.Delete",
  tab: "event.text == Key.Tab",
  "⇥": "event.text == Key.Tab",
  f1: "event.text == Key.F1",
  f2: "event.text == Key.F2",
  "flecha derecha": "event.text == Key.RightArrow",
  "right arrow": "event.text == Key.RightArrow",
  "→": "event.text == Key.RightArrow",
};

const CLICKS = new Set(["clic", "click", "doble clic", "double click"]);

const MODES = new Set(["uno", "varios", "one", "many"]);

const MODIFIER: Record<string, string> = {
  ctrl: "modifiers.control",
  alt: "modifiers.alt",
  shift: "modifiers.shift",
  "⌘": "modifiers.control",
  "⌥": "modifiers.alt",
  "⇧": "modifiers.shift",
  "⌃": "modifiers.meta",
};

const GLYPHS = new Set(["⌘", "⌥", "⇧", "⌃"]);

const DESCRIBED = new Set([
  "flechas",
  "arrows",
  "uno",
  "varios",
  "one",
  "many",
  "clic",
  "click",
  "doble clic",
  "double click",
  "space",
]);

// a Mac combination carries no separator: ⌘⌫ is the Command key and the one it holds
function pieces(combination: string): string[] {
  const said = combination.trim();
  if (said.includes("+")) {
    return said
      .split("+")
      .map((one) => one.trim())
      .filter((one) => one.length > 0);
  }
  const letters = [...said];
  let at = 0;
  while (at < letters.length && GLYPHS.has(letters[at])) {
    at += 1;
  }
  const held = letters.slice(0, at);
  const key = letters.slice(at).join("");
  return key.length > 0 ? [...held, key] : held;
}

function wanted(last: string) {
  const one = last.toLowerCase();
  const looked = LOOKED_FOR[one];
  if (looked) {
    return looked;
  }
  if (one.length === 1) {
    return `event.text == "${one}"`;
  }
  if (DESCRIBED.has(one) || one.startsWith("#")) {
    return null;
  }
  throw new Error(`«${last}» is neither a key the panel looks for nor a described gesture`);
}

function asked(combination: string): string[] | null {
  const parts = pieces(combination);
  const last = parts.at(-1);
  if (!last) {
    return null;
  }
  const looked = wanted(last);
  if (!looked) {
    return null;
  }
  return [
    looked,
    ...parts.slice(0, -1).flatMap((held) => {
      const one = MODIFIER[held.toLowerCase()];
      expect(one, `«${held}» is not a modifier`).toBeTruthy();
      return [one];
    }),
  ];
}

function rowsOn(mac: boolean): Binding[] {
  return ["es", "en"].flatMap((tongue) => {
    adopt(tongue);
    return panelKeys(mac);
  });
}

describe("the shortcut table", () => {
  it("promises only keys the panel really watches, on Windows and on a Mac", () => {
    for (const mac of [false, true]) {
      const where = mac ? "macOS" : "Windows";
      for (const row of rowsOn(mac)) {
        for (const combination of row.keys.split("·")) {
          const needs = asked(combination);
          if (!needs) {
            continue;
          }
          const found = BRANCHES.some((branch) => answers(branch, needs, mac));
          expect(
            found,
            `on ${where} «${row.keys}» promises ${needs.join(" + ")} and no branch reads it`,
          ).toBe(true);
        }
      }
    }
  });

  it("says out loud what a Mac only draws", () => {
    for (const tongue of ["es", "en"] as const) {
      adopt(tongue);
      for (const row of panelKeys(true)) {
        expect(row.said.length, `${tongue} «${row.keys}»`).toBeGreaterThan(0);
        for (const one of [...row.keys]) {
          expect(
            row.said.includes(one) || /[\p{L}\p{N}]/u.test(one) === false || one === " ",
            `«${row.keys}» leaves ${one} unsaid`,
          ).toBe(true);
        }
      }
    }
    adopt("es");
    const [remove] = panelKeys(true).filter((one) => one.id === "remove");
    expect(remove.said).toBe("Comando Retroceso");
    adopt("en");
    expect(panelKeys(true).filter((one) => one.id === "remove")[0].said).toBe("Command Backspace");
  });

  it("does not offer a Mac keys its keyboard lacks", () => {
    const forbidden = [/\bF\d/, /\bSupr\b/, /\bDelete\b/, /\bCtrl\b/, /\bAlt\b/, /\bShift\b/];
    for (const row of rowsOn(true)) {
      for (const one of forbidden) {
        expect(one.test(row.keys), `«${row.keys}» is not how a Mac keyboard is written`).toBe(
          false,
        );
      }
    }
  });

  it("what only answers on a Mac sits behind the platform", () => {
    const only = ['event.text == ","', "event.text == Key.Backspace && event.modifiers.control"];
    for (const one of only) {
      const branch = BRANCHES.find((said) => said.includes(one));
      expect(branch, `nothing in the panel reads ${one}`).toBeTruthy();
      expect(branch, `${one} answers on Windows too`).toContain(APPLE);
    }
    expect(
      BRANCHES.find((one) => one.includes("root.remove(")),
      "a bare forward delete still destroys a card on a Mac, and no table offers it",
    ).toContain("!Theme.apple && event.text == Key.Delete");
    expect(PANEL, "the kinds sheet ignores Command on a Mac").toContain(
      "(event.modifiers.alt || (Theme.apple && event.modifiers.control))",
    );
  });

  it("says the same in both languages, row by row", () => {
    for (const mac of [false, true]) {
      adopt("es");
      const es = panelKeys(mac);
      adopt("en");
      const en = panelKeys(mac);
      expect(es.length).toBe(en.length);
      expect(es.length).toBeGreaterThan(0);
      for (let at = 0; at < es.length; at += 1) {
        expect(es[at].keys.length).toBeGreaterThan(0);
        expect(en[at].keys.length).toBeGreaterThan(0);
        expect(es[at].does).not.toBe(en[at].does);
      }
    }
  });

  it("a click with a modifier really reads what is clicked", () => {
    const cards = ui("cards.slint");
    // the layers row tells this one in words instead of naming a key, so nothing above reaches it
    expect(cards, "no clic suma tipos en vez de cambiarlos").toContain(
      "self.adding = event.modifiers.control",
    );
    for (const row of rowsOn(true)) {
      for (const combination of row.keys.split("·")) {
        const parts = pieces(combination).map((one) => one.toLowerCase());
        const last = parts.at(-1);
        if (!last || !CLICKS.has(last) || parts.length < 2) {
          continue;
        }
        for (const held of parts.slice(0, -1)) {
          const looked = MODIFIER[held];
          expect(looked, `«${held}» is not a modifier`).toBeTruthy();
          expect(
            cards,
            `«${row.keys}» promises ${looked} and nothing that is clicked reads it`,
          ).toContain(looked);
        }
      }
    }
  });

  it("the button that promises to add kinds exists in the panel", () => {
    for (const tongue of ["es", "en"] as const) {
      adopt(tongue);
      const row = panelKeys(false).find((one) =>
        one.keys.split("·").some((word) => MODES.has(word.trim().toLowerCase())),
      );
      expect(row, `${tongue} never says how to filter by more than one kind`).toBeTruthy();
    }
    expect(PANEL, "nothing in the panel toggles the mode").toContain("keep-toggled");
  });

  it("filters by kinds the search box knows", () => {
    const view = readFileSync(
      join(__dirname, "..", "..", "..", "crates", "cp-panel", "src", "view.rs"),
      "utf8",
    );
    adopt("es");
    for (const row of panelKeys(false)) {
      for (const word of row.keys.split("·")) {
        const tag = word.trim();
        if (!tag.startsWith("#")) {
          continue;
        }
        expect(view, `«${tag}» is not a kind the search box knows`).toContain(
          `"${tag.slice(1)}" =>`,
        );
      }
    }
  });
});
