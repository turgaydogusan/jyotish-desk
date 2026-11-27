// Panchang ekleri (yol haritası §4.1): yogi noktası, yogi, eş yogi, avayogi ve dagdha rasi.
// Panchang panelinin ikinci tablosudur. Değerler `compute_panchang_extras`'tan gelir ve panel
// açıkken istenir (K1); burada hesap yoktur (C4). Doğum saati bilinmiyorsa Rust `approximate`
// döner ve değerler "≈" ile işaretlenir. Dagdha rasi tablosu sahibinin onayını bekler (Y4):
// tablo "doğrulanmamış tablo" notunu taşır.
import { useTranslation } from "react-i18next";
import type { OpenedClient, PanchangExtras } from "../bindings";
import { signIndex } from "../chart/adapter";
import type { GrahaId } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { useLastData } from "../modules/useModule";
import { usePanchangExtras } from "../modules/useNatalExtras";
import { Approx } from "./Approx";
import { formatDegree } from "./format";
import { nakshatraName } from "./labels";
import { PinButton } from "./PinButton";
import { PanelError, usePanelLoadPhase } from "./states";

export function PanchangExtrasSection({ opened }: { opened: OpenedClient }) {
  const module = usePanchangExtras(opened);
  // Aynı haritanın yeniden okunmasında (ayar değişimi) tablo yerinde durur.
  const extras = useLastData(module, String(opened.chart_record.id));
  usePanelLoadPhase(module.loading);
  if (module.error) return <PanelError error={module.error} onRetry={module.retry} />;
  if (extras === null) return null;
  return <ExtrasTable extras={extras} />;
}

function ExtrasTable({ extras }: { extras: PanchangExtras }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const { yogi } = extras;
  const pointSign = signIndex(yogi.sign);
  const dagdha = extras.dagdha_rasis.map(signIndex);
  const onGraha = (g: GrahaId) => active?.type === "graha" && active.id === g;
  const onSign = (signs: readonly number[]) => active?.type === "sign" && signs.includes(active.sign);
  const row = (on: boolean | undefined) => (on ? "is-highlighted" : undefined);
  const approx = { on: extras.approximate, label: t("table.approx") };
  const graha = (g: GrahaId) => <PinButton target={{ type: "graha", id: g }}>{t(`grahaName.${g}`)}</PinButton>;
  const sign = (s: number) => <PinButton target={{ type: "sign", sign: s }}>{t(`sign.${s}`)}</PinButton>;

  return (
    <>
      <table className="module-table">
        <caption>{t("panchangExtras.caption")}</caption>
        <thead>
          <tr>
            <th scope="col">{t("panchang.item")}</th>
            <th scope="col">{t("panchang.value")}</th>
          </tr>
        </thead>
        <tbody>
          <tr data-extra="yogi_point" className={row(onSign([pointSign]))}>
            <th scope="row">{t("panchangExtras.yogiPoint")}</th>
            <td>
              <Approx {...approx}>
                {sign(pointSign)} {yogi.deg_in_sign === null ? "—" : formatDegree(yogi.deg_in_sign)} · {nakshatraName(yogi.nakshatra)} · {t("panchang.pada", { pada: yogi.pada })}
              </Approx>
            </td>
          </tr>
          <tr data-extra="yogi" className={row(onGraha(yogi.yogi))}>
            <th scope="row">{t("panchangExtras.yogi")}</th>
            <td>
              <Approx {...approx}>{graha(yogi.yogi)}</Approx>
            </td>
          </tr>
          <tr data-extra="co_yogi" className={row(onGraha(yogi.co_yogi))}>
            <th scope="row">{t("panchangExtras.coYogi")}</th>
            <td>
              <Approx {...approx}>{graha(yogi.co_yogi)}</Approx>
            </td>
          </tr>
          <tr data-extra="avayogi" className={row(onGraha(yogi.avayogi))}>
            <th scope="row">{t("panchangExtras.avayogi")}</th>
            <td>
              <Approx {...approx}>
                {graha(yogi.avayogi)} · {nakshatraName(yogi.avayogi_nakshatra)}
              </Approx>
            </td>
          </tr>
          <tr data-extra="dagdha" className={row(onSign(dagdha))}>
            <th scope="row">{t("panchangExtras.dagdha")}</th>
            <td>
              <Approx {...approx}>
                {dagdha.length === 0
                  ? t("panchangExtras.none")
                  : dagdha.map((s, i) => (
                      <span key={s}>
                        {i > 0 && ", "}
                        {sign(s)}
                      </span>
                    ))}
              </Approx>
            </td>
          </tr>
        </tbody>
      </table>
      <p className="panel-message">{t("panchangExtras.unverified")}</p>
      <p className="panel-message">{t("panchangExtras.method", { method: extras.method })}</p>
    </>
  );
}
