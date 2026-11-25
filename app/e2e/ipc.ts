// Tauri IPC taklidi: `@tauri-apps/api/core` `window.__TAURI_INTERNALS__.invoke` çağırır.
// Yanıtlar gerçek veri fikstüründen (ui/test/fixtures/delhi.json, ui_fixture.rs üretir);
// her çağrı `window.__ipcCalls`'a yazılır, testler kayıtları buradan okur.
import { readFileSync } from "node:fs";
import type { Page } from "@playwright/test";

export const FIXTURE = JSON.parse(readFileSync(new URL("../ui/test/fixtures/delhi.json", import.meta.url), "utf8"));

export interface IpcOptions {
  /** `restore_session` dönüşü: `true` Delhi haritası, `false` oturum yok. */
  session: boolean;
}

export async function mockIpc(page: Page, { session }: IpcOptions) {
  await page.addInitScript(
    ({ fixture, session }) => {
      const calls: { cmd: string; args: unknown }[] = [];
      Object.assign(window, { __ipcCalls: calls });
      const respond: Record<string, (args: Record<string, unknown>) => unknown> = {
        restore_session: () => (session ? fixture.opened : null),
        load_panel_layout: () => null,
        save_panel_layout: () => null,
        save_view_state: () => null,
        create_client: () => fixture.opened,
        update_chart: () => fixture.opened,
        search_places: () => fixture.places,
        preview_moment: () => fixture.preview,
        dasha_children: (args) =>
          JSON.stringify(args.path) === JSON.stringify(fixture.dasha_children.path) ? fixture.dasha_children.periods : [],
        print_window: () => null,
      };
      Object.assign(window, {
        __TAURI_INTERNALS__: {
          invoke: async (cmd: string, args: Record<string, unknown> = {}) => {
            calls.push({ cmd, args });
            const handler = respond[cmd];
            if (!handler) throw { code: "ipc", params: { detail: `no mock for ${cmd}` } };
            return structuredClone(handler(args));
          },
          transformCallback: () => 0,
          unregisterCallback: () => {},
          metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main", windowLabel: "main" } },
        },
      });
    },
    { fixture: FIXTURE, session },
  );
}

export async function ipcCalls(page: Page, cmd: string): Promise<unknown[]> {
  return page.evaluate((c) => (window as unknown as { __ipcCalls: { cmd: string; args: unknown }[] }).__ipcCalls.filter((x) => x.cmd === c).map((x) => x.args), cmd);
}
