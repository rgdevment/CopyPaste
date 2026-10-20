import { useState } from "react";
import { asKeys, combination, type Kept, onMac, type Role, sameKeys, useKeys } from "../core";
import { panelKeys, t } from "../locales";
import { Band, Line } from "./Bits";

const FIELD: Record<Role, "shortcut" | "plain-shortcut" | "forms-shortcut"> = {
  panel: "shortcut",
  plain: "plain-shortcut",
  forms: "forms-shortcut",
};

const SYSTEM = [
  "Ctrl+C",
  "Ctrl+X",
  "Ctrl+V",
  "Ctrl+Insert",
  "Shift+Insert",
  "Shift+Delete",
  "Cmd+C",
  "Cmd+X",
  "Cmd+V",
];

export default function Keys({
  kept,
  change,
}: {
  kept: Kept;
  change: (what: Partial<Kept>) => Promise<void>;
}) {
  const mac = onMac();

  return (
    <>
      <h1>{t("railKeys")}</h1>

      <Band says={t("bandAnywhere")} />

      <Shortcut which="panel" says={t("keys")} why={t("keysWhy")} kept={kept} change={change} />

      <Line says={t("keysAsCopied")} why={t("keysAsCopiedWhy")}>
        <span className="keys">{asKeys(mac ? "Cmd+V" : "Ctrl+V", mac)}</span>
      </Line>

      <Shortcut
        which="plain"
        says={t("keysPlain")}
        why={t("keysPlainWhy")}
        kept={kept}
        change={change}
      />

      <Shortcut
        which="forms"
        says={t("keysForms")}
        why={t("keysFormsWhy")}
        kept={kept}
        change={change}
      />

      <Band says={t("keysTable")} />

      <p className="said-plain">{t("keysTableWhy")}</p>

      <table className="bindings">
        <tbody>
          {panelKeys(mac).map((one) => (
            <tr key={one.id}>
              <th scope="row" aria-label={one.said}>
                {one.keys}
              </th>
              <td>{one.does}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </>
  );
}

function refusalOf(system: boolean, repeated: boolean): "keysSystem" | "keysTwice" | null {
  if (system) {
    return "keysSystem";
  }
  return repeated ? "keysTwice" : null;
}

function Shortcut({
  which: role,
  says,
  why,
  kept,
  change,
}: {
  which: Role;
  says: string;
  why: string;
  kept: Kept;
  change: (what: Partial<Kept>) => Promise<void>;
}) {
  const field = FIELD[role];
  const said = kept[field] ?? "";
  const { keys, spare, recheck } = useKeys(said, role);
  const [asking, setAsking] = useState(false);
  const [refused, setRefused] = useState<"keysTwice" | "keysSystem" | null>(null);
  const mac = onMac();
  const off = said === "";
  const taken = !asking && !off && keys !== null && !keys.bound;
  const take = (one: string) => {
    const others = (Object.keys(FIELD) as Role[]).filter((other) => other !== role);
    const repeated = others.some((other) => sameKeys(one, kept[FIELD[other]] ?? ""));
    const system = SYSTEM.some((reserved) => sameKeys(one, reserved));
    const why = refusalOf(system, repeated);
    setRefused(why);
    if (why === null) {
      void change({ [field]: one }).finally(() => recheck(one));
    }
  };
  const told = asking
    ? t("keysAsk")
    : refused
      ? t(refused)
      : taken
        ? t(role === "panel" ? "keysTaken" : "keysTakenPaste")
        : why;

  return (
    <Line
      says={says}
      why={told}
      more={
        taken && spare.length > 0 ? (
          <div className="spare">
            <span>{t("keysFree")}</span>
            {spare.map((one) => (
              <button key={one} type="button" className="keys spare-one" onClick={() => take(one)}>
                {asKeys(one, mac)}
              </button>
            ))}
          </div>
        ) : null
      }
    >
      <span className={taken ? "keys taken" : off ? "keys off" : "keys"}>
        {off ? t("keysOff") : asKeys(said, mac)}
      </span>
      <button
        type="button"
        className="mild"
        onClick={() => {
          setRefused(null);
          setAsking(!asking);
        }}
        onKeyDown={(press) => {
          if (!asking) {
            return;
          }
          press.preventDefault();
          if (press.code === "Escape") {
            setAsking(false);
            return;
          }
          const pressed = combination(press);
          if (pressed !== null) {
            setAsking(false);
            take(pressed);
          }
        }}
      >
        {asking ? t("keysStop") : off ? t("keysSet") : t("keysChange")}
      </button>
      {role !== "panel" && !off && !asking ? (
        <button type="button" className="mild" onClick={() => take("")}>
          {t("keysClear")}
        </button>
      ) : null}
    </Line>
  );
}
