// tzdb farkı kararı (spec §4; Plan 11 D5). Saklanan an kullanılır; kullanıcı yeni tzdb'nin anını
// kabul eder (`update_chart`, aynı doğum verisi) ya da saklananı korur (`keep_stored_moment`).
// İki yol da görünüm kaydını önce boşaltır (C5) ve dönen haritayı açar; an sessizce kaymaz.
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { api, type Result } from "../api";
import type { OpenedClient } from "../bindings";
import { useSession } from "../session/store";
import { useViewStore } from "../view/store";
import { useNotices } from "./notices";

export function TzdbActions() {
  const { t } = useTranslation();
  const session = useSession();
  const { flush } = useViewStore();
  const { push } = useNotices();
  const [busy, setBusy] = useState(false);
  const o = session.opened;
  if (!o) return null;
  const chartId = o.chart_record.id;

  const run = async (call: () => Promise<Result<OpenedClient>>) => {
    if (busy) return;
    setBusy(true);
    flush();
    const r = await call();
    setBusy(false);
    if (r.status === "error") {
      push({ id: "tzdb", type: "error", messageKey: "notice.tzdbFailed", error: r.error });
      return;
    }
    session.open(r.data);
  };

  return (
    <span className="notice-actions">
      <button
        type="button"
        aria-disabled={busy || undefined}
        onClick={() => void run(() => api.updateChart(chartId, { label: o.chart_record.label, kind: o.chart_record.kind, birth: o.chart_record.birth }))}
      >
        {t("tzdb.accept")}
      </button>
      <button type="button" aria-disabled={busy || undefined} onClick={() => void run(() => api.keepStoredMoment(chartId))}>
        {t("tzdb.keep")}
      </button>
    </span>
  );
}
