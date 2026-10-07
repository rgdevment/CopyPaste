import { afterEach, describe, expect, it } from "vitest";
import { adopt, inEnglish, panelTroubleSaid } from "../locales";

describe("the language the window speaks", () => {
  afterEach(() => adopt("es"));

  it("speaks Spanish only to a locale that starts with es", () => {
    for (const locale of ["es", "es-CL", "ES-mx"]) {
      adopt(locale);
      expect(inEnglish(), locale).toBe(false);
      expect(document.documentElement.lang).toBe("es");
    }
  });

  it("speaks English to every other locale, not only to en", () => {
    for (const locale of ["en", "en-GB", "pt-BR", "fr", "de-DE", ""]) {
      adopt(locale);
      expect(inEnglish(), locale).toBe(true);
      expect(document.documentElement.lang).toBe("en");
    }
  });
});

describe("what the panel's trouble says", () => {
  afterEach(() => adopt("es"));

  const KEYS = [
    "unstarted",
    "history-replaced",
    "history-unopened",
    "undrawn",
    "unwatched",
    "unemptied",
    "clipboard-denied",
    "stopped",
    "unsteady",
    "unanswering",
  ];

  it("gives every key the panel sends its own words in both languages", () => {
    for (const locale of ["es", "en"]) {
      adopt(locale);
      const unknown = panelTroubleSaid("no-such-trouble");
      const said = KEYS.map((key) => panelTroubleSaid(key));
      for (const [at, one] of said.entries()) {
        expect(one, KEYS[at]).not.toBe(unknown);
      }
      expect(new Set(said).size).toBe(KEYS.length);
    }
  });

  it("is said in the language the window speaks", () => {
    adopt("es");
    expect(panelTroubleSaid("stopped")).toContain("dejó de vigilar el portapapeles");
    adopt("en");
    expect(panelTroubleSaid("stopped")).toContain("stopped watching the clipboard");
  });

  it("a note that leaves copying on never claims nothing is being kept", () => {
    adopt("en");
    expect(panelTroubleSaid("history-replaced")).toContain("still being kept");
    expect(panelTroubleSaid("unemptied")).toContain("still being kept");
    expect(panelTroubleSaid("stopped")).toContain("Nothing you copy is being kept");
  });

  it("never shows a key it does not know, nor what an object inherits", () => {
    adopt("en");
    expect(panelTroubleSaid("the panel is not answering")).toContain("something went wrong");
    expect(panelTroubleSaid("toString")).toContain("something went wrong");
  });
});
