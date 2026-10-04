import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { adopt } from "./locales";

export const notices = (): Promise<string> => invoke<string>("notices");

export const licences = (): Promise<string> => invoke<string>("licences");

export type Look = "system" | "light" | "dark";

export type Kept = {
  locale: string | null;
  theme: Look;
  shortcut: string;
  "hides-when-left": boolean;
  "keeps-days": number;
  "images-quota-mb": number;
};

export function wear(look: Look) {
  const root = document.documentElement;
  if (look === "system") {
    root.removeAttribute("data-theme");
    return;
  }
  root.setAttribute("data-theme", look);
}

export function useKept() {
  const [kept, setKept] = useState<Kept | null>(null);
  const [trouble, setTrouble] = useState<string | null>(null);
  const held = useRef<Kept | null>(null);
  const turn = useRef(0);
  const queue = useRef<Promise<unknown>>(Promise.resolve());
  const pending = useRef(0);

  const land = useCallback((one: Kept) => {
    held.current = one;
    setKept(one);
    wear(one.theme);
    adopt(one.locale);
    invoke("relabel", { locale: one.locale }).catch(() => {});
  }, []);

  const look = useCallback(() => {
    setTrouble(null);
    invoke<Kept>("settings")
      .then(land)
      .catch((why) => setTrouble(String(why)));
  }, [land]);

  useEffect(look, [look]);

  useEffect(() => {
    const heard = listen<Kept>("kept", (event) => {
      if (pending.current === 0) {
        land(event.payload);
      }
    });
    return () => {
      void heard.then((drop) => drop());
    };
  }, [land]);

  const change = useCallback(
    (what: Partial<Kept>) => {
      const was = held.current;
      if (!was) {
        return Promise.resolve();
      }
      const next = { ...was, ...what };
      held.current = next;
      setKept(next);
      if (what.theme) {
        wear(what.theme);
      }
      if ("locale" in what) {
        adopt(what.locale ?? null);
      }

      const mine = ++turn.current;
      pending.current += 1;
      queue.current = queue.current
        .then(() => invoke<Kept>("keep", { config: next }))
        .then((landed) => {
          if (mine === turn.current) {
            land(landed);
            setTrouble(null);
          }
        })
        .catch((why) => {
          look();
          setTrouble(String(why));
        })
        .finally(() => {
          pending.current -= 1;
        });
      return queue.current.then(() => undefined);
    },
    [land, look],
  );

  return { kept, trouble, change, look };
}

export function onMac() {
  return /Mac|iPhone|iPad/.test(navigator.userAgent);
}

const MAC_KEYS: Record<string, string> = { Ctrl: "⌃", Alt: "⌥", Shift: "⇧", Cmd: "⌘" };

// Apple lists them Control, Option, Shift, Command, whatever order the combination was stored in
const MAC_ORDER = Object.values(MAC_KEYS);

export function caps(shortcut: string, mac: boolean): string[] {
  const keys = shortcut
    .split("+")
    .map((one) => one.trim())
    .filter((one) => one.length > 0)
    .map((one) => (mac ? (MAC_KEYS[one] ?? one) : one));
  if (!mac) {
    return keys;
  }
  const held = MAC_ORDER.filter((one) => keys.includes(one));
  return [...held, ...keys.filter((one) => !MAC_ORDER.includes(one))];
}

export function asKeys(shortcut: string, mac: boolean): string {
  const keys = caps(shortcut, mac);
  if (!mac) {
    return keys.join(" + ");
  }
  // the glyphs sit against each other, but anything spelled out would weld into its neighbour
  return keys.reduce((said, one, at) => {
    const apart = at > 0 && (keys[at - 1].length > 1 || one.length > 1);
    return said + (apart ? " " : "") + one;
  }, "");
}

export function whereItLives() {
  return invoke<string>("where_it_lives");
}

export function combination(press: {
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
  code: string;
}): string | null {
  const held: string[] = [];
  if (press.ctrlKey) held.push("Ctrl");
  if (press.altKey) held.push("Alt");
  if (press.shiftKey) held.push("Shift");
  if (press.metaKey) held.push("Cmd");
  const code = press.code;
  let key: string | null = null;
  if (/^Key[A-Z]$/.test(code)) {
    key = code.slice(3);
  } else if (/^Digit[0-9]$/.test(code)) {
    key = code.slice(5);
  } else if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code)) {
    key = code;
  } else if (code === "Space") {
    key = "Space";
  }
  if (key === null || held.length === 0) {
    return null;
  }
  held.push(key);
  return held.join("+");
}

export function useTrouble() {
  const [said, setSaid] = useState<string | null>(null);

  useEffect(() => {
    const ask = () => {
      invoke<string | null>("trouble")
        .then(setSaid)
        .catch(() => setSaid(null));
    };
    ask();
    const again = setInterval(ask, 5_000);
    return () => clearInterval(again);
  }, []);

  return said;
}

export function empty(): Promise<void> {
  return invoke<void>("empty");
}

export type Keys = { wanted: string; bound: boolean };

export function useKeys(shortcut: string) {
  const [keys, setKeys] = useState<Keys | null>(null);
  const [spare, setSpare] = useState<string[]>([]);
  const live = useRef(true);

  const ask = useCallback((taken: string) => {
    invoke<Keys>("keys")
      .then((said) => {
        if (!live.current) {
          return;
        }
        setKeys(said);
        if (said.bound) {
          setSpare([]);
          return;
        }
        invoke<string[]>("spare", { taken })
          .then((free) => live.current && setSpare(free))
          .catch(() => live.current && setSpare([]));
      })
      .catch(() => live.current && setKeys(null));
  }, []);

  useEffect(() => {
    live.current = true;
    ask(shortcut);
    return () => {
      live.current = false;
    };
  }, [shortcut, ask]);

  return { keys, spare, recheck: ask };
}

export type Waking = { offered: boolean; wakes: boolean; theirs: boolean };

export function useWaking() {
  const [waking, setWaking] = useState<Waking | null>(null);
  const [trouble, setTrouble] = useState<string | null>(null);

  useEffect(() => {
    invoke<Waking>("waking")
      .then(setWaking)
      .catch((why) => setTrouble(String(why)));
  }, []);

  const ask = useCallback((wanted: boolean) => {
    invoke<Waking>("wake", { wanted })
      .then((one) => {
        setWaking(one);
        setTrouble(null);
      })
      .catch((why) => setTrouble(String(why)));
  }, []);

  return { waking, trouble, ask };
}

export type Trust = {
  offered: boolean;
  pastes: boolean;
  secureInput: boolean;
};

export const PRIVACY_PANE =
  "x-apple.systempreferences://com.apple.preference.security?Privacy_Accessibility";

export function useTrust() {
  const [trust, setTrust] = useState<Trust | null>(null);
  const [asked, setAsked] = useState(false);

  const look = useCallback(() => {
    invoke<Trust>("trust")
      .then(setTrust)
      .catch(() => setTrust(null));
  }, []);

  useEffect(look, [look]);

  useEffect(() => {
    const again = setInterval(look, 4_000);
    return () => clearInterval(again);
  }, [look]);

  const ask = useCallback(() => {
    setAsked(true);
    invoke<Trust>("ask_trust")
      .then(setTrust)
      .catch(() => {});
  }, []);

  return { trust, asked, ask };
}

export type Route = "store" | "brew" | "download";

export type Ready = { version: string; installs: boolean };

export type Looked = { route: Route; looked: boolean; ready: Ready | null };

export function useUpdate() {
  const [seen, setSeen] = useState<Looked | null>(null);
  const [busy, setBusy] = useState(false);
  const [trouble, setTrouble] = useState<string | null>(null);
  const alive = useRef(true);
  const inFlight = useRef(false);

  useEffect(() => {
    alive.current = true;
    return () => {
      alive.current = false;
    };
  }, []);

  const look = useCallback((nowPlease: boolean) => {
    if (inFlight.current) {
      return;
    }
    inFlight.current = true;
    setTrouble(null);
    setBusy(true);
    invoke<Looked>("update_ready", { nowPlease })
      .then((one) => {
        if (alive.current) setSeen(one);
      })
      .catch((why) => {
        if (alive.current) setTrouble(String(why));
      })
      .finally(() => {
        inFlight.current = false;
        if (alive.current) setBusy(false);
      });
  }, []);

  useEffect(() => look(false), [look]);

  const install = useCallback(() => {
    setTrouble(null);
    setBusy(true);
    invoke<void>("update_install").catch((why) => {
      setTrouble(String(why));
      setBusy(false);
      // the backend forgets what it could not install, so the offer goes with it — and looking
      // again here would wipe the reason before anyone could read it
      setSeen((was) => (was ? { ...was, ready: null } : was));
    });
  }, []);

  return { seen, busy, trouble, look, install };
}
