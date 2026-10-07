import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import { adopt } from "../locales";

type Invoke = (what: string, args?: never) => Promise<unknown>;

async function offering(version: string | null) {
  const { invoke } = await import("@tauri-apps/api/core");
  const real = vi.mocked(invoke).getMockImplementation() as Invoke;
  vi.mocked(invoke).mockImplementation(((what: string, args?: never) =>
    what === "update_ready"
      ? Promise.resolve({
          route: "download",
          looked: true,
          ready: version ? { version, installs: true } : null,
        })
      : real(what, args)) as never);
  return () => vi.mocked(invoke).mockImplementation(real as never);
}

async function heardOn(name: string) {
  const { listen } = await import("@tauri-apps/api/event");
  const call = [...vi.mocked(listen).mock.calls].reverse().find(([what]) => what === name);
  return (call as unknown as [string, (event: { payload: unknown }) => void])[1];
}

describe("a newer version", () => {
  const undo: (() => void)[] = [];

  afterEach(() => {
    while (undo.length > 0) {
      undo.pop()?.();
    }
  });

  it("is told in the rail without opening About, and the pill leads there", async () => {
    adopt("es");
    undo.push(await offering("3.0.4"));
    const who = userEvent.setup();
    render(<App />);
    const pill = await screen.findByRole("button", {
      name: "Acerca de · hay una versión nueva",
    });
    expect(pill).toHaveTextContent("Ver la 3.0.4 disponible");
    await who.click(pill);
    expect(screen.getByRole("heading", { level: 1 })).not.toHaveTextContent("General");
    expect(screen.queryByRole("button", { name: /hay una versión nueva/ })).toBeNull();
  });

  it("says nothing when there is nothing newer", async () => {
    adopt("es");
    undo.push(await offering(null));
    render(<App />);
    await screen.findByLabelText("Idioma");
    await waitFor(() => {
      expect(screen.queryByRole("button", { name: /hay una versión nueva/ })).toBeNull();
    });
  });

  it("the panel can send the window straight to About", async () => {
    adopt("es");
    render(<App />);
    await screen.findByLabelText("Idioma");
    const rail = await heardOn("rail");
    rail({ payload: "about" });
    expect(await screen.findByText("Todo local")).toBeDefined();
  });
});
