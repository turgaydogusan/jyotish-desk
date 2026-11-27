// Transit adım denetimleri (yol haritası Plan 26, §4.10): birim seçici, bir adım geri ve bir adım
// ileri düğmeleri, yanında oynatma denetimleri (`TransitPlayback`). Adımın hesabı Rust'tadır (`useTransitStepper`). Düğmeler kullanılamazken yerel
// `disabled` değildir: odakta durur, tıklama bir şey yapmaz. Hata yalnızca alan adını taşır; ad
// arayüz dilinde yazılır.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { CommandError, StepUnit } from "../bindings";
import { PanelError } from "../panels/states";
import "./stepper.css";
import { TransitPlayback } from "./TransitPlayback";
import { STEP_UNITS, useTransitStepper } from "./useTransitStepper";

/** `invalid_transit_input`'un alan adını arayüz dilinde yazar; öteki hatalar olduğu gibi durur. */
function fieldError(error: CommandError, name: (field: string) => string): CommandError {
  const field = error.params.field;
  if (error.code !== "invalid_transit_input" || typeof field !== "string") return error;
  return { ...error, params: { ...error.params, field: name(field) } };
}

export function TransitStepper() {
  const { t, i18n } = useTranslation();
  const stepper = useTransitStepper();
  const id = useId();
  const blocked = !stepper.ready;
  const shownError = stepper.error === null ? null : fieldError(stepper.error, (f) => (i18n.exists(`transitStep.field.${f}`) ? t(`transitStep.field.${f}`) : f));
  return (
    <>
      <div className="transit-stepper" role="group" aria-label={t("transitStep.group")}>
        <label htmlFor={`${id}-unit`}>{t("transitStep.unit")}</label>
        <select id={`${id}-unit`} value={stepper.unit} onChange={(e) => stepper.setUnit(e.target.value as StepUnit)}>
          {STEP_UNITS.map((u) => (
            <option key={u} value={u}>
              {t(`transitStep.units.${u}`)}
            </option>
          ))}
        </select>
        <button type="button" aria-disabled={blocked ? true : undefined} onClick={() => void stepper.step(-1)}>
          {t("transitStep.back")}
        </button>
        <button type="button" aria-disabled={blocked ? true : undefined} onClick={() => void stepper.step(1)}>
          {t("transitStep.forward")}
        </button>
        <TransitPlayback stepper={stepper} />
      </div>
      {shownError && <PanelError error={shownError} onRetry={stepper.retry} />}
    </>
  );
}
