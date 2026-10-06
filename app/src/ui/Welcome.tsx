import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { openUrl } from "@tauri-apps/plugin-opener";
import { type ReactNode, useCallback, useEffect, useMemo, useState } from "react";
import { asKeys, caps, type Kept, onMac, type Trust, useKept, useKeys } from "../core";
import { fill, inEnglish, items, oneKey, panelKey, t } from "../locales";
import news from "../news.json";
import Chrome from "./Chrome";

const RELEASES = "https://github.com/rgdevment/CopyPaste/releases";

export type Greeting =
  | { kind: "tour"; former: boolean }
  | { kind: "news"; versions: string[] }
  | { kind: "keys" }
  | { kind: "nothing" };

type Step = "former" | "hello" | "trust" | "keys" | "where" | "use" | "news";

type Former = {
  items: number;
  pictures: number;
  labelled: number;
  came: number | null;
  unreadable: string | null;
};

type Told = { title: string; said: string };

export function steps(former: boolean, asksTrust: boolean, tellsNews = false): Step[] {
  return [
    ...(former ? (["former"] as const) : []),
    "hello",
    ...(asksTrust ? (["trust"] as const) : []),
    "keys",
    "where",
    "use",
    ...(tellsNews ? (["news"] as const) : []),
  ];
}

export function newestStable(): string | null {
  const stable = news.map((one) => one.version).filter((one) => !one.includes("-"));
  const parts = (one: string) => one.split(".").map(Number);
  stable.sort((a, b) => {
    const [x, y] = [parts(a), parts(b)];
    return y[0] - x[0] || y[1] - x[1] || y[2] - x[2];
  });
  return stable[0] ?? null;
}

export function toldFor(
  versions: string[],
  english: boolean,
  mac: boolean,
): { version: string; told: Told[] }[] {
  return versions.flatMap((version) => {
    const found = news.find((one) => one.version === version);
    if (!found) {
      return [];
    }
    const told = (english ? found.en : found.es).map((one) => ({
      ...one,
      said: one.said.replace("{name}", oneKey("name", mac)),
    }));
    return [{ version, told }];
  });
}

function leave() {
  void getCurrentWindow().close();
}

function settings(rail: string) {
  invoke("open_settings", { rail }).catch(() => {});
}

export default function Welcome() {
  const { kept, trouble, change } = useKept();
  const [greeting, setGreeting] = useState<Greeting | null>(null);

  useEffect(() => {
    invoke<Greeting>("greeting")
      .then(setGreeting)
      .catch(() => setGreeting({ kind: "nothing" }));
    const heard = listen<Greeting>("greeting", (event) => setGreeting(event.payload));
    return () => {
      void heard.then((drop) => drop());
    };
  }, []);

  const lost = greeting?.kind === "nothing" || (trouble !== null && kept === null);
  useEffect(() => {
    if (lost) {
      leave();
    }
  }, [lost]);

  return (
    <>
      <Chrome knobs={false} />
      {kept && greeting?.kind === "tour" && (
        <Tour kept={kept} change={change} former={greeting.former} />
      )}
      {kept && greeting?.kind === "news" && <News versions={greeting.versions} />}
      {kept && greeting?.kind === "keys" && (
        <TryIt kept={kept} change={change} mac={onMac()} dots={null} next={leave} alone />
      )}
    </>
  );
}

function Tour({
  kept,
  change,
  former,
}: {
  kept: Kept;
  change: (what: Partial<Kept>) => Promise<void>;
  former: boolean;
}) {
  const mac = onMac();
  const [old, setOld] = useState<Former | null>(null);
  const [looked, setLooked] = useState(!former);
  const [trustAtStart, setTrustAtStart] = useState<Trust | null>(null);
  const [trustLooked, setTrustLooked] = useState(!mac);
  const [at, setAt] = useState(0);

  useEffect(() => {
    if (!former) {
      return;
    }
    invoke<Former | null>("former", { at: Date.now() })
      .then(setOld)
      .catch(() => setOld(null))
      .finally(() => setLooked(true));
  }, [former]);

  useEffect(() => {
    if (!mac) {
      return;
    }
    invoke<Trust>("trust")
      .then(setTrustAtStart)
      .catch(() => setTrustAtStart(null))
      .finally(() => setTrustLooked(true));
  }, [mac]);

  const worthBringing = !!old && !old.unreadable && old.items > 0 && (old.came ?? 0) === 0;
  const asksTrust = !!trustAtStart?.offered && !trustAtStart.pastes;
  const newest = former ? newestStable() : null;
  const all = useMemo(
    () => steps(worthBringing, asksTrust, newest !== null),
    [worthBringing, asksTrust, newest],
  );

  if (!looked || !trustLooked) {
    return null;
  }

  const step = all[Math.min(at, all.length - 1)];
  const next = () => (at + 1 < all.length ? setAt(at + 1) : leave());
  const dots = <Dots all={all} at={at} />;

  switch (step) {
    case "former":
      return <Bring old={old} dots={dots} next={next} />;
    case "hello":
      return (
        <Screen
          dots={dots}
          left={<Quiet says={t("welcomeSkip")} onPress={leave} />}
          right={<Strong says={t("welcomeStart")} onPress={next} />}
        >
          <div className="welcome-hello">
            <div className="welcome-mark">C</div>
            <h1>{t("welcomeHello")}</h1>
            <p>{t("welcomeHelloWhy")}</p>
            <div className="welcome-pills">
              <span>{t("welcomeLocal")}</span>
              <span>{t("welcomeNoAccount")}</span>
              <span>{t("welcomeNoTracking")}</span>
            </div>
          </div>
        </Screen>
      );
    case "trust":
      return <Permission dots={dots} next={next} />;
    case "news":
      return newest ? <News versions={[newest]} /> : null;
    case "keys":
      return <TryIt kept={kept} change={change} mac={mac} dots={dots} next={next} />;
    case "where":
      return (
        <Screen
          dots={dots}
          left={<Quiet says={t("welcomeSkip")} onPress={leave} />}
          right={<Strong says={t("welcomeNext")} onPress={next} />}
        >
          <Heading
            over={t("welcomeWhereOver")}
            title={mac ? t("welcomeWhereTitleMac") : t("welcomeWhereTitle")}
            said={mac ? t("welcomeWhereWhyMac") : t("welcomeWhereWhy")}
          />
          <div className="welcome-where">
            <dl>
              <div>
                <dt>{t("welcomeClick")}</dt>
                <dd>{t("welcomeClickDoes")}</dd>
              </div>
              <div>
                <dt>{t("welcomeRightClick")}</dt>
                <dd>{t("welcomeRightClickDoes")}</dd>
              </div>
            </dl>
            {mac ? <MenuBar /> : <Tray />}
          </div>
        </Screen>
      );
    case "use":
      return (
        <Screen
          dots={dots}
          left={
            <Quiet
              says={t("welcomeOpenSettings")}
              onPress={() => {
                settings("general");
                leave();
              }}
            />
          }
          right={
            <Strong
              says={at + 1 < all.length ? t("welcomeNext") : t("welcomeDone")}
              onPress={next}
            />
          }
        >
          <Heading
            over={t("welcomeUseOver")}
            title={t("welcomeUseTitle")}
            said={t("welcomeUseWhy")}
          />
          <dl className="welcome-use">
            {[
              [t("welcomeUseType"), t("welcomeUseTypeDoes")],
              [panelKey("paste", mac), t("welcomeUseEnterDoes")],
              [panelKey("plain", mac), t("welcomeUsePlainDoes")],
              [panelKey("settings", mac), t("welcomeUseSettingsDoes")],
            ].map(([keys, does]) => (
              <div key={keys}>
                <dt>{keys}</dt>
                <dd>{does}</dd>
              </div>
            ))}
          </dl>
          <button type="button" className="welcome-link" onClick={() => settings("keys")}>
            {t("welcomeAllKeys")}
          </button>
        </Screen>
      );
  }
}

function TryIt({
  kept,
  change,
  mac,
  dots,
  next,
  alone = false,
}: {
  kept: Kept;
  change: (what: Partial<Kept>) => Promise<void>;
  mac: boolean;
  dots: ReactNode;
  next: () => void;
  alone?: boolean;
}) {
  const { keys, spare, recheck } = useKeys(kept.shortcut);
  const over = alone ? t("welcomeKeysAloneOver") : t("welcomeKeysOver");
  const skip = alone ? t("chromeClose") : t("welcomeSkipStep");
  const [tried, setTried] = useState(false);

  useEffect(() => {
    const heard = listen("panel-shown", () => setTried(true));
    return () => {
      void heard.then((drop) => drop());
    };
  }, []);

  const taken = keys !== null && !keys.bound;
  const shown = asKeys(kept.shortcut, mac);

  if (taken) {
    return (
      <Screen dots={dots} left={<Quiet says={skip} onPress={next} />} right={null}>
        <Heading
          over={over}
          title={t("welcomeKeysTaken")}
          said={fill(spare.length > 0 ? "welcomeKeysTakenWhy" : "welcomeKeysTakenAlone", shown)}
        />
        {spare.length > 0 && (
          <div className="welcome-spare">
            {spare.map((one) => (
              <button
                key={one}
                type="button"
                onClick={() => void change({ shortcut: one }).finally(() => recheck(one))}
              >
                {asKeys(one, mac)}
              </button>
            ))}
          </div>
        )}
        <p className="welcome-aside">{t("welcomeKeysOwn")}</p>
      </Screen>
    );
  }

  return (
    <Screen
      dots={dots}
      left={
        tried ? (
          <Quiet says={t("welcomeTryAgain")} onPress={() => setTried(false)} />
        ) : (
          <Quiet says={skip} onPress={next} />
        )
      }
      right={
        tried ? <Strong says={alone ? t("welcomeDone") : t("welcomeNext")} onPress={next} /> : null
      }
    >
      <Heading over={over} title={t("welcomeKeysTitle")} said={t("welcomeKeysWhy")} />
      <div className="welcome-caps">
        {caps(kept.shortcut, mac).map((one, index) => (
          <span key={one} className="welcome-cap-pair">
            {index > 0 && !mac && <span className="welcome-plus">+</span>}
            <kbd>{one}</kbd>
          </span>
        ))}
      </div>
      {tried ? (
        <Done line={t("welcomeKeysDone")} small={t("welcomeKeysDoneWhy")} />
      ) : (
        <Waiting line={t("welcomeKeysWaiting")} small={t("welcomeKeysWaitingWhy")} />
      )}
    </Screen>
  );
}

function Permission({ dots, next }: { dots: ReactNode; next: () => void }) {
  const [asked, setAsked] = useState(false);
  const [granted, setGranted] = useState(false);

  const look = useCallback(() => {
    invoke<Trust>("trust")
      .then((one) => setGranted(one.pastes))
      .catch(() => {});
  }, []);

  useEffect(() => {
    if (!asked || granted) {
      return;
    }
    const again = setInterval(look, 1_000);
    return () => clearInterval(again);
  }, [asked, granted, look]);

  useEffect(() => {
    if (!granted) {
      return;
    }
    const onward = setTimeout(next, 1_200);
    return () => clearTimeout(onward);
  }, [granted, next]);

  return (
    <Screen
      dots={dots}
      left={<Quiet says={t("welcomeTrustLater")} onPress={next} />}
      right={
        asked ? null : (
          <Strong
            says={t("welcomeTrustAsk")}
            onPress={() => {
              setAsked(true);
              invoke<Trust>("ask_trust")
                .then((one) => setGranted(one.pastes))
                .catch(() => {});
            }}
          />
        )
      }
    >
      <Heading
        over={t("welcomeTrustOver")}
        title={t("welcomeTrustTitle")}
        said={t("welcomeTrustWhy")}
      />
      {granted ? (
        <Done line={t("welcomeTrustDone")} />
      ) : asked ? (
        <Waiting line={t("welcomeTrustWaiting")} small={t("welcomeTrustWaitingWhy")} />
      ) : (
        <ol className="welcome-steps">
          <li>{t("welcomeTrustStepOne")}</li>
          <li>{t("welcomeTrustStepTwo")}</li>
          <li>{t("welcomeTrustStepThree")}</li>
        </ol>
      )}
    </Screen>
  );
}

type Crossed = { added: number };

function Bring({ old, dots, next }: { old: Former | null; dots: ReactNode; next: () => void }) {
  const [busy, setBusy] = useState(false);
  const [said, setSaid] = useState<string | null>(null);
  const [came, setCame] = useState(false);

  const bring = () => {
    setBusy(true);
    invoke<Crossed>("bring_former", { at: Date.now() })
      .then((one) => {
        setSaid(fill("welcomeFormerCame", items(one.added)));
        setCame(true);
      })
      .catch(() => setSaid(t("welcomeFormerFailed")))
      .finally(() => setBusy(false));
  };

  return (
    <Screen
      dots={dots}
      left={came ? null : <Quiet says={t("welcomeFormerNotNow")} onPress={next} />}
      right={
        came ? (
          <Strong says={t("welcomeNext")} onPress={next} />
        ) : (
          <Strong
            says={busy ? t("welcomeFormerBringing") : t("welcomeFormerBring")}
            onPress={bring}
            busy={busy}
          />
        )
      }
    >
      <Heading
        over={t("welcomeFormerOver")}
        title={t("welcomeFormerTitle")}
        said={t("welcomeFormerWhy")}
      />
      {old && (
        <div className="welcome-stats">
          <div>
            <b>{old.items.toLocaleString()}</b>
            <span>{t("welcomeFormerItems")}</span>
          </div>
          <div>
            <b>{old.labelled.toLocaleString()}</b>
            <span>{t("welcomeFormerLabelled")}</span>
          </div>
          <div>
            <b>{old.pictures.toLocaleString()}</b>
            <span>{t("welcomeFormerPictures")}</span>
          </div>
        </div>
      )}
      {said ? (
        came ? (
          <Done line={said} />
        ) : (
          <p className="alarm">{said}</p>
        )
      ) : (
        <p className="welcome-aside">{t("welcomeFormerLater")}</p>
      )}
    </Screen>
  );
}

function News({ versions }: { versions: string[] }) {
  const told = toldFor(versions, inEnglish(), onMac());

  return (
    <main className="welcome">
      <div className="welcome-body">
        <div className="welcome-news-head">
          <span className="welcome-over">{t("welcomeNewsOver")}</span>
          {versions[0] && <span className="welcome-version">{versions[0]}</span>}
        </div>
        <h1>{t("welcomeNewsTitle")}</h1>
        <ul className="welcome-news">
          {told.flatMap((one) =>
            one.told.map((said) => (
              <li key={`${one.version}-${said.title}`}>
                <b>{said.title}</b>: {said.said}
              </li>
            )),
          )}
        </ul>
      </div>
      <footer className="welcome-foot news">
        <button type="button" className="welcome-link" onClick={() => void openUrl(RELEASES)}>
          {t("welcomeNewsAll")}
        </button>
        <Strong says={t("welcomeNewsOk")} onPress={leave} />
      </footer>
    </main>
  );
}

function Screen({
  dots,
  left,
  right,
  children,
}: {
  dots: ReactNode;
  left: ReactNode;
  right: ReactNode;
  children: ReactNode;
}) {
  return (
    <main className="welcome">
      <div className="welcome-body">{children}</div>
      <footer className="welcome-foot">
        <div className="welcome-left">{left}</div>
        {dots}
        <div className="welcome-right">{right}</div>
      </footer>
    </main>
  );
}

function Dots({ all, at }: { all: Step[]; at: number }) {
  return (
    <div
      className="welcome-dots"
      role="img"
      aria-label={fill("welcomeStep", `${at + 1} / ${all.length}`)}
    >
      {all.map((one, index) => (
        <span key={one} className={index === at ? "on" : undefined} />
      ))}
    </div>
  );
}

function Heading({ over, title, said }: { over: string; title: string; said: string }) {
  return (
    <div className="welcome-heading">
      <span className="welcome-over">{over}</span>
      <h1>{title}</h1>
      <p>{said}</p>
    </div>
  );
}

function Waiting({ line, small }: { line: string; small: string }) {
  return (
    <div className="welcome-state" aria-live="polite">
      <span className="welcome-pulse" />
      <div>
        <b>{line}</b>
        <span>{small}</span>
      </div>
    </div>
  );
}

function Done({ line, small }: { line: string; small?: string }) {
  return (
    <div className="welcome-state done" aria-live="polite">
      <svg viewBox="0 0 18 18" aria-hidden="true">
        <path d="M3.5 9.5l3.5 3.5 7.5-8" />
      </svg>
      <div>
        <b>{line}</b>
        {small && <span>{small}</span>}
      </div>
    </div>
  );
}

function Quiet({ says, onPress }: { says: string; onPress: () => void }) {
  return (
    <button type="button" className="welcome-quiet" onClick={onPress}>
      {says}
    </button>
  );
}

function Strong({ says, onPress, busy }: { says: string; onPress: () => void; busy?: boolean }) {
  return (
    <button type="button" className="strong welcome-strong" onClick={onPress} disabled={busy}>
      {says}
    </button>
  );
}

function Tray() {
  return (
    <div className="welcome-tray" aria-hidden="true">
      <div className="welcome-hidden">
        <span />
        <span className="ours">
          <i>C</i>
        </span>
        <span />
        <span />
      </div>
      <div className="welcome-bar">
        <span className="arrow">
          <svg viewBox="0 0 12 12" aria-hidden="true">
            <path d="M2.5 7.5L6 4l3.5 3.5" />
          </svg>
        </span>
        <span className="glyph" />
        <span className="glyph" />
        <span className="clock">
          16:40
          <br />
          02/10
        </span>
      </div>
    </div>
  );
}

function MenuBar() {
  return (
    <div className="welcome-tray" aria-hidden="true">
      <div className="welcome-bar mac">
        <span className="glyph" />
        <span className="ours">
          <i>C</i>
        </span>
        <span className="glyph" />
        <span className="clock">16:40</span>
      </div>
    </div>
  );
}
