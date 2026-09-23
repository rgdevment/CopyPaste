import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";

export type Look = "system" | "light" | "dark";

export type Kept = {
  locale: string | null;
  theme: Look;
  "wakes-with-session": boolean;
  shortcut: string;
  "hides-when-left": boolean;
  "keeps-days": number | null;
  "images-quota-mb": number | null;
};

export const FRESH: Kept = {
  locale: null,
  theme: "system",
  "wakes-with-session": true,
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
