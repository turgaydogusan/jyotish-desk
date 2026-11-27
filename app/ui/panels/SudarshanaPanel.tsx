// Sudarshana chakra paneli (yol haritası §4.6): lagna, Ay ve Güneş'i birinci ev sayan üç halka
// ve çizimin tablo karşılığı. Değerler `compute_sudarshana`'dan gelir; burada hesap yoktur (C4).
// Çizim tek bir resimdir; klavye ve ekran okuyucu tabloyu kullanır: her hücrede burç ve grahalar
// ortak vurgunun düğmeleridir (UI spec §3.4). Doğum saati bilinmiyorsa lagna halkası Rust'tan boş
// gelir (K2): halka çizilmez, sütunu çizgi taşır ve Ay'ın burcu yaklaşık (≈) işaretlenir.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { SudarshanaChakra } from "../bindings";
import { signIndex } from "../chart/adapter";
import { SudarshanaSvg, type SudarshanaRingData } from "../chart/SudarshanaSvg";
import { isGrahaId } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { useSudarshana } from "../modules/useSudarshana";
import { useSession } from "../session/store";
import { Approx } from "./Approx";
import { BalaScroll } from "./bala";
import { grahaAbbreviations } from "./labels";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

const DASH = "—";

export function SudarshanaPanel(_: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const module = useSudarshana(opened);
  const phase = usePanelLoadPhase(module.loading);
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  if (module.data === null) return phase === "skeleton" ? <PanelSkeleton /> : null;
  return <Chakra chakra={module.data} />;
}

function Chakra({ chakra }: { chakra: SudarshanaChakra }) {
  const { t } = useTranslation();
  const highlight = useHighlight();
  const { active } = highlight;
  const captionId = useId();
  // Çizim verisi: burç dizini ve graha kimlikleri (biçim dönüşümü; hesap değil).
  const rings: SudarshanaRingData[] = chakra.rings.map((r) => ({
    houses: r.houses === null ? null : r.houses.map((h) => ({ house: h.house, sign: signIndex(h.sign), grahas: h.grahas.filter(isGrahaId) })),
  }));
  // Ev vurgusu D1 evidir: lagna halkasının o evindeki burç.
  const activeSign = active?.type === "sign" ? active.sign : active?.type === "house" ? (rings[0]?.houses?.[active.house - 1]?.sign ?? null) : null;
  const activeGraha = active?.type === "graha" ? active.id : null;
  return (
    <div className="module-panel">
      {chakra.approximate && (
        <p className="panel-notice" role="note">
          {t("sudarshana.timeUnknown")}
        </p>
      )}
      <div className="sudarshana-figure">
        <SudarshanaSvg
          rings={rings}
          labels={{ title: t("sudarshana.title"), graha: grahaAbbreviations(t), lagna: t("chart.lagna") }}
          highlight={active}
          onHover={highlight.hover}
          onPin={highlight.togglePin}
        />
      </div>
      <p className="panel-message">{t("sudarshana.legend", { lagna: t("chart.lagna") })}</p>
      <BalaScroll labelledBy={captionId}>
        <table className="module-table">
          <caption id={captionId} className="visually-hidden">
            {t("sudarshana.caption")}
          </caption>
          <thead>
            <tr>
              <th scope="col">{t("sudarshana.house")}</th>
              {chakra.rings.map((r) => (
                <th key={r.reference} scope="col" data-ring={r.reference}>
                  <Approx on={chakra.approximate && r.reference === "Moon"} label={t("table.approx")}>
                    {t(`sudarshana.ring.${r.reference}`)}
                  </Approx>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {Array.from({ length: 12 }, (_, i) => (
              <tr key={i} data-house={i + 1}>
                <th scope="row">{i + 1}</th>
                {rings.map((ring, k) => {
                  const h = ring.houses?.[i];
                  if (h === undefined) return <td key={chakra.rings[k].reference}>{DASH}</td>;
                  const on = activeSign === h.sign || (activeGraha !== null && h.grahas.includes(activeGraha));
                  return (
                    <td key={chakra.rings[k].reference} className={on ? "is-highlighted" : undefined}>
                      <PinButton target={{ type: "sign", sign: h.sign }}>{t(`sign.${h.sign}`)}</PinButton>
                      {h.grahas.map((g) => (
                        <span key={g}>
                          {" "}
                          <PinButton target={{ type: "graha", id: g }} label={t(`grahaName.${g}`)}>
                            {t(`graha.${g}`)}
                          </PinButton>
                        </span>
                      ))}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </BalaScroll>
      <p className="panel-message">{t("sudarshana.method", { method: chakra.method })}</p>
    </div>
  );
}
