import { useTranslation } from "react-i18next";
import { SHORTCUT_LIST } from "../shortcuts";
import { Dialog } from "./Dialog";

/** Kısayol yardımı (`?`): liste `SHORTCUT_LIST`'ten gelir, burada elle yazılmaz. */
export function HelpDialog({ onClose }: { onClose(): void }) {
  const { t } = useTranslation();
  return (
    <Dialog title={t("shortcuts.title")} onClose={onClose}>
      <table className="shortcut-table">
        <thead>
          <tr>
            <th scope="col">{t("shortcuts.keys")}</th>
            <th scope="col">{t("shortcuts.action")}</th>
          </tr>
        </thead>
        <tbody>
          {SHORTCUT_LIST.map((s) => (
            <tr key={s.id}>
              <td>
                <kbd>{s.keys}</kbd>
              </td>
              <td>{t(`shortcuts.${s.id}`)}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <div className="dialog-actions">
        <button type="button" onClick={onClose}>
          {t("shortcuts.close")}
        </button>
      </div>
    </Dialog>
  );
}
