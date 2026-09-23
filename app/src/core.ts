import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import { adopt } from "./locales";

export type Look = "system" | "light" | "dark";

export type Kept = {
  locale: string | null;
  theme: Look;
  shortcut: string;
  "hides-when-left": boolean;
  "keeps-days": number | null;
  "images-quota-mb": number | null;
};

export const FRESH: Kept = {
  locale: null,
  theme: "system",
  shortcut: "Ctrl+Alt+V",
  "hides-when-left": true,
  "keeps-days": 30,
  "images-quota-mb": null,
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

  useEffect(() => {
    invoke<Kept>("settings")
      .then((one) => {
        setKept(one);
        wear(one.theme);
        adopt(one.locale);
      })
      .catch((why) => {
        setKept(FRESH);
        setTrouble(String(why));
      });
  }, []);

  const change = useCallback((what: Partial<Kept>) => {
    setKept((was) => {
      if (!was) return was;
      const next = { ...was, ...what };
      if (what.theme) wear(what.theme);
      if ("locale" in what) {
        adopt(what.locale ?? null);
        invoke("relabel", { locale: what.locale ?? null }).catch(() => {});
      }
      invoke<Kept>("keep", { config: next })
        .then((landed) => {
          setKept(landed);
          setTrouble(null);
        })
        .catch((why) => setTrouble(String(why)));
      return next;
    });
  }, []);

  return { kept, trouble, change };
}

export function whereItLives() {
  return invoke<string>("where_it_lives");
}

export type Waking = { offered: boolean; wakes: boolean; theirs: boolean };

export function useWaking() {
  const [waking, setWaking] = useState<Waking | null>(null);

  useEffect(() => {
    invoke<Waking>("waking")
      .then(setWaking)
      .catch(() => setWaking(null));
  }, []);

  const ask = useCallback((wanted: boolean) => {
    invoke<Waking>("wake", { wanted })
      .then(setWaking)
      .catch(() => {});
  }, []);

  return { waking, ask };
}
