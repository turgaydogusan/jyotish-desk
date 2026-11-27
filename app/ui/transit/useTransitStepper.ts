// Transit anını adım adım yürüten kanca (yol haritası Plan 26, §4.10). Adım aritmetiği Rust'tadır
// (`transit_step`: ay sonu, yaz saati); burada tarih hesabı yoktur (C4). Adımın çıkış anı transit
// deposunun anıdır: an bir Jülyen günüyse o, değilse ("şimdi", duvar saati) gösterilen yanıtın
// anı. Varılan an depoya `{ Jd }` olarak yazılır; halka ve tablo depodan birlikte güncellenir.
//
// Eşzamanlılık: bir adım isteği sürerken yeni adım başlatılmaz. İstek sürerken an başka bir
// yoldan değişirse (Şimdi, tarih kutusu, dasa şeridi), harita değişirse ya da bileşen kalkarsa
// geç gelen yanıt düşürülür. An ve dilim loglanmaz.
import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "../api";
import type { CommandError, StepUnit, TransitAt } from "../bindings";
import { useSession } from "../session/store";
import { browserTimeZone, useTransit, type Transit } from "./store";

/** Rust `StepUnit::ALL`. */
export const STEP_UNITS = ["Hour", "Day", "Week", "Month", "Year"] as const satisfies readonly StepUnit[];
export const DEFAULT_STEP_UNIT: StepUnit = "Day";

export interface TransitStepper {
  unit: StepUnit;
  setUnit(unit: StepUnit): void;
  /** Adımın çıkacağı an biliniyor. */
  ready: boolean;
  /** Bir adım isteği sürüyor. */
  busy: boolean;
  error: CommandError | null;
  /** `count` birim ileri (eksi: geri). An ilerlediyse `true`; düşen ya da başlamayan adım `false`. */
  step(count: number): Promise<boolean>;
  /** Hatayla biten son adımı yeniden dener. */
  retry(): void;
  /** Deponun kendisi: oynatma, yanıtın gelişini buradan izler. */
  transit: Transit;
}

function isJd(at: TransitAt): at is { Jd: number } {
  return typeof at === "object" && "Jd" in at && typeof at.Jd === "number";
}

/** Adımın çıkış anı; yanıt beklenirken ve an bir Jülyen günü değilken `null`. */
export function stepBase(transit: Pick<Transit, "at" | "view" | "loading">): number | null {
  if (isJd(transit.at)) return transit.at.Jd;
  if (transit.loading) return null;
  return transit.view?.snapshot.jd_ut ?? null;
}

export function useTransitStepper(): TransitStepper {
  const transit = useTransit();
  const { opened } = useSession();
  const chartId = opened?.chart_record.id ?? null;
  const [unit, setUnit] = useState<StepUnit>(DEFAULT_STEP_UNIT);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<CommandError | null>(null);
  const base = stepBase(transit);
  // Son çizimin değerleri: süren istek, yanıt geldiğinde güncel anı buradan okur.
  const latest = useRef({ base, unit, setAt: transit.setAt });
  latest.current = { base, unit, setAt: transit.setAt };
  const inFlight = useRef(false);
  const generation = useRef(0);
  const lastCount = useRef(1);

  // Başka harita ya da bileşenin kalkması: süren adımın yanıtı düşer.
  useEffect(() => {
    setError(null);
    setBusy(false);
    inFlight.current = false;
    return () => {
      generation.current += 1;
    };
  }, [chartId]);

  const step = useCallback(async (count: number): Promise<boolean> => {
    const from = latest.current.base;
    if (from === null || inFlight.current) return false;
    const mine = generation.current;
    inFlight.current = true;
    lastCount.current = count;
    setBusy(true);
    const r = await api.transitStep({ jd_ut: from, unit: latest.current.unit, count, tz: browserTimeZone() });
    if (generation.current !== mine) return false;
    inFlight.current = false;
    setBusy(false);
    if (r.status === "error") {
      setError(r.error);
      return false;
    }
    setError(null);
    // İstek sürerken an başka bir yoldan değişti: eski adım uygulanmaz.
    if (latest.current.base !== from || r.data.jd_ut === null) return false;
    latest.current.setAt({ Jd: r.data.jd_ut });
    return true;
  }, []);

  const retry = useCallback(() => {
    setError(null);
    void step(lastCount.current);
  }, [step]);

  return { unit, setUnit, ready: base !== null, busy, error, step, retry, transit };
}
