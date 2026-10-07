import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { type Kept, nearLimit } from "../core";
import { adopt, sized } from "../locales";
import History from "../ui/History";
import { scene as asked } from "./setup";

const A_GIGABYTE = 1024 ** 3;

async function heardOn(name: string): Promise<(event: unknown) => void> {
  const { listen } = await import("@tauri-apps/api/event");
  const calls = () => vi.mocked(listen).mock.calls.filter(([what]) => what === name);
  await waitFor(() => expect(calls().length).toBeGreaterThan(0));
  return calls().at(-1)?.[1] as (event: unknown) => void;
}

const kept = (quotaMb: number): Kept => ({
  locale: "es",
  theme: "system",
  shortcut: "Ctrl+Alt+V",
  "hides-when-left": true,
  "keeps-days": 30,
  "images-quota-mb": quotaMb,
});

describe("when the space is near its limit", () => {
  it("is near from seventy percent of the limit on", () => {
    const limitBytes = 5120 * 1024 * 1024;
    expect(nearLimit(limitBytes * 0.7, 5120)).toBe(true);
    expect(nearLimit(limitBytes * 0.7 - 1, 5120)).toBe(false);
    expect(nearLimit(limitBytes, 5120)).toBe(true);
    expect(nearLimit(limitBytes * 2, 5120)).toBe(true);
    expect(nearLimit(0, 5120)).toBe(false);
  });

  it("is never near when there is no limit", () => {
    expect(nearLimit(500 * A_GIGABYTE, 0)).toBe(false);
  });
});

describe("how a size is said", () => {
  afterEach(() => adopt("es"));

  it("uses gigabytes from one on, with the decimal mark of the language", () => {
    expect(sized(1.4 * A_GIGABYTE)).toBe("1,4 GB");
    expect(sized(2 * A_GIGABYTE)).toBe("2 GB");
    adopt("en");
    expect(sized(1.4 * A_GIGABYTE)).toBe("1.4 GB");
  });

  it("uses megabytes below a gigabyte", () => {
    expect(sized(300 * 1024 * 1024)).toBe("300 MB");
    expect(sized(0)).toBe("0 MB");
  });
});

describe("the space of the history in Settings", () => {
  afterEach(() => adopt("es"));

  it("tells how much is used against the limit and warns at seventy percent", async () => {
    adopt("es");
    asked.usedBytes = 4 * A_GIGABYTE;
    render(<History kept={kept(5120)} change={vi.fn()} />);
    expect(await screen.findByText("El historial ocupa 4 GB de 5 GB")).toBeInTheDocument();
    const near = screen.getByText(/se quitan los elementos más antiguos sin anclar/);
    expect(near).toHaveClass("said-plain");
    expect(near).not.toHaveClass("said");
  });

  it("tells the usage without warning while it is below seventy percent", async () => {
    adopt("es");
    asked.usedBytes = 3.4 * A_GIGABYTE;
    render(<History kept={kept(5120)} change={vi.fn()} />);
    expect(await screen.findByText("El historial ocupa 3,4 GB de 5 GB")).toBeInTheDocument();
    expect(screen.queryByText(/sin anclar/)).toBeNull();
  });

  it("says nothing about usage when there is no limit", async () => {
    adopt("es");
    asked.usedBytes = 5 * A_GIGABYTE;
    render(<History kept={kept(0)} change={vi.fn()} />);
    expect(await screen.findByLabelText("Espacio del historial")).toBeInTheDocument();
    await screen.findByText(/C:Users/);
    expect(screen.queryByText(/El historial ocupa/)).toBeNull();
    expect(screen.queryByText(/sin anclar/)).toBeNull();
  });

  it("offers the five gigabyte limit and a larger one", async () => {
    render(<History kept={kept(5120)} change={vi.fn()} />);
    expect(await screen.findByRole("option", { name: "5 GB" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "10 GB" })).toBeInTheDocument();
  });

  it("speaks English too", async () => {
    adopt("en");
    asked.usedBytes = 4 * A_GIGABYTE;
    render(<History kept={kept(5120)} change={vi.fn()} />);
    expect(await screen.findByText("The history takes up 4 GB of 5 GB")).toBeInTheDocument();
    expect(screen.getByText(/oldest unpinned items are removed/)).toBeInTheDocument();
  });

  it("weighs the history again once it is emptied", async () => {
    adopt("es");
    asked.usedBytes = 4 * A_GIGABYTE;
    render(<History kept={kept(5120)} change={vi.fn()} />);
    expect(await screen.findByText("El historial ocupa 4 GB de 5 GB")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Vaciar" }));
    await userEvent.click(screen.getByRole("button", { name: "¿Seguro?" }));
    expect(await screen.findByText("Listo, el historial quedó vacío")).toBeInTheDocument();
    expect(screen.getByText("El historial ocupa 4 GB de 5 GB")).toBeInTheDocument();
    asked.usedBytes = 0;
    const emptied = await heardOn("emptied");
    act(() => emptied({ payload: null }));
    expect(await screen.findByText("El historial ocupa 0 MB de 5 GB")).toBeInTheDocument();
    expect(screen.queryByText(/sin anclar/)).toBeNull();
  });

  it("weighs the history again when another window keeps settings or 2.x crosses", async () => {
    adopt("es");
    asked.usedBytes = 4 * A_GIGABYTE;
    render(<History kept={kept(5120)} change={vi.fn()} />);
    expect(await screen.findByText("El historial ocupa 4 GB de 5 GB")).toBeInTheDocument();
    for (const [name, gb] of [
      ["kept", 1],
      ["crossed", 3.6],
    ] as const) {
      const heard = await heardOn(name);
      asked.usedBytes = gb * A_GIGABYTE;
      act(() => heard({ payload: null }));
      expect(
        await screen.findByText(`El historial ocupa ${String(gb).replace(".", ",")} GB de 5 GB`),
      ).toBeInTheDocument();
    }
  });
});
