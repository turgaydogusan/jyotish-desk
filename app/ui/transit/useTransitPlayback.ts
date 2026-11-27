// Transit anının oynatılması (yol haritası Plan 26, §4.10): seçili birimle, sabit aralıkla ileri
// adım. Oynatma yeni bir adımı ancak önceki an yerleşince planlar: adım isteği bitmiş ve o anın
// transitleri gelmiş olmalıdır. Böylece tek ephemeris iş parçacığında iş birikmez ve oynatma
// sürerken harita açmak beklemez. Zamanlayıcı bileşen kalkınca (panel kapanınca), harita
// değişince, bir adım ya da transit hatasında ve sistem "hareketi azalt" isteyince durur.
// "Hareketi azalt" açıkken oynatma sunulmaz; adım düğmeleri çalışır.
import { useCallback, useEffect, useState } from "react";
import { useSession } from "../session/store";
import type { TransitStepper } from "./useTransitStepper";

export const PLAY_SPEEDS = ["slow", "normal", "fast"] as const;
export type PlaySpeed = (typeof PLAY_SPEEDS)[number];
export const DEFAULT_PLAY_SPEED: PlaySpeed = "normal";
/** Bir an yerleştikten sonra sonraki adıma dek beklenen süre. */
export const PLAY_INTERVAL_MS: Record<PlaySpeed, number> = { slow: 2000, normal: 1000, fast: 400 };
const REDUCED_MOTION_QUERY = "(prefers-reduced-motion: reduce)";

function reducedMotionQuery(): MediaQueryList | null {
  return typeof window.matchMedia === "function" ? window.matchMedia(REDUCED_MOTION_QUERY) : null;
}

/** Sistem "hareketi azalt" istiyor; ayar değişince yeniden çizer. */
export function useReducedMotion(): boolean {
  const [reduced, setReduced] = useState(() => reducedMotionQuery()?.matches ?? false);
  useEffect(() => {
    const query = reducedMotionQuery();
    if (query === null) return;
    const onChange = () => setReduced(query.matches);
    onChange();
    query.addEventListener("change", onChange);
    return () => query.removeEventListener("change", onChange);
  }, []);
  return reduced;
}

export interface TransitPlayback {
  /** Oynatma sunuluyor ("hareketi azalt" kapalı). */
  available: boolean;
  playing: boolean;
  speed: PlaySpeed;
  setSpeed(speed: PlaySpeed): void;
  toggle(): void;
}

export function useTransitPlayback(stepper: TransitStepper): TransitPlayback {
  const reduced = useReducedMotion();
  const { opened } = useSession();
  const chartId = opened?.chart_record.id ?? null;
  const [playing, setPlaying] = useState(false);
  const [speed, setSpeed] = useState<PlaySpeed>(DEFAULT_PLAY_SPEED);
  const { step, transit } = stepper;
  // An yerleşti: çıkış anı biliniyor, adım isteği bitti ve o anın transitleri geldi.
  const settled = stepper.ready && !stepper.busy && !transit.loading;
  const failed = stepper.error !== null || transit.error !== null;
  const frame = transit.view?.snapshot.jd_ut ?? null;

  // Başka harita: oynatma durur.
  useEffect(() => setPlaying(false), [chartId]);
  // Hata ya da "hareketi azalt": oynatma durur.
  useEffect(() => {
    if (failed || reduced) setPlaying(false);
  }, [failed, reduced]);
  // Yerleşen her an için tek bir sonraki adım planlanır; temizlik zamanlayıcıyı kaldırır.
  useEffect(() => {
    if (!playing || !settled || failed || reduced) return;
    const timer = setTimeout(() => void step(1), PLAY_INTERVAL_MS[speed]);
    return () => clearTimeout(timer);
  }, [playing, settled, failed, reduced, frame, speed, step]);

  const toggle = useCallback(() => setPlaying((on) => !on), []);
  return { available: !reduced, playing: playing && !reduced, speed, setSpeed, toggle };
}
