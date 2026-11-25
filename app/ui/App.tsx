import { useState } from "react";
import { HighlightProvider } from "./highlight/store";
import { SkyProvider } from "./panels/sky";
import { Shell } from "./shell/Shell";
import { initialViewModel } from "./view/model";
import { storePersistence, type ViewPersistence } from "./view/persistence";
import { ViewStateProvider } from "./view/store";

/**
 * Uygulama kökü. `initialLayout` açılışta store'dan okunan ham düzendir (main.tsx);
 * `persistence` testler için verilebilir, varsayılan store komutlarıdır.
 */
export function App({ initialLayout = null, persistence }: { initialLayout?: unknown; persistence?: ViewPersistence }) {
  const [store] = useState(() => persistence ?? storePersistence());
  const [initial] = useState(() => initialViewModel(initialLayout));
  return (
    <ViewStateProvider initial={initial} persistence={store}>
      <HighlightProvider>
        <SkyProvider>
          <Shell />
        </SkyProvider>
      </HighlightProvider>
    </ViewStateProvider>
  );
}
