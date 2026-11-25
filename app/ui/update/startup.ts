// Açılıştaki güncelleme denetimi: yalnızca kullanıcı tercihi açtıysa (`pref.ts`) çalışır.
// Yeni sürüm varsa bildirim bandına bir bilgi satırı düşer; kurulum Hakkında penceresinden
// kullanıcının ayrı onayıyla yapılır. Denetim başarısızsa (çevrimdışı, yapılandırılmamış
// derleme) sessizce geçilir: açılış engellenmez, hata gösterilmez ve loglanmaz (C1).
import { api } from "../api";
import type { Notice } from "../shell/notices";
import { loadAutoCheck } from "./pref";

export async function startupUpdateCheck(push: (notice: Notice) => void, storage: Storage = window.localStorage): Promise<void> {
  if (!loadAutoCheck(storage)) return;
  const r = await api.checkForUpdate();
  if (r.status === "ok" && r.data !== null) {
    push({ id: "update", type: "info", messageKey: "notice.updateAvailable", params: { version: r.data.version } });
  }
}
