// "Yeni prashna" (ileri modüller spec'i §11): sağdan kayan modal pencere. An (varsayılan şimdi),
// yer (varsayılan bulunulan yer), isteğe bağlı sayı, soru metni ve odak evi. "Prashna kur"
// `create_prashna`'yı çağırır; dönen harita `session.open` ile açılır. An ve yer Rust'ta çözülür,
// lagna Rust'ta kurulur: burada hesap yoktur (C4). Soru metni, yer ve an loglanmaz.
// Değişmiş taslakta Esc onay sorar; "Vazgeç" düğmesi sormadan kapatır.
import { useEffect, useId, useRef, useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CommandError } from "../bindings";
import "../birth/birth.css";
import { PlaceSearch } from "../birth/PlaceSearch";
import { errorMessage } from "../i18n/errors";
import { hitPlace, placeLabel } from "../modules/muhurta";
import { tzLabel } from "../panels/HomePlaceDialog";
import { useSession } from "../session/store";
import { usePrefs } from "../settings/prefs";
import { ConfirmDialog } from "../shell/ConfirmDialog";
import { Dialog } from "../shell/Dialog";
import { draftPlace, emptyDraft, errorField, MAX_QUESTION_CHARS, PRASHNA_NUMBER_MAX, toRequest, type DraftErrors, type PrashnaDraft, type PrashnaField, type PrashnaPlaceSource } from "./model";
import "./prashna.css";

const HOUSES = Array.from({ length: 12 }, (_, i) => i + 1);

export interface PrashnaFormProps {
  onClose(): void;
  /** Harita kurulup açıldıktan sonra (ör. Prashna düzenine geçmek için). */
  onCreated?(): void;
}

export function PrashnaForm({ onClose, onCreated }: PrashnaFormProps) {
  const { t, i18n } = useTranslation();
  const session = useSession();
  const { prefs } = usePrefs();
  const id = useId();
  // Form, açıldığı andaki danışana yazar: form açıkken geç gelen bir `open_client` yanıtı açık
  // haritayı değiştirse de prashna başka danışana gitmez.
  const [target] = useState(() => session.opened);
  const [start] = useState<PrashnaDraft>(() => emptyDraft(target));
  const [draft, setDraft] = useState<PrashnaDraft>(start);
  const [searchText, setSearchText] = useState("");
  const dirty = JSON.stringify(draft) !== JSON.stringify(start);
  const [confirmClose, setConfirmClose] = useState(false);
  const [shown, setShown] = useState<DraftErrors>({});
  const [serverField, setServerField] = useState<PrashnaField | null>(null);
  const [formError, setFormError] = useState<CommandError | null>(null);
  const [saving, setSaving] = useState(false);
  const formRef = useRef<HTMLFormElement>(null);
  const focusInvalid = useRef(false);

  const place = draftPlace(draft, prefs, target);
  const update = (patch: Partial<PrashnaDraft>, field?: PrashnaField) => {
    setDraft((d) => ({ ...d, ...patch }));
    // Alan düzeltilince hatası hemen kalkar.
    if (field) {
      setShown((s) => ({ ...s, [field]: undefined }));
      setServerField((f) => (f === field ? null : f));
    }
  };
  const fieldError = (f: PrashnaField): string | null => {
    const key = shown[f] ?? (serverField === f ? `prashna.form.error.${f}` : undefined);
    return key ? t(key) : null;
  };
  const errorCount = (["at", "place", "number", "question", "focus_house"] as const).filter((f) => fieldError(f) !== null).length;

  // Gönderimden sonra odak ilk hatalı alana gider.
  useEffect(() => {
    if (!focusInvalid.current) return;
    focusInvalid.current = false;
    formRef.current?.querySelector<HTMLElement>('[aria-invalid="true"]')?.focus();
  });

  const requestClose = () => (dirty ? setConfirmClose(true) : onClose());

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    if (saving) return;
    setFormError(null);
    setServerField(null);
    const checked = toRequest(draft, prefs, target);
    if (checked.request === null) {
      setShown(checked.errors);
      focusInvalid.current = true;
      return;
    }
    setShown({});
    setSaving(true);
    const r = await api.createPrashna(checked.request);
    setSaving(false);
    if (r.status === "ok") {
      session.open(r.data);
      onCreated?.();
      onClose();
      return;
    }
    const f = errorField(r.error);
    if (f) {
      setServerField(f);
      focusInvalid.current = true;
    } else setFormError(r.error);
  };

  const describe = (f: PrashnaField, hint?: string) => {
    const error = fieldError(f);
    return {
      "aria-invalid": error ? (true as const) : undefined,
      "aria-describedby": [error ? `${id}-${f}-error` : null, hint ? `${id}-${hint}` : null].filter(Boolean).join(" ") || undefined,
    };
  };
  const errorLine = (f: PrashnaField) => {
    const error = fieldError(f);
    return (
      error && (
        <p id={`${id}-${f}-error`} className="field-error">
          <span aria-hidden="true">▲ </span>
          {error}
        </p>
      )
    );
  };
  const sources: PrashnaPlaceSource[] = target ? ["home", "chart", "search"] : ["home", "search"];
  const questionLength = [...draft.question.trim()].length;

  return (
    <Dialog title={t("prashna.form.title")} onClose={requestClose} variant="sheet">
      <form ref={formRef} className="birth-form prashna-form" noValidate onSubmit={(e) => void submit(e)}>
        <fieldset className="form-field">
          <legend>{t("prashna.form.moment")}</legend>
          {(["now", "local"] as const).map((m, i) => (
            <label key={m} className="choice">
              <input type="radio" name={`${id}-moment`} checked={draft.moment === m} data-autofocus={i === 0 ? true : undefined} {...(m === "now" && draft.moment === "now" ? describe("at") : {})} onChange={() => update({ moment: m }, "at")} />
              {t(`prashna.form.moment_${m}`)}
            </label>
          ))}
        </fieldset>
        {/* "Şimdi" iken `at` hatası (saat okunamadı vb.) tarih alanı olmadığı için burada görünür. */}
        {draft.moment === "now" && errorLine("at")}
        {draft.moment === "local" && (
          <>
            <div className="form-row">
              <div className="form-field">
                <label htmlFor={`${id}-date`}>{t("prashna.form.date")}</label>
                <input id={`${id}-date`} type="date" value={draft.date} {...describe("at", "at-hint")} onChange={(e) => update({ date: e.target.value }, "at")} />
              </div>
              <div className="form-field">
                <label htmlFor={`${id}-time`}>{t("prashna.form.time")}</label>
                <input id={`${id}-time`} type="time" value={draft.time} {...describe("at", "at-hint")} onChange={(e) => update({ time: e.target.value }, "at")} />
              </div>
            </div>
            <p id={`${id}-at-hint`} className="field-hint">
              {t("prashna.form.atHint")}
            </p>
            {errorLine("at")}
          </>
        )}
        <div className="form-field">
          <label htmlFor={`${id}-source`}>{t("prashna.form.place")}</label>
          <select id={`${id}-source`} value={draft.source} {...describe("place", "place-current")} onChange={(e) => update({ source: e.target.value as PrashnaPlaceSource }, "place")}>
            {sources.map((s) => (
              <option key={s} value={s}>
                {t(`prashna.form.source_${s}`)}
              </option>
            ))}
          </select>
          {draft.source === "search" && (
            <>
              <label htmlFor={`${id}-search`}>{t("prashna.form.search")}</label>
              <PlaceSearch
                inputId={`${id}-search`}
                value={searchText}
                onChange={setSearchText}
                onSelect={(hit) => {
                  const found = hitPlace(hit);
                  if (found) {
                    update({ searched: found }, "place");
                    setSearchText("");
                  }
                }}
              />
            </>
          )}
          <p id={`${id}-place-current`} className="prashna-place" role="status">
            {place
              ? t("prashna.form.placeCurrent", { place: placeLabel(place), zone: tzLabel(place, t("muhurta.home.lmt")) })
              : t(`prashna.form.placeMissing_${draft.source}`)}
          </p>
          {errorLine("place")}
        </div>
        <div className="form-field">
          <label htmlFor={`${id}-number`}>{t("prashna.form.number")}</label>
          <input id={`${id}-number`} inputMode="numeric" value={draft.number} {...describe("number", "number-hint")} onChange={(e) => update({ number: e.target.value }, "number")} />
          <p id={`${id}-number-hint`} className="field-hint">
            {t("prashna.form.numberHint", { max: PRASHNA_NUMBER_MAX })}
          </p>
          {errorLine("number")}
        </div>
        <div className="form-field">
          <label htmlFor={`${id}-question`}>{t("prashna.form.question")}</label>
          <textarea id={`${id}-question`} rows={4} value={draft.question} {...describe("question", "question-hint")} onChange={(e) => update({ question: e.target.value }, "question")} />
          <p id={`${id}-question-hint`} className="field-hint">
            {t("prashna.form.questionHint", { count: questionLength, max: MAX_QUESTION_CHARS })}
          </p>
          {errorLine("question")}
        </div>
        <div className="form-field">
          <label htmlFor={`${id}-focus`}>{t("prashna.form.focus")}</label>
          <select id={`${id}-focus`} value={draft.focusHouse} {...describe("focus_house", "focus-hint")} onChange={(e) => update({ focusHouse: e.target.value }, "focus_house")}>
            <option value="">{t("prashna.focusNone")}</option>
            {HOUSES.map((h) => (
              <option key={h} value={h}>
                {t("prashna.focusHouse", { house: h })}
              </option>
            ))}
          </select>
          <p id={`${id}-focus-hint`} className="field-hint">
            {t("prashna.form.focusHint")}
          </p>
          {errorLine("focus_house")}
        </div>
        {target ? (
          <div className="form-field">
            <label className="choice">
              <input type="checkbox" checked={draft.toOpenClient} aria-describedby={`${id}-client-hint`} onChange={(e) => update({ toOpenClient: e.target.checked })} />
              {t("prashna.form.toOpenClient", { name: target.client.name })}
            </label>
            <p id={`${id}-client-hint`} className="field-hint">
              {t("prashna.form.bookHint")}
            </p>
          </div>
        ) : (
          <p className="field-hint">{t("prashna.form.bookOnly")}</p>
        )}
        <p className="visually-hidden" role="status">
          {errorCount > 0 ? t("birth.errorSummary", { count: errorCount }) : ""}
        </p>
        {formError && (
          <p className="field-error" role="alert">
            {t("prashna.form.failed", { message: errorMessage(t, i18n, formError) })}
          </p>
        )}
        <div className="dialog-actions">
          <button type="button" onClick={onClose}>
            {t("birth.cancel")}
          </button>
          <button type="submit" disabled={saving}>
            {t("prashna.form.submit")}
          </button>
        </div>
      </form>
      {confirmClose && (
        <ConfirmDialog
          title={t("birth.discardTitle")}
          message={t("birth.discardMessage")}
          confirmLabel={t("birth.discardConfirm")}
          onConfirm={onClose}
          onCancel={() => setConfirmClose(false)}
        />
      )}
    </Dialog>
  );
}
