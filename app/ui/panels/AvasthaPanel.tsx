// Avastha paneli (yol haritası §4.4): yedi grahanın Baladi, Jagradadi, Lajjitadi ve Shayanadi
// durumu; altında Shayanadi formülünün terimleri. Değerler `compute_avasthas`'tan gelir ve panel
// açıkken istenir (K1); burada hesap yoktur (C4): artan da Rust'ın alanıdır. Lajjitadi bir
// kümedir: bir graha birden çok durumda olabilir ya da hiçbirinde olmayabilir. Yaklaşık değer
// `≈` ve metinle, boş değer çizgi ve metinle gösterilir; renk anlam taşımaz (C9). Yorum ve
// hüküm yoktur.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { AvasthaReport, Graha, GrahaAvastha } from "../bindings";
import { useHighlight } from "../highlight/store";
import { useAvasthas } from "../modules/useAvasthas";
import { useSession } from "../session/store";
import { Approx } from "./Approx";
import "./avastha.css";
import { BALADI_NAMES, JAGRADADI_NAMES, LAJJITADI_NAMES, SHAYANADI_NAMES } from "./avasthaLabels";
import { BalaScroll } from "./bala";
import { NAKSHATRA_NAMES } from "./labels";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

const DASH = "—";

export function AvasthaPanel(_: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const module = useAvasthas(opened);
  const phase = usePanelLoadPhase(module.loading);
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  if (module.data === null) return phase === "skeleton" ? <PanelSkeleton /> : null;
  return <Avasthas report={module.data} />;
}

/** Boş değer: görünen çizgi ve ekran okuyucu için sözcük. */
function Empty({ label }: { label: string }) {
  return (
    <>
      <span aria-hidden="true">{DASH}</span>
      <span className="visually-hidden">{label}</span>
    </>
  );
}

function Avasthas({ report }: { report: AvasthaReport }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const id = useId();
  const on = (g: Graha) => active?.type === "graha" && active.id === g;
  // Saat bilinmiyorsa Ay'ın derecesi ve burcu gün içinde değişebilir: Ay'ın satırı yaklaşıktır.
  const approx = (row: GrahaAvastha) => !report.time_known && row.graha === "Moon";
  const basis = report.shayanadi_basis;

  return (
    <div className="module-panel">
      {!report.time_known && <p className="panel-notice">{t("avastha.timeUnknown")}</p>}
      {report.time_known && basis === null && <p className="panel-notice">{t("avastha.noSunrise")}</p>}
      {report.warnings.map((w) => (
        <p key={w} className="panel-notice">
          <strong>{t(`badge.${w}`)}</strong>: {t(`badgeHint.${w}`)}
        </p>
      ))}
      <p className="panel-notice">{t("avastha.unverified")}</p>
      <BalaScroll labelledBy={`${id}-states`}>
        <table className="module-table">
          <caption id={`${id}-states`} className="visually-hidden">
            {t("avastha.caption")}
          </caption>
          <thead>
            <tr>
              <th scope="col">{t("table.graha")}</th>
              <th scope="col">{t("avastha.baladi")}</th>
              <th scope="col">{t("avastha.jagradadi")}</th>
              <th scope="col">{t("avastha.lajjitadi")}</th>
              <th scope="col">{t("avastha.shayanadi")}</th>
            </tr>
          </thead>
          <tbody>
            {report.grahas.map((row) => (
              <tr key={row.graha} data-graha={row.graha} className={on(row.graha) ? "is-highlighted" : undefined}>
                <th scope="row">
                  <PinButton target={{ type: "graha", id: row.graha }}>{t(`grahaName.${row.graha}`)}</PinButton>
                </th>
                <td data-series="baladi">
                  <Approx on={approx(row)} label={t("table.approx")}>
                    {BALADI_NAMES[row.baladi]}
                  </Approx>
                </td>
                <td data-series="jagradadi">
                  <Approx on={approx(row)} label={t("table.approx")}>
                    {JAGRADADI_NAMES[row.jagradadi]}
                  </Approx>
                </td>
                <td data-series="lajjitadi">
                  {row.lajjitadi.length === 0 ? (
                    <Empty label={t("avastha.none")} />
                  ) : (
                    <Approx on={approx(row)} label={t("table.approx")}>
                      <ul className="avastha-states">
                        {row.lajjitadi.map((hit) => (
                          <li key={hit.state} data-state={hit.state}>
                            {LAJJITADI_NAMES[hit.state]}
                            {hit.grahas.length > 0 && <span className="avastha-by"> ({hit.grahas.map((g) => t(`grahaName.${g}`)).join(", ")})</span>}
                          </li>
                        ))}
                      </ul>
                    </Approx>
                  )}
                </td>
                <td data-series="shayanadi">{row.shayanadi ? SHAYANADI_NAMES[row.shayanadi.state] : <Empty label={t("avastha.notComputed")} />}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </BalaScroll>
      {basis !== null && (
        <>
          <BalaScroll labelledBy={`${id}-terms`}>
            <table className="module-table">
              <caption id={`${id}-terms`}>{t("avastha.termsCaption")}</caption>
              <thead>
                <tr>
                  <th scope="col">{t("table.graha")}</th>
                  {(["nakshatra", "grahaNumber", "navamsa", "total", "remainder"] as const).map((k) => (
                    <th key={k} scope="col" className="num">
                      {t(`avastha.${k}`)}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {report.grahas.map(
                  (row) =>
                    row.shayanadi && (
                      <tr key={row.graha} data-terms-of={row.graha} className={on(row.graha) ? "is-highlighted" : undefined}>
                        <th scope="row">{t(`grahaName.${row.graha}`)}</th>
                        <td className="num">{row.shayanadi.nakshatra}</td>
                        <td className="num">{row.shayanadi.graha_number}</td>
                        <td className="num">{row.shayanadi.navamsa}</td>
                        <td className="num">{row.shayanadi.total}</td>
                        <td className="num">{row.shayanadi.remainder}</td>
                      </tr>
                    ),
                )}
              </tbody>
            </table>
          </BalaScroll>
          <p className="panel-message">
            {t("avastha.basis", {
              nakshatra: basis.birth_nakshatra,
              name: NAKSHATRA_NAMES[basis.birth_nakshatra - 1] ?? DASH,
              ghatis: basis.ghatis,
              lagna: basis.lagna_sign_number,
              sign: t(`sign.${basis.lagna_sign_number - 1}`),
            })}
          </p>
          <p className="panel-message">{t("avastha.formula")}</p>
        </>
      )}
      <p className="panel-message">{t("avastha.note")}</p>
      <p className="panel-message">{t("avastha.method", { method: report.method })}</p>
    </div>
  );
}
