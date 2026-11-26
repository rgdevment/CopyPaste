import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Kept } from "../core";
import { adopt, panelKeys } from "../locales";
import General from "../ui/General";

const kept: Kept = {
  locale: "en",
  theme: "system",
  shortcut: "Ctrl+Alt+V",
  "hides-when-left": true,
  "keeps-days": 30,
  "images-quota-mb": 0,
};

afterEach(() => adopt("es"));

describe("pasting with a click", () => {
  it("is a double click while the file says nothing about it", async () => {
    adopt("en");
    render(<General kept={kept} change={vi.fn()} />);
    const select = await screen.findByLabelText("Paste with");
    expect(select).toHaveValue("double-click");
  });

  it("shows what the file chose", async () => {
    adopt("en");
    render(<General kept={{ ...kept, "paste-with": "single-click" }} change={vi.fn()} />);
    expect(await screen.findByLabelText("Paste with")).toHaveValue("single-click");
  });

  it("is saved as single-click when it is chosen", async () => {
    adopt("en");
    const change = vi.fn(() => Promise.resolve());
    render(<General kept={kept} change={change} />);
    await userEvent.selectOptions(await screen.findByLabelText("Paste with"), "single-click");
    await waitFor(() => expect(change).toHaveBeenCalledWith({ "paste-with": "single-click" }));
  });
});

describe("the keys the panel lists", () => {
  it("swap opening and pasting for click and Shift + click with one click", () => {
    adopt("en");
    const rows = panelKeys(false, true);
    const open = rows.find((row) => row.id === "open-card");
    const plain = rows.find((row) => row.id === "paste-card");
    expect(open?.keys).toBe("Click");
    expect(open?.does).toBe("paste that card");
    expect(plain?.keys).toBe("Shift + click");
    expect(plain?.does).toBe("paste it as plain text");
  });

  it("keep the double click rows otherwise", () => {
    adopt("en");
    const rows = panelKeys(false);
    const swapped = panelKeys(false, true);
    expect(rows).toHaveLength(swapped.length);
    const open = rows.find((row) => row.id === "open-card");
    const paste = rows.find((row) => row.id === "paste-card");
    expect(open?.does).toBe("open the card; another click closes it");
    expect(paste?.keys).toBe("Double click");
    expect(paste?.does).toBe("paste that card");
    expect(panelKeys(false, false)).toEqual(rows);
    const others = (list: typeof rows) =>
      list.filter((row) => row.id !== "open-card" && row.id !== "paste-card");
    expect(others(swapped)).toEqual(others(rows));
  });
});
