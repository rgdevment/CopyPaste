import type { CSSProperties } from "react";
import {
  type Choices,
  DENSITIES,
  type Density,
  type Kept,
  type Look,
  PANEL_SIZES,
  PLAIN_LOOK,
  SHUT_LINES,
  SIZES,
  type TextSize,
  ZOOM,
} from "../core";
import { t } from "../locales";
import { Band, Knob, Line } from "./Bits";
import FontPicker from "./FontPicker";

const SIZE_SAYS = {
  small: "sizeSmall",
  normal: "sizeNormal",
  large: "sizeLarge",
  larger: "sizeLarger",
} as const;

const ACCENT_SAYS: Record<string, Parameters<typeof t>[0]> = {
  indigo: "accentIndigo",
  blue: "accentBlue",
  teal: "accentTeal",
  green: "accentGreen",
  amber: "accentAmber",
  rose: "accentRose",
};

export function accentSaid(id: string, first: boolean): string {
  const said = ACCENT_SAYS[id] ? t(ACCENT_SAYS[id]) : id;
  return first ? `${said} · ${t("byDefault")}` : said;
}

const DENSITY_SAYS = {
  compact: "densityCompact",
  normal: "densityNormal",
  comfortable: "densityComfortable",
  list: "densityList",
} as const;

const PANEL_SAYS = {
  normal: "panelNormal",
  large: "panelLarge",
  tall: "panelTall",
} as const;

export function systemSaid(choices: Choices): string {
  return `${t("fontSystem")} (${choices.text.label}) · ${t("byDefault")}`;
}

export default function Appearance({
  kept,
  change,
  choices,
}: {
  kept: Kept;
  change: (what: Partial<Kept>) => Promise<void>;
  choices: Choices | null;
}) {
  const size = kept["text-size"] ?? "normal";
  const density = kept.density ?? "normal";
  const accent = kept.accent ?? "indigo";
  const panelSize = kept["panel-size"] ?? "normal";
  const contrast = kept["high-contrast"] ?? false;

  return (
    <>
      <h1>{t("railLook")}</h1>

      <Preview kept={kept} choices={choices} />

      <Band says={t("bandTheme")} />

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

      <Line says={t("accent")} why={t("accentWhy")}>
        <div className="swatches" role="radiogroup" aria-label={t("accent")}>
          {(choices?.accents ?? []).map((one, at) => (
            <label
              key={one.id}
              className="swatch"
              style={{ "--light": one.light, "--dark": one.dark } as CSSProperties}
            >
              <input
                type="radio"
                name="accent"
                value={one.id}
                checked={accent === one.id}
                tabIndex={accent === one.id ? 0 : -1}
                aria-label={accentSaid(one.id, at === 0)}
                onChange={() => change({ accent: one.id })}
              />
            </label>
          ))}
        </div>
      </Line>

      <Line says={t("contrast")} why={t("contrastWhy")}>
        <Knob
          on={contrast}
          says={t("contrast")}
          onPress={() => void change({ "high-contrast": !contrast })}
        />
      </Line>

      <Band says={t("bandText")} />

      <Line says={t("size")} why={t("sizeWhy")}>
        <Segments
          name="text-size"
          says={t("size")}
          all={SIZES}
          chosen={size}
          label={(one) => t(SIZE_SAYS[one])}
          pick={(one) => change({ "text-size": one })}
        />
      </Line>

      {choices && (
        <Line says={t("font")} why={t("fontWhy")}>
          <FontPicker
            says={t("font")}
            first={systemSaid(choices)}
            others={choices.texts}
            chosen={kept.font ?? null}
            pick={(id) => change({ font: id })}
          />
        </Line>
      )}

      {choices && (
        <Line says={t("codeFont")} why={t("codeFontWhy")}>
          <FontPicker
            says={t("codeFont")}
            first={`${choices.code.label} · ${t("byDefault")}`}
            others={choices.codes}
            chosen={kept["code-font"] ?? null}
            pick={(id) => change({ "code-font": id })}
          />
        </Line>
      )}

      <Band says={t("bandPanel")} />

      <Line says={t("panelSize")} why={t("panelSizeWhy")}>
        <Segments
          name="panel-size"
          says={t("panelSize")}
          all={PANEL_SIZES}
          chosen={panelSize}
          label={(one) => t(PANEL_SAYS[one])}
          pick={(one) => change({ "panel-size": one })}
        />
      </Line>

      <Line says={t("density")} why={t("densityWhy")}>
        <Segments
          name="density"
          says={t("density")}
          all={DENSITIES}
          chosen={density}
          label={(one) => t(DENSITY_SAYS[one])}
          pick={(one) => change({ density: one })}
        />
      </Line>

      <div className="look-reset">
        <button type="button" className="mild" onClick={() => void change(PLAIN_LOOK)}>
          {t("lookReset")}
        </button>
      </div>
    </>
  );
}

function Segments<T extends string>({
  name,
  says,
  all,
  chosen,
  label,
  pick,
}: {
  name: string;
  says: string;
  all: T[];
  chosen: T;
  label: (one: T) => string;
  pick: (one: T) => void;
}) {
  return (
    <div className="segments" role="radiogroup" aria-label={says}>
      {all.map((one) => (
        <label key={one} className={one === chosen ? "segment on" : "segment"}>
          <input
            type="radio"
            name={name}
            value={one}
            checked={one === chosen}
            tabIndex={one === chosen ? 0 : -1}
            onChange={() => pick(one)}
          />
          {label(one)}
        </label>
      ))}
    </div>
  );
}

export function previewStyle(kept: Kept, choices: Choices | null): CSSProperties {
  const size: TextSize = kept["text-size"] ?? "normal";
  const density: Density = kept.density ?? "normal";
  const paint = choices?.accents.find((one) => one.id === (kept.accent ?? "indigo"));
  const font = choices?.texts.find((one) => one.id === kept.font) ?? choices?.text;
  const code = choices?.codes.find((one) => one.id === kept["code-font"]) ?? choices?.code;
  return {
    "--zoom": ZOOM[size],
    "--lines": SHUT_LINES[density],
    "--light": paint?.light,
    "--dark": paint?.dark,
    "--light-selected": paint?.lightSelected,
    "--dark-selected": paint?.darkSelected,
    "--look-font": font?.css,
    "--look-mono": code?.css,
  } as CSSProperties;
}

function Preview({ kept, choices }: { kept: Kept; choices: Choices | null }) {
  const density = kept.density ?? "normal";
  const lines = SHUT_LINES[density];
  const strong = kept["high-contrast"] ?? false;
  return (
    <figure className="preview" aria-label={t("preview")} style={previewStyle(kept, choices)}>
      <div className={strong ? "mock strong" : "mock"}>
        <div className="mock-search">{t("previewSearch")}</div>
        <div className="mock-chips">
          <span className="mock-chip">
            <i className="dot text" />
            {t("previewKind")} <small>37</small>
          </span>
          <span className="mock-chip">
            <i className="dot code" />
            {t("previewCode")} <small>4</small>
          </span>
        </div>
        <div className="mock-day">{t("previewToday")}</div>
        {density === "list" ? <PreviewRows /> : null}
        <div className="mock-card lit" hidden={density === "list"}>
          <div className="mock-head">
            <b className="text">{t("previewKind")}</b>
            <span>· Notas</span>
          </div>
          <p>{t("previewText")}</p>
        </div>
        <div className="mock-card" hidden={density === "list"}>
          <div className="mock-head">
            <b className="code">{t("previewCode")}</b>
            <span>· Terminal</span>
            <em>{t("previewAge")}</em>
          </div>
          <code>{lines > 1 ? 'git commit -m "listo"\ngit push' : 'git commit -m "listo"'}</code>
        </div>
      </div>
    </figure>
  );
}

function PreviewRows() {
  const rows = [
    { kind: "text", said: t("previewText"), age: t("previewAge") },
    { kind: "code", said: 'git commit -m "listo"', age: "4 min" },
    { kind: "text", said: t("previewText"), age: "9 min" },
    { kind: "code", said: "git push", age: "15 min" },
  ];
  return (
    <>
      {rows.map((row, at) => (
        <div key={row.age} className={at === 0 ? "mock-row lit" : "mock-row"}>
          <i className={`dot ${row.kind}`} />
          <span className={row.kind === "code" ? "mono" : undefined}>{row.said}</span>
          <em>{row.age}</em>
        </div>
      ))}
    </>
  );
}
