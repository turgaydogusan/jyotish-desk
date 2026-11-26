// Bakışlar paneli (ileri modüller spec'i §4): bakan yedi graha × bakılan dokuz graha matrisi.
// Değerler `compute_drishti`'den gelir ve panel açıkken istenir (K1); burada hesap yoktur.
// "Tam bakış işareti" görünümü burç temelli tam bakışları `●` ile, "virupa" görünümü derece
// temelli bakış değerini sayı olarak gösterir; renk anlam taşımaz. Rashi drishti türü Jaimini
// alt projesiyle gelir. Ortak vurgu: bir graha vurgulanınca satırı ve sütunu `--select` alır.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { DrishtiTable, Graha, OpenedClient } from "../bindings";
import { GRAHA_IDS, type GrahaId } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { sameTarget, type HighlightTarget } from "../highlight/target";
import type { Panel } from "../layout/model";
import { chartInput, inputKey } from "../modules/input";
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
export const DRISHTI_VALUES = ["full", "virupa"] as const;
export type DrishtiValue = (typeof DRISHTI_VALUES)[number];
export const FULL_MARK = "●";
const DASH = "—";

export function drishtiValue(settings: Record<string, unknown>): DrishtiValue {
  return settings.value === "virupa" ? "virupa" : "full";
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
  const value = drishtiValue(panel.settings);
  const module = useModule(inputKey(opened), () => api.computeDrishti(chartInput(opened)));
  const phase = usePanelLoadPhase(module.loading);

  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  const table = module.data;
  if (table === null) return phase === "skeleton" ? <PanelSkeleton /> : null;

  const unknown = timeUnknown(opened);
  const full = fullAspectPairs(table);
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

  return (
    <div className="module-panel">
      <div className="module-toolbar">
        <label htmlFor={`${id}-value`}>{t("drishti.value")}</label>
        <select id={`${id}-value`} value={value} onChange={(e) => setPanelSettings(panel.id, { value: e.target.value })}>
          {DRISHTI_VALUES.map((v) => (
            <option key={v} value={v}>
              {t(`drishti.values.${v}`)}
            </option>
          ))}
        </select>
      </div>
      {unknown && <p className="panel-notice">{t("drishti.timeUnknown")}</p>}
      <table className="module-table drishti-table">
        <caption className="visually-hidden">{t("drishti.caption")}</caption>
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
          {ASPECTING_GRAHAS.map((from, i) => (
            <tr key={from} data-graha={from} className={on(from) ? "is-highlighted" : undefined}>
              <th scope="row">{header(from, false)}</th>
              {GRAHA_IDS.map((to, j) => {
                const self = from === to;
                const isFull = full.has(`${from}>${to}`);
                return (
                  <td key={to} className={on(to) ? "num is-highlighted" : "num"}>
                    {self ? (
                      DASH
                    ) : value === "virupa" ? (
                      formatVirupa(table.sphuta[i][j])
                    ) : isFull ? (
                      <>
                        <span aria-hidden="true">{FULL_MARK}</span>
                        <span className="visually-hidden">{t("drishti.full")}</span>
                      </>
                    ) : (
                      ""
                    )}
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
      <p className="panel-message">{value === "full" ? t("drishti.legendFull", { mark: FULL_MARK }) : t("drishti.legendVirupa")}</p>
      <p className="panel-message">{t("drishti.method", { method: table.method })}</p>
    </div>
  );
}
