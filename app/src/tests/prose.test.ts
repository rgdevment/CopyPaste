import { describe, expect, it } from "vitest";
import { t } from "../locales";
import { asProse } from "../ui/Bits";

describe("asProse", () => {
  it("cierra cada frase, que sueltas se leen pegadas", () => {
    expect(asProse(["una cosa", "otra cosa"])).toBe("una cosa. otra cosa.");
  });

  it("no dobla el punto de una frase que ya lo trae", () => {
    expect(asProse(["ya termina.", "y esta no"])).toBe("ya termina. y esta no.");
  });

  it("respeta los dos puntos y los signos de cierre", () => {
    expect(asProse(["mira esto:", "¿seguro?", "¡claro!"])).toBe("mira esto: ¿seguro? ¡claro!");
  });

  it("descarta lo vacio en vez de dejar un punto suelto", () => {
    expect(asProse(["", "   ", "solo esto"])).toBe("solo esto.");
    expect(asProse([])).toBe("");
  });

  it("deja el resumen de CopyPaste 2 como un parrafo con puntos", () => {
    const said = asProse([
      t("formerKeeps"),
      t("formerLosesPlain"),
      t("formerKeepsSecrets"),
      t("formerPanelRests"),
      t("formerStays"),
    ]);
    expect(said.endsWith(".")).toBe(true);
    expect(said).toContain("la pegaste. El resto");
    expect(said).toContain("ya no los tiene. La 2");
    expect(said).toContain("cruza igual. El panel");
  });
});
