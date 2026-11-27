// Navatara paneli (yol haritası §4.2): doğum Ay'ının nakshatrasından sayılan dokuz tara, üç
// turda 27 nakshatra; her nakshatrada duran doğum grahaları ve nakshatranın lordu. Altında
// transit anındaki dokuz grahanın tarası. Değerler `compute_navatara`'dan gelir ve panel
// açıkken istenir (K1); transit anı transit deposunun anıdır ve Rust'a olduğu gibi gönderilir.
// Burada hesap yoktur (C4): satırları taraya göre üç sütuna dizmek gösterimdir. Elverişli küme
// sözle yazılır; renk anlam taşımaz (C9).
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { Navatara, NavataraRow, OpenedClient } from "../bindings";
import type { GrahaId } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { inputKey } from "../modules/input";
import { useLastData } from "../modules/useModule";
import { useNavatara } from "../modules/useNatalExtras";
import { useSession } from "../session/store";
import { browserTimeZone, formatMoment, useTransit } from "../transit/store";
import { TransitDate } from "../transit/TransitDate";
import { BalaScroll } from "./bala";
import { nakshatraName, taraName } from "./labels";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

/** Dokuz tara ve üç tur. */
const TARAS = [1, 2, 3, 4, 5, 6, 7, 8, 9] as const;
const CYCLES = [1, 2, 3] as const;

export function NavataraPanel(_: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <NavataraBody opened={opened} />;
}

function NavataraBody({ opened }: { opened: OpenedClient }) {
  const { t, i18n } = useTranslation();
  const transit = useTransit();
  const view = transit.view;
  const module = useNavatara(opened, view?.snapshot.jd_ut ?? null);
  // Transit anı değişirken tablo yerinde durur; yeni yanıt gelince değişir.
  const data = useLastData(module, inputKey(opened));
  const phase = usePanelLoadPhase(module.loading || (transit.loading && view === null));
  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  if (data === null) return phase === "skeleton" ? <PanelSkeleton /> : null;
  // Gösterilen taralar transit deposundaki anın taralarıysa an yazılır; yeniden okuma sürerken
  // (bir önceki anın taraları duruyorken) yazılmaz.
  const when = view !== null && data.transit_jd_ut === view.snapshot.jd_ut ? formatMoment(view.utc, i18n.language, browserTimeZone()) : null;
  return (
    <div className="module-panel">
      {data.approximate && <p className="panel-notice">{t("navatara.timeUnknown")}</p>}
      <NatalTable data={data} />
      <p className="panel-message">{t("navatara.countNote")}</p>
      <TransitDate />
      {transit.error ? <PanelError error={transit.error} onRetry={transit.retry} /> : data.transit !== null && <TransitTable data={data} when={when} />}
      <p className="panel-message">{t("navatara.method", { method: data.method })}</p>
    </div>
  );
}

function NatalTable({ data }: { data: Navatara }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const id = useId();
  const on = (rows: readonly NavataraRow[]) => active?.type === "graha" && rows.some((r) => r.lord === active.id || r.grahas.includes(active.id));
  return (
    <BalaScroll labelledBy={`${id}-natal`}>
      <table className="module-table">
        <caption id={`${id}-natal`} className="visually-hidden">
          {t("navatara.caption", { nakshatra: nakshatraName(data.janma_nakshatra) })}
        </caption>
        <thead>
          <tr>
            <th scope="col">{t("navatara.tara")}</th>
            {CYCLES.map((c) => (
              <th key={c} scope="col">
                {t("navatara.cycle", { n: c })}
              </th>
            ))}
            <th scope="col">{t("navatara.lord")}</th>
            <th scope="col">{t("navatara.set")}</th>
          </tr>
        </thead>
        <tbody>
          {TARAS.map((n) => {
            const rows = data.rows.filter((r) => r.tara.number === n);
            const first = rows[0];
            if (first === undefined) return null;
            return (
              <tr key={n} data-tara={n} className={on(rows) ? "is-highlighted" : undefined}>
                <th scope="row">{t("navatara.taraValue", { number: n, name: taraName(n) })}</th>
                {CYCLES.map((c) => {
                  const cell = rows.find((r) => r.cycle === c);
                  return (
                    <td key={c} data-nakshatra={cell?.nakshatra}>
                      {cell && (
                        <>
                          {nakshatraName(cell.nakshatra)}
                          {cell.grahas.length > 0 && (
                            <>
                              {" ("}
                              {cell.grahas.map((g, i) => (
                                <span key={g}>
                                  {i > 0 && " "}
                                  <PinButton target={{ type: "graha", id: g as GrahaId }} label={t(`grahaName.${g}`)}>
                                    {t(`graha.${g}`)}
                                  </PinButton>
                                </span>
                              ))}
                              {")"}
                            </>
                          )}
                        </>
                      )}
                    </td>
                  );
                })}
                <td>
                  <PinButton target={{ type: "graha", id: first.lord }}>{t(`grahaName.${first.lord}`)}</PinButton>
                </td>
                <td>{t(first.tara.favourable ? "navatara.favourable" : "navatara.notFavourable")}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </BalaScroll>
  );
}

function TransitTable({ data, when }: { data: Navatara; when: string | null }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const id = useId();
  return (
    <BalaScroll labelledBy={`${id}-transit`}>
      <table className="module-table">
        <caption id={`${id}-transit`}>{when === null ? t("navatara.transitBusy") : t("navatara.transitCaption", { when })}</caption>
        <thead>
          <tr>
            <th scope="col">{t("navatara.graha")}</th>
            <th scope="col">{t("navatara.nakshatra")}</th>
            <th scope="col">{t("navatara.tara")}</th>
            <th scope="col" className="num">
              {t("navatara.cycleColumn")}
            </th>
            <th scope="col">{t("navatara.set")}</th>
          </tr>
        </thead>
        <tbody>
          {(data.transit ?? []).map((row) => (
            <tr key={row.graha} data-graha={row.graha} className={active?.type === "graha" && active.id === row.graha ? "is-highlighted" : undefined}>
              <th scope="row">
                <PinButton target={{ type: "graha", id: row.graha }}>{t(`grahaName.${row.graha}`)}</PinButton>
              </th>
              <td>
                {nakshatraName(row.nakshatra)} · {t("panchang.pada", { pada: row.pada })}
              </td>
              <td>{t("navatara.taraValue", { number: row.tara.number, name: taraName(row.tara.number) })}</td>
              <td className="num">{row.cycle}</td>
              <td>{t(row.tara.favourable ? "navatara.favourable" : "navatara.notFavourable")}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </BalaScroll>
  );
}
