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

describe("la combinación que el usuario presiona", () => {
  it("se escribe como el sistema la espera", () => {
    expect(combination(press("KeyV", { ctrl: true, alt: true }))).toBe("Ctrl+Alt+V");
    expect(combination(press("F9", { ctrl: true, alt: true }))).toBe("Ctrl+Alt+F9");
    expect(combination(press("Digit1", { ctrl: true, shift: true }))).toBe("Ctrl+Shift+1");
    expect(combination(press("Space", { meta: true, alt: true }))).toBe("Alt+Cmd+Space");
  });

  it("no acepta una tecla suelta, que secuestraría el teclado entero", () => {
    expect(combination(press("KeyV", {}))).toBeNull();
    expect(combination(press("F9", {}))).toBeNull();
  });

  it("no acepta modificadores sin una tecla de verdad", () => {
    expect(combination(press("ControlLeft", { ctrl: true }))).toBeNull();
    expect(combination(press("AltLeft", { ctrl: true, alt: true }))).toBeNull();
  });
});
