import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const sheet = readFileSync("src/index.css", "utf8");

describe("the three themes", () => {
  it("light is the base, so with no choice it does not fall to dark", () => {
    const root = sheet.slice(sheet.indexOf(":root {"), sheet.indexOf("}"));
    expect(root).toContain("--panel: #f4f4f6");
    expect(root).toContain("color-scheme: light");
  });

  it("the system one follows the system", () => {
    expect(sheet).toContain("@media (prefers-color-scheme: dark)");
    expect(sheet).toContain(':root:not([data-theme="light"])');
  });

  it("choosing one wins over what the system says, both ways", () => {
    expect(sheet).toContain(':root[data-theme="dark"]');
    const guarded = sheet.indexOf(':root:not([data-theme="light"])');
    expect(guarded).toBeGreaterThan(-1);
  });
});
