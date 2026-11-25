// Tek not: tarih damgası, otomatik kaydedilen metin ve onaylı silme. Yazım durunca
// `NOTE_SAVE_DEBOUNCE_MS` sonra (kesintisiz yazımda en geç `NOTE_SAVE_MAX_WAIT_MS`'de bir), odak
// çıkınca, pencere gizlenince ve panel kapanınca hemen kaydedilir (C17). Not metni loglanmaz ve
// hata bildirimine konmaz.
import { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CommandError, Note } from "../bindings";
import { errorMessage } from "../i18n/errors";
import { ConfirmDialog } from "../shell/ConfirmDialog";
import { useNotices } from "../shell/notices";
import { enqueueNoteSave } from "./saves";

export const NOTE_SAVE_DEBOUNCE_MS = 500;
/** Kesintisiz yazımda iki kayıt arasındaki en uzun süre. */
export const NOTE_SAVE_MAX_WAIT_MS = 2000;
/** UTF-16 birimi; 16 384 × 3 bayt Rust `MAX_NOTE_BYTES` (65 536) altında kalır. Değişirse notes.atLimit metinleri de değişir. */
export const MAX_NOTE_UNITS = 16_384;

export interface NoteEditorProps {
  note: Note;
  /** Yeni eklenen not: metin kutusu odak alır. */
  autoFocus: boolean;
  /** Bu haritaya bağlı not (açık harita). */
  onThisChart: boolean;
  onSaved(note: Note): void;
  /** "Yeni not" ile eklendi ve henüz bu oturumda yazılmadı: boş bırakılırsa silinir (D17). */
  discardWhenEmpty: boolean;
  /** Onaylı silme; `false`: silinemedi (bildirim panelin işidir), not yerinde kalır. `quiet`: boş taze notun onaysız silinmesi, odak taşınmaz. */
  onDelete(note: Note, quiet?: boolean): Promise<boolean>;
}

export function NoteEditor({ note, autoFocus, onThisChart, discardWhenEmpty, onSaved, onDelete }: NoteEditorProps) {
  const { t, i18n } = useTranslation();
  const { push } = useNotices();
  const id = useId();
  const [body, setBody] = useState(note.body);
  const [error, setError] = useState<CommandError | null>(null);
  const [confirm, setConfirm] = useState(false);
  // `saved`: Rust'a gönderilmiş son metin; `null` son kayıt başarısız oldu demektir.
  // `dirtyAt`: kaydedilmemiş ilk değişikliğin zamanı (en uzun bekleme bundan sayılır).
  const latest = useRef<{
    body: string;
    saved: string | null;
    timer: ReturnType<typeof setTimeout> | null;
    dirtyAt: number | null;
    deleted: boolean;
  }>({
    body: note.body,
    saved: note.body,
    timer: null,
    dirtyAt: null,
    deleted: false,
  });
  const mounted = useRef(true);
  const box = useRef<HTMLTextAreaElement>(null);

  const flush = () => {
    const now = latest.current;
    if (now.timer) clearTimeout(now.timer);
    now.timer = null;
    now.dirtyAt = null;
    if (now.deleted || now.body === now.saved) return;
    const text = now.body;
    now.saved = text;
    void enqueueNoteSave(() => api.updateNote(note.id, text)).then((r) => {
      if (r.status === "ok") {
        setError(null);
        onSaved(r.data);
      } else {
        // Kaydedilmedi: sonraki yazım ya da odak çıkışı yeniden dener.
        if (latest.current.saved === text) latest.current.saved = null;
        // Panel kapandıktan sonra düşen kayıt sessiz kalmaz: bildirim bandında görünür.
        if (mounted.current) setError(r.error);
        else push({ id: "note-save", type: "error", messageKey: "notice.noteSaveLost", error: r.error });
      }
    });
  };

  // Boş taze not: odak çıkınca ya da panel kapanınca onaysız silinir. Silme sürüyorsa tekrar edilmez.
  const discardable = () => discardWhenEmpty && !latest.current.deleted && latest.current.body.trim() === "";
  const discard = () => {
    const now = latest.current;
    now.deleted = true;
    if (now.timer) clearTimeout(now.timer);
    now.timer = null;
    // Sessiz silme: odak kullanıcının gittiği yerde durur (onaylı silmeden farklı olarak).
    // Silinemezse not listede durur; `deleted` geri alınır ki sonraki yazım kaydedilsin.
    void onDelete(note, true).then((ok) => {
      if (!ok) now.deleted = false;
    });
  };
  const discardRef = useRef({ discardable, discard });
  discardRef.current = { discardable, discard };

  // Panel kapanırken, pencere gizlenirken ve uygulama kapanırken bekleyen metin yazılır
  // (görünüm deposuyla aynı olaylar; Tauri webview kapanışı `pagehide` atlayabilir).
  // `flush` yalnızca ref ve kararlı değerleri okur (not kimliği anahtardır).
  useEffect(() => {
    // StrictMode geliştirmede etkiyi kurar, temizler, yeniden kurar: bayrak her kurulumda açılır.
    mounted.current = true;
    const onVisibility = () => {
      if (document.visibilityState === "hidden") flush();
    };
    window.addEventListener("pagehide", flush);
    document.addEventListener("visibilitychange", onVisibility);
    return () => {
      window.removeEventListener("pagehide", flush);
      document.removeEventListener("visibilitychange", onVisibility);
      mounted.current = false;
      flush();
      // StrictMode kur-temizle-kur yapar: bir mikro görev sonra hâlâ bağlı değilse boş taze not silinir.
      queueMicrotask(() => {
        if (!mounted.current && discardRef.current.discardable()) discardRef.current.discard();
      });
    };
  }, []);

  useEffect(() => {
    if (autoFocus) box.current?.focus();
  }, [autoFocus]);

  const change = (text: string) => {
    setBody(text);
    const now = latest.current;
    now.body = text;
    if (now.timer) clearTimeout(now.timer);
    now.dirtyAt ??= Date.now();
    // Kesintisiz yazımda da en geç NOTE_SAVE_MAX_WAIT_MS'de bir kaydedilir.
    const wait = Math.min(NOTE_SAVE_DEBOUNCE_MS, now.dirtyAt + NOTE_SAVE_MAX_WAIT_MS - Date.now());
    now.timer = setTimeout(flush, Math.max(0, wait));
  };

  const remove = async () => {
    setConfirm(false);
    // Silme sürerken ikinci onay yok sayılır.
    if (latest.current.deleted) return;
    latest.current.deleted = true;
    if (latest.current.timer) clearTimeout(latest.current.timer);
    if (await onDelete(note)) return;
    // Silinemedi: not yerinde kalır; silme başlarken durdurulan kayıt beklemeden gönderilir.
    latest.current.deleted = false;
    flush();
  };

  const date = new Intl.DateTimeFormat(i18n.language, { dateStyle: "medium", timeStyle: "short" }).format(new Date(note.created_at));
  return (
    <li className="note">
      <div className="note-head">
        <label htmlFor={`${id}-body`} className="note-date">
          <time dateTime={note.created_at}>{date}</time>
          {onThisChart && <span className="note-chart">{t("notes.thisChart")}</span>}
        </label>
        <button type="button" className="icon-button" aria-label={t("notes.deleteNamed", { date })} onClick={() => setConfirm(true)}>
          {t("notes.delete")}
        </button>
      </div>
      <textarea
        id={`${id}-body`}
        ref={box}
        value={body}
        maxLength={MAX_NOTE_UNITS}
        rows={3}
        aria-describedby={[error ? `${id}-error` : null, body.length >= MAX_NOTE_UNITS ? `${id}-limit` : null].filter(Boolean).join(" ") || undefined}
        onChange={(e) => change(e.target.value)}
        onBlur={(e) => {
          // Odak notun kendi düğmesine (Sil) geçiyorsa boş taze not silinmez; kullanıcı orada.
          if (discardable() && !e.currentTarget.closest("li")?.contains(e.relatedTarget as Node | null)) discard();
          else flush();
        }}
      />
      {body.length >= MAX_NOTE_UNITS && (
        <p id={`${id}-limit`} className="field-hint">
          {t("notes.atLimit")}
        </p>
      )}
      {error && (
        <p id={`${id}-error`} className="note-error" role="alert">
          {t("notes.saveFailed", { message: errorMessage(t, i18n, error) })}
        </p>
      )}
      {confirm && (
        <ConfirmDialog
          title={t("notes.deleteTitle")}
          message={t("notes.confirmDelete", { date })}
          confirmLabel={t("notes.deleteConfirm")}
          onConfirm={() => void remove()}
          onCancel={() => setConfirm(false)}
        />
      )}
    </li>
  );
}
