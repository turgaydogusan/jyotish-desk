// Transit oynatma denetimleri (yol haritası Plan 26, §4.10): oynat ya da duraklat ve hız seçici.
// Düğmenin adı durumu söyler ("Oynat" ya da "Duraklat"). Sistem "hareketi azalt" istiyorsa
// denetimler yerine nedeni yazan bir satır çizilir; adım düğmeleri çalışmayı sürdürür.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import { PLAY_SPEEDS, useTransitPlayback, type PlaySpeed } from "./useTransitPlayback";
import type { TransitStepper } from "./useTransitStepper";

export function TransitPlayback({ stepper }: { stepper: TransitStepper }) {
  const { t } = useTranslation();
  const playback = useTransitPlayback(stepper);
  const id = useId();
  if (!playback.available) {
    return (
      <p className="transit-stepper-note" role="note">
        {t("transitStep.reducedMotion")}
      </p>
    );
  }
  // Oynatma sürerken düğme hep kullanılabilir: duraklatmak her an mümkündür.
  const blocked = !stepper.ready && !playback.playing;
  return (
    <>
      <button type="button" data-playing={playback.playing ? "" : undefined} aria-disabled={blocked ? true : undefined} onClick={() => !blocked && playback.toggle()}>
        {t(playback.playing ? "transitStep.pause" : "transitStep.play")}
      </button>
      <label htmlFor={`${id}-speed`}>{t("transitStep.speed")}</label>
      <select id={`${id}-speed`} value={playback.speed} onChange={(e) => playback.setSpeed(e.target.value as PlaySpeed)}>
        {PLAY_SPEEDS.map((s) => (
          <option key={s} value={s}>
            {t(`transitStep.speeds.${s}`)}
          </option>
        ))}
      </select>
    </>
  );
}
