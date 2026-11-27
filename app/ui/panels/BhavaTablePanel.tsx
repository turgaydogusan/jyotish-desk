// Bhava tablosu paneli (yol haritası §4.5): haritanın ev sisteminde on iki bhavanın seçilen
// noktası (başlangıç, orta ya da bitiş: burç, derece, nakshatra, pada, burç lordu, nakshatra
// lordu) ve dokuz grahanın bhavası ile bhava başlangıcına ve ortasına uzaklığı. Değerler
// `compute_bhava_table`'dan gelir; burada hesap yoktur (C4). Burç evinden başka bir bhavadaki
// graha `*` ve dipnotla gösterilir; renk anlam taşımaz (C9). Doğum saati bilinmiyorsa tablo
// Rust'tan boş gelir (K2) ve panel bunu bir satırla söyler.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { BhavaPoint, BhavaTable } from "../bindings";
import { signIndex } from "../chart/adapter";
import { useHighlight } from "../highlight/store";
import type { Panel } from "../layout/model";
import { useBhavaTable } from "../modules/useBhavaTable";
import { useSession } from "../session/store";
import { useLayoutStore } from "../view/store";
import { BalaScroll } from "./bala";
import { formatDegree, formatSignedDegree } from "./format";
import { nakshatraName } from "./labels";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

export const BHAVA_POINTS = ["start", "mid", "end"] as const;
export type BhavaPointKind = (typeof BHAVA_POINTS)[number];
/** Burç evinden başka bir bhavadaki grahanın işareti. */
export const SHIFT_MARK = "*";
const DASH = "—";

/** Derece ve yay dakikası; Rust'ta sonlu olmayan değer (`null`) çizgidir. */
function deg(v: number | null): string {
  return v === null ? DASH : formatDegree(v);
}

/** Panelin nokta ayarı; tanınmayan değer ve eksik anahtar başlangıçtır. */
export function bhavaPointKind(settings: Record<string, unknown>): BhavaPointKind {
  return settings.point === "mid" || settings.point === "end" ? settings.point : "start";
}

export function BhavaTablePanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const module = useBhavaTable(opened);
  const phase = usePanelLoadPhase(module.loading);
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return (
    <div className="module-panel">
      {/* Araç çubuğu yükleme, hata ve "tablo yok" durumlarında da kullanılabilir. */}
      <div className="module-toolbar">
        <PointSelect panel={panel} />
      </div>
      {module.error ? (
        <PanelError error={module.error} onRetry={module.retry} />
      ) : module.loading ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : module.data === null ? (
        <p className="panel-notice">{t("bhavaTable.timeUnknown")}</p>
      ) : (
        <Tables table={module.data} point={bhavaPointKind(panel.settings)} />
      )}
    </div>
  );
}

/** Nokta seçicisi; değer panelin `point` ayarıdır ve düzenle birlikte saklanır. */
function PointSelect({ panel }: { panel: Panel }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  return (
    <>
      <label htmlFor={id}>{t("bhavaTable.point")}</label>
      <select id={id} value={bhavaPointKind(panel.settings)} onChange={(e) => setPanelSettings(panel.id, { point: e.target.value })}>
        {BHAVA_POINTS.map((p) => (
          <option key={p} value={p}>
            {t(`bhavaTable.points.${p}`)}
          </option>
        ))}
      </select>
    </>
  );
}

function Tables({ table, point }: { table: BhavaTable; point: BhavaPointKind }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const bhavaCaption = useId();
  const grahaCaption = useId();
  const pointName = t(`bhavaTable.points.${point}`);
  const noMid = table.rows.every((r) => r.mid === null);
  const anyShifted = table.grahas.some((g) => g.shifted);
  const activeHouse = active?.type === "house" ? active.house : null;
  return (
    <>
      <BalaScroll labelledBy={bhavaCaption}>
        <table className="module-table">
          <caption id={bhavaCaption} className="visually-hidden">
            {t("bhavaTable.caption", { point: t(`bhavaTable.pointName.${point}`) })}
          </caption>
          <thead>
            <tr>
              <th scope="col">{t("bhavaTable.bhava")}</th>
              <th scope="col">{pointName}</th>
              <th scope="col">{t("table.nakshatra")}</th>
              <th scope="col" className="num">
                {t("table.pada")}
              </th>
              <th scope="col">{t("bhavaTable.signLord")}</th>
              <th scope="col">{t("bhavaTable.nakshatraLord")}</th>
              <th scope="col" className="num">
                {t("bhavaTable.span")}
              </th>
            </tr>
          </thead>
          <tbody>
            {table.rows.map((row) => {
              const p: BhavaPoint | null = row[point];
              return (
                <tr key={row.house} data-house={row.house} className={activeHouse === row.house ? "is-highlighted" : undefined}>
                  <th scope="row">
                    <PinButton target={{ type: "house", house: row.house }} label={t("bhavaTable.houseLabel", { house: row.house })}>
                      {row.house}
                    </PinButton>
                  </th>
                  <td>{p === null ? DASH : `${t(`sign.${signIndex(p.sign)}`)} ${deg(p.deg_in_sign)}`}</td>
                  <td>{p === null ? DASH : nakshatraName(p.nakshatra)}</td>
                  <td className="num">{p === null ? DASH : p.pada}</td>
                  <td>{p === null ? DASH : <PinButton target={{ type: "graha", id: p.sign_lord }}>{t(`grahaName.${p.sign_lord}`)}</PinButton>}</td>
                  <td>{p === null ? DASH : <PinButton target={{ type: "graha", id: p.nakshatra_lord }}>{t(`grahaName.${p.nakshatra_lord}`)}</PinButton>}</td>
                  <td className="num">{deg(row.span_deg)}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </BalaScroll>
      {point === "mid" && noMid && (
        <p className="panel-notice" role="note">
          {t("bhavaTable.noMid")}
        </p>
      )}
      <BalaScroll labelledBy={grahaCaption}>
        <table className="module-table">
          <caption id={grahaCaption} className="visually-hidden">
            {t("bhavaTable.grahaCaption")}
          </caption>
          <thead>
            <tr>
              <th scope="col">{t("table.graha")}</th>
              <th scope="col" className="num">
                {t("bhavaTable.signHouse")}
              </th>
              <th scope="col" className="num">
                {t("bhavaTable.bhava")}
              </th>
              <th scope="col" className="num">
                {t("bhavaTable.fromStart")}
              </th>
              <th scope="col" className="num">
                {t("bhavaTable.fromMid")}
              </th>
            </tr>
          </thead>
          <tbody>
            {table.grahas.map((g) => {
              const on = (active?.type === "graha" && active.id === g.graha) || activeHouse === g.bhava;
              return (
                <tr key={g.graha} data-graha={g.graha} className={on ? "is-highlighted" : undefined}>
                  <th scope="row">
                    <PinButton target={{ type: "graha", id: g.graha }}>{t(`grahaName.${g.graha}`)}</PinButton>
                  </th>
                  <td className="num">{g.sign_house}</td>
                  <td className="num">
                    {g.bhava}
                    {g.shifted && (
                      <>
                        <span aria-hidden="true">{SHIFT_MARK}</span>
                        <span className="visually-hidden"> {t("bhavaTable.shifted")}</span>
                      </>
                    )}
                  </td>
                  <td className="num">{deg(g.from_start_deg)}</td>
                  <td className="num">{g.from_mid_deg === null ? DASH : formatSignedDegree(g.from_mid_deg)}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </BalaScroll>
      {anyShifted && <p className="panel-message">{t("bhavaTable.shiftedNote", { mark: SHIFT_MARK })}</p>}
      {table.porphyry_fallback && (
        <p className="panel-notice" role="note">
          {t("bhavaTable.fallback")}
        </p>
      )}
      <p className="panel-message">{t(`bhavaTable.system.${table.system}`)}</p>
      <p className="panel-message">{t("bhavaTable.method", { method: table.method })}</p>
    </>
  );
}
