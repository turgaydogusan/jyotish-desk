# Plan 09 — Arayüz iskeleti: doğum girişi, harita, graha tablosu, dasha ağacı, i18n ve kısayollar (M5)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** M5'i tamamlamak: doğum verisi girişi (atlas araması, canlı an önizlemesi, DST seçimi), açık haritanın üç stilde SVG çizimi (her varga), tam graha tablosu, Vimshottari dasha ağacı, tr/en/fr metinleri ve merkezi klavye kısayolları. `ViewState` tek merkezden yönetilir, serileştirilir ve store'a kaydedilir.

**Architecture:**
- **Veri yolu:** arayüz hesap yapmaz. Harita `create_client` / `update_chart` / `restore_session` dönüşündeki `OpenedClient.chart`'tan çizilir; 4–5. seviye dasha `dasha_children` ile, yer araması `search_places`, an önizlemesi `preview_moment` ile gelir (Rust). Arayüzdeki tek "hesap" gösterim biçimlemesidir (JD → yerel tarih, derece → `16°30′`).
- **Merkezi durum:** `app/ui/view/` görünüm deposu `{chartId, layout, dashaPath}` modelini tek reducer'la tutar; `toViewState` tek serileştiricidir. Kayıt 500 ms debounce ile `save_view_state(chartId)` (harita açıkken) ya da `save_panel_layout` (harita yokken) çağırır; harita değişince bekleyen kayıt önceki haritaya yazılır. Plan 04'ün localStorage düzeni açılışta bir kez store'a taşınır.
- **Oturum:** `app/ui/session/` açık haritayı tutar (`OpenedClient | null`); `boot.ts` açılışta düzeni ve son oturumu paralel okur. Vurgu (Plan 04) oturumluk kalır, `ViewState`'e girmez.
- **Kabuk:** üst çubukta danışan adı, harita etiketi ve uyarı rozetleri; modal olmayan bildirim bandı (geri yükleme ve kayıt hataları, store uyarıları); `?` kısayol yardımı; `Cmd+N`/`Cmd+E` sağdan kayan doğum formu; harita yokken tek eylemli boş durum.
- **Rust:** yeni `atlas_commands.rs` (`search_places`, `preview_moment`; atlas paket kaynağı, yoksa ölümcül değil), `store` `layout.rs` (`save_panel_layout`) ve iki store komutu (`load_panel_layout`, `save_panel_layout`). M0 deneme komutu `planet_longitudes` kaldırılır.

**Tech Stack:** React 19, Vite 8, TypeScript 7 (`strict`), Vitest 5 (jsdom), i18next 27 + react-i18next 18, `@tauri-apps/api` 2, tauri-specta rc.29; **yeni (yalnızca geliştirme):** `@playwright/test` `1.65.1` (Apache-2.0; WebKit 2388), `@types/node` `24.x` (MIT; yalnızca `e2e/` tip denetimi). Rust tarafında bağımlılık değişmez (`atlas` zaten uygulamanın bağımlılığıdır).

**Spec:** `docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md` §4 ("Kalıcılık", "Danışan açma"), §5 (`BirthData`, `ViewState`), §7 (DST hataları, `{code, params}`), §11 (M5); `docs/superpowers/specs/2026-11-24-ui-design.md` §2.3, §3.3, §3.4, §4, §5, §6, §8.

## Global Constraints

- Lisans: AGPL-3.0-or-later. Yeni npm geliştirme bağımlılıkları: `@playwright/test` 1.65.1 (Apache-2.0), `@types/node` 24 (MIT). Rust bağımlılığı değişmez; bu yüzden `cargo deny` bu planda zorunlu değildir (Task 14 yine de çalıştırır).
- "Kala", "Kāla", "Kaala", "Cala" ve türevleri hiçbir isimde, yorumda, metinde ya da dosya adında kullanılmaz. Türkçe sözcükler de kapsamdadır ("kalan", "yakalama", "şakalar", "vakalar", "kalabilir"): eklenen satırlarda `k(a|ā|aa)la` ve `\bcala\b` bulunmaz. Her commit adımı eklenen satırları `grep -inE 'k(a|ā|aa)la|\bcala\b'` ile tarar; çıktı boş olmalıdır.
- `pnpm --dir app test` ve `pnpm --dir app build` her görev sonunda yeşildir. `tsc` `strict`; `build` artık `e2e/` için de `tsc -p e2e` çalıştırır (Task 13).
- Rust değişen görevlerde (1, 2, 3, 5): `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, ilgili `cargo test`; kütüphane (`store`) için MSRV 1.93 `cargo check`. IPC tipi değişen görevde `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings` ile `app/ui/bindings.ts` yeniden üretilir ve commit'e eklenir (elle düzenlenmez).
- **Toolchain:** `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"` (oturum başına bir kez). MSRV: `PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p store -p jyotish-desk --all-targets`. Node 24+ (yerelde 26), pnpm 11.8.0.
- **Arayüz hesap yapmaz** (spec §4): burç, derece, nakshatra, pada, yanma, vargottama, dasha dönemleri ve an çözümü Rust'tan gelir. Arayüz yalnızca gruplar, süzer ve biçimler.
- **Renk yalnızca token'larla** (`app/ui/theme/tokens.css`); bileşenlerde sabit renk yok. **Renk tek başına anlam taşımaz:** rozet metinli ve ▲ işaretli, retro `℞` + gizli "retro" metni, yaklaşık değer `≈` + gizli "yaklaşık" metni, alan hatası metinli. Metin kontrastı WCAG AA: `--warn` metin rengi olarak kullanılmaz (yalnızca kenar/işaret, ≥ 3:1), hata metni `--danger` (AA, `tokens.test.ts` denetler).
- **Tam klavye:** her işlem klavyeyle yapılır; odak halkası 2 px `--ink`. Modal pencerede odak tuzağı ve kapanınca odağın geri dönüşü; modal açıkken genel kısayollar kapalıdır.
- **Kişisel veri loglanmaz:** `CommandError.params` ve form değerleri `console`'a yazılmaz; e2e IPC taklidi kayıtları yalnızca test sayfasının belleğindedir. `app/ui/no-console.test.ts` (Task 4) arayüz kodunda `console.` çağrısı olmadığını denetler; izinli tek istisna Plan 04'ün tema/dil tercihi kayıt uyarılarıdır (kişisel veri yok).
- **Plan 08'den taşınan kurallar:** `CommandError` parametreleri loglanmaz; içe/dışa aktarma komutları (`import_json`, `import_jhd`, `export_json`, `export_csv`) ertelenen madde 1 ve 3 (M6) kapanana kadar arayüzden erişilemez — `api.ts` bunları sarmaz ve bir test bunu denetler (Task 3).
- Commit mesajları İngilizce, `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` satırıyla biter. `.superpowers/` asla `git add` edilmez.
- Plan kodları `cargo fmt` / mevcut biçim düzeniyle yazılmıştır; farklar (`diff`) kopyada uygulanan commit'lerden alınmıştır. Bir `diff` bağlamı tutmazsa önceki görevin adımını denetle; elle "düzeltme" yapma.

### Bu planın kararları

| # | Konu | Karar | Gerekçe |
|---|---|---|---|
| D1 | Plan 04'ün kurdukları ve yeniler | **Hazır (değişmeden kullanılır):** pano ızgarası, `PanelFrame` (satır içi yükleme göstergesi), `LayoutPicker`, düzen modeli/reducer/ızgara, `ChartSvg` + `geometry.ts` (kuzey, güney, daire hücreleri ve etiket yerleşimi), vurgu deposu, tema token'ları, i18n altyapısı, panel kaydı, `PrintSpike`. **Değişen:** `ChartSvg` lagnasız çizim (Task 6), düzen reducer'ına `setPanelSettings` (Task 6), `layout/store.tsx` + `layout/persistence.ts` → `view/` (Task 4). **Yeni:** oturum ve açılış, bildirimler ve rozetler, gerçek haritadan çizim, varga/stil seçimi, tam graha tablosu, dasha ağacı, kısayol kaydı ve yardım, doğum formu, yer arama, e2e | Plan 04 iskeleti sağlam; M5 veriyi bağlar |
| D2 | Doğum girişi: `createClient`/`updateChart` mı yerel mi | **Store'a bağlı.** Yeni harita `create_client`, düzenleme (`Cmd+E`) `update_chart` ile kaydedilir; dönen `OpenedClient` açılır. Açılışta `restore_session` + `load_panel_layout`. Yalnızca bellekte harita yok | Store M4'te hazır; iki yol yazmak gereksiz. Plan 08 incelemesi §2.5'in bu ana bıraktığı test boşlukları (DST boşluğunda haritanın değişmediği, `save_view_state`'in geri yüklemeden sağ çıktığı, düzen kaydı) Task 3'te `tests/open_client.rs`'te kapanır. Bilinen sonuç: M5'te oluşturulan her danışana M6 danışan listesine kadar yalnızca oturum geri yüklemesiyle ulaşılır. Danışan listesi, `Cmd+K`, ikinci harita (`add_chart`), tzdb farkını kabul ve notlar M6'dadır |
| D3 | Yer arama komutu | `search_places(query, limit) -> Vec<PlaceHit>`: `Atlas::search` üzerine ince katman; sorgu kırpılır, 100 karakterde kesilir, `limit` 20'ye kırpılır; boş sorgu boş liste. `PlaceHit.geoname_id` `u32` (specta `i64` reddeder; sığmayan atlanır). Atlas `bundle.resources` ile `atlas/atlas.sqlite` olarak paketlenir; açılamazsa uygulama açılır, komut `atlas_unavailable` döner, elle koordinat çalışır | spec "manuel koordinat birinci sınıf"; atlas dosyası bozuk/eksikse açılış engellenmez |
| D4 | An önizleme komutu | `preview_moment(birth) -> MomentPreview{birth, resolved}`: `store::resolve_birth`'ü çağırır, store kilidi almaz, senkron. Hata kodları `create_client` ile aynıdır (`nonexistent_local_time{local,suggestion}`, `ambiguous_local_time`, `invalid_input{field}`) | Form canlı önizleme ister; kayıt komutu yan etkili ve yavaştır (motor) |
| D5 | Harita yokken düzen kaydı | `store::Store::save_panel_layout(Option<&PanelLayout>)` (yeni `layout.rs`) ve `load_panel_layout`/`save_panel_layout` komutları; `None` kaydı siler. **Plan 08 ertelenen madde 2 kapanır** | `save_view_state` bir `chart_id` ister; boş masada düzen değişikliği de kalıcı olmalı |
| D6 | Düzen ayrıştırıcıları | Rust ve TS ayrıştırıcıları aynı sınırları uygular: `MAX_LAYOUTS = 64`, `MAX_PANELS = 48`; Rust tam sayı alanlarda tam değerli float'ı (`3.0`, `1e1`) kabul eder; TS tam sayıları `u32` aralığıyla sınırlar. Ortak eşlik fikstürü `app/ui/layout/fixtures/layout-parity.json` iki tarafta test edilir. Sınır hazır düzenleri de sayar ve **yazarken de uygulanır**: arayüz hiçbir zaman `MAX_LAYOUTS`'tan fazla düzen yazmaz (reducer `saveAs`/`duplicate` sınırda durur, `LayoutPicker` iki düğmeyi ipucuyla kapatır), bu yüzden Rust okurken hiçbir şey atmaz. **Plan 08 ertelenen madde 12 kapanır** | İki ayrıştırıcının ayrışması sessiz veri kaybıdır |
| D7 | Plan 08 ertelenen madde 9 | **Dokunulmaz.** `charts.rs` ve `open.rs` değişmez (yeni `layout.rs` modülü bunun için); madde, bu dosyaları değiştiren ilk plana kalır | Kapsam dışı dosyalara dokunmamak |
| D8 | M0 deneme kodu | `planet_longitudes` komutu, `PlanetRow`, `GRAHAS`, `planet_rows`, sahte `Ephemeris` ve 4 testi silinir (Task 5); `sky.tsx`, `time.ts` silinir. `PrintSpike` kendi satır tipini (`SpikeRow`, burç Rust'tan) alır | Arayüz artık gerçek haritayı çizer; "şimdiki gökyüzü" yanıltıcıdır |
| D9 | Merkezi `ViewState` deposu | `view/model.ts` (`ViewModel`, `initialViewModel`, `toViewState`, `parseDashaPath`), `view/reducer.ts`, `view/store.tsx` (`ViewStateProvider`, `useViewStore`, `useLayoutStore`), `view/persistence.ts` (`storePersistence`, `memoryPersistence`, `loadLayout`). Kayıt: harita açıksa `save_view_state(chartId, toViewState(m))`, değilse `save_panel_layout`. `openChart` bekleyen kaydı önceki haritaya yazar, sonra yeni haritanın yolunu yükler. Kayıt `pagehide`, `visibilitychange: hidden` ve unmount'ta boşaltılır | Spec §11: "ViewState baştan tek merkezden yönetilir ve serileştirilebilir" |
| D10 | Dasha açık dal modeli | **Tek açık yol** (akordeon): bir düğümü açmak kardeşlerini kapatır; yol `ViewState.dasha_expanded_path`'tir ve haritayla kaydedilir. Hazır düzenlerdeki `settings.open` yok sayılır. 1–3. seviye haritayla gelir; 4–5. seviye `dasha_children` ile istenerek yüklenir, harita kimliği + tohum başına önbelleklenir (`update_chart` kimliği korur, tohumu değiştirir); yanıt yalnızca istenen yolun bir alt seviyesiyse kabul edilir | `ViewState`'in tek dasha alanı bir yoldur; yol modeli kaydı ve klavye gezintisini basit tutar |
| D11 | Saat bilinmiyor | `ascSign = null`: 1. ev Koç çizilir, lagna işareti ve dolgusu yok, evler burç adıyla anılır; tabloda lagna satırı ve ev sütunu gizli; Ay'ın saate bağlı değerleri (burç, derece, boylam, nakshatra, pada) `≈`; dasha panelinde "tarihler yaklaşık" notu; harita panelinde açıklama notu, D1 dışındaki vargalarda ek olarak "bölüm haritası yerleşimleri saate bağlı ve yaklaşık" (spec §7: D9'da Ay, D30–D60'ta her graha saate bağlıdır) | Spec §7: öğlen hesabı yanıltmamalı |
| D12 | Varga işaretleri | Retro her vargada; yanma yalnızca D1; vargottama D1 ve D9'da (`chartDataFromChart`) | Yanma bir boylam ilişkisidir (D1); vargottama D1–D9 eşliğidir |
| D13 | Dasha tarihleri | Doğumun çözülmüş sabit ofsetinde (`resolved.offset_secs`) `YYYY-MM-DD`; JD → tarih yalnızca gösterim aritmetiğidir (`formatJdDate`) | Danışanın takvimi; tz kuralları dönem sınırlarında yanıltıcı olabilir, tek ofset tutarlı |
| D14 | Uyarıların gösterimi | An ve harita uyarıları (`MomentWarning`, `ChartWarning`) üst çubukta rozet: amber kenar + ▲ işareti + metin etiketi + `title` ipucu + ekran okuyucu için gizli ipucu; odaklanabilir (`tabIndex=0`). Rozet metni `--text` (`--warn` AA vermez). `StoreWarning` (son harita silinmiş, ayar sıfırlandı, tzdb değişti) modal olmayan bildirim bandında (`role=status`); açılış ve kayıt hataları aynı bantta (`role=alert`), kapatılabilir. Doğum formu kaydedince `boot` ve `store:*` bildirimleri kalkar, dönen `OpenedClient.warnings` gösterilir (eski haritanın uyarısı yeni haritada kalmaz) | UI spec §5; renk tek başına anlam taşımaz |
| D15 | Graha tablosu sütunları | `sign, degree, lon, lat, speed, retro, nakshatra, pada, combust, dignity, house`; varsayılan `sign, degree, nakshatra, pada, retro, combust, dignity, house`. Seçim `settings.columns`'a yazılır (kanonik sıra, tekrarsız, son sütun kapatılamaz); `<details>` + onay kutuları. Lagna satırı vardır. Nakshatra tam adı (Sanskritçe çevriyazı, üç dilde aynı) | UI spec §3.3; "avastha" çekirdekte yok (ertelendi) |
| D16 | Kısayollar | Tek kaynak `app/ui/shortcuts.ts` (`matchShortcut`, `SHORTCUT_LIST`); yardım penceresi `?` listeyi buradan çizer. Yeni: `Cmd+N` (yeni harita), `Cmd+E` (düzenle), `?`. Mevcut: `Cmd+1…9` (fiziksel tuş), `Cmd+Shift+N`, `Esc`, `Alt+ok`, `Alt+Shift+ok`; dasha ağacı WAI-ARIA ağaç tuşları. Yazı alanında, tekrar, IME ve modal açıkken genel kısayol yok | UI spec §6; dağınık dinleyiciler birleşir |
| D17 | Doğum formu | Sağdan kayan `role=dialog` (`aria-modal`, odak tuzağı, Esc). Alanlar: ad (düzenlemede kilitli), etiket, tür, tarih, saat, saat kesinliği (kesin/yaklaşık/bilinmiyor), ARIA combobox yer araması (200 ms debounce, eski yanıt yok sayılır), elle enlem/boylam, saat dilimi kipi (IANA / sabit `+05:30`, işaret zorunlu / LMT), DST boşluğunda satır içi "04:30 kullan" düğmesi, örtüşmede "ilk kez / ikinci kez" seçimi, kaynak notu. Biçim denetimi istemcide (`toBirthData`), anlam denetimi Rust'ta; önizleme 300 ms debounce; `invalid_input.field` → alan. Hata metni blur'da görünür, düzeltilince kalkar | UI spec §4; "hiçbir zaman sessizce kaydırılmaz" |
| D18 | Boş durum | Harita yokken pano yerine tek eylem "Yeni harita" (`Cmd+N`); üst çubukta "Yeni harita" her zaman, "Düzenle" harita açıkken | UI spec §5 "Boş" |
| D19 | Görsel testler | (a) **Vitest dosya anlık görüntüsü:** Delhi D1 SVG işaretlemesi, üç stil (`toMatchFileSnapshot`, `app/ui/chart/__snapshots__/*.svg`), temadan bağımsız, CI'da. (b) **Playwright WebKit ekran görüntüsü:** 3 stil × açık/koyu, `pnpm e2e:visual`, **yalnızca yerel** (temel görüntüler `-darwin` son ekli, depoda) | Renkler CSS token'larında: işaretleme testi stil ve yerleşimi, ekran görüntüsü renkleri denetler; font yığını makineye bağlı |
| D20 | Playwright kapsamı ve indirme | `@playwright/test` 1.65.1, yalnızca WebKit (Tauri'nin macOS motoru). Vite geliştirme sunucusu `:1421`, Tauri IPC `window.__TAURI_INTERNALS__.invoke` taklidi (`e2e/ipc.ts`, gerçek veri fikstürü). `pnpm e2e` CI'da (`core-macos`): doğum girişi, `Cmd+2` düzen değişimi, klavyeyle panel taşıma ve kayıt, vurgu yayılımı, stil değişimi, dasha klavyesi ve tembel yükleme. **İndirme:** CI'da `playwright install webkit` (~70 MB), `actions/cache` ile `~/Library/Caches/ms-playwright` önbelleklenir (anahtar: `app/pnpm-lock.yaml` özeti, yani Playwright sürümü); yerelde önbellekte WebKit 2388 yoksa aynı komut | UI spec §8; gerçek uygulamaya karşı e2e (tauri-driver) macOS'ta desteklenmez |
| D21 | Gerçek veri fikstürü | `app/src-tauri/tests/ui_fixture.rs` gerçek motorla Delhi 2000-01-01 17:30 için `create_client` dönüşünü, ilk 3. seviye dönemin alt dönemlerini, "new delhi" aramasını, önizlemeyi ve İstanbul DST boşluğu hatasını `app/ui/test/fixtures/delhi.json`'a yazar (`UPDATE_FIXTURES=1`; klasör yoksa oluşturur), yoksa sayılarda göreli 1e-9 toleranslı yapısal eşitlik denetler (CI'daki farklı derleyici/libm son basamağı değiştirebilir). UUID'ler sabitlenir, saat sabittir | Arayüz testleri elle uydurulmuş veri yerine motor çıktısını çizer; şema kayması `cargo test`'te görülür |
| D22 | Hata metinleri | Arayüzden ulaşılabilen her `CommandError.code` için tr/en/fr metni; bir test kod listesini `tr.errors`'a karşı denetler. Parametreler i18next seçeneklerini değiştiremez (`safeParams`) | Spec §7 `{code, params}` |

### Spec belirsizlikleri ve çözümleri

- **Nakshatra kısaltmaları (UI spec §6):** M5'te harita nakshatra göstermez; tablo tam adı gösterir (Sanskritçe çevriyazı tr/en/fr'de aynıdır). Kısaltma seti, nakshatra'yı haritaya taşıyan planda eklenir.
- **Rozetlerin yeri (UI spec §5: "doğum bilgisi panelinde ve harita başlığında"):** doğum bilgisi paneli M6'dadır; M5'te rozetler üst çubuktaki harita başlığındadır.
- **"Açık dallar" ayarı (UI spec §3.3 `dasha_tree`):** `ViewState` yalnızca tek yol taşır (D10); panel ayarı `open` yok sayılır ve korunur (bilinmeyen ayarlar silinmez).
- **Dasha seçiminin transit tablosuna yansıması (UI spec §3.4):** transit tablosu M6'dadır.
- **Saat dilimi seçimi:** IANA adı serbest metindir (atlas sonucu doldurur); IANA listesi açılır menüsü yoktur. Geçersiz ad `unknown_time_zone` ile alana bağlanır.

## Dosya Yapısı (bu plan)

```
app/src-tauri/
  src/atlas_commands.rs        (yeni) search_places, preview_moment, AtlasState
  src/commands.rs              planet_longitudes ve PlanetRow silinir
  src/lib.rs                   komut kaydı, atlas durumu
  src/store_commands.rs        load_panel_layout, save_panel_layout
  tauri.conf.json              atlas kaynağı
  tests/ui_fixture.rs          (yeni) gerçek veri fikstürü üreticisi
crates/store/src/
  layout.rs                    (yeni) Store::save_panel_layout
  view_state.rs                MAX_LAYOUTS, MAX_PANELS, tam değerli float, eşlik testi
app/
  package.json, playwright.config.ts, e2e/{ipc.ts,desk.spec.ts,visual.spec.ts,tsconfig.json}
  ui/api.ts                    M5 komut sarmalayıcıları (içe/dışa aktarma yok)
  ui/boot.ts                   açılış: düzen + son oturum
  ui/shortcuts.ts              kısayol kaydı
  ui/view/{model,reducer,persistence}.ts, store.tsx   merkezi ViewState
  ui/session/store.tsx         açık harita
  ui/shell/{notices,ChartTitle,Dialog,HelpDialog,EmptyState}.tsx
  ui/chart/adapter.ts          Chart → ChartData (varga, işaretler, lagna)
  ui/panels/{ChartPanel,GrahaTablePanel}.tsx, grahaColumns.ts, labels.ts, format.ts
  ui/dasha/{tree.ts,useLazyDasha.ts,DashaTreePanel.tsx,dasha.css}
  ui/birth/{model.ts,PlaceSearch.tsx,BirthForm.tsx,birth.css}
  ui/i18n/{tr,en,fr}.ts, errors.ts
  ui/test/{fixture.ts,render.tsx,fixtures/delhi.json}
.github/workflows/ci.yml       e2e adımı
```

---

### Task 1: Atlas yer araması ve an önizleme komutları

Doğum formunun iki Rust ucu (D3, D4). Arama `atlas::Atlas::search` üzerine ince katmandır; atlas dosyası `tauri.conf.json` `bundle.resources` ile paketlenir ve açılamazsa uygulama yine açılır (`AtlasState::unavailable`). Önizleme `store::resolve_birth`'ü kaydetmeden çağırır. Testler modülün `#[cfg(test)] mod tests` bölümündedir ve depodaki gerçek `data/atlas/atlas.sqlite`'ı kullanır.

**Files:**
- Create: `app/src-tauri/src/atlas_commands.rs`
- Modify: `app/src-tauri/src/lib.rs`
- Modify: `app/src-tauri/tauri.conf.json`
- Modify: `app/ui/bindings.ts` (üretilir: `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings`)

**Interfaces:**
- Consumes: `atlas::{Atlas, AtlasError}` (Plan 03), `store::{resolve_birth, BirthData, ResolvedRecord}` (Plan 08), `client_commands::blocking`, `commands::CommandError`.
- Produces: `atlas_commands::{AtlasState, PlaceHit, MomentPreview, search_places_blocking(&AtlasState, &str, u32) -> Result<Vec<PlaceHit>, CommandError>, preview_moment_body(&BirthData) -> Result<MomentPreview, CommandError>, ATLAS_RESOURCE, MAX_PLACE_HITS = 20, MAX_QUERY_CHARS = 100}`; IPC komutları `search_places(query: String, limit: u32)` ve `preview_moment(birth: BirthData)`; TS `commands.searchPlaces`, `commands.previewMoment`, tipler `PlaceHit`, `MomentPreview`. Yeni hata kodları: `atlas_unavailable`, `atlas_sqlite`, `atlas_io`, `atlas_schema`.

- [ ] **Step 1: Uygulamayı yaz**

Önce `atlas_commands.rs` dosyasının sonundaki `mod tests` bölümünü yaz ve `lib.rs`'e `pub mod atlas_commands;` ekle; gövdeler yokken `cargo test -p jyotish-desk --lib atlas_commands` derleme hatasıyla (`cannot find function search_places_blocking`) düşer. Sonra dosyanın tamamını ve `lib.rs` kaydını yaz.

`app/src-tauri/src/atlas_commands.rs` oluştur:

```rust
//! Doğum formu komutları (M5): atlas yer araması ve kaydetmeden an önizlemesi. İnce
//! katman: arama `atlas`'ta, an çözümü `store::resolve_birth`'te; burada yalnızca sınırlar
//! ve hata eşleme var.

use std::path::Path;
use std::sync::{Arc, Mutex};

use atlas::{Atlas, AtlasError};
use serde::Serialize;
use specta::Type;
use store::{BirthData, ResolvedRecord};
use tauri::State;

use crate::client_commands::blocking;
use crate::commands::CommandError;

/// Paketlenen atlas dosyasının kaynak yolu (`tauri.conf.json` `bundle.resources`).
pub const ATLAS_RESOURCE: &str = "atlas/atlas.sqlite";
/// Bir aramada dönen en çok yer.
pub const MAX_PLACE_HITS: u32 = 20;
/// Arama metninin en çok uzunluğu (karakter); fazlası kesilir.
pub const MAX_QUERY_CHARS: usize = 100;

/// Atlas durumu. Dosya açılamazsa uygulama yine açılır: yer araması `atlas_unavailable`
/// döner, elle koordinat girişi çalışır (spec §12 "manuel koordinat birinci sınıf").
pub struct AtlasState(Option<Mutex<Atlas>>);

impl AtlasState {
    pub fn open(path: &Path) -> Self {
        Self(Atlas::open(path).ok().map(Mutex::new))
    }

    pub fn unavailable() -> Self {
        Self(None)
    }

    pub fn is_available(&self) -> bool {
        self.0.is_some()
    }
}

/// Arama sonucu: formun dolduracağı alanlar.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct PlaceHit {
    pub geoname_id: u32,
    pub name: String,
    pub admin1: String,
    pub country: String,
    pub lat: f64,
    pub lon: f64,
    /// IANA kimliği; form saat dilimini buradan önerir.
    pub timezone: String,
}

/// Kaydetmeden çözülmüş an: kanonik doğum verisi ve `ResolvedRecord`.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct MomentPreview {
    pub birth: BirthData,
    pub resolved: ResolvedRecord,
}

impl From<AtlasError> for CommandError {
    fn from(e: AtlasError) -> Self {
        match e {
            AtlasError::Sqlite(e) => Self::new("atlas_sqlite", &[("detail", e.to_string())]),
            AtlasError::Io(e) => Self::new("atlas_io", &[("detail", e.to_string())]),
            AtlasError::Geonames(e) => Self::new("atlas_io", &[("detail", e.to_string())]),
            AtlasError::SchemaMismatch { found, expected } => Self::new(
                "atlas_schema",
                &[
                    ("found", found.unwrap_or_default()),
                    ("expected", expected.to_owned()),
                ],
            ),
        }
    }
}

/// Önek araması (`Atlas::search`). Boş sorgu boş liste; `limit` `MAX_PLACE_HITS`'e kırpılır.
pub fn search_places_blocking(
    atlas: &AtlasState,
    query: &str,
    limit: u32,
) -> Result<Vec<PlaceHit>, CommandError> {
    let Some(atlas) = &atlas.0 else {
        return Err(CommandError::new("atlas_unavailable", &[]));
    };
    let query: String = query.trim().chars().take(MAX_QUERY_CHARS).collect();
    let limit = limit.min(MAX_PLACE_HITS) as usize;
    if query.is_empty() || limit == 0 {
        return Ok(Vec::new());
    }
    let places = atlas
        .lock()
        .map_err(|_| CommandError::new("atlas_unavailable", &[]))?
        .search(&query, limit)?;
    Ok(places
        .into_iter()
        .filter_map(|p| {
            Some(PlaceHit {
                geoname_id: u32::try_from(p.geoname_id).ok()?,
                name: p.name,
                admin1: p.admin1,
                country: p.country,
                lat: p.lat,
                lon: p.lon,
                timezone: p.timezone,
            })
        })
        .collect())
}

/// Doğum verisini kaydetmeden çözer (DST boşluğu ve örtüşmesi `create_client` ile aynı
/// hata kodlarıyla döner). Saf ve hızlıdır; store kilidi alınmaz.
pub fn preview_moment_body(birth: &BirthData) -> Result<MomentPreview, CommandError> {
    let (birth, resolved) = store::resolve_birth(birth)?;
    Ok(MomentPreview { birth, resolved })
}

#[tauri::command]
#[specta::specta]
pub async fn search_places(
    atlas: State<'_, Arc<AtlasState>>,
    query: String,
    limit: u32,
) -> Result<Vec<PlaceHit>, CommandError> {
    let atlas = Arc::clone(&atlas);
    blocking(move || search_places_blocking(&atlas, &query, limit)).await
}

#[tauri::command]
#[specta::specta]
pub fn preview_moment(birth: BirthData) -> Result<MomentPreview, CommandError> {
    preview_moment_body(&birth)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use store::{Place, TimeCertainty, TzChoice};

    fn shipped_atlas() -> AtlasState {
        let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/atlas/atlas.sqlite");
        let state = AtlasState::open(&p);
        assert!(state.is_available(), "data/atlas/atlas.sqlite must open");
        state
    }

    fn birth(date: &str, time: Option<&str>, tz: TzChoice) -> BirthData {
        BirthData {
            date: date.into(),
            time: time.map(Into::into),
            time_certainty: if time.is_some() {
                TimeCertainty::Exact
            } else {
                TimeCertainty::Unknown
            },
            place: Place {
                name: "New Delhi".into(),
                country: "India".into(),
                admin1: "NCT".into(),
                lat: 28.6139,
                lon: 77.2090,
                geoname_id: Some(1_261_481),
            },
            tz,
            fold: None,
            notes: String::new(),
        }
    }

    #[test]
    fn search_finds_new_delhi_with_its_time_zone_and_clamps_the_limit() {
        let atlas = shipped_atlas();
        let hits = search_places_blocking(&atlas, "  new delhi ", 5).unwrap();
        let first = hits.first().expect("a hit");
        assert_eq!(first.name, "New Delhi");
        assert_eq!(first.timezone, "Asia/Kolkata");
        assert!((first.lat - 28.6).abs() < 0.1 && (first.lon - 77.2).abs() < 0.1);
        assert!(hits.len() <= 5);
        let many = search_places_blocking(&atlas, "san", 500).unwrap();
        assert_eq!(many.len(), MAX_PLACE_HITS as usize);
    }

    #[test]
    fn empty_query_and_zero_limit_return_nothing_and_long_queries_are_cut() {
        let atlas = shipped_atlas();
        assert!(search_places_blocking(&atlas, "   ", 8).unwrap().is_empty());
        assert!(search_places_blocking(&atlas, "delhi", 0)
            .unwrap()
            .is_empty());
        let long = "a".repeat(10_000);
        assert!(search_places_blocking(&atlas, &long, 8).unwrap().is_empty());
    }

    #[test]
    fn missing_atlas_is_reported_not_fatal() {
        let state = AtlasState::open(Path::new("/nonexistent/atlas.sqlite"));
        assert!(!state.is_available());
        let e = search_places_blocking(&state, "delhi", 8).unwrap_err();
        assert_eq!(e.code, "atlas_unavailable");
        assert!(!AtlasState::unavailable().is_available());
    }

    #[test]
    fn preview_resolves_without_a_store() {
        let p = preview_moment_body(&birth(
            "2000-01-01",
            Some("17:30"),
            TzChoice::Iana("Asia/Kolkata".into()),
        ))
        .unwrap();
        assert_eq!(p.resolved.jd_ut, 2_451_545.0);
        assert_eq!(p.resolved.offset_secs, 19_800);
        assert_eq!(p.resolved.abbrev, "IST");
        assert_eq!(p.birth.time.as_deref(), Some("17:30:00"));
        let noon =
            preview_moment_body(&birth("2000-01-01", None, TzChoice::Fixed(19_800))).unwrap();
        assert_eq!(
            noon.resolved.warnings,
            [store::MomentWarning::TimeUnknownNoon]
        );
    }

    #[test]
    fn preview_reports_dst_gap_and_overlap_with_store_error_codes() {
        let mut gap = birth(
            "2014-03-31",
            Some("03:30"),
            TzChoice::Iana("Europe/Istanbul".into()),
        );
        gap.place.lon = 28.94966;
        let e = preview_moment_body(&gap).unwrap_err();
        assert_eq!(e.code, "nonexistent_local_time");
        assert_eq!(e.params["suggestion"], "2014-03-31T04:30:00");
        let mut overlap = birth(
            "2014-10-26",
            Some("03:30"),
            TzChoice::Iana("Europe/Istanbul".into()),
        );
        overlap.place.lon = 28.94966;
        let e = preview_moment_body(&overlap).unwrap_err();
        assert_eq!(e.code, "ambiguous_local_time");
        overlap.fold = Some(store::FoldChoice::Later);
        assert_eq!(
            preview_moment_body(&overlap).unwrap().resolved.offset_secs,
            7_200
        );
        let mut bad = birth("2000-13-01", Some("10:00"), TzChoice::Lmt);
        assert_eq!(
            preview_moment_body(&bad).unwrap_err().params["field"],
            "birth.date"
        );
        bad.date = "2000-01-01".into();
        bad.place.lat = 91.0;
        assert_eq!(
            preview_moment_body(&bad).unwrap_err().params["field"],
            "birth.place.lat"
        );
    }

    #[test]
    fn the_atlas_is_bundled_at_the_resource_path() {
        let conf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(conf).unwrap()).unwrap();
        assert_eq!(
            v["bundle"]["resources"]["../../data/atlas/atlas.sqlite"],
            ATLAS_RESOURCE
        );
    }

    #[test]
    fn atlas_error_codes_are_stable() {
        let e = CommandError::from(AtlasError::SchemaMismatch {
            found: Some("0".into()),
            expected: "1",
        });
        assert_eq!(
            (e.code.as_str(), e.params["found"].as_str()),
            ("atlas_schema", "0")
        );
        let e = CommandError::from(AtlasError::Io(std::io::Error::other("x")));
        assert_eq!(e.code, "atlas_io");
    }
}
```

`app/src-tauri/src/lib.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -1,3 +1,4 @@
+pub mod atlas_commands;
 pub mod client_commands;
 pub mod commands;
 pub mod store_commands;
@@ -49,6 +50,8 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             store_commands::export_csv,
             store_commands::import_json,
             store_commands::import_jhd,
+            atlas_commands::search_places,
+            atlas_commands::preview_moment,
         ])
 }
 
@@ -70,6 +73,12 @@ pub fn run() {
             let db = store::Store::open(&data_dir.join(client_commands::DB_FILE_NAME))?;
             let state: client_commands::StoreState = Arc::new(Mutex::new(db));
             app.manage(state);
+            // Atlas ölümcül değildir: açılamazsa yer araması `atlas_unavailable` döner, elle
+            // koordinat girişi çalışır.
+            let atlas_path = app
+                .path()
+                .resolve(atlas_commands::ATLAS_RESOURCE, BaseDirectory::Resource)?;
+            app.manage(Arc::new(atlas_commands::AtlasState::open(&atlas_path)));
             Ok(())
         })
         .run(tauri::generate_context!())
```

`app/src-tauri/tauri.conf.json` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/src-tauri/tauri.conf.json
+++ b/app/src-tauri/tauri.conf.json
@@ -17,7 +17,7 @@
     "active": true,
     "targets": ["app", "dmg"],
     "icon": ["icons/32x32.png", "icons/128x128.png", "icons/128x128@2x.png", "icons/icon.icns", "icons/icon.png"],
-    "resources": { "../../data/ephe/": "ephe/" },
+    "resources": { "../../data/ephe/": "ephe/", "../../data/atlas/atlas.sqlite": "atlas/atlas.sqlite" },
     "macOS": { "minimumSystemVersion": "13.0" }
   }
 }
```


- [ ] **Step 2: Bağlamaları yeniden üret**

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings
git diff --stat app/ui/bindings.ts
cargo test -p jyotish-desk --test bindings
```
Expected: ilk komut `app/ui/bindings.ts`'i yazar; ikinci çalıştırma `test result: ok. 1 passed` (dosya güncel). `bindings.ts` elle düzenlenmez.

- [ ] **Step 3: Testlerin geçtiğini gör**

```bash
cargo test -p jyotish-desk --lib atlas_commands
cargo test -p jyotish-desk --lib
```
Expected: `test result: ok. 7 passed` (atlas_commands), kütüphanenin tamamı `17 passed`.
```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```
Expected: ikisi de çıktısız, çıkış kodu 0.
```bash
pnpm --dir app test
pnpm --dir app build
```
Expected: tam takımda `Test Files  23 passed (23)`, `Tests  203 passed (203)`; `build` hatasız (`tsc --noEmit` çıktısız, `vite build` `✓ built`).

- [ ] **Step 4: Commit**

```bash
git add -A app/src-tauri/src/atlas_commands.rs app/src-tauri/src/lib.rs app/src-tauri/tauri.conf.json app/ui/bindings.ts
git diff --cached -U0 -- . ':!app/pnpm-lock.yaml' ':!*.png' ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git commit -m "feat(app): atlas place search and moment preview commands for the birth form

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Store: harita olmadan düzen kaydı, düzen sınırları ve eşlik fikstürü

Harita açık değilken düzen değişikliği de kalıcı olmalıdır (D5; Plan 08 ertelenen madde 2). Yeni `crates/store/src/layout.rs` `charts.rs`/`open.rs`'e dokunmadan `app_state`'e yazar (D7). Rust ve TS düzen ayrıştırıcıları aynı sınırları (`MAX_LAYOUTS = 64`, `MAX_PANELS = 48`) uygular ve ortak fikstürle denetlenir (D6; madde 12). Sınır yazarken de uygulanır: hazır düzenler sınırın içindedir, reducer `saveAs`/`duplicate` sınırda durumu değiştirmez ve `LayoutPicker` iki düğmeyi ipucuyla kapatır; böylece arayüz Rust'ın okuyacağından fazlasını yazmaz ve yeniden açılışta düzen kaybolmaz. Rust testleri modüllerin içindedir; TS tarafı `layout/model.test.ts`'tedir.

**Files:**
- Modify: `app/src-tauri/src/lib.rs`
- Modify: `app/src-tauri/src/store_commands.rs`
- Modify: `app/ui/bindings.ts` (üretilir: `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings`)
- Modify: `app/ui/i18n/en.ts`
- Modify: `app/ui/i18n/fr.ts`
- Modify: `app/ui/i18n/tr.ts`
- Create: `app/ui/layout/fixtures/layout-parity.json`
- Modify: `app/ui/layout/model.test.ts`
- Modify: `app/ui/layout/model.ts`
- Modify: `app/ui/layout/reducer.test.ts`
- Modify: `app/ui/layout/reducer.ts`
- Modify: `app/ui/shell/LayoutPicker.test.tsx`
- Modify: `app/ui/shell/LayoutPicker.tsx`
- Create: `crates/store/src/layout.rs`
- Modify: `crates/store/src/lib.rs`
- Modify: `crates/store/src/view_state.rs`

**Interfaces:**
- Consumes: `store::{Store, PanelLayout, parse_panel_layout}` (Plan 08), `app_state_get/app_state_set`.
- Produces: `Store::save_panel_layout(&mut self, Option<&PanelLayout>) -> Result<(), StoreError>`; `store::{MAX_LAYOUTS, MAX_PANELS}`; IPC `load_panel_layout() -> Option<PanelLayout>`, `save_panel_layout(layout: Option<PanelLayout>)`; TS `commands.loadPanelLayout`, `commands.savePanelLayout`, `MAX_LAYOUTS`, `MAX_PANELS` (`layout/model.ts`; `parseLayoutState` toplamı `MAX_LAYOUTS`'ta keser); reducer `saveAs`/`duplicate` sınırda `state`'i aynen döndürür; i18n `layout.limit`; fikstür `app/ui/layout/fixtures/layout-parity.json` (`cases[{name, input, expected}]`).

- [ ] **Step 1: Önce testleri yaz**

Rust testleri (`layout.rs`, `view_state.rs` içindeki `mod tests`) Step 3'teki dosyalarla birlikte yazılır; TS tarafı, ortak fikstür ve yazma tarafı sınır testleri burada:

`app/ui/layout/fixtures/layout-parity.json` oluştur:

```json
{
  "max_layouts": 64,
  "max_panels": 48,
  "cases": [
    {
      "name": "a valid user layout is kept as is",
      "input": {
        "active_layout_id": "user:a",
        "layouts": [
          {
            "id": "user:a",
            "name": "A",
            "builtin": false,
            "panels": [
              { "id": "p1", "kind": "chart", "x": 0, "y": 0, "w": 6, "h": 12, "settings": { "varga": "D9" } },
              { "id": "p2", "kind": "graha_table", "x": 6, "y": 0, "w": 6, "h": 9, "settings": {} }
            ]
          }
        ]
      },
      "layouts": [{ "id": "user:a", "panels": ["p1", "p2"] }]
    },
    {
      "name": "invalid panels are dropped, the first of duplicate ids wins, unknown kinds stay",
      "input": {
        "layouts": [
          {
            "id": "user:b",
            "name": "B",
            "panels": [
              { "id": "ok", "kind": "astro_clock", "x": 0, "y": 0, "w": 3, "h": 4 },
              { "id": "ok", "kind": "chart", "x": 3, "y": 0, "w": 3, "h": 4 },
              { "id": "neg", "kind": "chart", "x": -1, "y": 0, "w": 3, "h": 4 },
              { "id": "zero", "kind": "chart", "x": 0, "y": 0, "w": 0, "h": 4 },
              { "id": "half", "kind": "chart", "x": 1.5, "y": 0, "w": 3, "h": 4 },
              { "id": "big", "kind": "chart", "x": 4294967296, "y": 0, "w": 3, "h": 4 },
              { "id": "text", "kind": "chart", "x": 0, "y": "1", "w": 3, "h": 4 },
              { "id": " ", "kind": "chart", "x": 0, "y": 0, "w": 3, "h": 4 },
              { "id": "nokind", "kind": "", "x": 0, "y": 0, "w": 3, "h": 4 },
              { "id": "float", "kind": "chart", "x": 3.0, "y": 1e1, "w": 3, "h": 4 },
              "garbage"
            ]
          }
        ]
      },
      "layouts": [{ "id": "user:b", "panels": ["ok", "float"] }]
    },
    {
      "name": "invalid and duplicate layouts are dropped",
      "input": {
        "layouts": [
          { "id": "user:c", "name": "C", "panels": [] },
          { "id": "user:c", "name": "duplicate", "panels": [{ "id": "x", "kind": "chart", "x": 0, "y": 0, "w": 3, "h": 4 }] },
          { "id": "", "name": "no id" },
          { "id": "user:d", "name": "  " },
          { "id": "user:e", "name": "E", "panels": "not a list" },
          7
        ]
      },
      "layouts": [
        { "id": "user:c", "panels": [] },
        { "id": "user:e", "panels": [] }
      ]
    },
    {
      "name": "a builtin layout keeps the panels the user saved",
      "input": {
        "active_layout_id": "builtin:research",
        "layouts": [
          {
            "id": "builtin:research",
            "name": "Araştırma",
            "builtin": true,
            "panels": [{ "id": "only", "kind": "dasha_tree", "x": 0, "y": 0, "w": 6, "h": 9 }]
          }
        ]
      },
      "layouts": [{ "id": "builtin:research", "panels": ["only"] }]
    }
  ]
}
```

`app/ui/layout/model.test.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -4,12 +4,15 @@ import {
   defaultLayoutState,
   isKnownPanelKind,
   LAYOUT_SCHEMA_VERSION,
+  MAX_LAYOUTS,
+  MAX_PANELS,
   PANEL_KINDS,
   parseLayoutState,
   serializeLayoutState,
   type Layout,
   type Panel,
 } from "./model";
+import parityRaw from "./fixtures/layout-parity.json?raw";
 
 const ids = (layouts: Layout[]) => layouts.map((l) => l.id);
 const BUILTINS = [BUILTIN_LAYOUT_IDS.interview, BUILTIN_LAYOUT_IDS.research, BUILTIN_LAYOUT_IDS.transit];
@@ -168,3 +171,38 @@ test("a newer schema version is read best-effort", () => {
   expect(s.schema_version).toBe(LAYOUT_SCHEMA_VERSION);
   expect(s.active_layout_id).toBe("builtin:transit");
 });
+
+// Rust `store::parse_panel_layout` ile ortak fikstür (Plan 09): iki ayrıştırıcı aynı
+// düzenleri ve panelleri tutar. Arayüz ayrıca eksik hazır düzenleri ekler; onlar fikstürde
+// yoktur ve karşılaştırmaya girmez.
+interface ParityCase {
+  name: string;
+  input: unknown;
+  layouts: { id: string; panels: string[] }[];
+}
+const parity = JSON.parse(parityRaw) as { max_layouts: number; max_panels: number; cases: ParityCase[] };
+
+test("caps are the same numbers as the Rust store", () => {
+  expect([MAX_LAYOUTS, MAX_PANELS]).toEqual([parity.max_layouts, parity.max_panels]);
+});
+
+test.each(parity.cases.map((c) => [c.name, c] as const))("parity: %s", (_, c) => {
+  const keep = new Set(c.layouts.map((l) => l.id));
+  const got = parseLayoutState(c.input)
+    .layouts.filter((l) => keep.has(l.id) || !l.builtin)
+    .map((l) => ({ id: l.id, panels: l.panels.map((p) => p.id) }));
+  const byId = (a: { id: string }, b: { id: string }) => a.id.localeCompare(b.id);
+  expect(got.sort(byId)).toEqual([...c.layouts].sort(byId));
+});
+
+test("layout and panel counts are capped like the Rust store", () => {
+  const panels = Array.from({ length: MAX_PANELS + 10 }, (_, i) => ({ id: `p${i}`, kind: "chart", x: 0, y: i, w: 3, h: 4 }));
+  const layouts = Array.from({ length: MAX_LAYOUTS + 6 }, (_, i) => ({ id: `user:${i}`, name: "L", panels }));
+  const all = parseLayoutState({ layouts }).layouts;
+  const users = all.filter((l) => !l.builtin);
+  // Hazır düzenler sınırın içindedir: toplam `MAX_LAYOUTS`.
+  expect(all).toHaveLength(MAX_LAYOUTS);
+  expect(users).toHaveLength(MAX_LAYOUTS - 3);
+  expect(users.every((l) => l.panels.length === MAX_PANELS)).toBe(true);
+  expect(users[MAX_LAYOUTS - 4].id).toBe(`user:${MAX_LAYOUTS - 4}`);
+});
```

`app/ui/layout/reducer.test.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/layout/reducer.test.ts
+++ b/app/ui/layout/reducer.test.ts
@@ -1,5 +1,5 @@
 import { hasOverlap } from "./grid";
-import { BUILTIN_LAYOUT_IDS, builtinLayout, defaultLayoutState, type LayoutState } from "./model";
+import { BUILTIN_LAYOUT_IDS, builtinLayout, defaultLayoutState, MAX_LAYOUTS, type LayoutState } from "./model";
 import { activeLayout, layoutReducer, newId, NOTES_PANEL, normalizeState } from "./reducer";
 
 const { interview, research, transit } = BUILTIN_LAYOUT_IDS;
@@ -46,6 +46,14 @@ test("rename changes user layouts only", () => {
   expect(layoutReducer(s, { type: "rename", id: "user:1", name: "" })).toBe(s);
 });
 
+test("saveAs and duplicate stop at MAX_LAYOUTS so nothing is lost on the next read", () => {
+  let s = defaultLayoutState();
+  for (let i = s.layouts.length; i < MAX_LAYOUTS; i++) s = layoutReducer(s, { type: "saveAs", id: `user:${i}`, name: `L${i}` });
+  expect(s.layouts).toHaveLength(MAX_LAYOUTS);
+  expect(layoutReducer(s, { type: "saveAs", id: "user:x", name: "X" })).toBe(s);
+  expect(layoutReducer(s, { type: "duplicate", sourceId: research, id: "user:y", name: "Y" })).toBe(s);
+});
+
 test("duplicate inserts a copy right after its source without changing the active layout", () => {
   const s = layoutReducer(defaultLayoutState(), { type: "duplicate", sourceId: research, id: "user:2", name: "Araştırma kopyası" });
   expect(s.layouts.map((l) => l.id)).toEqual([interview, research, "user:2", transit]);
```

`app/ui/shell/LayoutPicker.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/LayoutPicker.test.tsx
+++ b/app/ui/shell/LayoutPicker.test.tsx
@@ -1,5 +1,5 @@
 import { fireEvent, render, screen } from "@testing-library/react";
-import { BUILTIN_LAYOUT_IDS } from "../layout/model";
+import { BUILTIN_LAYOUT_IDS, MAX_LAYOUTS } from "../layout/model";
 import { memoryPersistence } from "../layout/persistence";
 import { HighlightProvider, useHighlight } from "../highlight/store";
 import { LayoutProvider, useLayoutStore } from "../layout/store";
@@ -116,6 +116,19 @@ test("duplicate adds a copy after the source without switching", () => {
   expect(picker()).toHaveDisplayValue("Görüşme");
 });
 
+test("save as and duplicate are disabled at the layout limit", () => {
+  setup();
+  for (let i = 3; i < MAX_LAYOUTS; i++) {
+    menu();
+    fireEvent.click(item("Kopyala"));
+  }
+  expect(options()).toHaveLength(MAX_LAYOUTS);
+  menu();
+  expect(item("Farklı kaydet…")).toBeDisabled();
+  expect(item("Kopyala")).toBeDisabled();
+  expect(item("Kopyala")).toHaveAttribute("title", `En çok ${MAX_LAYOUTS} düzen kaydedilebilir.`);
+});
+
 describe("actions menu keyboard", () => {
   test("opening from the keyboard focuses the first item and wires aria attributes", () => {
     setup();
```


- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/layout ui/shell/LayoutPicker.test.tsx
```
Expected: FAIL — `MAX_LAYOUTS` dışa aktarılmıyor; 64'ten fazla düzen ve 48'den fazla panel kabul ediliyor; sınırda `saveAs` yeni düzen ekliyor; "Kopyala" sınırda etkin.

- [ ] **Step 3: Uygulamayı yaz**



`app/src-tauri/src/lib.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -46,6 +46,8 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             store_commands::list_tags,
             store_commands::get_app_prefs,
             store_commands::set_app_prefs,
+            store_commands::load_panel_layout,
+            store_commands::save_panel_layout,
             store_commands::export_json,
             store_commands::export_csv,
             store_commands::import_json,
```

`app/src-tauri/src/store_commands.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/src-tauri/src/store_commands.rs
+++ b/app/src-tauri/src/store_commands.rs
@@ -7,8 +7,8 @@ use std::sync::{Arc, Mutex};
 use serde::Serialize;
 use specta::Type;
 use store::{
-    AppPrefs, ChartRecord, Client, ExportOptions, Gender, ImportReport, Note, SearchHit, Store,
-    TagCount,
+    AppPrefs, ChartRecord, Client, ExportOptions, Gender, ImportReport, Note, PanelLayout,
+    SearchHit, Store, TagCount,
 };
 use tauri::State;
 
@@ -213,6 +213,25 @@ pub async fn set_app_prefs(
     with_store(&store, move |s| s.set_app_prefs(&prefs)).await
 }
 
+/// Kullanıcının pano düzeni (`app_state`); kayıt yoksa ya da okunamazsa `None`.
+#[tauri::command]
+#[specta::specta]
+pub async fn load_panel_layout(
+    store: State<'_, StoreState>,
+) -> Result<Option<PanelLayout>, CommandError> {
+    with_store(&store, |s| s.panel_layout()).await
+}
+
+/// Açık harita olmadan düzen kaydı (otomatik kayıt); `None` kayıtlı düzeni siler.
+#[tauri::command]
+#[specta::specta]
+pub async fn save_panel_layout(
+    store: State<'_, StoreState>,
+    layout: Option<PanelLayout>,
+) -> Result<(), CommandError> {
+    with_store(&store, move |s| s.save_panel_layout(layout.as_ref())).await
+}
+
 #[tauri::command]
 #[specta::specta]
 pub async fn export_json(
```

`app/ui/i18n/en.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -83,6 +83,7 @@ export const en: Messages = {
     cancel: "Cancel",
     copyName: "{{name}} copy",
     copyNameN: "{{name}} copy {{n}}",
+    limit: "At most {{max}} layouts can be saved.",
     nameEmpty: "Enter a name.",
     nameTaken: "Another layout already uses this name.",
     confirmDelete: "Delete the “{{name}}” layout? This cannot be undone.",
```

`app/ui/i18n/fr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -83,6 +83,7 @@ export const fr: Messages = {
     cancel: "Annuler",
     copyName: "Copie de {{name}}",
     copyNameN: "Copie {{n}} de {{name}}",
+    limit: "Au plus {{max}} dispositions peuvent être enregistrées.",
     nameEmpty: "Saisissez un nom.",
     nameTaken: "Une autre disposition porte déjà ce nom.",
     confirmDelete: "Supprimer la disposition « {{name}} » ? Cette action est irréversible.",
```

`app/ui/i18n/tr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -82,6 +82,7 @@ export const tr = {
     cancel: "Vazgeç",
     copyName: "{{name}} kopyası",
     copyNameN: "{{name}} kopyası {{n}}",
+    limit: "En çok {{max}} düzen kaydedilebilir.",
     nameEmpty: "Bir ad yazın.",
     nameTaken: "Bu ad başka bir düzende kullanılıyor.",
     confirmDelete: "“{{name}}” düzeni silinsin mi? Bu işlem geri alınamaz.",
```

`app/ui/layout/model.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -43,6 +43,12 @@ export interface LayoutState {
 }
 
 export const LAYOUT_SCHEMA_VERSION = 1;
+/** En çok düzen (hazır düzenler dahil) ve düzen başına panel; Rust `store`
+ *  `MAX_LAYOUTS`/`MAX_PANELS` ile aynı (fixtures/layout-parity.json). Sınır yazarken de
+ *  uygulanır (reducer `saveAs`/`duplicate`): arayüz Rust'ın okuyacağından fazlasını yazmaz. */
+export const MAX_LAYOUTS = 64;
+export const MAX_PANELS = 48;
+const MAX_U32 = 0xffff_ffff;
 
 export const BUILTIN_LAYOUT_IDS = {
   interview: "builtin:interview",
@@ -137,7 +143,7 @@ function isNonEmptyString(v: unknown): v is string {
 }
 
 function isInt(v: unknown, min: number): v is number {
-  return typeof v === "number" && Number.isInteger(v) && v >= min;
+  return typeof v === "number" && Number.isInteger(v) && v >= min && v <= MAX_U32;
 }
 
 function parsePanel(raw: unknown): Panel | null {
@@ -153,6 +159,7 @@ function parsePanels(raw: unknown): Panel[] {
   const seen = new Set<string>();
   const out: Panel[] = [];
   for (const item of raw) {
+    if (out.length === MAX_PANELS) break;
     const p = parsePanel(item);
     if (p === null || seen.has(p.id)) continue;
     seen.add(p.id);
@@ -178,18 +185,22 @@ function parseLayout(raw: unknown): Layout | null {
  * - bilinmeyen alanlar yok sayılır, geçersiz panel ve düzenler atılır,
  * - bilinmeyen panel türleri korunur,
  * - eksik hazır düzenler yeniden eklenir; hazır düzenler her zaman başta ve sabit sıradadır,
- * - geçersiz `active_layout_id` ilk hazır düzene döner.
+ * - geçersiz `active_layout_id` ilk hazır düzene döner,
+ * - en çok `MAX_LAYOUTS` düzen (hazır düzenler dahil) ve düzen başına `MAX_PANELS` panel okunur.
  * `schema_version` yoksa 1 kabul edilir. Daha yeni bir sürüm de aynı kurallarla okunur.
  */
 export function parseLayoutState(raw: unknown): LayoutState {
   if (!isRecord(raw)) return defaultLayoutState();
   const parsed = Array.isArray(raw.layouts) ? raw.layouts.map(parseLayout) : [];
   const byId = new Map<string, Layout>();
-  for (const l of parsed) if (l !== null && !byId.has(l.id)) byId.set(l.id, l);
+  for (const l of parsed) {
+    if (byId.size === MAX_LAYOUTS) break;
+    if (l !== null && !byId.has(l.id)) byId.set(l.id, l);
+  }
 
   const builtins = BUILTIN_ORDER.map((id) => byId.get(id) ?? BUILTIN_FACTORIES[id]());
   const users = [...byId.values()].filter((l) => !l.builtin);
-  const layouts = [...builtins, ...users];
+  const layouts = [...builtins, ...users].slice(0, MAX_LAYOUTS);
 
   const active = raw.active_layout_id;
   const activeId =
```

`app/ui/layout/reducer.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/layout/reducer.ts
+++ b/app/ui/layout/reducer.ts
@@ -1,6 +1,6 @@
 // Düzen durumu için saf reducer. Kimlikler eylemle gelir; reducer rastgelelik üretmez.
 import { movePanel, normalizePanels } from "./grid";
-import { builtinLayout, type Layout, type LayoutState, type Panel } from "./model";
+import { builtinLayout, MAX_LAYOUTS, type Layout, type LayoutState, type Panel } from "./model";
 
 export type LayoutAction =
   | { type: "select"; id: string }
@@ -50,7 +50,8 @@ export function layoutReducer(state: LayoutState, action: LayoutAction): LayoutS
     }
     case "saveAs": {
       const name = action.name.trim();
-      if (name === "" || state.layouts.some((l) => l.id === action.id)) return state;
+      // Sınırda yeni düzen eklenmez (Rust fazlasını okumaz; LayoutPicker düğmeyi kapatır).
+      if (name === "" || state.layouts.length >= MAX_LAYOUTS || state.layouts.some((l) => l.id === action.id)) return state;
       const copy: Layout = { id: action.id, name, builtin: false, panels: copyPanels(activeLayout(state).panels) };
       return { ...state, active_layout_id: copy.id, layouts: [...state.layouts, copy] };
     }
@@ -63,7 +64,7 @@ export function layoutReducer(state: LayoutState, action: LayoutAction): LayoutS
     case "duplicate": {
       const name = action.name.trim();
       const index = state.layouts.findIndex((l) => l.id === action.sourceId);
-      if (name === "" || index < 0 || state.layouts.some((l) => l.id === action.id)) return state;
+      if (name === "" || index < 0 || state.layouts.length >= MAX_LAYOUTS || state.layouts.some((l) => l.id === action.id)) return state;
       const copy: Layout = { id: action.id, name, builtin: false, panels: copyPanels(state.layouts[index].panels) };
       const layouts = [...state.layouts];
       layouts.splice(index + 1, 0, copy);
```

`app/ui/shell/LayoutPicker.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/LayoutPicker.tsx
+++ b/app/ui/shell/LayoutPicker.tsx
@@ -1,4 +1,5 @@
 // Düzen seçici ve işlemler menüsü: farklı kaydet, yeniden adlandır, kopyala, sil, sıfırla.
+// `MAX_LAYOUTS` düzende "farklı kaydet" ve "kopyala" devre dışıdır (ipucu sınırı söyler).
 // Ad girişi ve silme onayı satır içinde açılır (WKWebView'da window.prompt'a güvenilmez).
 // Menü ARIA menü düzenini izler: açılınca ilk etkin öğe odaklanır, oklar/Home/End gezdirir,
 // Esc kapatır (preventDefault: vurgu sabitlemesi korunur), Tab ve dış tıklama kapatır.
@@ -6,7 +7,7 @@
 import { useEffect, useId, useRef, useState, type FormEvent, type KeyboardEvent } from "react";
 import { useTranslation } from "react-i18next";
 import type { TFunction } from "i18next";
-import type { Layout } from "../layout/model";
+import { MAX_LAYOUTS, type Layout } from "../layout/model";
 import { useLayoutStore } from "../layout/store";
 
 export function layoutLabel(t: TFunction, layout: Layout): string {
@@ -33,6 +34,8 @@ export function LayoutPicker() {
   const activeLabel = layoutLabel(t, active);
   const target = store.state.layouts.find((l) => l.id === targetId) ?? active;
   const targetLabel = layoutLabel(t, target);
+  /** Düzen sınırı (`MAX_LAYOUTS`): yeni düzen eklenemez; düğmeler görünür biçimde kapanır. */
+  const atCap = store.state.layouts.length >= MAX_LAYOUTS;
 
   const sameName = (a: string, b: string) =>
     a.trim().localeCompare(b.trim(), i18n.language, { sensitivity: "accent" }) === 0;
@@ -188,7 +191,14 @@ export function LayoutPicker() {
           onKeyDown={onMenuKey}
         >
           <li role="none">
-            <button type="button" role="menuitem" tabIndex={-1} onClick={() => startMode("saveAs", uniqueCopyName())}>
+            <button
+              type="button"
+              role="menuitem"
+              tabIndex={-1}
+              disabled={atCap}
+              title={atCap ? t("layout.limit", { max: MAX_LAYOUTS }) : undefined}
+              onClick={() => startMode("saveAs", uniqueCopyName())}
+            >
               {t("layout.saveAs")}
             </button>
           </li>
@@ -208,6 +218,8 @@ export function LayoutPicker() {
               type="button"
               role="menuitem"
               tabIndex={-1}
+              disabled={atCap}
+              title={atCap ? t("layout.limit", { max: MAX_LAYOUTS }) : undefined}
               onClick={() => runItem(() => store.duplicate(active.id, uniqueCopyName()))}
             >
               {t("layout.duplicate")}
```

`crates/store/src/layout.rs` oluştur:

```rust
//! Kullanıcının pano düzeni, açık harita olmadan (UI spec §3.2: düzenler kullanıcınındır).
//! `save_view_state` bir harita kimliği ister; ilk açılışta ve danışan açık değilken
//! arayüz düzeni bununla kaydeder (Plan 08 ertelenen madde 2).

use rusqlite::params;

use crate::charts::PANEL_LAYOUT_KEY;
use crate::view_state::PanelLayout;
use crate::{Store, StoreError};

impl Store {
    /// Yalnızca düzeni yazar; `None` kayıtlı düzeni siler (arayüz hazır düzenlere döner).
    /// Okuma `Store::panel_layout` iledir (toleranslı).
    pub fn save_panel_layout(&mut self, layout: Option<&PanelLayout>) -> Result<(), StoreError> {
        match layout {
            Some(pl) => self.app_state_set(PANEL_LAYOUT_KEY, &serde_json::to_string(pl)?),
            None => {
                self.conn.execute(
                    "DELETE FROM app_state WHERE key = ?1",
                    params![PANEL_LAYOUT_KEY],
                )?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Map;

    use crate::db::testutil::store;
    use crate::view_state::{Layout, Panel, PanelLayout};

    fn layout(id: &str) -> PanelLayout {
        PanelLayout {
            active_layout_id: id.into(),
            layouts: vec![Layout {
                id: id.into(),
                name: "Seans".into(),
                builtin: false,
                panels: vec![Panel {
                    id: "d1".into(),
                    kind: "chart".into(),
                    x: 0,
                    y: 0,
                    w: 6,
                    h: 12,
                    settings: Map::new(),
                }],
            }],
        }
    }

    #[test]
    fn layout_is_saved_without_a_chart_and_replaced_then_cleared() {
        let mut s = store();
        assert_eq!(s.panel_layout().unwrap(), None);
        s.save_panel_layout(Some(&layout("user:1"))).unwrap();
        assert_eq!(s.panel_layout().unwrap(), Some(layout("user:1")));
        s.save_panel_layout(Some(&layout("user:2"))).unwrap();
        assert_eq!(s.panel_layout().unwrap(), Some(layout("user:2")));
        s.save_panel_layout(None).unwrap();
        assert_eq!(s.panel_layout().unwrap(), None);
        s.save_panel_layout(None).unwrap();
    }
}
```

`crates/store/src/lib.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/crates/store/src/lib.rs
+++ b/crates/store/src/lib.rs
@@ -9,6 +9,7 @@ mod db;
 mod error;
 pub mod exchange;
 pub mod jhd;
+mod layout;
 pub mod model;
 mod notes;
 mod open;
@@ -41,5 +42,5 @@ pub use settings::{
 };
 pub use view_state::{
     chart_view_json, chart_view_path, parse_dasha_path, parse_panel_layout, parse_view_state,
-    Layout, Panel, PanelLayout, ViewState, VIEW_STATE_SCHEMA_VERSION,
+    Layout, Panel, PanelLayout, ViewState, MAX_LAYOUTS, MAX_PANELS, VIEW_STATE_SCHEMA_VERSION,
 };
```

`crates/store/src/view_state.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/crates/store/src/view_state.rs
+++ b/crates/store/src/view_state.rs
@@ -14,6 +14,11 @@ use serde_json::{json, Map, Value};
 const MAX_PATH: usize = MAX_DASHA_LEVEL as usize;
 
 pub const VIEW_STATE_SCHEMA_VERSION: u32 = 1;
+/// Okunan en çok düzen (hazır düzenler dahil); fazlası atılır. Arayüzün `MAX_LAYOUTS`'u
+/// ile aynıdır (`app/ui/layout/fixtures/layout-parity.json`).
+pub const MAX_LAYOUTS: usize = 64;
+/// Bir düzende okunan en çok panel; fazlası atılır. Arayüzün `MAX_PANELS`'i ile aynıdır.
+pub const MAX_PANELS: usize = 48;
 
 #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
 #[cfg_attr(feature = "specta", derive(specta::Type))]
@@ -80,10 +85,21 @@ fn non_empty_str(v: Option<&Value>) -> Option<String> {
     }
 }
 
+/// Tamsayı konum/boyut: `3` ve kesirsiz `3.0`/`1e1` kabul edilir (arayüz `Number.isInteger`
+/// ile aynı), aralık `min..=u32::MAX`.
 fn int(v: Option<&Value>, min: u32) -> Option<u32> {
-    v.and_then(Value::as_u64)
-        .and_then(|n| u32::try_from(n).ok())
-        .filter(|n| *n >= min)
+    let v = v?;
+    let n = match v.as_u64() {
+        Some(n) => n,
+        None => {
+            let f = v.as_f64()?;
+            if f.fract() != 0.0 || !(0.0..=f64::from(u32::MAX)).contains(&f) {
+                return None;
+            }
+            f as u64
+        }
+    };
+    u32::try_from(n).ok().filter(|n| *n >= min)
 }
 
 fn parse_panel(v: &Value) -> Option<Panel> {
@@ -107,6 +123,9 @@ fn parse_layout(v: &Value) -> Option<Layout> {
     let mut panels: Vec<Panel> = Vec::new();
     if let Some(Value::Array(items)) = o.get("panels") {
         for p in items.iter().filter_map(parse_panel) {
+            if panels.len() == MAX_PANELS {
+                break;
+            }
             if !panels.iter().any(|q| q.id == p.id) {
                 panels.push(p);
             }
@@ -127,6 +146,9 @@ pub fn parse_panel_layout(v: &Value) -> Option<PanelLayout> {
     let mut layouts: Vec<Layout> = Vec::new();
     if let Some(Value::Array(items)) = o.get("layouts") {
         for l in items.iter().filter_map(parse_layout) {
+            if layouts.len() == MAX_LAYOUTS {
+                break;
+            }
             if !layouts.iter().any(|m| m.id == l.id) {
                 layouts.push(l);
             }
@@ -264,6 +286,45 @@ mod tests {
         assert!(l.panels[0].settings.is_empty());
     }
 
+    /// Arayüzle ortak fikstür: iki ayrıştırıcı aynı düzenleri ve panelleri tutar.
+    const PARITY: &str = include_str!("../../../app/ui/layout/fixtures/layout-parity.json");
+
+    #[test]
+    fn parity_fixture_matches_the_ui_parser() {
+        let fixture: Value = serde_json::from_str(PARITY).unwrap();
+        assert_eq!(fixture["max_layouts"], json!(MAX_LAYOUTS));
+        assert_eq!(fixture["max_panels"], json!(MAX_PANELS));
+        for case in fixture["cases"].as_array().unwrap() {
+            let name = case["name"].as_str().unwrap();
+            let pl = parse_panel_layout(&case["input"]).unwrap();
+            let got: Vec<Value> = pl
+                .layouts
+                .iter()
+                .map(|l| json!({"id": l.id, "panels": l.panels.iter().map(|p| &p.id).collect::<Vec<_>>()}))
+                .collect();
+            assert_eq!(Value::Array(got), case["layouts"], "{name}");
+        }
+    }
+
+    #[test]
+    fn layout_and_panel_counts_are_capped() {
+        let panels: Vec<Value> = (0..MAX_PANELS + 10)
+            .map(
+                |i| json!({"id": format!("p{i}"), "kind": "chart", "x": 0, "y": i, "w": 3, "h": 4}),
+            )
+            .collect();
+        let layouts: Vec<Value> = (0..MAX_LAYOUTS + 6)
+            .map(|i| json!({"id": format!("user:{i}"), "name": "L", "panels": panels}))
+            .collect();
+        let pl = parse_panel_layout(&json!({ "layouts": layouts })).unwrap();
+        assert_eq!(pl.layouts.len(), MAX_LAYOUTS);
+        assert!(pl.layouts.iter().all(|l| l.panels.len() == MAX_PANELS));
+        assert_eq!(
+            pl.layouts[MAX_LAYOUTS - 1].id,
+            format!("user:{}", MAX_LAYOUTS - 1)
+        );
+    }
+
     #[test]
     fn dasha_path_longer_than_five_levels_is_dropped() {
         let six = json!(["Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn"]);
```


- [ ] **Step 4: Bağlamaları yeniden üret**

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings
git diff --stat app/ui/bindings.ts
cargo test -p jyotish-desk --test bindings
```
Expected: ilk komut `app/ui/bindings.ts`'i yazar; ikinci çalıştırma `test result: ok. 1 passed` (dosya güncel). `bindings.ts` elle düzenlenmez.

- [ ] **Step 5: Testlerin geçtiğini gör**

```bash
cargo test -p store
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p store -p jyotish-desk --all-targets
```
Expected: `store` kütüphanesi `test result: ok. 64 passed` (`layout_is_saved_without_a_chart_and_replaced_then_cleared`, `parity_fixture_matches_the_ui_parser`, `layout_and_panel_counts_are_capped` dahil); MSRV denetimi hatasız.
```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```
Expected: ikisi de çıktısız, çıkış kodu 0.
```bash
pnpm --dir app exec vitest run ui/layout ui/shell/LayoutPicker.test.tsx
pnpm --dir app test
pnpm --dir app build
```
Expected: tam takımda `Test Files  23 passed (23)`, `Tests  211 passed (211)`; `build` hatasız (`tsc --noEmit` çıktısız, `vite build` `✓ built`).

- [ ] **Step 6: Commit**

```bash
git add -A app/src-tauri/src/lib.rs app/src-tauri/src/store_commands.rs app/ui/bindings.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts app/ui/i18n/tr.ts app/ui/layout/fixtures/layout-parity.json app/ui/layout/model.test.ts app/ui/layout/model.ts app/ui/layout/reducer.test.ts app/ui/layout/reducer.ts app/ui/shell/LayoutPicker.test.tsx app/ui/shell/LayoutPicker.tsx crates/store/src/layout.rs crates/store/src/lib.rs crates/store/src/view_state.rs
git diff --cached -U0 -- . ':!app/pnpm-lock.yaml' ':!*.png' ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git commit -m "feat(store): layout-only save, layout caps and a parser parity fixture shared with the UI

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Gerçek veri fikstürü ve M5 komut sarmalayıcıları

Arayüz testleri elle uydurulmuş veri yerine motor çıktısını çizer (D21). `tests/ui_fixture.rs` gerçek `Engine` ile Delhi 2000-01-01 17:30 IST haritasını oluşturur ve `app/ui/test/fixtures/delhi.json`'ı yazar ya da eşitliğini denetler. Denetim sayılarda göreli 1e-9 toleranslı yapısal eşitliktir (CI'daki farklı derleyici ve libm son basamağı değiştirebilir); üretim kipinde dosya birebir yazılır ve klasör yoksa oluşturulur. `api.ts` yalnızca M5'te kullanılan komutları sarar; içe/dışa aktarma komutları bilerek sarılmaz (Plan 08 ertelenen madde 1 ve 3). Arayüz bu planda `create_client`/`update_chart`/`save_view_state` çağırmaya başladığı için Plan 08 incelemesinin §2.5'te bu ana bıraktığı test boşlukları da burada kapanır (`tests/open_client.rs`): DST boşluğunda başarısız `update_chart` sonrası harita, doğum verisi ve `jd_ut` değişmemiştir; `save_view_state` ile yazılan dasha yolu ve düzen `restore_session`'dan sağ çıkar; `save_panel_layout(Some)` → `panel_layout()` → `save_panel_layout(None)` gidiş-dönüşü komutların kullandığı `Mutex<Store>` üzerinden denenir.

**Files:**
- Modify: `app/src-tauri/tests/open_client.rs`
- Create: `app/src-tauri/tests/ui_fixture.rs`
- Modify: `app/ui/api.test.ts`
- Modify: `app/ui/api.ts`
- Create: `app/ui/test/fixture.test.ts`
- Create: `app/ui/test/fixture.ts`
- Create: `app/ui/test/fixtures/delhi.json` (üretilir: `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture`)

**Interfaces:**
- Consumes: `create_client_blocking`, `dasha_children` gövdesi, `search_places_blocking`, `preview_moment_body` (Task 1), `Store`, `Engine`.
- Produces: `api` nesnesi: `engineStatus, printWindow, restoreSession, createClient(NewClient), updateChart(chartId, NewChart), saveViewState(chartId, ViewState), loadPanelLayout(), savePanelLayout(PanelLayout | null), dashaChildren(DashaSeed, Graha[]), searchPlaces(query, limit), previewMoment(BirthData), planetLongitudes` (son kalem Task 5'te silinir); `Result<T>`, `isCommandError`; `test/fixture.ts`: `UiFixture {opened, dasha_children: {path, periods}, places, preview, gap_error}`, `fixture()` (her çağrıda derin kopya), `unknownTimeOpened()`.

- [ ] **Step 1: Önce testleri yaz**



`app/src-tauri/tests/open_client.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/src-tauri/tests/open_client.rs
+++ b/app/src-tauri/tests/open_client.rs
@@ -168,6 +168,52 @@ fn open_client_round_trips_and_is_fast() {
     .unwrap_err();
     assert_eq!(gap.code, "nonexistent_local_time");
     assert_eq!(gap.params["suggestion"], "2014-03-31T04:30:00");
+    // Hata kaydı engeller: harita, doğum verisi ve çözülmüş an çağrıdan öncekiyle aynı.
+    let unchanged =
+        open_client_blocking(&engine, &store, client_id, Some(chart_id)).expect("reopen after gap");
+    assert_eq!(unchanged.chart_record.birth, after.chart_record.birth);
+    assert_eq!(
+        unchanged.chart_record.resolved.jd_ut,
+        after.chart_record.resolved.jd_ut
+    );
+    assert_eq!(unchanged.chart, after.chart);
+
+    // Arayüzün otomatik kaydı (`save_view_state`) oturum geri yüklemesinden sağ çıkar.
+    let vs2 = ViewState {
+        dasha_expanded_path: vec![Graha::Saturn, Graha::Mercury, Graha::Ketu],
+        ..vs.clone()
+    };
+    store
+        .lock()
+        .unwrap()
+        .save_view_state(chart_id, &vs2)
+        .expect("save view state");
+    let restored = restore_session_blocking(&engine, &store)
+        .expect("restore")
+        .expect("a session");
+    assert_eq!(
+        restored.view_state.dasha_expanded_path,
+        vs2.dasha_expanded_path
+    );
+    assert_eq!(restored.view_state.panel_layout, vs2.panel_layout);
+
+    // Harita olmadan düzen kaydı (`save_panel_layout`/`load_panel_layout` gövdeleri).
+    let alone = PanelLayout {
+        active_layout_id: "user:2".into(),
+        ..layout()
+    };
+    store
+        .lock()
+        .unwrap()
+        .save_panel_layout(Some(&alone))
+        .expect("save layout");
+    assert_eq!(store.lock().unwrap().panel_layout().unwrap(), Some(alone));
+    store
+        .lock()
+        .unwrap()
+        .save_panel_layout(None)
+        .expect("clear layout");
+    assert_eq!(store.lock().unwrap().panel_layout().unwrap(), None);
     let err = open_client_blocking(&engine, &store, 999, None).unwrap_err();
     assert_eq!(
         (err.code.as_str(), err.params["what"].as_str()),
```

`app/src-tauri/tests/ui_fixture.rs` oluştur:

```rust
//! Arayüz testlerinin (vitest ve Playwright) gerçek veri fikstürü:
//! `app/ui/test/fixtures/delhi.json`. Gerçek motor, store ve atlasla üretilir; saat sabit,
//! UUID'ler sabit dizgiyle değiştirilir. Arayüz kendi haritasını hesaplamadığı için
//! (spec §4) testler bu dosyadaki `OpenedClient`'ı çizer.
//! Güncellemek için: `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture`.
//! Denetim sayılarda göreli 1e-9 toleranslıdır: CI derleyicisi ve libm son basamakta
//! farklı sonuç verebilir. Motoru tek test açar: süreçte yalnızca bir SE örneği olabilir.

use std::path::PathBuf;
use std::sync::Mutex;

use atlas::Timestamp;
use ephemeris::Engine;
use jyotish_desk_lib::atlas_commands::{preview_moment_body, search_places_blocking, AtlasState};
use jyotish_desk_lib::client_commands::create_client_blocking;
use jyotish_desk_lib::commands::dasha_children;
use serde_json::{json, Value};
use store::{BirthData, ChartKind, NewChart, NewClient, Place, Store, TimeCertainty, TzChoice};

const CLIENT_UUID: &str = "00000000-0000-4000-8000-000000000001";
const CHART_UUID: &str = "00000000-0000-4000-8000-000000000002";

/// tests/golden/new-delhi-2000-01-01.json: 17:30 IST = JD 2451545.0.
fn delhi() -> BirthData {
    BirthData {
        date: "2000-01-01".into(),
        time: Some("17:30".into()),
        time_certainty: TimeCertainty::Exact,
        place: Place {
            name: "New Delhi".into(),
            country: "India".into(),
            admin1: "Delhi".into(),
            lat: 28.6139,
            lon: 77.2090,
            geoname_id: Some(1_261_481),
        },
        tz: TzChoice::Iana("Asia/Kolkata".into()),
        fold: None,
        notes: String::new(),
    }
}

#[test]
fn ui_fixture_is_up_to_date() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let engine = Engine::spawn(root.join("../../data/ephe")).expect("spawn engine");
    let clock = Box::new(|| Timestamp::from_second(1_795_600_800).expect("timestamp"));
    let store = Mutex::new(Store::open_in_memory().expect("store").with_clock(clock));
    let opened = create_client_blocking(
        &engine,
        &store,
        NewClient {
            name: "Örnek Danışan".into(),
            gender: None,
            chart: NewChart {
                label: String::new(),
                kind: ChartKind::Natal,
                birth: delhi(),
            },
        },
    )
    .expect("create client");

    // İlk pratyantar dönemin (3. seviye) alt dönemleri: tembel yüklenen 4. seviye.
    let path = opened
        .chart
        .dasha
        .periods
        .iter()
        .find(|p| p.level == 3)
        .expect("level 3")
        .lords
        .clone();
    let children = dasha_children(opened.chart.dasha.seed, path.clone()).expect("children");

    let atlas = AtlasState::open(&root.join("../../data/atlas/atlas.sqlite"));
    let places = search_places_blocking(&atlas, "new delhi", 8).expect("places");
    let preview = preview_moment_body(&delhi()).expect("preview");
    let mut gap = delhi();
    gap.date = "2014-03-31".into();
    gap.time = Some("03:30".into());
    gap.tz = TzChoice::Iana("Europe/Istanbul".into());
    let gap_error = preview_moment_body(&gap).expect_err("DST gap");

    let value = json!({
        "opened": opened,
        "dasha_children": { "path": path, "periods": children },
        "places": places,
        "preview": preview,
        "gap_error": gap_error,
    });
    let text = serde_json::to_string(&value)
        .expect("serialize")
        .replace(&opened.client.uuid, CLIENT_UUID)
        .replace(&opened.chart_record.uuid, CHART_UUID)
        + "\n";

    let path = root.join("../ui/test/fixtures/delhi.json");
    if std::env::var_os("UPDATE_FIXTURES").is_some() {
        std::fs::create_dir_all(path.parent().expect("fixture dir")).expect("create fixture dir");
        std::fs::write(&path, &text).expect("write fixture");
        return;
    }
    let committed: Value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(Value::Null);
    let fresh: Value = serde_json::from_str(&text).expect("parse fresh fixture");
    assert!(
        approx_eq(&committed, &fresh),
        "app/ui/test/fixtures/delhi.json is stale; run: UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture"
    );
}

/// Yapısal eşitlik; sayılar göreli 1e-9 toleransla, diğer değerler birebir.
fn approx_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => match (x.as_f64(), y.as_f64()) {
            (Some(x), Some(y)) => (x - y).abs() <= 1e-9 * x.abs().max(1.0),
            _ => x == y,
        },
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(x, y)| approx_eq(x, y))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(k, v)| y.get(k).is_some_and(|w| approx_eq(v, w)))
        }
        _ => a == b,
    }
}

#[test]
fn approx_eq_tolerates_last_digit_noise_only() {
    let a = json!({"lon": 256.515_696_012_345, "name": "Sun", "list": [1, 2.0, null]});
    let mut b = a.clone();
    b["lon"] = json!(256.515_696_012_346);
    assert!(approx_eq(&a, &b));
    b["lon"] = json!(256.515_7);
    assert!(!approx_eq(&a, &b));
    let mut c = a.clone();
    c["name"] = json!("Moon");
    assert!(!approx_eq(&a, &c));
    let mut d = a.clone();
    d["list"] = json!([1, 2.0]);
    assert!(!approx_eq(&a, &d));
    assert!(!approx_eq(&a, &Value::Null));
}
```

`app/ui/api.test.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -5,6 +5,15 @@ vi.mock("./bindings", () => ({
     engineStatus: vi.fn(),
     planetLongitudes: vi.fn(),
     printWindow: vi.fn(),
+    restoreSession: vi.fn(),
+    createClient: vi.fn(),
+    updateChart: vi.fn(),
+    saveViewState: vi.fn(),
+    loadPanelLayout: vi.fn(),
+    savePanelLayout: vi.fn(),
+    dashaChildren: vi.fn(),
+    searchPlaces: vi.fn(),
+    previewMoment: vi.fn(),
   },
 }));
 
@@ -43,3 +52,30 @@ test("catches thrown errors into an ipc error", async () => {
     error: { code: "ipc", params: { detail: "Error: boom" } },
   });
 });
+
+test("every wrapper forwards its arguments to the command of the same name", async () => {
+  const calls: [keyof typeof api, unknown[]][] = [
+    ["restoreSession", []],
+    ["createClient", [{ name: "A" }]],
+    ["updateChart", [3, { label: "" }]],
+    ["saveViewState", [3, { schema_version: 1 }]],
+    ["loadPanelLayout", []],
+    ["savePanelLayout", [null]],
+    ["dashaChildren", [{ birth_jd_ut: 1 }, ["Sun"]]],
+    ["searchPlaces", ["delhi", 8]],
+    ["previewMoment", [{ date: "2000-01-01" }]],
+  ];
+  for (const [name, args] of calls) {
+    const command = vi.mocked(commands[name] as (...a: unknown[]) => Promise<unknown>);
+    command.mockResolvedValueOnce({ status: "ok", data: name });
+    const call = api[name] as (...a: unknown[]) => Promise<unknown>;
+    expect(await call(...args)).toEqual({ status: "ok", data: name });
+    expect(command).toHaveBeenCalledWith(...args);
+  }
+});
+
+test("import and export commands are not reachable from the UI (Plan 08 deferred items 1 and 3)", () => {
+  for (const name of ["importJson", "importJhd", "exportJson", "exportCsv"]) {
+    expect(Object.keys(api)).not.toContain(name);
+  }
+});
```

`app/ui/test/fixture.test.ts` oluştur:

```ts
import { fixture, unknownTimeOpened } from "./fixture";

test("the fixture is a real Delhi chart with three dasha levels and atlas hits", () => {
  const f = fixture();
  expect(f.opened.chart.grahas).toHaveLength(9);
  expect(f.opened.chart.vargas).toHaveLength(16);
  expect(f.opened.chart.dasha.periods).toHaveLength(9 + 81 + 729);
  expect(f.opened.chart_record.resolved.jd_ut).toBe(2451545.0);
  expect(f.dasha_children.path).toHaveLength(3);
  expect(f.dasha_children.periods.every((p) => p.level === 4)).toBe(true);
  expect(f.places[0]).toMatchObject({ name: "New Delhi", timezone: "Asia/Kolkata" });
  expect(f.gap_error).toEqual({ code: "nonexistent_local_time", params: { local: "2014-03-31T03:30:00", suggestion: "2014-03-31T04:30:00" } });
});

test("fixture copies are independent", () => {
  const a = fixture();
  a.opened.client.name = "x";
  expect(fixture().opened.client.name).toBe("Örnek Danışan");
  expect(unknownTimeOpened().chart_record.birth.time_certainty).toBe("Unknown");
});
```

`app/ui/test/fixture.ts` oluştur:

```ts
// Gerçek veri fikstürü (app/src-tauri/tests/ui_fixture.rs üretir): Delhi 2000-01-01 17:30
// için `create_client` dönüşü, ilk 3. seviye dönemin alt dönemleri, "new delhi" atlas araması,
// an önizlemesi ve bir DST boşluğu hatası. Arayüz hesap yapmadığı için testler bunu çizer.
import type { CommandError, DashaPeriod, Graha, MomentPreview, OpenedClient, PlaceHit } from "../bindings";
import raw from "./fixtures/delhi.json?raw";

export interface UiFixture {
  opened: OpenedClient;
  dasha_children: { path: Graha[]; periods: DashaPeriod[] };
  places: PlaceHit[];
  preview: MomentPreview;
  gap_error: CommandError;
}

const FIXTURE = JSON.parse(raw) as UiFixture;

/** Her çağrıda derin kopya: testler sonucu değiştirebilir. */
export function fixture(): UiFixture {
  return structuredClone(FIXTURE);
}

/** Saati bilinmeyen doğum: aynı harita, `time_certainty: "Unknown"` ve öğle uyarısı. */
export function unknownTimeOpened(): OpenedClient {
  const o = fixture().opened;
  o.chart_record.birth.time = null;
  o.chart_record.birth.time_certainty = "Unknown";
  o.chart_record.resolved.warnings = ["TimeUnknownNoon"];
  return o;
}
```


- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
cargo test -p jyotish-desk --test ui_fixture
pnpm --dir app exec vitest run ui/api.test.ts ui/test/fixture.test.ts
```
Expected: Rust testi `delhi.json` yok diye düşer (`delhi.json is stale; run: UPDATE_FIXTURES=1 …`); `approx_eq_tolerates_last_digit_noise_only` geçer. `open_client.rs`'e eklenen denetimler Plan 08 davranışını sabitleyen gerileme testleridir ve hemen geçer (`cargo test -p jyotish-desk --test open_client` → `1 passed`). vitest `api.searchPlaces is not a function` ve `Failed to resolve import "./fixtures/delhi.json"` ile düşer.

- [ ] **Step 3: Uygulamayı yaz**



`app/ui/api.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -1,11 +1,26 @@
 // bindings.ts'in ince sarmalayıcısı: IPC hataları da arayüze `{code, params}` olarak ulaşır.
 // Üretilen `typedError` Tauri'nin düz string reddini (bilinmeyen komut, argüman çözme,
 // ACL, yönetilmeyen state) CommandError diye işaretler ve `Error`'ları yeniden fırlatır.
-import { commands, type CommandError } from "./bindings";
+//
+// Arayüz yalnızca burada listelenen komutları çağırır. İçe ve dışa aktarma komutları
+// (`importJson`, `importJhd`, `exportJson`, `exportCsv`) Plan 08 ertelenen maddeleri 1 ve 3
+// kapanana kadar (M6) bilerek yoktur; `api.test.ts` bunu denetler.
+// Hata parametreleri (`CommandError.params`) kişisel veri içerebilir: hiçbir yerde loglanmaz.
+import {
+  commands,
+  type CommandError,
+  type DashaSeed,
+  type Graha,
+  type NewChart,
+  type NewClient,
+  type BirthData,
+  type PanelLayout,
+  type ViewState,
+} from "./bindings";
 
 export type Result<T> = { status: "ok"; data: T } | { status: "error"; error: CommandError };
 
-function isCommandError(e: unknown): e is CommandError {
+export function isCommandError(e: unknown): e is CommandError {
   return typeof e === "object" && e !== null && typeof (e as { code?: unknown }).code === "string";
 }
 
@@ -27,4 +42,15 @@ export const api = {
   engineStatus: () => wrap(() => commands.engineStatus()),
   planetLongitudes: (jdUt: number | null) => wrap(() => commands.planetLongitudes(jdUt)),
   printWindow: () => wrap(() => commands.printWindow()),
+  restoreSession: () => wrap(() => commands.restoreSession()),
+  createClient: (input: NewClient) => wrap(() => commands.createClient(input)),
+  updateChart: (chartId: number, chart: NewChart) => wrap(() => commands.updateChart(chartId, chart)),
+  saveViewState: (chartId: number, viewState: ViewState) => wrap(() => commands.saveViewState(chartId, viewState)),
+  loadPanelLayout: () => wrap(() => commands.loadPanelLayout()),
+  savePanelLayout: (layout: PanelLayout | null) => wrap(() => commands.savePanelLayout(layout)),
+  dashaChildren: (seed: DashaSeed, path: Graha[]) => wrap(() => commands.dashaChildren(seed, path)),
+  searchPlaces: (query: string, limit: number) => wrap(() => commands.searchPlaces(query, limit)),
+  previewMoment: (birth: BirthData) => wrap(() => commands.previewMoment(birth)),
 };
+
+export type Api = typeof api;
```


- [ ] **Step 4: Fikstürü üret**

```bash
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture
cargo test -p jyotish-desk --test ui_fixture
```
Expected: ilk komut `app/ui/test/fixtures/` klasörünü ve `delhi.json`'ı yazar (tek satır JSON); ikinci `test result: ok. 2 passed`. İçerik: `chart_record.id = 1`, danışan "Örnek Danışan", lagna İkizler 16.34°, Güneş Yay 16.52°, Satürn Koç retro ve düşük, 819 dönem (1–3. seviye), `dasha_children.path = [Rahu, Rahu, Rahu]` ile 9 dönem. Dosya elle düzenlenmez.

- [ ] **Step 5: Testlerin geçtiğini gör**

```bash
cargo test -p jyotish-desk --test ui_fixture --test open_client
```
Expected: `ui_fixture` `2 passed`, `open_client` `1 passed`.
```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```
Expected: ikisi de çıktısız, çıkış kodu 0.
```bash
pnpm --dir app exec vitest run ui/api.test.ts ui/test/fixture.test.ts
pnpm --dir app test
pnpm --dir app build
```
Expected: tam takımda `Test Files  24 passed (24)`, `Tests  215 passed (215)`; `build` hatasız (`tsc --noEmit` çıktısız, `vite build` `✓ built`).

- [ ] **Step 6: Commit**

```bash
git add -A app/src-tauri/tests/open_client.rs app/src-tauri/tests/ui_fixture.rs app/ui/api.test.ts app/ui/api.ts app/ui/test/fixture.test.ts app/ui/test/fixture.ts app/ui/test/fixtures/delhi.json
git diff --cached -U0 -- . ':!app/pnpm-lock.yaml' ':!*.png' ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git commit -m "test(app): real-data UI fixture and api wrappers for the M5 commands

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Merkezi ViewState deposu ve store'a kayıt

Spec §11: "ViewState baştan tek merkezden yönetilir ve serileştirilebilir" (D9). Plan 04'ün `layout/store.tsx` ve `layout/persistence.ts`'i `app/ui/view/`'a taşınır ve genişler: model `{chartId, layout, dashaPath}`, tek reducer, tek serileştirici `toViewState`. Kayıt 500 ms debounce ile `save_view_state` ya da `save_panel_layout`'a gider; localStorage düzeni açılışta bir kez store'a taşınır ve silinir. Bileşenler `useLayoutStore()`'u eskisi gibi kullanır (yalnızca içe aktarma yolu değişir). Plan 04'ün `layout/persistence.ts`'indeki `console.warn` bu taşımayla gittiği için "kişisel veri loglanmaz" kuralının bekçisi `app/ui/no-console.test.ts` burada eklenir: `app/ui`'de test ve bağlama dışında `console.` çağrısı yoktur (yalnızca kişisel veri taşımayan tema ve dil tercihi uyarıları izinlidir).

**Files:**
- Modify: `app/ui/App.test.tsx`
- Modify: `app/ui/App.tsx`
- Modify: `app/ui/dashboard/Dashboard.test.tsx`
- Modify: `app/ui/dashboard/Dashboard.tsx`
- Delete: `app/ui/layout/persistence.ts`
- Delete: `app/ui/layout/store.tsx`
- Modify: `app/ui/main.tsx`
- Create: `app/ui/no-console.test.ts`
- Modify: `app/ui/shell/LayoutPicker.test.tsx`
- Modify: `app/ui/shell/LayoutPicker.tsx`
- Modify: `app/ui/shell/Shell.tsx`
- Create: `app/ui/view/model.test.ts`
- Create: `app/ui/view/model.ts`
- Create: `app/ui/view/persistence.test.ts`
- Create: `app/ui/view/persistence.ts`
- Create: `app/ui/view/reducer.ts`
- Move: `app/ui/layout/store.test.tsx` → `app/ui/view/store.test.tsx`
- Create: `app/ui/view/store.tsx`

**Interfaces:**
- Consumes: `api.saveViewState`, `api.savePanelLayout`, `api.loadPanelLayout` (Task 3); `layoutReducer`, `parseLayoutState`, `defaultLayoutState` (Plan 04).
- Produces: `view/model.ts`: `ViewModel {chartId: number | null; layout: LayoutState; dashaPath: Graha[]}`, `initialViewModel(rawLayout, chart?: {id, dashaPath})`, `toViewState(m): ViewState`, `parseDashaPath(raw): Graha[]`, `MAX_DASHA_PATH = 5`; `view/reducer.ts`: `ViewAction`, `viewReducer`; `view/persistence.ts`: `ViewSnapshot {chartId, viewState}`, `ViewPersistence {save(snapshot)}`, `storePersistence(onError)`, `memoryPersistence()` (`saved` dizisiyle), `loadLayout(storage?)`, `LEGACY_LAYOUT_KEY`; `view/store.tsx`: `ViewStateProvider({initial, persistence, debounceMs?, children})`, `useViewStore(): ViewStore {model, viewState, setDashaPath, openChart(chartId, dashaPath)}`, `useLayoutStore(): LayoutStore`, `SAVE_DEBOUNCE_MS = 500`.

- [ ] **Step 1: Önce testleri yaz**

`layout/store.test.tsx` `git mv` ile `view/store.test.tsx`'e taşınır ve yeni depoya göre yeniden yazılır:

`app/ui/App.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/App.test.tsx
+++ b/app/ui/App.test.tsx
@@ -17,7 +17,7 @@ vi.mock("./api", () => ({
 
 import { api } from "./api";
 import { App } from "./App";
-import { memoryPersistence } from "./layout/persistence";
+import { memoryPersistence } from "./view/persistence";
 import { THEME_STORAGE_KEY } from "./theme/theme";
 
 function renderApp() {
```

`app/ui/dashboard/Dashboard.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/dashboard/Dashboard.test.tsx
+++ b/app/ui/dashboard/Dashboard.test.tsx
@@ -14,9 +14,10 @@ import { api } from "../api";
 import { HighlightProvider } from "../highlight/store";
 import { hasOverlap } from "../layout/grid";
 import type { LayoutState, Panel } from "../layout/model";
-import { memoryPersistence } from "../layout/persistence";
-import { LayoutProvider } from "../layout/store";
 import { SkyProvider } from "../panels/sky";
+import { initialViewModel } from "../view/model";
+import { memoryPersistence } from "../view/persistence";
+import { ViewStateProvider } from "../view/store";
 import { Dashboard } from "./Dashboard";
 import { GRID_GAP_PX } from "./metrics";
 
@@ -45,15 +46,15 @@ afterEach(() => {
 const MOVE = { buttons: 1, pointerId: 1 };
 
 function setup(width = WIDTH, panels = PANELS, initial: LayoutState = state(panels)) {
-  const persistence = memoryPersistence(initial);
+  const persistence = memoryPersistence();
   const utils = render(
-    <LayoutProvider persistence={persistence} debounceMs={0}>
+    <ViewStateProvider initial={initialViewModel(initial)} persistence={persistence} debounceMs={0}>
       <HighlightProvider>
         <SkyProvider now={() => 0}>
           <Dashboard width={width} />
         </SkyProvider>
       </HighlightProvider>
-    </LayoutProvider>,
+    </ViewStateProvider>,
   );
   const frame = (id: string) => utils.container.querySelector<HTMLElement>(`[data-panel-id="${id}"]`)!;
   const box = (id: string) => {
@@ -106,7 +107,7 @@ test("dragging the header moves the panel by whole cells and commits on pointer
   fireEvent.pointerUp(window, { pointerId: 1 });
   expect(box("a")).toEqual([8, 2, 4, 4]);
   await waitFor(() => expect(persistence.saved).toHaveLength(1));
-  const saved = persistence.saved[0].layouts.find((l) => l.id === "user:t")!;
+  const saved = persistence.saved[0].viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!;
   expect(saved.panels.find((p) => p.id === "a")).toMatchObject({ x: 8, y: 2 });
 });
 
@@ -244,8 +245,8 @@ test("switching the active layout mid-drag cancels the drag and commits nothing"
   await new Promise((r) => setTimeout(r, 20));
   // Yalnızca seçim değişti: hiçbir kayıtta düzenlerin içeriği ilk kayıttan farklı olmamalı.
   expect(persistence.saved.length).toBeGreaterThan(0);
-  const first = JSON.stringify(persistence.saved[0].layouts);
-  for (const s of persistence.saved) expect(JSON.stringify(s.layouts)).toBe(first);
+  const first = JSON.stringify(persistence.saved[0].viewState.panel_layout!.layouts);
+  for (const s of persistence.saved) expect(JSON.stringify(s.viewState.panel_layout!.layouts)).toBe(first);
 });
 
 test("a drag that ends where it started commits nothing", async () => {
```

`app/ui/no-console.test.ts` oluştur:

```ts
// Kişisel veri loglanmaz (spec §4; `CommandError.params` doğum verisi taşıyabilir): arayüz
// kodunda `console` çağrısı yoktur. Testler ve üretilen bağlamalar hariç; tema ve dil tercihi
// (Plan 04) yalnızca localStorage yazma hatasını uyarır, kişisel veri içermez.
const sources = import.meta.glob("/ui/**/*.{ts,tsx}", {
  query: "?raw",
  import: "default",
  eager: true,
}) as Record<string, string>;

const EXEMPT = [/^\/ui\/bindings\.ts$/, /\.test\.tsx?$/];
const ALLOWED = new Set(["/ui/theme/theme.ts", "/ui/i18n/index.ts"]);

test("no console calls in UI code", () => {
  const files = Object.keys(sources).filter((f) => !EXEMPT.some((re) => re.test(f)));
  expect(files.length).toBeGreaterThan(0);
  const offenders = files.filter((f) => !ALLOWED.has(f) && /\bconsole\s*\./.test(sources[f]));
  expect(offenders).toEqual([]);
});
```

`app/ui/shell/LayoutPicker.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/LayoutPicker.test.tsx
+++ b/app/ui/shell/LayoutPicker.test.tsx
@@ -1,8 +1,9 @@
 import { fireEvent, render, screen } from "@testing-library/react";
 import { BUILTIN_LAYOUT_IDS, MAX_LAYOUTS } from "../layout/model";
-import { memoryPersistence } from "../layout/persistence";
 import { HighlightProvider, useHighlight } from "../highlight/store";
-import { LayoutProvider, useLayoutStore } from "../layout/store";
+import { initialViewModel } from "../view/model";
+import { memoryPersistence } from "../view/persistence";
+import { useLayoutStore, ViewStateProvider } from "../view/store";
 import { LayoutPicker } from "./LayoutPicker";
 
 function ActiveId() {
@@ -23,14 +24,14 @@ function Pin() {
 
 function setup() {
   render(
-    <LayoutProvider persistence={memoryPersistence()}>
+    <ViewStateProvider initial={initialViewModel(null)} persistence={memoryPersistence()}>
       <HighlightProvider>
         <LayoutPicker />
         <ActiveId />
         <Pin />
         <button type="button">outside</button>
       </HighlightProvider>
-    </LayoutProvider>,
+    </ViewStateProvider>,
   );
 }
 
```

`app/ui/view/model.test.ts` oluştur:

```ts
import { BUILTIN_LAYOUT_IDS, defaultLayoutState } from "../layout/model";
import { initialViewModel, parseDashaPath, toViewState, VIEW_STATE_SCHEMA_VERSION } from "./model";
import { viewReducer } from "./reducer";

test("parseDashaPath is tolerant like the Rust store", () => {
  expect(parseDashaPath(["Rahu", "Jupiter"])).toEqual(["Rahu", "Jupiter"]);
  expect(parseDashaPath(["Rahu", "Pluto"])).toEqual([]);
  expect(parseDashaPath(["Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn"])).toEqual([]);
  expect(parseDashaPath("Rahu")).toEqual([]);
  expect(parseDashaPath(null)).toEqual([]);
});

test("toViewState serialises the whole model in the IPC shape", () => {
  const m = initialViewModel({ active_layout_id: BUILTIN_LAYOUT_IDS.research }, { id: 3, dashaPath: ["Venus"] });
  const vs = toViewState(m);
  expect(vs).toEqual({
    schema_version: VIEW_STATE_SCHEMA_VERSION,
    panel_layout: { active_layout_id: BUILTIN_LAYOUT_IDS.research, layouts: defaultLayoutState().layouts },
    dasha_expanded_path: ["Venus"],
  });
  expect(m.chartId).toBe(3);
  // Serileştirilen durum yeniden okunduğunda aynı modeli verir.
  expect(initialViewModel(vs.panel_layout, { id: 3, dashaPath: vs.dasha_expanded_path })).toEqual(m);
});

test("the reducer keeps identity for no-op changes", () => {
  const m = initialViewModel(null);
  expect(viewReducer(m, { type: "setDashaPath", path: [] })).toBe(m);
  expect(viewReducer(m, { type: "layout", action: { type: "select", id: "nope" } })).toBe(m);
  const opened = viewReducer(m, { type: "openChart", chartId: 1, dashaPath: ["Sun"] });
  expect([opened.chartId, opened.dashaPath]).toEqual([1, ["Sun"]]);
  expect(opened.layout).toBe(m.layout);
});
```

`app/ui/view/persistence.test.ts` oluştur:

```ts
import { vi } from "vitest";

vi.mock("../api", () => ({
  api: { saveViewState: vi.fn(), savePanelLayout: vi.fn(), loadPanelLayout: vi.fn() },
}));

import { api } from "../api";
import { BUILTIN_LAYOUT_IDS } from "../layout/model";
import { LEGACY_LAYOUT_KEY, loadLayout, storePersistence } from "./persistence";

const ok = <T,>(data: T) => ({ status: "ok" as const, data });
const vs = { schema_version: 1, panel_layout: { active_layout_id: "a", layouts: [] }, dasha_expanded_path: [] };

afterEach(() => vi.mocked(api.savePanelLayout).mockReset());

test("with an open chart the whole ViewState goes to save_view_state, without one only the layout", async () => {
  vi.mocked(api.saveViewState).mockResolvedValue(ok(null));
  vi.mocked(api.savePanelLayout).mockResolvedValue(ok(null));
  const p = storePersistence();
  p.save({ chartId: 4, viewState: vs });
  p.save({ chartId: null, viewState: vs });
  expect(api.saveViewState).toHaveBeenCalledWith(4, vs);
  expect(api.savePanelLayout).toHaveBeenCalledWith(vs.panel_layout);
});

test("save errors reach onError", async () => {
  const error = { code: "store_sqlite", params: { detail: "disk full" } };
  vi.mocked(api.savePanelLayout).mockResolvedValue({ status: "error", error });
  const onError = vi.fn();
  storePersistence(onError).save({ chartId: null, viewState: vs });
  await vi.waitFor(() => expect(onError).toHaveBeenCalledWith(error));
});

test("loadLayout prefers the store and falls back to defaults on error", async () => {
  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(vs.panel_layout));
  expect(await loadLayout()).toEqual(vs.panel_layout);
  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce({ status: "error", error: { code: "store_unavailable", params: {} } });
  expect(await loadLayout()).toBeNull();
  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(null));
  expect(await loadLayout()).toBeNull();
  expect(api.savePanelLayout).not.toHaveBeenCalled();
});

test("a Plan 04 layout in localStorage moves to the store once", async () => {
  localStorage.setItem(LEGACY_LAYOUT_KEY, JSON.stringify({ active_layout_id: BUILTIN_LAYOUT_IDS.transit, layouts: [] }));
  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(null));
  vi.mocked(api.savePanelLayout).mockResolvedValueOnce(ok(null));
  const loaded = (await loadLayout()) as { active_layout_id: string };
  expect(loaded.active_layout_id).toBe(BUILTIN_LAYOUT_IDS.transit);
  expect(vi.mocked(api.savePanelLayout).mock.calls[0][0]).toMatchObject({ active_layout_id: BUILTIN_LAYOUT_IDS.transit });
  expect(localStorage.getItem(LEGACY_LAYOUT_KEY)).toBeNull();
});

test("a corrupt legacy entry is ignored and a failed move keeps it for the next start", async () => {
  localStorage.setItem(LEGACY_LAYOUT_KEY, "{not json");
  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(null));
  expect(await loadLayout()).toBeNull();
  localStorage.setItem(LEGACY_LAYOUT_KEY, JSON.stringify({ layouts: [] }));
  vi.mocked(api.loadPanelLayout).mockResolvedValueOnce(ok(null));
  vi.mocked(api.savePanelLayout).mockResolvedValueOnce({ status: "error", error: { code: "store_sqlite", params: {} } });
  expect(await loadLayout()).not.toBeNull();
  expect(localStorage.getItem(LEGACY_LAYOUT_KEY)).not.toBeNull();
});
```

`app/ui/view/store.test.tsx` oluştur (`git mv app/ui/layout/store.test.tsx app/ui/view/store.test.tsx` ile taşınır, içerik tamamen yenisidir):

```tsx
import { act, fireEvent, render, screen } from "@testing-library/react";
import { useLayoutEffect } from "react";
import type { Graha } from "../bindings";
import { BUILTIN_LAYOUT_IDS } from "../layout/model";
import { initialViewModel } from "./model";
import { memoryPersistence, type ViewPersistence, type ViewSnapshot } from "./persistence";
import { SAVE_DEBOUNCE_MS, useLayoutStore, useViewStore, ViewStateProvider } from "./store";

function Probe() {
  const store = useLayoutStore();
  const view = useViewStore();
  return (
    <div>
      <span data-testid="active">{store.active.id}</span>
      <span data-testid="count">{store.state.layouts.length}</span>
      <span data-testid="chart">{String(view.model.chartId)}</span>
      <span data-testid="path">{view.model.dashaPath.join(">")}</span>
      <button onClick={() => store.saveAs("Benim")}>saveAs</button>
      <button onClick={() => store.select(BUILTIN_LAYOUT_IDS.research)}>research</button>
      <button onClick={() => view.setDashaPath(["Rahu", "Jupiter"] as Graha[])}>path</button>
      <button onClick={() => view.openChart(7, ["Venus"])}>open7</button>
      <button onClick={() => view.openChart(8, "garbage")}>open8</button>
      <button
        onClick={() => {
          store.select(BUILTIN_LAYOUT_IDS.research);
          view.openChart(8, []);
        }}
      >
        researchThenOpen8
      </button>
    </div>
  );
}

function setup(persistence: ViewPersistence = memoryPersistence(), layout: unknown = null) {
  return render(
    <ViewStateProvider initial={initialViewModel(layout)} persistence={persistence}>
      <Probe />
      <input aria-label="field" />
    </ViewStateProvider>,
  );
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

test("loads the stored layout through the tolerant parser", () => {
  setup(memoryPersistence(), { active_layout_id: BUILTIN_LAYOUT_IDS.transit, layouts: [] });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  expect(screen.getByTestId("count")).toHaveTextContent("3");
  expect(screen.getByTestId("chart")).toHaveTextContent("null");
});

test("does not save on mount; saves the whole ViewState once 500 ms after the last change", () => {
  const p = memoryPersistence();
  setup(p);
  act(() => vi.advanceTimersByTime(2000));
  expect(p.saved).toHaveLength(0);
  fireEvent.click(screen.getByText("research"));
  act(() => vi.advanceTimersByTime(300));
  fireEvent.click(screen.getByText("saveAs"));
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS - 1));
  expect(p.saved).toHaveLength(0);
  act(() => vi.advanceTimersByTime(1));
  expect(p.saved).toHaveLength(1);
  const [s] = p.saved;
  expect(s.chartId).toBeNull();
  expect(s.viewState.schema_version).toBe(1);
  expect(s.viewState.panel_layout!.layouts).toHaveLength(4);
  expect(s.viewState.panel_layout!.active_layout_id).toMatch(/^user:/);
  expect(s.viewState.dasha_expanded_path).toEqual([]);
});

test("the dasha path is part of the saved ViewState of the open chart", () => {
  const p = memoryPersistence();
  setup(p);
  fireEvent.click(screen.getByText("open7"));
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS * 2));
  expect(p.saved).toHaveLength(0); // açılan haritanın kendi durumu yeniden yazılmaz
  expect(screen.getByTestId("path")).toHaveTextContent("Venus");
  fireEvent.click(screen.getByText("path"));
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS));
  expect(p.saved).toEqual<ViewSnapshot[]>([
    { chartId: 7, viewState: expect.objectContaining({ dasha_expanded_path: ["Rahu", "Jupiter"] }) },
  ]);
});

test("opening another chart flushes the pending save to the previous chart first", () => {
  const p = memoryPersistence();
  setup(p);
  fireEvent.click(screen.getByText("open7"));
  fireEvent.click(screen.getByText("path"));
  fireEvent.click(screen.getByText("open8"));
  expect(p.saved.map((s) => [s.chartId, s.viewState.dasha_expanded_path])).toEqual([[7, ["Rahu", "Jupiter"]]]);
  expect(screen.getByTestId("chart")).toHaveTextContent("8");
  expect(screen.getByTestId("path")).toHaveTextContent(""); // bozuk yol boş yol olur
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS * 2));
  expect(p.saved).toHaveLength(1);
});

test("a change dispatched in the same tick as openChart is kept and saved to the previous chart", () => {
  const p = memoryPersistence();
  setup(p);
  fireEvent.click(screen.getByText("researchThenOpen8"));
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.research);
  expect(screen.getByTestId("chart")).toHaveTextContent("8");
  expect(p.saved.map((s) => [s.chartId, s.viewState.panel_layout!.active_layout_id])).toEqual([
    [null, BUILTIN_LAYOUT_IDS.research],
  ]);
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS * 2));
  expect(p.saved).toHaveLength(1);
});

test("a committed change whose effects have not run yet is still flushed to the previous chart", () => {
  // Layout effect'ler passive effect'lerden önce çalışır: değişiklik commit edilmiş ama
  // deponun kendi effect'i henüz çalışmamışken `openChart` çağrılır.
  function OpenOnResearch() {
    const { active } = useLayoutStore();
    const { model, openChart } = useViewStore();
    useLayoutEffect(() => {
      if (active.id === BUILTIN_LAYOUT_IDS.research && model.chartId === 7) openChart(8, []);
    }, [active.id, model.chartId, openChart]);
    return null;
  }
  const p = memoryPersistence();
  render(
    <ViewStateProvider initial={initialViewModel(null)} persistence={p}>
      <Probe />
      <OpenOnResearch />
    </ViewStateProvider>,
  );
  fireEvent.click(screen.getByText("open7"));
  fireEvent.click(screen.getByText("research"));
  expect(screen.getByTestId("chart")).toHaveTextContent("8");
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.research);
  expect(p.saved.map((s) => [s.chartId, s.viewState.panel_layout!.active_layout_id])).toEqual([
    [7, BUILTIN_LAYOUT_IDS.research],
  ]);
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS * 2));
  expect(p.saved).toHaveLength(1);
});

test("a pending save is flushed on pagehide", () => {
  const p = memoryPersistence();
  setup(p);
  fireEvent.click(screen.getByText("research"));
  fireEvent(window, new Event("pagehide"));
  expect(p.saved).toHaveLength(1);
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS));
  expect(p.saved).toHaveLength(1);
});

test("a pending save is flushed when the document becomes hidden", () => {
  const p = memoryPersistence();
  setup(p);
  fireEvent.click(screen.getByText("research"));
  const spy = vi.spyOn(document, "visibilityState", "get").mockReturnValue("hidden");
  fireEvent(document, new Event("visibilitychange"));
  expect(p.saved).toHaveLength(1);
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS));
  expect(p.saved).toHaveLength(1);
  spy.mockRestore();
});

test("unmount flushes exactly once; later timers, pagehide and keydown do nothing", () => {
  const p = memoryPersistence();
  const { unmount } = setup(p);
  fireEvent.click(screen.getByText("research"));
  unmount();
  expect(p.saved).toHaveLength(1);
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS * 2));
  fireEvent(window, new Event("pagehide"));
  fireEvent.keyDown(window, { key: "1", code: "Digit1", metaKey: true });
  expect(p.saved).toHaveLength(1);
});

test("a fresh persistence object each parent render does not defeat the debounce", () => {
  const saved: ViewSnapshot[] = [];
  const mk = (): ViewPersistence => ({ save: (s) => void saved.push(s) });
  function Parent({ tick }: { tick: number }) {
    return (
      <ViewStateProvider initial={initialViewModel(null)} persistence={mk()}>
        <span>{tick}</span>
        <Probe />
      </ViewStateProvider>
    );
  }
  const { rerender } = render(<Parent tick={0} />);
  fireEvent.click(screen.getByText("research"));
  rerender(<Parent tick={1} />);
  act(() => vi.advanceTimersByTime(SAVE_DEBOUNCE_MS - 1));
  rerender(<Parent tick={2} />);
  expect(saved).toHaveLength(0);
  act(() => vi.advanceTimersByTime(1));
  expect(saved).toHaveLength(1);
});

test("Cmd+1…9 selects layouts by order", () => {
  setup();
  fireEvent.keyDown(window, { key: "3", code: "Digit3", metaKey: true });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  fireEvent.keyDown(window, { key: "2", code: "Digit2", metaKey: true, shiftKey: true });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  fireEvent.keyDown(window, { key: "1", code: "Digit1" });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  fireEvent.keyDown(window, { key: "1", code: "Digit1", metaKey: true });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.interview);
});

test("Cmd+digit uses the physical key (AZERTY) and ignores out-of-range, Ctrl, Alt, 0, repeat, IME and inputs", () => {
  setup();
  fireEvent.keyDown(window, { key: "&", code: "Digit3", metaKey: true });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
  fireEvent.click(screen.getByText("research"));
  const ignored = [
    { key: "1", code: "Digit1", ctrlKey: true },
    { key: "1", code: "Digit1", altKey: true },
    { key: "0", code: "Digit0", metaKey: true },
    { key: "3", code: "Digit3", metaKey: true, repeat: true },
    { key: "3", code: "Digit3", metaKey: true, isComposing: true },
  ];
  for (const init of ignored) {
    fireEvent.keyDown(window, init);
    expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.research);
  }
  fireEvent.keyDown(screen.getByLabelText("field"), { key: "3", code: "Digit3", metaKey: true });
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.research);
});

test("Cmd+9 with 3 layouts keeps the selection and does not preventDefault", () => {
  setup();
  fireEvent.click(screen.getByText("research"));
  const missing = new KeyboardEvent("keydown", { key: "9", code: "Digit9", metaKey: true, cancelable: true });
  act(() => void window.dispatchEvent(missing));
  expect(missing.defaultPrevented).toBe(false);
  expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.research);
  const present = new KeyboardEvent("keydown", { key: "1", code: "Digit1", metaKey: true, cancelable: true });
  act(() => void window.dispatchEvent(present));
  expect(present.defaultPrevented).toBe(true);
});

test("hooks outside the provider throw", () => {
  vi.spyOn(console, "error").mockImplementation(() => {});
  expect(() => render(<Probe />)).toThrow(/ViewStateProvider/);
});
```


- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/view
```
Expected: FAIL — `Failed to resolve import "./model"` / `"./store"` (`view/` modülleri henüz yok). `pnpm --dir app exec vitest run ui/no-console.test.ts` `/ui/layout/persistence.ts`'i gösterir; Step 3'te silinince geçer.

- [ ] **Step 3: Uygulamayı yaz**



`app/ui/App.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/App.tsx
+++ b/app/ui/App.tsx
@@ -1,20 +1,25 @@
 import { useState } from "react";
 import { HighlightProvider } from "./highlight/store";
-import { localStoragePersistence, type LayoutPersistence } from "./layout/persistence";
-import { LayoutProvider } from "./layout/store";
 import { SkyProvider } from "./panels/sky";
 import { Shell } from "./shell/Shell";
+import { initialViewModel } from "./view/model";
+import { storePersistence, type ViewPersistence } from "./view/persistence";
+import { ViewStateProvider } from "./view/store";
 
-/** Uygulama kökü. `persistence` testler için verilebilir; varsayılan localStorage'dır. */
-export function App({ persistence }: { persistence?: LayoutPersistence }) {
-  const [store] = useState(() => persistence ?? localStoragePersistence());
+/**
+ * Uygulama kökü. `initialLayout` açılışta store'dan okunan ham düzendir (main.tsx);
+ * `persistence` testler için verilebilir, varsayılan store komutlarıdır.
+ */
+export function App({ initialLayout = null, persistence }: { initialLayout?: unknown; persistence?: ViewPersistence }) {
+  const [store] = useState(() => persistence ?? storePersistence());
+  const [initial] = useState(() => initialViewModel(initialLayout));
   return (
-    <LayoutProvider persistence={store}>
+    <ViewStateProvider initial={initial} persistence={store}>
       <HighlightProvider>
         <SkyProvider>
           <Shell />
         </SkyProvider>
       </HighlightProvider>
-    </LayoutProvider>
+    </ViewStateProvider>
   );
 }
```

`app/ui/dashboard/Dashboard.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/dashboard/Dashboard.tsx
+++ b/app/ui/dashboard/Dashboard.tsx
@@ -6,7 +6,7 @@ import { useTranslation } from "react-i18next";
 import { useEffect, useRef, useState, type PointerEvent as ReactPointerEvent } from "react";
 import { columnsForWidth, movePanel, reflow, resizePanel, GRID_COLUMNS } from "../layout/grid";
 import type { Panel } from "../layout/model";
-import { useLayoutStore } from "../layout/store";
+import { useLayoutStore } from "../view/store";
 import "./dashboard.css";
 import { gridMetrics, pxToGridDelta } from "./metrics";
 import { PanelFrame, type DragMode } from "./PanelFrame";
```

`app/ui/layout/persistence.ts` silinir:

```bash
git rm app/ui/layout/persistence.ts
```

`app/ui/layout/store.tsx` silinir:

```bash
git rm app/ui/layout/store.tsx
```

`app/ui/main.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/main.tsx
+++ b/app/ui/main.tsx
@@ -4,6 +4,7 @@ import { App } from "./App";
 import "./i18n";
 import "./theme/tokens.css";
 import { applyTheme, loadThemePref } from "./theme/theme";
+import { loadLayout } from "./view/persistence";
 
 // İlk çizimden önce yalnızca `data-theme` yazılır (yanlış tema parlaması olmaz). "system"
 // dinleyicisi hemen kaldırılır: sistem değişikliklerini Shell'in kendi applyTheme'i izler,
@@ -11,8 +12,12 @@ import { applyTheme, loadThemePref } from "./theme/theme";
 const disposeBootTheme = applyTheme(loadThemePref());
 disposeBootTheme();
 
-createRoot(document.getElementById("root")!).render(
-  <StrictMode>
-    <App />
-  </StrictMode>,
-);
+// Düzen store'dan okunur (≈ milisaniyeler); ilk çizim kayıtlı düzenle yapılır, yanlış düzen
+// parlaması olmaz. Okuma hatası açılışı engellemez: hazır düzenler kullanılır.
+void loadLayout().then((initialLayout) => {
+  createRoot(document.getElementById("root")!).render(
+    <StrictMode>
+      <App initialLayout={initialLayout} />
+    </StrictMode>,
+  );
+});
```

`app/ui/shell/LayoutPicker.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/LayoutPicker.tsx
+++ b/app/ui/shell/LayoutPicker.tsx
@@ -8,7 +8,7 @@ import { useEffect, useId, useRef, useState, type FormEvent, type KeyboardEvent
 import { useTranslation } from "react-i18next";
 import type { TFunction } from "i18next";
 import { MAX_LAYOUTS, type Layout } from "../layout/model";
-import { useLayoutStore } from "../layout/store";
+import { useLayoutStore } from "../view/store";
 
 export function layoutLabel(t: TFunction, layout: Layout): string {
   return layout.builtin ? t(`layout.${layout.id.slice("builtin:".length)}`) : layout.name;
```

`app/ui/shell/Shell.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/Shell.tsx
+++ b/app/ui/shell/Shell.tsx
@@ -1,7 +1,7 @@
 import { useEffect, useLayoutEffect, useRef, useState } from "react";
 import { Dashboard } from "../dashboard/Dashboard";
 import { isEditableTarget } from "../keyboard";
-import { useLayoutStore } from "../layout/store";
+import { useLayoutStore } from "../view/store";
 import { applyTheme, loadThemePref, saveThemePref, type ThemePref } from "../theme/theme";
 import { PrintView } from "./PrintView";
 import "./shell.css";
```

`app/ui/view/model.ts` oluştur:

```ts
// Görünüm durumu (spec §5 `ViewState`, §11 M5): pano düzeni ve dasha ağacının açık dalı
// tek bir modelde tutulur ve tek bir yerden (`toViewState`) serileştirilir. Hesap ayarları
// (`CalcSettings`) burada değildir; ikisi ayrıdır (spec §4 "Ayar katmanları").
import type { Graha, ViewState } from "../bindings";
import { isGrahaId } from "../chart/types";
import { parseLayoutState, type LayoutState } from "../layout/model";

export const VIEW_STATE_SCHEMA_VERSION = 1;
/** Rust `MAX_DASHA_LEVEL`. */
export const MAX_DASHA_PATH = 5;

export interface ViewModel {
  /** Görünümün yazıldığı harita; danışan açık değilse `null` (yalnızca düzen kaydedilir). */
  chartId: number | null;
  layout: LayoutState;
  /** Dasha ağacında açık dal: kökten lord yolu (`ViewState.dasha_expanded_path`). */
  dashaPath: Graha[];
}

/** Toleranslı: dizi değilse, geçersiz lord içeriyorsa ya da 5'ten uzunsa boş yol (Rust ile aynı). */
export function parseDashaPath(raw: unknown): Graha[] {
  if (!Array.isArray(raw) || raw.length > MAX_DASHA_PATH) return [];
  return raw.every((g) => typeof g === "string" && isGrahaId(g)) ? ([...raw] as Graha[]) : [];
}

/** Açılış modeli: düzen ham kayıttan (toleranslı), dasha yolu açılan haritadan. */
export function initialViewModel(rawLayout: unknown, chart: { id: number; dashaPath: unknown } | null = null): ViewModel {
  return {
    chartId: chart?.id ?? null,
    layout: parseLayoutState(rawLayout),
    dashaPath: chart ? parseDashaPath(chart.dashaPath) : [],
  };
}

/** IPC biçimi (`save_view_state`, `save_panel_layout`). */
export function toViewState(m: ViewModel): ViewState {
  return {
    schema_version: VIEW_STATE_SCHEMA_VERSION,
    panel_layout: { active_layout_id: m.layout.active_layout_id, layouts: m.layout.layouts },
    dasha_expanded_path: [...m.dashaPath],
  };
}
```

`app/ui/view/persistence.ts` oluştur:

```ts
// Görünüm kalıcılığı (spec §4 otomatik kayıt): açık harita varsa `save_view_state`
// (düzen + haritanın dasha yolu), yoksa `save_panel_layout` (yalnızca düzen). Hatalar
// `onError` ile bildirilir ve loglanmaz (parametreler kişisel veri içerebilir).
import { api } from "../api";
import type { CommandError, ViewState } from "../bindings";
import { parseLayoutState } from "../layout/model";

export interface ViewSnapshot {
  chartId: number | null;
  viewState: ViewState;
}

export interface ViewPersistence {
  save(snapshot: ViewSnapshot): void;
}

export function storePersistence(onError: (error: CommandError) => void = () => {}): ViewPersistence {
  return {
    save({ chartId, viewState }) {
      const call = chartId === null ? api.savePanelLayout(viewState.panel_layout) : api.saveViewState(chartId, viewState);
      void call.then((r) => {
        if (r.status === "error") onError(r.error);
      });
    },
  };
}

/** Bellek içi kalıcılık: testler için. `saved` her kaydı sırayla tutar. */
export function memoryPersistence(): ViewPersistence & { saved: ViewSnapshot[] } {
  const saved: ViewSnapshot[] = [];
  return { saved, save: (s) => void saved.push(structuredClone(s)) };
}

/** Plan 04'te düzen localStorage'daydı; store'da düzen yoksa bir kez oraya taşınır. */
export const LEGACY_LAYOUT_KEY = "jyotish-desk.layout.v1";

function readLegacy(storage: Storage): unknown | null {
  try {
    const text = storage.getItem(LEGACY_LAYOUT_KEY);
    return text === null ? null : (JSON.parse(text) as unknown);
  } catch {
    return null;
  }
}

/**
 * Kayıtlı düzen (ham; `initialViewModel` toleranslı okur) ya da `null`. Hata açılışı
 * engellemez: hazır düzenler kullanılır (spec §7).
 */
export async function loadLayout(storage: Storage = window.localStorage): Promise<unknown | null> {
  const stored = await api.loadPanelLayout();
  if (stored.status === "error") return null;
  if (stored.data !== null) return stored.data;
  const legacy = readLegacy(storage);
  if (legacy === null) return null;
  const parsed = parseLayoutState(legacy);
  const moved = await api.savePanelLayout({ active_layout_id: parsed.active_layout_id, layouts: parsed.layouts });
  if (moved.status === "ok") {
    try {
      storage.removeItem(LEGACY_LAYOUT_KEY);
    } catch {
      // Silinemezse bir sonraki açılışta store'daki düzen önce gelir; zararı yok.
    }
  }
  return parsed;
}
```

`app/ui/view/reducer.ts` oluştur:

```ts
// Görünüm modeli için saf reducer: düzen eylemleri düzen reducer'ına gider, dasha yolu ve
// açılan harita burada değişir.
import type { Graha } from "../bindings";
import { layoutReducer, type LayoutAction } from "../layout/reducer";
import { parseDashaPath, type ViewModel } from "./model";

export type ViewAction =
  | { type: "layout"; action: LayoutAction }
  | { type: "setDashaPath"; path: Graha[] }
  | { type: "openChart"; chartId: number; dashaPath: unknown };

export function viewReducer(state: ViewModel, action: ViewAction): ViewModel {
  switch (action.type) {
    case "layout": {
      const layout = layoutReducer(state.layout, action.action);
      return layout === state.layout ? state : { ...state, layout };
    }
    case "setDashaPath": {
      const path = parseDashaPath(action.path);
      const same = path.length === state.dashaPath.length && path.every((g, i) => g === state.dashaPath[i]);
      return same ? state : { ...state, dashaPath: path };
    }
    case "openChart":
      return { ...state, chartId: action.chartId, dashaPath: parseDashaPath(action.dashaPath) };
  }
}
```

`app/ui/view/store.tsx` oluştur:

```tsx
// Görünüm durumunun tek merkezi (spec §11 M5): düzenler ve dasha yolu tek saf reducer'dan geçer;
// ≈ 500 ms debounce ile kayıt (spec §4), pencere gizlenince ya da kapanınca hemen kayıt,
// Cmd+1…9 ile sıradaki düzene geçiş (UI spec §3.2). Paneller düzene `useLayoutStore`,
// dasha yoluna `useViewStore` ile ulaşır; birbirini çağırmaz.
import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type Dispatch, type ReactNode } from "react";
import type { Graha, ViewState } from "../bindings";
import { isEditableTarget } from "../keyboard";
import type { Layout, LayoutState, Panel } from "../layout/model";
import { activeLayout, newId, normalizeState, type LayoutAction } from "../layout/reducer";
import { toViewState, type ViewModel } from "./model";
import type { ViewPersistence } from "./persistence";
import { viewReducer, type ViewAction } from "./reducer";

export const SAVE_DEBOUNCE_MS = 500;

export interface LayoutStore {
  state: LayoutState;
  active: Layout;
  dispatch: Dispatch<LayoutAction>;
  select(id: string): void;
  saveAs(name: string): void;
  rename(id: string, name: string): void;
  duplicate(sourceId: string, name: string): void;
  remove(id: string): void;
  resetBuiltin(id: string): void;
  updatePanels(panels: Panel[]): void;
  toggleNotes(): void;
}

export interface ViewStore {
  model: ViewModel;
  /** Serileştirilmiş biçim (kayıtla aynı). */
  viewState: ViewState;
  setDashaPath(path: Graha[]): void;
  /** Yeni açılan harita: bekleyen kayıt önceki haritaya yazılır, yeni haritanın yolu yüklenir. */
  openChart(chartId: number, dashaPath: unknown): void;
}

const ViewContext = createContext<{ view: ViewStore; layout: LayoutStore } | null>(null);

export function ViewStateProvider({
  initial,
  persistence,
  children,
  debounceMs = SAVE_DEBOUNCE_MS,
}: {
  initial: ViewModel;
  persistence: ViewPersistence;
  children: ReactNode;
  debounceMs?: number;
}) {
  const [state, setState] = useState(() => ({ ...initial, layout: normalizeState(initial.layout) }));

  // Üst bileşen her render'da yeni bir `persistence` verebilir; debounce sıfırlanmasın.
  const persistenceRef = useRef(persistence);
  persistenceRef.current = persistence;
  const debounceRef = useRef(debounceMs);
  debounceRef.current = debounceMs;

  // Model `dispatch` içinde eşzamanlı ilerler; `stateRef` asıl kaynaktır, `state` onun render'a
  // yansımasıdır. Böylece kirli durum ve kayıt, render ya da effect zamanlamasına bağlı kalmaz:
  // aynı tick'te gelen eylemler ve `openChart` sırayla, kayıpsız uygulanır.
  const stateRef = useRef(state);
  /** Kaydı bekleyen son durum; `null` ise son durum kayıtlıdır (açılış ya da yeni açılan harita). */
  const pending = useRef<ViewModel | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const flushRef = useRef(() => {
    clearTimeout(timer.current);
    if (pending.current) {
      const m = pending.current;
      pending.current = null;
      persistenceRef.current.save({ chartId: m.chartId, viewState: toViewState(m) });
    }
  });

  const dispatch = useCallback((action: ViewAction) => {
    // Yeni harita: bekleyen kayıt önce önceki haritaya yazılır; açılan haritanın kendi durumu
    // store'dan geldiği için yeniden yazılmaz.
    if (action.type === "openChart") flushRef.current();
    const next = viewReducer(stateRef.current, action);
    if (next === stateRef.current) return;
    stateRef.current = next;
    if (action.type !== "openChart") {
      pending.current = next;
      clearTimeout(timer.current);
      timer.current = setTimeout(flushRef.current, debounceRef.current);
    }
    setState(next);
  }, []);

  // Tauri webview kapanışı `pagehide` atlayabildiği için `visibilitychange` de dinlenir.
  useEffect(() => {
    const flush = () => flushRef.current();
    const onVisibility = () => {
      if (document.visibilityState === "hidden") flush();
    };
    window.addEventListener("pagehide", flush);
    document.addEventListener("visibilitychange", onVisibility);
    return () => {
      window.removeEventListener("pagehide", flush);
      document.removeEventListener("visibilitychange", onVisibility);
      flush();
    };
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.repeat || e.isComposing || isEditableTarget(e.target)) return;
      if (!e.metaKey || e.shiftKey || e.altKey || e.ctrlKey) return;
      // Fiziksel tuş (`code`): AZERTY'de `key` rakam değildir.
      if (!/^Digit[1-9]$/.test(e.code)) return;
      const index = Number(e.code.slice(5)) - 1;
      if (index >= stateRef.current.layout.layouts.length) return;
      e.preventDefault();
      dispatch({ type: "layout", action: { type: "selectIndex", index } });
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const value = useMemo(() => {
    const layoutDispatch: Dispatch<LayoutAction> = (action) => dispatch({ type: "layout", action });
    const layout: LayoutStore = {
      state: state.layout,
      active: activeLayout(state.layout),
      dispatch: layoutDispatch,
      select: (id) => layoutDispatch({ type: "select", id }),
      saveAs: (name) => layoutDispatch({ type: "saveAs", id: newId("user"), name }),
      rename: (id, name) => layoutDispatch({ type: "rename", id, name }),
      duplicate: (sourceId, name) => layoutDispatch({ type: "duplicate", sourceId, id: newId("user"), name }),
      remove: (id) => layoutDispatch({ type: "delete", id }),
      resetBuiltin: (id) => layoutDispatch({ type: "resetBuiltin", id }),
      updatePanels: (panels) => layoutDispatch({ type: "updatePanels", panels }),
      toggleNotes: () => layoutDispatch({ type: "toggleNotes", panelId: newId("notes") }),
    };
    const view: ViewStore = {
      model: state,
      viewState: toViewState(state),
      setDashaPath: (path) => dispatch({ type: "setDashaPath", path }),
      openChart: (chartId, dashaPath) => dispatch({ type: "openChart", chartId, dashaPath }),
    };
    return { view, layout };
  }, [state, dispatch]);
  return <ViewContext.Provider value={value}>{children}</ViewContext.Provider>;
}

function useViewContext() {
  const ctx = useContext(ViewContext);
  if (ctx === null) throw new Error("view state hooks must be used inside <ViewStateProvider>");
  return ctx;
}

export function useViewStore(): ViewStore {
  return useViewContext().view;
}

export function useLayoutStore(): LayoutStore {
  return useViewContext().layout;
}
```


- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app exec vitest run ui/view ui/no-console.test.ts
pnpm --dir app test
pnpm --dir app build
```
Expected: tam takımda `Test Files  27 passed (27)`, `Tests  226 passed (226)`; `build` hatasız (`tsc --noEmit` çıktısız, `vite build` `✓ built`).

- [ ] **Step 5: Commit**

```bash
git add -A app/ui/App.test.tsx app/ui/App.tsx app/ui/dashboard/Dashboard.test.tsx app/ui/dashboard/Dashboard.tsx app/ui/layout/persistence.ts app/ui/layout/store.test.tsx app/ui/layout/store.tsx app/ui/main.tsx app/ui/no-console.test.ts app/ui/shell/LayoutPicker.test.tsx app/ui/shell/LayoutPicker.tsx app/ui/shell/Shell.tsx app/ui/view/model.test.ts app/ui/view/model.ts app/ui/view/persistence.test.ts app/ui/view/persistence.ts app/ui/view/reducer.ts app/ui/view/store.test.tsx app/ui/view/store.tsx
git diff --cached -U0 -- . ':!app/pnpm-lock.yaml' ':!*.png' ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git commit -m "feat(ui): central ViewState store with store-backed persistence and a one-time localStorage move

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Oturum, açılış geri yüklemesi, bildirimler ve uyarı rozetleri; M0 denemesinin kaldırılması

Açık harita artık store'dan gelir (D2): `boot()` düzeni ve son oturumu paralel okur, `SessionProvider` açık `OpenedClient`'ı tutar, harita ve tablo panelleri `useSession()`'dan çizer. `StoreWarning` ve açılış/kayıt hataları modal olmayan bildirim bandında, an ve harita uyarıları üst çubukta rozet olarak görünür (D14). M0 "şimdiki gökyüzü" denemesi (`planet_longitudes`, `sky.tsx`, `time.ts`) silinir (D8). `chart/adapter.ts` `Chart`'tan `ChartData` kurar; varga işaretleri D12'ye göredir.

**Files:**
- Modify: `app/src-tauri/src/commands.rs`
- Modify: `app/src-tauri/src/lib.rs`
- Modify: `app/ui/App.test.tsx`
- Modify: `app/ui/App.tsx`
- Modify: `app/ui/PrintSpike.test.tsx`
- Modify: `app/ui/PrintSpike.tsx`
- Modify: `app/ui/api.test.ts`
- Modify: `app/ui/api.ts`
- Modify: `app/ui/bindings.ts` (üretilir: `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings`)
- Create: `app/ui/boot.ts`
- Modify: `app/ui/chart/adapter.test.ts`
- Modify: `app/ui/chart/adapter.ts`
- Modify: `app/ui/chart/types.ts`
- Modify: `app/ui/dashboard/Dashboard.test.tsx`
- Modify: `app/ui/i18n/en.ts`
- Create: `app/ui/i18n/errors.ts`
- Modify: `app/ui/i18n/fr.ts`
- Modify: `app/ui/i18n/i18n.test.ts`
- Modify: `app/ui/i18n/tr.ts`
- Modify: `app/ui/main.tsx`
- Modify: `app/ui/panels/ChartPanel.tsx`
- Modify: `app/ui/panels/GrahaTablePanel.tsx`
- Modify: `app/ui/panels/panels.test.tsx`
- Delete: `app/ui/panels/sky.tsx`
- Modify: `app/ui/panels/states.tsx`
- Create: `app/ui/session/store.tsx`
- Create: `app/ui/shell/ChartTitle.tsx`
- Modify: `app/ui/shell/PrintView.tsx`
- Modify: `app/ui/shell/Shell.tsx`
- Modify: `app/ui/shell/TopBar.tsx`
- Create: `app/ui/shell/notices.tsx`
- Modify: `app/ui/shell/shell.css`
- Create: `app/ui/test/render.tsx`
- Delete: `app/ui/time.test.ts`
- Delete: `app/ui/time.ts`

**Interfaces:**
- Consumes: `api.restoreSession`, `api.loadPanelLayout` (Task 3), `ViewStateProvider`, `initialViewModel`, `storePersistence` (Task 4), `fixture()` (Task 3).
- Produces: `boot(): Promise<Boot {layout, opened, error}>`, `EMPTY_BOOT`; `App({boot?, persistence?})`; `session/store.tsx`: `SessionProvider({initial})`, `useSession(): Session {opened, open(o)}` (`open` görünümü `openChart` ile yeni haritaya bağlar); `shell/notices.tsx`: `Notice`, `NoticeProvider`, `useNotices(): Notices {notices, push, dismiss}`, `storeWarningNotices(warnings)`, `NoticeBar`; `shell/ChartTitle.tsx`; `chart/adapter.ts`: `SIGNS`, `VARGAS`, `isVarga`, `signIndex`, `chartDataFromChart(chart, varga, {showLagna}): ChartData | null`; `test/render.tsx`: `renderWithStores(ui, {opened?, layout?, debounceMs?})`; i18n anahtarları `chart.vargaMissing`, `chart.kind.*`, `panel.noChart`, `badge.*`, `badgeHint.*`, `notice.*`, `storeWarning.*`, `shell.warnings`, `errors.*` (arka uç kodları).

- [ ] **Step 1: Önce testleri yaz**



`app/ui/App.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/App.test.tsx
+++ b/app/ui/App.test.tsx
@@ -3,42 +3,72 @@ import { vi } from "vitest";
 
 vi.mock("./api", () => ({
   api: {
-    engineStatus: vi.fn(async () => ({ status: "ok", data: { seVersion: "2.10.03" } })),
-    planetLongitudes: vi.fn(async () => ({
-      status: "ok",
-      data: [
-        { body: "Sun", lon: 256.5, retrograde: false },
-        { body: "Saturn", lon: 10.25, retrograde: true },
-      ],
-    })),
     printWindow: vi.fn(async () => ({ status: "ok", data: null })),
   },
 }));
 
 import { api } from "./api";
 import { App } from "./App";
-import { memoryPersistence } from "./view/persistence";
+import type { Boot } from "./boot";
+import { fixture, unknownTimeOpened } from "./test/fixture";
 import { THEME_STORAGE_KEY } from "./theme/theme";
+import { memoryPersistence } from "./view/persistence";
 
-function renderApp() {
-  return render(<App persistence={memoryPersistence()} />);
+function renderApp(boot: Partial<Boot> = {}) {
+  const full: Boot = { layout: null, opened: fixture().opened, error: null, ...boot };
+  return render(<App boot={full} persistence={memoryPersistence()} />);
 }
 
-test("opens on the interview layout with the current-sky chart", async () => {
+test("opens on the interview layout with the restored chart", () => {
   renderApp();
   expect(screen.getByRole("combobox", { name: "Düzen" })).toHaveDisplayValue("Görüşme");
-  expect(await screen.findByRole("group", { name: "D1 haritası" })).toBeInTheDocument();
-  expect(screen.getByRole("note")).toHaveTextContent("Lagna henüz hesaplanmıyor; şimdiki gökyüzü, 1. ev Koç");
-  expect(screen.getByText("D9 haritası sonraki bir sürümde gelecek.")).toBeInTheDocument();
+  expect(screen.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeInTheDocument();
+  expect(screen.getByText("Doğum haritası")).toBeInTheDocument();
+  expect(screen.getByRole("group", { name: "D1 haritası" })).toBeInTheDocument();
+  expect(screen.getByRole("group", { name: "D9 haritası" })).toBeInTheDocument();
   expect(screen.getByRole("region", { name: "Notlar" })).toBeInTheDocument();
-  expect(api.planetLongitudes).toHaveBeenCalled();
+  expect(screen.queryByRole("list", { name: "Uyarılar" })).toBeNull();
+});
+
+test("without a session the desk shows the app title and empty panels", () => {
+  renderApp({ opened: null });
+  expect(screen.getByRole("heading", { level: 1, name: "Jyotish Desk" })).toBeInTheDocument();
+  expect(screen.getAllByText("Açık harita yok.").length).toBeGreaterThan(0);
+});
+
+test("moment warnings are badges with a text label and a hint", () => {
+  renderApp({ opened: unknownTimeOpened() });
+  const badges = screen.getByRole("list", { name: "Uyarılar" });
+  const badge = within(badges).getByText("Saat bilinmiyor").closest("li")!;
+  expect(badge).toHaveAttribute("title", "Doğum saati bilinmiyor; öğlen 12:00 alındı.");
+  expect(badge).toHaveAttribute("tabindex", "0");
+  expect(badge).toHaveTextContent(": Doğum saati bilinmiyor; öğlen 12:00 alındı.");
+});
+
+test("a restore failure and store warnings are shown in the notice bar and can be dismissed", () => {
+  const { unmount } = renderApp({ opened: null, error: { code: "store_sqlite", params: { detail: "disk" } } });
+  const region = screen.getByRole("region", { name: "Bildirimler" });
+  expect(within(region).getByRole("alert")).toHaveTextContent(
+    "Son oturum açılamadı: Danışan veritabanı okunamadı ya da yazılamadı (disk).",
+  );
+  fireEvent.click(within(region).getByRole("button", { name: "Kapat" }));
+  expect(screen.queryByRole("region", { name: "Bildirimler" })).toBeNull();
+  unmount();
+  const opened = fixture().opened;
+  opened.warnings = ["LastChartMissing", { TzdbChanged: { stored_version: "2025a", current_version: "2026a", current_utc: null } }];
+  renderApp({ opened });
+  const statuses = within(screen.getByRole("region", { name: "Bildirimler" })).getAllByRole("status");
+  expect(statuses.map((s) => s.textContent)).toEqual([
+    "UyarıSon açılan harita silinmiş; en son güncellenen harita açıldı.Kapat",
+    "UyarıSaat dilimi verisi değişti (2025a → 2026a); saklanan an kullanılıyor.Kapat",
+  ]);
 });
 
-test("Cmd+2 switches to the research layout with the graha table", async () => {
+test("Cmd+2 switches to the research layout with the graha table", () => {
   renderApp();
   fireEvent.keyDown(window, { key: "2", code: "Digit2", metaKey: true });
   expect(screen.getByRole("combobox", { name: "Düzen" })).toHaveDisplayValue("Araştırma");
-  const table = await screen.findByRole("region", { name: "Graha tablosu" });
+  const table = screen.getByRole("region", { name: "Graha tablosu" });
   expect(within(table).getByRole("button", { name: "Satürn" }).closest("tr")).toHaveTextContent("℞");
 });
 
@@ -93,6 +123,7 @@ test("language menu switches every string", async () => {
   fireEvent.change(screen.getByRole("combobox", { name: "Dil" }), { target: { value: "en" } });
   expect(await screen.findByRole("combobox", { name: "Layout" })).toHaveDisplayValue("Consultation");
   expect(screen.getByRole("group", { name: "D1 chart" })).toBeInTheDocument();
+  expect(screen.getByText("Natal chart")).toBeInTheDocument();
   expect(screen.getByRole("combobox", { name: "Theme" })).toBeInTheDocument();
 });
 
@@ -100,6 +131,7 @@ test("the print test stays reachable and calls printWindow", async () => {
   renderApp();
   fireEvent.click(screen.getByRole("button", { name: "Baskı denemesi" }));
   expect(await screen.findByText(/Doğum haritası — ğüşıöç İ/)).toBeInTheDocument();
+  expect(screen.getByTestId("sign-8")).toHaveTextContent("Sun");
   fireEvent.click(screen.getByRole("button", { name: "Yazdır…" }));
   expect(api.printWindow).toHaveBeenCalledOnce();
   fireEvent.click(screen.getByRole("button", { name: "Panoya dön" }));
@@ -113,9 +145,3 @@ test("a print failure is shown with a retry", async () => {
   fireEvent.click(screen.getByRole("button", { name: "Yazdır…" }));
   expect(await screen.findByRole("alert")).toHaveTextContent("Yazdırma penceresi açılamadı.");
 });
-
-test("a failed sky request shows an in-panel error instead of a blank desk", async () => {
-  vi.mocked(api.planetLongitudes).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
-  renderApp();
-  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
-});
```

`app/ui/PrintSpike.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/PrintSpike.test.tsx
+++ b/app/ui/PrintSpike.test.tsx
@@ -2,8 +2,8 @@ import { render, screen } from "@testing-library/react";
 import { PrintSpike } from "./PrintSpike";
 
 const rows = [
-  { body: "Sun", lon: 256.5, retrograde: false },
-  { body: "Moon", lon: 10.0, retrograde: false },
+  { body: "Sun", sign: 8, lon: 256.5, retrograde: false },
+  { body: "Moon", sign: 0, lon: 10.0, retrograde: false },
 ];
 
 test("renders two A4 pages with chart svg and planet table", () => {
@@ -16,9 +16,9 @@ test("renders two A4 pages with chart svg and planet table", () => {
   expect(screen.getByText("राशि")).toBeInTheDocument();
 });
 
-test("planet sign index is floor(lon / 30)", () => {
+test("planets are placed by the sign the backend sent", () => {
   render(<PrintSpike rows={rows} />);
-  // Sun 256.5° → 9. burç (Dhanu, index 8); hücrede "Sun" etiketi.
+  // Sun: Dhanu (index 8); hücrede "Sun" etiketi.
   const cell = screen.getByTestId("sign-8");
   expect(cell).toHaveTextContent("Sun");
 });
```

`app/ui/api.test.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -3,7 +3,6 @@ import { vi } from "vitest";
 vi.mock("./bindings", () => ({
   commands: {
     engineStatus: vi.fn(),
-    planetLongitudes: vi.fn(),
     printWindow: vi.fn(),
     restoreSession: vi.fn(),
     createClient: vi.fn(),
@@ -30,8 +29,8 @@ test("passes ok results through", async () => {
 
 test("passes real CommandError objects through", async () => {
   const error = { code: "calc", params: { detail: "x" } };
-  vi.mocked(commands.planetLongitudes).mockResolvedValueOnce({ status: "error", error });
-  expect(await api.planetLongitudes(1)).toEqual({ status: "error", error });
+  vi.mocked(commands.restoreSession).mockResolvedValueOnce({ status: "error", error });
+  expect(await api.restoreSession()).toEqual({ status: "error", error });
 });
 
 test("normalizes a string rejection into an ipc error", async () => {
```

`app/ui/chart/adapter.test.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/chart/adapter.test.ts
+++ b/app/ui/chart/adapter.test.ts
@@ -1,37 +1,52 @@
-import { chartDataFromPlanetRows, signAndDegree } from "./adapter";
+import { fixture } from "../test/fixture";
+import { chartDataFromChart, isVarga, signIndex, SIGNS, VARGAS } from "./adapter";
 
-test("sign is floor(lon / 30) and degInSign the remainder", () => {
-  expect(signAndDegree(0)).toEqual({ sign: 0, degInSign: 0 });
-  expect(signAndDegree(256.5)).toEqual({ sign: 8, degInSign: 16.5 });
-  expect(signAndDegree(359.75)).toEqual({ sign: 11, degInSign: 29.75 });
-  expect(signAndDegree(360)).toEqual({ sign: 0, degInSign: 0 });
-  expect(signAndDegree(-30)).toEqual({ sign: 11, degInSign: 0 });
+test("sign and varga lists follow the Rust enums", () => {
+  expect(SIGNS).toHaveLength(12);
+  expect(signIndex("Aries")).toBe(0);
+  expect(signIndex("Pisces")).toBe(11);
+  expect(VARGAS).toEqual(fixture().opened.chart.vargas.map((v) => v.varga));
+  expect(isVarga("D60")).toBe(true);
+  expect(isVarga("D5")).toBe(false);
 });
 
-test("rows map to chart planets; null longitudes and unknown bodies are skipped", () => {
-  const data = chartDataFromPlanetRows([
-    { body: "Sun", lon: 256.5, retrograde: false },
-    { body: "Saturn", lon: 10.25, retrograde: true },
-    { body: "Mars", lon: null, retrograde: false },
-    { body: "Pluto", lon: 100, retrograde: false },
-    { body: "Rahu", lon: 100, retrograde: true },
-    { body: "Ketu", lon: 280, retrograde: true },
-  ]);
-  expect(data.ascSign).toBeNull();
-  expect(data.planets).toEqual([
-    { id: "Sun", sign: 8, degInSign: 16.5, retrograde: false },
-    { id: "Saturn", sign: 0, degInSign: 10.25, retrograde: true },
-    { id: "Rahu", sign: 3, degInSign: 10, retrograde: true },
-    { id: "Ketu", sign: 9, degInSign: 10, retrograde: true },
-  ]);
+test("D1 takes signs, degrees and markers from the chart without deriving anything", () => {
+  const { chart } = fixture().opened;
+  const data = chartDataFromChart(chart, "D1", { showLagna: true })!;
+  expect(data.ascSign).toBe(signIndex(chart.lagna.sign));
+  expect(data.planets).toHaveLength(9);
+  for (const g of chart.grahas) {
+    const p = data.planets.find((x) => x.id === g.graha)!;
+    expect(p).toEqual({
+      id: g.graha,
+      sign: signIndex(g.sign),
+      degInSign: chart.vargas[0].placements.find((x) => x.graha === g.graha)!.deg,
+      retrograde: g.retrograde,
+      combust: g.combust,
+      vargottama: g.vargottama,
+    });
+  }
 });
 
-test("non-finite longitudes are skipped explicitly", () => {
-  const data = chartDataFromPlanetRows([
-    { body: "Sun", lon: Number.NaN, retrograde: false },
-    { body: "Moon", lon: Number.POSITIVE_INFINITY, retrograde: false },
-    { body: "Mars", lon: Number.NEGATIVE_INFINITY, retrograde: false },
-    { body: "Venus", lon: 45, retrograde: false },
-  ]);
-  expect(data.planets).toEqual([{ id: "Venus", sign: 1, degInSign: 15, retrograde: false }]);
+test("D9 uses its own lagna and placements; combust is D1-only, vargottama D1 and D9", () => {
+  const { chart } = fixture().opened;
+  chart.grahas[0].combust = true;
+  chart.grahas[0].vargottama = true;
+  const d9 = chart.vargas.find((v) => v.varga === "D9")!;
+  const data = chartDataFromChart(chart, "D9", { showLagna: true })!;
+  expect(data.ascSign).toBe(signIndex(d9.lagna_sign));
+  const sun = data.planets.find((p) => p.id === "Sun")!;
+  expect(sun.sign).toBe(signIndex(d9.placements.find((p) => p.graha === "Sun")!.sign));
+  expect([sun.combust, sun.vargottama]).toEqual([false, true]);
+  const d10 = chartDataFromChart(chart, "D10", { showLagna: true })!;
+  expect(d10.planets.find((p) => p.id === "Sun")!.vargottama).toBe(false);
+});
+
+test("hidden lagna, missing vargas and null degrees", () => {
+  const { chart } = fixture().opened;
+  expect(chartDataFromChart(chart, "D1", { showLagna: false })!.ascSign).toBeNull();
+  chart.vargas[0].placements[0].deg = null;
+  expect(Number.isNaN(chartDataFromChart(chart, "D1", { showLagna: true })!.planets[0].degInSign)).toBe(true);
+  chart.vargas = [];
+  expect(chartDataFromChart(chart, "D1", { showLagna: true })).toBeNull();
 });
```

`app/ui/dashboard/Dashboard.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/dashboard/Dashboard.test.tsx
+++ b/app/ui/dashboard/Dashboard.test.tsx
@@ -1,23 +1,9 @@
-import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
+import { fireEvent, screen, waitFor } from "@testing-library/react";
 import { afterEach, vi } from "vitest";
 
-vi.mock("../api", () => ({
-  api: {
-    planetLongitudes: vi.fn(async () => ({
-      status: "ok",
-      data: [{ body: "Sun", lon: 256.5, retrograde: false }],
-    })),
-  },
-}));
-
-import { api } from "../api";
-import { HighlightProvider } from "../highlight/store";
 import { hasOverlap } from "../layout/grid";
 import type { LayoutState, Panel } from "../layout/model";
-import { SkyProvider } from "../panels/sky";
-import { initialViewModel } from "../view/model";
-import { memoryPersistence } from "../view/persistence";
-import { ViewStateProvider } from "../view/store";
+import { renderWithStores } from "../test/render";
 import { Dashboard } from "./Dashboard";
 import { GRID_GAP_PX } from "./metrics";
 
@@ -46,16 +32,7 @@ afterEach(() => {
 const MOVE = { buttons: 1, pointerId: 1 };
 
 function setup(width = WIDTH, panels = PANELS, initial: LayoutState = state(panels)) {
-  const persistence = memoryPersistence();
-  const utils = render(
-    <ViewStateProvider initial={initialViewModel(initial)} persistence={persistence} debounceMs={0}>
-      <HighlightProvider>
-        <SkyProvider now={() => 0}>
-          <Dashboard width={width} />
-        </SkyProvider>
-      </HighlightProvider>
-    </ViewStateProvider>,
-  );
+  const utils = renderWithStores(<Dashboard width={width} />, { layout: initial });
   const frame = (id: string) => utils.container.querySelector<HTMLElement>(`[data-panel-id="${id}"]`)!;
   const box = (id: string) => {
     const f = frame(id);
@@ -68,7 +45,7 @@ function setup(width = WIDTH, panels = PANELS, initial: LayoutState = state(pane
       w: Number(f.dataset.w),
       h: Number(f.dataset.h),
     }));
-  return { ...utils, persistence, frame, box, all };
+  return { ...utils, frame, box, all };
 }
 
 test("renders every panel of the active layout on a 12-column grid", async () => {
@@ -146,18 +123,6 @@ test("narrow windows reflow to 8 and 4 columns read-only", async () => {
   expect(narrow.container.querySelectorAll('.dashboard[data-columns="4"]')).toHaveLength(1);
 });
 
-test("the header shows an inline loading indicator between 100 ms and 1 s", async () => {
-  vi.useFakeTimers();
-  let resolve: (v: Awaited<ReturnType<typeof api.planetLongitudes>>) => void = () => {};
-  vi.mocked(api.planetLongitudes).mockReturnValueOnce(new Promise((r) => (resolve = r)));
-  setup(WIDTH, [PANELS[0]]);
-  expect(screen.queryByRole("status")).toBeNull();
-  act(() => vi.advanceTimersByTime(150));
-  expect(screen.getByRole("status")).toHaveTextContent("Yükleniyor");
-  await act(async () => resolve({ status: "ok", data: [] }));
-  expect(screen.queryByRole("status")).toBeNull();
-});
-
 // ---- Sürükleme sağlamlığı ----
 
 function listenerBalance() {
```

`app/ui/i18n/i18n.test.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/i18n.test.ts
+++ b/app/ui/i18n/i18n.test.ts
@@ -46,7 +46,7 @@ test("each language has 12 distinct sign abbreviations", () => {
 
 test("Turkish is the default and initialisation is synchronous", () => {
   expect(currentLanguage()).toBe("tr");
-  expect(i18n.t("chart.noLagna")).toBe("Lagna henüz hesaplanmıyor; şimdiki gökyüzü, 1. ev Koç");
+  expect(i18n.t("panel.noChart")).toBe("Açık harita yok.");
   expect(i18n.t("errors.calc", { detail: "x<y" })).toBe("Hesap tamamlanamadı (x<y).");
 });
 
@@ -65,3 +65,24 @@ test("chart accessible-name words exist in every language", () => {
   expect([en.chart.retro, en.chart.combust, en.chart.vargottama]).toEqual(["retrograde", "combust", "vargottama"]);
   expect([fr.chart.retro, fr.chart.combust, fr.chart.vargottama]).toEqual(["rétrograde", "combuste", "vargottama"]);
 });
+
+// Arka ucun döndürebildiği her `CommandError.code` (commands.rs, client_commands.rs,
+// atlas_commands.rs). Dosya ve içe aktarma kodları M6'da arayüze açılınca eklenir.
+const BACKEND_CODES = [
+  "ipc", "calc", "engine_stopped", "data_files_missing", "invalid_path", "env_override", "already_open", "job_panicked", "print_failed",
+  "store_sqlite", "store_migration", "store_json", "store_too_new", "store_unavailable", "not_found", "invalid_input",
+  "unknown_time_zone", "nonexistent_local_time", "ambiguous_local_time", "offset_out_of_range", "longitude_out_of_range",
+  "date_out_of_range", "last_chart", "unsupported_schema_version", "invalid_settings", "invalid_location", "invalid_julian_day",
+  "non_finite_result", "invalid_dasha_input", "panchang_search", "invalid_transit_input",
+  "atlas_unavailable", "atlas_sqlite", "atlas_io", "atlas_schema",
+] as const;
+
+test("every backend error code reachable from the UI has a message", () => {
+  for (const code of BACKEND_CODES) expect(Object.keys(tr.errors), code).toContain(code);
+});
+
+test("warning badges and hints cover every moment and chart warning", () => {
+  const warnings = ["TimeUnknownNoon", "LmtFromLongitude", "PreTzdbReliability", "LowPrecision", "PlacidusPolarFallback", "SunriseUnavailable"];
+  expect(Object.keys(tr.badge).sort()).toEqual([...warnings].sort());
+  expect(Object.keys(tr.badgeHint).sort()).toEqual([...warnings].sort());
+});
```

`app/ui/panels/panels.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -1,30 +1,12 @@
 import { act, fireEvent, render, screen } from "@testing-library/react";
-import type { ReactNode } from "react";
-import { vi } from "vitest";
-
-vi.mock("../api", () => ({
-  api: {
-    planetLongitudes: vi.fn(async () => ({
-      status: "ok",
-      data: [
-        { body: "Sun", lon: 256.5, retrograde: false },
-        { body: "Saturn", lon: 10.25, retrograde: true },
-        { body: "Mars", lon: null, retrograde: false },
-      ],
-    })),
-  },
-}));
-
-import { api } from "../api";
-import { HighlightProvider } from "../highlight/store";
 import { setLanguage } from "../i18n";
 import { tr } from "../i18n/tr";
 import { PANEL_KINDS, type Panel } from "../layout/model";
+import { fixture } from "../test/fixture";
+import { renderWithStores } from "../test/render";
 import { ChartPanel } from "./ChartPanel";
 import { GrahaTablePanel } from "./GrahaTablePanel";
 import { panelDefinition, PANEL_REGISTRY } from "./registry";
-import { SkyProvider } from "./sky";
-import { PanelStatusContext } from "./states";
 import { UnavailablePanel } from "./UnavailablePanel";
 
 const chartPanel = (settings: Record<string, unknown> = { varga: "D1", style: "north" }): Panel => ({
@@ -38,17 +20,6 @@ const chartPanel = (settings: Record<string, unknown> = { varga: "D1", style: "n
 });
 const tablePanel: Panel = { id: "t", kind: "graha_table", x: 6, y: 0, w: 6, h: 8, settings: {} };
 
-function Providers({ children }: { children: ReactNode }) {
-  return (
-    <HighlightProvider>
-      <SkyProvider now={() => 0}>{children}</SkyProvider>
-    </HighlightProvider>
-  );
-}
-
-beforeEach(() => vi.mocked(api.planetLongitudes).mockClear());
-afterEach(() => vi.useRealTimers());
-
 test("registry covers the seven kinds; only chart and graha_table are implemented here", () => {
   expect(Object.keys(PANEL_REGISTRY).sort()).toEqual([...PANEL_KINDS].sort());
   expect(PANEL_REGISTRY.chart.component).toBe(ChartPanel);
@@ -60,75 +31,79 @@ test("registry covers the seven kinds; only chart and graha_table are implemente
   const keys = new Set(Object.keys(tr.panel).map((k) => `panel.${k}`));
   for (const kind of [...PANEL_KINDS, "yoga_list"]) expect(keys.has(panelDefinition(kind).titleKey)).toBe(true);
   expect(PANEL_REGISTRY.chart.subtitle!(chartPanel({ varga: "D9" }))).toBe("D9");
+  expect(PANEL_REGISTRY.chart.subtitle!(chartPanel({ varga: "D5" }))).toBe("D1");
 });
 
-test("chart panel draws the current sky with the no-lagna notice, fetching once for jd(now)", async () => {
-  render(
-    <Providers>
-      <ChartPanel panel={chartPanel()} />
-      <GrahaTablePanel panel={tablePanel} />
-    </Providers>,
-  );
-  expect(await screen.findByRole("note")).toHaveTextContent("Lagna henüz hesaplanmıyor; şimdiki gökyüzü, 1. ev Koç");
+test("chart panel draws the open chart's D1 with its lagna", () => {
+  const { container } = renderWithStores(<ChartPanel panel={chartPanel()} />);
   expect(screen.getByRole("group", { name: "D1 haritası" })).toBeInTheDocument();
   expect(screen.getByRole("button", { name: "Gü, Yay 16°" })).toBeInTheDocument();
-  expect(api.planetLongitudes).toHaveBeenCalledTimes(1);
-  expect(api.planetLongitudes).toHaveBeenCalledWith(2440587.5);
+  // Lagna İkizler (İki): 1. ev.
+  expect(screen.getByRole("button", { name: "1. ev, İki" })).toBeInTheDocument();
+  expect(container.querySelector("svg")!.dataset.style).toBe("north");
 });
 
-test("chart panel honours the style setting and falls back to north", async () => {
-  const { container, unmount } = render(
-    <Providers>
-      <ChartPanel panel={chartPanel({ varga: "D1", style: "south" })} />
-    </Providers>,
-  );
-  await screen.findByRole("group");
+test("chart panel draws any varga of the chart and honours the style setting", () => {
+  const { container } = renderWithStores(<ChartPanel panel={chartPanel({ varga: "D9", style: "south" })} />);
+  expect(screen.getByRole("group", { name: "D9 haritası" })).toBeInTheDocument();
   expect(container.querySelector("svg")!.dataset.style).toBe("south");
-  unmount();
-  const r = render(
-    <Providers>
+  // D9 lagnası Kova (Kov).
+  expect(screen.getByRole("button", { name: "1. ev, Kov" })).toBeInTheDocument();
+});
+
+test("an unknown style falls back to north; a varga missing from the chart says so", () => {
+  const opened = fixture().opened;
+  opened.chart.vargas = opened.chart.vargas.filter((v) => v.varga !== "D60");
+  const { container } = renderWithStores(
+    <>
       <ChartPanel panel={chartPanel({ style: "spiral" })} />
-    </Providers>,
+      <ChartPanel panel={chartPanel({ varga: "D60" })} />
+    </>,
+    { opened },
   );
-  await r.findByRole("group");
-  expect(r.container.querySelector("svg")!.dataset.style).toBe("north");
+  expect(container.querySelector("svg")!.dataset.style).toBe("north");
+  expect(screen.getByText("D60 bu haritada yok.")).toBeInTheDocument();
 });
 
-test("vargas other than D1 show an explicit later-milestone message", () => {
-  render(
-    <Providers>
-      <ChartPanel panel={chartPanel({ varga: "D9" })} />
-    </Providers>,
+test("without an open chart both panels say so", () => {
+  renderWithStores(
+    <>
+      <ChartPanel panel={chartPanel()} />
+      <GrahaTablePanel panel={tablePanel} />
+    </>,
+    { opened: null },
   );
-  expect(screen.getByText("D9 haritası sonraki bir sürümde gelecek.")).toBeInTheDocument();
+  expect(screen.getAllByText("Açık harita yok.")).toHaveLength(2);
   expect(screen.queryByRole("group")).toBeNull();
 });
 
-test("graha table lists body, sign, degree with minutes and the retro marker", async () => {
-  render(
-    <Providers>
-      <GrahaTablePanel panel={tablePanel} />
-    </Providers>,
-  );
-  const sun = (await screen.findByRole("button", { name: "Güneş" })).closest("tr")!;
+test("graha table lists body, sign, degree with minutes and the retro marker", () => {
+  renderWithStores(<GrahaTablePanel panel={tablePanel} />);
+  const sun = screen.getByRole("button", { name: "Güneş" }).closest("tr")!;
   expect(sun).toHaveTextContent("Yay");
   expect(sun).toHaveTextContent("16°30′");
   const saturn = screen.getByRole("button", { name: "Satürn" }).closest("tr")!;
   expect(saturn).toHaveTextContent("Koç");
-  expect(saturn).toHaveTextContent("10°15′");
   expect(saturn).toHaveTextContent("℞");
+  expect(screen.getAllByRole("row")).toHaveLength(10);
+});
+
+test("a null degree is shown as a dash", () => {
+  const opened = fixture().opened;
+  opened.chart.grahas[2].deg_in_sign = null;
+  renderWithStores(<GrahaTablePanel panel={tablePanel} />, { opened });
   const mars = screen.getByRole("button", { name: "Mars" }).closest("tr")!;
-  expect(mars.querySelectorAll("td")[0]).toHaveTextContent("—");
+  expect(mars.querySelectorAll("td")[1]).toHaveTextContent("—");
 });
 
-test("hovering a graha in the table highlights it in the chart, click pins, Esc clears", async () => {
-  const { container } = render(
-    <Providers>
+test("hovering a graha in the table highlights it in the chart, click pins, Esc clears", () => {
+  const { container } = renderWithStores(
+    <>
       <ChartPanel panel={chartPanel()} />
       <GrahaTablePanel panel={tablePanel} />
-    </Providers>,
+    </>,
   );
-  const row = (await screen.findByRole("button", { name: "Satürn" })).closest("tr")!;
+  const row = screen.getByRole("button", { name: "Satürn" }).closest("tr")!;
   const chartSaturn = () => container.querySelector('svg [data-graha="Saturn"]')!;
   fireEvent.mouseEnter(row);
   expect(chartSaturn()).toHaveClass("chart-planet--highlight");
@@ -143,42 +118,6 @@ test("hovering a graha in the table highlights it in the chart, click pins, Esc
   expect(row).not.toHaveClass("is-highlighted");
 });
 
-test("an empty result shows the empty state", async () => {
-  vi.mocked(api.planetLongitudes).mockResolvedValueOnce({ status: "ok", data: [] });
-  render(
-    <Providers>
-      <GrahaTablePanel panel={tablePanel} />
-    </Providers>,
-  );
-  expect(await screen.findByText("Gösterilecek graha yok.")).toBeInTheDocument();
-});
-
-test("errors are shown in the panel and retry fetches again", async () => {
-  vi.mocked(api.planetLongitudes).mockResolvedValueOnce({ status: "error", error: { code: "engine_stopped", params: {} } });
-  render(
-    <Providers>
-      <GrahaTablePanel panel={tablePanel} />
-    </Providers>,
-  );
-  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap motoru durdu.");
-  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
-  expect(await screen.findByRole("button", { name: "Güneş" })).toBeInTheDocument();
-  expect(api.planetLongitudes).toHaveBeenCalledTimes(2);
-});
-
-test("a slow load shows a skeleton after one second", async () => {
-  vi.useFakeTimers();
-  vi.mocked(api.planetLongitudes).mockReturnValueOnce(new Promise(() => {}));
-  render(
-    <Providers>
-      <ChartPanel panel={chartPanel()} />
-    </Providers>,
-  );
-  expect(screen.queryByLabelText("Yükleniyor")).toBeNull();
-  act(() => vi.advanceTimersByTime(1000));
-  expect(screen.getByLabelText("Yükleniyor")).toHaveAttribute("aria-busy", "true");
-});
-
 test("unavailable and unknown panels explain themselves without an error", () => {
   render(<UnavailablePanel panel={{ ...tablePanel, kind: "dasha_tree" }} />);
   expect(screen.getByText("Bu panel sonraki bir sürümde gelecek.")).toBeInTheDocument();
@@ -189,23 +128,19 @@ test("unavailable and unknown panels explain themselves without an error", () =>
 
 test("labels follow the selected language", async () => {
   await setLanguage("en");
-  render(
-    <Providers>
+  renderWithStores(
+    <>
       <ChartPanel panel={chartPanel()} />
       <GrahaTablePanel panel={tablePanel} />
-    </Providers>,
+    </>,
   );
-  expect(await screen.findByRole("button", { name: "Su, Sg 16°" })).toBeInTheDocument();
+  expect(screen.getByRole("button", { name: "Su, Sg 16°" })).toBeInTheDocument();
   expect(screen.getByRole("button", { name: "Sun" }).closest("tr")).toHaveTextContent("Sg");
 });
 
-test("chart aria-pressed follows the pin only: focus hovers, Enter pins, Enter again unpins", async () => {
-  render(
-    <Providers>
-      <ChartPanel panel={chartPanel()} />
-    </Providers>,
-  );
-  const planet = await screen.findByRole("button", { name: "Gü, Yay 16°" });
+test("chart aria-pressed follows the pin only: focus hovers, Enter pins, Enter again unpins", () => {
+  renderWithStores(<ChartPanel panel={chartPanel()} />);
+  const planet = screen.getByRole("button", { name: "Gü, Yay 16°" });
   act(() => planet.focus());
   expect(planet).toHaveAttribute("aria-pressed", "false");
   fireEvent.keyDown(planet, { key: "Enter" });
@@ -213,35 +148,3 @@ test("chart aria-pressed follows the pin only: focus hovers, Enter pins, Enter a
   fireEvent.keyDown(planet, { key: "Enter" });
   expect(planet).toHaveAttribute("aria-pressed", "false");
 });
-
-test("a non-D1 chart reports no loading phase to the panel frame", () => {
-  vi.useFakeTimers();
-  vi.mocked(api.planetLongitudes).mockReturnValueOnce(new Promise(() => {}));
-  const reported: string[] = [];
-  render(
-    <PanelStatusContext.Provider value={(p) => reported.push(p)}>
-      <Providers>
-        <ChartPanel panel={chartPanel({ varga: "D9" })} />
-      </Providers>
-    </PanelStatusContext.Provider>,
-  );
-  act(() => vi.advanceTimersByTime(2000));
-  expect(reported.every((p) => p === "none")).toBe(true);
-});
-
-test("unmounting SkyProvider before the load resolves neither sets state nor warns", async () => {
-  let resolve!: (v: Awaited<ReturnType<typeof api.planetLongitudes>>) => void;
-  vi.mocked(api.planetLongitudes).mockReturnValueOnce(new Promise((r) => (resolve = r)));
-  const spy = vi.spyOn(console, "error").mockImplementation(() => {});
-  const { unmount } = render(
-    <SkyProvider now={() => 0}>
-      <span />
-    </SkyProvider>,
-  );
-  unmount();
-  await act(async () => {
-    resolve({ status: "ok", data: [] });
-  });
-  expect(spy).not.toHaveBeenCalled();
-  spy.mockRestore();
-});
```

`app/ui/test/render.tsx` oluştur:

```tsx
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
```

`app/ui/time.test.ts` silinir:

```bash
git rm app/ui/time.test.ts
```


- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/App.test.tsx ui/chart/adapter.test.ts ui/panels ui/i18n ui/PrintSpike.test.tsx ui/api.test.ts
```
Expected: FAIL — `Failed to resolve import "./session/store"`, `chartDataFromChart is not a function`, eksik i18n anahtarları (`badge.TimeUnknownNoon`).

- [ ] **Step 3: Uygulamayı yaz**



`app/src-tauri/src/commands.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/src-tauri/src/commands.rs
+++ b/app/src-tauri/src/commands.rs
@@ -3,7 +3,7 @@
 use std::collections::BTreeMap;
 use std::sync::Arc;
 
-use ephemeris::{Body, Engine, EphError, Ephemeris, SiderealMode, SwissEph, Zodiac};
+use ephemeris::{Engine, EphError, SwissEph};
 use jyotish_core::{
     Bhavas, CalcSettings, Chart, CoreError, DashaPeriod, DashaSeed, Graha, TransitSnapshot,
 };
@@ -17,14 +17,6 @@ pub struct EngineStatus {
     pub se_version: String,
 }
 
-#[derive(Debug, Clone, Serialize, Type)]
-#[serde(rename_all = "camelCase")]
-pub struct PlanetRow {
-    pub body: String,
-    pub lon: f64,
-    pub retrograde: bool,
-}
-
 /// Arayüze giden hata: `code` sabit bir anahtar, çeviri i18n katmanında yapılır.
 #[derive(Debug, Clone, Serialize, Type)]
 pub struct CommandError {
@@ -97,44 +89,6 @@ impl From<CoreError> for CommandError {
     }
 }
 
-const GRAHAS: [(Body, &str); 8] = [
-    (Body::Sun, "Sun"),
-    (Body::Moon, "Moon"),
-    (Body::Mars, "Mars"),
-    (Body::Mercury, "Mercury"),
-    (Body::Jupiter, "Jupiter"),
-    (Body::Venus, "Venus"),
-    (Body::Saturn, "Saturn"),
-    (Body::MeanNode, "Rahu"),
-];
-
-/// Dokuz grahanın Lahiri sidereal boylamları. Ketu = Rahu + 180°.
-/// Spike amaçlıdır; arayüz `compute_chart`'a geçince kaldırılır.
-pub fn planet_rows(eph: &mut dyn Ephemeris, jd_ut: f64) -> Result<Vec<PlanetRow>, EphError> {
-    let zodiac = Zodiac::Sidereal(SiderealMode::Lahiri);
-    let mut rows = Vec::with_capacity(9);
-    for (body, name) in GRAHAS {
-        let p = eph.body(jd_ut, body, zodiac)?;
-        rows.push(PlanetRow {
-            body: name.to_owned(),
-            lon: p.lon,
-            retrograde: p.is_retrograde(),
-        });
-    }
-    let rahu = rows
-        .iter()
-        .zip(GRAHAS)
-        .find(|(_, (body, _))| *body == Body::MeanNode)
-        .map(|(row, _)| row.clone())
-        .ok_or_else(|| EphError::Calc("rahu missing".into()))?;
-    rows.push(PlanetRow {
-        body: "Ketu".to_owned(),
-        lon: (rahu.lon + 180.0).rem_euclid(360.0),
-        retrograde: rahu.retrograde,
-    });
-    Ok(rows)
-}
-
 #[tauri::command]
 #[specta::specta]
 pub async fn engine_status() -> Result<EngineStatus, CommandError> {
@@ -143,24 +97,6 @@ pub async fn engine_status() -> Result<EngineStatus, CommandError> {
     })
 }
 
-/// Motor çağrısı milisaniyeler sürer ve `engine.call` kanal üzerinde bloklar. Bu yüzden
-/// çağrı `spawn_blocking` havuzunda yapılır; async çalışma zamanı iş parçacıkları serbest kalır.
-/// `State<'_, _>` `'static` olmadığından motor `Arc` olarak yönetilir ve kapanışa klonlanır.
-#[tauri::command]
-#[specta::specta]
-pub async fn planet_longitudes(
-    engine: State<'_, Arc<Engine>>,
-    jd_ut: f64,
-) -> Result<Vec<PlanetRow>, CommandError> {
-    let engine = Arc::clone(&engine);
-    let rows = tauri::async_runtime::spawn_blocking(move || {
-        engine.call(move |se: &mut SwissEph| planet_rows(se, jd_ut))
-    })
-    .await
-    .map_err(|e| CommandError::new("engine_stopped", &[("detail", e.to_string())]))???;
-    Ok(rows)
-}
-
 /// `compute_chart` komutunun gövdesi: hesap motor iş parçacığında yapılır, burada iş
 /// mantığı yoktur. Bloklar; async bağlamdan `spawn_blocking` içinde çağrılmalıdır.
 pub fn compute_chart_blocking(
@@ -247,89 +183,6 @@ pub fn print_window(window: tauri::WebviewWindow) -> Result<(), CommandError> {
 #[cfg(test)]
 mod tests {
     use super::*;
-    use ephemeris::{Ayanamsa, BodyPosition, HouseSystem, Houses, Precision, RiseSet, SunDisc};
-
-    /// Her cisme sabit boylam veren sahte motor; Rahu 100°.
-    struct Fake;
-    impl Ephemeris for Fake {
-        fn body(&mut self, _jd: f64, body: Body, _z: Zodiac) -> Result<BodyPosition, EphError> {
-            let lon = match body {
-                Body::MeanNode => 100.0,
-                Body::Saturn => 300.0,
-                _ => 10.0,
-            };
-            Ok(BodyPosition {
-                lon,
-                lat: 0.0,
-                distance_au: 1.0,
-                // Mean Node gerçekte hep geri gider (≈ -0.053°/gün).
-                lon_speed: match body {
-                    Body::Saturn => -0.05,
-                    Body::MeanNode => -0.053,
-                    _ => 1.0,
-                },
-                lat_speed: 0.0,
-                dist_speed: 0.0,
-                precision: Precision::SwissEph,
-            })
-        }
-        fn ayanamsa(&mut self, _jd: f64, _m: SiderealMode) -> Result<Ayanamsa, EphError> {
-            Ok(Ayanamsa {
-                deg: 24.0,
-                precision: Precision::SwissEph,
-            })
-        }
-        fn houses(
-            &mut self,
-            _jd: f64,
-            _lat: f64,
-            _lon: f64,
-            _s: HouseSystem,
-            _z: Zodiac,
-        ) -> Result<Houses, EphError> {
-            Err(EphError::Calc("fake: houses unused".into()))
-        }
-        fn sun_rise_set(
-            &mut self,
-            _after: f64,
-            _event: RiseSet,
-            _lat: f64,
-            _lon: f64,
-            _disc: SunDisc,
-        ) -> Result<Option<f64>, EphError> {
-            Ok(None)
-        }
-        fn swe_version(&self) -> String {
-            "fake".into()
-        }
-    }
-
-    #[test]
-    fn rows_are_nine_grahas_in_order() {
-        let rows = planet_rows(&mut Fake, 2451545.0).unwrap();
-        let names: Vec<_> = rows.iter().map(|r| r.body.as_str()).collect();
-        assert_eq!(
-            names,
-            ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"]
-        );
-    }
-
-    #[test]
-    fn ketu_is_opposite_rahu_and_both_retrograde() {
-        let rows = planet_rows(&mut Fake, 2451545.0).unwrap();
-        let rahu = rows.iter().find(|r| r.body == "Rahu").unwrap();
-        let ketu = rows.iter().find(|r| r.body == "Ketu").unwrap();
-        assert_eq!(rahu.lon, 100.0);
-        assert_eq!(ketu.lon, 280.0);
-        assert!(rahu.retrograde && ketu.retrograde);
-    }
-
-    #[test]
-    fn retrograde_follows_speed_sign() {
-        let rows = planet_rows(&mut Fake, 2451545.0).unwrap();
-        assert!(rows.iter().find(|r| r.body == "Saturn").unwrap().retrograde);
-        assert!(!rows.iter().find(|r| r.body == "Sun").unwrap().retrograde);
-    }
 
     #[test]
     fn error_codes_are_stable() {
@@ -349,19 +202,6 @@ mod tests {
         );
     }
 
-    #[test]
-    fn real_engine_produces_sidereal_rows() {
-        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/ephe");
-        let engine = ephemeris::Engine::spawn(dir).expect("spawn");
-        let rows = engine
-            .call(|se| planet_rows(se, 2451545.0))
-            .expect("alive")
-            .expect("calc");
-        let sun = rows.iter().find(|r| r.body == "Sun").unwrap();
-        // J2000 tropikal Güneş ≈ 280.37°, Lahiri ≈ 23.86° → sidereal ≈ 256.5°.
-        assert!((256.0..257.0).contains(&sun.lon), "sun {}", sun.lon);
-    }
-
     #[test]
     fn core_error_codes_are_stable() {
         use jyotish_core::CoreError;
```

`app/src-tauri/src/lib.rs` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -20,7 +20,6 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
         .disable_serde_phases()
         .commands(tauri_specta::collect_commands![
             commands::engine_status,
-            commands::planet_longitudes,
             commands::compute_chart,
             commands::compute_transits,
             commands::dasha_children,
```

`app/ui/App.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/App.tsx
+++ b/app/ui/App.tsx
@@ -1,25 +1,50 @@
-import { useState } from "react";
+import { useState, type ReactNode } from "react";
+import { EMPTY_BOOT, type Boot } from "./boot";
 import { HighlightProvider } from "./highlight/store";
-import { SkyProvider } from "./panels/sky";
+import { SessionProvider } from "./session/store";
+import { NoticeProvider, storeWarningNotices, useNotices, type Notice } from "./shell/notices";
 import { Shell } from "./shell/Shell";
 import { initialViewModel } from "./view/model";
 import { storePersistence, type ViewPersistence } from "./view/persistence";
 import { ViewStateProvider } from "./view/store";
 
+function bootNotices(boot: Boot): Notice[] {
+  if (boot.error) return [{ id: "boot", type: "error", messageKey: "notice.restoreFailed", error: boot.error }];
+  return boot.opened ? storeWarningNotices(boot.opened.warnings) : [];
+}
+
+/** Görünüm deposu ve kayıt: kayıt hatası bildirim bandına gider (aynı kimlik, tek bildirim). */
+function Stores({ boot, persistence, children }: { boot: Boot; persistence?: ViewPersistence; children: ReactNode }) {
+  const { push } = useNotices();
+  const [store] = useState(
+    () => persistence ?? storePersistence((error) => push({ id: "save", type: "error", messageKey: "notice.saveFailed", error })),
+  );
+  const [initial] = useState(() =>
+    initialViewModel(
+      boot.layout,
+      boot.opened && { id: boot.opened.chart_record.id, dashaPath: boot.opened.view_state.dasha_expanded_path },
+    ),
+  );
+  return (
+    <ViewStateProvider initial={initial} persistence={store}>
+      <SessionProvider initial={boot.opened}>{children}</SessionProvider>
+    </ViewStateProvider>
+  );
+}
+
 /**
- * Uygulama kökü. `initialLayout` açılışta store'dan okunan ham düzendir (main.tsx);
+ * Uygulama kökü. `boot` açılışta store'dan okunan düzen ve oturumdur (main.tsx);
  * `persistence` testler için verilebilir, varsayılan store komutlarıdır.
  */
-export function App({ initialLayout = null, persistence }: { initialLayout?: unknown; persistence?: ViewPersistence }) {
-  const [store] = useState(() => persistence ?? storePersistence());
-  const [initial] = useState(() => initialViewModel(initialLayout));
+export function App({ boot = EMPTY_BOOT, persistence }: { boot?: Boot; persistence?: ViewPersistence }) {
+  const [notices] = useState(() => bootNotices(boot));
   return (
-    <ViewStateProvider initial={initial} persistence={store}>
-      <HighlightProvider>
-        <SkyProvider>
+    <NoticeProvider initial={notices}>
+      <Stores boot={boot} persistence={persistence}>
+        <HighlightProvider>
           <Shell />
-        </SkyProvider>
-      </HighlightProvider>
-    </ViewStateProvider>
+        </HighlightProvider>
+      </Stores>
+    </NoticeProvider>
   );
 }
```

`app/ui/PrintSpike.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/PrintSpike.tsx
+++ b/app/ui/PrintSpike.tsx
@@ -1,6 +1,14 @@
-import type { PlanetRow } from "./bindings";
 import "./print.css";
 
+/** Baskı denemesinin satırı: burç Rust'tan gelir (`Chart.grahas[].sign`), burada türetilmez. */
+export interface SpikeRow {
+  body: string;
+  /** 0 = Mesha … 11 = Meena; `null` bilinmiyor. */
+  sign: number | null;
+  lon: number | null;
+  retrograde: boolean;
+}
+
 // Güney Hint düzeni: 4×4 ızgaranın dış halkası, Meena sol üstten saat yönünde.
 // [satır, sütun] → burç index (0 = Mesha).
 const SOUTH_CELLS: Array<[number, number, number]> = [
@@ -9,12 +17,11 @@ const SOUTH_CELLS: Array<[number, number, number]> = [
   [3, 1, 7], [3, 0, 8], [2, 0, 9], [1, 0, 10],
 ];
 
-export function PrintSpike({ rows }: { rows: PlanetRow[] }) {
+export function PrintSpike({ rows }: { rows: SpikeRow[] }) {
   const bySign = new Map<number, string[]>();
   for (const r of rows) {
-    if (r.lon === null) continue;
-    const s = Math.floor(r.lon / 30) % 12;
-    bySign.set(s, [...(bySign.get(s) ?? []), r.body]);
+    if (r.sign === null) continue;
+    bySign.set(r.sign, [...(bySign.get(r.sign) ?? []), r.body]);
   }
   const cell = 45;
   return (
```

`app/ui/api.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -40,7 +40,6 @@ async function wrap<T>(call: () => Promise<Result<T>>): Promise<Result<T>> {
 
 export const api = {
   engineStatus: () => wrap(() => commands.engineStatus()),
-  planetLongitudes: (jdUt: number | null) => wrap(() => commands.planetLongitudes(jdUt)),
   printWindow: () => wrap(() => commands.printWindow()),
   restoreSession: () => wrap(() => commands.restoreSession()),
   createClient: (input: NewClient) => wrap(() => commands.createClient(input)),
```

`app/ui/boot.ts` oluştur:

```ts
// Açılış: kayıtlı düzen ve son açılan harita tek seferde okunur (spec §4: uygulama kapanıp
// açılınca oturum geri yüklenir). Hiçbir hata açılışı engellemez; geri yükleme hatası
// bildirim bandında gösterilir.
import { api } from "./api";
import type { CommandError, OpenedClient } from "./bindings";
import { loadLayout } from "./view/persistence";

export interface Boot {
  layout: unknown | null;
  opened: OpenedClient | null;
  error: CommandError | null;
}

export async function boot(): Promise<Boot> {
  const [layout, session] = await Promise.all([loadLayout(), api.restoreSession()]);
  if (session.status === "error") return { layout, opened: null, error: session.error };
  return { layout, opened: session.data, error: null };
}

export const EMPTY_BOOT: Boot = { layout: null, opened: null, error: null };
```

`app/ui/chart/adapter.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/chart/adapter.ts
+++ b/app/ui/chart/adapter.ts
@@ -1,23 +1,55 @@
-// PlanetRow → ChartData. Yalnızca gösterim eşlemesi yapılır (burç = floor(lon / 30),
-// burç içi derece); astrolojik hesap değildir.
-import type { PlanetRow } from "../bindings";
-import { isGrahaId, type ChartData } from "./types";
+// Chart → ChartData. Burç, derece, retro, yanma ve vargottama Rust'tan hazır gelir
+// (`Chart.vargas`, `Chart.grahas`); arayüz hiçbir boylamdan burç türetmez (spec §4).
+import type { Chart, Sign, Varga } from "../bindings";
+import type { ChartData, ChartPlanet } from "./types";
 
-export function signAndDegree(lon: number): { sign: number; degInSign: number } {
-  const norm = ((lon % 360) + 360) % 360;
-  const sign = Math.floor(norm / 30);
-  return { sign, degInSign: norm - sign * 30 };
+/** 0 = Mesha (Koç) … 11 = Meena (Balık); bindings `Sign` sırası. */
+export const SIGNS = [
+  "Aries",
+  "Taurus",
+  "Gemini",
+  "Cancer",
+  "Leo",
+  "Virgo",
+  "Libra",
+  "Scorpio",
+  "Sagittarius",
+  "Capricorn",
+  "Aquarius",
+  "Pisces",
+] as const satisfies readonly Sign[];
+
+/** `Varga::ALL` sırası (Shodasavarga). */
+export const VARGAS = ["D1", "D2", "D3", "D4", "D7", "D9", "D10", "D12", "D16", "D20", "D24", "D27", "D30", "D40", "D45", "D60"] as const satisfies readonly Varga[];
+
+export function isVarga(v: unknown): v is Varga {
+  return typeof v === "string" && (VARGAS as readonly string[]).includes(v);
+}
+
+export function signIndex(sign: Sign): number {
+  return (SIGNS as readonly Sign[]).indexOf(sign);
 }
 
-/** Lagna henüz backend'den gelmediği için `ascSign` her zaman `null`. */
-export function chartDataFromPlanetRows(rows: readonly PlanetRow[]): ChartData {
-  const planets: ChartData["planets"] = [];
-  for (const r of rows) {
-    // Sonlu olmayan boylam (NaN, ±∞) gösterilemez; açıkça atlanır.
-    if (r.lon === null || !Number.isFinite(r.lon) || !isGrahaId(r.body)) continue;
-    const pos = signAndDegree(r.lon);
-    if (!Number.isInteger(pos.sign) || pos.sign < 0 || pos.sign > 11) continue;
-    planets.push({ id: r.body, ...pos, retrograde: r.retrograde });
-  }
-  return { ascSign: null, planets };
+/**
+ * Bir vargayı çizim girdisine çevirir; varga haritada yoksa `null`. Retro her vargada,
+ * yanma yalnızca D1'de, vargottama (D1 ve D9 burcu aynı) D1 ve D9'da gösterilir.
+ * `showLagna: false` (saat bilinmiyor) lagnayı gizler: 1. ev Koç çizilir (spec §7).
+ */
+export function chartDataFromChart(chart: Chart, varga: Varga, { showLagna }: { showLagna: boolean }): ChartData | null {
+  const v = chart.vargas.find((x) => x.varga === varga);
+  if (!v) return null;
+  const grahas = new Map(chart.grahas.map((g) => [g.graha, g]));
+  const planets: ChartPlanet[] = v.placements.map((p) => {
+    const g = grahas.get(p.graha);
+    return {
+      id: p.graha,
+      sign: signIndex(p.sign),
+      // `null`: Rust'ta sonlu olmayan değer (NaN); çizim bu gezegeni açıkça atlar.
+      degInSign: p.deg ?? Number.NaN,
+      retrograde: g?.retrograde ?? false,
+      combust: varga === "D1" && (g?.combust ?? false),
+      vargottama: (varga === "D1" || varga === "D9") && (g?.vargottama ?? false),
+    };
+  });
+  return { ascSign: showLagna ? signIndex(v.lagna_sign) : null, planets };
 }
```

`app/ui/chart/types.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/chart/types.ts
+++ b/app/ui/chart/types.ts
@@ -1,7 +1,8 @@
 // Harita çiziminin girdisi. Değerler Rust'tan hazır gelir; arayüz yalnızca çizer.
+import type { Graha } from "../bindings";
 
-/** Backend'in `PlanetRow.body` değerleriyle birebir aynı (app/src-tauri/src/commands.rs). */
-export const GRAHA_IDS = ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"] as const;
+/** `Graha::ALL` sırası (bindings `Graha`). */
+export const GRAHA_IDS = ["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"] as const satisfies readonly Graha[];
 export type GrahaId = (typeof GRAHA_IDS)[number];
 
 export function isGrahaId(v: string): v is GrahaId {
```

`app/ui/i18n/en.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -32,11 +32,14 @@ export const en: Messages = {
     title: "{{varga}} chart",
     lagna: "As",
     house: "House {{house}}, {{sign}}",
-    noLagna: "Lagna is not computed yet; showing the current sky with house 1 = Aries",
-    vargaUnavailable: "The {{varga}} chart arrives in a later version.",
     retro: "retrograde",
     combust: "combust",
     vargottama: "vargottama",
+    vargaMissing: "{{varga}} is not in this chart.",
+    kind: {
+      Natal: "Natal chart",
+      Event: "Event chart",
+    },
   },
   panel: {
     chart: "Chart",
@@ -53,6 +56,7 @@ export const en: Messages = {
     retry: "Try again",
     moveHint: "Alt+arrow moves, Alt+Shift+arrow resizes",
     announce: "Column {{col}}, row {{row}}, {{w}}×{{h}}",
+    noChart: "No chart is open.",
   },
   table: { graha: "Graha", sign: "Sign", degree: "Degree", retro: "Retro", empty: "No grahas to show." },
   errors: {
@@ -66,6 +70,32 @@ export const en: Messages = {
     job_panicked: "The calculation stopped unexpectedly.",
     print_failed: "The print dialog could not be opened.",
     unknown: "Something unexpected happened ({{code}}).",
+    store_sqlite: "The client database could not be read or written ({{detail}}).",
+    store_migration: "The database could not be upgraded ({{detail}}).",
+    store_json: "Stored data could not be read ({{detail}}).",
+    store_too_new: "The database is newer than this app (version {{found}}, supported {{supported}}). Update the app.",
+    store_unavailable: "The client database is unavailable.",
+    not_found: "Record not found ({{what}} {{id}}).",
+    invalid_input: "Invalid value: {{field}}.",
+    unknown_time_zone: "Unknown time zone: {{tz}}.",
+    nonexistent_local_time: "{{local}} does not exist locally (daylight-saving change). Suggested: {{suggestion}}.",
+    ambiguous_local_time: "{{local}} occurred twice (daylight-saving change); choose which one.",
+    offset_out_of_range: "Offset out of range ({{secs}} s).",
+    longitude_out_of_range: "Longitude out of range ({{lon}}).",
+    date_out_of_range: "Date outside the supported range.",
+    last_chart: "A client's last chart cannot be deleted.",
+    unsupported_schema_version: "Settings version not supported (version {{found}}, supported {{supported}}).",
+    invalid_settings: "Invalid calculation setting: {{field}}.",
+    invalid_location: "Invalid location ({{lat}}, {{lon}}).",
+    invalid_julian_day: "The moment could not be computed (JD {{jd}}).",
+    non_finite_result: "The calculation returned a non-finite value ({{what}}).",
+    invalid_dasha_input: "Invalid dasha request: {{field}}.",
+    panchang_search: "Panchang boundary not found ({{what}}).",
+    invalid_transit_input: "Invalid transit request: {{field}}.",
+    atlas_unavailable: "The place atlas is unavailable; enter coordinates manually.",
+    atlas_sqlite: "The place atlas could not be read ({{detail}}).",
+    atlas_io: "The place atlas file could not be read ({{detail}}).",
+    atlas_schema: "Place atlas version mismatch ({{found}}, expected {{expected}}).",
   },
   layout: {
     picker: "Layout",
@@ -91,5 +121,39 @@ export const en: Messages = {
   },
   theme: { label: "Theme", light: "Light", dark: "Dark", system: "System" },
   language: { label: "Language", tr: "Türkçe", en: "English", fr: "Français" },
-  shell: { printSpike: "Print test", backToDesk: "Back to desk", print: "Print…" },
+  shell: {
+    printSpike: "Print test",
+    backToDesk: "Back to desk",
+    print: "Print…",
+    warnings: "Warnings",
+  },
+  badge: {
+    TimeUnknownNoon: "Time unknown",
+    LmtFromLongitude: "LMT",
+    PreTzdbReliability: "Early time zone",
+    LowPrecision: "Low precision",
+    PlacidusPolarFallback: "No Placidus",
+    SunriseUnavailable: "No sunrise",
+  },
+  badgeHint: {
+    TimeUnknownNoon: "Birth time unknown; noon was used.",
+    LmtFromLongitude: "Local mean time was derived from longitude.",
+    PreTzdbReliability: "Time zone data before 1970 may be unreliable.",
+    LowPrecision: "Sun or Moon used the fallback ephemeris.",
+    PlacidusPolarFallback: "Placidus failed at this latitude; Porphyry was used.",
+    SunriseUnavailable: "No sunrise or sunset at the birth place.",
+  },
+  notice: {
+    region: "Notifications",
+    dismiss: "Dismiss",
+    error: "Error",
+    warning: "Warning",
+    restoreFailed: "The last session could not be opened: {{message}}",
+    saveFailed: "The view could not be saved: {{message}}",
+  },
+  storeWarning: {
+    LastChartMissing: "The last opened chart was deleted; the most recently updated chart was opened.",
+    CalcSettingsReset: "This chart's calculation settings could not be read; defaults are used.",
+    TzdbChanged: "Time zone data changed ({{stored}} → {{current}}); the stored moment is used.",
+  },
 };
```

`app/ui/i18n/errors.ts` oluştur:

```ts
// `{code, params}` → yerelleştirilmiş metin (spec §7). Arka uçtan gelen parametreler
// i18next seçeneklerini (dil, çoğul, bağlam…) değiştiremez. Parametreler loglanmaz.
import type { i18n as I18n, TFunction } from "i18next";
import type { CommandError } from "../bindings";

const RESERVED_OPTIONS = new Set(["count", "context", "lng", "lngs", "defaultValue", "ns", "replace", "returnObjects", "joinArrays", "postProcess", "interpolation", "fallbackLng"]);

function safeParams(params: Record<string, unknown>): Record<string, unknown> {
  return Object.fromEntries(Object.entries(params).filter(([k]) => !RESERVED_OPTIONS.has(k)));
}

export function errorMessage(t: TFunction, i18n: I18n, error: CommandError): string {
  const key = `errors.${error.code}`;
  return i18n.exists(key) ? t(key, safeParams(error.params)) : t("errors.unknown", { code: error.code });
}
```

`app/ui/i18n/fr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -32,11 +32,14 @@ export const fr: Messages = {
     title: "Carte {{varga}}",
     lagna: "As",
     house: "Maison {{house}}, {{sign}}",
-    noLagna: "Le lagna n’est pas encore calculé ; ciel actuel, maison 1 = Bélier",
-    vargaUnavailable: "La carte {{varga}} arrivera dans une version ultérieure.",
     retro: "rétrograde",
     combust: "combuste",
     vargottama: "vargottama",
+    vargaMissing: "{{varga}} absent de cette carte.",
+    kind: {
+      Natal: "Carte natale",
+      Event: "Carte d’événement",
+    },
   },
   panel: {
     chart: "Carte",
@@ -53,6 +56,7 @@ export const fr: Messages = {
     retry: "Réessayer",
     moveHint: "Alt+flèche déplace, Alt+Maj+flèche redimensionne",
     announce: "Colonne {{col}}, ligne {{row}}, {{w}}×{{h}}",
+    noChart: "Aucune carte ouverte.",
   },
   table: { graha: "Graha", sign: "Signe", degree: "Degré", retro: "Rétro", empty: "Aucun graha à afficher." },
   errors: {
@@ -66,6 +70,32 @@ export const fr: Messages = {
     job_panicked: "Le calcul s’est arrêté de façon inattendue.",
     print_failed: "La fenêtre d’impression n’a pas pu s’ouvrir.",
     unknown: "Un problème inattendu est survenu ({{code}}).",
+    store_sqlite: "La base des consultants n’a pas pu être lue ou écrite ({{detail}}).",
+    store_migration: "La base n’a pas pu être mise à jour ({{detail}}).",
+    store_json: "Les données enregistrées sont illisibles ({{detail}}).",
+    store_too_new: "La base est plus récente que l’application (version {{found}}, prise en charge {{supported}}). Mettez l’application à jour.",
+    store_unavailable: "La base des consultants est indisponible.",
+    not_found: "Enregistrement introuvable ({{what}} {{id}}).",
+    invalid_input: "Valeur invalide : {{field}}.",
+    unknown_time_zone: "Fuseau horaire inconnu : {{tz}}.",
+    nonexistent_local_time: "{{local}} n’existe pas en heure locale (changement d’heure). Suggestion : {{suggestion}}.",
+    ambiguous_local_time: "{{local}} a eu lieu deux fois (changement d’heure) ; choisissez laquelle.",
+    offset_out_of_range: "Décalage hors limites ({{secs}} s).",
+    longitude_out_of_range: "Longitude hors limites ({{lon}}).",
+    date_out_of_range: "Date hors de la plage prise en charge.",
+    last_chart: "La dernière carte d’un consultant ne peut pas être supprimée.",
+    unsupported_schema_version: "Version des réglages non prise en charge (version {{found}}, prise en charge {{supported}}).",
+    invalid_settings: "Réglage de calcul invalide : {{field}}.",
+    invalid_location: "Lieu invalide ({{lat}}, {{lon}}).",
+    invalid_julian_day: "L’instant n’a pas pu être calculé (JD {{jd}}).",
+    non_finite_result: "Le calcul a donné une valeur non finie ({{what}}).",
+    invalid_dasha_input: "Requête de dasha invalide : {{field}}.",
+    panchang_search: "Limite du panchang introuvable ({{what}}).",
+    invalid_transit_input: "Requête de transit invalide : {{field}}.",
+    atlas_unavailable: "L’atlas des lieux est indisponible ; saisissez les coordonnées.",
+    atlas_sqlite: "L’atlas des lieux est illisible ({{detail}}).",
+    atlas_io: "Le fichier de l’atlas est illisible ({{detail}}).",
+    atlas_schema: "Version de l’atlas incompatible ({{found}}, attendue {{expected}}).",
   },
   layout: {
     picker: "Disposition",
@@ -91,5 +121,39 @@ export const fr: Messages = {
   },
   theme: { label: "Thème", light: "Clair", dark: "Sombre", system: "Système" },
   language: { label: "Langue", tr: "Türkçe", en: "English", fr: "Français" },
-  shell: { printSpike: "Essai d’impression", backToDesk: "Retour au tableau", print: "Imprimer…" },
+  shell: {
+    printSpike: "Essai d’impression",
+    backToDesk: "Retour au tableau",
+    print: "Imprimer…",
+    warnings: "Avertissements",
+  },
+  badge: {
+    TimeUnknownNoon: "Heure inconnue",
+    LmtFromLongitude: "TML",
+    PreTzdbReliability: "Fuseau ancien",
+    LowPrecision: "Précision réduite",
+    PlacidusPolarFallback: "Pas de Placidus",
+    SunriseUnavailable: "Pas de lever",
+  },
+  badgeHint: {
+    TimeUnknownNoon: "Heure de naissance inconnue ; midi a été utilisé.",
+    LmtFromLongitude: "Temps moyen local calculé depuis la longitude.",
+    PreTzdbReliability: "Les données de fuseau avant 1970 peuvent être peu fiables.",
+    LowPrecision: "Soleil ou Lune calculés avec l’éphéméride de secours.",
+    PlacidusPolarFallback: "Placidus impossible à cette latitude ; Porphyry utilisé.",
+    SunriseUnavailable: "Pas de lever ni de coucher du Soleil au lieu de naissance.",
+  },
+  notice: {
+    region: "Notifications",
+    dismiss: "Fermer",
+    error: "Erreur",
+    warning: "Avertissement",
+    restoreFailed: "La dernière session n’a pas pu être ouverte : {{message}}",
+    saveFailed: "La vue n’a pas pu être enregistrée : {{message}}",
+  },
+  storeWarning: {
+    LastChartMissing: "La dernière carte ouverte a été supprimée ; la plus récemment modifiée a été ouverte.",
+    CalcSettingsReset: "Les réglages de calcul de cette carte sont illisibles ; les réglages par défaut sont utilisés.",
+    TzdbChanged: "Les données de fuseau ont changé ({{stored}} → {{current}}) ; l’instant enregistré est utilisé.",
+  },
 };
```

`app/ui/i18n/tr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -31,11 +31,14 @@ export const tr = {
     title: "{{varga}} haritası",
     lagna: "Lg",
     house: "{{house}}. ev, {{sign}}",
-    noLagna: "Lagna henüz hesaplanmıyor; şimdiki gökyüzü, 1. ev Koç",
-    vargaUnavailable: "{{varga}} haritası sonraki bir sürümde gelecek.",
     retro: "retro",
     combust: "yanık",
     vargottama: "vargottama",
+    vargaMissing: "{{varga}} bu haritada yok.",
+    kind: {
+      Natal: "Doğum haritası",
+      Event: "Olay haritası",
+    },
   },
   panel: {
     chart: "Harita",
@@ -52,6 +55,7 @@ export const tr = {
     retry: "Yeniden dene",
     moveHint: "Taşımak için Alt+ok, boyutlandırmak için Alt+Shift+ok",
     announce: "Sütun {{col}}, satır {{row}}, {{w}}×{{h}}",
+    noChart: "Açık harita yok.",
   },
   table: { graha: "Graha", sign: "Burç", degree: "Derece", retro: "Retro", empty: "Gösterilecek graha yok." },
   errors: {
@@ -65,6 +69,32 @@ export const tr = {
     job_panicked: "Hesap beklenmedik biçimde durdu.",
     print_failed: "Yazdırma penceresi açılamadı.",
     unknown: "Beklenmeyen bir sorun oluştu ({{code}}).",
+    store_sqlite: "Danışan veritabanı okunamadı ya da yazılamadı ({{detail}}).",
+    store_migration: "Veritabanı güncellenemedi ({{detail}}).",
+    store_json: "Saklanan veri okunamadı ({{detail}}).",
+    store_too_new: "Veritabanı bu sürümden yeni (sürüm {{found}}, desteklenen {{supported}}). Uygulamayı güncelleyin.",
+    store_unavailable: "Danışan veritabanı kullanılamıyor.",
+    not_found: "Kayıt bulunamadı ({{what}} {{id}}).",
+    invalid_input: "Geçersiz değer: {{field}}.",
+    unknown_time_zone: "Saat dilimi tanınmıyor: {{tz}}.",
+    nonexistent_local_time: "{{local}} yerel saatte yok (yaz saati geçişi). Önerilen: {{suggestion}}.",
+    ambiguous_local_time: "{{local}} iki kez yaşandı (yaz saati dönüşü); hangisi olduğunu seçin.",
+    offset_out_of_range: "Saat farkı aralık dışında ({{secs}} sn).",
+    longitude_out_of_range: "Boylam aralık dışında ({{lon}}).",
+    date_out_of_range: "Tarih desteklenen aralığın dışında.",
+    last_chart: "Danışanın son haritası silinemez.",
+    unsupported_schema_version: "Ayar sürümü desteklenmiyor (sürüm {{found}}, desteklenen {{supported}}).",
+    invalid_settings: "Hesap ayarı geçersiz: {{field}}.",
+    invalid_location: "Konum geçersiz ({{lat}}, {{lon}}).",
+    invalid_julian_day: "An hesaplanamadı (JD {{jd}}).",
+    non_finite_result: "Hesap sonlu olmayan bir değer verdi ({{what}}).",
+    invalid_dasha_input: "Dasa isteği geçersiz: {{field}}.",
+    panchang_search: "Panchang sınırı bulunamadı ({{what}}).",
+    invalid_transit_input: "Transit isteği geçersiz: {{field}}.",
+    atlas_unavailable: "Yer atlası açılamadı; koordinatları elle girin.",
+    atlas_sqlite: "Yer atlası okunamadı ({{detail}}).",
+    atlas_io: "Yer atlası dosyası okunamadı ({{detail}}).",
+    atlas_schema: "Yer atlası sürümü uyuşmuyor ({{found}}, beklenen {{expected}}).",
   },
   layout: {
     picker: "Düzen",
@@ -90,7 +120,41 @@ export const tr = {
   },
   theme: { label: "Tema", light: "Açık", dark: "Koyu", system: "Sistem" },
   language: { label: "Dil", tr: "Türkçe", en: "English", fr: "Français" },
-  shell: { printSpike: "Baskı denemesi", backToDesk: "Panoya dön", print: "Yazdır…" },
+  shell: {
+    printSpike: "Baskı denemesi",
+    backToDesk: "Panoya dön",
+    print: "Yazdır…",
+    warnings: "Uyarılar",
+  },
+  badge: {
+    TimeUnknownNoon: "Saat bilinmiyor",
+    LmtFromLongitude: "LMT",
+    PreTzdbReliability: "Eski saat dilimi",
+    LowPrecision: "Düşük hassasiyet",
+    PlacidusPolarFallback: "Placidus yok",
+    SunriseUnavailable: "Gün doğumu yok",
+  },
+  badgeHint: {
+    TimeUnknownNoon: "Doğum saati bilinmiyor; öğlen 12:00 alındı.",
+    LmtFromLongitude: "Yerel ortalama saat boylamdan hesaplandı.",
+    PreTzdbReliability: "1970 öncesi saat dilimi verisi güvenilir olmayabilir.",
+    LowPrecision: "Güneş veya Ay yedek ephemeris ile hesaplandı.",
+    PlacidusPolarFallback: "Placidus bu enlemde hesaplanamadı; Porphyry kullanıldı.",
+    SunriseUnavailable: "Doğum yerinde gün doğumu ya da batımı yok.",
+  },
+  notice: {
+    region: "Bildirimler",
+    dismiss: "Kapat",
+    error: "Hata",
+    warning: "Uyarı",
+    restoreFailed: "Son oturum açılamadı: {{message}}",
+    saveFailed: "Görünüm kaydedilemedi: {{message}}",
+  },
+  storeWarning: {
+    LastChartMissing: "Son açılan harita silinmiş; en son güncellenen harita açıldı.",
+    CalcSettingsReset: "Bu haritanın hesap ayarı okunamadı; varsayılan ayar kullanılıyor.",
+    TzdbChanged: "Saat dilimi verisi değişti ({{stored}} → {{current}}); saklanan an kullanılıyor.",
+  },
 };
 
 type Strings<T> = { [K in keyof T]: T[K] extends string ? string : Strings<T[K]> };
```

`app/ui/main.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/main.tsx
+++ b/app/ui/main.tsx
@@ -1,10 +1,10 @@
 import { StrictMode } from "react";
 import { createRoot } from "react-dom/client";
 import { App } from "./App";
+import { boot } from "./boot";
 import "./i18n";
 import "./theme/tokens.css";
 import { applyTheme, loadThemePref } from "./theme/theme";
-import { loadLayout } from "./view/persistence";
 
 // İlk çizimden önce yalnızca `data-theme` yazılır (yanlış tema parlaması olmaz). "system"
 // dinleyicisi hemen kaldırılır: sistem değişikliklerini Shell'in kendi applyTheme'i izler,
@@ -12,12 +12,12 @@ import { loadLayout } from "./view/persistence";
 const disposeBootTheme = applyTheme(loadThemePref());
 disposeBootTheme();
 
-// Düzen store'dan okunur (≈ milisaniyeler); ilk çizim kayıtlı düzenle yapılır, yanlış düzen
-// parlaması olmaz. Okuma hatası açılışı engellemez: hazır düzenler kullanılır.
-void loadLayout().then((initialLayout) => {
+// Düzen ve son oturum store'dan okunur (Rust tarafında ≤ 50 ms); ilk çizim kayıtlı düzen
+// ve açık haritayla yapılır, boş pano parlaması olmaz. Hatalar açılışı engellemez.
+void boot().then((initial) => {
   createRoot(document.getElementById("root")!).render(
     <StrictMode>
-      <App initialLayout={initialLayout} />
+      <App boot={initial} />
     </StrictMode>,
   );
 });
```

`app/ui/panels/ChartPanel.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/panels/ChartPanel.tsx
+++ b/app/ui/panels/ChartPanel.tsx
@@ -1,42 +1,31 @@
 import { useTranslation } from "react-i18next";
-import { chartDataFromPlanetRows } from "../chart/adapter";
+import type { Varga } from "../bindings";
+import { chartDataFromChart, isVarga } from "../chart/adapter";
 import { ChartSvg } from "../chart/ChartSvg";
 import { isChartStyle } from "../chart/types";
 import { useHighlight } from "../highlight/store";
+import { useSession } from "../session/store";
 import { chartLabels } from "./labels";
 import "./panels.css";
-import { useSky } from "./sky";
-import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
 import type { PanelProps } from "./types";
 
-export function chartVarga(settings: Record<string, unknown>): string {
-  return typeof settings.varga === "string" ? settings.varga : "D1";
+export function chartVarga(settings: Record<string, unknown>): Varga {
+  return isVarga(settings.varga) ? settings.varga : "D1";
 }
 
-/**
- * Harita paneli. Lagna ve varga komutları M2–M3'te gelir; şimdilik yalnızca D1 çizilir ve
- * şimdiki gökyüzü gösterilir (1. ev Koç). Diğer vargalar açık bir bilgi metni gösterir.
- */
+/** Harita paneli: açık haritanın `settings.varga` vargası, `settings.style` biçiminde. */
 export function ChartPanel({ panel }: PanelProps) {
   const { t } = useTranslation();
-  const sky = useSky();
+  const { opened } = useSession();
   const highlight = useHighlight();
   const varga = chartVarga(panel.settings);
-  const phase = usePanelLoadPhase(varga === "D1" && sky.state.status === "loading");
   const style = isChartStyle(panel.settings.style) ? panel.settings.style : "north";
 
-  if (varga !== "D1") return <p className="panel-message">{t("chart.vargaUnavailable", { varga })}</p>;
-  if (sky.state.status === "error") return <PanelError error={sky.state.error} onRetry={sky.retry} />;
-  if (sky.state.status === "loading") return phase === "skeleton" ? <PanelSkeleton /> : null;
-
-  const data = chartDataFromPlanetRows(sky.state.rows);
+  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
+  const data = chartDataFromChart(opened.chart, varga, { showLagna: true });
+  if (!data) return <p className="panel-message">{t("chart.vargaMissing", { varga })}</p>;
   return (
     <div className="chart-panel">
-      {data.ascSign === null && (
-        <p className="panel-notice" role="note">
-          {t("chart.noLagna")}
-        </p>
-      )}
       <div className="chart-panel-figure">
         <ChartSvg
           data={data}
```

`app/ui/panels/GrahaTablePanel.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/panels/GrahaTablePanel.tsx
+++ b/app/ui/panels/GrahaTablePanel.tsx
@@ -1,24 +1,20 @@
 import { useTranslation } from "react-i18next";
-import { signAndDegree } from "../chart/adapter";
-import { isGrahaId } from "../chart/types";
+import { signIndex } from "../chart/adapter";
 import { useHighlight } from "../highlight/store";
 import { sameTarget, type HighlightTarget } from "../highlight/target";
+import { useSession } from "../session/store";
 import { formatDegree } from "./format";
 import "./panels.css";
-import { useSky } from "./sky";
-import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
 import type { PanelProps } from "./types";
 
-/** Graha tablosu: graha, burç, burç içi derece, retro. Sütun seçimi (`settings.columns`) sonraki planda. */
+/** Graha tablosu: graha, burç, burç içi derece, retro. Değerler `Chart.grahas`'tan gelir. */
 export function GrahaTablePanel(_: PanelProps) {
   const { t } = useTranslation();
-  const sky = useSky();
+  const { opened } = useSession();
   const highlight = useHighlight();
-  const phase = usePanelLoadPhase(sky.state.status === "loading");
 
-  if (sky.state.status === "error") return <PanelError error={sky.state.error} onRetry={sky.retry} />;
-  if (sky.state.status === "loading") return phase === "skeleton" ? <PanelSkeleton /> : null;
-  if (sky.state.rows.length === 0) return <p className="panel-message">{t("table.empty")}</p>;
+  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
+  if (opened.chart.grahas.length === 0) return <p className="panel-message">{t("table.empty")}</p>;
 
   return (
     <table className="graha-table">
@@ -31,38 +27,33 @@ export function GrahaTablePanel(_: PanelProps) {
         </tr>
       </thead>
       <tbody>
-        {sky.state.rows.map((row) => {
-          const pos = row.lon === null ? null : signAndDegree(row.lon);
-          const target: HighlightTarget | null = isGrahaId(row.body) ? { type: "graha", id: row.body } : null;
-          const on = target !== null && sameTarget(highlight.active, target);
-          const pinned = target !== null && sameTarget(highlight.pinned, target);
+        {opened.chart.grahas.map((g) => {
+          const target: HighlightTarget = { type: "graha", id: g.graha };
+          const on = sameTarget(highlight.active, target);
+          const pinned = sameTarget(highlight.pinned, target);
           return (
             <tr
-              key={row.body}
-              data-graha={row.body}
+              key={g.graha}
+              data-graha={g.graha}
               className={on ? "is-highlighted" : undefined}
-              onMouseEnter={target ? () => highlight.hover(target) : undefined}
-              onMouseLeave={target ? () => highlight.hover(null) : undefined}
+              onMouseEnter={() => highlight.hover(target)}
+              onMouseLeave={() => highlight.hover(null)}
             >
               <th scope="row">
-                {target ? (
-                  <button
-                    type="button"
-                    className="link-button"
-                    aria-pressed={pinned}
-                    onClick={() => highlight.togglePin(target)}
-                    onFocus={() => highlight.hover(target)}
-                    onBlur={() => highlight.hover(null)}
-                  >
-                    {t(`grahaName.${row.body}`)}
-                  </button>
-                ) : (
-                  row.body
-                )}
+                <button
+                  type="button"
+                  className="link-button"
+                  aria-pressed={pinned}
+                  onClick={() => highlight.togglePin(target)}
+                  onFocus={() => highlight.hover(target)}
+                  onBlur={() => highlight.hover(null)}
+                >
+                  {t(`grahaName.${g.graha}`)}
+                </button>
               </th>
-              <td>{pos ? t(`sign.${pos.sign}`) : "—"}</td>
-              <td className="num">{pos ? formatDegree(pos.degInSign) : "—"}</td>
-              <td>{row.retrograde ? "℞" : ""}</td>
+              <td>{t(`sign.${signIndex(g.sign)}`)}</td>
+              <td className="num">{g.deg_in_sign === null ? "—" : formatDegree(g.deg_in_sign)}</td>
+              <td>{g.retrograde ? "℞" : ""}</td>
             </tr>
           );
         })}
```

`app/ui/panels/sky.tsx` silinir:

```bash
git rm app/ui/panels/sky.tsx
```

`app/ui/panels/states.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/panels/states.tsx
+++ b/app/ui/panels/states.tsx
@@ -3,6 +3,7 @@
 import { createContext, useContext, useEffect, useState } from "react";
 import { useTranslation } from "react-i18next";
 import type { CommandError } from "../bindings";
+import { errorMessage } from "../i18n/errors";
 
 export type LoadPhase = "none" | "inline" | "skeleton";
 export const INLINE_AFTER_MS = 100;
@@ -44,17 +45,9 @@ export function PanelSkeleton() {
   );
 }
 
-// Arka uçtan gelen parametreler i18next seçeneklerini (dil, çoğul, bağlam…) değiştirememeli.
-const RESERVED_OPTIONS = new Set(["count", "context", "lng", "lngs", "defaultValue", "ns", "replace", "returnObjects", "joinArrays", "postProcess", "interpolation", "fallbackLng"]);
-
-function safeParams(params: Record<string, unknown>): Record<string, unknown> {
-  return Object.fromEntries(Object.entries(params).filter(([k]) => !RESERVED_OPTIONS.has(k)));
-}
-
 export function PanelError({ error, onRetry }: { error: CommandError; onRetry: () => void }) {
   const { t, i18n } = useTranslation();
-  const key = `errors.${error.code}`;
-  const message = i18n.exists(key) ? t(key, safeParams(error.params)) : t("errors.unknown", { code: error.code });
+  const message = errorMessage(t, i18n, error);
   return (
     <div className="panel-error" role="alert">
       <p>{message}</p>
```

`app/ui/session/store.tsx` oluştur:

```tsx
// Açık harita (oturum). Harita Rust'ta hesaplanmış gelir (`OpenedClient`); arayüz yalnızca
// çizer (spec §4). Yeni harita açılınca görünüm deposu o haritanın dasha yoluna geçer.
import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from "react";
import type { OpenedClient } from "../bindings";
import { useViewStore } from "../view/store";

export interface Session {
  opened: OpenedClient | null;
  /** `create_client`/`update_chart` dönüşü: haritayı açar, görünümü ona bağlar. */
  open(opened: OpenedClient): void;
}

const SessionContext = createContext<Session | null>(null);

/**
 * `initial` açılışta geri yüklenen haritadır (`restore_session`); görünüm deposunun ilk
 * modeli aynı haritayla kurulmuş olmalıdır (`initialViewModel(layout, { id, dashaPath })`).
 */
export function SessionProvider({ initial, children }: { initial: OpenedClient | null; children: ReactNode }) {
  const [opened, setOpened] = useState(initial);
  const { openChart } = useViewStore();
  const open = useCallback(
    (o: OpenedClient) => {
      openChart(o.chart_record.id, o.view_state.dasha_expanded_path);
      setOpened(o);
    },
    [openChart],
  );
  const value = useMemo(() => ({ opened, open }), [opened, open]);
  return <SessionContext.Provider value={value}>{children}</SessionContext.Provider>;
}

export function useSession(): Session {
  const s = useContext(SessionContext);
  if (s === null) throw new Error("useSession must be used inside <SessionProvider>");
  return s;
}
```

`app/ui/shell/ChartTitle.tsx` oluştur:

```tsx
// Üst çubuğun solu (UI spec §4): danışan adı, harita etiketi ve uyarı rozetleri (UI spec §5).
// Rozet amber kenarlı ve işaretlidir ama anlamı metinle taşır; açıklama ipucu olarak
// (`title`) ve ekran okuyucu için gizli metin olarak verilir.
import { useTranslation } from "react-i18next";
import { useSession } from "../session/store";

export function ChartTitle() {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <h1 className="topbar-title">{t("app.title")}</h1>;
  const label = opened.chart_record.label || t(`chart.kind.${opened.chart_record.kind}`);
  const warnings = [...opened.chart_record.resolved.warnings, ...opened.chart.meta.warnings];
  return (
    <div className="topbar-title">
      <h1 className="topbar-client">{opened.client.name}</h1>
      <span className="topbar-chart">{label}</span>
      {warnings.length > 0 && (
        <ul className="badges" aria-label={t("shell.warnings")}>
          {warnings.map((w) => (
            <li key={w} className="badge" tabIndex={0} title={t(`badgeHint.${w}`)}>
              <span className="badge-mark" aria-hidden="true">
                ▲
              </span>
              {t(`badge.${w}`)}
              <span className="visually-hidden">: {t(`badgeHint.${w}`)}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
```

`app/ui/shell/PrintView.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/PrintView.tsx
+++ b/app/ui/shell/PrintView.tsx
@@ -2,20 +2,27 @@ import { useState } from "react";
 import { useTranslation } from "react-i18next";
 import { api } from "../api";
 import type { CommandError } from "../bindings";
-import { useSky } from "../panels/sky";
+import { signIndex } from "../chart/adapter";
 import { PanelError } from "../panels/states";
-import { PrintSpike } from "../PrintSpike";
+import { PrintSpike, type SpikeRow } from "../PrintSpike";
+import { useSession } from "../session/store";
 
 /** M0 baskı denemesi; M7'deki A4 baskı düzenleri gelene kadar menüden erişilir. */
 export function PrintView() {
   const { t } = useTranslation();
-  const sky = useSky();
+  const { opened } = useSession();
   const [error, setError] = useState<CommandError | null>(null);
   const print = async () => {
     setError(null);
     const r = await api.printWindow();
     if (r.status === "error") setError(r.error);
   };
+  const rows: SpikeRow[] = (opened?.chart.grahas ?? []).map((g) => ({
+    body: g.graha,
+    sign: signIndex(g.sign),
+    lon: g.lon,
+    retrograde: g.retrograde,
+  }));
   return (
     <div>
       <div className="print-view-actions no-print">
@@ -24,7 +31,7 @@ export function PrintView() {
         </button>
         {error && <PanelError error={error} onRetry={() => void print()} />}
       </div>
-      <PrintSpike rows={sky.state.status === "ok" ? sky.state.rows : []} />
+      <PrintSpike rows={rows} />
     </div>
   );
 }
```

`app/ui/shell/Shell.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/Shell.tsx
+++ b/app/ui/shell/Shell.tsx
@@ -3,6 +3,7 @@ import { Dashboard } from "../dashboard/Dashboard";
 import { isEditableTarget } from "../keyboard";
 import { useLayoutStore } from "../view/store";
 import { applyTheme, loadThemePref, saveThemePref, type ThemePref } from "../theme/theme";
+import { NoticeBar } from "./notices";
 import { PrintView } from "./PrintView";
 import "./shell.css";
 import { TopBar } from "./TopBar";
@@ -45,6 +46,7 @@ export function Shell() {
         printView={printView}
         onTogglePrint={() => setPrintView((v) => !v)}
       />
+      <NoticeBar />
       <main className="shell-main">
         <div ref={measureRef}>{printView ? <PrintView /> : <Dashboard width={width} />}</div>
       </main>
```

`app/ui/shell/TopBar.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/TopBar.tsx
+++ b/app/ui/shell/TopBar.tsx
@@ -1,6 +1,7 @@
 import { useTranslation } from "react-i18next";
 import { isLanguage, LANGUAGES, setLanguage, DEFAULT_LANGUAGE } from "../i18n";
 import { isThemePref, THEME_PREFS, type ThemePref } from "../theme/theme";
+import { ChartTitle } from "./ChartTitle";
 import { LayoutPicker } from "./LayoutPicker";
 
 export interface TopBarProps {
@@ -10,13 +11,13 @@ export interface TopBarProps {
   onTogglePrint(): void;
 }
 
-/** Üst çubuk (spec §4): solda başlık, ortada düzen seçici, sağda tema, dil ve baskı. */
+/** Üst çubuk (spec §4): solda danışan ve harita (uyarı rozetleriyle), ortada düzen seçici, sağda tema, dil ve baskı. */
 export function TopBar({ themePref, onThemeChange, printView, onTogglePrint }: TopBarProps) {
   const { t, i18n } = useTranslation();
   const language = isLanguage(i18n.language) ? i18n.language : DEFAULT_LANGUAGE;
   return (
     <header className="topbar no-print">
-      <h1 className="topbar-title">{t("app.title")}</h1>
+      <ChartTitle />
       <div className="topbar-center">{!printView && <LayoutPicker />}</div>
       <div className="topbar-right">
         <label>
```

`app/ui/shell/notices.tsx` oluştur:

```tsx
// Modal olmayan bildirim bandı (spec §7: hesap ve kayıt hataları, açılış uyarıları).
// Bildirimler yapı olarak saklanır ve çizimde çevrilir; dil değişince metin de değişir.
// Aynı `id` ile gelen bildirim öncekinin yerini alır (ör. tekrarlayan kayıt hatası).
import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import type { CommandError, StoreWarning } from "../bindings";
import { errorMessage } from "../i18n/errors";

export type Notice =
  | { id: string; type: "error"; messageKey: string; error: CommandError }
  | { id: string; type: "warning"; messageKey: string; params: Record<string, string> };

export interface Notices {
  notices: Notice[];
  push(notice: Notice): void;
  dismiss(id: string): void;
}

const NoticeContext = createContext<Notices | null>(null);

export function NoticeProvider({ initial = [], children }: { initial?: Notice[]; children: ReactNode }) {
  const [notices, setNotices] = useState(initial);
  const push = useCallback((n: Notice) => setNotices((all) => [...all.filter((m) => m.id !== n.id), n]), []);
  const dismiss = useCallback((id: string) => setNotices((all) => all.filter((m) => m.id !== id)), []);
  const value = useMemo(() => ({ notices, push, dismiss }), [notices, push, dismiss]);
  return <NoticeContext.Provider value={value}>{children}</NoticeContext.Provider>;
}

export function useNotices(): Notices {
  const n = useContext(NoticeContext);
  if (n === null) throw new Error("useNotices must be used inside <NoticeProvider>");
  return n;
}

/** `open_client` uyarıları (spec §7) → bildirimler. */
export function storeWarningNotices(warnings: readonly StoreWarning[]): Notice[] {
  return warnings.map((w): Notice => {
    if (w === "LastChartMissing") return { id: "store:LastChartMissing", type: "warning", messageKey: "storeWarning.LastChartMissing", params: {} };
    if ("CalcSettingsReset" in w) return { id: "store:CalcSettingsReset", type: "warning", messageKey: "storeWarning.CalcSettingsReset", params: {} };
    return {
      id: "store:TzdbChanged",
      type: "warning",
      messageKey: "storeWarning.TzdbChanged",
      params: { stored: w.TzdbChanged.stored_version, current: w.TzdbChanged.current_version },
    };
  });
}

export function NoticeBar() {
  const { t, i18n } = useTranslation();
  const { notices, dismiss } = useNotices();
  if (notices.length === 0) return null;
  return (
    <section className="notice-bar no-print" aria-label={t("notice.region")}>
      {notices.map((n) => (
        <div key={n.id} className={`notice notice--${n.type}`} role={n.type === "error" ? "alert" : "status"}>
          <strong className="notice-kind">{t(`notice.${n.type}`)}</strong>
          <span className="notice-text">
            {n.type === "error" ? t(n.messageKey, { message: errorMessage(t, i18n, n.error) }) : t(n.messageKey, n.params)}
          </span>
          <button type="button" onClick={() => dismiss(n.id)}>
            {t("notice.dismiss")}
          </button>
        </div>
      ))}
    </section>
  );
}
```

`app/ui/shell/shell.css` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/shell.css
+++ b/app/ui/shell/shell.css
@@ -16,6 +16,10 @@
   margin: 0;
   font-size: var(--fs-client);
   font-weight: var(--fw-medium);
+  display: flex;
+  align-items: baseline;
+  flex-wrap: wrap;
+  gap: 8px;
 }
 .topbar-center {
   flex: 1;
@@ -76,3 +80,58 @@
 .print-view-actions {
   margin-bottom: 8px;
 }
+.topbar-client {
+  margin: 0;
+  font-size: var(--fs-client);
+  font-weight: var(--fw-medium);
+}
+.topbar-chart {
+  color: var(--text-2);
+}
+/* Rozet: anlam metindedir; amber kenar ve ▲ işareti yalnızca dikkat çeker. --warn metin
+   rengi olarak AA vermez, bu yüzden metin --text'tir (tokens.test yalnızca 3:1 denetler). */
+.badges {
+  display: flex;
+  gap: 6px;
+  margin: 0;
+  padding: 0;
+  list-style: none;
+}
+.badge {
+  display: inline-flex;
+  align-items: center;
+  gap: 4px;
+  padding: 0 6px;
+  border: 1px solid var(--warn);
+  border-radius: 4px;
+  color: var(--text);
+  font-size: var(--fs-table);
+}
+.badge-mark {
+  color: var(--warn);
+}
+.notice-bar {
+  display: flex;
+  flex-direction: column;
+  gap: 4px;
+  padding: 4px 12px;
+  background: var(--panel);
+  border-bottom: 1px solid var(--line);
+}
+.notice {
+  display: flex;
+  align-items: center;
+  gap: 8px;
+  color: var(--text);
+}
+.notice--warning {
+  border-left: 3px solid var(--warn);
+  padding-left: 8px;
+}
+.notice--error {
+  border-left: 3px solid var(--danger);
+  padding-left: 8px;
+}
+.notice-text {
+  flex: 1;
+}
```

`app/ui/time.ts` silinir:

```bash
git rm app/ui/time.ts
```


- [ ] **Step 4: Bağlamaları yeniden üret**

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings
git diff --stat app/ui/bindings.ts
cargo test -p jyotish-desk --test bindings
```
Expected: ilk komut `app/ui/bindings.ts`'i yazar; ikinci çalıştırma `test result: ok. 1 passed` (dosya güncel). `bindings.ts` elle düzenlenmez.

- [ ] **Step 5: Testlerin geçtiğini gör**

```bash
cargo test -p jyotish-desk --lib
```
Expected: `test result: ok. 13 passed` (deneme testleri silindi).
```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```
Expected: ikisi de çıktısız, çıkış kodu 0.
```bash
pnpm --dir app test
pnpm --dir app build
```
Expected: tam takımda `Test Files  26 passed (26)`, `Tests  226 passed (226)`; `build` hatasız (`tsc --noEmit` çıktısız, `vite build` `✓ built`).

- [ ] **Step 6: Commit**

```bash
git add -A app/src-tauri/src/commands.rs app/src-tauri/src/lib.rs app/ui/App.test.tsx app/ui/App.tsx app/ui/PrintSpike.test.tsx app/ui/PrintSpike.tsx app/ui/api.test.ts app/ui/api.ts app/ui/bindings.ts app/ui/boot.ts app/ui/chart/adapter.test.ts app/ui/chart/adapter.ts app/ui/chart/types.ts app/ui/dashboard/Dashboard.test.tsx app/ui/i18n/en.ts app/ui/i18n/errors.ts app/ui/i18n/fr.ts app/ui/i18n/i18n.test.ts app/ui/i18n/tr.ts app/ui/main.tsx app/ui/panels/ChartPanel.tsx app/ui/panels/GrahaTablePanel.tsx app/ui/panels/panels.test.tsx app/ui/panels/sky.tsx app/ui/panels/states.tsx app/ui/session/store.tsx app/ui/shell/ChartTitle.tsx app/ui/shell/PrintView.tsx app/ui/shell/Shell.tsx app/ui/shell/TopBar.tsx app/ui/shell/notices.tsx app/ui/shell/shell.css app/ui/test/render.tsx app/ui/time.test.ts app/ui/time.ts
git diff --cached -U0 -- . ':!app/pnpm-lock.yaml' ':!*.png' ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git commit -m "feat(ui): store-backed session, boot restore, notices and warning badges; drop the current-sky spike

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: Harita paneli: varga ve stil seçimi, saat bilinmiyor çizimi, SVG anlık görüntüleri

Harita panelinin araç çubuğu iki seçim kutusu taşır (varga, biçim); seçim panelin `settings`'ine yazılır ve düzenle birlikte kaydedilir. Saat bilinmiyorsa (D11) lagna işareti ve dolgusu yoktur, 1. ev Koç'tur ve panel bir not gösterir; D1 dışındaki vargalarda not, bölüm haritası yerleşimlerinin saate bağlı ve yaklaşık olduğunu ekler (spec §7). Bu görevin commit'i üç SVG anlık görüntüsünü de içerir. Üç stil için Delhi D1 SVG işaretlemesi dosya anlık görüntüsüyle sabitlenir (D19a). `ChartSvg` geometrisi Plan 04'teki gibidir; yalnızca `hasLagna` eklenir.

**Files:**
- Modify: `app/ui/chart/ChartSvg.test.tsx`
- Modify: `app/ui/chart/ChartSvg.tsx`
- Create: `app/ui/chart/__snapshots__/delhi-d1-circle.svg` (üretilir: ilk `pnpm --dir app test` çalıştırması (`toMatchFileSnapshot`))
- Create: `app/ui/chart/__snapshots__/delhi-d1-north.svg` (üretilir: ilk `pnpm --dir app test` çalıştırması (`toMatchFileSnapshot`))
- Create: `app/ui/chart/__snapshots__/delhi-d1-south.svg` (üretilir: ilk `pnpm --dir app test` çalıştırması (`toMatchFileSnapshot`))
- Modify: `app/ui/chart/adapter.ts`
- Modify: `app/ui/i18n/en.ts`
- Modify: `app/ui/i18n/fr.ts`
- Modify: `app/ui/i18n/tr.ts`
- Modify: `app/ui/layout/reducer.test.ts`
- Modify: `app/ui/layout/reducer.ts`
- Create: `app/ui/panels/ChartPanel.test.tsx`
- Modify: `app/ui/panels/ChartPanel.tsx`
- Modify: `app/ui/panels/panels.css`
- Modify: `app/ui/view/store.tsx`

**Interfaces:**
- Consumes: `chartDataFromChart`, `useSession` (Task 5), `useLayoutStore` (Task 4), `ChartSvg`, `geometry.ts` (Plan 04).
- Produces: düzen eylemi `{type: "setPanelSettings", panelId, patch}` ve `LayoutStore.setPanelSettings(panelId, patch)`; `ChartSvg` içinde `hasLagna = data.ascSign !== null` (ayrı `ChartData` alanı yok); `VARGA_NAMES: Record<Varga, string>`; panel ayarları `settings.varga: Varga`, `settings.style: "north" | "south" | "circle"`; i18n `chart.varga`, `chart.style`, `chart.timeUnknown`, `chart.timeUnknownVarga`, `chartStyle.*`.

- [ ] **Step 1: Önce testleri yaz**



`app/ui/chart/ChartSvg.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/chart/ChartSvg.test.tsx
+++ b/app/ui/chart/ChartSvg.test.tsx
@@ -237,3 +237,12 @@ test("chart text uses tokens, --ink-strong signs and tabular numerals; highlight
   expect(rule(".chart-cell--lagna.chart-cell--highlight")).toMatch(/stroke-width/);
   expect(rule(".chart-planet--highlight .chart-planet-bg")).toMatch(/stroke: var\(--ink\)/);
 });
+
+test.each(STYLES)("%s without a lagna draws no lagna marker or fill and names houses by sign", (style) => {
+  const { container } = render(<ChartSvg data={{ ...data, ascSign: null }} style={style} labels={labels} />);
+  expect(container.querySelector(".chart-cell--lagna")).toBeNull();
+  expect([...container.querySelectorAll("text.chart-sign")].map((t) => t.textContent)).not.toContain("As");
+  expect(screen.queryByRole("button", { name: /^House / })).toBeNull();
+  expect(container.querySelector('[data-house="1"]')).toHaveAttribute("data-sign", "0");
+  expect(container.querySelector('[data-house="1"]')).toHaveAttribute("aria-label", "Ar");
+});
```

`app/ui/layout/reducer.test.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/layout/reducer.test.ts
+++ b/app/ui/layout/reducer.test.ts
@@ -119,3 +119,13 @@ test("newId yields distinct prefixed ids", () => {
   expect(a).toMatch(/^user:/);
   expect(a).not.toBe(b);
 });
+
+test("setPanelSettings merges into one panel of the active layout and keeps unknown keys", () => {
+  const state = defaultLayoutState();
+  const panel = activeLayout(state).panels.find((p) => p.kind === "chart")!;
+  const withExtra = layoutReducer(state, { type: "setPanelSettings", panelId: panel.id, patch: { future: 1 } });
+  const next = layoutReducer(withExtra, { type: "setPanelSettings", panelId: panel.id, patch: { varga: "D10" } });
+  const updated = activeLayout(next).panels.find((p) => p.id === panel.id)!;
+  expect(updated.settings).toEqual({ ...panel.settings, future: 1, varga: "D10" });
+  expect(layoutReducer(next, { type: "setPanelSettings", panelId: "missing", patch: { varga: "D2" } })).toEqual(next);
+});
```

`app/ui/panels/ChartPanel.test.tsx` oluştur:

```tsx
import { fireEvent, screen, waitFor } from "@testing-library/react";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { ChartPanel } from "./ChartPanel";

const LAYOUT = (settings: Record<string, unknown>) => ({
  schema_version: 1,
  active_layout_id: "user:t",
  layouts: [{ id: "user:t", name: "T", builtin: false, panels: [{ id: "c", kind: "chart", x: 0, y: 0, w: 6, h: 12, settings } satisfies Panel] }],
});

function renderPanel(settings: Record<string, unknown>, opened = fixture().opened) {
  const layout = LAYOUT(settings);
  const panel = layout.layouts[0].panels[0];
  return renderWithStores(<ChartPanel panel={panel} />, { layout, opened });
}

test("varga and style selectors write the panel settings and keep unknown keys", async () => {
  const { persistence } = renderPanel({ varga: "D1", style: "north", transit_ring: false });
  const varga = screen.getByRole("combobox", { name: "Varga" });
  expect(varga).toHaveDisplayValue("D1 · Rasi");
  expect(varga.querySelectorAll("option")).toHaveLength(16);
  fireEvent.change(varga, { target: { value: "D9" } });
  fireEvent.change(screen.getByRole("combobox", { name: "Biçim" }), { target: { value: "circle" } });
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  const saved = persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0];
  expect(saved.settings).toEqual({ varga: "D9", style: "circle", transit_ring: false });
  expect(persistence.saved.at(-1)!.chartId).toBe(fixture().opened.chart_record.id);
});

test("unknown birth time hides the lagna, names houses by sign and explains why", () => {
  const { container } = renderPanel({ varga: "D1", style: "north" }, unknownTimeOpened());
  expect(screen.getByRole("note")).toHaveTextContent("Doğum saati bilinmiyor: lagna ve evler gösterilmiyor, 1. ev Koç.");
  expect(container.querySelector(".chart-cell--lagna")).toBeNull();
  expect(container.querySelector('[data-house="1"]')).toHaveAttribute("data-sign", "0");
  expect(screen.queryByRole("button", { name: /\. ev,/ })).toBeNull();
  expect(screen.getByRole("note")).not.toHaveTextContent("Bölüm haritasındaki");
});

test("unknown birth time in a divisional chart adds that placements are approximate", () => {
  renderPanel({ varga: "D9", style: "north" }, unknownTimeOpened());
  expect(screen.getByRole("note")).toHaveTextContent("Bölüm haritasındaki yerleşimler doğum saatine bağlıdır ve yaklaşıktır.");
});

test("D9 marks vargottama and never combust", () => {
  const opened = fixture().opened;
  const jupiter = opened.chart.grahas.find((g) => g.graha === "Jupiter")!;
  expect(jupiter.vargottama).toBe(true);
  opened.chart.grahas.find((g) => g.graha === "Mercury")!.combust = true;
  renderPanel({ varga: "D9", style: "south" }, opened);
  expect(screen.getByRole("button", { name: /^Jü, .*vargottama$/ })).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: /yanık/ })).toBeNull();
});

// Görsel regresyon (temadan bağımsız işaretleme): üç biçim, gerçek Delhi D1. Renkler
// token'larla CSS'te; ekran görüntüsü karşılaştırması Playwright `e2e:visual`'da.
test.each(["north", "south", "circle"])("Delhi D1 %s markup matches the snapshot", async (style) => {
  const { container } = renderPanel({ varga: "D1", style });
  await expect(container.querySelector("svg")!.outerHTML).toMatchFileSnapshot(`../chart/__snapshots__/delhi-d1-${style}.svg`);
});
```


- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/panels/ChartPanel.test.tsx ui/chart/ChartSvg.test.tsx ui/layout/reducer.test.ts
```
Expected: FAIL — `Unable to find an accessible element with the role "combobox" and name "Varga"`, `setPanelSettings` eylemi düzeni değiştirmiyor.

- [ ] **Step 3: Uygulamayı yaz**



`app/ui/chart/ChartSvg.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/chart/ChartSvg.tsx
+++ b/app/ui/chart/ChartSvg.tsx
@@ -43,6 +43,9 @@ function planetName(p: ChartPlanet, labels: ChartLabels): string {
 }
 
 export function ChartSvg({ data, style, labels, highlight = null, pressed = highlight, onHover, onPin }: ChartSvgProps) {
+  // Saat bilinmiyorsa lagna yoktur: 1. ev Koç çizilir, lagna işareti ve dolgusu çizilmez,
+  // evler yalnızca burç adıyla anılır (ev numarası anlamsızdır).
+  const hasLagna = data.ascSign !== null;
   const asc = data.ascSign ?? 0;
   const cells = useMemo(() => chartCells(style, asc), [style, asc]);
   // Yerleşim yalnızca kısaltmalara ve lagna işaretine bağlıdır; `labels` nesnesi her
@@ -58,11 +61,11 @@ export function ChartSvg({ data, style, labels, highlight = null, pressed = high
     return cells.map((cell) => {
       const planets = bySign.get(cell.sign) ?? [];
       const texts = planets.map((p) => labelText(p, graha));
-      const layout: LabelLayout = layoutLabels(cell, texts, cell.lagnaAnchor ? lagna : "");
+      const layout: LabelLayout = layoutLabels(cell, texts, hasLagna && cell.lagnaAnchor ? lagna : "");
       return { cell, planets, layout };
     });
     // textKey, graha ve lagna'yı kapsar.
-  }, [cells, data.planets, textKey]);
+  }, [cells, data.planets, textKey, hasLagna]);
 
   const activate = (target: HighlightTarget) => (e: KeyboardEvent) => {
     if (e.key === "Enter" || e.key === " ") {
@@ -85,7 +88,7 @@ export function ChartSvg({ data, style, labels, highlight = null, pressed = high
         const on = cellHighlighted(cell, highlight);
         const isPressed = cellHighlighted(cell, pressed);
         const classes = ["chart-cell"];
-        if (cell.house === 1) classes.push("chart-cell--lagna");
+        if (hasLagna && cell.house === 1) classes.push("chart-cell--lagna");
         if (on) classes.push("chart-cell--highlight");
         return (
           <g
@@ -96,7 +99,7 @@ export function ChartSvg({ data, style, labels, highlight = null, pressed = high
             role="button"
             tabIndex={0}
             aria-pressed={isPressed}
-            aria-label={labels.house(cell.house, labels.signs[cell.sign])}
+            aria-label={hasLagna ? labels.house(cell.house, labels.signs[cell.sign]) : labels.signs[cell.sign]}
             onMouseEnter={() => onHover?.(house)}
             onMouseLeave={() => onHover?.(null)}
             onFocus={() => onHover?.(house)}
@@ -110,7 +113,7 @@ export function ChartSvg({ data, style, labels, highlight = null, pressed = high
                 {cell.sign + 1}
               </text>
             )}
-            {cell.lagnaAnchor && (
+            {hasLagna && cell.lagnaAnchor && (
               <text className="chart-sign" x={cell.lagnaAnchor.x} y={cell.lagnaAnchor.y} textAnchor="middle" dominantBaseline="central">
                 {labels.lagna}
               </text>
```

`app/ui/chart/adapter.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/chart/adapter.ts
+++ b/app/ui/chart/adapter.ts
@@ -22,6 +22,26 @@ export const SIGNS = [
 /** `Varga::ALL` sırası (Shodasavarga). */
 export const VARGAS = ["D1", "D2", "D3", "D4", "D7", "D9", "D10", "D12", "D16", "D20", "D24", "D27", "D30", "D40", "D45", "D60"] as const satisfies readonly Varga[];
 
+/** Varga adları (Sanskritçe terim; dile göre değişmez). */
+export const VARGA_NAMES: Record<Varga, string> = {
+  D1: "Rasi",
+  D2: "Hora",
+  D3: "Drekkana",
+  D4: "Chaturthamsa",
+  D7: "Saptamsa",
+  D9: "Navamsa",
+  D10: "Dasamsa",
+  D12: "Dwadasamsa",
+  D16: "Shodasamsa",
+  D20: "Vimsamsa",
+  D24: "Chaturvimsamsa",
+  D27: "Bhamsa",
+  D30: "Trimsamsa",
+  D40: "Khavedamsa",
+  D45: "Akshavedamsa",
+  D60: "Shashtiamsa",
+};
+
 export function isVarga(v: unknown): v is Varga {
   return typeof v === "string" && (VARGAS as readonly string[]).includes(v);
 }
```

`app/ui/i18n/en.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -40,6 +40,10 @@ export const en: Messages = {
       Natal: "Natal chart",
       Event: "Event chart",
     },
+    varga: "Varga",
+    style: "Style",
+    timeUnknown: "Birth time unknown: lagna and houses are hidden, house 1 = Aries. Moon, nakshatra and dasha are approximate (≈).",
+    timeUnknownVarga: "Divisional placements depend on the birth time and are approximate.",
   },
   panel: {
     chart: "Chart",
@@ -156,4 +160,9 @@ export const en: Messages = {
     CalcSettingsReset: "This chart's calculation settings could not be read; defaults are used.",
     TzdbChanged: "Time zone data changed ({{stored}} → {{current}}); the stored moment is used.",
   },
+  chartStyle: {
+    north: "North",
+    south: "South",
+    circle: "Circle",
+  },
 };
```

`app/ui/i18n/fr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -40,6 +40,10 @@ export const fr: Messages = {
       Natal: "Carte natale",
       Event: "Carte d’événement",
     },
+    varga: "Varga",
+    style: "Style",
+    timeUnknown: "Heure de naissance inconnue : lagna et maisons masqués, maison 1 = Bélier. Lune, nakshatra et dasha sont approximatifs (≈).",
+    timeUnknownVarga: "Les positions dans les cartes divisionnaires dépendent de l’heure de naissance et sont approximatives.",
   },
   panel: {
     chart: "Carte",
@@ -156,4 +160,9 @@ export const fr: Messages = {
     CalcSettingsReset: "Les réglages de calcul de cette carte sont illisibles ; les réglages par défaut sont utilisés.",
     TzdbChanged: "Les données de fuseau ont changé ({{stored}} → {{current}}) ; l’instant enregistré est utilisé.",
   },
+  chartStyle: {
+    north: "Nord",
+    south: "Sud",
+    circle: "Cercle",
+  },
 };
```

`app/ui/i18n/tr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -39,6 +39,10 @@ export const tr = {
       Natal: "Doğum haritası",
       Event: "Olay haritası",
     },
+    varga: "Varga",
+    style: "Biçim",
+    timeUnknown: "Doğum saati bilinmiyor: lagna ve evler gösterilmiyor, 1. ev Koç. Ay, nakshatra ve dasa yaklaşıktır (≈).",
+    timeUnknownVarga: "Bölüm haritasındaki yerleşimler doğum saatine bağlıdır ve yaklaşıktır.",
   },
   panel: {
     chart: "Harita",
@@ -155,6 +159,11 @@ export const tr = {
     CalcSettingsReset: "Bu haritanın hesap ayarı okunamadı; varsayılan ayar kullanılıyor.",
     TzdbChanged: "Saat dilimi verisi değişti ({{stored}} → {{current}}); saklanan an kullanılıyor.",
   },
+  chartStyle: {
+    north: "Kuzey",
+    south: "Güney",
+    circle: "Daire",
+  },
 };
 
 type Strings<T> = { [K in keyof T]: T[K] extends string ? string : Strings<T[K]> };
```

`app/ui/layout/reducer.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/layout/reducer.ts
+++ b/app/ui/layout/reducer.ts
@@ -11,7 +11,8 @@ export type LayoutAction =
   | { type: "delete"; id: string }
   | { type: "resetBuiltin"; id: string }
   | { type: "updatePanels"; panels: Panel[] }
-  | { type: "toggleNotes"; panelId: string };
+  | { type: "toggleNotes"; panelId: string }
+  | { type: "setPanelSettings"; panelId: string; patch: Record<string, unknown> };
 
 /** Notlar paneli düzende yoksa sağa 4 sütun genişliğinde eklenir (spec §3.2). */
 export const NOTES_PANEL = { x: 8, y: 0, w: 4, h: 8 } as const;
@@ -83,6 +84,13 @@ export function layoutReducer(state: LayoutState, action: LayoutAction): LayoutS
     }
     case "updatePanels":
       return mapLayout(state, state.active_layout_id, (l) => ({ ...l, panels: normalizePanels(action.panels) }));
+    case "setPanelSettings":
+      // Yalnızca verilen anahtarlar değişir; bilinmeyen ayarlar korunur (spec §3.3).
+      return mapLayout(state, state.active_layout_id, (l) =>
+        l.panels.some((p) => p.id === action.panelId)
+          ? { ...l, panels: l.panels.map((p) => (p.id === action.panelId ? { ...p, settings: { ...p.settings, ...action.patch } } : p)) }
+          : l,
+      );
     case "toggleNotes":
       return mapLayout(state, state.active_layout_id, (l) => {
         if (l.panels.some((p) => p.kind === "notes")) {
```

`app/ui/panels/ChartPanel.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/panels/ChartPanel.tsx
+++ b/app/ui/panels/ChartPanel.tsx
@@ -1,10 +1,12 @@
+import { useId } from "react";
 import { useTranslation } from "react-i18next";
-import type { Varga } from "../bindings";
-import { chartDataFromChart, isVarga } from "../chart/adapter";
+import type { OpenedClient, Varga } from "../bindings";
+import { chartDataFromChart, isVarga, VARGA_NAMES, VARGAS } from "../chart/adapter";
 import { ChartSvg } from "../chart/ChartSvg";
-import { isChartStyle } from "../chart/types";
+import { CHART_STYLES, isChartStyle } from "../chart/types";
 import { useHighlight } from "../highlight/store";
 import { useSession } from "../session/store";
+import { useLayoutStore } from "../view/store";
 import { chartLabels } from "./labels";
 import "./panels.css";
 import type { PanelProps } from "./types";
@@ -13,30 +15,77 @@ export function chartVarga(settings: Record<string, unknown>): Varga {
   return isVarga(settings.varga) ? settings.varga : "D1";
 }
 
-/** Harita paneli: açık haritanın `settings.varga` vargası, `settings.style` biçiminde. */
+/** Doğum saati bilinmiyor: lagna, evler ve saate bağlı değerler gösterilmez (spec §7). */
+export function timeUnknown(opened: OpenedClient): boolean {
+  return opened.chart_record.birth.time_certainty === "Unknown";
+}
+
+/**
+ * Harita paneli: açık haritanın `settings.varga` vargası, `settings.style` biçiminde.
+ * Varga ve biçim seçimi panel ayarına yazılır; düzenle birlikte kaydedilir.
+ */
 export function ChartPanel({ panel }: PanelProps) {
   const { t } = useTranslation();
   const { opened } = useSession();
+  const { setPanelSettings } = useLayoutStore();
   const highlight = useHighlight();
+  const id = useId();
   const varga = chartVarga(panel.settings);
   const style = isChartStyle(panel.settings.style) ? panel.settings.style : "north";
 
   if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
-  const data = chartDataFromChart(opened.chart, varga, { showLagna: true });
-  if (!data) return <p className="panel-message">{t("chart.vargaMissing", { varga })}</p>;
+  const unknown = timeUnknown(opened);
+  const data = chartDataFromChart(opened.chart, varga, { showLagna: !unknown });
   return (
     <div className="chart-panel">
-      <div className="chart-panel-figure">
-        <ChartSvg
-          data={data}
-          style={style}
-          labels={chartLabels(t, varga)}
-          highlight={highlight.active}
-          pressed={highlight.pinned}
-          onHover={highlight.hover}
-          onPin={highlight.togglePin}
-        />
+      <div className="chart-panel-toolbar">
+        <label htmlFor={`${id}-varga`}>{t("chart.varga")}</label>
+        <select
+          id={`${id}-varga`}
+          value={varga}
+          onChange={(e) => isVarga(e.target.value) && setPanelSettings(panel.id, { varga: e.target.value })}
+        >
+          {VARGAS.map((v) => (
+            <option key={v} value={v}>
+              {v} · {VARGA_NAMES[v]}
+            </option>
+          ))}
+        </select>
+        <label htmlFor={`${id}-style`}>{t("chart.style")}</label>
+        <select
+          id={`${id}-style`}
+          value={style}
+          onChange={(e) => isChartStyle(e.target.value) && setPanelSettings(panel.id, { style: e.target.value })}
+        >
+          {CHART_STYLES.map((s) => (
+            <option key={s} value={s}>
+              {t(`chartStyle.${s}`)}
+            </option>
+          ))}
+        </select>
       </div>
+      {unknown && (
+        <p className="panel-notice" role="note">
+          {t("chart.timeUnknown")}
+          {/* Bölüm haritalarında her yerleşim saate bağlıdır (D9'da Ay, D30–D60'ta hepsi). */}
+          {varga !== "D1" && <> {t("chart.timeUnknownVarga")}</>}
+        </p>
+      )}
+      {data ? (
+        <div className="chart-panel-figure">
+          <ChartSvg
+            data={data}
+            style={style}
+            labels={chartLabels(t, varga)}
+            highlight={highlight.active}
+            pressed={highlight.pinned}
+            onHover={highlight.hover}
+            onPin={highlight.togglePin}
+          />
+        </div>
+      ) : (
+        <p className="panel-message">{t("chart.vargaMissing", { varga })}</p>
+      )}
     </div>
   );
 }
```

`app/ui/panels/panels.css` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/panels/panels.css
+++ b/app/ui/panels/panels.css
@@ -59,3 +59,10 @@
 .panel-error p {
   margin: 0 0 8px;
 }
+.chart-panel-toolbar {
+  display: flex;
+  align-items: center;
+  flex-wrap: wrap;
+  gap: 6px;
+  color: var(--text-2);
+}
```

`app/ui/view/store.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/view/store.tsx
+++ b/app/ui/view/store.tsx
@@ -25,6 +25,8 @@ export interface LayoutStore {
   resetBuiltin(id: string): void;
   updatePanels(panels: Panel[]): void;
   toggleNotes(): void;
+  /** Etkin düzendeki bir panelin ayarlarına `patch`'i ekler (varga, biçim, sütunlar). */
+  setPanelSettings(panelId: string, patch: Record<string, unknown>): void;
 }
 
 export interface ViewStore {
@@ -127,6 +129,7 @@ export function ViewStateProvider({
       resetBuiltin: (id) => layoutDispatch({ type: "resetBuiltin", id }),
       updatePanels: (panels) => layoutDispatch({ type: "updatePanels", panels }),
       toggleNotes: () => layoutDispatch({ type: "toggleNotes", panelId: newId("notes") }),
+      setPanelSettings: (panelId, patch) => layoutDispatch({ type: "setPanelSettings", panelId, patch }),
     };
     const view: ViewStore = {
       model: state,
```


- [ ] **Step 4: SVG anlık görüntülerini üret ve incele**

```bash
pnpm --dir app exec vitest run ui/panels/ChartPanel.test.tsx
ls app/ui/chart/__snapshots__/
```
Expected: ilk çalıştırma `delhi-d1-north.svg`, `delhi-d1-south.svg`, `delhi-d1-circle.svg`'yi yazar (`toMatchFileSnapshot`). Dosyaları tarayıcıda açıp üç stilde lagnanın İkizler'de, Güneş'in Yay'da, Satürn'ün `℞` ile Koç'ta olduğunu gözle doğrula; sonraki çalıştırmalar eşitlik denetler.

- [ ] **Step 5: Testlerin geçtiğini gör**

```bash
pnpm --dir app exec vitest run ui/panels/ChartPanel.test.tsx ui/chart ui/layout
pnpm --dir app test
pnpm --dir app build
```
Expected: tam takımda `Test Files  27 passed (27)`, `Tests  237 passed (237)`; `build` hatasız (`tsc --noEmit` çıktısız, `vite build` `✓ built`).

- [ ] **Step 6: Commit**

```bash
git add -A app/ui/chart/ChartSvg.test.tsx app/ui/chart/ChartSvg.tsx app/ui/chart/__snapshots__/delhi-d1-circle.svg app/ui/chart/__snapshots__/delhi-d1-north.svg app/ui/chart/__snapshots__/delhi-d1-south.svg app/ui/chart/adapter.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts app/ui/i18n/tr.ts app/ui/layout/reducer.test.ts app/ui/layout/reducer.ts app/ui/panels/ChartPanel.test.tsx app/ui/panels/ChartPanel.tsx app/ui/panels/panels.css app/ui/view/store.tsx
git diff --cached -U0 -- . ':!app/pnpm-lock.yaml' ':!*.png' ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git commit -m "feat(ui): chart panel varga and style selectors, unknown-time rendering and SVG snapshots

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: Graha tablosu: nakshatra, dignity, ev, isteğe bağlı sütunlar, lagna satırı ve sütun seçici

Tam graha tablosu (D15): sütunlar `settings.columns`'tan gelir, `<details>` içindeki onay kutularıyla seçilir. Lagna satırı ilk satırdır; saat bilinmiyorsa lagna satırı ve ev sütunu gizlenir, Ay'ın saate bağlı değerleri `≈` ve gizli "yaklaşık" metniyle işaretlenir (D11). Nakshatra tam adı Sanskritçe çevriyazıdır ve çevrilmez. Değerlerin hepsi `Chart.grahas`'tan gelir; arayüz yalnızca biçimler.

**Files:**
- Modify: `app/ui/i18n/en.ts`
- Modify: `app/ui/i18n/fr.ts`
- Modify: `app/ui/i18n/tr.ts`
- Create: `app/ui/panels/GrahaTablePanel.test.tsx`
- Modify: `app/ui/panels/GrahaTablePanel.tsx`
- Modify: `app/ui/panels/format.test.ts`
- Modify: `app/ui/panels/format.ts`
- Create: `app/ui/panels/grahaColumns.test.ts`
- Create: `app/ui/panels/grahaColumns.ts`
- Modify: `app/ui/panels/labels.ts`
- Modify: `app/ui/panels/panels.css`
- Modify: `app/ui/panels/panels.test.tsx`

**Interfaces:**
- Consumes: `useSession` (Task 5), `setPanelSettings` (Task 6), `formatDegree`, `signAbbreviations`, `grahaAbbreviations` (Plan 04).
- Produces: `panels/grahaColumns.ts`: `GRAHA_COLUMNS`, `GrahaColumn`, `DEFAULT_GRAHA_COLUMNS`, `grahaColumns(settings)`, `toggleColumn(columns, column)`; `format.ts`: `formatSignedDegree(deg)`, `formatSpeed(degPerDay)`; `labels.ts`: `NAKSHATRA_NAMES`, `nakshatraName(n)`; i18n `table.*`, `dignity.*`.

- [ ] **Step 1: Önce testleri yaz**



`app/ui/panels/GrahaTablePanel.test.tsx` oluştur:

```tsx
import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { GrahaTablePanel } from "./GrahaTablePanel";

function renderTable(settings: Record<string, unknown> = {}, opened = fixture().opened) {
  const panel: Panel = { id: "t", kind: "graha_table", x: 0, y: 0, w: 6, h: 8, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<GrahaTablePanel panel={panel} />, { layout, opened });
}

const headers = () => screen.getAllByRole("columnheader").map((h) => h.textContent);
const row = (name: string) => screen.getByRole("button", { name }).closest("tr")!;

test("default columns and a lagna row, values straight from the chart", () => {
  renderTable();
  expect(headers()).toEqual(["Graha", "Burç", "Derece", "Nakshatra", "Pada", "Retro", "Yanık", "Durum", "Ev"]);
  const lagna = screen.getByRole("rowheader", { name: "Lagna" }).closest("tr")!;
  expect([...lagna.querySelectorAll("td")].map((c) => c.textContent)).toEqual(["İki", "16°20′", "Ardra", "3", "", "", "", "1"]);
  const moon = row("Ay");
  expect([...moon.querySelectorAll("td")].map((c) => c.textContent)).toEqual(["Ter", "19°28′", "Swati", "4", "", "", "Dost", "5"]);
  expect(row("Satürn")).toHaveTextContent("℞retro");
  expect(row("Satürn")).toHaveTextContent("Düşük");
  expect(row("Rahu").querySelectorAll("td")[6]).toHaveTextContent("—");
});

test("optional columns show longitude, signed latitude and speed", () => {
  renderTable({ columns: ["lon", "lat", "speed"] });
  expect(headers()).toEqual(["Graha", "Boylam", "Enlem", "Hız (°/gün)"]);
  expect([...row("Ay").querySelectorAll("td")].map((c) => c.textContent)).toEqual(["199°28′", "+5°10′", "+12.021"]);
});

test("the column chooser writes settings.columns in canonical order and keeps one column", async () => {
  const { persistence } = renderTable({ columns: ["house"] });
  const summary = screen.getByText("Sütunlar", { selector: "summary" });
  const chooser = summary.closest("details")!;
  fireEvent.click(summary);
  const house = within(chooser).getByRole("checkbox", { name: "Ev" });
  expect(house).toBeChecked();
  expect(house).toBeDisabled();
  fireEvent.click(within(chooser).getByRole("checkbox", { name: "Boylam" }));
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  const saved = persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!;
  expect(saved.panels[0].settings.columns).toEqual(["lon", "house"]);
});

test("unknown birth time hides the lagna row and house column and marks the Moon approximate", () => {
  renderTable({}, unknownTimeOpened());
  expect(screen.queryByRole("rowheader", { name: "Lagna" })).toBeNull();
  expect(headers()).not.toContain("Ev");
  const moon = row("Ay");
  expect(moon.querySelectorAll("td")[2]).toHaveTextContent("≈yaklaşık Swati");
  expect(moon.querySelector('[title="yaklaşık"]')).not.toBeNull();
  expect(row("Güneş").querySelector('[title="yaklaşık"]')).toBeNull();
});
```

`app/ui/panels/format.test.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/panels/format.test.ts
+++ b/app/ui/panels/format.test.ts
@@ -1,4 +1,4 @@
-import { formatDegree } from "./format";
+import { formatDegree, formatSignedDegree, formatSpeed } from "./format";
 
 test("formatDegree writes degrees and zero-padded arc minutes, rounding down", () => {
   expect(formatDegree(0)).toBe("0°00′");
@@ -7,3 +7,10 @@ test("formatDegree writes degrees and zero-padded arc minutes, rounding down", (
   expect(formatDegree(29.9999)).toBe("29°59′");
   expect(formatDegree(3.0166666666666666)).toBe("3°01′");
 });
+
+test("signed degrees and speeds carry an explicit sign and a true minus", () => {
+  expect(formatSignedDegree(5.170740538571619)).toBe("+5°10′");
+  expect(formatSignedDegree(-0.5)).toBe("−0°30′");
+  expect(formatSpeed(12.021263924093429)).toBe("+12.021");
+  expect(formatSpeed(-0.05)).toBe("−0.050");
+});
```

`app/ui/panels/grahaColumns.test.ts` oluştur:

```ts
import { DEFAULT_GRAHA_COLUMNS, grahaColumns, toggleColumn } from "./grahaColumns";

test("missing, empty or garbage settings fall back to the default columns", () => {
  expect(grahaColumns({})).toEqual(DEFAULT_GRAHA_COLUMNS);
  expect(grahaColumns({ columns: [] })).toEqual(DEFAULT_GRAHA_COLUMNS);
  expect(grahaColumns({ columns: "sign" })).toEqual(DEFAULT_GRAHA_COLUMNS);
  expect(grahaColumns({ columns: ["bogus", 3] })).toEqual(DEFAULT_GRAHA_COLUMNS);
});

test("known columns are kept in canonical order without duplicates", () => {
  expect(grahaColumns({ columns: ["house", "sign", "x", "sign", "lon"] })).toEqual(["sign", "lon", "house"]);
});

test("toggling adds in canonical order, removes, and never empties the table", () => {
  expect(toggleColumn(["sign", "house"], "lon")).toEqual(["sign", "lon", "house"]);
  expect(toggleColumn(["sign", "house"], "sign")).toEqual(["house"]);
  expect(toggleColumn(["house"], "house")).toEqual(["house"]);
});
```

`app/ui/panels/panels.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -85,7 +85,7 @@ test("graha table lists body, sign, degree with minutes and the retro marker", (
   const saturn = screen.getByRole("button", { name: "Satürn" }).closest("tr")!;
   expect(saturn).toHaveTextContent("Koç");
   expect(saturn).toHaveTextContent("℞");
-  expect(screen.getAllByRole("row")).toHaveLength(10);
+  expect(screen.getAllByRole("row")).toHaveLength(11); // başlık + lagna + 9 graha
 });
 
 test("a null degree is shown as a dash", () => {
```


- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/panels
```
Expected: FAIL — `Failed to resolve import "./grahaColumns"`, `formatSpeed is not a function`, "Sütunlar" düğmesi bulunamıyor.

- [ ] **Step 3: Uygulamayı yaz**



`app/ui/i18n/en.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -62,7 +62,24 @@ export const en: Messages = {
     announce: "Column {{col}}, row {{row}}, {{w}}×{{h}}",
     noChart: "No chart is open.",
   },
-  table: { graha: "Graha", sign: "Sign", degree: "Degree", retro: "Retro", empty: "No grahas to show." },
+  table: {
+    graha: "Graha",
+    sign: "Sign",
+    degree: "Degree",
+    retro: "Retro",
+    empty: "No grahas to show.",
+    lon: "Longitude",
+    lat: "Latitude",
+    speed: "Speed (°/day)",
+    nakshatra: "Nakshatra",
+    pada: "Pada",
+    combust: "Combust",
+    dignity: "Dignity",
+    house: "House",
+    columns: "Columns",
+    lagnaRow: "Lagna",
+    approx: "approximate",
+  },
   errors: {
     ipc: "Could not reach the application core.",
     calc: "The calculation could not finish ({{detail}}).",
@@ -165,4 +182,15 @@ export const en: Messages = {
     south: "South",
     circle: "Circle",
   },
+  dignity: {
+    Exalted: "Exalted",
+    Moolatrikona: "Moolatrikona",
+    OwnSign: "Own sign",
+    GreatFriend: "Great friend",
+    Friend: "Friend",
+    Neutral: "Neutral",
+    Enemy: "Enemy",
+    GreatEnemy: "Great enemy",
+    Debilitated: "Debilitated",
+  },
 };
```

`app/ui/i18n/fr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -62,7 +62,24 @@ export const fr: Messages = {
     announce: "Colonne {{col}}, ligne {{row}}, {{w}}×{{h}}",
     noChart: "Aucune carte ouverte.",
   },
-  table: { graha: "Graha", sign: "Signe", degree: "Degré", retro: "Rétro", empty: "Aucun graha à afficher." },
+  table: {
+    graha: "Graha",
+    sign: "Signe",
+    degree: "Degré",
+    retro: "Rétro",
+    empty: "Aucun graha à afficher.",
+    lon: "Longitude",
+    lat: "Latitude",
+    speed: "Vitesse (°/jour)",
+    nakshatra: "Nakshatra",
+    pada: "Pada",
+    combust: "Combuste",
+    dignity: "Dignité",
+    house: "Maison",
+    columns: "Colonnes",
+    lagnaRow: "Lagna",
+    approx: "approximatif",
+  },
   errors: {
     ipc: "Impossible de joindre le cœur de l’application.",
     calc: "Le calcul n’a pas pu aboutir ({{detail}}).",
@@ -165,4 +182,15 @@ export const fr: Messages = {
     south: "Sud",
     circle: "Cercle",
   },
+  dignity: {
+    Exalted: "Exalté",
+    Moolatrikona: "Moolatrikona",
+    OwnSign: "Domicile",
+    GreatFriend: "Grand ami",
+    Friend: "Ami",
+    Neutral: "Neutre",
+    Enemy: "Ennemi",
+    GreatEnemy: "Grand ennemi",
+    Debilitated: "Débilité",
+  },
 };
```

`app/ui/i18n/tr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -61,7 +61,24 @@ export const tr = {
     announce: "Sütun {{col}}, satır {{row}}, {{w}}×{{h}}",
     noChart: "Açık harita yok.",
   },
-  table: { graha: "Graha", sign: "Burç", degree: "Derece", retro: "Retro", empty: "Gösterilecek graha yok." },
+  table: {
+    graha: "Graha",
+    sign: "Burç",
+    degree: "Derece",
+    retro: "Retro",
+    empty: "Gösterilecek graha yok.",
+    lon: "Boylam",
+    lat: "Enlem",
+    speed: "Hız (°/gün)",
+    nakshatra: "Nakshatra",
+    pada: "Pada",
+    combust: "Yanık",
+    dignity: "Durum",
+    house: "Ev",
+    columns: "Sütunlar",
+    lagnaRow: "Lagna",
+    approx: "yaklaşık",
+  },
   errors: {
     ipc: "Uygulama çekirdeğine ulaşılamadı.",
     calc: "Hesap tamamlanamadı ({{detail}}).",
@@ -164,6 +181,17 @@ export const tr = {
     south: "Güney",
     circle: "Daire",
   },
+  dignity: {
+    Exalted: "Yüce",
+    Moolatrikona: "Mulatrikona",
+    OwnSign: "Kendi burcu",
+    GreatFriend: "Çok dost",
+    Friend: "Dost",
+    Neutral: "Nötr",
+    Enemy: "Düşman",
+    GreatEnemy: "Çok düşman",
+    Debilitated: "Düşük",
+  },
 };
 
 type Strings<T> = { [K in keyof T]: T[K] extends string ? string : Strings<T[K]> };
```

`app/ui/panels/GrahaTablePanel.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/panels/GrahaTablePanel.tsx
+++ b/app/ui/panels/GrahaTablePanel.tsx
@@ -1,63 +1,191 @@
+import type { TFunction } from "i18next";
+import type { ReactNode } from "react";
 import { useTranslation } from "react-i18next";
+import type { GrahaPosition, Lagna } from "../bindings";
 import { signIndex } from "../chart/adapter";
 import { useHighlight } from "../highlight/store";
 import { sameTarget, type HighlightTarget } from "../highlight/target";
 import { useSession } from "../session/store";
-import { formatDegree } from "./format";
+import { useLayoutStore } from "../view/store";
+import { timeUnknown } from "./ChartPanel";
+import { formatDegree, formatSignedDegree, formatSpeed } from "./format";
+import { GRAHA_COLUMNS, grahaColumns, toggleColumn, type GrahaColumn } from "./grahaColumns";
+import { nakshatraName } from "./labels";
 import "./panels.css";
 import type { PanelProps } from "./types";
 
-/** Graha tablosu: graha, burç, burç içi derece, retro. Değerler `Chart.grahas`'tan gelir. */
-export function GrahaTablePanel(_: PanelProps) {
+const NUMERIC: ReadonlySet<GrahaColumn> = new Set(["degree", "lon", "lat", "speed", "pada", "house"]);
+const dash = "—";
+
+function deg(v: number | null): string {
+  return v === null ? dash : formatDegree(v);
+}
+
+/** Yaklaşık değer: "≈" işareti ve ekran okuyucu için metin (renk tek başına taşımaz). */
+function Approx({ on, label, children }: { on: boolean; label: string; children: ReactNode }) {
+  if (!on) return <>{children}</>;
+  return (
+    <span title={label}>
+      <span aria-hidden="true">≈</span>
+      <span className="visually-hidden">{label} </span>
+      {children}
+    </span>
+  );
+}
+
+function grahaCell(t: TFunction, g: GrahaPosition, column: GrahaColumn, approx: boolean): ReactNode {
+  const a = (node: ReactNode) => (
+    <Approx on={approx} label={t("table.approx")}>
+      {node}
+    </Approx>
+  );
+  switch (column) {
+    case "sign":
+      return a(t(`sign.${signIndex(g.sign)}`));
+    case "degree":
+      return a(deg(g.deg_in_sign));
+    case "lon":
+      return a(deg(g.lon));
+    case "lat":
+      return g.lat === null ? dash : formatSignedDegree(g.lat);
+    case "speed":
+      return g.speed === null ? dash : formatSpeed(g.speed);
+    case "retro":
+      return g.retrograde ? (
+        <>
+          <span aria-hidden="true">℞</span>
+          <span className="visually-hidden">{t("chart.retro")}</span>
+        </>
+      ) : (
+        ""
+      );
+    case "nakshatra":
+      return a(nakshatraName(g.nakshatra));
+    case "pada":
+      return a(String(g.pada));
+    case "combust":
+      return g.combust ? t("chart.combust") : "";
+    case "dignity":
+      return g.dignity === null ? dash : t(`dignity.${g.dignity}`);
+    case "house":
+      return String(g.house);
+  }
+}
+
+function lagnaCell(t: TFunction, l: Lagna, column: GrahaColumn): ReactNode {
+  switch (column) {
+    case "sign":
+      return t(`sign.${signIndex(l.sign)}`);
+    case "degree":
+      return deg(l.deg_in_sign);
+    case "lon":
+      return deg(l.lon);
+    case "nakshatra":
+      return nakshatraName(l.nakshatra);
+    case "pada":
+      return String(l.pada);
+    case "house":
+      return "1";
+    default:
+      return "";
+  }
+}
+
+/**
+ * Graha tablosu (spec §3.3): sütunlar `settings.columns`'tan, değerler `Chart.grahas` ve
+ * `Chart.lagna`'dan. Saat bilinmiyorsa lagna satırı ve ev sütunu gizlenir, Ay'ın saate
+ * bağlı değerleri "≈" ile işaretlenir.
+ */
+export function GrahaTablePanel({ panel }: PanelProps) {
   const { t } = useTranslation();
   const { opened } = useSession();
+  const { setPanelSettings } = useLayoutStore();
   const highlight = useHighlight();
 
   if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
   if (opened.chart.grahas.length === 0) return <p className="panel-message">{t("table.empty")}</p>;
 
+  const unknown = timeUnknown(opened);
+  const chosen = grahaColumns(panel.settings);
+  const columns = unknown ? chosen.filter((c) => c !== "house") : chosen;
+  const setColumns = (next: GrahaColumn[]) => setPanelSettings(panel.id, { columns: next });
+
   return (
-    <table className="graha-table">
-      <thead>
-        <tr>
-          <th scope="col">{t("table.graha")}</th>
-          <th scope="col">{t("table.sign")}</th>
-          <th scope="col">{t("table.degree")}</th>
-          <th scope="col">{t("table.retro")}</th>
-        </tr>
-      </thead>
-      <tbody>
-        {opened.chart.grahas.map((g) => {
-          const target: HighlightTarget = { type: "graha", id: g.graha };
-          const on = sameTarget(highlight.active, target);
-          const pinned = sameTarget(highlight.pinned, target);
-          return (
-            <tr
-              key={g.graha}
-              data-graha={g.graha}
-              className={on ? "is-highlighted" : undefined}
-              onMouseEnter={() => highlight.hover(target)}
-              onMouseLeave={() => highlight.hover(null)}
-            >
-              <th scope="row">
-                <button
-                  type="button"
-                  className="link-button"
-                  aria-pressed={pinned}
-                  onClick={() => highlight.togglePin(target)}
-                  onFocus={() => highlight.hover(target)}
-                  onBlur={() => highlight.hover(null)}
-                >
-                  {t(`grahaName.${g.graha}`)}
-                </button>
+    <div className="graha-table-panel">
+      <details className="column-chooser">
+        <summary>{t("table.columns")}</summary>
+        <fieldset>
+          <legend className="visually-hidden">{t("table.columns")}</legend>
+          {GRAHA_COLUMNS.map((c) => (
+            <label key={c}>
+              <input
+                type="checkbox"
+                checked={chosen.includes(c)}
+                disabled={chosen.length === 1 && chosen.includes(c)}
+                onChange={() => setColumns(toggleColumn(chosen, c))}
+              />
+              {t(`table.${c}`)}
+            </label>
+          ))}
+        </fieldset>
+      </details>
+      <table className="graha-table">
+        <thead>
+          <tr>
+            <th scope="col">{t("table.graha")}</th>
+            {columns.map((c) => (
+              <th key={c} scope="col" className={NUMERIC.has(c) ? "num" : undefined}>
+                {t(`table.${c}`)}
               </th>
-              <td>{t(`sign.${signIndex(g.sign)}`)}</td>
-              <td className="num">{g.deg_in_sign === null ? "—" : formatDegree(g.deg_in_sign)}</td>
-              <td>{g.retrograde ? "℞" : ""}</td>
+            ))}
+          </tr>
+        </thead>
+        <tbody>
+          {!unknown && (
+            <tr data-lagna="">
+              <th scope="row">{t("table.lagnaRow")}</th>
+              {columns.map((c) => (
+                <td key={c} className={NUMERIC.has(c) ? "num" : undefined}>
+                  {lagnaCell(t, opened.chart.lagna, c)}
+                </td>
+              ))}
             </tr>
-          );
-        })}
-      </tbody>
-    </table>
+          )}
+          {opened.chart.grahas.map((g) => {
+            const target: HighlightTarget = { type: "graha", id: g.graha };
+            const on = sameTarget(highlight.active, target);
+            const pinned = sameTarget(highlight.pinned, target);
+            const approx = unknown && g.graha === "Moon";
+            return (
+              <tr
+                key={g.graha}
+                data-graha={g.graha}
+                className={on ? "is-highlighted" : undefined}
+                onMouseEnter={() => highlight.hover(target)}
+                onMouseLeave={() => highlight.hover(null)}
+              >
+                <th scope="row">
+                  <button
+                    type="button"
+                    className="link-button"
+                    aria-pressed={pinned}
+                    onClick={() => highlight.togglePin(target)}
+                    onFocus={() => highlight.hover(target)}
+                    onBlur={() => highlight.hover(null)}
+                  >
+                    {t(`grahaName.${g.graha}`)}
+                  </button>
+                </th>
+                {columns.map((c) => (
+                  <td key={c} className={NUMERIC.has(c) ? "num" : undefined}>
+                    {grahaCell(t, g, c, approx)}
+                  </td>
+                ))}
+              </tr>
+            );
+          })}
+        </tbody>
+      </table>
+    </div>
   );
 }
```

`app/ui/panels/format.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/panels/format.ts
+++ b/app/ui/panels/format.ts
@@ -5,3 +5,13 @@ export function formatDegree(deg: number): string {
   const m = totalMinutes % 60;
   return `${d}°${String(m).padStart(2, "0")}′`;
 }
+
+/** İşaretli derece (enlem): 5.17 → "+5°10′", -0.5 → "−0°30′". */
+export function formatSignedDegree(deg: number): string {
+  return `${deg < 0 ? "−" : "+"}${formatDegree(Math.abs(deg))}`;
+}
+
+/** Günlük hız, derece/gün, üç ondalık: 12.0212 → "+12.021", -0.05 → "−0.050". */
+export function formatSpeed(degPerDay: number): string {
+  return `${degPerDay < 0 ? "−" : "+"}${Math.abs(degPerDay).toFixed(3)}`;
+}
```

`app/ui/panels/grahaColumns.ts` oluştur:

```ts
// Graha tablosu sütunları (spec §3.3 `graha_table.settings.columns`). Bilinmeyen sütun
// kimlikleri okunurken atlanır; ayar hiç yoksa ya da boşsa varsayılan sütunlar kullanılır.
export const GRAHA_COLUMNS = ["sign", "degree", "lon", "lat", "speed", "retro", "nakshatra", "pada", "combust", "dignity", "house"] as const;
export type GrahaColumn = (typeof GRAHA_COLUMNS)[number];

export const DEFAULT_GRAHA_COLUMNS: readonly GrahaColumn[] = ["sign", "degree", "nakshatra", "pada", "retro", "combust", "dignity", "house"];

function isGrahaColumn(v: unknown): v is GrahaColumn {
  return typeof v === "string" && (GRAHA_COLUMNS as readonly string[]).includes(v);
}

/** Ayardaki sütunlar, `GRAHA_COLUMNS` sırasında ve tekrarsız. */
export function grahaColumns(settings: Record<string, unknown>): GrahaColumn[] {
  const raw = settings.columns;
  if (!Array.isArray(raw)) return [...DEFAULT_GRAHA_COLUMNS];
  const chosen = new Set(raw.filter(isGrahaColumn));
  return chosen.size === 0 ? [...DEFAULT_GRAHA_COLUMNS] : GRAHA_COLUMNS.filter((c) => chosen.has(c));
}

/** Sütunu aç/kapat; son sütun kapatılamaz (boş tablo olmaz). */
export function toggleColumn(columns: readonly GrahaColumn[], column: GrahaColumn): GrahaColumn[] {
  if (columns.includes(column)) return columns.length === 1 ? [...columns] : columns.filter((c) => c !== column);
  return GRAHA_COLUMNS.filter((c) => c === column || columns.includes(c));
}
```

`app/ui/panels/labels.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/panels/labels.ts
+++ b/app/ui/panels/labels.ts
@@ -21,3 +21,39 @@ export function chartLabels(t: TFunction, varga: string): ChartLabels {
     vargottama: t("chart.vargottama"),
   };
 }
+
+/** Nakshatra adları, 1 = Ashwini … 27 = Revati (Sanskritçe terim; dile göre değişmez). */
+export const NAKSHATRA_NAMES = [
+  "Ashwini",
+  "Bharani",
+  "Krittika",
+  "Rohini",
+  "Mrigashira",
+  "Ardra",
+  "Punarvasu",
+  "Pushya",
+  "Ashlesha",
+  "Magha",
+  "Purva Phalguni",
+  "Uttara Phalguni",
+  "Hasta",
+  "Chitra",
+  "Swati",
+  "Vishakha",
+  "Anuradha",
+  "Jyeshtha",
+  "Mula",
+  "Purva Ashadha",
+  "Uttara Ashadha",
+  "Shravana",
+  "Dhanishta",
+  "Shatabhisha",
+  "Purva Bhadrapada",
+  "Uttara Bhadrapada",
+  "Revati",
+] as const;
+
+/** 1 tabanlı nakshatra numarasının adı; aralık dışıysa numaranın kendisi. */
+export function nakshatraName(n: number): string {
+  return NAKSHATRA_NAMES[n - 1] ?? String(n);
+}
```

`app/ui/panels/panels.css` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/panels/panels.css
+++ b/app/ui/panels/panels.css
@@ -66,3 +66,24 @@
   gap: 6px;
   color: var(--text-2);
 }
+.graha-table-panel {
+  display: flex;
+  flex-direction: column;
+  gap: 4px;
+}
+.graha-table th.num {
+  text-align: right;
+}
+.column-chooser fieldset {
+  display: flex;
+  flex-wrap: wrap;
+  gap: 4px 12px;
+  margin: 4px 0;
+  padding: 4px;
+  border: 1px solid var(--line);
+}
+.column-chooser label {
+  display: inline-flex;
+  align-items: center;
+  gap: 4px;
+}
```


- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app exec vitest run ui/panels
pnpm --dir app test
pnpm --dir app build
```
Expected: tam takımda `Test Files  29 passed (29)`, `Tests  245 passed (245)`; `build` hatasız (`tsc --noEmit` çıktısız, `vite build` `✓ built`).

- [ ] **Step 5: Commit**

```bash
git add -A app/ui/i18n/en.ts app/ui/i18n/fr.ts app/ui/i18n/tr.ts app/ui/panels/GrahaTablePanel.test.tsx app/ui/panels/GrahaTablePanel.tsx app/ui/panels/format.test.ts app/ui/panels/format.ts app/ui/panels/grahaColumns.test.ts app/ui/panels/grahaColumns.ts app/ui/panels/labels.ts app/ui/panels/panels.css app/ui/panels/panels.test.tsx
git diff --cached -U0 -- . ':!app/pnpm-lock.yaml' ':!*.png' ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git commit -m "feat(ui): graha table with nakshatra, dignity, house and optional columns, a lagna row and a column chooser

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: Vimshottari dasha ağacı: kayıtlı tek açık yol, tembel 4–5. seviye, ağaç klavyesi

Dasha ağacı (D10, D13): 1–3. seviye haritayla gelir, 4–5. seviye açıldıkça `dasha_children` ile yüklenir ve harita ve tohum başına önbelleklenir: `update_chart` aynı kimlikle yeni tohum döndürür, eski doğum anının alt dönemleri ve geç gelen yanıtları atılır. Tek açık yol `ViewState.dasha_expanded_path`'tir; bir düğümü açmak kardeşlerini kapatır. WAI-ARIA ağaç deseni: tek sekme durağı, ↑/↓, →/← (aç/kapa, ebeveyne git), Home/End, Enter. Yanıt yalnızca istenen yolun bir alt seviyesiyse kabul edilir (sonsuz özyinelemeye karşı). Saat bilinmiyorsa "tarihler yaklaşık" notu çıkar.

**Files:**
- Create: `app/ui/dasha/DashaTreePanel.test.tsx`
- Create: `app/ui/dasha/DashaTreePanel.tsx`
- Create: `app/ui/dasha/dasha.css`
- Create: `app/ui/dasha/tree.test.ts`
- Create: `app/ui/dasha/tree.ts`
- Create: `app/ui/dasha/useLazyDasha.ts`
- Modify: `app/ui/dashboard/Dashboard.test.tsx`
- Modify: `app/ui/i18n/en.ts`
- Modify: `app/ui/i18n/fr.ts`
- Modify: `app/ui/i18n/tr.ts`
- Modify: `app/ui/panels/panels.test.tsx`
- Modify: `app/ui/panels/registry.ts`

**Interfaces:**
- Consumes: `useViewStore().setDashaPath`, `model.dashaPath` (Task 4), `useSession` (Task 5), `api.dashaChildren` (Task 3).
- Produces: `dasha/tree.ts`: `childrenOf(periods, path)`, `samePath(a, b)`, `isOpen(open, path)`, `formatJdDate(jd, offsetSecs)`, `MAX_DASHA_LEVEL = 5`, `PRELOADED_LEVEL = 3`; `dasha/useLazyDasha.ts`: `LazyEntry`, `pathKey`, `lazyPaths(open)`, `cacheKeyOf(chartId, seed)`, `useLazyDasha(chartId, seed, open) -> {entries, retry}`; `DashaTreePanel` (kayıtta `dasha_tree`); i18n `dasha.*` (`dasha.approx` dahil).

- [ ] **Step 1: Önce testleri yaz**



`app/ui/dasha/DashaTreePanel.test.tsx` oluştur:

```tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("../api", () => ({ api: { dashaChildren: vi.fn() } }));

import { api } from "../api";
import type { Graha, OpenedClient } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { useSession } from "../session/store";
import { renderWithStores } from "../test/render";
import { DashaTreePanel } from "./DashaTreePanel";

const panel: Panel = { id: "d", kind: "dasha_tree", x: 0, y: 0, w: 4, h: 8, settings: {} };

function renderTree(path: Graha[] = [], opened = fixture().opened) {
  opened.view_state.dasha_expanded_path = path;
  return renderWithStores(<DashaTreePanel panel={panel} />, { opened });
}

const tree = () => screen.getByRole("tree", { name: "Dasa ağacı" });
const item = (name: RegExp) => within(tree()).getByRole("treeitem", { name });
const click = (name: RegExp) => fireEvent.click(item(name).querySelector(".dasha-row")!);
const level = (n: number) => within(tree()).queryAllByRole("treeitem").filter((i) => i.getAttribute("aria-level") === String(n));

beforeEach(() => {
  vi.mocked(api.dashaChildren).mockReset();
  vi.mocked(api.dashaChildren).mockImplementation(async () => ({ status: "ok", data: fixture().dasha_children.periods }));
});
afterEach(() => vi.useRealTimers());

test("lists the nine mahadashas with dates in the birth's offset", () => {
  renderTree();
  const mahas = level(1);
  expect(mahas).toHaveLength(9);
  expect(mahas[0]).toHaveAccessibleName("Rahu, 1982-09-19 – 2000-09-18");
  expect(mahas[0]).toHaveAttribute("aria-expanded", "false");
  expect(mahas[0]).toHaveAttribute("aria-posinset", "1");
  expect(mahas[0]).toHaveAttribute("aria-setsize", "9");
  expect(mahas[0]).toHaveAttribute("tabindex", "0");
  expect(mahas[1]).toHaveAttribute("tabindex", "-1");
});

test("one open path: opening a sibling closes the other, and the path is saved with the chart", async () => {
  const { persistence } = renderTree();
  click(/^Jüpiter, /);
  expect(level(2)).toHaveLength(9);
  click(/^Jüpiter–Satürn, /);
  expect(level(3)).toHaveLength(9);
  click(/^Satürn, /);
  expect(level(2).map((i) => i.getAttribute("aria-label")!.split(",")[0])[0]).toBe("Satürn–Satürn");
  expect(level(3)).toHaveLength(0);
  await waitFor(() => expect(persistence.saved.at(-1)?.viewState.dasha_expanded_path).toEqual(["Saturn"]));
  expect(persistence.saved.at(-1)!.chartId).toBe(fixture().opened.chart_record.id);
  click(/^Satürn, /);
  expect(level(2)).toHaveLength(0);
});

test("keyboard follows the tree pattern", () => {
  renderTree();
  const first = level(1)[0];
  act(() => first.focus());
  fireEvent.keyDown(first, { key: "ArrowDown" });
  expect(document.activeElement).toHaveAccessibleName(/^Jüpiter, /);
  fireEvent.keyDown(document.activeElement!, { key: "ArrowRight" });
  expect(item(/^Jüpiter, /)).toHaveAttribute("aria-expanded", "true");
  fireEvent.keyDown(document.activeElement!, { key: "ArrowRight" });
  expect(document.activeElement).toHaveAccessibleName(/^Jüpiter–Jüpiter, /);
  fireEvent.keyDown(document.activeElement!, { key: "ArrowLeft" });
  expect(document.activeElement).toHaveAccessibleName(/^Jüpiter, /);
  fireEvent.keyDown(document.activeElement!, { key: "ArrowLeft" });
  expect(item(/^Jüpiter, /)).toHaveAttribute("aria-expanded", "false");
  fireEvent.keyDown(document.activeElement!, { key: "End" });
  expect(document.activeElement).toHaveAccessibleName(/^Mars, /);
  fireEvent.keyDown(document.activeElement!, { key: "Enter" });
  expect(item(/^Mars, /)).toHaveAttribute("aria-expanded", "true");
  fireEvent.keyDown(document.activeElement!, { key: "Home" });
  expect(document.activeElement).toHaveAccessibleName(/^Rahu, /);
});

test("levels 4 and 5 load on demand and are cached per chart", async () => {
  const { seed } = fixture().opened.chart.dasha;
  renderTree(["Rahu", "Rahu", "Rahu"]);
  expect(await within(tree()).findAllByRole("treeitem", { name: /^Rahu–Rahu–Rahu–/ })).toHaveLength(9);
  expect(api.dashaChildren).toHaveBeenCalledTimes(1);
  expect(api.dashaChildren).toHaveBeenCalledWith(seed, ["Rahu", "Rahu", "Rahu"]);
  expect(level(4)).toHaveLength(9);
  click(/^Rahu–Rahu–Rahu, /);
  click(/^Rahu–Rahu–Rahu, /);
  expect(level(4)).toHaveLength(9);
  expect(api.dashaChildren).toHaveBeenCalledTimes(1);
});

test("editing the birth data drops lazily loaded levels", async () => {
  let open: (o: OpenedClient) => void = () => {};
  function Grab() {
    open = useSession().open;
    return null;
  }
  const opened = fixture().opened;
  opened.view_state.dasha_expanded_path = ["Rahu", "Rahu", "Rahu"];
  renderWithStores(
    <>
      <Grab />
      <DashaTreePanel panel={panel} />
    </>,
    { opened },
  );
  await waitFor(() => expect(level(4)).toHaveLength(9));
  // `update_chart`: aynı harita kimliği, yeni doğum anı (yeni tohum). Yeni yanıt bekletilir.
  vi.mocked(api.dashaChildren).mockImplementation(() => new Promise(() => {}));
  const edited = fixture().opened;
  edited.view_state.dasha_expanded_path = ["Rahu", "Rahu", "Rahu"];
  edited.chart.dasha.seed.birth_jd_ut = edited.chart.dasha.seed.birth_jd_ut! + 0.5;
  act(() => open(edited));
  expect(level(4)).toHaveLength(0);
  expect(api.dashaChildren).toHaveBeenCalledTimes(2);
  expect(api.dashaChildren).toHaveBeenLastCalledWith(edited.chart.dasha.seed, ["Rahu", "Rahu", "Rahu"]);
});

test("keyboard reaches the lazily loaded level", async () => {
  renderTree(["Rahu", "Rahu", "Rahu"]);
  await waitFor(() => expect(level(4)).toHaveLength(9));
  const l3 = item(/^Rahu–Rahu–Rahu, /);
  act(() => l3.focus());
  fireEvent.keyDown(l3, { key: "ArrowRight" });
  expect(document.activeElement).toHaveAttribute("aria-level", "4");
  fireEvent.keyDown(document.activeElement!, { key: "ArrowDown" });
  expect(document.activeElement).toHaveAccessibleName(/^Rahu–Rahu–Rahu–Jüpiter, /);
});

test("a response that is not one level below the requested path is ignored", async () => {
  // Her yola aynı 4. seviye listesini dönen bozuk arka uç: 5. seviye isteği boş görünür.
  renderTree(["Rahu", "Rahu", "Rahu", "Rahu"]);
  await waitFor(() => expect(api.dashaChildren).toHaveBeenCalledTimes(2));
  await waitFor(() => expect(level(4)).toHaveLength(9));
  expect(level(5)).toHaveLength(0);
});

test("a failed level-4 load shows an in-tree error with retry", async () => {
  vi.mocked(api.dashaChildren).mockResolvedValueOnce({ status: "error", error: { code: "invalid_dasha_input", params: { field: "path" } } });
  renderTree(["Rahu", "Rahu", "Rahu"]);
  expect(await screen.findByRole("alert")).toHaveTextContent("Dasa isteği geçersiz: path.");
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await waitFor(() => expect(level(4)).toHaveLength(9));
  expect(api.dashaChildren).toHaveBeenCalledTimes(2);
});

test("a restored path opens immediately; unknown time adds an approximate note", () => {
  renderTree(["Jupiter", "Saturn"], unknownTimeOpened());
  expect(item(/^Jüpiter–Satürn, /)).toHaveAttribute("aria-expanded", "true");
  expect(level(3)).toHaveLength(9);
  expect(screen.getByRole("note")).toHaveTextContent("dasa tarihleri yaklaşıktır (≈)");
});

test("without an open chart the panel says so", () => {
  renderWithStores(<DashaTreePanel panel={panel} />, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
});
```

`app/ui/dasha/tree.test.ts` oluştur:

```ts
import { fixture } from "../test/fixture";
import { childrenOf, formatJdDate, isOpen, samePath } from "./tree";

test("children follow the lord prefix and keep the backend order", () => {
  const { periods } = fixture().opened.chart.dasha;
  const mahas = childrenOf(periods, []);
  expect(mahas.map((p) => p.lords[0])).toEqual(["Rahu", "Jupiter", "Saturn", "Mercury", "Ketu", "Venus", "Sun", "Moon", "Mars"]);
  const antars = childrenOf(periods, ["Jupiter"]);
  expect(antars).toHaveLength(9);
  expect(antars[0].lords).toEqual(["Jupiter", "Jupiter"]);
  expect(childrenOf(periods, ["Jupiter", "Saturn"]).every((p) => p.level === 3)).toBe(true);
  expect(childrenOf(periods, ["Jupiter", "Saturn", "Mercury"])).toEqual([]);
});

test("path helpers", () => {
  expect(samePath(["Sun", "Moon"], ["Sun", "Moon"])).toBe(true);
  expect(samePath(["Sun"], ["Sun", "Moon"])).toBe(false);
  expect(isOpen(["Sun", "Moon"], ["Sun"])).toBe(true);
  expect(isOpen(["Sun", "Moon"], ["Sun", "Mars"])).toBe(false);
  expect(isOpen([], ["Sun"])).toBe(false);
});

test("dates are shown in the birth's fixed offset", () => {
  // JD 2451545.0 = 2000-01-01T12:00Z; +05:30 aynı gün, −13:00 önceki gün.
  expect(formatJdDate(2451545.0, 19_800)).toBe("2000-01-01");
  expect(formatJdDate(2451545.0, -13 * 3600)).toBe("1999-12-31");
  expect(formatJdDate(2451544.6, 0)).toBe("2000-01-01");
  expect(formatJdDate(null, 0)).toBe("—");
});
```

`app/ui/dashboard/Dashboard.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/dashboard/Dashboard.test.tsx
+++ b/app/ui/dashboard/Dashboard.test.tsx
@@ -1,6 +1,11 @@
-import { fireEvent, screen, waitFor } from "@testing-library/react";
+import { act, fireEvent, screen, waitFor } from "@testing-library/react";
 import { afterEach, vi } from "vitest";
 
+vi.mock("../api", () => ({ api: { dashaChildren: vi.fn() } }));
+
+import { api } from "../api";
+import { fixture } from "../test/fixture";
+
 import { hasOverlap } from "../layout/grid";
 import type { LayoutState, Panel } from "../layout/model";
 import { renderWithStores } from "../test/render";
@@ -31,8 +36,8 @@ afterEach(() => {
 
 const MOVE = { buttons: 1, pointerId: 1 };
 
-function setup(width = WIDTH, panels = PANELS, initial: LayoutState = state(panels)) {
-  const utils = renderWithStores(<Dashboard width={width} />, { layout: initial });
+function setup(width = WIDTH, panels = PANELS, initial: LayoutState = state(panels), opened = fixture().opened) {
+  const utils = renderWithStores(<Dashboard width={width} />, { layout: initial, opened });
   const frame = (id: string) => utils.container.querySelector<HTMLElement>(`[data-panel-id="${id}"]`)!;
   const box = (id: string) => {
     const f = frame(id);
@@ -123,6 +128,20 @@ test("narrow windows reflow to 8 and 4 columns read-only", async () => {
   expect(narrow.container.querySelectorAll('.dashboard[data-columns="4"]')).toHaveLength(1);
 });
 
+test("the header shows an inline loading indicator between 100 ms and 1 s", async () => {
+  vi.useFakeTimers();
+  let resolve: (v: Awaited<ReturnType<typeof api.dashaChildren>>) => void = () => {};
+  vi.mocked(api.dashaChildren).mockReturnValueOnce(new Promise((r) => (resolve = r)));
+  const opened = fixture().opened;
+  opened.view_state.dasha_expanded_path = ["Rahu", "Rahu", "Rahu"];
+  setup(WIDTH, [{ id: "d", kind: "dasha_tree", x: 0, y: 0, w: 4, h: 8, settings: {} }], undefined, opened);
+  expect(screen.queryByRole("status")).toBeNull();
+  act(() => vi.advanceTimersByTime(150));
+  expect(screen.getByRole("status")).toHaveTextContent("Yükleniyor");
+  await act(async () => resolve({ status: "ok", data: fixture().dasha_children.periods }));
+  expect(screen.queryByRole("status")).toBeNull();
+});
+
 // ---- Sürükleme sağlamlığı ----
 
 function listenerBalance() {
```

`app/ui/panels/panels.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -1,4 +1,5 @@
 import { act, fireEvent, render, screen } from "@testing-library/react";
+import { DashaTreePanel } from "../dasha/DashaTreePanel";
 import { setLanguage } from "../i18n";
 import { tr } from "../i18n/tr";
 import { PANEL_KINDS, type Panel } from "../layout/model";
@@ -20,11 +21,12 @@ const chartPanel = (settings: Record<string, unknown> = { varga: "D1", style: "n
 });
 const tablePanel: Panel = { id: "t", kind: "graha_table", x: 6, y: 0, w: 6, h: 8, settings: {} };
 
-test("registry covers the seven kinds; only chart and graha_table are implemented here", () => {
+test("registry covers the seven kinds; chart, graha_table and dasha_tree are implemented", () => {
   expect(Object.keys(PANEL_REGISTRY).sort()).toEqual([...PANEL_KINDS].sort());
   expect(PANEL_REGISTRY.chart.component).toBe(ChartPanel);
   expect(PANEL_REGISTRY.graha_table.component).toBe(GrahaTablePanel);
-  for (const kind of ["dasha_tree", "dasha_strip", "transit_table", "notes", "birth_info"] as const) {
+  expect(PANEL_REGISTRY.dasha_tree.component).toBe(DashaTreePanel);
+  for (const kind of ["dasha_strip", "transit_table", "notes", "birth_info"] as const) {
     expect(PANEL_REGISTRY[kind].component).toBe(UnavailablePanel);
   }
   expect(panelDefinition("yoga_list")).toMatchObject({ titleKey: "panel.unknown", component: UnavailablePanel });
@@ -119,7 +121,7 @@ test("hovering a graha in the table highlights it in the chart, click pins, Esc
 });
 
 test("unavailable and unknown panels explain themselves without an error", () => {
-  render(<UnavailablePanel panel={{ ...tablePanel, kind: "dasha_tree" }} />);
+  render(<UnavailablePanel panel={{ ...tablePanel, kind: "dasha_strip" }} />);
   expect(screen.getByText("Bu panel sonraki bir sürümde gelecek.")).toBeInTheDocument();
   render(<UnavailablePanel panel={{ ...tablePanel, kind: "yoga_list" }} />);
   expect(screen.getByText("“yoga_list” paneli bu sürümde tanınmıyor. Düzen olduğu gibi korunuyor.")).toBeInTheDocument();
```


- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/dasha ui/dashboard ui/panels
```
Expected: FAIL — `Failed to resolve import "./tree"` ve `"./DashaTreePanel"`; panoda dasha paneli hâlâ yer tutucu.

- [ ] **Step 3: Uygulamayı yaz**



`app/ui/dasha/DashaTreePanel.tsx` oluştur:

```tsx
import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import type { DashaPeriod, Graha } from "../bindings";
import { timeUnknown } from "../panels/ChartPanel";
import { PanelError, usePanelLoadPhase } from "../panels/states";
import type { PanelProps } from "../panels/types";
import { useSession } from "../session/store";
import { useViewStore } from "../view/store";
import "./dasha.css";
import { childrenOf, formatJdDate, isOpen, MAX_DASHA_LEVEL, PRELOADED_LEVEL } from "./tree";
import { pathKey, useLazyDasha, type LazyEntry } from "./useLazyDasha";

interface Visible {
  period: DashaPeriod;
  expandable: boolean;
  open: boolean;
}

/**
 * Dasha ağacı (spec §3.3, Vimshottari). Tek açık yol: bir düğümü açmak kardeşlerini kapatır;
 * yol haritanın `ViewState.dasha_expanded_path`'idir ve haritayla kaydedilir. 1–3. seviye
 * haritayla gelir, 4–5. seviye istenerek yüklenir. Klavye: WAI-ARIA ağaç deseni.
 */
export function DashaTreePanel(_: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  const { model, setDashaPath } = useViewStore();
  const open = model.dashaPath;
  const chartId = opened?.chart_record.id ?? 0;
  const seed = opened?.chart.dasha.seed ?? { birth_jd_ut: null, moon_sidereal_lon: null, year_days: null };
  const lazy = useLazyDasha(chartId, seed, opened ? open : []);
  usePanelLoadPhase(opened !== null && lazy.loading);
  const [focusKey, setFocusKey] = useState<string | null>(null);
  const items = useRef(new Map<string, HTMLLIElement>());
  const treeRef = useRef<HTMLUListElement>(null);

  // Odak klavyeyle taşındıysa yeni öğeye geçer (fare ve ilk çizim odağı çalmaz).
  useEffect(() => {
    if (focusKey === null || !treeRef.current?.contains(document.activeElement)) return;
    items.current.get(focusKey)?.focus();
  }, [focusKey]);

  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  const { periods } = opened.chart.dasha;
  const offset = opened.chart_record.resolved.offset_secs;

  // Yüklenen liste de `childrenOf` süzgecinden geçer: yalnızca bu yolun bir alt seviyesi
  // kabul edilir (yanlış bir yanıt ağacı sonsuz döngüye sokamaz).
  const childList = (path: readonly Graha[]): DashaPeriod[] | LazyEntry | undefined => {
    if (path.length < PRELOADED_LEVEL) return childrenOf(periods, path);
    const entry = lazy.entries[pathKey(path)];
    return entry?.status === "ok" ? { status: "ok", periods: childrenOf(entry.periods, path) } : entry;
  };

  /** Yüklenmiş alt dönemler; yükleniyorsa ya da hatalıysa `null`. */
  const loaded = (path: readonly Graha[]): readonly DashaPeriod[] | null => {
    const list = childList(path);
    if (Array.isArray(list)) return list;
    return list?.status === "ok" ? list.periods : null;
  };

  // Görünür öğeler, ekrandaki sırayla (klavye gezintisi için).
  const visible: Visible[] = [];
  const walk = (path: readonly Graha[]) => {
    for (const p of loaded(path) ?? []) {
      const expandable = p.has_children && p.level < MAX_DASHA_LEVEL;
      const o = expandable && isOpen(open, p.lords);
      visible.push({ period: p, expandable, open: o });
      if (o) walk(p.lords);
    }
  };
  walk([]);
  const current = focusKey ?? (visible[0] ? pathKey(visible[0].period.lords) : null);

  const toggle = (p: DashaPeriod) => setDashaPath(isOpen(open, p.lords) ? p.lords.slice(0, -1) : p.lords);

  const onKeyDown = (e: KeyboardEvent) => {
    const i = visible.findIndex((v) => pathKey(v.period.lords) === current);
    if (i < 0) return;
    const v = visible[i];
    const go = (j: number) => visible[j] && setFocusKey(pathKey(visible[j].period.lords));
    switch (e.key) {
      case "ArrowDown":
        go(i + 1);
        break;
      case "ArrowUp":
        go(i - 1);
        break;
      case "Home":
        go(0);
        break;
      case "End":
        go(visible.length - 1);
        break;
      case "ArrowRight":
        if (v.expandable && !v.open) toggle(v.period);
        else if (v.open && visible[i + 1]?.period.level === v.period.level + 1) go(i + 1);
        break;
      case "ArrowLeft":
        if (v.open) toggle(v.period);
        else if (v.period.level > 1) setFocusKey(pathKey(v.period.lords.slice(0, -1)));
        break;
      case "Enter":
      case " ":
        if (v.expandable) toggle(v.period);
        break;
      default:
        return;
    }
    e.preventDefault();
  };

  const label = (p: DashaPeriod) =>
    `${p.lords.map((g) => t(`grahaName.${g}`)).join("–")}, ${formatJdDate(p.start_jd_ut, offset)} – ${formatJdDate(p.end_jd_ut, offset)}`;

  const renderLevel = (path: readonly Graha[]) => {
    const list = childList(path);
    if (list === undefined)
      return (
        <li role="none" className="dasha-pending">
          {t("panel.loading")}
        </li>
      );
    if (!Array.isArray(list)) {
      if (list.status === "error")
        return (
          <li role="none">
            <PanelError error={list.error} onRetry={() => lazy.retry(path)} />
          </li>
        );
      return renderItems(list.periods);
    }
    return renderItems(list);
  };

  const renderItems = (list: readonly DashaPeriod[]) =>
    list.map((p, i) => {
      const key = pathKey(p.lords);
      const expandable = p.has_children && p.level < MAX_DASHA_LEVEL;
      const o = expandable && isOpen(open, p.lords);
      return (
        <li
          key={key}
          ref={(el) => {
            if (el) items.current.set(key, el);
            else items.current.delete(key);
          }}
          role="treeitem"
          aria-level={p.level}
          aria-setsize={list.length}
          aria-posinset={i + 1}
          aria-expanded={expandable ? o : undefined}
          aria-label={label(p)}
          tabIndex={key === current ? 0 : -1}
          className="dasha-item"
          onFocus={(e) => e.target === e.currentTarget && setFocusKey(key)}
        >
          {/* Tıklama satırdadır: iç içe öğelerin (ve hata düğmesinin) tıklaması üst düğümü kapatmaz. */}
          <span
            className="dasha-row"
            style={{ paddingInlineStart: `${(p.level - 1) * 14}px` }}
            onClick={() => {
              setFocusKey(key);
              if (expandable) toggle(p);
            }}
          >
            <span className="dasha-twisty" aria-hidden="true">
              {expandable ? (o ? "▾" : "▸") : ""}
            </span>
            <span className="dasha-lord">{t(`grahaName.${p.lords[p.lords.length - 1]}`)}</span>
            <span className="dasha-dates">
              {formatJdDate(p.start_jd_ut, offset)} – {formatJdDate(p.end_jd_ut, offset)}
            </span>
          </span>
          {o && <ul role="group">{renderLevel(p.lords)}</ul>}
        </li>
      );
    });

  return (
    <div className="dasha-panel">
      {timeUnknown(opened) && (
        <p className="panel-notice" role="note">
          {t("dasha.approx")}
        </p>
      )}
      <ul ref={treeRef} role="tree" aria-label={t("panel.dasha_tree")} className="dasha-tree" onKeyDown={onKeyDown}>
        {renderLevel([])}
      </ul>
    </div>
  );
}
```

`app/ui/dasha/dasha.css` oluştur:

```css
.dasha-panel {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.dasha-tree,
.dasha-tree ul {
  margin: 0;
  padding: 0;
  list-style: none;
}
.dasha-item {
  cursor: default;
}
.dasha-item:focus-visible > .dasha-row {
  outline: 2px solid var(--ink);
  outline-offset: -2px;
}
.dasha-row {
  display: flex;
  gap: 8px;
  padding: 1px 4px;
}
.dasha-item[aria-expanded="true"] > .dasha-row {
  font-weight: var(--fw-medium);
}
.dasha-twisty {
  width: 1ch;
  color: var(--text-2);
}
.dasha-lord {
  min-width: 7ch;
}
.dasha-dates {
  color: var(--text-2);
  font-variant-numeric: tabular-nums;
}
.dasha-pending {
  padding-inline-start: 24px;
  color: var(--text-2);
}
```

`app/ui/dasha/tree.ts` oluştur:

```ts
// Dasha ağacı yardımcıları. Dönemler Rust'tan hazır gelir (`Chart.dasha.periods`: 1–3.
// seviye, ön-sıralı; 4–5. seviye `dasha_children` ile). Burada hesap yoktur: yalnızca
// gruplama ve gösterim için JD → yerel tarih biçimlemesi.
import type { DashaPeriod, Graha } from "../bindings";

/** Spec §5: 1 mahadasha … 5 prana. */
export const MAX_DASHA_LEVEL = 5;
/** `Chart.dasha.periods`'un içerdiği en derin seviye; daha derini istenerek yüklenir. */
export const PRELOADED_LEVEL = 3;

function startsWith(lords: readonly Graha[], prefix: readonly Graha[]): boolean {
  return prefix.every((g, i) => lords[i] === g);
}

/** `path` döneminin doğrudan alt dönemleri (boş yol: mahadashalar), sırası korunur. */
export function childrenOf(periods: readonly DashaPeriod[], path: readonly Graha[]): DashaPeriod[] {
  return periods.filter((p) => p.level === path.length + 1 && startsWith(p.lords, path));
}

export function samePath(a: readonly Graha[], b: readonly Graha[]): boolean {
  return a.length === b.length && startsWith(a, b);
}

/** `path`, `open` yolunun bir öneki mi (o düğüm açık mı)? */
export function isOpen(open: readonly Graha[], path: readonly Graha[]): boolean {
  return path.length <= open.length && startsWith(open, path);
}

const UNIX_EPOCH_JD = 2440587.5;
const DAY_MS = 86_400_000;

/**
 * JD (UT) → doğumun sabit ofsetinde "YYYY-MM-DD" (yalnızca gösterim; spec §4'teki
 * "UI hesap yapmaz" kuralı astrolojik hesap içindir, takvim biçimlemesi değil).
 */
export function formatJdDate(jd: number | null, offsetSecs: number): string {
  if (jd === null || !Number.isFinite(jd)) return "—";
  const d = new Date((jd - UNIX_EPOCH_JD) * DAY_MS + offsetSecs * 1000);
  const y = d.getUTCFullYear();
  const year = y < 0 ? `−${String(-y).padStart(4, "0")}` : String(y).padStart(4, "0");
  return `${year}-${String(d.getUTCMonth() + 1).padStart(2, "0")}-${String(d.getUTCDate()).padStart(2, "0")}`;
}
```

`app/ui/dasha/useLazyDasha.ts` oluştur:

```ts
// 4. ve 5. seviye dasha dönemleri istenerek yüklenir (`dasha_children`). Önbellek harita ve
// tohuma bağlıdır: başka harita açılınca ya da doğum verisi düzenlenince (aynı kimlik, yeni
// tohum) sıfırlanır; eski anahtarın geç gelen yanıtı atılır.
// Hata girdisi "yeniden dene" ile silinir ve istek tekrarlanır.
import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "../api";
import type { CommandError, DashaPeriod, DashaSeed, Graha } from "../bindings";
import { MAX_DASHA_LEVEL, PRELOADED_LEVEL } from "./tree";

export type LazyEntry = { status: "ok"; periods: DashaPeriod[] } | { status: "error"; error: CommandError };

export const pathKey = (path: readonly Graha[]) => path.join(",");

/** Açık yolun istenerek yüklenen önekleri: açık 3. ve 4. seviye düğümler. */
export function lazyPaths(open: readonly Graha[]): Graha[][] {
  const out: Graha[][] = [];
  for (let n = PRELOADED_LEVEL; n <= Math.min(open.length, MAX_DASHA_LEVEL - 1); n++) out.push(open.slice(0, n));
  return out;
}

/** Önbellek anahtarı: harita kimliği ve tohum (`update_chart` kimliği korur, tohumu değiştirir). */
export const cacheKeyOf = (chartId: number, seed: DashaSeed) => `${chartId}:${seed.birth_jd_ut}:${seed.moon_sidereal_lon}:${seed.year_days}`;

export function useLazyDasha(chartId: number, seed: DashaSeed, open: readonly Graha[]) {
  const cacheKey = cacheKeyOf(chartId, seed);
  const [state, setState] = useState<{ cacheKey: string; entries: Record<string, LazyEntry> }>({ cacheKey, entries: {} });
  if (state.cacheKey !== cacheKey) setState({ cacheKey, entries: {} });
  const entries = state.cacheKey === cacheKey ? state.entries : {};
  const inflight = useRef(new Set<string>());

  const needed = lazyPaths(open).filter((p) => entries[pathKey(p)] === undefined);
  const neededKey = needed.map(pathKey).join("|");

  useEffect(() => {
    for (const p of needed) {
      const key = `${cacheKey}:${pathKey(p)}`;
      if (inflight.current.has(key)) continue;
      inflight.current.add(key);
      void api.dashaChildren(seed, p).then((r) => {
        inflight.current.delete(key);
        const entry: LazyEntry = r.status === "ok" ? { status: "ok", periods: r.data } : { status: "error", error: r.error };
        setState((s) => (s.cacheKey === cacheKey ? { cacheKey, entries: { ...s.entries, [pathKey(p)]: entry } } : s));
      });
    }
    // `needed`, `neededKey` ile; `seed`, `cacheKey` ile temsil edilir.
  }, [neededKey, cacheKey]);

  const retry = useCallback((path: readonly Graha[]) => {
    setState((s) => {
      const next = { ...s.entries };
      delete next[pathKey(path)];
      return { ...s, entries: next };
    });
  }, []);

  return { entries, retry, loading: needed.length > 0 };
}
```

`app/ui/i18n/en.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -193,4 +193,7 @@ export const en: Messages = {
     GreatEnemy: "Great enemy",
     Debilitated: "Debilitated",
   },
+  dasha: {
+    approx: "Birth time unknown: dasha dates are approximate (≈).",
+  },
 };
```

`app/ui/i18n/fr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -193,4 +193,7 @@ export const fr: Messages = {
     GreatEnemy: "Grand ennemi",
     Debilitated: "Débilité",
   },
+  dasha: {
+    approx: "Heure de naissance inconnue : les dates de dasha sont approximatives (≈).",
+  },
 };
```

`app/ui/i18n/tr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -192,6 +192,9 @@ export const tr = {
     GreatEnemy: "Çok düşman",
     Debilitated: "Düşük",
   },
+  dasha: {
+    approx: "Doğum saati bilinmiyor: dasa tarihleri yaklaşıktır (≈).",
+  },
 };
 
 type Strings<T> = { [K in keyof T]: T[K] extends string ? string : Strings<T[K]> };
```

`app/ui/panels/registry.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -1,4 +1,5 @@
 // Panel türü → bileşen (spec §3.3). Yeni panel türü yalnızca buraya eklenir.
+import { DashaTreePanel } from "../dasha/DashaTreePanel";
 import { isKnownPanelKind, type KnownPanelKind } from "../layout/model";
 import { ChartPanel, chartVarga } from "./ChartPanel";
 import { GrahaTablePanel } from "./GrahaTablePanel";
@@ -8,7 +9,7 @@ import { UnavailablePanel } from "./UnavailablePanel";
 export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   chart: { titleKey: "panel.chart", subtitle: (p) => chartVarga(p.settings), component: ChartPanel },
   graha_table: { titleKey: "panel.graha_table", component: GrahaTablePanel },
-  dasha_tree: { titleKey: "panel.dasha_tree", component: UnavailablePanel },
+  dasha_tree: { titleKey: "panel.dasha_tree", component: DashaTreePanel },
   dasha_strip: { titleKey: "panel.dasha_strip", component: UnavailablePanel },
   transit_table: { titleKey: "panel.transit_table", component: UnavailablePanel },
   notes: { titleKey: "panel.notes", component: UnavailablePanel },
```


- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app exec vitest run ui/dasha ui/dashboard ui/panels
pnpm --dir app test
pnpm --dir app build
```
Expected: tam takımda `Test Files  31 passed (31)`, `Tests  259 passed (259)`; `build` hatasız (`tsc --noEmit` çıktısız, `vite build` `✓ built`).

- [ ] **Step 5: Commit**

```bash
git add -A app/ui/dasha/DashaTreePanel.test.tsx app/ui/dasha/DashaTreePanel.tsx app/ui/dasha/dasha.css app/ui/dasha/tree.test.ts app/ui/dasha/tree.ts app/ui/dasha/useLazyDasha.ts app/ui/dashboard/Dashboard.test.tsx app/ui/i18n/en.ts app/ui/i18n/fr.ts app/ui/i18n/tr.ts app/ui/panels/panels.test.tsx app/ui/panels/registry.ts
git diff --cached -U0 -- . ':!app/pnpm-lock.yaml' ':!*.png' ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git commit -m "feat(ui): Vimshottari dasha tree with one saved open path, lazy levels 4-5 and tree keyboard pattern

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 9: Kısayol kaydı, odak tuzaklı modal pencere ve ? yardımı

Dağınık klavye dinleyicileri tek kayda toplanır (D16): `matchShortcut` bir `KeyboardEvent`'i komuta çevirir; yazı alanında, tekrar ve IME sırasında ya da modal açıkken `null` döner. `Dialog` ortak modal penceredir (odak tuzağı, Esc, kapanınca odağın geri dönüşü, `center` ya da `sheet` biçimi; ilk odak `[data-autofocus]:not([disabled])`, yoksa ilk etkin öğe: devre dışı bir alan odağı pencerenin dışında bırakmaz); `?` ile açılan `HelpDialog` listeyi `SHORTCUT_LIST`'ten çizer. Üst çubukta "Kısayollar" düğmesi vardır.

**Files:**
- Modify: `app/ui/App.test.tsx`
- Modify: `app/ui/i18n/en.ts`
- Modify: `app/ui/i18n/fr.ts`
- Modify: `app/ui/i18n/tr.ts`
- Create: `app/ui/shell/Dialog.tsx`
- Create: `app/ui/shell/HelpDialog.tsx`
- Modify: `app/ui/shell/Shell.tsx`
- Modify: `app/ui/shell/TopBar.tsx`
- Modify: `app/ui/shell/shell.css`
- Create: `app/ui/shortcuts.test.ts`
- Create: `app/ui/shortcuts.ts`
- Modify: `app/ui/view/store.tsx`

**Interfaces:**
- Consumes: `useLayoutStore().select`, `toggleNotes` (Task 4).
- Produces: `shortcuts.ts`: `Command` (`selectLayout{index}`, `toggleNotes`, `newChart`, `editChart`, `help`), `ShortcutInfo {id, keys}`, `SHORTCUT_LIST`, `modalOpen()`, `matchShortcut(e)`; `shell/Dialog.tsx`: `Dialog({title, onClose, variant?, children})`; `shell/HelpDialog.tsx`: `HelpDialog({onClose})`; i18n `shortcuts.*`.

- [ ] **Step 1: Önce testleri yaz**



`app/ui/App.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/App.test.tsx
+++ b/app/ui/App.test.tsx
@@ -145,3 +145,24 @@ test("a print failure is shown with a retry", async () => {
   fireEvent.click(screen.getByRole("button", { name: "Yazdır…" }));
   expect(await screen.findByRole("alert")).toHaveTextContent("Yazdırma penceresi açılamadı.");
 });
+
+test("? opens the shortcut help as a modal that traps focus, blocks shortcuts and closes with Esc", () => {
+  renderApp();
+  const opener = screen.getByRole("button", { name: "Kısayollar" });
+  opener.focus();
+  fireEvent.keyDown(opener, { key: "?", shiftKey: true });
+  const dialog = screen.getByRole("dialog", { name: "Klavye kısayolları" });
+  expect(dialog).toHaveAttribute("aria-modal", "true");
+  expect(within(dialog).getByText("⌘N")).toBeInTheDocument();
+  expect(within(dialog).getByRole("row", { name: /⌘⇧N Notlar panelini aç\/kapat/ })).toBeInTheDocument();
+  const close = within(dialog).getByRole("button", { name: "Kapat" });
+  expect(document.activeElement).toBe(close);
+  fireEvent.keyDown(close, { key: "Tab" });
+  expect(document.activeElement).toBe(close);
+  // Modal açıkken arkadaki düzen değişmez.
+  fireEvent.keyDown(window, { key: "2", code: "Digit2", metaKey: true });
+  expect(screen.getByRole("combobox", { name: "Düzen" })).toHaveDisplayValue("Görüşme");
+  fireEvent.keyDown(close, { key: "Escape" });
+  expect(screen.queryByRole("dialog")).toBeNull();
+  expect(document.activeElement).toBe(opener);
+});
```

`app/ui/shortcuts.test.ts` oluştur:

```ts
import { matchShortcut, SHORTCUT_LIST } from "./shortcuts";
import { tr } from "./i18n/tr";

const key = (init: KeyboardEventInit, target: EventTarget = document.body) => {
  const e = new KeyboardEvent("keydown", init);
  Object.defineProperty(e, "target", { value: target });
  return matchShortcut(e);
};

test("global commands", () => {
  expect(key({ key: "3", code: "Digit3", metaKey: true })).toEqual({ id: "selectLayout", index: 2 });
  expect(key({ key: "\"", code: "Digit3", metaKey: true })).toEqual({ id: "selectLayout", index: 2 });
  expect(key({ key: "N", metaKey: true, shiftKey: true })).toEqual({ id: "toggleNotes" });
  expect(key({ key: "n", metaKey: true })).toEqual({ id: "newChart" });
  expect(key({ key: "e", metaKey: true })).toEqual({ id: "editChart" });
  expect(key({ key: "?", shiftKey: true })).toEqual({ id: "help" });
});

test("ignored: Ctrl/Alt, digit 0, repeat, composition, editable fields and open modals", () => {
  expect(key({ key: "1", code: "Digit1", ctrlKey: true })).toBeNull();
  expect(key({ key: "n", metaKey: true, altKey: true })).toBeNull();
  expect(key({ key: "0", code: "Digit0", metaKey: true })).toBeNull();
  expect(key({ key: "n", metaKey: true, repeat: true })).toBeNull();
  expect(key({ key: "?", isComposing: true })).toBeNull();
  expect(key({ key: "?" }, document.createElement("input"))).toBeNull();
  const modal = document.createElement("div");
  modal.setAttribute("aria-modal", "true");
  document.body.append(modal);
  expect(key({ key: "n", metaKey: true })).toBeNull();
  modal.remove();
  expect(key({ key: "n", metaKey: true })).toEqual({ id: "newChart" });
});

test("every listed shortcut has a description", () => {
  for (const s of SHORTCUT_LIST) expect(Object.keys(tr.shortcuts)).toContain(s.id);
  expect(Object.keys(tr.shortcuts).filter((k) => k !== "title" && k !== "open" && k !== "close" && k !== "keys" && k !== "action")).toEqual(
    SHORTCUT_LIST.map((s) => s.id),
  );
});
```


- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/shortcuts.test.ts ui/App.test.tsx
```
Expected: FAIL — `Failed to resolve import "./shortcuts"`; `?` tuşu yardım penceresini açmıyor.

- [ ] **Step 3: Uygulamayı yaz**



`app/ui/i18n/en.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -196,4 +196,20 @@ export const en: Messages = {
   dasha: {
     approx: "Birth time unknown: dasha dates are approximate (≈).",
   },
+  shortcuts: {
+    title: "Keyboard shortcuts",
+    open: "Shortcuts",
+    close: "Close",
+    keys: "Keys",
+    action: "Action",
+    newChart: "New chart",
+    editChart: "Edit the open chart",
+    selectLayout: "Switch to the layout at that position",
+    toggleNotes: "Show or hide the notes panel",
+    help: "Open this help",
+    escape: "Clear the highlight, close a window",
+    movePanel: "Move the focused panel",
+    resizePanel: "Resize the focused panel",
+    tree: "Move through, open and close the dasha tree",
+  },
 };
```

`app/ui/i18n/fr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -196,4 +196,20 @@ export const fr: Messages = {
   dasha: {
     approx: "Heure de naissance inconnue : les dates de dasha sont approximatives (≈).",
   },
+  shortcuts: {
+    title: "Raccourcis clavier",
+    open: "Raccourcis",
+    close: "Fermer",
+    keys: "Touches",
+    action: "Action",
+    newChart: "Nouvelle carte",
+    editChart: "Modifier la carte ouverte",
+    selectLayout: "Passer à la disposition de ce rang",
+    toggleNotes: "Afficher ou masquer les notes",
+    help: "Ouvrir cette aide",
+    escape: "Effacer la mise en évidence, fermer une fenêtre",
+    movePanel: "Déplacer le panneau actif",
+    resizePanel: "Redimensionner le panneau actif",
+    tree: "Parcourir, ouvrir et fermer l’arbre des dashas",
+  },
 };
```

`app/ui/i18n/tr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -195,6 +195,22 @@ export const tr = {
   dasha: {
     approx: "Doğum saati bilinmiyor: dasa tarihleri yaklaşıktır (≈).",
   },
+  shortcuts: {
+    title: "Klavye kısayolları",
+    open: "Kısayollar",
+    close: "Kapat",
+    keys: "Tuşlar",
+    action: "İşlem",
+    newChart: "Yeni harita",
+    editChart: "Açık haritayı düzenle",
+    selectLayout: "Sıradaki düzene geç",
+    toggleNotes: "Notlar panelini aç/kapat",
+    help: "Bu yardımı aç",
+    escape: "Vurguyu temizle, pencereyi kapat",
+    movePanel: "Odaktaki paneli taşı",
+    resizePanel: "Odaktaki paneli boyutlandır",
+    tree: "Dasa ağacında gezin, aç, kapat",
+  },
 };
 
 type Strings<T> = { [K in keyof T]: T[K] extends string ? string : Strings<T[K]> };
```

`app/ui/shell/Dialog.tsx` oluştur:

```tsx
// Modal pencere (WAI-ARIA dialog): açılınca ilk alana odaklanır, Tab içeride döner, Esc
// kapatır, kapanınca odak açan öğeye döner. `variant="sheet"` sağdan kayan paneldir
// (doğum formu); arka plan tıklamayla kapanmaz (yarım form kaybolmasın).
import { useEffect, useId, useRef, type KeyboardEvent, type ReactNode } from "react";

const FOCUSABLE = 'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

export interface DialogProps {
  title: string;
  onClose(): void;
  variant?: "center" | "sheet";
  children: ReactNode;
}

export function Dialog({ title, onClose, variant = "center", children }: DialogProps) {
  const ref = useRef<HTMLDivElement>(null);
  const titleId = useId();

  useEffect(() => {
    const previous = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const first = ref.current?.querySelector<HTMLElement>("[data-autofocus]:not([disabled])") ?? ref.current?.querySelector<HTMLElement>(FOCUSABLE);
    (first ?? ref.current)?.focus();
    return () => previous?.focus();
  }, []);

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      onClose();
      return;
    }
    if (e.key !== "Tab" || !ref.current) return;
    const items = [...ref.current.querySelectorAll<HTMLElement>(FOCUSABLE)];
    if (items.length === 0) return;
    const [first, last] = [items[0], items[items.length - 1]];
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault();
      last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault();
      first.focus();
    }
  };

  return (
    <div className="dialog-backdrop no-print">
      <div
        ref={ref}
        className={`dialog dialog--${variant}`}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        tabIndex={-1}
        onKeyDown={onKeyDown}
      >
        <h2 id={titleId} className="dialog-title">
          {title}
        </h2>
        {children}
      </div>
    </div>
  );
}
```

`app/ui/shell/HelpDialog.tsx` oluştur:

```tsx
import { useTranslation } from "react-i18next";
import { SHORTCUT_LIST } from "../shortcuts";
import { Dialog } from "./Dialog";

/** Kısayol yardımı (`?`): liste `SHORTCUT_LIST`'ten gelir, burada elle yazılmaz. */
export function HelpDialog({ onClose }: { onClose(): void }) {
  const { t } = useTranslation();
  return (
    <Dialog title={t("shortcuts.title")} onClose={onClose}>
      <table className="shortcut-table">
        <thead>
          <tr>
            <th scope="col">{t("shortcuts.keys")}</th>
            <th scope="col">{t("shortcuts.action")}</th>
          </tr>
        </thead>
        <tbody>
          {SHORTCUT_LIST.map((s) => (
            <tr key={s.id}>
              <td>
                <kbd>{s.keys}</kbd>
              </td>
              <td>{t(`shortcuts.${s.id}`)}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <div className="dialog-actions">
        <button type="button" onClick={onClose}>
          {t("shortcuts.close")}
        </button>
      </div>
    </Dialog>
  );
}
```

`app/ui/shell/Shell.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/Shell.tsx
+++ b/app/ui/shell/Shell.tsx
@@ -1,8 +1,9 @@
 import { useEffect, useLayoutEffect, useRef, useState } from "react";
 import { Dashboard } from "../dashboard/Dashboard";
-import { isEditableTarget } from "../keyboard";
+import { matchShortcut } from "../shortcuts";
 import { useLayoutStore } from "../view/store";
 import { applyTheme, loadThemePref, saveThemePref, type ThemePref } from "../theme/theme";
+import { HelpDialog } from "./HelpDialog";
 import { NoticeBar } from "./notices";
 import { PrintView } from "./PrintView";
 import "./shell.css";
@@ -13,6 +14,7 @@ export function Shell() {
   const { toggleNotes } = useLayoutStore();
   const [themePref, setThemePref] = useState<ThemePref>(() => loadThemePref());
   const [printView, setPrintView] = useState(false);
+  const [help, setHelp] = useState(false);
   const measureRef = useRef<HTMLDivElement>(null);
   const width = useElementWidth(measureRef);
 
@@ -23,15 +25,17 @@ export function Shell() {
     setThemePref(pref);
   };
 
-  // Cmd+Shift+N: notlar panelini etkin düzende aç/kapat (spec §3.2). Yazı alanlarında,
-  // tuş tekrarında, IME bileşiminde ve baskı görünümünde (pano yokken) yok sayılır.
+  // Genel kısayollar (shortcuts.ts). Cmd+Shift+N notlar panelini etkin düzende açar/kapatır
+  // (spec §3.2); baskı görünümünde (pano yokken) yok sayılır. `?` yardımı açar.
   useEffect(() => {
-    if (printView) return;
     const onKey = (e: KeyboardEvent) => {
-      if (e.repeat || e.isComposing || isEditableTarget(e.target)) return;
-      if (e.metaKey && e.shiftKey && !e.altKey && !e.ctrlKey && e.key.toLowerCase() === "n") {
+      const command = matchShortcut(e);
+      if (command?.id === "toggleNotes" && !printView) {
         e.preventDefault();
         toggleNotes();
+      } else if (command?.id === "help") {
+        e.preventDefault();
+        setHelp(true);
       }
     };
     window.addEventListener("keydown", onKey);
@@ -45,11 +49,13 @@ export function Shell() {
         onThemeChange={changeTheme}
         printView={printView}
         onTogglePrint={() => setPrintView((v) => !v)}
+        onHelp={() => setHelp(true)}
       />
       <NoticeBar />
       <main className="shell-main">
         <div ref={measureRef}>{printView ? <PrintView /> : <Dashboard width={width} />}</div>
       </main>
+      {help && <HelpDialog onClose={() => setHelp(false)} />}
     </div>
   );
 }
```

`app/ui/shell/TopBar.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/TopBar.tsx
+++ b/app/ui/shell/TopBar.tsx
@@ -9,10 +9,11 @@ export interface TopBarProps {
   onThemeChange(pref: ThemePref): void;
   printView: boolean;
   onTogglePrint(): void;
+  onHelp(): void;
 }
 
 /** Üst çubuk (spec §4): solda danışan ve harita (uyarı rozetleriyle), ortada düzen seçici, sağda tema, dil ve baskı. */
-export function TopBar({ themePref, onThemeChange, printView, onTogglePrint }: TopBarProps) {
+export function TopBar({ themePref, onThemeChange, printView, onTogglePrint, onHelp }: TopBarProps) {
   const { t, i18n } = useTranslation();
   const language = isLanguage(i18n.language) ? i18n.language : DEFAULT_LANGUAGE;
   return (
@@ -40,6 +41,9 @@ export function TopBar({ themePref, onThemeChange, printView, onTogglePrint }: T
             ))}
           </select>
         </label>
+        <button type="button" aria-keyshortcuts="?" onClick={onHelp}>
+          {t("shortcuts.open")}
+        </button>
         <button type="button" onClick={onTogglePrint}>
           {printView ? t("shell.backToDesk") : t("shell.printSpike")}
         </button>
```

`app/ui/shell/shell.css` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/shell.css
+++ b/app/ui/shell/shell.css
@@ -135,3 +135,56 @@
 .notice-text {
   flex: 1;
 }
+.dialog-backdrop {
+  position: fixed;
+  inset: 0;
+  z-index: 100;
+  display: flex;
+  align-items: center;
+  justify-content: center;
+  background: color-mix(in srgb, var(--ink-strong) 30%, transparent);
+}
+.dialog {
+  background: var(--panel);
+  color: var(--text);
+  border: 1px solid var(--line);
+  border-radius: 6px;
+  padding: 16px;
+  max-height: 90vh;
+  overflow: auto;
+}
+.dialog:focus-visible {
+  outline: 2px solid var(--ink);
+}
+.dialog--center {
+  min-width: 360px;
+}
+.dialog--sheet {
+  align-self: stretch;
+  margin-left: auto;
+  width: min(480px, 100vw);
+  max-height: none;
+  border-radius: 0;
+}
+.dialog-title {
+  margin: 0 0 12px;
+  font-size: var(--fs-panel-title);
+  font-weight: var(--fw-medium);
+}
+.dialog-actions {
+  display: flex;
+  justify-content: flex-end;
+  gap: 8px;
+  margin-top: 12px;
+}
+.shortcut-table th,
+.shortcut-table td {
+  text-align: left;
+  padding: 2px 8px;
+}
+kbd {
+  font-family: var(--font-ui);
+  border: 1px solid var(--line);
+  border-radius: 3px;
+  padding: 0 4px;
+}
```

`app/ui/shortcuts.ts` oluştur:

```ts
// Klavye kısayollarının tek kaynağı (spec §4 "tam klavye"). Genel komutlar burada eşlenir;
// yardım penceresi (`?`) listeyi buradan çizer. Panel içi tuşlar (Alt+ok, ağaç gezintisi)
// odaktaki öğede işlenir ve burada yalnızca listelenir.
import { isEditableTarget } from "./keyboard";

export type Command =
  | { id: "selectLayout"; index: number }
  | { id: "toggleNotes" }
  | { id: "newChart" }
  | { id: "editChart" }
  | { id: "help" };

export interface ShortcutInfo {
  /** i18n anahtarı: `shortcuts.<id>`. */
  id: "selectLayout" | "toggleNotes" | "newChart" | "editChart" | "help" | "escape" | "movePanel" | "resizePanel" | "tree";
  keys: string;
}

export const SHORTCUT_LIST: readonly ShortcutInfo[] = [
  { id: "newChart", keys: "⌘N" },
  { id: "editChart", keys: "⌘E" },
  { id: "selectLayout", keys: "⌘1 … ⌘9" },
  { id: "toggleNotes", keys: "⌘⇧N" },
  { id: "help", keys: "?" },
  { id: "escape", keys: "Esc" },
  { id: "movePanel", keys: "⌥ ←↑→↓" },
  { id: "resizePanel", keys: "⌥⇧ ←↑→↓" },
  { id: "tree", keys: "↑ ↓ ← → Home End ↵" },
];

/** Açık bir modal pencere varken genel kısayollar kapalıdır (arkadaki pano değişmez). */
export function modalOpen(): boolean {
  return document.querySelector('[aria-modal="true"]') !== null;
}

/**
 * Klavye olayı → genel komut. Yazı alanlarında, tuş tekrarında, IME bileşiminde ve modal
 * açıkken `null`. Rakamlar fiziksel tuştan (`code`) okunur: AZERTY'de `key` rakam değildir.
 */
export function matchShortcut(e: KeyboardEvent): Command | null {
  if (e.repeat || e.isComposing || isEditableTarget(e.target) || modalOpen()) return null;
  if (e.ctrlKey || e.altKey) return null;
  if (e.metaKey) {
    if (e.shiftKey) return e.key.toLowerCase() === "n" ? { id: "toggleNotes" } : null;
    if (/^Digit[1-9]$/.test(e.code)) return { id: "selectLayout", index: Number(e.code.slice(5)) - 1 };
    if (e.key.toLowerCase() === "n") return { id: "newChart" };
    if (e.key.toLowerCase() === "e") return { id: "editChart" };
    return null;
  }
  return e.key === "?" ? { id: "help" } : null;
}
```

`app/ui/view/store.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/view/store.tsx
+++ b/app/ui/view/store.tsx
@@ -4,10 +4,10 @@
 // dasha yoluna `useViewStore` ile ulaşır; birbirini çağırmaz.
 import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type Dispatch, type ReactNode } from "react";
 import type { Graha, ViewState } from "../bindings";
-import { isEditableTarget } from "../keyboard";
 import type { Layout, LayoutState, Panel } from "../layout/model";
 import { activeLayout, newId, normalizeState, type LayoutAction } from "../layout/reducer";
 import { toViewState, type ViewModel } from "./model";
+import { matchShortcut } from "../shortcuts";
 import type { ViewPersistence } from "./persistence";
 import { viewReducer, type ViewAction } from "./reducer";
 
@@ -102,14 +102,10 @@ export function ViewStateProvider({
 
   useEffect(() => {
     const onKey = (e: KeyboardEvent) => {
-      if (e.repeat || e.isComposing || isEditableTarget(e.target)) return;
-      if (!e.metaKey || e.shiftKey || e.altKey || e.ctrlKey) return;
-      // Fiziksel tuş (`code`): AZERTY'de `key` rakam değildir.
-      if (!/^Digit[1-9]$/.test(e.code)) return;
-      const index = Number(e.code.slice(5)) - 1;
-      if (index >= stateRef.current.layout.layouts.length) return;
+      const command = matchShortcut(e);
+      if (command?.id !== "selectLayout" || command.index >= stateRef.current.layout.layouts.length) return;
       e.preventDefault();
-      dispatch({ type: "layout", action: { type: "selectIndex", index } });
+      dispatch({ type: "layout", action: { type: "selectIndex", index: command.index } });
     };
     window.addEventListener("keydown", onKey);
     return () => window.removeEventListener("keydown", onKey);
```


- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app exec vitest run ui/shortcuts.test.ts ui/App.test.tsx
pnpm --dir app test
pnpm --dir app build
```
Expected: tam takımda `Test Files  32 passed (32)`, `Tests  263 passed (263)`; `build` hatasız (`tsc --noEmit` çıktısız, `vite build` `✓ built`).

- [ ] **Step 5: Commit**

```bash
git add -A app/ui/App.test.tsx app/ui/i18n/en.ts app/ui/i18n/fr.ts app/ui/i18n/tr.ts app/ui/shell/Dialog.tsx app/ui/shell/HelpDialog.tsx app/ui/shell/Shell.tsx app/ui/shell/TopBar.tsx app/ui/shell/shell.css app/ui/shortcuts.test.ts app/ui/shortcuts.ts app/ui/view/store.tsx
git diff --cached -U0 -- . ':!app/pnpm-lock.yaml' ':!*.png' ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git commit -m "feat(ui): central shortcut registry, modal dialog with focus trap and the ? shortcut help

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 10: Doğum formu modeli: biçim denetimi ve arka uç hata alanı eşlemesi

Formun saf modeli (D17): metin alanlarından oluşan `BirthDraft`, biçim denetimi (`toBirthData`; tarih, saat, enlem/boylam aralığı, sabit ofset `+05:30`; işaret zorunludur ve `05:30` reddedilir: işaret sınıfı `[+\-−]`'dir, `[+-−]` bir karakter aralığı olurdu ve işaretsiz girişi ters işaretli ofsete çevirirdi), atlas sonucunun ve DST önerisinin taslağa uygulanması, düzenleme için `OpenedClient`'tan taslak, ve `invalid_input.field` → form alanı eşlemesi. Anlam denetimi (geçersiz gün, DST) Rust'tadır; model onu tekrarlamaz.

**Files:**
- Create: `app/ui/birth/model.test.ts`
- Create: `app/ui/birth/model.ts`

**Interfaces:**
- Consumes: tipler `BirthData`, `NewChart`, `NewClient`, `OpenedClient`, `PlaceHit` (bağlamalar), `CommandError`.
- Produces: `birth/model.ts`: `TzMode`, `BirthDraft`, `BirthField` (`name | label | date | time | certainty | lat | lon | tz | fold`), `FieldErrors`, `emptyDraft()`, `parseFixedOffset(text)`, `formatFixedOffset(secs)`, `draftFromOpened(o)`, `applyPlace(d, hit)`, `setCoordinate(d, field, value)`, `applySuggestion(d, suggestion)`, `toBirthData(d)`, `toNewChart(d, birth)`, `toNewClient(d, birth)`, `errorField(error)`.

- [ ] **Step 1: Önce testleri yaz**



`app/ui/birth/model.test.ts` oluştur:

```ts
import { fixture } from "../test/fixture";
import {
  applyPlace,
  applySuggestion,
  draftFromOpened,
  emptyDraft,
  errorField,
  formatFixedOffset,
  parseFixedOffset,
  setCoordinate,
  toBirthData,
  toNewClient,
} from "./model";

const delhiDraft = () => ({ ...applyPlace({ ...emptyDraft(), name: " Ayşe ", date: "2000-01-01", time: "17:30" }, fixture().places[0]) });

test("fixed offsets parse and format both ways", () => {
  expect(parseFixedOffset("+05:30")).toBe(19_800);
  expect(parseFixedOffset("-3:00")).toBe(-10_800);
  expect(parseFixedOffset("−03:00")).toBe(-10_800);
  expect(parseFixedOffset("+5:75")).toBeNull();
  expect(parseFixedOffset("0530")).toBeNull();
  // İşaret zorunlu: işaretsiz ya da bozuk giriş ters işaretli bir ofsete dönmez.
  expect(parseFixedOffset("05:30")).toBeNull();
  expect(parseFixedOffset("105:30")).toBeNull();
  expect(parseFixedOffset("x5:30")).toBeNull();
  expect(formatFixedOffset(19_800)).toBe("+05:30");
  expect(formatFixedOffset(-34_200)).toBe("-09:30");
});

test("an atlas hit fills the place and suggests its IANA zone", () => {
  const d = delhiDraft();
  expect(d).toMatchObject({ placeName: "New Delhi", lat: "28.62137", tzMode: "iana", tzIana: "Asia/Kolkata", geonameId: 1261481 });
  const manual = setCoordinate(d, "lat", "28.7");
  expect(manual).toMatchObject({ lat: "28.7", geonameId: null, placeName: "New Delhi" });
});

test("a complete draft becomes NewClient; unknown time sends no time", () => {
  const { birth } = toBirthData(delhiDraft());
  expect(birth).toEqual({
    date: "2000-01-01",
    time: "17:30",
    time_certainty: "Exact",
    place: { name: "New Delhi", country: "India", admin1: "Delhi", lat: 28.62137, lon: 77.2148, geoname_id: 1261481 },
    tz: { Iana: "Asia/Kolkata" },
    fold: null,
    notes: "",
  });
  expect(toNewClient(delhiDraft(), birth!)).toMatchObject({ name: "Ayşe", gender: null, chart: { label: "", kind: "Natal" } });
  const unknown = toBirthData({ ...delhiDraft(), time: "", certainty: "Unknown" });
  expect(unknown.birth).toMatchObject({ time: null, time_certainty: "Unknown" });
  expect(toBirthData({ ...delhiDraft(), tzMode: "fixed", tzFixed: "+05:30" }).birth!.tz).toEqual({ Fixed: 19_800 });
  expect(toBirthData({ ...delhiDraft(), tzMode: "lmt" }).birth!.tz).toBe("Lmt");
  expect(toBirthData({ ...delhiDraft(), lat: "28,5" }).birth!.place.lat).toBe(28.5);
});

test("format errors are reported per field before anything is sent", () => {
  const { errors } = toBirthData({ ...emptyDraft(), lat: "north", tzMode: "fixed", tzFixed: "5" });
  expect(errors).toEqual({
    name: { key: "birth.error.required" },
    date: { key: "birth.error.required" },
    time: { key: "birth.error.required" },
    lat: { key: "birth.error.number" },
    lon: { key: "birth.error.number" },
    tz: { key: "birth.error.offset" },
  });
});

test("backend errors map to fields", () => {
  const e = (code: string, params: Record<string, string> = {}) => ({ code, params });
  expect(errorField(e("invalid_input", { field: "birth.date" }))).toBe("date");
  expect(errorField(e("invalid_input", { field: "birth.place.lat" }))).toBe("lat");
  expect(errorField(e("invalid_input", { field: "name" }))).toBe("name");
  expect(errorField(e("invalid_input", { field: "tags" }))).toBeNull();
  expect(errorField(fixture().gap_error)).toBe("time");
  expect(errorField(e("ambiguous_local_time"))).toBe("fold");
  expect(errorField(e("unknown_time_zone"))).toBe("tz");
  expect(errorField(e("store_sqlite"))).toBeNull();
});

test("the DST suggestion and editing a stored chart", () => {
  expect(applySuggestion(delhiDraft(), fixture().gap_error.params.suggestion)).toMatchObject({ date: "2014-03-31", time: "04:30" });
  const d = draftFromOpened(fixture().opened);
  expect(d).toMatchObject({ name: "Örnek Danışan", date: "2000-01-01", time: "17:30", tzMode: "iana", tzIana: "Asia/Kolkata", kind: "Natal" });
  expect(toBirthData(d).birth!.place).toEqual(fixture().opened.chart_record.birth.place);
});
```


- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/birth/model.test.ts
```
Expected: FAIL — `Failed to resolve import "./model"`.

- [ ] **Step 3: Uygulamayı yaz**



`app/ui/birth/model.ts` oluştur:

```ts
// Doğum formu modeli (saf). Form metin alanlarını tutar; `toNewChart` biçim denetimi yapar
// (boş zorunlu alan, sayı olmayan koordinat, ofset biçimi). Anlam denetimi (tarih geçerli mi,
// DST boşluğu, aralıklar) Rust'tadır: `preview_moment` ve kayıt komutları `invalid_input`
// ve an hatalarıyla döner; `errorField` bunları form alanına bağlar.
import type { BirthData, ChartKind, CommandError, FoldChoice, Gender, NewChart, NewClient, OpenedClient, PlaceHit, TimeCertainty, TzChoice } from "../bindings";

export type TzMode = "iana" | "fixed" | "lmt";

export interface BirthDraft {
  name: string;
  gender: Gender | null;
  label: string;
  kind: ChartKind;
  date: string;
  time: string;
  certainty: TimeCertainty;
  placeName: string;
  country: string;
  admin1: string;
  geonameId: number | null;
  lat: string;
  lon: string;
  tzMode: TzMode;
  tzIana: string;
  /** "+05:30" biçimi. */
  tzFixed: string;
  fold: FoldChoice | null;
  notes: string;
}

export type BirthField = "name" | "label" | "date" | "time" | "certainty" | "lat" | "lon" | "tz" | "fold";
/** Alan → i18n anahtarı (`birth.error.*`) ya da arka uç hatası. */
export type FieldErrors = Partial<Record<BirthField, { key: string } | { error: CommandError }>>;

export function emptyDraft(): BirthDraft {
  return {
    name: "",
    gender: null,
    label: "",
    kind: "Natal",
    date: "",
    time: "",
    certainty: "Exact",
    placeName: "",
    country: "",
    admin1: "",
    geonameId: null,
    lat: "",
    lon: "",
    tzMode: "iana",
    tzIana: "",
    tzFixed: "+00:00",
    fold: null,
    notes: "",
  };
}

/** "+05:30" / "-03:00" / "+5:45" → saniye; biçim dışıysa `null`. */
export function parseFixedOffset(text: string): number | null {
  const m = /^([+\-−])(\d{1,2}):(\d{2})$/.exec(text.trim());
  if (!m) return null;
  const [h, min] = [Number(m[2]), Number(m[3])];
  if (min >= 60) return null;
  return (m[1] === "+" ? 1 : -1) * (h * 3600 + min * 60);
}

export function formatFixedOffset(secs: number): string {
  const a = Math.abs(secs);
  const h = Math.floor(a / 3600);
  const m = Math.floor((a % 3600) / 60);
  return `${secs < 0 ? "-" : "+"}${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}`;
}

function parseCoord(text: string): number | null {
  const t = text.trim().replace(",", ".");
  if (t === "" || !/^[+-]?\d+(\.\d+)?$/.test(t)) return null;
  return Number(t);
}

/** Saklanan harita → düzenleme taslağı (Cmd+E). */
export function draftFromOpened(o: OpenedClient): BirthDraft {
  const b = o.chart_record.birth;
  // `{ Fixed } & { Iana?: never }` biçiminde `in` daraltmaz; alan değerine bakılır.
  const iana = b.tz === "Lmt" ? undefined : b.tz.Iana;
  const fixed = b.tz === "Lmt" ? undefined : b.tz.Fixed;
  return {
    name: o.client.name,
    gender: o.client.gender,
    label: o.chart_record.label,
    kind: o.chart_record.kind,
    date: b.date,
    time: b.time ? b.time.slice(0, 5) : "",
    certainty: b.time_certainty,
    placeName: b.place.name,
    country: b.place.country,
    admin1: b.place.admin1,
    geonameId: b.place.geoname_id,
    lat: b.place.lat === null ? "" : String(b.place.lat),
    lon: b.place.lon === null ? "" : String(b.place.lon),
    tzMode: iana !== undefined ? "iana" : fixed !== undefined ? "fixed" : "lmt",
    tzIana: iana ?? "",
    tzFixed: fixed !== undefined ? formatFixedOffset(fixed) : "+00:00",
    fold: b.fold,
    notes: b.notes,
  };
}

/** Atlas sonucu: yer alanları ve önerilen IANA saat dilimi. */
export function applyPlace(d: BirthDraft, hit: PlaceHit): BirthDraft {
  return {
    ...d,
    placeName: hit.name,
    country: hit.country,
    admin1: hit.admin1,
    geonameId: hit.geoname_id,
    lat: String(hit.lat),
    lon: String(hit.lon),
    tzMode: "iana",
    tzIana: hit.timezone,
    fold: null,
  };
}

/** Elle koordinat: atlas bağı kopar (`geoname_id` boşalır), ad korunur. */
export function setCoordinate(d: BirthDraft, field: "lat" | "lon", value: string): BirthDraft {
  return { ...d, [field]: value, geonameId: null };
}

/** DST boşluğu önerisi ("2014-03-31T04:30:00") tarih ve saate yazılır. */
export function applySuggestion(d: BirthDraft, suggestion: string): BirthDraft {
  const [date, time] = suggestion.split("T");
  return { ...d, date, time: time.slice(0, 5) };
}

function tzChoice(d: BirthDraft): TzChoice | null {
  if (d.tzMode === "lmt") return "Lmt";
  if (d.tzMode === "iana") return d.tzIana.trim() === "" ? null : { Iana: d.tzIana.trim() };
  const secs = parseFixedOffset(d.tzFixed);
  return secs === null ? null : { Fixed: secs };
}

/** Biçim denetimi: hatasızsa doğum verisi, değilse alan hataları. */
export function toBirthData(d: BirthDraft): { birth: BirthData; errors: null } | { birth: null; errors: FieldErrors } {
  const errors: FieldErrors = {};
  if (d.name.trim() === "") errors.name = { key: "birth.error.required" };
  if (d.date.trim() === "") errors.date = { key: "birth.error.required" };
  const unknown = d.certainty === "Unknown";
  if (!unknown && d.time.trim() === "") errors.time = { key: "birth.error.required" };
  const lat = parseCoord(d.lat);
  const lon = parseCoord(d.lon);
  if (lat === null) errors.lat = { key: "birth.error.number" };
  if (lon === null) errors.lon = { key: "birth.error.number" };
  const tz = tzChoice(d);
  if (tz === null) errors.tz = { key: d.tzMode === "fixed" ? "birth.error.offset" : "birth.error.required" };
  if (Object.keys(errors).length > 0 || lat === null || lon === null || tz === null) return { birth: null, errors };
  return {
    errors: null,
    birth: {
      date: d.date.trim(),
      time: unknown ? null : d.time.trim(),
      time_certainty: d.certainty,
      place: { name: d.placeName.trim(), country: d.country.trim(), admin1: d.admin1.trim(), lat, lon, geoname_id: d.geonameId },
      tz,
      fold: d.fold,
      notes: d.notes,
    },
  };
}

export function toNewChart(d: BirthDraft, birth: BirthData): NewChart {
  return { label: d.label.trim(), kind: d.kind, birth };
}

export function toNewClient(d: BirthDraft, birth: BirthData): NewClient {
  return { name: d.name.trim(), gender: d.gender, chart: toNewChart(d, birth) };
}

const INPUT_FIELDS: Record<string, BirthField> = {
  "birth.date": "date",
  "birth.time": "time",
  "birth.time_certainty": "certainty",
  "birth.place.lat": "lat",
  "birth.place.lon": "lon",
  name: "name",
  label: "label",
};

/** Arka uç hatası → form alanı; alana bağlanamayan hata `null` (form üstünde gösterilir). */
export function errorField(error: CommandError): BirthField | null {
  switch (error.code) {
    case "invalid_input":
      return INPUT_FIELDS[error.params.field] ?? null;
    case "nonexistent_local_time":
      return "time";
    case "ambiguous_local_time":
      return "fold";
    case "unknown_time_zone":
    case "offset_out_of_range":
      return "tz";
    case "longitude_out_of_range":
      return "lon";
    case "date_out_of_range":
      return "date";
    default:
      return null;
  }
}
```


- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app exec vitest run ui/birth/model.test.ts
pnpm --dir app test
pnpm --dir app build
```
Expected: tam takımda `Test Files  33 passed (33)`, `Tests  269 passed (269)`; `build` hatasız (`tsc --noEmit` çıktısız, `vite build` `✓ built`).

- [ ] **Step 5: Commit**

```bash
git add -A app/ui/birth/model.test.ts app/ui/birth/model.ts
git diff --cached -U0 -- . ':!app/pnpm-lock.yaml' ':!*.png' ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git commit -m "feat(ui): pure birth form model with format checks and backend error field mapping

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 11: Atlas üzerinde erişilebilir yer arama kutusu

WAI-ARIA combobox (D17): yazdıkça 200 ms debounce ile `search_places`; eski yanıtlar sıra numarasıyla yok sayılır; ↑/↓ ile seçenek, Enter ile seçim, Esc listeyi kapatır ve olayın üst pencereye (modal) gitmesini durdurur. Sonuç sayısı canlı bölgede duyurulur. `atlas_unavailable` hatası elle koordinat girişini engellemez.

**Files:**
- Create: `app/ui/birth/PlaceSearch.test.tsx`
- Create: `app/ui/birth/PlaceSearch.tsx`
- Create: `app/ui/birth/birth.css`
- Modify: `app/ui/i18n/en.ts`
- Modify: `app/ui/i18n/fr.ts`
- Modify: `app/ui/i18n/tr.ts`

**Interfaces:**
- Consumes: `api.searchPlaces` (Task 3), `PlaceHit` (Task 1).
- Produces: `birth/PlaceSearch.tsx`: `PlaceSearch(PlaceSearchProps {value, onChange, onSelect, invalid?, describedBy?})`, `SEARCH_DEBOUNCE_MS = 200`, `SEARCH_LIMIT = 8`; `birth/birth.css`; i18n `birth.place`, `birth.placeResults`, `birth.placeCount`.

- [ ] **Step 1: Önce testleri yaz**



`app/ui/birth/PlaceSearch.test.tsx` oluştur:

```tsx
import { act, fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { vi } from "vitest";

vi.mock("../api", () => ({ api: { searchPlaces: vi.fn() } }));

import { api } from "../api";
import type { PlaceHit } from "../bindings";
import { fixture } from "../test/fixture";
import { PlaceSearch, SEARCH_DEBOUNCE_MS, SEARCH_LIMIT } from "./PlaceSearch";

function Harness({ onSelect }: { onSelect(h: PlaceHit): void }) {
  const [value, setValue] = useState("");
  return <PlaceSearch value={value} onChange={setValue} onSelect={(h) => (setValue(h.name), onSelect(h))} />;
}

beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(api.searchPlaces).mockReset();
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "ok", data: fixture().places });
});
afterEach(() => vi.useRealTimers());

const box = () => screen.getByRole("combobox", { name: "Doğum yeri" });
async function type(text: string) {
  fireEvent.change(box(), { target: { value: text } });
  await act(async () => vi.advanceTimersByTime(SEARCH_DEBOUNCE_MS));
}

test("debounced search opens a listbox; arrows move the active option; Enter selects", async () => {
  const onSelect = vi.fn();
  render(<Harness onSelect={onSelect} />);
  fireEvent.change(box(), { target: { value: "ne" } });
  fireEvent.change(box(), { target: { value: "new delhi" } });
  expect(api.searchPlaces).not.toHaveBeenCalled();
  await act(async () => vi.advanceTimersByTime(SEARCH_DEBOUNCE_MS));
  expect(api.searchPlaces).toHaveBeenCalledTimes(1);
  expect(api.searchPlaces).toHaveBeenCalledWith("new delhi", SEARCH_LIMIT);
  expect(box()).toHaveAttribute("aria-expanded", "true");
  const options = screen.getAllByRole("option");
  expect(options.length).toBe(fixture().places.length);
  expect(options[0]).toHaveTextContent("New Delhi");
  expect(options[0]).toHaveTextContent("Asia/Kolkata");
  expect(box()).toHaveAttribute("aria-activedescendant", options[0].id);
  fireEvent.keyDown(box(), { key: "ArrowUp" });
  expect(box()).toHaveAttribute("aria-activedescendant", options[options.length - 1].id);
  fireEvent.keyDown(box(), { key: "ArrowDown" });
  fireEvent.keyDown(box(), { key: "Enter" });
  expect(onSelect).toHaveBeenCalledWith(fixture().places[0]);
  expect(box()).toHaveValue("New Delhi");
  expect(box()).toHaveAttribute("aria-expanded", "false");
  // Seçim yeni bir arama başlatmaz.
  await act(async () => vi.advanceTimersByTime(SEARCH_DEBOUNCE_MS * 2));
  expect(api.searchPlaces).toHaveBeenCalledTimes(1);
});

test("a stale response is ignored", async () => {
  let first!: (v: Awaited<ReturnType<typeof api.searchPlaces>>) => void;
  vi.mocked(api.searchPlaces).mockReturnValueOnce(new Promise((r) => (first = r)));
  render(<Harness onSelect={() => {}} />);
  await type("lon");
  await type("new delhi");
  await act(async () => first({ status: "ok", data: [{ ...fixture().places[0], name: "London", geoname_id: 1 }] }));
  expect(screen.queryByText("London")).toBeNull();
  expect(screen.getAllByRole("option")[0]).toHaveTextContent("New Delhi");
});

test("Escape closes the list without bubbling; mouse selection works", async () => {
  const onSelect = vi.fn();
  const onOuterKey = vi.fn();
  render(
    <div onKeyDown={onOuterKey}>
      <Harness onSelect={onSelect} />
    </div>,
  );
  await type("new delhi");
  fireEvent.keyDown(box(), { key: "Escape" });
  expect(box()).toHaveAttribute("aria-expanded", "false");
  expect(onOuterKey).not.toHaveBeenCalled();
  fireEvent.keyDown(box(), { key: "ArrowDown" });
  fireEvent.mouseDown(screen.getAllByRole("option")[0]);
  expect(onSelect).toHaveBeenCalledTimes(1);
});

test("an unavailable atlas is explained and leaves the field usable", async () => {
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "error", error: { code: "atlas_unavailable", params: {} } });
  render(<Harness onSelect={() => {}} />);
  await type("delhi");
  expect(screen.getByRole("status")).toHaveTextContent("Yer atlası açılamadı; koordinatları elle girin.");
  expect(box()).toHaveAttribute("aria-expanded", "false");
  expect(box()).toHaveValue("delhi");
});
```


- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/birth/PlaceSearch.test.tsx
```
Expected: FAIL — `Failed to resolve import "./PlaceSearch"`.

- [ ] **Step 3: Uygulamayı yaz**



`app/ui/birth/PlaceSearch.tsx` oluştur:

```tsx
// Yer arama kutusu (WAI-ARIA 1.2 combobox, liste açılır). Yazdıkça 200 ms sonra
// `search_places` çağrılır; eski isteğin geç gelen yanıtı yok sayılır. Atlas yoksa bilgi
// verilir ve elle koordinat girişi açık kalır (spec §12).
import { useEffect, useId, useRef, useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CommandError, PlaceHit } from "../bindings";
import { errorMessage } from "../i18n/errors";
import "./birth.css";

export const SEARCH_DEBOUNCE_MS = 200;
export const SEARCH_LIMIT = 8;

export interface PlaceSearchProps {
  value: string;
  onChange(text: string): void;
  onSelect(hit: PlaceHit): void;
  invalid?: boolean;
  describedBy?: string;
}

export function PlaceSearch({ value, onChange, onSelect, invalid, describedBy }: PlaceSearchProps) {
  const { t, i18n } = useTranslation();
  const id = useId();
  const [hits, setHits] = useState<PlaceHit[]>([]);
  const [open, setOpen] = useState(false);
  const [active, setActive] = useState(-1);
  const [error, setError] = useState<CommandError | null>(null);
  const [query, setQuery] = useState<string | null>(null);
  const seq = useRef(0);

  // Yalnızca kullanıcı yazınca aranır (`query`); seçim ya da dışarıdan değer araması tetiklemez.
  useEffect(() => {
    if (query === null) return;
    const mine = ++seq.current;
    if (query.trim() === "") {
      setHits([]);
      setOpen(false);
      return;
    }
    const timer = setTimeout(() => {
      void api.searchPlaces(query, SEARCH_LIMIT).then((r) => {
        if (mine !== seq.current) return;
        if (r.status === "error") {
          setError(r.error);
          setHits([]);
          setOpen(false);
          return;
        }
        setError(null);
        setHits(r.data);
        setActive(r.data.length > 0 ? 0 : -1);
        setOpen(true);
      });
    }, SEARCH_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [query]);

  const choose = (hit: PlaceHit) => {
    seq.current++;
    setQuery(null);
    setOpen(false);
    setHits([]);
    onSelect(hit);
  };

  const onKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "ArrowDown" && hits.length > 0) {
      e.preventDefault();
      setOpen(true);
      setActive((a) => (a + 1) % hits.length);
    } else if (e.key === "ArrowUp" && hits.length > 0) {
      e.preventDefault();
      setOpen(true);
      setActive((a) => (a <= 0 ? hits.length - 1 : a - 1));
    } else if (e.key === "Enter" && open && active >= 0 && hits[active]) {
      e.preventDefault();
      choose(hits[active]);
    } else if (e.key === "Escape" && open) {
      // Liste kapanır; pencere açık kalır.
      e.preventDefault();
      e.stopPropagation();
      setOpen(false);
    }
  };

  const listId = `${id}-list`;
  const statusId = `${id}-status`;
  const shown = open && hits.length > 0;
  return (
    <div className="place-search">
      <input
        id={`${id}-input`}
        role="combobox"
        aria-autocomplete="list"
        aria-expanded={shown}
        aria-controls={listId}
        aria-activedescendant={shown && active >= 0 ? `${id}-opt-${active}` : undefined}
        aria-invalid={invalid || undefined}
        aria-describedby={[statusId, describedBy].filter(Boolean).join(" ")}
        aria-label={t("birth.place")}
        autoComplete="off"
        value={value}
        onChange={(e) => {
          onChange(e.target.value);
          setQuery(e.target.value);
        }}
        onKeyDown={onKeyDown}
        onBlur={() => setOpen(false)}
      />
      <ul id={listId} role="listbox" aria-label={t("birth.placeResults")} className="place-results" hidden={!shown}>
        {hits.map((h, i) => (
          <li
            key={h.geoname_id}
            id={`${id}-opt-${i}`}
            role="option"
            aria-selected={i === active}
            className={i === active ? "is-active" : undefined}
            // mousedown: input'un blur'u listeyi kapatmadan seçilir.
            onMouseDown={(e) => {
              e.preventDefault();
              choose(h);
            }}
          >
            <span>{h.name}</span>
            <span className="place-meta">
              {[h.admin1, h.country].filter(Boolean).join(", ")} · {h.timezone}
            </span>
          </li>
        ))}
      </ul>
      <p id={statusId} className="field-hint" role="status">
        {error ? errorMessage(t, i18n, error) : open && query ? t("birth.placeCount", { count: hits.length }) : ""}
      </p>
    </div>
  );
}
```

`app/ui/birth/birth.css` oluştur:

```css
.place-search {
  position: relative;
}
.place-results {
  position: absolute;
  z-index: 5;
  left: 0;
  right: 0;
  margin: 2px 0 0;
  padding: 2px;
  list-style: none;
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: 4px;
  max-height: 240px;
  overflow: auto;
}
.place-results li {
  display: flex;
  flex-direction: column;
  padding: 2px 6px;
  cursor: default;
}
.place-results li.is-active {
  background: var(--select);
  color: var(--ink-strong);
}
.place-meta {
  color: var(--text-2);
  font-size: var(--fs-table);
}
.field-hint {
  margin: 2px 0 0;
  min-height: 1em;
  color: var(--text-2);
  font-size: var(--fs-table);
}
```

`app/ui/i18n/en.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -212,4 +212,9 @@ export const en: Messages = {
     resizePanel: "Resize the focused panel",
     tree: "Move through, open and close the dasha tree",
   },
+  birth: {
+    place: "Birth place",
+    placeResults: "Matching places",
+    placeCount: "Results: {{count}}",
+  },
 };
```

`app/ui/i18n/fr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -212,4 +212,9 @@ export const fr: Messages = {
     resizePanel: "Redimensionner le panneau actif",
     tree: "Parcourir, ouvrir et fermer l’arbre des dashas",
   },
+  birth: {
+    place: "Lieu de naissance",
+    placeResults: "Lieux trouvés",
+    placeCount: "Résultats : {{count}}",
+  },
 };
```

`app/ui/i18n/tr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -211,6 +211,11 @@ export const tr = {
     resizePanel: "Odaktaki paneli boyutlandır",
     tree: "Dasa ağacında gezin, aç, kapat",
   },
+  birth: {
+    place: "Doğum yeri",
+    placeResults: "Bulunan yerler",
+    placeCount: "{{count}} sonuç",
+  },
 };
 
 type Strings<T> = { [K in keyof T]: T[K] extends string ? string : Strings<T[K]> };
```


- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app exec vitest run ui/birth
pnpm --dir app test
pnpm --dir app build
```
Expected: tam takımda `Test Files  34 passed (34)`, `Tests  273 passed (273)`; `build` hatasız (`tsc --noEmit` çıktısız, `vite build` `✓ built`).

- [ ] **Step 5: Commit**

```bash
git add -A app/ui/birth/PlaceSearch.test.tsx app/ui/birth/PlaceSearch.tsx app/ui/birth/birth.css app/ui/i18n/en.ts app/ui/i18n/fr.ts app/ui/i18n/tr.ts
git diff --cached -U0 -- . ':!app/pnpm-lock.yaml' ':!*.png' ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git commit -m "feat(ui): accessible place search combobox over the bundled atlas

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 12: Doğum formu, boş durum ve Cmd+N / Cmd+E

Sağdan kayan doğum formu (D17) ve boş durum (D18). Form 300 ms debounce ile `preview_moment` çağırır ve çözülmüş anı (UTC, ofset, kısaltma) gösterir; DST boşluğunda öneriyi uygulayan satır içi düğme, örtüşmede "ilk kez / ikinci kez" seçimi çıkar. Kaydet yeni haritada `create_client`, düzenlemede `update_chart` çağırır ve dönen `OpenedClient`'ı `session.open` ile açar; açılış ve önceki haritanın bildirimleri (`boot`, `store:*`) kaldırılır, dönen haritanın `warnings`'i bildirime çevrilir (`save` kayıt hatası otomatik kayda aittir, yerinde durur). Düzenleme kipinde ad alanı kilitlidir; odak yine pencereye girer (Task 9 `Dialog`) ve Esc tıklamadan çalışır. Hata metinleri alana bağlıdır (`aria-describedby`, `aria-invalid`), blur'da görünür, düzeltilince kalkar. `PlaceSearch` dışarıdaki etiket için `inputId` alır.

**Files:**
- Modify: `app/ui/App.test.tsx`
- Create: `app/ui/birth/BirthForm.test.tsx`
- Create: `app/ui/birth/BirthForm.tsx`
- Modify: `app/ui/birth/PlaceSearch.tsx`
- Modify: `app/ui/birth/birth.css`
- Modify: `app/ui/i18n/en.ts`
- Modify: `app/ui/i18n/fr.ts`
- Modify: `app/ui/i18n/tr.ts`
- Create: `app/ui/shell/EmptyState.tsx`
- Modify: `app/ui/shell/Shell.tsx`
- Modify: `app/ui/shell/TopBar.tsx`
- Modify: `app/ui/shell/shell.css`

**Interfaces:**
- Consumes: `birth/model.ts` (Task 10), `PlaceSearch` (Task 11), `Dialog`, `matchShortcut` (Task 9), `useSession`, `useNotices`, `storeWarningNotices` (Task 5), `api.previewMoment`, `api.createClient`, `api.updateChart` (Task 3).
- Produces: `birth/BirthForm.tsx`: `BirthForm({mode: "new" | "edit", onClose})`, `PREVIEW_DEBOUNCE_MS = 300`; `shell/EmptyState.tsx`: `EmptyState({onNew})`; `PlaceSearchProps.inputId?`; Shell form durumu (`Cmd+N`, `Cmd+E`, üst çubuk "Yeni harita"/"Düzenle"); i18n `birth.*`, `empty.*`.

- [ ] **Step 1: Önce testleri yaz**



`app/ui/App.test.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/App.test.tsx
+++ b/app/ui/App.test.tsx
@@ -4,6 +4,10 @@ import { vi } from "vitest";
 vi.mock("./api", () => ({
   api: {
     printWindow: vi.fn(async () => ({ status: "ok", data: null })),
+    previewMoment: vi.fn(async () => ({ status: "error", error: { code: "invalid_input", params: { field: "birth.date" } } })),
+    searchPlaces: vi.fn(async () => ({ status: "ok", data: [] })),
+    createClient: vi.fn(),
+    updateChart: vi.fn(),
   },
 }));
 
@@ -30,10 +34,33 @@ test("opens on the interview layout with the restored chart", () => {
   expect(screen.queryByRole("list", { name: "Uyarılar" })).toBeNull();
 });
 
-test("without a session the desk shows the app title and empty panels", () => {
+test("without a session the desk is one empty state whose action opens the birth form", () => {
   renderApp({ opened: null });
   expect(screen.getByRole("heading", { level: 1, name: "Jyotish Desk" })).toBeInTheDocument();
-  expect(screen.getAllByText("Açık harita yok.").length).toBeGreaterThan(0);
+  const empty = screen.getByRole("region", { name: "Açık harita yok" });
+  expect(screen.queryByRole("group", { name: "D1 haritası" })).toBeNull();
+  expect(screen.queryByRole("button", { name: "Düzenle" })).toBeNull();
+  fireEvent.click(within(empty).getByRole("button", { name: "Yeni harita" }));
+  expect(screen.getByRole("dialog", { name: "Yeni harita" })).toBeInTheDocument();
+});
+
+test("Cmd+N creates a chart that replaces the empty state; Cmd+E edits it", async () => {
+  vi.mocked(api.createClient).mockResolvedValueOnce({ status: "ok", data: fixture().opened });
+  renderApp({ opened: null });
+  fireEvent.keyDown(window, { key: "n", metaKey: true });
+  const form = screen.getByRole("dialog", { name: "Yeni harita" });
+  fireEvent.change(within(form).getByLabelText("Danışan adı"), { target: { value: "Örnek Danışan" } });
+  fireEvent.change(within(form).getByLabelText("Tarih"), { target: { value: "2000-01-01" } });
+  fireEvent.change(within(form).getByLabelText("Saat"), { target: { value: "17:30" } });
+  fireEvent.change(within(form).getByLabelText("Enlem"), { target: { value: "28.6139" } });
+  fireEvent.change(within(form).getByLabelText("Boylam"), { target: { value: "77.209" } });
+  fireEvent.change(within(form).getByLabelText("Bölge adı"), { target: { value: "Asia/Kolkata" } });
+  fireEvent.click(within(form).getByRole("button", { name: "Haritayı kaydet" }));
+  expect(await screen.findByRole("group", { name: "D1 haritası" })).toBeInTheDocument();
+  expect(screen.queryByRole("dialog")).toBeNull();
+  expect(screen.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeInTheDocument();
+  fireEvent.keyDown(window, { key: "e", metaKey: true });
+  expect(screen.getByRole("dialog", { name: "Haritayı düzenle" })).toBeInTheDocument();
 });
 
 test("moment warnings are badges with a text label and a hint", () => {
```

`app/ui/birth/BirthForm.test.tsx` oluştur:

```tsx
import { act, fireEvent, screen, within } from "@testing-library/react";
import { useEffect } from "react";
import { vi } from "vitest";

vi.mock("../api", () => ({
  api: { searchPlaces: vi.fn(), previewMoment: vi.fn(), createClient: vi.fn(), updateChart: vi.fn() },
}));

import { api } from "../api";
import { useSession } from "../session/store";
import { fixture } from "../test/fixture";
import { NoticeBar, useNotices, type Notice } from "../shell/notices";
import { renderWithStores } from "../test/render";
import { BirthForm, PREVIEW_DEBOUNCE_MS } from "./BirthForm";
import { SEARCH_DEBOUNCE_MS } from "./PlaceSearch";

function OpenedName() {
  const { opened } = useSession();
  return <output aria-label="açık">{opened?.client.name ?? "-"}</output>;
}

const onClose = vi.fn();
function renderForm(mode: "new" | "edit", opened = mode === "edit" ? fixture().opened : null) {
  return renderWithStores(
    <>
      <OpenedName />
      <BirthForm mode={mode} onClose={onClose} />
    </>,
    { opened },
  );
}

const dialog = () => screen.getByRole("dialog");
const input = (name: string | RegExp) => within(dialog()).getByLabelText(name);
const wait = (ms: number) => act(async () => vi.advanceTimersByTime(ms));

async function fillDelhi() {
  fireEvent.change(input("Danışan adı"), { target: { value: "Ayşe" } });
  fireEvent.change(input("Tarih"), { target: { value: "2000-01-01" } });
  fireEvent.change(input("Saat"), { target: { value: "17:30" } });
  fireEvent.change(input("Doğum yeri"), { target: { value: "new delhi" } });
  await wait(SEARCH_DEBOUNCE_MS);
  fireEvent.keyDown(input("Doğum yeri"), { key: "Enter" });
  await wait(PREVIEW_DEBOUNCE_MS);
}

beforeEach(() => {
  vi.useFakeTimers();
  onClose.mockReset();
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "ok", data: fixture().places });
  vi.mocked(api.previewMoment).mockResolvedValue({ status: "ok", data: fixture().preview });
  vi.mocked(api.createClient).mockResolvedValue({ status: "ok", data: fixture().opened });
  vi.mocked(api.updateChart).mockResolvedValue({ status: "ok", data: fixture().opened });
});
afterEach(() => vi.useRealTimers());

test("new chart: atlas place, live preview, save opens the returned chart", async () => {
  renderForm("new");
  expect(dialog()).toHaveAccessibleName("Yeni harita");
  expect(document.activeElement).toBe(input("Danışan adı"));
  await fillDelhi();
  expect(input("Bölge adı")).toHaveValue("Asia/Kolkata");
  expect(input("Enlem")).toHaveValue("28.62137");
  expect(within(dialog()).getByRole("region", { name: "Önizleme" })).toHaveTextContent("UTC 2000-01-01 12:00:00 · IST +05:30");
  fireEvent.click(within(dialog()).getByRole("button", { name: "Haritayı kaydet" }));
  await wait(0);
  expect(api.createClient).toHaveBeenCalledWith({
    name: "Ayşe",
    gender: null,
    chart: {
      label: "",
      kind: "Natal",
      birth: {
        date: "2000-01-01",
        time: "17:30",
        time_certainty: "Exact",
        place: { name: "New Delhi", country: "India", admin1: "Delhi", lat: 28.62137, lon: 77.2148, geoname_id: 1261481 },
        tz: { Iana: "Asia/Kolkata" },
        fold: null,
        notes: "",
      },
    },
  });
  expect(screen.getByRole("status", { name: "açık" })).toHaveTextContent("Örnek Danışan");
  expect(onClose).toHaveBeenCalledOnce();
});

test("format errors show on blur and submit, clear on edit, and nothing is sent", async () => {
  renderForm("new");
  fireEvent.blur(input("Danışan adı"));
  expect(input("Danışan adı")).toHaveAttribute("aria-invalid", "true");
  expect(input("Danışan adı")).toHaveAccessibleDescription("Bu alan gerekli.");
  fireEvent.change(input("Danışan adı"), { target: { value: "A" } });
  expect(input("Danışan adı")).not.toHaveAttribute("aria-invalid");
  fireEvent.change(input("Enlem"), { target: { value: "kuzey" } });
  fireEvent.click(within(dialog()).getByRole("button", { name: "Haritayı kaydet" }));
  await wait(0);
  expect(input("Tarih")).toHaveAttribute("aria-invalid", "true");
  expect(input("Enlem")).toHaveAccessibleDescription("Ondalık sayı girin (ör. 28.61). Ondalık derece; kuzey ve doğu pozitif.");
  expect(api.createClient).not.toHaveBeenCalled();
  expect(api.previewMoment).not.toHaveBeenCalled();
});

test("a DST gap is shown on the time field with a one-click fix", async () => {
  vi.mocked(api.previewMoment).mockResolvedValueOnce({ status: "error", error: fixture().gap_error });
  renderForm("new");
  await fillDelhi();
  expect(input("Saat")).toHaveAttribute("aria-invalid", "true");
  expect(input("Saat")).toHaveAccessibleDescription(/2014-03-31T03:30:00 yerel saatte yok/);
  fireEvent.click(within(dialog()).getByRole("button", { name: "04:30 kullan" }));
  expect(input("Saat")).toHaveValue("04:30");
  expect(input("Tarih")).toHaveValue("2014-03-31");
});

test("an ambiguous time asks which occurrence and sends the choice", async () => {
  vi.mocked(api.previewMoment).mockResolvedValueOnce({
    status: "error",
    error: { code: "ambiguous_local_time", params: { local: "2014-10-26T03:30:00", earlier_offset_secs: "10800", later_offset_secs: "7200" } },
  });
  renderForm("new");
  await fillDelhi();
  const group = within(dialog()).getByRole("group", { name: "Bu saat iki kez yaşandı" });
  fireEvent.click(within(group).getByRole("radio", { name: "İkinci kez (geçişten sonra)" }));
  await wait(PREVIEW_DEBOUNCE_MS);
  expect(vi.mocked(api.previewMoment).mock.lastCall![0].fold).toBe("Later");
});

test("edit mode starts from the stored chart and saves with update_chart", async () => {
  renderForm("edit");
  expect(dialog()).toHaveAccessibleName("Haritayı düzenle");
  expect(input("Danışan adı")).toBeDisabled();
  // Ad alanı kilitli: odak yine pencereye girer, Esc tıklamadan çalışır.
  expect(dialog()).toContainElement(document.activeElement as HTMLElement);
  expect(document.activeElement).not.toBe(input("Danışan adı"));
  fireEvent.keyDown(document.activeElement!, { key: "Escape" });
  expect(onClose).toHaveBeenCalledOnce();
  onClose.mockReset();
  expect(input("Saat")).toHaveValue("17:30");
  fireEvent.change(input("Harita etiketi"), { target: { value: "Düzeltilmiş" } });
  fireEvent.click(within(dialog()).getByRole("button", { name: "Haritayı kaydet" }));
  await wait(0);
  const { chart_record } = fixture().opened;
  expect(api.updateChart).toHaveBeenCalledWith(chart_record.id, { label: "Düzeltilmiş", kind: "Natal", birth: expect.objectContaining({ date: "2000-01-01", time: "17:30" }) });
  expect(api.createClient).not.toHaveBeenCalled();
});

test("saving replaces boot and store notices with the warnings of the returned chart", async () => {
  const stale: Notice[] = [
    { id: "boot", type: "error", messageKey: "notice.restoreFailed", error: { code: "store_sqlite", params: {} } },
    { id: "store:TzdbChanged", type: "warning", messageKey: "storeWarning.TzdbChanged", params: { stored: "2025a", current: "2025b" } },
    { id: "save", type: "error", messageKey: "notice.saveFailed", error: { code: "store_sqlite", params: {} } },
  ];
  function Seed() {
    const { push } = useNotices();
    useEffect(() => stale.forEach(push), [push]);
    return null;
  }
  const returned = fixture().opened;
  returned.warnings = ["LastChartMissing"];
  vi.mocked(api.updateChart).mockResolvedValueOnce({ status: "ok", data: returned });
  renderWithStores(
    <>
      <Seed />
      <NoticeBar />
      <BirthForm mode="edit" onClose={onClose} />
    </>,
    { opened: fixture().opened },
  );
  const bar = () => screen.getByRole("region", { name: "Bildirimler" });
  expect(within(bar()).getAllByRole("status")).toHaveLength(1);
  fireEvent.click(within(dialog()).getByRole("button", { name: "Haritayı kaydet" }));
  await wait(0);
  expect(onClose).toHaveBeenCalledOnce();
  expect(within(bar()).getByRole("status")).toHaveTextContent("Son açılan harita silinmiş");
  expect(bar()).not.toHaveTextContent("Saat dilimi verisi değişti");
  expect(bar()).not.toHaveTextContent("Son oturum açılamadı");
  // Kayıt hatası bildirimi otomatik kayda aittir; form onu kaldırmaz.
  expect(within(bar()).getAllByRole("alert")).toHaveLength(1);
});

test("save errors land on their field or above the buttons; Esc closes", async () => {
  vi.mocked(api.createClient).mockResolvedValueOnce({ status: "error", error: { code: "invalid_input", params: { field: "birth.date" } } });
  renderForm("new");
  await fillDelhi();
  fireEvent.click(within(dialog()).getByRole("button", { name: "Haritayı kaydet" }));
  await wait(0);
  expect(input("Tarih")).toHaveAccessibleDescription("Geçersiz değer: birth.date.");
  vi.mocked(api.createClient).mockResolvedValueOnce({ status: "error", error: { code: "store_sqlite", params: { detail: "disk" } } });
  fireEvent.click(within(dialog()).getByRole("button", { name: "Haritayı kaydet" }));
  await wait(0);
  expect(within(dialog()).getByRole("alert")).toHaveTextContent("Kaydedilemedi: Danışan veritabanı okunamadı ya da yazılamadı (disk).");
  expect(onClose).not.toHaveBeenCalled();
  fireEvent.keyDown(input("Tarih"), { key: "Escape" });
  expect(onClose).toHaveBeenCalledOnce();
});
```


- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/birth/BirthForm.test.tsx ui/App.test.tsx
```
Expected: FAIL — `Failed to resolve import "./BirthForm"`; boş durumda "Yeni harita" düğmesi yok.

- [ ] **Step 3: Uygulamayı yaz**



`app/ui/birth/BirthForm.tsx` oluştur:

```tsx
// Doğum verisi girişi (spec §4 "Yeni danışan/harita" ve "Düzenle"). Sağdan kayan modal
// pencere. Form biçimi `model.ts`'de denetlenir; an Rust'ta çözülür: alanlar değiştikçe
// 300 ms sonra `preview_moment` çağrılır (UTC, ofset, uyarılar, DST boşluğu/örtüşmesi).
// Kayıt `create_client` (yeni) ya da `update_chart` (düzenle) ile yapılır; dönen harita açılır.
// Açılış ve önceki haritanın bildirimleri (`boot`, `store:*`) kalkar; yeni haritanın store
// uyarıları gösterilir.
import { useEffect, useId, useRef, useState, type FormEvent, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CommandError, MomentPreview } from "../bindings";
import { errorMessage } from "../i18n/errors";
import { useSession } from "../session/store";
import { Dialog } from "../shell/Dialog";
import { storeWarningNotices, useNotices } from "../shell/notices";
import "./birth.css";
import {
  applyPlace,
  applySuggestion,
  draftFromOpened,
  emptyDraft,
  errorField,
  formatFixedOffset,
  setCoordinate,
  toBirthData,
  toNewChart,
  toNewClient,
  type BirthDraft,
  type BirthField,
  type FieldErrors,
  type TzMode,
} from "./model";
import { PlaceSearch } from "./PlaceSearch";

export const PREVIEW_DEBOUNCE_MS = 300;

export interface BirthFormProps {
  mode: "new" | "edit";
  onClose(): void;
}

export function BirthForm({ mode, onClose }: BirthFormProps) {
  const { t, i18n } = useTranslation();
  const session = useSession();
  const id = useId();
  const notices = useNotices();
  const editing = mode === "edit" && session.opened !== null;
  const [draft, setDraft] = useState<BirthDraft>(() => (editing ? draftFromOpened(session.opened!) : emptyDraft()));
  const [shown, setShown] = useState<FieldErrors>({});
  const [preview, setPreview] = useState<{ ok: MomentPreview } | { error: CommandError } | null>(null);
  const [formError, setFormError] = useState<CommandError | null>(null);
  const [saving, setSaving] = useState(false);
  const seq = useRef(0);

  // Önizleme: biçimi tamam olan her taslak için; eski yanıt yok sayılır.
  const checked = toBirthData(draft);
  const previewKey = checked.birth ? JSON.stringify(checked.birth) : null;
  useEffect(() => {
    const mine = ++seq.current;
    if (previewKey === null) {
      setPreview(null);
      return;
    }
    const timer = setTimeout(() => {
      void api.previewMoment(JSON.parse(previewKey)).then((r) => {
        if (mine !== seq.current) return;
        setPreview(r.status === "ok" ? { ok: r.data } : { error: r.error });
      });
    }, PREVIEW_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [previewKey]);

  const update = (patch: Partial<BirthDraft>, field?: BirthField) => {
    setDraft((d) => ({ ...d, ...patch }));
    // Alan düzeltilince hatası hemen kalkar (spec §7: blur-first, anında temizle).
    if (field) setShown((s) => ({ ...s, [field]: undefined }));
  };

  // Önizleme hatası alana bağlanır; biçim hatası yalnızca dokunulan/gönderilen alanda görünür.
  const previewError = preview && "error" in preview ? preview.error : null;
  const previewField = previewError ? errorField(previewError) : null;
  const fieldError = (f: BirthField): string | null => {
    const e = shown[f] ?? (previewField === f && previewError ? { error: previewError } : undefined);
    if (!e) return null;
    return "key" in e ? t(e.key) : errorMessage(t, i18n, e.error);
  };
  const blur = (f: BirthField) => () => {
    const e = toBirthData(draft).errors?.[f];
    if (e) setShown((s) => ({ ...s, [f]: e }));
  };

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    setFormError(null);
    const { birth, errors } = toBirthData(draft);
    if (!birth) {
      setShown(errors);
      return;
    }
    setSaving(true);
    const r =
      editing && session.opened
        ? await api.updateChart(session.opened.chart_record.id, toNewChart(draft, birth))
        : await api.createClient(toNewClient(draft, birth));
    setSaving(false);
    if (r.status === "ok") {
      for (const n of notices.notices) if (n.id === "boot" || n.id.startsWith("store:")) notices.dismiss(n.id);
      for (const n of storeWarningNotices(r.data.warnings)) notices.push(n);
      session.open(r.data);
      onClose();
      return;
    }
    const f = errorField(r.error);
    if (f) setShown((s) => ({ ...s, [f]: { error: r.error } }));
    else setFormError(r.error);
  };

  const field = (f: BirthField, label: string, input: (p: { id: string; "aria-invalid"?: true; "aria-describedby"?: string }) => ReactNode, hint?: string) => {
    const error = fieldError(f);
    const inputId = `${id}-${f}`;
    const hintId = hint ? `${inputId}-hint` : undefined;
    const errorId = error ? `${inputId}-error` : undefined;
    return (
      <div className="form-field">
        <label htmlFor={inputId}>{label}</label>
        {input({ id: inputId, "aria-invalid": error ? true : undefined, "aria-describedby": [errorId, hintId].filter(Boolean).join(" ") || undefined })}
        {hint && (
          <p id={hintId} className="field-hint">
            {hint}
          </p>
        )}
        {error && (
          <p id={errorId} className="field-error">
            <span aria-hidden="true">▲ </span>
            {error}
          </p>
        )}
      </div>
    );
  };

  const suggestion = previewError?.code === "nonexistent_local_time" ? previewError.params.suggestion : null;
  const ambiguous = previewError?.code === "ambiguous_local_time" || draft.fold !== null;

  return (
    <Dialog title={editing ? t("birth.titleEdit") : t("birth.titleNew")} onClose={onClose} variant="sheet">
      <form className="birth-form" noValidate onSubmit={(e) => void submit(e)}>
        {field("name", t("birth.name"), (p) => (
          <input {...p} value={draft.name} disabled={editing} data-autofocus onChange={(e) => update({ name: e.target.value }, "name")} onBlur={blur("name")} />
        ))}
        {field("label", t("birth.label"), (p) => (
          <input {...p} value={draft.label} onChange={(e) => update({ label: e.target.value }, "label")} />
        ))}
        <div className="form-field">
          <label htmlFor={`${id}-kind`}>{t("birth.kind")}</label>
          <select id={`${id}-kind`} value={draft.kind} onChange={(e) => update({ kind: e.target.value === "Event" ? "Event" : "Natal" })}>
            <option value="Natal">{t("chart.kind.Natal")}</option>
            <option value="Event">{t("chart.kind.Event")}</option>
          </select>
        </div>
        <div className="form-row">
          {field("date", t("birth.date"), (p) => (
            <input {...p} type="date" value={draft.date} onChange={(e) => update({ date: e.target.value }, "date")} onBlur={blur("date")} />
          ))}
          {field("time", t("birth.time"), (p) => (
            <input
              {...p}
              type="time"
              value={draft.time}
              disabled={draft.certainty === "Unknown"}
              onChange={(e) => update({ time: e.target.value, fold: null }, "time")}
              onBlur={blur("time")}
            />
          ))}
        </div>
        {suggestion && (
          <p className="field-fix">
            <button type="button" onClick={() => update(applySuggestion(draft, suggestion), "time")}>
              {t("birth.useSuggestion", { time: suggestion.slice(11, 16) })}
            </button>
          </p>
        )}
        <fieldset className="form-field">
          <legend>{t("birth.certainty")}</legend>
          {(["Exact", "Approx", "Unknown"] as const).map((c) => (
            <label key={c} className="choice">
              <input type="radio" name={`${id}-certainty`} checked={draft.certainty === c} onChange={() => update({ certainty: c }, "time")} />
              {t(`birth.certainty${c}`)}
            </label>
          ))}
        </fieldset>
        {ambiguous && (
          <fieldset className="form-field" aria-invalid={previewField === "fold" || undefined}>
            <legend>{t("birth.fold")}</legend>
            {(["Earlier", "Later"] as const).map((f) => (
              <label key={f} className="choice">
                <input type="radio" name={`${id}-fold`} checked={draft.fold === f} onChange={() => update({ fold: f }, "fold")} />
                {t(`birth.fold${f}`)}
              </label>
            ))}
            {fieldError("fold") && <p className="field-error">{fieldError("fold")}</p>}
          </fieldset>
        )}
        <div className="form-field">
          <label htmlFor={`${id}-place`}>{t("birth.place")}</label>
          <PlaceSearch
            inputId={`${id}-place`}
            value={draft.placeName}
            onChange={(text) => update({ placeName: text, geonameId: null })}
            onSelect={(hit) => {
              setDraft((d) => applyPlace(d, hit));
              setShown((s) => ({ ...s, lat: undefined, lon: undefined, tz: undefined }));
            }}
          />
        </div>
        <div className="form-row">
          {field(
            "lat",
            t("birth.lat"),
            (p) => <input {...p} inputMode="decimal" value={draft.lat} onChange={(e) => update(setCoordinate(draft, "lat", e.target.value), "lat")} onBlur={blur("lat")} />,
            t("birth.coordHint"),
          )}
          {field("lon", t("birth.lon"), (p) => (
            <input {...p} inputMode="decimal" value={draft.lon} onChange={(e) => update(setCoordinate(draft, "lon", e.target.value), "lon")} onBlur={blur("lon")} />
          ))}
        </div>
        <div className="form-row">
          <div className="form-field">
            <label htmlFor={`${id}-tzmode`}>{t("birth.tzMode")}</label>
            <select
              id={`${id}-tzmode`}
              value={draft.tzMode}
              onChange={(e) => update({ tzMode: e.target.value as TzMode, fold: null }, "tz")}
            >
              <option value="iana">{t("birth.tzIana")}</option>
              <option value="fixed">{t("birth.tzFixed")}</option>
              <option value="lmt">{t("birth.tzLmt")}</option>
            </select>
          </div>
          {draft.tzMode === "iana" &&
            field("tz", t("birth.tzIanaName"), (p) => (
              <input {...p} value={draft.tzIana} placeholder="Europe/Istanbul" onChange={(e) => update({ tzIana: e.target.value, fold: null }, "tz")} onBlur={blur("tz")} />
            ))}
          {draft.tzMode === "fixed" &&
            field("tz", t("birth.tzFixedValue"), (p) => (
              <input {...p} value={draft.tzFixed} placeholder="+05:30" onChange={(e) => update({ tzFixed: e.target.value }, "tz")} onBlur={blur("tz")} />
            ))}
        </div>
        <div className="form-field">
          <label htmlFor={`${id}-notes`}>{t("birth.notes")}</label>
          <textarea id={`${id}-notes`} rows={2} value={draft.notes} onChange={(e) => update({ notes: e.target.value })} />
        </div>
        <section className="birth-preview" aria-label={t("birth.preview")} aria-live="polite">
          {preview && "ok" in preview && (
            <>
              <p>
                {t("birth.previewValue", {
                  utc: preview.ok.resolved.utc.replace("T", " ").replace("Z", ""),
                  abbrev: preview.ok.resolved.abbrev,
                  offset: formatFixedOffset(preview.ok.resolved.offset_secs),
                })}
              </p>
              {preview.ok.resolved.warnings.map((w) => (
                <p key={w} className="field-hint">
                  <span aria-hidden="true">▲ </span>
                  {t(`badgeHint.${w}`)}
                </p>
              ))}
            </>
          )}
          {previewError && previewField === null && <p className="field-error">{errorMessage(t, i18n, previewError)}</p>}
        </section>
        {formError && (
          <p className="field-error" role="alert">
            {t("birth.saveFailed", { message: errorMessage(t, i18n, formError) })}
          </p>
        )}
        <div className="dialog-actions">
          <button type="button" onClick={onClose}>
            {t("birth.cancel")}
          </button>
          <button type="submit" disabled={saving}>
            {t("birth.save")}
          </button>
        </div>
      </form>
    </Dialog>
  );
}
```

`app/ui/birth/PlaceSearch.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/birth/PlaceSearch.tsx
+++ b/app/ui/birth/PlaceSearch.tsx
@@ -17,9 +17,11 @@ export interface PlaceSearchProps {
   onSelect(hit: PlaceHit): void;
   invalid?: boolean;
   describedBy?: string;
+  /** Dışarıdaki `<label htmlFor>` için; verilmezse kutu kendi `aria-label`'ını taşır. */
+  inputId?: string;
 }
 
-export function PlaceSearch({ value, onChange, onSelect, invalid, describedBy }: PlaceSearchProps) {
+export function PlaceSearch({ value, onChange, onSelect, invalid, describedBy, inputId }: PlaceSearchProps) {
   const { t, i18n } = useTranslation();
   const id = useId();
   const [hits, setHits] = useState<PlaceHit[]>([]);
@@ -90,7 +92,7 @@ export function PlaceSearch({ value, onChange, onSelect, invalid, describedBy }:
   return (
     <div className="place-search">
       <input
-        id={`${id}-input`}
+        id={inputId ?? `${id}-input`}
         role="combobox"
         aria-autocomplete="list"
         aria-expanded={shown}
@@ -98,7 +100,7 @@ export function PlaceSearch({ value, onChange, onSelect, invalid, describedBy }:
         aria-activedescendant={shown && active >= 0 ? `${id}-opt-${active}` : undefined}
         aria-invalid={invalid || undefined}
         aria-describedby={[statusId, describedBy].filter(Boolean).join(" ")}
-        aria-label={t("birth.place")}
+        aria-label={inputId ? undefined : t("birth.place")}
         autoComplete="off"
         value={value}
         onChange={(e) => {
```

`app/ui/birth/birth.css` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/birth/birth.css
+++ b/app/ui/birth/birth.css
@@ -35,3 +35,51 @@
   color: var(--text-2);
   font-size: var(--fs-table);
 }
+.birth-form {
+  display: flex;
+  flex-direction: column;
+  gap: 8px;
+}
+.form-field {
+  display: flex;
+  flex-direction: column;
+  gap: 2px;
+  margin: 0;
+  padding: 0;
+  border: 0;
+  min-width: 0;
+}
+.form-field legend {
+  padding: 0;
+  margin-bottom: 2px;
+}
+.form-row {
+  display: flex;
+  gap: 8px;
+}
+.form-row > .form-field {
+  flex: 1;
+}
+.choice {
+  display: inline-flex;
+  align-items: center;
+  gap: 4px;
+  margin-right: 12px;
+}
+.field-error {
+  margin: 2px 0 0;
+  color: var(--danger);
+  font-size: var(--fs-table);
+}
+.field-fix {
+  margin: 0;
+}
+.birth-preview {
+  min-height: 2em;
+  padding: 4px 8px;
+  border: 1px solid var(--line);
+  border-radius: 4px;
+}
+.birth-preview p {
+  margin: 0;
+}
```

`app/ui/i18n/en.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -216,5 +216,46 @@ export const en: Messages = {
     place: "Birth place",
     placeResults: "Matching places",
     placeCount: "Results: {{count}}",
+    titleNew: "New chart",
+    titleEdit: "Edit chart",
+    edit: "Edit",
+    name: "Client name",
+    label: "Chart label",
+    kind: "Chart type",
+    date: "Date",
+    time: "Time",
+    certainty: "Time accuracy",
+    certaintyExact: "Exact",
+    certaintyApprox: "Approximate",
+    certaintyUnknown: "Unknown",
+    fold: "This time occurred twice",
+    foldEarlier: "First time (before the change)",
+    foldLater: "Second time (after the change)",
+    lat: "Latitude",
+    lon: "Longitude",
+    coordHint: "Decimal degrees; north and east are positive.",
+    tzMode: "Time zone",
+    tzIana: "Region (IANA)",
+    tzFixed: "Fixed offset",
+    tzLmt: "Local mean time (LMT)",
+    tzIanaName: "Region name",
+    tzFixedValue: "Offset (+05:30)",
+    notes: "Source note",
+    preview: "Preview",
+    previewValue: "UTC {{utc}} · {{abbrev}} {{offset}}",
+    useSuggestion: "Use {{time}}",
+    save: "Save chart",
+    cancel: "Cancel",
+    saveFailed: "Could not save: {{message}}",
+    error: {
+      required: "This field is required.",
+      number: "Enter a decimal number (e.g. 28.61).",
+      offset: "Enter the offset as +05:30.",
+    },
+  },
+  empty: {
+    title: "No chart is open",
+    body: "Start by entering the client's birth data.",
+    action: "New chart",
   },
 };
```

`app/ui/i18n/fr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -216,5 +216,46 @@ export const fr: Messages = {
     place: "Lieu de naissance",
     placeResults: "Lieux trouvés",
     placeCount: "Résultats : {{count}}",
+    titleNew: "Nouvelle carte",
+    titleEdit: "Modifier la carte",
+    edit: "Modifier",
+    name: "Nom du consultant",
+    label: "Libellé de la carte",
+    kind: "Type de carte",
+    date: "Date",
+    time: "Heure",
+    certainty: "Précision de l’heure",
+    certaintyExact: "Exacte",
+    certaintyApprox: "Approximative",
+    certaintyUnknown: "Inconnue",
+    fold: "Cette heure a eu lieu deux fois",
+    foldEarlier: "Première fois (avant le changement)",
+    foldLater: "Seconde fois (après le changement)",
+    lat: "Latitude",
+    lon: "Longitude",
+    coordHint: "Degrés décimaux ; nord et est positifs.",
+    tzMode: "Fuseau horaire",
+    tzIana: "Région (IANA)",
+    tzFixed: "Décalage fixe",
+    tzLmt: "Temps moyen local (TML)",
+    tzIanaName: "Nom de la région",
+    tzFixedValue: "Décalage (+05:30)",
+    notes: "Note de source",
+    preview: "Aperçu",
+    previewValue: "UTC {{utc}} · {{abbrev}} {{offset}}",
+    useSuggestion: "Utiliser {{time}}",
+    save: "Enregistrer la carte",
+    cancel: "Annuler",
+    saveFailed: "Enregistrement impossible : {{message}}",
+    error: {
+      required: "Ce champ est obligatoire.",
+      number: "Saisissez un nombre décimal (ex. 28.61).",
+      offset: "Saisissez le décalage sous la forme +05:30.",
+    },
+  },
+  empty: {
+    title: "Aucune carte ouverte",
+    body: "Commencez par saisir les données de naissance du consultant.",
+    action: "Nouvelle carte",
   },
 };
```

`app/ui/i18n/tr.ts` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -215,6 +215,47 @@ export const tr = {
     place: "Doğum yeri",
     placeResults: "Bulunan yerler",
     placeCount: "{{count}} sonuç",
+    titleNew: "Yeni harita",
+    titleEdit: "Haritayı düzenle",
+    edit: "Düzenle",
+    name: "Danışan adı",
+    label: "Harita etiketi",
+    kind: "Harita türü",
+    date: "Tarih",
+    time: "Saat",
+    certainty: "Saat kesinliği",
+    certaintyExact: "Kesin",
+    certaintyApprox: "Yaklaşık",
+    certaintyUnknown: "Bilinmiyor",
+    fold: "Bu saat iki kez yaşandı",
+    foldEarlier: "İlk kez (geçişten önce)",
+    foldLater: "İkinci kez (geçişten sonra)",
+    lat: "Enlem",
+    lon: "Boylam",
+    coordHint: "Ondalık derece; kuzey ve doğu pozitif.",
+    tzMode: "Saat dilimi",
+    tzIana: "Bölge (IANA)",
+    tzFixed: "Sabit ofset",
+    tzLmt: "Yerel ortalama saat (LMT)",
+    tzIanaName: "Bölge adı",
+    tzFixedValue: "Ofset (+05:30)",
+    notes: "Kaynak notu",
+    preview: "Önizleme",
+    previewValue: "UTC {{utc}} · {{abbrev}} {{offset}}",
+    useSuggestion: "{{time}} kullan",
+    save: "Haritayı kaydet",
+    cancel: "Vazgeç",
+    saveFailed: "Kaydedilemedi: {{message}}",
+    error: {
+      required: "Bu alan gerekli.",
+      number: "Ondalık sayı girin (ör. 28.61).",
+      offset: "Ofseti +05:30 biçiminde girin.",
+    },
+  },
+  empty: {
+    title: "Açık harita yok",
+    body: "Danışanın doğum bilgisini girerek başlayın.",
+    action: "Yeni harita",
   },
 };
 
```

`app/ui/shell/EmptyState.tsx` oluştur:

```tsx
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
```

`app/ui/shell/Shell.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/Shell.tsx
+++ b/app/ui/shell/Shell.tsx
@@ -1,8 +1,11 @@
 import { useEffect, useLayoutEffect, useRef, useState } from "react";
+import { BirthForm } from "../birth/BirthForm";
 import { Dashboard } from "../dashboard/Dashboard";
+import { useSession } from "../session/store";
 import { matchShortcut } from "../shortcuts";
 import { useLayoutStore } from "../view/store";
 import { applyTheme, loadThemePref, saveThemePref, type ThemePref } from "../theme/theme";
+import { EmptyState } from "./EmptyState";
 import { HelpDialog } from "./HelpDialog";
 import { NoticeBar } from "./notices";
 import { PrintView } from "./PrintView";
@@ -15,6 +18,8 @@ export function Shell() {
   const [themePref, setThemePref] = useState<ThemePref>(() => loadThemePref());
   const [printView, setPrintView] = useState(false);
   const [help, setHelp] = useState(false);
+  const [form, setForm] = useState<"new" | "edit" | null>(null);
+  const { opened } = useSession();
   const measureRef = useRef<HTMLDivElement>(null);
   const width = useElementWidth(measureRef);
 
@@ -36,11 +41,17 @@ export function Shell() {
       } else if (command?.id === "help") {
         e.preventDefault();
         setHelp(true);
+      } else if (command?.id === "newChart") {
+        e.preventDefault();
+        setForm("new");
+      } else if (command?.id === "editChart" && opened) {
+        e.preventDefault();
+        setForm("edit");
       }
     };
     window.addEventListener("keydown", onKey);
     return () => window.removeEventListener("keydown", onKey);
-  }, [toggleNotes, printView]);
+  }, [toggleNotes, printView, opened]);
 
   return (
     <div className="shell">
@@ -50,12 +61,15 @@ export function Shell() {
         printView={printView}
         onTogglePrint={() => setPrintView((v) => !v)}
         onHelp={() => setHelp(true)}
+        onNewChart={() => setForm("new")}
+        onEditChart={opened ? () => setForm("edit") : undefined}
       />
       <NoticeBar />
       <main className="shell-main">
-        <div ref={measureRef}>{printView ? <PrintView /> : <Dashboard width={width} />}</div>
+        <div ref={measureRef}>{printView ? <PrintView /> : opened ? <Dashboard width={width} /> : <EmptyState onNew={() => setForm("new")} />}</div>
       </main>
       {help && <HelpDialog onClose={() => setHelp(false)} />}
+      {form && <BirthForm mode={form} onClose={() => setForm(null)} />}
     </div>
   );
 }
```

`app/ui/shell/TopBar.tsx` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/TopBar.tsx
+++ b/app/ui/shell/TopBar.tsx
@@ -10,15 +10,28 @@ export interface TopBarProps {
   printView: boolean;
   onTogglePrint(): void;
   onHelp(): void;
+  onNewChart(): void;
+  /** Açık harita yoksa verilmez (düğme gösterilmez). */
+  onEditChart?: () => void;
 }
 
 /** Üst çubuk (spec §4): solda danışan ve harita (uyarı rozetleriyle), ortada düzen seçici, sağda tema, dil ve baskı. */
-export function TopBar({ themePref, onThemeChange, printView, onTogglePrint, onHelp }: TopBarProps) {
+export function TopBar({ themePref, onThemeChange, printView, onTogglePrint, onHelp, onNewChart, onEditChart }: TopBarProps) {
   const { t, i18n } = useTranslation();
   const language = isLanguage(i18n.language) ? i18n.language : DEFAULT_LANGUAGE;
   return (
     <header className="topbar no-print">
       <ChartTitle />
+      <div className="topbar-actions">
+        <button type="button" aria-keyshortcuts="Meta+N" onClick={onNewChart}>
+          {t("empty.action")}
+        </button>
+        {onEditChart && (
+          <button type="button" aria-keyshortcuts="Meta+E" onClick={onEditChart}>
+            {t("birth.edit")}
+          </button>
+        )}
+      </div>
       <div className="topbar-center">{!printView && <LayoutPicker />}</div>
       <div className="topbar-right">
         <label>
```

`app/ui/shell/shell.css` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/ui/shell/shell.css
+++ b/app/ui/shell/shell.css
@@ -188,3 +188,20 @@ kbd {
   border-radius: 3px;
   padding: 0 4px;
 }
+.empty-state {
+  max-width: 420px;
+  margin: 15vh auto 0;
+  text-align: center;
+  color: var(--text);
+}
+.empty-state h2 {
+  font-size: var(--fs-panel-title);
+  font-weight: var(--fw-medium);
+}
+.empty-state p {
+  color: var(--text-2);
+}
+.topbar-actions {
+  display: flex;
+  gap: 6px;
+}
```


- [ ] **Step 4: Testlerin geçtiğini gör**

```bash
pnpm --dir app exec vitest run ui/birth ui/App.test.tsx
pnpm --dir app test
pnpm --dir app build
```
Expected: tam takımda `Test Files  35 passed (35)`, `Tests  281 passed (281)`; `build` hatasız (`tsc --noEmit` çıktısız, `vite build` `✓ built`).

- [ ] **Step 5: Commit**

```bash
git add -A app/ui/App.test.tsx app/ui/birth/BirthForm.test.tsx app/ui/birth/BirthForm.tsx app/ui/birth/PlaceSearch.tsx app/ui/birth/birth.css app/ui/i18n/en.ts app/ui/i18n/fr.ts app/ui/i18n/tr.ts app/ui/shell/EmptyState.tsx app/ui/shell/Shell.tsx app/ui/shell/TopBar.tsx app/ui/shell/shell.css
git diff --cached -U0 -- . ':!app/pnpm-lock.yaml' ':!*.png' ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git commit -m "feat(ui): birth form sheet with live moment preview, DST fixes and field errors; empty state and Cmd+N/Cmd+E

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 13: Playwright WebKit uçtan uca testler, yerel görsel temeller ve CI

UI spec §8 e2e senaryoları (D20) ve ekran görüntüsü testleri (D19b). Tauri IPC `window.__TAURI_INTERNALS__.invoke` taklidiyle değiştirilir (`e2e/ipc.ts`); taklit gerçek veri fikstürünü döner ve çağrıları sayfa belleğinde tutar. `e2e` projesi CI'da `core-macos` işinde çalışır (WebKit `actions/cache` ile `~/Library/Caches/ms-playwright`'ta, anahtar `app/pnpm-lock.yaml` özeti); `visual` projesi yalnızca yereldir (temel görüntüler `-darwin` son ekli, depoda). `e2e/` kendi `tsconfig.json`'ıyla denetlenir; `@types/node` yalnızca orada görünür, arayüz koduna sızmaz.

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `.gitignore`
- Create: `app/e2e/desk.spec.ts`
- Create: `app/e2e/ipc.ts`
- Create: `app/e2e/tsconfig.json`
- Create: `app/e2e/visual.spec.ts`
- Create: `app/e2e/visual.spec.ts-snapshots/d1-circle-dark-visual-darwin.png` (üretilir: `pnpm --dir app e2e:visual --update-snapshots`)
- Create: `app/e2e/visual.spec.ts-snapshots/d1-circle-light-visual-darwin.png` (üretilir: `pnpm --dir app e2e:visual --update-snapshots`)
- Create: `app/e2e/visual.spec.ts-snapshots/d1-north-dark-visual-darwin.png` (üretilir: `pnpm --dir app e2e:visual --update-snapshots`)
- Create: `app/e2e/visual.spec.ts-snapshots/d1-north-light-visual-darwin.png` (üretilir: `pnpm --dir app e2e:visual --update-snapshots`)
- Create: `app/e2e/visual.spec.ts-snapshots/d1-south-dark-visual-darwin.png` (üretilir: `pnpm --dir app e2e:visual --update-snapshots`)
- Create: `app/e2e/visual.spec.ts-snapshots/d1-south-light-visual-darwin.png` (üretilir: `pnpm --dir app e2e:visual --update-snapshots`)
- Modify: `app/package.json`
- Create: `app/playwright.config.ts`
- Modify: `app/pnpm-lock.yaml` (üretilir: `pnpm add` adımı)

**Interfaces:**
- Consumes: arayüzün tamamı (Task 4–12), `app/ui/test/fixtures/delhi.json` (Task 3).
- Produces: `app/playwright.config.ts` (projeler `e2e`, `visual`; WebKit "Desktop Safari", 1440×900; `webServer` `pnpm exec vite --port 1421 --strictPort`); `e2e/ipc.ts`: `mockIpc(page, {session})`, `ipcCalls(page, cmd)`; `package.json` betikleri `e2e`, `e2e:visual`; `build` artık `tsc --noEmit -p e2e` da çalıştırır.

- [ ] **Step 1: Geliştirme bağımlılıklarını ekle ve WebKit'i hazırla**

```bash
pnpm --dir app add -D @playwright/test@1.65.1 @types/node@24
pnpm --dir app exec playwright --version
ls ~/Library/Caches/ms-playwright/ | grep webkit || pnpm --dir app exec playwright install webkit
```
Expected: `Version 1.65.1`; `webkit-2388` önbellekte varsa indirme yok, yoksa `playwright install webkit` (~70 MB, CI'da her çalıştırmada). Lisanslar: `@playwright/test` Apache-2.0, `@types/node` MIT. `app/pnpm-lock.yaml` elle düzenlenmez.

- [ ] **Step 2: Önce testleri yaz**

`package.json` ve yapılandırma Step 4'tedir; önce senaryolar:

`app/e2e/desk.spec.ts` oluştur:

```ts
import { expect, test } from "@playwright/test";
import { ipcCalls, mockIpc } from "./ipc";

test("birth entry: empty state, atlas place by keyboard, preview, save opens the chart", async ({ page }) => {
  await mockIpc(page, { session: false });
  await page.goto("/");
  await page.getByRole("region", { name: "Açık harita yok" }).getByRole("button", { name: "Yeni harita" }).click();
  const form = page.getByRole("dialog", { name: "Yeni harita" });
  await expect(form.getByLabel("Danışan adı", { exact: true })).toBeFocused();
  await form.getByLabel("Danışan adı", { exact: true }).fill("Örnek Danışan");
  await form.getByLabel("Tarih", { exact: true }).fill("2000-01-01");
  await form.getByLabel("Saat", { exact: true }).fill("17:30");
  const place = form.getByRole("combobox", { name: "Doğum yeri" });
  await place.fill("new delhi");
  await expect(form.getByRole("listbox", { name: "Bulunan yerler" }).getByRole("option").first()).toContainText("New Delhi");
  await place.press("Enter");
  await expect(form.getByLabel("Bölge adı", { exact: true })).toHaveValue("Asia/Kolkata");
  await expect(form.getByRole("region", { name: "Önizleme" })).toContainText("UTC 2000-01-01 12:00:00 · IST +05:30");
  await form.getByRole("button", { name: "Haritayı kaydet" }).click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await expect(page.getByRole("group", { name: "D1 haritası" })).toBeVisible();
  const created = (await ipcCalls(page, "create_client")) as { input: { chart: { birth: { tz: unknown } } } }[];
  expect(created[0].input.chart.birth.tz).toEqual({ Iana: "Asia/Kolkata" });
});

test("layout switch with Cmd+2 and a keyboard panel move that is saved with the chart", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+Digit2");
  await expect(page.getByRole("combobox", { name: "Düzen" }).locator("option:checked")).toHaveText("Araştırma");
  await expect(page.getByRole("region", { name: "Graha tablosu" })).toBeVisible();
  const table = page.getByRole("region", { name: "Graha tablosu" });
  const before = Number(await table.getAttribute("data-y"));
  await table.focus();
  await page.keyboard.press("Alt+ArrowDown");
  await expect(table).toHaveAttribute("data-y", String(before + 1));
  await expect.poll(async () => (await ipcCalls(page, "save_view_state")).length).toBeGreaterThan(0);
  const saved = (await ipcCalls(page, "save_view_state")).at(-1) as { chartId: number };
  expect(saved.chartId).toBe(1);
});

test("hovering a graha row highlights it in the chart; the style selector redraws", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await page.keyboard.press("Meta+Digit2");
  const row = page.getByRole("region", { name: "Graha tablosu" }).locator('tr[data-graha="Saturn"]');
  await row.hover();
  await expect(page.locator('svg [data-graha="Saturn"]').first()).toHaveClass(/chart-planet--highlight/);
  const chart = page.getByRole("region", { name: "Harita D1", exact: true });
  await chart.getByRole("combobox", { name: "Biçim" }).selectOption("south");
  await expect(chart.locator("svg.chart")).toHaveAttribute("data-style", "south");
});

test("dasha tree by keyboard loads level 4 on demand", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await page.keyboard.press("Meta+Digit2");
  const tree = page.getByRole("tree", { name: "Dasa ağacı" });
  await expect(tree).toBeVisible();
  await tree.getByRole("treeitem").first().focus();
  // Sağ ok: kapalıysa açar, açıksa ilk alt döneme iner (1 → 2 → 3. seviye).
  for (const key of ["ArrowRight", "ArrowRight", "ArrowRight", "ArrowRight", "ArrowRight"]) await page.keyboard.press(key);
  await expect(page.locator(":focus")).toHaveAttribute("aria-level", "3");
  await expect(tree.getByRole("treeitem", { name: /^Rahu–Rahu–Rahu–Rahu, / })).toBeVisible();
  await page.keyboard.press("ArrowRight");
  await expect(page.locator(":focus")).toHaveAttribute("aria-level", "4");
  expect(await ipcCalls(page, "dasha_children")).toHaveLength(1);
});
```

`app/e2e/visual.spec.ts` oluştur:

```ts
// Görsel regresyon (yalnızca yerel: `pnpm e2e:visual`). Üç harita biçimi × açık/koyu tema,
// gerçek Delhi D1. Temel görüntüler `e2e/visual.spec.ts-snapshots/` altında, darwin son ekli;
// başka font yığınında (Linux CI) piksel farkı beklenir, bu yüzden CI'da koşmaz.
import { expect, test } from "@playwright/test";
import { mockIpc } from "./ipc";

for (const theme of ["light", "dark"] as const) {
  for (const style of ["north", "south", "circle"] as const) {
    test(`D1 ${style} ${theme}`, async ({ page }) => {
      await mockIpc(page, { session: true });
      await page.addInitScript((t) => localStorage.setItem("jyotish-desk.theme.v1", t), theme);
      await page.goto("/");
      const chart = page.getByRole("region", { name: "Harita D1", exact: true });
      await chart.getByRole("combobox", { name: "Biçim" }).selectOption(style);
      await expect(chart.locator("svg.chart")).toHaveAttribute("data-style", style);
      await expect(chart.locator("svg.chart")).toHaveScreenshot(`d1-${style}-${theme}.png`);
    });
  }
}
```


- [ ] **Step 3: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec playwright test --project=e2e
```
Expected: FAIL — `playwright.config.ts` yok (`Error: No tests found` ya da yapılandırma hatası).

- [ ] **Step 4: Uygulamayı yaz**



`.github/workflows/ci.yml` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/.github/workflows/ci.yml
+++ b/.github/workflows/ci.yml
@@ -21,6 +21,14 @@ jobs:
       - run: pnpm --dir app install --frozen-lockfile
       - run: pnpm --dir app test
       - run: pnpm --dir app build
+      # Uçtan uca arayüz testleri (WebKit, IPC taklidi). Görsel karşılaştırma yalnızca yerelde.
+      # WebKit önbelleği Playwright sürümüne (kilit dosyası) bağlı; varsa kurulum indirme yapmaz.
+      - uses: actions/cache@v4
+        with:
+          path: ~/Library/Caches/ms-playwright
+          key: playwright-webkit-${{ runner.os }}-${{ hashFiles('app/pnpm-lock.yaml') }}
+      - run: pnpm --dir app exec playwright install webkit
+      - run: pnpm --dir app e2e
       - uses: dtolnay/rust-toolchain@stable
         with:
           components: rustfmt, clippy
```

`.gitignore` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/.gitignore
+++ b/.gitignore
@@ -4,3 +4,5 @@
 .superpowers/
 /app/dist
 /app/src-tauri/gen
+/app/test-results
+/app/playwright-report
```

`app/e2e/ipc.ts` oluştur:

```ts
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
```

`app/e2e/tsconfig.json` oluştur:

```json
{
  "extends": "../tsconfig.json",
  "compilerOptions": {
    "types": ["node"]
  },
  "include": [".", "../playwright.config.ts"]
}
```

`app/package.json` değiştir (birleşik fark; `@@` satırları bağlamı gösterir):

```diff
--- a/app/package.json
+++ b/app/package.json
@@ -7,8 +7,10 @@
   "packageManager": "pnpm@11.8.0",
   "scripts": {
     "dev": "vite",
-    "build": "tsc --noEmit && vite build",
+    "build": "tsc --noEmit && tsc --noEmit -p e2e && vite build",
     "test": "vitest run",
+    "e2e": "playwright test --project=e2e",
+    "e2e:visual": "playwright test --project=visual",
     "tauri": "tauri"
   },
   "dependencies": {
@@ -19,9 +21,11 @@
     "react-i18next": "^18.0.3"
   },
   "devDependencies": {
+    "@playwright/test": "1.65.1",
     "@tauri-apps/cli": "^2.12.6",
     "@testing-library/jest-dom": "^8.0.0",
     "@testing-library/react": "^16.3.3",
+    "@types/node": "^24.19.2",
     "@types/react": "^19.3.0",
     "@types/react-dom": "^19.3.0",
     "@vitejs/plugin-react": "^6.3.0",
```

`app/playwright.config.ts` oluştur:

```ts
import { defineConfig, devices } from "@playwright/test";

// Uçtan uca testler Vite geliştirme sunucusundaki arayüzü WebKit'te (Tauri'nin macOS
// web görünümüyle aynı motor) çalıştırır; Tauri IPC `e2e/ipc.ts` ile taklit edilir.
// "e2e" CI'da koşar. "visual" ekran görüntüsü karşılaştırmasıdır: font yığınına bağlı
// olduğundan yalnızca yerelde (`pnpm e2e:visual`) koşar.
export default defineConfig({
  testDir: "e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? "github" : "list",
  use: {
    ...devices["Desktop Safari"],
    viewport: { width: 1440, height: 900 },
    baseURL: "http://localhost:1421",
    locale: "tr-TR",
    trace: "retain-on-failure",
  },
  projects: [
    { name: "e2e", testMatch: /desk\.spec\.ts$/ },
    { name: "visual", testMatch: /visual\.spec\.ts$/, expect: { toHaveScreenshot: { maxDiffPixelRatio: 0.002 } } },
  ],
  webServer: {
    command: "pnpm exec vite --port 1421 --strictPort",
    url: "http://localhost:1421",
    reuseExistingServer: !process.env.CI,
  },
});
```


- [ ] **Step 5: Görsel temelleri üret ve incele**

```bash
pnpm --dir app e2e:visual --update-snapshots
ls app/e2e/visual.spec.ts-snapshots/
```
Expected: `6 passed`; 6 PNG (`d1-{north,south,circle}-{light,dark}-visual-darwin.png`). Her görüntüyü açıp renklerin token'larla tutarlı, metnin okunur, lagna ve retro işaretlerinin görünür olduğunu doğrula; sonraki `pnpm --dir app e2e:visual` eşitlik denetler.

- [ ] **Step 6: Testlerin geçtiğini gör**

```bash
pnpm --dir app e2e
pnpm --dir app e2e
pnpm --dir app e2e:visual
pnpm --dir app test
pnpm --dir app build
```
Expected: `e2e` iki çalıştırmada da `4 passed` (kararsızlık yok); `e2e:visual` `6 passed`; vitest `35 passed (35)` / `279 passed (279)`; `build` hatasız. `git status` `app/test-results` ve `app/playwright-report`'u göstermez (`.gitignore`).

- [ ] **Step 7: Commit**

```bash
git add -A .github/workflows/ci.yml .gitignore app/e2e/desk.spec.ts app/e2e/ipc.ts app/e2e/tsconfig.json app/e2e/visual.spec.ts app/e2e/visual.spec.ts-snapshots/d1-circle-dark-visual-darwin.png app/e2e/visual.spec.ts-snapshots/d1-circle-light-visual-darwin.png app/e2e/visual.spec.ts-snapshots/d1-north-dark-visual-darwin.png app/e2e/visual.spec.ts-snapshots/d1-north-light-visual-darwin.png app/e2e/visual.spec.ts-snapshots/d1-south-dark-visual-darwin.png app/e2e/visual.spec.ts-snapshots/d1-south-light-visual-darwin.png app/package.json app/playwright.config.ts app/pnpm-lock.yaml
git diff --cached -U0 -- . ':!app/pnpm-lock.yaml' ':!*.png' ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'   # boş olmalı
git commit -m "test(app): Playwright WebKit end-to-end tests with a mocked Tauri IPC, local visual baselines and CI

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 14: Son doğrulama

Kod değişmez; bütün kapılar sırayla çalıştırılır. Bir kapı düşerse ilgili görevin adımına dön; burada yama yapma.

**Files:** yok.

**Interfaces:** yok.

- [ ] **Step 1: Biçim ve lint**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```
Expected: ikisi de çıktısız.

- [ ] **Step 2: Rust testleri ve MSRV**

```bash
cargo test --workspace 2>&1 | grep -E '^test result|Running'
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p store -p jyotish-desk --all-targets
cargo deny check
```
Expected: her `test result: ok.`; `store` kütüphanesi 64, `jyotish-desk` kütüphanesi 13, `ui_fixture` 2, `bindings` 1, `open_client` 1 (Task 3'te genişledi), `jyotish-core` 108 test (sayılar Plan 08 tabanına göredir; düşen ya da `ignored` dışı eksilen yok). MSRV denetimi hatasız. `cargo deny` `advisories ok, bans ok, licenses ok, sources ok` (Rust bağımlılığı değişmedi; denetim yalnızca güvence).

- [ ] **Step 3: Arayüz testleri, derleme, e2e**

```bash
pnpm --dir app test
pnpm --dir app build
pnpm --dir app e2e
pnpm --dir app e2e:visual
```
Expected: `Test Files  35 passed (35)`, `Tests  281 passed (281)`; `build` hatasız; e2e `4 passed`; görsel `6 passed` (yalnızca macOS'ta, yerel).

- [ ] **Step 4: Yasak dizgi taraması ve temiz çalışma ağacı**

```bash
BASE=$(git log --format=%H -1 --grep='atlas place search')^
git diff "$BASE" -U0 -- . ':!app/pnpm-lock.yaml' ':!*.png' | grep '^+' | grep -inE 'k(a|ā|aa)la|\bcala\b'
grep -inE 'k(a|ā|aa)la|\bcala\b' app/ui/test/fixtures/delhi.json app/ui/chart/__snapshots__/*.svg
git ls-files .superpowers | wc -l
git status --short
```
Expected: iki `grep` de boş; `.superpowers` altında izlenen dosya `0`; `git status` boş.

- [ ] **Step 5: Elle duman testi (isteğe bağlı, önerilir)**

```bash
pnpm --dir app tauri dev
```
Gerçek Tauri penceresinde (yalnızca orada doğrulanabilir) sırayla:

1. Boş masada "Yeni harita" görünür; `⌘N` formu açar ve odak "Danışan adı"ndadır.
2. Formu doldur (New Delhi araması, 2000-01-01 17:30) ve Kaydet: harita, tablo ve dasha ağacı dolar.
3. `⌘E` düzenleme formunu açar; odak pencerenin içindedir (ad alanı kilitli), Esc tıklamadan kapatır.
4. `⌘1`, `⌘2`, `⌘3` düzenleri değiştirir; macOS menüsü bu tuşları (ve `⌘N`/`⌘E`'yi) yutmaz.
5. `?` kısayol yardımını açar, Esc kapatır ve odak tetikleyene döner.
6. Uygulamayı kapatıp açınca aynı harita, düzen ve dasha yolu geri gelir.
7. Terminalde kişisel veri içeren bir satır yoktur.

Expected: yedi madde de beklendiği gibi; biri tutmazsa ilgili görevin testine dönülür.

---

## Bu planın tamamlanma ölçütü

- Doğum verisi formdan girilir (atlas araması ya da elle koordinat; IANA, sabit ofset ya da LMT; DST boşluğu ve örtüşmesi açıkça çözülür), `create_client`/`update_chart` ile kaydedilir ve açılır; uygulama yeniden açıldığında son harita, düzen ve dasha yolu geri gelir.
- Harita paneli her vargayı üç stilde çizer; saat bilinmiyorsa lagnasız çizer ve bunu söyler. Graha tablosu sütun seçimli, lagna satırlı ve renk dışı işaretlidir. Dasha ağacı 5 seviyeye kadar klavyeyle gezilir.
- Bütün görünür metinler tr/en/fr'dedir; arayüzden ulaşılabilen her hata kodunun metni vardır.
- Kısayollar tek kayıttadır ve `?` ile listelenir; modal pencerelerde odak tuzağı ve geri dönüşü vardır.
- `ViewState` tek depodan serileştirilir ve store'a yazılır; localStorage düzeni bir kez taşınır.
- Plan 08 ertelenen madde 2 (harita olmadan düzen kaydı) ve 12 (ayrıştırıcı eşliği) kapanır.
- Düzen sınırı yazarken de uygulanır: arayüz 64'ten fazla düzen yazmaz, yeniden açılışta düzen kaybolmaz.
- Task 14'ün bütün adımları beklenen çıktıyı verir; vitest 279, e2e 4, görsel 6 (yerel, PNG temelleri yalnızca bu adımda karşılaştırılır).

## Bu planın kapsamadıkları

1. **M6:** danışan listesi ve kenar çubuğu, `Cmd+K` arama, aynı danışana ikinci harita (`add_chart`), notlar paneli, tzdb farkını kabul akışı, içe/dışa aktarma arayüzü (Plan 08 ertelenen madde 1 ve 3; komutlar `api.ts`'te sarılmadan kalır), cinsiyet alanı (taslak değeri korur, form göstermez).
2. `dasha_strip`, `transit_table` ve `birth_info` panelleri (kayıtta yer tutucu olarak durur); rozetlerin doğum bilgisi panelinde de gösterilmesi.
3. Graha tablosunda avastha sütunu (çekirdekte hesap yok).
4. Haritada transit halkası ve nakshatra kısaltmaları.
5. Dasha seçiminden transit tablosuna tarih bağlantısı; güncel dönemin vurgulanması (bugünün tarihi arayüzde hesap gerektirir; Rust ucu eklenince).
6. M7 baskı akışı (`PrintSpike` deneme olarak durur).
7. Görsel (ekran görüntüsü) testlerinin CI'da çalışması: font yığını ve WebKit sürümü sabitlenince.
8. Plan 08 ertelenen madde 9 (`charts.rs` / `open.rs`'e dokunan ilk plana).
9. macOS menü çubuğu: `Cmd+N` ve `Cmd+E` yerel menü öğelerine bağlanmadı; Tauri menüsü eklendiğinde çakışma denetlenmeli.
10. **Plan 08'in bu planda kapanmayan ertelenen maddeleri:** madde 1 ve 3 (gerçek JHD dosyaları, içe aktarma boyut sınırı, toplu içe aktarma; M6, içe/dışa aktarma arayüzünün ön koşulu); madde 9 (`add_chart`, `delete_chart`, `open_client` işlemleri; `charts.rs`/`open.rs`'e dokunan ilk plan); madde 10 (veri klasörü ve dosyası için 0700/0600; M8); madde 11 (panik eden görev için `task_failed` kodu; M6 hata eşlemesi, kilit zehirlenmesi M8; bugün store görevi paniği `engine_stopped` metniyle görünür); madde 13 (atomik dışa aktarma yazımı; M6); madde 14 (`chart_tags(tag_id)` indeksi; sonraki şema göçü).
11. **Plan 08 incelemesinin M6'ya bıraktığı küçük bulgular:** Task 3 (`AppPrefs.schema_version` denetlenmiyor, tercih `calc_settings`'in sessiz geri dönüşü; ayarlar arayüzü), Task 6 (kırpma testi yok, etiket sırası yalnızca ASCII; etiket girişi), Task 7 (etiket sorgusu ham biçimde, NFD işaretleri, `load_client` N+1; `Cmd+K`), Task 8 (D13 sessiz yolu eski kısaltma/uyarıyı tutar; tzdb bandı), Task 9 (okunamayan JSON'da `null` dışa aktarma, bilinmeyen alan politikası; dışa aktarma belgesi), Task 10 (eksik testler, `"; "` etiket ayırıcısı, sınırsız dışa aktarma), Task 11 (toplu içe aktarmada yarıda DB hatası raporu kaybeder; gerçek dosya kurallarıyla proptest), Task 13 (`file_io` parametrelerinde tam yol, atomik olmayan dışa aktarma, ince dosya komutu testleri, boş içe aktarmanın kilidi alması).
12. **M5'te oluşturulan danışanlar:** M6 danışan listesine kadar yalnızca oturum geri yüklemesiyle (son açılan harita) ulaşılır; listelenemez, yeniden seçilemez, silinemez.
13. **İnceleme önerileri (M6):** atlas açılamama nedeni (`AtlasState` hatayı saklayıp `search_places`'ten döndürebilir; şimdi yalnızca `atlas_unavailable`); tarih değişince `fold` seçiminin sıfırlanması; Esc ile yarım taslağın kaybı (onay ya da taslak saklama).

---

## Self-Review

### Spec kapsamı

| Gereksinim | Kaynak | Görev |
|---|---|---|
| Doğum verisi girişi (tarih, saat, kesinlik, yer, tz kipi, kaynak notu) | v1 §5 `BirthData`, UI §4 | 10, 11, 12 |
| Atlas araması, elle koordinat birinci sınıf | v1 §4, §12; UI §4 | 1, 11, 12 |
| DST boşluğu/örtüşmesi asla sessizce kaydırılmaz | v1 §7 | 1, 10, 12 |
| Harita SVG, üç stil, her varga | v1 §11 M5; UI §2.3, §3.3 | 5, 6 |
| Saat bilinmiyor çizimi ve uyarısı | v1 §7; UI §5 | 5, 6, 7, 8 |
| Graha tablosu sütunları | UI §3.3 | 7 |
| Dasha ağacı, tembel derin seviyeler | v1 §11 M5; UI §3.3, §3.4 | 8 |
| i18n tr/en/fr, hata kodları `{code, params}` | v1 §7, §11 | 5–12 |
| Klavye kısayolları ve yardım | UI §6 | 9, 12 |
| Merkezi, serileştirilebilir `ViewState` | v1 §5, §11 | 4 |
| Kalıcılık: düzen, dasha yolu, son oturum | v1 §4 | 2, 4, 5 |
| Bildirimler ve rozetler; renk tek başına anlam taşımaz | UI §5, §7 | 5 |
| Boş durum | UI §5 | 12 |
| Görsel ve e2e testler | UI §8 | 6, 13 |
| Arayüz hesap yapmaz | v1 §4 | 1, 3 (fikstür), hepsi |
| Plan 08 ertelenen 2 ve 12 (yazma tarafı sınırı dahil) | Plan 08 | 2 |
| Plan 08 §2.5 test boşlukları (DST boşluğunda değişmeyen harita, `save_view_state`, düzen kaydı) | Plan 08 incelemesi | 3 |
| Kişisel veri loglanmaz (bekçi test) | v1 §4 | 4 |
| Saat bilinmiyorken bölüm haritası notu | v1 §7 | 6 |

### Yer tutucu taraması

Belirsiz ya da ertelenmiş adım yok; her kod adımı tam kodu ya da birleşik farkı içerir. Üretilen dosyalar (`bindings.ts`, `delhi.json`, `pnpm-lock.yaml`, SVG ve PNG temelleri) elle yazılmaz; her biri üreten komutla birlikte verilir.

### Tip tutarlılığı

- `OpenedClient`, `PlaceHit`, `MomentPreview`, `PanelLayout`, `ViewState`, `DashaSeed` yalnızca `bindings.ts`'ten gelir (Task 1, 2, 5 yeniden üretir).
- `useLayoutStore()` Task 4'te `view/store.tsx`'e taşınır; Task 6 `setPanelSettings`'i ekler; Task 7 aynı imzayı kullanır.
- `useSession().open` Task 5'te tanımlanır, Task 12'de çağrılır; `useViewStore().setDashaPath` Task 4'te, Task 8'de kullanılır.
- `matchShortcut` Task 9'da tanımlanır; Task 12 `newChart`/`editChart` komutlarını Shell'de bağlar.
- `PlaceSearchProps.inputId` Task 12'de eklenir (Task 11 testleri onsuz geçer).
- `useNotices`/`storeWarningNotices` Task 5'te tanımlanır, Task 12 `BirthForm` kayıttan sonra kullanır; `MAX_LAYOUTS` Task 2'de `layout/model.ts`'ten dışa aktarılır ve aynı görevde reducer ile `LayoutPicker` kullanır.
- `useLazyDasha` önbellek anahtarı `cacheKeyOf(chartId, seed)` (Task 8); `DashaSeed` alanları `bindings.ts`'teki adlarla (`birth_jd_ut`, `moon_sidereal_lon`, `year_days`).

### Görev başına test sayıları (kopyada ölçüldü)

| Görev | Vitest dosya | Vitest test | Rust |
|---|---|---|---|
| Taban (Plan 08) | 23 | 203 | `jyotish-desk` lib 10, `store` lib 61 |
| 1 | 23 | 203 | `jyotish-desk` lib 17 (+7 atlas_commands) |
| 2 | 23 | 211 | `store` lib 64 (+3) |
| 3 | 24 | 215 | `ui_fixture` 2 (yeni), `open_client` 1 (genişledi) |
| 4 | 27 | 226 | — |
| 5 | 26 | 226 | `jyotish-desk` lib 13 (−4 deneme) |
| 6 | 27 | 237 | — |
| 7 | 29 | 245 | — |
| 8 | 31 | 259 | — |
| 9 | 32 | 263 | — |
| 10 | 33 | 269 | — |
| 11 | 34 | 273 | — |
| 12 | 35 | 281 | — |
| 13 | 35 | 281 | + Playwright e2e 4, görsel 6 (yerel) |

Plan kopyada (`/Users/ttd/jd-p09`) iki kez uygulandı; ikincisinde plan dosyasından (yeni dosyalar ve farklar betikle uygulanarak) yeniden kuruldu, inceleme düzeltmeleri ilgili görevlere işlendi ve her commit'te vitest + `build` ölçüldü: bütün kapılar geçti, e2e üç kez kararsızlıksız çalıştı, görsel temeller birebir yeniden üretildi. Yeniden kurulum iki plan hatasını da gösterdi ve ikisi düzeltildi: `pnpm --dir app vitest run` pnpm 11'de çalışmaz (`Command "app" not found`), komutlar `pnpm --dir app exec vitest run`'dır; `ui_fixture` fikstür klasörünü kendisi oluşturur. Doğrulama sırasında bulunan iki hata Task 8 koduna işlendi: (1) klavye tembel yüklenen 4. seviyeye ulaşamıyordu (`loaded()` yardımcısı), (2) istenen yolun bir alt seviyesi olmayan yanıt sonsuz özyinelemeye yol açıyordu (`childrenOf` süzgeci); ikisinin de gerileme testi vardır.
