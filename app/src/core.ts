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
