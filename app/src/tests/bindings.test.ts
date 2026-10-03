import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { adopt, type Binding, panelKeys } from "../locales";

function ui(name: string) {
  return readFileSync(join(__dirname, "..", "..", "..", "crates", "cp-panel", "ui", name), "utf8");
}

const PANEL = ui("panel.slint");

// every binding ends its branch with one, so each piece holds a single condition
const BRANCHES = PANEL.split("return accept;");

const APPLE = "root.apple";

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

describe("la tabla de atajos", () => {
  it("promete solo teclas que el panel mira de verdad, en Windows y en un Mac", () => {
    for (const mac of [false, true]) {
      const where = mac ? "macOS" : "Windows";
      for (const row of rowsOn(mac)) {
        for (const combination of row.keys.split("·")) {
          const needs = asked(combination);
          if (!needs) {
            continue;
          }
          const found = BRANCHES.some((branch) => needs.every((one) => branch.includes(one)));
          expect(
            found,
            `on ${where} «${row.keys}» promises ${needs.join(" + ")} and no branch reads it`,
          ).toBe(true);
        }
      }
    }
  });

  it("no ofrece en un Mac teclas que su teclado no tiene", () => {
    const forbidden = [/\bF\d/, /\bSupr\b/, /\bDelete\b/, /\bCtrl\b/, /\bAlt\b/, /\bShift\b/];
    for (const row of rowsOn(true)) {
      for (const one of forbidden) {
        expect(one.test(row.keys), `«${row.keys}» is not how a Mac keyboard is written`).toBe(
          false,
        );
      }
    }
  });

  it("lo que solo responde en un Mac va detrás de la plataforma", () => {
    const only = ['event.text == ","', "event.text == Key.Backspace && event.modifiers.control"];
    for (const one of only) {
      const branch = BRANCHES.find((said) => said.includes(one));
      expect(branch, `nothing in the panel reads ${one}`).toBeTruthy();
      expect(branch, `${one} answers on Windows too`).toContain(APPLE);
    }
    expect(PANEL, "the kinds sheet ignores Command on a Mac").toContain(
      "(event.modifiers.alt || (root.apple && event.modifiers.control))",
    );
  });

  it("dice lo mismo en los dos idiomas, fila por fila", () => {
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

  it("un clic con modificador lo lee de verdad lo que se pincha", () => {
    const cards = ui("cards.slint");
    let found = 0;
    for (const row of rowsOn(false)) {
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
          found += 1;
        }
      }
    }
    expect(found).toBeLessThan(8);
  });

  it("el botón que promete sumar tipos existe en el panel", () => {
    for (const tongue of ["es", "en"] as const) {
      adopt(tongue);
      const row = panelKeys(false).find((one) =>
        one.keys.split("·").some((word) => MODES.has(word.trim().toLowerCase())),
      );
      expect(row, `${tongue} never says how to filter by more than one kind`).toBeTruthy();
    }
    expect(PANEL, "nothing in the panel toggles the mode").toContain("keep-toggled");
  });

  it("filtra por tipos que el buscador conoce", () => {
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
