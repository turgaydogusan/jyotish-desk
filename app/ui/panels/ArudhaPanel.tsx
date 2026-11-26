// Arudha padalar paneli (ileri modüller spec'i §5): A1 (AL) … A12 (UL) için padanın burcu ve
// evin hesapta kullanılan lordu. Değerler `compute_jaimini`'den gelir; burada hesap yoktur (C4).
// İstisna `*` ve dipnotla gösterilir; renk anlam taşımaz (C9). Doğum saati bilinmiyorsa padalar
// Rust'tan boş gelir (K2) ve panel bunu bir satırla söyler.
import { useTranslation } from "react-i18next";
import type { JaiminiReport, OpenedClient } from "../bindings";
import { signIndex } from "../chart/adapter";
import { useHighlight } from "../highlight/store";
import { useJaimini } from "../modules/useJaimini";
import { useSession } from "../session/store";
import { arudhaLabel } from "./labels";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

export const EXCEPTION_MARK = "*";

export function ArudhaPanel(_: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const module = useJaimini(opened);
  const phase = usePanelLoadPhase(module.loading);
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  if (module.data === null) return phase === "skeleton" ? <PanelSkeleton /> : null;
  if (module.data.arudhas === null) return <p className="panel-notice">{t("arudha.timeUnknown")}</p>;
  return <Padas report={module.data} padas={module.data.arudhas} opened={opened} />;
}

function Padas({ report, padas, opened }: { report: JaiminiReport; padas: NonNullable<JaiminiReport["arudhas"]>; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const lagna = signIndex(opened.chart.lagna.sign);
  // Ev vurgusu D1 evidir: lagna burcundan sayılan burç (gösterim eşlemesi, hesap değil).
  const activeSign = active?.type === "sign" ? active.sign : active?.type === "house" ? (lagna + active.house - 1) % 12 : null;
  const anyException = padas.some((p) => p.exception);

  return (
    <div className="module-panel">
      <table className="module-table">
        <caption className="visually-hidden">{t("arudha.caption")}</caption>
        <thead>
          <tr>
            <th scope="col">{t("arudha.pada")}</th>
            <th scope="col">{t("table.sign")}</th>
            <th scope="col">{t("arudha.lord")}</th>
          </tr>
        </thead>
        <tbody>
          {padas.map((p) => {
            const sign = signIndex(p.sign);
            const on = activeSign === sign || (active?.type === "graha" && active.id === p.lord);
            const label = arudhaLabel(p.house);
            return (
              <tr key={p.house} data-pada={p.house} className={on ? "is-highlighted" : undefined}>
                <th scope="row">
                  {`A${p.house}`}
                  {label !== `A${p.house}` && <> · {label}</>}
                </th>
                <td>
                  <PinButton target={{ type: "sign", sign }}>{t(`sign.${sign}`)}</PinButton>
                  {p.exception && (
                    <>
                      <span aria-hidden="true">{EXCEPTION_MARK}</span>
                      <span className="visually-hidden"> {t("arudha.exception")}</span>
                    </>
                  )}
                </td>
                <td>
                  <PinButton target={{ type: "graha", id: p.lord }}>{t(`grahaName.${p.lord}`)}</PinButton>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
      {anyException && <p className="panel-message">{t("arudha.exceptionNote", { mark: EXCEPTION_MARK })}</p>}
      <p className="panel-message">{t(report.co_lords ? "arudha.coLordsOn" : "arudha.coLordsOff")}</p>
      <p className="panel-message">{t("jaimini.method", { method: report.method })}</p>
    </div>
  );
}
