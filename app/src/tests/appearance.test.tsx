import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import { type Kept, reachable } from "../core";
import { adopt } from "../locales";
import { accentSaid, previewStyle } from "../ui/Appearance";

async function opened() {
  const who = userEvent.setup();
  render(<App />);
  await who.click(await screen.findByRole("button", { name: "Apariencia" }));
  await screen.findByRole("button", { name: /^Fuente:/ });
  return who;
}

async function saved() {
  const { invoke } = await import("@tauri-apps/api/core");
  return vi.mocked(invoke);
}

const KEPT: Kept = {
  locale: "es",
  theme: "system",
  shortcut: "Ctrl+Alt+V",
  "hides-when-left": true,
  "keeps-days": 30,
  "images-quota-mb": 0,
};

const CHOICES = {
  text: { id: "", label: "SF Pro", css: "-apple-system" },
  texts: [{ id: "avenir-next", label: "Avenir Next", css: '"Avenir Next"' }],
  code: { id: "", label: "Menlo", css: "Menlo" },
  codes: [{ id: "sf-mono", label: "SF Mono", css: "ui-monospace" }],
  plainSystem: true,
  accents: [
    {
      id: "indigo",
      light: "#4F46E5",
      dark: "#A5B4FC",
      lightSelected: "#E9E9FB",
      darkSelected: "#282C46",
    },
    {
      id: "teal",
      light: "#0F766E",
      dark: "#5EEAD4",
      lightSelected: "#DFF3F0",
      darkSelected: "#1E3639",
    },
  ],
};

describe("the appearance", () => {
  beforeEach(() => {
    adopt("es");
    vi.clearAllMocks();
  });

  it("starts on what the panel always looked like, each one marked as the default", async () => {
    await opened();
    const size = screen.getByRole("radiogroup", { name: "Tamaño del texto" });
    expect(within(size).getByRole("radio", { name: "Normal" })).toBeChecked();
    expect(screen.getByRole("radio", { name: "Índigo · predeterminado" })).toBeChecked();
    expect(screen.getByRole("button", { name: /^Fuente:/ })).toHaveTextContent(
      "Sistema (SF Pro) · predeterminado",
    );
    expect(screen.getByRole("button", { name: /^Fuente del código:/ })).toHaveTextContent(
      "Menlo · predeterminado",
    );
    const height = screen.getByRole("radiogroup", { name: "Densidad" });
    expect(within(height).getByRole("radio", { name: "Normal" })).toBeChecked();
  });

  it("each choice is sent to be saved as the panel reads it", async () => {
    const who = await opened();
    const invoke = await saved();
    await who.click(screen.getByRole("radio", { name: "Más grande" }));
    expect(invoke).toHaveBeenCalledWith("keep", {
      config: expect.objectContaining({ "text-size": "larger" }),
    });
    await who.click(screen.getByRole("radio", { name: "Amplia" }));
    expect(invoke).toHaveBeenCalledWith("keep", {
      config: expect.objectContaining({ density: "comfortable" }),
    });
    await who.click(screen.getByRole("radio", { name: "Verde azulado" }));
    expect(invoke).toHaveBeenCalledWith("keep", {
      config: expect.objectContaining({ accent: "teal" }),
    });
    await who.click(screen.getByRole("button", { name: /^Fuente:/ }));
    await who.click(screen.getByRole("option", { name: "Avenir Next" }));
    expect(invoke).toHaveBeenCalledWith("keep", {
      config: expect.objectContaining({ font: "avenir-next" }),
    });
    await who.click(screen.getByRole("button", { name: /^Fuente del código:/ }));
    await who.click(screen.getByRole("option", { name: "SF Mono" }));
    expect(invoke).toHaveBeenCalledWith("keep", {
      config: expect.objectContaining({ "code-font": "sf-mono" }),
    });
    await who.click(screen.getByRole("button", { name: /^Fuente:/ }));
    await who.click(screen.getByRole("option", { name: "Sistema (SF Pro) · predeterminado" }));
    expect(invoke).toHaveBeenCalledWith("keep", {
      config: expect.objectContaining({ font: null }),
    });
  });

  it("resetting brings back size, fonts, height and colour but leaves the theme alone", async () => {
    const who = await opened();
    const invoke = await saved();
    await who.selectOptions(screen.getByLabelText("Tema"), "dark");
    await who.click(
      within(screen.getByRole("radiogroup", { name: "Tamaño del texto" })).getByRole("radio", {
        name: "Grande",
      }),
    );
    await who.click(screen.getByRole("button", { name: "Restablecer la apariencia" }));
    expect(invoke).toHaveBeenCalledWith("keep", {
      config: expect.objectContaining({
        theme: "dark",
        "text-size": "normal",
        density: "normal",
        font: null,
        "code-font": null,
        accent: "indigo",
        "panel-size": "normal",
        "high-contrast": false,
      }),
    });
  });

  it("the density offers the list and picking it is saved as such", async () => {
    const who = await opened();
    const invoke = await saved();
    const density = screen.getByRole("radiogroup", { name: "Densidad" });
    await who.click(within(density).getByRole("radio", { name: "Lista" }));
    expect(invoke).toHaveBeenCalledWith("keep", {
      config: expect.objectContaining({ density: "list" }),
    });
  });

  it("the panel size is offered as normal, large and tall and is saved", async () => {
    const who = await opened();
    const invoke = await saved();
    const panel = screen.getByRole("radiogroup", { name: "Tamaño del panel" });
    expect(within(panel).getAllByRole("radio")).toHaveLength(3);
    expect(within(panel).getByRole("radio", { name: "Normal" })).toBeChecked();
    await who.click(within(panel).getByRole("radio", { name: "Grande" }));
    expect(invoke).toHaveBeenCalledWith("keep", {
      config: expect.objectContaining({ "panel-size": "large" }),
    });
    await who.click(within(panel).getByRole("radio", { name: "Alto" }));
    expect(invoke).toHaveBeenCalledWith("keep", {
      config: expect.objectContaining({ "panel-size": "tall" }),
    });
  });

  it("the high contrast switch is saved on and off", async () => {
    const who = await opened();
    const invoke = await saved();
    const contrast = screen.getByRole("switch", { name: "Contraste alto" });
    expect(contrast).not.toBeChecked();
    await who.click(contrast);
    expect(invoke).toHaveBeenCalledWith("keep", {
      config: expect.objectContaining({ "high-contrast": true }),
    });
  });

  it("the preview shows rows for the list and marks the high contrast", async () => {
    const who = await opened();
    const preview = screen.getByRole("figure");
    expect(preview.querySelectorAll(".mock-row")).toHaveLength(0);
    expect(preview.querySelector(".mock")).not.toHaveClass("strong");
    const density = screen.getByRole("radiogroup", { name: "Densidad" });
    await who.click(within(density).getByRole("radio", { name: "Lista" }));
    await who.click(screen.getByRole("switch", { name: "Contraste alto" }));
    expect(preview.querySelectorAll(".mock-row")).toHaveLength(4);
    expect(preview.querySelector(".mock")).toHaveClass("strong");
  });

  it("the size and the height move with the arrow keys, as one stop each", async () => {
    const who = await opened();
    const size = screen.getByRole("radiogroup", { name: "Tamaño del texto" });
    within(size).getByRole("radio", { name: "Normal" }).focus();
    await who.keyboard("{ArrowRight}");
    expect(within(size).getByRole("radio", { name: "Grande" })).toBeChecked();
  });

  it("the preview wears the chosen size, lines, colour and fonts", () => {
    const style = previewStyle(
      { ...KEPT, "text-size": "large", density: "compact", accent: "teal", font: "avenir-next" },
      CHOICES,
    ) as Record<string, unknown>;
    expect(style["--zoom"]).toBe(1.08);
    expect(style["--lines"]).toBe(1);
    expect(style["--light"]).toBe("#0F766E");
    expect(style["--dark"]).toBe("#5EEAD4");
    expect(style["--light-selected"]).toBe("#DFF3F0");
    expect(style["--dark-selected"]).toBe("#1E3639");
    expect(style["--look-font"]).toBe('"Avenir Next"');
    expect(style["--look-mono"]).toBe("Menlo");
  });

  it("an old file or a font no longer installed previews as the defaults", () => {
    const style = previewStyle({ ...KEPT, font: "gone" }, CHOICES) as Record<string, unknown>;
    expect(style["--zoom"]).toBe(1);
    expect(style["--lines"]).toBe(2);
    expect(style["--light"]).toBe("#4F46E5");
    expect(style["--look-font"]).toBe("-apple-system");
  });

  it("names a colour it does not know by its id", () => {
    expect(accentSaid("teal", false)).toBe("Verde azulado");
    expect(accentSaid("violeta", false)).toBe("violeta");
  });
});

describe("reaching every control with Tab", () => {
  it("asks for a tab stop on every control that does not say otherwise", () => {
    const root = document.createElement("div");
    root.innerHTML =
      '<button>a</button><select></select><input/><a href="#">b</a><div role="switch"></div><button tabindex="-1">c</button><span>d</span>';
    reachable(root);
    const stops = [...root.children].map((one) => one.getAttribute("tabindex"));
    expect(stops).toEqual(["0", "0", "0", "0", "0", "-1", null]);
  });
});

describe("supporting it", () => {
  it("the star takes the whole row and sponsoring and the coffee share the next one", async () => {
    adopt("es");
    const who = userEvent.setup();
    render(<App />);
    await who.click(await screen.findByRole("button", { name: "Acerca de" }));
    const star = await screen.findByRole("button", { name: /estrella/i });
    const sponsor = screen.getByRole("button", { name: /Patrocina/i });
    const coffee = screen.getByRole("button", { name: /café/i });
    expect(star).toHaveClass("wide");
    expect(sponsor).not.toHaveClass("wide");
    expect(coffee).not.toHaveClass("wide");
  });
});

describe("each choice group", () => {
  it("is a single Tab stop on the chosen option", async () => {
    adopt("es");
    const who = userEvent.setup();
    render(<App />);
    await who.click(await screen.findByRole("button", { name: "Apariencia" }));
    await screen.findByRole("button", { name: /^Fuente:/ });
    const size = screen.getByRole("radiogroup", { name: "Tamaño del texto" });
    const stops = within(size)
      .getAllByRole("radio")
      .map((one) => one.getAttribute("tabindex"));
    expect(stops).toEqual(["-1", "0", "-1", "-1"]);
    const colours = screen.getByRole("radiogroup", { name: "Color de acento" });
    expect(
      within(colours)
        .getAllByRole("radio")
        .filter((one) => one.getAttribute("tabindex") === "0"),
    ).toHaveLength(1);
  });
});
