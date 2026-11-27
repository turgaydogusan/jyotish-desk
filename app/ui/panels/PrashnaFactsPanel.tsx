// Prashna olguları paneli (ileri modüller spec'i §11): soru metni (düzenlenebilir), odak evi
// seçimi ve soru haritasının olguları: lagna, lagna lordu ve odak evinin lordu, iki lordun bağı,
// Ay, kendralardaki ve odak evindeki grahalar, anın panchangı ve yoga listesi. Değerler
// `compute_prashna_facts`'ten ve açık haritadan gelir; burada hesap yoktur (C4). Hüküm, konu
// çıkarımı ve evet/hayır sonucu yoktur: panel olguları sayar.
//
// Soru metni not gövdesi gibi hassastır: loglanmaz. Soru ve odak evi otomatik kaydedilir
// (`update_prashna`); olgular yeniden okunurken son okunan olgular ve odak yerinde durur.
import { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CommandError, FocusFacts, LagnaFacts, LordFacts, Occupant, OpenedClient, PrashnaFacts, PrashnaMeta } from "../bindings";
import { signIndex } from "../chart/adapter";
import { useHighlight } from "../highlight/store";
import { errorMessage } from "../i18n/errors";
import { useLastData } from "../modules/useModule";
import { usePrashnaFacts } from "../modules/usePrashnaFacts";
import { MAX_QUESTION_CHARS } from "../prashna/model";
import "../prashna/prashna.css";
import { useSession } from "../session/store";
import { formatDegree } from "./format";
import { nakshatraName, RISING_NAMES, tithiInPaksha, tithiName } from "./labels";
import { panchangRows } from "./PanchangPanel";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";
import { YogaList } from "./YogasPanel";

/** Soru metni yazıldıktan bu kadar sonra kaydedilir. */
export const QUESTION_SAVE_MS = 600;
const HOUSES = Array.from({ length: 12 }, (_, i) => i + 1);
/** Anın panchangından gösterilen satırlar (gün doğumu ve batımı panchang panelindedir). */
const MOMENT_ROWS = ["tithi", "vara", "nakshatra", "yoga", "karana"] as const;

export function PrashnaFactsPanel(_: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  if (opened.prashna === null) {
    return (
      <p className="panel-notice" role="note">
        {t("prashna.notPrashna")}
      </p>
    );
  }
  // Başka bir prashna açılınca taslak ve kayıt durumu sıfırlanır.
  return <Prashna key={opened.chart_record.id} opened={opened} meta={opened.prashna} />;
}

function Prashna({ opened, meta }: { opened: OpenedClient; meta: PrashnaMeta }) {
  const { t, i18n } = useTranslation();
  const session = useSession();
  const id = useId();
  const chartId = opened.chart_record.id;
  const [question, setQuestion] = useState(meta.question);
  // Seçilen odak evi: kayıt sürerken ve kayıt düşse de seçim budur. Her kayıt bunu taşır;
  // böylece sırada bekleyen bir soru kaydı yeni seçilen evi eskisiyle ezmez.
  const [focus, setFocus] = useState(meta.focus_house);
  const [saveError, setSaveError] = useState<CommandError | null>(null);
  const [saved, setSaved] = useState(false);
  // Kayıtlar sırayla gider: eski metin yenisinin üstüne yazılmaz.
  const chain = useRef<Promise<unknown>>(Promise.resolve());
  const latest = useRef({ question, focus, stored: meta.question });
  latest.current = { question, focus, stored: meta.question };
  const { setPrashna } = session;

  const save = (text: string, focus: number | null) => {
    const run = chain.current.then(async () => {
      const r = await api.updatePrashna(chartId, { question: text, focus_house: focus });
      if (r.status === "error") {
        setSaved(false);
        setSaveError(r.error);
        return;
      }
      setSaveError(null);
      setSaved(true);
      setPrashna(chartId, r.data);
    });
    chain.current = run;
    return run;
  };
  const saveRef = useRef(save);
  saveRef.current = save;

  // Otomatik kayıt: yazma durunca. Panel kapanırken bekleyen metin hemen gönderilir.
  const tooLong = [...question.trim()].length > MAX_QUESTION_CHARS;
  useEffect(() => {
    if (question.trim() === meta.question || tooLong) return;
    const timer = setTimeout(() => void saveRef.current(question, latest.current.focus), QUESTION_SAVE_MS);
    return () => clearTimeout(timer);
  }, [question, meta.question, tooLong]);
  useEffect(
    () => () => {
      const now = latest.current;
      if (now.question.trim() !== now.stored && [...now.question.trim()].length <= MAX_QUESTION_CHARS) void saveRef.current(now.question, now.focus);
    },
    [],
  );

  // Çok uzun taslak gönderilmez: odak evi kaydı ve yeniden deneme o sırada kayıtlı soruyla gider.
  const sendable = tooLong ? meta.question : question;
  const module = usePrashnaFacts(opened, focus);
  const facts = useLastData(module, String(chartId));
  const phase = usePanelLoadPhase(module.loading);
  const length = [...question.trim()].length;

  return (
    <div className="module-panel prashna-facts" aria-busy={module.loading && facts !== null ? true : undefined}>
      <div className="prashna-question">
        <label htmlFor={`${id}-question`}>{t("prashna.question")}</label>
        <textarea
          id={`${id}-question`}
          rows={3}
          value={question}
          aria-invalid={tooLong ? true : undefined}
          aria-describedby={`${id}-question-status`}
          onChange={(e) => {
            setSaved(false);
            setQuestion(e.target.value);
          }}
        />
        <p id={`${id}-question-status`} className="panel-message" role="status">
          {tooLong
            ? t("prashna.questionTooLong", { count: length, max: MAX_QUESTION_CHARS })
            : saved && question.trim() === meta.question
              ? t("prashna.questionSaved")
              : t("prashna.questionHint", { count: length, max: MAX_QUESTION_CHARS })}
        </p>
        {saveError && (
          <p className="panel-error" role="alert">
            {t("prashna.saveFailed", { message: errorMessage(t, i18n, saveError) })}{" "}
            <button type="button" onClick={() => void save(sendable, focus)}>
              {t("panel.retry")}
            </button>
          </p>
        )}
      </div>
      <div className="module-toolbar">
        <label htmlFor={`${id}-focus`}>{t("prashna.focus")}</label>
        <select
          id={`${id}-focus`}
          value={focus ?? ""}
          onChange={(e) => {
            const chosen = e.target.value === "" ? null : Number(e.target.value);
            setFocus(chosen);
            void save(sendable, chosen);
          }}
        >
          <option value="">{t("prashna.focusNone")}</option>
          {HOUSES.map((h) => (
            <option key={h} value={h}>
              {t("prashna.focusHouse", { house: h })}
            </option>
          ))}
        </select>
        {meta.number !== null && <span className="prashna-number">{t("prashna.numberBadge", { number: meta.number })}</span>}
      </div>
      {module.error ? (
        <PanelError error={module.error} onRetry={module.retry} />
      ) : facts === null ? (
        phase === "skeleton" ? (
          <PanelSkeleton />
        ) : null
      ) : (
        <Facts facts={facts} opened={opened} meta={meta} />
      )}
    </div>
  );
}

function Occupants({ list }: { list: Occupant[] }) {
  const { t } = useTranslation();
  if (list.length === 0) return <>{t("prashna.empty")}</>;
  return (
    <>
      {list.map((o, i) => (
        <span key={o.graha}>
          {i > 0 && ", "}
          <PinButton target={{ type: "graha", id: o.graha }}>{t(`grahaName.${o.graha}`)}</PinButton> ({t(`prashna.nature.${o.nature}`)})
        </span>
      ))}
    </>
  );
}

function LordRow({ role, lord, on }: { role: string; lord: LordFacts; on: boolean }) {
  const { t } = useTranslation();
  const sign = signIndex(lord.sign);
  return (
    <tr data-lord={role} className={on ? "is-highlighted" : undefined}>
      <th scope="row">{t(`prashna.lord.${role}`)}</th>
      <td>
        <PinButton target={{ type: "graha", id: lord.graha }}>{t(`grahaName.${lord.graha}`)}</PinButton>
      </td>
      <td>
        <PinButton target={{ type: "sign", sign }}>{t(`sign.${sign}`)}</PinButton>
      </td>
      <td className="num">{lord.house}</td>
      <td>{lord.dignity ? t(`dignity.${lord.dignity}`) : "—"}</td>
      <td>{[lord.retrograde ? t("prashna.retrograde") : null, lord.combust ? t("prashna.combust") : null].filter(Boolean).join(", ") || "—"}</td>
    </tr>
  );
}

function Facts({ facts, opened, meta }: { facts: PrashnaFacts; opened: OpenedClient; meta: PrashnaMeta }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const lordsId = useId();
  const housesId = useId();
  const yogasId = useId();
  const { lagna, focus, moon } = facts;
  const isGraha = (g: string) => active?.type === "graha" && active.id === g;
  const moonSign = signIndex(moon.sign);
  const moment = panchangRows(t, opened.chart.panchang, opened.chart_record.resolved.offset_secs).filter((r) => (MOMENT_ROWS as readonly string[]).includes(r.key));

  return (
    <>
      {lagna === null ? (
        <p className="panel-notice" role="note">
          {t("prashna.timeUnknown")}
        </p>
      ) : (
        <LagnaSection lagna={lagna} focus={focus} opened={opened} meta={meta} lordsId={lordsId} housesId={housesId} isGraha={isGraha} />
      )}
      <section className="prashna-section" aria-label={t("prashna.moon.title")}>
        <h3>{t("prashna.moon.title")}</h3>
        <dl className={`prashna-list${isGraha("Moon") ? " is-highlighted" : ""}`}>
          <dt>{t("table.sign")}</dt>
          <dd>
            <PinButton target={{ type: "sign", sign: moonSign }}>{t(`sign.${moonSign}`)}</PinButton>
            {moon.house !== null && <> · {t("prashna.focusHouse", { house: moon.house })}</>}
          </dd>
          <dt>{t("panchang.nakshatra")}</dt>
          <dd>
            {nakshatraName(moon.nakshatra)} · {t("panchang.pada", { pada: moon.pada })}
          </dd>
          <dt>{t("panchang.tithi")}</dt>
          <dd>
            {moon.paksha} {tithiInPaksha(moon.tithi)} · {tithiName(moon.tithi)}
          </dd>
          <dt>{t("prashna.moon.aspectedBy")}</dt>
          <dd>
            {moon.aspected_by.length === 0
              ? t("prashna.none")
              : moon.aspected_by.map((g, i) => (
                  <span key={g}>
                    {i > 0 && ", "}
                    <PinButton target={{ type: "graha", id: g }}>{t(`grahaName.${g}`)}</PinButton>
                  </span>
                ))}
          </dd>
        </dl>
      </section>
      <section className="prashna-section" aria-label={t("prashna.moment.title")}>
        <h3>{t("prashna.moment.title")}</h3>
        <dl className="prashna-list">
          {moment.map((r) => (
            <div key={r.key} data-limb={r.key}>
              <dt>{r.label}</dt>
              <dd>{r.value}</dd>
            </div>
          ))}
        </dl>
      </section>
      <section className="prashna-section" aria-labelledby={yogasId}>
        <h3 id={yogasId}>{t("prashna.yogas")}</h3>
        <YogaList report={facts.yogas} filter="all" opened={opened} />
      </section>
      <p className="panel-message">{t("prashna.note")}</p>
      <p className="panel-message">{t("prashna.method", { method: facts.method })}</p>
    </>
  );
}

function LagnaSection({
  lagna,
  focus,
  opened,
  meta,
  lordsId,
  housesId,
  isGraha,
}: {
  lagna: LagnaFacts;
  focus: FocusFacts | null;
  opened: OpenedClient;
  meta: PrashnaMeta;
  lordsId: string;
  housesId: string;
  isGraha(g: string): boolean;
}) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const sign = signIndex(lagna.sign);
  const lagnaIndex = sign;
  // Ev vurgusu D1 evidir: lagna burcundan sayılan burç (gösterim eşlemesi, hesap değil).
  const activeSign = active?.type === "sign" ? active.sign : active?.type === "house" ? (lagnaIndex + active.house - 1) % 12 : null;
  const houses = [...lagna.kendras.map((k) => ({ ...k, focus: false })), ...(focus ? [{ house: focus.house, sign: focus.sign, grahas: focus.occupants, focus: true }] : [])];
  const rel = focus?.relation ?? null;

  return (
    <>
      <section className="prashna-section" aria-label={t("prashna.lagna.title")}>
        <h3>{t("prashna.lagna.title")}</h3>
        <dl className="prashna-list">
          <dt>{t("table.sign")}</dt>
          <dd>
            <PinButton target={{ type: "sign", sign }}>{t(`sign.${sign}`)}</PinButton> {formatDegree(opened.chart.lagna.deg_in_sign ?? 0)}
            {meta.number !== null && <> · {t("prashna.lagna.navamsaMiddle", { number: meta.number })}</>}
          </dd>
          <dt>{t("prashna.lagna.kind")}</dt>
          <dd>{t(`prashna.kind.${lagna.kind}`)}</dd>
          <dt>{t("prashna.lagna.rising")}</dt>
          <dd>
            {RISING_NAMES[lagna.rising]} · {t(`prashna.rising.${lagna.rising}`)}
          </dd>
        </dl>
      </section>
      <section className="prashna-section" aria-label={t("prashna.lords.title")}>
        <h3 id={lordsId}>{t("prashna.lords.title")}</h3>
        <div className="bala-scroll" role="group" tabIndex={0} aria-labelledby={lordsId}>
          <table className="module-table">
            <caption className="visually-hidden">{t("prashna.lords.caption")}</caption>
            <thead>
              <tr>
                <th scope="col">{t("prashna.lords.role")}</th>
                <th scope="col">{t("table.graha")}</th>
                <th scope="col">{t("table.sign")}</th>
                <th scope="col" className="num">
                  {t("prashna.lords.house")}
                </th>
                <th scope="col">{t("prashna.lords.dignity")}</th>
                <th scope="col">{t("prashna.lords.state")}</th>
              </tr>
            </thead>
            <tbody>
              <LordRow role="lagna" lord={lagna.lord} on={isGraha(lagna.lord.graha)} />
              {focus && <LordRow role="focus" lord={focus.lord} on={isGraha(focus.lord.graha)} />}
            </tbody>
          </table>
        </div>
        {focus === null ? (
          <p className="panel-message">{t("prashna.noFocus")}</p>
        ) : rel?.same_graha ? (
          <p className="prashna-relation" data-relation="same">
            {t("prashna.relation.same", { house: focus.house, graha: t(`grahaName.${focus.lord.graha}`) })}
          </p>
        ) : (
          rel && (
            <>
              <p className="prashna-relation-label" id={`${lordsId}-links`}>
                {t("prashna.relation.title")}
              </p>
              <ul className="prashna-links" aria-labelledby={`${lordsId}-links`}>
                {rel.links.length === 0 ? (
                  <li data-link="none">{t("prashna.relation.none")}</li>
                ) : (
                  rel.links.map((link) => (
                    <li key={link} data-link={link}>
                      {t(`prashna.link.${link}`)}
                    </li>
                  ))
                )}
              </ul>
              <p className="panel-message">{t("prashna.relation.counts", { forward: rel.focus_lord_from_lagna_lord, backward: rel.lagna_lord_from_focus_lord })}</p>
            </>
          )
        )}
      </section>
      <section className="prashna-section" aria-label={t("prashna.houses.title")}>
        <h3 id={housesId}>{t("prashna.houses.title")}</h3>
        <div className="bala-scroll" role="group" tabIndex={0} aria-labelledby={housesId}>
          <table className="module-table">
            <caption className="visually-hidden">{t("prashna.houses.caption")}</caption>
            <thead>
              <tr>
                <th scope="col">{t("prashna.houses.house")}</th>
                <th scope="col">{t("table.sign")}</th>
                <th scope="col">{t("prashna.houses.grahas")}</th>
              </tr>
            </thead>
            <tbody>
              {houses.map((h) => {
                const s = signIndex(h.sign);
                const on = activeSign === s || h.grahas.some((o) => isGraha(o.graha));
                return (
                  <tr key={`${h.focus ? "focus" : "kendra"}:${h.house}`} data-house={h.house} data-focus={h.focus ? "true" : undefined} className={on ? "is-highlighted" : undefined}>
                    <th scope="row">
                      {t("prashna.focusHouse", { house: h.house })}
                      {h.focus && <> · {t("prashna.houses.focus")}</>}
                    </th>
                    <td>
                      <PinButton target={{ type: "sign", sign: s }}>{t(`sign.${s}`)}</PinButton>
                    </td>
                    <td>
                      <Occupants list={h.grahas} />
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </section>
    </>
  );
}
