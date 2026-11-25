import { type KeyboardEvent, useId, useRef, useState } from "react";
import { t } from "../locales";

export type Pickable = { id: string; label: string };

const folded = (said: string) =>
  said
    .normalize("NFD")
    .replace(/\p{Diacritic}/gu, "")
    .toLowerCase();

export function matching(all: Pickable[], query: string): Pickable[] {
  const asked = folded(query.trim());
  if (!asked) return all;
  return all.filter((one) => folded(one.label).includes(asked) || folded(one.id).includes(asked));
}

export default function FontPicker({
  says,
  first,
  others,
  chosen,
  pick,
}: {
  says: string;
  first: string;
  others: Pickable[];
  chosen: string | null;
  pick: (id: string | null) => void;
}) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [lit, setLit] = useState(0);
  const opener = useRef<HTMLButtonElement>(null);
  const list = useId();
  const all = [{ id: "", label: first }, ...others];
  const current = all.find((one) => one.id === (chosen ?? "")) ?? all[0];
  const shown = matching(all, query);

  const close = (refocus: boolean) => {
    setOpen(false);
    setQuery("");
    if (refocus) opener.current?.focus();
  };

  const choose = (one: Pickable) => {
    close(true);
    if (one.id !== current.id) pick(one.id || null);
  };

  const toggle = () => {
    if (open) {
      close(true);
      return;
    }
    setLit(Math.max(0, all.indexOf(current)));
    setOpen(true);
  };

  const steer = (event: KeyboardEvent<HTMLInputElement>) => {
    const last = shown.length - 1;
    const moves: Record<string, number> = {
      ArrowDown: Math.min(lit + 1, last),
      ArrowUp: Math.max(lit - 1, 0),
      Home: 0,
      End: last,
    };
    if (event.key in moves) {
      event.preventDefault();
      setLit(moves[event.key]);
      document.getElementById(`${list}-${moves[event.key]}`)?.scrollIntoView({ block: "nearest" });
    } else if (event.key === "Enter" && shown[lit]) {
      event.preventDefault();
      choose(shown[lit]);
    } else if (event.key === "Escape") {
      event.preventDefault();
      close(true);
    }
  };

  return (
    <div className="font-picker">
      <button
        ref={opener}
        type="button"
        className="font-picked"
        aria-label={`${says}: ${current.label}`}
        aria-haspopup="listbox"
        aria-expanded={open}
        onMouseDown={(event) => {
          if (open) event.preventDefault();
        }}
        onClick={toggle}
      >
        {current.label}
      </button>
      {open && (
        <div className="font-drop">
          <input
            type="search"
            role="combobox"
            aria-label={t("fontSearch")}
            aria-expanded="true"
            aria-controls={list}
            aria-activedescendant={shown[lit] ? `${list}-${lit}` : undefined}
            placeholder={t("fontSearch")}
            value={query}
            // biome-ignore lint/a11y/noAutofocus: the search takes the keys as soon as the list opens
            autoFocus
            onChange={(event) => {
              setQuery(event.target.value);
              setLit(0);
            }}
            onKeyDown={steer}
            onBlur={() => close(false)}
          />
          <div
            id={list}
            role="listbox"
            aria-label={says}
            className="font-options"
            onMouseDown={(event) => event.preventDefault()}
          >
            {shown.map((one, at) => (
              <div
                key={one.id || "first"}
                id={`${list}-${at}`}
                role="option"
                tabIndex={-1}
                aria-selected={one.id === current.id}
                className={at === lit ? "lit" : undefined}
                onMouseEnter={() => setLit(at)}
                onClick={() => choose(one)}
                onKeyDown={(event) => {
                  if (event.key === "Enter" || event.key === " ") choose(one);
                }}
              >
                {one.label}
              </div>
            ))}
            {shown.length === 0 && <p className="font-none">{t("fontNone")}</p>}
          </div>
        </div>
      )}
    </div>
  );
}
