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

describe("the window", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("lands on the shortcuts when the panel asks for settings", async () => {
    render(<App />);
    await screen.findByLabelText("Idioma");

    const rail = await heardOn("rail");
    rail({ payload: "backup" });
    expect(await screen.findByText("Exportar")).toBeDefined();

    rail({ payload: "keys" });
    expect(await screen.findByText("Atajo del panel")).toBeDefined();
  });

  it("does not move when asked for a section that does not exist", async () => {
    render(<App />);
    await screen.findByLabelText("Idioma");

    const rail = await heardOn("rail");
    rail({ payload: "inventada" });
    await waitFor(() => {
      expect(screen.getByLabelText("Idioma")).toBeDefined();
    });
  });
});
