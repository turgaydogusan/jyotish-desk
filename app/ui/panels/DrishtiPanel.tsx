// Bakışlar paneli (ileri modüller spec'i §4): bakan yedi graha × bakılan dokuz graha matrisi.
// Değerler `compute_drishti`'den gelir ve panel açıkken istenir (K1); burada hesap yoktur.
// "Tam bakış işareti" görünümü burç temelli tam bakışları `●` ile, "virupa" görünümü derece
// temelli bakış değerini sayı olarak gösterir; renk anlam taşımaz. "Rashi drishti" türü
// (ileri modüller spec'i §5) dokuz grahanın burç bakışını `compute_jaimini`'den okur: bir
// burçtaki grahalar o burcun baktığı burçlardaki grahalara bakar; virupa değeri yoktur.
// Ortak vurgu: bir graha vurgulanınca satırı ve sütunu `--select` alır.
import { useId, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { DrishtiTable, Graha, JaiminiReport, OpenedClient } from "../bindings";
import { GRAHA_IDS, type GrahaId } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { sameTarget, type HighlightTarget } from "../highlight/target";
import type { Panel } from "../layout/model";
import { chartInput, inputKey } from "../modules/input";
import { useJaimini } from "../modules/useJaimini";
import { useModule } from "../modules/useModule";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { useLayoutStore } from "../view/store";
import { Approx } from "./Approx";
import "./panels.css";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

/** Bakan grahalar: `DrishtiTable.sphuta` satır sırası (Rust `ASPECTING_GRAHAS`). */
export const ASPECTING_GRAHAS = ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn"] as const satisfies readonly GrahaId[];
export const DRISHTI_KINDS = ["graha", "rashi"] as const;
export type DrishtiKind = (typeof DRISHTI_KINDS)[number];
export const DRISHTI_VALUES = ["full", "virupa"] as const;
export type DrishtiValue = (typeof DRISHTI_VALUES)[number];
export const FULL_MARK = "●";
const DASH = "—";

export function drishtiValue(settings: Record<string, unknown>): DrishtiValue {
  return settings.value === "virupa" ? "virupa" : "full";
}

export function drishtiKind(settings: Record<string, unknown>): DrishtiKind {
  return settings.kind === "rashi" ? "rashi" : "graha";
}

/** Rashi drishtisi olan (bakan, bakılan) çiftleri: bakılan graha hedef burçtadır. */
export function rashiAspectPairs(report: JaiminiReport): Set<string> {
  const pairs = new Set<string>();
  for (const a of report.graha_rashi_drishti) for (const target of a.occupants) pairs.add(`${a.graha}>${target}`);
  return pairs;
}

/** Burç temelli tam bakışı olan (bakan, bakılan) çiftleri: bakılan graha hedef burçtadır. */
export function fullAspectPairs(table: DrishtiTable): Set<string> {
  const pairs = new Set<string>();
  for (const a of table.sign_aspects) for (const target of a.occupants) pairs.add(`${a.graha}>${target}`);
  return pairs;
}

/** Virupa değeri, bir ondalık; sonlu olmayan değer çizgi. */
export function formatVirupa(v: number | null): string {
  return v === null || !Number.isFinite(v) ? DASH : v.toFixed(1);
}

export function DrishtiPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <DrishtiMatrix panel={panel} opened={opened} />;
}

function DrishtiMatrix({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const highlight = useHighlight();
  const id = useId();
  const kind = drishtiKind(panel.settings);
  const value = drishtiValue(panel.settings);
  // Yalnızca seçili türün komutu çağrılır; diğerinin anahtarı boştur.
  const graha = useModule(kind === "graha" ? inputKey(opened) : null, () => api.computeDrishti(chartInput(opened)));
  const rashi = useJaimini(opened, kind === "rashi");
  const module = kind === "graha" ? graha : rashi;
  const phase = usePanelLoadPhase(module.loading);

  const unknown = timeUnknown(opened);
  const active = highlight.active;
  const on = (g: Graha) => active?.type === "graha" && active.id === g;
  const header = (g: GrahaId, short: boolean) => {
    const target: HighlightTarget = { type: "graha", id: g };
    return (
      <Approx on={unknown && g === "Moon"} label={t("table.approx")}>
        <button
          type="button"
          className="link-button"
          aria-label={short ? t(`grahaName.${g}`) : undefined}
          aria-pressed={sameTarget(highlight.pinned, target)}
          onClick={() => highlight.togglePin(target)}
          onMouseEnter={() => highlight.hover(target)}
          onMouseLeave={() => highlight.hover(null)}
          onFocus={() => highlight.hover(target)}
          onBlur={() => highlight.hover(null)}
        >
          {short ? t(`graha.${g}`) : t(`grahaName.${g}`)}
        </button>
      </Approx>
    );
  };
  const fullMark = (label: string) => (
    <>
      <span aria-hidden="true">{FULL_MARK}</span>
      <span className="visually-hidden">{label}</span>
    </>
  );
  /** Matris: `rows` bakan grahalar, sütunlar dokuz graha; `cell` hücrenin içeriğini verir. */
  const matrix = (caption: string, rows: readonly GrahaId[], cell: (from: GrahaId, i: number, to: GrahaId, j: number) => ReactNode) => (
    <table className="module-table drishti-table">
      <caption className="visually-hidden">{caption}</caption>
      <thead>
        <tr>
          <th scope="col">{t("drishti.aspecting")}</th>
          {GRAHA_IDS.map((g) => (
            <th key={g} scope="col" data-graha={g} className={on(g) ? "num is-highlighted" : "num"}>
              {header(g, true)}
            </th>
          ))}
        </tr>
      </thead>
      <tbody>
        {rows.map((from, i) => (
          <tr key={from} data-graha={from} className={on(from) ? "is-highlighted" : undefined}>
            <th scope="row">{header(from, false)}</th>
            {GRAHA_IDS.map((to, j) => (
              <td key={to} className={on(to) ? "num is-highlighted" : "num"}>
                {from === to ? DASH : cell(from, i, to, j)}
              </td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );

  const grahaBody = (table: DrishtiTable) => {
    const full = fullAspectPairs(table);
    return (
      <>
        {matrix(t("drishti.caption"), ASPECTING_GRAHAS, (from, i, to, j) =>
          value === "virupa" ? formatVirupa(table.sphuta[i][j]) : full.has(`${from}>${to}`) ? fullMark(t("drishti.full")) : "",
        )}
        <p className="panel-message">{value === "full" ? t("drishti.legendFull", { mark: FULL_MARK }) : t("drishti.legendVirupa")}</p>
        <p className="panel-message">{t("drishti.method", { method: table.method })}</p>
      </>
    );
  };
  const rashiBody = (report: JaiminiReport) => {
    const pairs = rashiAspectPairs(report);
    return (
      <>
        {matrix(t("drishti.captionRashi"), GRAHA_IDS, (from, _i, to) => (pairs.has(`${from}>${to}`) ? fullMark(t("drishti.rashiFull")) : ""))}
        <p className="panel-message">{t("drishti.legendRashi", { mark: FULL_MARK })}</p>
        <p className="panel-message">{t("drishti.method", { method: report.method })}</p>
      </>
    );
  };

  return (
    <div className="module-panel">
      <div className="module-toolbar">
        <label htmlFor={`${id}-kind`}>{t("drishti.kind")}</label>
        <select id={`${id}-kind`} value={kind} onChange={(e) => setPanelSettings(panel.id, { kind: e.target.value })}>
          {DRISHTI_KINDS.map((k) => (
            <option key={k} value={k}>
              {t(`drishti.kinds.${k}`)}
            </option>
          ))}
        </select>
        {kind === "graha" && (
          <>
            <label htmlFor={`${id}-value`}>{t("drishti.value")}</label>
            <select id={`${id}-value`} value={value} onChange={(e) => setPanelSettings(panel.id, { value: e.target.value })}>
              {DRISHTI_VALUES.map((v) => (
                <option key={v} value={v}>
                  {t(`drishti.values.${v}`)}
                </option>
              ))}
            </select>
          </>
        )}
      </div>
      {module.error ? (
        <PanelError error={module.error} onRetry={module.retry} />
      ) : module.data === null ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : (
        <>
          {unknown && <p className="panel-notice">{t("drishti.timeUnknown")}</p>}
          {kind === "graha" ? graha.data && grahaBody(graha.data) : rashi.data && rashiBody(rashi.data)}
        </>
      )}
    </div>
  );
}
