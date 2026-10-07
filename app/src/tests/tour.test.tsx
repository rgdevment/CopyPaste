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

describe("the welcome of a fresh install", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("brings the 2.x over, teaches the shortcut until it is tried and closes at the end", async () => {
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
    await press("Siguiente");

    expect(await screen.findByText("Hay novedades en CopyPaste")).toBeInTheDocument();
    await press("Entendido");
    expect(theWindow.close).toHaveBeenCalled();
  });

  it("a fresh install ends on the essentials and closes with Done", async () => {
    asked.greeting = { kind: "tour", former: false };
    render(<Welcome />);

    await press("Comenzar");
    const shown = await heardOn("panel-shown");
    act(() => shown({ payload: null }));
    await press("Siguiente");
    await press("Siguiente");

    expect(await screen.findByText("Lo esencial")).toBeInTheDocument();
    expect(screen.queryByText("Hay novedades en CopyPaste")).toBeNull();
    await press("Listo");
    expect(theWindow.close).toHaveBeenCalled();
  });

  it("offers the free shortcuts when its own is taken", async () => {
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

  it("skipping from the start closes the window without going through anything", async () => {
    asked.greeting = { kind: "tour", former: false };
    render(<Welcome />);

    await press("Omitir");
    expect(theWindow.close).toHaveBeenCalled();
  });

  it("has no window buttons: you leave with its own", async () => {
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
  // the own property shadows the prototype's getter, so putting the getter back leaves it standing
  Object.defineProperty(navigator, "userAgent", { value: MAC, configurable: true });
  return () => {
    Reflect.deleteProperty(navigator, "userAgent");
  };
}

describe("the welcome on a Mac", () => {
  let back = () => {};

  beforeEach(() => {
    vi.clearAllMocks();
    asked.greeting = { kind: "tour", former: false };
    asked.trust = { offered: true, pastes: false, secureInput: false, clipboard: "allowed" };
    back = asAMac();
  });

  afterEach(() => back());

  it("asks for permission before the shortcut and moves on alone once it is granted", async () => {
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

    asked.trust = { offered: true, pastes: true, secureInput: false, clipboard: "allowed" };
    expect(
      await screen.findByText("Listo, ya puede pegar", {}, { timeout: 4_000 }),
    ).toBeInTheDocument();
    expect(await screen.findByText("Pruébalo ahora", {}, { timeout: 4_000 })).toBeInTheDocument();
  }, 20_000);

  it("tells how to let the clipboard be read when macOS refuses it", async () => {
    asked.trust = { offered: true, pastes: false, secureInput: false, clipboard: "denied" };
    render(<Welcome />);

    await press("Comenzar");
    expect(await screen.findByText("Un permiso para poder pegar")).toBeInTheDocument();
    expect(screen.getByText(/Pegar desde otras apps/)).toBeInTheDocument();
  });

  it("still stops at the permission step when pasting is allowed but reading is refused", async () => {
    asked.trust = { offered: true, pastes: true, secureInput: false, clipboard: "denied" };
    render(<Welcome />);

    await press("Comenzar");
    expect(await screen.findByText("Un permiso para poder pegar")).toBeInTheDocument();
    expect(screen.getByText(/Pegar desde otras apps/)).toBeInTheDocument();
    expect(screen.getByText("Listo, ya puede pegar")).toBeInTheDocument();
    await press("Siguiente");
    expect(await screen.findByText("Pruébalo ahora", {}, { timeout: 4_000 })).toBeInTheDocument();
  });

  it("says nothing about reading while the clipboard is allowed", async () => {
    render(<Welcome />);

    await press("Comenzar");
    expect(await screen.findByText("Un permiso para poder pegar")).toBeInTheDocument();
    expect(screen.queryByText(/Pegar desde otras apps/)).toBeNull();
  });

  it("shows the menu bar and the essentials with the Mac keys", async () => {
    asked.trust = { offered: true, pastes: true, secureInput: false, clipboard: "allowed" };
    render(<Welcome />);

    await press("Comenzar");
    expect(await screen.findByText("Pruébalo ahora")).toBeInTheDocument();
    const shown = await heardOn("panel-shown");
    act(() => shown({ payload: null }));
    await press("Siguiente");

    expect(await screen.findByText("En la barra de menús")).toBeInTheDocument();
    await press("Siguiente");

    expect(await screen.findByText("Lo esencial")).toBeInTheDocument();
    expect(screen.getByText("⏎")).toBeInTheDocument();
    expect(screen.getByText("⇧⏎")).toBeInTheDocument();
    expect(screen.getByText("⌘,")).toBeInTheDocument();
    expect(screen.queryByText("F1")).toBeNull();
  });

  it("with permission already granted the step does not appear", async () => {
    asked.trust = { offered: true, pastes: true, secureInput: false, clipboard: "allowed" };
    render(<Welcome />);

    await press("Comenzar");
    expect(await screen.findByText("Pruébalo ahora")).toBeInTheDocument();
    expect(screen.queryByText("Un permiso para poder pegar")).toBeNull();
  });
});

describe("what changes while the window is open", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("with nothing to show it closes itself instead of staying blank", async () => {
    asked.greeting = { kind: "nothing" };
    render(<Welcome />);

    await waitFor(() => {
      expect(theWindow.close).toHaveBeenCalled();
    });
  });

  it("goes from the news to the tour when asked from About", async () => {
    asked.greeting = { kind: "news", versions: ["3.0.0"] };
    render(<Welcome />);

    expect(await screen.findByText("Hay novedades en CopyPaste")).toBeInTheDocument();
    const greeting = await heardOn("greeting");
    act(() => greeting({ payload: { kind: "tour", former: false } }));
    expect(await screen.findByText("Tu portapapeles, con memoria")).toBeInTheDocument();
  });

  it("follows what another window saved, so as not to overwrite it later", async () => {
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

describe("the news after an update", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("tells what is new in the chosen language and closes with a button", async () => {
    asked.greeting = { kind: "news", versions: ["3.0.0"] };
    asked.locale = "en";
    render(<Welcome />);

    expect(await screen.findByText("CopyPaste has been updated")).toBeInTheDocument();
    expect(screen.getByText("Copy without pasting, and names")).toBeInTheDocument();
    await press("Got it");
    expect(theWindow.close).toHaveBeenCalled();
  });
});

describe("opening the app while it is already running", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("teaches only the shortcut, without the tour, and closes with a button", async () => {
    asked.greeting = { kind: "keys" };
    asked.locale = "en";
    render(<Welcome />);

    expect(await screen.findByText("The shortcut")).toBeInTheDocument();
    expect(screen.queryByText("1 · The shortcut")).toBeNull();
    await press("Close");
    expect(theWindow.close).toHaveBeenCalled();
  });
});
