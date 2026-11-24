import { useState } from "react";
import { HighlightProvider } from "./highlight/store";
import { localStoragePersistence, type LayoutPersistence } from "./layout/persistence";
import { LayoutProvider } from "./layout/store";
import { SkyProvider } from "./panels/sky";
import { Shell } from "./shell/Shell";

/** Uygulama kökü. `persistence` testler için verilebilir; varsayılan localStorage'dır. */
export function App({ persistence }: { persistence?: LayoutPersistence }) {
  const [store] = useState(() => persistence ?? localStoragePersistence());
  return (
    <LayoutProvider persistence={store}>
      <HighlightProvider>
        <SkyProvider>
          <Shell />
        </SkyProvider>
      </HighlightProvider>
    </LayoutProvider>
  );
}
