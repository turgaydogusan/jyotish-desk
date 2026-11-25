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
      // İkinci danışan: aynı hesaplanmış harita, başka kimlik ve ad (arayüz hesap yapmaz).
      const ayse = structuredClone(fixture.opened);
      ayse.client = { ...ayse.client, id: 2, uuid: "00000000-0000-4000-8000-000000000003", name: "Ayşe Yılmaz", last_chart_id: 20 };
      ayse.chart_record = { ...ayse.chart_record, id: 20, client_id: 2, uuid: "00000000-0000-4000-8000-000000000004" };
      ayse.recent_notes = [];
      const opened: Record<number, typeof ayse> = { 1: fixture.opened, 2: ayse };
      const clients = () => [opened[2].client, opened[1].client];
      // Notlar bellek içi; kimlik ve zaman sırayla artar.
      const notes: { id: number; uuid: string; client_id: number; chart_id: number | null; body: string; created_at: string; updated_at: string }[] = [];
      let noteSeq = 0;
      const stamp = () => new Date(Date.UTC(2026, 10, 25, 10, 0, noteSeq)).toISOString();
      const respond: Record<string, (args: Record<string, unknown>) => unknown> = {
        open_client: (args) => opened[args.clientId as number],
        list_clients: () => clients(),
        list_clients_by_tag: () => [],
        search_clients: (args) => {
          const q = String(args.query).toLocaleLowerCase("tr");
          return clients()
            .filter((c) => c.name.toLocaleLowerCase("tr").includes(q))
            .map((client) => ({ client, matched: "Name", snippet: null }));
        },
        list_tags: () => [],
        list_charts: (args) => [opened[args.clientId as number].chart_record],
        set_chart_tags: (args) => args.tags,
        list_notes: (args) => notes.filter((n) => n.client_id === args.clientId).reverse(),
        add_note: (args) => {
          noteSeq += 1;
          const n = { id: noteSeq, uuid: `note-${noteSeq}`, client_id: args.clientId as number, chart_id: args.chartId as number | null, body: String(args.body), created_at: stamp(), updated_at: stamp() };
          notes.push(n);
          return n;
        },
        update_note: (args) => {
          const n = notes.find((x) => x.id === args.noteId)!;
          n.body = String(args.body);
          return n;
        },
        delete_note: (args) => {
          const at = notes.findIndex((x) => x.id === args.noteId);
          if (at < 0) throw { code: "ipc", params: { detail: `no note ${String(args.noteId)}` } };
          notes.splice(at, 1);
          return null;
        },
        restore_session: () => (session ? fixture.opened : null),
        load_panel_layout: () => null,
        get_app_prefs: () => fixture.prefs,
        set_app_prefs: () => null,
        // Yeniden hesap taklit edilmez: aynı harita, yeni ayarların kopyasıyla döner.
        save_calc_settings: (args) => ({ ...fixture.opened.chart, meta: { ...fixture.opened.chart.meta, settings: args.settings } }),
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
