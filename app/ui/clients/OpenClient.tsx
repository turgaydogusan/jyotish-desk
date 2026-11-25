// Kenar çubuğunun açık danışan bölümü: haritaları (`list_charts`), harita ekleme, harita ve
// danışan silme (onaylı, geri alınamaz) ve açık haritanın etiketleri. Açık harita silinirse
// danışanın en son güncellenen haritası açılır; açık danışan silinirse pano boşalır.
import { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { ChartRecord, CommandError, OpenedClient } from "../bindings";
import { errorMessage } from "../i18n/errors";
import { useSession } from "../session/store";
import { ConfirmDialog } from "../shell/ConfirmDialog";
import { useNotices } from "../shell/notices";
import { useViewStore } from "../view/store";
import { CLIENT_SEARCH_ID } from "./ClientList";
import { TagEditor } from "./TagEditor";

type Pending = { kind: "chart"; chart: ChartRecord } | { kind: "client" };

export function OpenClient({ opened, onAddChart }: { opened: OpenedClient; onAddChart(): void }) {
  const { t, i18n } = useTranslation();
  const session = useSession();
  const { push } = useNotices();
  const { flush } = useViewStore();
  const id = useId();
  const clientId = opened.client.id;
  const [charts, setCharts] = useState<ChartRecord[]>([]);
  const [error, setError] = useState<CommandError | null>(null);
  const [pending, setPending] = useState<Pending | null>(null);
  const headingRef = useRef<HTMLHeadingElement>(null);
  // Silme sürerken başka harita ya da danışan açılırsa silme bitince geri dönülmez.
  const current = useRef(opened);
  current.current = opened;
  const live = useRef(true);
  useEffect(() => {
    live.current = true;
    return () => {
      live.current = false;
    };
  }, []);

  useEffect(() => {
    let live = true;
    void api.listCharts(clientId).then((r) => {
      if (!live) return;
      if (r.status === "ok") {
        setCharts(r.data);
        setError(null);
      } else setError(r.error);
    });
    return () => {
      live = false;
    };
  }, [clientId, session.revision]);

  const label = (c: ChartRecord) => c.label || t(`chart.kind.${c.kind}`);
  const failed = (e: CommandError) => push({ id: "delete", type: "error", messageKey: "notice.deleteFailed", error: e });

  const deleteChart = async (chart: ChartRecord) => {
    setPending(null);
    // Bekleyen görünüm kaydı silmeden önce yazılır; silinen haritaya sonradan kayıt gitmez (C5).
    flush();
    const r = await api.deleteChart(chart.id);
    if (r.status === "error") return failed(r.error);
    // Silme sürerken başka danışana (bileşen ayrıldı) ya da başka haritaya geçildiyse yalnızca listeler yenilenir.
    if (!live.current || chart.id !== current.current.chart_record.id) {
      session.touch();
      // Silinen satırın düğmesi kaybolur: odak danışan başlığına geçer.
      if (live.current) headingRef.current?.focus();
      return;
    }
    // Açık harita silindi: diğer haritalardan en son güncelleneni açılır (uyarısız).
    const rest = await api.listCharts(clientId);
    if (!live.current || current.current.chart_record.id !== chart.id) return session.touch();
    const next = rest.status === "ok" ? rest.data[0] : undefined;
    const e = next ? await session.openClient(clientId, next.id) : rest.status === "error" ? rest.error : null;
    // Diğer harita açılamazsa silinen harita ekranda bırakılmaz: pano boşalır.
    if (!next || e) {
      session.close();
      document.getElementById(CLIENT_SEARCH_ID)?.focus();
    } else headingRef.current?.focus();
    if (e) push({ id: "client", type: "error", messageKey: "notice.openFailed", error: e });
  };

  const deleteClient = async () => {
    setPending(null);
    flush();
    const r = await api.deleteClient(clientId);
    if (r.status === "error") return failed(r.error);
    session.close();
    document.getElementById(CLIENT_SEARCH_ID)?.focus();
  };

  const onlyChart = charts.length <= 1;
  return (
    <section className="open-client" aria-labelledby={`${id}-name`}>
      <h2 id={`${id}-name`} ref={headingRef} tabIndex={-1} className="sidebar-heading">
        {opened.client.name}
      </h2>
      <h3 className="sidebar-subheading">{t("clients.charts")}</h3>
      {error && (
        <p className="sidebar-error" role="alert">
          {t("clients.loadFailed", { message: errorMessage(t, i18n, error) })}
        </p>
      )}
      <ul className="chart-rows">
        {charts.map((c) => (
          <li key={c.id}>
            <button
              type="button"
              aria-current={c.id === opened.chart_record.id ? "true" : undefined}
              onClick={() => void session.openClient(clientId, c.id).then((e) => e && push({ id: "client", type: "error", messageKey: "notice.openFailed", error: e }))}
            >
              <span className="client-name">{label(c)}</span>
              <span className="client-meta">{c.birth.date}</span>
            </button>
            <button
              type="button"
              className="icon-button"
              aria-label={t("clients.deleteChartNamed", { label: label(c) })}
              aria-disabled={onlyChart ? "true" : undefined}
              aria-describedby={onlyChart ? `${id}-last` : undefined}
              onClick={() => !onlyChart && setPending({ kind: "chart", chart: c })}
            >
              {t("clients.deleteChart")}
            </button>
          </li>
        ))}
      </ul>
      {onlyChart && (
        <p id={`${id}-last`} className="sidebar-hint">
          {t("clients.lastChartHint")}
        </p>
      )}
      <div className="sidebar-actions">
        <button type="button" onClick={onAddChart}>
          {t("clients.addChart")}
        </button>
        <button type="button" onClick={() => setPending({ kind: "client" })}>
          {t("clients.deleteClient")}
        </button>
      </div>
      {/* Harita başına anahtar: önceki haritanın geç gelen kayıt yanıtı yeni haritanın kutusuna yazılmaz. */}
      <TagEditor key={opened.chart_record.id} chartId={opened.chart_record.id} tags={opened.chart_record.tags} />
      {pending?.kind === "chart" && (
        <ConfirmDialog
          title={t("clients.deleteChartTitle")}
          message={t("clients.confirmDeleteChart", { label: label(pending.chart) })}
          confirmLabel={t("clients.deleteChart")}
          onConfirm={() => void deleteChart(pending.chart)}
          onCancel={() => setPending(null)}
        />
      )}
      {pending?.kind === "client" && (
        <ConfirmDialog
          title={t("clients.deleteClientTitle")}
          message={t("clients.confirmDeleteClient", { name: opened.client.name })}
          confirmLabel={t("clients.deleteClientConfirm")}
          onConfirm={() => void deleteClient()}
          onCancel={() => setPending(null)}
        />
      )}
    </section>
  );
}
