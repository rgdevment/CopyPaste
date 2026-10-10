import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import About from "../ui/About";

type Invoke = (what: string, args?: never) => Promise<unknown>;

async function answering(answers: Record<string, () => Promise<unknown>>) {
  const { invoke } = await import("@tauri-apps/api/core");
  const real = vi.mocked(invoke).getMockImplementation() as Invoke;
  const asked: string[] = [];
  vi.mocked(invoke).mockImplementation(((what: string, args?: never) => {
    const answer = answers[what];
    if (!answer) return real(what, args);
    asked.push(what);
    return answer();
  }) as never);
  return {
    asked,
    undo: () => vi.mocked(invoke).mockImplementation(real as never),
  };
}

describe("the buttons for when something goes wrong", () => {
  const undo: (() => void)[] = [];

  afterEach(() => {
    while (undo.length > 0) {
      undo.pop()?.();
    }
  });

  it("save a report and offer to show it where it was saved", async () => {
    const who = userEvent.setup();
    const said = await answering({ save_report: () => Promise.resolve("/donde/informe.txt") });
    undo.push(said.undo);
    const { revealItemInDir } = await import("@tauri-apps/plugin-opener");
    render(<About />);

    await who.click(await screen.findByRole("button", { name: "Guardar informe…" }));
    expect(await screen.findByText("Informe guardado.")).toBeDefined();
    expect(said.asked).toEqual(["save_report"]);

    await who.click(screen.getByRole("button", { name: "Mostrar en la carpeta" }));
    expect(vi.mocked(revealItemInDir)).toHaveBeenCalledWith("/donde/informe.txt");
  });

  it("say nothing when the save dialog is closed without choosing", async () => {
    const who = userEvent.setup();
    const said = await answering({ save_report: () => Promise.resolve(null) });
    undo.push(said.undo);
    render(<About />);

    const button = await screen.findByRole("button", { name: "Guardar informe…" });
    await who.click(button);
    await waitFor(() => expect((button as HTMLButtonElement).disabled).toBe(false));
    expect(screen.queryByText("Informe guardado.")).toBeNull();
    expect(screen.queryByRole("button", { name: "Mostrar en la carpeta" })).toBeNull();
  });

  it("tell when the report could not be saved", async () => {
    const who = userEvent.setup();
    const said = await answering({ save_report: () => Promise.reject("denied") });
    undo.push(said.undo);
    render(<About />);

    await who.click(await screen.findByRole("button", { name: "Guardar informe…" }));
    expect(await screen.findByText("No se pudo guardar el informe")).toBeDefined();
  });

  it("tell when the saved report is no longer there to be shown", async () => {
    const who = userEvent.setup();
    const said = await answering({ save_report: () => Promise.resolve("/donde/informe.txt") });
    undo.push(said.undo);
    const { revealItemInDir } = await import("@tauri-apps/plugin-opener");
    vi.mocked(revealItemInDir).mockRejectedValueOnce("gone");
    render(<About />);

    await who.click(await screen.findByRole("button", { name: "Guardar informe…" }));
    await who.click(await screen.findByRole("button", { name: "Mostrar en la carpeta" }));
    expect(await screen.findByText("No se encontró el informe donde se guardó")).toBeDefined();
  });

  it("open the log folder, and tell when it would not open", async () => {
    const who = userEvent.setup();
    const said = await answering({ open_log: () => Promise.reject("denied") });
    undo.push(said.undo);
    render(<About />);

    await who.click(await screen.findByRole("button", { name: "Abrir el registro" }));
    expect(said.asked).toEqual(["open_log"]);
    expect(await screen.findByText("No se pudo abrir la carpeta del registro")).toBeDefined();
  });
});
