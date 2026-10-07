import { listen } from "@tauri-apps/api/event";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { useCallback, useEffect, useState } from "react";
import { A_MEGABYTE, empty, type Kept, nearLimit, storageUsed, whereItLives } from "../core";
import { fill, sized, t } from "../locales";
import { Band, Line, wentWrong } from "./Bits";

const KEPT = [7, 30, 90];
const QUOTA = [0, 512, 1024, 2048, 5120, 10240];

export default function History({
  kept,
  change,
}: {
  kept: Kept;
  change: (what: Partial<Kept>) => void;
}) {
  const [where, setWhere] = useState<string | null>(null);
  const [sure, setSure] = useState(false);
  const [said, setSaid] = useState<string | null>(null);
  const [used, setUsed] = useState<number | null>(null);
  const days = kept["keeps-days"];
  const DAYS = KEPT.includes(days) || days === 0 ? KEPT : [...KEPT, days].sort((a, b) => a - b);
  const mb = kept["images-quota-mb"];
  const QUOTAS = QUOTA.includes(mb) ? QUOTA : [...QUOTA, mb].sort((a, b) => a - b);

  const weigh = useCallback(() => {
    storageUsed()
      .then(setUsed)
      .catch(() => setUsed(null));
  }, []);

  useEffect(() => {
    whereItLives()
      .then(setWhere)
      .catch(() => setWhere(null));
  }, []);

  useEffect(() => {
    weigh();
    const heard = ["kept", "crossed", "emptied"].map((name) => listen(name, weigh));
    return () => {
      for (const one of heard) void one.then((drop) => drop());
    };
  }, [weigh]);

  useEffect(() => {
    if (!sure) {
      return;
    }
    const forgets = setTimeout(() => setSure(false), 4_000);
    return () => clearTimeout(forgets);
  }, [sure]);

  return (
    <>
      <h1>{t("railHistory")}</h1>

      <Band says={t("bandKeeps")} />

      <Line says={t("keeps")} why={t("keepsWhy")}>
        <select
          aria-label={t("keeps")}
          value={String(kept["keeps-days"])}
          onChange={(event) => change({ "keeps-days": Number(event.target.value) })}
        >
          {DAYS.map((days) => (
            <option key={days} value={String(days)}>
              {fill("keepsDays", String(days))}
            </option>
          ))}
          <option value="0">{t("keepsForever")}</option>
        </select>
      </Line>

      <Line
        says={t("quota")}
        why={t("quotaWhy")}
        more={
          mb > 0 && used !== null ? (
            <>
              <div className="said-plain">
                {fill("quotaUsed", sized(used), sized(mb * A_MEGABYTE))}
              </div>
              {nearLimit(used, mb) && <div className="said-plain">{t("quotaNear")}</div>}
            </>
          ) : null
        }
      >
        <select
          aria-label={t("quota")}
          value={String(kept["images-quota-mb"])}
          onChange={(event) => change({ "images-quota-mb": Number(event.target.value) })}
        >
          {QUOTAS.map((one) => (
            <option key={one} value={String(one)}>
              {one === 0 ? t("quotaNone") : one >= 1024 ? `${one / 1024} GB` : `${one} MB`}
            </option>
          ))}
        </select>
      </Line>

      <Band says={t("bandWhere")} />

      <Line says={t("where")} why={<span className="path">{where ?? t("whereUnknown")}</span>}>
        <button
          type="button"
          className="mild"
          disabled={!where}
          onClick={() => {
            if (where) void revealItemInDir(where).catch(() => {});
          }}
        >
          {t("whereOpen")}
        </button>
      </Line>

      <Line says={t("empty")} why={said ?? t("emptyWhy")}>
        <button
          type="button"
          className="grave"
          onClick={() => {
            if (!sure) {
              setSure(true);
              return;
            }
            setSure(false);
            empty()
              .then(() => setSaid(t("emptyGone")))
              .catch((why) => setSaid(wentWrong("failedEmpty", why)));
          }}
        >
          {sure ? t("emptySure") : t("emptyDo")}
        </button>
      </Line>
    </>
  );
}
