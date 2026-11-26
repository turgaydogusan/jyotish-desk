// Argala paneli (ileri modüller spec'i §5): bir referans burç için dört çiftin iki graha listesi
// ve durumu. Referans lagna burcu ya da ortak seçimde sabitlenen burçtur (burç, ev ya da bir
// grahanın burcu). Değerler `compute_jaimini`'den gelir; burada hesap yoktur (C4). Durumlar
// yalnızca metinle ayrılır; renk anlam taşımaz (C9).
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { ArgalaPair, Graha, JaiminiReport, OpenedClient } from "../bindings";
import { signIndex } from "../chart/adapter";
import { useHighlight } from "../highlight/store";
import type { HighlightTarget } from "../highlight/target";
import type { Panel } from "../layout/model";
import { useJaimini } from "../modules/useJaimini";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { useLayoutStore } from "../view/store";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

export const ARGALA_REFERENCES = ["lagna", "selected"] as const;
export type ArgalaReference = (typeof ARGALA_REFERENCES)[number];
const SIGNS = Array.from({ length: 12 }, (_, i) => i);
const DASH = "—";

export function argalaReference(settings: Record<string, unknown>): ArgalaReference {
  return settings.reference === "selected" ? "selected" : "lagna";
}

/**
 * Sabitlenen öğenin burcu: burç kendisi, ev lagna burcundan sayılan burç (lagna yoksa haritanın
 * çizdiği gibi Koç'tan), graha haritadaki burcu. Gösterim eşlemesidir; hesap değildir.
 */
export function selectedSign(pinned: HighlightTarget | null, opened: OpenedClient, lagna: number | null): number | null {
  if (pinned === null) return null;
  if (pinned.type === "sign") return pinned.sign;
  if (pinned.type === "house") return ((lagna ?? 0) + pinned.house - 1) % 12;
  const g = opened.chart.grahas.find((x) => x.graha === pinned.id);
  return g ? signIndex(g.sign) : null;
}

export function ArgalaPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const module = useJaimini(opened);
  const phase = usePanelLoadPhase(module.loading);
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  if (module.data === null) return phase === "skeleton" ? <PanelSkeleton /> : null;
  return <Argala panel={panel} report={module.data} opened={opened} />;
}

function Argala({ panel, report, opened }: { panel: Panel; report: JaiminiReport; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const { pinned } = useHighlight();
  const id = useId();
  const mode = argalaReference(panel.settings);
  const lagna = timeUnknown(opened) ? null : signIndex(opened.chart.lagna.sign);
  const reference = mode === "lagna" ? lagna : selectedSign(pinned, opened, lagna);
  const row = reference === null ? undefined : report.argala.find((r) => signIndex(r.reference) === reference);

  const grahas = (list: Graha[]) =>
    list.length === 0
      ? DASH
      : list.map((g, i) => (
          <span key={g}>
            {i > 0 && " "}
            <PinButton target={{ type: "graha", id: g }} label={t(`grahaName.${g}`)}>
              {t(`graha.${g}`)}
            </PinButton>
          </span>
        ));
  const pairRow = (p: ArgalaPair) => (
    <tr key={p.argala_house} data-pair={p.argala_house} data-status={p.status}>
      <th scope="row" className="num">
        {p.argala_house}
      </th>
      <td>{grahas(p.argala_grahas)}</td>
      <td className="num">{p.virodha_house}</td>
      <td>{grahas(p.virodha_grahas)}</td>
      <td>{t(`argala.status.${p.status}`)}</td>
    </tr>
  );

  return (
    <div className="module-panel">
      <div className="module-toolbar">
        <label htmlFor={`${id}-reference`}>{t("argala.reference")}</label>
        <select id={`${id}-reference`} value={mode} onChange={(e) => setPanelSettings(panel.id, { reference: e.target.value })}>
          {ARGALA_REFERENCES.map((r) => (
            <option key={r} value={r}>
              {t(`argala.references.${r}`)}
            </option>
          ))}
        </select>
      </div>
      {mode === "selected" && (
        <div className="argala-signs" role="group" aria-label={t("argala.pickLabel")}>
          {SIGNS.map((s) => (
            <PinButton key={s} target={{ type: "sign", sign: s }}>
              {t(`sign.${s}`)}
            </PinButton>
          ))}
        </div>
      )}
      {mode === "lagna" && lagna === null && <p className="panel-notice">{t("argala.noLagna")}</p>}
      {mode === "selected" && reference === null && <p className="panel-message">{t("argala.pickSign")}</p>}
      {row && reference !== null && (
        <>
          <p className="argala-reference">
            {t("argala.referenceLine", { sign: t(`sign.${reference}`) })}
            {reference === lagna && <span className="av-lagna"> {t("chart.lagna")}</span>}
          </p>
          <table className="module-table">
            <caption className="visually-hidden">{t("argala.caption", { sign: t(`sign.${reference}`) })}</caption>
            <thead>
              <tr>
                <th scope="col" className="num">
                  {t("argala.argalaHouse")}
                </th>
                <th scope="col">{t("argala.argalaGrahas")}</th>
                <th scope="col" className="num">
                  {t("argala.virodhaHouse")}
                </th>
                <th scope="col">{t("argala.virodhaGrahas")}</th>
                <th scope="col">{t("argala.statusColumn")}</th>
              </tr>
            </thead>
            <tbody>{row.pairs.map(pairRow)}</tbody>
          </table>
          {row.third_house_malefic_argala && <p className="panel-notice">{t("argala.thirdHouse")}</p>}
        </>
      )}
      <p className="panel-message">{t("argala.note")}</p>
      <p className="panel-message">{t("jaimini.method", { method: report.method })}</p>
    </div>
  );
}
