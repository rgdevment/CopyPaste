import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import { adopt } from "../locales";

describe("la ventana", () => {
  beforeEach(() => {
    adopt("es");
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
    expect(await screen.findByLabelText("Conservar")).toHaveValue("30");
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

  it("la barra propia minimiza y cierra de verdad, no solo lo dibuja", async () => {
    const who = userEvent.setup();
    const { theWindow } = await import("./setup");
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Minimizar" }));
    expect(theWindow.minimize).toHaveBeenCalled();
    await who.click(screen.getByRole("button", { name: "Cerrar" }));
    expect(theWindow.close).toHaveBeenCalled();
  });

  it("elegir «Siempre» se guarda como cero, que es lo que el archivo entiende", async () => {
    const who = userEvent.setup();
    const { invoke } = await import("@tauri-apps/api/core");
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Historial" }));
    await who.selectOptions(await screen.findByLabelText("Conservar"), "0");
    expect(invoke).toHaveBeenCalledWith("keep", {
      config: expect.objectContaining({ "keeps-days": 0 }),
    });
    expect(screen.queryByText(/invalid type/)).toBeNull();
  });

  it("el arranque con la sesión lo decide el sistema, no el archivo", async () => {
    const who = userEvent.setup();
    const { invoke } = await import("@tauri-apps/api/core");
    render(<App />);
    const knob = await screen.findByLabelText("Arranca con la sesión");
    expect(knob).toHaveAttribute("aria-checked", "false");
    await who.click(knob);
    expect(invoke).toHaveBeenCalledWith("wake", { wanted: true });
    expect(await screen.findByLabelText("Arranca con la sesión")).toHaveAttribute(
      "aria-checked",
      "true",
    );
  });

  it("no afirma que está actualizada cuando nadie lo ha comprobado", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Acerca de" }));
    expect(await screen.findByText(/Todavía no busca actualizaciones/)).toBeDefined();
    expect(screen.getByRole("button", { name: /Buscar ahora/ })).toBeDisabled();
    expect(screen.queryByText("Recibir versiones de prueba")).toBeNull();
  });

  it("la versión sale del propio programa, no de un texto escrito a mano", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Acerca de" }));
    expect(await screen.findByText("3.0.0")).toBeDefined();
  });

  it("un enlace que no se puede abrir se dice, no se traga", async () => {
    const who = userEvent.setup();
    const { openUrl } = await import("@tauri-apps/plugin-opener");
    vi.mocked(openUrl).mockRejectedValueOnce(new Error("forbidden"));
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Acerca de" }));
    await who.click(await screen.findByRole("button", { name: /Dale una estrella/ }));
    expect(await screen.findByText(/No se pudo abrir/)).toBeDefined();
  });

  it("la copia de seguridad avisa de la 2 y de lo que se pierde al traerla", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Copia de seguridad" }));
    expect(await screen.findByText(/214\.0 MB/)).toBeDefined();
    expect(screen.getByText(/empezar de cero es lo recomendado/i)).toBeDefined();
  });

  it("elegir English cambia la ventana entera, no solo la fila del idioma", async () => {
    const who = userEvent.setup();
    const { invoke } = await import("@tauri-apps/api/core");
    render(<App />);
    await who.selectOptions(await screen.findByLabelText("Idioma"), "en");
    expect(await screen.findByRole("button", { name: "History" })).toBeDefined();
    expect(screen.getByLabelText("Theme")).toBeDefined();
    await who.click(screen.getByRole("button", { name: "About" }));
    expect(screen.getByText("All local")).toBeDefined();
    expect(invoke).toHaveBeenCalledWith("relabel", { locale: "en" });
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
