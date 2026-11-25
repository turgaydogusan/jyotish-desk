// Panel ve kabuk testleri için sağlayıcılar: bildirim, görünüm deposu (bellek kalıcılığı),
// oturum (fikstür haritası) ve vurgu. Arayüz hesap yapmaz; testler fikstürü çizer.
import { render } from "@testing-library/react";
import type { ReactNode } from "react";
import type { OpenedClient } from "../bindings";
import { HighlightProvider } from "../highlight/store";
import type { LayoutState } from "../layout/model";
import { SessionProvider } from "../session/store";
import { NoticeProvider } from "../shell/notices";
import { initialViewModel } from "../view/model";
import { memoryPersistence } from "../view/persistence";
import { ViewStateProvider } from "../view/store";
import { fixture } from "./fixture";

export interface RenderOptions {
  /** `null`: açık harita yok. Varsayılan: Delhi fikstürü. */
  opened?: OpenedClient | null;
  layout?: LayoutState | null;
  debounceMs?: number;
}

export function renderWithStores(ui: ReactNode, { opened = fixture().opened, layout = null, debounceMs = 0 }: RenderOptions = {}) {
  const persistence = memoryPersistence();
  const initial = initialViewModel(
    layout,
    opened && { id: opened.chart_record.id, dashaPath: opened.view_state.dasha_expanded_path },
  );
  const utils = render(
    <NoticeProvider>
      <ViewStateProvider initial={initial} persistence={persistence} debounceMs={debounceMs}>
        <SessionProvider initial={opened}>
          <HighlightProvider>{ui}</HighlightProvider>
        </SessionProvider>
      </ViewStateProvider>
    </NoticeProvider>,
  );
  return { ...utils, persistence };
}
