import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const sheet = readFileSync("src/index.css", "utf8");

describe("los tres temas", () => {
  it("el claro es la base, así que sin elección no se cae al oscuro", () => {
    const root = sheet.slice(sheet.indexOf(":root {"), sheet.indexOf("}"));
    expect(root).toContain("--panel: #f7f8fb");
    expect(root).toContain("color-scheme: light");
  });

  it("el del sistema sigue al sistema", () => {
    expect(sheet).toContain("@media (prefers-color-scheme: dark)");
    expect(sheet).toContain(':root:not([data-theme="light"])');
  });

  it("elegir uno gana sobre lo que diga el sistema, en los dos sentidos", () => {
    expect(sheet).toContain(':root[data-theme="dark"]');
    const guarded = sheet.indexOf(':root:not([data-theme="light"])');
    expect(guarded).toBeGreaterThan(-1);
  });
});
