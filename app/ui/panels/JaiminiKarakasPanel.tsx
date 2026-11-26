// Jaimini karaka tablosu (ileri modüller spec'i §5): rol, graha, burç içi derece; altında
// karakamsha burcu ve grahaların karakamshadan ev sayıları (D1 ve D9). Değerler
// `compute_jaimini`'den gelir; gerçek derece açık haritadandır. Burada hesap yoktur (C4).
// Eşitlik `=` ve metinle, yaklaşık değer `≈` ve metinle gösterilir; renk anlam taşımaz (C9).
import { useTranslation } from "react-i18next";
import type { JaiminiReport, OpenedClient } from "../bindings";
import { signIndex } from "../chart/adapter";
import { GRAHA_IDS } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { useJaimini } from "../modules/useJaimini";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { Approx } from "./Approx";
import { formatDegree } from "./format";
import { KARAKA_ROLES } from "./labels";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

export const TIE_MARK = "=";
const DASH = "—";

/** Derece metni; sonlu olmayan değer çizgi. */
export function degreeText(deg: number | null): string {
  return deg === null || !Number.isFinite(deg) ? DASH : formatDegree(deg);
}

export function JaiminiKarakasPanel(_: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const module = useJaimini(opened);
  const phase = usePanelLoadPhase(module.loading);
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  if (module.data === null) return phase === "skeleton" ? <PanelSkeleton /> : null;
  return <Karakas report={module.data} opened={opened} />;
}

function Karakas({ report, opened }: { report: JaiminiReport; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const unknown = timeUnknown(opened);
  const positions = new Map(opened.chart.grahas.map((g) => [g.graha, g]));
  const grahaOn = (g: string) => active?.type === "graha" && active.id === g;
  const km = report.karakamsha;
  const kmSign = signIndex(km.sign);
  const anyTie = report.karakas.some((k) => k.tie);
  const hasRahu = report.karakas.some((k) => k.graha === "Rahu");

  return (
    <div className="module-panel">
      {unknown && <p className="panel-notice">{t("jaimini.timeUnknown")}</p>}
      {report.warnings.map((w) => (
        <p key={w} className="panel-notice">
          <strong>{t(`badge.${w}`)}</strong>: {t(`badgeHint.${w}`)}
        </p>
      ))}
      <table className="module-table">
        <caption className="visually-hidden">{t("jaimini.karakaCaption")}</caption>
        <thead>
          <tr>
            <th scope="col">{t("jaimini.role")}</th>
            <th scope="col">{t("table.graha")}</th>
            <th scope="col">{t("table.sign")}</th>
            <th scope="col" className="num">
              {t("table.degree")}
            </th>
          </tr>
        </thead>
        <tbody>
          {report.karakas.map((k) => {
            const role = KARAKA_ROLES[k.role];
            const g = positions.get(k.graha);
            const real = degreeText(g?.deg_in_sign ?? null);
            return (
              <tr key={k.role} data-role={k.role} data-graha={k.graha} className={grahaOn(k.graha) ? "is-highlighted" : undefined}>
                <th scope="row">
                  <abbr title={role.name}>{role.short}</abbr>
                  <span className="visually-hidden"> {role.name}</span>
                </th>
                <td>
                  <PinButton target={{ type: "graha", id: k.graha }}>{t(`grahaName.${k.graha}`)}</PinButton>
                </td>
                <td>{g ? t(`sign.${signIndex(g.sign)}`) : DASH}</td>
                <td className="num">
                  <Approx on={unknown && k.graha === "Moon"} label={t("table.approx")}>
                    {real}
                  </Approx>
                  {k.graha === "Rahu" && <> ({t("jaimini.effective", { deg: degreeText(k.effective_deg) })})</>}
                  {k.tie && (
                    <>
                      {" "}
                      <span aria-hidden="true">{TIE_MARK}</span>
                      <span className="visually-hidden">{t("jaimini.tie")}</span>
                    </>
                  )}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      {anyTie && <p className="panel-message">{t("jaimini.tieLegend", { mark: TIE_MARK })}</p>}
      {hasRahu && <p className="panel-message">{t("jaimini.rahuLegend")}</p>}
      <p className="jaimini-karakamsha">
        {t("jaimini.karakamshaLabel")}{" "}
        <Approx on={unknown} label={t("table.approx")}>
          <PinButton target={{ type: "sign", sign: kmSign }}>{t(`sign.${kmSign}`)}</PinButton>
        </Approx>{" "}
        <span className="panel-message">{t("jaimini.karakamshaOf", { graha: t(`grahaName.${km.atmakaraka}`) })}</span>
      </p>
      <table className="module-table">
        <caption className="visually-hidden">{t("jaimini.housesCaption")}</caption>
        <thead>
          <tr>
            <th scope="col">{t("table.graha")}</th>
            <th scope="col" className="num">
              {t("jaimini.d1")}
            </th>
            <th scope="col" className="num">
              {t("jaimini.d9")}
            </th>
          </tr>
        </thead>
        <tbody>
          {GRAHA_IDS.map((g, i) => (
            <tr key={g} data-house-of={g} className={grahaOn(g) ? "is-highlighted" : undefined}>
              <th scope="row">
                <PinButton target={{ type: "graha", id: g }}>{t(`grahaName.${g}`)}</PinButton>
              </th>
              <td className="num">{km.houses_in_d1[i]}</td>
              <td className="num">{km.houses_in_d9[i]}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <p className="panel-message">{t("jaimini.scheme", { scheme: t(`settings.karakaSchemes.${report.scheme}`) })}</p>
      <p className="panel-message">{t("jaimini.method", { method: report.method })}</p>
    </div>
  );
}
