import { useState, type ReactNode } from "react";
import { EMPTY_BOOT, type Boot } from "./boot";
import { HighlightProvider } from "./highlight/store";
import { PrefsProvider } from "./settings/prefs";
import { SessionProvider } from "./session/store";
import { NoticeProvider, storeWarningNotices, useNotices, type Notice } from "./shell/notices";
import { Shell } from "./shell/Shell";
import { initialViewModel } from "./view/model";
import { storePersistence, type ViewPersistence } from "./view/persistence";
import { ViewStateProvider } from "./view/store";

function bootNotices(boot: Boot): Notice[] {
  const layout: Notice[] = [
    ...(boot.layoutError ? [{ id: "layout", type: "error", messageKey: "notice.layoutLoadFailed", error: boot.layoutError } as Notice] : []),
    ...(boot.prefsError ? [{ id: "prefs", type: "error", messageKey: "notice.prefsLoadFailed", error: boot.prefsError } as Notice] : []),
  ];
  if (boot.error) return [...layout, { id: "boot", type: "error", messageKey: "notice.restoreFailed", error: boot.error }];
  return [...layout, ...(boot.opened ? storeWarningNotices(boot.opened.warnings) : [])];
}

/**
 * Görünüm deposu ve kayıt: kayıt hatası bildirim bandına gider (aynı kimlik, tek bildirim) ve
 * sonraki başarılı kayıtla kalkar. Kayıtlı düzen okunamadıysa kayıtlar tutulur (M6-3).
 */
function Stores({ boot, persistence, children }: { boot: Boot; persistence?: ViewPersistence; children: ReactNode }) {
  const { push, dismiss } = useNotices();
  const [store] = useState(
    () =>
      persistence ??
      storePersistence(
        {
          onError: (error) => push({ id: "save", type: "error", messageKey: "notice.saveFailed", error }),
          onSaved: () => dismiss("save"),
        },
        { hold: boot.layoutError !== null },
      ),
  );
  const [initial] = useState(() =>
    initialViewModel(
      boot.layout,
      boot.opened && { id: boot.opened.chart_record.id, dashaPath: boot.opened.view_state.dasha_expanded_path },
    ),
  );
  return (
    <ViewStateProvider initial={initial} persistence={store}>
      <SessionProvider initial={boot.opened}>{children}</SessionProvider>
    </ViewStateProvider>
  );
}

/**
 * Uygulama kökü. `boot` açılışta store'dan okunan düzen ve oturumdur (main.tsx);
 * `persistence` testler için verilebilir, varsayılan store komutlarıdır.
 */
export function App({ boot = EMPTY_BOOT, persistence }: { boot?: Boot; persistence?: ViewPersistence }) {
  const [notices] = useState(() => bootNotices(boot));
  return (
    <NoticeProvider initial={notices}>
      <PrefsProvider initial={boot.prefs?.prefs ?? null}>
        <Stores boot={boot} persistence={persistence}>
          <HighlightProvider>
            <Shell />
          </HighlightProvider>
        </Stores>
      </PrefsProvider>
    </NoticeProvider>
  );
}
