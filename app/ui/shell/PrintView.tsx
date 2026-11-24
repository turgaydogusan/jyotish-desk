import { useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CommandError } from "../bindings";
import { useSky } from "../panels/sky";
import { PanelError } from "../panels/states";
import { PrintSpike } from "../PrintSpike";

/** M0 baskı denemesi; M7'deki A4 baskı düzenleri gelene kadar menüden erişilir. */
export function PrintView() {
  const { t } = useTranslation();
  const sky = useSky();
  const [error, setError] = useState<CommandError | null>(null);
  const print = async () => {
    setError(null);
    const r = await api.printWindow();
    if (r.status === "error") setError(r.error);
  };
  return (
    <div>
      <div className="print-view-actions no-print">
        <button type="button" onClick={() => void print()}>
          {t("shell.print")}
        </button>
        {error && <PanelError error={error} onRetry={() => void print()} />}
      </div>
      <PrintSpike rows={sky.state.status === "ok" ? sky.state.rows : []} />
    </div>
  );
}
