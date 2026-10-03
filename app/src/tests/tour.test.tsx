import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import Welcome from "../ui/Welcome";
import { scene as asked, theWindow } from "./setup";

type Heard = (event: { payload: unknown }) => void;

async function heardOn(name: string): Promise<Heard> {
  const { listen } = await import("@tauri-apps/api/event");
  await waitFor(() => {
    expect(vi.mocked(listen).mock.calls.some(([what]) => what === name)).toBe(true);
  });
  const call = vi.mocked(listen).mock.calls.find(([what]) => what === name) as [string, Heard];
  return call[1];
}

async function press(name: string) {
  fireEvent.click(await screen.findByRole("button", { name }));
}

describe("la bienvenida de una instalación nueva", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("trae la 2.x, enseña el atajo hasta que se prueba y se cierra al final", async () => {
    render(<Welcome />);

    expect(await screen.findByText("Tu historial anterior te espera")).toBeInTheDocument();
    await press("Importar historial");
    expect(
      await screen.findByText("Listo: 1180 elementos ya están en tu historial."),
    ).toBeInTheDocument();
    await press("Siguiente");

    expect(await screen.findByText("Tu portapapeles, con memoria")).toBeInTheDocument();
    await press("Comenzar");

    expect(await screen.findByText("Pruébalo ahora")).toBeInTheDocument();
    expect(screen.getByText("Esperando que lo pruebes…")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Siguiente" })).toBeNull();
    const shown = await heardOn("panel-shown");
    act(() => shown({ payload: null }));
    expect(await screen.findByText("¡Listo! Así se abre")).toBeInTheDocument();
    await press("Siguiente");

    expect(await screen.findByText("Junto al reloj")).toBeInTheDocument();
    await press("Siguiente");

    expect(await screen.findByText("Lo esencial")).toBeInTheDocument();
    await press("Listo");
    expect(theWindow.close).toHaveBeenCalled();
  });

  it("ofrece los atajos libres cuando el suyo está ocupado", async () => {
    asked.greeting = { kind: "tour", former: false };
    asked.bound = false;
    const { invoke } = await import("@tauri-apps/api/core");
    render(<Welcome />);

    await press("Comenzar");
    expect(await screen.findByText("Ese atajo ya está en uso")).toBeInTheDocument();
    await press("Ctrl + Shift + V");
    await waitFor(() => {
      expect(vi.mocked(invoke)).toHaveBeenCalledWith(
        "keep",
        expect.objectContaining({ config: expect.objectContaining({ shortcut: "Ctrl+Shift+V" }) }),
      );
    });
  });

  it("omitir desde el principio cierra la ventana sin recorrer nada", async () => {
    asked.greeting = { kind: "tour", former: false };
    render(<Welcome />);

    await press("Omitir");
    expect(theWindow.close).toHaveBeenCalled();
  });

  it("no tiene botones de ventana: se sale con los suyos", async () => {
    asked.greeting = { kind: "tour", former: false };
    render(<Welcome />);

    await screen.findByText("Tu portapapeles, con memoria");
    expect(screen.queryByRole("button", { name: "Minimizar" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Cerrar" })).toBeNull();
  });
});

const MAC =
  "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)";

function asAMac() {
  const was = Object.getOwnPropertyDescriptor(Navigator.prototype, "userAgent");
  Object.defineProperty(navigator, "userAgent", { value: MAC, configurable: true });
  return () => {
    if (was) {
      Object.defineProperty(Navigator.prototype, "userAgent", was);
    }
  };
}

describe("la bienvenida en un Mac", () => {
  let back = () => {};

  beforeEach(() => {
    vi.clearAllMocks();
    asked.greeting = { kind: "tour", former: false };
    asked.trust = { offered: true, pastes: false, secureInput: false };
    back = asAMac();
  });

  afterEach(() => back());

  it("pide el permiso antes del atajo y sigue sola cuando se concede", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    render(<Welcome />);

    await press("Comenzar");
    expect(await screen.findByText("Un permiso para poder pegar")).toBeInTheDocument();
    expect(
      screen.getByText(
        "1 · Abrimos Configuración del Sistema › Privacidad y seguridad › Accesibilidad",
      ),
    ).toBeInTheDocument();

    await press("Dar permiso");
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("ask_trust");
    expect(await screen.findByText("Esperando el permiso…")).toBeInTheDocument();
    expect(
      screen.getByText(
        "Activa CopyPaste en Accesibilidad. Si ya aparecía activado y no responde, quítalo con − y agrégalo de nuevo.",
      ),
    ).toBeInTheDocument();

    asked.trust = { offered: true, pastes: true, secureInput: false };
    expect(
      await screen.findByText("Listo, ya puede pegar", {}, { timeout: 4_000 }),
    ).toBeInTheDocument();
    expect(await screen.findByText("Pruébalo ahora", {}, { timeout: 4_000 })).toBeInTheDocument();
  }, 20_000);

  it("enseña la barra de menús y lo esencial con las teclas del Mac", async () => {
    asked.trust = { offered: true, pastes: true, secureInput: false };
    render(<Welcome />);

    await press("Comenzar");
    expect(await screen.findByText("Pruébalo ahora")).toBeInTheDocument();
    const shown = await heardOn("panel-shown");
    act(() => shown({ payload: null }));
    await press("Siguiente");

    expect(await screen.findByText("En la barra de menús")).toBeInTheDocument();
    await press("Siguiente");

    expect(await screen.findByText("Lo esencial")).toBeInTheDocument();
    expect(screen.getByText("↩")).toBeInTheDocument();
    expect(screen.getByText("⇧↩")).toBeInTheDocument();
    expect(screen.getByText("⌘,")).toBeInTheDocument();
    expect(screen.queryByText("F1")).toBeNull();
  });

  it("con el permiso ya concedido el paso no aparece", async () => {
    asked.trust = { offered: true, pastes: true, secureInput: false };
    render(<Welcome />);

    await press("Comenzar");
    expect(await screen.findByText("Pruébalo ahora")).toBeInTheDocument();
    expect(screen.queryByText("Un permiso para poder pegar")).toBeNull();
  });
});

describe("lo que cambia con la ventana abierta", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("sin nada que mostrar se cierra sola en vez de quedar en blanco", async () => {
    asked.greeting = { kind: "nothing" };
    render(<Welcome />);

    await waitFor(() => {
      expect(theWindow.close).toHaveBeenCalled();
    });
  });

  it("pasa de las novedades al recorrido cuando se pide desde Acerca de", async () => {
    asked.greeting = { kind: "news", versions: ["3.0.0"] };
    render(<Welcome />);

    expect(await screen.findByText("Hay novedades en CopyPaste")).toBeInTheDocument();
    const greeting = await heardOn("greeting");
    act(() => greeting({ payload: { kind: "tour", former: false } }));
    expect(await screen.findByText("Tu portapapeles, con memoria")).toBeInTheDocument();
  });

  it("sigue lo que otra ventana guardó, para no pisarlo después", async () => {
    asked.greeting = { kind: "tour", former: false };
    render(<Welcome />);

    expect(await screen.findByText("Tu portapapeles, con memoria")).toBeInTheDocument();
    const kept = await heardOn("kept");
    act(() =>
      kept({
        payload: {
          locale: "en",
          theme: "system",
          shortcut: "Ctrl+Alt+V",
          "hides-when-left": true,
          "keeps-days": 30,
          "images-quota-mb": 0,
        },
      }),
    );
    expect(await screen.findByText("Your clipboard, with a memory")).toBeInTheDocument();
  });
});

describe("las novedades tras una actualización", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("dicen lo nuevo en el idioma elegido y se cierran con un botón", async () => {
    asked.greeting = { kind: "news", versions: ["3.0.0"] };
    asked.locale = "en";
    render(<Welcome />);

    expect(await screen.findByText("CopyPaste has been updated")).toBeInTheDocument();
    expect(screen.getByText("Copy without pasting")).toBeInTheDocument();
    await press("Got it");
    expect(theWindow.close).toHaveBeenCalled();
  });
});
