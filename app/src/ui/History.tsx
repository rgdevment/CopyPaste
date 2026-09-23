import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { useEffect, useState } from "react";
import { type Kept, whereItLives } from "../core";
import { fill, t } from "../locales";
import { Band, Line } from "./Bits";

const KEPT = [7, 30, 90];

export default function History({
  kept,
  change,
}: {
  kept: Kept;
  change: (what: Partial<Kept>) => void;
}) {
  const [where, setWhere] = useState<string | null>(null);
  const days = kept["keeps-days"];
  const DAYS = KEPT.includes(days) || days === 0 ? KEPT : [...KEPT, days].sort((a, b) => a - b);

  useEffect(() => {
    whereItLives()
      .then(setWhere)
      .catch(() => setWhere(null));
  }, []);

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

      <Line says={t("quota")} why={t("quotaWhy")}>
        <select
          aria-label={t("quota")}
          value={String(kept["images-quota-mb"])}
          onChange={(event) => change({ "images-quota-mb": Number(event.target.value) })}
        >
          <option value="0">{t("quotaNone")}</option>
          <option value="256">256 MB</option>
          <option value="512">512 MB</option>
          <option value="1024">1 GB</option>
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

      <Line says={t("empty")} why={t("emptyWhy")}>
        <button type="button" className="grave" disabled>
          {t("emptyDo")}
        </button>
      </Line>
    </>
  );
}
