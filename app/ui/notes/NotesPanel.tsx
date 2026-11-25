// Notlar paneli (spec §3.3, §4): açık danışanın tarih damgalı notları, en yeni önce. Liste
// `list_notes` ile okunur (bekleyen kayıtlar bittikten sonra); `open_client`'ın `recent_notes`
// alanı kullanılmaz, çünkü bu oturumdaki düzenlemelerden sonra eskir. "Yeni not" boş bir not
// ekler ve ona odaklanır; metin otomatik kaydedilir.
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CommandError, Note, OpenedClient } from "../bindings";
import { errorMessage } from "../i18n/errors";
import type { PanelProps } from "../panels/types";
import { useSession } from "../session/store";
import { useNotices } from "../shell/notices";
import { NoteEditor } from "./NoteEditor";
import { enqueueNoteSave, notesIdle } from "./saves";
import "./notes.css";

/** Panelde gösterilen en çok not (Rust `MAX_LIST` 500). */
export const NOTES_LIMIT = 100;

export function NotesPanel(_props: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <ClientNotes key={opened.client.id} opened={opened} />;
}

function ClientNotes({ opened }: { opened: OpenedClient }) {
  const { t, i18n } = useTranslation();
  const { push } = useNotices();
  const clientId = opened.client.id;
  const [notes, setNotes] = useState<Note[] | null>(null);
  const [error, setError] = useState<CommandError | null>(null);
  const [retry, setRetry] = useState(0);
  const [fresh, setFresh] = useState<number | null>(null);
  const addRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    let live = true;
    void notesIdle()
      .then(() => api.listNotes(clientId, NOTES_LIMIT))
      .then((r) => {
        if (!live) return;
        if (r.status === "ok") {
          setNotes(r.data);
          setError(null);
        } else setError(r.error);
      });
    return () => {
      live = false;
    };
  }, [clientId, retry]);

  // Ekleme sürerken ikinci tıklama ikinci bir boş not açmaz.
  const adding = useRef(false);
  const add = async () => {
    if (adding.current) return;
    adding.current = true;
    try {
      const r = await api.addNote(clientId, opened.chart_record.id, "");
      if (r.status === "error") return push({ id: "note", type: "error", messageKey: "notice.noteFailed", error: r.error });
      setNotes((list) => [r.data, ...(list ?? [])]);
      setFresh(r.data.id);
    } finally {
      adding.current = false;
    }
  };

  const saved = useCallback((n: Note) => setNotes((list) => list && list.map((x) => (x.id === n.id ? n : x))), []);

  const remove = async (n: Note) => {
    // Silme, kayıtlarla aynı sırada gider: notun bekleyen kaydı silinmiş nota yazmaya çalışmaz.
    const r = await enqueueNoteSave(() => api.deleteNote(n.id));
    if (r.status === "error") {
      push({ id: "delete", type: "error", messageKey: "notice.deleteFailed", error: r.error });
      return false;
    }
    setNotes((list) => list && list.filter((x) => x.id !== n.id));
    addRef.current?.focus();
    return true;
  };

  return (
    <div className="notes-panel">
      <div className="notes-actions">
        <button ref={addRef} type="button" onClick={() => void add()} disabled={notes === null}>
          {t("notes.add")}
        </button>
      </div>
      {error && (
        <div className="note-error" role="alert">
          <p>{t("notes.loadFailed", { message: errorMessage(t, i18n, error) })}</p>
          <button type="button" onClick={() => setRetry((n) => n + 1)}>
            {t("notes.retry")}
          </button>
        </div>
      )}
      {notes && notes.length === 0 && <p className="panel-message">{t("notes.empty")}</p>}
      {notes && notes.length > 0 && (
        <ul className="notes-list" aria-label={t("notes.list", { name: opened.client.name })}>
          {notes.map((n) => (
            <NoteEditor
              key={n.id}
              note={n}
              autoFocus={n.id === fresh}
              onThisChart={n.chart_id === opened.chart_record.id}
              onSaved={saved}
              onDelete={remove}
            />
          ))}
        </ul>
      )}
    </div>
  );
}
