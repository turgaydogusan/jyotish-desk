// Kenar çubuğunun veri bölümü (spec §5 içe ve dışa aktarma): bütün danışanlar JSON ya da CSV
// olarak dışa, JSON olarak içe aktarılır. Dosya penceresi Rust'ta açılır; arayüz yol görmez.
// "Notları dahil et" varsayılan kapalıdır (notlar hassas veridir). JHD içe aktarma bu sürümde
// yoktur (gerçek dosyalarla doğrulanacak, Plan 11).
import { useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import { useSession } from "../session/store";
import { Dialog } from "../shell/Dialog";
import { useNotices } from "../shell/notices";

export function DataActions() {
  const { t } = useTranslation();
  const session = useSession();
  const { push, dismiss } = useNotices();
  const id = useId();
  const [asking, setAsking] = useState(false);
  const [includeNotes, setIncludeNotes] = useState(false);
  const [busy, setBusy] = useState(false);

  const run = async (work: () => Promise<void>) => {
    setBusy(true);
    dismiss("exchange");
    try {
      await work();
    } finally {
      setBusy(false);
    }
  };

  // Her açılışta kapalı başlar: önceki dışa aktarmanın seçimi taşınmaz (notlar hassas veridir).
  const openExport = () => {
    setIncludeNotes(false);
    setAsking(true);
  };

  const exported = (file: string | null) => file !== null && push({ id: "exchange", type: "info", messageKey: "data.exported", params: { file } });

  const exportJson = () =>
    run(async () => {
      setAsking(false);
      const r = await api.exportClientsJson(includeNotes);
      if (r.status === "error") return push({ id: "exchange", type: "error", messageKey: "notice.exportFailed", error: r.error });
      exported(r.data);
    });

  const exportCsv = () =>
    run(async () => {
      const r = await api.exportClientsCsv();
      if (r.status === "error") return push({ id: "exchange", type: "error", messageKey: "notice.exportFailed", error: r.error });
      exported(r.data);
    });

  const importJson = () =>
    run(async () => {
      const r = await api.importClientsJson();
      if (r.status === "error") return push({ id: "exchange", type: "error", messageKey: "notice.importFailed", error: r.error });
      if (r.data === null) return;
      const { file, report } = r.data;
      push({
        id: "exchange",
        type: report.warnings.length > 0 ? "warning" : "info",
        messageKey: report.warnings.length > 0 ? "data.importedWithWarnings" : "data.imported",
        params: {
          file,
          clients: String(report.clients_added),
          charts: String(report.charts_added),
          notes: String(report.notes_added),
          skipped: String(report.skipped),
          warnings: String(report.warnings.length),
        },
      });
      session.touch();
    });

  return (
    <section className="data-actions" aria-labelledby={`${id}-heading`}>
      <h2 id={`${id}-heading`} className="sidebar-heading">
        {t("data.heading")}
      </h2>
      <div className="sidebar-actions">
        <button type="button" disabled={busy} onClick={openExport}>
          {t("data.exportJson")}
        </button>
        <button type="button" disabled={busy} onClick={() => void exportCsv()}>
          {t("data.exportCsv")}
        </button>
        <button type="button" disabled={busy} onClick={() => void importJson()}>
          {t("data.importJson")}
        </button>
      </div>
      {asking && (
        <Dialog title={t("data.exportTitle")} onClose={() => setAsking(false)}>
          <p>{t("data.exportIntro")}</p>
          <label className="checkbox-row">
            <input type="checkbox" checked={includeNotes} aria-describedby={`${id}-notes-hint`} onChange={(e) => setIncludeNotes(e.target.checked)} />
            {t("data.includeNotes")}
          </label>
          <p id={`${id}-notes-hint`} className="sidebar-hint">
            {t("data.includeNotesHint")}
          </p>
          <div className="dialog-actions">
            <button type="button" onClick={() => setAsking(false)}>
              {t("clients.cancel")}
            </button>
            <button type="button" data-autofocus onClick={() => void exportJson()}>
              {t("data.chooseFile")}
            </button>
          </div>
        </Dialog>
      )}
    </section>
  );
}
