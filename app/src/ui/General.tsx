import { openUrl } from "@tauri-apps/plugin-opener";
import { type Kept, onMac, PRIVACY_PANE, useTrust, useWaking } from "../core";
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

      <Band says={t("bandTongue")} />

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

      <Band says={t("bandDoes")} />

      {waking?.managed && <Line says={t("wake")} why={t("wakeManaged")} />}

      {waking && !waking.managed && waking.offered && (
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
            asleep={waking.theirs && !waking.wakes}
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
