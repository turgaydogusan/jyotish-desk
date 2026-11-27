// Tauri IPC taklidi: `@tauri-apps/api/core` `window.__TAURI_INTERNALS__.invoke` çağırır.
// Yanıtlar gerçek veri fikstüründen (ui/test/fixtures/delhi.json, ui_fixture.rs üretir);
// her çağrı `window.__ipcCalls`'a yazılır, testler kayıtları buradan okur.
import { readFileSync } from "node:fs";
import type { Page } from "@playwright/test";

export const FIXTURE = JSON.parse(readFileSync(new URL("../ui/test/fixtures/delhi.json", import.meta.url), "utf8"));

export interface IpcOptions {
  /** `restore_session` dönüşü: `true` Delhi haritası, `false` oturum yok. */
  session: boolean;
  /** Doğum saati bilinmeyen harita: uyarı ve notlar basılır (baskının en uzun sayfaları). */
  unknownTime?: boolean;
  /** Uyum: eş haritanın (Ayşe) doğum saati bilinmiyor; `chart_input` `time_known: false` döner. */
  partnerUnknownTime?: boolean;
}

export async function mockIpc(page: Page, { session, unknownTime = false, partnerUnknownTime = false }: IpcOptions) {
  await page.addInitScript(
    ({ fixture, session, unknownTime, partnerUnknownTime }) => {
      if (unknownTime) {
        fixture.opened.chart_record.birth.time = null;
        fixture.opened.chart_record.birth.time_certainty = "Unknown";
        fixture.opened.chart_record.resolved.warnings = ["TimeUnknownNoon"];
      }
      const calls: { cmd: string; args: unknown }[] = [];
      Object.assign(window, { __ipcCalls: calls });
      // İkinci danışan: aynı hesaplanmış harita, başka kimlik ve ad (arayüz hesap yapmaz).
      const ayse = structuredClone(fixture.opened);
      ayse.client = { ...ayse.client, id: 2, uuid: "00000000-0000-4000-8000-000000000003", name: "Ayşe Yılmaz", last_chart_id: 20 };
      ayse.chart_record = { ...ayse.chart_record, id: 20, client_id: 2, uuid: "00000000-0000-4000-8000-000000000004" };
      ayse.recent_notes = [];
      // Ayşe'nin haritası uyum fikstürünün eş haritasıdır: girdisi ve raporlar fikstürden gelir.
      const partnerUuid: string = ayse.chart_record.uuid;
      const isOpenChart = (input: { jd_ut: number }) => input.jd_ut === fixture.opened.chart.jd_ut;
      const opened: Record<number, typeof ayse> = { 1: fixture.opened, 2: ayse };
      const clients = () => [opened[2].client, opened[1].client];
      // Notlar bellek içi; kimlik ve zaman sırayla artar.
      const notes: { id: number; uuid: string; client_id: number; chart_id: number | null; body: string; created_at: string; updated_at: string }[] = [];
      let noteSeq = 0;
      // Etiketler bellek içi: harita kimliği → etiketler (Rust normalleştirmesi taklit edilmez).
      const tags: Record<number, string[]> = {};
      // Tercihler ve kayıtlı muhurta süzgeçleri bellek içi: bulunulan yer penceresi ve süzgeç
      // kaydı gerçek bir gidiş dönüş görür.
      let prefs = fixture.prefs.prefs;
      let muhurtaFilters: unknown[] = [];
      const chartOf = (clientId: number) => opened[clientId].chart_record.id;
      const stamp = () => new Date(Date.UTC(2026, 10, 25, 10, 0, noteSeq)).toISOString();
      const respond: Record<string, (args: Record<string, unknown>) => unknown> = {
        open_client: (args) => opened[args.clientId as number],
        list_clients: (args) => clients().slice(0, args.limit as number),
        list_clients_by_tag: (args) => clients().filter((c) => (tags[chartOf(c.id)] ?? []).includes(String(args.tag))).slice(0, args.limit as number),
        search_clients: (args) => {
          const q = String(args.query).toLocaleLowerCase("tr");
          return clients()
            .filter((c) => c.name.toLocaleLowerCase("tr").includes(q))
            .map((client) => ({ client, matched: "Name", snippet: null }))
            .slice(0, args.limit as number);
        },
        list_tags: () => {
          const count = new Map<string, number>();
          for (const list of Object.values(tags)) for (const t of list) count.set(t, (count.get(t) ?? 0) + 1);
          return [...count].map(([name, charts]) => ({ name, charts }));
        },
        list_charts: (args) => [opened[args.clientId as number].chart_record],
        set_chart_tags: (args) => {
          tags[args.chartId as number] = (args.tags as string[]).filter((t) => t.trim() !== "");
          return tags[args.chartId as number];
        },
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
        get_app_prefs: () => ({ ...fixture.prefs, prefs }),
        set_app_prefs: (args) => {
          prefs = args.prefs as typeof prefs;
          return null;
        },
        // Yeniden hesap taklit edilmez: aynı harita, yeni ayarların kopyasıyla döner.
        save_calc_settings: (args) => ({ ...fixture.opened.chart, meta: { ...fixture.opened.chart.meta, settings: args.settings } }),
        save_panel_layout: () => null,
        save_view_state: () => null,
        keep_stored_moment: () => fixture.opened,
        update_client: (args) => ({ ...opened[args.clientId as number].client, name: args.name, gender: args.gender }),
        create_client: () => fixture.opened,
        update_chart: () => fixture.opened,
        search_places: () => fixture.places,
        preview_moment: () => fixture.preview,
        dasha_children: (args) =>
          JSON.stringify(args.path) === JSON.stringify(fixture.dasha_children.path) ? fixture.dasha_children.periods : [],
        transits_at: () => fixture.transit,
        // Bakış tablosu yeniden hesaplanmaz: fikstürdeki gerçek sonuç döner.
        compute_drishti: () => fixture.drishti,
        // Jaimini raporu: fikstürdeki gerçek sonuç; saat bilinmiyorsa Rust padaları boş döner.
        compute_jaimini: (args) =>
          (args.input as { time_known: boolean }).time_known ? fixture.jaimini : { ...fixture.jaimini, arudhas: null },
        // Chara dasha: fikstürdeki gerçek sonuç; saat bilinmiyorsa Rust dashayı `null`, alt
        // dönemleri boş liste, anın yolunu `null` döner. 3. seviye yalnızca fikstürde bulunan iki
        // yol için vardır; transit anının yolu sabittir.
        compute_chara_dasha: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.chara_dasha : null),
        chara_dasha_children: (args) => {
          if (!(args.input as { time_known: boolean }).time_known) return [];
          const asked = JSON.stringify([args.cycle, args.path]);
          if (asked === JSON.stringify([fixture.chara_dasha_children.cycle, fixture.chara_dasha_children.path])) return fixture.chara_dasha_children.periods;
          if (asked === JSON.stringify([fixture.chara_dasha_at.cycle, fixture.chara_dasha_at.signs.slice(0, 2)])) return fixture.chara_dasha_at_children;
          return [];
        },
        chara_dasha_at: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.chara_dasha_at : null),
        // Shadbala: fikstürdeki gerçek sonuç; saat bilinmiyorsa Rust `null` döner.
        compute_shadbala: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.shadbala : null),
        // Yogalar: fikstürdeki iki gerçek sonuç (doğum saati bilinen ve bilinmeyen girdi).
        compute_yogas: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.yogas : fixture.yogas_time_unknown),
        // Uyum: eş haritanın girdisi ve kısa kimliği; başka kimlik `null` döner (harita yok).
        // Rapor yeniden hesaplanmaz, rollerden seçilir: açık harita damat rolündeyse `report`,
        // gelin rolündeyse `report_swapped`; gelin rolündeki eşin doğum saati bilinmiyorsa
        // `report_time_unknown`. Bu ikisinin birleşimi fikstürde yoktur ve hata verir.
        chart_input: (args) => (args.chartUuid === partnerUuid ? { ...fixture.compat.input, time_known: !partnerUnknownTime } : null),
        chart_brief: (args) => (args.chartUuid === partnerUuid ? { ...fixture.compat.brief, client_name: ayse.client.name } : null),
        compute_compat: (args) => {
          const groom = args.groom as { jd_ut: number; time_known: boolean };
          const bride = args.bride as { jd_ut: number; time_known: boolean };
          if (!isOpenChart(groom)) {
            // Fikstürde roller değişmişken eşin doğum saatinin bilinmediği rapor yoktur: yaklaşık
            // olmayan rapor sessizce dönmez.
            if (!groom.time_known) throw { code: "ipc", params: { detail: "no fixture: swapped roles with an unknown birth time" } };
            return fixture.compat.report_swapped;
          }
          return bride.time_known ? fixture.compat.report : fixture.compat.report_time_unknown;
        },
        // Muhurta: gün yeniden hesaplanmaz, fikstürdeki gerçek gün döner. Arama fikstürün üç
        // dilimini imlecin sırasıyla verir (ilk çağrıda imleç yoktur).
        day_panchang: () => fixture.muhurta.day,
        muhurta_search: (args) => {
          const cursor = args.cursor as { cursor: { chunk: number } } | null;
          return fixture.muhurta.pages[cursor === null ? 0 : cursor.cursor.chunk];
        },
        load_muhurta_filters: () => muhurtaFilters,
        save_muhurta_filters: (args) => {
          muhurtaFilters = args.filters as unknown[];
          return muhurtaFilters;
        },
        print_window: () => null,
        // Güncelleme: sahte yayın; ağ yoktur. Kurulum yalnızca denetimin bildirdiği sürümü kabul eder.
        app_info: () => ({ version: "0.9.0", updaterConfigured: true }),
        check_for_update: () => ({ version: "0.9.1", notes: "- Baskı düzeltmesi", published: "2026-12-01" }),
        install_update: (args) => {
          if (args.version !== "0.9.1") throw { code: "update_changed", params: {} };
          return null;
        },
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
    { fixture: FIXTURE, session, unknownTime, partnerUnknownTime },
  );
}

export async function ipcCalls(page: Page, cmd: string): Promise<unknown[]> {
  return page.evaluate((c) => (window as unknown as { __ipcCalls: { cmd: string; args: unknown }[] }).__ipcCalls.filter((x) => x.cmd === c).map((x) => x.args), cmd);
}
