import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useEffect, useState } from "react";
import { fill, t } from "../locales";
import { CloudOff, Code, Gift, Info, Key } from "./Icons";

const STARS = "https://github.com/rgdevment/CopyPaste";
const SPONSOR = "https://github.com/sponsors/rgdevment";
const COFFEE = "https://buymeacoffee.com/rgdevment";
const ALTERNATIVE = "https://alternativeto.net/software/copypaste/about/";
const RATING = "ms-windows-store://review/?ProductId=9NBJRZF3K856";
const PRIVACY = "https://github.com/rgdevment/CopyPaste/blob/main/PRIVACY.md";
const NOTICES = "https://github.com/rgdevment/CopyPaste/blob/main/THIRD-PARTY.md";
const onMac = navigator.userAgent.includes("Macintosh");

const TOOLS = [
  {
    name: "Tisty",
    hue: "#7c6cf0",
    says: "toolTisty",
    at: "https://github.com/rgdevment/Tisty",
  },
  {
    name: "LinkUnbound",
    hue: "#3fbfa6",
    says: "toolLinkUnbound",
    at: "https://github.com/rgdevment/LinkUnbound",
  },
];

export default function About() {
  const [trouble, setTrouble] = useState<string | null>(null);
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    getVersion()
      .then(setVersion)
      .catch(() => setVersion(null));
  }, []);

  const go = (where: string) => {
    setTrouble(null);
    openUrl(where).catch(() => setTrouble(fill("linkRefused", where)));
  };

  return (
    <>
      {trouble && <p className="alarm">{trouble}</p>}

      <div className="brow">
        <span className="mark" aria-hidden="true">
          C
        </span>
        <span>
          <h1>CopyPaste</h1>
          <span className="line2">
            <span>{version ?? "—"}</span>
            <i />
            <span>GPL-3.0</span>
          </span>
        </span>
      </div>

      <div className="what-is">
        <p className="eyebrow">
          <Info />
          {t("aboutIs")}
        </p>
        <p>{t("aboutWhat")}</p>
        <p>{t("aboutPrivacy")}</p>
        <div className="badges">
          <span className="badge">
            <Key />
            {t("badgeLocal")}
          </span>
          <span className="badge">
            <Code />
            {t("badgeOpen")}
          </span>
          <span className="badge">
            <Gift />
            {t("badgeFree")}
          </span>
          <span className="badge">
            <CloudOff />
            {t("badgeQuiet")}
          </span>
        </div>
      </div>

      <div className="newer">
        <span className="pip" />
        <span className="grow">
          <b>{t("updateNone")}</b>
          <span>{t("updateWhen")}</span>
        </span>
        <button type="button" className="mild" disabled>
          {t("updateLook")}
          <span className="soon">{t("soon")}</span>
        </button>
      </div>

      <div className="rule">{t("supportTitle")}</div>
      <p className="quiet">{t("supportWhy")}</p>
      <div className="gives" style={{ marginTop: 10 }}>
        <button type="button" className="give" onClick={() => go(STARS)}>
          <svg viewBox="0 0 16 16" aria-hidden="true">
            <path
              fill="#e3b341"
              d="M8 1.2l2.1 4.3 4.7.7-3.4 3.3.8 4.7L8 12l-4.2 2.2.8-4.7L1.2 6.2l4.7-.7L8 1.2z"
            />
          </svg>
          <span>
            <b>{t("supportStar")}</b>
            <span>github.com/rgdevment/CopyPaste</span>
          </span>
        </button>
        {!onMac && (
          <button type="button" className="give" onClick={() => go(RATING)}>
            <svg viewBox="0 0 16 16" aria-hidden="true">
              <path
                fill="#0078d4"
                d="M2 3h12a1 1 0 0 1 1 1v7a1 1 0 0 1-1 1H6l-4 3V4a1 1 0 0 1 1-1Z"
              />
            </svg>
            <span>
              <b>{t("supportRate")}</b>
              <span>Microsoft Store</span>
            </span>
          </button>
        )}
        <button type="button" className="give" onClick={() => go(SPONSOR)}>
          <svg viewBox="0 0 16 16" aria-hidden="true">
            <path
              fill="#db61a2"
              d="M8 14.25 6.84 13.2C2.72 9.47 0 7.01 0 4.5 0 2.42 1.57 1 3.5 1c1.1 0 2.16.51 2.84 1.32h1.32C8.34 1.51 9.4 1 10.5 1 12.43 1 14 2.42 14 4.5c0 2.51-2.72 4.97-6.84 8.7L8 14.25Z"
            />
          </svg>
          <span>
            <b>{t("supportSponsor")}</b>
            <span>github.com/sponsors</span>
          </span>
        </button>
        <button type="button" className="give" onClick={() => go(COFFEE)}>
          <svg viewBox="0 0 16 16" aria-hidden="true">
            <path
              fill="#c8892a"
              d="M2 5h9v5a3 3 0 0 1-3 3H5a3 3 0 0 1-3-3V5Zm10 0h1.5A2.5 2.5 0 0 1 16 7.5 2.5 2.5 0 0 1 13.5 10H12V5ZM2 14h9v1H2v-1Z"
            />
          </svg>
          <span>
            <b>{t("supportCoffee")}</b>
            <span>buymeacoffee.com</span>
          </span>
        </button>
      </div>

      <div className="rule">{t("otherTools")}</div>
      {TOOLS.map((tool) => (
        <button key={tool.name} type="button" className="tool" onClick={() => go(tool.at)}>
          <span className="ico" style={{ background: tool.hue }}>
            {tool.name.slice(0, 1)}
          </span>
          <span style={{ flex: 1, minWidth: 0 }}>
            <b>{tool.name}</b>
            <span>{t(tool.says as "toolTisty")}</span>
          </span>
        </button>
      ))}

      <div className="trouble">
        <div className="rule" style={{ marginTop: 0 }}>
          {t("troubleTitle")}
        </div>
        <p className="quiet">
          {t("troubleWhat")} <em>{t("troubleNeverSent")}</em>
          {t("troubleYours")}
        </p>
        <div className="feet">
          <button type="button" className="mild" disabled>
            {t("troubleReport")}
            <span className="soon">{t("soon")}</span>
          </button>
          <button type="button" className="mild" disabled>
            {t("troubleLog")}
            <span className="soon">{t("soon")}</span>
          </button>
        </div>
      </div>

      <div className="links">
        <button type="button" onClick={() => go(STARS)}>
          {t("aboutRepo")}
        </button>
        <button type="button" onClick={() => go(ALTERNATIVE)}>
          AlternativeTo
        </button>
        <button type="button" onClick={() => go(PRIVACY)}>
          {t("aboutPrivacyLink")}
        </button>
        <button type="button" onClick={() => go(NOTICES)}>
          {t("aboutNotices")}
        </button>
      </div>
    </>
  );
}
