import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { LINKS } from "../ui/About";

type Allowed = { url: string };
type Permission = string | { identifier: string; allow?: Allowed[] };

const TOOLS = ["https://rgdevment.com/tisty/", "https://rgdevment.com/linkunbound/"];

function allowed(): string[] {
  const said = readFileSync("src-tauri/capabilities/default.json", "utf8");
  const permissions: Permission[] = JSON.parse(said).permissions;
  const opener = permissions.find(
    (one) => typeof one !== "string" && one.identifier === "opener:allow-open-url",
  );
  if (typeof opener === "string" || !opener?.allow) {
    throw new Error("la ventana no tiene permiso para abrir enlaces");
  }
  return opener.allow.map((one) => one.url);
}

function covers(pattern: string, url: string): boolean {
  const star = pattern.indexOf("*");
  if (star < 0) {
    return pattern === url;
  }
  return url.startsWith(pattern.slice(0, star)) && url.endsWith(pattern.slice(star + 1));
}

describe("los enlaces del acerca de", () => {
  it("todos están permitidos, así que ninguno nace muerto", () => {
    const patterns = allowed();
    for (const url of [...LINKS, ...TOOLS]) {
      expect(
        patterns.some((one) => covers(one, url)),
        `${url} no lo permite ningún patrón`,
      ).toBe(true);
    }
  });

  it("las otras herramientas llevan a su propia página, no al repositorio", () => {
    for (const url of TOOLS) {
      expect(url.startsWith("https://rgdevment.com/")).toBe(true);
    }
  });
});
