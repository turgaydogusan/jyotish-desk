# Plan 21 — Alt proje 3.8: Prashna (soru anının haritası, sayı prashnası, olgu paneli; "Yeni prashna" penceresi ve Prashna düzeni)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Soru anı için harita kurmak ve o haritanın olgularını göstermek. Üç parça: `jyotish-core`'da lagnası verilen harita (`compute_chart_with_lagna`), sayı prashnasının lagnası (1–108 → navamsa ortası) ve soru haritasının olguları (`prashna_facts`: lagnanın burcu, türü ve doğuş biçimi; lagna lordu ve odak evinin lordu; iki lordun bağı; Ay; kendralardaki ve odak evindeki grahalar; yoga listesi); `store`'da prashna yan tablosu (şema sürümü 2), yerleşik "Prashna" danışanı ve soru metninin saklanması; uygulamada `create_prashna`, `update_prashna` ve `compute_prashna_facts` komutları, "Yeni prashna" penceresi (⌘⇧P), `prashna_facts` paneli ve "Prashna" hazır düzeni. Ayrı bir prashna ekolünün kuralı, konu çıkarımı, hüküm ve evet/hayır sonucu yoktur.

**Architecture:**
- **Aynı motor, verilen lagna (spec §11, K3).** `compute_chart` gövdesi `chart_with(…, lagna_sidereal_lon: Option<f64>)` olur; `compute_chart_with_lagna` aynı gövdeyi verilen lagnayla çağırır. Grahalar, dasha ve panchang değişmez; lagna, bhavalar (lagnadan burç evleri), grahaların evi, vargaların lagnası ve ashtakavarga verilen lagnadan gelir. Ephemeris'ten ev istenmez.
- **Sayı, modül girdisinin alanıdır (K2'nin genişlemesi).** `ChartInput` `lagna_number: Option<u8>` kazanır; `compute_chart_for(eph, &ChartInput)` haritayı sayı varsa sayının lagnasıyla kurar ve bütün modül toplayıcıları (bakışlar, Jaimini, chara dasha, Shadbala, yogalar, uyum) haritayı buradan alır. Böylece sayı prashnasında her panel, haritanın gösterdiği lagnayı kullanır; "sayı prashnasında bu panel yanlış lagnayla hesaplar" durumu oluşmaz.
- **Saf olgular (K3).** `prashna::prashna_facts(&Positions, &CombustionOrbs, focus_house)` ephemeris görmez: lordlar Plan 14'ün `house_lord`'undan, dignity ve yanma v1 fonksiyonlarından, bakışlar Plan 14'ün burç temelli tam bakışlarından, doğal sınıf K6'nın `nature`'ından, yoga listesi Plan 18'in `yogas`'ından gelir. El örnekleri sentetik konumlarla kurulur.
- **Kalıcı veri tek yerde.** SQLite şeması sürüm 2'ye çıkar: `prashna(chart_id, question, number, focus_house, created_at)`. Bir harita, bu tabloda satırı varsa prashnadır; `charts.kind` ve göç 1 değişmez. Soru metni arama dizinine girmez ve dışa aktarıma yalnızca notlarla birlikte yazılır. Modül sonuçları saklanmaz.
- **Üç komut, Plan 14 ve Plan 20'nin kalıbıyla (K1).** `create_prashna(request)` anı ve yeri Rust'ta çözer, haritayı kaydeder ve açar; `update_prashna(chart_id, edit)` soru metnini ve odak evini yazar; `compute_prashna_facts(input, focus_house)` olguları hesaplar. `open_client` dönüşü `prashna: Option<PrashnaMeta>` kazanır ve sayı prashnasının haritası her açılışta sayının lagnasıyla hesaplanır. Bu yolların hataları yalnızca alan adı taşır (`invalid_prashna_input { field }`).
- **Arayüz hesap yapmaz.** Pencere taslağı isteğe çevirir (yalnızca biçim denetimi); panel olguları ve açık haritanın panchangını çizer. Soru ve odak evi otomatik kaydedilir; olgular yeniden okunurken son okunan olgular ve odak yerinde durur.
- **Testler:** sahte ephemeris'le harita testleri ve iki spec değişmezi (proptest), elle yazılmış ikinci tablolar (sayı → lagna, on iki burcun doğuş biçimi), on dört el fikstürü (`tests/golden/modules/prashna/`, kaynak yalnızca `hand`; bağımsız bir ikinci türetmeyle yazıldı), olguların iç tutarlılık değişmezi, `store` testleri (göç, gidiş-dönüş, silme, arama, dışa ve içe aktarım), gerçek motorla entegrasyon (v1 haritasıyla çapraz denetim, sayı değişmezi, bütçeler), Vitest (pencere, panel, oturum, düzen), Playwright e2e.

**Tech Stack:** Rust 1.93+ (stable; yerelde ve CI'da 1.100), Tauri 2.12, tauri-specta 2.0.0-rc.29, serde, rusqlite ve rusqlite_migration, jiff (atlas üzerinden), proptest, React 19, TypeScript 7, Vitest 5, Playwright 1.65.1. **Yeni Rust ya da npm bağımlılığı yok.**

**Spec:** `docs/superpowers/specs/2026-11-26-alt-proje-3-ileri-moduller-design.md` (Plan 20'nin geri bildirimi işlenmiş hâli): §2 (K1–K12), §4 (`ChartInput`, `Positions`), §10 (`home_place`, alan adlı hatalar), §11 (Alt proje 3.8), §13 (ortak test stratejisi), §14 (gizlilik), §15 (bütçeler), §17 (riskler), Ek A (adlandırma). UI spec: `2026-11-24-ui-design.md` §3.2 (düzenler), §3.3–§3.4, §4 (palet, sağdan kayan panel), §5, §6, §8.

**Ön koşul:** Taban, Plan 20'nin son ucudur: `main` üzerindeki commit `0b61716` (Plan 20'nin son inceleme düzeltmeleri `43ca372`, `14eb60a`, `cb9fd74`, `79e53eb`; el örneklerinin yeniden türetilmesi `bb810f2`; spec'e işlenen geri bildirim `0b61716`). `main` bu commit'ten ilerlediyse Task 1 adım 1 tabanları yeniden ölçer ve sayaç zinciri o ölçümden yürütülür (farklar aynıdır). Plan 20'nin elle denetim kapısının yalnızca insana ait maddeleri ve muhurta tablolarının basılı kaynak onayı sahibine devredilmiştir; bu planı engellemez.

## Kararlar

Sahibi kararları devretti; muhafazakâr seçenek seçildi ve her karar bir satır gerekçeyle yazıldı. "Spec kusuru" ya da "Spec boşluğu" diye işaretlenenler spec'te düzeltilmesi gereken yerlerdir ve belgenin sonundaki "Spec'e geri bildirim" bölümünde aynı numarayla toplanmıştır. D24 ve sayaç zinciri bu plan yazılırken deponun bir kopyasında ölçülmüştür (macOS 27.2, Rust 1.100).

- **D1 — Çekirdek modülü `crates/jyotish-core/src/prashna/`'dır: `mod.rs` (sayının lagnası) ve `facts.rs` (olgular). Yöntem kimliği `PRASHNA_METHOD = "prashna.parashari_v1"`'dir ve `PrashnaFacts` `method` taşır.** Gerekçe: spec modülü adlandırmaz; K7 her modül sonucunun yöntem kimliği taşımasını ister. Kimlik, hesabın Parashari motoru olduğunu söyler (spec §11: ayrı bir prashna ekolü uygulanmaz).
- **D2 — `compute_chart_with_lagna(eph, jd_ut, lat, lon, settings, lagna_sidereal_lon)` verilen lagnayla şunu döner: lagna (rasi çerçevesindeki boylamı `sidereal + ayanamsa` ile, tropikal-rasi kipinde), bhavalar her zaman lagnadan burç evleri (`bhavas.system` `WholeSign` döner; `meta.settings` ayarın kopyası olarak değişmez), grahaların evi burç evi, vargaların lagnası ve ashtakavarga verilen lagnadan. Grahalar, dasha, panchang, ayanamsa ve `LowPrecision` uyarısı `compute_chart`'ınkidir. Ephemeris'ten ev istenmez; `PlacidusPolarFallback` üretilmez. Sonlu olmayan lagna `InvalidPrashnaInput("lagna")`'dır; 0–360 dışındaki boylam sarılır.** Gerekçe: spec "Evler bu lagnadan rasi evidir" der; aynı bölümün değişmezi "gerçek lagna verildiğinde `compute_chart` ile aynı haritayı döner" der. İkisi yalnızca burç evi sisteminde birlikte doğrudur: Sripati ya da Placidus ayarında `compute_chart` burç evi vermez. Değişmez burç evi sistemi için sınanır (sidereal kipte bit bit; tropikal-rasi kipinde lagnanın rasi boylamı yeniden kurulduğu için 1e-9° payla). **Spec kusuru 5.**
- **D3 — Sayının lagnası `number_lagna_lon(n)`'dir: `((n − 1) × 200 + 100) / 60` derece (yay dakikası üzerinden; aynı sayı her zaman aynı `f64`). `n` 1..=108 değilse `InvalidPrashnaInput("number")`. Sayı, zodyağın `n`. navamsasıdır; aynı zamanda `n`. nakshatra padasıdır.** Gerekçe: spec'in formülü `(n − 1) × 3°20′ + 1°40′`. `3°20′`'yi `10/3` olarak çarpmak yerine tamsayı yay dakikası kullanmak sınır değerlerinde (burç başına dokuz sayı) kayan nokta sürprizini önler; test 108 sayının hepsinin kendi padasına düştüğünü sabitler.
- **D4 — `ChartInput` `lagna_number: Option<u8>` kazanır ve `compute_chart_for(eph, &ChartInput)` eklenir: sayı varsa `compute_chart_with_lagna`, yoksa `compute_chart`. `compute_positions` ile Jaimini, chara dasha ve Shadbala toplayıcıları haritayı `compute_chart_for`'dan alır (yogalar, bakışlar ve uyum `compute_positions` üzerinden). `time_known` kuralı değişmez: yanlışsa lagnaya bağlı çıktı yine boştur. Arayüzün `chartInput(opened)` yardımcısı sayıyı `opened.prashna.number`'dan doldurur; `store::chart_input` (eş harita) yan tablodan okur.** Gerekçe: spec `compute_prashna_facts(input: ChartInput, number, focus_house)` yazar ve sayıyı yalnızca bu komuta verir. Oysa sayı prashnası açıkken paletten eklenen her modül paneli (yogalar, bakışlar, Jaimini, Shadbala, chara dasha) `ChartInput` ile hesaplar ve K2'nin beş alanı gerçek lagnayı verir: harita Başak lagnası çizerken yoga paneli başka bir lagnadan sayardı. Sayıyı ortak girdiye koymak hatayı sınıf olarak önler; `compute_prashna_facts`'in ayrı `number` parametresi gereksizleşir. Bedeli: Rust'taki `ChartInput` değişmezleri bir satır kazanır (23 yer) ve bağlamalar yeniden üretilir. **Spec kusuru 2.** **Son inceleme kararı (istisna; `plan21-ruling-shadbala`, commit `5bf0b54`):** Shadbala toplayıcısı haritayı yine `compute_chart_for`'dan alır (sayı, ayar, an ve yer denetlenir), ama `lagna_number` doluysa rapor üretmez: `compute_shadbala` `Ok(None)` döner ve Bhava bala aynı rapordan geldiği için o da yoktur. Gerekçe: sayı bir lagna verir, meridyen vermez; dig bala MC ister. Anın gerçek MC'sini sayının lagnasıyla birleştirmek yanlış bir olgu, sayıdan MC türetmek uydurma olurdu. İki panel bunu kendi notuyla söyler (`bala.numberPrashna`; "saat bilinmiyor" notu değil). Meridyeni ya da ev başlangıçlarını kullanan başka modül yoktur. Sayaçlar: `jyotish-core` lib 375, Vitest 83 / 842 (zincirdeki 374 ve 840 yerine). **Spec boşluğu 19.**
- **D5 — `prashna_facts(positions, orbs, focus_house) -> Result<PrashnaFacts, CoreError>`; `PrashnaFacts { lagna: Option<LagnaFacts>, focus: Option<FocusFacts>, moon: MoonFacts, yogas: YogaReport, warnings, method }`. `LagnaFacts { sign, kind: SignKind, rising: RisingMode, lord: LordFacts, kendras: Vec<HouseOccupants> }` (dört kendra, boş evler dahil); `FocusFacts { house, sign, lord: LordFacts, relation: LordRelation, occupants }`; `LordFacts { graha, sign, house, dignity, retrograde, combust }`; `MoonFacts { sign, house, nakshatra, pada, tithi, paksha, aspected_by }`. Lagna yoksa (`time_known` yanlış) `lagna` ve `focus` `None`'dır; odak evi seçilmemişse `focus` `None`'dır. Odak evi 1..=12 değilse `InvalidPrashnaInput("focus_house")`.** Gerekçe: spec olguları madde madde sayar, tipi tanımlamaz. Lagnaya bağlı her şeyi iki `Option` altında toplamak K2'yi tipte uygular: arayüz gizlemeye gerek duymaz. **Spec boşluğu 6.**
- **D6 — Anın panchangı olguların içinde yinelenmez; yoga listesi olguların içindedir.** Panchang v1'den beri `Chart.panchang`'tadır ve panel onu açık haritadan okur (Prashna düzeninde ayrıca panchang paneli de vardır). Yoga listesi `prashna_facts` içinde aynı konumlardan hesaplanır (`yogas(&positions)`): panel ikinci bir komut çağırmaz. Gerekçe: spec "Anın panchangı (v1) ve 3.5 kuruluysa yoga listesi" der; 3.5 kuruludur. Aynı veriyi iki yerde taşımak ayrışma riskidir. **Spec boşluğu 7.**
- **D7 — İki lordun bağı tek bir değer değil, geçerli bağların listesidir: `LordRelation { same_graha, links: Vec<LordLink>, focus_lord_from_lagna_lord, lagna_lord_from_focus_lord }`; `LordLink { Conjunct, Exchange, MutualAspect, LagnaLordAspects, FocusLordAspects }`. Boş liste "hiçbiri"dir. İki evin lordu aynı grahaysa (odak evi 1; Koç lagnasında 8. ev) `same_graha` doğrudur ve bağ aranmaz. Bakış, Plan 14'ün burç temelli tam bakışıdır (özel bakışlar dahil); değişim Plan 18'in `is_exchange` tanımıdır.** Gerekçe: spec beş türü ("birlikte, karşılıklı bakış, tek yönlü bakış, değişim, hiçbiri") tek seçenek gibi sayar; oysa değişim ile karşılıklı bakış birlikte oluşabilir (Güneş Kova'da, Satürn Aslan'da) ve tek yönlü bakışın yönü olgudur. Listeyle hiçbir olgu yutulmaz. **Spec boşluğu 8.**
- **D8 — Doğuş biçimi `RisingMode { HeadFirst, BackFirst, BothWays }`'dir; tablo spec'in listesidir. Burç türü Plan 15'in `SignKind`'ıdır. Görünen klasik adlar üç dilde aynı yazılır (Shirshodaya, Prishthodaya, Ubhayodaya; K11), yanında çevrilen kısa ek. Tablo spec'te yazılıdır, klasik derlemelerin yaygın biçimidir ve bellekten aktarılmıştır: basılı bir kaynakla karşılaştırılması sahibin onay maddesidir (Task 8 Step 6) ve bilinen varyant noktaları `tests/golden/modules/prashna/REVIEW.md`'de sayılıdır.** Gerekçe: §13 katman 3 elle yazılmış ikinci tabloyu ister (testte `HAND_RISING`, iki ayrı yerde); yürüten ajanın elinde basılı kaynak yoktur ve başka yazılımın çıktısı kaynak olamaz (K9).
- **D9 — Ay'ın aldığı bakışlar: Ay'ın burcuna tam bakışla bakan grahalar (Güneş, Mars, Merkür, Jüpiter, Venüs, Satürn sırasıyla). Düğümler bakış atmaz; Ay'la aynı burçtaki graha "bakan" sayılmaz. Tithi ve nakshatra sidereal boylamdan, burç rasi çerçevesindendir (K5); tam 180° uzanım 16. tithidir (v1 sınırı).** Gerekçe: spec "aldığı burç temelli bakışlar" der; Plan 14'ün bakış tanımı kullanıldı.
- **D10 — Yeni tek hata türü `CoreError::InvalidPrashnaInput(&'static str)` → `invalid_prashna_input { field }`'dir. Çekirdek alanları: `number`, `focus_house`, `lagna`; komut alanları: `place`, `at`, `number`, `focus_house`, `question`. `create_prashna` yolunda v1'in doğum verisi hataları (`unknown_time_zone { tz }`, `nonexistent_local_time { local, suggestion }`, `ambiguous_local_time`, `invalid_input { field: "birth.…" }`, `longitude_out_of_range { lon }`) üretilmez: hepsi `place` ya da `at` alanına iner. Yer her şeyden önce, sayı, odak evi ve soru andan önce denetlenir. `compute_prashna_facts` odak evini ve sayıyı konumdan önce denetler; açık haritanın kendi konumu geçersizse v1'in `invalid_location`'ı döner (o konum zaten haritanın kaydıdır ve bu komuta arayüzden gelmez). Veritabanı hataları ve `not_found { what, id }` olduğu gibi geçer.** Gerekçe: spec §14 "yeni hata kodlarının parametreleri yalnızca alan adı, sayı ya da kimlik taşır"; §11 "soru metni hata iletisine girmez". Plan 20'nin son incelemesinde iki sızıntı bulunmuştu (bir JD aralık denetimi ve çözülemeyen anın v1 hatasıyla geçmesi); bu planda komutun bütün girdi hataları tek bir eşleme fonksiyonundan (`store_error`) geçer ve test her alan için parametre sayısının bir olduğunu sabitler. **Spec boşluğu 9.**
- **D11 — Şema sürümü 2: göç listesine `V2` eklenir (`prashna` tablosu; `number` ve `focus_house` CHECK kısıtlı; `chart_id` birincil anahtar ve `ON DELETE CASCADE`). Göç 1 değişmez. Daha eski bir uygulama sürüm 2 veritabanını açmaz (`store_too_new`; v1 davranışı).** Gerekçe: spec tabloyu yazar. Bu, projenin ilk şema göçüdür: Plan 09'un C3 kuralı ("göç V1 donmuştur; `schema.rs` değişmez") bu planda "göç 1 değişmez; `schema.rs` yalnızca yeni göçü kazanır" olarak okunur. Test, yalnızca göç 1'le kurulmuş bir veritabanının açılışta sürüm 2'ye çıktığını ve var olan haritaların durduğunu sabitler. **Spec boşluğu 10.**
- **D12 — `store`: `PrashnaMeta { question, number, focus_house, created_at }` (IPC tipi), `NewPrashna { client_id, birth, number, question, focus_house }` (IPC'yi geçmez), `Store::create_prashna(&NewPrashna, label) -> (client_id, ChartRecord)`, `Store::prashna_meta(chart_id)`, `Store::update_prashna(chart_id, question, focus_house) -> PrashnaMeta`, `Store::prashna_client_id()`. Harita `Event` türündedir ve genel varsayılan ayarların kopyasını alır. Soru kırpılır ve en çok 2 000 karakterdir (bayt değil). `update_prashna` yalnızca soruyu ve odak evini değiştirir: sayı haritanın lagnasıdır ve değişmez; an doğum formundan (⌘E) düzenlenir ve yan kayıt korunur.** Gerekçe: spec komutları adlandırır; dönüş tiplerini, sayının ve anın düzenlenip düzenlenmeyeceğini yazmaz. Sayıyı sonradan değiştirmek başka bir harita kurmaktır. **Spec boşluğu 11.**
- **D13 — Yerleşik "Prashna" danışanı adından değil kaydından tanınır: `app_state` anahtarı `prashna_client_uuid`. İlk danışansız prashnada aynı işlemde oluşturulur (ad "Prashna": klasik terim, üç dilde aynı). Kullanıcı adını değiştirebilir; silerse sonraki danışansız prashna yenisini açar; "Prashna" adlı başka bir danışan yerleşik sayılmaz. Kayıt dışa aktarılmaz; içe alınan dosyadaki danışan sıradan bir danışandır.** Gerekçe: spec "Prashna adlı yerleşik danışan altında toplanır (ilk kullanımda oluşturulur)" der; ada göre tanımak, aynı adlı gerçek bir danışanı ya da yeniden adlandırmayı bozar. **Spec boşluğu 12.**
- **D14 — Dışa aktarım: prashna haritasının JSON kaydı `prashna: { number, focus_house, question, created_at }` alanını kazanır; `question` yalnızca "notları dahil et" işaretliyse yazılır, değilse `null`'dır. Prashna olmayan haritada alan hiç yazılmaz; `EXPORT_SCHEMA_VERSION` 1'dir ve değişmez (eklenen alan isteğe bağlıdır). İçe aktarım satırı geri kurar; geçersiz sayı, odak evi ya da fazla uzun soru kaydın UUID'siyle ve `prashna` alanıyla reddedilir. CSV dışa aktarım soru metnini taşımaz.** Gerekçe: spec yalnızca soru metninin koşulunu yazar. Sayı ve odak evi dosyaya yazılmazsa dışa aktarılıp geri alınan bir sayı prashnası sessizce başka bir lagnayla açılırdı. **Spec boşluğu 13.** **Son inceleme kararı (`84619c0`):** dosyadaki `number` ve `focus_house` ham JSON değeri olarak okunur; aralık dışı, `u8`'e sığmayan, negatif, kesirli ya da sayı olmayan her değer kayıt hatasıdır (`ImportRecord { item: UUID, field: "prashna" }`), dosya biçimi hatası değildir ve değerin kendisi hata iletisine girmez (önceden `u8`'e sığmayan değer `import_format { detail }` içinde yineleniyordu). İki alanı hiç taşımayan kayıt geçerlidir.
- **D15 — Komutlar: `create_prashna(request: PrashnaRequest) -> OpenedClient`, `update_prashna(chart_id, edit: PrashnaEdit) -> PrashnaMeta`, `compute_prashna_facts(input: ChartInput, focus_house: Option<u8>) -> PrashnaFacts`. `PrashnaRequest { client_id: Option<u32>, at: PrashnaMoment, place: HomePlace, number, question, focus_house }`; `PrashnaMoment { Now, Local { date, time } }`; `PrashnaEdit { question, focus_house }`. Hiçbir komut doğrudan dizgi bağımsız değişken almaz (soru bir nesnenin alanıdır): `api.test.ts`'in dizgi bağımsız değişken listesi değişmez. `OpenedClient` `prashna: Option<PrashnaMeta>` kazanır.** Gerekçe: spec `create_prashna(client_id, at, place, number, question, focus_house) -> OpenedChart` yazar: `place`'in tipi belirsizdir (`Place` saat dilimi taşımaz; Plan 20 D21'in aynısı), dönüş tipi `OpenedChart` diye bir tip yoktur ve soru metni çıplak bir dizgi bağımsız değişkeni olurdu (C2′ her dizgi bağımsız değişkenini adıyla listeler). **Spec kusuru 4.**
- **D16 — `open_client` dönüşü değişir ve sayı prashnasının haritası her hesapta sayının lagnasıyla kurulur: `open_client_blocking` ve `save_calc_settings_blocking` haritayı `compute_chart_for_blocking(engine, ChartInput)` ile hesaplar (sayı yan tablodan). `Chart` tipi değişmez; `open_client` bütçesi (≤ 50 ms) ölçülür.** Gerekçe: K1 "`Chart` ve `open_client` değişmez" der; §11 "`open_client` dönüşüne `prashna: Option<PrashnaMeta>` eklenir" der ve sayı prashnasının lagnası `open_client`'ın hesapladığı haritadadır. §11 daha özeldir; K1'in amacı (bütçe, modüllerin ayrı komutları) korunur: yan kayıt tek bir satır okumasıdır ve modül sonucu `open_client`'a girmez. **Spec kusuru 1.**
- **D17 — Soru anı: `Now` Rust'ta okunur ve sorunun yerinin duvar saatine çevrilir (`atlas::offset_at`); duvar saati yaz saati örtüşmesine düşüyorsa `now`'ı veren taraf seçilir ve kayda yazılır (yerel ortalama saatte ofset saniyeye yuvarlandığı için bir saniye pay tanınır). `Local { date, time }` sorunun yerinin duvar saatidir (tarayıcının dilimi değil); boşluğa ya da örtüşmeye düşen ya da okunamayan an `at` alanıyla reddedilir, sessizce kaydırılmaz. Pencere örtüşme için ayrı bir seçim sunmaz: yılda bir saatlik bu durumda astrolog haritayı bir dakika öncesiyle kurup ⌘E ile düzeltir (doğum formu örtüşmeyi açık seçimle çözer) ve hata iletisi bunu söyler.** Gerekçe: spec "an = şimdi (ya da girilen an)" der; girilen anın hangi dilimde olduğunu ve yaz saati geçişlerini yazmaz. Harita yerle birlikte saklandığı için anın yerin saati olması doğum verisiyle aynı modeldir (UI spec §4: "hiçbir zaman sessizce kaydırılmaz"). **Spec boşluğu 14.**
- **D18 — Haritanın etiketi Rust'ta yazılır: `Prashna YYYY-MM-DD SS:DD` (sorunun yerinin duvar saati). Kullanıcı ⌘E ile değiştirebilir.** Gerekçe: spec etiketi yazmaz; boş etiket harita listelerinde "Olay haritası" görünürdü ve prashnalar birbirinden ayırt edilmezdi. Etiket veritabanındadır, log değildir. **Spec boşluğu 15.**
- **D19 — "Yeni prashna" penceresi (`prashna/PrashnaForm.tsx`, `shell/Dialog` `variant="sheet"`): soru anı (Şimdi / Başka bir an: tarih ve saat), yer (kaynak seçici: bulunulan yer / açık haritanın yeri / başka bir yer ara; seçili yer ve dilimi bir durum satırında), sayı (isteğe bağlı, 1–108), soru (2 000 karakter sayacıyla), odak evi (Seçilmedi / 1–12) ve, açık danışan varsa, "… danışanına ekle" kutusu (işaretli gelir; işaret kalkarsa ya da açık danışan yoksa prashna yerleşik danışana gider). Bulunulan yer seçilmemişse pencere yer ister ve haritanın yerine sessizce düşmez (Plan 20 D29). Pencere yalnızca biçim denetler (sayı tamsayı mı, tarih ve saat yazılmış mı); Rust'ın alan adlı hatası alanın altında sabit bir iletiyle gösterilir. Değişmiş taslakta Esc onay sorar. Açılış: ⌘⇧P ve paletteki "Yeni prashna" komutu (açık harita gerekmez); baskı önizlemesinde tuş tüketilir ama pencere açılmaz.** Gerekçe: spec alanları sayar; yer kaynağını, danışan seçimini ve hataların gösterimini yazmaz. "Danışan seçili değilse" koşulu pratikte nadirdir (oturum son danışanı geri yükler): kutu, astrologun açık bir danışan varken de danışansız prashna kurabilmesini sağlar. ⌘⇧P bir kısayol ekler: Plan 10'un C16 kuralı ("⌘ kısayolları değişmez") bu planda "var olan kısayollar değişmez; ⌘⇧P eklenir" olarak okunur. **Spec boşluğu 16.** **Son inceleme kararı (`4790146`):** an "Şimdi" iken gelen `at` hatasının kendi iletisi vardır (`prashna.form.error.atNow`): düzeltilecek tarih ve saat yoktur; ileti yeniden denemeyi ya da "Başka bir an"ı önerir.
- **D20 — "Prashna kur" haritayı açar ve etkin düzeni "Prashna" hazır düzenine alır. Düzen dördüncü hazır düzendir (`builtin:prashna`: D1, `prashna_facts`, `panchang`, notlar) ve ⌘4'tür; kullanıcının kendi düzenleri ⌘5'ten başlar (önceden ⌘4'ten başlıyordu). Kayıtlı düzen durumunda bu düzen yoksa toleranslı okuma ekler (var olan kural). `prashna_facts` ayrıca `MODULE_PANELS`'tedir (7 × 20) ve paletten eklenip çıkarılır (K12).** Gerekçe: spec "Yerleşik düzenlere 'Prashna' düzeni eklenir: D1, `prashna_facts`, `panchang`, notlar" der; K12 ise "Hazır düzenlere modül paneli eklenmez" der. §11 daha özeldir: Prashna düzeni modül paneli taşıyan tek hazır düzendir ve test bunu adıyla sabitler. Kurulan prashnanın olgularını göstermeyen bir düzende açılması, pencerenin amacını boşa çıkarırdı. Kısayol kayması bir sürüm notu maddesidir. **Spec kusuru 3.**
- **D21 — `prashna_facts` paneli: üstte soru metni (düzenlenebilir; yazma durduktan 600 ms sonra ve panel kapanırken kaydedilir; durum satırı sayacı, "Kaydedildi"yi ya da "çok uzun"u söyler; kayıt hatası kendi satırında ve "Yeniden dene" ile); araç çubuğunda odak evi seçicisi (değişince hemen kaydedilir) ve sayı prashnasında "sayı: n"; sonra bölümler: Lagna (burç ve derece, sayı prashnasında "n. navamsanın ortası"; tür; doğuş biçimi), Lordlar (tablo: lord, graha, burç, ev, dignity, durum; altında iki lordun bağı ve ev sayıları), Kendralar ve odak evi (tablo; sınıf sözle), Ay, Anın panchangı (beş uzuv, açık haritadan), Yogalar (Plan 18'in listesi), okuma notu ve yöntem kimliği. Harita prashna değilse panel bunu bir notla söyler ve komut çağırmaz. Harita başlığında sayı prashnası "sayı: n" taşır.** Gerekçe: spec panelin içeriğini sayar ("odak evi panel başlığından değiştirilir. Soru metni panelin üstünde, düzenlenebilir"); sütunları ve kayıt davranışını yazmaz. Burçlar öteki tablolardaki gibi kısaltmayla yazılır. **Spec boşluğu 17.** **Son inceleme kararı (`7262d05`):** odak evi seçimi panelin kendi durumudur (taslak soru gibi): seçici ve olgular seçilen evi gösterir, her kayıt (soru zamanlayıcısı, panel kapanışı, yeniden deneme) seçilen evi taşır; böylece sırada bekleyen bir soru kaydı yeni evi eskisiyle ezmez ve düşen bir odak kaydında seçim geri atlamaz, "Yeniden dene" aynı evi gönderir. Taslak soru 2 000 karakteri aşıyorsa odak evi kaydı ve yeniden deneme kayıtlı soruyla gider (taslak ve uyarısı yerinde durur). Kendralar tablosunun graha hücresi `aria-label` taşımaz: adını içeriğinden alır.
- **D22 — Yeniden okuma ve odak: olgular `useModule` ile istenir ve odak evi değişince panel son okunan olguları göstermeyi sürdürür (`useLastData`, kapsam harita kimliği): bölümler DOM'da durur, panel `aria-busy` olur, odak seçicide durur. Hatadan sonraki okuma (yeniden deneme dahil) boş başlar: hatadan önceki olgular gösterilmez. Soru kutusu ve odak evi seçicisi hata durumunda da yerindedir. Başka bir prashna açılınca taslak sıfırlanır (bileşen harita kimliğiyle anahtarlanır) ve önceki haritanın bekleyen metni kendi haritasına kaydedilir. `session.setPrashna(chartId, meta)` yalnızca açık ve prashna olan haritaya yazar.** Gerekçe: Plan 16, 19 ve 20'nin dersleri (liste ve araç çubuğu yeniden okumada sökülmez; odaktaki denetimi götüren eylem odağı bilerek taşır; son veriyi tutan kanca aynı kapsamda yeniden denemede hata öncesi veriyi göstermez).
- **D23 — "Saat bilinmiyor" durumu: prashna her zaman kesin saatle kurulur (`TimeCertainty::Exact`). Durum yalnızca prashna haritası doğum formundan saati bilinmeyen bir kayda çevrilirse oluşur; o zaman `time_known` yanlış gider, Rust lagnaya bağlı bölümleri boş döner ve panel bunu bir notla söyler (Ay, anın panchangı ve yogalar durur).** Gerekçe: §13 her panel için "saat bilinmiyor" durumunu ister; K2 gizlemeyi Rust'a verir. **Spec boşluğu 18.**
- **D24 — Bütçeler entegrasyon testinde ölçülür (hata ayıklama derlemesi): `compute_prashna_facts` ≤ 15 ms (temel harita dahil), `create_prashna` ≤ 100 ms uçtan uca (kayıt ve açılış), `open_client` ≤ 50 ms.** Gerekçe: ölçüm (ortanca): `compute_prashna_facts` 0,8–1,9 ms; `create_prashna` 1,6 ms; `open_client` 1,2–1,4 ms. Bütçelerin onda birinden azdır.
- **D25 — Arayüz fikstürü `prashna` ve `prashna_ids` kazanır: fikstürün sabit saatinde, açık danışanın altında ve Delhi'de kurulmuş iki harita (zaman prashnası ve 54 sayılı sayı prashnası; deneme sorusu), üç olgu sonucu ve Rust'ın `RisingMode::ALL`, `LordLink::ALL`, `PRASHNA_NUMBER_MAX`, `MAX_QUESTION_CHARS` değerleri. Fikstürdeki iki haritanın UUID'si sabitlenir. E2e taklidi haritaları ve olguları fikstürden, soru ve odak evi kaydını bellekten verir.** Gerekçe: arayüz hesap yapmaz; testler gerçek motorun ürettiği sonucu çizer (Plan 18 D17, Plan 20 D36).
- **D26 — Adlandırma: bu planın getirdiği klasik adların hiçbiri yasak diziyi içermez (prashna, lagna, navamsa, kendra, Shirshodaya, Prishthodaya, Ubhayodaya). Türkçe metinlerde dizi şu sözcüklerle oluşur ve yerlerine öteki sözcükler yazılır: "… sayı durur", "sürer", "tutar", "yerinde durur", "gider", "bulur". Sayı prashnasının yay birimleri ′ işaretiyle yazılır (Ek A).** Gerekçe: v1 spec §10 ve Ek A; yasak harf dizisi düzeyindedir.
- **D27 — Yeni görsel temel (PNG) ve yeni baskı düzeni yoktur (K10). `pnpm build` Vite'ın 500 kB parça uyarısını basmayı sürdürür (ölçüm: 643,5 kB) ve kapılar `✓ built` satırını arar (Plan 15 D22).** Gerekçe: on görsel temel modül paneli ve Prashna düzenini taşımaz (ölçüm: `e2e:visual` 10 geçti, temel dosyalar değişmedi).

## Global Constraints

- Lisans: AGPL-3.0-or-later. Yeni bağımlılık yok: `Cargo.toml`, `Cargo.lock`, `deny.toml`, `app/package.json`, `app/pnpm-lock.yaml` ve `app/vite.config.ts` değişmez; `cargo deny check` Task 8'de `advisories ok, bans ok, licenses ok, sources ok` verir.
- **Yasaklı ad kuralı (v1 spec §10 "İsim kuralları", ilk madde; ileri modüller spec'i Ek A):** o maddedeki dört ad ve türevleri hiçbir isimde, yorumda, metinde, dosya adında ya da commit mesajında kullanılmaz. Kural harf dizisi düzeyindedir: Türkçe sözcüklerin içinde de geçemez (yerine ör. "öteki", "durur", "geri", "sürer", "tutar", "bulur", "gider" yazılır; D26). `-ka` ile biten bir sözcüğe (`karaka`, "dakika") `-lar` çoğul eki ya da `L` ile başlayan bir ad bitişik yazılınca (camelCase'te de) yasak dizi oluşur; böyle birleşimler kurulmaz. Her commit adımı eklenen satırları, dosya adlarını ve commit mesajını `grep -inE 'k(a|ā|aa)la|\bcala\b'` ile tarar; çıktı boş olmalıdır (C13). Bu belgede de eşleşme yoktur.
- **Clean-room ve yorum metni yok:** kurallar klasik tanımdan özgün olarak uygulanır; başka ürünün adı, düzeni, metni ya da çıktısı kullanılmaz; kaynak gösterimi geneldir ("BPHS, burç nitelikleri bölümü") ve alıntı yapılmaz. Panel hesaplanmış olguları ve klasik etiketleri gösterir. **Konu çıkarımı, "olumlu/olumsuz/uğurlu" hükmü, evet/hayır sonucu, renkli derecelendirme ve bir olgunun ne anlama geldiğini anlatan metin eklenmez.** "İyicil" ve "kötücül" K6'nın doğal sınıfının adlarıdır ve yalnızca o bağlamda geçer. Odak evini astrolog seçer; modül sorudan konu çıkarmaz.
- **Çerçeveler (K4, K5):** evler lagnadan sayılan burç evleridir; burçlar rasi çerçevesindedir (`settings.zodiac`'ı izler); nakshatra, pada ve tithi sidereal boylamdandır. Sayının lagnası sidereal boylamdır; tropikal-rasi kipinde lagnanın burcu `sidereal + ayanamsa`'nın burcudur. Yanma v1'in kuralı ve haritanın orblarıdır.
- **Doğum saati bilinmiyorsa (K2, D23):** `time_known` yanlışsa `PrashnaFacts.lagna` ve `.focus` `None`'dır ve Ay'ın evi boştur; arayüzün gizlemesine güvenilmez.
- **Soru, yer ve an (spec §14, §11 "Gizlilik"; D10):** soru metni not gövdesi gibi hassastır; yer ve an kişisel veridir. (1) Soru metni, yer, an, sayı ve danışan adı hiçbir yerde loglanmaz. (2) Prashna yollarının girdi hataları yalnızca alan adı taşır (`invalid_prashna_input { field }`); soru, enlem, boylam, saat dilimi, tarih ve saat hata parametresine girmez. (3) Soru metni arama dizinine (FTS) eklenmez. (4) JSON dışa aktarımda soru yalnızca "notları dahil et" işaretliyse yazılır; CSV'de hiç yoktur. (5) Ağ isteği yoktur; konum servisi kullanılmaz. (6) Olgular veritabanına yazılmaz, oturumda saklanmaz. (7) Elle denetimde yalnızca deneme danışanları ve deneme soruları kullanılır. (8) Testlerdeki ve fikstürdeki sorular uydurma deneme metinleridir.
- **El örnekleri (§13, K9):** `source` yalnızca `hand`'dir; beklenen değer kuraldan elle türetilir ve `derivation`'da adım adım yazılır. Bir el örneği motorla uyuşmazsa önce kural yeniden okunur; **beklenen değer motor çıktısına bakarak düzeltilmez.** Gözden geçirme döngüsünün zorunlu maddesi: motoru yazan dışında bir gözden geçiren on dört örneği kuraldan yeniden hesaplar ve `REVIEW.md`'deki tabloya adını yazar (Task 8 Step 5). **Bellekten aktarılan klasik tablo ve yöntem** (doğuş biçimi; sayının lagnası) sahibin onayını bekler: bilinen varyant noktalarıyla `REVIEW.md`'de sayılıdır (Task 8 Step 6); yürüten ajan tabloyu değiştirmez ve başka bir yazılımın çıktısıyla ya da çevrimiçi bir tabloyla karşılaştırmaz.
- Kapılar: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` (1.100), `cargo test --workspace`, `pnpm --dir app test`, `pnpm --dir app build` her görev sonunda yeşil; `pnpm --dir app e2e` Task 7 ve 8'de. `tsc` `strict`. `pnpm build` çıktısında aranan satır `✓ built`'tir (D27). Tek bir Vitest dosyası `pnpm --dir app exec vitest run ui/…` ile koşulur: yol `app/` dizinine göredir. `cargo test --workspace` çıktısı uzun olduğu için başarısızlık şu sayaçla denetlenir: `cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."` `0` yazmalıdır.
- **Toolchain:** `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"` (oturum başına bir kez). Node 24+, pnpm 11.8.0. Depo kısa bir yolda durmalıdır: Swiss Ephemeris 242 bayttan uzun ephemeris yolunu reddeder (çalışma ağacı uzun bir yoldaysa `/tmp` altında kısa bir yola klonlanır).
- **Çalışan `pnpm tauri dev` durdurulmaz** (port 1420); e2e 1421'i kullanır. `pkill -f` kullanılmaz. Üretilen `.app` çalıştırılmaz. `~/Library/Application Support/io.github.turgaydogusan.jyotishdesk` dizinine dokunulmaz (gerçek veritabanı bu planın göçünü ancak sahibi uygulamayı açınca görür; Task 8 Step 4'ün **[insan]** maddesi bunun için önce yedek ister).
- Yalnızca macOS hedeflenir. Yorum metni ve AI paneli bu planın dışındadır. Arama (3.9) bu planın dışındadır.
- **Tabanlar (`0b61716`):** `jyotish-core` lib 351, golden 8, `modules_golden` 15; `atlas` `moment_fixed_lmt` 11, `moment_iana` 13 (`build` 6, `search` 8, `shipped` 3); `store` lib 103 (+1 `#[ignore]`); `jyotish-desk` lib 49, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; Vitest `Test Files  80 passed (80)`, `Tests  806 passed (806)`; e2e 22; görsel 10 (yerel, darwin); yayın betiği testleri 14.
- **Sayaç zinciri** (her satır görev bitince; boş hücre "önceki satırla aynı"). Bu plan yazılırken `0b61716`'in bir kopyasında her görev ayrı commit olarak uygulanıp ölçülmüştür.

  | Görev | `jyotish-core` lib | `modules_golden` | `store` lib | `jyotish-desk` lib | Vitest dosya / test | e2e |
  |---|---|---|---|---|---|---|
  | Taban | 351 | 15 | 103 (+1 ign.) | 49 | 80 / 806 | 22 |
  | Task 1 | 363 | | | | 80 / 806 | |
  | Task 2 | 374 | 17 | | | | |
  | Task 3 | | | 112 (+1 ign.) | | | |
  | Task 4 | | | | 54 | 80 / 806 | |
  | Task 5 | | | | | 82 / 822 | |
  | Task 6 | | | | | 83 / 840 | |
  | Task 7 | | | | | | 24 |

  Değişmeyenler: `jyotish-core` golden 8; `atlas` 11 / 13, `build` 6, `search` 8, `shipped` 3; `jyotish-desk` `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; görsel 10; yayın betiği testleri 14. `jyotish-core` lib Task 1'de 12 test (`prashna/mod.rs` 3, `chart.rs` 6, `positions.rs` 1, proptest 2), Task 2'de 11 test (`prashna/facts.rs` 10, proptest 1) kazanır; `modules_golden` iki test kazanır (`prashna_hand_fixtures`, `prashna_fixtures_cover_the_required_cases`). `store` lib dokuz test (`prashna.rs` 8, `schema.rs` 1), `jyotish-desk` lib beş test (`prashna_commands.rs`) kazanır. `jyotish-desk` `modules` ve `open_client` birer test olarak durur (süreçte tek SE örneği); Task 4 onlara `prashna_checks`'i ve `prashna_round_trip`'i ekler. Vitest: Task 1 ve 4'te sayı değişmez (var olan testlerin beklentileri ve tabloları büyür); Task 5 iki dosya ve 16 test (`prashna/model.test.ts` 6, `prashna/PrashnaForm.test.tsx` 9, `App.test.tsx` 1), Task 6 bir dosya ve 18 test (`PrashnaFactsPanel.test.tsx` 15, `ChartTitle.test.tsx` 1, `session/store.test.tsx` 1, `App.test.tsx` 1) ekler. Task 7 iki e2e testi ekler.
- **Taşınan kurallar (Plan 09 C1–C14, Plan 10 C2′, C15–C17), her görevin gereksinimidir; bu planda değişen okumalar açıkça yazılıdır:**
  - **C1** Gizlilik (spec §14): danışan adları, doğum verisi, not gövdeleri, soru metinleri, yer bilgisi, dosya yolları ve `CommandError.params` loglanmaz; `no-console.test.ts` durur ve izin listesi büyümez. Hiçbir modül ağ isteği yapmaz. Yeni tek `CommandError` kodu `invalid_prashna_input`'tur (parametresi yalnızca `field`); `not_found` bir yeni `what` değeri taşır (`prashna`).
  - **C2′** Hiçbir IPC komutu arayüzden dosya yolu ya da adres almaz. Bu planın üç komutu doğrudan dizgi bağımsız değişken almaz (D15); `api.test.ts`'in dizgi bağımsız değişken listesi değişmez.
  - **C3 (bu planda değişen okuma, D11)** Göç 1 değişmez; `crates/store/src/schema.rs` yalnızca göç 2'yi ve testini kazanır; `SCHEMA_VERSION` 2 olur. `CalcSettings`, `ViewState` ve `AppPrefs` tipleri ve bütün öteki şema sürümleri (`CALC_SETTINGS_SCHEMA_VERSION`, `VIEW_STATE_SCHEMA_VERSION`, `APP_PREFS_SCHEMA_VERSION`, `EXPORT_SCHEMA_VERSION`) değişmez.
  - **C4** Arayüz astrolojik hesap yapmaz: lagna, lordlar, bağlar, sınıflar, yogalar ve anın çözümü Rust'tan gelir; "şimdi" Rust'ta okunur. Sayının 1–108 arasında bir tamsayı olup olmadığına bakmak, karakter saymak, derecenin "28°20′" diye yazılması, kimliğin adına çevrilmesi ve ev vurgusunun burca eşlenmesi gösterimdir.
  - **C5/C17** Otomatik kayıtlar ve boşaltılmaları değişmez; soru ve odak evi kendi sırasıyla kaydedilir ve panel kapanırken bekleyen metin gönderilir. **C6** Düzen ayrıştırıcıları (`parseLayoutState`, Rust `check_panel_layout`) değişmez; bir panel türü ve bir hazır düzen kataloğa eklenir.
  - **C7** Kısayollar yalnızca `shortcuts.ts`'tedir. **C16 (bu planda değişen okuma, D19)** var olan ⌘ kısayolları değişmez; ⌘⇧P eklenir. **C8** Her pencere `shell/Dialog`'u kullanır ("Yeni prashna" dahil).
  - **C9** Renk yalnızca token'lardan gelir (hex yok) ve tek başına anlam taşımaz: doğal sınıf sözle, retro ve yanma sözle, bağlar kenarlıklı metin etiketleriyle, vurgu zemin ve `aria-pressed` ile gösterilir.
  - **C10** Üretilen dosyalar elle düzenlenmez: `app/ui/bindings.ts` ve `app/ui/test/fixtures/delhi.json` Task 1 ve Task 4'te komutla yeniden üretilir.
  - **C11** Görsel PNG temelleri yereldir; bu planda değişmeleri beklenmez (D27).
  - **C12** Her `CommandError.code` üç dilde çevrilidir (bu plan birini ekler: Task 1). Her yeni i18n anahtarı üç dilde vardır (`i18n.test.ts` anahtar kümelerini ve yer tutucuları karşılaştırır). Fransızca metinler tipografik kesme işareti (’) kullanır; testler düz kesme işaretinin yokluğunu sabitler.
  - **C13** Yasaklı ad taraması her commit'te; depo dışındaki ilerleme notları ve `.superpowers/` hiçbir zaman `git add` edilmez.
  - **C14** Plan 07'nin gün doğumu toleransı ve `compute_panchang` değişmez (`panchang.rs`'e dokunulmaz). **C15** `alert`, `confirm`, `prompt`, `window.print()` ve `window.open()` çağrılmaz.
- **Retro** her yerde düz `R`'dir ve Rahu ile Ketu'da gösterilmez (UI spec §2.3); olgu panelinde lordun durumu sözle yazılır ("retro", "yanık"): lordlar yedi grahadan biridir.
- **Erişilebilirlik:** WCAG AA. Tablolar `caption`, `scope="col"` ve `scope="row"` taşır ve klavye odağı alan kaydırma kaplarındadır (`tabIndex=0`, `role="group"`, adları bölüm başlıklarıdır); bölümler adlı `region`'lardır; iki lordun bağı adlı bir listedir; form alanlarının görünür etiketi vardır, hata alanın açıklamasına bağlanır (`aria-describedby`) ve gönderimden sonra odak ilk hatalı alana gider; seçenek kümeleri `fieldset` ve `legend` taşır; durum satırları `role="status"`, hatalar `role="alert"`'tir. **Yeniden okuma içeriği DOM'dan çıkarmaz ve odağı düşürmez (D22).**
- Commit mesajları İngilizcedir ve deponun `Co-Authored-By:` satırıyla biter (`Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`). Her görev tek commit'tir (gerekirse inceleme düzeltmesi ayrı commit). `git push`, `git tag`, yıkıcı git komutu ve `.superpowers/` commit'i yoktur.
- Kod blokları `0b61716` ağacına göre yazılmış ve bu plan yazılırken deponun bir kopyasında koşulmuştur (Self-Review madde 5). Yeni dosyalar tam içerikle verilir ("**Yeni dosya:**"). Değişen dosyalar `diff` bloğuyla verilir: blok bir dosyaya konup depo kökünde `git apply <dosya>` ile ya da elle uygulanır; bağlam tutmazsa önceki görevin adımı denetlenir, elle uydurma yapılmaz. Bir dosyanın test ve uygulama değişiklikleri ayrı adımlarda ayrı bloklardır; yeni bir Rust dosyasının `mod tests` bölümü dosyayla birlikte gelir (dosyayı yazarken önce testleri okuyun). Dosya ve diff blokları dört ters tırnakla çevrilidir (içlerinde üç ters tırnak geçebilir); komut blokları üç ters tırnaklıdır. `app/ui/bindings.ts` ve `app/ui/test/fixtures/delhi.json` diff olarak verilmez; adımlardaki komutlar üretir.

## Sonraki alt projelere verilen API

Bu planın ürettiği ve sonraki işlerin tüketeceği adlar. Rust adları yıldızlıysa `jyotish_core` kökünden dışa aktarılır; imzalar görevlerde tam olarak verilir.

| API | Tüketen | Not |
|---|---|---|
| `compute_chart_with_lagna`\*, `compute_chart_for`\*, `number_lagna_lon`\*, `PRASHNA_NUMBER_MAX`\*; `ChartInput.lagna_number` | 3.9 `search_facts` (prashna haritaları da aranır); AI paketleyicisi | Sayı prashnasında her modül aynı lagnayı görür |
| `prashna_facts`\*, `compute_prashna_facts`\*, `lord_relation`\*, `moon_facts`\*, `rising_mode`\*, `PrashnaFacts`\*, `LagnaFacts`\*, `FocusFacts`\*, `LordFacts`\*, `LordRelation`\*, `LordLink`\*, `MoonFacts`\*, `HouseOccupants`\*, `Occupant`\*, `RisingMode`\*, `PRASHNA_METHOD`\* | AI paneli; 3.9 (lord ilişkisi koşulu) | Saf; ephemeris görmez |
| `CoreError::InvalidPrashnaInput`; `CommandError` kodu `invalid_prashna_input` | 3.9 | Yalnızca alan adı |
| `store`: `PrashnaMeta`, `NewPrashna`, `Store::{create_prashna, update_prashna, prashna_meta, prashna_client_id}`, `MAX_QUESTION_CHARS`, `MAX_PRASHNA_NUMBER`, `PRASHNA_CLIENT_NAME`; `SCHEMA_VERSION` 2 | 3.9 (`saved_searches` aynı göç kalıbıyla tablo ekleyebilir) | Soru aranmaz; notlarla dışa aktarılır |
| `prashna_commands::{PrashnaRequest, PrashnaMoment, PrashnaEdit, prashna_birth, create_prashna_blocking, update_prashna_body, compute_prashna_facts_blocking}`; `commands::compute_chart_for_blocking`; `OpenedClient.prashna` | 3.9; AI paneli | Girdi hataları alan adına iner |
| Arayüz: `prashna/model.ts` (`toRequest`, `draftPlace`, `errorField`), `prashna/PrashnaForm.tsx`, `modules/usePrashnaFacts.ts`, `panels/PrashnaFactsPanel.tsx`, `panels/YogasPanel.tsx` (`YogaList` dışa aktarılır), `panels/labels.ts` (`RISING_NAMES`), `session.setPrashna`, `BUILTIN_LAYOUT_IDS.prashna`; fikstürde `prashna`, `prashna_ids` | AI paneli; 3.9 | i18n: `prashna.*`, `panel.prashna_facts`, `layout.prashna` |

## Dosya Yapısı (bu plan)

```
crates/jyotish-core/
  src/chart.rs                    chart_with, compute_chart_with_lagna, compute_chart_for
  src/positions.rs                ChartInput.lagna_number; compute_positions haritayı compute_chart_for'dan alır
  src/prashna/mod.rs              number_lagna_lon, PRASHNA_NUMBER_MAX (yeni)
  src/prashna/facts.rs            prashna_facts, compute_prashna_facts ve tipleri (yeni)
  src/error.rs                    InvalidPrashnaInput
  src/jaimini/mod.rs, src/jaimini/chara_dasha.rs, src/shadbala/collect.rs   compute_chart_for
  src/drishti.rs, src/yogas/mod.rs, src/kuta/mod.rs                        testlerdeki ChartInput değişmezleri
  src/lib.rs                      modül kaydı ve dışa aktarımlar
  src/proptests.rs                prashna_properties
  tests/modules_golden.rs         prashna fikstür düzeneği ve kapsam testi
tests/golden/modules/
  README.md                       prashna satırı
  prashna/REVIEW.md               okuma kuralları, tablolar, onay bekleyen noktalar, kayıtlar (yeni)
  prashna/*.json                  14 el örneği (yeni)
crates/store/
  src/schema.rs                   göç 2: prashna tablosu; SCHEMA_VERSION 2
  src/model.rs                    PrashnaMeta, NewPrashna
  src/prashna.rs                  create_prashna, update_prashna, prashna_meta, yerleşik danışan (yeni)
  src/open.rs                     OpenRecord.prashna
  src/compat.rs                   chart_input sayıyı taşır
  src/exchange.rs                 ExportChart.prashna; içe aktarım
  src/db.rs, src/lib.rs
app/src-tauri/
  src/commands.rs                 invalid_prashna_input eşlemesi; compute_chart_for_blocking
  src/prashna_commands.rs         create_prashna, update_prashna, compute_prashna_facts (yeni)
  src/client_commands.rs          OpenedClient.prashna; sayı prashnasının haritası
  src/lib.rs                      komut kaydı
  tests/modules.rs                prashna_checks: çapraz denetim, sayı değişmezi, hatalar, bütçe
  tests/open_client.rs            prashna_round_trip: gidiş-dönüş ve create_prashna bütçesi
  tests/ui_fixture.rs             fikstüre prashna ve prashna_ids
app/ui/
  bindings.ts                     üretilir
  test/fixtures/delhi.json        üretilir
  test/fixture.ts                 UiFixture'ın iki yeni alanı
  api.ts, api.test.ts             üç sarmalayıcı
  modules/input.ts (+test)        chartInput sayıyı taşır
  modules/usePrashnaFacts.ts      kanca (yeni)
  prashna/model.ts (+test)        taslak, istek, alan eşlemesi (yeni)
  prashna/PrashnaForm.tsx (+test) "Yeni prashna" penceresi (yeni)
  prashna/prashna.css             (yeni)
  panels/PrashnaFactsPanel.tsx (+test)  (yeni)
  panels/YogasPanel.tsx           YogaList dışa aktarılır
  panels/labels.ts                RISING_NAMES
  panels/registry.ts
  layout/model.ts (+test), layout/reducer.test.ts   prashna_facts türü, MODULE_PANELS, Prashna hazır düzeni
  session/store.tsx (+test)       setPrashna
  shell/Shell.tsx, shell/ChartTitle.tsx (+test), shell/LayoutPicker.test.tsx
  shortcuts.ts (+test)            ⌘⇧P
  palette/Palette.tsx, palette/items.ts (+test)
  view/store.test.tsx, App.test.tsx
  dasha/*.test.tsx, panels/{Drishti,JaiminiKarakas,Shadbala}Panel.test.tsx   girdi beklentileri
  i18n/{tr,en,fr}.ts, i18n.test.ts      errors.invalid_prashna_input, shortcuts, palette, panel, layout, prashna
app/e2e/{ipc.ts,desk.spec.ts}     üç komutun taklidi; iki akış
```

### Kapsam

| Spec maddesi | Yer |
|---|---|
| §11 Zaman prashnası: an = şimdi ya da girilen an, yer = `home_place`; `compute_chart` ile aynı | Task 4 (`prashna_birth`; D17), Task 5 (pencere; D19) |
| §11 Sayı prashnası: `lagna_lon = (n − 1) × 3°20′ + 1°40′`; `compute_chart_with_lagna` | Task 1 (D2, D3, D4) |
| §11 Odak evi: astrolog seçer; modül konu çıkarmaz | Task 2 (`focus_house`), Task 5, Task 6 |
| §11 Olgular: lagna (burç, tür, doğuş biçimi) | Task 2 (D5, D8); Task 8 Step 6 |
| §11 Olgular: lagna lordu ve odak evinin lordu; aralarındaki ilişki ve ev sayıları | Task 2 (D5, D7) |
| §11 Olgular: Ay (burç, ev, nakshatra, tithi ve paksha, aldığı bakışlar) | Task 2 (D9) |
| §11 Olgular: kendralardaki ve odak evindeki grahalar, K6 sınıfıyla | Task 2 |
| §11 Olgular: anın panchangı (v1) ve yoga listesi | Task 2 (yogalar), Task 6 (panchang açık haritadan; D6) |
| §11 Saklama: `prashna` tablosu; `charts.kind` değişmez; soru en çok 2 000 karakter | Task 3 (D11, D12) |
| §11 Komutlar: `create_prashna`, `update_prashna`, `compute_prashna_facts`; `open_client` dönüşüne `prashna` | Task 4 (D15, D16) |
| §11 Arayüz: palet ve ⌘⇧P "Yeni prashna"; sağdan kayan panel; "Prashna kur" | Task 5 (D19) |
| §11 Arayüz: danışan seçili değilse yerleşik "Prashna" danışanı | Task 3 (D13), Task 4, Task 5 |
| §11 Arayüz: `prashna_facts` paneli; odak evi panelden; soru üstte, düzenlenebilir | Task 6 (D21, D22) |
| §11 Arayüz: sayı prashnasında "sayı: n" rozeti; "navamsa ortası" notu | Task 6 |
| §11 Arayüz: "Prashna" hazır düzeni | Task 6 (D20) |
| §11 Test: el örnekleri (1, 9, 10, 54, 108; 0 ve 109 hata; on iki burcun doğuş biçimi; lord ilişkilerinin beş türü) | Task 1 (`prashna/mod.rs`), Task 2 (on dört fikstür, kapsam testi, `HAND_RISING`) |
| §11 Test: property (sayı yalnızca lagnayı ve evleri değiştirir; gerçek lagna aynı haritayı verir) | Task 1 (`prashna_properties`), Task 4 (gerçek ephemeris) |
| §11 Test: store (silme, dışa aktarımda soru yokluğu, gidiş-dönüş) | Task 3, Task 4 (`prashna_round_trip`) |
| §11 Gizlilik: soru loglanmaz, hata iletisine girmez, notlarla dışa aktarılır, FTS'e eklenmez | Task 3 (testler), Task 4 (D10); Task 8 Step 3 |
| §11, §15 Performans: `create_prashna` ≤ 100 ms; `compute_prashna_facts` ≤ 15 ms | Task 4 (D24) |
| §13 Panel durumları: boş, yükleniyor, hata, dolu, "saat bilinmiyor"; ortak seçim; i18n | Task 6 |
| K12 Paletten ekleme | Task 6, Task 7 |
| Uçtan uca akış | Task 7 |
| Son doğrulama, elle denetim, el örneklerinin gözden geçirilmesi, klasik tablonun onayı | Task 8 |

---

### Task 1: Çekirdek: lagnası verilen harita, sayının lagnası, modül girdisinde sayı ve yeni hata türü

Spec §11 "Sayı prashnası" ve "Test" (sayı → lagna; iki property); K2, K3, K5, K8; D2, D3, D4, D10. `compute_chart`'ın davranışı değişmez (gövdesi `chart_with` olur ve sayı yokken aynı yoldan geçer); `ChartInput` bir alan kazanır ve toplayıcılar haritayı `compute_chart_for`'dan alır. Yeni hata türü çekirdeğe eklendiği için uygulamanın hata eşlemesi ve üç dildeki iletisi de bu görevdedir (derleme ve C12 bunu ister); `ChartInput` değiştiği için bağlamalar ve fikstür yeniden üretilir ve arayüzün girdi yardımcısı alanı `null` olarak doldurur (sayıyı Task 4 doldurur).

**Files:**
- Create: `crates/jyotish-core/src/prashna/mod.rs`
- Modify: `crates/jyotish-core/src/chart.rs`, `crates/jyotish-core/src/positions.rs`, `crates/jyotish-core/src/error.rs`, `crates/jyotish-core/src/lib.rs`, `crates/jyotish-core/src/jaimini/mod.rs`, `crates/jyotish-core/src/jaimini/chara_dasha.rs`, `crates/jyotish-core/src/shadbala/collect.rs`, `crates/jyotish-core/src/drishti.rs`, `crates/jyotish-core/src/yogas/mod.rs`, `crates/jyotish-core/src/kuta/mod.rs`, `crates/jyotish-core/src/proptests.rs`, `crates/store/src/compat.rs`, `app/src-tauri/src/commands.rs`, `app/src-tauri/tests/modules.rs`, `app/src-tauri/tests/ui_fixture.rs`, `app/ui/modules/input.ts`, `app/ui/modules/input.test.ts`, `app/ui/dasha/CharaDashaStrip.test.tsx`, `app/ui/dasha/CharaDashaTree.test.tsx`, `app/ui/panels/DrishtiPanel.test.tsx`, `app/ui/panels/JaiminiKarakasPanel.test.tsx`, `app/ui/panels/ShadbalaPanel.test.tsx`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`, `app/ui/i18n/i18n.test.ts`; üretilen: `app/ui/bindings.ts`, `app/ui/test/fixtures/delhi.json`
- Test: `prashna/mod.rs`'in `mod tests`'i (3), `chart.rs` (6), `positions.rs` (1), `proptests.rs` `prashna_properties` (2); var olan testlerin `ChartInput` değişmezleri ve beklentileri

**Interfaces:**
- Consumes (v1, Plan 14): `ephemeris::Ephemeris`, `CalcSettings`, `chart::{check_moment, raw_grahas, rasi_frame, to_sidereal}`, `houses::{Lagna, whole_sign_cusps, whole_sign_house}`, `zodiac::{normalize, nakshatra_pada, Sign}`, `positions::{ChartInput, Positions}`.
- Produces:
  - `jyotish_core::{compute_chart_with_lagna(eph, jd_ut, geo_lat, geo_lon, settings, lagna_sidereal_lon) -> Result<Chart, CoreError>, compute_chart_for(eph, &ChartInput) -> Result<Chart, CoreError>}`; crate içi `chart::from_sidereal`.
  - `jyotish_core::prashna::{number_lagna_lon(number: u8) -> Result<f64, CoreError>, PRASHNA_NUMBER_MAX}` (kökten de dışa aktarılır).
  - `ChartInput.lagna_number: Option<u8>`; `compute_positions`, `compute_jaimini`, `compute_chara_dasha`, `compute_shadbala` (ve onlar üzerinden yogalar, bakışlar, uyum) sayıyı izler.
  - `CoreError::InvalidPrashnaInput(&'static str)`; `CommandError` kodu `invalid_prashna_input { field }`; `errors.invalid_prashna_input` (üç dil).

- [ ] **Step 1: Tabanları ölç**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
git log -1 --format=%h
git status --short
rustc --version
cargo test --workspace 2>&1 | grep -E "^test result|Running"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
```
Expected: `git status --short` boş; `rustc 1.100.0` ya da yenisi; `jyotish-core` lib `351 passed`, golden `8 passed`, `modules_golden` `15 passed`; `atlas` `moment_fixed_lmt` `11 passed`, `moment_iana` `13 passed`; `store` lib `103 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `49 passed`, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; Vitest `80 passed (80)`, `806 passed (806)`. Commit `0b61716` değilse ölçülen sayılar ilerleme notlarına taban olarak yazılır ve zincir oradan yürütülür (farklar aynıdır). (Vitest'te `NotesPanel.test.tsx › New note adds an empty note…` yük altında seyrek olarak düşebilir; yeniden koşunca geçer ve bu planın dosyalarına dokunmaz.)

- [ ] **Step 2: Testler: lagnası verilen harita, girdideki sayı, iki değişmez ve yeni hata kodu**

`chart.rs` altı test kazanır (verilen lagna yalnızca lagnayı ve evleri değiştirir; gerçek lagna aynı haritayı verir; tropikal-rasi kipinde lagna siderealdir; ev sistemi ayarı uygulanmaz ve ephemeris'ten ev istenmez; denetimler; `compute_chart_for` sayıyı izler). `positions.rs` bir test kazanır. `prashna_properties` spec §11'in iki değişmezidir. Var olan testlerdeki `ChartInput` değişmezleri `lagna_number: None` satırını kazanır (çekirdekte 7, `store`'da 2, uygulama testlerinde 14 yer); `store`'un eş harita testi alan listesini altıya çıkarır. Arayüzde girdinin tam nesnesini bekleyen yedi beklenti `lagna_number: null` kazanır. Uygulamanın hata testi ve arayüzün hata kodu listesi yeni kodu bekler.

````diff
diff --git a/app/src-tauri/src/commands.rs b/app/src-tauri/src/commands.rs
index 91ca7bd..ada445a 100644
--- a/app/src-tauri/src/commands.rs
+++ b/app/src-tauri/src/commands.rs
@@ -248,6 +248,11 @@ mod tests {
         // Yalnızca alan adı: yer ve tarih hata parametresine girmez.
         assert_eq!(e.params.len(), 1);
         assert_eq!(e.params["field"], "place");
+        let e = CommandError::from(CoreError::InvalidPrashnaInput("number"));
+        assert_eq!(e.code, "invalid_prashna_input");
+        // Yalnızca alan adı: sayı, soru, yer ve an hata parametresine girmez.
+        assert_eq!(e.params.len(), 1);
+        assert_eq!(e.params["field"], "number");
     }
 
     #[test]
diff --git a/app/src-tauri/tests/modules.rs b/app/src-tauri/tests/modules.rs
index adf990b..f6c529e 100644
--- a/app/src-tauri/tests/modules.rs
+++ b/app/src-tauri/tests/modules.rs
@@ -61,6 +61,7 @@ fn golden_inputs() -> Vec<(String, ChartInput)> {
                 lon: num("lon"),
                 settings: serde_json::from_value(v["settings"].clone()).expect("settings"),
                 time_known: true,
+                lagna_number: None,
             };
             (v["id"].as_str().expect("id").to_owned(), input)
         })
@@ -132,6 +133,7 @@ fn drishti_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
         // Bakış tablosu lagnaya bağlı değildir: saat bilinmese de aynıdır.
         let unknown = ChartInput {
             time_known: false,
+            lagna_number: None,
             ..input.clone()
         };
         assert_eq!(
@@ -330,6 +332,7 @@ fn jaimini_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
             engine,
             ChartInput {
                 time_known: false,
+                lagna_number: None,
                 ..input.clone()
             },
         )
@@ -525,6 +528,7 @@ fn chara_dasha_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
         // Saat bilinmiyorsa dasha Rust'ta boş döner (K2): sıra lagnaya bağlıdır.
         let unknown = ChartInput {
             time_known: false,
+            lagna_number: None,
             ..input.clone()
         };
         assert_eq!(
@@ -816,6 +820,7 @@ fn shadbala_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
         // Doğum saati bilinmiyorsa rapor yoktur (K2).
         let unknown = ChartInput {
             time_known: false,
+            lagna_number: None,
             ..input.clone()
         };
         assert_eq!(
@@ -1024,6 +1029,7 @@ fn shadbala_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
     let bad_place = ChartInput {
         lat: 91.0,
         time_known: false,
+        lagna_number: None,
         ..delhi.clone()
     };
     assert_eq!(
@@ -1230,6 +1236,7 @@ fn yogas_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
             engine,
             ChartInput {
                 time_known: false,
+                lagna_number: None,
                 ..input.clone()
             },
         )
@@ -1269,6 +1276,7 @@ fn yogas_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
     let bad_place = ChartInput {
         lat: 91.0,
         time_known: false,
+        lagna_number: None,
         ..delhi.clone()
     };
     assert_eq!(
@@ -1503,6 +1511,7 @@ fn compat_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
     for (id, input) in inputs {
         let unknown = ChartInput {
             time_known: false,
+            lagna_number: None,
             ..input.clone()
         };
         let report = compute_compat_blocking(engine, delhi.clone(), unknown.clone())
@@ -1574,6 +1583,7 @@ fn compat_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
     let bad_place = ChartInput {
         lat: 91.0,
         time_known: false,
+        lagna_number: None,
         ..delhi.clone()
     };
     for (groom, bride) in [(&bad_place, &istanbul), (&istanbul, &bad_place)] {
@@ -1624,6 +1634,7 @@ fn compat_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
     );
     let unknown = |input: &ChartInput| ChartInput {
         time_known: false,
+        lagna_number: None,
         ..input.clone()
     };
     let (median, max) = median_of_15(|| {
diff --git a/app/src-tauri/tests/ui_fixture.rs b/app/src-tauri/tests/ui_fixture.rs
index 0917910..8ab8771 100644
--- a/app/src-tauri/tests/ui_fixture.rs
+++ b/app/src-tauri/tests/ui_fixture.rs
@@ -139,6 +139,7 @@ fn ui_fixture_is_up_to_date() {
         lon: opened.chart.geo_lon,
         settings: opened.chart.meta.settings.clone(),
         time_known: true,
+        lagna_number: None,
     };
     let drishti = compute_drishti_blocking(&engine, module_input.clone()).expect("drishti");
     let jaimini = compute_jaimini_blocking(&engine, module_input.clone()).expect("jaimini");
@@ -174,6 +175,7 @@ fn ui_fixture_is_up_to_date() {
         &engine,
         ChartInput {
             time_known: false,
+            lagna_number: None,
             ..module_input.clone()
         },
     )
@@ -221,6 +223,7 @@ fn ui_fixture_is_up_to_date() {
         module_input.clone(),
         ChartInput {
             time_known: false,
+            lagna_number: None,
             ..partner_input.clone()
         },
     )
diff --git a/app/ui/dasha/CharaDashaStrip.test.tsx b/app/ui/dasha/CharaDashaStrip.test.tsx
index e3e2f52..6819d41 100644
--- a/app/ui/dasha/CharaDashaStrip.test.tsx
+++ b/app/ui/dasha/CharaDashaStrip.test.tsx
@@ -58,7 +58,7 @@ test("chara, antar (default): the twelve sub-periods of the dasha at the transit
   renderStrip();
   const items = await buttons();
   const opened = fixture().opened;
-  const input = { jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true };
+  const input = { jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null };
   expect(api.computeCharaDasha).toHaveBeenCalledWith(input);
   // Güncel dönem Rust'tan gelir: transit anı (`snapshot.jd_ut`) ile sorulur.
   expect(api.charaDashaAt).toHaveBeenCalledTimes(1);
diff --git a/app/ui/dasha/CharaDashaTree.test.tsx b/app/ui/dasha/CharaDashaTree.test.tsx
index 2170867..bccea02 100644
--- a/app/ui/dasha/CharaDashaTree.test.tsx
+++ b/app/ui/dasha/CharaDashaTree.test.tsx
@@ -57,7 +57,7 @@ test("chara: direction and the twelve years in a header line; rows carry sign ab
   await tree();
   const opened = fixture().opened;
   expect(api.computeCharaDasha).toHaveBeenCalledTimes(1);
-  expect(api.computeCharaDasha).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true });
+  expect(api.computeCharaDasha).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null });
   expect(screen.getByRole("combobox", { name: "Sistem" })).toHaveDisplayValue("Chara dasha");
   // Delhi fikstürü: lagna İkizler, 9. burç Kova (ters sayan): geri.
   expect(screen.getByText("Yön: geri")).toBeInTheDocument();
diff --git a/app/ui/i18n/i18n.test.ts b/app/ui/i18n/i18n.test.ts
index b2925d0..8b32667 100644
--- a/app/ui/i18n/i18n.test.ts
+++ b/app/ui/i18n/i18n.test.ts
@@ -74,7 +74,7 @@ const BACKEND_CODES = [
   "store_sqlite", "store_migration", "store_json", "store_too_new", "store_unavailable", "not_found", "invalid_input",
   "unknown_time_zone", "nonexistent_local_time", "ambiguous_local_time", "offset_out_of_range", "longitude_out_of_range",
   "date_out_of_range", "last_chart", "unsupported_schema_version", "invalid_settings", "invalid_location", "invalid_julian_day",
-  "non_finite_result", "invalid_dasha_input", "panchang_search", "invalid_transit_input", "invalid_muhurta_input",
+  "non_finite_result", "invalid_dasha_input", "panchang_search", "invalid_transit_input", "invalid_muhurta_input", "invalid_prashna_input",
   "atlas_unavailable", "atlas_sqlite", "atlas_io", "atlas_schema",
   "file_io", "dialog_path", "import_too_large", "import_format", "import_version", "import_record",
   "prefs_too_new",
diff --git a/app/ui/modules/input.test.ts b/app/ui/modules/input.test.ts
index c538ed3..9327bbc 100644
--- a/app/ui/modules/input.test.ts
+++ b/app/ui/modules/input.test.ts
@@ -5,7 +5,7 @@ import { chartInput, inputKey } from "./input";
 test("module input is the computed chart's moment, place and settings", () => {
   const opened = fixture().opened;
   const input = chartInput(opened);
-  expect(input).toEqual({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true });
+  expect(input).toEqual({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null });
   // Hesapta kullanılan ayarlar: haritanın `meta.settings` kopyası, `advanced` bloğuyla.
   expect(input.settings.schema_version).toBe(2);
   expect(input.settings.advanced).toEqual({ karaka_scheme: "Eight", jaimini_co_lords: true });
diff --git a/app/ui/panels/DrishtiPanel.test.tsx b/app/ui/panels/DrishtiPanel.test.tsx
index d91040c..412caad 100644
--- a/app/ui/panels/DrishtiPanel.test.tsx
+++ b/app/ui/panels/DrishtiPanel.test.tsx
@@ -35,7 +35,7 @@ test("asks for the open chart's aspects and marks full sign aspects, seven rows
   await screen.findByRole("table", { name: "Graha bakışları: satırlar bakan, sütunlar bakılan graha" });
   const opened = fixture().opened;
   expect(api.computeDrishti).toHaveBeenCalledTimes(1);
-  expect(api.computeDrishti).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true });
+  expect(api.computeDrishti).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null });
   expect(screen.getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Bakan", "Gü", "Ay", "Ma", "Me", "Jü", "Ve", "Sa", "Ra", "Ke"]);
   expect(screen.getAllByRole("rowheader").map((h) => h.textContent)).toEqual(["Güneş", "Ay", "Mars", "Merkür", "Jüpiter", "Venüs", "Satürn"]);
   // Delhi fikstürü: Ay Terazi'den Koç'a (Jüpiter, Satürn); Jüpiter Koç'tan Terazi'ye (Ay) ve
@@ -182,7 +182,7 @@ test("rashi drishti: nine rows by nine columns from the Jaimini report, marked b
   renderPanel({ kind: "rashi" });
   await screen.findByRole("table", { name: "Rashi drishti: satırlar bakan, sütunlar bakılan graha" });
   expect(api.computeJaimini).toHaveBeenCalledTimes(1);
-  expect(api.computeJaimini).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: fixture().opened.chart.meta.settings, time_known: true });
+  expect(api.computeJaimini).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: fixture().opened.chart.meta.settings, time_known: true, lagna_number: null });
   expect(api.computeDrishti).not.toHaveBeenCalled();
   // Rashi drishtide düğümler de bakar: dokuz satır. Virupa değeri yoktur: "Değer" seçici çizilmez.
   expect(screen.getAllByRole("rowheader").map((h) => h.textContent)).toEqual(["Güneş", "Ay", "Mars", "Merkür", "Jüpiter", "Venüs", "Satürn", "Rahu", "Ketu"]);
diff --git a/app/ui/panels/JaiminiKarakasPanel.test.tsx b/app/ui/panels/JaiminiKarakasPanel.test.tsx
index a4bb7fd..7b1985a 100644
--- a/app/ui/panels/JaiminiKarakasPanel.test.tsx
+++ b/app/ui/panels/JaiminiKarakasPanel.test.tsx
@@ -33,7 +33,7 @@ test("asks for the open chart's report and lists the eight karakas in role order
   const table = await karakaTable();
   const opened = fixture().opened;
   expect(api.computeJaimini).toHaveBeenCalledTimes(1);
-  expect(api.computeJaimini).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true });
+  expect(api.computeJaimini).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null });
   expect(within(table).getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Rol", "Graha", "Burç", "Derece"]);
   const rows = within(table).getAllByRole("row").slice(1);
   // Delhi fikstürü, sekizli şema: Rahu'nun gerçek derecesi 11°11′, etkin derecesi 30° − 11°11′.
diff --git a/app/ui/panels/ShadbalaPanel.test.tsx b/app/ui/panels/ShadbalaPanel.test.tsx
index 0c02d4d..d59d696 100644
--- a/app/ui/panels/ShadbalaPanel.test.tsx
+++ b/app/ui/panels/ShadbalaPanel.test.tsx
@@ -43,7 +43,7 @@ test("summary: seven grahas with the six strengths, total, required value, ratio
   const table = await summary();
   const opened = fixture().opened;
   expect(api.computeShadbala).toHaveBeenCalledTimes(1);
-  expect(api.computeShadbala).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true });
+  expect(api.computeShadbala).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null });
   expect(cells(within(table).getAllByRole("row")[0])).toEqual(["Graha", "Sthana", "Dig", "Zaman gücü", "Cheshta", "Naisargika", "Drik", "Toplam", "Gerekli", "Oran", "Sıra"]);
   // Delhi fikstürü, virupa, bir ondalık; oran iki ondalık.
   expect(bodyRows(table)).toEqual([
diff --git a/crates/jyotish-core/src/chart.rs b/crates/jyotish-core/src/chart.rs
index 8628bce..00978d5 100644
--- a/crates/jyotish-core/src/chart.rs
+++ b/crates/jyotish-core/src/chart.rs
@@ -661,6 +661,158 @@ mod tests {
         );
     }
 
+    /// Sayı prashnasının haritası (spec §11): grahalar, dasha ve panchang değişmez; lagna
+    /// verilen sidereal boylamdır ve evler ondan sayılan burç evleridir.
+    #[test]
+    fn a_given_lagna_changes_only_the_lagna_and_the_houses() {
+        let mut eph = FakeEph::default();
+        let s = CalcSettings::default();
+        let base = chart(&mut eph, &s);
+        // 100° sidereal: Yengeç 10°; Pushya'nın 3. padası.
+        let c = compute_chart_with_lagna(&mut eph, JD, 41.0, 29.0, &s, 100.0).unwrap();
+        assert_eq!(c.lagna, Lagna::new(100.0, 100.0));
+        assert_eq!(c.lagna.sign, Sign::Cancer);
+        assert_eq!((c.lagna.nakshatra, c.lagna.pada), (8, 3));
+        assert_eq!(c.bhavas.system, HouseSystem::WholeSign);
+        assert_eq!(c.bhavas.cusps, whole_sign_cusps(Sign::Cancer));
+        for (a, b) in c.grahas.iter().zip(&base.grahas) {
+            // Ev dışında her alan aynıdır.
+            let mut same_house = a.clone();
+            same_house.house = b.house;
+            assert_eq!(&same_house, b, "{:?}", a.graha);
+            assert_eq!(a.house, whole_sign_house(a.sign, Sign::Cancer));
+        }
+        assert_eq!((c.jd_ut, c.geo_lat, c.geo_lon), (JD, 41.0, 29.0));
+        assert_eq!(c.ayanamsa_deg, base.ayanamsa_deg);
+        assert_eq!(c.dasha, base.dasha);
+        assert_eq!(c.panchang, base.panchang);
+        assert_eq!(c.meta, base.meta);
+        // Lagnaya bağlı türevler yeni lagnadan gelir.
+        assert_eq!(
+            c.ashtakavarga,
+            compute_ashtakavarga(std::array::from_fn(|i| c.grahas[i].sign), Sign::Cancer)
+        );
+        let points: Vec<(Graha, f64)> = c.grahas.iter().map(|g| (g.graha, g.lon)).collect();
+        assert_eq!(c.vargas, all_vargas(100.0, &points));
+        // 0..360 dışındaki boylam sarılır.
+        let wrapped = compute_chart_with_lagna(&mut eph, JD, 41.0, 29.0, &s, 460.0).unwrap();
+        assert_eq!(wrapped, c);
+    }
+
+    #[test]
+    fn the_real_lagna_gives_the_same_chart() {
+        let mut eph = FakeEph::default();
+        let s = CalcSettings::default();
+        let base = chart(&mut eph, &s);
+        let again = compute_chart_with_lagna(&mut eph, JD, 41.0, 29.0, &s, base.lagna.sidereal_lon)
+            .unwrap();
+        assert_eq!(again, base);
+    }
+
+    #[test]
+    fn a_given_lagna_is_sidereal_in_the_tropical_rasi_mode() {
+        // FakeEph: ayanamsa 24°. Sidereal 100° → rasi çerçevesinde 124° (Aslan 4°).
+        let mut eph = FakeEph::default();
+        let s = tropical_rasi();
+        let c = compute_chart_with_lagna(&mut eph, JD, 41.0, 29.0, &s, 100.0).unwrap();
+        assert_eq!(c.lagna.sidereal_lon, 100.0);
+        assert!((c.lagna.lon - 124.0).abs() < 1e-9);
+        assert_eq!(c.lagna.sign, Sign::Leo);
+        // Nakshatra sidereal boylamdandır.
+        assert_eq!((c.lagna.nakshatra, c.lagna.pada), (8, 3));
+        assert_eq!(c.bhavas.cusps, whole_sign_cusps(Sign::Leo));
+        // Gerçek lagna bu kipte de aynı haritayı verir (boylam yuvarlama payıyla).
+        let base = chart(&mut eph, &s);
+        let again = compute_chart_with_lagna(&mut eph, JD, 41.0, 29.0, &s, base.lagna.sidereal_lon)
+            .unwrap();
+        assert!((again.lagna.lon - base.lagna.lon).abs() < 1e-9);
+        assert_eq!(again.lagna.sign, base.lagna.sign);
+        assert_eq!(again.grahas, base.grahas);
+        assert_eq!(again.bhavas, base.bhavas);
+    }
+
+    #[test]
+    fn a_given_lagna_ignores_the_house_system_and_asks_for_no_houses() {
+        let s = CalcSettings {
+            house_system: HouseSystem::Placidus,
+            ..CalcSettings::default()
+        };
+        let mut eph = FakeEph {
+            porphyry_fallback: true,
+            ..FakeEph::default()
+        };
+        let c = compute_chart_with_lagna(&mut eph, JD, 41.0, 29.0, &s, 100.0).unwrap();
+        assert_eq!(eph.last_house_system, None);
+        assert_eq!(c.bhavas.system, HouseSystem::WholeSign);
+        assert_eq!(c.bhavas.cusps, whole_sign_cusps(Sign::Cancer));
+        assert!(!c
+            .meta
+            .warnings
+            .contains(&ChartWarning::PlacidusPolarFallback));
+        // Ayarın kopyası değişmez: harita hangi ayarla kurulduğunu söyler.
+        assert_eq!(c.meta.settings.house_system, HouseSystem::Placidus);
+        for g in &c.grahas {
+            assert_eq!(g.house, whole_sign_house(g.sign, Sign::Cancer));
+        }
+    }
+
+    #[test]
+    fn a_given_lagna_is_validated_like_the_chart_and_by_field_name() {
+        let mut eph = FakeEph::default();
+        let s = CalcSettings::default();
+        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
+            assert_eq!(
+                compute_chart_with_lagna(&mut eph, JD, 41.0, 29.0, &s, bad),
+                Err(CoreError::InvalidPrashnaInput("lagna"))
+            );
+        }
+        assert_eq!(
+            compute_chart_with_lagna(&mut eph, JD, 91.0, 0.0, &s, 100.0),
+            Err(CoreError::InvalidLocation {
+                lat: 91.0,
+                lon: 0.0
+            })
+        );
+        assert!(matches!(
+            compute_chart_with_lagna(&mut eph, f64::NAN, 0.0, 0.0, &s, 100.0),
+            Err(CoreError::InvalidJulianDay(_))
+        ));
+    }
+
+    #[test]
+    fn the_chart_of_an_input_follows_the_lagna_number() {
+        let mut eph = FakeEph::default();
+        let s = CalcSettings::default();
+        let mut input = ChartInput {
+            jd_ut: JD,
+            lat: 41.0,
+            lon: 29.0,
+            settings: s.clone(),
+            time_known: true,
+            lagna_number: None,
+        };
+        assert_eq!(
+            compute_chart_for(&mut eph, &input).unwrap(),
+            chart(&mut eph, &s)
+        );
+        // 30. sayı: (30 − 1) × 3°20′ + 1°40′ = 98°20′ → Yengeç 8°20′.
+        input.lagna_number = Some(30);
+        let c = compute_chart_for(&mut eph, &input).unwrap();
+        assert_eq!(c.lagna.sign, Sign::Cancer);
+        assert!((c.lagna.deg_in_sign - (8.0 + 20.0 / 60.0)).abs() < 1e-9);
+        assert_eq!(
+            c,
+            compute_chart_with_lagna(&mut eph, JD, 41.0, 29.0, &s, 5900.0 / 60.0).unwrap()
+        );
+        for bad in [0, 109] {
+            input.lagna_number = Some(bad);
+            assert_eq!(
+                compute_chart_for(&mut eph, &input),
+                Err(CoreError::InvalidPrashnaInput("number"))
+            );
+        }
+    }
+
     #[test]
     fn missing_sunrise_becomes_a_warning() {
         let mut eph = FakeEph {
diff --git a/crates/jyotish-core/src/drishti.rs b/crates/jyotish-core/src/drishti.rs
index 6d3077f..c04d39e 100644
--- a/crates/jyotish-core/src/drishti.rs
+++ b/crates/jyotish-core/src/drishti.rs
@@ -180,6 +180,7 @@ mod tests {
             lon: 29.0,
             settings: CalcSettings::default(),
             time_known: true,
+            lagna_number: None,
         };
         let table = compute_drishti(&mut eph, &input).unwrap();
         let positions = compute_positions(&mut eph, &input).unwrap();
diff --git a/crates/jyotish-core/src/jaimini/chara_dasha.rs b/crates/jyotish-core/src/jaimini/chara_dasha.rs
index 0fe620b..bc70e76 100644
--- a/crates/jyotish-core/src/jaimini/chara_dasha.rs
+++ b/crates/jyotish-core/src/jaimini/chara_dasha.rs
@@ -312,6 +312,7 @@ pub fn compute_chara_dasha_at<E: Ephemeris + ?Sized>(
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::chart::compute_chart;
     use crate::fake::FakeEph;
     use crate::settings::{AdvancedSettings, CalcSettings, KarakaScheme};
     use Sign::*;
@@ -697,6 +698,7 @@ mod tests {
             lon: 29.0,
             settings,
             time_known,
+            lagna_number: None,
         }
     }
 
diff --git a/crates/jyotish-core/src/jaimini/mod.rs b/crates/jyotish-core/src/jaimini/mod.rs
index b34a807..de84d96 100644
--- a/crates/jyotish-core/src/jaimini/mod.rs
+++ b/crates/jyotish-core/src/jaimini/mod.rs
@@ -117,6 +117,7 @@ pub fn compute_jaimini<E: Ephemeris + ?Sized>(
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::chart::compute_chart;
     use crate::fake::FakeEph;
     use crate::graha::Graha;
     use crate::settings::CalcSettings;
@@ -133,6 +134,7 @@ mod tests {
                 ..CalcSettings::default()
             },
             time_known,
+            lagna_number: None,
         }
     }
 
diff --git a/crates/jyotish-core/src/kuta/mod.rs b/crates/jyotish-core/src/kuta/mod.rs
index 7a3e533..700f693 100644
--- a/crates/jyotish-core/src/kuta/mod.rs
+++ b/crates/jyotish-core/src/kuta/mod.rs
@@ -502,6 +502,7 @@ mod tests {
             lon: 29.0,
             settings,
             time_known,
+            lagna_number: None,
         }
     }
 
diff --git a/crates/jyotish-core/src/positions.rs b/crates/jyotish-core/src/positions.rs
index 703d723..1628d0e 100644
--- a/crates/jyotish-core/src/positions.rs
+++ b/crates/jyotish-core/src/positions.rs
@@ -156,6 +156,7 @@ pub fn compute_positions<E: Ephemeris + ?Sized>(
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::chart::compute_chart;
     use crate::fake::FakeEph;
     use crate::settings::ZodiacMode;
 
@@ -168,6 +169,7 @@ mod tests {
             lon: 29.0,
             settings,
             time_known,
+            lagna_number: None,
         }
     }
 
@@ -239,6 +241,31 @@ mod tests {
         assert!((p.lagna_lon.unwrap() - 46.0).abs() < 1e-9);
     }
 
+    #[test]
+    fn a_lagna_number_moves_the_lagna_and_keeps_the_grahas() {
+        let mut eph = FakeEph::default();
+        let s = CalcSettings::default();
+        let plain = compute_positions(&mut eph, &input(s.clone(), true)).unwrap();
+        // 30. sayı: 98°20′ sidereal, Yengeç.
+        let mut numbered = input(s, true);
+        numbered.lagna_number = Some(30);
+        let p = compute_positions(&mut eph, &numbered).unwrap();
+        assert!((p.lagna_lon.unwrap() - 5900.0 / 60.0).abs() < 1e-12);
+        assert_eq!(p.lagna_sign, Some(Sign::Cancer));
+        assert_eq!(p.grahas, plain.grahas);
+        // FakeEph'te Ay sidereal 71° (İkizler): Yengeç lagnasından 12. ev.
+        assert_eq!(p.house_of(Graha::Moon), Some(12));
+        // `time_known` kuralı değişmez.
+        numbered.time_known = false;
+        let hidden = compute_positions(&mut eph, &numbered).unwrap();
+        assert_eq!((hidden.lagna_lon, hidden.lagna_sign), (None, None));
+        numbered.lagna_number = Some(0);
+        assert_eq!(
+            compute_positions(&mut eph, &numbered),
+            Err(CoreError::InvalidPrashnaInput("number"))
+        );
+    }
+
     #[test]
     fn collector_reports_the_chart_errors() {
         let mut eph = FakeEph::default();
diff --git a/crates/jyotish-core/src/proptests.rs b/crates/jyotish-core/src/proptests.rs
index f301c98..e8262dc 100644
--- a/crates/jyotish-core/src/proptests.rs
+++ b/crates/jyotish-core/src/proptests.rs
@@ -1909,3 +1909,151 @@ mod muhurta_properties {
         }
     }
 }
+
+mod prashna_properties {
+    use ephemeris::Body;
+    use proptest::prelude::*;
+
+    use crate::chart::{compute_chart, compute_chart_with_lagna};
+    use crate::fake::FakeEph;
+    use crate::houses::{whole_sign_cusps, whole_sign_house};
+    use crate::positions::{compute_positions, ChartInput};
+    use crate::prashna::{number_lagna_lon, PRASHNA_NUMBER_MAX};
+    use crate::settings::{CalcSettings, HouseSystem, ZodiacMode};
+    use crate::zodiac::Sign;
+
+    const JD: f64 = 2_451_545.0;
+    const PLANETS: [Body; 7] = [
+        Body::Sun,
+        Body::Moon,
+        Body::Mars,
+        Body::Mercury,
+        Body::Jupiter,
+        Body::Venus,
+        Body::Saturn,
+    ];
+    const SPEEDS: [f64; 7] = [0.98, 13.2, 0.6, 1.2, -0.1, 1.1, 0.05];
+
+    fn sky(lons: [f64; 7], asc: f64, node: f64, ayanamsa: f64) -> FakeEph {
+        let mut eph = FakeEph {
+            asc,
+            ayanamsa,
+            ..FakeEph::default()
+        };
+        for ((body, lon), speed) in PLANETS.iter().zip(lons).zip(SPEEDS) {
+            eph = eph.with(*body, lon, speed);
+        }
+        eph.with(Body::MeanNode, node, -0.053)
+    }
+
+    fn settings(tropical: bool, house_system: HouseSystem) -> CalcSettings {
+        CalcSettings {
+            zodiac: if tropical {
+                ZodiacMode::TropicalRasiSiderealNakshatra
+            } else {
+                ZodiacMode::Sidereal
+            },
+            house_system,
+            ..CalcSettings::default()
+        }
+    }
+
+    fn house_systems() -> impl Strategy<Value = HouseSystem> {
+        prop_oneof![
+            Just(HouseSystem::WholeSign),
+            Just(HouseSystem::Sripati),
+            Just(HouseSystem::Placidus),
+        ]
+    }
+
+    proptest! {
+        /// Spec §11: sayı prashnasında grahaların boylamı aynı anın zaman prashnasıyla aynıdır;
+        /// yalnızca lagna ve evler değişir.
+        #[test]
+        fn a_number_changes_only_the_lagna_and_the_houses(
+            lons in proptest::array::uniform7(0.0f64..360.0),
+            asc in 0.0f64..360.0,
+            node in 0.0f64..360.0,
+            ayanamsa in 20.0f64..26.0,
+            number in 1u8..=PRASHNA_NUMBER_MAX,
+            tropical in any::<bool>(),
+            house_system in house_systems(),
+        ) {
+            let mut eph = sky(lons, asc, node, ayanamsa);
+            let s = settings(tropical, house_system);
+            let time = compute_chart(&mut eph, JD, 10.0, 20.0, &s).unwrap();
+            let lagna = number_lagna_lon(number).unwrap();
+            let c = compute_chart_with_lagna(&mut eph, JD, 10.0, 20.0, &s, lagna).unwrap();
+            prop_assert_eq!(c.lagna.sidereal_lon, lagna);
+            prop_assert_eq!(
+                (u16::from(c.lagna.nakshatra) - 1) * 4 + u16::from(c.lagna.pada),
+                u16::from(number)
+            );
+            prop_assert_eq!(c.lagna.sign, Sign::of(c.lagna.lon));
+            if !tropical {
+                prop_assert_eq!(c.lagna.sign.index(), usize::from(number - 1) / 9);
+            }
+            prop_assert_eq!(c.bhavas.system, HouseSystem::WholeSign);
+            prop_assert_eq!(&c.bhavas.cusps, &whole_sign_cusps(c.lagna.sign));
+            for (a, b) in c.grahas.iter().zip(&time.grahas) {
+                prop_assert_eq!(a.graha, b.graha);
+                prop_assert_eq!(a.lon, b.lon);
+                prop_assert_eq!(a.sidereal_lon, b.sidereal_lon);
+                prop_assert_eq!(a.speed, b.speed);
+                prop_assert_eq!((a.sign, a.nakshatra, a.pada), (b.sign, b.nakshatra, b.pada));
+                prop_assert_eq!((a.retrograde, a.combust, a.dignity), (b.retrograde, b.combust, b.dignity));
+                prop_assert_eq!(a.house, whole_sign_house(a.sign, c.lagna.sign));
+            }
+            prop_assert_eq!(&c.dasha, &time.dasha);
+            prop_assert_eq!(&c.panchang, &time.panchang);
+            prop_assert_eq!(c.ayanamsa_deg, time.ayanamsa_deg);
+            // Modül girdisi aynı haritayı görür.
+            let input = ChartInput {
+                jd_ut: JD,
+                lat: 10.0,
+                lon: 20.0,
+                settings: s,
+                time_known: true,
+                lagna_number: Some(number),
+            };
+            let p = compute_positions(&mut eph, &input).unwrap();
+            prop_assert_eq!(p.lagna_lon, Some(lagna));
+            prop_assert_eq!(p.lagna_sign, Some(c.lagna.sign));
+        }
+
+        /// Spec §11: `compute_chart_with_lagna` gerçek lagna verildiğinde `compute_chart` ile
+        /// aynı haritayı döner. Burç evi sisteminde ve sidereal kipte eşitlik bit bittir;
+        /// tropikal-rasi kipinde lagnanın rasi boylamı `sidereal + ayanamsa` ile yeniden
+        /// kurulur (yuvarlama payı) ve burç sınırına bu paydan yakın lagnalar atlanır.
+        #[test]
+        fn the_real_lagna_gives_the_chart_of_compute_chart(
+            lons in proptest::array::uniform7(0.0f64..360.0),
+            asc in 0.0f64..360.0,
+            node in 0.0f64..360.0,
+            ayanamsa in 20.0f64..26.0,
+            tropical in any::<bool>(),
+        ) {
+            let mut eph = sky(lons, asc, node, ayanamsa);
+            let s = settings(tropical, HouseSystem::WholeSign);
+            let base = compute_chart(&mut eph, JD, 10.0, 20.0, &s).unwrap();
+            let again =
+                compute_chart_with_lagna(&mut eph, JD, 10.0, 20.0, &s, base.lagna.sidereal_lon)
+                    .unwrap();
+            if tropical {
+                let d = base.lagna.deg_in_sign;
+                prop_assume!(d > 1e-6 && d < 30.0 - 1e-6);
+                prop_assert!((again.lagna.lon - base.lagna.lon).abs() < 1e-9);
+                prop_assert_eq!(again.lagna.sidereal_lon, base.lagna.sidereal_lon);
+                prop_assert_eq!(again.lagna.sign, base.lagna.sign);
+                prop_assert_eq!(&again.grahas, &base.grahas);
+                prop_assert_eq!(&again.bhavas, &base.bhavas);
+                prop_assert_eq!(&again.ashtakavarga, &base.ashtakavarga);
+                prop_assert_eq!(&again.dasha, &base.dasha);
+                prop_assert_eq!(&again.panchang, &base.panchang);
+                prop_assert_eq!(&again.meta, &base.meta);
+            } else {
+                prop_assert_eq!(&again, &base);
+            }
+        }
+    }
+}
diff --git a/crates/jyotish-core/src/shadbala/collect.rs b/crates/jyotish-core/src/shadbala/collect.rs
index 3379d1f..31c25fd 100644
--- a/crates/jyotish-core/src/shadbala/collect.rs
+++ b/crates/jyotish-core/src/shadbala/collect.rs
@@ -134,6 +134,7 @@ pub fn compute_shadbala<E: Ephemeris + ?Sized>(
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::chart::compute_chart;
     use crate::fake::FakeEph;
     use crate::panchang::Vara;
     use crate::settings::{CalcSettings, ZodiacMode};
@@ -149,11 +150,12 @@ mod tests {
             lon: 29.0,
             settings,
             time_known,
+            lagna_number: None,
         }
     }
 
     fn astro_of(eph: &mut FakeEph, input: &ChartInput) -> Option<ShadbalaAstro> {
-        let chart = compute_chart(eph, input.jd_ut, input.lat, input.lon, &input.settings).unwrap();
+        let chart = compute_chart_for(eph, input).unwrap();
         compute_shadbala_astro(eph, input, &chart).unwrap()
     }
 
diff --git a/crates/jyotish-core/src/yogas/mod.rs b/crates/jyotish-core/src/yogas/mod.rs
index 245f382..2a9a617 100644
--- a/crates/jyotish-core/src/yogas/mod.rs
+++ b/crates/jyotish-core/src/yogas/mod.rs
@@ -672,6 +672,7 @@ mod tests {
             lon: 29.0,
             settings,
             time_known,
+            lagna_number: None,
         };
         let mut eph = FakeEph::default();
         let s = CalcSettings::default();
diff --git a/crates/store/src/compat.rs b/crates/store/src/compat.rs
index c3f159d..cde7ff8 100644
--- a/crates/store/src/compat.rs
+++ b/crates/store/src/compat.rs
@@ -334,9 +334,10 @@ mod tests {
                 lon: a.birth.place.lon,
                 settings: raman,
                 time_known: true,
+                lagna_number: None,
             }
         );
-        // Dönen değer ad, not, yer adı, tarih metni ve saat dilimi taşımaz: yalnızca beş alan.
+        // Dönen değer ad, not, yer adı, tarih metni ve saat dilimi taşımaz: yalnızca altı alan.
         let json = serde_json::to_value(&input).unwrap();
         let mut keys: Vec<&str> = json
             .as_object()
@@ -345,7 +346,17 @@ mod tests {
             .map(String::as_str)
             .collect();
         keys.sort_unstable();
-        assert_eq!(keys, ["jd_ut", "lat", "lon", "settings", "time_known"]);
+        assert_eq!(
+            keys,
+            [
+                "jd_ut",
+                "lagna_number",
+                "lat",
+                "lon",
+                "settings",
+                "time_known"
+            ]
+        );
         let text = json.to_string();
         for private in ["Ayşe", "gizli", "New Delhi", "Asia/Kolkata", "2000-01-01"] {
             assert!(!text.contains(private), "{private}");
````

- [ ] **Step 3: Testlerin başarısız olduğunu doğrula**

```bash
cargo test -p jyotish-core --lib 2>&1 | grep -E "^error" | sort | uniq -c | sort -rn | head -6
pnpm --dir app exec vitest run ui/i18n ui/modules/input.test.ts 2>&1 | grep -E "Test Files|Tests "
```
Expected: derleme hataları: `cannot find function `compute_chart_with_lagna` in this scope`, `struct `positions::ChartInput` has no field named `lagna_number``, `cannot find function `compute_chart_for` in this scope`, `no variant … named `InvalidPrashnaInput``, `unresolved import `crate::prashna``. Vitest: seçilen dosyalarda iki test düşer (`errors.invalid_prashna_input` anahtarı yok; girdi `lagna_number` taşımıyor).

- [ ] **Step 4: Uygulama**

`chart.rs`: `compute_chart` gövdesi `chart_with`'e taşınır; verilen lagna yolunda ephemeris'ten ev istenmez. `prashna/mod.rs` sayının lagnasını verir (testleri dosyanın sonundadır; önce onları okuyun). Toplayıcılar haritayı `compute_chart_for`'dan alır; test modülleri `compute_chart`'ı kendileri içe aktarır.

````diff
diff --git a/app/src-tauri/src/commands.rs b/app/src-tauri/src/commands.rs
index ada445a..4f57808 100644
--- a/app/src-tauri/src/commands.rs
+++ b/app/src-tauri/src/commands.rs
@@ -88,6 +88,9 @@ impl From<CoreError> for CommandError {
             CoreError::InvalidMuhurtaInput(field) => {
                 Self::new("invalid_muhurta_input", &[("field", field.to_owned())])
             }
+            CoreError::InvalidPrashnaInput(field) => {
+                Self::new("invalid_prashna_input", &[("field", field.to_owned())])
+            }
         }
     }
 }
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index d52464f..a60943d 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -575,6 +575,7 @@ export const en: Messages = {
     panchang_search: "Panchang boundary not found ({{what}}).",
     invalid_transit_input: "Invalid transit request: {{field}}.",
     invalid_muhurta_input: "Invalid muhurta request: {{field}}.",
+    invalid_prashna_input: "Invalid prashna request: {{field}}.",
     atlas_unavailable: "Place search is unavailable ({{reason}}). You can enter latitude, longitude and time zone by hand.",
     atlas_sqlite: "The place atlas could not be read ({{detail}}).",
     atlas_io: "The place atlas file could not be read ({{detail}}).",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index de68f73..4c60ef7 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -575,6 +575,7 @@ export const fr: Messages = {
     panchang_search: "Limite du panchang introuvable ({{what}}).",
     invalid_transit_input: "Requête de transit invalide : {{field}}.",
     invalid_muhurta_input: "Requête de muhurta invalide : {{field}}.",
+    invalid_prashna_input: "Requête de prashna invalide : {{field}}.",
     atlas_unavailable: "La recherche de lieux est indisponible ({{reason}}). Vous pouvez saisir la latitude, la longitude et le fuseau à la main.",
     atlas_sqlite: "L’atlas des lieux est illisible ({{detail}}).",
     atlas_io: "Le fichier de l’atlas est illisible ({{detail}}).",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index f6ccd38..a87aae8 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -574,6 +574,7 @@ export const tr = {
     panchang_search: "Panchang sınırı bulunamadı ({{what}}).",
     invalid_transit_input: "Transit isteği geçersiz: {{field}}.",
     invalid_muhurta_input: "Muhurta isteği geçersiz: {{field}}.",
+    invalid_prashna_input: "Prashna isteği geçersiz: {{field}}.",
     atlas_unavailable: "Yer araması kullanılamıyor ({{reason}}). Enlem, boylam ve saat dilimini elle girebilirsiniz.",
     atlas_sqlite: "Yer atlası okunamadı ({{detail}}).",
     atlas_io: "Yer atlası dosyası okunamadı ({{detail}}).",
diff --git a/app/ui/modules/input.ts b/app/ui/modules/input.ts
index 5aa901d..355b230 100644
--- a/app/ui/modules/input.ts
+++ b/app/ui/modules/input.ts
@@ -6,7 +6,7 @@ import { timeUnknown } from "../session/birth";
 /** Açık haritanın modül girdisi: hesaplanan haritanın anı, konumu ve ayarları. */
 export function chartInput(opened: OpenedClient): ChartInput {
   const { chart } = opened;
-  return { jd_ut: chart.jd_ut, lat: chart.geo_lat, lon: chart.geo_lon, settings: chart.meta.settings, time_known: !timeUnknown(opened) };
+  return { jd_ut: chart.jd_ut, lat: chart.geo_lat, lon: chart.geo_lon, settings: chart.meta.settings, time_known: !timeUnknown(opened), lagna_number: null };
 }
 
 /**
diff --git a/crates/jyotish-core/src/chart.rs b/crates/jyotish-core/src/chart.rs
index 00978d5..4f10ff9 100644
--- a/crates/jyotish-core/src/chart.rs
+++ b/crates/jyotish-core/src/chart.rs
@@ -11,6 +11,8 @@ use crate::dignity::{dignity, sign_lord};
 use crate::graha::{Graha, GrahaPosition};
 use crate::houses::{house_from_cusps, whole_sign_cusps, whole_sign_house, Bhavas, Lagna};
 use crate::panchang::{compute_panchang, Panchang};
+use crate::positions::ChartInput;
+use crate::prashna::number_lagna_lon;
 use crate::settings::{CalcSettings, HouseSystem, NodeKind, ZodiacMode, JD_UT_RANGE};
 use crate::varga::{all_vargas, is_vargottama, VargaChart};
 use crate::zodiac::{deg_in_sign, nakshatra_pada, normalize, Sign};
@@ -188,6 +190,75 @@ pub fn compute_chart<E: Ephemeris + ?Sized>(
     geo_lat: f64,
     geo_lon: f64,
     settings: &CalcSettings,
+) -> Result<Chart, CoreError> {
+    chart_with(eph, jd_ut, geo_lat, geo_lon, settings, None)
+}
+
+/// Lagnası verilen harita (ileri modüller spec'i §11, sayı prashnası): grahalar, dasha ve
+/// panchang `compute_chart` ile aynıdır; lagna `lagna_sidereal_lon`'dur (sidereal boylam) ve
+/// ev sistemi ayarı ne olursa olsun evler bu lagnadan sayılan burç evleridir
+/// (`bhavas.system` `WholeSign` döner). Ephemeris'ten ev istenmez; `PlacidusPolarFallback`
+/// uyarısı bu yolda üretilmez. Sonlu olmayan lagna `InvalidPrashnaInput("lagna")`'dır.
+pub fn compute_chart_with_lagna<E: Ephemeris + ?Sized>(
+    eph: &mut E,
+    jd_ut: f64,
+    geo_lat: f64,
+    geo_lon: f64,
+    settings: &CalcSettings,
+    lagna_sidereal_lon: f64,
+) -> Result<Chart, CoreError> {
+    if !lagna_sidereal_lon.is_finite() {
+        return Err(CoreError::InvalidPrashnaInput("lagna"));
+    }
+    chart_with(
+        eph,
+        jd_ut,
+        geo_lat,
+        geo_lon,
+        settings,
+        Some(normalize(lagna_sidereal_lon)),
+    )
+}
+
+/// `ChartInput`'un haritası: `lagna_number` doluysa sayı prashnasının lagnasıyla
+/// (`number_lagna_lon`), değilse `compute_chart`. Modül toplayıcıları haritayı buradan alır;
+/// böylece sayı prashnasında her modül, haritanın gösterdiği lagnayı kullanır.
+pub fn compute_chart_for<E: Ephemeris + ?Sized>(
+    eph: &mut E,
+    input: &ChartInput,
+) -> Result<Chart, CoreError> {
+    match input.lagna_number {
+        None => compute_chart(eph, input.jd_ut, input.lat, input.lon, &input.settings),
+        Some(number) => {
+            // Sayı, ephemeris sorulmadan önce denetlenir.
+            let lagna = number_lagna_lon(number)?;
+            compute_chart_with_lagna(
+                eph,
+                input.jd_ut,
+                input.lat,
+                input.lon,
+                &input.settings,
+                lagna,
+            )
+        }
+    }
+}
+
+/// Sidereal boylamın rasi çerçevesindeki karşılığı (`to_sidereal`'in tersi).
+pub(crate) fn from_sidereal(zodiac: ZodiacMode, sidereal_lon: f64, ayanamsa_deg: f64) -> f64 {
+    match zodiac {
+        ZodiacMode::Sidereal => sidereal_lon,
+        ZodiacMode::TropicalRasiSiderealNakshatra => normalize(sidereal_lon + ayanamsa_deg),
+    }
+}
+
+fn chart_with<E: Ephemeris + ?Sized>(
+    eph: &mut E,
+    jd_ut: f64,
+    geo_lat: f64,
+    geo_lon: f64,
+    settings: &CalcSettings,
+    lagna_sidereal_lon: Option<f64>,
 ) -> Result<Chart, CoreError> {
     settings.validate()?;
     check_moment(jd_ut, geo_lat, geo_lon)?;
@@ -203,24 +274,36 @@ pub fn compute_chart<E: Ephemeris + ?Sized>(
         warnings.push(ChartWarning::LowPrecision);
     }
 
-    let houses = eph.houses(
-        jd_ut,
-        geo_lat,
-        geo_lon,
-        settings.house_system.ephemeris_system(),
-        frame,
-    )?;
-    finite(houses.asc, "lagna")?;
-    for c in houses.cusps {
-        finite(c, "bhavas")?;
-    }
-    if houses.porphyry_fallback {
-        warnings.push(ChartWarning::PlacidusPolarFallback);
-    }
-    let lagna = Lagna::new(houses.asc, sidereal(houses.asc));
-    let cusps = match settings.house_system {
-        HouseSystem::WholeSign => whole_sign_cusps(lagna.sign),
-        HouseSystem::Sripati | HouseSystem::Placidus => houses.cusps.to_vec(),
+    let (lagna, cusps, house_system) = match lagna_sidereal_lon {
+        Some(sidereal_lon) => {
+            // Verilen lagna: evler bu lagnadan burç evidir; ephemeris'ten ev istenmez.
+            let lon = from_sidereal(settings.zodiac, sidereal_lon, ayanamsa.deg);
+            let lagna = Lagna::new(lon, sidereal_lon);
+            let cusps = whole_sign_cusps(lagna.sign);
+            (lagna, cusps, HouseSystem::WholeSign)
+        }
+        None => {
+            let houses = eph.houses(
+                jd_ut,
+                geo_lat,
+                geo_lon,
+                settings.house_system.ephemeris_system(),
+                frame,
+            )?;
+            finite(houses.asc, "lagna")?;
+            for c in houses.cusps {
+                finite(c, "bhavas")?;
+            }
+            if houses.porphyry_fallback {
+                warnings.push(ChartWarning::PlacidusPolarFallback);
+            }
+            let lagna = Lagna::new(houses.asc, sidereal(houses.asc));
+            let cusps = match settings.house_system {
+                HouseSystem::WholeSign => whole_sign_cusps(lagna.sign),
+                HouseSystem::Sripati | HouseSystem::Placidus => houses.cusps.to_vec(),
+            };
+            (lagna, cusps, settings.house_system)
+        }
     };
 
     let sun_lon = raw[0].lon;
@@ -254,7 +337,7 @@ pub fn compute_chart<E: Ephemeris + ?Sized>(
                     &settings.combustion_orbs,
                 ),
                 dignity: dignity(r.graha, sign, deg, lord_sign),
-                house: match settings.house_system {
+                house: match house_system {
                     HouseSystem::WholeSign => whole_sign_house(sign, lagna.sign),
                     HouseSystem::Sripati | HouseSystem::Placidus => house_from_cusps(r.lon, &cusps),
                 },
@@ -292,7 +375,7 @@ pub fn compute_chart<E: Ephemeris + ?Sized>(
         lagna,
         grahas,
         bhavas: Bhavas {
-            system: settings.house_system,
+            system: house_system,
             cusps,
         },
         vargas,
diff --git a/crates/jyotish-core/src/error.rs b/crates/jyotish-core/src/error.rs
index f874f81..d7500a8 100644
--- a/crates/jyotish-core/src/error.rs
+++ b/crates/jyotish-core/src/error.rs
@@ -27,4 +27,7 @@ pub enum CoreError {
     /// (değer taşımaz).
     #[error("invalid muhurta input: {0}")]
     InvalidMuhurtaInput(&'static str),
+    /// Prashna girdisi geçersiz (sayı, odak evi, lagna); alan adını taşır (değer taşımaz).
+    #[error("invalid prashna input: {0}")]
+    InvalidPrashnaInput(&'static str),
 }
diff --git a/crates/jyotish-core/src/jaimini/chara_dasha.rs b/crates/jyotish-core/src/jaimini/chara_dasha.rs
index bc70e76..19c3d7f 100644
--- a/crates/jyotish-core/src/jaimini/chara_dasha.rs
+++ b/crates/jyotish-core/src/jaimini/chara_dasha.rs
@@ -19,7 +19,7 @@
 use ephemeris::Ephemeris;
 use serde::{Deserialize, Serialize};
 
-use crate::chart::compute_chart;
+use crate::chart::compute_chart_for;
 use crate::positions::{ChartInput, Positions};
 use crate::zodiac::Sign;
 use crate::CoreError;
@@ -271,7 +271,7 @@ pub fn compute_chara_dasha<E: Ephemeris + ?Sized>(
     eph: &mut E,
     input: &ChartInput,
 ) -> Result<Option<CharaDasha>, CoreError> {
-    let chart = compute_chart(eph, input.jd_ut, input.lat, input.lon, &input.settings)?;
+    let chart = compute_chart_for(eph, input)?;
     if !input.time_known {
         return Ok(None);
     }
diff --git a/crates/jyotish-core/src/jaimini/mod.rs b/crates/jyotish-core/src/jaimini/mod.rs
index de84d96..004d881 100644
--- a/crates/jyotish-core/src/jaimini/mod.rs
+++ b/crates/jyotish-core/src/jaimini/mod.rs
@@ -14,7 +14,7 @@ mod rashi;
 use ephemeris::Ephemeris;
 use serde::{Deserialize, Serialize};
 
-use crate::chart::{compute_chart, ChartWarning};
+use crate::chart::{compute_chart_for, ChartWarning};
 use crate::drishti::SignAspect;
 use crate::positions::{ChartInput, Positions};
 use crate::settings::{AdvancedSettings, KarakaScheme};
@@ -96,7 +96,7 @@ pub fn compute_jaimini<E: Ephemeris + ?Sized>(
     eph: &mut E,
     input: &ChartInput,
 ) -> Result<JaiminiReport, CoreError> {
-    let chart = compute_chart(eph, input.jd_ut, input.lat, input.lon, &input.settings)?;
+    let chart = compute_chart_for(eph, input)?;
     let positions = Positions::from(&chart);
     let positions = if input.time_known {
         positions
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index e7498b6..72c0809 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -7,6 +7,7 @@
 //! çekirdeği (Alt proje 3.2) ve chara dasha (Alt proje 3.3): `jaimini`. Shadbala ve Bhava bala
 //! (Alt proje 3.4): `shadbala`. Yogalar (Alt proje 3.5): `yogas`. Uyum (Alt proje 3.6): `kuta`.
 //! Muhurta (Alt proje 3.7): `timeline` (aralık dizileri) ve `muhurta` (gün panchangı, arama).
+//! Prashna (Alt proje 3.8): `prashna` (sayı prashnasının lagnası, soru haritasının olguları).
 
 pub mod ashtakavarga;
 mod chart;
@@ -26,6 +27,7 @@ pub mod muhurta;
 pub mod nature;
 pub mod panchang;
 pub mod positions;
+pub mod prashna;
 #[cfg(test)]
 mod proptests;
 pub mod settings;
@@ -39,7 +41,10 @@ pub mod zodiac;
 pub use ashtakavarga::{
     compute_ashtakavarga, Ashtakavarga, ASHTAKAVARGA_GRAHAS, BAV_TOTALS, SAV_TOTAL,
 };
-pub use chart::{compute_chart, Chart, ChartMeta, ChartWarning, ENGINE_VERSION};
+pub use chart::{
+    compute_chart, compute_chart_for, compute_chart_with_lagna, Chart, ChartMeta, ChartWarning,
+    ENGINE_VERSION,
+};
 pub use combustion::is_combust;
 pub use dasha::{
     birth_dasha, dasha_children, dasha_path_at, mahadashas, vimshottari, vimshottari_years, Dasha,
@@ -87,6 +92,7 @@ pub use panchang::{
     END_BISECTIONS, END_SEARCH_DAYS, KARANA_SPAN_DEG, SUNRISE_WINDOW_DAYS, TITHI_SPAN_DEG,
 };
 pub use positions::{compute_positions, ChartInput, GrahaPoint, Positions};
+pub use prashna::{number_lagna_lon, PRASHNA_NUMBER_MAX};
 pub use settings::{
     AdvancedSettings, AyanamsaSystem, CalcSettings, CombustionOrbs, HouseSystem, KarakaScheme,
     NodeKind, SunriseDef, ZodiacMode, CALC_SETTINGS_SCHEMA_VERSION, JD_UT_RANGE,
diff --git a/crates/jyotish-core/src/positions.rs b/crates/jyotish-core/src/positions.rs
index 1628d0e..7de8d21 100644
--- a/crates/jyotish-core/src/positions.rs
+++ b/crates/jyotish-core/src/positions.rs
@@ -5,7 +5,7 @@
 use ephemeris::Ephemeris;
 use serde::{Deserialize, Serialize};
 
-use crate::chart::{compute_chart, Chart};
+use crate::chart::{compute_chart_for, Chart};
 use crate::graha::Graha;
 use crate::settings::CalcSettings;
 use crate::zodiac::{deg_in_sign, normalize, Sign};
@@ -24,6 +24,11 @@ pub struct ChartInput {
     pub lon: f64,
     pub settings: CalcSettings,
     pub time_known: bool,
+    /// Sayı prashnası (ileri modüller spec'i §11): doluysa lagna bu sayıdan gelir (1..=108,
+    /// `number_lagna_lon`) ve evler o lagnadan burç evidir; grahalar anın grahalarıdır. Diğer
+    /// her haritada `None`. `time_known` kuralı değişmez: yanlışsa lagnaya bağlı çıktı yine
+    /// boştur (soru anı her zaman bilinir; arayüz sayı prashnasında `true` gönderir).
+    pub lagna_number: Option<u8>,
 }
 
 /// Bir grahanın modüllerin kullandığı konumu.
@@ -139,12 +144,13 @@ impl Positions {
 }
 
 /// Toplayıcı: temel haritayı hesaplar ve konumlara indirger. `input.time_known` yanlışsa
-/// lagna boştur. Ayar, an ve konum denetimi `compute_chart`'ınkidir.
+/// lagna boştur; sayı prashnasında (`input.lagna_number`) lagna sayıdan gelir
+/// (`compute_chart_for`). Ayar, an ve konum denetimi `compute_chart`'ınkidir.
 pub fn compute_positions<E: Ephemeris + ?Sized>(
     eph: &mut E,
     input: &ChartInput,
 ) -> Result<Positions, CoreError> {
-    let chart = compute_chart(eph, input.jd_ut, input.lat, input.lon, &input.settings)?;
+    let chart = compute_chart_for(eph, input)?;
     let positions = Positions::from(&chart);
     Ok(if input.time_known {
         positions
diff --git a/crates/jyotish-core/src/shadbala/collect.rs b/crates/jyotish-core/src/shadbala/collect.rs
index 31c25fd..5739400 100644
--- a/crates/jyotish-core/src/shadbala/collect.rs
+++ b/crates/jyotish-core/src/shadbala/collect.rs
@@ -4,7 +4,7 @@
 
 use ephemeris::{Body, Ephemeris, HouseSystem, Zodiac};
 
-use crate::chart::{compute_chart, finite, Chart, ChartWarning};
+use crate::chart::{compute_chart_for, finite, Chart, ChartWarning};
 use crate::graha::Graha;
 use crate::panchang::{following_sunrise, vara_at};
 use crate::positions::{ChartInput, Positions};
@@ -111,7 +111,7 @@ pub fn compute_shadbala<E: Ephemeris + ?Sized>(
     eph: &mut E,
     input: &ChartInput,
 ) -> Result<Option<ShadbalaReport>, CoreError> {
-    let chart = compute_chart(eph, input.jd_ut, input.lat, input.lon, &input.settings)?;
+    let chart = compute_chart_for(eph, input)?;
     if !input.time_known {
         return Ok(None);
     }
diff --git a/crates/store/src/compat.rs b/crates/store/src/compat.rs
index cde7ff8..ea73dd3 100644
--- a/crates/store/src/compat.rs
+++ b/crates/store/src/compat.rs
@@ -111,6 +111,7 @@ impl Store {
             lon: chart.birth.place.lon,
             settings,
             time_known: chart.birth.time_certainty != TimeCertainty::Unknown,
+            lagna_number: None,
         }))
     }
 
````

**Yeni dosya:** `crates/jyotish-core/src/prashna/mod.rs`

````rust
//! Prashna (ileri modüller spec'i §11): soru anının haritası ve o haritanın olguları. Hesap
//! Parashari motorunun aynısıdır; ayrı bir prashna ekolünün kuralları, hüküm ve evet/hayır
//! sonucu yoktur.
//!
//! Bu dosya sayı prashnasının lagnasını verir: grahalar anın grahalarıdır, lagna 1–108
//! arasındaki sayıdan gelir (`number_lagna_lon`). Harita `compute_chart_with_lagna` ile kurulur.

use crate::CoreError;

/// Sayı prashnasının en büyük sayısı: 12 burç × 9 navamsa.
pub const PRASHNA_NUMBER_MAX: u8 = 108;

/// Bir navamsanın yay dakikası olarak genişliği (3°20′).
const NAVAMSA_ARCMIN: u16 = 200;

/// Sayı prashnasının lagnası: `n`. navamsanın ortası, sidereal boylam (derece):
/// `(n − 1) × 3°20′ + 1°40′`. Sayı 1..=108 değilse `InvalidPrashnaInput("number")`.
/// Yay dakikası üzerinden hesaplanır: aynı sayı her zaman aynı `f64` değerini verir.
pub fn number_lagna_lon(number: u8) -> Result<f64, CoreError> {
    if !(1..=PRASHNA_NUMBER_MAX).contains(&number) {
        return Err(CoreError::InvalidPrashnaInput("number"));
    }
    let arcmin = (u16::from(number) - 1) * NAVAMSA_ARCMIN + NAVAMSA_ARCMIN / 2;
    Ok(f64::from(arcmin) / 60.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zodiac::{deg_in_sign, nakshatra_pada, Sign};

    /// Derece ve dakika olarak elle yazılmış beklenen lagnalar (spec §11 "Test").
    const HAND: [(u8, Sign, u16, u16); 5] = [
        (1, Sign::Aries, 1, 40),
        (9, Sign::Aries, 28, 20),
        (10, Sign::Taurus, 1, 40),
        (54, Sign::Virgo, 28, 20),
        (108, Sign::Pisces, 28, 20),
    ];

    #[test]
    fn the_number_gives_the_middle_of_its_navamsa() {
        for (n, sign, deg, min) in HAND {
            let lon = number_lagna_lon(n).unwrap();
            assert_eq!(Sign::of(lon), sign, "{n}");
            let want = f64::from(deg) + f64::from(min) / 60.0;
            assert!((deg_in_sign(lon) - want).abs() < 1e-9, "{n}");
        }
        assert_eq!(number_lagna_lon(1).unwrap(), 100.0 / 60.0);
        assert_eq!(number_lagna_lon(108).unwrap(), 21_500.0 / 60.0);
    }

    #[test]
    fn every_number_is_a_distinct_navamsa_in_order() {
        // 108 navamsa 27 nakshatranın 4 padasıdır: n. sayı, n. padanın ortasıdır.
        for n in 1..=PRASHNA_NUMBER_MAX {
            let lon = number_lagna_lon(n).unwrap();
            assert!((0.0..360.0).contains(&lon));
            let (nakshatra, pada) = nakshatra_pada(lon);
            assert_eq!((nakshatra - 1) * 4 + pada, n);
            // Burç başına dokuz sayı.
            assert_eq!(Sign::of(lon).index(), usize::from(n - 1) / 9);
        }
    }

    #[test]
    fn numbers_outside_the_range_are_rejected_by_field_name() {
        for n in [0, 109, 200, u8::MAX] {
            assert_eq!(
                number_lagna_lon(n),
                Err(CoreError::InvalidPrashnaInput("number"))
            );
        }
    }
}
````

- [ ] **Step 5: Bağlamaları ve fikstürü yeniden üret; kapılar**

```bash
cargo fmt --all
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep -E "^test result"
git diff --stat -- app/ui/bindings.ts app/ui/test/fixtures/delhi.json
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p jyotish-core --lib 2>&1 | grep -E "^test result"
PROPTEST_CASES=4000 cargo test -p jyotish-core --lib prashna_properties 2>&1 | grep -E "^test result"
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
ls crates/jyotish-core/proptest-regressions 2>/dev/null
```
Expected: iki `ok`; `bindings.ts` 7 satır kazanır (`ChartInput.lagna_number` ve açıklaması), `delhi.json` tek satır değişir (uyum fikstürünün girdisi `lagna_number: null` taşır); fmt ve clippy çıktısız; `jyotish-core` lib `363 passed`; değişmez testleri `2 passed`; sayaç `0`; Vitest `80 passed (80)`, `806 passed (806)`; `✓ built`; `ls` çıktısız.

- [ ] **Step 6: Commit**

```bash
git add -A
git status --short
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): chart with a given lagna, the prashna number lagna and a lagna number on the module input; invalid_prashna_input" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: `git status --short` yalnızca bu görevin dosyalarını listeler (bir yeni dosya); `grep` çıktısız.

---

### Task 2: Çekirdek: prashna olguları; on dört el fikstürü

Spec §11 "Olgular" ve "Test" (el örnekleri); §13 katman 1–3; K3–K6, K9; D5–D9. Saf modül: ephemeris görmez. El örnekleri bu plan yazılırken motordan bağımsız bir ikinci türetmeyle üretilmiştir (Self-Review madde 5); dosyalar aşağıda tam içerikle verilir ve **motor çıktısına bakılarak değiştirilmez**.

**Files:**
- Create: `crates/jyotish-core/src/prashna/facts.rs`, `tests/golden/modules/prashna/REVIEW.md`, `tests/golden/modules/prashna/*.json` (14 dosya)
- Modify: `crates/jyotish-core/src/prashna/mod.rs`, `crates/jyotish-core/src/lib.rs`, `crates/jyotish-core/src/proptests.rs`, `crates/jyotish-core/tests/modules_golden.rs`, `tests/golden/modules/README.md`
- Test: `facts.rs`'in `mod tests`'i (10), `proptests.rs` `facts_are_consistent` (1), `modules_golden.rs` (2)

**Interfaces:**
- Consumes (v1, Plan 14, 15, 18; Task 1): `Positions`, `ChartInput`, `compute_chart_for`, `house_lord`, `house_sign`, `sign_lord`, `dignity`, `is_combust`, `graha_aspect_signs`, `ASPECTING_GRAHAS`, `nature`, `sign_kind`, `SignKind`, `is_exchange`, `is_mutual_aspect`, `yogas`, `YogaReport`, `KENDRA_HOUSES`, `panchang::{elongation, tithi_no, paksha, Paksha}`, `nakshatra_pada`, `CombustionOrbs`.
- Produces (`jyotish_core::prashna`; hepsi kökten de dışa aktarılır):
  - Tipler: `RisingMode { HeadFirst, BackFirst, BothWays }` (`ALL`), `LordFacts`, `LordLink` (`ALL`), `LordRelation`, `Occupant`, `HouseOccupants`, `LagnaFacts`, `FocusFacts`, `MoonFacts`, `PrashnaFacts`; `PRASHNA_METHOD`.
  - `rising_mode(sign) -> RisingMode`, `lord_relation(lagna_lord, focus_lord, &Positions) -> LordRelation`, `moon_facts(&Positions) -> MoonFacts`, `prashna_facts(&Positions, &CombustionOrbs, focus_house: Option<u8>) -> Result<PrashnaFacts, CoreError>`, `compute_prashna_facts(eph, &ChartInput, focus_house) -> Result<PrashnaFacts, CoreError>`.
  - El fikstürü şeması: `input.{lagna_lon, number, focus_house, grahas, retrograde}`; `expected.{lagna, focus, moon}`.

- [ ] **Step 1: Testler: fikstür düzeneği, on dört el örneği ve iç tutarlılık değişmezi**

`modules_golden.rs` prashna düzeneğini kazanır: girdi sentetik konumlardır (lagna boylamdan ya da sayıdan), beklenen değerler lagna, odak ve Ay bölümleridir; yoga listesi konumların `yogas` sonucuyla karşılaştırılır. Kapsam testi spec'in istediği durumları sabitler: beş sayı (1, 9, 10, 54, 108) elle yazılmış ikinci tabloyla (`HAND_NUMBER_LAGNA`), 0 ve 109'un hatası, on iki lagna burcu ve doğuş biçimleri elle yazılmış ikinci tabloyla (`HAND_RISING`), beş bağın her biri, "hiçbiri", aynı graha, iki bağ birden, odak evi seçilmemiş ve lagnasız harita, retro ve yanık lord, dört dignity, iki paksha, kendrada iki sınıf, kötücül Merkür ve bir düğüm. `facts_are_consistent` olguların iç tutarlılığını her girdide ister.

````diff
diff --git a/crates/jyotish-core/src/proptests.rs b/crates/jyotish-core/src/proptests.rs
index e8262dc..0a9a5a3 100644
--- a/crates/jyotish-core/src/proptests.rs
+++ b/crates/jyotish-core/src/proptests.rs
@@ -1916,10 +1916,15 @@ mod prashna_properties {
 
     use crate::chart::{compute_chart, compute_chart_with_lagna};
     use crate::fake::FakeEph;
+    use crate::graha::Graha;
     use crate::houses::{whole_sign_cusps, whole_sign_house};
-    use crate::positions::{compute_positions, ChartInput};
-    use crate::prashna::{number_lagna_lon, PRASHNA_NUMBER_MAX};
-    use crate::settings::{CalcSettings, HouseSystem, ZodiacMode};
+    use crate::lordship::{house_lord, house_sign};
+    use crate::nature::{moon_is_waxing, nature};
+    use crate::panchang::Paksha;
+    use crate::positions::{compute_positions, ChartInput, Positions};
+    use crate::prashna::{number_lagna_lon, prashna_facts, LordLink, PRASHNA_NUMBER_MAX};
+    use crate::settings::{CalcSettings, CombustionOrbs, HouseSystem, ZodiacMode};
+    use crate::yogas::yogas;
     use crate::zodiac::Sign;
 
     const JD: f64 = 2_451_545.0;
@@ -2021,6 +2026,76 @@ mod prashna_properties {
             prop_assert_eq!(p.lagna_sign, Some(c.lagna.sign));
         }
 
+        /// Olguların iç tutarlılığı: her graha lagnadan sayılan evindedir, kendralar yalnızca
+        /// 1, 4, 7 ve 10. evlerin grahalarını taşır, iki lordun ev sayıları birbirinin
+        /// tümleyenidir ve bağlar kendi tanımlarını sağlar.
+        #[test]
+        fn facts_are_consistent(
+            lons in proptest::array::uniform9(0.0f64..360.0),
+            lagna in 0.0f64..360.0,
+            focus in 1u8..=12,
+        ) {
+            let p = Positions::synthetic(Some(lagna), lons);
+            let f = prashna_facts(&p, &CombustionOrbs::default(), Some(focus)).unwrap();
+            let l = f.lagna.unwrap();
+            let lagna_sign = p.lagna_sign.unwrap();
+            prop_assert_eq!(l.sign, lagna_sign);
+            prop_assert_eq!(l.lord.graha, house_lord(lagna_sign, 1));
+            prop_assert_eq!(l.lord.sign, p.sign_of(l.lord.graha));
+            prop_assert_eq!(Some(l.lord.house), p.house_of(l.lord.graha));
+            let in_kendras: Vec<Graha> = l
+                .kendras
+                .iter()
+                .flat_map(|k| k.grahas.iter().map(|o| o.graha))
+                .collect();
+            for g in Graha::ALL {
+                let house = p.house_of(g).unwrap();
+                prop_assert_eq!(in_kendras.contains(&g), matches!(house, 1 | 4 | 7 | 10));
+            }
+            for k in &l.kendras {
+                prop_assert_eq!(k.sign, house_sign(lagna_sign, k.house));
+                for o in &k.grahas {
+                    prop_assert_eq!(p.sign_of(o.graha), k.sign);
+                    prop_assert_eq!(o.nature, nature(o.graha, &p));
+                }
+            }
+            let fo = f.focus.unwrap();
+            prop_assert_eq!(fo.sign, house_sign(lagna_sign, focus));
+            prop_assert_eq!(fo.lord.graha, house_lord(lagna_sign, focus));
+            let r = &fo.relation;
+            let (a, b) = (r.focus_lord_from_lagna_lord, r.lagna_lord_from_focus_lord);
+            // Aynı burç 1 ve 1; değilse iki sayım 14'e tamamlanır.
+            prop_assert!((a == 1 && b == 1) || a + b == 14);
+            prop_assert_eq!(r.same_graha, l.lord.graha == fo.lord.graha);
+            if r.same_graha {
+                prop_assert!(r.links.is_empty());
+            }
+            prop_assert_eq!(r.links.contains(&LordLink::Conjunct), !r.same_graha && a == 1);
+            let one_way = [LordLink::LagnaLordAspects, LordLink::FocusLordAspects];
+            let aspect_links = r
+                .links
+                .iter()
+                .filter(|k| **k == LordLink::MutualAspect || one_way.contains(k))
+                .count();
+            prop_assert!(aspect_links <= 1);
+            // Birlikte duran iki graha birbirinin burcuna bakmaz ve değişimde olamaz.
+            if r.links.contains(&LordLink::Conjunct) {
+                prop_assert_eq!(r.links.len(), 1);
+            }
+            let mut sorted = r.links.clone();
+            sorted.sort_by_key(|k| LordLink::ALL.iter().position(|x| x == k));
+            prop_assert_eq!(&sorted, &r.links);
+            // Ay: ev, tithi ve paksha birbirini tutar.
+            prop_assert_eq!(f.moon.house, p.house_of(Graha::Moon));
+            prop_assert_eq!(f.moon.paksha == Paksha::Shukla, moon_is_waxing(&p));
+            prop_assert!(!f.moon.aspected_by.contains(&Graha::Moon));
+            prop_assert_eq!(&f.yogas, &yogas(&p));
+            // Lagnasız haritada lagnaya bağlı hiçbir şey yoktur.
+            let hidden = prashna_facts(&p.clone().without_lagna(), &CombustionOrbs::default(), Some(focus)).unwrap();
+            prop_assert!(hidden.lagna.is_none() && hidden.focus.is_none());
+            prop_assert_eq!(hidden.moon.house, None);
+        }
+
         /// Spec §11: `compute_chart_with_lagna` gerçek lagna verildiğinde `compute_chart` ile
         /// aynı haritayı döner. Burç evi sisteminde ve sidereal kipte eşitlik bit bittir;
         /// tropikal-rasi kipinde lagnanın rasi boylamı `sidereal + ayanamsa` ile yeniden
diff --git a/crates/jyotish-core/tests/modules_golden.rs b/crates/jyotish-core/tests/modules_golden.rs
index b7f64a0..2c15cfd 100644
--- a/crates/jyotish-core/tests/modules_golden.rs
+++ b/crates/jyotish-core/tests/modules_golden.rs
@@ -19,6 +19,10 @@ use jyotish_core::{
     CHARA_DASHA_METHOD, DRISHTI_METHOD, JAIMINI_METHOD, KUTA_METHOD, SHADBALA_GRAHAS,
     SHADBALA_METHOD, YOGA_CATALOG, YOGA_CATALOG_VERSION, YOGA_CONDITIONS,
 };
+use jyotish_core::{
+    number_lagna_lon, prashna_facts, rising_mode, CombustionOrbs, CoreError, Dignity, LordLink,
+    Paksha, RisingMode, SignKind, PRASHNA_METHOD,
+};
 use serde::de::DeserializeOwned;
 use serde::Deserialize;
 
@@ -31,6 +35,7 @@ const MIN_SHADBALA: usize = 4;
 const MIN_YOGAS: usize = 15;
 const MIN_KUTA: usize = 8;
 const MIN_MUHURTA: usize = 7;
+const MIN_PRASHNA: usize = 14;
 /// El hesabı ile motor arasındaki en büyük fark, virupa.
 const VIRUPA_TOL: f64 = 1e-9;
 /// Shadbala çalışma kâğıdının toleransı, virupa (ileri modüller spec'i §7 "Test").
@@ -1703,3 +1708,330 @@ fn muhurta_fixtures_cover_the_required_cases() {
         (696.0, 744.0)
     );
 }
+
+/// Prashna örneklerinin girdisi: sentetik konumlar ve sorunun seçimleri. Lagna ya `lagna_lon`
+/// ile verilir ya da `number` ile (sayı prashnası: lagna sayıdan türetilir); ikisi birlikte
+/// yazılmaz. `retrograde` listesindeki grahaların hızı eksidir.
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct PrashnaInput {
+    lagna_lon: Option<f64>,
+    number: Option<u8>,
+    focus_house: Option<u8>,
+    grahas: BTreeMap<Graha, f64>,
+    retrograde: Vec<Graha>,
+}
+
+#[derive(Debug, Deserialize, PartialEq)]
+#[serde(deny_unknown_fields)]
+struct ExpectedLord {
+    graha: Graha,
+    sign: Sign,
+    house: u8,
+    dignity: Option<Dignity>,
+    retrograde: bool,
+    combust: bool,
+}
+
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct ExpectedKendra {
+    house: u8,
+    sign: Sign,
+    grahas: Vec<(Graha, Nature)>,
+}
+
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct ExpectedLagna {
+    sign: Sign,
+    /// Lagnanın burç içi derecesi (sayı prashnasında navamsanın ortası).
+    deg_in_sign: f64,
+    kind: SignKind,
+    rising: RisingMode,
+    lord: ExpectedLord,
+    kendras: Vec<ExpectedKendra>,
+}
+
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct ExpectedFocus {
+    house: u8,
+    sign: Sign,
+    lord: ExpectedLord,
+    same_graha: bool,
+    links: Vec<LordLink>,
+    focus_lord_from_lagna_lord: u8,
+    lagna_lord_from_focus_lord: u8,
+    occupants: Vec<(Graha, Nature)>,
+}
+
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct ExpectedMoon {
+    sign: Sign,
+    house: Option<u8>,
+    nakshatra: u8,
+    pada: u8,
+    tithi: u8,
+    paksha: Paksha,
+    aspected_by: Vec<Graha>,
+}
+
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct PrashnaExpected {
+    lagna: Option<ExpectedLagna>,
+    focus: Option<ExpectedFocus>,
+    moon: ExpectedMoon,
+}
+
+/// Lagnanın burç içi derecesinde el hesabı ile motor arasındaki en büyük fark.
+const LAGNA_DEG_TOL: f64 = 1e-9;
+
+fn prashna_fixtures() -> Vec<(Fixture<PrashnaExpected, PrashnaInput>, Positions)> {
+    load_raw::<PrashnaExpected, PrashnaInput>("prashna", MIN_PRASHNA)
+        .into_iter()
+        .map(|f| {
+            let id = &f.id;
+            let i = &f.input;
+            assert_eq!(i.grahas.len(), 9, "{id}: all nine grahas are required");
+            assert!(
+                i.lagna_lon.is_none() || i.number.is_none(),
+                "{id}: the lagna comes from the longitude or from the number, not both"
+            );
+            let lagna = match i.number {
+                Some(n) => Some(number_lagna_lon(n).unwrap_or_else(|e| panic!("{id}: {e}"))),
+                None => i.lagna_lon,
+            };
+            let mut positions =
+                Positions::synthetic(lagna, std::array::from_fn(|k| i.grahas[&Graha::ALL[k]]));
+            for g in &i.retrograde {
+                positions.grahas[*g as usize].speed = -1.0;
+            }
+            (f, positions)
+        })
+        .collect()
+}
+
+fn lord_of(l: &jyotish_core::LordFacts) -> ExpectedLord {
+    ExpectedLord {
+        graha: l.graha,
+        sign: l.sign,
+        house: l.house,
+        dignity: l.dignity,
+        retrograde: l.retrograde,
+        combust: l.combust,
+    }
+}
+
+fn occupants_of(list: &[jyotish_core::Occupant]) -> Vec<(Graha, Nature)> {
+    list.iter().map(|o| (o.graha, o.nature)).collect()
+}
+
+#[test]
+fn prashna_hand_fixtures() {
+    // Yanma orbları v1'in varsayılanlarıdır (el örnekleri bu orblarla türetilmiştir).
+    let orbs = CombustionOrbs::default();
+    for (f, positions) in prashna_fixtures() {
+        let id = &f.id;
+        let got = prashna_facts(&positions, &orbs, f.input.focus_house)
+            .unwrap_or_else(|e| panic!("{id}: {e}"));
+        assert_eq!(got.method, PRASHNA_METHOD, "{id}");
+        assert!(got.warnings.is_empty(), "{id}");
+        assert_eq!(got.yogas, yogas(&positions), "{id}: yoga list");
+
+        match (&f.expected.lagna, &got.lagna) {
+            (None, None) => assert!(positions.lagna_lon.is_none(), "{id}"),
+            (Some(want), Some(have)) => {
+                assert_eq!(
+                    (have.sign, have.kind, have.rising),
+                    (want.sign, want.kind, want.rising),
+                    "{id}: lagna"
+                );
+                let lon = positions.lagna_lon.expect("lagna");
+                assert_eq!(Sign::of(lon), want.sign, "{id}: lagna sign");
+                assert!(
+                    (jyotish_core::deg_in_sign(lon) - want.deg_in_sign).abs() < LAGNA_DEG_TOL,
+                    "{id}: lagna degree"
+                );
+                assert_eq!(lord_of(&have.lord), want.lord, "{id}: lagna lord");
+                assert_eq!(have.kendras.len(), 4, "{id}");
+                assert_eq!(want.kendras.len(), 4, "{id}: four kendras");
+                for (h, w) in have.kendras.iter().zip(&want.kendras) {
+                    assert_eq!((h.house, h.sign), (w.house, w.sign), "{id}: kendra");
+                    assert_eq!(occupants_of(&h.grahas), w.grahas, "{id}: house {}", w.house);
+                }
+            }
+            (want, have) => panic!(
+                "{id}: lagna expected {}, got {}",
+                want.is_some(),
+                have.is_some()
+            ),
+        }
+
+        match (&f.expected.focus, &got.focus) {
+            (None, None) => {}
+            (Some(want), Some(have)) => {
+                assert_eq!(Some(want.house), f.input.focus_house, "{id}");
+                assert_eq!(
+                    (have.house, have.sign),
+                    (want.house, want.sign),
+                    "{id}: focus"
+                );
+                assert_eq!(lord_of(&have.lord), want.lord, "{id}: focus lord");
+                assert_eq!(have.relation.same_graha, want.same_graha, "{id}");
+                assert_eq!(have.relation.links, want.links, "{id}: links");
+                assert_eq!(
+                    (
+                        have.relation.focus_lord_from_lagna_lord,
+                        have.relation.lagna_lord_from_focus_lord
+                    ),
+                    (
+                        want.focus_lord_from_lagna_lord,
+                        want.lagna_lord_from_focus_lord
+                    ),
+                    "{id}: counts"
+                );
+                assert_eq!(occupants_of(&have.occupants), want.occupants, "{id}");
+            }
+            (want, have) => panic!(
+                "{id}: focus expected {}, got {}",
+                want.is_some(),
+                have.is_some()
+            ),
+        }
+
+        let (m, w) = (&got.moon, &f.expected.moon);
+        assert_eq!(
+            (m.sign, m.house, m.nakshatra, m.pada, m.tithi, m.paksha),
+            (w.sign, w.house, w.nakshatra, w.pada, w.tithi, w.paksha),
+            "{id}: moon"
+        );
+        assert_eq!(m.aspected_by, w.aspected_by, "{id}: aspects on the moon");
+    }
+}
+
+/// Sayı → lagna eşlemesinin elle yazılmış ikinci tablosu (spec §11 "Test"): sayı, burç, derece,
+/// dakika. Örneklerin `deg_in_sign` değeri de bu tabloyla karşılaştırılır.
+const HAND_NUMBER_LAGNA: [(u8, Sign, f64, f64); 5] = [
+    (1, Sign::Aries, 1.0, 40.0),
+    (9, Sign::Aries, 28.0, 20.0),
+    (10, Sign::Taurus, 1.0, 40.0),
+    (54, Sign::Virgo, 28.0, 20.0),
+    (108, Sign::Pisces, 28.0, 20.0),
+];
+
+/// On iki burcun doğuş biçiminin elle yazılmış ikinci tablosu (spec §11), burç sırasıyla.
+const HAND_RISING: [(Sign, RisingMode); 12] = [
+    (Sign::Aries, RisingMode::BackFirst),
+    (Sign::Taurus, RisingMode::BackFirst),
+    (Sign::Gemini, RisingMode::HeadFirst),
+    (Sign::Cancer, RisingMode::BackFirst),
+    (Sign::Leo, RisingMode::HeadFirst),
+    (Sign::Virgo, RisingMode::HeadFirst),
+    (Sign::Libra, RisingMode::HeadFirst),
+    (Sign::Scorpio, RisingMode::HeadFirst),
+    (Sign::Sagittarius, RisingMode::BackFirst),
+    (Sign::Capricorn, RisingMode::BackFirst),
+    (Sign::Aquarius, RisingMode::HeadFirst),
+    (Sign::Pisces, RisingMode::BothWays),
+];
+
+#[test]
+fn prashna_fixtures_cover_the_required_cases() {
+    let fixtures = prashna_fixtures();
+    // Sayı → lagna: 1, 9, 10, 54, 108; sınır dışı sayılar hatadır.
+    for (n, sign, deg, min) in HAND_NUMBER_LAGNA {
+        let (f, _) = fixtures
+            .iter()
+            .find(|(f, _)| f.input.number == Some(n))
+            .unwrap_or_else(|| panic!("no fixture with the number {n}"));
+        let lagna = f.expected.lagna.as_ref().expect("a number gives a lagna");
+        assert_eq!(lagna.sign, sign, "{}", f.id);
+        assert!(
+            (lagna.deg_in_sign - (deg + min / 60.0)).abs() < LAGNA_DEG_TOL,
+            "{}",
+            f.id
+        );
+    }
+    for bad in [0, 109] {
+        assert_eq!(
+            number_lagna_lon(bad),
+            Err(CoreError::InvalidPrashnaInput("number"))
+        );
+    }
+    // On iki burcun doğuş biçimi: her burç bir örneğin lagnasıdır ve el tablosuyla aynıdır.
+    for (sign, rising) in HAND_RISING {
+        assert_eq!(rising_mode(sign), rising, "{sign:?}");
+        let (f, _) = fixtures
+            .iter()
+            .find(|(f, _)| f.expected.lagna.as_ref().is_some_and(|l| l.sign == sign))
+            .unwrap_or_else(|| panic!("no fixture with the lagna {sign:?}"));
+        assert_eq!(
+            f.expected.lagna.as_ref().unwrap().rising,
+            rising,
+            "{}",
+            f.id
+        );
+    }
+    // Lord ilişkilerinin beş türü (tek yönlü bakışın iki yönü ayrı), "hiçbiri" ve aynı graha.
+    let focuses: Vec<&ExpectedFocus> = fixtures
+        .iter()
+        .filter_map(|(f, _)| f.expected.focus.as_ref())
+        .collect();
+    for link in LordLink::ALL {
+        assert!(
+            focuses.iter().any(|f| f.links.contains(&link)),
+            "no fixture with {link:?}"
+        );
+    }
+    assert!(focuses.iter().any(|f| f.links.is_empty() && !f.same_graha));
+    assert!(focuses.iter().any(|f| f.same_graha));
+    assert!(
+        focuses.iter().any(|f| f.links.len() > 1),
+        "two links at once"
+    );
+    // Odak evi seçilmemiş ve lagnasız (saat bilinmiyor) örnekler.
+    assert!(fixtures
+        .iter()
+        .any(|(f, _)| f.input.focus_house.is_none() && f.expected.lagna.is_some()));
+    assert!(fixtures.iter().any(|(f, _)| {
+        f.expected.lagna.is_none() && f.expected.focus.is_none() && f.expected.moon.house.is_none()
+    }));
+    // Retro ve yanık lord; iki paksha; kendrada iki sınıftan graha ve düğüm.
+    let lords: Vec<&ExpectedLord> = fixtures
+        .iter()
+        .filter_map(|(f, _)| f.expected.lagna.as_ref())
+        .map(|l| &l.lord)
+        .chain(focuses.iter().map(|f| &f.lord))
+        .collect();
+    assert!(lords.iter().any(|l| l.retrograde));
+    assert!(lords.iter().any(|l| l.combust));
+    for want in [
+        Dignity::Exalted,
+        Dignity::Debilitated,
+        Dignity::Moolatrikona,
+        Dignity::OwnSign,
+    ] {
+        assert!(lords.iter().any(|l| l.dignity == Some(want)), "{want:?}");
+    }
+    for paksha in [Paksha::Shukla, Paksha::Krishna] {
+        assert!(fixtures
+            .iter()
+            .any(|(f, _)| f.expected.moon.paksha == paksha));
+    }
+    assert!(fixtures
+        .iter()
+        .any(|(f, _)| !f.expected.moon.aspected_by.is_empty()));
+    let in_kendras: Vec<(Graha, Nature)> = fixtures
+        .iter()
+        .filter_map(|(f, _)| f.expected.lagna.as_ref())
+        .flat_map(|l| l.kendras.iter().flat_map(|k| k.grahas.clone()))
+        .collect();
+    for nature in [Nature::Benefic, Nature::Malefic] {
+        assert!(in_kendras.iter().any(|(_, n)| *n == nature));
+    }
+    assert!(in_kendras.contains(&(Graha::Mercury, Nature::Malefic)));
+    assert!(in_kendras.iter().any(|(g, _)| g.is_node()));
+}
diff --git a/tests/golden/modules/README.md b/tests/golden/modules/README.md
index 06889a2..268e1ad 100644
--- a/tests/golden/modules/README.md
+++ b/tests/golden/modules/README.md
@@ -55,6 +55,7 @@ doğum anı ve 365,25 günlük yıl kullanır, tarihleri el yıllarının biriki
 | `yogas` | `hits` (raporun bütün kayıtları, katalog sırasıyla: `id`, `grahas`, `houses`, `relations`, `conditions`), `absent` (bu örneğin "oluşmayan durum" olarak gösterdiği kurallar), `cancelled` (iptal örnekleri), `evaluated`, `skipped_time_unknown` | 15 |
 | `kuta` | `scores` (sekiz kuta, tablo sırasıyla: `kuta`, `score`, `groom_class`, `bride_class`), `total`, `bhakoot_exception`, `nadi_exception`, `kuja_dosha` (`[damat rolü, gelin rolü]`), `overlay` (`groom` ve `bride`: dokuzar satır `[diğer tarafın lagnasından ev ya da null, Ay'ından ev]`), `approximate` | 8 |
 | `muhurta` | `eighth_min`, `rahu`, `yamaganda`, `gulika` (`segment`, `from_min`, `to_min`), `abhijit` (`from_min`, `to_min`), `abhijit_weekday_exception`, `hora_min`, `hora_lords` (24), `tithi_group`, `nakshatra_nature`, `tara` (`number`, `favourable`), `chandra` (`house`, `favourable`) | 7 |
+| `prashna` | `lagna` (`sign`, `deg_in_sign`, `kind`, `rising`, `lord`, dört `kendras`; lagna yoksa `null`), `focus` (`house`, `sign`, `lord`, `same_graha`, `links`, iki ev sayısı, `occupants`; odak evi ya da lagna yoksa `null`), `moon` (`sign`, `house`, `nakshatra`, `pada`, `tithi`, `paksha`, `aspected_by`) | 14 |
 
 Shadbala örnekleri çalışma kâğıdıdır (tasarım belgesi §7 "Test"): `shadbala/WORKSHEET.md` her
 kalemin aritmetiğini adım adım yazar, JSON dosyaları aynı girdiyi ve sonuçları taşır. Tolerans
@@ -90,4 +91,14 @@ tasarım belgesinin örneğinin (06:00–18:00 Pazartesi) aralarında bulunduğu
 kuralları, motorun tabloları, sahibin onayını bekleyen varyant noktaları ve bağımsız gözden
 geçirmenin kaydı `muhurta/REVIEW.md`'dedir.
 
+Prashna örnekleri (tasarım belgesi §11 "Test") girdiye üç alan ekler ve `advanced` taşımaz:
+`number` (sayı prashnası: lagna sayıdan türetilir; o zaman `lagna_lon` `null` yazılır),
+`focus_house` (astrologun seçtiği ev; `null` olabilir) ve `retrograde` (hızı eksi sayılan
+grahalar). `lord` nesnesi `graha`, `sign`, `house`, `dignity`, `retrograde` ve `combust` taşır
+(yanma v1'in varsayılan orblarıyladır); kendra ve odak evi listeleri `[graha, sınıf]` çiftleridir.
+Ayrı bir test örneklerin beş sayıyı (1, 9, 10, 54, 108), on iki lagna burcunu, lord ilişkisinin
+beş bağını, "hiçbiri"ni ve aynı grahayı, odak evi seçilmemiş ve lagnasız haritayı, retro ve yanık
+lordu ve iki pakshayı kapsadığını sabitler. Okuma kuralları, motorun tabloları, sahibin onayını
+bekleyen varyant noktaları ve bağımsız gözden geçirmenin kaydı `prashna/REVIEW.md`'dedir.
+
 Sonraki alt projeler kendi klasörünü ve `expected` yapısını ekler.
````

**Yeni dosya:** `tests/golden/modules/prashna/REVIEW.md`

````markdown
# Prashna el örnekleri: okuma kuralları, tablolar, onay bekleyen noktalar ve gözden geçirme kaydı

Kaynak: ileri modüller tasarım belgesi §11 (sayı prashnasının lagnası ve soru haritasının
olguları), K4–K6 (burç evleri, rasi çerçevesi, doğal sınıf) ve v1'in dignity ile yanma
kuralları. Bu dizindeki her JSON dosyası sentetik bir haritadır: dokuz grahanın sidereal
boylamı, lagna (boylam olarak ya da 1–108 arası bir sayı olarak; `null` ise saat bilinmiyor),
retro sayılan grahalar ve astrologun seçtiği odak evi. Ephemeris yoktur; rasi çerçevesi sidereal
sayılır. `derivation` lagnayı, burcun türünü ve doğuş biçimini, lagna lordunu, kendraları, odak
evini ve lordunu, iki lordun bağını ve ev sayılarını, Ay'ı ve Ay'ın aldığı bakışları sırayla
yazar. Değerler kuraldan elle türetilmiştir; motor çıktısı buraya kopyalanmaz (K9). Bir değer
motorla uyuşmazsa önce kural yeniden okunur. Hiçbir satır hüküm değildir: örnekler olguları
sayar, sorunun yanıtını söylemez.

## Okuma kuralları

1. **Sayı prashnasının lagnası.** Sayı `n` (1–108) zodyağın `n`. navamsasını gösterir; lagna o
   navamsanın ortasıdır: `(n − 1) × 3°20′ + 1°40′`, sidereal boylam. 0 ve 109 geçersizdir.
   Burç başına dokuz sayı düşer (1–9 Koç, 10–18 Boğa … 100–108 Balık). Grahalar anın
   grahalarıdır; evler bu lagnadan sayılan burç evleridir.
2. **Burç türü.** Koç, Yengeç, Terazi, Oğlak yürük (`Movable`); Boğa, Aslan, Akrep, Kova sabit
   (`Fixed`); İkizler, Başak, Yay, Balık çift doğalı (`Dual`).
3. **Doğuş biçimi** aşağıdaki tablodan.
4. **Ev ve lord.** Ev, lagna burcundan sayılan burçtur (lagna burcu 1). Bir evin lordu, o
   burcun lordudur (tablo aşağıda). Lordun evi, bulunduğu burcun lagnadan sayısıdır.
5. **Dignity** (v1 kuralı, sırayla): yücelme burcu (Ay ve Merkür'de yalnızca moolatrikona
   aralığından önceki dereceler); düşüş burcu (yücelmenin karşısı); moolatrikona aralığı; kendi
   burcu; değilse beşli ilişki: burcun lorduyla doğal ilişki (dost, nötr, düşman) ile geçici
   ilişkinin (lordun bulunduğu burç, grahanın burcundan 2, 3, 4, 10, 11 ya da 12. ise dost,
   değilse düşman) birleşimi: dost + dost `GreatFriend`, dost + düşman `Neutral`, nötr + dost
   `Friend`, nötr + düşman `Enemy`, düşman + dost `Neutral`, düşman + düşman `GreatEnemy`.
6. **Retro** girdideki `retrograde` listesidir. **Yanma:** grahanın Güneş'e açısal uzaklığı orbun
   içindeyse (sınır dahil) yanıktır. Orblar: Ay 12°, Mars 17°, Merkür 14° (retro 12°),
   Jüpiter 11°, Venüs 10° (retro 8°), Satürn 15°. Güneş ve düğümler yanmaz.
7. **İki lordun bağı.** Lagna lordu ile odak evinin lordu aynı grahaysa `same_graha` doğrudur ve
   bağ aranmaz. Değilse sırayla: aynı burçtalarsa `Conjunct`; her biri diğerinin sahip olduğu bir
   burçtaysa `Exchange`; bakış için her grahanın tam bakışla baktığı burçlar bulunur (kendi
   burcundan sayılır: herkes 7.; Mars ayrıca 4. ve 8.; Jüpiter 5. ve 9.; Satürn 3. ve 10.):
   ikisi de diğerinin burcuna bakıyorsa `MutualAspect`, yalnızca lagna lordu bakıyorsa
   `LagnaLordAspects`, yalnızca odak lordu bakıyorsa `FocusLordAspects`. Birden çok bağ birlikte
   geçerli olabilir; hiçbiri yoksa liste boştur.
8. **Ev sayıları.** `focus_lord_from_lagna_lord`: odak lordunun burcunun lagna lordunun
   burcundan sayısı (aynı burç 1); `lagna_lord_from_focus_lord` tersidir.
9. **Doğal sınıf (K6).** Jüpiter ve Venüs iyicil; Güneş, Mars, Satürn, Rahu, Ketu kötücül; Ay,
   Güneş'ten uzanımı `[0°, 180°)` ise iyicil, değilse kötücül (tam 180° kötücül); Merkür aynı
   burçta kötücül varken iyicil yoksa kötücül, değilse iyicil.
10. **Kendralar** 1, 4, 7 ve 10. evlerdir; her birindeki grahalar Güneş, Ay, Mars, Merkür,
    Jüpiter, Venüs, Satürn, Rahu, Ketu sırasıyla ve sınıflarıyla yazılır. Odak evindeki grahalar
    aynı biçimdedir.
11. **Ay.** Burç ve lagnadan ev; nakshatra ve pada sidereal boylamdan (bir pada 3°20′, bir
    nakshatra dört pada); tithi, Güneş'ten uzanımın 12°'lik dilimidir (0°–12° 1. tithi; tam 180°
    16. tithi); 1–15 `Shukla`, 16–30 `Krishna`. `aspected_by`: Ay'ın burcuna tam bakışla bakan
    grahalar (madde 7'nin bakışları), Güneş, Mars, Merkür, Jüpiter, Venüs, Satürn sırasıyla.
12. **Lagna yoksa** (`lagna_lon` ve `number` ikisi de `null`) `lagna` ve `focus` `null`'dır ve
    Ay'ın evi `null`'dır. **Odak evi seçilmemişse** `focus` `null`'dır.

## Tablolar

Doğuş biçimi (BPHS, burç nitelikleri bölümü):

| Biçim | Burçlar |
|---|---|
| Başla doğan (`HeadFirst`) | İkizler, Aslan, Başak, Terazi, Akrep, Kova |
| Sırtla doğan (`BackFirst`) | Koç, Boğa, Yengeç, Yay, Oğlak |
| İki yönlü (`BothWays`) | Balık |

Burç lordları, yücelme ve moolatrikona (v1; BPHS, graha nitelikleri bölümü):

| Graha | Kendi burçları | Yücelme burcu | Moolatrikona |
|---|---|---|---|
| Güneş | Aslan | Koç | Aslan 0°–20° |
| Ay | Yengeç | Boğa | Boğa 3°–30° |
| Mars | Koç, Akrep | Oğlak | Koç 0°–12° |
| Merkür | İkizler, Başak | Başak | Başak 15°–20° |
| Jüpiter | Yay, Balık | Yengeç | Yay 0°–10° |
| Venüs | Boğa, Terazi | Balık | Terazi 0°–15° |
| Satürn | Oğlak, Kova | Terazi | Kova 0°–20° |

Doğal ilişkiler (yazılmayan nötrdür):

| Graha | Dostları | Düşmanları |
|---|---|---|
| Güneş | Ay, Mars, Jüpiter | Venüs, Satürn |
| Ay | Güneş, Merkür | — |
| Mars | Güneş, Ay, Jüpiter | Merkür |
| Merkür | Güneş, Venüs | Ay |
| Jüpiter | Güneş, Ay, Mars | Merkür, Venüs |
| Venüs | Merkür, Satürn | Güneş, Ay |
| Satürn | Merkür, Venüs | Güneş, Ay, Mars |

## Sahibin onayını bekleyen klasik tablolar ve bilinen varyant noktaları

Doğuş biçimi tablosu tasarım belgesinde yazılıdır ve klasik derlemelerin yaygın biçimidir; sayı
yöntemi de tasarım belgesindeki biçimdedir. İkisi bu depoya bellekten aktarılmıştır, basılı bir
kaynakla karşılaştırılmamıştır. Aşağıdaki noktalarda gelenekler ayrışır. Karşılaştırma bir
insanın işidir: elinde basılı bir kaynak bulunan kişi her satırı denetler, kaynağı genel adıyla
(alıntı yapmadan) ve sonucu aşağıdaki tabloya yazar. Fark bulunursa tablo ayrı bir commit'le
düzeltilir ve bu dizindeki örnekler kuraldan yeniden türetilir.

| No | Nokta | Bu depodaki okuma | Bilinen diğer okuma |
|---|---|---|---|
| 1 | İkizler'in doğuş biçimi | Başla doğan | İkizler'i gece burçlarıyla birlikte sırtla doğan sayan listeler (gece burçları Koç, Boğa, İkizler, Yengeç, Yay, Oğlak'tır; doğuş biçimi listesiyle bir burç ayrışır) |
| 2 | Balık | İki yönlü | Balık'ı başla doğanlara katan iki sınıflı listeler |
| 3 | Sayının lagnası | 1–108: `n`. navamsanın ortası (`(n − 1) × 3°20′ + 1°40′`) | Navamsanın başını alan okuma; sayıyı on ikiye bölüp yalnızca burç veren okuma; 1–249 sayı sistemi (tasarım belgesinde kapsam dışı) |
| 4 | Sayı haritasının evleri | Sayı lagnasından burç evleri; ev sistemi ayarı uygulanmaz | Anın gerçek lagnasını ikinci bir lagna olarak yanında gösteren uygulamalar |
| 5 | İki lordun bakışı | Burç temelli tam bakışlar (özel bakışlar dahil); kısmi bakış ve orb yok | Derece temelli (sphuta) bakış; Tajika bakışları ve yogaları (tasarım belgesinde kapsam dışı) |
| 6 | Değişim | Her lord diğerinin sahip olduğu herhangi bir burçta | Yalnızca ilgili iki evin burçları arasındaki değişimi sayan dar okuma |
| 7 | Ay'ın aldığı bakışlar | Düğümler bakış atmaz; Ay'la aynı burçtaki graha "bakan" sayılmaz | Rahu ve Ketu'ya 5., 7. ve 9. bakış veren okumalar |

| Denetleyen | Tarih | Kaynak (genel ad) | Sonuç |
|---|---|---|---|
| — | — | — | Bekliyor: sahibine devredildi (Plan 21 Task 8) |

## Gözden geçirme kaydı

Gözden geçiren (motoru yazan dışında biri) her örnekte yalnızca `input`'tan başlar, motoru
çalıştırmadan ve Rust kaynağını okumadan lagnayı (sayıdan ya da boylamdan), burç türünü ve
doğuş biçimini, lagna lordunun burcunu, evini, dignity'sini ve yanmasını, dört kendrayı, odak
evinin burcunu ve lordunu, iki lordun bağını ve ev sayılarını, Ay'ın burcunu, evini,
nakshatrasını, padasını, tithisini ve aldığı bakışları tasarım belgesinden, yukarıdaki okuma
kurallarından ve tablolardan yeniden hesaplar; sonucunu `expected` ile karşılaştırır ve aşağıya
adını, tarihini ve bulduğu farkları yazar.

| Gözden geçiren | Tarih | Sonuç |
|---|---|---|
| — | — | Bekliyor (Plan 21 Task 8 Step 5) |
````

**Yeni dosya:** `tests/golden/modules/prashna/aquarius-exchange-and-one-way-aspect-nodes-in-kendras.json`

````json
{
  "id": "aquarius-exchange-and-one-way-aspect-nodes-in-kendras",
  "source": "hand",
  "rule_ref": "BPHS, burç nitelikleri, graha nitelikleri ve bakışlar bölümleri; prashna derlemelerinin sayı yöntemi (alıntı yapılmaz); ileri modüller spec'i §11",
  "derivation": [
    "Lagna 310°00′ sidereal → Kova 10°00′.",
    "Kova: burç türü sabit; doğuş biçimi başla doğan.",
    "Lagna lordu Satürn: Akrep 15°00′; lagnadan 10. ev. Dignity: burcun lordu Mars: doğal ilişki düşman; Mars Oğlak burcunda, Akrep'dan 3. burç → geçici dost → Neutral → Neutral. Retro: hayır. Yanma: Güneş'e uzaklık 155°00′, orb 15° → yanık değil.",
    "Kendralar. 1. ev Kova: Rahu (kötücül); 4. ev Boğa: boş; 7. ev Aslan: Ay (iyicil), Ketu (kötücül); 10. ev Akrep: Satürn (kötücül).",
    "Odak evi 3: Kova'dan 3. burç Koç; lordu Mars.",
    "Odak lordu Mars: Oğlak 15°00′; lagnadan 12. ev. Dignity: yücelme burcu Oğlak → Exalted. Retro: hayır. Yanma: Güneş'e uzaklık 145°00′, orb 17° → yanık değil.",
    "Bağ. Satürn Akrep burcunda, Mars Oğlak burcunda. Aynı burç: hayır. Değişim (Satürn Mars'in burcunda ve tersi): evet. Satürn'in tam bakışları (3, 7, 10. burçlar): Oğlak, Boğa, Aslan → Oğlak'a bakar. Mars'in tam bakışları: Koç, Yengeç, Aslan → Akrep'a bakmaz. Sonuç: Exchange, LagnaLordAspects.",
    "Ev sayıları: odak lordu lagna lordundan 3. burçta; lagna lordu odak lordundan 11. burçta.",
    "Odak evindeki grahalar (Koç): yok.",
    "Ay 130°00′ → Aslan, lagnadan 7. ev; nakshatra 10. (Magha), pada 4. Güneş'ten uzanım 60°00′ → tithi 6 (Shukla); Ay büyüyor: iyicil.",
    "Ay'ın burcuna (Aslan) tam bakışla bakanlar: Mars (Oğlak'dan 8. burç), Satürn (Akrep'dan 10. burç)."
  ],
  "input": {
    "lagna_lon": 310,
    "number": null,
    "focus_house": 3,
    "grahas": {
      "Sun": 70.0,
      "Moon": 130.0,
      "Mars": 285.0,
      "Mercury": 85.0,
      "Jupiter": 190.0,
      "Venus": 100.0,
      "Saturn": 225.0,
      "Rahu": 305.0,
      "Ketu": 125.0
    },
    "retrograde": []
  },
  "expected": {
    "lagna": {
      "sign": "Aquarius",
      "deg_in_sign": 10.0,
      "kind": "Fixed",
      "rising": "HeadFirst",
      "lord": {
        "graha": "Saturn",
        "sign": "Scorpio",
        "house": 10,
        "dignity": "Neutral",
        "retrograde": false,
        "combust": false
      },
      "kendras": [
        {
          "house": 1,
          "sign": "Aquarius",
          "grahas": [
            [
              "Rahu",
              "Malefic"
            ]
          ]
        },
        {
          "house": 4,
          "sign": "Taurus",
          "grahas": []
        },
        {
          "house": 7,
          "sign": "Leo",
          "grahas": [
            [
              "Moon",
              "Benefic"
            ],
            [
              "Ketu",
              "Malefic"
            ]
          ]
        },
        {
          "house": 10,
          "sign": "Scorpio",
          "grahas": [
            [
              "Saturn",
              "Malefic"
            ]
          ]
        }
      ]
    },
    "focus": {
      "house": 3,
      "sign": "Aries",
      "lord": {
        "graha": "Mars",
        "sign": "Capricorn",
        "house": 12,
        "dignity": "Exalted",
        "retrograde": false,
        "combust": false
      },
      "same_graha": false,
      "links": [
        "Exchange",
        "LagnaLordAspects"
      ],
      "focus_lord_from_lagna_lord": 3,
      "lagna_lord_from_focus_lord": 11,
      "occupants": []
    },
    "moon": {
      "sign": "Leo",
      "house": 7,
      "nakshatra": 10,
      "pada": 4,
      "tithi": 6,
      "paksha": "Shukla",
      "aspected_by": [
        "Mars",
        "Saturn"
      ]
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/prashna/cancer-waning-moon-no-relation.json`

````json
{
  "id": "cancer-waning-moon-no-relation",
  "source": "hand",
  "rule_ref": "BPHS, burç nitelikleri, graha nitelikleri ve bakışlar bölümleri; prashna derlemelerinin sayı yöntemi (alıntı yapılmaz); ileri modüller spec'i §11",
  "derivation": [
    "Lagna 100°00′ sidereal → Yengeç 10°00′.",
    "Yengeç: burç türü yürük; doğuş biçimi sırtla doğan.",
    "Lagna lordu Ay: Akrep 10°00′; lagnadan 5. ev. Dignity: düşüş burcu Akrep → Debilitated. Retro: hayır. Yanma: Güneş'e uzaklık 30°00′, orb 12° → yanık değil.",
    "Kendralar. 1. ev Yengeç: boş; 4. ev Terazi: boş; 7. ev Oğlak: boş; 10. ev Koç: Jüpiter (iyicil).",
    "Odak evi 2: Yengeç'dan 2. burç Aslan; lordu Güneş.",
    "Odak lordu Güneş: Yay 10°00′; lagnadan 6. ev. Dignity: burcun lordu Jüpiter: doğal ilişki dost; Jüpiter Koç burcunda, Yay'dan 5. burç → geçici düşman → Neutral → Neutral. Retro: hayır. Yanma: Güneş yanmaz.",
    "Bağ. Ay Akrep burcunda, Güneş Yay burcunda. Aynı burç: hayır. Değişim (Ay Güneş'in burcunda ve tersi): hayır. Ay'in tam bakışları (7. burçlar): Boğa → Yay'a bakmaz. Güneş'in tam bakışları: İkizler → Akrep'a bakmaz. Sonuç: hiçbiri.",
    "Ev sayıları: odak lordu lagna lordundan 2. burçta; lagna lordu odak lordundan 12. burçta.",
    "Odak evindeki grahalar (Aslan): Mars (kötücül).",
    "Ay 220°00′ → Akrep, lagnadan 5. ev; nakshatra 17. (Anuradha), pada 3. Güneş'ten uzanım 330°00′ → tithi 28 (Krishna); Ay küçülüyor: kötücül.",
    "Ay'ın burcuna (Akrep) tam bakışla bakanlar: Mars (Aslan'dan 4. burç)."
  ],
  "input": {
    "lagna_lon": 100,
    "number": null,
    "focus_house": 2,
    "grahas": {
      "Sun": 250.0,
      "Moon": 220.0,
      "Mars": 130.0,
      "Mercury": 265.0,
      "Jupiter": 10.0,
      "Venus": 300.0,
      "Saturn": 75.0,
      "Rahu": 40.0,
      "Ketu": 220.0
    },
    "retrograde": []
  },
  "expected": {
    "lagna": {
      "sign": "Cancer",
      "deg_in_sign": 10.0,
      "kind": "Movable",
      "rising": "BackFirst",
      "lord": {
        "graha": "Moon",
        "sign": "Scorpio",
        "house": 5,
        "dignity": "Debilitated",
        "retrograde": false,
        "combust": false
      },
      "kendras": [
        {
          "house": 1,
          "sign": "Cancer",
          "grahas": []
        },
        {
          "house": 4,
          "sign": "Libra",
          "grahas": []
        },
        {
          "house": 7,
          "sign": "Capricorn",
          "grahas": []
        },
        {
          "house": 10,
          "sign": "Aries",
          "grahas": [
            [
              "Jupiter",
              "Benefic"
            ]
          ]
        }
      ]
    },
    "focus": {
      "house": 2,
      "sign": "Leo",
      "lord": {
        "graha": "Sun",
        "sign": "Sagittarius",
        "house": 6,
        "dignity": "Neutral",
        "retrograde": false,
        "combust": false
      },
      "same_graha": false,
      "links": [],
      "focus_lord_from_lagna_lord": 2,
      "lagna_lord_from_focus_lord": 12,
      "occupants": [
        [
          "Mars",
          "Malefic"
        ]
      ]
    },
    "moon": {
      "sign": "Scorpio",
      "house": 5,
      "nakshatra": 17,
      "pada": 3,
      "tithi": 28,
      "paksha": "Krishna",
      "aspected_by": [
        "Mars"
      ]
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/prashna/capricorn-full-moon-debilitated-lagna-lord.json`

````json
{
  "id": "capricorn-full-moon-debilitated-lagna-lord",
  "source": "hand",
  "rule_ref": "BPHS, burç nitelikleri, graha nitelikleri ve bakışlar bölümleri; prashna derlemelerinin sayı yöntemi (alıntı yapılmaz); ileri modüller spec'i §11",
  "derivation": [
    "Lagna 280°00′ sidereal → Oğlak 10°00′.",
    "Oğlak: burç türü yürük; doğuş biçimi sırtla doğan.",
    "Lagna lordu Satürn: Koç 15°00′; lagnadan 4. ev. Dignity: düşüş burcu Koç → Debilitated. Retro: hayır. Yanma: Güneş'e uzaklık 10°00′, orb 15° → yanık.",
    "Kendralar. 1. ev Oğlak: Ketu (kötücül); 4. ev Koç: Güneş (kötücül), Satürn (kötücül); 7. ev Yengeç: Rahu (kötücül); 10. ev Terazi: Ay (kötücül).",
    "Odak evi 9: Oğlak'dan 9. burç Başak; lordu Merkür.",
    "Odak lordu Merkür: İkizler 5°00′; lagnadan 6. ev. Dignity: kendi burcu İkizler → OwnSign. Retro: hayır. Yanma: Güneş'e uzaklık 60°00′, orb 14° → yanık değil.",
    "Bağ. Satürn Koç burcunda, Merkür İkizler burcunda. Aynı burç: hayır. Değişim (Satürn Merkür'in burcunda ve tersi): hayır. Satürn'in tam bakışları (3, 7, 10. burçlar): İkizler, Terazi, Oğlak → İkizler'a bakar. Merkür'in tam bakışları: Yay → Koç'a bakmaz. Sonuç: LagnaLordAspects.",
    "Ev sayıları: odak lordu lagna lordundan 3. burçta; lagna lordu odak lordundan 11. burçta.",
    "Odak evindeki grahalar (Başak): yok.",
    "Ay 185°00′ → Terazi, lagnadan 10. ev; nakshatra 14. (Chitra), pada 4. Güneş'ten uzanım 180°00′ → tithi 16 (Krishna); Ay küçülüyor: kötücül.",
    "Ay'ın burcuna (Terazi) tam bakışla bakanlar: Güneş (Koç'dan 7. burç), Jüpiter (Kova'dan 9. burç), Satürn (Koç'dan 7. burç)."
  ],
  "input": {
    "lagna_lon": 280,
    "number": null,
    "focus_house": 9,
    "grahas": {
      "Sun": 5.0,
      "Moon": 185.0,
      "Mars": 220.0,
      "Mercury": 65.0,
      "Jupiter": 310.0,
      "Venus": 40.0,
      "Saturn": 15.0,
      "Rahu": 100.0,
      "Ketu": 280.0
    },
    "retrograde": []
  },
  "expected": {
    "lagna": {
      "sign": "Capricorn",
      "deg_in_sign": 10.0,
      "kind": "Movable",
      "rising": "BackFirst",
      "lord": {
        "graha": "Saturn",
        "sign": "Aries",
        "house": 4,
        "dignity": "Debilitated",
        "retrograde": false,
        "combust": true
      },
      "kendras": [
        {
          "house": 1,
          "sign": "Capricorn",
          "grahas": [
            [
              "Ketu",
              "Malefic"
            ]
          ]
        },
        {
          "house": 4,
          "sign": "Aries",
          "grahas": [
            [
              "Sun",
              "Malefic"
            ],
            [
              "Saturn",
              "Malefic"
            ]
          ]
        },
        {
          "house": 7,
          "sign": "Cancer",
          "grahas": [
            [
              "Rahu",
              "Malefic"
            ]
          ]
        },
        {
          "house": 10,
          "sign": "Libra",
          "grahas": [
            [
              "Moon",
              "Malefic"
            ]
          ]
        }
      ]
    },
    "focus": {
      "house": 9,
      "sign": "Virgo",
      "lord": {
        "graha": "Mercury",
        "sign": "Gemini",
        "house": 6,
        "dignity": "OwnSign",
        "retrograde": false,
        "combust": false
      },
      "same_graha": false,
      "links": [
        "LagnaLordAspects"
      ],
      "focus_lord_from_lagna_lord": 3,
      "lagna_lord_from_focus_lord": 11,
      "occupants": []
    },
    "moon": {
      "sign": "Libra",
      "house": 10,
      "nakshatra": 14,
      "pada": 4,
      "tithi": 16,
      "paksha": "Krishna",
      "aspected_by": [
        "Sun",
        "Jupiter",
        "Saturn"
      ]
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/prashna/gemini-eighth-lord-aspects-the-lagna-lord.json`

````json
{
  "id": "gemini-eighth-lord-aspects-the-lagna-lord",
  "source": "hand",
  "rule_ref": "BPHS, burç nitelikleri, graha nitelikleri ve bakışlar bölümleri; prashna derlemelerinin sayı yöntemi (alıntı yapılmaz); ileri modüller spec'i §11",
  "derivation": [
    "Lagna 75°00′ sidereal → İkizler 15°00′.",
    "İkizler: burç türü çift doğalı; doğuş biçimi başla doğan.",
    "Lagna lordu Merkür: Balık 15°00′; lagnadan 10. ev. Dignity: düşüş burcu Balık → Debilitated. Retro: hayır. Yanma: Güneş'e uzaklık 35°00′, orb 14° → yanık değil.",
    "Kendralar. 1. ev İkizler: boş; 4. ev Başak: boş; 7. ev Yay: boş; 10. ev Balık: Merkür (iyicil).",
    "Odak evi 8: İkizler'dan 8. burç Oğlak; lordu Satürn.",
    "Odak lordu Satürn: Oğlak 15°00′; lagnadan 8. ev. Dignity: kendi burcu Oğlak → OwnSign. Retro: hayır. Yanma: Güneş'e uzaklık 25°00′, orb 15° → yanık değil.",
    "Bağ. Merkür Balık burcunda, Satürn Oğlak burcunda. Aynı burç: hayır. Değişim (Merkür Satürn'in burcunda ve tersi): hayır. Merkür'in tam bakışları (7. burçlar): Başak → Oğlak'a bakmaz. Satürn'in tam bakışları: Balık, Yengeç, Terazi → Balık'a bakar. Sonuç: FocusLordAspects.",
    "Ev sayıları: odak lordu lagna lordundan 11. burçta; lagna lordu odak lordundan 3. burçta.",
    "Odak evindeki grahalar (Oğlak): Venüs (iyicil), Satürn (kötücül).",
    "Ay 100°00′ → Yengeç, lagnadan 2. ev; nakshatra 8. (Pushya), pada 3. Güneş'ten uzanım 150°00′ → tithi 13 (Shukla); Ay büyüyor: iyicil.",
    "Ay'ın burcuna (Yengeç) tam bakışla bakanlar: Venüs (Oğlak'dan 7. burç), Satürn (Oğlak'dan 7. burç)."
  ],
  "input": {
    "lagna_lon": 75,
    "number": null,
    "focus_house": 8,
    "grahas": {
      "Sun": 310.0,
      "Moon": 100.0,
      "Mars": 40.0,
      "Mercury": 345.0,
      "Jupiter": 200.0,
      "Venus": 280.0,
      "Saturn": 285.0,
      "Rahu": 10.0,
      "Ketu": 190.0
    },
    "retrograde": []
  },
  "expected": {
    "lagna": {
      "sign": "Gemini",
      "deg_in_sign": 15.0,
      "kind": "Dual",
      "rising": "HeadFirst",
      "lord": {
        "graha": "Mercury",
        "sign": "Pisces",
        "house": 10,
        "dignity": "Debilitated",
        "retrograde": false,
        "combust": false
      },
      "kendras": [
        {
          "house": 1,
          "sign": "Gemini",
          "grahas": []
        },
        {
          "house": 4,
          "sign": "Virgo",
          "grahas": []
        },
        {
          "house": 7,
          "sign": "Sagittarius",
          "grahas": []
        },
        {
          "house": 10,
          "sign": "Pisces",
          "grahas": [
            [
              "Mercury",
              "Benefic"
            ]
          ]
        }
      ]
    },
    "focus": {
      "house": 8,
      "sign": "Capricorn",
      "lord": {
        "graha": "Saturn",
        "sign": "Capricorn",
        "house": 8,
        "dignity": "OwnSign",
        "retrograde": false,
        "combust": false
      },
      "same_graha": false,
      "links": [
        "FocusLordAspects"
      ],
      "focus_lord_from_lagna_lord": 11,
      "lagna_lord_from_focus_lord": 3,
      "occupants": [
        [
          "Venus",
          "Benefic"
        ],
        [
          "Saturn",
          "Malefic"
        ]
      ]
    },
    "moon": {
      "sign": "Cancer",
      "house": 2,
      "nakshatra": 8,
      "pada": 3,
      "tithi": 13,
      "paksha": "Shukla",
      "aspected_by": [
        "Venus",
        "Saturn"
      ]
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/prashna/leo-exchange-and-mutual-aspect.json`

````json
{
  "id": "leo-exchange-and-mutual-aspect",
  "source": "hand",
  "rule_ref": "BPHS, burç nitelikleri, graha nitelikleri ve bakışlar bölümleri; prashna derlemelerinin sayı yöntemi (alıntı yapılmaz); ileri modüller spec'i §11",
  "derivation": [
    "Lagna 130°00′ sidereal → Aslan 10°00′.",
    "Aslan: burç türü sabit; doğuş biçimi başla doğan.",
    "Lagna lordu Güneş: Kova 15°00′; lagnadan 7. ev. Dignity: burcun lordu Satürn: doğal ilişki düşman; Satürn Aslan burcunda, Kova'dan 7. burç → geçici düşman → GreatEnemy → GreatEnemy. Retro: hayır. Yanma: Güneş yanmaz.",
    "Kendralar. 1. ev Aslan: Satürn (kötücül); 4. ev Akrep: boş; 7. ev Kova: Güneş (kötücül), Merkür (kötücül); 10. ev Boğa: Jüpiter (iyicil).",
    "Odak evi 7: Aslan'dan 7. burç Kova; lordu Satürn.",
    "Odak lordu Satürn: Aslan 5°00′; lagnadan 1. ev. Dignity: burcun lordu Güneş: doğal ilişki düşman; Güneş Kova burcunda, Aslan'dan 7. burç → geçici düşman → GreatEnemy → GreatEnemy. Retro: hayır. Yanma: Güneş'e uzaklık 170°00′, orb 15° → yanık değil.",
    "Bağ. Güneş Kova burcunda, Satürn Aslan burcunda. Aynı burç: hayır. Değişim (Güneş Satürn'in burcunda ve tersi): evet. Güneş'in tam bakışları (7. burçlar): Aslan → Aslan'a bakar. Satürn'in tam bakışları: Terazi, Kova, Boğa → Kova'a bakar. Sonuç: Exchange, MutualAspect.",
    "Ev sayıları: odak lordu lagna lordundan 7. burçta; lagna lordu odak lordundan 7. burçta.",
    "Odak evindeki grahalar (Kova): Güneş (kötücül), Merkür (kötücül).",
    "Ay 70°00′ → İkizler, lagnadan 11. ev; nakshatra 6. (Ardra), pada 2. Güneş'ten uzanım 115°00′ → tithi 10 (Shukla); Ay büyüyor: iyicil.",
    "Ay'ın burcuna (İkizler) tam bakışla bakanlar: Mars (Yay'dan 7. burç)."
  ],
  "input": {
    "lagna_lon": 130,
    "number": null,
    "focus_house": 7,
    "grahas": {
      "Sun": 315.0,
      "Moon": 70.0,
      "Mars": 250.0,
      "Mercury": 300.0,
      "Jupiter": 40.0,
      "Venus": 350.0,
      "Saturn": 125.0,
      "Rahu": 190.0,
      "Ketu": 10.0
    },
    "retrograde": []
  },
  "expected": {
    "lagna": {
      "sign": "Leo",
      "deg_in_sign": 10.0,
      "kind": "Fixed",
      "rising": "HeadFirst",
      "lord": {
        "graha": "Sun",
        "sign": "Aquarius",
        "house": 7,
        "dignity": "GreatEnemy",
        "retrograde": false,
        "combust": false
      },
      "kendras": [
        {
          "house": 1,
          "sign": "Leo",
          "grahas": [
            [
              "Saturn",
              "Malefic"
            ]
          ]
        },
        {
          "house": 4,
          "sign": "Scorpio",
          "grahas": []
        },
        {
          "house": 7,
          "sign": "Aquarius",
          "grahas": [
            [
              "Sun",
              "Malefic"
            ],
            [
              "Mercury",
              "Malefic"
            ]
          ]
        },
        {
          "house": 10,
          "sign": "Taurus",
          "grahas": [
            [
              "Jupiter",
              "Benefic"
            ]
          ]
        }
      ]
    },
    "focus": {
      "house": 7,
      "sign": "Aquarius",
      "lord": {
        "graha": "Saturn",
        "sign": "Leo",
        "house": 1,
        "dignity": "GreatEnemy",
        "retrograde": false,
        "combust": false
      },
      "same_graha": false,
      "links": [
        "Exchange",
        "MutualAspect"
      ],
      "focus_lord_from_lagna_lord": 7,
      "lagna_lord_from_focus_lord": 7,
      "occupants": [
        [
          "Sun",
          "Malefic"
        ],
        [
          "Mercury",
          "Malefic"
        ]
      ]
    },
    "moon": {
      "sign": "Gemini",
      "house": 11,
      "nakshatra": 6,
      "pada": 2,
      "tithi": 10,
      "paksha": "Shukla",
      "aspected_by": [
        "Mars"
      ]
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/prashna/libra-retrograde-combust-lagna-lord.json`

````json
{
  "id": "libra-retrograde-combust-lagna-lord",
  "source": "hand",
  "rule_ref": "BPHS, burç nitelikleri, graha nitelikleri ve bakışlar bölümleri; prashna derlemelerinin sayı yöntemi (alıntı yapılmaz); ileri modüller spec'i §11",
  "derivation": [
    "Lagna 190°00′ sidereal → Terazi 10°00′.",
    "Terazi: burç türü yürük; doğuş biçimi başla doğan.",
    "Lagna lordu Venüs: Terazi 14°00′; lagnadan 1. ev. Dignity: Terazi 0°–15° moolatrikona aralığı → Moolatrikona. Retro: evet. Yanma: Güneş'e uzaklık 6°00′, orb 8° → yanık.",
    "Kendralar. 1. ev Terazi: Güneş (kötücül), Merkür (iyicil), Venüs (iyicil); 4. ev Oğlak: Mars (kötücül); 7. ev Koç: Jüpiter (iyicil); 10. ev Yengeç: Ay (kötücül).",
    "Odak evi 11: Terazi'dan 11. burç Aslan; lordu Güneş.",
    "Odak lordu Güneş: Terazi 20°00′; lagnadan 1. ev. Dignity: düşüş burcu Terazi → Debilitated. Retro: hayır. Yanma: Güneş yanmaz.",
    "Bağ. Venüs Terazi burcunda, Güneş Terazi burcunda. Aynı burç: evet. Değişim (Venüs Güneş'in burcunda ve tersi): hayır. Venüs'in tam bakışları (7. burçlar): Koç → Terazi'a bakmaz. Güneş'in tam bakışları: Koç → Terazi'a bakmaz. Sonuç: Conjunct.",
    "Ev sayıları: odak lordu lagna lordundan 1. burçta; lagna lordu odak lordundan 1. burçta.",
    "Odak evindeki grahalar (Aslan): Rahu (kötücül).",
    "Ay 95°00′ → Yengeç, lagnadan 10. ev; nakshatra 8. (Pushya), pada 1. Güneş'ten uzanım 255°00′ → tithi 22 (Krishna); Ay küçülüyor: kötücül.",
    "Ay'ın burcuna (Yengeç) tam bakışla bakanlar: Mars (Oğlak'dan 7. burç)."
  ],
  "input": {
    "lagna_lon": 190,
    "number": null,
    "focus_house": 11,
    "grahas": {
      "Sun": 200.0,
      "Moon": 95.0,
      "Mars": 280.0,
      "Mercury": 185.0,
      "Jupiter": 5.0,
      "Venus": 194.0,
      "Saturn": 340.0,
      "Rahu": 130.0,
      "Ketu": 310.0
    },
    "retrograde": [
      "Venus"
    ]
  },
  "expected": {
    "lagna": {
      "sign": "Libra",
      "deg_in_sign": 10.0,
      "kind": "Movable",
      "rising": "HeadFirst",
      "lord": {
        "graha": "Venus",
        "sign": "Libra",
        "house": 1,
        "dignity": "Moolatrikona",
        "retrograde": true,
        "combust": true
      },
      "kendras": [
        {
          "house": 1,
          "sign": "Libra",
          "grahas": [
            [
              "Sun",
              "Malefic"
            ],
            [
              "Mercury",
              "Benefic"
            ],
            [
              "Venus",
              "Benefic"
            ]
          ]
        },
        {
          "house": 4,
          "sign": "Capricorn",
          "grahas": [
            [
              "Mars",
              "Malefic"
            ]
          ]
        },
        {
          "house": 7,
          "sign": "Aries",
          "grahas": [
            [
              "Jupiter",
              "Benefic"
            ]
          ]
        },
        {
          "house": 10,
          "sign": "Cancer",
          "grahas": [
            [
              "Moon",
              "Malefic"
            ]
          ]
        }
      ]
    },
    "focus": {
      "house": 11,
      "sign": "Leo",
      "lord": {
        "graha": "Sun",
        "sign": "Libra",
        "house": 1,
        "dignity": "Debilitated",
        "retrograde": false,
        "combust": false
      },
      "same_graha": false,
      "links": [
        "Conjunct"
      ],
      "focus_lord_from_lagna_lord": 1,
      "lagna_lord_from_focus_lord": 1,
      "occupants": [
        [
          "Rahu",
          "Malefic"
        ]
      ]
    },
    "moon": {
      "sign": "Cancer",
      "house": 10,
      "nakshatra": 8,
      "pada": 1,
      "tithi": 22,
      "paksha": "Krishna",
      "aspected_by": [
        "Mars"
      ]
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/prashna/number-1-aries-lords-together-in-capricorn.json`

````json
{
  "id": "number-1-aries-lords-together-in-capricorn",
  "source": "hand",
  "rule_ref": "BPHS, burç nitelikleri, graha nitelikleri ve bakışlar bölümleri; prashna derlemelerinin sayı yöntemi (alıntı yapılmaz); ileri modüller spec'i §11",
  "derivation": [
    "Sayı 1: lagna = (1 − 1) × 3°20′ + 1°40′ = 1°40′ sidereal → Koç 1°40′ (1. navamsanın ortası).",
    "Koç: burç türü yürük; doğuş biçimi sırtla doğan.",
    "Lagna lordu Mars: Oğlak 10°00′; lagnadan 10. ev. Dignity: yücelme burcu Oğlak → Exalted. Retro: hayır. Yanma: Güneş'e uzaklık 120°00′, orb 17° → yanık değil.",
    "Kendralar. 1. ev Koç: Venüs (iyicil); 4. ev Yengeç: Ay (iyicil); 7. ev Terazi: boş; 10. ev Oğlak: Mars (kötücül), Satürn (kötücül).",
    "Odak evi 10: Koç'dan 10. burç Oğlak; lordu Satürn.",
    "Odak lordu Satürn: Oğlak 20°00′; lagnadan 10. ev. Dignity: kendi burcu Oğlak → OwnSign. Retro: hayır. Yanma: Güneş'e uzaklık 110°00′, orb 15° → yanık değil.",
    "Bağ. Mars Oğlak burcunda, Satürn Oğlak burcunda. Aynı burç: evet. Değişim (Mars Satürn'in burcunda ve tersi): hayır. Mars'in tam bakışları (4, 7, 8. burçlar): Koç, Yengeç, Aslan → Oğlak'a bakmaz. Satürn'in tam bakışları: Balık, Yengeç, Terazi → Oğlak'a bakmaz. Sonuç: Conjunct.",
    "Ev sayıları: odak lordu lagna lordundan 1. burçta; lagna lordu odak lordundan 1. burçta.",
    "Odak evindeki grahalar (Oğlak): Mars (kötücül), Satürn (kötücül).",
    "Ay 100°00′ → Yengeç, lagnadan 4. ev; nakshatra 8. (Pushya), pada 3. Güneş'ten uzanım 60°00′ → tithi 6 (Shukla); Ay büyüyor: iyicil.",
    "Ay'ın burcuna (Yengeç) tam bakışla bakanlar: Mars (Oğlak'dan 7. burç), Satürn (Oğlak'dan 7. burç)."
  ],
  "input": {
    "lagna_lon": null,
    "number": 1,
    "focus_house": 10,
    "grahas": {
      "Sun": 40.0,
      "Moon": 100.0,
      "Mars": 280.0,
      "Mercury": 55.0,
      "Jupiter": 130.0,
      "Venus": 20.0,
      "Saturn": 290.0,
      "Rahu": 160.0,
      "Ketu": 340.0
    },
    "retrograde": []
  },
  "expected": {
    "lagna": {
      "sign": "Aries",
      "deg_in_sign": 1.6666666666666667,
      "kind": "Movable",
      "rising": "BackFirst",
      "lord": {
        "graha": "Mars",
        "sign": "Capricorn",
        "house": 10,
        "dignity": "Exalted",
        "retrograde": false,
        "combust": false
      },
      "kendras": [
        {
          "house": 1,
          "sign": "Aries",
          "grahas": [
            [
              "Venus",
              "Benefic"
            ]
          ]
        },
        {
          "house": 4,
          "sign": "Cancer",
          "grahas": [
            [
              "Moon",
              "Benefic"
            ]
          ]
        },
        {
          "house": 7,
          "sign": "Libra",
          "grahas": []
        },
        {
          "house": 10,
          "sign": "Capricorn",
          "grahas": [
            [
              "Mars",
              "Malefic"
            ],
            [
              "Saturn",
              "Malefic"
            ]
          ]
        }
      ]
    },
    "focus": {
      "house": 10,
      "sign": "Capricorn",
      "lord": {
        "graha": "Saturn",
        "sign": "Capricorn",
        "house": 10,
        "dignity": "OwnSign",
        "retrograde": false,
        "combust": false
      },
      "same_graha": false,
      "links": [
        "Conjunct"
      ],
      "focus_lord_from_lagna_lord": 1,
      "lagna_lord_from_focus_lord": 1,
      "occupants": [
        [
          "Mars",
          "Malefic"
        ],
        [
          "Saturn",
          "Malefic"
        ]
      ]
    },
    "moon": {
      "sign": "Cancer",
      "house": 4,
      "nakshatra": 8,
      "pada": 3,
      "tithi": 6,
      "paksha": "Shukla",
      "aspected_by": [
        "Mars",
        "Saturn"
      ]
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/prashna/number-10-taurus-exchange-with-the-fifth-lord.json`

````json
{
  "id": "number-10-taurus-exchange-with-the-fifth-lord",
  "source": "hand",
  "rule_ref": "BPHS, burç nitelikleri, graha nitelikleri ve bakışlar bölümleri; prashna derlemelerinin sayı yöntemi (alıntı yapılmaz); ileri modüller spec'i §11",
  "derivation": [
    "Sayı 10: lagna = (10 − 1) × 3°20′ + 1°40′ = 31°40′ sidereal → Boğa 1°40′ (10. navamsanın ortası).",
    "Boğa: burç türü sabit; doğuş biçimi sırtla doğan.",
    "Lagna lordu Venüs: Başak 10°00′; lagnadan 5. ev. Dignity: düşüş burcu Başak → Debilitated. Retro: hayır. Yanma: Güneş'e uzaklık 60°00′, orb 10° → yanık değil.",
    "Kendralar. 1. ev Boğa: Merkür (iyicil); 4. ev Aslan: Rahu (kötücül); 7. ev Akrep: Satürn (kötücül); 10. ev Kova: Jüpiter (iyicil), Ketu (kötücül).",
    "Odak evi 5: Boğa'dan 5. burç Başak; lordu Merkür.",
    "Odak lordu Merkür: Boğa 10°00′; lagnadan 1. ev. Dignity: burcun lordu Venüs: doğal ilişki dost; Venüs Başak burcunda, Boğa'dan 5. burç → geçici düşman → Neutral → Neutral. Retro: hayır. Yanma: Güneş'e uzaklık 60°00′, orb 14° → yanık değil.",
    "Bağ. Venüs Başak burcunda, Merkür Boğa burcunda. Aynı burç: hayır. Değişim (Venüs Merkür'in burcunda ve tersi): evet. Venüs'in tam bakışları (7. burçlar): Balık → Boğa'a bakmaz. Merkür'in tam bakışları: Akrep → Başak'a bakmaz. Sonuç: Exchange.",
    "Ev sayıları: odak lordu lagna lordundan 9. burçta; lagna lordu odak lordundan 5. burçta.",
    "Odak evindeki grahalar (Başak): Venüs (iyicil).",
    "Ay 250°00′ → Yay, lagnadan 8. ev; nakshatra 19. (Mula), pada 4. Güneş'ten uzanım 150°00′ → tithi 13 (Shukla); Ay büyüyor: iyicil.",
    "Ay'ın burcuna (Yay) tam bakışla bakanlar: yok."
  ],
  "input": {
    "lagna_lon": null,
    "number": 10,
    "focus_house": 5,
    "grahas": {
      "Sun": 100.0,
      "Moon": 250.0,
      "Mars": 10.0,
      "Mercury": 40.0,
      "Jupiter": 300.0,
      "Venus": 160.0,
      "Saturn": 220.0,
      "Rahu": 130.0,
      "Ketu": 310.0
    },
    "retrograde": []
  },
  "expected": {
    "lagna": {
      "sign": "Taurus",
      "deg_in_sign": 1.6666666666666667,
      "kind": "Fixed",
      "rising": "BackFirst",
      "lord": {
        "graha": "Venus",
        "sign": "Virgo",
        "house": 5,
        "dignity": "Debilitated",
        "retrograde": false,
        "combust": false
      },
      "kendras": [
        {
          "house": 1,
          "sign": "Taurus",
          "grahas": [
            [
              "Mercury",
              "Benefic"
            ]
          ]
        },
        {
          "house": 4,
          "sign": "Leo",
          "grahas": [
            [
              "Rahu",
              "Malefic"
            ]
          ]
        },
        {
          "house": 7,
          "sign": "Scorpio",
          "grahas": [
            [
              "Saturn",
              "Malefic"
            ]
          ]
        },
        {
          "house": 10,
          "sign": "Aquarius",
          "grahas": [
            [
              "Jupiter",
              "Benefic"
            ],
            [
              "Ketu",
              "Malefic"
            ]
          ]
        }
      ]
    },
    "focus": {
      "house": 5,
      "sign": "Virgo",
      "lord": {
        "graha": "Mercury",
        "sign": "Taurus",
        "house": 1,
        "dignity": "Neutral",
        "retrograde": false,
        "combust": false
      },
      "same_graha": false,
      "links": [
        "Exchange"
      ],
      "focus_lord_from_lagna_lord": 9,
      "lagna_lord_from_focus_lord": 5,
      "occupants": [
        [
          "Venus",
          "Benefic"
        ]
      ]
    },
    "moon": {
      "sign": "Sagittarius",
      "house": 8,
      "nakshatra": 19,
      "pada": 4,
      "tithi": 13,
      "paksha": "Shukla",
      "aspected_by": []
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/prashna/number-108-pisces-lagna-lord-aspects-the-fourth-lord.json`

````json
{
  "id": "number-108-pisces-lagna-lord-aspects-the-fourth-lord",
  "source": "hand",
  "rule_ref": "BPHS, burç nitelikleri, graha nitelikleri ve bakışlar bölümleri; prashna derlemelerinin sayı yöntemi (alıntı yapılmaz); ileri modüller spec'i §11",
  "derivation": [
    "Sayı 108: lagna = (108 − 1) × 3°20′ + 1°40′ = 358°20′ sidereal → Balık 28°20′ (108. navamsanın ortası).",
    "Balık: burç türü çift doğalı; doğuş biçimi iki yönlü.",
    "Lagna lordu Jüpiter: Koç 15°00′; lagnadan 2. ev. Dignity: burcun lordu Mars: doğal ilişki dost; Mars Oğlak burcunda, Koç'dan 10. burç → geçici dost → GreatFriend → GreatFriend. Retro: hayır. Yanma: Güneş'e uzaklık 145°00′, orb 11° → yanık değil.",
    "Kendralar. 1. ev Balık: boş; 4. ev İkizler: boş; 7. ev Başak: Güneş (kötücül); 10. ev Yay: boş.",
    "Odak evi 4: Balık'dan 4. burç İkizler; lordu Merkür.",
    "Odak lordu Merkür: Aslan 20°00′; lagnadan 6. ev. Dignity: burcun lordu Güneş: doğal ilişki dost; Güneş Başak burcunda, Aslan'dan 2. burç → geçici dost → GreatFriend → GreatFriend. Retro: hayır. Yanma: Güneş'e uzaklık 20°00′, orb 14° → yanık değil.",
    "Bağ. Jüpiter Koç burcunda, Merkür Aslan burcunda. Aynı burç: hayır. Değişim (Jüpiter Merkür'in burcunda ve tersi): hayır. Jüpiter'in tam bakışları (5, 7, 9. burçlar): Aslan, Terazi, Yay → Aslan'a bakar. Merkür'in tam bakışları: Kova → Koç'a bakmaz. Sonuç: LagnaLordAspects.",
    "Ev sayıları: odak lordu lagna lordundan 5. burçta; lagna lordu odak lordundan 9. burçta.",
    "Odak evindeki grahalar (İkizler): yok.",
    "Ay 50°00′ → Boğa, lagnadan 3. ev; nakshatra 4. (Rohini), pada 4. Güneş'ten uzanım 250°00′ → tithi 21 (Krishna); Ay küçülüyor: kötücül.",
    "Ay'ın burcuna (Boğa) tam bakışla bakanlar: yok."
  ],
  "input": {
    "lagna_lon": null,
    "number": 108,
    "focus_house": 4,
    "grahas": {
      "Sun": 160.0,
      "Moon": 50.0,
      "Mars": 270.0,
      "Mercury": 140.0,
      "Jupiter": 15.0,
      "Venus": 185.0,
      "Saturn": 320.0,
      "Rahu": 100.0,
      "Ketu": 280.0
    },
    "retrograde": []
  },
  "expected": {
    "lagna": {
      "sign": "Pisces",
      "deg_in_sign": 28.333333333333332,
      "kind": "Dual",
      "rising": "BothWays",
      "lord": {
        "graha": "Jupiter",
        "sign": "Aries",
        "house": 2,
        "dignity": "GreatFriend",
        "retrograde": false,
        "combust": false
      },
      "kendras": [
        {
          "house": 1,
          "sign": "Pisces",
          "grahas": []
        },
        {
          "house": 4,
          "sign": "Gemini",
          "grahas": []
        },
        {
          "house": 7,
          "sign": "Virgo",
          "grahas": [
            [
              "Sun",
              "Malefic"
            ]
          ]
        },
        {
          "house": 10,
          "sign": "Sagittarius",
          "grahas": []
        }
      ]
    },
    "focus": {
      "house": 4,
      "sign": "Gemini",
      "lord": {
        "graha": "Mercury",
        "sign": "Leo",
        "house": 6,
        "dignity": "GreatFriend",
        "retrograde": false,
        "combust": false
      },
      "same_graha": false,
      "links": [
        "LagnaLordAspects"
      ],
      "focus_lord_from_lagna_lord": 5,
      "lagna_lord_from_focus_lord": 9,
      "occupants": []
    },
    "moon": {
      "sign": "Taurus",
      "house": 3,
      "nakshatra": 4,
      "pada": 4,
      "tithi": 21,
      "paksha": "Krishna",
      "aspected_by": []
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/prashna/number-54-virgo-mutual-aspect-with-the-seventh-lord.json`

````json
{
  "id": "number-54-virgo-mutual-aspect-with-the-seventh-lord",
  "source": "hand",
  "rule_ref": "BPHS, burç nitelikleri, graha nitelikleri ve bakışlar bölümleri; prashna derlemelerinin sayı yöntemi (alıntı yapılmaz); ileri modüller spec'i §11",
  "derivation": [
    "Sayı 54: lagna = (54 − 1) × 3°20′ + 1°40′ = 178°20′ sidereal → Başak 28°20′ (54. navamsanın ortası).",
    "Başak: burç türü çift doğalı; doğuş biçimi başla doğan.",
    "Lagna lordu Merkür: İkizler 15°00′; lagnadan 10. ev. Dignity: kendi burcu İkizler → OwnSign. Retro: hayır. Yanma: Güneş'e uzaklık 55°00′, orb 14° → yanık değil.",
    "Kendralar. 1. ev Başak: Venüs (iyicil); 4. ev Yay: Jüpiter (iyicil); 7. ev Balık: boş; 10. ev İkizler: Merkür (iyicil).",
    "Odak evi 7: Başak'dan 7. burç Balık; lordu Jüpiter.",
    "Odak lordu Jüpiter: Yay 15°00′; lagnadan 4. ev. Dignity: kendi burcu Yay → OwnSign. Retro: hayır. Yanma: Güneş'e uzaklık 125°00′, orb 11° → yanık değil.",
    "Bağ. Merkür İkizler burcunda, Jüpiter Yay burcunda. Aynı burç: hayır. Değişim (Merkür Jüpiter'in burcunda ve tersi): hayır. Merkür'in tam bakışları (7. burçlar): Yay → Yay'a bakar. Jüpiter'in tam bakışları: Koç, İkizler, Aslan → İkizler'a bakar. Sonuç: MutualAspect.",
    "Ev sayıları: odak lordu lagna lordundan 7. burçta; lagna lordu odak lordundan 7. burçta.",
    "Odak evindeki grahalar (Balık): yok.",
    "Ay 310°00′ → Kova, lagnadan 6. ev; nakshatra 24. (Shatabhisha), pada 2. Güneş'ten uzanım 180°00′ → tithi 16 (Krishna); Ay küçülüyor: kötücül.",
    "Ay'ın burcuna (Kova) tam bakışla bakanlar: Güneş (Aslan'dan 7. burç), Mars (Yengeç'dan 8. burç)."
  ],
  "input": {
    "lagna_lon": null,
    "number": 54,
    "focus_house": 7,
    "grahas": {
      "Sun": 130.0,
      "Moon": 310.0,
      "Mars": 100.0,
      "Mercury": 75.0,
      "Jupiter": 255.0,
      "Venus": 160.0,
      "Saturn": 20.0,
      "Rahu": 200.0,
      "Ketu": 20.0
    },
    "retrograde": []
  },
  "expected": {
    "lagna": {
      "sign": "Virgo",
      "deg_in_sign": 28.333333333333332,
      "kind": "Dual",
      "rising": "HeadFirst",
      "lord": {
        "graha": "Mercury",
        "sign": "Gemini",
        "house": 10,
        "dignity": "OwnSign",
        "retrograde": false,
        "combust": false
      },
      "kendras": [
        {
          "house": 1,
          "sign": "Virgo",
          "grahas": [
            [
              "Venus",
              "Benefic"
            ]
          ]
        },
        {
          "house": 4,
          "sign": "Sagittarius",
          "grahas": [
            [
              "Jupiter",
              "Benefic"
            ]
          ]
        },
        {
          "house": 7,
          "sign": "Pisces",
          "grahas": []
        },
        {
          "house": 10,
          "sign": "Gemini",
          "grahas": [
            [
              "Mercury",
              "Benefic"
            ]
          ]
        }
      ]
    },
    "focus": {
      "house": 7,
      "sign": "Pisces",
      "lord": {
        "graha": "Jupiter",
        "sign": "Sagittarius",
        "house": 4,
        "dignity": "OwnSign",
        "retrograde": false,
        "combust": false
      },
      "same_graha": false,
      "links": [
        "MutualAspect"
      ],
      "focus_lord_from_lagna_lord": 7,
      "lagna_lord_from_focus_lord": 7,
      "occupants": []
    },
    "moon": {
      "sign": "Aquarius",
      "house": 6,
      "nakshatra": 24,
      "pada": 2,
      "tithi": 16,
      "paksha": "Krishna",
      "aspected_by": [
        "Sun",
        "Mars"
      ]
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/prashna/number-9-aries-eighth-house-same-lord.json`

````json
{
  "id": "number-9-aries-eighth-house-same-lord",
  "source": "hand",
  "rule_ref": "BPHS, burç nitelikleri, graha nitelikleri ve bakışlar bölümleri; prashna derlemelerinin sayı yöntemi (alıntı yapılmaz); ileri modüller spec'i §11",
  "derivation": [
    "Sayı 9: lagna = (9 − 1) × 3°20′ + 1°40′ = 28°20′ sidereal → Koç 28°20′ (9. navamsanın ortası).",
    "Koç: burç türü yürük; doğuş biçimi sırtla doğan.",
    "Lagna lordu Mars: Akrep 5°00′; lagnadan 8. ev. Dignity: kendi burcu Akrep → OwnSign. Retro: hayır. Yanma: Güneş'e uzaklık 95°00′, orb 17° → yanık değil.",
    "Kendralar. 1. ev Koç: Ay (iyicil); 4. ev Yengeç: Jüpiter (iyicil); 7. ev Terazi: Satürn (kötücül); 10. ev Oğlak: boş.",
    "Odak evi 8: Koç'dan 8. burç Akrep; lordu Mars.",
    "Odak lordu Mars: Akrep 5°00′; lagnadan 8. ev. Dignity: kendi burcu Akrep → OwnSign. Retro: hayır. Yanma: Güneş'e uzaklık 95°00′, orb 17° → yanık değil.",
    "İki evin lordu aynı grahadır: bağ aranmaz.",
    "Ev sayıları: odak lordu lagna lordundan 1. burçta; lagna lordu odak lordundan 1. burçta.",
    "Odak evindeki grahalar (Akrep): Mars (kötücül).",
    "Ay 20°00′ → Koç, lagnadan 1. ev; nakshatra 2. (Bharani), pada 3. Güneş'ten uzanım 70°00′ → tithi 6 (Shukla); Ay büyüyor: iyicil.",
    "Ay'ın burcuna (Koç) tam bakışla bakanlar: Satürn (Terazi'dan 7. burç)."
  ],
  "input": {
    "lagna_lon": null,
    "number": 9,
    "focus_house": 8,
    "grahas": {
      "Sun": 310.0,
      "Moon": 20.0,
      "Mars": 215.0,
      "Mercury": 325.0,
      "Jupiter": 95.0,
      "Venus": 350.0,
      "Saturn": 190.0,
      "Rahu": 70.0,
      "Ketu": 250.0
    },
    "retrograde": []
  },
  "expected": {
    "lagna": {
      "sign": "Aries",
      "deg_in_sign": 28.333333333333332,
      "kind": "Movable",
      "rising": "BackFirst",
      "lord": {
        "graha": "Mars",
        "sign": "Scorpio",
        "house": 8,
        "dignity": "OwnSign",
        "retrograde": false,
        "combust": false
      },
      "kendras": [
        {
          "house": 1,
          "sign": "Aries",
          "grahas": [
            [
              "Moon",
              "Benefic"
            ]
          ]
        },
        {
          "house": 4,
          "sign": "Cancer",
          "grahas": [
            [
              "Jupiter",
              "Benefic"
            ]
          ]
        },
        {
          "house": 7,
          "sign": "Libra",
          "grahas": [
            [
              "Saturn",
              "Malefic"
            ]
          ]
        },
        {
          "house": 10,
          "sign": "Capricorn",
          "grahas": []
        }
      ]
    },
    "focus": {
      "house": 8,
      "sign": "Scorpio",
      "lord": {
        "graha": "Mars",
        "sign": "Scorpio",
        "house": 8,
        "dignity": "OwnSign",
        "retrograde": false,
        "combust": false
      },
      "same_graha": true,
      "links": [],
      "focus_lord_from_lagna_lord": 1,
      "lagna_lord_from_focus_lord": 1,
      "occupants": [
        [
          "Mars",
          "Malefic"
        ]
      ]
    },
    "moon": {
      "sign": "Aries",
      "house": 1,
      "nakshatra": 2,
      "pada": 3,
      "tithi": 6,
      "paksha": "Shukla",
      "aspected_by": [
        "Saturn"
      ]
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/prashna/sagittarius-mercury-with-saturn-in-a-kendra.json`

````json
{
  "id": "sagittarius-mercury-with-saturn-in-a-kendra",
  "source": "hand",
  "rule_ref": "BPHS, burç nitelikleri, graha nitelikleri ve bakışlar bölümleri; prashna derlemelerinin sayı yöntemi (alıntı yapılmaz); ileri modüller spec'i §11",
  "derivation": [
    "Lagna 250°00′ sidereal → Yay 10°00′.",
    "Yay: burç türü çift doğalı; doğuş biçimi sırtla doğan.",
    "Lagna lordu Jüpiter: Yengeç 5°00′; lagnadan 8. ev. Dignity: yücelme burcu Yengeç → Exalted. Retro: hayır. Yanma: Güneş'e uzaklık 145°00′, orb 11° → yanık değil.",
    "Kendralar. 1. ev Yay: boş; 4. ev Balık: Merkür (kötücül), Satürn (kötücül); 7. ev İkizler: Mars (kötücül); 10. ev Başak: boş.",
    "Odak evi 6: Yay'dan 6. burç Boğa; lordu Venüs.",
    "Odak lordu Venüs: Aslan 5°00′; lagnadan 9. ev. Dignity: burcun lordu Güneş: doğal ilişki düşman; Güneş Kova burcunda, Aslan'dan 7. burç → geçici düşman → GreatEnemy → GreatEnemy. Retro: hayır. Yanma: Güneş'e uzaklık 175°00′, orb 10° → yanık değil.",
    "Bağ. Jüpiter Yengeç burcunda, Venüs Aslan burcunda. Aynı burç: hayır. Değişim (Jüpiter Venüs'in burcunda ve tersi): hayır. Jüpiter'in tam bakışları (5, 7, 9. burçlar): Akrep, Oğlak, Balık → Aslan'a bakmaz. Venüs'in tam bakışları: Kova → Yengeç'a bakmaz. Sonuç: hiçbiri.",
    "Ev sayıları: odak lordu lagna lordundan 2. burçta; lagna lordu odak lordundan 12. burçta.",
    "Odak evindeki grahalar (Boğa): yok.",
    "Ay 190°00′ → Terazi, lagnadan 11. ev; nakshatra 15. (Swati), pada 2. Güneş'ten uzanım 240°00′ → tithi 21 (Krishna); Ay küçülüyor: kötücül.",
    "Ay'ın burcuna (Terazi) tam bakışla bakanlar: yok."
  ],
  "input": {
    "lagna_lon": 250,
    "number": null,
    "focus_house": 6,
    "grahas": {
      "Sun": 310.0,
      "Moon": 190.0,
      "Mars": 70.0,
      "Mercury": 335.0,
      "Jupiter": 95.0,
      "Venus": 125.0,
      "Saturn": 340.0,
      "Rahu": 20.0,
      "Ketu": 200.0
    },
    "retrograde": []
  },
  "expected": {
    "lagna": {
      "sign": "Sagittarius",
      "deg_in_sign": 10.0,
      "kind": "Dual",
      "rising": "BackFirst",
      "lord": {
        "graha": "Jupiter",
        "sign": "Cancer",
        "house": 8,
        "dignity": "Exalted",
        "retrograde": false,
        "combust": false
      },
      "kendras": [
        {
          "house": 1,
          "sign": "Sagittarius",
          "grahas": []
        },
        {
          "house": 4,
          "sign": "Pisces",
          "grahas": [
            [
              "Mercury",
              "Malefic"
            ],
            [
              "Saturn",
              "Malefic"
            ]
          ]
        },
        {
          "house": 7,
          "sign": "Gemini",
          "grahas": [
            [
              "Mars",
              "Malefic"
            ]
          ]
        },
        {
          "house": 10,
          "sign": "Virgo",
          "grahas": []
        }
      ]
    },
    "focus": {
      "house": 6,
      "sign": "Taurus",
      "lord": {
        "graha": "Venus",
        "sign": "Leo",
        "house": 9,
        "dignity": "GreatEnemy",
        "retrograde": false,
        "combust": false
      },
      "same_graha": false,
      "links": [],
      "focus_lord_from_lagna_lord": 2,
      "lagna_lord_from_focus_lord": 12,
      "occupants": []
    },
    "moon": {
      "sign": "Libra",
      "house": 11,
      "nakshatra": 15,
      "pada": 2,
      "tithi": 21,
      "paksha": "Krishna",
      "aspected_by": []
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/prashna/scorpio-no-focus-house.json`

````json
{
  "id": "scorpio-no-focus-house",
  "source": "hand",
  "rule_ref": "BPHS, burç nitelikleri, graha nitelikleri ve bakışlar bölümleri; prashna derlemelerinin sayı yöntemi (alıntı yapılmaz); ileri modüller spec'i §11",
  "derivation": [
    "Lagna 220°00′ sidereal → Akrep 10°00′.",
    "Akrep: burç türü sabit; doğuş biçimi başla doğan.",
    "Lagna lordu Mars: Kova 0°00′; lagnadan 4. ev. Dignity: burcun lordu Satürn: doğal ilişki nötr; Satürn Yengeç burcunda, Kova'dan 6. burç → geçici düşman → Enemy → Enemy. Retro: hayır. Yanma: Güneş'e uzaklık 75°00′, orb 17° → yanık değil.",
    "Kendralar. 1. ev Akrep: Jüpiter (iyicil); 4. ev Kova: Mars (kötücül); 7. ev Boğa: Merkür (iyicil), Venüs (iyicil); 10. ev Aslan: Ay (iyicil).",
    "Odak evi seçilmedi: odak bölümü boştur.",
    "Ay 135°00′ → Aslan, lagnadan 10. ev; nakshatra 11. (Purva Phalguni), pada 1. Güneş'ten uzanım 120°00′ → tithi 11 (Shukla); Ay büyüyor: iyicil.",
    "Ay'ın burcuna (Aslan) tam bakışla bakanlar: Mars (Kova'dan 7. burç)."
  ],
  "input": {
    "lagna_lon": 220,
    "number": null,
    "focus_house": null,
    "grahas": {
      "Sun": 15.0,
      "Moon": 135.0,
      "Mars": 300.0,
      "Mercury": 30.0,
      "Jupiter": 225.0,
      "Venus": 50.0,
      "Saturn": 100.0,
      "Rahu": 250.0,
      "Ketu": 70.0
    },
    "retrograde": []
  },
  "expected": {
    "lagna": {
      "sign": "Scorpio",
      "deg_in_sign": 10.0,
      "kind": "Fixed",
      "rising": "HeadFirst",
      "lord": {
        "graha": "Mars",
        "sign": "Aquarius",
        "house": 4,
        "dignity": "Enemy",
        "retrograde": false,
        "combust": false
      },
      "kendras": [
        {
          "house": 1,
          "sign": "Scorpio",
          "grahas": [
            [
              "Jupiter",
              "Benefic"
            ]
          ]
        },
        {
          "house": 4,
          "sign": "Aquarius",
          "grahas": [
            [
              "Mars",
              "Malefic"
            ]
          ]
        },
        {
          "house": 7,
          "sign": "Taurus",
          "grahas": [
            [
              "Mercury",
              "Benefic"
            ],
            [
              "Venus",
              "Benefic"
            ]
          ]
        },
        {
          "house": 10,
          "sign": "Leo",
          "grahas": [
            [
              "Moon",
              "Benefic"
            ]
          ]
        }
      ]
    },
    "focus": null,
    "moon": {
      "sign": "Leo",
      "house": 10,
      "nakshatra": 11,
      "pada": 1,
      "tithi": 11,
      "paksha": "Shukla",
      "aspected_by": [
        "Mars"
      ]
    }
  }
}
````

**Yeni dosya:** `tests/golden/modules/prashna/time-unknown-moon-only.json`

````json
{
  "id": "time-unknown-moon-only",
  "source": "hand",
  "rule_ref": "BPHS, burç nitelikleri, graha nitelikleri ve bakışlar bölümleri; prashna derlemelerinin sayı yöntemi (alıntı yapılmaz); ileri modüller spec'i §11",
  "derivation": [
    "Lagna yok (saat bilinmiyor): lagnaya bağlı bölümler boştur.",
    "Ay 40°00′ → Boğa; nakshatra 4. (Rohini), pada 1. Güneş'ten uzanım 300°00′ → tithi 26 (Krishna); Ay küçülüyor: kötücül.",
    "Ay'ın burcuna (Boğa) tam bakışla bakanlar: yok."
  ],
  "input": {
    "lagna_lon": null,
    "number": null,
    "focus_house": 5,
    "grahas": {
      "Sun": 100.0,
      "Moon": 40.0,
      "Mars": 10.0,
      "Mercury": 115.0,
      "Jupiter": 250.0,
      "Venus": 80.0,
      "Saturn": 300.0,
      "Rahu": 200.0,
      "Ketu": 20.0
    },
    "retrograde": []
  },
  "expected": {
    "lagna": null,
    "focus": null,
    "moon": {
      "sign": "Taurus",
      "house": null,
      "nakshatra": 4,
      "pada": 1,
      "tithi": 26,
      "paksha": "Krishna",
      "aspected_by": []
    }
  }
}
````

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
cargo test -p jyotish-core --test modules_golden 2>&1 | grep -E "^error" | head -4
```
Expected: `error[E0432]: unresolved imports `jyotish_core::prashna_facts`, `jyotish_core::rising_mode`, `jyotish_core::LordLink`, `jyotish_core::RisingMode`, `jyotish_core::PRASHNA_METHOD`` ve `cannot find type `LordFacts`` ile `cannot find type `Occupant``.

- [ ] **Step 3: Uygulama**

`facts.rs` olguları kurar (testleri dosyanın sonundadır; önce onları okuyun: doğuş biçiminin elle yazılmış ikinci tablosu, bir örnek haritanın bütün olguları, beş bağın sentetik konumları, retro ve yanık lord, lagnasız harita, tam dolunay sınırı, alan adlı hatalar ve toplayıcı).

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 72c0809..8a25641 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -92,7 +92,11 @@ pub use panchang::{
     END_BISECTIONS, END_SEARCH_DAYS, KARANA_SPAN_DEG, SUNRISE_WINDOW_DAYS, TITHI_SPAN_DEG,
 };
 pub use positions::{compute_positions, ChartInput, GrahaPoint, Positions};
-pub use prashna::{number_lagna_lon, PRASHNA_NUMBER_MAX};
+pub use prashna::{
+    compute_prashna_facts, lord_relation, moon_facts, number_lagna_lon, prashna_facts, rising_mode,
+    FocusFacts, HouseOccupants, LagnaFacts, LordFacts, LordLink, LordRelation, MoonFacts, Occupant,
+    PrashnaFacts, RisingMode, PRASHNA_METHOD, PRASHNA_NUMBER_MAX,
+};
 pub use settings::{
     AdvancedSettings, AyanamsaSystem, CalcSettings, CombustionOrbs, HouseSystem, KarakaScheme,
     NodeKind, SunriseDef, ZodiacMode, CALC_SETTINGS_SCHEMA_VERSION, JD_UT_RANGE,
diff --git a/crates/jyotish-core/src/prashna/mod.rs b/crates/jyotish-core/src/prashna/mod.rs
index 8e98b37..3317d3b 100644
--- a/crates/jyotish-core/src/prashna/mod.rs
+++ b/crates/jyotish-core/src/prashna/mod.rs
@@ -4,6 +4,15 @@
 //!
 //! Bu dosya sayı prashnasının lagnasını verir: grahalar anın grahalarıdır, lagna 1–108
 //! arasındaki sayıdan gelir (`number_lagna_lon`). Harita `compute_chart_with_lagna` ile kurulur.
+//! Haritanın olguları `facts`'tedir.
+
+mod facts;
+
+pub use facts::{
+    compute_prashna_facts, lord_relation, moon_facts, prashna_facts, rising_mode, FocusFacts,
+    HouseOccupants, LagnaFacts, LordFacts, LordLink, LordRelation, MoonFacts, Occupant,
+    PrashnaFacts, RisingMode, PRASHNA_METHOD,
+};
 
 use crate::CoreError;
 
````

**Yeni dosya:** `crates/jyotish-core/src/prashna/facts.rs`

````rust
//! Soru haritasının olguları (ileri modüller spec'i §11 "Olgular"). Saf hesap: ephemeris
//! görmez, `Positions` okur (K3). Hüküm, konu çıkarımı ve evet/hayır sonucu yoktur; her alan
//! haritadan okunan bir olgudur. Evler lagnadan sayılan burç evleridir (K4); burçlar rasi
//! çerçevesindedir, nakshatra ve tithi sidereal boylamdandır (K5).

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::chart::{compute_chart_for, ChartWarning};
use crate::combustion::is_combust;
use crate::dignity::{dignity, sign_lord, Dignity};
use crate::drishti::{graha_aspect_signs, ASPECTING_GRAHAS};
use crate::graha::Graha;
use crate::jaimini::{sign_kind, SignKind};
use crate::lordship::{house_lord, house_sign};
use crate::nature::{nature, Nature};
use crate::panchang::{elongation, paksha, tithi_no, Paksha};
use crate::positions::{ChartInput, Positions};
use crate::settings::CombustionOrbs;
use crate::yogas::{is_exchange, is_mutual_aspect, yogas, YogaReport, KENDRA_HOUSES};
use crate::zodiac::{nakshatra_pada, Sign};
use crate::CoreError;

/// Yöntem kimliği (K7): Parashari motorunun olguları; ayrı bir prashna ekolü uygulanmaz.
pub const PRASHNA_METHOD: &str = "prashna.parashari_v1";

/// Burcun doğuş biçimi (BPHS, burç nitelikleri bölümü): başla doğan (shirshodaya), sırtla
/// doğan (prishthodaya), iki yönlü (ubhayodaya).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum RisingMode {
    HeadFirst,
    BackFirst,
    BothWays,
}

impl RisingMode {
    pub const ALL: [RisingMode; 3] = [
        RisingMode::HeadFirst,
        RisingMode::BackFirst,
        RisingMode::BothWays,
    ];
}

/// Başla doğan: İkizler, Aslan, Başak, Terazi, Akrep, Kova. Sırtla doğan: Koç, Boğa, Yengeç,
/// Yay, Oğlak. İki yönlü: Balık.
pub fn rising_mode(sign: Sign) -> RisingMode {
    match sign {
        Sign::Gemini | Sign::Leo | Sign::Virgo | Sign::Libra | Sign::Scorpio | Sign::Aquarius => {
            RisingMode::HeadFirst
        }
        Sign::Aries | Sign::Taurus | Sign::Cancer | Sign::Sagittarius | Sign::Capricorn => {
            RisingMode::BackFirst
        }
        Sign::Pisces => RisingMode::BothWays,
    }
}

/// Bir ev lordunun haritadaki durumu.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct LordFacts {
    pub graha: Graha,
    pub sign: Sign,
    /// Lagnadan sayılan burç evi, 1..=12.
    pub house: u8,
    pub dignity: Option<Dignity>,
    pub retrograde: bool,
    pub combust: bool,
}

/// Lagna lordu ile odak evinin lordu arasındaki bağ. Birden çoğu birlikte geçerli olabilir
/// (ör. değişim ve karşılıklı bakış); liste bu enum'un sırasındadır.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum LordLink {
    /// Birlikte: aynı burçtalar.
    Conjunct,
    /// Değişim: her biri diğerinin burcunda.
    Exchange,
    /// Karşılıklı bakış: her biri diğerinin burcuna tam bakışla bakar.
    MutualAspect,
    /// Tek yönlü bakış: yalnızca lagna lordu odak lordunun burcuna bakar.
    LagnaLordAspects,
    /// Tek yönlü bakış: yalnızca odak lordu lagna lordunun burcuna bakar.
    FocusLordAspects,
}

impl LordLink {
    pub const ALL: [LordLink; 5] = [
        LordLink::Conjunct,
        LordLink::Exchange,
        LordLink::MutualAspect,
        LordLink::LagnaLordAspects,
        LordLink::FocusLordAspects,
    ];
}

/// İki lordun ilişkisi (spec §11: birlikte, karşılıklı bakış, tek yönlü bakış, değişim,
/// hiçbiri) ve birbirinden ev sayıları.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct LordRelation {
    /// İki ev aynı grahanındır (ör. odak evi 1; Koç lagnasında 8. ev): bağ aranmaz.
    pub same_graha: bool,
    /// Geçerli bağlar, `LordLink::ALL` sırasında; boş liste "hiçbiri"dir.
    pub links: Vec<LordLink>,
    /// Odak lordunun burcunun, lagna lordunun burcundan sayısı (1..=12).
    pub focus_lord_from_lagna_lord: u8,
    /// Lagna lordunun burcunun, odak lordunun burcundan sayısı (1..=12).
    pub lagna_lord_from_focus_lord: u8,
}

/// Bir evdeki graha ve doğal sınıfı (K6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Occupant {
    pub graha: Graha,
    pub nature: Nature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct HouseOccupants {
    pub house: u8,
    pub sign: Sign,
    /// `Graha::ALL` sırasında.
    pub grahas: Vec<Occupant>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct LagnaFacts {
    pub sign: Sign,
    /// Burç türü: yürük, sabit, çift doğalı.
    pub kind: SignKind,
    pub rising: RisingMode,
    pub lord: LordFacts,
    /// Dört kendra (1, 4, 7, 10), ev sırasıyla; boş evler de listededir.
    pub kendras: Vec<HouseOccupants>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct FocusFacts {
    /// Astrologun seçtiği ev, 1..=12.
    pub house: u8,
    pub sign: Sign,
    pub lord: LordFacts,
    pub relation: LordRelation,
    /// Odak evindeki grahalar, `Graha::ALL` sırasında.
    pub occupants: Vec<Occupant>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MoonFacts {
    pub sign: Sign,
    /// Lagnadan sayılan burç evi; lagna yoksa `None`.
    pub house: Option<u8>,
    pub nakshatra: u8,
    pub pada: u8,
    /// 1..=30.
    pub tithi: u8,
    pub paksha: Paksha,
    /// Ay'ın burcuna tam bakışla bakan grahalar (burç temelli graha drishti),
    /// `ASPECTING_GRAHAS` sırasında.
    pub aspected_by: Vec<Graha>,
}

/// `compute_prashna_facts` sonucu. Lagnaya bağlı iki bölüm (`lagna`, `focus`) lagna yoksa
/// `None`'dır (K2); `focus` odak evi seçilmemişse de `None`'dır.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PrashnaFacts {
    pub lagna: Option<LagnaFacts>,
    pub focus: Option<FocusFacts>,
    pub moon: MoonFacts,
    /// Soru haritasının yoga listesi (Alt proje 3.5'in kataloğu).
    pub yogas: YogaReport,
    /// Temel haritanın konum duyarlılığı uyarısı (`LowPrecision`).
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

fn lord_facts(
    graha: Graha,
    lagna_sign: Sign,
    positions: &Positions,
    orbs: &CombustionOrbs,
) -> LordFacts {
    let p = positions.point(graha);
    let sign = p.rasi_sign;
    let retrograde = p.speed < 0.0;
    LordFacts {
        graha,
        sign,
        house: sign.house_from(lagna_sign),
        dignity: dignity(
            graha,
            sign,
            p.deg_in_sign,
            positions.sign_of(sign_lord(sign)),
        ),
        retrograde,
        // Fark iki çerçevede de aynıdır: sidereal boylamlar yeter.
        combust: is_combust(
            graha,
            p.sidereal_lon,
            positions.point(Graha::Sun).sidereal_lon,
            retrograde,
            orbs,
        ),
    }
}

/// Lagna lordu (`lagna_lord`) ile odak lordunun (`focus_lord`) ilişkisi.
pub fn lord_relation(lagna_lord: Graha, focus_lord: Graha, positions: &Positions) -> LordRelation {
    let (a, b) = (positions.sign_of(lagna_lord), positions.sign_of(focus_lord));
    let mut links = Vec::new();
    if lagna_lord != focus_lord {
        if a == b {
            links.push(LordLink::Conjunct);
        }
        if is_exchange(lagna_lord, focus_lord, positions) {
            links.push(LordLink::Exchange);
        }
        let forward = graha_aspect_signs(lagna_lord, a).contains(&b);
        let backward = graha_aspect_signs(focus_lord, b).contains(&a);
        debug_assert_eq!(
            forward && backward,
            is_mutual_aspect(lagna_lord, focus_lord, positions)
        );
        match (forward, backward) {
            (true, true) => links.push(LordLink::MutualAspect),
            (true, false) => links.push(LordLink::LagnaLordAspects),
            (false, true) => links.push(LordLink::FocusLordAspects),
            (false, false) => {}
        }
    }
    LordRelation {
        same_graha: lagna_lord == focus_lord,
        links,
        focus_lord_from_lagna_lord: b.house_from(a),
        lagna_lord_from_focus_lord: a.house_from(b),
    }
}

fn occupants(sign: Sign, positions: &Positions) -> Vec<Occupant> {
    positions
        .occupants(sign)
        .into_iter()
        .map(|graha| Occupant {
            graha,
            nature: nature(graha, positions),
        })
        .collect()
}

/// Ay'ın olguları. Tithi ve nakshatra sidereal boylamdan, burç rasi çerçevesindendir.
pub fn moon_facts(positions: &Positions) -> MoonFacts {
    let moon = positions.point(Graha::Moon);
    let sun = positions.point(Graha::Sun);
    let (nakshatra, pada) = nakshatra_pada(moon.sidereal_lon);
    let tithi = tithi_no(elongation(sun.sidereal_lon, moon.sidereal_lon));
    MoonFacts {
        sign: moon.rasi_sign,
        house: positions.house_of(Graha::Moon),
        nakshatra,
        pada,
        tithi,
        paksha: paksha(tithi),
        aspected_by: ASPECTING_GRAHAS
            .into_iter()
            .filter(|&g| {
                g != Graha::Moon
                    && graha_aspect_signs(g, positions.sign_of(g)).contains(&moon.rasi_sign)
            })
            .collect(),
    }
}

/// Saf hesap: konumlardan prashna olguları. `focus_house` astrologun seçtiği evdir (1..=12;
/// değilse `InvalidPrashnaInput("focus_house")`); modül konu çıkarmaz. Lagna yoksa `lagna` ve
/// `focus` boştur; Ay ve yoga listesi yine verilir.
pub fn prashna_facts(
    positions: &Positions,
    orbs: &CombustionOrbs,
    focus_house: Option<u8>,
) -> Result<PrashnaFacts, CoreError> {
    if focus_house.is_some_and(|h| !(1..=12).contains(&h)) {
        return Err(CoreError::InvalidPrashnaInput("focus_house"));
    }
    let lagna = positions.lagna_sign.map(|sign| LagnaFacts {
        sign,
        kind: sign_kind(sign),
        rising: rising_mode(sign),
        lord: lord_facts(house_lord(sign, 1), sign, positions, orbs),
        kendras: KENDRA_HOUSES
            .into_iter()
            .map(|house| {
                let of_house = house_sign(sign, house);
                HouseOccupants {
                    house,
                    sign: of_house,
                    grahas: occupants(of_house, positions),
                }
            })
            .collect(),
    });
    let focus = match (&lagna, focus_house) {
        (Some(l), Some(house)) => {
            let sign = house_sign(l.sign, house);
            let lord = lord_facts(house_lord(l.sign, house), l.sign, positions, orbs);
            Some(FocusFacts {
                house,
                sign,
                relation: lord_relation(l.lord.graha, lord.graha, positions),
                lord,
                occupants: occupants(sign, positions),
            })
        }
        _ => None,
    };
    Ok(PrashnaFacts {
        lagna,
        focus,
        moon: moon_facts(positions),
        yogas: yogas(positions),
        warnings: Vec::new(),
        method: PRASHNA_METHOD.to_owned(),
    })
}

/// `compute_prashna_facts` komutunun hesabı: temel harita (sayı prashnasında sayının
/// lagnasıyla: `input.lagna_number`), konumlar, olgular. Odak evi ephemeris sorulmadan önce
/// denetlenir. Ayar, an ve konum denetimi `compute_chart`'ınkidir. `input.time_known` yanlışsa
/// lagnaya bağlı bölümler boştur.
pub fn compute_prashna_facts<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
    focus_house: Option<u8>,
) -> Result<PrashnaFacts, CoreError> {
    if focus_house.is_some_and(|h| !(1..=12).contains(&h)) {
        return Err(CoreError::InvalidPrashnaInput("focus_house"));
    }
    let chart = compute_chart_for(eph, input)?;
    let positions = Positions::from(&chart);
    let positions = if input.time_known {
        positions
    } else {
        positions.without_lagna()
    };
    let mut facts = prashna_facts(&positions, &input.settings.combustion_orbs, focus_house)?;
    facts.warnings = chart
        .meta
        .warnings
        .iter()
        .copied()
        .filter(|w| *w == ChartWarning::LowPrecision)
        .collect();
    Ok(facts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::settings::{CalcSettings, ZodiacMode};

    /// Elle yazılmış ikinci tablo (§13 katman 3), `Sign::ALL` sırasında.
    const HAND_RISING: [RisingMode; 12] = [
        RisingMode::BackFirst, // Koç
        RisingMode::BackFirst, // Boğa
        RisingMode::HeadFirst, // İkizler
        RisingMode::BackFirst, // Yengeç
        RisingMode::HeadFirst, // Aslan
        RisingMode::HeadFirst, // Başak
        RisingMode::HeadFirst, // Terazi
        RisingMode::HeadFirst, // Akrep
        RisingMode::BackFirst, // Yay
        RisingMode::BackFirst, // Oğlak
        RisingMode::HeadFirst, // Kova
        RisingMode::BothWays,  // Balık
    ];

    fn orbs() -> CombustionOrbs {
        CombustionOrbs::default()
    }

    /// Koç lagnası; Güneş Koç 10°, Ay İkizler 5°, Mars Oğlak 10°, Merkür Koç 25°,
    /// Jüpiter Yengeç 5°, Venüs Balık 20°, Satürn Terazi 10°, Rahu Boğa, Ketu Akrep.
    fn sample() -> Positions {
        Positions::synthetic(
            Some(15.0),
            [10.0, 65.0, 280.0, 25.0, 95.0, 350.0, 190.0, 40.0, 220.0],
        )
    }

    #[test]
    fn rising_modes_match_the_hand_table() {
        for (sign, want) in Sign::ALL.into_iter().zip(HAND_RISING) {
            assert_eq!(rising_mode(sign), want, "{sign:?}");
        }
        let count = |m| HAND_RISING.iter().filter(|&&x| x == m).count();
        assert_eq!(
            RisingMode::ALL.map(count),
            [6, 5, 1],
            "six head-first, five back-first, one both ways"
        );
    }

    #[test]
    fn facts_describe_the_lagna_its_lord_the_kendras_and_the_moon() {
        let p = sample();
        let f = prashna_facts(&p, &orbs(), None).unwrap();
        assert_eq!(f.method, PRASHNA_METHOD);
        assert!(f.focus.is_none() && f.warnings.is_empty());
        assert_eq!(f.yogas, yogas(&p));
        let l = f.lagna.unwrap();
        assert_eq!(
            (l.sign, l.kind, l.rising),
            (Sign::Aries, SignKind::Movable, RisingMode::BackFirst)
        );
        assert_eq!(
            l.lord,
            LordFacts {
                graha: Graha::Mars,
                sign: Sign::Capricorn,
                house: 10,
                dignity: Some(Dignity::Exalted),
                retrograde: false,
                combust: false,
            }
        );
        let houses: Vec<u8> = l.kendras.iter().map(|k| k.house).collect();
        assert_eq!(houses, [1, 4, 7, 10]);
        let signs: Vec<Sign> = l.kendras.iter().map(|k| k.sign).collect();
        assert_eq!(
            signs,
            [Sign::Aries, Sign::Cancer, Sign::Libra, Sign::Capricorn]
        );
        // Merkür Güneş'le (kötücül) aynı burçta, iyicil yok: kötücül (K6).
        assert_eq!(
            l.kendras[0].grahas,
            [
                Occupant {
                    graha: Graha::Sun,
                    nature: Nature::Malefic
                },
                Occupant {
                    graha: Graha::Mercury,
                    nature: Nature::Malefic
                },
            ]
        );
        assert_eq!(
            l.kendras[1].grahas,
            [Occupant {
                graha: Graha::Jupiter,
                nature: Nature::Benefic
            }]
        );
        // Ay İkizler 5°: Mrigashira'nın 4. padası; uzanım 55° → 5. tithi.
        assert_eq!(
            f.moon,
            MoonFacts {
                sign: Sign::Gemini,
                house: Some(3),
                nakshatra: 5,
                pada: 4,
                tithi: 5,
                paksha: Paksha::Shukla,
                aspected_by: vec![],
            }
        );
    }

    #[test]
    fn the_focus_house_names_its_lord_and_the_relation() {
        let p = sample();
        // 7. ev Terazi; lordu Venüs Balık'ta (yücelmiş), 12. evde. Mars Oğlak'tan 4, 7, 8.
        // burçlara bakar: Koç, Yengeç, Aslan → Balık'a bakmaz. Venüs Balık'tan Başak'a bakar.
        let f = prashna_facts(&p, &orbs(), Some(7)).unwrap().focus.unwrap();
        assert_eq!((f.house, f.sign), (7, Sign::Libra));
        assert_eq!(
            (f.lord.graha, f.lord.sign, f.lord.house, f.lord.dignity),
            (Graha::Venus, Sign::Pisces, 12, Some(Dignity::Exalted))
        );
        assert_eq!(
            f.relation,
            LordRelation {
                same_graha: false,
                links: vec![],
                focus_lord_from_lagna_lord: 3,
                lagna_lord_from_focus_lord: 11,
            }
        );
        assert_eq!(
            f.occupants,
            [Occupant {
                graha: Graha::Saturn,
                nature: Nature::Malefic
            }]
        );
        // 1. ve 8. ev aynı grahanındır.
        for house in [1, 8] {
            let f = prashna_facts(&p, &orbs(), Some(house))
                .unwrap()
                .focus
                .unwrap();
            assert!(f.relation.same_graha && f.relation.links.is_empty());
            assert_eq!(
                (
                    f.relation.focus_lord_from_lagna_lord,
                    f.relation.lagna_lord_from_focus_lord
                ),
                (1, 1)
            );
        }
    }

    #[test]
    fn the_five_relations_between_two_lords() {
        // Mars (a) ile Venüs (b); diğer grahalar önemsizdir.
        let at = |mars: f64, venus: f64| {
            let p = Positions::synthetic(
                Some(15.0),
                [100.0, 65.0, mars, 130.0, 95.0, venus, 190.0, 40.0, 220.0],
            );
            lord_relation(Graha::Mars, Graha::Venus, &p).links
        };
        // Birlikte: ikisi de Yay'da.
        assert_eq!(at(250.0, 260.0), [LordLink::Conjunct]);
        // Değişim ve karşılıklı bakış: Mars Terazi'de, Venüs Koç'ta.
        assert_eq!(
            at(190.0, 10.0),
            [LordLink::Exchange, LordLink::MutualAspect]
        );
        // Yalnızca değişim: Mars Boğa'da, Venüs Koç'ta.
        assert_eq!(at(40.0, 10.0), [LordLink::Exchange]);
        // Yalnızca karşılıklı bakış: Mars İkizler'de, Venüs Yay'da.
        assert_eq!(at(70.0, 250.0), [LordLink::MutualAspect]);
        // Tek yönlü: Mars İkizler'den 4. burca (Başak) bakar; Venüs Başak'tan Balık'a bakar.
        assert_eq!(at(70.0, 160.0), [LordLink::LagnaLordAspects]);
        // Ters yön: roller değişince bakan odak lordudur.
        let p = Positions::synthetic(
            Some(15.0),
            [100.0, 65.0, 70.0, 130.0, 95.0, 160.0, 190.0, 40.0, 220.0],
        );
        assert_eq!(
            lord_relation(Graha::Venus, Graha::Mars, &p).links,
            [LordLink::FocusLordAspects]
        );
        // Hiçbiri: Mars İkizler'de, Venüs Aslan'da.
        assert!(at(70.0, 130.0).is_empty());
    }

    #[test]
    fn retrograde_and_combust_lords_are_marked() {
        let mut p = Positions::synthetic(
            Some(190.0),
            [200.0, 95.0, 280.0, 185.0, 5.0, 194.0, 340.0, 130.0, 310.0],
        );
        p.grahas[Graha::Venus as usize].speed = -0.3;
        let l = prashna_facts(&p, &orbs(), None).unwrap().lagna.unwrap();
        // Venüs Terazi 14°: moolatrikona; Güneş'e 6°, retro orbu 8° → yanık.
        assert_eq!(
            (
                l.lord.graha,
                l.lord.dignity,
                l.lord.retrograde,
                l.lord.combust
            ),
            (Graha::Venus, Some(Dignity::Moolatrikona), true, true)
        );
        // Orb ayardan gelir.
        let tight = CombustionOrbs {
            venus_retro: 5.0,
            ..orbs()
        };
        assert!(
            !prashna_facts(&p, &tight, None)
                .unwrap()
                .lagna
                .unwrap()
                .lord
                .combust
        );
    }

    #[test]
    fn without_a_lagna_only_the_moon_and_the_yogas_remain() {
        let p = sample().without_lagna();
        let f = prashna_facts(&p, &orbs(), Some(7)).unwrap();
        assert!(f.lagna.is_none() && f.focus.is_none());
        assert_eq!(f.moon.house, None);
        assert_eq!((f.moon.sign, f.moon.tithi), (Sign::Gemini, 5));
        assert_eq!(f.yogas, yogas(&p));
    }

    #[test]
    fn the_moon_takes_sign_aspects_and_a_full_moon_is_the_sixteenth_tithi() {
        // Güneş Koç 5°, Ay Terazi 5°: uzanım tam 180° → 16. tithi, Krishna (K6 sınırı).
        // Güneş karşıdan bakar; Satürn Koç'tan 7. burca, Jüpiter Kova'dan 9. burca bakar.
        let p = Positions::synthetic(
            Some(280.0),
            [5.0, 185.0, 220.0, 65.0, 310.0, 40.0, 15.0, 100.0, 280.0],
        );
        let m = moon_facts(&p);
        assert_eq!((m.tithi, m.paksha), (16, Paksha::Krishna));
        assert_eq!(m.aspected_by, [Graha::Sun, Graha::Jupiter, Graha::Saturn]);
        assert_eq!(m.house, Some(10));
    }

    #[test]
    fn a_focus_house_outside_one_to_twelve_is_rejected_by_field_name() {
        let p = sample();
        for bad in [0, 13, 255] {
            assert_eq!(
                prashna_facts(&p, &orbs(), Some(bad)),
                Err(CoreError::InvalidPrashnaInput("focus_house"))
            );
        }
    }

    fn input(settings: CalcSettings) -> ChartInput {
        ChartInput {
            jd_ut: 2_451_545.0,
            lat: 41.0,
            lon: 29.0,
            settings,
            time_known: true,
            lagna_number: None,
        }
    }

    #[test]
    fn collector_follows_the_number_the_time_and_the_rasi_frame() {
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        // FakeEph: lagna tropikal 70°, ayanamsa 24° → sidereal 46° (Boğa).
        let time = compute_prashna_facts(&mut eph, &input(s.clone()), Some(4)).unwrap();
        assert_eq!(time.lagna.as_ref().unwrap().sign, Sign::Taurus);
        assert_eq!(time.focus.as_ref().unwrap().sign, Sign::Leo);
        // Sayı 30: Yengeç 8°20′; grahalar aynı an, yalnızca lagna ve evler değişir.
        let mut numbered = input(s.clone());
        numbered.lagna_number = Some(30);
        let number = compute_prashna_facts(&mut eph, &numbered, Some(4)).unwrap();
        assert_eq!(number.lagna.as_ref().unwrap().sign, Sign::Cancer);
        assert_eq!(number.focus.as_ref().unwrap().sign, Sign::Libra);
        assert_eq!(
            (number.moon.sign, number.moon.nakshatra, number.moon.tithi),
            (time.moon.sign, time.moon.nakshatra, time.moon.tithi)
        );
        assert_ne!(number.moon.house, time.moon.house);
        // Saat bilinmiyorsa lagnaya bağlı bölümler boştur (K2).
        let mut unknown = input(s);
        unknown.time_known = false;
        let u = compute_prashna_facts(&mut eph, &unknown, Some(4)).unwrap();
        assert!(u.lagna.is_none() && u.focus.is_none() && u.moon.house.is_none());
        // Tropikal-rasi kipinde lagna burcu tropikaldir (İkizler); nakshatra sidereal.
        let tropical = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..CalcSettings::default()
        };
        let t = compute_prashna_facts(&mut eph, &input(tropical), None).unwrap();
        assert_eq!(t.lagna.unwrap().sign, Sign::Gemini);
        assert_eq!(t.moon.nakshatra, time.moon.nakshatra);
    }

    #[test]
    fn collector_checks_the_focus_house_and_the_number_before_the_chart() {
        let mut eph = FakeEph::default();
        let mut bad = input(CalcSettings::default());
        bad.lat = 91.0;
        // Odak evi ve sayı konumdan önce denetlenir: hata yalnızca alan adı taşır.
        assert_eq!(
            compute_prashna_facts(&mut eph, &bad, Some(13)),
            Err(CoreError::InvalidPrashnaInput("focus_house"))
        );
        bad.lagna_number = Some(109);
        assert_eq!(
            compute_prashna_facts(&mut eph, &bad, Some(3)),
            Err(CoreError::InvalidPrashnaInput("number"))
        );
        bad.lagna_number = None;
        assert_eq!(
            compute_prashna_facts(&mut eph, &bad, Some(3)),
            Err(CoreError::InvalidLocation {
                lat: 91.0,
                lon: 29.0
            })
        );
    }
}
````

- [ ] **Step 4: Testleri ve kapıları koş**

```bash
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p jyotish-core --lib 2>&1 | grep -E "^test result"
cargo test -p jyotish-core --test modules_golden 2>&1 | grep -E "^test result"
for i in 1 2 3; do PROPTEST_CASES=4000 cargo test -p jyotish-core --lib prashna_properties 2>&1 | grep -E "^test result"; done
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
grep -rn '"source"' tests/golden/modules/prashna | grep -vc '"source": "hand"'
ls tests/golden/modules/prashna/*.json | wc -l
ls crates/jyotish-core/proptest-regressions 2>/dev/null
```
Expected: fmt ve clippy çıktısız; lib `374 passed`; `modules_golden` `17 passed`; üç koşuda `3 passed`; sayaçlar `0` ve `0`; `14`; `ls` çıktısız. **Bir el örneği düşerse** önce `REVIEW.md`'deki okuma kuralı ve örneğin `derivation` satırları yeniden okunur; hata motordaysa motor düzeltilir. Beklenen değer motor çıktısından kopyalanmaz; örnek yanlışsa kuraldan yeniden türetilir ve bu ilerleme notlarına yazılır.

- [ ] **Step 5: Commit**

```bash
git add -A
git status --short
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): prashna facts (lagna, lords and their relation, the moon, kendras, yogas); fourteen hand fixtures" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: on altı yeni dosya (`facts.rs`, `REVIEW.md`, on dört JSON); `grep` çıktısız.

---

### Task 3: Store: prashna yan tablosu (şema sürümü 2), kayıt ve güncelleme, yerleşik danışan, dışa ve içe aktarım

Spec §11 "Saklama" ve "Test" (store), "Gizlilik"; §14; D11–D14. Projenin ilk şema göçü: göç 1 değişmez, liste bir göç kazanır. Hesap yoktur.

**Files:**
- Create: `crates/store/src/prashna.rs`
- Modify: `crates/store/src/schema.rs`, `crates/store/src/db.rs`, `crates/store/src/model.rs`, `crates/store/src/open.rs`, `crates/store/src/compat.rs`, `crates/store/src/exchange.rs`, `crates/store/src/lib.rs`
- Test: `prashna.rs`'in `mod tests`'i (8), `schema.rs` (1); `db.rs`'in tablo sayımı

**Interfaces:**
- Consumes: `Store`, `resolve_birth`, `charts::{clean_name, insert_chart, insert_client, ChartRow}`, `empty_chart_view_json`, `jyotish_core::PRASHNA_NUMBER_MAX`.
- Produces:
  - `store::{PrashnaMeta { question, number, focus_house, created_at }, NewPrashna { client_id, birth, number, question, focus_house }, MAX_QUESTION_CHARS, MAX_PRASHNA_NUMBER, PRASHNA_CLIENT_NAME}`; `SCHEMA_VERSION` 2.
  - `Store::create_prashna(&mut self, &NewPrashna, label: &str) -> Result<(u32, ChartRecord), StoreError>`, `Store::prashna_meta(&self, chart_id) -> Result<Option<PrashnaMeta>, StoreError>`, `Store::update_prashna(&mut self, chart_id, question: &str, focus_house: Option<u8>) -> Result<PrashnaMeta, StoreError>`, `Store::prashna_client_id(&self) -> Result<Option<u32>, StoreError>`.
  - `OpenRecord.prashna: Option<PrashnaMeta>`; `Store::chart_input` sayıyı `lagna_number`'a yazar; `ExportChart.prashna: Option<ExportPrashna>`.
  - `StoreError::InvalidInput` yeni alanları: `prashna.question`, `prashna.number`, `prashna.focus_house`; `NotFound { what: "prashna" }`; `ImportRecord { field: "prashna" }`.

- [ ] **Step 1: Testler: göç ve tablo sayımı**

`schema.rs` testi yalnızca göç 1'le kurulmuş bir veritabanını (bir danışan ve bir haritayla) son sürüme çıkarır: sürüm 2 olur, harita durur, hiçbir harita prashna değildir; CHECK kısıtları, yabancı anahtar ve silmeyle birlikte satırın silinmesi sınanır. `db.rs`'in tablo sayımı dokuza çıkar. Öteki testler `prashna.rs`'le birlikte gelir (Step 3).

````diff
diff --git a/crates/store/src/db.rs b/crates/store/src/db.rs
index 1a78dec..f0f5162 100644
--- a/crates/store/src/db.rs
+++ b/crates/store/src/db.rs
@@ -197,12 +197,12 @@ mod tests {
             .query_row(
                 "SELECT count(*) FROM sqlite_master WHERE name IN
                  ('clients', 'charts', 'client_notes', 'tags', 'chart_tags', 'app_state',
-                  'clients_fts', 'notes_fts')",
+                  'clients_fts', 'notes_fts', 'prashna')",
                 [],
                 |r| r.get(0),
             )
             .unwrap();
-        assert_eq!(tables, 8);
+        assert_eq!(tables, 9);
     }
 
     #[test]
diff --git a/crates/store/src/schema.rs b/crates/store/src/schema.rs
index 889c5fa..53275ce 100644
--- a/crates/store/src/schema.rs
+++ b/crates/store/src/schema.rs
@@ -122,4 +122,61 @@ mod tests {
         assert_eq!(MIGRATIONS.validate(), Ok(()));
         assert_eq!(MIGRATION_LIST.len(), SCHEMA_VERSION);
     }
+
+    /// Sürüm 1 veritabanı (yalnızca ilk göç) açılışta sürüm 2'ye çıkar: var olan haritalar
+    /// durur, hiçbiri prashna değildir ve yan tablo kısıtlarıyla gelir.
+    #[test]
+    fn a_version_one_database_gains_the_prashna_table() {
+        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
+        conn.pragma_update(None, "foreign_keys", true).unwrap();
+        Migrations::from_slice(&MIGRATION_LIST[..1])
+            .to_latest(&mut conn)
+            .unwrap();
+        let version = |c: &rusqlite::Connection| -> i64 {
+            c.query_row("PRAGMA user_version", [], |r| r.get(0))
+                .unwrap()
+        };
+        assert_eq!(version(&conn), 1);
+        conn.execute_batch(
+            "INSERT INTO clients (id, uuid, name, created_at, updated_at)
+             VALUES (1, 'c', 'Deneme', 't', 't');
+             INSERT INTO charts (id, uuid, client_id, label, kind, date, time_certainty,
+                place_name, country, admin1, lat, lon, tz_kind, birth_notes, resolved_utc,
+                resolved_jd_ut, resolved_offset_secs, resolved_abbrev, resolved_warnings_json,
+                tzdb_version, calc_settings_json, view_state_json, created_at, updated_at)
+             VALUES (7, 'h', 1, '', 'Event', '2000-01-01', 'Exact', '', '', '', 0, 0, 'Lmt', '',
+                'u', 2451545.0, 0, '', '[]', 'v', '{}', '{}', 't', 't');",
+        )
+        .unwrap();
+        MIGRATIONS.to_latest(&mut conn).unwrap();
+        assert_eq!(version(&conn), 2);
+        let count = |c: &rusqlite::Connection, sql: &str| -> i64 {
+            c.query_row(sql, [], |r| r.get(0)).unwrap()
+        };
+        assert_eq!(count(&conn, "SELECT count(*) FROM charts"), 1);
+        assert_eq!(count(&conn, "SELECT count(*) FROM prashna"), 0);
+        // Kısıtlar: sayı 1..=108, odak evi 1..=12, harita var olmalı.
+        let insert = |c: &rusqlite::Connection, values: &str| {
+            c.execute(
+                &format!(
+                    "INSERT INTO prashna (chart_id, number, focus_house, created_at) VALUES {values}"
+                ),
+                [],
+            )
+        };
+        assert!(insert(&conn, "(7, 109, NULL, 't')").is_err());
+        assert!(insert(&conn, "(7, 0, NULL, 't')").is_err());
+        assert!(insert(&conn, "(7, NULL, 13, 't')").is_err());
+        assert!(insert(&conn, "(8, NULL, NULL, 't')").is_err());
+        assert_eq!(insert(&conn, "(7, 108, 12, 't')"), Ok(1));
+        assert_eq!(
+            conn.query_row("SELECT question FROM prashna", [], |r| r
+                .get::<_, String>(0))
+                .unwrap(),
+            ""
+        );
+        // Harita silinince satır da silinir.
+        conn.execute("DELETE FROM charts WHERE id = 7", []).unwrap();
+        assert_eq!(count(&conn, "SELECT count(*) FROM prashna"), 0);
+    }
 }
````

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
cargo test -p store --lib 2>&1 | grep -E "FAILED|^test result"
```
Expected: `db::tests::fresh_database_has_schema_and_foreign_keys ... FAILED`, `schema::tests::a_version_one_database_gains_the_prashna_table ... FAILED`; `102 passed; 2 failed; 1 ignored`.

- [ ] **Step 3: Uygulama**

`schema.rs` göç 2'yi kazanır. `prashna.rs` kaydı, okumayı, güncellemeyi ve yerleşik danışanı verir (testleri dosyanın sonundadır; önce onları okuyun: yan satır; yerleşik danışanın adından değil kaydından tanınması; gidiş-dönüş; güncelleme; alan adlı hatalar ve hiçbir şeyin yazılmaması; silme; arama ve dışa aktarım; içe aktarım). `exchange.rs` alanı dışa ve içe aktarır; `compat.rs` sayıyı eş haritanın girdisine yazar.

````diff
diff --git a/crates/store/src/compat.rs b/crates/store/src/compat.rs
index ea73dd3..34f38ff 100644
--- a/crates/store/src/compat.rs
+++ b/crates/store/src/compat.rs
@@ -111,7 +111,8 @@ impl Store {
             lon: chart.birth.place.lon,
             settings,
             time_known: chart.birth.time_certainty != TimeCertainty::Unknown,
-            lagna_number: None,
+            // Sayı prashnası eş harita olarak da sayının lagnasıyla hesaplanır.
+            lagna_number: crate::prashna::load_prashna(&self.conn, id)?.and_then(|p| p.number),
         }))
     }
 
diff --git a/crates/store/src/exchange.rs b/crates/store/src/exchange.rs
index 0762aaf..c844227 100644
--- a/crates/store/src/exchange.rs
+++ b/crates/store/src/exchange.rs
@@ -11,7 +11,10 @@ use uuid::Uuid;
 use crate::birth::{parse_birth, resolve_parsed, ParsedBirth};
 use crate::charts::{clean_name, insert_chart, insert_client, ChartRow};
 use crate::db::format_ts;
-use crate::model::{BirthData, ChartKind, Gender, ResolvedRecord, TzChoice};
+use crate::model::{BirthData, ChartKind, Gender, PrashnaMeta, ResolvedRecord, TzChoice};
+use crate::prashna::{
+    check_focus_house, check_number, clean_question, insert_prashna, load_prashna,
+};
 use crate::settings::{load_calc_value, SettingsLoad};
 use crate::view_state::{
     canonical_uuid, chart_view_json, parse_chara_path, parse_compat, parse_dasha_path, ViewState,
@@ -66,6 +69,19 @@ pub struct ExportChart {
     pub tags: Vec<String>,
     pub created_at: String,
     pub updated_at: String,
+    /// Harita prashnaysa yan kaydı; değilse alan hiç yazılmaz (eski dosyalarda da yoktur).
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub prashna: Option<ExportPrashna>,
+}
+
+/// Prashna yan kaydının dosyadaki biçimi (ileri modüller spec'i §11). Soru metni not gövdesi
+/// gibi hassastır: yalnızca "notları dahil et" işaretliyse yazılır, değilse `null`'dır.
+#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
+pub struct ExportPrashna {
+    pub number: Option<u8>,
+    pub focus_house: Option<u8>,
+    pub question: Option<String>,
+    pub created_at: String,
 }
 
 #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
@@ -244,6 +260,12 @@ impl Store {
                     tags: ch.tags.clone(),
                     created_at: ch.created_at.clone(),
                     updated_at: ch.updated_at.clone(),
+                    prashna: load_prashna(&self.conn, ch.id)?.map(|p| ExportPrashna {
+                        number: p.number,
+                        focus_house: p.focus_house,
+                        question: opts.include_notes.then_some(p.question),
+                        created_at: p.created_at,
+                    }),
                 });
             }
             let notes = if opts.include_notes {
@@ -416,6 +438,23 @@ impl Store {
                 for t in crate::notes::normalize_tags(&ch.tags).map_err(|_| record("tags"))? {
                     crate::notes::attach_tag(&tx, chart_id, &t)?;
                 }
+                if let Some(p) = &ch.prashna {
+                    // Geçersiz sayı, odak evi ya da soru kaydın UUID'siyle reddedilir.
+                    check_number(p.number).map_err(|_| record("prashna"))?;
+                    check_focus_house(p.focus_house).map_err(|_| record("prashna"))?;
+                    let question = clean_question(p.question.as_deref().unwrap_or(""))
+                        .map_err(|_| record("prashna"))?;
+                    insert_prashna(
+                        &tx,
+                        chart_id,
+                        &PrashnaMeta {
+                            question,
+                            number: p.number,
+                            focus_house: p.focus_house,
+                            created_at: ts_or(&p.created_at, &now),
+                        },
+                    )?;
+                }
                 first_new_chart.get_or_insert(chart_id);
                 report.charts_added += 1;
             }
diff --git a/crates/store/src/lib.rs b/crates/store/src/lib.rs
index 868c7ca..3f6ee94 100644
--- a/crates/store/src/lib.rs
+++ b/crates/store/src/lib.rs
@@ -15,6 +15,7 @@ pub mod model;
 mod muhurta;
 mod notes;
 mod open;
+mod prashna;
 pub mod schema;
 mod search;
 pub mod settings;
@@ -33,11 +34,13 @@ pub use exchange::{
 pub use jhd::{parse_jhd, JhdChart, JhdImport};
 pub use model::{
     BirthData, ChartKind, ChartRecord, Client, FoldChoice, Gender, HomePlace, MomentWarning,
-    NewChart, NewClient, Note, Place, ResolvedRecord, TimeCertainty, TzChoice,
+    NewChart, NewClient, NewPrashna, Note, Place, PrashnaMeta, ResolvedRecord, TimeCertainty,
+    TzChoice,
 };
 pub use muhurta::{SavedMuhurtaFilter, MAX_MUHURTA_FILTERS, MUHURTA_FILTERS_SCHEMA_VERSION};
 pub use notes::{normalize_tags, TagCount, MAX_NOTE_BYTES, MAX_TAG_CHARS, RECENT_NOTES};
 pub use open::{OpenRecord, StoreWarning};
+pub use prashna::{MAX_PRASHNA_NUMBER, MAX_QUESTION_CHARS, PRASHNA_CLIENT_NAME};
 pub use schema::SCHEMA_VERSION;
 pub use search::{SearchHit, SearchMatch};
 pub use settings::{
diff --git a/crates/store/src/model.rs b/crates/store/src/model.rs
index 8e6cfd3..d0c6c5a 100644
--- a/crates/store/src/model.rs
+++ b/crates/store/src/model.rs
@@ -148,6 +148,32 @@ pub struct Note {
     pub updated_at: String,
 }
 
+/// Bir prashna haritasının yan kaydı (ileri modüller spec'i §11). Soru metni not gövdesi gibi
+/// hassastır: loglanmaz, hata iletisine girmez.
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+#[cfg_attr(feature = "specta", derive(specta::Type))]
+pub struct PrashnaMeta {
+    /// En çok `MAX_QUESTION_CHARS` karakter; boş olabilir.
+    pub question: String,
+    /// Sayı prashnası: lagna bu sayıdan gelir (1..=108). Zaman prashnasında `None`.
+    pub number: Option<u8>,
+    /// Astrologun seçtiği ev (1..=12); seçilmemişse `None`.
+    pub focus_house: Option<u8>,
+    pub created_at: String,
+}
+
+/// Yeni prashna (store girdisi; IPC'yi geçmez: komut anı ve yeri çözüp bunu kurar).
+/// `client_id` boşsa prashna yerleşik "Prashna" danışanına eklenir.
+#[derive(Debug, Clone, PartialEq)]
+pub struct NewPrashna {
+    pub client_id: Option<u32>,
+    /// Soru anı ve yeri; `time_certainty` `Exact` olmalıdır.
+    pub birth: BirthData,
+    pub number: Option<u8>,
+    pub question: String,
+    pub focus_house: Option<u8>,
+}
+
 #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
 #[cfg_attr(feature = "specta", derive(specta::Type))]
 pub struct NewChart {
diff --git a/crates/store/src/open.rs b/crates/store/src/open.rs
index fcb23db..e3a3e62 100644
--- a/crates/store/src/open.rs
+++ b/crates/store/src/open.rs
@@ -7,7 +7,7 @@ use rusqlite::{params, Connection, OptionalExtension};
 use serde::{Deserialize, Serialize};
 
 use crate::birth::{parse_birth, resolve_parsed};
-use crate::model::{ChartRecord, Client, Note, TzChoice};
+use crate::model::{ChartRecord, Client, Note, PrashnaMeta, TzChoice};
 use crate::notes::RECENT_NOTES;
 use crate::settings::SettingsLoad;
 use crate::view_state::ViewState;
@@ -41,6 +41,8 @@ pub struct OpenRecord {
     pub view_state: ViewState,
     /// En yeni önce, en çok `RECENT_NOTES`.
     pub recent_notes: Vec<Note>,
+    /// Harita prashnaysa yan kaydı (ileri modüller spec'i §11); değilse `None`.
+    pub prashna: Option<PrashnaMeta>,
     pub warnings: Vec<StoreWarning>,
 }
 
@@ -76,6 +78,7 @@ impl Store {
         let load = self.calc_settings(chart_id)?;
         let view_state = self.view_state(chart_id)?;
         let recent_notes = self.notes(client_id, RECENT_NOTES)?;
+        let prashna = self.prashna_meta(chart_id)?;
 
         // Ayar göçü, tzdb sürümü ve "son açılan" işaretleri tek işlemde yazılır: biri
         // başarısız olursa hiçbiri kalmaz (Plan 08 ertelenen madde 9).
@@ -101,6 +104,7 @@ impl Store {
             calc_settings,
             view_state,
             recent_notes,
+            prashna,
             warnings,
         })
     }
diff --git a/crates/store/src/schema.rs b/crates/store/src/schema.rs
index 53275ce..33159b0 100644
--- a/crates/store/src/schema.rs
+++ b/crates/store/src/schema.rs
@@ -1,5 +1,6 @@
 //! SQLite şeması ve göçler (spec §5). Sürüm `PRAGMA user_version`'da tutulur
 //! (`rusqlite_migration`). Göçler yalnızca eklenir; yayımlanmış bir göç değiştirilmez.
+//! Sürüm 2 (Plan 21) `prashna` yan tablosunu ekler.
 
 use rusqlite_migration::{Migrations, M};
 
@@ -106,10 +107,24 @@ CREATE TRIGGER notes_au AFTER UPDATE OF body ON client_notes BEGIN
 END;
 ";
 
-const MIGRATION_LIST: &[M<'static>] = &[M::up(V1)];
+/// Göç 2: prashna yan tablosu (ileri modüller spec'i §11). Bir harita, bu tabloda satırı varsa
+/// prashnadır; `charts.kind` ve CHECK kısıtı değişmez (kısıtı değiştirmek tabloyu yeniden kurmayı
+/// gerektirirdi). Soru metni haritadan ayrı durur ve arama dizinine girmez. Harita silinince
+/// satır da silinir.
+const V2: &str = "
+CREATE TABLE prashna (
+    chart_id    INTEGER PRIMARY KEY REFERENCES charts(id) ON DELETE CASCADE,
+    question    TEXT NOT NULL DEFAULT '',
+    number      INTEGER CHECK (number IS NULL OR number BETWEEN 1 AND 108),
+    focus_house INTEGER CHECK (focus_house IS NULL OR focus_house BETWEEN 1 AND 12),
+    created_at  TEXT NOT NULL
+);
+";
+
+const MIGRATION_LIST: &[M<'static>] = &[M::up(V1), M::up(V2)];
 
 /// Bu sürümün şema sürümü (`MIGRATION_LIST.len()`).
-pub const SCHEMA_VERSION: usize = 1;
+pub const SCHEMA_VERSION: usize = 2;
 
 pub(crate) const MIGRATIONS: Migrations<'static> = Migrations::from_slice(MIGRATION_LIST);
 
````

**Yeni dosya:** `crates/store/src/prashna.rs`

````rust
//! Prashna kayıtları (ileri modüller spec'i §11 "Saklama"). Prashna haritası danışana bağlı bir
//! `Event` haritasıdır; `prashna` yan tablosunda satırı olan harita prashnadır. Yan tablo soru
//! metnini, isteğe bağlı sayıyı (sayı prashnası: lagna sayıdan gelir) ve astrologun seçtiği
//! odak evini tutar.
//!
//! Soru metni not gövdesi gibi hassastır: loglanmaz, hata iletisine girmez, arama dizinine
//! eklenmez ve dışa aktarıma yalnızca notlar dahil edilirse yazılır. Bu dosyanın ürettiği
//! hatalar yalnızca alan adı taşır.

use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::birth::resolve_birth;
use crate::charts::{clean_name, insert_chart, insert_client, ChartRow};
use crate::model::{ChartKind, ChartRecord, NewPrashna, PrashnaMeta};
use crate::view_state::empty_chart_view_json;
use crate::{Store, StoreError};

/// Soru metninin en büyük uzunluğu, karakter (spec §11).
pub const MAX_QUESTION_CHARS: usize = 2000;
/// Sayı prashnasının en büyük sayısı (`jyotish_core::PRASHNA_NUMBER_MAX` ile aynı).
pub const MAX_PRASHNA_NUMBER: u8 = jyotish_core::PRASHNA_NUMBER_MAX;
/// Danışansız prashnaların toplandığı yerleşik danışanın adı. Klasik terimdir: üç dilde aynı
/// yazılır (K11). Kullanıcı adı değiştirebilir; danışan adından değil, `app_state`'teki
/// UUID'sinden tanınır.
pub const PRASHNA_CLIENT_NAME: &str = "Prashna";
/// `app_state` anahtarı: yerleşik prashna danışanının UUID'si (JSON dizgi).
const PRASHNA_CLIENT_KEY: &str = "prashna_client_uuid";

/// Soru metnini denetler ve kırpar (baştaki ve sondaki boşluk atılır).
pub(crate) fn clean_question(question: &str) -> Result<String, StoreError> {
    let q = question.trim();
    if q.chars().count() > MAX_QUESTION_CHARS {
        return Err(StoreError::InvalidInput("prashna.question"));
    }
    Ok(q.to_owned())
}

pub(crate) fn check_number(number: Option<u8>) -> Result<(), StoreError> {
    match number {
        Some(n) if !(1..=MAX_PRASHNA_NUMBER).contains(&n) => {
            Err(StoreError::InvalidInput("prashna.number"))
        }
        _ => Ok(()),
    }
}

pub(crate) fn check_focus_house(house: Option<u8>) -> Result<(), StoreError> {
    match house {
        Some(h) if !(1..=12).contains(&h) => Err(StoreError::InvalidInput("prashna.focus_house")),
        _ => Ok(()),
    }
}

/// Yan tablo satırını yazar (yeni harita ve içe aktarma ortak kullanır).
pub(crate) fn insert_prashna(
    conn: &Connection,
    chart_id: u32,
    meta: &PrashnaMeta,
) -> Result<(), StoreError> {
    conn.execute(
        "INSERT INTO prashna (chart_id, question, number, focus_house, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            chart_id,
            meta.question,
            meta.number,
            meta.focus_house,
            meta.created_at
        ],
    )?;
    Ok(())
}

/// Haritanın prashna kaydı; harita prashna değilse `None`.
pub(crate) fn load_prashna(
    conn: &Connection,
    chart_id: u32,
) -> Result<Option<PrashnaMeta>, StoreError> {
    Ok(conn
        .query_row(
            "SELECT question, number, focus_house, created_at FROM prashna WHERE chart_id = ?1",
            params![chart_id],
            |r| {
                Ok(PrashnaMeta {
                    question: r.get(0)?,
                    number: r.get(1)?,
                    focus_house: r.get(2)?,
                    created_at: r.get(3)?,
                })
            },
        )
        .optional()?)
}

impl Store {
    /// Yerleşik prashna danışanının kimliği; hiç oluşturulmadıysa ya da silindiyse `None`.
    pub fn prashna_client_id(&self) -> Result<Option<u32>, StoreError> {
        let Some(json) = self.app_state_get(PRASHNA_CLIENT_KEY)? else {
            return Ok(None);
        };
        // Okunamayan kayıt "yok" sayılır: sonraki prashna yeni bir yerleşik danışan açar.
        let Ok(uuid) = serde_json::from_str::<String>(&json) else {
            return Ok(None);
        };
        Ok(self
            .conn
            .query_row(
                "SELECT id FROM clients WHERE uuid = ?1",
                params![uuid],
                |r| r.get(0),
            )
            .optional()?)
    }

    /// Prashna haritasını kurar: `Event` türünde bir harita ve yan tablo satırı, tek işlemde.
    /// `input.client_id` verilmişse o danışana, verilmemişse yerleşik "Prashna" danışanına
    /// eklenir (yoksa aynı işlemde oluşturulur). Harita, genel varsayılan ayarların tam
    /// kopyasını alır. Dönen çift danışanın kimliği ve yeni haritadır.
    pub fn create_prashna(
        &mut self,
        input: &NewPrashna,
        label: &str,
    ) -> Result<(u32, ChartRecord), StoreError> {
        check_number(input.number)?;
        check_focus_house(input.focus_house)?;
        let question = clean_question(&input.question)?;
        let label = clean_name(label, "label")?;
        if let Some(id) = input.client_id {
            self.client(id)?;
        }
        let (birth, resolved) = resolve_birth(&input.birth)?;
        let calc_json = serde_json::to_string(&self.app_prefs()?.calc_settings)?;
        let book = match input.client_id {
            Some(_) => None,
            None => self.prashna_client_id()?,
        };
        let now = self.now();
        let tx = self.conn.transaction()?;
        let (client_id, created) = match input.client_id.or(book) {
            Some(id) => (id, false),
            None => {
                let uuid = Uuid::new_v4().to_string();
                let id = insert_client(&tx, &uuid, PRASHNA_CLIENT_NAME, None, &now, &now)?;
                tx.execute(
                    "INSERT INTO app_state (key, value_json) VALUES (?1, ?2)
                     ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
                    params![PRASHNA_CLIENT_KEY, serde_json::to_string(&uuid)?],
                )?;
                (id, true)
            }
        };
        let chart_id = insert_chart(
            &tx,
            &ChartRow {
                uuid: &Uuid::new_v4().to_string(),
                client_id,
                label: &label,
                kind: ChartKind::Event,
                birth: &birth,
                resolved: &resolved,
                calc_json: &calc_json,
                view_json: &empty_chart_view_json(),
                created_at: &now,
                updated_at: &now,
            },
        )?;
        insert_prashna(
            &tx,
            chart_id,
            &PrashnaMeta {
                question,
                number: input.number,
                focus_house: input.focus_house,
                created_at: now.clone(),
            },
        )?;
        if created {
            tx.execute(
                "UPDATE clients SET last_chart_id = ?1 WHERE id = ?2",
                params![chart_id, client_id],
            )?;
        } else {
            tx.execute(
                "UPDATE clients SET updated_at = ?1 WHERE id = ?2",
                params![now, client_id],
            )?;
        }
        tx.commit()?;
        Ok((client_id, self.chart(chart_id)?))
    }

    /// Haritanın prashna kaydı; harita prashna değilse `None`.
    pub fn prashna_meta(&self, chart_id: u32) -> Result<Option<PrashnaMeta>, StoreError> {
        load_prashna(&self.conn, chart_id)
    }

    /// Soru metnini ve odak evini değiştirir (otomatik kayıt). Sayı ve an değişmez: sayı
    /// haritanın lagnasıdır, an doğum formundan düzenlenir. Harita prashna değilse `NotFound`.
    pub fn update_prashna(
        &mut self,
        chart_id: u32,
        question: &str,
        focus_house: Option<u8>,
    ) -> Result<PrashnaMeta, StoreError> {
        check_focus_house(focus_house)?;
        let question = clean_question(question)?;
        let changed = self.conn.execute(
            "UPDATE prashna SET question = ?1, focus_house = ?2 WHERE chart_id = ?3",
            params![question, focus_house, chart_id],
        )?;
        if changed == 0 {
            return Err(StoreError::NotFound {
                what: "prashna",
                id: chart_id,
            });
        }
        load_prashna(&self.conn, chart_id)?.ok_or(StoreError::NotFound {
            what: "prashna",
            id: chart_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::birth::fixtures::{delhi, istanbul};
    use crate::charts::testutil::new_client;
    use crate::db::testutil::store;
    use crate::model::{NewChart, TimeCertainty};
    use crate::{ExportOptions, SCHEMA_VERSION};

    const QUESTION: &str = "Kayıp anahtar bulunacak mı?";

    fn new_prashna(client_id: Option<u32>) -> NewPrashna {
        NewPrashna {
            client_id,
            birth: delhi(),
            number: None,
            question: format!("  {QUESTION}\n"),
            focus_house: Some(4),
        }
    }

    #[test]
    fn a_prashna_is_an_event_chart_with_a_side_row() {
        let mut s = store();
        let (client, natal) = s.create_client(&new_client("Ayşe")).unwrap();
        let (owner, chart) = s
            .create_prashna(&new_prashna(Some(client.id)), "Prashna 2000-01-01 17:30")
            .unwrap();
        assert_eq!(owner, client.id);
        assert_eq!(chart.client_id, client.id);
        assert_eq!(chart.kind, ChartKind::Event);
        assert_eq!(chart.label, "Prashna 2000-01-01 17:30");
        assert_eq!(chart.birth.time_certainty, TimeCertainty::Exact);
        let meta = s.prashna_meta(chart.id).unwrap().expect("a prashna row");
        // Soru kırpılır; sayı yoksa zaman prashnasıdır.
        assert_eq!(meta.question, QUESTION);
        assert_eq!((meta.number, meta.focus_house), (None, Some(4)));
        assert_eq!(meta.created_at, chart.created_at);
        // Doğum haritası prashna değildir.
        assert_eq!(s.prashna_meta(natal.id).unwrap(), None);
        // Harita genel varsayılan ayarların kopyasını alır ve danışanın haritaları arasındadır.
        assert_eq!(s.charts_of(client.id).unwrap().len(), 2);
        assert!(s.client(client.id).unwrap().updated_at >= chart.created_at);
        // Şema sürümü 2'dir ve `kind` değişmemiştir.
        assert_eq!((s.schema_version().unwrap(), SCHEMA_VERSION), (2, 2));
    }

    #[test]
    fn without_a_client_prashnas_gather_under_one_built_in_client() {
        let mut s = store();
        assert_eq!(s.prashna_client_id().unwrap(), None);
        let (book, first) = s.create_prashna(&new_prashna(None), "Prashna 1").unwrap();
        let client = s.client(book).unwrap();
        assert_eq!(client.name, PRASHNA_CLIENT_NAME);
        assert_eq!(client.last_chart_id, Some(first.id));
        assert_eq!(s.prashna_client_id().unwrap(), Some(book));
        // İkinci prashna aynı danışana eklenir; danışan ada göre değil kayda göre bulunur.
        s.update_client(book, "Sorular", None).unwrap();
        let (again, second) = s.create_prashna(&new_prashna(None), "Prashna 2").unwrap();
        assert_eq!(again, book);
        assert_eq!(second.client_id, book);
        assert_eq!(s.list_clients(10).unwrap().len(), 1);
        // "Prashna" adlı başka bir danışan yerleşik danışan sayılmaz.
        let (other, _) = s.create_client(&new_client(PRASHNA_CLIENT_NAME)).unwrap();
        assert_ne!(other.id, book);
        assert_eq!(s.prashna_client_id().unwrap(), Some(book));
        // Yerleşik danışan silinirse sonraki prashna yenisini açar.
        s.delete_client(book).unwrap();
        assert_eq!(s.prashna_client_id().unwrap(), None);
        let (fresh, _) = s.create_prashna(&new_prashna(None), "Prashna 3").unwrap();
        assert_ne!(fresh, book);
        assert_eq!(s.client(fresh).unwrap().name, PRASHNA_CLIENT_NAME);
        // Okunamayan kayıt da "yok"tur.
        s.app_state_set(PRASHNA_CLIENT_KEY, "{").unwrap();
        assert_eq!(s.prashna_client_id().unwrap(), None);
    }

    #[test]
    fn question_number_and_focus_house_round_trip_through_open_client() {
        let mut s = store();
        let (client, _) = s.create_client(&new_client("Ayşe")).unwrap();
        let input = NewPrashna {
            number: Some(54),
            ..new_prashna(Some(client.id))
        };
        let (_, chart) = s.create_prashna(&input, "Prashna").unwrap();
        // Kur → başka haritayı aç → yeniden aç.
        let natal = s.charts_of(client.id).unwrap()[1].id;
        assert_eq!(s.open_client(client.id, Some(natal)).unwrap().prashna, None);
        let opened = s.open_client(client.id, Some(chart.id)).unwrap();
        let meta = opened.prashna.expect("prashna meta");
        assert_eq!(meta.question, QUESTION);
        assert_eq!((meta.number, meta.focus_house), (Some(54), Some(4)));
        // Modül girdisi sayıyı taşır: eş harita olarak seçilen sayı prashnası da sayının
        // lagnasıyla hesaplanır.
        let module = s.chart_input(&chart.uuid).unwrap().unwrap();
        assert_eq!(module.lagna_number, Some(54));
        assert!(module.time_known);
        let plain = s
            .chart_input(&s.chart(natal).unwrap().uuid)
            .unwrap()
            .unwrap();
        assert_eq!(plain.lagna_number, None);
    }

    #[test]
    fn update_changes_the_question_and_the_focus_house_only() {
        let mut s = store();
        let (client, natal) = s.create_client(&new_client("Ayşe")).unwrap();
        let input = NewPrashna {
            number: Some(9),
            ..new_prashna(Some(client.id))
        };
        let (_, chart) = s.create_prashna(&input, "Prashna").unwrap();
        let before = s.prashna_meta(chart.id).unwrap().unwrap();
        let after = s.update_prashna(chart.id, " Yeni soru ", None).unwrap();
        assert_eq!(after.question, "Yeni soru");
        assert_eq!(after.focus_house, None);
        assert_eq!(
            (after.number, &after.created_at),
            (Some(9), &before.created_at)
        );
        assert_eq!(s.prashna_meta(chart.id).unwrap(), Some(after));
        // Boş soru geçerlidir.
        assert_eq!(
            s.update_prashna(chart.id, "", Some(12)).unwrap().question,
            ""
        );
        // Prashna olmayan harita ve olmayan harita.
        for id in [natal.id, 9999] {
            assert!(matches!(
                s.update_prashna(id, "x", None),
                Err(StoreError::NotFound { what: "prashna", id: got }) if got == id
            ));
        }
        // Doğum formundan düzenleme prashna kaydını korur.
        let edited = NewChart {
            label: "Düzeltilmiş".into(),
            kind: ChartKind::Event,
            birth: istanbul("2024-06-01", "14:30:00"),
        };
        s.update_chart(chart.id, &edited).unwrap();
        assert_eq!(s.prashna_meta(chart.id).unwrap().unwrap().number, Some(9));
    }

    #[test]
    fn invalid_input_names_the_field_and_writes_nothing() {
        let mut s = store();
        let (client, _) = s.create_client(&new_client("Ayşe")).unwrap();
        let long = "ş".repeat(MAX_QUESTION_CHARS + 1);
        let cases: [(NewPrashna, &str); 5] = [
            (
                NewPrashna {
                    number: Some(0),
                    ..new_prashna(Some(client.id))
                },
                "prashna.number",
            ),
            (
                NewPrashna {
                    number: Some(109),
                    ..new_prashna(Some(client.id))
                },
                "prashna.number",
            ),
            (
                NewPrashna {
                    focus_house: Some(0),
                    ..new_prashna(Some(client.id))
                },
                "prashna.focus_house",
            ),
            (
                NewPrashna {
                    focus_house: Some(13),
                    ..new_prashna(Some(client.id))
                },
                "prashna.focus_house",
            ),
            (
                NewPrashna {
                    question: long.clone(),
                    ..new_prashna(Some(client.id))
                },
                "prashna.question",
            ),
        ];
        for (input, field) in cases {
            match s.create_prashna(&input, "Prashna") {
                Err(StoreError::InvalidInput(f)) => assert_eq!(f, field),
                other => panic!("{field}: {other:?}"),
            }
        }
        // Tam sınırdaki soru (çok baytlı karakterlerle) geçerlidir.
        let edge = NewPrashna {
            question: "ş".repeat(MAX_QUESTION_CHARS),
            ..new_prashna(Some(client.id))
        };
        let (_, chart) = s.create_prashna(&edge, "Prashna").unwrap();
        assert!(matches!(
            s.update_prashna(chart.id, &long, None),
            Err(StoreError::InvalidInput("prashna.question"))
        ));
        assert!(matches!(
            s.update_prashna(chart.id, "x", Some(13)),
            Err(StoreError::InvalidInput("prashna.focus_house"))
        ));
        // Olmayan danışan: hiçbir şey yazılmaz, yerleşik danışan da oluşmaz.
        assert!(matches!(
            s.create_prashna(&new_prashna(Some(9999)), "Prashna"),
            Err(StoreError::NotFound { what: "client", .. })
        ));
        let rows: u32 = s
            .conn
            .query_row("SELECT count(*) FROM prashna", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 1);
        assert_eq!(s.prashna_client_id().unwrap(), None);
    }

    #[test]
    fn deleting_the_chart_or_the_client_deletes_the_prashna_row() {
        let mut s = store();
        let (client, _) = s.create_client(&new_client("Ayşe")).unwrap();
        let (_, a) = s
            .create_prashna(&new_prashna(Some(client.id)), "A")
            .unwrap();
        let (_, b) = s
            .create_prashna(&new_prashna(Some(client.id)), "B")
            .unwrap();
        let rows = |s: &Store| -> u32 {
            s.conn
                .query_row("SELECT count(*) FROM prashna", [], |r| r.get(0))
                .unwrap()
        };
        assert_eq!(rows(&s), 2);
        s.delete_chart(a.id).unwrap();
        assert_eq!(rows(&s), 1);
        assert_eq!(s.prashna_meta(b.id).unwrap().unwrap().question, QUESTION);
        s.delete_client(client.id).unwrap();
        assert_eq!(rows(&s), 0);
    }

    #[test]
    fn the_question_is_not_searchable_and_is_exported_only_with_the_notes() {
        let mut s = store();
        let (client, _) = s.create_client(&new_client("Ayşe")).unwrap();
        let input = NewPrashna {
            number: Some(108),
            ..new_prashna(Some(client.id))
        };
        let (_, chart) = s.create_prashna(&input, "Prashna").unwrap();
        // Arama dizini soru metnini görmez.
        for word in ["anahtar", "Kayıp", "bulunacak"] {
            assert!(s.search_clients(word, 10).unwrap().is_empty(), "{word}");
        }
        // Varsayılan dışa aktarım: sayı ve odak evi var, soru yok.
        let plain = s.export_json(&ExportOptions::default()).unwrap();
        assert!(!plain.contains("anahtar"));
        let value: serde_json::Value = serde_json::from_str(&plain).unwrap();
        let exported = value["clients"][0]["charts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["uuid"] == chart.uuid.as_str())
            .unwrap();
        assert_eq!(exported["prashna"]["number"], 108);
        assert_eq!(exported["prashna"]["focus_house"], 4);
        assert!(exported["prashna"]["question"].is_null());
        // Prashna olmayan haritada alan hiç yazılmaz.
        let natal = value["clients"][0]["charts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["uuid"] != chart.uuid.as_str())
            .unwrap();
        assert!(natal.get("prashna").is_none());
        // "Notları dahil et" işaretliyse soru yazılır.
        let with_notes = s
            .export_json(&ExportOptions {
                include_notes: true,
                client_ids: None,
            })
            .unwrap();
        assert!(with_notes.contains(QUESTION));
        // CSV dışa aktarım soru metnini hiç taşımaz.
        assert!(!s.export_csv().unwrap().contains("anahtar"));
    }

    #[test]
    fn import_restores_the_prashna_row_and_rejects_bad_values() {
        let mut s = store();
        let (client, _) = s.create_client(&new_client("Ayşe")).unwrap();
        let input = NewPrashna {
            number: Some(10),
            ..new_prashna(Some(client.id))
        };
        let (_, chart) = s.create_prashna(&input, "Prashna").unwrap();
        let with_notes = s
            .export_json(&ExportOptions {
                include_notes: true,
                client_ids: None,
            })
            .unwrap();
        let plain = s.export_json(&ExportOptions::default()).unwrap();

        let mut t = store();
        let report = t.import_json(&with_notes).unwrap();
        assert_eq!((report.clients_added, report.charts_added), (1, 2));
        let id: u32 = t
            .conn
            .query_row(
                "SELECT id FROM charts WHERE uuid = ?1",
                params![chart.uuid],
                |r| r.get(0),
            )
            .unwrap();
        let meta = t.prashna_meta(id).unwrap().expect("imported prashna");
        assert_eq!(meta.question, QUESTION);
        assert_eq!((meta.number, meta.focus_house), (Some(10), Some(4)));
        // İçe alınan yerleşik danışan kaydı taşınmaz: tercih ve durum içe aktarılmaz.
        assert_eq!(t.prashna_client_id().unwrap(), None);

        // Sorusuz dosya: satır boş soruyla kurulur, harita yine prashnadır.
        let mut u = store();
        u.import_json(&plain).unwrap();
        let id: u32 = u
            .conn
            .query_row(
                "SELECT id FROM charts WHERE uuid = ?1",
                params![chart.uuid],
                |r| r.get(0),
            )
            .unwrap();
        let meta = u.prashna_meta(id).unwrap().unwrap();
        assert_eq!((meta.question.as_str(), meta.number), ("", Some(10)));

        // Geçersiz değerler kaydın UUID'siyle ve alan adıyla reddedilir; hiçbir şey yazılmaz.
        for (from, to) in [
            ("\"number\": 10", "\"number\": 109"),
            ("\"focus_house\": 4", "\"focus_house\": 13"),
        ] {
            assert!(plain.contains(from), "{from}");
            let mut v = store();
            match v.import_json(&plain.replace(from, to)) {
                Err(StoreError::ImportRecord { item, field }) => {
                    assert_eq!((item.as_str(), field), (chart.uuid.as_str(), "prashna"));
                }
                other => panic!("{to}: {other:?}"),
            }
            assert!(v.list_clients(10).unwrap().is_empty());
        }
    }
}
````

- [ ] **Step 4: Testleri ve kapıları koş**

```bash
cargo fmt --all
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p store --lib 2>&1 | grep -E "^test result"
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
git diff HEAD --stat -- crates/store/src/schema.rs | tail -1
git diff HEAD -U0 -- crates/store/src/schema.rs | grep -c '^-[^-]'
```
Expected: fmt ve clippy çıktısız; `store` lib `112 passed; 0 failed; 1 ignored`; sayaç `0`; `schema.rs` yalnızca satır kazanır ve iki satırı değişir: son `grep -c` `2` yazar (göç listesi ve `SCHEMA_VERSION`; göç 1'in metni değişmez).

- [ ] **Step 5: Commit**

```bash
git add -A
git status --short
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(store): prashna side table (schema version 2), create and update, the built-in prashna client; question exported only with notes" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: bir yeni dosya (`prashna.rs`); `grep` çıktısız.

---

### Task 4: Uygulama: `create_prashna`, `update_prashna`, `compute_prashna_facts`; sayı prashnasının açılışı; bağlamalar, sarmalayıcılar ve fikstür

Spec §11 "Veri modeli ve IPC", "Test" (property; store gidiş-dönüşü), "Performans"; §13 katman 4 ve "komut katmanı"; §14; §15; K1, K2, K8; D10, D15–D18, D24, D25. Komutlar Plan 14 ve Plan 20'nin kalıbını izler.

**Files:**
- Create: `app/src-tauri/src/prashna_commands.rs`
- Modify: `app/src-tauri/src/lib.rs`, `app/src-tauri/src/commands.rs`, `app/src-tauri/src/client_commands.rs`, `app/src-tauri/tests/modules.rs`, `app/src-tauri/tests/open_client.rs`, `app/src-tauri/tests/ui_fixture.rs`, `app/ui/api.ts`, `app/ui/api.test.ts`, `app/ui/modules/input.ts`, `app/ui/modules/input.test.ts`, `app/ui/test/fixture.ts`, `app/e2e/ipc.ts`; üretilen: `app/ui/bindings.ts`, `app/ui/test/fixtures/delhi.json`
- Test: `prashna_commands.rs` (5); `tests/modules.rs` (`prashna_checks`: var olan tek testin içinde); `tests/open_client.rs` (`prashna_round_trip`: var olan tek testin içinde); `api.test.ts` ve `input.test.ts`'in beklentileri

**Interfaces:**
- Consumes (Task 1–3, Plan 11, 20): çekirdeğin `compute_chart_for`, `compute_prashna_facts`, `PrashnaFacts`; `store::{HomePlace, NewPrashna, PrashnaMeta, resolve_birth, MAX_NAME_CHARS, MAX_QUESTION_CHARS}`; `atlas::{offset_at, Timestamp, TzSpec}`; `client_commands::{blocking, lock, open_client_blocking, OpenedClient, StoreState}`; `Engine::call`.
- Produces:
  - `prashna_commands::{PrashnaMoment { Now, Local { date, time } }, PrashnaRequest { client_id, at, place, number, question, focus_house }, PrashnaEdit { question, focus_house }, PRASHNA_LABEL}`.
  - `prashna_birth(&PrashnaRequest, now) -> Result<(BirthData, String), CommandError>`, `create_prashna_record(store, &request, now) -> Result<(u32, u32), CommandError>`, `create_prashna_blocking(engine, store, request, now) -> Result<OpenedClient, CommandError>`, `update_prashna_body(store, chart_id, &edit) -> Result<PrashnaMeta, CommandError>`, `compute_prashna_facts_blocking(engine, input, focus_house) -> Result<PrashnaFacts, CommandError>`; Tauri komutları `create_prashna(request)`, `update_prashna(chart_id, edit)`, `compute_prashna_facts(input, focus_house)`.
  - `commands::compute_chart_for_blocking(engine, ChartInput) -> Result<Chart, CommandError>`; `OpenedClient.prashna: Option<PrashnaMeta>`; `open_client_blocking` ve `save_calc_settings_blocking` sayı prashnasını sayının lagnasıyla hesaplar.
  - TS: `api.createPrashna`, `api.updatePrashna`, `api.computePrashnaFacts`; `chartInput(opened).lagna_number`; `UiFixture.prashna`, `UiFixture.prashna_ids`; e2e taklidinde üç komut.

- [ ] **Step 1: Entegrasyon testleri, fikstür üreticisi ve sarmalayıcı tablosu (testler)**

`prashna_checks` gerçek ephemeris'le koşar: beş golden anın her birinde `compute_chart_for` sayı yokken v1 haritasının aynısıdır; `compute_prashna_facts` saf fonksiyonun v1 haritasının konumlarından verdiği olgularla aynıdır; lordların burcu, dignity'si, retrosu ve yanması v1'in graha tablosuyla, Ay'ın tithisi v1 panchangıyla tutar. Sayı değişmezi beş sayıyla sınanır (grahalar aynı, lagna sayıdan, evler burç evi; yoga komutu da aynı lagnayı görür) ve gerçek lagna aynı haritayı verir. Hatalar yalnızca alan adı taşır; bütçe ölçülür. `prashna_round_trip` uçtan ucadır: zaman ve sayı prashnası kurulur; başka harita açılıp yeniden açılınca soru, sayı ve odak evi korunur; oturum geri yükleme, ayar değişikliği ve doğum formundan düzenleme sayının lagnasını korur; danışansız prashna yerleşik danışanı açar; geçersiz yer hiçbir şey yazmaz; `create_prashna` ≤ 100 ms. Fikstür üreticisi iki prashna haritası ve üç olgu sonucu yazar (öteki bütün okumalardan sonra: önceki kayıtların kimlikleri değişmez). `api.test.ts` üç sarmalayıcıyı, `input.test.ts` sayının girdiye girdiğini bekler.

````diff
diff --git a/app/src-tauri/tests/modules.rs b/app/src-tauri/tests/modules.rs
index f6c529e..a68a2f7 100644
--- a/app/src-tauri/tests/modules.rs
+++ b/app/src-tauri/tests/modules.rs
@@ -25,7 +25,8 @@ use jyotish_core::{
     MAX_SUNRISE_GAP_DAYS, MUHURTA_METHOD, SEVEN_KARAKA_ROLES, SHADBALA_GRAHAS, SHADBALA_METHOD,
     YOGA_CATALOG, YOGA_CATALOG_VERSION,
 };
-use jyotish_desk_lib::commands::compute_chart_blocking;
+use jyotish_core::{number_lagna_lon, prashna_facts, rising_mode, ZodiacMode, PRASHNA_METHOD};
+use jyotish_desk_lib::commands::{compute_chart_blocking, compute_chart_for_blocking};
 use jyotish_desk_lib::module_commands::{
     chara_dasha_at_blocking, chara_dasha_children_blocking, compute_chara_dasha_blocking,
     compute_compat_blocking, compute_drishti_blocking, compute_jaimini_blocking,
@@ -35,6 +36,7 @@ use jyotish_desk_lib::muhurta_commands::{
     day_panchang_blocking, muhurta_search_blocking, MuhurtaPlace, MuhurtaQuery, MuhurtaSearchView,
     MuhurtaWindowView,
 };
+use jyotish_desk_lib::prashna_commands::compute_prashna_facts_blocking;
 use jyotish_desk_lib::transit_commands::TransitAt;
 use serde_json::Value;
 use store::TzChoice;
@@ -84,6 +86,7 @@ fn module_commands_run_on_the_engine_thread_and_map_errors() {
     compat_checks(&engine, &inputs);
     timeline_checks(&engine, &inputs);
     muhurta_checks(&engine, &inputs);
+    prashna_checks(&engine, &inputs);
 }
 
 fn drishti_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
@@ -2376,3 +2379,177 @@ fn muhurta_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
         }
     }
 }
+
+/// Prashna (ileri modüller spec'i §11): gerçek ephemeris'le olgular, sayı prashnasının
+/// değişmezi, v1 haritasıyla çapraz denetim, alan adlı hatalar ve bütçe.
+fn prashna_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
+    for (id, input) in inputs {
+        let chart = compute_chart_blocking(
+            engine,
+            input.jd_ut,
+            input.lat,
+            input.lon,
+            input.settings.clone(),
+        )
+        .expect("chart");
+        // `ChartInput`'un haritası, sayı yokken v1 haritasının aynısıdır.
+        assert_eq!(
+            compute_chart_for_blocking(engine, input.clone()).expect("chart for input"),
+            chart,
+            "{id}"
+        );
+        let positions = Positions::from(&chart);
+        for focus in [None, Some(1), Some(7), Some(10)] {
+            let facts = compute_prashna_facts_blocking(engine, input.clone(), focus)
+                .unwrap_or_else(|e| panic!("{id}: {}", e.code));
+            assert_eq!(facts.method, PRASHNA_METHOD, "{id}");
+            // Çapraz denetim: saf fonksiyon v1 haritasının konumlarından aynı olguları verir.
+            let mut pure =
+                prashna_facts(&positions, &input.settings.combustion_orbs, focus).expect("pure");
+            pure.warnings.clone_from(&facts.warnings);
+            assert_eq!(facts, pure, "{id}");
+            // v1 olgularıyla çapraz denetim: lordların burcu, evi, dignity'si, retrosu ve
+            // yanması haritanın graha tablosundakiyle aynıdır (golden fixture'larla doğrulanmış).
+            let lagna = facts.lagna.as_ref().expect("lagna");
+            assert_eq!(lagna.sign, chart.lagna.sign, "{id}");
+            assert_eq!(lagna.rising, rising_mode(chart.lagna.sign), "{id}");
+            let v1 = |g: Graha| chart.grahas.iter().find(|p| p.graha == g).expect("graha");
+            let mut lords = vec![&lagna.lord];
+            if let Some(f) = &facts.focus {
+                assert_eq!(Some(f.house), focus, "{id}");
+                assert_eq!(f.lord.graha, house_lord(chart.lagna.sign, f.house), "{id}");
+                lords.push(&f.lord);
+            } else {
+                assert_eq!(focus, None, "{id}");
+            }
+            for l in lords {
+                let g = v1(l.graha);
+                assert_eq!(
+                    (l.sign, l.dignity, l.retrograde, l.combust),
+                    (g.sign, g.dignity, g.retrograde, g.combust),
+                    "{id}: {:?}",
+                    l.graha
+                );
+                assert_eq!(l.house, g.sign.house_from(chart.lagna.sign), "{id}");
+            }
+            // Ay ve anın panchangı (v1) birbirini tutar.
+            let moon = v1(Graha::Moon);
+            assert_eq!(
+                (facts.moon.sign, facts.moon.nakshatra, facts.moon.pada),
+                (moon.sign, moon.nakshatra, moon.pada),
+                "{id}"
+            );
+            assert_eq!(
+                (facts.moon.tithi, facts.moon.paksha),
+                (chart.panchang.tithi.no, chart.panchang.tithi.paksha),
+                "{id}"
+            );
+            assert_eq!(facts.yogas, yogas(&positions), "{id}");
+        }
+
+        // Sayı prashnası: grahalar aynı anın grahalarıdır; yalnızca lagna ve evler değişir.
+        for number in [1, 9, 10, 54, 108] {
+            let numbered = ChartInput {
+                lagna_number: Some(number),
+                ..input.clone()
+            };
+            let c = compute_chart_for_blocking(engine, numbered.clone()).expect("number chart");
+            let want = number_lagna_lon(number).expect("lagna");
+            assert!(
+                (c.lagna.sidereal_lon - want).abs() < 1e-12,
+                "{id}: {number}"
+            );
+            assert_eq!(c.bhavas.system, HouseSystem::WholeSign, "{id}");
+            for (a, b) in c.grahas.iter().zip(&chart.grahas) {
+                assert_eq!(
+                    (a.lon, a.sidereal_lon, a.speed, a.sign, a.nakshatra),
+                    (b.lon, b.sidereal_lon, b.speed, b.sign, b.nakshatra),
+                    "{id}: {number} {:?}",
+                    a.graha
+                );
+                assert_eq!(a.house, a.sign.house_from(c.lagna.sign), "{id}");
+            }
+            assert_eq!(c.dasha, chart.dasha, "{id}");
+            assert_eq!(c.panchang, chart.panchang, "{id}");
+            let facts = compute_prashna_facts_blocking(engine, numbered.clone(), Some(7))
+                .expect("number facts");
+            assert_eq!(facts.lagna.as_ref().unwrap().sign, c.lagna.sign, "{id}");
+            // Her modül aynı lagnayı görür (ör. bakış tablosu ve yoga listesi).
+            let numbered_positions = Positions::from(&c);
+            assert_eq!(facts.yogas, yogas(&numbered_positions), "{id}");
+            assert_eq!(
+                compute_yogas_blocking(engine, numbered).expect("yogas"),
+                yogas(&numbered_positions),
+                "{id}"
+            );
+        }
+        // Gerçek lagna verilince (burç evi sistemi, sidereal kip) harita v1 haritasının aynısıdır.
+        if input.settings.house_system == HouseSystem::WholeSign
+            && input.settings.zodiac == ZodiacMode::Sidereal
+        {
+            let (jd, lat, lon, s) = (input.jd_ut, input.lat, input.lon, input.settings.clone());
+            let lagna = chart.lagna.sidereal_lon;
+            let again = engine
+                .call(move |se: &mut ephemeris::SwissEph| {
+                    jyotish_core::compute_chart_with_lagna(se, jd, lat, lon, &s, lagna)
+                })
+                .expect("engine")
+                .expect("chart");
+            assert_eq!(again, chart, "{id}");
+        }
+    }
+
+    // Hatalar yalnızca alan adı taşır; odak evi ve sayı konumdan önce denetlenir.
+    let (_, first) = &inputs[0];
+    let bad_place = ChartInput {
+        lat: 95.0,
+        ..first.clone()
+    };
+    for (input, focus, field) in [
+        (first.clone(), Some(0), "focus_house"),
+        (first.clone(), Some(13), "focus_house"),
+        (bad_place.clone(), Some(13), "focus_house"),
+        (
+            ChartInput {
+                lagna_number: Some(0),
+                ..bad_place.clone()
+            },
+            None,
+            "number",
+        ),
+        (
+            ChartInput {
+                lagna_number: Some(109),
+                ..first.clone()
+            },
+            Some(3),
+            "number",
+        ),
+    ] {
+        let e = compute_prashna_facts_blocking(engine, input, focus).unwrap_err();
+        assert_eq!(e.code, "invalid_prashna_input");
+        assert_eq!(e.params.len(), 1);
+        assert_eq!(e.params["field"], field);
+    }
+    // Saat bilinmiyorsa lagnaya bağlı bölümler boştur (K2).
+    let unknown = ChartInput {
+        time_known: false,
+        ..first.clone()
+    };
+    let facts = compute_prashna_facts_blocking(engine, unknown, Some(5)).expect("facts");
+    assert!(facts.lagna.is_none() && facts.focus.is_none() && facts.moon.house.is_none());
+
+    // Bütçe: spec §15 `compute_prashna_facts` ≤ 15 ms (temel harita hesabı dahil).
+    let numbered = ChartInput {
+        lagna_number: Some(54),
+        ..first.clone()
+    };
+    let (median, max) = median_of_15(|| {
+        compute_prashna_facts_blocking(engine, numbered.clone(), Some(7)).expect("facts");
+    });
+    eprintln!("compute_prashna_facts median {median:?}, max {max:?}");
+    assert!(
+        median <= Duration::from_millis(15),
+        "compute_prashna_facts median {median:?}"
+    );
+}
diff --git a/app/src-tauri/tests/open_client.rs b/app/src-tauri/tests/open_client.rs
index 26707cb..a4664e2 100644
--- a/app/src-tauri/tests/open_client.rs
+++ b/app/src-tauri/tests/open_client.rs
@@ -7,15 +7,18 @@ use std::sync::Mutex;
 use std::time::{Duration, Instant};
 
 use ephemeris::Engine;
-use jyotish_core::{AyanamsaSystem, CalcSettings, Graha, Sign, SignDashaPath};
+use jyotish_core::{AyanamsaSystem, CalcSettings, Graha, HouseSystem, Sign, SignDashaPath};
 use jyotish_desk_lib::client_commands::{
     add_chart_blocking, create_client_blocking, keep_stored_moment_blocking, open_client_blocking,
     restore_session_blocking, save_calc_settings_blocking, update_chart_blocking, DB_FILE_NAME,
 };
+use jyotish_desk_lib::prashna_commands::{
+    create_prashna_blocking, update_prashna_body, PrashnaEdit, PrashnaMoment, PrashnaRequest,
+};
 use serde_json::{json, Map};
 use store::{
-    AppPrefs, BirthData, ChartKind, CompatRoles, Layout, NewChart, NewClient, Panel, PanelLayout,
-    Place, Store, TimeCertainty, TzChoice, ViewState,
+    AppPrefs, BirthData, ChartKind, CompatRoles, HomePlace, Layout, NewChart, NewClient, Panel,
+    PanelLayout, Place, Store, TimeCertainty, TzChoice, ViewState,
 };
 
 /// tests/golden/new-delhi-2000-01-01.json: 17:30 IST = JD 2451545.0.
@@ -298,6 +301,10 @@ fn open_client_round_trips_and_is_fast() {
         ("not_found", "client")
     );
 
+    // Prashna gidiş-dönüşü; sonra danışanın son haritası yine doğum haritasıdır.
+    prashna_round_trip(&engine, &store, client_id);
+    open_client_blocking(&engine, &store, client_id, Some(chart_id)).expect("reopen natal");
+
     // Performans: spec §4 "Rust tarafında ≤ 50 ms". Ortanca, ısınmadan sonra.
     open_client_blocking(&engine, &store, client_id, None).expect("warm-up");
     let mut times: Vec<Duration> = (0..15)
@@ -318,3 +325,146 @@ fn open_client_round_trips_and_is_fast() {
         "open_client median {median:?}"
     );
 }
+
+/// Prashna uçtan uca (ileri modüller spec'i §11): kur → başka haritayı aç → yeniden aç; soru,
+/// sayı ve odak evi korunur; sayı prashnası her açılışta ve ayar değişikliğinde sayının
+/// lagnasıyla hesaplanır; `create_prashna` ≤ 100 ms.
+fn prashna_round_trip(engine: &Engine, store: &Mutex<Store>, client_id: u32) {
+    let now: atlas::Timestamp = "2026-11-27T12:34:56Z".parse().expect("timestamp");
+    let delhi_place = HomePlace {
+        place: delhi().place,
+        tz: delhi().tz,
+    };
+    let request = |number: Option<u8>, client_id: Option<u32>| PrashnaRequest {
+        client_id,
+        at: PrashnaMoment::Now,
+        place: delhi_place.clone(),
+        number,
+        question: "Deneme sorusu".into(),
+        focus_house: Some(7),
+    };
+
+    // Zaman prashnası: anın haritası, `compute_chart` ile aynı.
+    let time = create_prashna_blocking(engine, store, request(None, Some(client_id)), now)
+        .expect("time prashna");
+    assert_eq!(time.client.id, client_id);
+    assert_eq!(time.chart_record.kind, ChartKind::Event);
+    assert_eq!(time.chart_record.label, "Prashna 2026-11-27 18:04");
+    assert_eq!(time.chart_record.resolved.utc, "2026-11-27T12:34:56Z");
+    let meta = time.prashna.clone().expect("prashna meta");
+    assert_eq!(meta.question, "Deneme sorusu");
+    assert_eq!((meta.number, meta.focus_house), (None, Some(7)));
+    // Harita genel varsayılan ayarların kopyasını alır.
+    let defaults = store.lock().unwrap().app_prefs().unwrap().calc_settings;
+    assert_eq!(time.calc_settings, defaults);
+
+    // Sayı prashnası: aynı an, lagna sayıdan (54 → Başak 28°20′); grahalar aynı.
+    let number = create_prashna_blocking(engine, store, request(Some(54), Some(client_id)), now)
+        .expect("number prashna");
+    assert_eq!(number.prashna.as_ref().unwrap().number, Some(54));
+    assert_eq!(number.chart.lagna.sign, Sign::Virgo);
+    assert!((number.chart.lagna.deg_in_sign - (28.0 + 20.0 / 60.0)).abs() < 1e-9);
+    assert_eq!(number.chart.bhavas.system, HouseSystem::WholeSign);
+    for (a, b) in number.chart.grahas.iter().zip(&time.chart.grahas) {
+        assert_eq!((a.graha, a.lon, a.sign), (b.graha, b.lon, b.sign));
+    }
+    assert_ne!(
+        number.chart.lagna.sidereal_lon,
+        time.chart.lagna.sidereal_lon
+    );
+    let number_chart = number.chart_record.id;
+
+    // Kur → başka haritayı aç → yeniden aç: soru, sayı ve odak evi korunur; harita yine sayının
+    // lagnasıyla hesaplanır.
+    let other = open_client_blocking(engine, store, client_id, Some(time.chart_record.id))
+        .expect("open the time prashna");
+    assert_eq!(other.prashna.as_ref().unwrap().number, None);
+    assert_eq!(other.chart, time.chart);
+    let again = open_client_blocking(engine, store, client_id, Some(number_chart)).expect("reopen");
+    assert_eq!(again.prashna, number.prashna);
+    assert_eq!(again.chart, number.chart);
+    // Oturum geri yükleme de aynı haritayı verir.
+    let restored = restore_session_blocking(engine, store)
+        .expect("restore")
+        .expect("a session");
+    assert_eq!(restored.chart_record.id, number_chart);
+    assert_eq!(restored.chart.lagna.sign, Sign::Virgo);
+
+    // Ayar değişikliği sayının lagnasını korur (ev sistemi ne olursa olsun burç evleri).
+    let sripati = CalcSettings {
+        house_system: HouseSystem::Sripati,
+        ..CalcSettings::default()
+    };
+    let recomputed =
+        save_calc_settings_blocking(engine, store, number_chart, sripati).expect("save settings");
+    assert_eq!(recomputed.lagna, number.chart.lagna);
+    assert_eq!(recomputed.bhavas.system, HouseSystem::WholeSign);
+    assert_eq!(recomputed.meta.settings.house_system, HouseSystem::Sripati);
+
+    // Doğum formundan düzenleme (an değişir) prashna kaydını ve sayının lagnasını korur.
+    let edited = update_chart_blocking(
+        engine,
+        store,
+        number_chart,
+        NewChart {
+            label: "Düzeltilmiş soru".into(),
+            kind: ChartKind::Event,
+            birth: delhi(),
+        },
+    )
+    .expect("edit");
+    assert_eq!(edited.prashna.as_ref().unwrap().number, Some(54));
+    assert_eq!(edited.chart.jd_ut, 2_451_545.0);
+    assert_eq!(edited.chart.lagna.sign, Sign::Virgo);
+
+    // Soru ve odak evi otomatik kaydı; sonraki açılış yeni değerleri verir.
+    let edit = PrashnaEdit {
+        question: "Yeni soru".into(),
+        focus_house: None,
+    };
+    let saved = update_prashna_body(store, number_chart, &edit).expect("update");
+    assert_eq!(
+        (saved.question.as_str(), saved.focus_house),
+        ("Yeni soru", None)
+    );
+    let after = open_client_blocking(engine, store, client_id, Some(number_chart)).expect("open");
+    assert_eq!(after.prashna, Some(saved));
+
+    // Danışan seçilmemişse yerleşik "Prashna" danışanı açılır.
+    let book = create_prashna_blocking(engine, store, request(Some(1), None), now).expect("book");
+    assert_ne!(book.client.id, client_id);
+    assert_eq!(book.client.name, "Prashna");
+    assert_eq!(book.chart.lagna.sign, Sign::Aries);
+
+    // Geçersiz yer hiçbir şey yazmaz ve yalnızca alan adı taşır.
+    let mut bad = request(None, Some(client_id));
+    bad.place.place.lat = 95.0;
+    let before = store.lock().unwrap().charts_of(client_id).unwrap().len();
+    let err = create_prashna_blocking(engine, store, bad, now).unwrap_err();
+    assert_eq!(err.code, "invalid_prashna_input");
+    assert_eq!(err.params.len(), 1);
+    assert_eq!(err.params["field"], "place");
+    assert_eq!(
+        store.lock().unwrap().charts_of(client_id).unwrap().len(),
+        before
+    );
+
+    // Performans: spec §11 "`create_prashna` ≤ 100 ms uçtan uca". Ortanca, ısınmadan sonra.
+    let mut times: Vec<Duration> = (0..15)
+        .map(|_| {
+            let t = Instant::now();
+            create_prashna_blocking(engine, store, request(Some(108), None), now).expect("create");
+            t.elapsed()
+        })
+        .collect();
+    times.sort();
+    let median = times[times.len() / 2];
+    eprintln!(
+        "create_prashna median {median:?}, max {:?}",
+        times[times.len() - 1]
+    );
+    assert!(
+        median <= Duration::from_millis(100),
+        "create_prashna median {median:?}"
+    );
+}
diff --git a/app/src-tauri/tests/ui_fixture.rs b/app/src-tauri/tests/ui_fixture.rs
index 8ab8771..1615b52 100644
--- a/app/src-tauri/tests/ui_fixture.rs
+++ b/app/src-tauri/tests/ui_fixture.rs
@@ -13,8 +13,8 @@ use atlas::Timestamp;
 use ephemeris::Engine;
 use jyotish_core::muhurta::{NakshatraNature, TithiGroup, MUHURTA_MAX_MIN_MINUTES};
 use jyotish_core::{
-    kuta_class_ids, ChartInput, Graha, Kuta, MuhurtaCriteria, MuhurtaFlag, NatalMoon, Vara,
-    YOGA_CATALOG, YOGA_CONDITIONS,
+    kuta_class_ids, ChartInput, Graha, Kuta, LordLink, MuhurtaCriteria, MuhurtaFlag, NatalMoon,
+    RisingMode, Vara, PRASHNA_NUMBER_MAX, YOGA_CATALOG, YOGA_CONDITIONS,
 };
 use jyotish_desk_lib::atlas_commands::{preview_moment_body, search_places_blocking, AtlasState};
 use jyotish_desk_lib::client_commands::create_client_blocking;
@@ -27,11 +27,14 @@ use jyotish_desk_lib::module_commands::{
 use jyotish_desk_lib::muhurta_commands::{
     day_panchang_blocking, muhurta_search_blocking, MuhurtaPlace, MuhurtaQuery,
 };
+use jyotish_desk_lib::prashna_commands::{
+    compute_prashna_facts_blocking, create_prashna_blocking, PrashnaMoment, PrashnaRequest,
+};
 use jyotish_desk_lib::transit_commands::{transits_at_blocking, TransitAt};
 use serde_json::{json, Value};
 use store::{
     BirthData, ChartKind, Gender, HomePlace, NewChart, NewClient, Place, Store, TimeCertainty,
-    TzChoice,
+    TzChoice, MAX_QUESTION_CHARS,
 };
 
 const CLIENT_UUID: &str = "00000000-0000-4000-8000-000000000001";
@@ -39,6 +42,9 @@ const CHART_UUID: &str = "00000000-0000-4000-8000-000000000002";
 /// Uyum fikstürünün eş haritası (ikinci danışan).
 const PARTNER_CLIENT_UUID: &str = "00000000-0000-4000-8000-000000000011";
 const PARTNER_CHART_UUID: &str = "00000000-0000-4000-8000-000000000012";
+/// Prashna fikstürünün iki haritası (zaman ve sayı prashnası).
+const PRASHNA_CHART_UUID: &str = "00000000-0000-4000-8000-000000000021";
+const PRASHNA_NUMBER_CHART_UUID: &str = "00000000-0000-4000-8000-000000000022";
 
 /// Eş harita: İstanbul, 1987-03-17 10:00 (deneme verisi; gerçek kişi değildir).
 fn istanbul() -> BirthData {
@@ -317,6 +323,47 @@ fn ui_fixture_is_up_to_date() {
         .expect("store")
         .app_prefs_load()
         .expect("prefs");
+
+    // Prashna: fikstürün sabit saatinde ("şimdi"), açık danışanın altında, bulunulan yerde
+    // (Delhi) iki harita: zaman prashnası ve 54 sayısıyla sayı prashnası (deneme sorusu; gerçek
+    // kişi ya da soru değildir). Olgular odak evi 7 ile; bir de odak evi seçilmemiş hâli.
+    // Öteki bütün okumalardan sonra kurulur: önceki kayıtların kimlikleri değişmez.
+    let prashna_now = Timestamp::from_second(1_795_600_800).expect("timestamp");
+    let prashna_request = |number: Option<u8>| PrashnaRequest {
+        client_id: Some(opened.client.id),
+        at: PrashnaMoment::Now,
+        place: home_place.clone(),
+        number,
+        question: "Deneme sorusu: yolculuk bu ay olur mu?".into(),
+        focus_house: Some(7),
+    };
+    let prashna_time = create_prashna_blocking(&engine, &store, prashna_request(None), prashna_now)
+        .expect("time prashna");
+    let prashna_number =
+        create_prashna_blocking(&engine, &store, prashna_request(Some(54)), prashna_now)
+            .expect("number prashna");
+    let prashna_input = |o: &jyotish_desk_lib::client_commands::OpenedClient| ChartInput {
+        jd_ut: o.chart.jd_ut,
+        lat: o.chart.geo_lat,
+        lon: o.chart.geo_lon,
+        settings: o.calc_settings.clone(),
+        time_known: true,
+        lagna_number: o.prashna.as_ref().and_then(|p| p.number),
+    };
+    let prashna_facts =
+        compute_prashna_facts_blocking(&engine, prashna_input(&prashna_time), Some(7))
+            .expect("prashna facts");
+    let prashna_facts_no_focus =
+        compute_prashna_facts_blocking(&engine, prashna_input(&prashna_time), None)
+            .expect("prashna facts without a focus house");
+    let prashna_facts_number =
+        compute_prashna_facts_blocking(&engine, prashna_input(&prashna_number), Some(7))
+            .expect("number prashna facts");
+    assert_eq!(
+        prashna_number.chart.lagna.sign,
+        jyotish_core::Sign::Virgo,
+        "54 is the sixth sign"
+    );
     let value = json!({
         "opened": opened,
         "dasha_children": { "path": path, "periods": children },
@@ -354,6 +401,19 @@ fn ui_fixture_is_up_to_date() {
             "query": muhurta_query,
             "pages": muhurta_pages,
         },
+        "prashna": {
+            "opened": prashna_time,
+            "opened_number": prashna_number,
+            "facts": prashna_facts,
+            "facts_no_focus": prashna_facts_no_focus,
+            "facts_number": prashna_facts_number,
+        },
+        "prashna_ids": {
+            "rising": RisingMode::ALL,
+            "links": LordLink::ALL,
+            "max_number": PRASHNA_NUMBER_MAX,
+            "max_question_chars": MAX_QUESTION_CHARS,
+        },
         "muhurta_ids": {
             "flags": MuhurtaFlag::ALL,
             "tithi_groups": TithiGroup::ALL,
@@ -367,6 +427,8 @@ fn ui_fixture_is_up_to_date() {
         .replace(&opened.chart_record.uuid, CHART_UUID)
         .replace(&partner.client.uuid, PARTNER_CLIENT_UUID)
         .replace(&partner.chart_record.uuid, PARTNER_CHART_UUID)
+        .replace(&prashna_time.chart_record.uuid, PRASHNA_CHART_UUID)
+        .replace(&prashna_number.chart_record.uuid, PRASHNA_NUMBER_CHART_UUID)
         + "\n";
 
     let path = root.join("../ui/test/fixtures/delhi.json");
diff --git a/app/ui/api.test.ts b/app/ui/api.test.ts
index 7c75efd..ad94415 100644
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -42,6 +42,9 @@ vi.mock("./bindings", () => ({
     muhurtaSearch: vi.fn(),
     loadMuhurtaFilters: vi.fn(),
     saveMuhurtaFilters: vi.fn(),
+    createPrashna: vi.fn(),
+    updatePrashna: vi.fn(),
+    computePrashnaFacts: vi.fn(),
     searchPlaces: vi.fn(),
     previewMoment: vi.fn(),
     getAppPrefs: vi.fn(),
@@ -135,6 +138,9 @@ test("every wrapper forwards its arguments to the command of the same name", asy
     ["muhurtaSearch", [{ from_date: null, to_date: null }, { ayanamsa: "Lahiri" }, { cursor: { chunk: 1 } }]],
     ["loadMuhurtaFilters", []],
     ["saveMuhurtaFilters", [[{ name: "Yolculuk", criteria: {} }]]],
+    ["createPrashna", [{ client_id: null, at: "Now", number: 54, question: "", focus_house: 7 }]],
+    ["updatePrashna", [3, { question: "", focus_house: null }]],
+    ["computePrashnaFacts", [{ jd_ut: 1, time_known: true, lagna_number: 54 }, 7]],
     ["searchPlaces", ["delhi", 8]],
     ["previewMoment", [{ date: "2000-01-01" }]],
     ["getAppPrefs", []],
diff --git a/app/ui/modules/input.test.ts b/app/ui/modules/input.test.ts
index 9327bbc..784bf54 100644
--- a/app/ui/modules/input.test.ts
+++ b/app/ui/modules/input.test.ts
@@ -13,6 +13,11 @@ test("module input is the computed chart's moment, place and settings", () => {
 
 test("an unknown birth time is passed on so that Rust hides lagna-dependent output", () => {
   expect(chartInput(unknownTimeOpened()).time_known).toBe(false);
+  // Sayı prashnasında sayı girdiyle gider; zaman prashnasında ve öteki haritalarda gitmez.
+  const { prashna } = fixture();
+  expect(chartInput(prashna.opened_number)).toMatchObject({ jd_ut: prashna.opened_number.chart.jd_ut, time_known: true, lagna_number: 54 });
+  expect(chartInput(prashna.opened).lagna_number).toBeNull();
+  expect(inputKey(prashna.opened_number)).not.toBe(inputKey(prashna.opened));
 });
 
 test("the input key changes with the chart, the birth time certainty and the settings", () => {
````

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
cargo test -p jyotish-desk --test modules 2>&1 | grep -E "^error" | head -3
pnpm --dir app exec vitest run ui/api.test.ts ui/modules/input.test.ts 2>&1 | grep -E "Test Files|Tests "
```
Expected: `error[E0432]: unresolved import `jyotish_desk_lib::prashna_commands``, `error[E0432]: unresolved import `jyotish_desk_lib::commands::compute_chart_for_blocking``. Vitest: seçilen iki dosyada testler düşer (sarmalayıcılar ve fikstürün `prashna` alanı yok).

- [ ] **Step 3: Uygulama**

`prashna_commands.rs` üç komutu verir (testleri dosyanın sonundadır; önce onları okuyun: "şimdi"nin yerin duvar saatine çevrilmesi; örtüşmede anın korunması; girilen an; her hata alanı ve hiçbir parametrenin yeri, anı ya da soruyu taşımaması; kayıt ve güncelleme). `client_commands.rs` açılan haritayı `compute_chart_for_blocking` ile hesaplar. E2e taklidi üç komutu kazanır (Task 7 kullanır).

````diff
diff --git a/app/e2e/ipc.ts b/app/e2e/ipc.ts
index 0df1043..35af2b3 100644
--- a/app/e2e/ipc.ts
+++ b/app/e2e/ipc.ts
@@ -44,6 +44,8 @@ export async function mockIpc(page: Page, { session, unknownTime = false, partne
       // kaydı gerçek bir gidiş dönüş görür.
       let prefs = fixture.prefs.prefs;
       let muhurtaFilters: unknown[] = [];
+      // Bu oturumda kurulan prashna haritaları: harita kimliği → açılan kayıt.
+      const prashnas: Record<number, typeof ayse> = {};
       const chartOf = (clientId: number) => opened[clientId].chart_record.id;
       const stamp = () => new Date(Date.UTC(2026, 10, 25, 10, 0, noteSeq)).toISOString();
       const respond: Record<string, (args: Record<string, unknown>) => unknown> = {
@@ -155,6 +157,37 @@ export async function mockIpc(page: Page, { session, unknownTime = false, partne
           muhurtaFilters = args.filters as unknown[];
           return muhurtaFilters;
         },
+        // Prashna: harita yeniden hesaplanmaz. Sayı verilmişse fikstürün sayı prashnası (54),
+        // verilmemişse zaman prashnası döner; soru ve odak evi isteğinkidir. Açılan harita
+        // danışanın haritaları arasına girer; soru ve odak evi kaydı bellek içidir.
+        create_prashna: (args) => {
+          const request = args.request as { client_id: number | null; number: number | null; question: string; focus_house: number | null };
+          const base = structuredClone(request.number === null ? fixture.prashna.opened : fixture.prashna.opened_number);
+          base.prashna = { ...base.prashna, number: request.number, question: request.question.trim(), focus_house: request.focus_house };
+          if (request.client_id === null) base.client = { ...base.client, id: 9, uuid: "00000000-0000-4000-8000-000000000009", name: "Prashna", last_chart_id: base.chart_record.id };
+          base.chart_record = { ...base.chart_record, client_id: base.client.id };
+          opened[base.client.id] = base;
+          prashnas[base.chart_record.id] = base;
+          return base;
+        },
+        update_prashna: (args) => {
+          const target = prashnas[args.chartId as number];
+          if (!target) throw { code: "not_found", params: { what: "prashna", id: String(args.chartId) } };
+          const edit = args.edit as { question: string; focus_house: number | null };
+          target.prashna = { ...target.prashna, question: edit.question.trim(), focus_house: edit.focus_house };
+          return target.prashna;
+        },
+        // Olgular fikstürden: sayı prashnası, odak evi seçilmemiş ve odak evli zaman prashnası.
+        // Fikstürde yalnızca 7. evin olguları vardır; başka bir ev aynı olguları o evin
+        // numarasıyla döner (arayüz hesap yapmaz, numarayı çizer).
+        compute_prashna_facts: (args) => {
+          const input = args.input as { lagna_number: number | null };
+          const focus = args.focusHouse as number | null;
+          if (focus === null) return fixture.prashna.facts_no_focus;
+          const facts = structuredClone(input.lagna_number === null ? fixture.prashna.facts : fixture.prashna.facts_number);
+          facts.focus.house = focus;
+          return facts;
+        },
         print_window: () => null,
         // Güncelleme: sahte yayın; ağ yoktur. Kurulum yalnızca denetimin bildirdiği sürümü kabul eder.
         app_info: () => ({ version: "0.9.0", updaterConfigured: true }),
diff --git a/app/src-tauri/src/client_commands.rs b/app/src-tauri/src/client_commands.rs
index 4fad432..3922622 100644
--- a/app/src-tauri/src/client_commands.rs
+++ b/app/src-tauri/src/client_commands.rs
@@ -5,15 +5,16 @@ use std::sync::{Arc, Mutex};
 
 use atlas::MomentError;
 use ephemeris::Engine;
-use jyotish_core::{CalcSettings, Chart};
+use jyotish_core::{CalcSettings, Chart, ChartInput};
 use serde::Serialize;
 use specta::Type;
 use store::{
-    ChartRecord, Client, NewChart, NewClient, Note, Store, StoreError, StoreWarning, ViewState,
+    ChartRecord, Client, NewChart, NewClient, Note, PrashnaMeta, Store, StoreError, StoreWarning,
+    ViewState,
 };
 use tauri::State;
 
-use crate::commands::{compute_chart_blocking, CommandError};
+use crate::commands::{compute_chart_for_blocking, CommandError};
 
 /// Tauri durumunda tek store örneği.
 pub type StoreState = Arc<Mutex<Store>>;
@@ -31,6 +32,9 @@ pub struct OpenedClient {
     pub view_state: ViewState,
     pub chart: Chart,
     pub recent_notes: Vec<Note>,
+    /// Harita prashnaysa yan kaydı (soru, sayı, odak evi); değilse `null`. Sayı prashnasında
+    /// `chart` sayının lagnasıyla hesaplanmıştır (ileri modüller spec'i §11).
+    pub prashna: Option<PrashnaMeta>,
     pub warnings: Vec<StoreWarning>,
 }
 
@@ -143,6 +147,24 @@ pub(crate) fn lock(store: &Mutex<Store>) -> Result<std::sync::MutexGuard<'_, Sto
     }
 }
 
+/// Kayıtlı bir haritanın hesap girdisi: çözülmüş an, yer ve ayarlar; sayı prashnasında sayı.
+/// Harita her zaman lagnasıyla hesaplanır (`time_known`); saati bilinmeyen haritada lagnanın
+/// gösterilmemesi v1'deki gibi arayüzün işidir.
+fn stored_input(
+    chart: &ChartRecord,
+    settings: CalcSettings,
+    prashna: Option<&PrashnaMeta>,
+) -> ChartInput {
+    ChartInput {
+        jd_ut: chart.resolved.jd_ut,
+        lat: chart.birth.place.lat,
+        lon: chart.birth.place.lon,
+        settings,
+        time_known: true,
+        lagna_number: prashna.and_then(|p| p.number),
+    }
+}
+
 /// `open_client` gövdesi: kayıt (store kilidi kısa tutulur), sonra motor iş parçacığında
 /// harita. Bloklar; async bağlamdan `spawn_blocking` içinde çağrılmalıdır.
 pub fn open_client_blocking(
@@ -152,13 +174,9 @@ pub fn open_client_blocking(
     chart_id: Option<u32>,
 ) -> Result<OpenedClient, CommandError> {
     let rec = lock(store)?.open_client(client_id, chart_id)?;
-    let place = &rec.chart.birth.place;
-    let chart = compute_chart_blocking(
+    let chart = compute_chart_for_blocking(
         engine,
-        rec.chart.resolved.jd_ut,
-        place.lat,
-        place.lon,
-        rec.calc_settings.clone(),
+        stored_input(&rec.chart, rec.calc_settings.clone(), rec.prashna.as_ref()),
     )?;
     Ok(OpenedClient {
         client: rec.client,
@@ -167,6 +185,7 @@ pub fn open_client_blocking(
         view_state: rec.view_state,
         chart,
         recent_notes: rec.recent_notes,
+        prashna: rec.prashna,
         warnings: rec.warnings,
     })
 }
@@ -243,13 +262,13 @@ pub fn save_calc_settings_blocking(
     chart_id: u32,
     settings: CalcSettings,
 ) -> Result<Chart, CommandError> {
-    let rec = {
+    let (rec, prashna) = {
         let mut s = lock(store)?;
         s.save_calc_settings(chart_id, &settings)?;
-        s.chart(chart_id)?
+        (s.chart(chart_id)?, s.prashna_meta(chart_id)?)
     };
-    let place = &rec.birth.place;
-    compute_chart_blocking(engine, rec.resolved.jd_ut, place.lat, place.lon, settings)
+    // Sayı prashnası yeni ayarla da sayının lagnasıyla hesaplanır.
+    compute_chart_for_blocking(engine, stored_input(&rec, settings, prashna.as_ref()))
 }
 
 /// Bloklayan gövdeyi havuzda çalıştırır. Görev paniklerse (`JoinError`) `task_failed` döner
diff --git a/app/src-tauri/src/commands.rs b/app/src-tauri/src/commands.rs
index 4f57808..bda0772 100644
--- a/app/src-tauri/src/commands.rs
+++ b/app/src-tauri/src/commands.rs
@@ -5,7 +5,8 @@ use std::sync::Arc;
 
 use ephemeris::{Engine, EphError, SwissEph};
 use jyotish_core::{
-    Bhavas, CalcSettings, Chart, CoreError, DashaPeriod, DashaSeed, Graha, TransitSnapshot,
+    Bhavas, CalcSettings, Chart, ChartInput, CoreError, DashaPeriod, DashaSeed, Graha,
+    TransitSnapshot,
 };
 use serde::Serialize;
 use specta::Type;
@@ -118,6 +119,18 @@ pub fn compute_chart_blocking(
     Ok(chart)
 }
 
+/// Bir modül girdisinin haritası: sayı prashnasında (`input.lagna_number`) lagna sayıdan gelir,
+/// diğer her haritada `compute_chart_blocking` ile aynıdır. Açılan danışanın haritası buradan
+/// hesaplanır; böylece harita ile modül panelleri aynı lagnayı görür. Bloklar.
+pub fn compute_chart_for_blocking(
+    engine: &Engine,
+    input: ChartInput,
+) -> Result<Chart, CommandError> {
+    let chart =
+        engine.call(move |se: &mut SwissEph| jyotish_core::compute_chart_for(se, &input))??;
+    Ok(chart)
+}
+
 /// Haritayı hesaplar. `jd_ut` ve konum atlas katmanında çözülmüş olarak gelir.
 #[tauri::command]
 #[specta::specta]
diff --git a/app/src-tauri/src/lib.rs b/app/src-tauri/src/lib.rs
index fa5b6d7..7d8d4bb 100644
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -4,6 +4,7 @@ pub mod commands;
 pub mod data_dir;
 pub mod module_commands;
 pub mod muhurta_commands;
+pub mod prashna_commands;
 pub mod store_commands;
 pub mod transit_commands;
 pub mod update_commands;
@@ -40,6 +41,9 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             module_commands::compute_compat,
             muhurta_commands::day_panchang,
             muhurta_commands::muhurta_search,
+            prashna_commands::create_prashna,
+            prashna_commands::update_prashna,
+            prashna_commands::compute_prashna_facts,
             client_commands::open_client,
             client_commands::restore_session,
             client_commands::create_client,
diff --git a/app/ui/api.ts b/app/ui/api.ts
index be3b6ff..5f38ad0 100644
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -23,6 +23,8 @@ import {
   type NewClient,
   type BirthData,
   type PanelLayout,
+  type PrashnaEdit,
+  type PrashnaRequest,
   type SavedMuhurtaFilter,
   type Sign,
   type TransitAt,
@@ -111,6 +113,12 @@ export const api = {
   // Kayıtlı süzgeçler: ad ve koşullar; yer, tarih ve kişi kaydedilmez. Kayıt listenin tamamını yazar.
   loadMuhurtaFilters: () => wrap(() => commands.loadMuhurtaFilters()),
   saveMuhurtaFilters: (filters: SavedMuhurtaFilter[]) => wrap(() => commands.saveMuhurtaFilters(filters)),
+  // Prashna: soru anının haritası ("şimdi" Rust'ta okunur; girilen an sorunun yerinin duvar
+  // saatidir). `createPrashna` haritayı kurar ve açar; `updatePrashna` soru metnini ve odak evini
+  // kaydeder; olgular panel açıkken istenir ve saklanmaz. Soru metni, yer ve an loglanmaz.
+  createPrashna: (request: PrashnaRequest) => wrap(() => commands.createPrashna(request)),
+  updatePrashna: (chartId: number, edit: PrashnaEdit) => wrap(() => commands.updatePrashna(chartId, edit)),
+  computePrashnaFacts: (input: ChartInput, focusHouse: number | null) => wrap(() => commands.computePrashnaFacts(input, focusHouse)),
   searchPlaces: (query: string, limit: number) => wrap(() => commands.searchPlaces(query, limit)),
   previewMoment: (birth: BirthData) => wrap(() => commands.previewMoment(birth)),
   appInfo: () => wrap(() => commands.appInfo()),
diff --git a/app/ui/modules/input.ts b/app/ui/modules/input.ts
index 355b230..2d42d8e 100644
--- a/app/ui/modules/input.ts
+++ b/app/ui/modules/input.ts
@@ -3,10 +3,20 @@
 import type { ChartInput, OpenedClient } from "../bindings";
 import { timeUnknown } from "../session/birth";
 
-/** Açık haritanın modül girdisi: hesaplanan haritanın anı, konumu ve ayarları. */
+/**
+ * Açık haritanın modül girdisi: hesaplanan haritanın anı, konumu ve ayarları. Sayı prashnasında
+ * sayı da gider: her modül, haritanın gösterdiği (sayıdan gelen) lagnayı kullanır.
+ */
 export function chartInput(opened: OpenedClient): ChartInput {
   const { chart } = opened;
-  return { jd_ut: chart.jd_ut, lat: chart.geo_lat, lon: chart.geo_lon, settings: chart.meta.settings, time_known: !timeUnknown(opened), lagna_number: null };
+  return {
+    jd_ut: chart.jd_ut,
+    lat: chart.geo_lat,
+    lon: chart.geo_lon,
+    settings: chart.meta.settings,
+    time_known: !timeUnknown(opened),
+    lagna_number: opened.prashna?.number ?? null,
+  };
 }
 
 /**
diff --git a/app/ui/test/fixture.ts b/app/ui/test/fixture.ts
index 17c0f3b..d4f07a6 100644
--- a/app/ui/test/fixture.ts
+++ b/app/ui/test/fixture.ts
@@ -2,8 +2,8 @@
 // için `create_client` dönüşü, ilk 3. seviye dönemin alt dönemleri, "new delhi" atlas araması,
 // an önizlemesi, bir DST boşluğu hatası, aynı sabit saatte transit, bakış tablosu, Jaimini raporu,
 // chara dasha, Shadbala raporu, yoga raporu (kataloğuyla), ikinci bir danışanla uyum raporları,
-// gün panchangı (iki gün) ve üç dilimlik bir muhurta araması. Arayüz hesap yapmadığı için testler
-// bunu çizer.
+// gün panchangı (iki gün), üç dilimlik bir muhurta araması ve iki prashna haritası (olgularıyla).
+// Arayüz hesap yapmadığı için testler bunu çizer.
 import type {
   CharaDasha,
   ChartBrief,
@@ -17,6 +17,7 @@ import type {
   JaiminiReport,
   Kuta,
   KutaReport,
+  LordLink,
   MomentPreview,
   MuhurtaFlag,
   MuhurtaPlace,
@@ -26,7 +27,9 @@ import type {
   NatalMoon,
   OpenedClient,
   PlaceHit,
+  PrashnaFacts,
   PrefsLoad,
+  RisingMode,
   ShadbalaReport,
   Sign,
   SignDashaPath,
@@ -102,6 +105,22 @@ export interface UiFixture {
     query: MuhurtaQuery;
     pages: MuhurtaSearchView[];
   };
+  /**
+   * Prashna: fikstürün sabit saatinde ("şimdi"), açık danışanın altında ve Delhi'de kurulmuş iki
+   * harita. `opened` zaman prashnasının, `opened_number` 54 sayılı sayı prashnasının
+   * `create_prashna` dönüşüdür (deneme sorusu; odak evi 7). `facts` zaman prashnasının,
+   * `facts_number` sayı prashnasının odak evi 7 ile, `facts_no_focus` zaman prashnasının odak evi
+   * seçilmeden `compute_prashna_facts` dönüşüdür.
+   */
+  prashna: {
+    opened: OpenedClient;
+    opened_number: OpenedClient;
+    facts: PrashnaFacts;
+    facts_no_focus: PrashnaFacts;
+    facts_number: PrashnaFacts;
+  };
+  /** Rust `RisingMode::ALL`, `LordLink::ALL`, `PRASHNA_NUMBER_MAX` ve `MAX_QUESTION_CHARS`. */
+  prashna_ids: { rising: RisingMode[]; links: LordLink[]; max_number: number; max_question_chars: number };
   /** Rust `MuhurtaFlag::ALL`, `TithiGroup::ALL`, `NakshatraNature::ALL` ve `MUHURTA_MAX_MIN_MINUTES`. */
   muhurta_ids: { flags: MuhurtaFlag[]; tithi_groups: TithiGroup[]; nakshatra_natures: NakshatraNature[]; max_min_minutes: number };
 }
````

**Yeni dosya:** `app/src-tauri/src/prashna_commands.rs`

````rust
//! Prashna komutları (ileri modüller spec'i §11): soru anının haritasını kurma, soru metnini ve
//! odak evini değiştirme, haritanın olguları. Ortak kalıp (K1–K3): hesap motor iş parçacığında,
//! komut gövdesi `blocking` havuzunda, hata `CommandError`. "Şimdi" Rust'ta okunur; girilen an
//! sorunun yerinin duvar saatidir ve Rust'ta çözülür (C4).
//!
//! Soru metni not gövdesi gibi hassastır; yer ve an kişisel veridir: hiçbiri loglanmaz ve hata
//! parametresine girmez. Bu dosyanın ürettiği hatalar yalnızca alan adı taşır
//! (`invalid_prashna_input`): v1'in doğum verisi hataları (`unknown_time_zone { tz }`,
//! `nonexistent_local_time { local, suggestion }`, `invalid_input { field: "birth.…" }`) bu
//! yollarda üretilmez.

use std::sync::{Arc, Mutex};

use atlas::{Timestamp, TzSpec};
use ephemeris::{Engine, SwissEph};
use jyotish_core::{ChartInput, PrashnaFacts, PRASHNA_NUMBER_MAX};
use serde::Deserialize;
use specta::Type;
use store::{
    BirthData, FoldChoice, HomePlace, NewPrashna, PrashnaMeta, Store, StoreError, TimeCertainty,
    TzChoice, MAX_NAME_CHARS, MAX_QUESTION_CHARS,
};
use tauri::State;

use crate::client_commands::{blocking, lock, open_client_blocking, OpenedClient, StoreState};
use crate::commands::CommandError;

/// Prashna haritasının etiketinin başı. Klasik terimdir: üç dilde aynı yazılır (K11).
pub const PRASHNA_LABEL: &str = "Prashna";

/// Soru anı. `Local`: sorunun yerinin duvar saati (`YYYY-MM-DD`, `HH:MM` ya da `HH:MM:SS`).
#[derive(Debug, Clone, PartialEq, Deserialize, Type)]
pub enum PrashnaMoment {
    Now,
    Local { date: String, time: String },
}

/// Yeni prashnanın girdisi. `client_id` boşsa harita yerleşik "Prashna" danışanına eklenir.
/// `place` sorunun yeridir (varsayılanı bulunulan yer) ve haritanın yeri olarak saklanır.
/// `number` doluysa sayı prashnasıdır: lagna sayıdan gelir (1..=108).
#[derive(Debug, Clone, PartialEq, Deserialize, Type)]
pub struct PrashnaRequest {
    pub client_id: Option<u32>,
    pub at: PrashnaMoment,
    pub place: HomePlace,
    pub number: Option<u8>,
    pub question: String,
    pub focus_house: Option<u8>,
}

/// Soru metni ve odak evi (otomatik kayıt).
#[derive(Debug, Clone, PartialEq, Deserialize, Type)]
pub struct PrashnaEdit {
    pub question: String,
    pub focus_house: Option<u8>,
}

fn invalid(field: &str) -> CommandError {
    CommandError::new("invalid_prashna_input", &[("field", field.to_owned())])
}

/// Store hatasının prashna yollarındaki karşılığı. Alan hataları ve an çözümü hataları yalnızca
/// alan adına iner: doğum verisinin hataları tarihi, saati ve dilimin adını parametreye yazar.
/// Veritabanı hataları ve `not_found { what, id }` olduğu gibi geçer (kişi verisi taşımazlar).
fn store_error(e: StoreError) -> CommandError {
    match e {
        StoreError::InvalidInput("prashna.question") => invalid("question"),
        StoreError::InvalidInput("prashna.number") => invalid("number"),
        StoreError::InvalidInput("prashna.focus_house") => invalid("focus_house"),
        StoreError::InvalidInput("birth.place.lat" | "birth.place.lon") => invalid("place"),
        StoreError::InvalidInput(_) | StoreError::Moment(_) => invalid("at"),
        other => other.into(),
    }
}

fn tz_spec(tz: &TzChoice) -> TzSpec {
    match tz {
        TzChoice::Iana(id) => TzSpec::Iana(id.trim().to_owned()),
        TzChoice::Fixed(secs) => TzSpec::Fixed(*secs),
        TzChoice::Lmt => TzSpec::Lmt,
    }
}

/// Yerin denetimi ve `now` anındaki ofseti. Geçersiz koordinat, fazla uzun ad ya da
/// çözülemeyen saat dilimi yalnızca alan adıyla döner.
fn place_offset(place: &HomePlace, now: Timestamp) -> Result<i32, CommandError> {
    let p = &place.place;
    let coords_ok = (-90.0..=90.0).contains(&p.lat) && (-180.0..=180.0).contains(&p.lon);
    let names_ok = [&p.name, &p.country, &p.admin1]
        .iter()
        .all(|s| s.chars().count() <= MAX_NAME_CHARS);
    if !(coords_ok && names_ok) {
        return Err(invalid("place"));
    }
    atlas::offset_at(&tz_spec(&place.tz), p.lon, now).map_err(|_| invalid("place"))
}

fn birth_at(place: &HomePlace, date: &str, time: &str, fold: Option<FoldChoice>) -> BirthData {
    BirthData {
        date: date.to_owned(),
        time: Some(time.to_owned()),
        time_certainty: TimeCertainty::Exact,
        place: place.place.clone(),
        tz: place.tz.clone(),
        fold,
        notes: String::new(),
    }
}

/// Prashnanın doğum verisi (soru anı ve yeri) ve haritanın etiketi. Denetim sırası: yer, sayı,
/// odak evi, soru, an. `Now` anı yerin duvar saatine çevrilir; duvar saati yaz saati
/// örtüşmesine düşüyorsa `now`'ı veren taraf seçilir. Girilen duvar saati boşluğa ya da
/// örtüşmeye düşüyorsa ya da okunamıyorsa `at` alanıyla reddedilir (sessizce kaydırılmaz).
pub fn prashna_birth(
    request: &PrashnaRequest,
    now: Timestamp,
) -> Result<(BirthData, String), CommandError> {
    let offset = place_offset(&request.place, now)?;
    if request
        .number
        .is_some_and(|n| !(1..=PRASHNA_NUMBER_MAX).contains(&n))
    {
        return Err(invalid("number"));
    }
    if request.focus_house.is_some_and(|h| !(1..=12).contains(&h)) {
        return Err(invalid("focus_house"));
    }
    if request.question.trim().chars().count() > MAX_QUESTION_CHARS {
        return Err(invalid("question"));
    }
    let birth = match &request.at {
        PrashnaMoment::Now => {
            let local = Timestamp::from_second(now.as_second() + i64::from(offset))
                .map_err(|_| invalid("at"))?;
            let date = local.strftime("%Y-%m-%d").to_string();
            let time = local.strftime("%H:%M:%S").to_string();
            // Örtüşmede iki an vardır: `now`'ı veren taraf seçilir. Diğer her durumda katlama
            // seçimi gereksizdir ve kayda yazılmaz. Yerel ortalama saatte ofset saniyeye
            // yuvarlanır: çözülen an `now`'dan bir saniyeye dek ayrışabilir.
            let wanted = now.as_second();
            let hits = |fold: Option<FoldChoice>| {
                store::resolve_birth(&birth_at(&request.place, &date, &time, fold))
                    .ok()
                    .and_then(|(_, r)| r.utc.parse::<Timestamp>().ok())
                    .is_some_and(|t| (t.as_second() - wanted).abs() <= 1)
            };
            let fold = [None, Some(FoldChoice::Earlier), Some(FoldChoice::Later)]
                .into_iter()
                .find(|f| hits(*f))
                .ok_or_else(|| invalid("at"))?;
            birth_at(&request.place, &date, &time, fold)
        }
        PrashnaMoment::Local { date, time } => birth_at(&request.place, date, time, None),
    };
    let (canonical, _) = store::resolve_birth(&birth).map_err(store_error)?;
    let clock = canonical.time.as_deref().unwrap_or_default();
    let label = format!(
        "{PRASHNA_LABEL} {} {}",
        canonical.date,
        clock.get(..5).unwrap_or(clock)
    );
    Ok((canonical, label))
}

/// Prashna haritasını kaydeder; dönen çift danışanın ve haritanın kimliğidir.
pub fn create_prashna_record(
    store: &Mutex<Store>,
    request: &PrashnaRequest,
    now: Timestamp,
) -> Result<(u32, u32), CommandError> {
    let (birth, label) = prashna_birth(request, now)?;
    let input = NewPrashna {
        client_id: request.client_id,
        birth,
        number: request.number,
        question: request.question.clone(),
        focus_house: request.focus_house,
    };
    let (client_id, chart) = lock(store)?
        .create_prashna(&input, &label)
        .map_err(store_error)?;
    Ok((client_id, chart.id))
}

/// `create_prashna` gövdesi; `now` testte sabitlenir. Bloklar.
pub fn create_prashna_blocking(
    engine: &Engine,
    store: &Mutex<Store>,
    request: PrashnaRequest,
    now: Timestamp,
) -> Result<OpenedClient, CommandError> {
    let (client_id, chart_id) = create_prashna_record(store, &request, now)?;
    open_client_blocking(engine, store, client_id, Some(chart_id))
}

/// Soru anının haritasını kurar ve açar. An "şimdi" ya da sorunun yerinin duvar saatidir; yer
/// haritanın yeri olarak saklanır. Harita `Event` türündedir ve genel varsayılan ayarların
/// kopyasını alır. Soru metni, yer ve an loglanmaz; hatalar yalnızca alan adı taşır.
#[tauri::command]
#[specta::specta]
pub async fn create_prashna(
    engine: State<'_, Arc<Engine>>,
    store: State<'_, StoreState>,
    request: PrashnaRequest,
) -> Result<OpenedClient, CommandError> {
    let (engine, store) = (Arc::clone(&engine), Arc::clone(&store));
    blocking(move || create_prashna_blocking(&engine, &store, request, Timestamp::now())).await
}

/// `update_prashna` gövdesi.
pub fn update_prashna_body(
    store: &Mutex<Store>,
    chart_id: u32,
    edit: &PrashnaEdit,
) -> Result<PrashnaMeta, CommandError> {
    lock(store)?
        .update_prashna(chart_id, &edit.question, edit.focus_house)
        .map_err(store_error)
}

/// Açık prashnanın soru metnini ve odak evini kaydeder (otomatik kayıt). Harita yeniden
/// hesaplanmaz: ikisi de lagnayı ve grahaları etkilemez.
#[tauri::command]
#[specta::specta]
pub async fn update_prashna(
    store: State<'_, StoreState>,
    chart_id: u32,
    edit: PrashnaEdit,
) -> Result<PrashnaMeta, CommandError> {
    let store = Arc::clone(&store);
    blocking(move || update_prashna_body(&store, chart_id, &edit)).await
}

/// `compute_prashna_facts` komutunun gövdesi; bloklar.
pub fn compute_prashna_facts_blocking(
    engine: &Engine,
    input: ChartInput,
    focus_house: Option<u8>,
) -> Result<PrashnaFacts, CommandError> {
    let facts = engine.call(move |se: &mut SwissEph| {
        jyotish_core::compute_prashna_facts(se, &input, focus_house)
    })??;
    Ok(facts)
}

/// Soru haritasının olguları: lagna ve lordu, odak evinin lordu ve iki lordun bağı, Ay,
/// kendralardaki grahalar ve yoga listesi. `input` açık haritanın girdisidir (sayı
/// prashnasında `lagna_number` doludur); `focus_house` astrologun seçtiği evdir. Sonuç
/// saklanmaz.
#[tauri::command]
#[specta::specta]
pub async fn compute_prashna_facts(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
    focus_house: Option<u8>,
) -> Result<PrashnaFacts, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_prashna_facts_blocking(&engine, input, focus_house)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use store::Place;

    const QUESTION: &str = "Kayıp anahtar bulunacak mı?";

    fn place(name: &str, lat: f64, lon: f64, tz: TzChoice) -> HomePlace {
        HomePlace {
            place: Place {
                name: name.into(),
                country: String::new(),
                admin1: String::new(),
                lat,
                lon,
                geoname_id: None,
            },
            tz,
        }
    }

    fn berlin() -> HomePlace {
        place(
            "Berlin",
            52.52,
            13.405,
            TzChoice::Iana("Europe/Berlin".into()),
        )
    }

    fn request(at: PrashnaMoment, place: HomePlace) -> PrashnaRequest {
        PrashnaRequest {
            client_id: None,
            at,
            place,
            number: None,
            question: QUESTION.into(),
            focus_house: Some(4),
        }
    }

    fn ts(text: &str) -> Timestamp {
        text.parse().expect("timestamp")
    }

    fn local(date: &str, time: &str) -> PrashnaMoment {
        PrashnaMoment::Local {
            date: date.into(),
            time: time.into(),
        }
    }

    fn utc_of(birth: &BirthData) -> String {
        store::resolve_birth(birth).expect("resolves").1.utc
    }

    #[test]
    fn now_becomes_the_wall_clock_of_the_place() {
        // 2026-11-27 12:34:56 UTC; Berlin kışın UTC+1.
        let now = ts("2026-11-27T12:34:56Z");
        let (birth, label) = prashna_birth(&request(PrashnaMoment::Now, berlin()), now).unwrap();
        assert_eq!(birth.date, "2026-11-27");
        assert_eq!(birth.time.as_deref(), Some("13:34:56"));
        assert_eq!(birth.time_certainty, TimeCertainty::Exact);
        assert_eq!(birth.fold, None);
        assert_eq!(birth.place.name, "Berlin");
        assert_eq!(label, "Prashna 2026-11-27 13:34");
        assert_eq!(utc_of(&birth), "2026-11-27T12:34:56Z");
        // Yerel ortalama saat: boylam 77,209° → +5 sa 08 dk 50 sn.
        let lmt = place("", 28.6139, 77.209, TzChoice::Lmt);
        let (birth, label) = prashna_birth(&request(PrashnaMoment::Now, lmt), now).unwrap();
        assert_eq!(birth.time.as_deref(), Some("17:43:46"));
        assert_eq!(label, "Prashna 2026-11-27 17:43");
        // Ofset saniyeye yuvarlanır (18 530,16 sn): çözülen an bir saniyenin içindedir.
        let resolved: Timestamp = utc_of(&birth).parse().unwrap();
        assert!((resolved.as_second() - now.as_second()).abs() <= 1);
    }

    #[test]
    fn now_inside_an_overlap_keeps_the_instant() {
        // Berlin 2026-10-25: 03:00 CEST'te saat 02:00 CET'e alınır; 02:30 iki kez yaşanır.
        for (utc, fold) in [
            ("2026-10-25T00:30:00Z", FoldChoice::Earlier),
            ("2026-10-25T01:30:00Z", FoldChoice::Later),
        ] {
            let (birth, _) =
                prashna_birth(&request(PrashnaMoment::Now, berlin()), ts(utc)).unwrap();
            assert_eq!(birth.time.as_deref(), Some("02:30:00"), "{utc}");
            assert_eq!(birth.fold, Some(fold), "{utc}");
            assert_eq!(utc_of(&birth), utc);
        }
    }

    #[test]
    fn a_typed_moment_is_the_wall_clock_of_the_place() {
        let now = ts("2026-11-27T12:00:00Z");
        let (birth, label) =
            prashna_birth(&request(local("2026-06-01", "14:05"), berlin()), now).unwrap();
        assert_eq!(
            (birth.date.as_str(), birth.time.as_deref()),
            ("2026-06-01", Some("14:05:00"))
        );
        assert_eq!(label, "Prashna 2026-06-01 14:05");
        // Yazın Berlin UTC+2.
        assert_eq!(utc_of(&birth), "2026-06-01T12:05:00Z");
    }

    #[test]
    fn errors_name_the_field_and_never_carry_the_place_the_moment_or_the_question() {
        let now = ts("2026-11-27T12:00:00Z");
        let field = |r: &PrashnaRequest| {
            let e = prashna_birth(r, now).unwrap_err();
            assert_eq!(e.code, "invalid_prashna_input");
            // Yalnızca alan adı: yer, an ve soru hata parametresine girmez.
            assert_eq!(e.params.len(), 1);
            let text = format!("{e:?}");
            for private in [
                "Berlin",
                "Europe",
                "2026-03-29",
                "02:30",
                "anahtar",
                "Mars/",
            ] {
                assert!(!text.contains(private), "{private}");
            }
            e.params["field"].clone()
        };
        // Yer: koordinat, bilinmeyen dilim, aralık dışı sabit ofset, fazla uzun ad.
        let mut bad = request(PrashnaMoment::Now, berlin());
        bad.place.place.lat = 91.0;
        assert_eq!(field(&bad), "place");
        bad.place.place.lat = f64::NAN;
        assert_eq!(field(&bad), "place");
        let zone = place(
            "Berlin",
            52.52,
            13.405,
            TzChoice::Iana("Mars/Olympus".into()),
        );
        assert_eq!(field(&request(PrashnaMoment::Now, zone)), "place");
        let offset = place("Berlin", 52.52, 13.405, TzChoice::Fixed(100 * 3600));
        assert_eq!(field(&request(PrashnaMoment::Now, offset)), "place");
        let mut long = request(PrashnaMoment::Now, berlin());
        long.place.place.name = "x".repeat(MAX_NAME_CHARS + 1);
        assert_eq!(field(&long), "place");
        // Yer her şeyden önce denetlenir: an da sayı da geçersizken alan `place`'tir.
        let mut all = request(local("bozuk", "25:99"), berlin());
        all.place.place.lon = 181.0;
        all.number = Some(0);
        assert_eq!(field(&all), "place");
        // Sayı, odak evi ve soru andan önce denetlenir.
        let mut r = request(local("bozuk", "25:99"), berlin());
        r.number = Some(109);
        assert_eq!(field(&r), "number");
        r.number = Some(0);
        assert_eq!(field(&r), "number");
        r.number = Some(108);
        r.focus_house = Some(13);
        assert_eq!(field(&r), "focus_house");
        r.focus_house = None;
        r.question = "ş".repeat(MAX_QUESTION_CHARS + 1);
        assert_eq!(field(&r), "question");
        r.question = QUESTION.into();
        // An: bozuk tarih ya da saat; yaz saati boşluğu (Berlin 2026-03-29 02:30 yoktur);
        // örtüşme (2026-10-25 02:30 iki kez yaşanır); hesap aralığının dışı.
        assert_eq!(field(&r), "at");
        for (date, time) in [
            ("2026-13-01", "10:00"),
            ("2026-03-29", "02:30"),
            ("2026-10-25", "02:30"),
            ("9999-01-01", "10:00"),
            ("", ""),
        ] {
            assert_eq!(field(&request(local(date, time), berlin())), "at", "{date}");
        }
    }

    #[test]
    fn the_record_is_written_under_the_client_or_the_built_in_one() {
        let store = Mutex::new(Store::open_in_memory().expect("store"));
        let now = ts("2026-11-27T12:34:56Z");
        let mut r = request(PrashnaMoment::Now, berlin());
        r.number = Some(54);
        let (book, chart_id) = create_prashna_record(&store, &r, now).unwrap();
        {
            let s = store.lock().unwrap();
            assert_eq!(s.prashna_client_id().unwrap(), Some(book));
            let chart = s.chart(chart_id).unwrap();
            assert_eq!(chart.label, "Prashna 2026-11-27 13:34");
            assert_eq!(chart.resolved.utc, "2026-11-27T12:34:56Z");
            let meta = s.prashna_meta(chart_id).unwrap().unwrap();
            assert_eq!(meta.question, QUESTION);
            assert_eq!((meta.number, meta.focus_house), (Some(54), Some(4)));
        }
        // Seçili danışan verilirse ona eklenir.
        r.client_id = Some(book);
        r.at = local("2026-06-01", "14:05");
        let (owner, second) = create_prashna_record(&store, &r, now).unwrap();
        assert_eq!(owner, book);
        assert_ne!(second, chart_id);
        // Olmayan danışan `not_found`'dur; kimlikten başka bir şey taşımaz.
        r.client_id = Some(9999);
        let e = create_prashna_record(&store, &r, now).unwrap_err();
        assert_eq!(e.code, "not_found");
        assert_eq!(e.params["what"], "client");
        // Düzenleme: soru ve odak evi; alan hataları yalnızca alan adı taşır.
        let edit = PrashnaEdit {
            question: "Yeni soru".into(),
            focus_house: None,
        };
        let meta = update_prashna_body(&store, chart_id, &edit).unwrap();
        assert_eq!(
            (meta.question.as_str(), meta.focus_house, meta.number),
            ("Yeni soru", None, Some(54))
        );
        for (edit, field) in [
            (
                PrashnaEdit {
                    question: "ş".repeat(MAX_QUESTION_CHARS + 1),
                    focus_house: None,
                },
                "question",
            ),
            (
                PrashnaEdit {
                    question: String::new(),
                    focus_house: Some(13),
                },
                "focus_house",
            ),
        ] {
            let e = update_prashna_body(&store, chart_id, &edit).unwrap_err();
            assert_eq!(e.code, "invalid_prashna_input");
            assert_eq!(e.params.len(), 1);
            assert_eq!(e.params["field"], field);
        }
        let e = update_prashna_body(&store, 9999, &edit_of("x")).unwrap_err();
        assert_eq!(e.code, "not_found");
        assert_eq!(e.params["what"], "prashna");
    }

    fn edit_of(question: &str) -> PrashnaEdit {
        PrashnaEdit {
            question: question.into(),
            focus_house: None,
        }
    }
}
````

- [ ] **Step 4: Bağlamaları ve fikstürü yeniden üret**

```bash
cargo fmt --all
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep -E "^test result"
cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep -E "^test result"
git diff --stat -- app/ui/bindings.ts app/ui/test/fixtures/delhi.json
grep -c "createPrashna\|updatePrashna\|computePrashnaFacts" app/ui/bindings.ts
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
```
Expected: üç `ok` (üçüncüsü üretilen fikstürün ikinci koşuda da aynı olduğunu söyler: prashna haritalarının UUID'leri sabitlenmiştir); `bindings.ts` 180 satır kazanır (üç komut ve tipleri); `delhi.json` tek satır değişir; `grep -c` en az `3`; son sayaç `0`.

- [ ] **Step 5: Testleri ve kapıları koş**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p jyotish-desk --lib 2>&1 | grep -E "^test result"
cargo test -p jyotish-desk --test modules -- --nocapture 2>&1 | grep -E "compute_prashna_facts|^test result"
cargo test -p jyotish-desk --test open_client -- --nocapture 2>&1 | grep -E "median|^test result"
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
pnpm --dir app exec tsc --noEmit -p . 2>&1 | head -3
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
```
Expected: fmt ve clippy çıktısız; lib `54 passed`; `compute_prashna_facts median` 15 ms'nin altında (ölçüm: ~1–2 ms) ve `1 passed`; `create_prashna median` 100 ms'nin, `open_client median` 50 ms'nin altında (ölçüm: ~1,6 ms ve ~1,3 ms) ve `1 passed`; sayaç `0`; `tsc` çıktısız; Vitest `80 passed (80)`, `806 passed (806)`; `✓ built`.

- [ ] **Step 6: Commit**

```bash
git add -A
git status --short
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached -U0 -- crates app/src-tauri/src | grep '^+' | grep -E 'println!|eprintln!|dbg!|log::|tracing::'
git commit -m "feat(app): create_prashna, update_prashna and compute_prashna_facts commands; number charts on open; bindings, wrappers and fixture" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: bir yeni dosya (`prashna_commands.rs`); iki `grep` çıktısız.

---

### Task 5: Arayüz: "Yeni prashna" penceresi, ⌘⇧P ve palet komutu

Spec §11 "Arayüz" (ilk madde); UI spec §4, §5, §6; D17, D19. Pencere `shell/Dialog`'un sağdan kayan biçimidir ve doğum formunun alan, hata ve onay kalıbını izler. Burada hesap yoktur.

**Files:**
- Create: `app/ui/prashna/model.ts`, `app/ui/prashna/model.test.ts`, `app/ui/prashna/PrashnaForm.tsx`, `app/ui/prashna/PrashnaForm.test.tsx`, `app/ui/prashna/prashna.css`
- Modify: `app/ui/shortcuts.ts`, `app/ui/shortcuts.test.ts`, `app/ui/shell/Shell.tsx`, `app/ui/palette/items.ts`, `app/ui/palette/Palette.tsx`, `app/ui/App.test.tsx`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`
- Test: `model.test.ts` (6), `PrashnaForm.test.tsx` (9), `App.test.tsx` (+1); `shortcuts.test.ts`'in beklentisi

**Interfaces:**
- Consumes (Task 4, Plan 11, 20): `api.createPrashna`, `PrashnaRequest`, `HomePlace`; `modules/muhurta.ts` (`chartPlace`, `hitPlace`, `placeLabel`); `panels/HomePlaceDialog.tsx` (`tzLabel`); `birth/PlaceSearch`; `shell/Dialog`, `shell/ConfirmDialog`; `usePrefs`, `useSession`.
- Produces:
  - `prashna/model.ts`: `PrashnaDraft`, `PrashnaField`, `PRASHNA_FIELDS`, `PRASHNA_PLACE_SOURCES`, `PRASHNA_NUMBER_MAX`, `MAX_QUESTION_CHARS`, `emptyDraft(opened)`, `draftPlace(draft, prefs, opened)`, `toRequest(draft, prefs, opened)`, `errorField(error)`.
  - `prashna/PrashnaForm.tsx`: `PrashnaForm({ onClose, onCreated? })`.
  - `shortcuts.ts`: komut `newPrashna` (⌘⇧P); palet komutu `newPrashna`; i18n `shortcuts.newPrashna`, `palette.command.newPrashna`, `prashna.focusNone`, `prashna.focusHouse`, `prashna.form.*`.

- [ ] **Step 1: Testler**

`model.test.ts`: sınırlar Rust'ınkilerdir (fikstürden); boş taslak; yer seçilen kaynaktan gelir ve sessizce değişmez; yazılanlar isteğe olduğu gibi girer; biçim hataları alanı adlandırır; Rust'ın alan adlı hatası alana eşlenir. `PrashnaForm.test.tsx`: varsayılanlar ve kurulan haritanın açılması; girilen an, sayı, soru ve odak evi; bulunulan yer yokken pencerenin yer istemesi; atlastan yer ve yerleşik danışan; biçim hatalarında odak ve hatanın düzeltmeyle silinmesi; Rust hatasının alanda sabit iletiyle gösterilmesi; iki basışta tek komut; Esc onayı; üç dil. `App.test.tsx`: ⌘⇧P ve palet, açık harita olsun olmasın; baskı önizlemesinde yok. `shortcuts.test.ts`: ⌘⇧P artık bir komuttur.

````diff
diff --git a/app/ui/App.test.tsx b/app/ui/App.test.tsx
index ee2b776..6b201a8 100644
--- a/app/ui/App.test.tsx
+++ b/app/ui/App.test.tsx
@@ -9,6 +9,7 @@ vi.mock("./api", () => ({
     previewMoment: vi.fn(async () => ({ status: "error", error: { code: "invalid_input", params: { field: "birth.date" } } })),
     searchPlaces: vi.fn(async () => ({ status: "ok", data: [] })),
     createClient: vi.fn(),
+    createPrashna: vi.fn(),
     updateChart: vi.fn(),
     saveViewState: vi.fn(async () => ({ status: "ok", data: null })),
     savePanelLayout: vi.fn(async () => ({ status: "ok", data: null })),
@@ -550,3 +551,37 @@ test("the palette opens About too", async () => {
   fireEvent.click(await within(palette).findByRole("option", { name: /Hakkında ve güncellemeler/ }));
   expect(await screen.findByRole("dialog", { name: "Jyotish Desk hakkında" })).toBeInTheDocument();
 });
+
+test("Cmd+Shift+P and the palette open the new prashna sheet, with or without an open chart; not in print view", async () => {
+  const { unmount } = renderApp({ opened: null });
+  fireEvent.keyDown(window, { key: "P", metaKey: true, shiftKey: true });
+  let sheet = screen.getByRole("dialog", { name: "Yeni prashna" });
+  // Açık danışan yok: prashna yerleşik danışana gider ve bunu form söyler.
+  expect(within(sheet).queryByRole("checkbox")).toBeNull();
+  fireEvent.click(within(sheet).getByRole("button", { name: "Vazgeç" }));
+  expect(screen.queryByRole("dialog")).toBeNull();
+  // Paletten: komut açık harita olmadan da vardır.
+  fireEvent.keyDown(window, { key: "k", metaKey: true });
+  const palette = screen.getByRole("dialog", { name: "Komut paleti" });
+  fireEvent.change(within(palette).getByRole("combobox"), { target: { value: "prashna" } });
+  fireEvent.click(await within(palette).findByRole("option", { name: /Yeni prashna/ }));
+  expect(screen.getByRole("dialog", { name: "Yeni prashna" })).toBeInTheDocument();
+  unmount();
+
+  renderApp();
+  // Kısayol yazı alanından da çalışır; pencere açıkken ikinci basış yok sayılır (modal).
+  fireEvent.keyDown(window, { key: "p", metaKey: true, shiftKey: true });
+  sheet = screen.getByRole("dialog", { name: "Yeni prashna" });
+  expect(within(sheet).getByRole("checkbox", { name: "“Örnek Danışan” danışanına ekle" })).toBeChecked();
+  fireEvent.keyDown(window, { key: "P", metaKey: true, shiftKey: true });
+  expect(screen.getAllByRole("dialog")).toHaveLength(1);
+  fireEvent.click(within(sheet).getByRole("button", { name: "Vazgeç" }));
+  // Baskı önizlemesinde tuş tüketilir ama pencere açılmaz; ⌘P'nin anlamı değişmez.
+  fireEvent.keyDown(window, { key: "p", metaKey: true });
+  expect(printToolbar()).toBeInTheDocument();
+  const ev = new KeyboardEvent("keydown", { key: "P", metaKey: true, shiftKey: true, cancelable: true });
+  window.dispatchEvent(ev);
+  expect(ev.defaultPrevented).toBe(true);
+  expect(screen.queryByRole("dialog", { name: "Yeni prashna" })).toBeNull();
+  expect(api.createPrashna).not.toHaveBeenCalled();
+});
diff --git a/app/ui/shortcuts.test.ts b/app/ui/shortcuts.test.ts
index 264b9ef..0a7473a 100644
--- a/app/ui/shortcuts.test.ts
+++ b/app/ui/shortcuts.test.ts
@@ -15,7 +15,9 @@ test("global commands", () => {
   expect(key({ key: "e", metaKey: true })).toEqual({ id: "editChart" });
   expect(key({ key: "k", metaKey: true })).toEqual({ id: "palette" });
   expect(key({ key: "p", metaKey: true })).toEqual({ id: "print" });
-  expect(key({ key: "P", metaKey: true, shiftKey: true })).toBeNull();
+  expect(key({ key: "P", metaKey: true, shiftKey: true })).toEqual({ id: "newPrashna" });
+  expect(key({ key: "p", metaKey: true, shiftKey: true })).toEqual({ id: "newPrashna" });
+  expect(key({ key: "K", metaKey: true, shiftKey: true })).toBeNull();
   expect(key({ key: "\\", code: "Backslash", metaKey: true })).toEqual({ id: "sidebar" });
   expect(key({ key: ",", code: "Backslash", metaKey: true })).toEqual({ id: "sidebar" });
   expect(key({ key: "?", shiftKey: true })).toEqual({ id: "help" });
````

**Yeni dosya:** `app/ui/prashna/PrashnaForm.test.tsx`

````tsx
import { act, fireEvent, screen, within } from "@testing-library/react";
import { vi } from "vitest";

vi.mock("../api", () => ({ api: { searchPlaces: vi.fn(), createPrashna: vi.fn(), setAppPrefs: vi.fn() } }));

import { api } from "../api";
import type { AppPrefs, OpenedClient } from "../bindings";
import { SEARCH_DEBOUNCE_MS } from "../birth/PlaceSearch";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import { useSession } from "../session/store";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { PrashnaForm } from "./PrashnaForm";

function OpenedLabel() {
  const { opened } = useSession();
  return <output aria-label="açık">{opened ? `${opened.client.name} / ${opened.chart_record.label}` : "-"}</output>;
}

const onClose = vi.fn();
const onCreated = vi.fn();
const home = () => fixture().muhurta.home_place;
const withHome = (): AppPrefs => ({ ...fixture().prefs.prefs, home_place: home() });

function renderForm({ opened = fixture().opened, prefs = withHome() }: { opened?: OpenedClient | null; prefs?: AppPrefs | null } = {}) {
  return renderWithStores(
    <>
      <OpenedLabel />
      <PrashnaForm onClose={onClose} onCreated={onCreated} />
    </>,
    { opened, prefs },
  );
}

const dialog = () => screen.getByRole("dialog", { name: "Yeni prashna" });
const field = (name: string | RegExp) => within(dialog()).getByLabelText(name);
const submit = () => fireEvent.click(within(dialog()).getByRole("button", { name: "Prashna kur" }));
const wait = (ms = 0) => act(async () => vi.advanceTimersByTime(ms));

beforeEach(() => {
  vi.useFakeTimers();
  onClose.mockReset();
  onCreated.mockReset();
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "ok", data: fixture().places });
  vi.mocked(api.createPrashna).mockResolvedValue({ status: "ok", data: fixture().prashna.opened });
});
afterEach(() => vi.useRealTimers());

test("defaults: now, the current place, no number, no focus house, the open client; casting opens the returned chart", async () => {
  renderForm();
  expect(document.activeElement).toBe(field("Şimdi"));
  expect(field("Şimdi")).toBeChecked();
  expect(within(dialog()).queryByLabelText("Tarih")).toBeNull();
  expect(field("Yer")).toHaveDisplayValue("Bulunulan yer");
  expect(dialog().querySelector(".prashna-place")).toHaveTextContent("New Delhi, Delhi, India · Asia/Kolkata");
  expect(field("Sayı (isteğe bağlı)")).toHaveValue("");
  expect(field("Odak evi")).toHaveDisplayValue("Seçilmedi");
  expect(field("“Örnek Danışan” danışanına ekle")).toBeChecked();
  submit();
  await wait();
  expect(api.createPrashna).toHaveBeenCalledWith({ client_id: 1, at: "Now", place: home(), number: null, question: "", focus_house: null });
  expect(screen.getByRole("status", { name: "açık" })).toHaveTextContent("Örnek Danışan / Prashna 2026-11-25 15:30");
  expect(onCreated).toHaveBeenCalledTimes(1);
  expect(onClose).toHaveBeenCalledTimes(1);
});

test("a typed moment, a number, a question and a focus house are sent as typed", async () => {
  vi.mocked(api.createPrashna).mockResolvedValue({ status: "ok", data: fixture().prashna.opened_number });
  renderForm();
  fireEvent.click(field("Başka bir an"));
  fireEvent.change(field("Tarih"), { target: { value: "2026-11-27" } });
  fireEvent.change(field("Saat"), { target: { value: "14:05" } });
  expect(field("Tarih")).toHaveAccessibleDescription("Sorunun yerinin duvar saati.");
  fireEvent.change(field("Sayı (isteğe bağlı)"), { target: { value: "54" } });
  fireEvent.change(field("Soru"), { target: { value: "Deneme sorusu" } });
  expect(field("Soru")).toHaveAccessibleDescription(/^13 \/ 2000 karakter\./);
  fireEvent.change(field("Odak evi"), { target: { value: "7" } });
  fireEvent.click(field("“Örnek Danışan” danışanına ekle"));
  submit();
  await wait();
  expect(api.createPrashna).toHaveBeenCalledWith({
    client_id: null,
    at: { Local: { date: "2026-11-27", time: "14:05" } },
    place: home(),
    number: 54,
    question: "Deneme sorusu",
    focus_house: 7,
  });
  expect(onClose).toHaveBeenCalledTimes(1);
});

test("without a current place the form asks for one and does not fall back to the chart's place", async () => {
  renderForm({ prefs: fixture().prefs.prefs });
  expect(dialog().querySelector(".prashna-place")).toHaveTextContent("Bulunulan yer seçilmemiş.");
  submit();
  await wait();
  expect(api.createPrashna).not.toHaveBeenCalled();
  expect(field("Yer")).toHaveAttribute("aria-invalid", "true");
  expect(field("Yer")).toHaveAccessibleDescription(/Bir yer seçin\./);
  expect(document.activeElement).toBe(field("Yer"));
  expect(within(dialog()).getByText("Düzeltilmesi gereken 1 alan var.")).toBeInTheDocument();
  // Açık haritanın yeri bir seçimdir; seçilince hata hemen kalkar.
  fireEvent.change(field("Yer"), { target: { value: "chart" } });
  expect(field("Yer")).not.toHaveAttribute("aria-invalid");
  expect(dialog().querySelector(".prashna-place")).toHaveTextContent("New Delhi, Delhi, India · Asia/Kolkata");
  submit();
  await wait();
  const { place, tz } = fixture().opened.chart_record.birth;
  expect(vi.mocked(api.createPrashna).mock.calls[0][0].place).toEqual({ place, tz });
});

test("another place is searched in the atlas; with no open client the prashna goes to the built-in client", async () => {
  renderForm({ opened: null, prefs: fixture().prefs.prefs });
  // Açık harita yokken "açık haritanın yeri" seçeneği ve danışan kutusu yoktur.
  expect(within(field("Yer")).getAllByRole("option").map((o) => o.textContent)).toEqual(["Bulunulan yer", "Başka bir yer ara"]);
  expect(within(dialog()).queryByRole("checkbox")).toBeNull();
  expect(within(dialog()).getByText(/Açık danışan yok: prashna “Prashna” adlı yerleşik danışanın altında toplanır\./)).toBeInTheDocument();
  fireEvent.change(field("Yer"), { target: { value: "search" } });
  expect(dialog().querySelector(".prashna-place")).toHaveTextContent("Henüz yer seçilmedi.");
  fireEvent.change(field("Yer ara"), { target: { value: "new delhi" } });
  await wait(SEARCH_DEBOUNCE_MS);
  fireEvent.keyDown(field("Yer ara"), { key: "Enter" });
  const hit = fixture().places[0];
  expect(dialog().querySelector(".prashna-place")).toHaveTextContent(`${hit.name}`);
  submit();
  await wait();
  const sent = vi.mocked(api.createPrashna).mock.calls[0][0];
  expect(sent.client_id).toBeNull();
  expect(sent.place).toEqual({ place: { name: hit.name, country: hit.country, admin1: hit.admin1, lat: hit.lat, lon: hit.lon, geoname_id: hit.geoname_id }, tz: { Iana: hit.timezone } });
});

test("format errors stay in the form, focus the first invalid field and clear when the field is edited", async () => {
  renderForm();
  fireEvent.click(field("Başka bir an"));
  fireEvent.change(field("Sayı (isteğe bağlı)"), { target: { value: "109" } });
  submit();
  await wait();
  expect(api.createPrashna).not.toHaveBeenCalled();
  expect(field("Tarih")).toHaveAttribute("aria-invalid", "true");
  expect(document.activeElement).toBe(field("Tarih"));
  expect(field("Sayı (isteğe bağlı)")).toHaveAccessibleDescription(/Sayı 1 ile 108 arasında bir tamsayı olmalı\./);
  expect(within(dialog()).getByText("Düzeltilmesi gereken 2 alan var.")).toBeInTheDocument();
  fireEvent.change(field("Sayı (isteğe bağlı)"), { target: { value: "108" } });
  expect(field("Sayı (isteğe bağlı)")).not.toHaveAttribute("aria-invalid");
  fireEvent.click(field("Şimdi"));
  expect(within(dialog()).queryByText(/Düzeltilmesi gereken/)).toBeNull();
});

test("a field error from Rust is shown on its field with a static message; other errors are a form alert", async () => {
  vi.mocked(api.createPrashna).mockResolvedValueOnce({ status: "error", error: { code: "invalid_prashna_input", params: { field: "at" } } });
  renderForm();
  fireEvent.click(field("Başka bir an"));
  fireEvent.change(field("Tarih"), { target: { value: "2026-03-29" } });
  fireEvent.change(field("Saat"), { target: { value: "02:30" } });
  submit();
  await wait();
  expect(onClose).not.toHaveBeenCalled();
  expect(field("Saat")).toHaveAttribute("aria-invalid", "true");
  expect(field("Saat")).toHaveAccessibleDescription(/Bu an bu yerde çözülemedi\./);
  expect(document.activeElement).toBe(field("Tarih"));
  // Açık harita değişmedi.
  expect(screen.getByRole("status", { name: "açık" })).toHaveTextContent("Örnek Danışan /");
  // Alan düzeltilince hata kalkar; başka bir hata formun uyarısıdır.
  fireEvent.change(field("Saat"), { target: { value: "03:30" } });
  expect(field("Saat")).not.toHaveAttribute("aria-invalid");
  vi.mocked(api.createPrashna).mockResolvedValueOnce({ status: "error", error: { code: "not_found", params: { what: "client", id: "1" } } });
  submit();
  await wait();
  expect(within(dialog()).getByRole("alert")).toHaveTextContent(/^Prashna kurulamadı: /);
  expect(onClose).not.toHaveBeenCalled();
});

test("the cast button is locked while the chart is being cast: one command for two presses", async () => {
  let finish: (v: Awaited<ReturnType<typeof api.createPrashna>>) => void = () => {};
  vi.mocked(api.createPrashna).mockReturnValue(new Promise((resolve) => (finish = resolve)));
  renderForm();
  submit();
  await wait();
  submit();
  await wait();
  expect(api.createPrashna).toHaveBeenCalledTimes(1);
  await act(async () => finish({ status: "ok", data: fixture().prashna.opened }));
  expect(onClose).toHaveBeenCalledTimes(1);
});

test("Escape asks before discarding a changed draft; Cancel closes without asking", async () => {
  renderForm();
  fireEvent.keyDown(document, { key: "Escape" });
  expect(onClose).toHaveBeenCalledTimes(1);
  onClose.mockReset();
  fireEvent.change(field("Soru"), { target: { value: "Deneme" } });
  fireEvent.keyDown(document, { key: "Escape" });
  expect(onClose).not.toHaveBeenCalled();
  const confirm = screen.getByRole("dialog", { name: "Değişiklikler atılsın mı?" });
  fireEvent.keyDown(document, { key: "Escape" });
  expect(confirm).not.toBeInTheDocument();
  expect(field("Soru")).toHaveValue("Deneme");
  fireEvent.click(within(dialog()).getByRole("button", { name: "Vazgeç" }));
  expect(onClose).toHaveBeenCalledTimes(1);
  expect(api.createPrashna).not.toHaveBeenCalled();
});

test("the form texts exist in three languages and the classical word is the same in each", () => {
  for (const lang of [tr, en, fr]) {
    expect(lang.prashna.form.title).toContain("prashna");
    expect(lang.shortcuts.newPrashna).toBe(lang.prashna.form.title);
    expect(lang.palette.command.newPrashna).toBe(lang.prashna.form.title);
    expect(Object.keys(lang.prashna.form.error).sort()).toEqual(["at", "atMissing", "focus_house", "number", "place", "placeMissing", "question"]);
  }
  // Fransızca metinler tipografik kesme işareti kullanır.
  expect(JSON.stringify(fr.prashna)).not.toContain("'");
});
````

**Yeni dosya:** `app/ui/prashna/model.test.ts`

````ts
import { expect, test } from "vitest";
import { fixture } from "../test/fixture";
import { draftPlace, emptyDraft, errorField, MAX_QUESTION_CHARS, PRASHNA_NUMBER_MAX, toRequest, type PrashnaDraft } from "./model";

const home = () => fixture().muhurta.home_place;
const prefsWithHome = () => ({ ...fixture().prefs.prefs, home_place: home() });

test("the limits are the ones Rust checks", () => {
  const { prashna_ids } = fixture();
  expect(PRASHNA_NUMBER_MAX).toBe(prashna_ids.max_number);
  expect(MAX_QUESTION_CHARS).toBe(prashna_ids.max_question_chars);
});

test("the empty draft is a time prashna now, at the current place, for the open client", () => {
  const { opened } = fixture();
  expect(emptyDraft(opened)).toEqual({ moment: "now", date: "", time: "", source: "home", searched: null, number: "", question: "", focusHouse: "", toOpenClient: true });
  expect(emptyDraft(null).toOpenClient).toBe(false);
  const checked = toRequest(emptyDraft(opened), prefsWithHome(), opened);
  expect(checked.request).toEqual({ client_id: opened.client.id, at: "Now", place: home(), number: null, question: "", focus_house: null });
});

test("the place comes from the chosen source and is never silently replaced", () => {
  const { opened } = fixture();
  const draft = emptyDraft(opened);
  // Bulunulan yer seçilmemişse haritanın yerine düşülmez: form yer ister.
  expect(draftPlace(draft, fixture().prefs.prefs, opened)).toBeNull();
  expect(draftPlace(draft, null, opened)).toBeNull();
  expect(toRequest(draft, fixture().prefs.prefs, opened)).toEqual({ request: null, errors: { place: "prashna.form.error.placeMissing" } });
  const chart = { ...draft, source: "chart" as const };
  expect(draftPlace(chart, null, opened)).toEqual({ place: opened.chart_record.birth.place, tz: opened.chart_record.birth.tz });
  expect(draftPlace(chart, null, null)).toBeNull();
  const searched = { ...draft, source: "search" as const };
  expect(draftPlace(searched, prefsWithHome(), opened)).toBeNull();
  expect(draftPlace({ ...searched, searched: home() }, null, null)).toEqual(home());
});

test("a typed moment, a number, a question and a focus house go into the request as typed", () => {
  const { opened } = fixture();
  const draft: PrashnaDraft = { ...emptyDraft(opened), moment: "local", date: "2026-11-27", time: "14:05", number: " 54 ", question: "  Deneme sorusu ", focusHouse: "7", toOpenClient: false };
  expect(toRequest(draft, prefsWithHome(), opened).request).toEqual({
    client_id: null,
    at: { Local: { date: "2026-11-27", time: "14:05" } },
    place: home(),
    number: 54,
    // Kırpma Rust'ın işidir; metin yazıldığı gibi gider.
    question: "  Deneme sorusu ",
    focus_house: 7,
  });
  // Açık danışan yoksa kutu işaretli olsa da kimlik gitmez.
  expect(toRequest({ ...draft, toOpenClient: true }, prefsWithHome(), null).request?.client_id).toBeNull();
});

test("format errors name the field; the values themselves are checked in Rust", () => {
  const { opened } = fixture();
  const base = emptyDraft(opened);
  const errors = (patch: Partial<PrashnaDraft>) => toRequest({ ...base, ...patch }, prefsWithHome(), opened).errors;
  for (const number of ["0", "109", "1.5", "-3", "on", "1e2", "0108"]) expect(errors({ number })).toEqual({ number: "prashna.form.error.number" });
  for (const number of ["1", "9", "108", ""]) expect(errors({ number })).toBeNull();
  expect(errors({ moment: "local" })).toEqual({ at: "prashna.form.error.atMissing" });
  expect(errors({ moment: "local", date: "2026-11-27" })).toEqual({ at: "prashna.form.error.atMissing" });
  // "Şimdi" seçiliyken yazılı tarih ve saat yok sayılır.
  expect(toRequest({ ...base, date: "2026-11-27" }, prefsWithHome(), opened).request?.at).toBe("Now");
  // Sınır karakterle sayılır (çok baytlı harfler dahil) ve kırpılmış metne uygulanır.
  expect(errors({ question: ` ${"ş".repeat(MAX_QUESTION_CHARS)} ` })).toBeNull();
  expect(errors({ question: "ş".repeat(MAX_QUESTION_CHARS + 1) })).toEqual({ question: "prashna.form.error.question" });
  // Birden çok hata birlikte döner.
  expect(toRequest({ ...base, number: "0", moment: "local" }, fixture().prefs.prefs, opened).errors).toEqual({
    place: "prashna.form.error.placeMissing",
    at: "prashna.form.error.atMissing",
    number: "prashna.form.error.number",
  });
});

test("a field error from Rust maps to its form field; anything else is a form error", () => {
  for (const field of ["at", "place", "number", "question", "focus_house"]) {
    expect(errorField({ code: "invalid_prashna_input", params: { field } })).toBe(field);
  }
  expect(errorField({ code: "invalid_prashna_input", params: { field: "lagna" } })).toBeNull();
  expect(errorField({ code: "not_found", params: { what: "client", id: "9" } })).toBeNull();
  expect(errorField({ code: "store_sqlite", params: { detail: "x" } })).toBeNull();
});
````

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
```
Expected: `Test Files  4 failed | 78 passed (82)`, `Tests  2 failed | 805 passed (807)` (iki yeni dosya içe aktaramaz; `shortcuts.test.ts` ve `App.test.tsx` birer testte düşer).

- [ ] **Step 3: Uygulama**

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index a60943d..f23103d 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -504,6 +504,48 @@ export const en: Messages = {
     capped: "Windows shown: the first {{shown}} of {{n}}. Narrow the filter or shorten the range.",
     note: "The limbs and the lagna are for the moment the window starts. Windows are facts, not a verdict on suitability.",
   },
+  prashna: {
+    focusNone: "Not chosen",
+    focusHouse: "House {{house}}",
+    form: {
+      title: "New prashna",
+      moment: "Moment of the question",
+      moment_now: "Now",
+      moment_local: "Another moment",
+      date: "Date",
+      time: "Time",
+      atHint: "Wall-clock time at the place of the question.",
+      place: "Place",
+      source_home: "Current place",
+      source_chart: "Place of the open chart",
+      source_search: "Search another place",
+      search: "Search a place",
+      placeCurrent: "{{place}} · {{zone}}",
+      placeMissing_home: "No current place is set. Choose another source or set the place from the day panchang panel.",
+      placeMissing_chart: "No chart is open.",
+      placeMissing_search: "No place chosen yet.",
+      number: "Number (optional)",
+      numberHint: "1–{{max}}. Empty is a time prashna; with a number the lagna comes from the number (middle of the navamsa) and the grahas are those of the moment.",
+      question: "Question",
+      questionHint: "{{count}} / {{max}} characters. The question is not searchable and is exported only together with the notes.",
+      focus: "Focus house",
+      focusHint: "You choose the house of the question; it can be changed later from the panel.",
+      toOpenClient: "Add to the client “{{name}}”",
+      bookHint: "If unchecked, the prashna is kept under the built-in client named “Prashna”.",
+      bookOnly: "No client is open: the prashna is kept under the built-in client named “Prashna”.",
+      submit: "Cast prashna",
+      failed: "The prashna could not be cast: {{message}}",
+      error: {
+        placeMissing: "Choose a place.",
+        atMissing: "Enter the date and the time, or choose “Now”.",
+        at: "This moment could not be resolved at this place. Check the date and the time; for a time inside a daylight-saving change, cast it a minute earlier and correct the chart with ⌘E.",
+        place: "This place cannot be used (its coordinates or time zone could not be resolved). Choose another place.",
+        number: "The number must be a whole number from 1 to 108.",
+        question: "The question can be at most 2000 characters.",
+        focus_house: "The focus house must be from 1 to 12.",
+      },
+    },
+  },
   arudha: {
     caption: "Arudha padas: the pada of each of the twelve houses, its sign and the lord of the house",
     pada: "Pada",
@@ -728,6 +770,7 @@ export const en: Messages = {
     print: "Open the print preview; print from the preview",
     selectLayout: "Switch to the layout at that position",
     toggleNotes: "Show or hide the notes panel",
+    newPrashna: "New prashna",
     help: "Open this help",
     escape: "Clear the highlight, close a window",
     movePanel: "Move the focused panel",
@@ -851,6 +894,7 @@ export const en: Messages = {
     togglePanel: "Show or hide the {{name}} panel",
     command: {
       newChart: "New client and chart",
+      newPrashna: "New prashna",
       addChart: "Add a chart to the open client",
       editChart: "Edit the open chart",
       settings: "Settings",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 4c60ef7..3b8bae6 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -504,6 +504,48 @@ export const fr: Messages = {
     capped: "Fenêtres affichées : les {{shown}} premières sur {{n}}. Resserrez le filtre ou raccourcissez l'intervalle.",
     note: "Les membres et le lagna sont ceux de l'instant où la fenêtre commence. Les fenêtres sont des faits, non un jugement de convenance.",
   },
+  prashna: {
+    focusNone: "Non choisie",
+    focusHouse: "Maison {{house}}",
+    form: {
+      title: "Nouveau prashna",
+      moment: "Moment de la question",
+      moment_now: "Maintenant",
+      moment_local: "Un autre moment",
+      date: "Date",
+      time: "Heure",
+      atHint: "Heure locale du lieu de la question.",
+      place: "Lieu",
+      source_home: "Lieu actuel",
+      source_chart: "Lieu du thème ouvert",
+      source_search: "Chercher un autre lieu",
+      search: "Chercher un lieu",
+      placeCurrent: "{{place}} · {{zone}}",
+      placeMissing_home: "Aucun lieu actuel n’est défini. Choisissez une autre source ou définissez le lieu depuis le panneau du panchang du jour.",
+      placeMissing_chart: "Aucun thème n’est ouvert.",
+      placeMissing_search: "Aucun lieu n’est encore choisi.",
+      number: "Nombre (facultatif)",
+      numberHint: "1–{{max}}. Vide : prashna du moment ; avec un nombre, le lagna vient du nombre (milieu du navamsa) et les grahas sont ceux du moment.",
+      question: "Question",
+      questionHint: "{{count}} / {{max}} caractères. La question n’apparaît pas dans la recherche et n’est exportée qu’avec les notes.",
+      focus: "Maison de la question",
+      focusHint: "C’est vous qui choisissez la maison de la question ; elle peut être changée ensuite depuis le panneau.",
+      toOpenClient: "Ajouter au consultant « {{name}} »",
+      bookHint: "Si la case n’est pas cochée, le prashna est rangé sous le consultant intégré nommé « Prashna ».",
+      bookOnly: "Aucun consultant n’est ouvert : le prashna est rangé sous le consultant intégré nommé « Prashna ».",
+      submit: "Établir le prashna",
+      failed: "Le prashna n’a pas pu être établi : {{message}}",
+      error: {
+        placeMissing: "Choisissez un lieu.",
+        atMissing: "Saisissez la date et l’heure, ou choisissez « Maintenant ».",
+        at: "Ce moment n’a pas pu être résolu pour ce lieu. Vérifiez la date et l’heure ; pour une heure tombant dans un changement d’heure, établissez le thème une minute plus tôt puis corrigez-le avec ⌘E.",
+        place: "Ce lieu est inutilisable (coordonnées ou fuseau horaire non résolus). Choisissez un autre lieu.",
+        number: "Le nombre doit être un entier de 1 à 108.",
+        question: "La question ne peut pas dépasser 2000 caractères.",
+        focus_house: "La maison de la question doit être comprise entre 1 et 12.",
+      },
+    },
+  },
   arudha: {
     caption: "Arudha padas : le pada de chacune des douze maisons, son signe et le maître de la maison",
     pada: "Pada",
@@ -728,6 +770,7 @@ export const fr: Messages = {
     print: "Ouvrir l’aperçu avant impression ; imprimer depuis l’aperçu",
     selectLayout: "Passer à la disposition de ce rang",
     toggleNotes: "Afficher ou masquer les notes",
+    newPrashna: "Nouveau prashna",
     help: "Ouvrir cette aide",
     escape: "Effacer la mise en évidence, fermer une fenêtre",
     movePanel: "Déplacer le panneau actif",
@@ -851,6 +894,7 @@ export const fr: Messages = {
     togglePanel: "Afficher ou masquer le panneau {{name}}",
     command: {
       newChart: "Nouveau client et carte",
+      newPrashna: "Nouveau prashna",
       addChart: "Ajouter une carte au client ouvert",
       editChart: "Modifier la carte ouverte",
       settings: "Réglages",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index a87aae8..73c1411 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -503,6 +503,48 @@ export const tr = {
     capped: "İlk {{shown}} pencere gösteriliyor (toplam {{n}}). Süzgeci daraltın ya da aralığı kısaltın.",
     note: "Uzuvlar ve lagna pencerenin başlangıç anı içindir. Pencereler olgudur; uygunluk hükmü değildir.",
   },
+  prashna: {
+    focusNone: "Seçilmedi",
+    focusHouse: "{{house}}. ev",
+    form: {
+      title: "Yeni prashna",
+      moment: "Soru anı",
+      moment_now: "Şimdi",
+      moment_local: "Başka bir an",
+      date: "Tarih",
+      time: "Saat",
+      atHint: "Sorunun yerinin duvar saati.",
+      place: "Yer",
+      source_home: "Bulunulan yer",
+      source_chart: "Açık haritanın yeri",
+      source_search: "Başka bir yer ara",
+      search: "Yer ara",
+      placeCurrent: "{{place}} · {{zone}}",
+      placeMissing_home: "Bulunulan yer seçilmemiş. Başka bir kaynak seçin ya da yeri gün panchangı panelinden belirleyin.",
+      placeMissing_chart: "Açık harita yok.",
+      placeMissing_search: "Henüz yer seçilmedi.",
+      number: "Sayı (isteğe bağlı)",
+      numberHint: "1–{{max}}. Boşsa zaman prashnasıdır; sayı girilirse lagna sayıdan gelir (navamsa ortası), grahalar anın grahalarıdır.",
+      question: "Soru",
+      questionHint: "{{count}} / {{max}} karakter. Soru metni aramada görünmez ve yalnızca notlarla birlikte dışa aktarılır.",
+      focus: "Odak evi",
+      focusHint: "Sorunun evini siz seçersiniz; sonradan panelden değiştirilebilir.",
+      toOpenClient: "“{{name}}” danışanına ekle",
+      bookHint: "İşaretli değilse prashna “Prashna” adlı yerleşik danışanın altında toplanır.",
+      bookOnly: "Açık danışan yok: prashna “Prashna” adlı yerleşik danışanın altında toplanır.",
+      submit: "Prashna kur",
+      failed: "Prashna kurulamadı: {{message}}",
+      error: {
+        placeMissing: "Bir yer seçin.",
+        atMissing: "Tarihi ve saati girin ya da “Şimdi”yi seçin.",
+        at: "Bu an bu yerde çözülemedi. Tarihi ve saati denetleyin; yaz saati geçişine düşen saat için önce bir dakika öncesiyle kurup haritayı ⌘E ile düzeltin.",
+        place: "Bu yer kullanılamıyor (koordinat ya da saat dilimi çözülemedi). Başka bir yer seçin.",
+        number: "Sayı 1 ile 108 arasında bir tamsayı olmalı.",
+        question: "Soru en çok 2000 karakter olabilir.",
+        focus_house: "Odak evi 1 ile 12 arasında olmalı.",
+      },
+    },
+  },
   arudha: {
     caption: "Arudha padalar: on iki evin padası, burcu ve evin lordu",
     pada: "Pada",
@@ -727,6 +769,7 @@ export const tr = {
     print: "Baskı önizlemesini aç; önizlemede yazdır",
     selectLayout: "Sıradaki düzene geç",
     toggleNotes: "Notlar panelini aç/kapat",
+    newPrashna: "Yeni prashna",
     help: "Bu yardımı aç",
     escape: "Vurguyu temizle, pencereyi kapat",
     movePanel: "Odaktaki paneli taşı",
@@ -850,6 +893,7 @@ export const tr = {
     togglePanel: "{{name}} panelini aç/kapat",
     command: {
       newChart: "Yeni danışan ve harita",
+      newPrashna: "Yeni prashna",
       addChart: "Açık danışana harita ekle",
       editChart: "Açık haritayı düzenle",
       settings: "Ayarlar",
diff --git a/app/ui/palette/Palette.tsx b/app/ui/palette/Palette.tsx
index c2f16a2..8bf529a 100644
--- a/app/ui/palette/Palette.tsx
+++ b/app/ui/palette/Palette.tsx
@@ -90,6 +90,8 @@ export function Palette({ onClose, onRun }: PaletteProps) {
   const items = useMemo(() => {
     const commands: { command: PaletteCommand; label: string }[] = [
       { command: "newChart", label: t("palette.command.newChart") },
+      // Açık harita gerekmez: danışan seçili değilse prashna yerleşik danışana eklenir.
+      { command: "newPrashna", label: t("palette.command.newPrashna") },
       ...(opened
         ? [
             { command: "addChart" as const, label: t("palette.command.addChart") },
diff --git a/app/ui/palette/items.ts b/app/ui/palette/items.ts
index 984cd8f..2ea2c03 100644
--- a/app/ui/palette/items.ts
+++ b/app/ui/palette/items.ts
@@ -4,7 +4,7 @@
 import type { ChartRecord, SearchHit, Varga } from "../bindings";
 import type { Layout, ModulePanelKind } from "../layout/model";
 
-export type PaletteCommand = "newChart" | "addChart" | "editChart" | "settings" | "print" | "toggleNotes" | "sidebar" | "help" | "about";
+export type PaletteCommand = "newChart" | "newPrashna" | "addChart" | "editChart" | "settings" | "print" | "toggleNotes" | "sidebar" | "help" | "about";
 
 export type PaletteItem =
   | { kind: "client"; key: string; label: string; detail: string | null; clientId: number }
diff --git a/app/ui/shell/Shell.tsx b/app/ui/shell/Shell.tsx
index 25312ca..7bb3af5 100644
--- a/app/ui/shell/Shell.tsx
+++ b/app/ui/shell/Shell.tsx
@@ -19,6 +19,7 @@ import { leaveNotes, leavePanel, leaveSidebar, MAIN_ID } from "./focus";
 import { HelpDialog } from "./HelpDialog";
 import { NoticeBar, useNotices } from "./notices";
 import { loadSidebarOpen, saveSidebarOpen } from "./sidebarPref";
+import { PrashnaForm } from "../prashna/PrashnaForm";
 import { PrintView } from "../print/PrintView";
 import "./shell.css";
 import { TopBar } from "./TopBar";
@@ -34,6 +35,7 @@ export function Shell() {
   const [settings, setSettings] = useState(false);
   const [about, setAbout] = useState(false);
   const [form, setForm] = useState<"new" | "add" | "edit" | null>(null);
+  const [prashna, setPrashna] = useState(false);
   const [sidebar, setSidebar] = useState(() => loadSidebarOpen());
   // Açık/kapalı durumu bir sonraki açılışta da geçerlidir (Plan 12 D11).
   useEffect(() => saveSidebarOpen(sidebar), [sidebar]);
@@ -120,6 +122,10 @@ export function Shell() {
       } else if (command?.id === "newChart") {
         e.preventDefault();
         setForm("new");
+      } else if (command?.id === "newPrashna") {
+        // ⌘⇧P her zaman tüketilir; baskı önizlemesinde (pano yokken) pencere açılmaz.
+        e.preventDefault();
+        if (!now.printView) setPrashna(true);
       } else if (command?.id === "editChart" && now.opened) {
         e.preventDefault();
         setForm("edit");
@@ -167,6 +173,7 @@ export function Shell() {
           return setForm(item.command === "newChart" ? "new" : item.command === "addChart" ? "add" : "edit");
         }
         // Kısayollarındaki gibi: baskı görünümünde (pano ve kenar çubuğu yokken) yok sayılır.
+        if (item.command === "newPrashna") return printView ? undefined : setPrashna(true);
         if (item.command === "settings") return setSettings(true);
         if (item.command === "print") return printView ? undefined : togglePrint();
         if (item.command === "toggleNotes") {
@@ -208,6 +215,7 @@ export function Shell() {
       {settings && <SettingsDialog onClose={() => setSettings(false)} />}
       {about && <AboutDialog onClose={() => setAbout(false)} />}
       {form && <BirthForm mode={form} onClose={() => setForm(null)} />}
+      {prashna && <PrashnaForm onClose={() => setPrashna(false)} />}
     </div>
   );
 }
diff --git a/app/ui/shortcuts.ts b/app/ui/shortcuts.ts
index dc34b1b..5617ddd 100644
--- a/app/ui/shortcuts.ts
+++ b/app/ui/shortcuts.ts
@@ -7,6 +7,7 @@ export type Command =
   | { id: "selectLayout"; index: number }
   | { id: "toggleNotes" }
   | { id: "newChart" }
+  | { id: "newPrashna" }
   | { id: "editChart" }
   | { id: "settings" }
   | { id: "print" }
@@ -16,7 +17,7 @@ export type Command =
 
 export interface ShortcutInfo {
   /** i18n anahtarı: `shortcuts.<id>`. */
-  id: "palette" | "sidebar" | "selectLayout" | "toggleNotes" | "newChart" | "editChart" | "settings" | "print" | "help" | "escape" | "movePanel" | "resizePanel" | "tree";
+  id: "palette" | "sidebar" | "selectLayout" | "toggleNotes" | "newChart" | "newPrashna" | "editChart" | "settings" | "print" | "help" | "escape" | "movePanel" | "resizePanel" | "tree";
   keys: string;
 }
 
@@ -29,6 +30,7 @@ export const SHORTCUT_LIST: readonly ShortcutInfo[] = [
   { id: "print", keys: "⌘P" },
   { id: "selectLayout", keys: "⌘1 … ⌘9" },
   { id: "toggleNotes", keys: "⌘⇧N" },
+  { id: "newPrashna", keys: "⌘⇧P" },
   { id: "help", keys: "?" },
   { id: "escape", keys: "Esc" },
   { id: "movePanel", keys: "⌥ ←↑→↓" },
@@ -55,7 +57,11 @@ export function matchShortcut(e: KeyboardEvent): Command | null {
   if (e.repeat || e.isComposing || e.keyCode === 229 || modalOpen()) return null;
   if (e.ctrlKey || e.altKey) return null;
   if (e.metaKey) {
-    if (e.shiftKey) return e.key.toLowerCase() === "n" ? { id: "toggleNotes" } : null;
+    if (e.shiftKey) {
+      const shifted = e.key.toLowerCase();
+      // ⌘⇧P "Yeni prashna"dır (ileri modüller spec'i §11); ⌘P baskıdır.
+      return shifted === "n" ? { id: "toggleNotes" } : shifted === "p" ? { id: "newPrashna" } : null;
+    }
     // Önizlemenin ardındaki düzen görünmeden değişmesin (Plan 04 incelemesi M4).
     if (/^Digit[1-9]$/.test(e.code)) return printPreviewOpen() ? null : { id: "selectLayout", index: Number(e.code.slice(5)) - 1 };
     if (e.code === "Backslash" || e.key === "\\") return { id: "sidebar" };
````

**Yeni dosya:** `app/ui/prashna/PrashnaForm.tsx`

````tsx
// "Yeni prashna" (ileri modüller spec'i §11): sağdan kayan modal pencere. An (varsayılan şimdi),
// yer (varsayılan bulunulan yer), isteğe bağlı sayı, soru metni ve odak evi. "Prashna kur"
// `create_prashna`'yı çağırır; dönen harita `session.open` ile açılır. An ve yer Rust'ta çözülür,
// lagna Rust'ta kurulur: burada hesap yoktur (C4). Soru metni, yer ve an loglanmaz.
// Değişmiş taslakta Esc onay sorar; "Vazgeç" düğmesi sormadan kapatır.
import { useEffect, useId, useRef, useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CommandError } from "../bindings";
import "../birth/birth.css";
import { PlaceSearch } from "../birth/PlaceSearch";
import { errorMessage } from "../i18n/errors";
import { hitPlace, placeLabel } from "../modules/muhurta";
import { tzLabel } from "../panels/HomePlaceDialog";
import { useSession } from "../session/store";
import { usePrefs } from "../settings/prefs";
import { ConfirmDialog } from "../shell/ConfirmDialog";
import { Dialog } from "../shell/Dialog";
import { draftPlace, emptyDraft, errorField, MAX_QUESTION_CHARS, PRASHNA_NUMBER_MAX, toRequest, type DraftErrors, type PrashnaDraft, type PrashnaField, type PrashnaPlaceSource } from "./model";
import "./prashna.css";

const HOUSES = Array.from({ length: 12 }, (_, i) => i + 1);

export interface PrashnaFormProps {
  onClose(): void;
  /** Harita kurulup açıldıktan sonra (ör. Prashna düzenine geçmek için). */
  onCreated?(): void;
}

export function PrashnaForm({ onClose, onCreated }: PrashnaFormProps) {
  const { t, i18n } = useTranslation();
  const session = useSession();
  const { prefs } = usePrefs();
  const id = useId();
  // Form, açıldığı andaki danışana yazar: form açıkken geç gelen bir `open_client` yanıtı açık
  // haritayı değiştirse de prashna başka danışana gitmez.
  const [target] = useState(() => session.opened);
  const [start] = useState<PrashnaDraft>(() => emptyDraft(target));
  const [draft, setDraft] = useState<PrashnaDraft>(start);
  const [searchText, setSearchText] = useState("");
  const dirty = JSON.stringify(draft) !== JSON.stringify(start);
  const [confirmClose, setConfirmClose] = useState(false);
  const [shown, setShown] = useState<DraftErrors>({});
  const [serverField, setServerField] = useState<PrashnaField | null>(null);
  const [formError, setFormError] = useState<CommandError | null>(null);
  const [saving, setSaving] = useState(false);
  const formRef = useRef<HTMLFormElement>(null);
  const focusInvalid = useRef(false);

  const place = draftPlace(draft, prefs, target);
  const update = (patch: Partial<PrashnaDraft>, field?: PrashnaField) => {
    setDraft((d) => ({ ...d, ...patch }));
    // Alan düzeltilince hatası hemen kalkar.
    if (field) {
      setShown((s) => ({ ...s, [field]: undefined }));
      setServerField((f) => (f === field ? null : f));
    }
  };
  const fieldError = (f: PrashnaField): string | null => {
    const key = shown[f] ?? (serverField === f ? `prashna.form.error.${f}` : undefined);
    return key ? t(key) : null;
  };
  const errorCount = (["at", "place", "number", "question", "focus_house"] as const).filter((f) => fieldError(f) !== null).length;

  // Gönderimden sonra odak ilk hatalı alana gider.
  useEffect(() => {
    if (!focusInvalid.current) return;
    focusInvalid.current = false;
    formRef.current?.querySelector<HTMLElement>('[aria-invalid="true"]')?.focus();
  });

  const requestClose = () => (dirty ? setConfirmClose(true) : onClose());

  const submit = async (e: FormEvent) => {
    e.preventDefault();
    if (saving) return;
    setFormError(null);
    setServerField(null);
    const checked = toRequest(draft, prefs, target);
    if (checked.request === null) {
      setShown(checked.errors);
      focusInvalid.current = true;
      return;
    }
    setShown({});
    setSaving(true);
    const r = await api.createPrashna(checked.request);
    setSaving(false);
    if (r.status === "ok") {
      session.open(r.data);
      onCreated?.();
      onClose();
      return;
    }
    const f = errorField(r.error);
    if (f) {
      setServerField(f);
      focusInvalid.current = true;
    } else setFormError(r.error);
  };

  const describe = (f: PrashnaField, hint?: string) => {
    const error = fieldError(f);
    return {
      "aria-invalid": error ? (true as const) : undefined,
      "aria-describedby": [error ? `${id}-${f}-error` : null, hint ? `${id}-${hint}` : null].filter(Boolean).join(" ") || undefined,
    };
  };
  const errorLine = (f: PrashnaField) => {
    const error = fieldError(f);
    return (
      error && (
        <p id={`${id}-${f}-error`} className="field-error">
          <span aria-hidden="true">▲ </span>
          {error}
        </p>
      )
    );
  };
  const sources: PrashnaPlaceSource[] = target ? ["home", "chart", "search"] : ["home", "search"];
  const questionLength = [...draft.question.trim()].length;

  return (
    <Dialog title={t("prashna.form.title")} onClose={requestClose} variant="sheet">
      <form ref={formRef} className="birth-form prashna-form" noValidate onSubmit={(e) => void submit(e)}>
        <fieldset className="form-field">
          <legend>{t("prashna.form.moment")}</legend>
          {(["now", "local"] as const).map((m, i) => (
            <label key={m} className="choice">
              <input type="radio" name={`${id}-moment`} checked={draft.moment === m} data-autofocus={i === 0 ? true : undefined} onChange={() => update({ moment: m }, "at")} />
              {t(`prashna.form.moment_${m}`)}
            </label>
          ))}
        </fieldset>
        {draft.moment === "local" && (
          <>
            <div className="form-row">
              <div className="form-field">
                <label htmlFor={`${id}-date`}>{t("prashna.form.date")}</label>
                <input id={`${id}-date`} type="date" value={draft.date} {...describe("at", "at-hint")} onChange={(e) => update({ date: e.target.value }, "at")} />
              </div>
              <div className="form-field">
                <label htmlFor={`${id}-time`}>{t("prashna.form.time")}</label>
                <input id={`${id}-time`} type="time" value={draft.time} {...describe("at", "at-hint")} onChange={(e) => update({ time: e.target.value }, "at")} />
              </div>
            </div>
            <p id={`${id}-at-hint`} className="field-hint">
              {t("prashna.form.atHint")}
            </p>
            {errorLine("at")}
          </>
        )}
        <div className="form-field">
          <label htmlFor={`${id}-source`}>{t("prashna.form.place")}</label>
          <select id={`${id}-source`} value={draft.source} {...describe("place", "place-current")} onChange={(e) => update({ source: e.target.value as PrashnaPlaceSource }, "place")}>
            {sources.map((s) => (
              <option key={s} value={s}>
                {t(`prashna.form.source_${s}`)}
              </option>
            ))}
          </select>
          {draft.source === "search" && (
            <>
              <label htmlFor={`${id}-search`}>{t("prashna.form.search")}</label>
              <PlaceSearch
                inputId={`${id}-search`}
                value={searchText}
                onChange={setSearchText}
                onSelect={(hit) => {
                  const found = hitPlace(hit);
                  if (found) {
                    update({ searched: found }, "place");
                    setSearchText("");
                  }
                }}
              />
            </>
          )}
          <p id={`${id}-place-current`} className="prashna-place" role="status">
            {place
              ? t("prashna.form.placeCurrent", { place: placeLabel(place), zone: tzLabel(place, t("muhurta.home.lmt")) })
              : t(`prashna.form.placeMissing_${draft.source}`)}
          </p>
          {errorLine("place")}
        </div>
        <div className="form-field">
          <label htmlFor={`${id}-number`}>{t("prashna.form.number")}</label>
          <input id={`${id}-number`} inputMode="numeric" value={draft.number} {...describe("number", "number-hint")} onChange={(e) => update({ number: e.target.value }, "number")} />
          <p id={`${id}-number-hint`} className="field-hint">
            {t("prashna.form.numberHint", { max: PRASHNA_NUMBER_MAX })}
          </p>
          {errorLine("number")}
        </div>
        <div className="form-field">
          <label htmlFor={`${id}-question`}>{t("prashna.form.question")}</label>
          <textarea id={`${id}-question`} rows={4} value={draft.question} {...describe("question", "question-hint")} onChange={(e) => update({ question: e.target.value }, "question")} />
          <p id={`${id}-question-hint`} className="field-hint">
            {t("prashna.form.questionHint", { count: questionLength, max: MAX_QUESTION_CHARS })}
          </p>
          {errorLine("question")}
        </div>
        <div className="form-field">
          <label htmlFor={`${id}-focus`}>{t("prashna.form.focus")}</label>
          <select id={`${id}-focus`} value={draft.focusHouse} {...describe("focus_house", "focus-hint")} onChange={(e) => update({ focusHouse: e.target.value }, "focus_house")}>
            <option value="">{t("prashna.focusNone")}</option>
            {HOUSES.map((h) => (
              <option key={h} value={h}>
                {t("prashna.focusHouse", { house: h })}
              </option>
            ))}
          </select>
          <p id={`${id}-focus-hint`} className="field-hint">
            {t("prashna.form.focusHint")}
          </p>
          {errorLine("focus_house")}
        </div>
        {target ? (
          <div className="form-field">
            <label className="choice">
              <input type="checkbox" checked={draft.toOpenClient} aria-describedby={`${id}-client-hint`} onChange={(e) => update({ toOpenClient: e.target.checked })} />
              {t("prashna.form.toOpenClient", { name: target.client.name })}
            </label>
            <p id={`${id}-client-hint`} className="field-hint">
              {t("prashna.form.bookHint")}
            </p>
          </div>
        ) : (
          <p className="field-hint">{t("prashna.form.bookOnly")}</p>
        )}
        <p className="visually-hidden" role="status">
          {errorCount > 0 ? t("birth.errorSummary", { count: errorCount }) : ""}
        </p>
        {formError && (
          <p className="field-error" role="alert">
            {t("prashna.form.failed", { message: errorMessage(t, i18n, formError) })}
          </p>
        )}
        <div className="dialog-actions">
          <button type="button" onClick={onClose}>
            {t("birth.cancel")}
          </button>
          <button type="submit" disabled={saving}>
            {t("prashna.form.submit")}
          </button>
        </div>
      </form>
      {confirmClose && (
        <ConfirmDialog
          title={t("birth.discardTitle")}
          message={t("birth.discardMessage")}
          confirmLabel={t("birth.discardConfirm")}
          onConfirm={onClose}
          onCancel={() => setConfirmClose(false)}
        />
      )}
    </Dialog>
  );
}
````

**Yeni dosya:** `app/ui/prashna/model.ts`

````ts
// "Yeni prashna" formunun taslağı ve isteğe çevrilmesi (ileri modüller spec'i §11). Burada
// astrolojik hesap yoktur (C4): an ve yer Rust'ta çözülür, lagna Rust'ta kurulur. Yalnızca alan
// biçimi denetlenir (sayı 1–108 tamsayı mı, tarih ve saat yazılmış mı). Soru metni, yer ve an
// kişisel veridir: loglanmaz.
import type { AppPrefs, CommandError, HomePlace, OpenedClient, PrashnaRequest } from "../bindings";
import { chartPlace } from "../modules/muhurta";

/** Rust `PRASHNA_NUMBER_MAX` (fikstürde `prashna_ids.max_number`). */
export const PRASHNA_NUMBER_MAX = 108;
/** Rust `MAX_QUESTION_CHARS` (fikstürde `prashna_ids.max_question_chars`). */
export const MAX_QUESTION_CHARS = 2000;

/** Yerin kaynağı: bulunulan yer (tercihlerden), açık haritanın yeri ya da atlastan aranan yer. */
export const PRASHNA_PLACE_SOURCES = ["home", "chart", "search"] as const;
export type PrashnaPlaceSource = (typeof PRASHNA_PLACE_SOURCES)[number];

export type PrashnaField = "at" | "place" | "number" | "question" | "focus_house";
export const PRASHNA_FIELDS: readonly PrashnaField[] = ["at", "place", "number", "question", "focus_house"];

export interface PrashnaDraft {
  /** `now`: an Rust'ta okunur; `local`: sorunun yerinin duvar saati. */
  moment: "now" | "local";
  date: string;
  time: string;
  source: PrashnaPlaceSource;
  /** Atlastan seçilen yer (`source === "search"`). */
  searched: HomePlace | null;
  /** Yazıldığı gibi; boşsa zaman prashnasıdır. */
  number: string;
  question: string;
  /** Boş dizgi: odak evi seçilmedi. */
  focusHouse: string;
  /** Açık danışana ekle; işaretli değilse (ya da açık danışan yoksa) yerleşik "Prashna" danışanı. */
  toOpenClient: boolean;
}

export function emptyDraft(opened: OpenedClient | null): PrashnaDraft {
  return {
    moment: "now",
    date: "",
    time: "",
    // Bulunulan yer seçilmemişse haritanın yerine sessizce düşülmez: form yer ister.
    source: "home",
    searched: null,
    number: "",
    question: "",
    focusHouse: "",
    toOpenClient: opened !== null,
  };
}

/** Seçilen kaynağın yeri; yoksa `null` (form yer ister). */
export function draftPlace(draft: PrashnaDraft, prefs: AppPrefs | null, opened: OpenedClient | null): HomePlace | null {
  if (draft.source === "home") return prefs?.home_place ?? null;
  if (draft.source === "chart") return opened ? chartPlace(opened) : null;
  return draft.searched;
}

export type DraftErrors = Partial<Record<PrashnaField, string>>;

/**
 * Taslağı isteğe çevirir. Biçim hataları alan başına bir i18n anahtarıdır (`prashna.form.error.*`);
 * değerlerin anlamı (an o yerde var mı, yer çözülüyor mu) Rust'ta denetlenir.
 */
export function toRequest(
  draft: PrashnaDraft,
  prefs: AppPrefs | null,
  opened: OpenedClient | null,
): { request: PrashnaRequest; errors: null } | { request: null; errors: DraftErrors } {
  const errors: DraftErrors = {};
  const place = draftPlace(draft, prefs, opened);
  if (place === null) errors.place = "prashna.form.error.placeMissing";
  if (draft.moment === "local" && (draft.date.trim() === "" || draft.time.trim() === "")) errors.at = "prashna.form.error.atMissing";
  const typed = draft.number.trim();
  let number: number | null = null;
  if (typed !== "") {
    if (/^\d{1,3}$/.test(typed) && Number(typed) >= 1 && Number(typed) <= PRASHNA_NUMBER_MAX) number = Number(typed);
    else errors.number = "prashna.form.error.number";
  }
  if ([...draft.question.trim()].length > MAX_QUESTION_CHARS) errors.question = "prashna.form.error.question";
  if (place === null || Object.keys(errors).length > 0) return { request: null, errors };
  return {
    request: {
      client_id: draft.toOpenClient && opened ? opened.client.id : null,
      at: draft.moment === "now" ? "Now" : { Local: { date: draft.date.trim(), time: draft.time.trim() } },
      place,
      number,
      question: draft.question,
      focus_house: draft.focusHouse === "" ? null : Number(draft.focusHouse),
    },
    errors: null,
  };
}

/** Rust'ın alan adlı hatasının (`invalid_prashna_input { field }`) formdaki alanı; başka hata `null`. */
export function errorField(error: CommandError): PrashnaField | null {
  if (error.code !== "invalid_prashna_input") return null;
  const field = error.params.field;
  return (PRASHNA_FIELDS as readonly string[]).includes(field) ? (field as PrashnaField) : null;
}
````

**Yeni dosya:** `app/ui/prashna/prashna.css`

````css
/* "Yeni prashna" formu ve prashna olguları paneli. Renk yalnızca belirteçlerden gelir. */
.prashna-form textarea {
  resize: vertical;
}

.prashna-place {
  margin: 4px 0 0;
  color: var(--text-2);
  font-size: 12px;
}
````

- [ ] **Step 4: Testleri ve kapıları koş**

```bash
pnpm --dir app exec vitest run ui/prashna ui/shortcuts.test.ts ui/App.test.tsx ui/i18n 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
grep -rnE 'localStorage|sessionStorage|new Date\(|Date\.now|console\.' app/ui/prashna
```
Expected: `Test Files  5 passed (5)`; tamamı `82 passed (82)`, `822 passed (822)`; `✓ built`; `grep` çıktısız (pencere saat okumaz ve log yazmaz: "şimdi" Rust'ta okunur).

- [ ] **Step 5: Commit**

```bash
git add -A
git status --short
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached -U0 -- 'app/ui/**/*.css' | grep '^+' | grep -E '#[0-9a-fA-F]{3,8}\b|rgb\('
git commit -m "feat(ui): new prashna sheet with the moment, place, number, question and focus house; Cmd+Shift+P and a palette command" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: beş yeni dosya; iki `grep` çıktısız.

---

### Task 6: Arayüz: `prashna_facts` paneli, Prashna hazır düzeni ve sayı rozeti

Spec §11 "Arayüz" (ikinci, üçüncü ve dördüncü madde); §13 (panel durumları); K11, K12; UI spec §3.2–§3.4, §5, §6; D20–D23. Panel hesap yapmaz.

**Files:**
- Create: `app/ui/modules/usePrashnaFacts.ts`, `app/ui/panels/PrashnaFactsPanel.tsx`, `app/ui/panels/PrashnaFactsPanel.test.tsx`
- Modify: `app/ui/panels/registry.ts`, `app/ui/panels/YogasPanel.tsx`, `app/ui/panels/labels.ts`, `app/ui/layout/model.ts`, `app/ui/layout/model.test.ts`, `app/ui/layout/reducer.test.ts`, `app/ui/session/store.tsx`, `app/ui/session/store.test.tsx`, `app/ui/shell/Shell.tsx`, `app/ui/shell/ChartTitle.tsx`, `app/ui/shell/ChartTitle.test.tsx`, `app/ui/shell/LayoutPicker.test.tsx`, `app/ui/view/store.test.tsx`, `app/ui/palette/items.test.ts`, `app/ui/App.test.tsx`, `app/ui/prashna/prashna.css`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`
- Test: `PrashnaFactsPanel.test.tsx` (15), `ChartTitle.test.tsx` (+1), `session/store.test.tsx` (+1), `App.test.tsx` (+1); dördüncü hazır düzenin değiştirdiği beklentiler (`layout/model.test.ts`, `layout/reducer.test.ts`, `LayoutPicker.test.tsx`, `view/store.test.tsx`, `palette/items.test.ts`)

**Interfaces:**
- Consumes (Task 4, 5; Plan 14, 18, 20): `api.computePrashnaFacts`, `api.updatePrashna`, `PrashnaFacts`, `PrashnaMeta`; `chartInput`, `inputKey`, `useModule`, `useLastData`; `panchangRows` (`PanchangPanel`), `PinButton`, `PanelError`, `PanelSkeleton`, `usePanelLoadPhase`, `useHighlight`; `formatDegree`; `MAX_QUESTION_CHARS`.
- Produces:
  - `modules/usePrashnaFacts.ts`: `usePrashnaFacts(opened, focusHouse): ModuleState<PrashnaFacts>` (prashna olmayan haritada komut çağırmaz).
  - `panels/PrashnaFactsPanel.tsx`: `PrashnaFactsPanel`, `QUESTION_SAVE_MS`; `panels/YogasPanel.tsx`: `YogaList` dışa aktarılır; `panels/labels.ts`: `RISING_NAMES`.
  - Panel türü `prashna_facts` (`PANEL_KINDS`, `MODULE_PANELS` 7 × 20, `PANEL_REGISTRY`); hazır düzen `BUILTIN_LAYOUT_IDS.prashna` (`builtin:prashna`).
  - `Session.setPrashna(chartId, meta)`; harita başlığında `.topbar-number`; `PrashnaForm`'un `onCreated`'ı Prashna düzenini seçer.
  - i18n: `panel.prashna_facts`, `layout.prashna`, `prashna.*` (panel metinleri, `nature`, `kind`, `rising`, `link`, `lagna`, `lords`, `lord`, `relation`, `houses`, `moon`, `moment`).

- [ ] **Step 1: Testler**

`PrashnaFactsPanel.test.tsx`: prashna olmayan haritada not ve komutsuzluk; açık harita yokken ileti; dolu durum (bölümler, tablolar, anın panchangı, yoga listesi, hüküm sözünün yokluğu, adlı ve odaklanır kaydırma kapları); sayı prashnası; odak evinin panelden değişmesi (kayıt, oturum, yerinde yeniden okuma, `aria-busy`, odağın seçicide durması); odak evi seçilmemiş durum; bağların listesi ve aynı graha; retro ve yanık lord; sorunun otomatik kaydı ve çok uzun soru; panel giderken bekleyen metin ve kayıt hatası; okuma hatası ve hata öncesi verinin yeniden denemede gösterilmemesi; saati bilinmeyen harita; ortak vurgu; başka prashnaya geçişte taslağın sıfırlanması; üç dil. Dördüncü hazır düzen var olan beş test dosyasının beklentilerini değiştirir (düzen sayısı, sıra, ⌘4'ün artık Prashna düzeni olması; kullanıcı düzenleri ⌘5'ten başlar).

````diff
diff --git a/app/ui/App.test.tsx b/app/ui/App.test.tsx
index 6b201a8..99b18ba 100644
--- a/app/ui/App.test.tsx
+++ b/app/ui/App.test.tsx
@@ -10,6 +10,8 @@ vi.mock("./api", () => ({
     searchPlaces: vi.fn(async () => ({ status: "ok", data: [] })),
     createClient: vi.fn(),
     createPrashna: vi.fn(),
+    computePrashnaFacts: vi.fn(async () => ({ status: "ok", data: (await import("./test/fixture")).fixture().prashna.facts_number })),
+    updatePrashna: vi.fn(),
     updateChart: vi.fn(),
     saveViewState: vi.fn(async () => ({ status: "ok", data: null })),
     savePanelLayout: vi.fn(async () => ({ status: "ok", data: null })),
@@ -585,3 +587,27 @@ test("Cmd+Shift+P and the palette open the new prashna sheet, with or without an
   expect(screen.queryByRole("dialog", { name: "Yeni prashna" })).toBeNull();
   expect(api.createPrashna).not.toHaveBeenCalled();
 });
+
+test("casting a prashna opens it on the Prashna layout: D1, facts, panchang and notes", async () => {
+  const { prashna } = fixture();
+  vi.mocked(api.createPrashna).mockResolvedValueOnce({ status: "ok", data: prashna.opened_number });
+  const home = fixture().muhurta.home_place;
+  renderApp({ prefs: { prefs: { ...fixture().prefs.prefs, home_place: home }, warnings: [] } });
+  expect(screen.getByRole("combobox", { name: "Düzen" })).toHaveDisplayValue("Görüşme");
+  fireEvent.keyDown(window, { key: "P", metaKey: true, shiftKey: true });
+  const sheet = screen.getByRole("dialog", { name: "Yeni prashna" });
+  fireEvent.change(within(sheet).getByLabelText("Sayı (isteğe bağlı)"), { target: { value: "54" } });
+  fireEvent.click(within(sheet).getByRole("button", { name: "Prashna kur" }));
+  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
+  expect(vi.mocked(api.createPrashna).mock.calls[0][0]).toMatchObject({ client_id: 1, at: "Now", number: 54, place: home });
+  expect(screen.getByRole("combobox", { name: "Düzen" })).toHaveDisplayValue("Prashna");
+  expect(screen.getByText("Prashna 2026-11-25 15:30")).toBeInTheDocument();
+  expect(screen.getByText("sayı: 54", { selector: ".topbar-number" })).toBeInTheDocument();
+  expect(screen.getByRole("group", { name: "D1 haritası" })).toBeInTheDocument();
+  for (const name of ["Prashna olguları", "Panchang", "Notlar"]) expect(screen.getByRole("region", { name })).toBeInTheDocument();
+  // ⌘4 de aynı düzendir (dördüncü hazır düzen).
+  fireEvent.keyDown(window, { key: "1", code: "Digit1", metaKey: true });
+  expect(screen.getByRole("combobox", { name: "Düzen" })).toHaveDisplayValue("Görüşme");
+  fireEvent.keyDown(window, { key: "4", code: "Digit4", metaKey: true });
+  expect(screen.getByRole("combobox", { name: "Düzen" })).toHaveDisplayValue("Prashna");
+});
diff --git a/app/ui/layout/model.test.ts b/app/ui/layout/model.test.ts
index 8489460..fa94c8e 100644
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -18,7 +18,7 @@ import {
 import parityRaw from "./fixtures/layout-parity.json?raw";
 
 const ids = (layouts: Layout[]) => layouts.map((l) => l.id);
-const BUILTINS = [BUILTIN_LAYOUT_IDS.interview, BUILTIN_LAYOUT_IDS.research, BUILTIN_LAYOUT_IDS.transit];
+const BUILTINS = [BUILTIN_LAYOUT_IDS.interview, BUILTIN_LAYOUT_IDS.research, BUILTIN_LAYOUT_IDS.transit, BUILTIN_LAYOUT_IDS.prashna];
 
 function overlaps(a: Panel, b: Panel) {
   return a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
@@ -45,6 +45,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
     "compat",
     "muhurta_day",
     "muhurta_search",
+    "prashna_facts",
   ]);
   expect(isKnownPanelKind("chart")).toBe(true);
   expect(isKnownPanelKind("drishti")).toBe(true);
@@ -52,23 +53,26 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
 });
 
 test("module panels are known kinds, listed once, with a size the grid accepts", () => {
-  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "yogas", "compat", "muhurta_day", "muhurta_search"]);
+  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "yogas", "compat", "muhurta_day", "muhurta_search", "prashna_facts"]);
   for (const m of MODULE_PANELS) {
     expect(isKnownPanelKind(m.kind)).toBe(true);
     expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
     expect(m.h).toBeGreaterThanOrEqual(minSize(m.kind).h);
     expect(m.w).toBeLessThanOrEqual(GRID_COLUMNS);
   }
-  // Hazır düzenler değişmedi: modül panelleri paletten eklenir.
-  for (const l of builtinLayouts()) expect(l.panels.some((p) => MODULE_PANELS.some((m) => m.kind === p.kind))).toBe(false);
+  // v1'in üç hazır düzeni değişmedi: modül panelleri paletten eklenir. Modül paneli taşıyan tek
+  // hazır düzen Prashna'dır (ileri modüller spec'i §11).
+  for (const l of builtinLayouts()) {
+    expect(l.panels.some((p) => MODULE_PANELS.some((m) => m.kind === p.kind))).toBe(l.id === BUILTIN_LAYOUT_IDS.prashna);
+  }
 });
 
-test("default state has the three builtin layouts with interview active", () => {
+test("default state has the four builtin layouts with interview active", () => {
   const s = defaultLayoutState();
   expect(s.schema_version).toBe(LAYOUT_SCHEMA_VERSION);
   expect(s.active_layout_id).toBe(BUILTIN_LAYOUT_IDS.interview);
   expect(ids(s.layouts)).toEqual(BUILTINS);
-  expect(s.layouts.map((l) => l.name)).toEqual(["Görüşme", "Araştırma", "Transit"]);
+  expect(s.layouts.map((l) => l.name)).toEqual(["Görüşme", "Araştırma", "Transit", "Prashna"]);
   expect(s.layouts.every((l) => l.builtin)).toBe(true);
 });
 
@@ -84,6 +88,8 @@ test("builtin presets follow the spec text and fit the 12-column grid without ov
   ]);
   expect(kinds(BUILTIN_LAYOUT_IDS.transit)).toEqual(["chart:D1", "transit_table:", "dasha_strip:"]);
   expect(builtinLayout(BUILTIN_LAYOUT_IDS.transit)!.panels[0].settings.transit_ring).toBe(true);
+  // Prashna düzeni (ileri modüller spec'i §11): D1, olgular, anın panchangı ve notlar.
+  expect(kinds(BUILTIN_LAYOUT_IDS.prashna)).toEqual(["chart:D1", "prashna_facts:", "panchang:", "notes:"]);
 
   for (const id of BUILTINS) {
     const panels = builtinLayout(id)!.panels;
@@ -230,7 +236,7 @@ test("layout and panel counts are capped like the Rust store", () => {
   const users = all.filter((l) => !l.builtin);
   // Hazır düzenler sınırın içindedir: toplam `MAX_LAYOUTS`.
   expect(all).toHaveLength(MAX_LAYOUTS);
-  expect(users).toHaveLength(MAX_LAYOUTS - 3);
+  expect(users).toHaveLength(MAX_LAYOUTS - BUILTINS.length);
   expect(users.every((l) => l.panels.length === MAX_PANELS)).toBe(true);
-  expect(users[MAX_LAYOUTS - 4].id).toBe(`user:${MAX_LAYOUTS - 4}`);
+  expect(users[MAX_LAYOUTS - BUILTINS.length - 1].id).toBe(`user:${MAX_LAYOUTS - BUILTINS.length - 1}`);
 });
diff --git a/app/ui/layout/reducer.test.ts b/app/ui/layout/reducer.test.ts
index a90903d..e117600 100644
--- a/app/ui/layout/reducer.test.ts
+++ b/app/ui/layout/reducer.test.ts
@@ -2,7 +2,7 @@ import { hasOverlap } from "./grid";
 import { BUILTIN_LAYOUT_IDS, builtinLayout, defaultLayoutState, MAX_LAYOUTS, MAX_PANELS, type LayoutState } from "./model";
 import { activeLayout, layoutReducer, newId, NOTES_PANEL, normalizeState } from "./reducer";
 
-const { interview, research, transit } = BUILTIN_LAYOUT_IDS;
+const { interview, research, transit, prashna } = BUILTIN_LAYOUT_IDS;
 
 function withUser(): LayoutState {
   return layoutReducer(defaultLayoutState(), { type: "saveAs", id: "user:1", name: "Benim" });
@@ -17,7 +17,8 @@ test("select switches the active layout and ignores unknown ids", () => {
 test("selectIndex follows layout order and ignores out-of-range", () => {
   const s = withUser();
   expect(layoutReducer(s, { type: "selectIndex", index: 2 }).active_layout_id).toBe(transit);
-  expect(layoutReducer(s, { type: "selectIndex", index: 3 }).active_layout_id).toBe("user:1");
+  expect(layoutReducer(s, { type: "selectIndex", index: 3 }).active_layout_id).toBe(prashna);
+  expect(layoutReducer(s, { type: "selectIndex", index: 4 }).active_layout_id).toBe("user:1");
   expect(layoutReducer(s, { type: "selectIndex", index: 8 })).toBe(s);
 });
 
@@ -56,7 +57,7 @@ test("saveAs and duplicate stop at MAX_LAYOUTS so nothing is lost on the next re
 
 test("duplicate inserts a copy right after its source without changing the active layout", () => {
   const s = layoutReducer(defaultLayoutState(), { type: "duplicate", sourceId: research, id: "user:2", name: "Araştırma kopyası" });
-  expect(s.layouts.map((l) => l.id)).toEqual([interview, research, "user:2", transit]);
+  expect(s.layouts.map((l) => l.id)).toEqual([interview, research, "user:2", transit, prashna]);
   expect(s.layouts[2].builtin).toBe(false);
   expect(s.layouts[2].panels).toEqual(builtinLayout(research)!.panels);
   expect(s.active_layout_id).toBe(interview);
@@ -66,7 +67,7 @@ test("builtins cannot be deleted; deleting the active user layout falls back to
   const s = withUser();
   expect(layoutReducer(s, { type: "delete", id: interview })).toBe(s);
   const d = layoutReducer(s, { type: "delete", id: "user:1" });
-  expect(d.layouts.map((l) => l.id)).toEqual([interview, research, transit]);
+  expect(d.layouts.map((l) => l.id)).toEqual([interview, research, transit, prashna]);
   expect(d.active_layout_id).toBe(interview);
 });
 
diff --git a/app/ui/palette/items.test.ts b/app/ui/palette/items.test.ts
index 67ade41..921ece6 100644
--- a/app/ui/palette/items.test.ts
+++ b/app/ui/palette/items.test.ts
@@ -22,7 +22,7 @@ test("local items keep command, chart, layout, varga order and are filtered by t
     commands: [{ command: "newChart", label: "Yeni harita" }],
     panels: [{ kind: "drishti", label: "Bakışlar panelini aç/kapat" }],
   };
-  expect(localItems(src, "", "tr").map((i) => i.kind)).toEqual(["command", "panel", "chart", "layout", "layout", "layout", "varga"]);
+  expect(localItems(src, "", "tr").map((i) => i.kind)).toEqual(["command", "panel", "chart", "layout", "layout", "layout", "layout", "varga"]);
   expect(localItems(src, "bakış", "tr")).toEqual([{ kind: "panel", key: "panel:drishti", label: "Bakışlar panelini aç/kapat", panelKind: "drishti" }]);
   expect(localItems(src, "harita", "tr").map((i) => i.key)).toEqual(["command:newChart", `chart:${chart.id}`]);
   expect(localItems(src, "navamsa", "tr")).toEqual([{ kind: "varga", key: "varga:D9", label: "D9 Navamsa", varga: "D9" }]);
diff --git a/app/ui/session/store.test.tsx b/app/ui/session/store.test.tsx
index 993fe84..78530fb 100644
--- a/app/ui/session/store.test.tsx
+++ b/app/ui/session/store.test.tsx
@@ -104,3 +104,33 @@ test("setTags updates the open chart's tags and bumps the revision", () => {
   expect(out("tags")).toHaveTextContent("araştırma");
   expect(out("revision")).toHaveTextContent("1");
 });
+
+test("setPrashna rewrites the side record of the open prashna only", () => {
+  function Side() {
+    const session = useSession();
+    const meta = session.opened?.prashna;
+    return (
+      <div>
+        <output aria-label="prashna">{meta ? `${meta.focus_house ?? "-"}|${meta.number ?? "-"}|${meta.question}` : "none"}</output>
+        <output aria-label="revision">{session.revision}</output>
+        <button onClick={() => meta && session.setPrashna(session.opened!.chart_record.id, { ...meta, question: "Yeni", focus_house: null })}>mine</button>
+        <button onClick={() => session.setPrashna(9999, { question: "Başka", number: 3, focus_house: 2, created_at: "t" })}>other</button>
+        <button onClick={() => session.setPrashna(session.opened!.chart_record.id, { question: "Başka", number: 3, focus_house: 2, created_at: "t" })}>force</button>
+      </div>
+    );
+  }
+  const first = renderWithStores(<Side />, { opened: fixture().prashna.opened_number });
+  expect(out("prashna")).toHaveTextContent("7|54|Deneme sorusu: yolculuk bu ay olur mu?");
+  // Başka haritanın geç gelen kaydı açık haritaya yazılmaz.
+  fireEvent.click(screen.getByText("other"));
+  expect(out("prashna")).toHaveTextContent("7|54|Deneme sorusu");
+  fireEvent.click(screen.getByText("mine"));
+  expect(out("prashna")).toHaveTextContent("-|54|Yeni");
+  // Listeleri etkilemez: revizyon artmaz.
+  expect(out("revision")).toHaveTextContent("0");
+  first.unmount();
+  // Prashna olmayan harita bir kayıtla prashna olmaz.
+  renderWithStores(<Side />);
+  fireEvent.click(screen.getByText("force"));
+  expect(out("prashna")).toHaveTextContent("none");
+});
diff --git a/app/ui/shell/ChartTitle.test.tsx b/app/ui/shell/ChartTitle.test.tsx
index 72e0aa6..fca8ee6 100644
--- a/app/ui/shell/ChartTitle.test.tsx
+++ b/app/ui/shell/ChartTitle.test.tsx
@@ -27,3 +27,19 @@ test("a differing advanced setting is named in the badge", () => {
   renderWithStores(<ChartTitle onSettings={() => {}} />, { opened });
   expect(screen.getByRole("button", { name: /Ayar farkı/ })).toHaveAttribute("title", "Bu haritanın ayarı varsayılandan farklı: İleri modül ayarları");
 });
+
+test("a number prashna carries its number beside the chart label; a time prashna and other charts do not", () => {
+  const { prashna } = fixture();
+  const first = renderWithStores(<ChartTitle onSettings={() => {}} />, { opened: prashna.opened_number });
+  const badge = screen.getByText("sayı: 54");
+  expect(badge).toHaveAttribute("title", "Sayı prashnası: lagna sayıdan gelir (navamsa ortası); grahalar anın grahalarıdır.");
+  // Açıklama ekran okuyucuya da okunur.
+  expect(badge).toHaveTextContent("sayı: 54: Sayı prashnası: lagna sayıdan gelir (navamsa ortası); grahalar anın grahalarıdır.");
+  expect(screen.getByText("Prashna 2026-11-25 15:30")).toBeInTheDocument();
+  first.unmount();
+  const second = renderWithStores(<ChartTitle onSettings={() => {}} />, { opened: prashna.opened });
+  expect(screen.queryByText(/^sayı:/)).toBeNull();
+  second.unmount();
+  renderWithStores(<ChartTitle onSettings={() => {}} />);
+  expect(screen.queryByText(/^sayı:/)).toBeNull();
+});
diff --git a/app/ui/shell/LayoutPicker.test.tsx b/app/ui/shell/LayoutPicker.test.tsx
index 22675c2..ad4d1fc 100644
--- a/app/ui/shell/LayoutPicker.test.tsx
+++ b/app/ui/shell/LayoutPicker.test.tsx
@@ -50,7 +50,7 @@ const saveAsNamed = (value: string) => {
 
 test("lists builtin layouts by translated name and selects on change", () => {
   setup();
-  expect([...picker().querySelectorAll("option")].map((o) => o.textContent)).toEqual(["Görüşme", "Araştırma", "Transit"]);
+  expect([...picker().querySelectorAll("option")].map((o) => o.textContent)).toEqual(["Görüşme", "Araştırma", "Transit", "Prashna"]);
   fireEvent.change(picker(), { target: { value: BUILTIN_LAYOUT_IDS.transit } });
   expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
 });
@@ -106,7 +106,7 @@ test("user layouts can be renamed and deleted after a named confirmation", () =>
   fireEvent.click(item("Sil…"));
   fireEvent.click(screen.getByRole("button", { name: "“Akşam” düzenini sil" }));
   expect(picker()).toHaveDisplayValue("Görüşme");
-  expect([...picker().querySelectorAll("option")]).toHaveLength(3);
+  expect([...picker().querySelectorAll("option")]).toHaveLength(4);
 });
 
 test("duplicate adds a copy after the source without switching", () => {
@@ -118,13 +118,14 @@ test("duplicate adds a copy after the source without switching", () => {
     "Görüşme kopyası",
     "Araştırma",
     "Transit",
+    "Prashna",
   ]);
   expect(picker()).toHaveDisplayValue("Görüşme");
 });
 
 test("save as and duplicate are disabled at the layout limit", () => {
   setup();
-  for (let i = 3; i < MAX_LAYOUTS; i++) {
+  for (let i = 4; i < MAX_LAYOUTS; i++) {
     menu();
     fireEvent.click(item("Kopyala"));
   }
@@ -236,7 +237,7 @@ describe("focus return to the trigger", () => {
     expect(screen.queryByRole("textbox", { name: "Düzen adı" })).toBeNull();
     expect(trigger()).toHaveFocus();
     expect(screen.getByTestId("pinned")).toHaveTextContent("pinned");
-    expect(options()).toHaveLength(3);
+    expect(options()).toHaveLength(4);
   });
 
   test("after confirming or cancelling delete, and Esc cancels the confirmation", () => {
@@ -284,7 +285,7 @@ describe("inline name validation", () => {
     expect(input).toHaveAccessibleDescription("Bir ad yazın.");
     fireEvent.submit(input.closest("form")!);
     expect(screen.getByText("Bir ad yazın.").closest("[aria-live]")).toBeInTheDocument();
-    expect(options()).toHaveLength(3);
+    expect(options()).toHaveLength(4);
   });
 
   test("names used by another layout are rejected case-insensitively", () => {
@@ -296,7 +297,7 @@ describe("inline name validation", () => {
     expect(screen.getByRole("button", { name: "Kaydet" })).toBeDisabled();
     expect(input).toHaveAccessibleDescription("Bu ad başka bir düzende kullanılıyor.");
     fireEvent.submit(input.closest("form")!);
-    expect(options()).toHaveLength(3);
+    expect(options()).toHaveLength(4);
   });
 
   test("renaming may keep the layout's own name", () => {
@@ -318,7 +319,7 @@ describe("inline name validation", () => {
     fireEvent.click(item("Kopyala"));
     menu();
     fireEvent.click(item("Kopyala"));
-    expect(options()).toEqual(["Görüşme", "Görüşme kopyası 2", "Görüşme kopyası", "Araştırma", "Transit"]);
+    expect(options()).toEqual(["Görüşme", "Görüşme kopyası 2", "Görüşme kopyası", "Araştırma", "Transit", "Prashna"]);
     menu();
     fireEvent.click(item("Farklı kaydet…"));
     expect(screen.getByRole("textbox", { name: "Düzen adı" })).toHaveValue("Görüşme kopyası 3");
@@ -330,9 +331,9 @@ describe("form target is pinned to the layout it was opened for", () => {
     setup();
     saveAsNamed("A");
     saveAsNamed("B");
-    expect(options()).toEqual(["Görüşme", "Araştırma", "Transit", "A", "B"]);
+    expect(options()).toEqual(["Görüşme", "Araştırma", "Transit", "Prashna", "A", "B"]);
   };
-  const switchAway = () => fireEvent.keyDown(window, { key: "4", code: "Digit4", metaKey: true });
+  const switchAway = () => fireEvent.keyDown(window, { key: "5", code: "Digit5", metaKey: true });
 
   test("rename: switching layouts while the form is open never renames another layout", () => {
     twoUserLayouts();
@@ -345,7 +346,7 @@ describe("form target is pinned to the layout it was opened for", () => {
       fireEvent.change(input, { target: { value: "Yeni" } });
       fireEvent.click(screen.getByRole("button", { name: "Kaydet" }));
     }
-    expect(options()).toEqual(["Görüşme", "Araştırma", "Transit", "A", "B"]);
+    expect(options()).toEqual(["Görüşme", "Araştırma", "Transit", "Prashna", "A", "B"]);
   });
 
   test("delete: switching layouts while the confirmation is open never deletes another layout", () => {
@@ -354,7 +355,7 @@ describe("form target is pinned to the layout it was opened for", () => {
     fireEvent.click(item("Sil…"));
     switchAway();
     for (const b of screen.queryAllByRole("button", { name: /düzenini sil/ })) fireEvent.click(b);
-    expect(options()).toEqual(["Görüşme", "Araştırma", "Transit", "A", "B"]);
+    expect(options()).toEqual(["Görüşme", "Araştırma", "Transit", "Prashna", "A", "B"]);
   });
 
   test("the open form closes when the layout select changes", () => {
diff --git a/app/ui/view/store.test.tsx b/app/ui/view/store.test.tsx
index 839ca86..7570d40 100644
--- a/app/ui/view/store.test.tsx
+++ b/app/ui/view/store.test.tsx
@@ -57,7 +57,7 @@ afterEach(() => vi.useRealTimers());
 test("loads the stored layout through the tolerant parser", () => {
   setup(memoryPersistence(), { active_layout_id: BUILTIN_LAYOUT_IDS.transit, layouts: [] });
   expect(screen.getByTestId("active")).toHaveTextContent(BUILTIN_LAYOUT_IDS.transit);
-  expect(screen.getByTestId("count")).toHaveTextContent("3");
+  expect(screen.getByTestId("count")).toHaveTextContent("4");
   expect(screen.getByTestId("chart")).toHaveTextContent("null");
 });
 
@@ -76,7 +76,7 @@ test("does not save on mount; saves the whole ViewState once 500 ms after the la
   const [s] = p.saved;
   expect(s.chartId).toBeNull();
   expect(s.viewState.schema_version).toBe(1);
-  expect(s.viewState.panel_layout!.layouts).toHaveLength(4);
+  expect(s.viewState.panel_layout!.layouts).toHaveLength(5);
   expect(s.viewState.panel_layout!.active_layout_id).toMatch(/^user:/);
   expect(s.viewState.dasha_expanded_path).toEqual([]);
 });
````

**Yeni dosya:** `app/ui/panels/PrashnaFactsPanel.test.tsx`

````tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computePrashnaFacts: vi.fn(), updatePrashna: vi.fn() } }));

import { api, type Result } from "../api";
import type { CommandError, OpenedClient, PrashnaFacts, PrashnaMeta } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import type { Panel } from "../layout/model";
import { MAX_QUESTION_CHARS } from "../prashna/model";
import { useSession } from "../session/store";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { RISING_NAMES } from "./labels";
import { PrashnaFactsPanel, QUESTION_SAVE_MS } from "./PrashnaFactsPanel";

const PANEL: Panel = { id: "pf", kind: "prashna_facts", x: 0, y: 0, w: 7, h: 20, settings: {} };
const ok = <T,>(data: T): Result<T> => ({ status: "ok", data });
const failure: Result<never> = { status: "error", error: { code: "calc", params: { detail: "x" } } };
const QUESTION = "Deneme sorusu: yolculuk bu ay olur mu?";

function Meta() {
  const { opened } = useSession();
  return <output aria-label="kayıt">{opened?.prashna ? `${opened.prashna.focus_house ?? "-"}|${opened.prashna.question}` : "-"}</output>;
}

function renderPanel(opened: OpenedClient | null = fixture().prashna.opened) {
  return renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Prashna olguları">
      <Meta />
      <PrashnaFactsPanel panel={PANEL} />
    </section>,
    { opened },
  );
}

const section = (name: string) => screen.getByRole("region", { name });
const terms = (root: HTMLElement) => [...root.querySelectorAll("dt")].map((dt) => `${dt.textContent}: ${dt.nextElementSibling?.textContent}`);
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const bodyRows = (table: HTMLElement) => within(table).getAllByRole("row").slice(1).map(cells);
const focusSelect = () => screen.getByRole("combobox", { name: "Odak evi" });
const question = () => screen.getByRole("textbox", { name: "Soru" });

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.computePrashnaFacts).mockImplementation(async (input, focus) => {
    const { prashna } = fixture();
    if (focus === null) return ok(prashna.facts_no_focus);
    return ok(input.lagna_number === null ? prashna.facts : prashna.facts_number);
  });
  vi.mocked(api.updatePrashna).mockImplementation(async (_chartId, edit) =>
    ok<PrashnaMeta>({ ...fixture().prashna.opened.prashna!, question: edit.question.trim(), focus_house: edit.focus_house }),
  );
});

test("a chart that is not a prashna gets a note and no command", () => {
  renderPanel(fixture().opened);
  expect(screen.getByRole("note")).toHaveTextContent("Bu harita bir prashna değil. Yeni bir prashna kurmak için ⌘⇧P.");
  expect(api.computePrashnaFacts).not.toHaveBeenCalled();
  expect(screen.queryByRole("textbox")).toBeNull();
});

test("without an open chart the panel says so", () => {
  renderPanel(null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computePrashnaFacts).not.toHaveBeenCalled();
});

test("draws the facts of the question chart: lagna, lords and their relation, houses, the moon, the moment and the yogas", async () => {
  renderPanel();
  expect(question()).toHaveValue(QUESTION);
  expect(focusSelect()).toHaveDisplayValue("7. ev");
  await screen.findByRole("region", { name: "Lagna" });
  const { opened } = fixture().prashna;
  expect(api.computePrashnaFacts).toHaveBeenCalledTimes(1);
  expect(api.computePrashnaFacts).toHaveBeenCalledWith(
    { jd_ut: opened.chart.jd_ut, lat: opened.chart.geo_lat, lon: opened.chart.geo_lon, settings: opened.chart.meta.settings, time_known: true, lagna_number: null },
    7,
  );
  // Lagna: burç ve derece, tür, doğuş biçimi (klasik ad ve çevrilen ek).
  const lagna = terms(section("Lagna"));
  expect(lagna[0]).toMatch(/^Burç: Koç \d+°\d{2}′$/);
  expect(lagna.slice(1)).toEqual(["Burç türü: yürük", "Doğuş biçimi: Prishthodaya · sırtla doğan"]);
  // Lordlar: lagna lordu Mars Aslan'da 5. evde; odak lordu Venüs Terazi'de 7. evde (burçlar
  // öteki tablolardaki gibi kısaltmayla yazılır).
  const lords = within(section("Lordlar")).getByRole("table");
  expect(lords.querySelector("caption")).toHaveTextContent("Lagna lordu ve odak evinin lordu");
  expect(bodyRows(lords)).toEqual([
    ["Lagna lordu", "Mars", "Asl", "5", "Çok dost", "—"],
    ["Odak lordu", "Venüs", "Ter", "7", "Mulatrikona", "—"],
  ]);
  expect(within(section("Lordlar")).getByRole("list", { name: "İki lordun bağı" })).toHaveTextContent("hiçbiri");
  expect(section("Lordlar")).toHaveTextContent("Odak lordu lagna lordundan 3. burçta; lagna lordu odak lordundan 11. burçta.");
  // Kendralar ve odak evi: sınıf sözle yazılır (renk anlam taşımaz).
  const houses = within(section("Kendralar ve odak evi")).getByRole("table");
  expect(bodyRows(houses)).toEqual([
    ["1. ev", "Koç", "boş"],
    ["4. ev", "Yen", "boş"],
    ["7. ev", "Ter", "Merkür (iyicil), Venüs (iyicil)"],
    ["10. ev", "Oğl", "boş"],
    ["7. ev · odak", "Ter", "Merkür (iyicil), Venüs (iyicil)"],
  ]);
  // Ay ve anın panchangı (v1 haritasından).
  expect(terms(section("Ay"))).toEqual(["Burç: Boğ · 2. ev", "Nakshatra: Rohini · pada 4", "Tithi: Krishna 1 · Pratipada", "Burcuna bakanlar: Güneş, Satürn"]);
  expect(terms(section("Anın panchangı")).map((l) => l.split(":")[0])).toEqual(["Tithi", "Vara", "Nakshatra", "Nitya yoga", "Karana"]);
  expect(terms(section("Anın panchangı"))[0]).toBe("Tithi: Krishna 1 · Pratipada");
  // Yoga listesi (Alt proje 3.5'in kataloğu) ve kapanış satırları; hüküm sözü yoktur.
  const yogas = section("Yogalar");
  expect(yogas.querySelectorAll("tr[data-yoga]")).toHaveLength(fixture().prashna.facts.yogas.hits.length);
  expect(screen.getByText("Bu panel olguları sayar; sorunun yanıtını ya da bir hükmü söylemez.")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: prashna.parashari_v1")).toBeInTheDocument();
  const text = document.body.textContent ?? "";
  for (const verdict of ["uğurlu", "uğursuz", "olumlu", "olumsuz", "evet", "hayır"]) expect(text.toLocaleLowerCase("tr")).not.toContain(verdict);
  // Geniş tablolar klavyeyle kaydırılır: adlı ve odaklanır kaplar.
  for (const name of ["Lordlar", "Kendralar ve odak evi"]) {
    const scroll = screen.getByRole("group", { name });
    expect(scroll).toHaveAttribute("tabindex", "0");
  }
});

test("a number prashna says so: the badge, the navamsa middle and the number in the module input", async () => {
  renderPanel(fixture().prashna.opened_number);
  await screen.findByRole("region", { name: "Lagna" });
  expect(vi.mocked(api.computePrashnaFacts).mock.calls[0][0].lagna_number).toBe(54);
  expect(screen.getByText("sayı: 54")).toBeInTheDocument();
  expect(terms(section("Lagna"))[0]).toBe("Burç: Baş 28°20′ · 54. navamsanın ortası");
  // Lagna sayıdan gelir: lordlar ve evler Başak'tan sayılır.
  const lords = bodyRows(within(section("Lordlar")).getByRole("table"));
  expect(lords.map((r) => r[0])).toEqual(["Lagna lordu", "Odak lordu"]);
  expect(lords[0][1]).toBe("Merkür");
  expect(lords[1][1]).toBe("Jüpiter");
});

test("the focus house is changed from the panel: saved, written to the session and the facts are read again in place", async () => {
  renderPanel();
  await screen.findByRole("region", { name: "Lordlar" });
  let release: (r: Result<PrashnaFacts>) => void = () => {};
  vi.mocked(api.computePrashnaFacts).mockImplementationOnce(() => new Promise((resolve) => (release = resolve)));
  focusSelect().focus();
  fireEvent.change(focusSelect(), { target: { value: "10" } });
  await waitFor(() => expect(screen.getByRole("status", { name: "kayıt" })).toHaveTextContent(`10|${QUESTION}`));
  expect(api.updatePrashna).toHaveBeenCalledWith(fixture().prashna.opened.chart_record.id, { question: QUESTION, focus_house: 10 });
  await waitFor(() => expect(api.computePrashnaFacts).toHaveBeenCalledTimes(2));
  expect(vi.mocked(api.computePrashnaFacts).mock.calls[1][1]).toBe(10);
  // Yeni olgular okunurken eskiler ve odak yerinde durur; panel meşgul olduğunu söyler.
  expect(screen.getByRole("region", { name: "Lordlar" })).toBeInTheDocument();
  expect(document.querySelector(".prashna-facts")).toHaveAttribute("aria-busy", "true");
  expect(focusSelect()).toHaveFocus();
  expect(focusSelect()).toHaveDisplayValue("10. ev");
  const next = fixture().prashna.facts;
  next.focus!.house = 10;
  await act(async () => release(ok(next)));
  expect(document.querySelector(".prashna-facts")).not.toHaveAttribute("aria-busy");
  expect(bodyRows(within(section("Kendralar ve odak evi")).getByRole("table")).at(-1)![0]).toBe("10. ev · odak");
  expect(focusSelect()).toHaveFocus();
});

test("with no focus house only the lagna lord is listed and the panel says why", async () => {
  const opened = fixture().prashna.opened;
  opened.prashna = { ...opened.prashna!, focus_house: null };
  renderPanel(opened);
  await screen.findByRole("region", { name: "Lordlar" });
  expect(vi.mocked(api.computePrashnaFacts).mock.calls[0][1]).toBeNull();
  expect(focusSelect()).toHaveDisplayValue("Seçilmedi");
  expect(bodyRows(within(section("Lordlar")).getByRole("table")).map((r) => r[0])).toEqual(["Lagna lordu"]);
  expect(section("Lordlar")).toHaveTextContent("Odak evi seçilmedi: odak evinin lordu ve iki lordun bağı gösterilmez.");
  expect(within(section("Lordlar")).queryByRole("list")).toBeNull();
  expect(bodyRows(within(section("Kendralar ve odak evi")).getByRole("table"))).toHaveLength(4);
});

test("the relation lists every link that holds, and says when both houses have the same lord", async () => {
  const both = fixture().prashna.facts;
  both.focus!.relation = { same_graha: false, links: ["Exchange", "MutualAspect"], focus_lord_from_lagna_lord: 7, lagna_lord_from_focus_lord: 7 };
  vi.mocked(api.computePrashnaFacts).mockResolvedValueOnce(ok(both));
  const first = renderPanel();
  const list = await screen.findByRole("list", { name: "İki lordun bağı" });
  expect(within(list).getAllByRole("listitem").map((li) => li.textContent)).toEqual(["değişim", "karşılıklı bakış"]);
  first.unmount();
  const same = fixture().prashna.facts;
  same.focus!.lord = { ...same.lagna!.lord };
  same.focus!.relation = { same_graha: true, links: [], focus_lord_from_lagna_lord: 1, lagna_lord_from_focus_lord: 1 };
  vi.mocked(api.computePrashnaFacts).mockResolvedValueOnce(ok(same));
  renderPanel();
  await screen.findByRole("region", { name: "Lordlar" });
  expect(section("Lordlar")).toHaveTextContent("7. evin lordu da Mars: iki evin lordu aynı graha.");
  expect(within(section("Lordlar")).queryByRole("list")).toBeNull();
});

test("a retrograde, combust lord is marked in words", async () => {
  const facts = fixture().prashna.facts;
  facts.lagna!.lord = { ...facts.lagna!.lord, retrograde: true, combust: true, dignity: "Debilitated" };
  vi.mocked(api.computePrashnaFacts).mockResolvedValueOnce(ok(facts));
  renderPanel();
  await screen.findByRole("region", { name: "Lordlar" });
  expect(bodyRows(within(section("Lordlar")).getByRole("table"))[0].slice(4)).toEqual(["Düşük", "retro, yanık"]);
});

test("the question is saved a moment after typing stops; a question that is too long is not sent", async () => {
  vi.useFakeTimers();
  try {
    renderPanel();
    await act(async () => vi.advanceTimersByTime(0));
    fireEvent.change(question(), { target: { value: "Yeni soru" } });
    expect(question()).toHaveAccessibleDescription("9 / 2000 karakter · yazdıkça kaydedilir");
    await act(async () => vi.advanceTimersByTime(QUESTION_SAVE_MS - 1));
    expect(api.updatePrashna).not.toHaveBeenCalled();
    fireEvent.change(question(), { target: { value: "Yeni soru mu" } });
    await act(async () => vi.advanceTimersByTime(QUESTION_SAVE_MS));
    expect(api.updatePrashna).toHaveBeenCalledTimes(1);
    expect(api.updatePrashna).toHaveBeenCalledWith(fixture().prashna.opened.chart_record.id, { question: "Yeni soru mu", focus_house: 7 });
    expect(screen.getByRole("status", { name: "kayıt" })).toHaveTextContent("7|Yeni soru mu");
    expect(question()).toHaveAccessibleDescription("Kaydedildi");
    // Soru olguları etkilemez: yeniden hesap yok, metin ve odak yerinde.
    expect(api.computePrashnaFacts).toHaveBeenCalledTimes(1);
    expect(question()).toHaveValue("Yeni soru mu");
    // Çok uzun soru gönderilmez ve bunu satır söyler.
    fireEvent.change(question(), { target: { value: "ş".repeat(MAX_QUESTION_CHARS + 1) } });
    await act(async () => vi.advanceTimersByTime(QUESTION_SAVE_MS * 2));
    expect(api.updatePrashna).toHaveBeenCalledTimes(1);
    expect(question()).toHaveAttribute("aria-invalid", "true");
    expect(question()).toHaveAccessibleDescription("2001 / 2000 karakter: soru çok uzun, kaydedilmedi.");
  } finally {
    vi.useRealTimers();
  }
});

test("a pending question is sent when the panel goes away; a failed save is shown and can be retried", async () => {
  vi.useFakeTimers();
  try {
    const first = renderPanel();
    await act(async () => vi.advanceTimersByTime(0));
    fireEvent.change(question(), { target: { value: "Yarım" } });
    first.unmount();
    await act(async () => vi.advanceTimersByTime(0));
    expect(api.updatePrashna).toHaveBeenCalledWith(fixture().prashna.opened.chart_record.id, { question: "Yarım", focus_house: 7 });
    vi.mocked(api.updatePrashna).mockClear();

    const error: CommandError = { code: "store_unavailable", params: {} };
    vi.mocked(api.updatePrashna).mockResolvedValueOnce({ status: "error", error });
    renderPanel();
    await act(async () => vi.advanceTimersByTime(0));
    fireEvent.change(question(), { target: { value: "Başka" } });
    await act(async () => vi.advanceTimersByTime(QUESTION_SAVE_MS));
    const alert = screen.getByRole("alert");
    expect(alert).toHaveTextContent("Kaydedilemedi: Danışan veritabanı kullanılamıyor.");
    // Yazılan metin durur; kayıt değişmedi.
    expect(question()).toHaveValue("Başka");
    expect(screen.getByRole("status", { name: "kayıt" })).toHaveTextContent(`7|${QUESTION}`);
    fireEvent.click(within(alert).getByRole("button", { name: "Yeniden dene" }));
    await act(async () => vi.advanceTimersByTime(0));
    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.getByRole("status", { name: "kayıt" })).toHaveTextContent("7|Başka");
  } finally {
    vi.useRealTimers();
  }
});

test("an error keeps the question and the focus house; retry reads again and shows nothing from before the error", async () => {
  renderPanel();
  await screen.findByRole("region", { name: "Lordlar" });
  // Odak evi değişir, okuma düşer: hata panelin içindedir; soru ve seçim yerinde durur.
  vi.mocked(api.computePrashnaFacts).mockResolvedValueOnce(failure);
  fireEvent.change(focusSelect(), { target: { value: "4" } });
  const retry = await screen.findByRole("button", { name: "Yeniden dene" });
  expect(screen.getByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(question()).toHaveValue(QUESTION);
  expect(focusSelect()).toHaveDisplayValue("4. ev");
  expect(screen.queryByRole("region", { name: "Lordlar" })).toBeNull();
  // Yeniden deneme boş başlar: hatadan önceki olgular geri gelmez.
  let release: (r: Result<PrashnaFacts>) => void = () => {};
  vi.mocked(api.computePrashnaFacts).mockImplementationOnce(() => new Promise((resolve) => (release = resolve)));
  retry.focus();
  fireEvent.click(retry);
  await waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
  expect(screen.queryByRole("region", { name: "Lordlar" })).toBeNull();
  const next = fixture().prashna.facts;
  next.focus!.house = 4;
  await act(async () => release(ok(next)));
  expect(screen.getByRole("region", { name: "Lordlar" })).toBeInTheDocument();
  // Odaktaki düğme kalkınca odak gövdeye düşmez: panelin içinde durur.
  expect(document.activeElement).not.toBe(document.body);
});

test("a chart whose time is unknown shows no lagna facts and says why; the moon and the yogas remain", async () => {
  const facts = fixture().prashna.facts;
  facts.lagna = null;
  facts.focus = null;
  facts.moon.house = null;
  vi.mocked(api.computePrashnaFacts).mockResolvedValueOnce(ok(facts));
  const opened = fixture().prashna.opened;
  opened.chart_record.birth.time = null;
  opened.chart_record.birth.time_certainty = "Unknown";
  renderPanel(opened);
  await screen.findByRole("region", { name: "Ay" });
  expect(vi.mocked(api.computePrashnaFacts).mock.calls[0][0].time_known).toBe(false);
  expect(screen.getByRole("note")).toHaveTextContent("Bu haritanın saati bilinmiyor: lagnaya bağlı olgular (lagna, lordlar, evler) gösterilmez.");
  for (const name of ["Lagna", "Lordlar", "Kendralar ve odak evi"]) expect(screen.queryByRole("region", { name })).toBeNull();
  expect(terms(section("Ay"))[0]).toBe("Burç: Boğ");
  expect(section("Yogalar")).toBeInTheDocument();
});

test("the lagna lord is a shared highlight: pinning it marks its row", async () => {
  renderPanel();
  await screen.findByRole("region", { name: "Lordlar" });
  const table = within(section("Lordlar")).getByRole("table");
  const mars = within(table).getByRole("button", { name: "Mars" });
  expect(mars).toHaveAttribute("aria-pressed", "false");
  fireEvent.click(mars);
  expect(mars).toHaveAttribute("aria-pressed", "true");
  expect(table.querySelector('tr[data-lord="lagna"]')).toHaveClass("is-highlighted");
  expect(table.querySelector('tr[data-lord="focus"]')).not.toHaveClass("is-highlighted");
});

test("another prashna starts with its own question, not the draft of the previous one", async () => {
  function Switch() {
    const session = useSession();
    return (
      <button type="button" onClick={() => session.open({ ...fixture().prashna.opened_number, prashna: { ...fixture().prashna.opened_number.prashna!, question: "İkinci soru" } })}>
        öteki
      </button>
    );
  }
  renderWithStores(
    <>
      <Switch />
      <PrashnaFactsPanel panel={PANEL} />
    </>,
    { opened: fixture().prashna.opened },
  );
  await screen.findByRole("region", { name: "Lagna" });
  fireEvent.change(question(), { target: { value: "Taslak" } });
  fireEvent.click(screen.getByRole("button", { name: "öteki" }));
  await waitFor(() => expect(question()).toHaveValue("İkinci soru"));
  // Önceki haritanın taslağı kendi haritasına kaydedilir.
  expect(api.updatePrashna).toHaveBeenCalledWith(fixture().prashna.opened.chart_record.id, { question: "Taslak", focus_house: 7 });
});

test("every rising mode, link, kind and nature has a name in three languages; classical names are the same in each", () => {
  const { prashna_ids } = fixture();
  expect(Object.keys(RISING_NAMES).sort()).toEqual([...prashna_ids.rising].sort());
  for (const lang of [tr, en, fr]) {
    expect(Object.keys(lang.prashna.rising).sort()).toEqual([...prashna_ids.rising].sort());
    expect(Object.keys(lang.prashna.link).sort()).toEqual([...prashna_ids.links].sort());
    expect(Object.keys(lang.prashna.kind).sort()).toEqual(["Dual", "Fixed", "Movable"]);
    expect(Object.keys(lang.prashna.nature).sort()).toEqual(["Benefic", "Malefic"]);
    expect(lang.panel.prashna_facts).toContain(lang === fr ? "prashna" : lang === en ? "Prashna" : "Prashna");
    expect(lang.layout.prashna).toBe("Prashna");
  }
  // Fransızca metinler tipografik kesme işareti kullanır.
  expect(JSON.stringify(fr.prashna)).not.toContain("'");
});
````

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
```
Expected: `Test Files  9 failed | 74 passed (83)`, `Tests  26 failed | 799 passed (825)` (panel dosyası içe aktaramaz; öteki sekiz dosyada dördüncü hazır düzeni, `setPrashna`'yı, rozeti ve Prashna düzenine geçişi bekleyen testler düşer).

- [ ] **Step 3: Uygulama**

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index f23103d..cf0475c 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -75,6 +75,7 @@ export const en: Messages = {
     compat: "Compatibility",
     muhurta_day: "Day panchang",
     muhurta_search: "Muhurta search",
+    prashna_facts: "Prashna facts",
     unknown: "Unknown panel",
     loading: "Loading",
     unavailable: "This panel arrives in a later version.",
@@ -505,6 +506,46 @@ export const en: Messages = {
     note: "The limbs and the lagna are for the moment the window starts. Windows are facts, not a verdict on suitability.",
   },
   prashna: {
+    notPrashna: "This chart is not a prashna. Press ⌘⇧P to cast a new one.",
+    question: "Question",
+    questionHint: "{{count}} / {{max}} characters · saved as you type",
+    questionSaved: "Saved",
+    questionTooLong: "{{count}} / {{max}} characters: the question is too long and was not saved.",
+    saveFailed: "Could not be saved: {{message}}",
+    focus: "Focus house",
+    numberBadge: "number: {{number}}",
+    numberHint: "Number prashna: the lagna comes from the number (middle of the navamsa); the grahas are those of the moment.",
+    timeUnknown: "The time of this chart is unknown: facts that depend on the lagna (lagna, lords, houses) are not shown.",
+    noFocus: "No focus house is chosen: its lord and the relation of the two lords are not shown.",
+    none: "none",
+    empty: "empty",
+    retrograde: "retrograde",
+    combust: "combust",
+    yogas: "Yogas",
+    note: "This panel lists facts; it gives no answer to the question and no judgement.",
+    method: "Method: {{method}}",
+    nature: { Benefic: "benefic", Malefic: "malefic" },
+    kind: { Movable: "movable", Fixed: "fixed", Dual: "dual" },
+    rising: { HeadFirst: "rising head first", BackFirst: "rising back first", BothWays: "rising both ways" },
+    lagna: { title: "Lagna", kind: "Sign kind", rising: "Rising mode", navamsaMiddle: "middle of navamsa {{number}}" },
+    lords: { title: "Lords", caption: "The lagna lord and the lord of the focus house", role: "Lord", house: "House", dignity: "Dignity", state: "State" },
+    lord: { lagna: "Lagna lord", focus: "Focus lord" },
+    relation: {
+      title: "Relation of the two lords",
+      none: "none",
+      same: "House {{house}} is also ruled by {{graha}}: both houses have the same lord.",
+      counts: "The focus lord is in sign {{forward}} from the lagna lord; the lagna lord is in sign {{backward}} from the focus lord.",
+    },
+    link: {
+      Conjunct: "together",
+      Exchange: "exchange",
+      MutualAspect: "mutual aspect",
+      LagnaLordAspects: "one-way aspect: the lagna lord aspects",
+      FocusLordAspects: "one-way aspect: the focus lord aspects",
+    },
+    houses: { title: "Kendras and the focus house", caption: "Grahas in the kendras and in the focus house", house: "House", grahas: "Grahas", focus: "focus" },
+    moon: { title: "Moon", aspectedBy: "Aspecting its sign" },
+    moment: { title: "Panchang of the moment" },
     focusNone: "Not chosen",
     focusHouse: "House {{house}}",
     form: {
@@ -643,6 +684,7 @@ export const en: Messages = {
     interview: "Consultation",
     research: "Research",
     transit: "Transit",
+    prashna: "Prashna",
     actions: "Layout actions",
     saveAs: "Save as…",
     rename: "Rename…",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 3b8bae6..cca6fc5 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -75,6 +75,7 @@ export const fr: Messages = {
     compat: "Compatibilité",
     muhurta_day: "Panchang du jour",
     muhurta_search: "Recherche de muhurta",
+    prashna_facts: "Faits du prashna",
     unknown: "Panneau inconnu",
     loading: "Chargement",
     unavailable: "Ce panneau arrivera dans une version ultérieure.",
@@ -505,6 +506,46 @@ export const fr: Messages = {
     note: "Les membres et le lagna sont ceux de l'instant où la fenêtre commence. Les fenêtres sont des faits, non un jugement de convenance.",
   },
   prashna: {
+    notPrashna: "Ce thème n’est pas un prashna. Appuyez sur ⌘⇧P pour en établir un.",
+    question: "Question",
+    questionHint: "{{count}} / {{max}} caractères · enregistrée pendant la saisie",
+    questionSaved: "Enregistrée",
+    questionTooLong: "{{count}} / {{max}} caractères : la question est trop longue et n’a pas été enregistrée.",
+    saveFailed: "Enregistrement impossible : {{message}}",
+    focus: "Maison de la question",
+    numberBadge: "nombre : {{number}}",
+    numberHint: "Prashna par nombre : le lagna vient du nombre (milieu du navamsa) ; les grahas sont ceux du moment.",
+    timeUnknown: "L’heure de ce thème est inconnue : les faits qui dépendent du lagna (lagna, maîtres, maisons) ne sont pas affichés.",
+    noFocus: "Aucune maison de la question n’est choisie : son maître et le lien des deux maîtres ne sont pas affichés.",
+    none: "aucun",
+    empty: "vide",
+    retrograde: "rétrograde",
+    combust: "combuste",
+    yogas: "Yogas",
+    note: "Ce panneau énumère des faits ; il ne donne ni réponse à la question ni jugement.",
+    method: "Méthode : {{method}}",
+    nature: { Benefic: "bénéfique", Malefic: "maléfique" },
+    kind: { Movable: "mobile", Fixed: "fixe", Dual: "double" },
+    rising: { HeadFirst: "se lève par la tête", BackFirst: "se lève par le dos", BothWays: "se lève des deux façons" },
+    lagna: { title: "Lagna", kind: "Type de signe", rising: "Mode de lever", navamsaMiddle: "milieu du navamsa {{number}}" },
+    lords: { title: "Maîtres", caption: "Le maître du lagna et le maître de la maison de la question", role: "Maître", house: "Maison", dignity: "Dignité", state: "État" },
+    lord: { lagna: "Maître du lagna", focus: "Maître de la question" },
+    relation: {
+      title: "Lien des deux maîtres",
+      none: "aucun",
+      same: "La maison {{house}} a aussi {{graha}} pour maître : les deux maisons ont le même maître.",
+      counts: "Le maître de la question est au signe {{forward}} depuis le maître du lagna ; le maître du lagna est au signe {{backward}} depuis le maître de la question.",
+    },
+    link: {
+      Conjunct: "ensemble",
+      Exchange: "échange",
+      MutualAspect: "aspect mutuel",
+      LagnaLordAspects: "aspect à sens unique : le maître du lagna regarde",
+      FocusLordAspects: "aspect à sens unique : le maître de la question regarde",
+    },
+    houses: { title: "Kendras et maison de la question", caption: "Grahas dans les kendras et dans la maison de la question", house: "Maison", grahas: "Grahas", focus: "question" },
+    moon: { title: "Lune", aspectedBy: "Regardent son signe" },
+    moment: { title: "Panchang du moment" },
     focusNone: "Non choisie",
     focusHouse: "Maison {{house}}",
     form: {
@@ -643,6 +684,7 @@ export const fr: Messages = {
     interview: "Consultation",
     research: "Recherche",
     transit: "Transits",
+    prashna: "Prashna",
     actions: "Actions sur la disposition",
     saveAs: "Enregistrer sous…",
     rename: "Renommer…",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 73c1411..b24a804 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -74,6 +74,7 @@ export const tr = {
     compat: "Uyum",
     muhurta_day: "Gün panchangı",
     muhurta_search: "Muhurta araması",
+    prashna_facts: "Prashna olguları",
     unknown: "Bilinmeyen panel",
     loading: "Yükleniyor",
     unavailable: "Bu panel sonraki bir sürümde gelecek.",
@@ -504,6 +505,46 @@ export const tr = {
     note: "Uzuvlar ve lagna pencerenin başlangıç anı içindir. Pencereler olgudur; uygunluk hükmü değildir.",
   },
   prashna: {
+    notPrashna: "Bu harita bir prashna değil. Yeni bir prashna kurmak için ⌘⇧P.",
+    question: "Soru",
+    questionHint: "{{count}} / {{max}} karakter · yazdıkça kaydedilir",
+    questionSaved: "Kaydedildi",
+    questionTooLong: "{{count}} / {{max}} karakter: soru çok uzun, kaydedilmedi.",
+    saveFailed: "Kaydedilemedi: {{message}}",
+    focus: "Odak evi",
+    numberBadge: "sayı: {{number}}",
+    numberHint: "Sayı prashnası: lagna sayıdan gelir (navamsa ortası); grahalar anın grahalarıdır.",
+    timeUnknown: "Bu haritanın saati bilinmiyor: lagnaya bağlı olgular (lagna, lordlar, evler) gösterilmez.",
+    noFocus: "Odak evi seçilmedi: odak evinin lordu ve iki lordun bağı gösterilmez.",
+    none: "yok",
+    empty: "boş",
+    retrograde: "retro",
+    combust: "yanık",
+    yogas: "Yogalar",
+    note: "Bu panel olguları sayar; sorunun yanıtını ya da bir hükmü söylemez.",
+    method: "Yöntem: {{method}}",
+    nature: { Benefic: "iyicil", Malefic: "kötücül" },
+    kind: { Movable: "yürük", Fixed: "sabit", Dual: "çift doğalı" },
+    rising: { HeadFirst: "başla doğan", BackFirst: "sırtla doğan", BothWays: "iki yönlü" },
+    lagna: { title: "Lagna", kind: "Burç türü", rising: "Doğuş biçimi", navamsaMiddle: "{{number}}. navamsanın ortası" },
+    lords: { title: "Lordlar", caption: "Lagna lordu ve odak evinin lordu", role: "Lord", house: "Ev", dignity: "Dignity", state: "Durum" },
+    lord: { lagna: "Lagna lordu", focus: "Odak lordu" },
+    relation: {
+      title: "İki lordun bağı",
+      none: "hiçbiri",
+      same: "{{house}}. evin lordu da {{graha}}: iki evin lordu aynı graha.",
+      counts: "Odak lordu lagna lordundan {{forward}}. burçta; lagna lordu odak lordundan {{backward}}. burçta.",
+    },
+    link: {
+      Conjunct: "birlikte",
+      Exchange: "değişim",
+      MutualAspect: "karşılıklı bakış",
+      LagnaLordAspects: "tek yönlü bakış: lagna lordu bakıyor",
+      FocusLordAspects: "tek yönlü bakış: odak lordu bakıyor",
+    },
+    houses: { title: "Kendralar ve odak evi", caption: "Kendralardaki ve odak evindeki grahalar", house: "Ev", grahas: "Grahalar", focus: "odak" },
+    moon: { title: "Ay", aspectedBy: "Burcuna bakanlar" },
+    moment: { title: "Anın panchangı" },
     focusNone: "Seçilmedi",
     focusHouse: "{{house}}. ev",
     form: {
@@ -642,6 +683,7 @@ export const tr = {
     interview: "Görüşme",
     research: "Araştırma",
     transit: "Transit",
+    prashna: "Prashna",
     actions: "Düzen işlemleri",
     saveAs: "Farklı kaydet…",
     rename: "Yeniden adlandır…",
diff --git a/app/ui/layout/model.ts b/app/ui/layout/model.ts
index 518980d..0b41942 100644
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -1,7 +1,7 @@
 // Pano düzeni veri modeli (spec §3.2). Kalıcı biçim `ViewState.panel_layout` ile aynıdır;
 // M4'teki store komutu gelene kadar localStorage'da tutulur (bkz. persistence.ts).
 
-/** Panel kataloğu: v1 (UI spec §3.3) ve ileri modül panelleri (ileri modüller spec'i §4, §5, §7, §8, §9, §10). */
+/** Panel kataloğu: v1 (UI spec §3.3) ve ileri modül panelleri (ileri modüller spec'i §4, §5, §7, §8, §9, §10, §11). */
 export const PANEL_KINDS = [
   "chart",
   "graha_table",
@@ -22,6 +22,7 @@ export const PANEL_KINDS = [
   "compat",
   "muhurta_day",
   "muhurta_search",
+  "prashna_facts",
 ] as const;
 export type KnownPanelKind = (typeof PANEL_KINDS)[number];
 
@@ -47,6 +48,8 @@ export const MODULE_PANELS = [
   // Muhurta (Plan 20).
   { kind: "muhurta_day", w: 7, h: 18 },
   { kind: "muhurta_search", w: 8, h: 18 },
+  // Prashna (Plan 21).
+  { kind: "prashna_facts", w: 7, h: 20 },
 ] as const satisfies readonly { kind: KnownPanelKind; w: number; h: number }[];
 export type ModulePanelKind = (typeof MODULE_PANELS)[number]["kind"];
 
@@ -91,6 +94,7 @@ export const BUILTIN_LAYOUT_IDS = {
   interview: "builtin:interview",
   research: "builtin:research",
   transit: "builtin:transit",
+  prashna: "builtin:prashna",
 } as const;
 export type BuiltinLayoutId = (typeof BUILTIN_LAYOUT_IDS)[keyof typeof BUILTIN_LAYOUT_IDS];
 
@@ -98,6 +102,7 @@ const BUILTIN_ORDER: readonly BuiltinLayoutId[] = [
   BUILTIN_LAYOUT_IDS.interview,
   BUILTIN_LAYOUT_IDS.research,
   BUILTIN_LAYOUT_IDS.transit,
+  BUILTIN_LAYOUT_IDS.prashna,
 ];
 
 export function isKnownPanelKind(kind: string): kind is KnownPanelKind {
@@ -152,6 +157,19 @@ const BUILTIN_FACTORIES: Record<BuiltinLayoutId, () => Layout> = {
       panel("dasha", "dasha_strip", 0, 12, 12, 4, { level: "antar" }),
     ],
   }),
+  // Prashna (ileri modüller spec'i §11): D1, olgular, anın panchangı ve notlar. Modül paneli
+  // taşıyan tek hazır düzendir.
+  [BUILTIN_LAYOUT_IDS.prashna]: () => ({
+    id: BUILTIN_LAYOUT_IDS.prashna,
+    name: "Prashna",
+    builtin: true,
+    panels: [
+      chart("d1", "D1", 0, 0, 5, 12),
+      panel("facts", "prashna_facts", 5, 0, 7, 21),
+      panel("panchang", "panchang", 0, 12, 5, 9),
+      panel("notes", "notes", 0, 21, 12, 5),
+    ],
+  }),
 };
 
 /** Hazır düzenin taze kopyası; bilinmeyen kimlikte `undefined`. */
diff --git a/app/ui/panels/YogasPanel.tsx b/app/ui/panels/YogasPanel.tsx
index 43c86ce..2d95233 100644
--- a/app/ui/panels/YogasPanel.tsx
+++ b/app/ui/panels/YogasPanel.tsx
@@ -89,7 +89,8 @@ function Yogas({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
   );
 }
 
-function YogaList({ report, filter, opened }: { report: YogaReport; filter: YogaGroupFilter; opened: OpenedClient }) {
+/** Yoga raporunun gruplu listesi; prashna olguları paneli de kullanır. */
+export function YogaList({ report, filter, opened }: { report: YogaReport; filter: YogaGroupFilter; opened: OpenedClient }) {
   const { t } = useTranslation();
   const { active } = useHighlight();
   const listId = useId();
diff --git a/app/ui/panels/labels.ts b/app/ui/panels/labels.ts
index 9f31ab9..de82ce7 100644
--- a/app/ui/panels/labels.ts
+++ b/app/ui/panels/labels.ts
@@ -1,5 +1,5 @@
 import type { TFunction } from "i18next";
-import type { KarakaRole } from "../bindings";
+import type { KarakaRole, RisingMode } from "../bindings";
 import { GRAHA_IDS, type ChartLabels, type GrahaId } from "../chart/types";
 
 export function grahaAbbreviations(t: TFunction): Record<GrahaId, string> {
@@ -154,6 +154,9 @@ export function arudhaLabel(house: number): string {
 }
 
 /** Dokuz taranın adı, 1 = Janma … 9 = Parama Mitra (Sanskritçe terim; dile göre değişmez). */
+/** Doğuş biçiminin klasik adı; üç dilde aynı yazılır (K11). Çevrilen kısa ek `prashna.rising.*`'tadır. */
+export const RISING_NAMES = { HeadFirst: "Shirshodaya", BackFirst: "Prishthodaya", BothWays: "Ubhayodaya" } as const satisfies Record<RisingMode, string>;
+
 export const TARA_NAMES = ["Janma", "Sampat", "Vipat", "Kshema", "Pratyak", "Sadhana", "Naidhana", "Mitra", "Parama Mitra"] as const;
 
 /** 1 tabanlı tara numarasının adı; aralık dışıysa numaranın kendisi. */
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index a271209..8588b41 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -17,6 +17,7 @@ import { JaiminiKarakasPanel } from "./JaiminiKarakasPanel";
 import { MuhurtaDayPanel } from "./MuhurtaDayPanel";
 import { MuhurtaSearchPanel } from "./MuhurtaSearchPanel";
 import { PanchangPanel } from "./PanchangPanel";
+import { PrashnaFactsPanel } from "./PrashnaFactsPanel";
 import { ShadbalaPanel } from "./ShadbalaPanel";
 import type { PanelDefinition } from "./types";
 import { UnavailablePanel } from "./UnavailablePanel";
@@ -48,6 +49,8 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   // Muhurta (Plan 20).
   muhurta_day: { titleKey: "panel.muhurta_day", component: MuhurtaDayPanel },
   muhurta_search: { titleKey: "panel.muhurta_search", component: MuhurtaSearchPanel },
+  // Prashna (Plan 21).
+  prashna_facts: { titleKey: "panel.prashna_facts", component: PrashnaFactsPanel },
 };
 
 const UNKNOWN: PanelDefinition = { titleKey: "panel.unknown", component: UnavailablePanel };
diff --git a/app/ui/prashna/prashna.css b/app/ui/prashna/prashna.css
index 0568b0b..97da941 100644
--- a/app/ui/prashna/prashna.css
+++ b/app/ui/prashna/prashna.css
@@ -8,3 +8,62 @@
   color: var(--text-2);
   font-size: 12px;
 }
+
+.prashna-question label {
+  display: block;
+  color: var(--text-2);
+}
+.prashna-question textarea {
+  box-sizing: border-box;
+  width: 100%;
+  resize: vertical;
+  font: inherit;
+}
+.prashna-number {
+  border: 1px solid var(--line);
+  border-radius: 4px;
+  padding: 0 6px;
+}
+.prashna-section h3 {
+  margin: 10px 0 4px;
+  font-size: 13px;
+  font-weight: 500;
+}
+.prashna-list {
+  display: grid;
+  grid-template-columns: max-content 1fr;
+  gap: 2px 12px;
+  margin: 0;
+}
+.prashna-list > div {
+  display: contents;
+}
+.prashna-list dt {
+  color: var(--text-2);
+}
+.prashna-list dd {
+  margin: 0;
+}
+.prashna-list.is-highlighted {
+  background: var(--select);
+}
+.prashna-relation-label {
+  margin: 6px 0 2px;
+  color: var(--text-2);
+}
+.prashna-links {
+  display: flex;
+  flex-wrap: wrap;
+  gap: 4px 8px;
+  margin: 0;
+  padding: 0;
+  list-style: none;
+}
+.prashna-links li {
+  border: 1px solid var(--line);
+  border-radius: 4px;
+  padding: 0 6px;
+}
+.prashna-relation {
+  margin: 6px 0 0;
+}
diff --git a/app/ui/session/store.tsx b/app/ui/session/store.tsx
index c252d1a..5bc57eb 100644
--- a/app/ui/session/store.tsx
+++ b/app/ui/session/store.tsx
@@ -3,7 +3,7 @@
 // önceki haritanın bekleyen görünüm kaydı önce ona yazılır (carried rule C5).
 import { createContext, useCallback, useContext, useMemo, useRef, useState, type ReactNode } from "react";
 import { api } from "../api";
-import type { CalcSettings, Chart, CommandError, OpenedClient } from "../bindings";
+import type { CalcSettings, Chart, CommandError, OpenedClient, PrashnaMeta } from "../bindings";
 import { storeWarningNotices, useNotices } from "../shell/notices";
 import { compatOf } from "../view/model";
 import { useViewStore } from "../view/store";
@@ -33,6 +33,8 @@ export interface Session {
   touch(): void;
   /** Ayar kaydı yeniden hesapladı (`save_calc_settings` dönüşü): açık harita bu haritaysa ayar ve harita değişir. */
   updateSettings(chartId: number, settings: CalcSettings, chart: Chart): void;
+  /** Soru metni ya da odak evi kaydedildi (`update_prashna` dönüşü): açık harita bu haritaysa yan kaydı değişir. */
+  setPrashna(chartId: number, prashna: PrashnaMeta): void;
 }
 
 const SessionContext = createContext<Session | null>(null);
@@ -95,9 +97,14 @@ export function SessionProvider({ initial, children }: { initial: OpenedClient |
     setOpened((o) => (o && o.chart_record.id === chartId ? { ...o, calc_settings: settings, chart } : o));
   }, []);
 
+  const setPrashna = useCallback((chartId: number, prashna: PrashnaMeta) => {
+    // Yalnızca prashna olan açık haritaya yazılır: geç gelen bir yanıt başka haritayı prashna yapmaz.
+    setOpened((o) => (o && o.chart_record.id === chartId && o.prashna !== null ? { ...o, prashna } : o));
+  }, []);
+
   const value = useMemo(
-    () => ({ opened, revision, open, openClient, close, setTags, touch, updateSettings }),
-    [opened, revision, open, openClient, close, setTags, touch, updateSettings],
+    () => ({ opened, revision, open, openClient, close, setTags, touch, updateSettings, setPrashna }),
+    [opened, revision, open, openClient, close, setTags, touch, updateSettings, setPrashna],
   );
   return <SessionContext.Provider value={value}>{children}</SessionContext.Provider>;
 }
diff --git a/app/ui/shell/ChartTitle.tsx b/app/ui/shell/ChartTitle.tsx
index 0bab9b5..0aefbca 100644
--- a/app/ui/shell/ChartTitle.tsx
+++ b/app/ui/shell/ChartTitle.tsx
@@ -19,6 +19,12 @@ export function ChartTitle({ onSettings }: { onSettings?: () => void }) {
     <div className="topbar-title">
       <h1 className="topbar-client">{opened.client.name}</h1>
       <span className="topbar-chart">{label}</span>
+      {opened.prashna && opened.prashna.number !== null && (
+        <span className="topbar-chart topbar-number" title={t("prashna.numberHint")}>
+          {t("prashna.numberBadge", { number: opened.prashna.number })}
+          <span className="visually-hidden">: {t("prashna.numberHint")}</span>
+        </span>
+      )}
       {(warnings.length > 0 || diff.length > 0) && (
         <ul className="badges" aria-label={t("shell.warnings")}>
           {warnings.map((w) => (
diff --git a/app/ui/shell/Shell.tsx b/app/ui/shell/Shell.tsx
index 7bb3af5..9e211ae 100644
--- a/app/ui/shell/Shell.tsx
+++ b/app/ui/shell/Shell.tsx
@@ -6,7 +6,7 @@ import { api } from "../api";
 import type { CommandError } from "../bindings";
 import { BirthForm } from "../birth/BirthForm";
 import { Sidebar } from "../clients/Sidebar";
-import type { ModulePanelKind } from "../layout/model";
+import { BUILTIN_LAYOUT_IDS, type ModulePanelKind } from "../layout/model";
 import type { PaletteItem } from "../palette/items";
 import { Palette } from "../palette/Palette";
 import { Dashboard } from "../dashboard/Dashboard";
@@ -215,7 +215,7 @@ export function Shell() {
       {settings && <SettingsDialog onClose={() => setSettings(false)} />}
       {about && <AboutDialog onClose={() => setAbout(false)} />}
       {form && <BirthForm mode={form} onClose={() => setForm(null)} />}
-      {prashna && <PrashnaForm onClose={() => setPrashna(false)} />}
+      {prashna && <PrashnaForm onClose={() => setPrashna(false)} onCreated={() => layout.select(BUILTIN_LAYOUT_IDS.prashna)} />}
     </div>
   );
 }
````

**Yeni dosya:** `app/ui/modules/usePrashnaFacts.ts`

````ts
// Prashna olgularını (`compute_prashna_facts`) isteyen kanca (ileri modüller spec'i §11). Sonuç
// saklanmaz ve arayüzde hesap yoktur (K1, C4); panel açıkken istenir. Yalnızca prashna
// haritasında çağrılır: başka haritada anahtar `null`'dır ve komut çağrılmaz. Odak evi
// değişince olgular yeniden okunur; panel o sırada son okunan olguları göstermeyi sürdürür
// (`useLastData`). Girdi kişisel veridir: loglanmaz.
import { api } from "../api";
import type { OpenedClient, PrashnaFacts } from "../bindings";
import { chartInput, inputKey } from "./input";
import { useModule, type ModuleState } from "./useModule";

export function usePrashnaFacts(opened: OpenedClient | null, focusHouse: number | null): ModuleState<PrashnaFacts> {
  const key = opened !== null && opened.prashna !== null ? JSON.stringify([inputKey(opened), focusHouse]) : null;
  return useModule(key, () => api.computePrashnaFacts(chartInput(opened as OpenedClient), focusHouse));
}
````

**Yeni dosya:** `app/ui/panels/PrashnaFactsPanel.tsx`

````tsx
// Prashna olguları paneli (ileri modüller spec'i §11): soru metni (düzenlenebilir), odak evi
// seçimi ve soru haritasının olguları: lagna, lagna lordu ve odak evinin lordu, iki lordun bağı,
// Ay, kendralardaki ve odak evindeki grahalar, anın panchangı ve yoga listesi. Değerler
// `compute_prashna_facts`'ten ve açık haritadan gelir; burada hesap yoktur (C4). Hüküm, konu
// çıkarımı ve evet/hayır sonucu yoktur: panel olguları sayar.
//
// Soru metni not gövdesi gibi hassastır: loglanmaz. Soru ve odak evi otomatik kaydedilir
// (`update_prashna`); olgular yeniden okunurken son okunan olgular ve odak yerinde durur.
import { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { CommandError, FocusFacts, LagnaFacts, LordFacts, Occupant, OpenedClient, PrashnaFacts, PrashnaMeta } from "../bindings";
import { signIndex } from "../chart/adapter";
import { useHighlight } from "../highlight/store";
import { errorMessage } from "../i18n/errors";
import { useLastData } from "../modules/useModule";
import { usePrashnaFacts } from "../modules/usePrashnaFacts";
import { MAX_QUESTION_CHARS } from "../prashna/model";
import "../prashna/prashna.css";
import { useSession } from "../session/store";
import { formatDegree } from "./format";
import { nakshatraName, RISING_NAMES, tithiInPaksha, tithiName } from "./labels";
import { panchangRows } from "./PanchangPanel";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";
import { YogaList } from "./YogasPanel";

/** Soru metni yazıldıktan bu kadar sonra kaydedilir. */
export const QUESTION_SAVE_MS = 600;
const HOUSES = Array.from({ length: 12 }, (_, i) => i + 1);
/** Anın panchangından gösterilen satırlar (gün doğumu ve batımı panchang panelindedir). */
const MOMENT_ROWS = ["tithi", "vara", "nakshatra", "yoga", "karana"] as const;

export function PrashnaFactsPanel(_: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  if (opened.prashna === null) {
    return (
      <p className="panel-notice" role="note">
        {t("prashna.notPrashna")}
      </p>
    );
  }
  // Başka bir prashna açılınca taslak ve kayıt durumu sıfırlanır.
  return <Prashna key={opened.chart_record.id} opened={opened} meta={opened.prashna} />;
}

function Prashna({ opened, meta }: { opened: OpenedClient; meta: PrashnaMeta }) {
  const { t, i18n } = useTranslation();
  const session = useSession();
  const id = useId();
  const chartId = opened.chart_record.id;
  const [question, setQuestion] = useState(meta.question);
  const [saveError, setSaveError] = useState<CommandError | null>(null);
  const [saved, setSaved] = useState(false);
  // Kayıtlar sırayla gider: eski metin yenisinin üstüne yazılmaz.
  const chain = useRef<Promise<unknown>>(Promise.resolve());
  const latest = useRef({ question, focus: meta.focus_house, stored: meta.question });
  latest.current = { question, focus: meta.focus_house, stored: meta.question };
  const { setPrashna } = session;

  const save = (text: string, focus: number | null) => {
    const run = chain.current.then(async () => {
      const r = await api.updatePrashna(chartId, { question: text, focus_house: focus });
      if (r.status === "error") {
        setSaved(false);
        setSaveError(r.error);
        return;
      }
      setSaveError(null);
      setSaved(true);
      setPrashna(chartId, r.data);
    });
    chain.current = run;
    return run;
  };
  const saveRef = useRef(save);
  saveRef.current = save;

  // Otomatik kayıt: yazma durunca. Panel kapanırken bekleyen metin hemen gönderilir.
  const tooLong = [...question.trim()].length > MAX_QUESTION_CHARS;
  useEffect(() => {
    if (question.trim() === meta.question || tooLong) return;
    const timer = setTimeout(() => void saveRef.current(question, latest.current.focus), QUESTION_SAVE_MS);
    return () => clearTimeout(timer);
  }, [question, meta.question, tooLong]);
  useEffect(
    () => () => {
      const now = latest.current;
      if (now.question.trim() !== now.stored && [...now.question.trim()].length <= MAX_QUESTION_CHARS) void saveRef.current(now.question, now.focus);
    },
    [],
  );

  const module = usePrashnaFacts(opened, meta.focus_house);
  const facts = useLastData(module, String(chartId));
  const phase = usePanelLoadPhase(module.loading);
  const length = [...question.trim()].length;

  return (
    <div className="module-panel prashna-facts" aria-busy={module.loading && facts !== null ? true : undefined}>
      <div className="prashna-question">
        <label htmlFor={`${id}-question`}>{t("prashna.question")}</label>
        <textarea
          id={`${id}-question`}
          rows={3}
          value={question}
          aria-invalid={tooLong ? true : undefined}
          aria-describedby={`${id}-question-status`}
          onChange={(e) => {
            setSaved(false);
            setQuestion(e.target.value);
          }}
        />
        <p id={`${id}-question-status`} className="panel-message" role="status">
          {tooLong
            ? t("prashna.questionTooLong", { count: length, max: MAX_QUESTION_CHARS })
            : saved && question.trim() === meta.question
              ? t("prashna.questionSaved")
              : t("prashna.questionHint", { count: length, max: MAX_QUESTION_CHARS })}
        </p>
        {saveError && (
          <p className="panel-error" role="alert">
            {t("prashna.saveFailed", { message: errorMessage(t, i18n, saveError) })}{" "}
            <button type="button" onClick={() => void save(question, meta.focus_house)}>
              {t("panel.retry")}
            </button>
          </p>
        )}
      </div>
      <div className="module-toolbar">
        <label htmlFor={`${id}-focus`}>{t("prashna.focus")}</label>
        <select id={`${id}-focus`} value={meta.focus_house ?? ""} onChange={(e) => void save(question, e.target.value === "" ? null : Number(e.target.value))}>
          <option value="">{t("prashna.focusNone")}</option>
          {HOUSES.map((h) => (
            <option key={h} value={h}>
              {t("prashna.focusHouse", { house: h })}
            </option>
          ))}
        </select>
        {meta.number !== null && <span className="prashna-number">{t("prashna.numberBadge", { number: meta.number })}</span>}
      </div>
      {module.error ? (
        <PanelError error={module.error} onRetry={module.retry} />
      ) : facts === null ? (
        phase === "skeleton" ? (
          <PanelSkeleton />
        ) : null
      ) : (
        <Facts facts={facts} opened={opened} meta={meta} />
      )}
    </div>
  );
}

function natureLabel(t: (k: string) => string, o: Occupant): string {
  return `${t(`grahaName.${o.graha}`)} (${t(`prashna.nature.${o.nature}`)})`;
}

function Occupants({ list }: { list: Occupant[] }) {
  const { t } = useTranslation();
  if (list.length === 0) return <>{t("prashna.empty")}</>;
  return (
    <>
      {list.map((o, i) => (
        <span key={o.graha}>
          {i > 0 && ", "}
          <PinButton target={{ type: "graha", id: o.graha }}>{t(`grahaName.${o.graha}`)}</PinButton> ({t(`prashna.nature.${o.nature}`)})
        </span>
      ))}
    </>
  );
}

function LordRow({ role, lord, on }: { role: string; lord: LordFacts; on: boolean }) {
  const { t } = useTranslation();
  const sign = signIndex(lord.sign);
  return (
    <tr data-lord={role} className={on ? "is-highlighted" : undefined}>
      <th scope="row">{t(`prashna.lord.${role}`)}</th>
      <td>
        <PinButton target={{ type: "graha", id: lord.graha }}>{t(`grahaName.${lord.graha}`)}</PinButton>
      </td>
      <td>
        <PinButton target={{ type: "sign", sign }}>{t(`sign.${sign}`)}</PinButton>
      </td>
      <td className="num">{lord.house}</td>
      <td>{lord.dignity ? t(`dignity.${lord.dignity}`) : "—"}</td>
      <td>{[lord.retrograde ? t("prashna.retrograde") : null, lord.combust ? t("prashna.combust") : null].filter(Boolean).join(", ") || "—"}</td>
    </tr>
  );
}

function Facts({ facts, opened, meta }: { facts: PrashnaFacts; opened: OpenedClient; meta: PrashnaMeta }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const lordsId = useId();
  const housesId = useId();
  const yogasId = useId();
  const { lagna, focus, moon } = facts;
  const isGraha = (g: string) => active?.type === "graha" && active.id === g;
  const moonSign = signIndex(moon.sign);
  const moment = panchangRows(t, opened.chart.panchang, opened.chart_record.resolved.offset_secs).filter((r) => (MOMENT_ROWS as readonly string[]).includes(r.key));

  return (
    <>
      {lagna === null ? (
        <p className="panel-notice" role="note">
          {t("prashna.timeUnknown")}
        </p>
      ) : (
        <LagnaSection lagna={lagna} focus={focus} opened={opened} meta={meta} lordsId={lordsId} housesId={housesId} isGraha={isGraha} />
      )}
      <section className="prashna-section" aria-label={t("prashna.moon.title")}>
        <h3>{t("prashna.moon.title")}</h3>
        <dl className={`prashna-list${isGraha("Moon") ? " is-highlighted" : ""}`}>
          <dt>{t("table.sign")}</dt>
          <dd>
            <PinButton target={{ type: "sign", sign: moonSign }}>{t(`sign.${moonSign}`)}</PinButton>
            {moon.house !== null && <> · {t("prashna.focusHouse", { house: moon.house })}</>}
          </dd>
          <dt>{t("panchang.nakshatra")}</dt>
          <dd>
            {nakshatraName(moon.nakshatra)} · {t("panchang.pada", { pada: moon.pada })}
          </dd>
          <dt>{t("panchang.tithi")}</dt>
          <dd>
            {moon.paksha} {tithiInPaksha(moon.tithi)} · {tithiName(moon.tithi)}
          </dd>
          <dt>{t("prashna.moon.aspectedBy")}</dt>
          <dd>
            {moon.aspected_by.length === 0
              ? t("prashna.none")
              : moon.aspected_by.map((g, i) => (
                  <span key={g}>
                    {i > 0 && ", "}
                    <PinButton target={{ type: "graha", id: g }}>{t(`grahaName.${g}`)}</PinButton>
                  </span>
                ))}
          </dd>
        </dl>
      </section>
      <section className="prashna-section" aria-label={t("prashna.moment.title")}>
        <h3>{t("prashna.moment.title")}</h3>
        <dl className="prashna-list">
          {moment.map((r) => (
            <div key={r.key} data-limb={r.key}>
              <dt>{r.label}</dt>
              <dd>{r.value}</dd>
            </div>
          ))}
        </dl>
      </section>
      <section className="prashna-section" aria-labelledby={yogasId}>
        <h3 id={yogasId}>{t("prashna.yogas")}</h3>
        <YogaList report={facts.yogas} filter="all" opened={opened} />
      </section>
      <p className="panel-message">{t("prashna.note")}</p>
      <p className="panel-message">{t("prashna.method", { method: facts.method })}</p>
    </>
  );
}

function LagnaSection({
  lagna,
  focus,
  opened,
  meta,
  lordsId,
  housesId,
  isGraha,
}: {
  lagna: LagnaFacts;
  focus: FocusFacts | null;
  opened: OpenedClient;
  meta: PrashnaMeta;
  lordsId: string;
  housesId: string;
  isGraha(g: string): boolean;
}) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const sign = signIndex(lagna.sign);
  const lagnaIndex = sign;
  // Ev vurgusu D1 evidir: lagna burcundan sayılan burç (gösterim eşlemesi, hesap değil).
  const activeSign = active?.type === "sign" ? active.sign : active?.type === "house" ? (lagnaIndex + active.house - 1) % 12 : null;
  const houses = [...lagna.kendras.map((k) => ({ ...k, focus: false })), ...(focus ? [{ house: focus.house, sign: focus.sign, grahas: focus.occupants, focus: true }] : [])];
  const rel = focus?.relation ?? null;

  return (
    <>
      <section className="prashna-section" aria-label={t("prashna.lagna.title")}>
        <h3>{t("prashna.lagna.title")}</h3>
        <dl className="prashna-list">
          <dt>{t("table.sign")}</dt>
          <dd>
            <PinButton target={{ type: "sign", sign }}>{t(`sign.${sign}`)}</PinButton> {formatDegree(opened.chart.lagna.deg_in_sign ?? 0)}
            {meta.number !== null && <> · {t("prashna.lagna.navamsaMiddle", { number: meta.number })}</>}
          </dd>
          <dt>{t("prashna.lagna.kind")}</dt>
          <dd>{t(`prashna.kind.${lagna.kind}`)}</dd>
          <dt>{t("prashna.lagna.rising")}</dt>
          <dd>
            {RISING_NAMES[lagna.rising]} · {t(`prashna.rising.${lagna.rising}`)}
          </dd>
        </dl>
      </section>
      <section className="prashna-section" aria-label={t("prashna.lords.title")}>
        <h3 id={lordsId}>{t("prashna.lords.title")}</h3>
        <div className="bala-scroll" role="group" tabIndex={0} aria-labelledby={lordsId}>
          <table className="module-table">
            <caption className="visually-hidden">{t("prashna.lords.caption")}</caption>
            <thead>
              <tr>
                <th scope="col">{t("prashna.lords.role")}</th>
                <th scope="col">{t("table.graha")}</th>
                <th scope="col">{t("table.sign")}</th>
                <th scope="col" className="num">
                  {t("prashna.lords.house")}
                </th>
                <th scope="col">{t("prashna.lords.dignity")}</th>
                <th scope="col">{t("prashna.lords.state")}</th>
              </tr>
            </thead>
            <tbody>
              <LordRow role="lagna" lord={lagna.lord} on={isGraha(lagna.lord.graha)} />
              {focus && <LordRow role="focus" lord={focus.lord} on={isGraha(focus.lord.graha)} />}
            </tbody>
          </table>
        </div>
        {focus === null ? (
          <p className="panel-message">{t("prashna.noFocus")}</p>
        ) : rel?.same_graha ? (
          <p className="prashna-relation" data-relation="same">
            {t("prashna.relation.same", { house: focus.house, graha: t(`grahaName.${focus.lord.graha}`) })}
          </p>
        ) : (
          rel && (
            <>
              <p className="prashna-relation-label" id={`${lordsId}-links`}>
                {t("prashna.relation.title")}
              </p>
              <ul className="prashna-links" aria-labelledby={`${lordsId}-links`}>
                {rel.links.length === 0 ? (
                  <li data-link="none">{t("prashna.relation.none")}</li>
                ) : (
                  rel.links.map((link) => (
                    <li key={link} data-link={link}>
                      {t(`prashna.link.${link}`)}
                    </li>
                  ))
                )}
              </ul>
              <p className="panel-message">{t("prashna.relation.counts", { forward: rel.focus_lord_from_lagna_lord, backward: rel.lagna_lord_from_focus_lord })}</p>
            </>
          )
        )}
      </section>
      <section className="prashna-section" aria-label={t("prashna.houses.title")}>
        <h3 id={housesId}>{t("prashna.houses.title")}</h3>
        <div className="bala-scroll" role="group" tabIndex={0} aria-labelledby={housesId}>
          <table className="module-table">
            <caption className="visually-hidden">{t("prashna.houses.caption")}</caption>
            <thead>
              <tr>
                <th scope="col">{t("prashna.houses.house")}</th>
                <th scope="col">{t("table.sign")}</th>
                <th scope="col">{t("prashna.houses.grahas")}</th>
              </tr>
            </thead>
            <tbody>
              {houses.map((h) => {
                const s = signIndex(h.sign);
                const on = activeSign === s || h.grahas.some((o) => isGraha(o.graha));
                return (
                  <tr key={`${h.focus ? "focus" : "kendra"}:${h.house}`} data-house={h.house} data-focus={h.focus ? "true" : undefined} className={on ? "is-highlighted" : undefined}>
                    <th scope="row">
                      {t("prashna.focusHouse", { house: h.house })}
                      {h.focus && <> · {t("prashna.houses.focus")}</>}
                    </th>
                    <td>
                      <PinButton target={{ type: "sign", sign: s }}>{t(`sign.${s}`)}</PinButton>
                    </td>
                    <td aria-label={h.grahas.length === 0 ? undefined : h.grahas.map((o) => natureLabel(t, o)).join(", ")}>
                      <Occupants list={h.grahas} />
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </section>
    </>
  );
}
````

- [ ] **Step 4: Testleri ve kapıları koş**

```bash
pnpm --dir app exec vitest run ui/panels/PrashnaFactsPanel.test.tsx ui/layout ui/shell ui/session ui/view ui/palette ui/App.test.tsx ui/i18n 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
lsof -i :1421
pnpm --dir app e2e 2>&1 | tail -1
pnpm --dir app e2e:visual 2>&1 | tail -1
rm -rf app/test-results
git status --short -- app/e2e
```
Expected: seçilen dosyalar yeşil; tamamı `83 passed (83)`, `840 passed (840)`; `✓ built`; `lsof` çıktısız; e2e `22 passed` (dördüncü hazır düzen var olan akışları bozmaz); görsel `10 passed`; `git status` çıktısız (temel PNG'ler değişmedi: D27).

- [ ] **Step 5: Commit**

```bash
git add -A
git status --short
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached -U0 -- 'app/ui/**/*.css' | grep '^+' | grep -E '#[0-9a-fA-F]{3,8}\b|rgb\('
git commit -m "feat(ui): prashna facts panel with the editable question and focus house; Prashna layout; number badge" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: üç yeni dosya; iki `grep` çıktısız.

---

### Task 7: Uçtan uca: prashna kurma ve olgu paneli

Spec §11 "Arayüz"; K12; UI spec §8 (uçtan uca). İki test WebKit'te, Tauri IPC taklidiyle koşar (Task 4'ün `e2e/ipc.ts` eklemeleri): ⌘⇧P ile açık danışana sayı prashnası kurulur, Prashna düzeni ve olgular çizilir, odak evi ve soru panelden değişir; açık danışan yokken paletten, aranan bir yerle ve girilen bir anla kurulan prashna yerleşik danışanın altında açılır.

**Files:**
- Modify: `app/e2e/desk.spec.ts`

**Interfaces:**
- Consumes: Task 4–6'nın tamamı; `mockIpc`, `ipcCalls`, `FIXTURE`.
- Produces: iki e2e testi (toplam 24).

- [ ] **Step 1: Testleri yaz**

Yer listesinden seçim klavyeyle yapılır (pencerede bir de `<select>` seçenekleri vardır: rolüyle ilk "option" o olurdu). "Panchang" paneli adıyla tam eşleşmeyle aranır ("Anın panchangı" bölümü de aynı sözcüğü taşır).

````diff
diff --git a/app/e2e/desk.spec.ts b/app/e2e/desk.spec.ts
index 5cb0acd..eb6bea1 100644
--- a/app/e2e/desk.spec.ts
+++ b/app/e2e/desk.spec.ts
@@ -948,3 +948,116 @@ test("muhurta search: the filter form, paged windows, a saved filter, and a row
   await expect(results.locator("tbody tr")).toHaveCount(30);
   await expect(first).toBeFocused();
 });
+
+test("prashna: Cmd+Shift+P casts a number prashna for the open client; the Prashna layout, the facts, the focus house and the question", async ({ page }) => {
+  await mockIpc(page, { session: true });
+  await page.goto("/");
+  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
+  await page.keyboard.press("Meta+Shift+p");
+  const sheet = page.getByRole("dialog", { name: "Yeni prashna" });
+  await expect(sheet.getByRole("radio", { name: "Şimdi" })).toBeFocused();
+  // Bulunulan yer seçilmemiş: form yer ister, haritanın yerine kendiliğinden düşmez ve komut çağırmaz.
+  await expect(sheet.locator(".prashna-place")).toContainText("Bulunulan yer seçilmemiş.");
+  await sheet.getByRole("button", { name: "Prashna kur" }).click();
+  await expect(sheet.getByRole("combobox", { name: "Yer" })).toBeFocused();
+  await expect(sheet.getByText("Bir yer seçin.")).toBeVisible();
+  expect(await ipcCalls(page, "create_prashna")).toHaveLength(0);
+  await sheet.getByRole("combobox", { name: "Yer" }).selectOption("chart");
+  await expect(sheet.locator(".prashna-place")).toHaveText("New Delhi, Delhi, India · Asia/Kolkata");
+  await sheet.getByRole("textbox", { name: "Sayı (isteğe bağlı)" }).fill("54");
+  await sheet.getByRole("textbox", { name: "Soru" }).fill("Deneme sorusu");
+  await sheet.getByRole("combobox", { name: "Odak evi" }).selectOption("7");
+  await sheet.getByRole("button", { name: "Prashna kur" }).click();
+  await expect(page.getByRole("dialog")).toHaveCount(0);
+
+  // İstek: an "şimdi" (Rust okur), yer haritanın yeri, sayı ve soru yazıldığı gibi; açık danışana.
+  type Request = { request: { client_id: number | null; at: unknown; place: { place: { name: string }; tz: unknown }; number: number | null; question: string; focus_house: number | null } };
+  const [cast] = (await ipcCalls(page, "create_prashna")) as Request[];
+  expect(cast.request).toMatchObject({ client_id: 1, at: "Now", number: 54, question: "Deneme sorusu", focus_house: 7 });
+  expect(cast.request.place).toEqual(FIXTURE.muhurta.home_place);
+
+  // Harita açılır ve Prashna düzenine geçilir: D1, olgular, panchang ve notlar.
+  await expect(page.getByRole("combobox", { name: "Düzen" })).toHaveValue("builtin:prashna");
+  await expect(page.locator(".topbar-chart").first()).toHaveText("Prashna 2026-11-25 15:30");
+  await expect(page.locator(".topbar-number")).toContainText("sayı: 54");
+  const facts = page.getByRole("region", { name: "Prashna olguları" });
+  await expect(page.getByRole("region", { name: "Panchang", exact: true })).toBeVisible();
+  await expect(page.getByRole("region", { name: "Notlar" })).toBeVisible();
+  await expect(facts.getByRole("textbox", { name: "Soru" })).toHaveValue("Deneme sorusu");
+  // Lagna sayıdan gelir (54 → Başak 28°20′, navamsa ortası); modül girdisi sayıyı taşır.
+  await expect(facts.getByRole("region", { name: "Lagna" })).toContainText("Baş 28°20′ · 54. navamsanın ortası");
+  await expect(facts.getByRole("region", { name: "Lagna" })).toContainText("Shirshodaya · başla doğan");
+  type FactsCall = { input: { jd_ut: number; time_known: boolean; lagna_number: number | null }; focusHouse: number | null };
+  const factCalls = async () => (await ipcCalls(page, "compute_prashna_facts")) as FactsCall[];
+  expect(await factCalls()).toHaveLength(1);
+  expect((await factCalls())[0]).toMatchObject({ input: { jd_ut: FIXTURE.prashna.opened_number.chart.jd_ut, time_known: true, lagna_number: 54 }, focusHouse: 7 });
+  const lords = facts.getByRole("table", { name: "Lagna lordu ve odak evinin lordu" });
+  await expect(lords.locator("tbody tr")).toHaveCount(2);
+  await expect(lords.locator("tbody tr").first().locator("th, td").first()).toHaveText("Lagna lordu");
+  await expect(facts.getByRole("region", { name: "Ay" })).toContainText("Rohini");
+  await expect(facts.getByText("Bu panel olguları sayar; sorunun yanıtını ya da bir hükmü söylemez.")).toBeVisible();
+
+  // Odak evi panelden değişir: kaydedilir, olgular yeniden okunur, odak seçicide durur.
+  const focus = facts.getByRole("combobox", { name: "Odak evi" });
+  await focus.focus();
+  await focus.selectOption("10");
+  await expect.poll(async () => (await factCalls()).length).toBe(2);
+  expect((await factCalls())[1].focusHouse).toBe(10);
+  await expect(facts.locator('tr[data-focus="true"] th')).toHaveText("10. ev · odak");
+  await expect(focus).toBeFocused();
+  // Soru yazdıkça kaydedilir; olgular yeniden hesaplanmaz.
+  await facts.getByRole("textbox", { name: "Soru" }).fill("Deneme sorusu, ikinci yazım");
+  await expect(facts.getByText("Kaydedildi")).toBeVisible();
+  type Edit = { chartId: number; edit: { question: string; focus_house: number | null } };
+  const edits = (await ipcCalls(page, "update_prashna")) as Edit[];
+  expect(edits.at(-1)).toEqual({ chartId: FIXTURE.prashna.opened_number.chart_record.id, edit: { question: "Deneme sorusu, ikinci yazım", focus_house: 10 } });
+  expect(await factCalls()).toHaveLength(2);
+  // Lagna lordu ortak vurgudur: haritadaki graha da işaretlenir.
+  await lords.getByRole("button", { name: "Merkür" }).click();
+  await expect(lords.locator('tr[data-lord="lagna"]')).toHaveClass(/is-highlighted/);
+});
+
+test("prashna without an open client: the palette command, a searched place, a typed moment; the chart opens under the built-in client", async ({ page }) => {
+  await mockIpc(page, { session: false });
+  await page.goto("/");
+  await expect(page.getByRole("heading", { level: 1, name: "Jyotish Desk" })).toBeVisible();
+  await page.keyboard.press("Meta+k");
+  const palette = page.getByRole("dialog", { name: "Komut paleti" });
+  await palette.getByRole("combobox").fill("prashna");
+  await palette.getByRole("option", { name: /Yeni prashna/ }).click();
+  const sheet = page.getByRole("dialog", { name: "Yeni prashna" });
+  // Açık danışan yok: danışan kutusu ve "açık haritanın yeri" seçeneği yoktur.
+  await expect(sheet.getByRole("checkbox")).toHaveCount(0);
+  await expect(sheet.getByText(/Açık danışan yok: prashna “Prashna” adlı yerleşik danışanın altında toplanır\./)).toBeVisible();
+  await expect(sheet.getByRole("combobox", { name: "Yer" }).locator("option")).toHaveText(["Bulunulan yer", "Başka bir yer ara"]);
+  await sheet.getByRole("combobox", { name: "Yer" }).selectOption("search");
+  await sheet.getByRole("combobox", { name: "Yer ara" }).fill("new delhi");
+  await expect(sheet.getByRole("listbox").getByRole("option").first()).toBeVisible();
+  await page.keyboard.press("Enter");
+  await expect(sheet.locator(".prashna-place")).toContainText("Asia/Kolkata");
+  // Girilen an sorunun yerinin duvar saatidir; çözümü Rust yapar.
+  await sheet.getByRole("radio", { name: "Başka bir an" }).check();
+  await sheet.getByLabel("Tarih").fill("2026-11-25");
+  await sheet.getByLabel("Saat").fill("15:30");
+  await sheet.getByRole("button", { name: "Prashna kur" }).click();
+  await expect(page.getByRole("dialog")).toHaveCount(0);
+  const [cast] = (await ipcCalls(page, "create_prashna")) as { request: { client_id: number | null; at: unknown; number: number | null; focus_house: number | null } }[];
+  expect(cast.request).toMatchObject({ client_id: null, at: { Local: { date: "2026-11-25", time: "15:30" } }, number: null, focus_house: null });
+
+  // Yerleşik danışan açılır; zaman prashnasında sayı rozeti yoktur ve odak evi seçilmemiştir.
+  await expect(page.getByRole("heading", { level: 1, name: "Prashna" })).toBeVisible();
+  await expect(page.getByRole("combobox", { name: "Düzen" })).toHaveValue("builtin:prashna");
+  await expect(page.locator(".topbar-number")).toHaveCount(0);
+  const facts = page.getByRole("region", { name: "Prashna olguları" });
+  await expect(facts.getByRole("combobox", { name: "Odak evi" })).toHaveValue("");
+  await expect(facts.getByText("Odak evi seçilmedi: odak evinin lordu ve iki lordun bağı gösterilmez.")).toBeVisible();
+  const calls = (await ipcCalls(page, "compute_prashna_facts")) as { input: { lagna_number: number | null }; focusHouse: number | null }[];
+  expect(calls).toEqual([expect.objectContaining({ focusHouse: null })]);
+  expect(calls[0].input.lagna_number).toBeNull();
+  // Olgular paneli paletten kapatılıp açılabilir (K12); kapatınca odak gövdeye düşmez.
+  await page.keyboard.press("Meta+k");
+  await page.getByRole("dialog", { name: "Komut paleti" }).getByRole("combobox").fill("prashna olgu");
+  await page.getByRole("dialog", { name: "Komut paleti" }).getByRole("option", { name: /Prashna olguları panelini aç\/kapat/ }).click();
+  await expect(page.getByRole("region", { name: "Prashna olguları" })).toHaveCount(0);
+  await expect(page.locator("body")).not.toBeFocused();
+});
````

- [ ] **Step 2: Koş**

```bash
lsof -i :1421
pnpm --dir app exec tsc --noEmit -p e2e 2>&1 | head -3
pnpm --dir app e2e 2>&1 | tail -1
pnpm --dir app e2e 2>&1 | tail -1
rm -rf app/test-results
```
Expected: `lsof` ve `tsc` çıktısız; iki koşuda da `24 passed`.

- [ ] **Step 3: Commit**

```bash
git add -A
git status --short
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "test(e2e): prashna from the shortcut and the palette; number prashna, the Prashna layout, focus house and question; the built-in client" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```
Expected: yalnızca `app/e2e/desk.spec.ts`; `grep` çıktısız.

---

### Task 8: Son doğrulama, elle denetim kapısı, el örneklerinin gözden geçirilmesi ve klasik tablonun onayı

Kod değişmez; bulunan bir hata kendi düzeltme commit'iyle çözülür ve bu görev baştan koşulur. Tek dosya değişikliği, `REVIEW.md`'deki iki kayıt tablosuna yazılan satırlardır (Step 5 ve Step 6).

**Files:**
- Modify: `tests/golden/modules/prashna/REVIEW.md` (yalnızca iki kayıt tablosu). İlerleme notları depo dışıdır; `git add` edilmez.

**Interfaces:**
- Consumes: Task 1–7'nin tamamı.
- Produces: yeşil kapılar, kural ve gizlilik taramaları, elle denetim sonucu, sahibine devredilen maddeler, el örneklerinin bağımsız gözden geçirmesi ve klasik tablonun onay kaydı (`REVIEW.md`'ye ve ilerleme notlarına yazılır).

- [ ] **Step 1: Rust kapıları**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
cargo test -p jyotish-desk --test modules -- --nocapture 2>&1 | grep -E "median|muhurta_search"
cargo test -p jyotish-desk --test open_client -- --nocapture 2>&1 | grep -E "median"
cargo deny check 2>&1 | tail -1
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p jyotish-core -p atlas -p store -p jyotish-desk --all-targets 2>&1 | tail -1
git diff 0b61716..HEAD --stat -- deny.toml Cargo.toml Cargo.lock app/package.json app/pnpm-lock.yaml app/vite.config.ts app/src-tauri/capabilities app/src-tauri/tauri.conf.json app/src-tauri/Cargo.toml crates/store/Cargo.toml crates/atlas crates/ephemeris crates/jyotish-core/Cargo.toml crates/jyotish-core/src/settings.rs crates/jyotish-core/src/panchang.rs crates/jyotish-core/src/timeline.rs crates/jyotish-core/src/muhurta crates/jyotish-core/src/dasha.rs crates/jyotish-core/src/dignity.rs crates/jyotish-core/src/nature.rs crates/jyotish-core/src/lordship.rs crates/store/src/view_state.rs crates/store/src/settings.rs crates/store/src/charts.rs crates/store/src/notes.rs crates/store/src/search.rs
git diff 0b61716..HEAD -U0 -- crates/store/src/schema.rs | grep -c '^-[^-]'
ls crates/jyotish-core/proptest-regressions 2>/dev/null
```
Expected: fmt ve clippy çıktısız; testler: `jyotish-core` lib `374 passed`, golden `8 passed`, `modules_golden` `17 passed`; `atlas` `moment_fixed_lmt` `11 passed`, `moment_iana` `13 passed`; `store` lib `112 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `54 passed`, `bindings` `1 passed`, `compute_chart` `1 passed`, `modules` `1 passed`, `open_client` `1 passed`, `ui_fixture` `2 passed`; sayaç `0`; on bir `median` satırı ve üç `muhurta_search` satırı, hepsi bütçesinin içinde (`compute_prashna_facts` ≤ 15 ms); `create_prashna median` 100 ms'nin, `open_client median` 50 ms'nin altında; deny `advisories ok, bans ok, licenses ok, sources ok`; MSRV `Finished`; `git diff --stat` çıktısız (bağımlılık, yetenek, yapılandırma, ayar tipi, panchang, zaman çizelgesi, muhurta, dasha, dignity, doğal sınıf, lordluk, `ViewState`, tercihler, harita kayıtları, notlar ve arama değişmedi); `schema.rs` için `2` (yalnızca göç listesi ve `SCHEMA_VERSION` satırları değişti; göç 1'in metni değişmedi: D11); `ls` çıktısız. Taban `0b61716` değilse komutlardaki commit, Task 1 adım 1'de not edilen tabandır.

- [ ] **Step 2: Arayüz kapıları**

```bash
lsof -i :1421
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
pnpm --dir app e2e 2>&1 | tail -1
pnpm --dir app e2e:visual 2>&1 | tail -1
rm -rf app/test-results
node --test "scripts/release/*.test.mjs" 2>&1 | grep -E "ℹ (pass|fail)"
git status --short
```
Expected: `lsof` çıktısız; `Test Files  83 passed (83)`, `Tests  840 passed (840)`; `✓ built`; e2e `24 passed`; görsel `10 passed` (temel PNG'ler değişmedi: D27); betik testleri `pass 14`, `fail 0`; `git status --short` boş.

- [ ] **Step 3: Kural ve gizlilik taramaları (`0b61716`'den bu yana)**

```bash
git diff 0b61716..HEAD -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git log 0b61716..HEAD --format=%B | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff 0b61716..HEAD --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
git diff 0b61716..HEAD --name-only | grep -vE '^(app|crates|scripts|tests|docs)/'
echo "trailers: $(git log 0b61716..HEAD --format=%B | grep -c '^Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>$') commits: $(git rev-list --count 0b61716..HEAD)"
git diff 0b61716..HEAD -U0 -- app/ui | grep '^+' | grep -E 'console\.(log|info|warn|error|debug)'
git diff 0b61716..HEAD -U0 -- crates app/src-tauri/src | grep '^+' | grep -E 'println!|eprintln!|dbg!|log::|tracing::'
git diff 0b61716..HEAD -U0 -- app/ui | grep '^+' | grep -E '\b(alert|confirm|prompt)\(|window\.(print|open)\('
git diff 0b61716..HEAD -U0 -- 'app/ui/**/*.css' | grep '^+' | grep -E '#[0-9a-fA-F]{3,8}\b|rgb\('
grep -rn '"source"' tests/golden/modules | grep -v '"source": "hand"'
ls tests/golden/modules/prashna/*.json | wc -l
grep -rnE '#\[serde\(default' crates/jyotish-core/src/prashna crates/store/src/prashna.rs crates/store/src/model.rs app/src-tauri/src/prashna_commands.rs
grep -rnE 'XMLHttpRequest|WebSocket|\bfetch\(' app/ui/prashna app/ui/panels/PrashnaFactsPanel.tsx app/ui/modules/usePrashnaFacts.ts
grep -rnE 'localStorage|sessionStorage|new Date\(|Date\.now' app/ui/prashna app/ui/panels/PrashnaFactsPanel.tsx app/ui/modules/usePrashnaFacts.ts
grep -cE 'prashna_commands::(create_prashna|update_prashna|compute_prashna_facts)' app/src-tauri/src/lib.rs
grep -cE 'CommandError::new\(' app/src-tauri/src/prashna_commands.rs
grep -n "question" crates/store/src/schema.rs | grep -iE 'fts|trigger'
git diff 0b61716..HEAD -U0 -- crates/store/src/exchange.rs | grep '^+' | grep -c 'include_notes.then_some(p.question)'
git diff 0b61716..HEAD --stat -- app/ui/chart app/ui/print app/e2e/visual.spec.ts-snapshots
git tag --points-at HEAD
```
Expected: `trailers:` satırındaki iki sayı eşit; `grep -ciE` satırı `0`, `ls … | wc -l` satırı `14`, komut sayacı `3` (üç komut kayıtlıdır), `CommandError::new` sayacı `1` (komut dosyası yalnızca `invalid_prashna_input` üretir; öteki her hata `store_error` eşlemesinden geçer: D10), dışa aktarım sayacı `1` (soru yalnızca notlarla yazılır) yazar; bunların dışındaki her komut çıktısızdır (çıktısız `grep` 1 ile çıkar; beklenen budur): yasaklı ad yok (üretilen fikstürde de), beklenmeyen yerde dosya yok, arayüzde `console` ve Rust'ta log çağrısı yok (soru, yer ve an loglanmaz: C1), yasak çağrı yok, yeni stilde hex ya da `rgb(` yok (C9), el örneklerinde `hand` dışında kaynak yok (K9), IPC tiplerinde `serde(default)` yok (yalnızca dosya biçimi olan `ExportChart.prashna` taşır; o dosya bu listede değildir), prashna kodunda ağ çağrısı, tarayıcı deposu ve saat okuması yok ("şimdi" Rust'ta okunur: C4), soru metni arama dizinine bağlanmaz, `ChartSvg`, baskı ve görsel temeller değişmedi (K10, D27), ajan etiket atmamış.

- [ ] **Step 4: Elle denetim kapısı (`pnpm --dir app tauri dev`)**

Gerçek uygulamada, **yalnızca deneme danışanları ve deneme sorularıyla** (gerçek danışan verisi, gerçek soru ve gerçek ev adresi kullanılmaz; yer olarak bir kent seçilir). Her madde ilerleme notlarına "geçti" ya da bulgu olarak yazılır; bulgu varsa düzeltme commit'i ve bu görev baştan. Her maddenin başında, otomatik bir karşılığı olup olmadığı yazılıdır. **[otomatik]** maddelerin karşılığı testlerde vardır ve yürütücü ajan bunları testin adını göstererek "geçti" sayabilir; **[insan]** maddeler yalnızca gerçek uygulamaya bakan bir insan tarafından doğrulanabilir ve sahibi yoksa ona devredilir (bu devir `git push`'u engellemez; hiçbir şey yayımlanmaz).

- [ ] **[otomatik]** ⌘⇧P ve paletteki "Yeni prashna" komutu pencereyi açık harita olsun olmasın açar; baskı önizlemesinde açmaz; ⌘P'nin anlamı değişmez. (`App.test.tsx › Cmd+Shift+P and the palette …`, `shortcuts.test.ts`, e2e)
- [ ] **[otomatik]** Pencere "şimdi", bulunulan yer, sayısız ve odak evsiz açılır; bulunulan yer seçilmemişse yer ister ve haritanın yerine sessizce düşmez; atlastan yer aranır; girilen an sorunun yerinin duvar saatidir. (`PrashnaForm.test.tsx`, `prashna/model.test.ts`, `prashna_commands.rs › now_becomes_…`, `› a_typed_moment_…`, e2e)
- [ ] **[otomatik]** Sayı prashnasında lagna sayıdan gelir (navamsa ortası), grahalar anın grahalarıdır ve yalnızca lagna ile evler değişir; gerçek lagna aynı haritayı verir; her modül aynı lagnayı görür. (çekirdek `prashna_properties`, `chart.rs` testleri; Rust `modules` testi `prashna_checks`; `open_client` testi `prashna_round_trip`)
- [ ] **[otomatik]** "Prashna kur" haritayı açar ve Prashna düzenine geçer (D1, olgular, panchang, notlar); sayı prashnasında başlıkta "sayı: n" ve lagna satırında "n. navamsanın ortası" yazar; danışansız prashna yerleşik "Prashna" danışanının altında açılır. (`App.test.tsx › casting a prashna …`, `ChartTitle.test.tsx`, `store/prashna.rs › without_a_client_…`, e2e)
- [ ] **[otomatik]** Olgu paneli lagnayı, iki lordu ve bağlarını, kendraları ve odak evini, Ay'ı, anın panchangını ve yoga listesini çizer; hüküm sözü yoktur; prashna olmayan haritada not çıkar ve komut çağrılmaz. (`PrashnaFactsPanel.test.tsx › draws the facts …`, `› a chart that is not a prashna …`; çekirdek el fikstürleri)
- [ ] **[otomatik]** Odak evi panelden değişir: kaydedilir, olgular yerinde yeniden okunur, odak seçicide durur; soru yazdıkça kaydedilir ve olgular yeniden hesaplanmaz; panel kapanırken bekleyen metin gönderilir; kayıt hatası görünür ve yeniden denenir. (`PrashnaFactsPanel.test.tsx › the focus house is changed …`, `› the question is saved …`, `› a pending question …`, e2e)
- [ ] **[otomatik]** Okuma hatası panelin içindedir; soru ve odak evi yerinde durur; yeniden deneme hata öncesi olguları göstermez. (`PrashnaFactsPanel.test.tsx › an error keeps …`)
- [ ] **[otomatik]** Saati bilinmeyen haritada lagnaya bağlı bölümler Rust'tan boş gelir ve panel bunu söyler. (`PrashnaFactsPanel.test.tsx › a chart whose time is unknown …`; çekirdek `without_a_lagna_…`, `collector_follows_…`)
- [ ] **[otomatik]** Kur → başka haritayı aç → yeniden aç soru, sayı ve odak evini korur; ayar değişikliği ve doğum formundan düzenleme sayının lagnasını korur; harita silinince yan satır silinir. (`open_client` testi; `store/prashna.rs › question_number_and_focus_house_round_trip_…`, `› deleting_the_chart_…`)
- [ ] **[otomatik]** Soru metni aramada bulunmaz, varsayılan JSON dışa aktarımda ve CSV'de yoktur, "notları dahil et" ile yazılır; içe aktarım yan satırı geri kurar. (`store/prashna.rs › the_question_is_not_searchable_…`, `› import_restores_…`)
- [ ] **[otomatik]** Hatalar yalnızca alan adı taşır; yer, an, sayı ve soru hata parametresine girmez; yer her şeyden önce denetlenir. (`prashna_commands.rs › errors_name_the_field_…`; Rust `modules` ve `open_client` testleri; `commands.rs`)
- [ ] **[otomatik]** Sürüm 1 veritabanı açılışta sürüm 2'ye çıkar ve var olan haritalar durur. (`schema.rs › a_version_one_database_gains_the_prashna_table`)
- [ ] **[otomatik]** Her doğuş biçiminin, bağın, türün ve sınıfın üç dilde adı vardır; klasik adlar üç dilde aynıdır; Fransızca metinlerde düz kesme işareti yoktur. (`PrashnaFactsPanel.test.tsx › every rising mode …`, `PrashnaForm.test.tsx › the form texts …`, `i18n.test.ts`)
- [ ] **[insan]** Gerçek veritabanının **yedeği alındıktan sonra** uygulama açılır: şema sürüm 2'ye çıkar, var olan danışanlar ve haritalar eksiksizdir; bu sürümle açılmış veritabanını önceki bir sürümün açmadığı (`store_too_new`) bilinir ve sürüm notuna yazılır.
- [ ] **[insan]** Gerçek motorla tam akış: ⌘⇧P ile "şimdi" ve bulunulan yerle bir zaman prashnası kurulur; haritanın anı o anın yerel saatidir ve etiket onu taşır; uygulama yeniden başlatılınca prashna, sorusu ve odak evi geri gelir; JSON dışa aktarım dosyasında soru metni yalnızca "notları dahil et" işaretliyken vardır.
- [ ] **[insan]** Sayı prashnası (ör. 54) kurulur: D1 Başak lagnasını çizer; paletten eklenen yoga, bakış ve Jaimini panelleri aynı lagnadan sayar; ayarlardan ev sistemi değiştirilince lagna ve evler değişmez.
- [ ] **[insan]** Yaz saati uygulayan bir yerde (ör. Berlin) geçiş gecesinin boşluğuna düşen bir an girilince pencere alanın altında iletiyi gösterir ve hiçbir şey kurulmaz; örtüşen saatte de aynı ileti çıkar ve önerilen yol (bir dakika öncesi, sonra ⌘E) işler.
- [ ] **[insan]** Düzen seçicide dördüncü hazır düzen "Prashna"dır; ⌘4 ona, ⌘5 kullanıcının ilk düzenine gider (önceki sürümde ⌘4 idi: sürüm notu maddesi).
- [ ] **[insan]** Pencere ve panel açık ve koyu temada, Türkçe, İngilizce ve Fransızca dilde okunur; dar panelde tablolar yatay kaydırılır; sınıflar, bağlar ve lordun durumu renk olmadan da ayırt edilir.
- [ ] **[insan]** VoiceOver: pencere başlığıyla duyurulur; hata alanın açıklaması olarak okunur; panelde bölümler adlarıyla, tablolar başlık ve sütun adlarıyla, "İki lordun bağı" listesi adıyla okunur; "Kaydedildi" duyurulur.
- [ ] **[insan]** Bütün denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, soru metni, yer, koordinat, an ya da dosya yolu görünmez.

Kapı geçtiyse ilerleme notlarına `Plan 21 Task 8 elle denetim: geçti (<tarih>)` yazılır; **[insan]** maddeleri devredildiyse `Plan 21 Task 8 elle denetim: otomatik maddeler geçti, insan maddeleri sahibine devredildi (<tarih>)` yazılır. Bu adım commit üretmez.

- [ ] **Step 5: El örneklerinin bağımsız gözden geçirilmesi (gözden geçirme maddesi)**

Spec §13: "`derivation` ayrı bir gözden geçirmede kuraldan yeniden hesaplanır." Gözden geçiren (motoru yazan dışında bir ajan ya da insan), `tests/golden/modules/prashna/` altındaki on dört örneğin her birinde yalnızca `input`'tan başlar ve motoru çalıştırmadan, Rust kaynağını okumadan, spec §11'den ve `REVIEW.md`'deki okuma kuralları ile tablolardan yeniden hesaplar: (1) lagna (sayıdan ya da boylamdan), burcu ve burç içi derecesi; (2) burç türü ve doğuş biçimi; (3) lagna lordunun burcu, evi, dignity'si, retrosu ve yanması; (4) dört kendranın burcu ve grahaları, sınıflarıyla; (5) odak evinin burcu ve lordu, lordun aynı beş alanı; (6) iki lordun bağları ve iki ev sayısı; (7) odak evindeki grahalar; (8) Ay'ın burcu, evi, nakshatrası, padası, tithisi ve pakshası; (9) Ay'ın burcuna bakanlar. Sonra kendi sonucunu `expected` ile karşılaştırır; farklar yazılır. Fark varsa önce kural yeniden okunur; düzeltme hangi tarafta olursa olsun ayrı commit'tir ve beklenen değer motor çıktısına bakarak değiştirilmez.

Gözden geçiren, `REVIEW.md`'nin sonundaki "Gözden geçirme kaydı" tablosuna adını (ya da ajan kimliğini), tarihi ve sonucu yazar:

```bash
git add tests/golden/modules/prashna/REVIEW.md
git diff --cached --stat
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "docs(golden): prashna hand fixtures re-derived by an independent reviewer" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
cargo test -p jyotish-core --test modules_golden 2>&1 | grep -E "^test result"
```
Expected: yalnızca `REVIEW.md` değişir (tablo satırı; fark bulunduysa ilgili örnek ayrı commit'tedir); `grep` çıktısız; `17 passed`. Sonuç ilerleme notlarına `Plan 21 el örnekleri yeniden türetildi: 14/14 (<tarih>, <gözden geçiren>)` olarak yazılır.

- [ ] **Step 6: [insan] Bellekten aktarılan klasik tablonun ve yöntemin onayı**

Doğuş biçimi tablosu ve sayının lagnası spec'te yazılıdır ve bu depoya bellekten aktarılmıştır (D3, D8). Bu adım yalnızca elinde basılı bir klasik kaynak ya da prashna derlemesi bulunan bir insan tarafından yapılabilir; yürütücü ajan bu adımı **yapmaz, başka bir yazılımın çıktısıyla ya da çevrimiçi bir tabloyla ikame etmez** ve `REVIEW.md`'deki "Bekliyor: sahibine devredildi" satırını olduğu gibi bırakır. Denetleyen kişi `REVIEW.md`'deki doğuş biçimi tablosunu ve "bilinen varyant noktaları" listesinin yedi satırını (İkizler'in doğuş biçimi; Balık; sayının lagnası; sayı haritasının evleri; iki lordun bakışı; değişim; Ay'ın aldığı bakışlar) kaynakla karşılaştırır; kaynağı genel adıyla (alıntı yapmadan) ve sonucu tabloya yazar. Fark bulunursa: tablo ayrı bir commit'le düzeltilir, `REVIEW.md` güncellenir, etkilenen el örnekleri kuraldan yeniden türetilir (motor çıktısından değil) ve Step 1 ile Step 5 yeniden koşulur; ikinci bir okuma istenirse (ör. navamsanın başı) K7'ye göre ancak ikinci seçenek uygulanınca ayar olur. Bu madde devredildiyse ilerleme notlarına `Plan 21 prashna tablosu: onay sahibine devredildi (<tarih>)` yazılır; devir `git push`'u engellemez, ama **tablo onaylanana dek doğuş biçimi ve sayı prashnası beta astrologlarına "doğrulanmamış tablo" notuyla sunulur** (sürüm notu sahibinin işidir).

---

## Başarı ölçütleri

- `jyotish-core` lagnası verilen haritayı kurar: grahalar, dasha ve panchang `compute_chart`'ınkidir; lagna, burç evleri ve lagnaya bağlı türevler verilen lagnadandır; gerçek lagna aynı haritayı verir (burç evi sisteminde). Sayı 1–108 kendi navamsasının ortasına düşer; 0 ve 109 alan adıyla reddedilir. Sayı, modül girdisinin alanıdır ve her modül aynı lagnayı görür.
- Prashna olguları saf bir fonksiyondan gelir: lagnanın burcu, türü ve doğuş biçimi; lagna lordu ve odak evinin lordu (burç, ev, dignity, retro, yanma); iki lordun geçerli bütün bağları ve birbirinden ev sayıları; Ay (burç, ev, nakshatra, tithi, paksha, aldığı bakışlar); dört kendra ve odak evi, K6 sınıfıyla; yoga listesi. Lagna yoksa lagnaya bağlı bölümler boştur. Olgular on dört el fikstürü (kaynak `hand`), elle yazılmış ikinci tablolar ve değişmezlerle; gerçek motorda v1 haritasıyla çapraz denetimle doğrulanır.
- Prashna haritası danışana bağlı bir `Event` haritasıdır ve yan tabloda satırı vardır (şema sürümü 2; göç 1 değişmedi). Soru en çok 2 000 karakterdir, aranmaz, yalnızca notlarla dışa aktarılır; sayı ve odak evi dışa aktarılır ve içe aktarım satırı geri kurar. Danışansız prashnalar kaydından tanınan yerleşik danışanın altında toplanır. Harita silinince satır silinir.
- Üç komut çalışır: `create_prashna` anı ve yeri Rust'ta çözer, kaydeder ve açar; `update_prashna` soruyu ve odak evini yazar; `compute_prashna_facts` olguları verir. Girdi hataları yalnızca alan adı taşır; soru, yer ve an hiçbir yerde loglanmaz. Bütçeler ölçülür ve tutar (`compute_prashna_facts` ≤ 15 ms; `create_prashna` ≤ 100 ms; `open_client` ≤ 50 ms).
- "Yeni prashna" penceresi ⌘⇧P ve paletten açılır; "Prashna kur" haritayı Prashna düzeninde açar. Olgu paneli soruyu ve odak evini düzenletir, olguları çizer; boş, yükleniyor, hata, dolu ve "saat bilinmiyor" durumları testlidir; yeniden okuma içeriği sökmez ve odak hiçbir eylemde gövdeye düşmez; renk tek başına anlam taşımaz; hüküm ve konu çıkarımı yoktur; her kimliğin üç dilde adı vardır.
- `Chart` tipi, `CalcSettings`, `ViewState`, `AppPrefs`, öteki şema sürümleri, önceki modüllerin sonuçları, `ChartSvg`, baskı düzenleri ve görsel temeller değişmemiştir; v1 golden testleri değişmeden geçer.
- Kapılar: `jyotish-core` lib 374, `modules_golden` 17, `store` lib 112 (+1 ign.), `jyotish-desk` lib 54; Vitest 83/840; e2e 24; görsel 10; `cargo deny` temiz; öteki sayılar tabandaki gibi. Elle denetim kapısının **[otomatik]** maddeleri testlerle doğrulanmış, **[insan]** maddeleri yapılmış ya da sahibine devredilmiştir; el örnekleri bağımsız olarak yeniden türetilmiş ve `REVIEW.md`'ye yazılmıştır; klasik tablonun onayı yapılmış ya da sahibine devredilmiştir.

## Bu planın kapsamadıkları

- **Spec'in 3.8 için kapsam dışı saydıkları:** Tajika bakışları ve yogaları, 1–249 sayı sistemi ve alt lord yöntemleri, ashtamangala, arudha ve chhatra rasi prashnası, nimitta ve alamet girişi, yitik eşya yönü gibi hüküm çıkarımları, evet/hayır sonucu.
- **Sonraki alt proje (3.9):** arama. Prashna haritalarının harita aramasında ayrı bir koşulu ("yalnızca prashnalar") ve soru metninde arama bu planda yoktur (spec: FTS'e eklenmez; gerekirse sonra).
- **Klasik tablonun basılı kaynakla onayı** (D3, D8; Task 8 Step 6) ve ikinci okumalar: navamsanın başını alan sayı yöntemi, sayının yalnızca burç vermesi, İkizler'in sırtla doğan sayılması, derece temelli bakış, düğümlerin bakışı. K7'ye göre ancak ikinci seçenek uygulanınca ayar olur; bu planda ayar eklenmez ve yöntem kimliği `prashna.parashari_v1`'dir.
- Sayı prashnasında anın gerçek lagnasının ikinci bir lagna olarak gösterilmesi; sayı haritasında Sripati ya da Placidus bhavaları (D2: her zaman burç evleri).
- **Sayı prashnasında Shadbala ve Bhava bala** (D4 istisnası): hesaplanmaz; iki panel nedenini söyler. Sayıdan bir meridyen türeten ya da dig balayı dışarıda bırakan bir okuma eklenmez; istenirse ayrı bir karar ve el örnekleri gerektirir.
- Kurulmuş bir prashnanın sayısının sonradan değiştirilmesi; bir doğum ya da olay haritasının sonradan prashnaya çevrilmesi ve tersi (D12).
- Pencerede yaz saati örtüşmesi için açık seçim (D17: an reddedilir; doğum formu çözer); pencereden bulunulan yerin belirlenmesi (yer gün panchangı panelinin penceresinden belirlenir: Plan 20 D29); elle koordinat girişi.
- Kenar çubuğunda ve palette prashna haritalarının ayrı bir işaretle listelenmesi (etiket "Prashna …" ile ayırt edilir: D18); danışanlar arasında prashna taşıma; birden çok yerleşik danışan.
- Olguların baskı düzeni (K10); olguların dışa aktarılması ya da saklanması (spec §16); soru metninin geçmişi.
- `EXPORT_SCHEMA_VERSION` artışı (D14: eklenen alan isteğe bağlıdır; önceki bir sürüm alanı yok sayar ve sayı prashnasını zaman haritası olarak içe alır: sürüm notu maddesi).
- Paket parçalama (D27: 500 kB uyarısı).
- Yorum metinleri ve AI yorum paneli; Windows ve Android.
- v1'den taşınan seyrek test dalgalanması (`NotesPanel.test.tsx › New note adds an empty note…`, yük altında): bu planın dosyalarına dokunmaz; ayrı bir bakım işidir.

## Spec'e geri bildirim (spec kusurları ve boşlukları)

Plan yazılırken bulunan ve Kararlar'da çözülen maddeler. "Kusur" spec'in kendi içinde çelişen, harfiyen uygulanamayan ya da var olan kodla tutmayan yeridir; "boşluk" yazılmamış ayrıntıdır.

1. **Kusur — K1 ile §11 "Veri modeli ve IPC".** K1 "`Chart` ve `open_client` değişmez" der; §11 "`open_client` dönüşüne `prashna: Option<PrashnaMeta>` eklenir" der ve sayı prashnasının lagnası `open_client`'ın hesapladığı haritadadır. Çözüm (D16): `OpenedClient.prashna` eklenir; açılış ve ayar kaydı sayı prashnasını sayının lagnasıyla hesaplar; `Chart` tipi değişmez ve bütçe ölçülür. Spec K1'e bu istisnayı yazmalı.
2. **Kusur — K2 `ChartInput { jd_ut, lat, lon, settings, time_known }` ile §11 sayı prashnası; `compute_prashna_facts(input, number, focus_house)`.** Sayı yalnızca prashna komutuna verilirse, sayı prashnası açıkken öteki bütün modül panelleri (`ChartInput` ile hesaplayan yogalar, bakışlar, Jaimini, Shadbala, chara dasha, uyum) gerçek lagnayı kullanır ve haritayla çelişir. Çözüm (D4): `ChartInput.lagna_number: Option<u8>` ve `compute_chart_for`; `compute_prashna_facts(input, focus_house)`. Spec K2'yi ve §4'ü altıncı alanla yazmalı ve §11 imzasından `number`'ı çıkarmalı.
3. **Kusur — K12 ile §11 "Arayüz".** K12 "Hazır düzenlere modül paneli eklenmez" der; §11 "Yerleşik düzenlere 'Prashna' düzeni eklenir: D1, `prashna_facts`, `panchang`, notlar" der. Çözüm (D20): Prashna düzeni modül paneli taşıyan tek hazır düzendir; dördüncü hazır düzen ⌘4'tür ve kullanıcı düzenleri ⌘5'ten başlar. Spec K12'ye istisnayı ve kısayol kaymasını yazmalı; UI spec §3.2'nin hazır düzen listesi de güncellenmeli.
4. **Kusur — §11 `create_prashna(client_id, at: PrashnaMoment, place, number, question, focus_house) -> OpenedChart`.** `place`'in tipi yoktur (`Place` saat dilimi taşımaz; §10'da `HomePlace` olarak düzeltilmişti), `OpenedChart` diye bir tip yoktur ve soru metni çıplak bir dizgi bağımsız değişkeni olur. Çözüm (D15): `create_prashna(request: PrashnaRequest) -> OpenedClient`, `update_prashna(chart_id, edit: PrashnaEdit) -> PrashnaMeta`; `place: HomePlace`. Spec imzaları ve `PrashnaMoment { Now, Local { date, time } }` tipini böyle yazmalı.
5. **Kusur — §11 "Evler bu lagnadan rasi evidir" ile "`compute_chart_with_lagna` gerçek lagna verildiğinde `compute_chart` ile aynı haritayı döner".** Sripati ya da Placidus ayarında `compute_chart` burç evi vermez: iki cümle birlikte doğru olamaz. Çözüm (D2): verilen lagna yolunda bhavalar her zaman burç evidir (`bhavas.system` `WholeSign`); değişmez burç evi sistemi için yazılır (tropikal-rasi kipinde lagnanın rasi boylamı 1e-9° payla). Spec değişmezi bu koşulla yazmalı; "yalnızca lagna ve evler değişir" cümlesine vargaların lagnasını ve ashtakavargayı da eklemeli.
6. **Boşluk — §11 `PrashnaFacts` ve `PrashnaMeta` tipleri.** Olgular madde madde sayılmış, tip tanımlanmamış; saat bilinmeyince hangi alanların boş olduğu yazılmamış. Çözüm (D5, D12).
7. **Boşluk — §11 "Anın panchangı (v1) ve 3.5 kuruluysa yoga listesi".** Panchang `Chart`'tadır; olgularda yinelenmesi gerekip gerekmediği yazılmamış. Çözüm (D6): panel panchangı açık haritadan okur; yoga listesi olguların içindedir.
8. **Boşluk — §11 lord ilişkisinin beş türü.** Türler tek seçenek gibi sayılmış; birlikte oluşanlar (değişim ve karşılıklı bakış), tek yönlü bakışın yönü ve iki evin aynı grahaya ait olması yazılmamış. Çözüm (D7): bağ listesi, `same_graha`, yönlü iki tek yönlü bakış.
9. **Boşluk — §14 ve §11 ile v1 hataları.** §10 için yazılan "bu yollar konumu, saat dilimini ve anı kendileri denetler" kuralı §11'de yoktur; oysa `create_prashna` doğum verisi çözümünden geçer ve o hatalar tarihi, saati ve dilimin adını parametreye yazar. Çözüm (D10): `invalid_prashna_input { field }`; yer her şeyden önce denetlenir. Spec kuralı §11'e de yazmalı.
10. **Boşluk — §11 "Yeni tablo".** Şema sürümünün 2 olacağı, bunun ilk göç olduğu ve önceki sürümlerin veritabanını açmayacağı yazılmamış. Çözüm (D11). Spec ve sürüm notu bunu söylemeli.
11. **Boşluk — §11 `update_prashna`.** Dönüş tipi, sayının ve anın düzenlenip düzenlenmeyeceği, doğum formundan düzenlemenin yan kaydı koruyup korumayacağı yazılmamış. Çözüm (D12).
12. **Boşluk — §11 yerleşik "Prashna" danışanı.** Nasıl tanınacağı, yeniden adlandırma, silme ve aynı adlı gerçek danışan yazılmamış. Çözüm (D13): `app_state`'teki UUID.
13. **Boşluk — §11 dışa ve içe aktarım.** Yalnızca soru metninin koşulu yazılmış; sayı ve odak evi dosyaya girmezse sayı prashnası geri alındığında başka bir lagnayla açılır. Çözüm (D14): `prashna` alanı; `EXPORT_SCHEMA_VERSION` değişmez.
14. **Boşluk — §11 "an = şimdi (ya da girilen an)".** Girilen anın hangi dilimde olduğu ve yaz saati boşluğu ile örtüşmesi yazılmamış. Çözüm (D17): sorunun yerinin duvar saati; boşluk ve örtüşme reddedilir; "şimdi" örtüşmede anı korur.
15. **Boşluk — §11 haritanın etiketi.** Çözüm (D18): `Prashna YYYY-MM-DD SS:DD`.
16. **Boşluk — §11 pencerenin ayrıntıları.** Yer kaynağı, açık danışan varken danışansız prashna, hataların gösterimi ve baskı önizlemesindeki davranış yazılmamış; ⌘⇧P'nin "kısayol eklenmez" (K12) ve "⌘ kısayolları değişmez" (plan kuralı C16) ile ilişkisi belirtilmemiş. Çözüm (D19).
17. **Boşluk — §11 panelin düzeni ve kayıt davranışı.** Bölümler, sütunlar, sorunun ne zaman kaydedildiği ve prashna olmayan haritada panelin ne gösterdiği yazılmamış. Çözüm (D21, D22).
18. **Boşluk — K2 ve §13 "saat bilinmiyor" durumu.** Prashna her zaman kesin saatle kurulur; durumun yalnızca düzenlemeyle oluştuğu yazılmamış. Çözüm (D23).
19. **Boşluk — §7 Shadbala ile §11 sayı prashnası.** §7 dig bala için MC ister ve `compute_shadbala`'nın `None` döndüğü durumları sayar (saat bilinmiyor; doğuş ya da batış yok); §11 sayı prashnasında lagnanın sayıdan geldiğini söyler ama meridyenin ne olacağını ve Shadbala ile Bhava balanın hesaplanıp hesaplanmayacağını yazmaz. Çözüm (D4 istisnası): sayı prashnasında ikisi de hesaplanmaz (`None`); paneller kendi notunu gösterir. Spec §7'nin `None` listesine ve §11'in sayı prashnası maddesine yazılmalı.

## Self-Review

**1. Spec kapsamı.**

| Gereksinim (ileri modüller spec'i) | Görev |
|---|---|
| §11 Zaman prashnası; an şimdi ya da girilen an; yer `home_place` | Task 4, 5 (D17, D19) |
| §11 Sayı prashnası; formül; `compute_chart_with_lagna` | Task 1 (D2–D4) |
| §11 Odak evi astrologun seçimidir | Task 2, 5, 6 |
| §11 Olgular: lagna, lordlar ve ilişki, Ay, kendralar ve odak evi, panchang, yogalar | Task 2 (D5–D9), Task 6 (D6, D21) |
| §11 Saklama: yan tablo; `kind` değişmez; 2 000 karakter | Task 3 (D11, D12) |
| §11 Komutlar; `open_client` dönüşü | Task 4 (D15, D16 farklarıyla) |
| §11 Arayüz: ⌘⇧P, palet, sağdan kayan panel, "Prashna kur", yerleşik danışan | Task 3, 5 (D13, D19) |
| §11 Arayüz: `prashna_facts` paneli; odak evi ve soru | Task 6 (D21, D22) |
| §11 Arayüz: "sayı: n" rozeti; "navamsa ortası" | Task 6 |
| §11 Arayüz: Prashna hazır düzeni | Task 6 (D20) |
| §11 Test: el örnekleri | Task 1, 2 |
| §11 Test: property | Task 1; Task 4 (gerçek ephemeris) |
| §11 Test: store | Task 3, 4 |
| §11 Gizlilik | Task 3, 4; Task 8 Step 3 |
| §11, §15 Performans | Task 4 (D24) |
| K1, K2, K3 | Task 1, 2, 4 (D4, D16) |
| K4, K5, K6 | Global Constraints "Çerçeveler"; Task 1 (tropikal-rasi testleri), Task 2 |
| K7 (`method`; ayar yok) | D1; "Bu planın kapsamadıkları" |
| K8, §14 | D10; C1 |
| K9, §13 | Task 2; Task 8 Step 5, 6 |
| K10, K11, K12 | D27; D8; Task 6, 7 (D20) |
| §17 "El örneği yazanın kuralı yanlış anlaması" | bağımsız ikinci türetme (madde 5); Task 8 Step 5 |
| Ek A | D26; Global Constraints; C13 taramaları |

Boşluk: yok. Spec'in istediği ama planın başka biçimde verdiği parçalar: `compute_prashna_facts`'in `number` parametresi (D4: girdinin alanı), `create_prashna`'nın altı bağımsız değişkeni (D15: tek istek nesnesi), tek değerli lord ilişkisi (D7: liste), olguların içinde panchang (D6: açık haritadan). Spec'in istemediği ama planın eklediği parçalar: `ChartInput.lagna_number` ve `compute_chart_for`, `LordRelation.same_graha`, yerleşik danışanın `app_state` kaydı, dışa aktarımda `prashna` alanı, haritanın etiketi, `session.setPrashna`, `YogaList`'in dışa aktarılması, fikstürdeki `prashna_ids`, `REVIEW.md`'deki varyant listesi.

**2. Yer tutucu taraması.** "TODO", "TBD", "daha sonra", "uygun hata işleme" yok. Task 8'deki `<tarih>` ve `<gözden geçiren>` denetimin yapıldığı günün ve yapanın yazılacağı yerdir; `REVIEW.md`'deki "Bekliyor" satırları Step 5 ve Step 6'nın kaydıdır. Her kod adımı tam dosya ya da `0b61716` ağacına (ve önceki görevlere) karşı üretilmiş diff taşır.

**3. Tip tutarlılığı.**
- `ChartInput.lagna_number` Task 1'de Rust'ta eklenir; TS bağlaması aynı görevde üretilir; `chartInput(opened)` Task 1'de `null`, Task 4'te `opened.prashna?.number ?? null` yazar; `store::chart_input` Task 3'te yan tablodan okur; testler ve e2e taklidi aynı adı okur.
- `RisingMode` (3), `LordLink` (5), `LordFacts`, `LordRelation`, `Occupant`, `HouseOccupants`, `LagnaFacts`, `FocusFacts`, `MoonFacts`, `PrashnaFacts` Task 2'de Rust'ta tanımlanır; TS bağlaması Task 4'te üretilir; `ui_fixture.rs`, `tests/modules.rs`, panel ve e2e aynı adları okur. `RisingMode` değerleri Rust'ta, `RISING_NAMES`'te, fikstürün `prashna_ids.rising` listesinde ve üç dil dosyasının `prashna.rising` ağacında aynıdır; Vitest son üçünü karşılaştırır. `LordLink` için `prashna.link` aynı biçimde denetlenir.
- `PrashnaMeta { question, number, focus_house, created_at }` `store`'da (Task 3), `OpenRecord` ve `OpenedClient`'ta, `update_prashna` dönüşünde, `session.setPrashna`'da ve e2e taklidinde aynıdır.
- `PrashnaRequest { client_id, at, place, number, question, focus_house }`, `PrashnaMoment { Now, Local { date, time } }`, `PrashnaEdit { question, focus_house }` Rust'ta (Task 4), `toRequest`'in ürettiği nesnede (Task 5), panelin kaydında (Task 6) ve e2e beklentilerinde aynıdır.
- El fikstürü alanları (`input.{lagna_lon, number, focus_house, grahas, retrograde}`; `expected.lagna.{sign, deg_in_sign, kind, rising, lord, kendras}`; `expected.focus.{house, sign, lord, same_graha, links, focus_lord_from_lagna_lord, lagna_lord_from_focus_lord, occupants}`; `expected.moon.{sign, house, nakshatra, pada, tithi, paksha, aspected_by}`) düzenekteki `PrashnaInput`, `PrashnaExpected`, README satırı ve on dört JSON dosyasında aynıdır.
- Komut adları: Rust `create_prashna`, `update_prashna`, `compute_prashna_facts`; gövdeler `create_prashna_blocking`, `update_prashna_body`, `compute_prashna_facts_blocking`; TS `createPrashna`, `updatePrashna`, `computePrashnaFacts`; bağımsız değişken adları `request`, `chartId` ve `edit`, `input` ve `focusHouse`; e2e taklidindeki anahtarlar aynıdır.
- Fikstür anahtarları `prashna.{opened, opened_number, facts, facts_no_focus, facts_number}` ve `prashna_ids.{rising, links, max_number, max_question_chars}` `ui_fixture.rs`, `test/fixture.ts`, testler ve `e2e/ipc.ts`'te aynıdır.
- Panel türü `prashna_facts`: `PANEL_KINDS`, `MODULE_PANELS`, `PANEL_REGISTRY`, `panel.prashna_facts` ve hazır düzenin paneli aynı adı taşır. Hazır düzen kimliği `builtin:prashna`: `BUILTIN_LAYOUT_IDS.prashna`, `layout.prashna` ve e2e beklentisi aynıdır.
- Hata alanları: çekirdek `number`, `focus_house`, `lagna`; komut `place`, `at`, `number`, `focus_house`, `question`; store `prashna.question`, `prashna.number`, `prashna.focus_house`; arayüz `PRASHNA_FIELDS` (`at`, `place`, `number`, `question`, `focus_house`) ve `prashna.form.error.*`.

**4. Kural denetimi.** C1 (soru, yer ve an loglanmaz; tek yeni kod yalnızca alan adı taşır), C2′ (yeni dizgi bağımsız değişkeni yok), C3 (göç 1 değişmez; şema sürümü 2; öteki şema sürümleri değişmez), C4 (bütün değerler ve "şimdi" Rust'tan; biçim denetimi ve yazım gösterim), C5/C6 (otomatik kayıt kendi sırasıyla; ayrıştırıcılar değişmez), C7/C16 (⌘⇧P yalnızca `shortcuts.ts`'te; var olan kısayollar değişmez), C8 (pencere `Dialog`), C9 (sınıf ve durum sözle; yeni stilde hex yok), C10 (iki üretilen dosya komutla, iki görevde), C12 (yeni kod ve her yeni anahtar üç dilde; Fransızcada ’), C13 (her commit'te tarama), C14 (`panchang.rs`'e dokunulmaz), K9 (yalnızca `hand`; tablo başka yazılımla karşılaştırılmaz), K10 (baskı ve görsel temeller değişmez) görevlerin adımlarında ve Task 8 Step 3'te denetlenir. Önceki planların dersleri: yeniden okuma içeriği sökmez (`useLastData`; soru kutusu ve seçici hata durumunda da yerinde), odaktaki denetimi götüren eylem odağı düşürmez (yeniden deneme), son veriyi tutan kanca aynı kapsamda yeniden denemede hata öncesi veriyi göstermez (testli), kaydırma kapları odak alır ve adlıdır, her yeni hata yolu yalnızca sabit alan adı taşır ve yer andan önce denetlenir (Plan 20'nin iki sızıntısının karşılığı: tek eşleme fonksiyonu ve parametre sayısı testi), tercih kaydı bu planda yazılmaz (pencere `home_place`'i yalnızca okur), Fransızca metinler ’ kullanır, Vitest yolları `app/` dizinine göredir, bellekten aktarılan tablo varyant noktalarıyla sahibin onayına işaretlidir.

**5. Koşulan doğrulama.** Bu plandaki kod, deponun `0b61716` üzerindeki bir kopyasında (kısa bir yolda, Rust 1.100) yedi commit olarak uygulanmış ve her commit'te ölçülmüştür: `jyotish-core` lib 351 → 363 → 374; `modules_golden` 15 → 17; `store` lib 103 → 112 (+1 ign.); `jyotish-desk` lib 49 → 54; `modules` 1 ve `open_client` 1 (ortancalar: `compute_prashna_facts` 0,8–1,9 ms; `create_prashna` 1,6 ms; `open_client` 1,2–1,4 ms); Vitest 806 → 806 → 806 → 822 → 840 (dosya 80 → 82 → 83); e2e 22 → 24 (iki koşu). Her commit'te `cargo fmt --check` ve `clippy -D warnings` (1.100) çıktısızdır ve `pnpm build` `✓ built` verir. Son durumda `cargo test --workspace` (başarısız sonuç satırı 0), `cargo deny check` (dört `ok`), 1.93 ile `cargo check`, `pnpm build` (643,5 kB; D27), görsel 10 (temel dosyalar değişmeden) ve yayın betiği testleri 14 yeşildir; iki değişmez ve iç tutarlılık değişmezi 3 × 4000 durumla koşulmuştur. "Testlerin başarısız olduğunu doğrula" adımlarındaki çıktılar da aynı kopyada, bir önceki görevin ağacına yalnızca o adımın test değişiklikleri konarak ölçülmüştür (Task 1–6). Ayrıca bu belgedeki bloklar (27 yeni dosya ve 13 diff bloğu) belgeden okunup sırayla temiz bir `0b61716` ağacına uygulanmış ve her görevin sonunda o commit'in ağacıyla birebir aynı sonucu vermiştir (üretilen iki dosyayı Task 1 ve Task 4'ün adımlarındaki komutlar üretir). Plan yazılırken `main` bir kez ilerledi (`84a4fcb` → `0b61716`: Plan 20'nin son inceleme commit'leri); kopya yeni uca taşındı (çakışmasız) ve sonraki bütün ölçümler `0b61716` üzerinde yapıldı; ilk iki görevin çekirdek sayıları taşımadan önce de aynı farkları vermişti.

El fikstürlerinin değerleri bu plan yazılırken motordan bağımsız bir hesapla türetilmiştir: kural metninden (spec §11, K4–K6 ve v1'in dignity ile yanma kuralları) başka bir dilde, sözlükler ve ad listeleriyle ve kesirli aritmetikle yazılmış ayrı bir hesap on dört haritanın `derivation` satırlarını ve `expected` değerlerini üretmiştir (burç lordları, yücelme ve moolatrikona, dost ve düşman listeleri, bakış evleri, orblar ve doğuş biçimi ayrı sözlüklerdir). Motor on dört örnekte ilk koşuda birebir uyuşmuştur. Hiçbir beklenen değer motor çıktısına bakılarak değiştirilmemiştir. İki türetmeyi de aynı yazar yazmıştır ve **tablolar iki hesapta da aynı kaynaktan (spec'in listeleri ve yazarın belleği) gelir**: bu uyum aritmetiği ve okumayı doğrular, tabloların kendisini değil. Task 8 Step 5 (bağımsız gözden geçirme) ve Step 6 (basılı kaynakla onay) bunun için vardır ve bu belgedeki doğrulamayla yerine getirilmiş sayılmaz.

Yazım sırasında düşenler ve düzeltmeleri (hiçbirinde beklenen değer motor çıktısından kopyalanmadı): (1) İlk taslakta sayı yalnızca prashna komutuna veriliyordu; toplayıcılar okununca sayı prashnasında öteki modül panellerinin gerçek lagnayla hesaplayacağı görüldü ve sayı `ChartInput`'a alındı (D4). (2) `ChartInput` alan kazanınca `store`'un eş harita testi düştü (alan listesi beş anahtar bekliyordu) ve ilk koşuda gözden kaçtı: sonuç satırları sayılmadan "yeşil" denmişti. Test düzeltildi ve o günden sonra her adımda başarısız sonuç satırı sayılır (Global Constraints'teki sayaç). (3) "Şimdi"nin yerel ortalama saatte çözülmesi bir saniye ayrışıyordu (ofset saniyeye yuvarlanır, çözüm yuvarlamaz): örtüşme seçimi anı bulamıyor ve `at` hatası dönüyordu. Bir saniyelik pay eklendi ve test bunu sabitler (D17). (4) Entegrasyon testinin ilk taslağı kurulan prashnanın ayarını yerleşik varsayılanla karşılaştırıyordu; aynı test daha önce genel varsayılanı değiştirdiği için düştü. Beklenti store'un o anki tercihleri oldu; kod değişmedi. (5) Dördüncü hazır düzen yirmi iki var olan testin beklentisini değiştirdi (düzen sayısı, sıra, ⌘4); hepsi Task 6'nın test adımındadır ve kısayol kayması D20'ye yazıldı. (6) Panel testlerinin ilk beklentileri burçları tam adıyla, padayı "4. pada" ve yogayı "Yoga" diye yazıyordu; arayüzün var olan anahtarları kısaltma, "pada 4" ve "Nitya yoga"dır. Beklentiler düzeltildi, yeni anahtar eklenmedi. (7) Panel kapanırken gönderilen kayıt eşzamanlı değildir; test bir mikro görev bekleyecek biçimde düzeltildi. (8) İlk e2e taslağı yer listesinin ilk "option"ını tıklıyordu; pencerede `<select>` seçenekleri de aynı rolü taşır ve tıklama 30 saniye bekledi. Seçim klavyeyle yapıldı. "Panchang" paneli de "Anın panchangı" bölümüyle ad çakışması verdi; tam eşleşmeyle arandı. **Koşulmayanlar:** `pnpm tauri dev` ile gerçek uygulama ve gerçek veritabanı (Task 8 Step 4'ün **[insan]** maddeleri; özellikle gerçek veritabanının sürüm 2'ye göçü ve uygulama yeniden başlatılınca prashnanın geri gelmesi), VoiceOver, ve klasik tablonun basılı kaynakla onayı (Task 8 Step 6): bunlar yürütme sırasında yapılır ya da sahibine devredilir.
