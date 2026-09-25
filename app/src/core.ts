import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";
import { adopt } from "./locales";

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

  const change = useCallback(
    (what: Partial<Kept>) => {
      const was = held.current;
      if (!was) {
        return;
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
      queue.current = queue.current
        .then(() => invoke<Kept>("keep", { config: next }))
        .then((landed) => {
          if (mine === turn.current) {
            land(landed);
            setTrouble(null);
          }
        })
        .catch((why) => setTrouble(String(why)));
    },
    [land],
  );

  return { kept, trouble, change, look };
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

export function useKeys() {
  const [keys, setKeys] = useState<Keys | null>(null);

  useEffect(() => {
    invoke<Keys>("keys")
      .then(setKeys)
      .catch(() => setKeys(null));
  }, []);

  return keys;
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
