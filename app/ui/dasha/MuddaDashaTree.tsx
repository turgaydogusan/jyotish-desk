// Mudda dasa ağacı (yol haritası §4.14): seçilen yılın dokuz dönemi ve alt dönemleri. Dönemler,
// ilk lord ve bölümden artan `compute_varsha`'dan gelir ve panel açıkken istenir (K1); burada
// hesap yoktur (C4). Yıl, Varshaphala paneliyle ortak depodadır (`varsha/yearStore`): iki panel
// aynı yılı gösterir. Dönemler yere bağlı değildir; istek doğum yeriyle yapılır ve anlar doğum
// yerinin yıl başındaki ofsetiyle yazılır. Açık dönem panelin kendi durumudur (kaydedilmez).
// Kural bellekten aktarılmıştır: sahibin onayına dek "doğrulanmamış tablo" notu durur (Y4).
import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { DashaPeriod, Graha, OpenedClient, VarshaView } from "../bindings";
import { inputKey } from "../modules/input";
import { chartPlace } from "../modules/muhurta";
import { useLastData } from "../modules/useModule";
import { useVarsha } from "../modules/useVarsha";
import { formatJdMinute } from "../panels/format";
import { nakshatraName } from "../panels/labels";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "../panels/states";
import "../panels/varsha.css";
import { VarshaYearControls } from "../varsha/YearControls";
import { useVarshaAge } from "../varsha/yearStore";
import { zoneLabel } from "../varsha/zone";
import { PeriodTree, type PeriodLevel } from "./PeriodTree";
import { childrenOf } from "./tree";

export function MuddaDashaTree({ opened }: { opened: OpenedClient }) {
  const { age: selected, setAge } = useVarshaAge(opened.chart_record.id);
  const module = useVarsha(opened, selected, chartPlace(opened));
  // Yıl değişirken önceki yılın ağacı yerinde durur; başka harita eskiyi unutturur.
  const view = useLastData(module, inputKey(opened));
  const phase = usePanelLoadPhase(module.loading);
  return (
    <>
      <div className="module-toolbar">
        <VarshaYearControls age={selected ?? view?.report.age ?? null} current={selected === null} onChange={setAge} />
      </div>
      {module.error && <PanelError error={module.error} onRetry={module.retry} />}
      {module.error === null && view === null && phase === "skeleton" && <PanelSkeleton />}
      {module.error === null && view !== null && <Tree key={view.report.age} view={view} />}
    </>
  );
}

function Tree({ view }: { view: VarshaView }) {
  const { t } = useTranslation();
  const { report, offset_secs: offset } = view;
  const { mudda } = report;
  // Tek açık yol: bir dönemi açmak ötekini kapatır.
  const [open, setOpen] = useState<Graha | null>(null);
  const at = (jd: number | null) => formatJdMinute(jd, offset);
  const dates = (p: DashaPeriod) => `${at(p.start_jd_ut)} – ${at(p.end_jd_ut)}`;
  const name = (g: Graha) => t(`grahaName.${g}`);

  const level = (path: readonly Graha[]): PeriodLevel => ({
    status: "ok",
    nodes: childrenOf(mudda.periods, path).map((p) => {
      const lord = p.lords[p.lords.length - 1];
      const isOpen = p.has_children && open === lord;
      return {
        key: p.lords.join(","),
        level: p.level,
        name: name(lord),
        label: `${p.lords.map(name).join("–")}, ${dates(p)}`,
        dates: dates(p),
        expandable: p.has_children,
        open: isOpen,
        kids: isOpen ? level(p.lords) : undefined,
        toggle: () => setOpen(isOpen ? null : lord),
      };
    }),
  });

  return (
    <>
      <p className="panel-notice" role="note">
        {t("varsha.mudda.unverified")}
      </p>
      {!report.time_known && (
        <p className="panel-notice" role="note">
          {t("varsha.mudda.approx")}
        </p>
      )}
      <p className="varsha-summary">
        <span>{t("varsha.mudda.summary", { number: report.age + 1, age: report.age, start: at(report.start_jd_ut), end: at(report.end_jd_ut), zone: zoneLabel(offset) })}</span>
        <span>{t("varsha.mudda.first", { lord: name(mudda.first_lord), nakshatra: mudda.birth_nakshatra, name: nakshatraName(mudda.birth_nakshatra), age: mudda.age, remainder: mudda.remainder })}</span>
      </p>
      <PeriodTree label={t("varsha.mudda.system")} roots={level([])} />
      <p className="panel-message">{t("varsha.method", { method: report.method })}</p>
    </>
  );
}
