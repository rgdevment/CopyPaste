import { fireEvent, render, renderHook, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { type Choices, type Kept, useChoices, wearFonts, wornFonts } from "../core";
import { adopt } from "../locales";
import { systemSaid } from "../ui/Appearance";
import FontPicker, { matching } from "../ui/FontPicker";

const KEPT: Kept = {
  locale: "es",
  theme: "system",
  shortcut: "Ctrl+Alt+V",
  "hides-when-left": true,
  "keeps-days": 30,
  "images-quota-mb": 0,
};

const CHOICES: Choices = {
  text: { id: "", label: "Segoe UI", css: '"Segoe UI"' },
  texts: [
    { id: "calibri", label: "Calibri", css: "Calibri" },
    { id: "Comic Sans MS", label: "Comic Sans MS", css: '"Comic Sans MS"' },
  ],
  code: { id: "", label: "Consolas", css: "Consolas" },
  codes: [{ id: "Fira Code", label: "Fira Code", css: '"Fira Code"' }],
  plainSystem: true,
  accents: [],
};

const SYSTEM_IS_COMIC: Choices = {
  ...CHOICES,
  text: { id: "", label: "Comic Sans MS", css: '"Comic Sans MS"' },
  plainSystem: false,
};

describe("the fonts the window wears", () => {
  it("keeps its own text stack while the system font is the plain platform one", () => {
    expect(wornFonts(KEPT, CHOICES)).toEqual({ text: null, code: "Consolas" });
  });

  it("wears the system font when the system names another one", () => {
    expect(wornFonts(KEPT, SYSTEM_IS_COMIC)).toEqual({ text: '"Comic Sans MS"', code: "Consolas" });
  });

  it("wears the fonts that were chosen", () => {
    const worn = wornFonts({ ...KEPT, font: "calibri", "code-font": "Fira Code" }, CHOICES);
    expect(worn).toEqual({ text: "Calibri", code: '"Fira Code"' });
  });

  it("falls back to the system font when the chosen one is gone", () => {
    const kept = { ...KEPT, font: "gone", "code-font": "gone" };
    expect(wornFonts(kept, CHOICES)).toEqual({ text: null, code: "Consolas" });
    expect(wornFonts(kept, SYSTEM_IS_COMIC)).toEqual({ text: '"Comic Sans MS"', code: "Consolas" });
  });

  it("sets the properties on the page and removes them again", () => {
    const style = document.documentElement.style;
    wearFonts({ text: "Calibri", code: '"Fira Code"' });
    expect(style.getPropertyValue("--worn-text")).toBe("Calibri");
    expect(style.getPropertyValue("--worn-code")).toBe('"Fira Code"');
    wearFonts({ text: null, code: '"Fira Code"' });
    expect(style.getPropertyValue("--worn-text")).toBe("");
    expect(style.getPropertyValue("--worn-code")).toBe('"Fira Code"');
    wearFonts({ text: null, code: null });
    expect(style.getPropertyValue("--worn-code")).toBe("");
  });

  it("asks for the fonts again when the window becomes visible", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockClear();
    const { result } = renderHook(() => useChoices());
    await waitFor(() => expect(result.current).not.toBeNull());
    const asked = () => vi.mocked(invoke).mock.calls.filter(([what]) => what === "looks").length;
    expect(asked()).toBe(1);
    document.dispatchEvent(new Event("visibilitychange"));
    await waitFor(() => expect(asked()).toBe(2));
  });
});

describe("looking a font up", () => {
  const all = [
    { id: "", label: "Sistema (Segoe UI) · predeterminado" },
    { id: "Yu Gothic", label: "Yu Gótico" },
    { id: "Arial", label: "Arial" },
  ];

  it("shows everything when nothing is asked", () => {
    expect(matching(all, "")).toBe(all);
    expect(matching(all, "   ")).toBe(all);
  });

  it("folds case and accents", () => {
    expect(matching(all, "GOTICO").map((one) => one.id)).toEqual(["Yu Gothic"]);
    expect(matching(all, "gótico").map((one) => one.id)).toEqual(["Yu Gothic"]);
  });

  it("also matches the id, which stays in English", () => {
    expect(matching(all, "gothic").map((one) => one.id)).toEqual(["Yu Gothic"]);
  });

  it("matches nothing for a name that does not exist", () => {
    expect(matching(all, "zzz")).toEqual([]);
  });
});

describe("saying what the system font is", () => {
  beforeEach(() => adopt("es"));

  it("names the font the system uses and that it is the default", () => {
    expect(systemSaid(SYSTEM_IS_COMIC)).toBe("Sistema (Comic Sans MS) · predeterminado");
  });
});

function Harness({ pick }: { pick: (id: string | null) => void }) {
  const [chosen, setChosen] = useState<string | null>(null);
  return (
    <FontPicker
      says="Fuente"
      first={systemSaid(CHOICES)}
      others={CHOICES.texts}
      chosen={chosen}
      pick={(id) => {
        setChosen(id);
        pick(id);
      }}
    />
  );
}

const opener = () => screen.getByRole("button", { name: /^Fuente:/ });

describe("the font picker", () => {
  beforeEach(() => adopt("es"));

  it("shows the chosen font on its button, the system one at first", () => {
    render(<Harness pick={vi.fn()} />);
    expect(opener()).toHaveTextContent("Sistema (Segoe UI) · predeterminado");
    expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
  });

  it("opens with the search box and every font listed, the chosen one marked", async () => {
    const who = userEvent.setup();
    render(<Harness pick={vi.fn()} />);
    await who.click(opener());
    expect(screen.getByRole("combobox", { name: "Buscar una fuente" })).toHaveFocus();
    const options = screen.getAllByRole("option");
    expect(options.map((one) => one.textContent)).toEqual([
      "Sistema (Segoe UI) · predeterminado",
      "Calibri",
      "Comic Sans MS",
    ]);
    expect(options[0]).toHaveAttribute("aria-selected", "true");
  });

  it("narrows the list as something is typed and says when nothing matches", async () => {
    const who = userEvent.setup();
    render(<Harness pick={vi.fn()} />);
    await who.click(opener());
    await who.type(screen.getByRole("combobox"), "COMIC");
    expect(screen.getAllByRole("option").map((one) => one.textContent)).toEqual(["Comic Sans MS"]);
    await who.clear(screen.getByRole("combobox"));
    await who.type(screen.getByRole("combobox"), "zzz");
    expect(screen.queryByRole("option")).not.toBeInTheDocument();
    expect(screen.getByText("Ninguna fuente se llama así")).toBeInTheDocument();
  });

  it("picks with a click and keeps the family id", async () => {
    const who = userEvent.setup();
    const pick = vi.fn();
    render(<Harness pick={pick} />);
    await who.click(opener());
    await who.click(screen.getByRole("option", { name: "Comic Sans MS" }));
    expect(pick).toHaveBeenCalledWith("Comic Sans MS");
    expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
    expect(opener()).toHaveTextContent("Comic Sans MS");
  });

  it("picks with the keyboard: arrows move, Enter chooses", async () => {
    const who = userEvent.setup();
    const pick = vi.fn();
    render(<Harness pick={pick} />);
    await who.click(opener());
    await who.keyboard("{ArrowDown}{ArrowDown}{ArrowUp}{Enter}");
    expect(pick).toHaveBeenCalledWith("calibri");
    await who.click(opener());
    await who.keyboard("{End}{Enter}");
    expect(pick).toHaveBeenLastCalledWith("Comic Sans MS");
    await who.click(opener());
    await who.keyboard("{Home}{Enter}");
    expect(pick).toHaveBeenLastCalledWith(null);
  });

  it("choosing the system font sends null", async () => {
    const who = userEvent.setup();
    const pick = vi.fn();
    render(<Harness pick={pick} />);
    await who.click(opener());
    await who.click(screen.getByRole("option", { name: "Calibri" }));
    await who.click(opener());
    await who.click(screen.getByRole("option", { name: "Sistema (Segoe UI) · predeterminado" }));
    expect(pick).toHaveBeenLastCalledWith(null);
  });

  it("choosing what is already chosen sends nothing", async () => {
    const who = userEvent.setup();
    const pick = vi.fn();
    render(<Harness pick={pick} />);
    await who.click(opener());
    await who.keyboard("{Enter}");
    expect(pick).not.toHaveBeenCalled();
  });

  it("Escape closes the list without choosing and gives the focus back", async () => {
    const who = userEvent.setup();
    const pick = vi.fn();
    render(<Harness pick={pick} />);
    await who.click(opener());
    await who.keyboard("{Escape}");
    expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
    expect(opener()).toHaveFocus();
    expect(pick).not.toHaveBeenCalled();
  });

  it("pressing on the list, its scrollbar too, keeps it open and the search focused", async () => {
    const who = userEvent.setup();
    render(<Harness pick={vi.fn()} />);
    await who.click(opener());
    expect(fireEvent.mouseDown(screen.getByRole("listbox"))).toBe(false);
    expect(screen.getByRole("combobox")).toHaveFocus();
  });

  it("the button closes the list it opened and keeps the focus", async () => {
    const who = userEvent.setup();
    render(<Harness pick={vi.fn()} />);
    await who.click(opener());
    await who.click(opener());
    expect(screen.queryByRole("listbox")).not.toBeInTheDocument();
    expect(opener()).toHaveFocus();
  });
});
