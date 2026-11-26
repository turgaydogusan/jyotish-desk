// Chara dasha ağacı (ileri modüller spec'i §6): dashalar burçtur. Yön, yıllar ve iki seviye
// dönem `compute_chara_dasha`'dan gelir ve panel açıkken istenir (K1); 3. seviye
// `chara_dasha_children` ile istenerek yüklenir. Burada hesap yoktur (C4). Açık yol
// `ViewState.chara_expanded_path`'tir (çevrim ve burç yolu). İkinci çevrimin satırları "2."
// önekiyle ayrılır. Doğum saati bilinmiyorsa komut `null` döner ve ağaç yerine bir satır
// açıklama çıkar: sıra lagnaya bağlıdır.
import { useTranslation } from "react-i18next";
import type { CharaDasha, OpenedClient, Sign, SignDashaPeriod } from "../bindings";
import { signIndex } from "../chart/adapter";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "../panels/states";
import { useViewStore } from "../view/store";
import {
  CHARA_PRELOADED_LEVEL,
  charaChildrenOf,
  charaDashas,
  charaKey,
  charaName,
  charaYears,
  isCharaOpen,
  MAX_CHARA_LEVEL,
  useCharaChildren,
  useCharaDasha,
} from "./chara";
import { PeriodTree, type PeriodLevel } from "./PeriodTree";
import { formatJdDate } from "./tree";

export function CharaDashaTree({ opened }: { opened: OpenedClient }) {
  const { t } = useTranslation();
  const module = useCharaDasha(opened);
  const { model } = useViewStore();
  const open = model.charaPath;
  // Tek açık yol: istenerek yüklenen tek seviye, açık yolun ikinci düğümünün alt dönemleridir.
  // Yol bu haritada yoksa (ör. o burç ikinci çevrimde atlanıyor) istek yapılmaz.
  const wanted = open.signs.slice(0, CHARA_PRELOADED_LEVEL);
  const known = (module.data?.periods ?? []).some((p) => p.cycle === open.cycle && p.level === CHARA_PRELOADED_LEVEL && wanted.every((s, i) => p.signs[i] === s));
  const lazyPath = wanted.length === CHARA_PRELOADED_LEVEL && known ? wanted : null;
  const lazy = useCharaChildren(opened, open.cycle, lazyPath);
  const phase = usePanelLoadPhase(module.loading || lazy.loading);
  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  if (module.loading) return phase === "skeleton" ? <PanelSkeleton /> : null;
  if (module.data === null) return <p className="panel-notice">{t("dasha.chara.timeUnknown")}</p>;
  return <Tree dasha={module.data} opened={opened} lazy={lazy} />;
}

function Tree({ dasha, opened, lazy }: { dasha: CharaDasha; opened: OpenedClient; lazy: ReturnType<typeof useCharaChildren> }) {
  const { t } = useTranslation();
  const { model, setCharaPath } = useViewStore();
  const open = model.charaPath;
  const offset = opened.chart_record.resolved.offset_secs;
  const dates = (p: SignDashaPeriod) => `${formatJdDate(p.start_jd_ut, offset)} – ${formatJdDate(p.end_jd_ut, offset)}`;
  const toggle = (p: SignDashaPeriod) => setCharaPath({ cycle: p.cycle, signs: isCharaOpen(open, p.cycle, p.signs) ? p.signs.slice(0, -1) : p.signs });

  const nodes = (periods: readonly SignDashaPeriod[]): PeriodLevel => ({
    status: "ok",
    nodes: periods.map((p) => {
      const expandable = p.has_children && p.level < MAX_CHARA_LEVEL;
      const o = expandable && isCharaOpen(open, p.cycle, p.signs);
      return {
        key: charaKey(p.cycle, p.signs),
        level: p.level,
        name: charaName(t, p.level === 1 ? p.cycle : 1, p.signs.slice(-1)),
        label: `${charaName(t, p.cycle, p.signs)}, ${dates(p)}`,
        dates: dates(p),
        expandable,
        open: o,
        kids: o ? level(p.cycle, p.signs) : undefined,
        toggle: () => toggle(p),
      };
    }),
  });

  // Yüklenen 3. seviye de süzgeçten geçer: yalnızca istenen yolun bir alt seviyesi kabul
  // edilir. Boş yanıt hatadır: her dönemin on iki alt dönemi vardır.
  const level = (cycle: number, path: readonly Sign[]): PeriodLevel => {
    if (path.length < CHARA_PRELOADED_LEVEL) return nodes(charaChildrenOf(dasha.periods, cycle, path));
    if (lazy.error) return { status: "error", error: lazy.error, retry: lazy.retry };
    if (lazy.data === null) return { status: "loading" };
    const kids = charaChildrenOf(lazy.data, cycle, path);
    return kids.length > 0 ? nodes(kids) : { status: "error", error: { code: "dasha_empty", params: {} }, retry: lazy.retry };
  };

  const years = charaYears(dasha)
    .map(({ sign, years: y }) => `${t(`sign.${signIndex(sign)}`)} ${y}`)
    .join(" · ");
  return (
    <>
      <p className="dasha-chara-summary">
        <span>{t("dasha.chara.direction", { direction: t(dasha.forward ? "dasha.chara.forward" : "dasha.chara.backward") })}</span>
        <span>{t("dasha.chara.years", { list: years })}</span>
      </p>
      <PeriodTree label={t("panel.dasha_tree")} roots={nodes(charaDashas(dasha.periods))} />
      <p className="panel-message">{t("dasha.chara.method", { method: dasha.method })}</p>
    </>
  );
}
