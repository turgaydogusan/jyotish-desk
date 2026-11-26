// Chara dasha şeridi (ileri modüller spec'i §6). Dönemler `compute_chara_dasha`'dan (iki
// seviye) ve `chara_dasha_children`'dan (3. seviye), transit anının dönem yolu
// `chara_dasha_at`'tan gelir; arayüz tarih karşılaştırmaz (C4). "Maha" seviyesi iki çevrimin
// dashalarını birlikte gösterir; ikinci çevrim "2." önekiyle ayrılır. Bir döneme tıklamak
// transit anını o dönemin başlangıcına taşır (Vimshottari şeridiyle aynı).
//
// Yeni anın yolu (ve 3. seviyesi) istenirken son çizilen şerit yerinde durur (`aria-busy`):
// şerit boşalmaz ve seçilen düğme odağını yitirmez. İskelet yalnızca gösterilecek eski şerit
// yokken çizilir. Odaktaki düğme şeritten çıkarsa (yeni an başka bir dönemin alt dönemlerinde
// ya da şeridin yerini hata aldı) odak güncel döneme, şerit yoksa panel çerçevesine geçer.
import { useLayoutEffect, useRef, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { OpenedClient, SignDashaPath, SignDashaPeriod } from "../bindings";
import { chartInput, inputKey } from "../modules/input";
import { useModule } from "../modules/useModule";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "../panels/states";
import { MAIN_ID } from "../shell/focus";
import { useTransit } from "../transit/store";
import { sameCharaPath } from "../view/model";
import { charaChildrenOf, charaDashas, charaKey, charaName, useCharaChildren, useCharaDasha } from "./chara";
import { PARENT_DEPTH, type StripLevel } from "./system";
import { formatJdDate } from "./tree";

/** Son çizilen şerit: hangi girdi ve seviye için, hangi dönemlerle. */
interface Shown {
  key: string;
  level: StripLevel;
  now: SignDashaPath;
  periods: SignDashaPeriod[];
}

export function CharaDashaStrip({ opened, level }: { opened: OpenedClient; level: StripLevel }) {
  const { t } = useTranslation();
  const transit = useTransit();
  const module = useCharaDasha(opened);
  const dasha = module.data;
  const key = inputKey(opened);
  const jd = transit.view?.snapshot.jd_ut ?? null;
  // Transit anının dönem yolu; dasha yoksa (saat bilinmiyor) ya da an henüz gelmediyse istenmez.
  const at = useModule(dasha !== null && jd !== null ? `${key}|at|${jd}` : null, () => api.charaDashaAt(chartInput(opened), jd as number));
  const path = at.data;
  const depth = PARENT_DEPTH[level];
  // 3. seviye istenerek yüklenir: anın alt döneminin (yolun ilk iki burcu) alt dönemleri.
  const lazy = useCharaChildren(opened, path?.cycle ?? 1, path !== null && depth === 2 ? path.signs.slice(0, 2) : null);
  const pending = module.loading || transit.view === null || at.loading || lazy.loading;
  const phase = usePanelLoadPhase(module.loading || at.loading || lazy.loading || (transit.loading && transit.view === null));

  const shown = useRef<Shown | null>(null);
  const list = useRef<HTMLOListElement>(null);
  // Odaktayken DOM'dan çıkan dönem düğmesinin panel çerçevesi (bkz. `StripButton`).
  const orphaned = useRef<HTMLElement | null>(null);
  useLayoutEffect(() => {
    const frame = orphaned.current;
    orphaned.current = null;
    if (frame === null || (document.activeElement !== null && document.activeElement !== document.body)) return;
    const target = list.current?.querySelector<HTMLElement>('[aria-current="true"]') ?? list.current?.querySelector<HTMLElement>("button") ?? frame;
    target.focus({ preventScroll: true });
  });

  let next: Shown | null = null;
  let body: ReactNode;
  const failed = transit.error ? transit : module.error ? module : at.error ? at : lazy.error ? lazy : null;
  if (failed?.error) body = <PanelError error={failed.error} onRetry={failed.retry} />;
  else if (pending) {
    // Yanıt beklenirken aynı girdinin ve seviyenin son şeridi durur; yoksa iskelet.
    if (shown.current?.key === key && shown.current.level === level) next = shown.current;
    else body = phase === "skeleton" ? <PanelSkeleton /> : null;
  } else if (dasha === null) body = <p className="panel-notice">{t("dasha.chara.timeUnknown")}</p>;
  else if (path === null) body = <p className="panel-message">{t("dasha.chara.outside")}</p>;
  else {
    const parent = path.signs.slice(0, depth);
    const periods: SignDashaPeriod[] =
      depth === 0 ? charaDashas(dasha.periods) : depth === 1 ? charaChildrenOf(dasha.periods, path.cycle, parent) : charaChildrenOf(lazy.data ?? [], path.cycle, parent);
    // Boş yanıt sessizce boş şerit olarak gösterilmez: her dönemin on iki alt dönemi vardır.
    if (periods.length === 0) body = <PanelError error={{ code: "dasha_empty", params: {} }} onRetry={depth === 2 ? lazy.retry : module.retry} />;
    else next = { key, level, now: { cycle: path.cycle, signs: path.signs.slice(0, depth + 1) }, periods };
  }
  shown.current = next;
  if (next === null) return body;

  const offset = opened.chart_record.resolved.offset_secs;
  const { now, periods } = next;
  return (
    <ol ref={list} className="dasha-strip" aria-label={t("dasha.strip.label", { level: t(`dasha.strip.levels.${level}`) })} aria-busy={pending ? "true" : undefined}>
      {periods.map((p) => {
        const current = sameCharaPath({ cycle: p.cycle, signs: p.signs }, now);
        return (
          <li key={charaKey(p.cycle, p.signs)}>
            <StripButton period={p} current={current} orphaned={orphaned} onChoose={() => transit.setAt({ Jd: p.start_jd_ut })}>
              <span className="dasha-strip-name">{charaName(t, p.cycle, p.signs)}</span>
              <span className="dasha-strip-date">{formatJdDate(p.start_jd_ut, offset)}</span>
              {current && <span className="dasha-strip-now">{t("dasha.strip.current")}</span>}
            </StripButton>
          </li>
        );
      })}
    </ol>
  );
}

/**
 * Dönem düğmesi. Odaktayken DOM'dan çıkarsa odak gövdeye düşer; layout effect'in temizliği
 * düğüm çıkmadan önce koşar ve panel çerçevesini `orphaned`'a yazar, şerit de yeni çizimden
 * sonra odağı taşır (`PanelError`'daki "Yeniden dene" ile aynı yöntem).
 */
function StripButton({
  period,
  current,
  orphaned,
  onChoose,
  children,
}: {
  period: SignDashaPeriod;
  current: boolean;
  orphaned: { current: HTMLElement | null };
  onChoose(): void;
  children: ReactNode;
}) {
  const button = useRef<HTMLButtonElement>(null);
  useLayoutEffect(() => {
    const el = button.current;
    return () => {
      if (el === null || document.activeElement !== el) return;
      orphaned.current = el.closest<HTMLElement>(".panel") ?? document.getElementById(MAIN_ID);
    };
  }, [orphaned]);
  return (
    <button
      ref={button}
      type="button"
      className={`dasha-strip-item${current ? " dasha-strip-item--current" : ""}`}
      data-sign={period.signs[period.signs.length - 1]}
      data-cycle={period.cycle}
      aria-current={current ? "true" : undefined}
      onClick={onChoose}
    >
      {children}
    </button>
  );
}
