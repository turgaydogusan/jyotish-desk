// Uyum paneli (ileri modüller spec'i §9): Ashtakuta'nın sekiz puanı ve iki tarafın sınıfları,
// toplam, iki istisna olgusu, iki haritanın Kuja dosha olgusu ve karşılıklı yerleşim. Değerler
// `compute_compat`'tan gelir; burada hesap yoktur (C4). Uyum yüzdesi, hüküm ve açıklama metni
// yoktur; adlar ve sınıflar etikettir. Eş harita açık haritanın görünüm kaydındadır ve eş
// seçiciyle atanır; roller ("damat rolü", "gelin rolü") açıkça gösterilir ve değiştirilebilir.
import { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type { KujaDosha, Kuta, KutaReport, KutaRole, KutaScore, OpenedClient, OverlayRow } from "../bindings";
import { signIndex, SIGNS } from "../chart/adapter";
import { GRAHA_IDS } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { useCompat, type CompatPartner } from "../modules/useCompat";
import { timeUnknown } from "../session/birth";
import { useSession } from "../session/store";
import { Approx } from "./Approx";
import { BalaScroll } from "./bala";
import "./panels.css";
import { PartnerPicker } from "./PartnerPicker";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";

/** Rust `Kuta::ALL`: puan tablosunun sırası. */
export const KUTAS = ["Varna", "Vashya", "Tara", "Yoni", "GrahaMaitri", "Gana", "Bhakoot", "Nadi"] as const satisfies readonly Kuta[];
const ROLES = ["Groom", "Bride"] as const satisfies readonly KutaRole[];
const DASH = "—";

/**
 * Puan: tam sayı ya da yarım ("1½"); Rust'tan ½'nin katı olarak gelir. Boş değer (Rust'ta sonlu
 * olmayan sayı) çizgidir. Gösterimdir (C4).
 */
export function formatKutaScore(score: number | null): string {
  if (score === null) return DASH;
  const whole = Math.floor(score);
  if (score === whole) return String(whole);
  return whole === 0 ? "½" : `${whole}½`;
}

type Translate = ReturnType<typeof useTranslation>["t"];

/** Bir sınıf kimliğinin görünen adı: graha maitride lordun, Bhakoot'ta burcun adı. */
export function kutaClassLabel(t: Translate, kuta: Kuta, id: string): string {
  if (kuta === "GrahaMaitri") return t(`grahaName.${id}`);
  if (kuta === "Bhakoot") return (SIGNS as readonly string[]).includes(id) ? t(`sign.${signIndex(id as (typeof SIGNS)[number])}`) : id;
  return t(`compat.class.${kuta}.${id}`);
}

export function CompatPanel() {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <Compat opened={opened} />;
}

function Compat({ opened }: { opened: OpenedClient }) {
  const { t } = useTranslation();
  const compat = useCompat(opened);
  const phase = usePanelLoadPhase(compat.loading);
  const [picking, setPicking] = useState(false);
  const pickButton = useRef<HTMLButtonElement>(null);
  const changeButton = useRef<HTMLButtonElement>(null);
  // Odak kaybı: boş durumun düğmesi eş atanınca, araç çubuğunun düğmeleri eş çıkarılınca DOM'dan
  // gider. Eylem sonrası odağın gideceği düğme istenir; düğme ortaya çıkınca odak oraya alınır.
  const [wantFocus, setWantFocus] = useState<"pick" | "change" | null>(null);
  const { partner, report } = compat;
  const empty = partner === null && !compat.loading && compat.error === null;
  const linked = compat.partnerUuid !== null && (partner !== null || compat.error !== null);
  useEffect(() => {
    if (wantFocus === null || picking) return;
    const target = wantFocus === "pick" ? pickButton.current : changeButton.current;
    if (target === null) return;
    // Kullanıcı bu arada başka bir kontrole geçtiyse odağı çalma.
    const active = document.activeElement;
    if (active === null || active === document.body || active.matches("main[tabindex]")) target.focus({ preventScroll: true });
    setWantFocus(null);
  }, [wantFocus, picking, empty, partner]);
  const picker = picking && (
    <PartnerPicker
      openChartId={opened.chart_record.id}
      onClose={() => setPicking(false)}
      onPick={(chart, client) => {
        // Boş durumdan seçilirse tetikleyen düğme kalkar: eş okununca odak "Eşi değiştir"e gider.
        if (empty) setWantFocus("change");
        compat.setPartner(chart.uuid, client.gender);
      }}
    />
  );
  // Eş bağı yok (ya da eş harita silinmiş ve bağ kaldırılmış): tek eylemli boş durum.
  if (empty) {
    return (
      <div className="module-panel compat-panel">
        <p className="panel-message">{t("compat.empty")}</p>
        <div>
          <button ref={pickButton} type="button" onClick={() => setPicking(true)}>
            {t("compat.pick")}
          </button>
        </div>
        {picker}
      </div>
    );
  }
  return (
    <div className="module-panel compat-panel" aria-busy={compat.loading && partner !== null ? true : undefined}>
      {partner !== null && <People opened={opened} partner={partner} openRole={compat.openRole} />}
      {linked && (
        <div className="module-toolbar">
          {partner !== null && (
            <button type="button" onClick={compat.swapRoles}>
              {t("compat.swap")}
            </button>
          )}
          <button ref={changeButton} type="button" onClick={() => setPicking(true)}>
            {t("compat.change")}
          </button>
          <button
            type="button"
            onClick={() => {
              setWantFocus("pick");
              compat.clearPartner();
            }}
          >
            {t("compat.remove")}
          </button>
        </div>
      )}
      {compat.error ? (
        <PanelError error={compat.error} onRetry={compat.retry} />
      ) : report === null || partner === null ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : (
        <Report report={report} names={roleNames(opened, partner, compat.openRole)} settingsDiffer={JSON.stringify(opened.chart.meta.settings) !== JSON.stringify(partner.input.settings)} />
      )}
      {picker}
    </div>
  );
}

/** `[damat rolünün adı, gelin rolünün adı]`. */
function roleNames(opened: OpenedClient, partner: CompatPartner, openRole: KutaRole): [string, string] {
  return openRole === "Groom" ? [opened.client.name, partner.brief.client_name] : [partner.brief.client_name, opened.client.name];
}

/** Başlık: iki kişinin adı, haritası ve rolü; damat rolü önce. */
function People({ opened, partner, openRole }: { opened: OpenedClient; partner: CompatPartner; openRole: KutaRole }) {
  const { t } = useTranslation();
  const chartName = (label: string, kind: string) => label || t(`chart.kind.${kind}`);
  const open = { name: opened.client.name, chart: chartName(opened.chart_record.label, opened.chart_record.kind), unknown: timeUnknown(opened) };
  const other = { name: partner.brief.client_name, chart: chartName(partner.brief.chart_label, partner.brief.chart_kind), unknown: !partner.input.time_known };
  const people = openRole === "Groom" ? [open, other] : [other, open];
  return (
    <dl className="compat-people">
      {ROLES.map((role, i) => (
        <div key={role} data-role={role}>
          <dt>{t(`compat.role.${role}`)}</dt>
          <dd>
            <Approx on={people[i].unknown} label={t("compat.approximate")}>
              {t("compat.person", { name: people[i].name, chart: people[i].chart })}
            </Approx>
          </dd>
        </div>
      ))}
    </dl>
  );
}

function Report({ report, names, settingsDiffer }: { report: KutaReport; names: [string, string]; settingsDiffer: boolean }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const scoresId = useId();
  const overlayId = useId();
  const kujaValue = (v: boolean | null) => t(`compat.kujaValue.${v === null ? "unknown" : v ? "yes" : "no"}`);
  const kujaLine = (dosha: KujaDosha, name: string) =>
    t("compat.kuja", { name, lagna: kujaValue(dosha.from_lagna), moon: kujaValue(dosha.from_moon), venus: kujaValue(dosha.from_venus) });
  const scoreRow = (s: KutaScore) => (
    <tr key={s.kuta} data-kuta={s.kuta}>
      <th scope="row">{t(`compat.kutaName.${s.kuta}`)}</th>
      <td>{kutaClassLabel(t, s.kuta, s.groom_class)}</td>
      <td>{kutaClassLabel(t, s.kuta, s.bride_class)}</td>
      <td className="num">{t("compat.scoreOf", { score: formatKutaScore(s.score), max: formatKutaScore(s.max) })}</td>
    </tr>
  );
  // Karşılıklı yerleşim: ilk dokuz satır damat rolünün, son dokuzu gelin rolünün grahalarıdır.
  // Bilinmeyen ev çizgiyle çizilir; ekran okuyucuya hücrenin "bilinmiyor" diyen adı verilir.
  const cell = (row: OverlayRow) =>
    row.house_from_lagna === null ? (
      <span role="img" aria-label={t("compat.overlayCell", { lagna: kujaValue(null), moon: row.house_from_moon })}>
        {t("compat.overlayCell", { lagna: DASH, moon: row.house_from_moon })}
      </span>
    ) : (
      t("compat.overlayCell", { lagna: row.house_from_lagna, moon: row.house_from_moon })
    );
  return (
    <>
      {report.approximate && (
        <p className="panel-notice compat-approx" role="note">
          <span className="badge">
            <span className="badge-mark" aria-hidden="true">
              ≈
            </span>
            {t("compat.approximate")}
          </span>{" "}
          {t("compat.timeUnknown")}
        </p>
      )}
      {ROLES.map((role, i) => (
        <MoonChange key={role} name={names[i]} nakshatra={report.moon_day_change[i].nakshatra} sign={report.moon_day_change[i].sign} />
      ))}
      {settingsDiffer && (
        <p className="panel-notice" role="note">
          {t("compat.settingsDiffer")}
        </p>
      )}
      <BalaScroll labelledBy={scoresId}>
        <table className="module-table compat-table">
          <caption id={scoresId}>{t("compat.scoresTable")}</caption>
          <thead>
            <tr>
              <th scope="col">{t("compat.col.kuta")}</th>
              <th scope="col">{t("compat.col.classOf", { role: t("compat.role.Groom"), name: names[0] })}</th>
              <th scope="col">{t("compat.col.classOf", { role: t("compat.role.Bride"), name: names[1] })}</th>
              <th scope="col" className="num">
                {t("compat.col.score")}
              </th>
            </tr>
          </thead>
          <tbody>{report.scores.map(scoreRow)}</tbody>
          <tfoot>
            <tr>
              <th scope="row" colSpan={3}>
                {t("compat.total")}
              </th>
              <td className="num">{t("compat.scoreOf", { score: formatKutaScore(report.total), max: 36 })}</td>
            </tr>
          </tfoot>
        </table>
      </BalaScroll>
      {/* İstisna ve Kuja dosha olguları düz metin satırlarıdır; puanı değiştirmezler. */}
      <ul className="compat-facts" aria-label={t("compat.facts")}>
        {report.bhakoot_exception && <li data-fact="bhakoot">{t("compat.bhakootException")}</li>}
        {report.nadi_exception && <li data-fact="nadi">{t("compat.nadiException")}</li>}
        {ROLES.map((role, i) => (
          <li key={role} data-fact={`kuja-${role}`}>
            {kujaLine(report.kuja_dosha[i], names[i])}
          </li>
        ))}
      </ul>
      <BalaScroll labelledBy={overlayId}>
        <table className="module-table compat-table">
          <caption id={overlayId}>{t("compat.overlayTable")}</caption>
          <thead>
            <tr>
              <th scope="col">{t("compat.col.graha")}</th>
              <th scope="col">{t("compat.col.overlay", { owner: names[0], other: names[1] })}</th>
              <th scope="col">{t("compat.col.overlay", { owner: names[1], other: names[0] })}</th>
            </tr>
          </thead>
          <tbody>
            {GRAHA_IDS.map((g, i) => (
              <tr key={g} data-graha={g} className={active?.type === "graha" && active.id === g ? "is-highlighted" : undefined}>
                <th scope="row">
                  <PinButton target={{ type: "graha", id: g }} label={t(`grahaName.${g}`)}>
                    {t(`graha.${g}`)}
                  </PinButton>
                </th>
                <td>{cell(report.overlay[i])}</td>
                <td>{cell(report.overlay[i + 9])}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </BalaScroll>
      <p className="panel-message">{t("compat.note")}</p>
      <p className="panel-message">{t("compat.method", { method: report.method })}</p>
    </>
  );
}

/** Doğum saati bilinmeyen haritada Ay o gün nakshatra ya da burç değiştiriyorsa uyarı. */
function MoonChange({ name, nakshatra, sign }: { name: string; nakshatra: boolean; sign: boolean }) {
  const { t } = useTranslation();
  if (!nakshatra && !sign) return null;
  return (
    <p className="panel-notice compat-warning" role="note">
      {nakshatra && t("compat.moonNakshatra", { name })}
      {nakshatra && sign && " "}
      {sign && t("compat.moonSign", { name })}
    </p>
  );
}
