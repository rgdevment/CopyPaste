import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import App from "../App";

type Invoke = (what: string, args?: never) => Promise<unknown>;

async function insteadOf(handle: (what: string, real?: Invoke) => Promise<unknown> | null) {
  const { invoke } = await import("@tauri-apps/api/core");
  const real = vi.mocked(invoke).getMockImplementation() as Invoke;
  vi.mocked(invoke).mockImplementation(((what: string, args?: never) => {
    const mine = handle(what, real);
    return mine ?? real(what, args);
  }) as never);
  return () => vi.mocked(invoke).mockImplementation(real as never);
}

describe("cuando el backend dice que no", () => {
  const undo: (() => void)[] = [];

  afterEach(() => {
    while (undo.length > 0) {
      undo.pop()?.();
    }
  });

  it("muestra el motivo y vuelve a leer lo que quedó guardado", async () => {
    const who = userEvent.setup();
    undo.push(
      await insteadOf((what) =>
        what === "keep" ? Promise.reject(new Error("el sistema no cedió esa tecla")) : null,
      ),
    );
    render(<App />);
    const look = await screen.findByLabelText("Tema");
    const { invoke } = await import("@tauri-apps/api/core");
    const asked = vi.mocked(invoke).mock.calls.filter(([what]) => what === "settings").length;

    await who.selectOptions(look, "dark");

    expect(await screen.findByText(/el sistema no cedió esa tecla/)).toBeDefined();
    await waitFor(() => {
      expect(
        vi.mocked(invoke).mock.calls.filter(([what]) => what === "settings").length,
      ).toBeGreaterThan(asked);
    });
  });

  it("calla sobre el atajo si ni siquiera puede preguntar", async () => {
    undo.push(
      await insteadOf((what) => (what === "keys" ? Promise.reject(new Error("no answer")) : null)),
    );
    render(<App />);
    expect(await screen.findByText("Atajo del panel")).toBeDefined();
    expect(screen.queryByText(/Otro programa ya usa/)).toBeNull();
  });

  it("avisa del atajo tomado aunque no consiga proponer otros", async () => {
    undo.push(
      await insteadOf((what) => {
        if (what === "keys") {
          return Promise.resolve({ wanted: "Ctrl+Alt+V", bound: false });
        }
        return what === "spare" ? Promise.reject(new Error("no answer")) : null;
      }),
    );
    render(<App />);
    expect(await screen.findByText(/Otro programa ya usa esa combinación/)).toBeDefined();
    expect(screen.queryByText(/Estas están libres/)).toBeNull();
  });
});
