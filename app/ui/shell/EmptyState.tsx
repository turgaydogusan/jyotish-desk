import { useTranslation } from "react-i18next";

/** Açık harita yokken pano yerine tek eylem: yeni harita (⌘N). */
export function EmptyState({ onNew }: { onNew(): void }) {
  const { t } = useTranslation();
  return (
    <section className="empty-state" aria-labelledby="empty-state-title">
      <h2 id="empty-state-title">{t("empty.title")}</h2>
      <p>{t("empty.body")}</p>
      <button type="button" aria-keyshortcuts="Meta+N" onClick={onNew}>
        {t("empty.action")}
      </button>
    </section>
  );
}
