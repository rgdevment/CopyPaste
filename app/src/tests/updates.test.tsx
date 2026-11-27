import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "../App";
import { asksToUpdateItself, featureLine, type Kept, type Looked } from "../core";
import { adopt } from "../locales";
import About from "../ui/About";

type Invoke = (what: string, args?: never) => Promise<unknown>;
type Heard = (event: { payload: unknown }) => void;

const nothing = () => Promise.resolve();

const take = { version: "3.6.0", installs: true };

function looked(over: Partial<Looked>): Looked {
  return { route: "download", looked: true, ready: null, ...over };
}

async function scenario(seen: Looked, install: () => Promise<unknown> = () => Promise.resolve()) {
  const { invoke } = await import("@tauri-apps/api/core");
  const { listen } = await import("@tauri-apps/api/event");
  const realInvoke = vi.mocked(invoke).getMockImplementation() as Invoke;
  const realListen = vi.mocked(listen).getMockImplementation();
  const kept: Record<string, unknown>[] = [];
  const asked: string[] = [];
  const heard: Record<string, Heard> = {};
  vi.mocked(invoke).mockImplementation(((what: string, args?: never) => {
    if (what === "update_ready") {
      asked.push(what);
      return Promise.resolve(seen);
    }
    if (what === "update_install") {
      asked.push(what);
      return install();
    }
    if (what === "install_route") return Promise.resolve(seen.route);
    if (what === "keep") {
      kept.push((args as unknown as { config: Record<string, unknown> }).config);
    }
    return realInvoke(what, args);
  }) as never);
  vi.mocked(listen).mockImplementation(((name: string, handler: Heard) => {
    heard[name] = handler;
    return Promise.resolve(() => {});
  }) as never);
  return {
    kept,
    asked,
    say: (name: string, payload: unknown) => act(() => heard[name]({ payload })),
    undo: () => {
      vi.mocked(invoke).mockImplementation(realInvoke as never);
      vi.mocked(listen).mockImplementation(realListen as never);
    },
  };
}

const sent: Partial<Kept>[] = [];

function Holder({ start }: { start: Kept }) {
  const [kept, setKept] = useState<Kept>(start);
  return (
    <About
      kept={kept}
      change={(what) => {
        sent.push(what);
        setKept((was) => ({ ...was, ...what }));
        return Promise.resolve();
      }}
    />
  );
}

describe("updates", () => {
  const undo: (() => void)[] = [];

  beforeEach(() => adopt("es"));

  afterEach(() => {
    while (undo.length > 0) {
      undo.pop()?.();
    }
  });

  async function given(seen: Looked, install?: () => Promise<unknown>) {
    const one = await scenario(seen, install);
    undo.push(one.undo);
    return one;
  }

  describe("the question about updating itself", () => {
    const question = "¿Quieres que CoPanda se actualice sola de ahora en adelante?";

    async function asked(here: string, offered: string, start: Partial<Kept> = {}, over = {}) {
      const { getVersion } = await import("@tauri-apps/api/app");
      vi.mocked(getVersion).mockResolvedValue(here);
      sent.length = 0;
      await given(looked({ ready: { version: offered, installs: true }, ...over }));
      render(<Holder start={start as Kept} />);
      await screen.findByText(here);
      await screen.findByText(`La versión ${offered} ya está disponible`);
    }

    afterEach(async () => {
      const { getVersion } = await import("@tauri-apps/api/app");
      vi.mocked(getVersion).mockResolvedValue("3.0.0");
    });

    it("is asked inside the notice when the first or second number changes", async () => {
      await asked("3.5.1", "3.6.0");
      expect(screen.getByText(question, { exact: false })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Sí" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Ahora no" })).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "No volver a preguntar" })).toBeInTheDocument();
    });

    it("is asked when the first number changes", async () => {
      await asked("3.6.0", "4.0.0");
      expect(screen.getByText(question, { exact: false })).toBeInTheDocument();
    });

    it("lays the three answers on a line of their own under the question", async () => {
      await asked("3.5.1", "3.6.0");
      const answers = document.body.querySelector(".answers") as HTMLElement;
      expect(answers).not.toBeNull();
      for (const name of ["Sí", "Ahora no", "No volver a preguntar"]) {
        const button = screen.getByRole("button", { name });
        expect(answers.contains(button)).toBe(true);
      }
    });

    it("is not asked for a fix", async () => {
      await asked("3.5.1", "3.5.2");
      expect(screen.queryByText(question, { exact: false })).toBeNull();
    });

    it("keeps yes as updates-itself true and the line goes", async () => {
      const who = userEvent.setup();
      await asked("3.5.1", "3.6.0");
      await who.click(screen.getByRole("button", { name: "Sí" }));
      expect(sent).toEqual([{ "updates-itself": true }]);
      await waitFor(() => expect(screen.queryByText(question, { exact: false })).toBeNull());
    });

    it("keeps never ask again as updates-itself false and the line goes", async () => {
      const who = userEvent.setup();
      await asked("3.5.1", "3.6.0");
      await who.click(screen.getByRole("button", { name: "No volver a preguntar" }));
      expect(sent).toEqual([{ "updates-itself": false }]);
      await waitFor(() => expect(screen.queryByText(question, { exact: false })).toBeNull());
    });

    it("keeps not now as the feature line offered and the line goes", async () => {
      const who = userEvent.setup();
      await asked("3.5.1", "3.6.0");
      await who.click(screen.getByRole("button", { name: "Ahora no" }));
      expect(sent).toEqual([{ "updates-itself-put-off": "3.6" }]);
      await waitFor(() => expect(screen.queryByText(question, { exact: false })).toBeNull());
    });

    it("does not come back for a later fix of what was put off", async () => {
      await asked("3.5.1", "3.6.1", { "updates-itself-put-off": "3.6" });
      expect(screen.queryByText(question, { exact: false })).toBeNull();
    });

    it("comes back for the next feature line after not now", async () => {
      await asked("3.5.1", "3.7.0", { "updates-itself-put-off": "3.6" });
      expect(screen.getByText(question, { exact: false })).toBeInTheDocument();
    });

    it("is not asked once answered either way", async () => {
      await asked("3.5.1", "3.6.0", { "updates-itself": true });
      expect(screen.queryByText(question, { exact: false })).toBeNull();
      cleanup();
      await asked("3.5.1", "3.6.0", { "updates-itself": false });
      expect(screen.queryByText(question, { exact: false })).toBeNull();
    });

    it("is not asked for a copy the Store updates", async () => {
      await asked("3.5.1", "3.6.0", {}, { route: "store" });
      expect(screen.queryByText(question, { exact: false })).toBeNull();
    });

    it("is not asked when what waits cannot be installed from here", async () => {
      const { getVersion } = await import("@tauri-apps/api/app");
      vi.mocked(getVersion).mockResolvedValue("3.5.1");
      await given(looked({ ready: { version: "3.6.0", installs: false } }));
      render(<About kept={{} as Kept} change={nothing} />);
      await screen.findByText("La versión 3.6.0 ya está disponible");
      expect(screen.queryByText(question, { exact: false })).toBeNull();
    });

    it("decides from the numbers alone", () => {
      const seen = looked({ ready: take });
      const none = {} as Kept;
      expect(asksToUpdateItself(seen, none, "3.5.1")).toBe(true);
      expect(asksToUpdateItself(seen, none, "3.6.0")).toBe(false);
      expect(asksToUpdateItself(seen, { "updates-itself": null } as Kept, "3.5.1")).toBe(true);
      expect(asksToUpdateItself(seen, { "updates-itself": true } as Kept, "3.5.1")).toBe(false);
      expect(asksToUpdateItself(seen, { "updates-itself": false } as Kept, "3.5.1")).toBe(false);
      expect(asksToUpdateItself(seen, { "updates-itself-put-off": "3.6" } as Kept, "3.5.1")).toBe(
        false,
      );
      expect(asksToUpdateItself(seen, null, "3.5.1")).toBe(false);
      expect(asksToUpdateItself(null, none, "3.5.1")).toBe(false);
      expect(asksToUpdateItself(looked({ route: "store", ready: take }), none, "3.5.1")).toBe(
        false,
      );
    });

    it("reads the feature line out of a version", () => {
      expect(featureLine("3.6.0")).toBe("3.6");
      expect(featureLine("3.6.0-rc1")).toBe("3.6");
      expect(featureLine("10.02.1")).toBe("10.2");
      expect(featureLine("nonsense")).toBeNull();
      expect(featureLine("3")).toBeNull();
    });
  });

  describe("the switch in About", () => {
    it("shows the state kept and flips it", async () => {
      const who = userEvent.setup();
      await given(looked({ replacesItself: true }));
      const change = vi.fn(nothing);
      render(<About kept={{ "updates-itself": false } as Kept} change={change} />);
      const knob = await screen.findByRole("switch", { name: "Actualizar automáticamente" });
      expect(knob).toHaveAttribute("aria-checked", "false");
      await who.click(knob);
      expect(change).toHaveBeenCalledWith({ "updates-itself": true });
    });

    it("turns it off when it was on", async () => {
      const who = userEvent.setup();
      await given(looked({ replacesItself: true }));
      const change = vi.fn(nothing);
      render(<About kept={{ "updates-itself": true } as Kept} change={change} />);
      const knob = await screen.findByRole("switch", { name: "Actualizar automáticamente" });
      expect(knob).toHaveAttribute("aria-checked", "true");
      await who.click(knob);
      expect(change).toHaveBeenCalledWith({ "updates-itself": false });
    });

    it("is not there when the copy cannot replace itself", async () => {
      const said = await given(looked({ replacesItself: false }));
      render(<About kept={{} as Kept} change={nothing} />);
      await waitFor(() => expect(said.asked).toContain("update_ready"));
      await screen.findByRole("button", { name: "Buscar ahora" });
      expect(screen.queryByText("Actualizar automáticamente")).toBeNull();
    });
  });

  describe("a copy the Store updates", () => {
    const store = (over: Partial<Looked>) => looked({ route: "store", ...over });

    it("is offered an update with its number", async () => {
      await given(store({ ready: take }));
      render(<About kept={null} change={nothing} />);
      expect(await screen.findByText("La versión 3.6.0 ya está disponible")).toBeInTheDocument();
      expect(
        screen.getByText("La instala la Microsoft Store, sin salir de aquí"),
      ).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Actualizar" })).toBeInTheDocument();
    });

    it("is offered an update without a number", async () => {
      await given(store({ ready: { version: "", installs: true } }));
      render(<About kept={null} change={nothing} />);
      expect(
        await screen.findByText("La Microsoft Store tiene una versión nueva"),
      ).toBeInTheDocument();
      expect(screen.getByRole("button", { name: "Actualizar" })).toBeInTheDocument();
    });

    it.each([
      ["queued", /la trae por su cuenta/],
      ["paused", /Ábrela para reanudarla/],
      ["notYet", /todavía no se la ofrece/],
    ] as const)("tells why a %s update waits and offers to open the Store", async (coming, why) => {
      await given(store({ ready: { version: "3.6.0", installs: false, coming } }));
      render(<About kept={null} change={nothing} />);
      expect(
        await screen.findByRole("button", { name: "Abrir Microsoft Store" }),
      ).toBeInTheDocument();
      expect(screen.getByText(why)).toBeInTheDocument();
      expect(screen.queryByRole("button", { name: "Actualizar" })).toBeNull();
    });

    it("opens the Store on its updates shelf", async () => {
      const who = userEvent.setup();
      await given(store({ ready: { version: "3.6.0", installs: false, coming: "paused" } }));
      const { openUrl } = await import("@tauri-apps/plugin-opener");
      render(<About kept={null} change={nothing} />);
      await who.click(await screen.findByRole("button", { name: "Abrir Microsoft Store" }));
      expect(vi.mocked(openUrl)).toHaveBeenCalledWith("ms-windows-store://downloadsandupdates");
    });

    it("what landed has no button to press", async () => {
      await given(store({ ready: { version: "3.6.0", installs: false } }));
      render(<About kept={null} change={nothing} />);
      expect(
        await screen.findByText("Ya está en este equipo y entra cuando CoPanda se cierre"),
      ).toBeInTheDocument();
      expect(screen.queryByRole("button", { name: "Actualizar" })).toBeNull();
      expect(screen.queryByRole("button", { name: "Abrir Microsoft Store" })).toBeNull();
      expect(screen.queryByRole("button", { name: "Buscar ahora" })).toBeNull();
    });

    it("a copy the Store did not sell has no button to look now", async () => {
      await given(store({ looked: false, notSold: true }));
      render(<About kept={null} change={nothing} />);
      expect(
        await screen.findByText("La Microsoft Store no actualiza esta copia"),
      ).toBeInTheDocument();
      expect(screen.getByText(/no vino de la Microsoft Store/)).toBeInTheDocument();
      expect(screen.queryByRole("button", { name: "Buscar ahora" })).toBeNull();
    });

    it("shows the download as a bar that fills with what the Store reports", async () => {
      const who = userEvent.setup();
      const said = await given(store({ ready: take }), () => new Promise(() => {}));
      render(<About kept={null} change={nothing} />);
      await who.click(await screen.findByRole("button", { name: "Actualizar" }));
      await said.say("updating", { stage: "getting", far: 40 });
      const bar = await screen.findByRole("progressbar");
      expect(bar).toHaveAttribute("value", "40");
      expect(screen.getByText("Descargando… 40 %")).toBeInTheDocument();
    });

    it("says the Store is installing once the download is done", async () => {
      const who = userEvent.setup();
      const said = await given(store({ ready: take }), () => new Promise(() => {}));
      render(<About kept={null} change={nothing} />);
      await who.click(await screen.findByRole("button", { name: "Actualizar" }));
      await said.say("updating", { stage: "installing", far: 100 });
      expect(
        await screen.findByText("La Microsoft Store está instalando la actualización…"),
      ).toBeInTheDocument();
      expect(screen.queryByRole("progressbar")).toBeNull();
    });

    it("says the update was stopped when the person canceled it in the Store", async () => {
      const who = userEvent.setup();
      await given(store({ ready: take }), () => Promise.reject("stopped"));
      render(<About kept={null} change={nothing} />);
      await who.click(await screen.findByRole("button", { name: "Actualizar" }));
      expect(await screen.findByText("La actualización se detuvo.")).toBeInTheDocument();
    });

    it("says the Store did not answer", async () => {
      const who = userEvent.setup();
      await given(store({ ready: take }), () => Promise.reject("unanswered"));
      render(<About kept={null} change={nothing} />);
      await who.click(await screen.findByRole("button", { name: "Actualizar" }));
      expect(
        await screen.findByText("La Microsoft Store no respondió. Vuelve a intentarlo en un rato."),
      ).toBeInTheDocument();
    });
  });

  describe("outside the Store", () => {
    it("a copy with something it cannot install goes back to offering to look now", async () => {
      await given(looked({ ready: { version: "3.6.0", installs: false } }));
      render(<About kept={null} change={nothing} />);
      expect(await screen.findByRole("button", { name: "Buscar ahora" })).toBeInTheDocument();
    });

    it("after installing, looks again only when the Store did it", async () => {
      const who = userEvent.setup();
      const said = await given(looked({ ready: take }));
      render(<About kept={null} change={nothing} />);
      await who.click(await screen.findByRole("button", { name: "Actualizar" }));
      await waitFor(() => expect(said.asked).toContain("update_install"));
      expect(said.asked.filter((one) => one === "update_ready")).toHaveLength(1);
    });

    it("after the Store installs, looks again", async () => {
      const who = userEvent.setup();
      const said = await given({ ...looked({ ready: take }), route: "store" });
      render(<About kept={null} change={nothing} />);
      await who.click(await screen.findByRole("button", { name: "Actualizar" }));
      await waitFor(() =>
        expect(said.asked.filter((one) => one === "update_ready")).toHaveLength(2),
      );
    });

    it("shows the bar for the download and installing for the rest", async () => {
      const who = userEvent.setup();
      const said = await given(looked({ ready: take }), () => new Promise(() => {}));
      render(<About kept={null} change={nothing} />);
      await who.click(await screen.findByRole("button", { name: "Actualizar" }));
      await said.say("updating", { stage: "getting", far: 10 });
      expect(await screen.findByRole("progressbar")).toHaveAttribute("value", "10");
      await said.say("updating", { stage: "installing", far: 100 });
      expect(await screen.findByText("Instalando…")).toBeInTheDocument();
    });
  });

  describe("the chip on the rail", () => {
    it("names the version waiting", async () => {
      await given(looked({ ready: take }));
      render(<App />);
      expect(await screen.findByText("Ver la 3.6.0 disponible")).toBeInTheDocument();
    });

    it("is not drawn when the version comes empty", async () => {
      const said = await given(looked({ ready: { version: "", installs: true } }));
      render(<App />);
      await screen.findByRole("heading", { level: 1 });
      await waitFor(() => expect(said.asked).toContain("update_ready"));
      expect(screen.queryByText(/Ver la .* disponible/)).toBeNull();
    });
  });

  describe("the way out when the update cannot be taken", () => {
    const web = "Descargar desde la web";
    const releases = "https://github.com/rgdevment/CopyPaste/releases";

    async function failing(why: string, over: Partial<Looked> = {}) {
      const who = userEvent.setup();
      await given(looked({ ready: take, ...over }), () => Promise.reject(why));
      render(<About kept={null} change={nothing} />);
      await who.click(await screen.findByRole("button", { name: "Actualizar" }));
      return who;
    }

    it.each(["failed", "foreign", "boom"])("offers the website after %s", async (why) => {
      const who = await failing(why);
      const { invoke } = await import("@tauri-apps/api/core");
      const link = await screen.findByRole("button", { name: web });
      expect(link.closest(".asks")).not.toBeNull();
      await who.click(link);
      expect(vi.mocked(invoke)).toHaveBeenCalledWith("open_web", { url: releases });
    });

    it.each(["gone", "busy", "publishing", "nothing"])("does not after %s", async (why) => {
      await failing(why);
      await new Promise((done) => setTimeout(done, 50));
      expect(screen.queryByRole("button", { name: web })).toBeNull();
    });

    it("does not for a copy the Store updates", async () => {
      await failing("failed", { route: "store" });
      await new Promise((done) => setTimeout(done, 50));
      expect(screen.queryByRole("button", { name: web })).toBeNull();
    });

    it("goes away when the update is tried again", async () => {
      const who = userEvent.setup();
      let tries = 0;
      await given(looked({ ready: take }), () => {
        tries += 1;
        return tries === 1 ? Promise.reject("failed") : new Promise(() => {});
      });
      render(<About kept={null} change={nothing} />);
      await who.click(await screen.findByRole("button", { name: "Actualizar" }));
      await screen.findByRole("button", { name: web });
      await who.click(screen.getByRole("button", { name: "Actualizar" }));
      await waitFor(() => expect(screen.queryByRole("button", { name: web })).toBeNull());
    });
  });
});
