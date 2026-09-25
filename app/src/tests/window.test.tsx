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

  it("avisa cuando el panel no está funcionando, en vez de callarlo", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const real = vi.mocked(invoke).getMockImplementation();
    vi.mocked(invoke).mockImplementation(((what: string, args?: never) =>
      what === "trouble"
        ? Promise.resolve("no se pudo abrir el historial: base dañada")
        : real?.(what, args)) as never);
    render(<App />);
    expect(await screen.findByText(/no se pudo abrir el historial/)).toBeDefined();
    expect(screen.getByText(/no se está guardando/)).toBeDefined();
    vi.mocked(invoke).mockImplementation(real as never);
  });

  it("no inventa problemas del panel cuando todo va bien", async () => {
    render(<App />);
    await screen.findByRole("button", { name: "General" });
    expect(screen.queryByText(/no se está guardando/)).toBeNull();
  });

  it("dice que el atajo no responde cuando otro programa lo tiene tomado", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const real = vi.mocked(invoke).getMockImplementation();
    vi.mocked(invoke).mockImplementation(((what: string, args?: never) =>
      what === "keys"
        ? Promise.resolve({ wanted: "Ctrl+Alt+V", bound: false })
        : real?.(what, args)) as never);
    render(<App />);
    expect(await screen.findByText(/Otro programa ya usa esa combinación/)).toBeDefined();
    vi.mocked(invoke).mockImplementation(real as never);
  });

  it("calla sobre el atajo cuando el sistema sí lo cedió", async () => {
    render(<App />);
    expect(await screen.findByText("Ctrl + Alt + V")).toBeDefined();
    expect(screen.queryByText(/Otro programa ya usa/)).toBeNull();
  });

  it("un tope de imágenes que no está en la lista se muestra tal cual", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const real = vi.mocked(invoke).getMockImplementation();
    vi.mocked(invoke).mockImplementation(((what: string, args?: never) =>
      what === "settings"
        ? Promise.resolve({
            locale: "es",
            theme: "system",
            shortcut: "Ctrl+Alt+V",
            "hides-when-left": true,
            "keeps-days": 30,
            "images-quota-mb": 700,
          })
        : real?.(what, args)) as never);
    render(<App />);
    await userEvent.click(await screen.findByRole("button", { name: "Historial" }));
    const picked = (await screen.findByLabelText("Espacio del historial")) as HTMLSelectElement;
    expect(picked.value).toBe("700");
    expect(screen.getByRole("option", { name: "700 MB" })).toBeDefined();
    vi.mocked(invoke).mockImplementation(real as never);
  });

  it("los botones de copia de seguridad dicen que todavía no están", async () => {
    render(<App />);
    await userEvent.click(await screen.findByRole("button", { name: "Copia de seguridad" }));
    const exporta = await screen.findByRole("button", { name: "Exportar…" });
    expect(exporta).toBeDisabled();
    expect(screen.getAllByText("Todavía no").length).toBeGreaterThan(0);
  });

  it("cambiar el atajo guarda la combinación que se presiona", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    render(<App />);
    await userEvent.click(await screen.findByRole("button", { name: "Cambiar" }));
    const stop = await screen.findByRole("button", { name: "Dejarlo como está" });
    stop.focus();
    await userEvent.keyboard("{Control>}{Alt>}[F9]{/Alt}{/Control}");
    const kept = vi
      .mocked(invoke)
      .mock.calls.filter(([what]) => what === "keep")
      .pop();
    const said = kept?.[1] as { config: { shortcut: string } } | undefined;
    expect(said?.config.shortcut).toBe("Ctrl+Alt+F9");
  });

  it("vaciar el historial pide confirmación antes de hacerlo", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    render(<App />);
    await userEvent.click(await screen.findByRole("button", { name: "Historial" }));
    const wipe = await screen.findByRole("button", { name: "Vaciar" });
    await userEvent.click(wipe);
    expect(vi.mocked(invoke).mock.calls.some(([what]) => what === "empty")).toBe(false);
    await userEvent.click(await screen.findByRole("button", { name: "¿Seguro?" }));
    expect(vi.mocked(invoke).mock.calls.some(([what]) => what === "empty")).toBe(true);
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
