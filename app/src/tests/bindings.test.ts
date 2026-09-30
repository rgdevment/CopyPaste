import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { adopt, panelKeys } from "../locales";

const PANEL = readFileSync(
  join(__dirname, "..", "..", "..", "crates", "cp-panel", "ui", "panel.slint"),
  "utf8",
);

const LOOKED_FOR: Record<string, string> = {
  enter: "event.text == Key.Return",
  esc: "event.text == Key.Escape",
  retroceso: "event.text == Key.Backspace",
  backspace: "event.text == Key.Backspace",
  supr: "event.text == Key.Delete",
  delete: "event.text == Key.Delete",
  tab: "event.text == Key.Tab",
  "flecha derecha": "event.text == Key.RightArrow",
  "right arrow": "event.text == Key.RightArrow",
};

const DESCRIBED = new Set([
  "flechas",
  "arrows",
  "clic",
  "click",
  "doble clic",
  "double click",
  "space",
]);

function wanted(last: string) {
  const one = last.toLowerCase();
  if (one.length === 1) {
    return `event.text == "${one}"`;
  }
  if (DESCRIBED.has(one) || one.startsWith("#")) {
    return null;
  }
  const looked = LOOKED_FOR[one];
  if (!looked) {
    throw new Error(`«${last}» is neither a key the panel looks for nor a described gesture`);
  }
  return looked;
}

describe("la tabla de atajos", () => {
  it("promete solo teclas que el panel mira de verdad", () => {
    adopt("en");
    for (const row of panelKeys()) {
      for (const combination of row.keys.split("·")) {
        const parts = combination
          .split("+")
          .map((one) => one.trim())
          .filter((one) => one.length > 0);
        const last = parts.at(-1);
        if (!last) {
          continue;
        }
        const looked = wanted(last);
        if (!looked) {
          continue;
        }
        expect(PANEL, `«${row.keys}» promises ${looked} and the panel never looks for it`).toContain(
          looked,
        );
      }
    }
  });

  it("dice lo mismo en los dos idiomas, fila por fila", () => {
    adopt("es");
    const es = panelKeys();
    adopt("en");
    const en = panelKeys();
    expect(es.length).toBe(en.length);
    expect(es.length).toBeGreaterThan(0);
    for (let at = 0; at < es.length; at += 1) {
      expect(es[at].keys.length).toBeGreaterThan(0);
      expect(en[at].keys.length).toBeGreaterThan(0);
      expect(es[at].does).not.toBe(en[at].does);
    }
  });

  it("filtra por tipos que el buscador conoce", () => {
    const view = readFileSync(
      join(__dirname, "..", "..", "..", "crates", "cp-panel", "src", "view.rs"),
      "utf8",
    );
    adopt("es");
    for (const row of [...panelKeys()]) {
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
