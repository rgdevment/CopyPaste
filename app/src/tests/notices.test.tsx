import { readFileSync } from "node:fs";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import { composed } from "../markdown";
import About from "../ui/About";

type Invoke = (what: string, args?: never) => Promise<unknown>;

async function answering(notices: () => Promise<string>) {
  const { invoke } = await import("@tauri-apps/api/core");
  const real = vi.mocked(invoke).getMockImplementation() as Invoke;
  let asked = 0;
  vi.mocked(invoke).mockImplementation(((what: string, args?: never) => {
    if (what !== "notices") return real(what, args);
    asked += 1;
    return notices();
  }) as never);
  return {
    asked: () => asked,
    undo: () => vi.mocked(invoke).mockImplementation(real as never),
  };
}

describe("the notices each bundled licence asks for", () => {
  const undo: (() => void)[] = [];

  afterEach(() => {
    while (undo.length > 0) {
      undo.pop()?.();
    }
  });

  it("are shown from the window, and only fetched when opened", async () => {
    const who = userEvent.setup();
    const said = await answering(() => Promise.resolve("MIT License\n\nCopyright (c) alguien"));
    undo.push(said.undo);
    render(<About />);

    const button = await screen.findByRole("button", { name: "Avisos de terceros" });
    expect(said.asked()).toBe(0);

    await who.click(button);
    expect(await screen.findByText(/Copyright \(c\) alguien/)).toBeDefined();
    expect(said.asked()).toBe(1);
    expect(button.getAttribute("aria-expanded")).toBe("true");

    await who.click(button);
    await waitFor(() => {
      expect(screen.queryByText(/Copyright \(c\) alguien/)).toBeNull();
    });
    expect(button.getAttribute("aria-expanded")).toBe("false");
  });

  it("are drawn as text, not as the markdown they are written in", async () => {
    const who = userEvent.setup();
    const said = await answering(() =>
      Promise.resolve(
        "# Avisos\n\n<!-- Written by `npm run notices`. Do not edit by hand. -->\n\n## In the window\n\n| Package | Licence |\n| --- | --- |\n| react | MIT |",
      ),
    );
    undo.push(said.undo);
    render(<About />);

    await who.click(await screen.findByRole("button", { name: "Avisos de terceros" }));

    const cell = await screen.findByRole("cell", { name: "react" });
    expect(cell.closest("table")).toBeTruthy();
    expect(screen.queryByText(/\| --- \|/)).toBeNull();
    expect(screen.queryByText(/^## /)).toBeNull();
    expect(screen.queryByText(/Do not edit by hand/)).toBeNull();
  });

  it("do not run whatever html they carry", async () => {
    const who = userEvent.setup();
    const said = await answering(() => Promise.resolve("<img src=x onerror=alert(1)> y nada más"));
    undo.push(said.undo);
    const { container } = render(<About />);

    await who.click(await screen.findByRole("button", { name: "Avisos de terceros" }));

    expect(await screen.findByText(/y nada más/)).toBeDefined();
    expect(container.querySelector(".notices img")).toBeNull();
  });

  it("a link inside opens outside, without taking the window with it", async () => {
    const who = userEvent.setup();
    const { invoke } = await import("@tauri-apps/api/core");
    const said = await answering(() => Promise.resolve("Ver https://crates.io/crates/slint"));
    undo.push(said.undo);
    render(<About />);

    await who.click(await screen.findByRole("button", { name: "Avisos de terceros" }));
    await who.click(await screen.findByRole("link", { name: "https://crates.io/crates/slint" }));

    expect(invoke).toHaveBeenCalledWith("open_web", { url: "https://crates.io/crates/slint" });
  });

  it("every link in the real file names its site, because a relative one points at the window", () => {
    const holder = document.createElement("div");
    holder.innerHTML = composed(readFileSync("../THIRD-PARTY-BUNDLED.md", "utf8"));
    const hrefs = Array.from(holder.querySelectorAll("a")).map((one) => one.getAttribute("href"));
    expect(hrefs.length).toBeGreaterThan(0);
    for (const href of hrefs) {
      expect(href, `${href} no sale de la ventana`).toMatch(/^https?:\/\//);
    }
  });

  it("a retry that works clears the notice of the earlier failure", async () => {
    const who = userEvent.setup();
    let tries = 0;
    const said = await answering(() => {
      tries += 1;
      return tries === 1 ? Promise.reject(new Error("no")) : Promise.resolve("MIT License");
    });
    undo.push(said.undo);
    render(<About />);

    const button = await screen.findByRole("button", { name: "Avisos de terceros" });
    await who.click(button);
    expect(await screen.findByText("No se pudieron leer los avisos de terceros")).toBeDefined();

    await who.click(button);
    expect(await screen.findByText("MIT License")).toBeDefined();
    expect(screen.queryByText("No se pudieron leer los avisos de terceros")).toBeNull();
  });

  it("the licence texts have their own button, and are only fetched when it opens", async () => {
    const who = userEvent.setup();
    const { invoke } = await import("@tauri-apps/api/core");
    const real = vi.mocked(invoke).getMockImplementation() as Invoke;
    let asked = 0;
    vi.mocked(invoke).mockImplementation(((what: string, args?: never) => {
      if (what === "notices") return Promise.resolve("Avisos de la ventana");
      if (what !== "licences") return real(what, args);
      asked += 1;
      return Promise.resolve(
        "## Text 1\n\nCarried by `serde` 1.0.\n\n```text\nApache License\n```",
      );
    }) as never);
    undo.push(() => vi.mocked(invoke).mockImplementation(real as never));
    render(<About />);

    const button = await screen.findByRole("button", { name: "Textos de las licencias" });
    expect(asked).toBe(0);

    await who.click(button);
    expect(await screen.findByRole("region", { name: "Textos de las licencias" })).toBeDefined();
    expect(screen.getByText("Apache License")).toBeDefined();
    expect(screen.queryByText("Avisos de la ventana")).toBeNull();
    expect(asked).toBe(1);

    await who.click(button);
    await waitFor(() => {
      expect(screen.queryByText("Apache License")).toBeNull();
    });
  });

  it("says so when the texts cannot be read", async () => {
    const who = userEvent.setup();
    const { invoke } = await import("@tauri-apps/api/core");
    const real = vi.mocked(invoke).getMockImplementation() as Invoke;
    vi.mocked(invoke).mockImplementation(((what: string, args?: never) =>
      what === "licences" ? Promise.reject(new Error("no")) : real(what, args)) as never);
    undo.push(() => vi.mocked(invoke).mockImplementation(real as never));
    render(<About />);

    await who.click(await screen.findByRole("button", { name: "Textos de las licencias" }));

    expect(
      await screen.findByText("No se pudieron leer los textos de las licencias"),
    ).toBeDefined();
  });

  it("every link in the real texts names its site", () => {
    const holder = document.createElement("div");
    holder.innerHTML = composed(readFileSync("../THIRD-PARTY-LICENSES.md", "utf8"));
    for (const href of Array.from(holder.querySelectorAll("a")).map((one) =>
      one.getAttribute("href"),
    )) {
      expect(href, `${href} no sale de la ventana`).toMatch(/^https?:\/\//);
    }
    expect(holder.querySelectorAll("pre").length).toBeGreaterThan(100);
  });

  it("says so when they cannot be read instead of staying silent", async () => {
    const who = userEvent.setup();
    const said = await answering(() => Promise.reject(new Error("no")));
    undo.push(said.undo);
    render(<About />);

    await who.click(await screen.findByRole("button", { name: "Avisos de terceros" }));

    expect(await screen.findByText("No se pudieron leer los avisos de terceros")).toBeDefined();
    expect(document.querySelector(".notices")).toBeNull();
  });
});
