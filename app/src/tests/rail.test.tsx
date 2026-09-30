import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";

type Heard = (event: { payload: unknown }) => void;

async function heardOn(name: string): Promise<Heard> {
  const { listen } = await import("@tauri-apps/api/event");
  const call = vi.mocked(listen).mock.calls.find(([what]) => what === name) as
    | [string, Heard]
    | undefined;
  expect(call, `nobody is listening for «${name}»`).toBeDefined();
  return (call as [string, Heard])[1];
}

describe("la ventana", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("aterriza en los atajos cuando el panel pide los ajustes", async () => {
    render(<App />);
    await screen.findByLabelText("Idioma");

    const rail = await heardOn("rail");
    rail({ payload: "backup" });
    expect(await screen.findByText("Exportar")).toBeDefined();

    rail({ payload: "keys" });
    expect(await screen.findByText("Atajo del panel")).toBeDefined();
  });

  it("no se mueve si le piden una sección que no existe", async () => {
    render(<App />);
    await screen.findByLabelText("Idioma");

    const rail = await heardOn("rail");
    rail({ payload: "inventada" });
    await waitFor(() => {
      expect(screen.getByLabelText("Idioma")).toBeDefined();
    });
  });
});
