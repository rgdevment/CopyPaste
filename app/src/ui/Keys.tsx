import { useState } from "react";
import { combination, type Kept, useKeys } from "../core";
import { panelKeys, t } from "../locales";
import { Band, Line } from "./Bits";

export default function Keys({
  kept,
  change,
}: {
  kept: Kept;
  change: (what: Partial<Kept>) => Promise<void>;
}) {
  const { keys, spare, recheck } = useKeys(kept.shortcut);
  const [asking, setAsking] = useState(false);

  return (
    <>
      <h1>{t("railKeys")}</h1>

      <Band says={t("bandOpens")} />

      <Line
        says={t("keys")}
        why={asking ? t("keysAsk") : keys && !keys.bound ? t("keysTaken") : t("keysWhy")}
        more={
          !asking && keys && !keys.bound && spare.length > 0 ? (
            <div className="spare">
              <span>{t("keysFree")}</span>
              {spare.map((one) => (
                <button
                  key={one}
                  type="button"
                  className="keys spare-one"
                  onClick={() => void change({ shortcut: one }).finally(() => recheck(one))}
                >
                  {one.replaceAll("+", " + ")}
                </button>
              ))}
            </div>
          ) : null
        }
      >
        <span className={keys && !keys.bound && !asking ? "keys taken" : "keys"}>
          {kept.shortcut.replaceAll("+", " + ")}
        </span>
        <button
          type="button"
          className="mild"
          onClick={() => setAsking(!asking)}
          onKeyDown={(press) => {
            if (!asking) {
              return;
            }
            press.preventDefault();
            if (press.code === "Escape") {
              setAsking(false);
              return;
            }
            const said = combination(press);
            if (said !== null) {
              setAsking(false);
              void change({ shortcut: said }).finally(() => recheck(said));
            }
          }}
        >
          {asking ? t("keysStop") : t("keysChange")}
        </button>
      </Line>

      <Band says={t("keysTable")} />

      <p className="said-plain">{t("keysTableWhy")}</p>

      <table className="bindings">
        <tbody>
          {panelKeys().map((one) => (
            <tr key={one.keys}>
              <th scope="row">{one.keys}</th>
              <td>{one.does}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </>
  );
}
