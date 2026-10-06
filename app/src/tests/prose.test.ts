import { describe, expect, it } from "vitest";
import { t } from "../locales";
import { asProse } from "../ui/Bits";

describe("asProse", () => {
  it("closes every sentence, since loose ones read glued together", () => {
    expect(asProse(["una cosa", "otra cosa"])).toBe("una cosa. otra cosa.");
  });

  it("does not double the full stop of a sentence that already has one", () => {
    expect(asProse(["ya termina.", "y esta no"])).toBe("ya termina. y esta no.");
  });

  it("respects colons and closing marks", () => {
    expect(asProse(["mira esto:", "¿seguro?", "¡claro!"])).toBe("mira esto: ¿seguro? ¡claro!");
  });

  it("drops what is empty instead of leaving a stray full stop", () => {
    expect(asProse(["", "   ", "solo esto"])).toBe("solo esto.");
    expect(asProse([])).toBe("");
  });

  it("leaves the CopyPaste 2 summary as one paragraph with full stops", () => {
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
