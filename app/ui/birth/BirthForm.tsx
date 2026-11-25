// Doğum verisi girişi (spec §4 "Yeni danışan/harita" ve "Düzenle"). Sağdan kayan modal
// pencere. Form biçimi `model.ts`'de denetlenir; an Rust'ta çözülür: alanlar değiştikçe
// 300 ms sonra `preview_moment` çağrılır (UTC, ofset, uyarılar, DST boşluğu/örtüşmesi).
// Kayıt `create_client` (yeni danışan), `add_chart` (açık danışana yeni harita) ya da
// `update_chart` (düzenle) ile yapılır; dönen harita `session.open` ile açılır (önceki haritanın
// bildirimleri orada kalkar, yeni haritanın store uyarıları gösterilir).
import { useEffect, useId, useRef, useState, type FormEvent, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CommandError, MomentPreview, OpenedClient } from "../bindings";
import { errorMessage } from "../i18n/errors";
import { useSession } from "../session/store";
import { Dialog } from "../shell/Dialog";
import "./birth.css";
import {
  applyPlace,
  applySuggestion,
  draftFromOpened,
  emptyDraft,
  errorField,
  formatFixedOffset,
  setCoordinate,
  toBirthData,
  toNewChart,
  toNewClient,
  type BirthDraft,
  type BirthField,
  type FieldErrors,
  type TzMode,
} from "./model";
import { PlaceSearch } from "./PlaceSearch";

export const PREVIEW_DEBOUNCE_MS = 300;

export interface BirthFormProps {
  /** `new`: yeni danışan; `add`: açık danışana yeni harita; `edit`: açık haritayı düzenle. */
  mode: "new" | "add" | "edit";
  onClose(): void;
}

function initialDraft(mode: BirthFormProps["mode"], opened: OpenedClient | null): BirthDraft {
  if (!opened || mode === "new") return emptyDraft();
  if (mode === "edit") return draftFromOpened(opened);
  return { ...emptyDraft(), name: opened.client.name, gender: opened.client.gender };
}

export function BirthForm({ mode, onClose }: BirthFormProps) {
  const { t, i18n } = useTranslation();
  const session = useSession();
  const id = useId();
  // Form, açıldığı andaki haritaya ve danışana yazar: form açıkken geç gelen bir `open_client`
  // yanıtı açık haritayı değiştirse de kayıt başka haritaya ya da danışana gitmez.
  const [target] = useState(() => session.opened);
  const editing = mode === "edit" && target !== null;
  const adding = mode === "add" && target !== null;
  const [draft, setDraft] = useState<BirthDraft>(() => initialDraft(mode, target));
  const [shown, setShown] = useState<FieldErrors>({});
  const [preview, setPreview] = useState<{ ok: MomentPreview } | { error: CommandError } | null>(null);
  const [formError, setFormError] = useState<CommandError | null>(null);
  const [saving, setSaving] = useState(false);
  const seq = useRef(0);

  // Önizleme: biçimi tamam olan her taslak için; eski yanıt yok sayılır.
  const checked = toBirthData(draft);
  const previewKey = checked.birth ? JSON.stringify(checked.birth) : null;
  useEffect(() => {
    const mine = ++seq.current;
    if (previewKey === null) {
      setPreview(null);
      return;
    }
    const timer = setTimeout(() => {
      void api.previewMoment(JSON.parse(previewKey)).then((r) => {
        if (mine !== seq.current) return;
        setPreview(r.status === "ok" ? { ok: r.data } : { error: r.error });
      });
    }, PREVIEW_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [previewKey]);

  const update = (patch: Partial<BirthDraft>, field?: BirthField) => {
    setDraft((d) => ({ ...d, ...patch }));
    // Alan düzeltilince hatası hemen kalkar (spec §7: blur-first, anında temizle).
    if (field) setShown((s) => ({ ...s, [field]: undefined }));
  };

  // Önizleme hatası alana bağlanır; biçim hatası yalnızca dokunulan/gönderilen alanda görünür.
  const previewError = preview && "error" in preview ? preview.error : null;
  const previewField = previewError ? errorField(previewError) : null;
  const fieldError = (f: BirthField): string | null => {
    const e = shown[f] ?? (previewField === f && previewError ? { error: previewError } : undefined);
    if (!e) return null;
    return "key" in e ? t(e.key) : errorMessage(t, i18n, e.error);
  };
  const blur = (f: BirthField) => () => {
    const e = toBirthData(draft).errors?.[f];
    if (e) setShown((s) => ({ ...s, [f]: e }));
  };

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    setFormError(null);
    const { birth, errors } = toBirthData(draft);
    if (!birth) {
      setShown(errors);
      return;
    }
    setSaving(true);
    const r =
      editing && target
        ? await api.updateChart(target.chart_record.id, toNewChart(draft, birth))
        : adding && target
          ? await api.addChart(target.client.id, toNewChart(draft, birth))
          : await api.createClient(toNewClient(draft, birth));
    setSaving(false);
    if (r.status === "ok") {
      session.open(r.data);
      onClose();
      return;
    }
    const f = errorField(r.error);
    if (f) setShown((s) => ({ ...s, [f]: { error: r.error } }));
    else setFormError(r.error);
  };

  const field = (f: BirthField, label: string, input: (p: { id: string; "aria-invalid"?: true; "aria-describedby"?: string }) => ReactNode, hint?: string) => {
    const error = fieldError(f);
    const inputId = `${id}-${f}`;
    const hintId = hint ? `${inputId}-hint` : undefined;
    const errorId = error ? `${inputId}-error` : undefined;
    return (
      <div className="form-field">
        <label htmlFor={inputId}>{label}</label>
        {input({ id: inputId, "aria-invalid": error ? true : undefined, "aria-describedby": [errorId, hintId].filter(Boolean).join(" ") || undefined })}
        {hint && (
          <p id={hintId} className="field-hint">
            {hint}
          </p>
        )}
        {error && (
          <p id={errorId} className="field-error">
            <span aria-hidden="true">▲ </span>
            {error}
          </p>
        )}
      </div>
    );
  };

  const suggestion = previewError?.code === "nonexistent_local_time" ? previewError.params.suggestion : null;
  const ambiguous = previewError?.code === "ambiguous_local_time" || draft.fold !== null;

  return (
    <Dialog title={editing ? t("birth.titleEdit") : adding ? t("birth.titleAdd") : t("birth.titleNew")} onClose={onClose} variant="sheet">
      <form className="birth-form" noValidate onSubmit={(e) => void submit(e)}>
        {field("name", t("birth.name"), (p) => (
          <input {...p} value={draft.name} disabled={editing || adding} data-autofocus onChange={(e) => update({ name: e.target.value }, "name")} onBlur={blur("name")} />
        ))}
        {field("label", t("birth.label"), (p) => (
          <input {...p} value={draft.label} onChange={(e) => update({ label: e.target.value }, "label")} />
        ))}
        <div className="form-field">
          <label htmlFor={`${id}-kind`}>{t("birth.kind")}</label>
          <select id={`${id}-kind`} value={draft.kind} onChange={(e) => update({ kind: e.target.value === "Event" ? "Event" : "Natal" })}>
            <option value="Natal">{t("chart.kind.Natal")}</option>
            <option value="Event">{t("chart.kind.Event")}</option>
          </select>
        </div>
        <div className="form-row">
          {field("date", t("birth.date"), (p) => (
            <input {...p} type="date" value={draft.date} onChange={(e) => update({ date: e.target.value }, "date")} onBlur={blur("date")} />
          ))}
          {field("time", t("birth.time"), (p) => (
            <input
              {...p}
              type="time"
              value={draft.time}
              disabled={draft.certainty === "Unknown"}
              onChange={(e) => update({ time: e.target.value, fold: null }, "time")}
              onBlur={blur("time")}
            />
          ))}
        </div>
        {suggestion && (
          <p className="field-fix">
            <button type="button" onClick={() => update(applySuggestion(draft, suggestion), "time")}>
              {t("birth.useSuggestion", { time: applySuggestion(draft, suggestion).time })}
            </button>
          </p>
        )}
        <fieldset className="form-field">
          <legend>{t("birth.certainty")}</legend>
          {(["Exact", "Approx", "Unknown"] as const).map((c) => (
            <label key={c} className="choice">
              <input type="radio" name={`${id}-certainty`} checked={draft.certainty === c} onChange={() => update({ certainty: c }, "time")} />
              {t(`birth.certainty${c}`)}
            </label>
          ))}
        </fieldset>
        {ambiguous && (
          <fieldset className="form-field" aria-invalid={previewField === "fold" || undefined}>
            <legend>{t("birth.fold")}</legend>
            {(["Earlier", "Later"] as const).map((f) => (
              <label key={f} className="choice">
                <input type="radio" name={`${id}-fold`} checked={draft.fold === f} onChange={() => update({ fold: f }, "fold")} />
                {t(`birth.fold${f}`)}
              </label>
            ))}
            {fieldError("fold") && <p className="field-error">{fieldError("fold")}</p>}
          </fieldset>
        )}
        <div className="form-field">
          <label htmlFor={`${id}-place`}>{t("birth.place")}</label>
          <PlaceSearch
            inputId={`${id}-place`}
            value={draft.placeName}
            onChange={(text) => update({ placeName: text, geonameId: null })}
            onSelect={(hit) => {
              setDraft((d) => applyPlace(d, hit));
              setShown((s) => ({ ...s, lat: undefined, lon: undefined, tz: undefined }));
            }}
          />
        </div>
        <div className="form-row">
          {field(
            "lat",
            t("birth.lat"),
            (p) => <input {...p} inputMode="decimal" value={draft.lat} onChange={(e) => update(setCoordinate(draft, "lat", e.target.value), "lat")} onBlur={blur("lat")} />,
            t("birth.coordHint"),
          )}
          {field("lon", t("birth.lon"), (p) => (
            <input {...p} inputMode="decimal" value={draft.lon} onChange={(e) => update(setCoordinate(draft, "lon", e.target.value), "lon")} onBlur={blur("lon")} />
          ))}
        </div>
        <div className="form-row">
          <div className="form-field">
            <label htmlFor={`${id}-tzmode`}>{t("birth.tzMode")}</label>
            <select
              id={`${id}-tzmode`}
              value={draft.tzMode}
              onChange={(e) => update({ tzMode: e.target.value as TzMode, fold: null }, "tz")}
            >
              <option value="iana">{t("birth.tzIana")}</option>
              <option value="fixed">{t("birth.tzFixed")}</option>
              <option value="lmt">{t("birth.tzLmt")}</option>
            </select>
          </div>
          {draft.tzMode === "iana" &&
            field("tz", t("birth.tzIanaName"), (p) => (
              <input {...p} value={draft.tzIana} placeholder="Europe/Istanbul" onChange={(e) => update({ tzIana: e.target.value, fold: null }, "tz")} onBlur={blur("tz")} />
            ))}
          {draft.tzMode === "fixed" &&
            field("tz", t("birth.tzFixedValue"), (p) => (
              <input {...p} value={draft.tzFixed} placeholder="+05:30" onChange={(e) => update({ tzFixed: e.target.value }, "tz")} onBlur={blur("tz")} />
            ))}
        </div>
        <div className="form-field">
          <label htmlFor={`${id}-notes`}>{t("birth.notes")}</label>
          <textarea id={`${id}-notes`} rows={2} value={draft.notes} onChange={(e) => update({ notes: e.target.value })} />
        </div>
        <section className="birth-preview" aria-label={t("birth.preview")} aria-live="polite">
          {preview && "ok" in preview && (
            <>
              <p>
                {t("birth.previewValue", {
                  utc: preview.ok.resolved.utc.replace("T", " ").replace("Z", ""),
                  abbrev: preview.ok.resolved.abbrev,
                  offset: formatFixedOffset(preview.ok.resolved.offset_secs),
                })}
              </p>
              {preview.ok.resolved.warnings.map((w) => (
                <p key={w} className="field-hint">
                  <span aria-hidden="true">▲ </span>
                  {t(`badgeHint.${w}`)}
                </p>
              ))}
            </>
          )}
          {previewError && previewField === null && <p className="field-error">{errorMessage(t, i18n, previewError)}</p>}
        </section>
        {formError && (
          <p className="field-error" role="alert">
            {t("birth.saveFailed", { message: errorMessage(t, i18n, formError) })}
          </p>
        )}
        <div className="dialog-actions">
          <button type="button" onClick={onClose}>
            {t("birth.cancel")}
          </button>
          <button type="submit" disabled={saving}>
            {t("birth.save")}
          </button>
        </div>
      </form>
    </Dialog>
  );
}
