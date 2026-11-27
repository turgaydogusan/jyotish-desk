// Yogalar paneli (ileri modüller spec'i §8): kural kataloğunun D1'de oluşan kayıtları, gruplara
// ayrılmış listeyle. Her satır yoganın adını, oluşturan grahaları, lagnadan sayılan evleri,
// ilişki türünü ve sağlanan alt koşulları yazar. Değerler `compute_yogas`'tan gelir; burada hesap
// yoktur (C4). Yoganın ne anlama geldiğini anlatan metin yoktur; adlar ve koşullar etikettir.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { OpenedClient, YogaGroup, YogaHit, YogaReport } from "../bindings";
import { signIndex } from "../chart/adapter";
import { useHighlight } from "../highlight/store";
import type { HighlightTarget } from "../highlight/target";
import type { Panel } from "../layout/model";
import { useYogas } from "../modules/useYogas";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { useLayoutStore } from "../view/store";
import { Approx } from "./Approx";
import { BalaScroll } from "./bala";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

/** Rust `YogaGroup::ALL`: katalog sırası. */
export const YOGA_GROUPS = [
  "Mahapurusha",
  "Lunar",
  "Solar",
  "Raja",
  "Dhana",
  "Exchange",
  "ViparitaRaja",
  "NeechaBhanga",
  "NabhasaCount",
  "NabhasaAshraya",
  "Dosha",
] as const satisfies readonly YogaGroup[];
export type YogaGroupFilter = "all" | YogaGroup;
const DASH = "—";

/** Panelin grup süzgeci; tanınmayan değer ve eksik anahtar "tümü"dür. */
export function yogaGroupFilter(settings: Record<string, unknown>): YogaGroupFilter {
  return (YOGA_GROUPS as readonly unknown[]).includes(settings.group) ? (settings.group as YogaGroup) : "all";
}

/** Ortak vurgu bu kaydın bir grahasını ya da evini gösteriyor mu (burç, lagnadan sayılan evidir). */
function hitIsActive(hit: YogaHit, active: HighlightTarget | null, lagna: number | null): boolean {
  if (active === null) return false;
  if (active.type === "graha") return hit.grahas.some((g) => g === active.id);
  if (active.type === "house") return hit.houses.includes(active.house);
  return lagna !== null && hit.houses.includes(((active.sign - lagna + 12) % 12) + 1);
}

export function YogasPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <Yogas panel={panel} opened={opened} />;
}

function Yogas({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const module = useYogas(opened);
  const phase = usePanelLoadPhase(module.loading);
  const id = useId();
  const filter = yogaGroupFilter(panel.settings);
  return (
    <div className="module-panel">
      {/* Süzgeç yükleme ve hata durumlarında da kullanılabilir. */}
      <div className="module-toolbar">
        <label htmlFor={`${id}-group`}>{t("yogas.groupLabel")}</label>
        <select id={`${id}-group`} value={filter} onChange={(e) => setPanelSettings(panel.id, { group: e.target.value })}>
          <option value="all">{t("yogas.allGroups")}</option>
          {YOGA_GROUPS.map((g) => (
            <option key={g} value={g}>
              {t(`yogas.group.${g}`)}
            </option>
          ))}
        </select>
      </div>
      {module.error ? (
        <PanelError error={module.error} onRetry={module.retry} />
      ) : module.data === null ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : (
        <YogaList report={module.data} filter={filter} opened={opened} />
      )}
    </div>
  );
}

/** Yoga raporunun gruplu listesi; prashna olguları paneli de kullanır. */
export function YogaList({ report, filter, opened }: { report: YogaReport; filter: YogaGroupFilter; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const listId = useId();
  const lagna = timeUnknown(opened) ? null : signIndex(opened.chart.lagna.sign);
  // "All" follows the report: hits arrive in catalog order, so the groups in order of first
  // appearance are the catalog's groups. A group this build has no name for is still listed.
  const groups = (filter === "all" ? [...new Set(report.hits.map((h) => h.group))] : [filter])
    .map((group) => ({ group, hits: report.hits.filter((h) => h.group === group) }))
    .filter((g) => g.hits.length > 0);

  const row = (hit: YogaHit, index: number) => (
    <tr key={`${hit.id}:${index}`} data-yoga={hit.id} className={hitIsActive(hit, active, lagna) ? "is-highlighted" : undefined}>
      <th scope="row">
        <Approx on={hit.approximate} label={t("yogas.approximate")}>
          {t(`yogas.name.${hit.id}`)}
        </Approx>
      </th>
      <td>
        {hit.grahas.map((g, i) => (
          <span key={g}>
            {i > 0 && " "}
            <PinButton target={{ type: "graha", id: g }} label={t(`grahaName.${g}`)}>
              {t(`graha.${g}`)}
            </PinButton>
          </span>
        ))}
      </td>
      <td className="num">
        {hit.houses.length === 0
          ? DASH
          : hit.houses.map((h, i) => (
              <span key={h}>
                {i > 0 && " "}
                <PinButton target={{ type: "house", house: h }} label={t("yogas.houseLabel", { house: h })}>
                  {h}
                </PinButton>
              </span>
            ))}
      </td>
      <td>{hit.relations.length === 0 ? DASH : hit.relations.map((r) => t(`yogas.relation.${r}`)).join(", ")}</td>
      <td>{hit.conditions.length === 0 ? DASH : hit.conditions.map((c) => t(`yogas.condition.${c}`)).join("; ")}</td>
    </tr>
  );

  return (
    <>
      {report.hits.length === 0 ? (
        <p className="panel-message">{t("yogas.empty", { n: report.evaluated })}</p>
      ) : groups.length === 0 ? (
        <p className="panel-message">{t("yogas.emptyGroup", { group: t(`yogas.group.${filter}`) })}</p>
      ) : (
        // İlişki ve koşul sütunlarında odaklanabilir denetim yoktur: dar panelde kaydırma kabı
        // klavye odağı alır ve adını gizli başlıktan okur.
        <BalaScroll labelledBy={listId}>
          <p id={listId} className="visually-hidden">
            {t("yogas.listLabel")}
          </p>
          {groups.map(({ group, hits }) => (
            <table key={group} className="module-table yoga-table" data-group={group}>
              <caption>{t(`yogas.group.${group}`)}</caption>
              <thead>
                <tr>
                  <th scope="col">{t("yogas.yoga")}</th>
                  <th scope="col">{t("yogas.grahas")}</th>
                  <th scope="col" className="num">
                    {t("yogas.houses")}
                  </th>
                  <th scope="col">{t("yogas.relationColumn")}</th>
                  <th scope="col">{t("yogas.conditions")}</th>
                </tr>
              </thead>
              <tbody>{hits.map(row)}</tbody>
            </table>
          ))}
        </BalaScroll>
      )}
      {report.skipped_time_unknown > 0 && <p className="panel-notice">{t("yogas.timeUnknown", { n: report.skipped_time_unknown })}</p>}
      <p className="panel-message">{t("yogas.evaluated", { n: report.evaluated, hits: report.hits.length })}</p>
      <p className="panel-message">{t("yogas.note")}</p>
      <p className="panel-message">{t("yogas.catalog", { version: report.catalog_version })}</p>
    </>
  );
}
