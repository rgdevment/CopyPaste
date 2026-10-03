import { describe, expect, it } from "vitest";
import { landing } from "../App";
import { caps } from "../core";
import news from "../news.json";
import { steps, toldFor } from "../ui/Welcome";

describe("la bienvenida", () => {
  it("lleva el historial de la 2.x antes de todo y el permiso antes del atajo", () => {
    expect(steps(true, true)).toEqual(["former", "hello", "trust", "keys", "where", "use"]);
    expect(steps(false, false)).toEqual(["hello", "keys", "where", "use"]);
  });

  it("dibuja el atajo tecla a tecla, con símbolos en el Mac", () => {
    expect(caps("Ctrl+Alt+V", false)).toEqual(["Ctrl", "Alt", "V"]);
    expect(caps("Cmd+Alt+V", true)).toEqual(["⌘", "⌥", "V"]);
    expect(caps("Shift+Ctrl+Space", true)).toEqual(["⇧", "⌃", "Space"]);
    expect(caps("", false)).toEqual([]);
  });

  it("cuenta solo las novedades de las versiones pedidas, en el idioma de la ventana", () => {
    const [first] = news;
    expect(toldFor([first.version], false)[0].told).toBe(first.es);
    expect(toldFor([first.version], true)[0].told).toBe(first.en);
    expect(toldFor(["0.0.1"], false)).toEqual([]);
  });

  it("dice lo mismo en español y en inglés para cada versión", () => {
    for (const one of news) {
      expect(one.es.length, one.version).toBe(one.en.length);
      for (const said of [...one.es, ...one.en]) {
        expect(said.title.trim().length, one.version).toBeGreaterThan(0);
        expect(said.said.trim().length, one.version).toBeGreaterThan(0);
      }
    }
  });
});

describe("los ajustes abiertos desde la bienvenida", () => {
  it("aterrizan en la sección pedida, y en General si no existe", () => {
    expect(landing("#keys")).toBe("keys");
    expect(landing("#about")).toBe("about");
    expect(landing("#welcome")).toBe("general");
    expect(landing("")).toBe("general");
  });
});
