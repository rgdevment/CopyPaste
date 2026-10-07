import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import { adopt } from "../locales";
import { scene as asked } from "./setup";

describe("the window", () => {
  beforeEach(() => {
    adopt("es");
    document.documentElement.removeAttribute("data-theme");
  });

  it("opens on General, which is where the things touched once live", async () => {
    render(<App />);
    expect(await screen.findByRole("heading", { level: 1 })).toHaveTextContent("General");
    expect(await screen.findByLabelText("Idioma")).toBeDefined();
  });

  it("tells in General how to allow reading the clipboard when macOS refuses it", async () => {
    asked.trust = { offered: true, pastes: true, secureInput: false, clipboard: "denied" };
    render(<App />);
    expect(await screen.findByText(/Pegar desde otras apps/)).toBeInTheDocument();
  });

  it("stays quiet about reading the clipboard when it is allowed", async () => {
    asked.trust = { offered: true, pastes: true, secureInput: false, clipboard: "allowed" };
    render(<App />);
    expect(await screen.findByText("Permiso para pegar")).toBeInTheDocument();
    expect(screen.queryByText(/Pegar desde otras apps/)).toBeNull();
  });

  it("the shortcut and what the panel answers live in their own section", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Atajos de teclado" }));
    expect(screen.getByRole("heading", { level: 1 })).toHaveTextContent("Atajos de teclado");
    expect(await screen.findByText("Ctrl + Alt + V")).toBeDefined();
    expect(screen.getByText("Doble clic")).toBeDefined();
  });

  it("offers four sections and About, and nothing else", () => {
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

  it("switches section when one is chosen", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Historial" }));
    expect(screen.getByRole("heading", { level: 1 })).toHaveTextContent("Historial");
    expect(await screen.findByLabelText("Conservar")).toHaveValue("30");
  });

  it("the chosen theme is written on the root, which is what paints it", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.selectOptions(await screen.findByLabelText("Tema"), "light");
    expect(document.documentElement.getAttribute("data-theme")).toBe("light");
    await who.selectOptions(screen.getByLabelText("Tema"), "system");
    expect(document.documentElement.hasAttribute("data-theme")).toBe(false);
  });

  it("what changes is sent to be saved, it does not stay in the window", async () => {
    const who = userEvent.setup();
    const { invoke } = await import("@tauri-apps/api/core");
    render(<App />);
    await who.selectOptions(await screen.findByLabelText("Tema"), "dark");
    expect(invoke).toHaveBeenCalledWith("keep", {
      config: expect.objectContaining({ theme: "dark" }),
    });
  });

  it("its own title bar really minimizes and closes, it does not just draw them", async () => {
    const who = userEvent.setup();
    const { theWindow } = await import("./setup");
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Minimizar" }));
    expect(theWindow.minimize).toHaveBeenCalled();
    await who.click(screen.getByRole("button", { name: "Cerrar" }));
    expect(theWindow.close).toHaveBeenCalled();
  });

  it("choosing «Forever» is saved as zero, which is what the file understands", async () => {
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

  it("starting with the session is decided by the system, not by the file", async () => {
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

  it("an installed Store copy shows who manages the startup instead of a switch", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const real = vi.mocked(invoke).getMockImplementation();
    vi.mocked(invoke).mockImplementation(((what: string, args?: never) =>
      what === "waking"
        ? Promise.resolve({ offered: true, wakes: false, theirs: false, managed: true })
        : real?.(what, args)) as never);
    render(<App />);
    expect(await screen.findByText(/Windows gestiona el arranque/)).toBeInTheDocument();
    expect(screen.queryByLabelText("Arranca con la sesión")).toBeNull();
    vi.mocked(invoke).mockImplementation(real as never);
  });

  it("warns when the panel is not working, instead of hiding it", async () => {
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

  it("does not invent panel problems when all is well", async () => {
    render(<App />);
    await screen.findByRole("button", { name: "General" });
    expect(screen.queryByText(/no se está guardando/)).toBeNull();
  });

  it("says the shortcut does not answer when another program has taken it", async () => {
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

  it("offers the free combinations and adopts the one pressed", async () => {
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

  it("says nothing about the shortcut when the system did give it up", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Atajos de teclado" }));
    expect(await screen.findByText("Ctrl + Alt + V")).toBeDefined();
    expect(screen.queryByText(/Otro programa ya usa/)).toBeNull();
  });

  it("an image cap that is not in the list is shown as it is", async () => {
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

  it("exporting writes the chosen file and says how much it kept", async () => {
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

  it("importing brings what is missing and does not duplicate what was there", async () => {
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

  it("says what is lost before bringing the 2.x history over", async () => {
    render(<App />);
    await userEvent.click(await screen.findByRole("button", { name: "Copia de seguridad" }));
    expect(await screen.findByText("1200 elementos guardados en CopyPaste 2")).toBeDefined();
    expect(screen.getByText(/260 conservan sus estilos/)).toBeDefined();
    expect(screen.getByText(/El resto llega en plano/)).toBeDefined();
    expect(screen.getByText(/3 imágenes ya no están en el disco/)).toBeDefined();
    expect(screen.getByText(/Nada de CopyPaste 2 se toca ni se borra/)).toBeDefined();
  });

  it("warns about what the time you keep will take as soon as it arrives", async () => {
    render(<App />);
    await userEvent.click(await screen.findByRole("button", { name: "Copia de seguridad" }));
    expect(
      await screen.findByText(/860 elementos son más antiguos que el tiempo que guardas/),
    ).toBeDefined();
    expect(screen.getByText(/un gestor de contraseñas marca en privado/)).toBeDefined();
    expect(screen.getByText(/El panel se detiene mientras cruza/)).toBeDefined();
  });

  it("brings the history over and tells what arrived and what did not", async () => {
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

  it("once crossed, the 2.x stops being offered and says when it was", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const real = vi.mocked(invoke).getMockImplementation();
    vi.mocked(invoke).mockImplementation(async (what: string, args?: unknown) => {
      const said = await (real as (a: string, b?: unknown) => Promise<unknown>)(what, args);
      if (what !== "former") return said;
      return {
        ...(said as object),
        came: 1180,
        cameStill: 1180,
        cameAt: Date.UTC(2026, 9, 2, 12),
      };
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

  it("deleting the 2.x data asks for confirmation before touching anything", async () => {
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

  it("changing the shortcut saves the combination pressed", async () => {
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

  it("emptying the history asks for confirmation before doing it", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    render(<App />);
    await userEvent.click(await screen.findByRole("button", { name: "Historial" }));
    const wipe = await screen.findByRole("button", { name: "Vaciar" });
    await userEvent.click(wipe);
    expect(vi.mocked(invoke).mock.calls.some(([what]) => what === "empty")).toBe(false);
    await userEvent.click(await screen.findByRole("button", { name: "¿Seguro?" }));
    expect(vi.mocked(invoke).mock.calls.some(([what]) => what === "empty")).toBe(true);
  });

  it("does not claim to be up to date while nobody has checked", async () => {
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

  it("a Store copy claims nothing: the Store takes care of it", async () => {
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

  it("says you are up to date only after having looked", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Acerca de" }));
    expect(await screen.findByText("Estás en la última versión")).toBeDefined();
    expect(document.querySelector(".pip.ok")).not.toBeNull();
  });

  it("offers to install the version it found, and installs it", async () => {
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

  it("if the version is gone, it says so and stops offering it", async () => {
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
        return Promise.reject("gone");
      }
      return (real as (a: string, b?: unknown) => Promise<unknown>)(what, args);
    });
    try {
      const who = userEvent.setup();
      render(<App />);
      await who.click(screen.getByRole("button", { name: "Acerca de" }));
      await who.click(await screen.findByRole("button", { name: "Actualizar" }));
      expect(await screen.findByText(/ya no está disponible/)).toBeDefined();
      expect(screen.queryByRole("button", { name: "Actualizar" })).toBeNull();
    } finally {
      vi.mocked(invoke).mockImplementation(real as never);
    }
  });

  it("a check that fails does not leave the dot green", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const real = vi.mocked(invoke).getMockImplementation();
    vi.mocked(invoke).mockImplementation((what: string, args?: unknown) => {
      if (what === "update_ready") return Promise.reject("offline");
      return (real as (a: string, b?: unknown) => Promise<unknown>)(what, args);
    });
    try {
      render(<App />);
      await userEvent.click(screen.getByRole("button", { name: "Acerca de" }));
      expect(await screen.findByText(/Revisa tu conexión/)).toBeDefined();
      expect(screen.queryByText("Estás en la última versión")).toBeNull();
      expect(document.querySelector(".pip.ok")).toBeNull();
    } finally {
      vi.mocked(invoke).mockImplementation(real as never);
    }
  });

  it("the version comes from the program itself, not from text written by hand", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Acerca de" }));
    expect(await screen.findByText("3.0.0")).toBeDefined();
  });

  it("a link that cannot be opened is said, not swallowed", async () => {
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

  it("links go out through the same door, the one that knows about LinkUnbound", async () => {
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

  it("without LinkUnbound it is recommended, and with it it only says links already open through it", async () => {
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

  it("the backup warns about the 2.x and what is lost by bringing it over", async () => {
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Copia de seguridad" }));
    expect(await screen.findByText(/214\.0 MB/)).toBeDefined();
    expect(screen.getByText(/El resto llega en plano/)).toBeDefined();
  });

  it("choosing English changes the whole window, not just the language row", async () => {
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

  it("About says what it is and that nothing leaves this machine", async () => {
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
