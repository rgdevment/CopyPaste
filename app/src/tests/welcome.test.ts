import { describe, expect, it } from "vitest";
import { landing } from "../App";
import { asKeys, caps } from "../core";
import news from "../news.json";
import { newestStable, steps, toldFor } from "../ui/Welcome";

describe("the welcome", () => {
  it("brings the 2.x history before anything else and the permission before the shortcut", () => {
    expect(steps(true, true)).toEqual(["former", "hello", "trust", "keys", "where", "use"]);
    expect(steps(false, false)).toEqual(["hello", "keys", "where", "use"]);
    expect(steps(true, false, true)).toEqual(["former", "hello", "keys", "where", "use", "news"]);
  });

  it("draws the shortcut key by key, with symbols and in the Mac order", () => {
    expect(caps("Ctrl+Alt+V", false)).toEqual(["Ctrl", "Alt", "V"]);
    expect(caps("Cmd+Alt+V", true)).toEqual(["⌥", "⌘", "V"]);
    expect(caps("Shift+Ctrl+Space", true)).toEqual(["⌃", "⇧", "Space"]);
    expect(caps("", false)).toEqual([]);
  });

  it("joins the keys without gluing a word to a symbol", () => {
    expect(asKeys("Ctrl+Alt+V", false)).toBe("Ctrl + Alt + V");
    expect(asKeys("Cmd+Alt+V", true)).toBe("⌥⌘V");
    expect(asKeys("Shift+Cmd+Space", true)).toBe("⇧⌘ Space");
    expect(asKeys("Ctrl+Alt+F9", true)).toBe("⌃⌥ F9");
    expect(asKeys("Super+V", true), "what nothing maps is left as it was stored").toBe("Super V");
  });

  it("tells only the news of the versions asked for, in the window's language", () => {
    const [first] = news;
    expect(toldFor([first.version], false, false)[0].told.map((one) => one.title)).toEqual(
      first.es.map((one) => one.title),
    );
    expect(toldFor([first.version], true, false)[0].told.map((one) => one.title)).toEqual(
      first.en.map((one) => one.title),
    );
    expect(toldFor(["0.0.1"], false, false)).toEqual([]);
  });

  it("names each platform's key, and leaves no gap unfilled", () => {
    for (const one of news) {
      for (const mac of [false, true]) {
        for (const english of [false, true]) {
          const [told] = toldFor([one.version], english, mac);
          for (const said of told.told) {
            expect(said.said, `${one.version} ${said.title}`).not.toMatch(/\{[a-z-]+\}/);
          }
        }
      }
    }
    const first = news.find((one) => one.es.some((said) => said.said.includes("{name}")));
    if (!first) throw new Error("una novedad nombra la tecla del nombre");
    const [onAMac] = toldFor([first.version], false, true);
    expect(onAMac.told.some((one) => one.said.includes("⌘E"))).toBe(true);
    const [onWindows] = toldFor([first.version], false, false);
    expect(onWindows.told.some((one) => one.said.includes("F2"))).toBe(true);
  });

  it("says the same in Spanish and in English for every version", () => {
    for (const one of news) {
      expect(one.es.length, one.version).toBe(one.en.length);
      for (const said of [...one.es, ...one.en]) {
        expect(said.title.trim().length, one.version).toBeGreaterThan(0);
        expect(said.said.trim().length, one.version).toBeGreaterThan(0);
      }
    }
  });
});

describe("settings opened from the welcome", () => {
  it("land on the section asked for, and on General if it does not exist", () => {
    expect(landing("#keys")).toBe("keys");
    expect(landing("#about")).toBe("about");
    expect(landing("#welcome")).toBe("general");
    expect(landing("")).toBe("general");
  });

  it("tells someone coming from 2.x what the newest stable release brings", () => {
    const newest = newestStable();
    expect(newest).not.toBeNull();
    expect(newest).not.toContain("-");
    expect(news.some((one) => one.version === newest)).toBe(true);
  });
});
