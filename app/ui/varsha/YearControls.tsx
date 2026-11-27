// Varshaphala yıl seçicisi: önceki ve sonraki yıl, yaşın kendisi ve "bu yıl". Varshaphala paneli
// ve dasa ağacının mudda görünümü aynı denetimi kullanır. `age` gösterilen yaştır (seçili yaş ya
// da Rust'ın bulduğu süren yıl); henüz bilinmiyorsa `null`. Burada tarih hesabı yoktur (C4).
import { useId } from "react";
import { useTranslation } from "react-i18next";
import { isVarshaAge, MAX_VARSHA_AGE } from "./yearStore";

export function VarshaYearControls({ age, current, onChange }: { age: number | null; /** "Bu yıl" seçili. */ current: boolean; onChange(age: number | null): void }) {
  const { t } = useTranslation();
  const id = useId();
  const step = (delta: number) => {
    if (age !== null && isVarshaAge(age + delta)) onChange(age + delta);
  };
  const atMin = age === null || age <= 0;
  const atMax = age === null || age >= MAX_VARSHA_AGE;
  return (
    <>
      {/* Kapalı düğme yerel `disabled` değildir: odağı tutar, tıklamak yılı değiştirmez. */}
      <button type="button" aria-disabled={atMin || undefined} onClick={() => step(-1)}>
        {t("varsha.previous")}
      </button>
      <label htmlFor={`${id}-age`}>{t("varsha.age")}</label>
      <input
        id={`${id}-age`}
        className="varsha-age"
        type="number"
        inputMode="numeric"
        min={0}
        max={MAX_VARSHA_AGE}
        step={1}
        value={age ?? ""}
        onChange={(e) => {
          const next = Number(e.target.value);
          if (e.target.value.trim() !== "" && isVarshaAge(next)) onChange(next);
        }}
      />
      <button type="button" aria-disabled={atMax || undefined} onClick={() => step(1)}>
        {t("varsha.next")}
      </button>
      <button type="button" aria-pressed={current} onClick={() => onChange(null)}>
        {t("varsha.current")}
      </button>
    </>
  );
}