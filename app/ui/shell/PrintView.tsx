import { useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CommandError } from "../bindings";
import { signIndex } from "../chart/adapter";
import { showsRetro } from "../chart/types";
import { PanelError } from "../panels/states";
import { PrintSpike, type SpikeRow } from "../PrintSpike";
import { useSession } from "../session/store";

/** M0 baskı denemesi; M7'deki A4 baskı düzenleri gelene kadar menüden erişilir. */
export function PrintView() {
  const { t } = useTranslation();
  const { opened } = useSession();
  const [error, setError] = useState<CommandError | null>(null);
  const print = async () => {
    setError(null);
    const r = await api.printWindow();
    if (r.status === "error") setError(r.error);
  };
  const rows: SpikeRow[] = (opened?.chart.grahas ?? []).map((g) => ({
    body: g.graha,
    sign: signIndex(g.sign),
    lon: g.lon,
    showRetro: showsRetro(g.graha, g.retrograde),
  }));
  return (
    <div>
      <div className="print-view-actions no-print">
        <button type="button" onClick={() => void print()}>
          {t("shell.print")}
        </button>
        {error && <PanelError error={error} onRetry={() => void print()} />}
      </div>
      <PrintSpike rows={rows} />
    </div>
  );
}
