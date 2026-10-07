import { describe, expect, it } from "vitest";
import { combination } from "../core";

function press(code: string, held: Partial<Record<"ctrl" | "alt" | "shift" | "meta", boolean>>) {
  return {
    code,
    ctrlKey: held.ctrl === true,
    altKey: held.alt === true,
    shiftKey: held.shift === true,
    metaKey: held.meta === true,
  };
}

describe("the combination the user presses", () => {
  it("is written the way the system expects it", () => {
    expect(combination(press("KeyV", { ctrl: true, alt: true }))).toBe("Ctrl+Alt+V");
    expect(combination(press("F9", { ctrl: true, alt: true }))).toBe("Ctrl+Alt+F9");
    expect(combination(press("Digit1", { ctrl: true, shift: true }))).toBe("Ctrl+Shift+1");
    expect(combination(press("Space", { meta: true, alt: true }))).toBe("Alt+Cmd+Space");
  });

  it("refuses a lone key, which would hijack the whole keyboard", () => {
    expect(combination(press("KeyV", {}))).toBeNull();
    expect(combination(press("F9", {}))).toBeNull();
  });

  it("refuses Shift alone, which would take a capital letter from every app", () => {
    expect(combination(press("KeyV", { shift: true }))).toBeNull();
    expect(combination(press("Digit1", { shift: true }))).toBeNull();
    expect(combination(press("Space", { shift: true }))).toBeNull();
  });

  it("takes Shift alone with a function key, which types nothing", () => {
    expect(combination(press("F9", { shift: true }))).toBe("Shift+F9");
    expect(combination(press("F24", { shift: true }))).toBe("Shift+F24");
  });

  it("takes Shift beside a real modifier", () => {
    expect(combination(press("KeyV", { shift: true, meta: true }))).toBe("Shift+Cmd+V");
    expect(combination(press("KeyV", { shift: true, alt: true }))).toBe("Alt+Shift+V");
  });

  it("refuses modifiers without a real key", () => {
    expect(combination(press("ControlLeft", { ctrl: true }))).toBeNull();
    expect(combination(press("AltLeft", { ctrl: true, alt: true }))).toBeNull();
  });
});
