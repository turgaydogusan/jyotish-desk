// Sol kenar çubuğu (UI spec §4): açık danışanın haritaları ve etiketleri, danışan listesi,
// arama, etiket süzgeci ve son açılanlar. `Cmd+\` ile açılıp kapanır (Shell).
import { useTranslation } from "react-i18next";
import { useSession } from "../session/store";
import { ClientList } from "./ClientList";
import "./clients.css";
import { OpenClient } from "./OpenClient";

export const SIDEBAR_ID = "client-sidebar";

export function Sidebar({ onAddChart }: { onAddChart(): void }) {
  const { t } = useTranslation();
  const { opened } = useSession();
  return (
    <aside id={SIDEBAR_ID} className="sidebar no-print" aria-label={t("clients.region")}>
      {opened && <OpenClient key={opened.client.id} opened={opened} onAddChart={onAddChart} />}
      <ClientList />
    </aside>
  );
}
