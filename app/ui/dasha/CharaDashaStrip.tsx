// Chara dasha şeridi (ileri modüller spec'i §6). Dönemler `compute_chara_dasha`'dan (iki
// seviye) ve `chara_dasha_children`'dan (3. seviye), transit anının dönem yolu
// `chara_dasha_at`'tan gelir; arayüz tarih karşılaştırmaz (C4). "Maha" seviyesi iki çevrimin
// dashalarını birlikte gösterir; ikinci çevrim "2." önekiyle ayrılır. Bir döneme tıklamak
// transit anını o dönemin başlangıcına taşır (Vimshottari şeridiyle aynı).
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { OpenedClient, SignDashaPeriod } from "../bindings";
import { chartInput, inputKey } from "../modules/input";
import { useModule } from "../modules/useModule";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "../panels/states";
import { useTransit } from "../transit/store";
import { sameCharaPath } from "../view/model";
import { charaChildrenOf, charaDashas, charaKey, charaName, useCharaChildren, useCharaDasha } from "./chara";
import { PARENT_DEPTH, type StripLevel } from "./system";
import { formatJdDate } from "./tree";

export function CharaDashaStrip({ opened, level }: { opened: OpenedClient; level: StripLevel }) {
  const { t } = useTranslation();
  const transit = useTransit();
  const module = useCharaDasha(opened);
  const dasha = module.data;
  const jd = transit.view?.snapshot.jd_ut ?? null;
  // Transit anının dönem yolu; dasha yoksa (saat bilinmiyor) ya da an henüz gelmediyse istenmez.
  const at = useModule(dasha !== null && jd !== null ? `${inputKey(opened)}|at|${jd}` : null, () => api.charaDashaAt(chartInput(opened), jd as number));
  const path = at.data;
  const depth = PARENT_DEPTH[level];
  // 3. seviye istenerek yüklenir: anın alt döneminin (yolun ilk iki burcu) alt dönemleri.
  const lazy = useCharaChildren(opened, path?.cycle ?? 1, path !== null && depth === 2 ? path.signs.slice(0, 2) : null);
  const loading = module.loading || at.loading || lazy.loading || (transit.loading && transit.view === null);
  const phase = usePanelLoadPhase(loading);

  if (transit.error) return <PanelError error={transit.error} onRetry={transit.retry} />;
  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  if (at.error) return <PanelError error={at.error} onRetry={at.retry} />;
  if (lazy.error) return <PanelError error={lazy.error} onRetry={lazy.retry} />;
  if (module.loading || transit.view === null || at.loading || lazy.loading) return phase === "skeleton" ? <PanelSkeleton /> : null;
  if (dasha === null) return <p className="panel-notice">{t("dasha.chara.timeUnknown")}</p>;
  if (path === null) return <p className="panel-message">{t("dasha.chara.outside")}</p>;

  const parent = path.signs.slice(0, depth);
  const periods: SignDashaPeriod[] =
    depth === 0 ? charaDashas(dasha.periods) : depth === 1 ? charaChildrenOf(dasha.periods, path.cycle, parent) : charaChildrenOf(lazy.data ?? [], path.cycle, parent);
  // Boş yanıt sessizce boş şerit olarak gösterilmez: her dönemin on iki alt dönemi vardır.
  if (periods.length === 0) return <PanelError error={{ code: "dasha_empty", params: {} }} onRetry={depth === 2 ? lazy.retry : module.retry} />;
  const offset = opened.chart_record.resolved.offset_secs;
  const now = { cycle: path.cycle, signs: path.signs.slice(0, depth + 1) };
  return (
    <ol className="dasha-strip" aria-label={t("dasha.strip.label", { level: t(`dasha.strip.levels.${level}`) })}>
      {periods.map((p) => {
        const current = sameCharaPath({ cycle: p.cycle, signs: p.signs }, now);
        return (
          <li key={charaKey(p.cycle, p.signs)}>
            <button
              type="button"
              className={`dasha-strip-item${current ? " dasha-strip-item--current" : ""}`}
              data-sign={p.signs[p.signs.length - 1]}
              data-cycle={p.cycle}
              aria-current={current ? "true" : undefined}
              onClick={() => transit.setAt({ Jd: p.start_jd_ut })}
            >
              <span className="dasha-strip-name">{charaName(t, p.cycle, p.signs)}</span>
              <span className="dasha-strip-date">{formatJdDate(p.start_jd_ut, offset)}</span>
              {current && <span className="dasha-strip-now">{t("dasha.strip.current")}</span>}
            </button>
          </li>
        );
      })}
    </ol>
  );
}
