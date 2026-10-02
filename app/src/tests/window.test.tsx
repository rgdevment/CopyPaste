import { cleanup, render, screen } from "@testing-library/react";
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
    expect(await screen.findByLabelText("Idioma")).toBeDefined();
  });

  it("el atajo y lo que responde el panel viven en su propia sección", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Atajos de teclado" }));
    expect(screen.getByRole("heading", { level: 1 })).toHaveTextContent("Atajos de teclado");
    expect(await screen.findByText("Ctrl + Alt + V")).toBeDefined();
    expect(screen.getByText("Doble clic")).toBeDefined();
  });

  it("ofrece cuatro secciones y el acerca de, y nada más", () => {
    render(<App />);
    const rail = screen.getByRole("navigation", { name: "Secciones" });
    const says = buttonsIn(rail).map((one) => one.textContent);
    expect(says).toEqual([
      "General",
      "Atajos de teclado",
      "Historial",
      "Copia de seguridad",
      "Acerca de",
    ]);
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
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Atajos de teclado" }));
    expect(await screen.findByText(/Otro programa ya usa esa combinación/)).toBeDefined();
    vi.mocked(invoke).mockImplementation(real as never);
  });

  it("ofrece las combinaciones libres y adopta la que se pulsa", async () => {
    const who = userEvent.setup();
    const { invoke } = await import("@tauri-apps/api/core");
    const real = vi.mocked(invoke).getMockImplementation();
    vi.mocked(invoke).mockImplementation(((what: string, args?: never) =>
      what === "keys"
        ? Promise.resolve({ wanted: "Ctrl+Alt+V", bound: false })
        : real?.(what, args)) as never);
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Atajos de teclado" }));
    expect(await screen.findByText(/Estas están libres ahora mismo/)).toBeDefined();
    const offered = await screen.findByRole("button", { name: "Ctrl + Shift + V" });
    await who.click(offered);
    const asked = vi
      .mocked(invoke)
      .mock.calls.filter(([what]) => what === "keep")
      .pop();
    expect(asked).toBeDefined();
    const sent = asked?.[1] as { config: { shortcut: string } } | undefined;
    expect(sent?.config.shortcut).toBe("Ctrl+Shift+V");
    vi.mocked(invoke).mockImplementation(real as never);
  });

  it("calla sobre el atajo cuando el sistema sí lo cedió", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Atajos de teclado" }));
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

  it("exportar escribe el archivo que se elija y dice cuánto guardó", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const { save } = await import("@tauri-apps/plugin-dialog");
    render(<App />);
    await userEvent.click(await screen.findByRole("button", { name: "Copia de seguridad" }));
    await userEvent.click(await screen.findByRole("button", { name: "Exportar…" }));
    expect(save).toHaveBeenCalled();
    expect(invoke).toHaveBeenCalledWith(
      "save_backup",
      expect.objectContaining({ path: "/donde/quiera/CopyPaste.cpbackup" }),
    );
    expect(await screen.findByText("Guardado: 3 elementos")).toBeDefined();
  });

  it("importar trae lo que falta y no duplica lo que ya estaba", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    render(<App />);
    await userEvent.click(await screen.findByRole("button", { name: "Copia de seguridad" }));
    await userEvent.click(await screen.findByRole("button", { name: "Elegir archivo…" }));
    expect(invoke).toHaveBeenCalledWith(
      "load_backup",
      expect.objectContaining({ path: "/donde/quiera/CopyPaste.cpbackup" }),
    );
    expect(
      await screen.findByText("Llegaron 2 elementos · 1 elemento ya estaban y no se duplicaron"),
    ).toBeDefined();
  });

  it("dice lo que se pierde antes de traer el historial de la 2", async () => {
    render(<App />);
    await userEvent.click(await screen.findByRole("button", { name: "Copia de seguridad" }));
    expect(await screen.findByText("1200 elementos guardados en CopyPaste 2")).toBeDefined();
    expect(screen.getByText(/260 conservan sus estilos/)).toBeDefined();
    expect(screen.getByText(/El resto llega en plano/)).toBeDefined();
    expect(screen.getByText(/3 imágenes ya no están en el disco/)).toBeDefined();
    expect(screen.getByText(/Nada de CopyPaste 2 se toca ni se borra/)).toBeDefined();
  });

  it("avisa de lo que el tiempo que guardas se llevará apenas llegue", async () => {
    render(<App />);
    await userEvent.click(await screen.findByRole("button", { name: "Copia de seguridad" }));
    expect(
      await screen.findByText(/860 elementos son más antiguos que el tiempo que guardas/),
    ).toBeDefined();
    expect(screen.getByText(/un gestor de contraseñas marca en privado/)).toBeDefined();
    expect(screen.getByText(/El panel se detiene mientras cruza/)).toBeDefined();
  });

  it("trae el historial y cuenta lo que llegó y lo que no", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const who = userEvent.setup();
    render(<App />);
    await who.click(await screen.findByRole("button", { name: "Copia de seguridad" }));
    await who.click(await screen.findByRole("button", { name: "Traer el historial" }));
    expect(invoke).toHaveBeenCalledWith("bring_former", expect.objectContaining({}));
    expect(
      await screen.findByText(
        "Llegaron 1180 elementos · 3 elementos sin su imagen · 860 elementos se fueron por el tiempo que guardas",
      ),
    ).toBeDefined();
  });

  it("una vez cruzada, la 2 deja de ofrecerse y dice cuándo fue", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const real = vi.mocked(invoke).getMockImplementation();
    vi.mocked(invoke).mockImplementation(async (what: string, args?: unknown) => {
      const said = await (real as (a: string, b?: unknown) => Promise<unknown>)(what, args);
      if (what !== "former") return said;
      return { ...(said as object), came: 1180, cameAt: Date.UTC(2026, 9, 2, 12) };
    });
    try {
      const who = userEvent.setup();
      render(<App />);
      await who.click(await screen.findByRole("button", { name: "Copia de seguridad" }));

      expect(await screen.findByText(/Ya lo trajiste el/)).toBeDefined();
      expect(screen.getByText(/1180 elementos de CopyPaste 2 siguen aquí/)).toBeDefined();
      expect(screen.getByText(/Lo que borraste desde entonces no vuelve/)).toBeDefined();
      expect(screen.queryByRole("button", { name: "Traer el historial" })).toBeNull();
      expect(await screen.findByRole("button", { name: "Volver a revisar" })).toBeDefined();
    } finally {
      vi.mocked(invoke).mockImplementation(real as never);
    }
  });

  it("borrar los datos de la 2 pide confirmación antes de tocar nada", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const real = vi.mocked(invoke).getMockImplementation();
    vi.mocked(invoke).mockImplementation((what: string, args?: unknown) => {
      if (what === "drop_former") return Promise.resolve({ files: 41, bytes: 900 });
      return (real as (a: string, b?: unknown) => Promise<unknown>)(what, args);
    });
    try {
      const who = userEvent.setup();
      render(<App />);
      await who.click(await screen.findByRole("button", { name: "Copia de seguridad" }));
      const button = await screen.findByRole("button", {
        name: "Borrar los datos de CopyPaste 2",
      });
      await who.click(button);
      expect(invoke).not.toHaveBeenCalledWith("drop_former");
      expect(screen.getByText(/Los archivos que copiaste no se tocan/)).toBeDefined();

      await who.click(await screen.findByRole("button", { name: "Sí, borrarlos" }));
      expect(invoke).toHaveBeenCalledWith("drop_former");
      expect(await screen.findByText("Borrado: 41 archivos")).toBeDefined();
    } finally {
      vi.mocked(invoke).mockImplementation(real as never);
    }
  });

  it("cambiar el atajo guarda la combinación que se presiona", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    render(<App />);
    await userEvent.click(screen.getByRole("button", { name: "Atajos de teclado" }));
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

  it("no afirma que está actualizada mientras nadie lo ha comprobado", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const real = vi.mocked(invoke).getMockImplementation();
    vi.mocked(invoke).mockImplementation((what: string, args?: unknown) => {
      if (what === "update_ready") return new Promise(() => {});
      return (real as (a: string, b?: unknown) => Promise<unknown>)(what, args);
    });
    try {
      const who = userEvent.setup();
      render(<App />);
      await who.click(screen.getByRole("button", { name: "Acerca de" }));
      expect(await screen.findByText(/Se mira una vez al día/)).toBeDefined();
      expect(screen.queryByText("Estás en la última versión")).toBeNull();
      expect(document.querySelector(".pip.ok")).toBeNull();
      expect(screen.queryByText("Recibir versiones de prueba")).toBeNull();
    } finally {
      vi.mocked(invoke).mockImplementation(real as never);
    }
  });

  it("una copia de la Store no afirma nada: la Store se encarga", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const real = vi.mocked(invoke).getMockImplementation();
    vi.mocked(invoke).mockImplementation((what: string, args?: unknown) => {
      if (what === "update_ready") {
        return Promise.resolve({ route: "store", looked: false, ready: null });
      }
      return (real as (a: string, b?: unknown) => Promise<unknown>)(what, args);
    });
    try {
      render(<App />);
      await userEvent.click(screen.getByRole("button", { name: "Acerca de" }));
      expect(await screen.findByText("La Microsoft Store se encarga")).toBeDefined();
      expect(screen.queryByText("Estás en la última versión")).toBeNull();
      expect(document.querySelector(".pip.ok")).toBeNull();
      expect(screen.queryByRole("button", { name: "Buscar ahora" })).toBeNull();
    } finally {
      vi.mocked(invoke).mockImplementation(real as never);
    }
  });

  it("dice que estás al día solo después de haber mirado", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Acerca de" }));
    expect(await screen.findByText("Estás en la última versión")).toBeDefined();
    expect(document.querySelector(".pip.ok")).not.toBeNull();
  });

  it("ofrece instalar la versión que encontró, y la instala", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const real = vi.mocked(invoke).getMockImplementation();
    vi.mocked(invoke).mockImplementation((what: string, args?: unknown) => {
      if (what === "update_ready") {
        return Promise.resolve({
          route: "download",
          looked: true,
          ready: { version: "3.1.0", installs: true },
        });
      }
      return (real as (a: string, b?: unknown) => Promise<unknown>)(what, args);
    });
    try {
      const who = userEvent.setup();
      render(<App />);
      await who.click(screen.getByRole("button", { name: "Acerca de" }));
      expect(await screen.findByText("La versión 3.1.0 ya está disponible")).toBeDefined();
      await who.click(screen.getByRole("button", { name: "Actualizar" }));
      expect(invoke).toHaveBeenCalledWith("update_install");
    } finally {
      vi.mocked(invoke).mockImplementation(real as never);
    }
  });

  it("si la instalación falla, se ve por qué y deja de ofrecerla", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const real = vi.mocked(invoke).getMockImplementation();
    vi.mocked(invoke).mockImplementation((what: string, args?: unknown) => {
      if (what === "update_ready") {
        return Promise.resolve({
          route: "download",
          looked: true,
          ready: { version: "3.1.0", installs: true },
        });
      }
      if (what === "update_install") {
        return Promise.reject(new Error("3.1.0 is not on the feed any more"));
      }
      return (real as (a: string, b?: unknown) => Promise<unknown>)(what, args);
    });
    try {
      const who = userEvent.setup();
      render(<App />);
      await who.click(screen.getByRole("button", { name: "Acerca de" }));
      await who.click(await screen.findByRole("button", { name: "Actualizar" }));
      expect(await screen.findByText(/is not on the feed any more/)).toBeDefined();
      expect(screen.queryByRole("button", { name: "Actualizar" })).toBeNull();
    } finally {
      vi.mocked(invoke).mockImplementation(real as never);
    }
  });

  it("una comprobación que falla no deja el punto en verde", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const real = vi.mocked(invoke).getMockImplementation();
    vi.mocked(invoke).mockImplementation((what: string, args?: unknown) => {
      if (what === "update_ready") return Promise.reject(new Error("no hay red"));
      return (real as (a: string, b?: unknown) => Promise<unknown>)(what, args);
    });
    try {
      render(<App />);
      await userEvent.click(screen.getByRole("button", { name: "Acerca de" }));
      expect(await screen.findByText(/no hay red/)).toBeDefined();
      expect(screen.queryByText("Estás en la última versión")).toBeNull();
      expect(document.querySelector(".pip.ok")).toBeNull();
    } finally {
      vi.mocked(invoke).mockImplementation(real as never);
    }
  });

  it("con brew no ofrece instalar: dice el comando", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const real = vi.mocked(invoke).getMockImplementation();
    vi.mocked(invoke).mockImplementation((what: string, args?: unknown) => {
      if (what === "update_ready") {
        return Promise.resolve({
          route: "brew",
          looked: true,
          ready: { version: "3.1.0", installs: false },
        });
      }
      return (real as (a: string, b?: unknown) => Promise<unknown>)(what, args);
    });
    try {
      render(<App />);
      await userEvent.click(screen.getByRole("button", { name: "Acerca de" }));
      expect(await screen.findByText(/brew upgrade --cask copypaste/)).toBeDefined();
      expect(screen.queryByRole("button", { name: "Actualizar" })).toBeNull();
    } finally {
      vi.mocked(invoke).mockImplementation(real as never);
    }
  });

  it("la versión sale del propio programa, no de un texto escrito a mano", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Acerca de" }));
    expect(await screen.findByText("3.0.0")).toBeDefined();
  });

  it("un enlace que no se puede abrir se dice, no se traga", async () => {
    const who = userEvent.setup();
    const { invoke } = await import("@tauri-apps/api/core");
    const real = vi.mocked(invoke).getMockImplementation();
    vi.mocked(invoke).mockImplementation((what: string, args?: unknown) => {
      if (what === "open_web") return Promise.reject(new Error("forbidden"));
      return (real as (a: string, b?: unknown) => Promise<unknown>)(what, args);
    });
    try {
      render(<App />);
      await who.click(screen.getByRole("button", { name: "Acerca de" }));
      await who.click(await screen.findByRole("button", { name: /Dale una estrella/ }));
      expect(await screen.findByText(/No se pudo abrir/)).toBeDefined();
    } finally {
      vi.mocked(invoke).mockImplementation(real as never);
    }
  });

  it("los enlaces salen por la misma puerta, la que sabe de LinkUnbound", async () => {
    const who = userEvent.setup();
    const { invoke } = await import("@tauri-apps/api/core");
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Acerca de" }));
    await who.click(await screen.findByRole("button", { name: /Dale una estrella/ }));
    expect(invoke).toHaveBeenCalledWith(
      "open_web",
      expect.objectContaining({ url: "https://github.com/rgdevment/CopyPaste" }),
    );
  });

  it("sin LinkUnbound se le recomienda, y con él solo se dice que ya abre por ahí", async () => {
    const who = userEvent.setup();
    const { invoke } = await import("@tauri-apps/api/core");
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Acerca de" }));
    expect(await screen.findByText(/Elige con qué navegador se abre cada enlace/)).toBeDefined();
    expect(screen.queryByText(/tus enlaces salen por él/)).toBeNull();

    const real = vi.mocked(invoke).getMockImplementation();
    vi.mocked(invoke).mockImplementation((what: string, args?: unknown) => {
      if (what === "linkunbound_here") return Promise.resolve(true);
      return (real as (a: string, b?: unknown) => Promise<unknown>)(what, args);
    });
    try {
      cleanup();
      render(<App />);
      await who.click(screen.getByRole("button", { name: "Acerca de" }));
      expect(await screen.findByText(/tus enlaces salen por él/)).toBeDefined();
      expect(screen.queryByText(/Elige con qué navegador se abre cada enlace/)).toBeNull();
    } finally {
      vi.mocked(invoke).mockImplementation(real as never);
    }
  });

  it("la copia de seguridad avisa de la 2 y de lo que se pierde al traerla", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Copia de seguridad" }));
    expect(await screen.findByText(/214\.0 MB/)).toBeDefined();
    expect(screen.getByText(/El resto llega en plano/)).toBeDefined();
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
