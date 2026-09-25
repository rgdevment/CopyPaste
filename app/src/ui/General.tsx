import { useState } from "react";
import { combination, type Kept, type Look, useKeys, useWaking } from "../core";
import { t } from "../locales";
import { Band, Knob, Line } from "./Bits";

export default function General({
  kept,
  change,
}: {
  kept: Kept;
  change: (what: Partial<Kept>) => void;
}) {
  const { waking, trouble, ask } = useWaking();
  const keys = useKeys();
  const [asking, setAsking] = useState(false);

  return (
    <>
      <h1>{t("railGeneral")}</h1>

      <Band says={t("bandLook")} />

      <Line says={t("tongue")} why={t("tongueWhy")}>
        <select
          aria-label={t("tongue")}
          value={kept.locale ?? ""}
          onChange={(event) => change({ locale: event.target.value || null })}
        >
          <option value="">{t("tongueTheirs")}</option>
          <option value="es">Español</option>
          <option value="en">English</option>
        </select>
      </Line>

      <Line says={t("look")} why={t("lookWhy")}>
        <select
          aria-label={t("look")}
          value={kept.theme}
          onChange={(event) => change({ theme: event.target.value as Look })}
        >
          <option value="system">{t("lookTheirs")}</option>
          <option value="light">{t("lookLight")}</option>
          <option value="dark">{t("lookDark")}</option>
        </select>
      </Line>

      <Band says={t("bandDoes")} />

      {waking?.offered ? (
        <Line
          says={t("wake")}
          why={t("wakeWhy")}
          more={
            waking.theirs ? (
              <div className="said">{t("wakeTheirs")}</div>
            ) : trouble ? (
              <div className="alarm">{trouble}</div>
            ) : null
          }
        >
          <Knob
            on={waking.wakes}
            says={t("wake")}
            asleep={waking.theirs}
            onPress={() => ask(!waking.wakes)}
          />
        </Line>
      ) : (
        <Line says={t("wake")} why={t("wakeMac")} />
      )}

      <Line
        says={t("keys")}
        why={asking ? t("keysAsk") : keys && !keys.bound ? t("keysTaken") : t("keysWhy")}
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
              change({ shortcut: said });
            }
          }}
        >
          {asking ? t("keysStop") : t("keysChange")}
        </button>
      </Line>

      <Line says={t("hides")} why={t("hidesWhy")}>
        <Knob
          on={kept["hides-when-left"]}
          says={t("hides")}
          onPress={() => change({ "hides-when-left": !kept["hides-when-left"] })}
        />
      </Line>
    </>
  );
}
