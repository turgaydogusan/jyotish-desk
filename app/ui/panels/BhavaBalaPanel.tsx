// Bhava bala paneli (ileri modüller spec'i §7): on iki bhavanın lordu, üç kalemi (bhavadhipati,
// dig, drishti), toplamı ve sırası. Değerler Shadbala paneliyle aynı rapordan
// (`compute_shadbala`) gelir; burada hesap yoktur (C4). Bhavalar eşittir ve lagna burcundan
// sayılan rasi evleriyle aynı burçtadır; ev sistemi ayarından bağımsızdır.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { OpenedClient, ShadbalaReport } from "../bindings";
import { signIndex } from "../chart/adapter";
import { useHighlight } from "../highlight/store";
import type { Panel } from "../layout/model";
import { useShadbala } from "../modules/useShadbala";
import { useSession } from "../session/store";
import { BalaScroll, BalaUnavailable, balaUnit, formatBala, UnitSelect, type BalaUnit } from "./bala";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

export function BhavaBalaPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <BhavaBala panel={panel} opened={opened} />;
}

function BhavaBala({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const module = useShadbala(opened);
  const phase = usePanelLoadPhase(module.loading);
  return (
    <div className="module-panel">
      {/* Araç çubuğu yükleme, hata ve "rapor yok" durumlarında da kullanılabilir. */}
      <div className="module-toolbar">
        <UnitSelect panel={panel} />
      </div>
      {module.error ? (
        <PanelError error={module.error} onRetry={module.retry} />
      ) : module.loading ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : module.data === null ? (
        <BalaUnavailable opened={opened} />
      ) : (
        <Bhavas report={module.data} unit={balaUnit(panel.settings)} opened={opened} />
      )}
    </div>
  );
}

function Bhavas({ report, unit, opened }: { report: ShadbalaReport; unit: BalaUnit; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const captionId = useId();
  const lagna = signIndex(opened.chart.lagna.sign);
  // Burç vurgusu D1 evidir: lagna burcundan sayılan burç (gösterim eşlemesi, hesap değil).
  const activeHouse = active?.type === "house" ? active.house : active?.type === "sign" ? ((active.sign - lagna + 12) % 12) + 1 : null;
  return (
    <>
      <BalaScroll labelledBy={captionId}>
        <table className="module-table">
          <caption id={captionId} className="visually-hidden">{t("bhavaBala.caption")}</caption>
          <thead>
            <tr>
              <th scope="col">{t("bhavaBala.bhava")}</th>
              <th scope="col">{t("bhavaBala.lord")}</th>
              {(["lordBala", "dig", "drishti"] as const).map((k) => (
                <th key={k} scope="col" className="num">
                  {t(`bhavaBala.${k}`)}
                </th>
              ))}
              <th scope="col" className="num">
                {t("bala.total")}
              </th>
              <th scope="col" className="num">
                {t("bala.rank")}
              </th>
            </tr>
          </thead>
          <tbody>
            {report.bhavas.map((b) => {
              const on = activeHouse === b.house || (active?.type === "graha" && active.id === b.lord);
              return (
                <tr key={b.house} data-house={b.house} className={on ? "is-highlighted" : undefined}>
                  <th scope="row">
                    <PinButton target={{ type: "house", house: b.house }} label={t("bhavaBala.houseLabel", { house: b.house })}>
                      {b.house}
                    </PinButton>
                  </th>
                  <td>
                    <PinButton target={{ type: "graha", id: b.lord }}>{t(`grahaName.${b.lord}`)}</PinButton>
                  </td>
                  <td className="num">{formatBala(b.lord_bala, unit)}</td>
                  <td className="num">{formatBala(b.dig, unit)}</td>
                  <td className="num">{formatBala(b.drishti, unit)}</td>
                  <td className="num">{formatBala(b.total, unit)}</td>
                  <td className="num">{b.rank}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </BalaScroll>
      <p className="panel-message">{t("bhavaBala.note")}</p>
      <p className="panel-message">{t(`bala.legend.${unit}`)}</p>
      <p className="panel-message">{t("bala.method", { method: report.method })}</p>
    </>
  );
}
