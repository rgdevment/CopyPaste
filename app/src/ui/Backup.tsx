import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { t } from "../locales";
import { Band, Line } from "./Bits";

type Former = { path: string; bytes: number };

function weighed(bytes: number) {
  const mb = bytes / (1024 * 1024);
  return mb >= 1 ? `${mb.toFixed(1)} MB` : `${Math.round(bytes / 1024)} KB`;
}

export default function Backup() {
  const [former, setFormer] = useState<Former | null>(null);

  const [trouble, setTrouble] = useState<string | null>(null);

  useEffect(() => {
    invoke<Former | null>("former")
      .then(setFormer)
      .catch((why) => setTrouble(String(why)));
  }, []);

  return (
    <>
      <h1>{t("railBackup")}</h1>

      <Band says={t("bandCopies")} />

      <Line says={t("out")} why={t("outWhy")}>
        <button type="button" className="strong" disabled>
          {t("outDo")}
        </button>
        <span className="soon">{t("soon")}</span>
      </Line>

      <Line says={t("in")} why={t("inWhy")}>
        <button type="button" className="mild" disabled>
          {t("inDo")}
        </button>
        <span className="soon">{t("soon")}</span>
      </Line>

      <Band says={t("bandFormer")} />

      {former ? (
        <Line
          says={t("former")}
          why={<span className="path">{`${former.path} · ${weighed(former.bytes)}`}</span>}
          more={<div className="said">{t("formerLoses")}</div>}
        >
          <button type="button" className="mild" disabled>
            {t("formerBring")}
          </button>
          <button type="button" className="grave" disabled>
            {t("formerDrop")}
          </button>
          <span className="soon">{t("soon")}</span>
        </Line>
      ) : (
        <Line says={t("former")} why={trouble ?? t("formerNone")} />
      )}
    </>
  );
}
