import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it } from "vitest";
import App from "../App";

describe("la ventana", () => {
  beforeEach(() => {
    document.documentElement.removeAttribute("data-theme");
  });

  it("abre en General, que es donde está lo que se toca una vez", async () => {
    render(<App />);
    expect(await screen.findByRole("heading", { level: 1 })).toHaveTextContent("General");
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
    expect(await screen.findByLabelText("Guardar durante")).toHaveValue("30");
  });

  it("el tema elegido se escribe en la raíz, que es lo que lo pinta", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.selectOptions(await screen.findByLabelText("Tema"), "light");
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
    await who.selectOptions(screen.getByLabelText("Tema"), "system");
    expect(document.documentElement.hasAttribute("data-theme")).toBe(false);
  });

  it("lo que se cambia se manda a guardar, no se queda en la ventana", async () => {
    const who = userEvent.setup();
    const { invoke } = await import("@tauri-apps/api/core");
    render(<App />);
    await who.selectOptions(await screen.findByLabelText("Tema"), "dark");
    expect(invoke).toHaveBeenCalledWith("keep", {
      config: expect.objectContaining({ theme: "dark" }),
    });
  });

  it("la barra propia ofrece minimizar y cerrar, que es lo que la nativa daba", () => {
    render(<App />);
    expect(screen.getByRole("button", { name: "Minimizar" })).toBeDefined();
    expect(screen.getByRole("button", { name: "Cerrar" })).toBeDefined();
  });

  it("la copia de seguridad avisa de la 2 y de lo que se pierde al traerla", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Copia de seguridad" }));
    expect(await screen.findByText(/214\.0 MB/)).toBeDefined();
    expect(screen.getByText(/empezar de cero es lo recomendado/i)).toBeDefined();
  });

  it("el acerca de dice qué es y que no sale de aquí", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Acerca de" }));
    expect(screen.getByText(/sin telemetría/i)).toBeDefined();
    expect(screen.getByText("Todo local")).toBeDefined();
    expect(screen.getByText("Sin nube")).toBeDefined();
  });
});

function buttonsIn(rail: HTMLElement) {
  return Array.from(rail.querySelectorAll("button"));
}
