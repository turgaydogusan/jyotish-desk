// Açık haritanın etiketleri (spec §5: etiketler harita düzeyindedir). Virgülle ayrılmış metin;
// Enter ya da alandan çıkış kaydeder. Normalleştirme (kırpma, NFC, büyük/küçük harf, tekrar)
// Rust'tadır; alan kayıttan dönen listeyi gösterir.
import { useEffect, useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CommandError } from "../bindings";
import { errorMessage } from "../i18n/errors";
import { useSession } from "../session/store";

export function splitTags(text: string): string[] {
  return text
    .split(",")
    .map((t) => t.trim())
    .filter((t) => t !== "");
}

export function TagEditor({ chartId, tags }: { chartId: number; tags: string[] }) {
  const { t, i18n } = useTranslation();
  const { setTags } = useSession();
  const id = useId();
  const saved = tags.join(", ");
  const [text, setText] = useState(saved);
  const [error, setError] = useState<CommandError | null>(null);
  useEffect(() => setText(saved), [saved]);

  const commit = async () => {
    const next = splitTags(text);
    if (next.join(", ") === saved) return;
    const r = await api.setChartTags(chartId, next);
    if (r.status === "error") {
      setError(r.error);
      return;
    }
    setError(null);
    setText(r.data.join(", "));
    setTags(chartId, r.data);
  };

  return (
    <div className="tag-editor">
      <label htmlFor={`${id}-tags`}>{t("clients.tags")}</label>
      <input
        id={`${id}-tags`}
        value={text}
        aria-describedby={`${id}-hint${error ? ` ${id}-error` : ""}`}
        aria-invalid={error ? true : undefined}
        onChange={(e) => setText(e.target.value)}
        onBlur={() => void commit()}
        onKeyDown={(e) => {
          // IME bileşimini onaylayan Enter kaydetmez (WebKit bunu 229 koduyla bildirir).
          if (e.key === "Enter" && !e.nativeEvent.isComposing && e.keyCode !== 229) {
            e.preventDefault();
            void commit();
          }
        }}
      />
      <p id={`${id}-hint`} className="sidebar-hint">
        {t("clients.tagsHint")}
      </p>
      {error && (
        <p id={`${id}-error`} className="sidebar-error" role="alert">
          {t("notice.tagsFailed", { message: errorMessage(t, i18n, error) })}
        </p>
      )}
    </div>
  );
}
