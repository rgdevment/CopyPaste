import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { type Kept, useKept } from "../core";

type Heard = (event: { payload: unknown }) => void;

const saved: Kept = {
  locale: "es",
  theme: "system",
  shortcut: "Ctrl+Alt+V",
  "hides-when-left": true,
  "keeps-days": 30,
  "images-quota-mb": 0,
};

async function heardOn(name: string): Promise<Heard> {
  const { listen } = await import("@tauri-apps/api/event");
  await waitFor(() => {
    expect(vi.mocked(listen).mock.calls.some(([what]) => what === name)).toBe(true);
  });
  const call = vi.mocked(listen).mock.calls.find(([what]) => what === name) as [string, Heard];
  return call[1];
}

describe("los ajustes guardados desde otra ventana", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("se adoptan cuando esta ventana no tiene nada por guardar", async () => {
    const { result } = renderHook(() => useKept());
    await waitFor(() => expect(result.current.kept).not.toBeNull());

    const kept = await heardOn("kept");
    act(() => kept({ payload: { ...saved, "keeps-days": 7 } }));
    expect(result.current.kept?.["keeps-days"]).toBe(7);
  });

  it("no pisan un cambio de esta ventana que todavía se está guardando", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const real = vi.mocked(invoke).getMockImplementation();
    let release: () => void = () => {};
    vi.mocked(invoke).mockImplementation((what, args) => {
      if (what === "keep") {
        return new Promise((done) => {
          release = () => done((args as { config: Kept }).config);
        });
      }
      return real ? real(what, args) : Promise.reject(new Error(what));
    });

    const { result } = renderHook(() => useKept());
    await waitFor(() => expect(result.current.kept).not.toBeNull());
    const kept = await heardOn("kept");

    let saving: Promise<void> = Promise.resolve();
    act(() => {
      saving = result.current.change({ "keeps-days": 90 });
    });
    await waitFor(() => expect(vi.mocked(invoke)).toHaveBeenCalledWith("keep", expect.anything()));
    act(() => kept({ payload: { ...saved, "keeps-days": 30 } }));
    expect(result.current.kept?.["keeps-days"]).toBe(90);

    await act(async () => {
      release();
      await saving;
    });
    expect(result.current.kept?.["keeps-days"]).toBe(90);
    vi.mocked(invoke).mockImplementation(real ?? (() => Promise.resolve()));
  });
});
