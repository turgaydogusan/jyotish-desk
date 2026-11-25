// Geri alınamaz işlemin onayı (silme). Odak "Vazgeç"te başlar: Enter yanlışlıkla silmez.
import { useTranslation } from "react-i18next";
import { Dialog } from "./Dialog";

export interface ConfirmDialogProps {
  title: string;
  message: string;
  /** Onay düğmesinin metni; işlemi adlandırır ("Danışanı sil"). */
  confirmLabel: string;
  onConfirm(): void;
  onCancel(): void;
}

export function ConfirmDialog({ title, message, confirmLabel, onConfirm, onCancel }: ConfirmDialogProps) {
  const { t } = useTranslation();
  return (
    <Dialog title={title} onClose={onCancel}>
      <p>{message}</p>
      <div className="dialog-actions">
        <button type="button" data-autofocus onClick={onCancel}>
          {t("clients.cancel")}
        </button>
        <button type="button" className="button-danger" onClick={onConfirm}>
          {confirmLabel}
        </button>
      </div>
    </Dialog>
  );
}
