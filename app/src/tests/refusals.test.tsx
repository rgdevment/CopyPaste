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

describe("when the backend says no", () => {
  const undo: (() => void)[] = [];

  afterEach(() => {
    while (undo.length > 0) {
      undo.pop()?.();
    }
  });

  it("shows the reason and reads back what was saved", async () => {
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

  it("says nothing about the shortcut if it cannot even ask", async () => {
    undo.push(
      await insteadOf((what) => (what === "keys" ? Promise.reject(new Error("no answer")) : null)),
    );
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Atajos de teclado" }));
    expect(await screen.findByText("Atajo del panel")).toBeDefined();
    expect(screen.queryByText(/Otro programa ya usa/)).toBeNull();
  });

  it("warns the shortcut is taken even when it cannot offer others", async () => {
    undo.push(
      await insteadOf((what) => {
        if (what === "keys") {
          return Promise.resolve({ wanted: "Ctrl+Alt+V", bound: false });
        }
        return what === "spare" ? Promise.reject(new Error("no answer")) : null;
      }),
    );
    const who = userEvent.setup();
    render(<App />);
    await who.click(screen.getByRole("button", { name: "Atajos de teclado" }));
    expect(await screen.findByText(/Otro programa ya usa esa combinación/)).toBeDefined();
    expect(screen.queryByText(/Estas están libres/)).toBeNull();
  });
});
