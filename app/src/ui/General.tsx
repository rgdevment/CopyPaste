import { openUrl } from "@tauri-apps/plugin-opener";
import { type Kept, type Look, onMac, PRIVACY_PANE, useTrust, useWaking } from "../core";
import { t } from "../locales";
import { Band, Knob, Line } from "./Bits";

export default function General({
  kept,
  change,
}: {
  kept: Kept;
  change: (what: Partial<Kept>) => Promise<void>;
}) {
  const { waking, trouble, ask } = useWaking();
  const { trust, asked, ask: askTrust } = useTrust();

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

      {waking?.managed && <Line says={t("wake")} why={t("wakeManaged")} />}

      {waking && !waking.managed && (
        <Line
          says={t("wake")}
          why={t("wakeWhy")}
          more={
            waking.theirs ? (
              <div className="said">{t(onMac() ? "wakeTheirsMac" : "wakeTheirs")}</div>
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
      )}

      {trust?.offered && (
        <Line
          says={t("trust")}
          why={trust.pastes ? t("trustGranted") : t("trustMissing")}
          more={
            <>
              {trust.secureInput && <div className="said">{t("trustSecure")}</div>}
              {trust.clipboard !== "allowed" && (
                <div className="said">
                  {trust.clipboard === "denied" ? t("clipboardDenied") : t("clipboardAsks")}
                </div>
              )}
            </>
          }
        >
          {trust.pastes ? (
            <Knob on says={t("trust")} asleep onPress={() => {}} />
          ) : asked ? (
            <button
              type="button"
              className="strong"
              onClick={() => {
                void openUrl(PRIVACY_PANE).catch(() => {});
              }}
            >
              {t("trustOpen")}
            </button>
          ) : (
            <button type="button" className="strong" onClick={askTrust}>
              {t("trustAsk")}
            </button>
          )}
        </Line>
      )}

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
