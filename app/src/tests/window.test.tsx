import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import App from "../App";

describe("la ventana", () => {
  beforeEach(() => {
    document.documentElement.removeAttribute("data-theme");
  });

  it("abre en General, que es donde está lo que se toca una vez", () => {
    render(<App />);
    expect(screen.getByRole("heading", { level: 1 })).toHaveTextContent("General");
    expect(screen.getByText("Ctrl + Alt + V")).toBeDefined();
  });

  it("ofrece tres secciones y el acerca de, y nada más", () => {
    render(<App />);
    const rail = screen.getByRole("navigation", { name: "Secciones" });
    const says = buttonsIn(rail).map((one) => one.textContent);
    expect(says).toEqual(["General", "Historial", "Copia de seguridad", "Acerca de"]);
  });

  it("cambia de sección al elegirla", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Historial" }));
    expect(screen.getByRole("heading", { level: 1 })).toHaveTextContent("Historial");
    expect(screen.getByLabelText("Guardar durante")).toHaveValue("30");
  });

  it("el tema elegido se escribe en la raíz, que es lo que lo pinta", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.selectOptions(screen.getByLabelText("Tema"), "light");
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
    await who.selectOptions(screen.getByLabelText("Tema"), "");
    expect(document.documentElement.hasAttribute("data-theme")).toBe(false);
  });

  it("el acerca de dice qué es y que no sale de aquí", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Acerca de" }));
    expect(screen.getByText(/no sale de aquí/i)).toBeDefined();
    expect(screen.getByText("Sin nube")).toBeDefined();
  });
});

function buttonsIn(rail: HTMLElement) {
  return Array.from(rail.querySelectorAll("button"));
}
