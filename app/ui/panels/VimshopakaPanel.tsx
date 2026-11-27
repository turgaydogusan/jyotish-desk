// Vimshopaka bala paneli (yol haritası §4.3): yedi grahanın dört varga kümesindeki (shadvarga,
// saptavarga, dashavarga, shodashavarga) yirmi üzerinden değeri; "Vargalar" görünümünde on altı
// vargadaki burç ve basamak değeri. Değerler `compute_vimshopaka`'dan gelir ve panel açıkken
// istenir (K1); burada hesap yoktur (C4). Değerler 0,025'in katıdır ve üç ondalıkla,
// yuvarlanmadan yazılır. Ağırlık ve basamak tabloları sahibinin onayını bekler (Y4): panel
// "doğrulanmamış tablo" notunu taşır. Renk anlam taşımaz (C9).
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { GrahaVimshopaka, OpenedClient, VimshopakaReport } from "../bindings";
import { signIndex } from "../chart/adapter";
import { useHighlight } from "../highlight/store";
import type { Panel } from "../layout/model";
import { useLastData } from "../modules/useModule";
import { useVimshopaka } from "../modules/useNatalExtras";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { useLayoutStore } from "../view/store";
import { BalaScroll, formatFixed } from "./bala";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

export const VIMSHOPAKA_DETAILS = ["summary", "vargas"] as const;
export type VimshopakaDetail = (typeof VIMSHOPAKA_DETAILS)[number];

/** Dört küme: raporun alanı ve kümenin klasik adı (üç dilde aynı yazılır, K11). Rust `VargaScheme::ALL` sırası. */
export const VIMSHOPAKA_SCHEMES = [
  ["shadvarga", "Shadvarga"],
  ["saptavarga", "Saptavarga"],
  ["dashavarga", "Dashavarga"],
  ["shodashavarga", "Shodashavarga"],
] as const satisfies readonly (readonly [keyof GrahaVimshopaka, string])[];

/** Panelin ayrıntı ayarı; tanınmayan değer ve eksik anahtar özettir. */
export function vimshopakaDetail(settings: Record<string, unknown>): VimshopakaDetail {
  return settings.detail === "vargas" ? "vargas" : "summary";
}

/** Küme değeri, üç ondalık (değerler 0,025'in katıdır: yuvarlama olmaz). */
export function formatVimshopaka(value: number | null): string {
  return formatFixed(value, 3);
}

export function VimshopakaPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <Vimshopaka panel={panel} opened={opened} />;
}

function Vimshopaka({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  const module = useVimshopaka(opened);
  // Aynı haritanın yeniden okunmasında (ayar değişimi) tablo yerinde durur.
  const report = useLastData(module, String(opened.chart_record.id));
  const phase = usePanelLoadPhase(module.loading);
  const detail = vimshopakaDetail(panel.settings);
  // Saat kesinliği doğum kaydındandır (gösterim kararı; hesap değil).
  const approxTime = !timeUnknown(opened) && opened.chart_record.birth.time_certainty === "Approx";
  return (
    <div className="module-panel">
      {/* Araç çubuğu yükleme ve hata durumlarında da kullanılabilir. */}
      <div className="module-toolbar">
        <label htmlFor={`${id}-detail`}>{t("vimshopaka.detail")}</label>
        <select id={`${id}-detail`} value={detail} onChange={(e) => setPanelSettings(panel.id, { detail: e.target.value })}>
          {VIMSHOPAKA_DETAILS.map((d) => (
            <option key={d} value={d}>
              {t(`vimshopaka.details.${d}`)}
            </option>
          ))}
        </select>
      </div>
      {module.error ? (
        <PanelError error={module.error} onRetry={module.retry} />
      ) : report === null ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : (
        <>
          {report.approximate && <p className="panel-notice">{t("vimshopaka.timeUnknown")}</p>}
          {!report.approximate && approxTime && <p className="panel-notice">{t("vimshopaka.timeApprox")}</p>}
          <Report report={report} detail={detail} />
        </>
      )}
    </div>
  );
}

function Report({ report, detail }: { report: VimshopakaReport; detail: VimshopakaDetail }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const id = useId();
  const row = (g: GrahaVimshopaka) => (active?.type === "graha" && active.id === g.graha ? "is-highlighted" : undefined);
  return (
    <>
      <BalaScroll labelledBy={`${id}-summary`}>
        <table className="module-table">
          <caption id={`${id}-summary`} className="visually-hidden">
            {t("vimshopaka.caption")}
          </caption>
          <thead>
            <tr>
              <th scope="col">{t("vimshopaka.graha")}</th>
              {VIMSHOPAKA_SCHEMES.map(([key, name]) => (
                <th key={key} scope="col" className="num">
                  {name}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {report.grahas.map((g) => (
              <tr key={g.graha} data-graha={g.graha} className={row(g)}>
                <th scope="row">
                  <PinButton target={{ type: "graha", id: g.graha }}>{t(`grahaName.${g.graha}`)}</PinButton>
                </th>
                {VIMSHOPAKA_SCHEMES.map(([key]) => (
                  <td key={key} className="num">
                    {formatVimshopaka(g[key])}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </BalaScroll>
      {detail === "vargas" && (
        <BalaScroll labelledBy={`${id}-vargas`}>
          <table className="module-table">
            <caption id={`${id}-vargas`}>{t("vimshopaka.vargasCaption")}</caption>
            <thead>
              <tr>
                <th scope="col">{t("vimshopaka.graha")}</th>
                {(report.grahas[0]?.vargas ?? []).map((v) => (
                  <th key={v.varga} scope="col">
                    {v.varga}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {report.grahas.map((g) => (
                <tr key={g.graha} data-graha={g.graha} className={row(g)}>
                  <th scope="row">{t(`grahaName.${g.graha}`)}</th>
                  {g.vargas.map((v) => (
                    <td key={v.varga} data-varga={v.varga}>
                      {t(`sign.${signIndex(v.sign)}`)} {v.points === null ? "—" : v.points}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </BalaScroll>
      )}
      <p className="panel-message">{t("vimshopaka.legend")}</p>
      <p className="panel-message">{t("vimshopaka.reading")}</p>
      <p className="panel-message">{t("vimshopaka.unverified")}</p>
      <p className="panel-message">{t("vimshopaka.method", { method: report.method })}</p>
    </>
  );
}
