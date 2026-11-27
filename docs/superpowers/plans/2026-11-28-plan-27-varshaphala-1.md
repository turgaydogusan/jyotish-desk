# Plan 27 — Varshaphala I (dönüş anı ve yıllık harita; muntha ve yılın lagna olguları; mudda dasa; `compute_varsha` ve `save_varsha_chart` komutları, Varshaphala paneli ve dasa ağacının mudda görünümü)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Bu plan Claude dışı bir kodlama ajanı (Cursor) tarafından, ayrı bir git çalışma ağacında yürütülür: gereken her yol, kod ve komut bu belgededir; başka bir oturumun bağlamına gönderme yoktur. **Yürüten ajan uçtan uca (`pnpm e2e`) ve görsel (`pnpm e2e:visual`) testleri koşmaz;** bunların kodu bu planda tamdır ve **[kontrolör]** diye işaretlenen adımlarda, yürütmeden sonra kontrolör tarafından koşulur.

**Goal:** Tajika yıllık haritasının ilk yarısını eklemek (yol haritası §4.14, "Plan 27"). (1) **Dönüş anı:** Güneş'in doğumdaki sidereal boylamına döndüğü an; yıl o anda başlar ve sonraki dönüşte biter. (2) **Yıllık harita:** dönüş anının, seçilen yerdeki (doğum yeri ya da bulunulan yer) ve doğum haritasının ayarlarıyla hesaplanan haritası. (3) **Muntha ve yılın lagna olguları:** doğum lagnasının burcundan tamamlanan yıl kadar burç ilerleyen nokta, lordu, iki haritadaki evi; yıllık lagnanın ve doğum lagnasının lordu. (4) **Mudda dasa:** Vimshottari paylarının bir yıla sığdırılması; ilk lord doğum nakshatrasından ve tamamlanan yıldan bulunur; dokuz dönem ve seksen bir alt dönem. Üç katman: `jyotish-core`'da saf `varsha` modülü ve ince toplayıcı; uygulamada iki komut (`compute_varsha`, `save_varsha_chart`); arayüzde paletten açılan `varsha` paneli ve dasa ağacı panelinde üçüncü sistem olarak mudda dasa. Yıllık harita var olan olay haritası yoluyla kaydedilebilir. Yorum, puan ve hüküm yoktur; yıl lordu, Tajika bakışları ve sahamlar Plan 28'dedir.

**Architecture:**
- **Saf çekirdek ve ince toplayıcı (K3).** `muntha_sign`, `year_facts` ve `mudda_dasha` ephemeris görmez; `solar_return_jd`, `varsha_span` ve `completed_years_at` yalnızca Güneş'in sidereal boylamını sorar; `compute_varsha(eph, &ChartInput, VarshaYear, place)` doğum haritasını (`compute_chart_for`), dönüş anını ve yıllık haritayı (`compute_chart`) hesaplar ve saf parçaları birleştirir. Var olan hiçbir hesabın davranışı değişmez: `chart`, `dasha`, `timeline`, `search`, `dignity` ve `zodiac` yalnızca okunur.
- **Modül `Chart`'a eklenmez (K1).** Rapor istek üzerine hesaplanır ve saklanmaz. Veritabanı şeması değişmez (Y6): yıllık harita saklanmak istenirse var olan `ChartKind::Event` kaydı ve var olan `Store::add_chart` yolu kullanılır.
- **Tek okuma, `method` kimliğiyle (Y5).** Dönüşün çerçevesi, munthanın sayımı ve mudda dasanın ilk lord kuralı ile yıl uzunluğu birden çok okumaya açıktır; her yerde tek okuma seçilir (Kararlar D3–D9), `VARSHA_METHOD = "varsha.sidereal_return_v1"` kimliğine bağlanır ve `tests/golden/modules/varsha/REVIEW.md`'de varyantlarıyla yazılır. Mudda dasanın kuralı ve payları bellekten aktarılmıştır: sahibin basılı kaynak onayına dek mudda görünümü "doğrulanmamış tablo" notunu taşır (Y4). Uygulanan tek ikinci seçenek yıllık haritanın yeridir (doğum yeri ya da bulunulan yer; D7) ve panelin ayarıdır.
- **Lagna yoksa lagnaya bağlı çıktı yoktur (K2).** Doğum saati bilinmiyorsa yıllık harita ve yılın lagna olguları (muntha dahil) Rust'ta boş döner; dönüş anı ve mudda dasa döner ve arayüzde yaklaşık diye işaretlenir.
- **Arayüz hesap yapmaz (C4).** Süren yılı Rust bulur ("şimdi" Rust'ta okunur); mudda dasanın bölümden artanı, dönem sınırları ve anların gösterileceği UTC ofseti Rust alanıdır. Arayüz kimliği adına çevirir, Jülyen gününü verilen ofsetle yazar ve seçili yaşı bir artırıp azaltır.
- **Paralel planlarla çatışmayı azaltan yerleşim.** Plan 26 (geçiş araçları) ve Plan 29 (geçiş takvimi) başka çalışma ağaçlarında aynı tabandan yürür. Bu plan yeni kodu yeni dosyalara koyar (çekirdekte `varsha/`, uygulamada `varsha_commands.rs` ve `tests/varsha.rs`, el örneklerinde `tests/varsha_golden.rs`, arayüzde kendi panel, kanca, depo, test, e2e ve görsel dosyaları) ve ortak dosyalara yalnızca küçük, ekleme türünde dokunur; bunların tam listesi "Paylaşılan dosyalar" bölümündedir. **Hiçbir ad `jyotish_core` kökünden yeniden dışa aktarılmaz** (kökteki `pub use` bloğuna dokunulmaz): geçen birleştirmede iki planın aynı adı kökten dışa aktarması birleştirmeyi kırmıştı.
- **Testler:** çekirdekte dönüş anı (±1″), mudda dasa ve muntha için birim testleri (sınırlar, ikinci yazımlar, doğrulama hataları, toplayıcının sahte ephemeris testleri); el örnekleri (beş muntha ve dokuz mudda dosyası; kapsam testi); uygulamada gerçek motorla ±1″ denetimi, olay aramasıyla çapraz denetim, kayıt, hata eşlemesi ve bütçe; arayüzde Vitest (panel, mudda ağacı, ortak yıl deposu, adlar), iki uçtan uca akış ve iki görsel temel.

**Tech Stack:** Rust 1.93+ (stable; yerelde ve CI'da 1.100), Tauri 2.12, tauri-specta 2.0.0-rc.29, serde, React 19, TypeScript 7, Vitest 5, Playwright 1.65.1. **Yeni Rust ya da npm bağımlılığı yok.**

**Spec:** `docs/superpowers/specs/2026-11-27-eksikler-yol-haritasi.md`: §1 (değişmeyen kurallar), §2 (Y1–Y8), §3 (Varshaphala satırı), §4.14, §5 "Plan 27", §7, §8, Ek A. Taşınan kurallar: `2026-11-26-alt-proje-3-ileri-moduller-design.md` §2 (K1–K12), §4 (`ChartInput`, `Positions`), §13 (ortak test stratejisi), §14 (gizlilik), §15 (bütçeler), Ek A. UI spec: `2026-11-24-ui-design.md` §3.3–§3.4 (paneller, ortak vurgu), §5 (yükleme evreleri), §8 (panel durumları).

**Ön koşul:** Taban, Plan 23–25'in birleştiği uçtur. Bu plan yazılırken `main` commit `9727c55`'teydi; kod blokları ve sayaçlar o ağaca göre yazılmış ve ölçülmüştür. `main` bu commit'ten ilerlediyse Task 1 Step 1 tabanları yeniden ölçer ve sayaç zinciri o ölçümden yürütülür (farklar aynıdır); bir `diff` bloğunun bağlamı tutmazsa önce o satırlara dokunan commit okunur. `BASE` her zaman Task 1 Step 1'de okunan commit'tir. Plan 26 ve Plan 29 bu planın ön koşulu değildir ve bu planı beklemez. Plan 28 (yıl lordu, sahamlar) bu planın üstüne yazılır.

## Kararlar

Sahibi kararları devretti; muhafazakâr seçenek seçildi ve her karar bir satır gerekçeyle yazıldı. "Yol haritası boşluğu" ya da "Yol haritası farkı" diye işaretlenenler yol haritasında düzeltilmesi gereken yerlerdir ve belgenin sonundaki "Spec'e geri bildirim" bölümünde aynı numarayla toplanmıştır. D13'teki ölçüm ve sayaç zinciri bu plan yazılırken deponun bir kopyasında, her görev ayrı commit olarak uygulanarak alınmıştır (macOS 27.2, Rust 1.100, hata ayıklama derlemesi).

- **D1 — Çekirdek modülü `crates/jyotish-core/src/varsha/`'dır: `mod.rs` (kayıt, yöntem kimliği, en büyük yaş), `solar_return.rs` (dönüş anı), `mudda.rs` (mudda dasa), `muntha.rs` (muntha ve yılın lagna olguları), `collect.rs` (rapor ve toplayıcı). Yöntem kimliği `VARSHA_METHOD = "varsha.sidereal_return_v1"`'dir. Adlar `jyotish_core` kökünden yeniden dışa aktarılmaz; tüketiciler `jyotish_core::varsha::…` yolunu kullanır.** Gerekçe: K7 her sonucun yöntem kimliği taşımasını ister. Kökte yeniden dışa aktarım `lib.rs`'in uzun `pub use` bloğuna satır ekler ve paralel planlarla hem metin hem ad çatışması üretir; `avastha` modülü de alt yoldan kullanılır.
- **D2 — Yıl "tamamlanan yıl" ile adlandırılır (`age: u8`): 0 doğum yılıdır (doğumdan ilk dönüşe), 1 ilk dönüşle başlayan yıldır. Arayüzde "N. yıl" yazılan sayı tamamlanan yılın bir fazlasıdır. En büyük değer `MAX_VARSHA_AGE = 120`'dir.** Gerekçe: klasik kurallar (muntha, mudda dasanın ilk lordu) tamamlanan yılı kullanır; 120, Vimshottari'nin ömür sınırıdır ve uygulamanın dasa ağacının bittiği yerdir.
- **D3 — Dönüş her zodyak kipinde sidereal boylama dönüştür: Güneş, haritanın seçili ayanamsasıyla sidereal sorulur ve doğum haritasının `sidereal_lon` değerine döndüğü an alınır. Tropikal-rasi kipinde de böyledir.** Gerekçe: yol haritasının önerisi (§4.14 "Dönüş anı"); tropikal dönüş her yıl yaklaşık yirmi dakika erken gelir ve yıllık lagnayı değiştirir. Uygulamanın olay araması da doğum boylamına gelişi sidereal boylamdan bulur (`TransitToNatal`). Tropikal dönüş REVIEW'de 1. noktadır; uygulanmadığı için ayar değildir (Y5).
- **D4 — Dönüş anı kendi aramasıyla bulunur: ilk tahmin doğum + yaş × 365,256363 gündür (sidereal yıl); tahminin ±3 gün çevresinde işaretli yayın sıfırdan geçtiği an ikiye bölmeyle, 0,01 saniye çözünürlükle bulunur ve Güneş'in doğum boylamına varmış olduğu ilk örnek döner. Yaş 0 doğum anının kendisidir (arama yapılmaz). Yılın sonu sonraki dönüştür (`varsha_span`); yılın uzunluğu iki dönüşün arasıdır.** Gerekçe: yol haritası "±1″ testi" ister; 0,01 saniyede Güneş 0,0004″ yürür. Olay aramasının yürüyücüsü (`timeline::walk`) bir aralığı adım adım tarar ve tek bir anı bulmak için yüz kat fazla ephemeris sorgusu yapar; `search/events.rs` ve `timeline.rs` ayrıca paralel planların da okuduğu dosyalardır ve değiştirilmez. İki arama birbirinin ikinci yazımıdır: uygulama testi, bu aramanın sonucunu olay aramasının bulduğu `TransitToNatal` anıyla bir saniye içinde karşılaştırır (D12). **Yol haritası farkı 1.**
- **D5 — Süren yıl Rust'ta bulunur: `completed_years_at`, verilen anda dönüşü geçmiş en büyük yaşı döner (doğumdan önce 0, 120. dönüşten sonra 120). Komut, yaş verilmediğinde (`age: null`) sistem saatini Rust'ta okur.** Gerekçe: arayüz saat okumaz ve tarih karşılaştırmaz (C4); geçiş panellerindeki "Şimdi" ile aynı kalıp.
- **D6 — Yıllık harita, dönüş anının `compute_chart` haritasıdır ve tam `Chart` olarak döner: doğum haritasının ayarlarıyla (ayanamsa, düğüm, ev sistemi, zodyak kipi), seçilen yerde. Saklanmaz.** Gerekçe: arayüzün harita çizimi ve adaptörü `Chart` okur; ayrı bir "hafif harita" tipi ikinci bir çizim yolu isterdi. Fikstür bu yüzden yaklaşık 270 kB büyür (iki yıllık harita).
- **D7 — Yıllık haritanın yeri doğum yeridir; astrolog panelde "Bulunulan yer"i seçebilir (tercihlerdeki `home_place`). Seçim panelin `place` ayarıdır (`"birth"` | `"home"`; tanınmayan değer doğum yeridir) ve düzenle kaydedilir. Çekirdek yeri `Option<(enlem, boylam)>` olarak alır; komut var olan `MuhurtaPlace` tipini alır (koordinat ve saat dilimi; ad taşımaz).** Gerekçe: yol haritası doğum yerini önerir ve görev taslağında komutu "yer seçimiyle" ister; iki okuma da uygulamada yaygındır (doğduğu yerden taşınmış danışan). Yer dönüş anını değiştirmez, yalnızca lagnayı ve evleri değiştirir. Özel bir üçüncü yer (atlastan) eklenmez: bulunulan yer zaten atlastan seçilir.
- **D8 — Muntha: burcu doğum lagnasının burcu + tamamlanan yıldır (on ikide bir başa döner); lordu burç lordudur; evi burç evidir ve iki haritadan ayrı ayrı sayılır (`natal_house`, `annual_house`). Burçlar rasi çerçevesindedir (K5). `YearFacts` munthayla birlikte yıllık lagnanın ve doğum lagnasının burcunu ve lordunu taşır. Munthanın yıl içindeki derece ilerleyişi ve yıl lordu seçimi yoktur.** Gerekçe: yol haritası §4.14 "Muntha: aritmetik (lagna burcu + tamamlanan yıl); risk yok". Üç lord (muntha lordu, yıl lagnası lordu, doğum lagnası lordu) Plan 28'in beş adayından tablo riski taşımayan üçüdür ve "yıl başlığı olguları" olarak şimdiden gösterilir; seçim, hadda ve Tajika bakışı Plan 28'dedir.
- **D9 — Mudda dasa. Sıra `MUDDA_ORDER`: Güneş, Ay, Mars, Rahu, Jüpiter, Satürn, Merkür, Ketu, Venüs (Vimshottari sırasının Güneş'ten başlayan yazımı). Paylar 360 üzerinden gündür: 18, 30, 21, 54, 48, 57, 51, 21, 60 (Vimshottari yılının üç katı). İlk lord: `(doğum nakshatrası + tamamlanan yıl − 2)`'nin 9'a bölümünden artan; 1 Güneş … 8 Ketu, 0 Venüs. İlk dönem dönüş anında başlar ve tamdır (kesinti yok). Yıl dönüşten dönüşedir: her dönem yılın `gün(lord) / 360` kadarıdır ve dokuz dönem yılı boşluksuz doldurur. Alt dönemler üst dönemin lordundan başlar ve aynı paylarla böler. Rapor iki seviye taşır (9 + 81 dönem, ön-sırada; var olan `DashaPeriod` tipiyle) ve doğum nakshatrasını, yaşı, artanı ve ilk lordu ayrıca yazar.** Gerekçe: yol haritası "mudda dasa Vimshottari oranlarının bir yıla sıkıştırılmasıdır" der ve iki noktayı (ilk lord kuralı, yıl uzunluğu) basılı kaynağa bırakır; ikisi için de tek okuma seçildi ve REVIEW'de 6–11. noktalar olarak yazıldı. Dönüşten dönüşe yıl seçildi, çünkü 360 takvim günü yılın son beş gününü dönemsiz bırakır. Artan Rust alanıdır, çünkü arayüz bölme yapmaz (C4) ve sahibin denetimi ara değeri ister. `dasha.rs`'in `subdivide` fonksiyonu özeldir ve dosya değiştirilmez: on beş satırlık bölme `mudda.rs`'te kendi paylarıyla yeniden yazılır.
- **D10 — Rapor her zaman döner (`VarshaReport`, `Option` değil) ve `time_known` taşır. Doğum saati bilinmiyorsa (`input.time_known == false`) `chart` ve `facts` `None`'dır; `age`, `start_jd_ut`, `end_jd_ut` ve `mudda` döner. Sayı prashnasında doğum lagnası sayıdan gelir (`compute_chart_for`) ve muntha o lagnadan sayılır; yıllık harita dönüş anının kendi lagnasıyla hesaplanır (sayı yıllık haritaya taşınmaz). Doğum haritasının ya da yıllık haritanın `LowPrecision` uyarısı rapora taşınır.** Gerekçe: K2 (arayüzün gizlemesine güvenilmez). Saat bilinmiyorsa Güneş'in doğum boylamı yarım dereceye dek, dönüş anı yarım güne dek belirsizdir: yıllık lagna anlamsızdır ve verilmez; mudda dasanın tarihleri yaklaşık olarak yine işe yarar.
- **D11 — Yeni hata türü ve yeni hata kodu yoktur. Çekirdek var olan `CoreError` kollarını kullanır: sınır dışı yaş `InvalidDashaInput("age")`, hesap aralığının dışına düşen an `InvalidJulianDay`, geçersiz yer `InvalidLocation`, Güneş'in aranan aralıkta dönmemesi (yalnızca bozuk ya da sahte ephemeris) `PanchangSearch("solar_return")`, mudda dasanın girdileri `InvalidDashaInput("nakshatra" | "range")`. Uygulama katmanı kendi durumları için var olan `invalid_input` kodunu alan adıyla üretir (`varsha.place`, `varsha.moment`, `birth.time`, `home_place`).** Gerekçe: `error.rs`, `commands.rs` ve `i18n.test.ts`'in hata listesi ortak dosyalardır ve dokunulmaz; arayüz yaşı 0–120 ile sınırlar, bu hatalar olağan kullanımda görülmez.
- **D12 — El örnekleri `tests/golden/modules/varsha/muntha/` (beş dosya) ve `tests/golden/modules/varsha/mudda/` (dokuz dosya) altındadır ve ayrı bir test dosyasıyla yüklenir: `crates/jyotish-core/tests/varsha_golden.rs`. `modules_golden.rs`'e dokunulmaz. Dönüş anının el örneği yoktur (tablo değildir): gerçek motorla ±1″ denetimi ve olay aramasıyla çapraz denetim `app/src-tauri/tests/varsha.rs`'tedir. İkinci yazımlar: mudda dasada "doğum nakshatrasının Vimshottari lordu her yıl sırada bir adım ilerler" yürüyüşü (27 nakshatra × 121 yaş) ve payların Vimshottari yılından türetilmesi; munthada burç burç yürüyen sayaç (12 lagna × 121 yaş); örneklerde örneğin kendi aritmetiğinin motordan bağımsız sağlaması.** Gerekçe: yol haritasının (4) ve (5) görevleri; §13 ve K9. Beklenen değerler bu plan yazılırken motorun kaynağından ayrı yazılmış bir türetmeyle kural metninden üretilmiş ve motorla karşılaştırılmıştır (başka yazılımın çıktısı yoktur).
- **D13 — İki komut, yeni bir dosyada: `app/src-tauri/src/varsha_commands.rs`. `compute_varsha(input: ChartInput, age: Option<u8>, place: MuhurtaPlace) -> VarshaView { report, offset_secs }`: `offset_secs`, yerin saat diliminin yıl başındaki UTC ofsetidir. `save_varsha_chart(chart_id: u32, age: u8, home: bool) -> ChartRecord`. Komut listesine `commands::dasha_children`'dan hemen sonra eklenirler. Gerçek motorla entegrasyon testi de ayrı dosyadır: `app/src-tauri/tests/varsha.rs`. Bütçe: doğum haritası, iki dönüş araması ve yıllık harita dahil ortanca ≤ 20 ms (ölçüm: 2,0 ms) ve testte `assert!` ile doğrulanır.** Gerekçe: `module_commands.rs`, `tests/modules.rs` ve komut listesinin sonu paralel planların doğal ekleme yeridir; ayrı dosya ve ayrı çapa satırı çatışmayı önler. Her entegrasyon testi dosyası ayrı süreçtir, dolayısıyla "süreçte tek motor" kuralı bozulmaz.
- **D14 — "Olay haritası olarak kaydet" var olan yolu kullanır: `Store::add_chart`, `ChartKind::Event`. Komut yalnızca kaynak haritanın kimliğini, yaşı ve "bulunulan yer" seçimini alır; kaynak haritanın doğum verisini, ayarlarını ve (istenirse) bulunulan yeri veritabanından okur, dönüş anını yeniden hesaplar, yerin duvar saatine çevirir (saniyeye yuvarlanır; yaz saati örtüşmesinde dönüş anını veren taraf seçilir) ve haritayı kaynak haritanın danışanına `Varshaphala <tarih>` etiketiyle ekler. Kaynak haritanın hesap ayarları yeni haritaya kopyalanır. Harita açılmaz: dönen kayıt panelde bildirilir ve harita listeleri yenilenir. Doğum saati bilinmeyen haritada ve bulunulan yer seçilmemişken `invalid_input` döner; panel bu durumlarda düğmeyi göstermez.** Gerekçe: yol haritasının (7) görevi "var olan akış" der ve Y6 yeni şemayı yasaklar. Arayüzden an ya da yer alınmaz (C2′, C4). Ayar kopyası gerekir, çünkü `add_chart` yeni haritaya genel varsayılanı yazar ve yıllık harita kaynak haritanın ayarıyla hesaplanmıştır. Harita açılmaz, çünkü açmak astrologu baktığı doğum haritasından ve yıl seçiminden koparırdı; kayıt kenar çubuğundaki harita listesinden açılır.
- **D15 — Panel `varsha` türüdür (`MODULE_PANELS`: 7 × 24; paletten eklenir ve çıkarılır, K12) ve hazır düzenlere eklenmez. Sırayla çizer: araç çubuğu (yıl seçici, yer), yılın olguları tablosu (yıl, dönüş anı, yıl sonu, harita yeri, vara, yıl lagnası, muntha, doğum lagnası, mudda dasanın ilk lordu), yıllık haritanın D1 çizimi (biçim seçimi panelin `style` ayarıdır; muntha burcu "Mu" ile işaretlenir), çizimin tablo karşılığı (lagna ve dokuz graha: burç, derece, nakshatra, ev), "Olay haritası olarak kaydet" düğmesi, okuma notu ve yöntem kimliği. Katalogda `avastha`'dan hemen sonra durur.** Gerekçe: yol haritasının (6) görevi "yıl seçici, harita, muntha" der. Tablo karşılığı erişilebilirlik içindir ve astrologun dereceleri okumasını sağlar. Katalog sırasındaki yer, paralel planların listenin sonuna yapacağı eklemeyle aynı satırlara düşmemek içindir.
- **D16 — Seçili yıl bellekte, açık haritaya bağlı ortak bir depodadır: `app/ui/varsha/yearStore.ts` (`useSyncExternalStore`; sağlayıcı bileşeni yoktur). `null` "bu yıl"dır. Varshaphala paneli ve dasa ağacının mudda görünümü aynı depoyu okur: biri yılı değiştirince öteki de değişir. Başka harita açılınca ve uygulama yeniden başlayınca seçim "bu yıl"a döner.** Gerekçe: iki panel aynı yılı göstermelidir; seçimi panel ayarına yazmak ikisini ayırırdı. Y6 yeni kalıcı veri istemez. Sağlayıcısız depo `App.tsx`'e ve test sağlayıcılarına (`test/render.tsx`) dokunmaz.
- **D17 — Mudda görünümü yalnızca dasa ağacı panelindedir: `system` ayarının üçüncü değeri `"mudda"`dır. Ağaç iki seviyedir (dönem ve alt dönem), tek açık yolludur ve açık dönem panelin kendi durumudur (kaydedilmez). Kendi yıl seçicisini taşır (Varshaphala paneli açık olmasa da yıl seçilebilsin). Dönemler yere bağlı olmadığı için istek doğum yeriyle yapılır. Dasa şeridi paneli mudda dasayı sunmaz. Yeni sistem `app/ui/dasha/muddaSystem.tsx`'te tanımlanır; `dasha/system.tsx` değişmez.** Gerekçe: yol haritası "var olan dasa panellerinde mudda görünümü" der; şerit transit anının dönem yolunu Rust'tan alır ve mudda için ayrı bir "andaki yol" komutu isterdi (sonraki işe bırakıldı). `system.tsx`'i değiştirmemek şeridin seçicisini iki sistemde tutar. **Yol haritası farkı 2.**
- **D18 — Anlar tek bir sabit ofsetle yazılır ve ofset yanında söylenir ("UTC+05:30"): yıllık haritanın yerinin (mudda görünümünde doğum yerinin) saat diliminin yıl başındaki UTC ofseti. Tarihler dakikaya dek yazılır (`formatJdMinute`). Yıl içinde yaz saati değişse de ofset değişmez.** Gerekçe: dasa panelleri de doğumun sabit ofsetini kullanır; ama doğum ofseti onlarca yıl sonraki dönüş anının duvar saatinden bir saat ayrışabilir (yaz saati, dilim değişikliği), astrolog dönüş saatini duvar saatiyle sağlar. Ofsetin yazılması belirsizliği kaldırır. Dönem başına ofset eklenmez (doksan dönem, yılda en çok iki değişim).
- **D19 — "Doğrulanmamış tablo: mudda dasanın ilk lord kuralı ve dönem payları basılı kaynakla karşılaştırılmayı bekliyor." notu (`varsha.mudda.unverified`) mudda görünümünde her zaman durur. Varshaphala paneli notu taşımaz; okuma seçimlerini bir satırla söyler (`varsha.reading`) ve yöntem kimliğini yazar. Not, sahibi REVIEW'deki onay tablosunu doldurduktan sonra ayrı bir commit'le çıkarılır.** Gerekçe: Y4 bellekten aktarılan tablo içindir; bu planda tablo yalnızca mudda dasadadır. Dönüş, yer ve muntha tablo değil okuma seçimidir (yol haritası "risk yok" der) ve REVIEW'de 1–5. noktalardır.
- **D20 — Klasik terimler Sanskritçe transliterasyondur ve üç dilde aynıdır (K11): "Varshaphala", "Muntha", "Vara", çizimdeki "Mu" işareti. Mudda dasanın adı her dilin var olan yazımını izler: Türkçe "Mudda dasa", İngilizce ve Fransızca "Mudda dasha". Nakshatra ve vara adları var olan sabit tablolardan okunur (`panels/labels.ts`; dosya değişmez). Adlandırma: bu planın getirdiği adların hiçbiri yasak diziyi içermez; "Tajika" sözcüğüne `-lar` eki bitişik yazılmaz; Türkçe metinlerde bölümden artan için "artan" yazılır.** Gerekçe: Y8 ve Ek A; yasak harf dizisi düzeyindedir.
- **D21 — Arayüz fikstürü bir alan kazanır: `varsha` = `{ current, next, time_unknown, mudda_order, max_age }`. `current` yaş verilmeden (fikstürün sabit saatinde süren yıl: 26), `next` 27 yaşıyla, `time_unknown` `time_known: false` ile `compute_varsha` dönüşüdür; `mudda_order` ve `max_age` Rust sabitleridir. E2e taklidi raporu `time_known`'a ve yaşa göre seçer.** Gerekçe: arayüz hesap yapmaz; testler gerçek motorun ürettiği veriyi çizer. Fikstür bloğu `ui_fixture.rs`'te kendi kapsamındadır (`use` satırları bloğun içinde): ortak `use` listesi yeniden sarılmaz.
- **D22 — Uçtan uca akışlar ve görsel testler yeni dosyalardadır: `app/e2e/varsha.desk.spec.ts` ve `app/e2e/varsha.visual.spec.ts`. Playwright'ın "e2e" projesi adı `desk.spec.ts` ile, "visual" projesi adı `visual.spec.ts` ile biten her dosyayı koşar; yapılandırma, `desk.spec.ts` ve `visual.spec.ts` değişmez. İki yeni görsel temel (yıllık harita, açık ve koyu tema) kontrolör tarafından yerelde üretilir.** Gerekçe: iki ortak test dosyası paralel planların sonuna test ekleyeceği yerdir. Temel PNG'ler yereldir (C11) ve bir plan belgesinde taşınamaz; yürüten ajan görsel testleri koşmaz.
- **D23 — Görev sırası yol haritasının taslağından üç yerde ayrılır: mudda dasa çekirdeği (Task 2) toplayıcıdan önce gelir, çünkü rapor onu taşır; panelin i18n metinleri, "saat bilinmiyor" durumu ve kaydetme düğmesi panelle birlikte Task 6'dadır (panel metinsiz ve durumsuz commit edilemez); Task 7 dasa ağacının mudda görünümü ile uçtan uca ve görsel testlerdir. REVIEW.md Task 8'de, kapılarla birlikte yazılır.** Gerekçe: her görev yeşil kapılarla tek commit'tir. **Yol haritası farkı 3.**
- **D24 — Yeni baskı düzeni yoktur (K10). `pnpm build` Vite'ın 500 kB parça uyarısını basmayı sürdürür ve kapılar `✓ built` satırını arar.** Gerekçe: on beş görsel temel Varshaphala panelini taşımaz ve değişmez; iki yeni temel yalnızca yeni paneldir.

## Global Constraints

- Lisans: AGPL-3.0-or-later. Yeni bağımlılık yok: `Cargo.toml`, `Cargo.lock`, `deny.toml`, `app/package.json`, `app/pnpm-lock.yaml` ve `app/vite.config.ts` değişmez; `cargo deny check` Task 8'de `advisories ok, bans ok, licenses ok, sources ok` verir.
- **Yasaklı ad kuralı (v1 spec §10 "İsim kuralları", ilk madde; ileri modüller spec'i Ek A; yol haritası Y8):** o maddedeki dört ad ve türevleri hiçbir isimde, yorumda, metinde, dosya adında ya da commit mesajında kullanılmaz. Kural harf dizisi düzeyindedir: Türkçe sözcüklerin içinde de geçemez (D20). Her commit'ten önce eklenen satırlar şu komutla taranır ve çıktı boş olmalıdır: `git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'`.
- **Clean-room ve yorum metni yok:** kurallar klasik tanımdan özgün olarak uygulanır; başka ürünün adı, düzeni, metni ya da çıktısı kullanılmaz; kaynak gösterimi geneldir ve alıntı yapılmaz. Modül olgu döner: **"iyi/kötü/güçlü/zayıf" hükmü, puan, renkli derecelendirme ve munthanın ya da bir dönemin ne anlama geldiğini anlatan metin eklenmez.**
- **Doğrulanmamış tablo (Y4):** mudda dasanın ilk lord kuralı ve payları bellekten aktarılmıştır. Yürüten ajan bunları başka bir yazılımın çıktısıyla ya da çevrimiçi bir kaynakla **karşılaştırmaz** ve bu planın okumasını "düzeltmez"; şüphe duyduğu yeri ilerleme notlarına yazar. Onay sahibinindir (Task 8).
- **Çerçeveler (K4, K5; D3, D8):** dönüş her kipte sidereal boylamdandır; evler lagnadan sayılan burç evleridir; burçlar rasi çerçevesindedir; nakshatra sidereal boylamdandır.
- **Doğum saati bilinmiyorsa (K2, D10):** `time_known` yanlışsa yıllık harita ve `facts` Rust'ta boş döner; arayüzün gizlemesine güvenilmez.
- **Gizlilik (ileri modüller spec'i §14):** (1) Girdi (an, konum, ayar), yer ve rapor hiçbir yerde loglanmaz. (2) Yeni hata kodu yoktur; bu planın ürettiği hatalar yalnızca alan adı taşır. (3) Rapor saklanmaz ve dışa aktarıma girmez; kaydedilen olay haritası sıradan bir harita kaydıdır. (4) Hiçbir modül ağ isteği yapmaz. (5) Testler ve fikstürler yalnızca deneme adları ve sentetik konumlar kullanır ("Örnek Danışan"); gerçek kişi verisi yoktur.
- **Bütçe testte doğrulanır** (D13): `assert!` ile; `eprintln!` yalnızca ölçümü görünür kılar. Bütçe testleri makine yükü altında düşebilir: paralel çalışma ağaçları aynı makinede derleme yaparken var olan `tests/modules.rs`'in bütçe iddiaları da düşer (bu plan yazılırken yük ortalaması 100'ün üstündeyken bir kez oldu). Bir bütçe iddiası düşerse önce `uptime` ile yüke bakılır ve o test makine sakinken tek başına yeniden koşulur (`cargo test -p jyotish-desk --test <dosya>`); kod değiştirilmez, bütçe gevşetilmez.
- **El örnekleri (§13, K9):** beklenen değer kuraldan türetilir ve `derivation` alanında ya da test yorumunda yazılıdır. Bir el örneği motorla uyuşmazsa önce kural yeniden okunur; **beklenen değer motor çıktısına bakarak düzeltilmez.** (Bu plan yazılırken bir kez oldu: `tests/varsha.rs`'te kaydedilen haritanın tarihi önce "2026-01-02" diye yazılmıştı; takvim aritmetiği yeniden yapıldı (26 sidereal yıl 9496,665 gün; 2000-01-01'den 2026-01-01'e 9497 gün) ve beklenti türetmeden "2026-01-01" olarak düzeltildi.)
- Kapılar: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` (1.100), `cargo test --workspace`, `pnpm --dir app test`, `pnpm --dir app build` her görev sonunda yeşil; `cargo deny check` Task 8'de. `pnpm --dir app e2e` ve `pnpm --dir app e2e:visual` **yalnızca kontrolör tarafından**, Task 7 ve Task 8'in **[kontrolör]** adımlarında koşulur. `tsc` `strict`; `pnpm build` e2e dosyalarının tip denetimini de yapar (`tsc --noEmit -p e2e`), yani yürüten ajan e2e kodunun derlendiğini görür. `pnpm build` çıktısında aranan satır `✓ built`'tir (D24). Tek bir Vitest dosyası `pnpm --dir app exec vitest run ui/…` ile koşulur: yol `app/` dizinine göredir. `cargo test --workspace` çıktısı uzun olduğu için başarısızlık şu sayaçla denetlenir ve `0` yazmalıdır: `cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."`.
- **Toolchain:** `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"` (oturum başına bir kez; her yeni kabukta yeniden). Node 24+, pnpm 11.8.0. Depo ya da çalışma ağacı kısa bir yolda durmalıdır: Swiss Ephemeris 242 bayttan uzun ephemeris yolunu reddeder.
- **Çalışma ağacı.** Yeni bir git çalışma ağacında `app/node_modules` ve `target/` yoktur: Task 1 Step 1 bağımlılıkları kilit dosyasından kurar (`pnpm --dir app install --frozen-lockfile`; kilit dosyası değişmez) ve ilk `cargo test` her şeyi derler (birkaç dakika). Bütün komutlar çalışma ağacının kökünden verilir.
- **Üretilen dosyalar (C10) elle düzenlenmez** ve yalnızca şu iki komutla güncellenir: `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings` (`app/ui/bindings.ts`) ve `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture` (`app/ui/test/fixtures/delhi.json`). İkisi de Task 5 Step 4'te koşulur.
- **Gerçek uygulama çalıştırılmaz.** Yürüten ajan `pnpm tauri dev` ya da `pnpm --dir app tauri dev` **koşmaz**, üretilen `.app`'i açmaz ve `~/Library/Application Support/io.github.turgaydogusan.jyotishdesk` dizinine dokunmaz. Kapı testlerdir (Y3); gerçek uygulamaya bakan maddeler Task 8'de **[insan]** diye işaretlidir ve sahibine devredilir. Başkasının çalışan `tauri dev`'i (port 1420) durdurulmaz; `pkill -f` kullanılmaz.
- **[kontrolör] E2e ve özel port.** E2e, Vite sunucusunu 1421'de açar ve açık bir sunucu varsa onu kullanır. Plan 26 ve 29 aynı makinede koşuyorsa 1421 başka bir çalışma ağacının sunucusu olabilir ve testler yanlış arayüzü görür. Kontrolör e2e ve görsel testleri **özel bir portta** koşar: `app/` altına, commit edilmeyen geçici bir yapılandırma yazar (aşağıda), testleri `-c` ile onunla koşar ve işi bitince siler. Dosya `git add` edilmez.

  ````ts
  // app/playwright.p27.config.ts (geçici; commit edilmez)
  import base from "./playwright.config";

  const PORT = 14527;
  export default {
    ...base,
    use: { ...base.use, baseURL: `http://localhost:${PORT}` },
    webServer: { command: `pnpm exec vite --port ${PORT} --strictPort`, url: `http://localhost:${PORT}`, reuseExistingServer: false },
  };
  ````
- Yalnızca macOS hedeflenir. Yorum metni ve AI paneli bu planın dışındadır (Y7).
- **Tabanlar (`9727c55`):** `jyotish-core` lib 466, `avastha_golden` 5, golden 8, `modules_golden` 17; `atlas` `build` 6, `moment_fixed_lmt` 11, `moment_iana` 13, `search` 8, `shipped` 3; `ephemeris` lib 1, `engine` 7, `env_override` 1, `ffi_smoke` 1, `golden` 1, `houses` 7, `rise_set` 5, `sanity` 15; `store` lib 122 (+1 `#[ignore]`); `jyotish-desk` lib 58, `avastha` 1, `bhava` 1, `bindings` 1, `compute_chart` 1, `modules` 1, `natal_extras` 1, `open_client` 1, `ui_fixture` 2; Vitest `Test Files  97 passed (97)`, `Tests  973 passed (973)`; e2e 33; görsel 15 (yerel, darwin); yayın betiği testleri 14.
- **Sayaç zinciri** (her satır görev bitince; boş hücre "önceki satırla aynı"). Bu plan yazılırken `9727c55`'in bir kopyasında her görev ayrı commit olarak uygulanıp ölçülmüştür.

  | Görev | `jyotish-core` lib | `varsha_golden` | `jyotish-desk` `varsha` | Vitest dosya / test | e2e **[kontrolör]** | görsel **[kontrolör]** |
  |---|---|---|---|---|---|---|
  | Taban | 466 | yok | yok | 97 / 973 | 33 | 15 |
  | Task 1 | 471 | | | | | |
  | Task 2 | 478 | | | | | |
  | Task 3 | 487 | | | | | |
  | Task 4 | | 3 | | | | |
  | Task 5 | | | 1 | | | |
  | Task 6 | | | | 98 / 990 | | |
  | Task 7 | | | | 99 / 998 | 35 | 17 |
  | Task 8 | | | | | | |

  Değişmeyenler: `jyotish-core` `avastha_golden` 5, golden 8, `modules_golden` 17; `atlas`, `ephemeris` ve `store` sayaçları; `jyotish-desk` lib 58, `avastha` 1, `bhava` 1, `bindings` 1, `compute_chart` 1, `modules` 1, `natal_extras` 1, `open_client` 1, `ui_fixture` 2; yayın betiği testleri 14. `jyotish-core` lib Task 1'de 5 test (`varsha/solar_return.rs`), Task 2'de 7 test (`varsha/mudda.rs`), Task 3'te 9 test (`varsha/muntha.rs` 3, `varsha/collect.rs` 6) kazanır. Vitest Task 6'da bir dosya ve 17 test (`panels/VarshaPanel.test.tsx`; `api.test.ts`'in tablosunda iki satır), Task 7'de bir dosya ve 8 test (`dasha/MuddaDashaTree.test.tsx`) kazanır. E2e Task 7'de iki akış (`varsha.desk.spec.ts`), görsel iki test (`varsha.visual.spec.ts`) kazanır. Paralel bir plan önce birleştiyse tabanlar onun eklediği kadar büyüktür; farklar aynıdır.
- **Taşınan kurallar (Plan 09 C1–C14, Plan 10 C2′, C15–C17), her görevin gereksinimidir; bu planda değişen okumalar açıkça yazılıdır:**
  - **C1** Gizlilik: yukarıdaki "Gizlilik" maddesi; `no-console.test.ts` durur ve izin listesi büyümez. Yeni `CommandError` kodu yoktur (D11).
  - **C2′** Hiçbir IPC komutu arayüzden dosya yolu ya da adres almaz. `save_varsha_chart` yalnızca bir kimlik, bir sayı ve bir mantıksal değer alır; `compute_varsha` gün panchangının aldığı `MuhurtaPlace` tipini alır (koordinat ve saat dilimi; yer adı gitmez). `api.test.ts`'in sarmalayıcı tablosu iki satır kazanır.
  - **C3** Var olan göçler ve şema sürümleri değişmez; `crates/store` bu planda hiç değişmez (Y6). `CalcSettings`, `ViewState` ve `AppPrefs` tipleri değişmez.
  - **C4** Arayüz astrolojik hesap yapmaz: dönüş anı, süren yıl, muntha, lordlar, mudda dasanın artanı ve dönem sınırları, anların ofseti Rust'tan gelir. Kimliği adına çevirmek, Jülyen gününü verilen ofsetle yazmak, seçili yaşı bir artırıp azaltmak, tamamlanan yıla bir ekleyip "N. yıl" yazmak ve nakshatra numarasından adını okumak gösterimdir.
  - **C5/C17** Otomatik kayıtlar değişmez; panelin `place` ve `style` ayarı ile dasa ağacının `system` ayarı düzenle birlikte var olan yoldan kaydedilir. **C6** Düzen ayrıştırıcıları (`parseLayoutState`, Rust `check_panel_layout`) değişmez; bir panel türü kataloğa eklenir.
  - **C7, C16** Kısayol eklenmez; `shortcuts.ts` değişmez. **C8** Yeni pencere yoktur (bulunulan yer penceresi var olan `HomePlaceDialog`'dur).
  - **C9** Renk yalnızca token'lardan gelir (hex yok) ve tek başına anlam taşımaz: yaklaşık değer sözle, retro "R" ve sözcükle, kapalı düğme `aria-disabled` ile yazılır.
  - **C10** Üretilen dosyalar elle düzenlenmez (yukarıda). **C11** Görsel PNG temelleri yereldir; var olan on beş temelin değişmesi beklenmez, iki yeni temel kontrolörün işidir (D22).
  - **C12** Her yeni i18n anahtarı üç dilde vardır (`i18n.test.ts` anahtar kümelerini ve yer tutucuları karşılaştırır). Fransızca metinler tipografik kesme işareti (’) kullanır; panel testi `varsha` bloğunda düz kesme işaretinin yokluğunu sabitler.
  - **C13** Yasaklı ad taraması her commit'te; depo dışındaki ilerleme notları ve `.superpowers/` hiçbir zaman `git add` edilmez.
  - **C14** Plan 07'nin gün doğumu toleransı ve `compute_panchang` değişmez (`panchang.rs`'e dokunulmaz). **C15** `alert`, `confirm`, `prompt`, `window.print()` ve `window.open()` çağrılmaz.
- **Erişilebilirlik:** WCAG AA. Tablolar `caption`, `scope="col"` ve `scope="row"` taşır; kaydırma kabı klavye odağı alır ve tablosunun adını taşır; yıl seçicinin alanı görünür etiketlidir; kapalı düğme yerel `disabled` değildir (odağı tutar, `aria-disabled` taşır); kayıt bildirimi `role="status"`, hata `role="alert"`'tir ve "Yeniden dene"den sonra odak gövdeye düşmez; çizimdeki muntha işareti ayrıca sözcükle okunur; mudda ağacı WAI-ARIA ağaç desenidir (var olan `PeriodTree`); her işlem klavyeyle yapılır.
- Commit mesajları İngilizcedir ve tam olarak şu satırla biter: `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` (komutlarda ikinci `-m` olarak verilir; yürüten araç kendi imza satırını eklemez). Her görev tek commit'tir (gerekirse inceleme düzeltmesi ayrı commit). `git push`, `git tag`, yıkıcı git komutu ve `.superpowers/` commit'i yoktur.
- Kod blokları `9727c55` ağacına göre yazılmış ve bu plan yazılırken deponun bir kopyasında koşulmuştur (Self-Review madde 5). Yeni dosyalar tam içerikle verilir ("**Yeni dosya:**"). Değişen dosyalar `diff` bloğuyla verilir: blok bir dosyaya konup çalışma ağacının kökünde `git apply <dosya>` ile ya da elle uygulanır; bağlam tutmazsa önceki görevin adımı denetlenir, elle uydurma yapılmaz. Yeni bir Rust dosyasının testleri ve uygulaması ayrı adımlarda verilir: önce dosya yalnızca `#[cfg(test)] mod tests` bloğuyla oluşturulur ("**Yeni dosya (şimdilik yalnızca test modülü):**"), sonra uygulama **o bloğun üstüne**, arada bir boş satırla eklenir ("**Dosyanın başına eklenir:**"); `cargo fmt --all --check` birleştirmeyi doğrular. Kod blokları dört ters tırnakla çevrilidir; çevreleyen satırlar dosyaya yazılmaz; her dosya tek bir satır sonuyla biter.

## Paylaşılan dosyalar

Plan 26 (geçiş araçları: nakshatra lordları, Sankranti listesi, efemeris tablosu, adımlı geçiş) ve Plan 29 (geçiş takvimi) başka çalışma ağaçlarında aynı tabandan yürür. Aşağıdaki liste bu planın dokunduğu ve öteki planların da dokunması beklenen **bütün** dosyalardır; her satır değişikliğin boyutunu, yerini ve birleştirme çatışmasında ne yapılacağını söyler. Bu listenin dışındaki her dosya ya yenidir ya da yalnızca bu plana aittir.

| Dosya | Görev | Değişiklik | Çatışmada |
|---|---|---|---|
| `crates/jyotish-core/src/lib.rs` | 1 | Tek satır: `pub mod varsha;` (`varga`'dan sonra, abecesel yerinde). `pub use` bloğuna ve dosyanın başındaki açıklamaya dokunulmaz. | İki tarafın satırları korunur (abecesel sırayla; `rustfmt` sırayı denetlemez, modül satırları elle sıralıdır). |
| `app/src-tauri/src/lib.rs` | 5 | Üç satır: `pub mod varsha_commands;` (`update_commands`'tan sonra, listenin sonunda) ve komut listesinde `commands::dasha_children,`'dan sonra `varsha_commands::compute_varsha,` ile `varsha_commands::save_varsha_chart,`. | İki tarafın satırları korunur; `pub mod` satırları abecesel sırayla. Sonra bağlamalar yeniden üretilir. |
| `app/src-tauri/tests/ui_fixture.rs` | 5 | Avastha hesaplarından hemen sonra kırk satırlık, kendi `use` satırlarını taşıyan blok (`let varsha = { … };`); `json!` içinde `"avastha_ids"`'ten sonra tek anahtar (`"varsha": varsha,`). Ortak `use` listesi değişmez. | İki tarafın blokları ve anahtarları korunur. Sonra fikstür yeniden üretilir. |
| `app/ui/bindings.ts` | 5 | Üretilir (89 satır ekleme). | Elle çözülmez: bir taraf alınır, sonra `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings`. |
| `app/ui/test/fixtures/delhi.json` | 5 | Üretilir (tek satırlık dosya; yaklaşık 270 kB büyür). | Elle çözülmez: bir taraf alınır, sonra `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture`. |
| `app/ui/test/fixture.ts` | 6 | Tip içe aktarımına bir ad (`VarshaView`, abecesel yerinde); `UiFixture`'a `avastha_ids`'ten sonra bir alan. | İki tarafın adları ve alanları korunur; içe aktarım listesi abecesel sırayla birleştirilir. |
| `app/ui/api.ts` | 6 | `computeAvasthas`'tan sonra iki sarmalayıcı (beş satır). | İki tarafın sarmalayıcıları korunur. |
| `app/ui/api.test.ts` | 6 | Dört satır: taklit listesinde ve sarmalayıcı tablosunda `computeVarsha` ve `saveVarshaChart`. | İki tarafın satırları korunur. |
| `app/ui/layout/model.ts` | 6 | `PANEL_KINDS`'e `"avastha"`'dan sonra `"varsha"`; `MODULE_PANELS`'e `avastha`'dan sonra iki satır. | İki tarafın türleri korunur. |
| `app/ui/layout/model.test.ts` | 6 | Katalog listesine bir satır; **tek satırlık** `MODULE_PANELS` beklentisine `"varsha"` (`"avastha"`'dan sonra). | Tek satırlık beklenti öteki planlarla kesin çatışır: sonuç, `model.ts`'teki `MODULE_PANELS` sırasının aynısıdır (iki tarafın türleri birlikte). |
| `app/ui/panels/registry.ts` | 6, 7 | Task 6: bir içe aktarım (`UnavailablePanel`'den sonra) ve `avastha`'dan sonra iki satır. Task 7: `dasha_tree` kaydının `nameDetail` satırı ve o satırın içe aktarımı değişir (`dashaSystem` → `treeSystem`, `treeSystemName`). | İki tarafın satırları korunur. `dasha_tree` satırına başka bir plan dokunduysa sonuç bu planın satırıdır (mudda dasanın adı ancak onunla yazılır). |
| `app/ui/i18n/tr.ts`, `en.ts`, `fr.ts` | 6 | İki ekleme: `panel` bloğunda `avastha`'dan sonra `varsha`; `shortcuts` bloğundan hemen önce yeni `varsha` bloğu (45 satır). Var olan hiçbir satır değişmez (`dasha.systems` satırına dokunulmaz: mudda dasanın adı `varsha.mudda.system`'dedir). | İki tarafın anahtarları korunur; `i18n.test.ts` üç dilin eşitliğini denetler. |
| `app/ui/dasha/DashaTreePanel.tsx` | 7 | On satır: iki içe aktarım değişir, sistem `treeSystem` ile okunur, seçici `TreeSystemSelect` olur, üçüncü kol `MuddaDashaTree`'dir. | Plan 26 ve 29'un bu dosyaya dokunması beklenmez; dokunduysa iki tarafın değişikliği korunur. |
| `app/e2e/ipc.ts` | 7 | `compute_avasthas` taklidinden sonra sekiz satır (iki taklit). | İki tarafın taklitleri korunur. |
| `tests/golden/modules/README.md` | 8 | Tablonun sonuna bir satır; Avastha paragrafından sonra bir paragraf. | İki tarafın satır ve paragrafları korunur. |

Metin çatışması beklenen yerler: `app/ui/layout/model.test.ts`'in tek satırlık beklentisi, `README.md` tablosunun son satırı ve üretilen iki dosya. Öteki eklemeler dosyaların ortasındaki çapa satırlarındadır (`avastha`, `dasha_children`, `computeAvasthas`); Plan 26 ve Plan 29'un planları bu belge yazılırken depoda değildi, aynı çapayı seçtilerse iki tarafın satırları sırayla korunur. Birleştirmeden sonra üretilen iki dosya iki komutla yeniden üretilir ve bütün kapılar yeniden koşulur (birleştirenin işidir, bu planın değil).

**Kökten dışa aktarım yok.** Bu plan `crates/jyotish-core/src/lib.rs`'e `pub use` satırı eklemez. Modülün dışa açtığı adlar yalnızca `jyotish_core::varsha::…` yolundadır; Rust'ta başka bir modülün aynı adlı öğesiyle çatışamazlar. Arayüz bağlamalarındaki yeni tip adları şunlardır ve başka bir planın aynı adı üretmesi specta'da ad çatışmasıdır: `MuddaDasha`, `Muntha`, `VarshaReport`, `VarshaView`, `YearFacts`. Yeni komut adları: `compute_varsha`, `save_varsha_chart`. Birleştiren, öteki planların bağlamalarında bu adlardan biri varsa (özellikle `YearFacts`) birleştirmeden önce haber verir; ad bu planda `Varsha` önekiyle değiştirilir.

Bilerek dokunulmayan ortak dosyalar: `crates/jyotish-core/src/error.rs`, `crates/jyotish-core/src/dasha.rs`, `crates/jyotish-core/src/timeline.rs`, `crates/jyotish-core/src/search/events.rs`, `crates/jyotish-core/src/chart.rs`, `crates/jyotish-core/tests/modules_golden.rs`, `app/src-tauri/src/commands.rs`, `app/src-tauri/src/module_commands.rs`, `app/src-tauri/src/muhurta_commands.rs`, `app/src-tauri/src/client_commands.rs`, `app/src-tauri/tests/modules.rs`, `app/ui/App.tsx`, `app/ui/test/render.tsx`, `app/ui/dasha/system.tsx`, `app/ui/dasha/DashaStripPanel.tsx`, `app/ui/dasha/dasha.css`, `app/ui/panels/panels.test.tsx`, `app/ui/panels/panels.css`, `app/ui/panels/labels.ts`, `app/ui/panels/format.ts`, `app/ui/modules/muhurta.ts`, `app/ui/transit/store.tsx`, `app/ui/i18n/i18n.test.ts`, `app/e2e/desk.spec.ts`, `app/e2e/visual.spec.ts`, `app/playwright.config.ts`.

## Sonraki işlere verilen API

Bu planın ürettiği ve sonraki işlerin (özellikle Plan 28) tüketeceği adlar. Rust adları `jyotish_core::varsha` yolundadır (D1); imzalar görevlerde tam olarak verilir.

| API | Tüketen | Not |
|---|---|---|
| `solar_return_jd`, `varsha_span`, `completed_years_at`, `SIDEREAL_YEAR_DAYS`, `RETURN_BRACKET_DAYS`, `RETURN_RESOLUTION_DAYS` | Plan 28 (yıl lordu için dönüş anının gündüz ya da gece oluşu), ay dönüşü gibi sonraki teknikler | Yalnızca Güneş'in sidereal boylamını sorar |
| `mudda_dasha`, `mudda_first_lord`, `mudda_remainder`, `mudda_days`, `MuddaDasha`, `MUDDA_ORDER`, `MUDDA_TOTAL_DAYS`, `MUDDA_LEVELS`, `MUDDA_OFFSET` | Dasa şeridinin mudda görünümü (sonraki iş), AI paneli | Saf; dönemler `DashaPeriod` tipindedir |
| `muntha_sign`, `year_facts`, `Muntha`, `YearFacts` | Plan 28: beş adayın üçü (muntha lordu, yıl lagnası lordu, doğum lagnası lordu) | Saf; burçlardan |
| `compute_varsha`, `VarshaReport`, `VarshaYear`, `VARSHA_METHOD`, `MAX_VARSHA_AGE` | Plan 28: rapora `year_lord` ve `sahams` alanları eklenir; yıllık `Chart` oradan okunur | `time_known` ve `method` taşır |
| `varsha_commands::{compute_varsha, compute_varsha_blocking, save_varsha_chart, save_varsha_chart_blocking, VarshaView, VARSHA_LABEL}` | Plan 28'in panel ekleri, AI paneli | Yeni hata kodu yok |
| Arayüz: `modules/useVarsha.ts`, `varsha/yearStore.ts`, `varsha/YearControls.tsx`, `varsha/zone.ts`, `panels/VarshaPanel.tsx`, `dasha/MuddaDashaTree.tsx`, `dasha/muddaSystem.tsx`; fikstürde `varsha` | Plan 28: panele yıl lordu dökümü ve saham tablosu | i18n: `varsha.*`, `panel.varsha` |

## Dosya Yapısı (bu plan)

```
crates/jyotish-core/
  src/varsha/mod.rs               modül kaydı, VARSHA_METHOD, MAX_VARSHA_AGE (yeni)
  src/varsha/solar_return.rs      dönüş anı, yılın aralığı, süren yıl (yeni)
  src/varsha/mudda.rs             mudda dasa: sıra, paylar, ilk lord, dönemler (yeni)
  src/varsha/muntha.rs            muntha, YearFacts (yeni)
  src/varsha/collect.rs           VarshaReport, VarshaYear, compute_varsha (yeni)
  src/lib.rs                      pub mod varsha;
  tests/varsha_golden.rs          el örneklerinin testi (yeni)
tests/golden/modules/
  varsha/muntha/*.json            beş el örneği (yeni)
  varsha/mudda/*.json             dokuz el örneği (yeni)
  varsha/REVIEW.md                okuma kuralları, tablolar, onay bekleyen noktalar (yeni)
  README.md                       bir satır ve bir paragraf
app/src-tauri/
  src/varsha_commands.rs          compute_varsha, save_varsha_chart (yeni)
  src/lib.rs                      modül ve komut kaydı
  tests/varsha.rs                 gerçek motorla ±1″, çapraz denetim, kayıt, hatalar, bütçe (yeni)
  tests/ui_fixture.rs             fikstüre varsha
app/ui/
  bindings.ts                     üretilir
  test/fixtures/delhi.json        üretilir
  test/fixture.ts                 UiFixture'ın yeni alanı
  api.ts, api.test.ts             iki sarmalayıcı
  modules/useVarsha.ts            rapor kancası, yer seçimi (yeni)
  varsha/yearStore.ts             ortak yıl deposu (yeni)
  varsha/YearControls.tsx         yıl seçici (yeni)
  varsha/zone.ts                  ofset yazımı (yeni)
  panels/varsha.css               (yeni)
  panels/VarshaPanel.tsx (+test)  (yeni)
  panels/registry.ts
  layout/model.ts (+test)         varsha türü, MODULE_PANELS
  dasha/muddaSystem.tsx           ağacın üç sistemi (yeni)
  dasha/MuddaDashaTree.tsx (+test)  (yeni)
  dasha/DashaTreePanel.tsx        üçüncü sistem
  i18n/{tr,en,fr}.ts              panel.varsha, varsha
app/e2e/
  ipc.ts                          compute_varsha ve save_varsha_chart taklidi
  varsha.desk.spec.ts             iki akış (yeni)
  varsha.visual.spec.ts           iki görsel test (yeni; temelleri kontrolör üretir)
```

### Kapsam

| Yol haritası maddesi | Yer |
|---|---|
| §5 (1) Tabanlar ve dönüş anı çekirdeği (yaş → an; ±1″ testi) | Task 1 (D2–D5); gerçek motorla ±1″: Task 5 |
| §4.14 Dönüş anı: sidereal dönüş, seçili ayanamsayla | Task 1 (D3); REVIEW 1 |
| §5 (2) Yıllık harita toplayıcısı ve komutu (yer seçimiyle) | Task 3 (toplayıcı), Task 5 (komut) (D6, D7, D13) |
| §4.14 Yer: doğum yeri (öneri) ya da bulunulan yer | Task 3, Task 5, Task 6 (D7); REVIEW 2 |
| §5 (3) Muntha ve yıl başlığı olguları | Task 3 (D8); REVIEW 3–5 |
| §5 (4) Mudda dasa çekirdeği ve ikinci yazım | Task 2 (D9); REVIEW 6–11 |
| §5 (5) El örnekleri | Task 4 (D12) |
| §5 (6) Varshaphala paneli (yıl seçici, harita, muntha) | Task 6 (D15, D16, D18) |
| §5 (6) Var olan dasa panellerinde mudda görünümü | Task 7 (D17); şerit kapsam dışı |
| §5 (7) i18n, "saat bilinmiyor" | Task 6, Task 7 (D10, D20) |
| §5 (7) Olay haritası olarak kaydetme (var olan akış) | Task 5 (komut), Task 6 (düğme) (D14) |
| K12 Paletten ekleme; uçtan uca akış | Task 6, Task 7 (D22) |
| Y4 REVIEW kaydı, "doğrulanmamış tablo" notu | Task 7 (D19), Task 8 |
| §5 (8) Kapılar ve elle denetim; sahibin denetimi | Task 8 |
| §4.14 Kapsam dışı: yıl lordu, sahamlar, Tajika yogaları, tri-pataki | "Bu planın kapsamadıkları"; Plan 28 |

---

### Task 1: Çekirdek: `varsha` modülü ve dönüş anı

Yol haritası §4.14 ("Dönüş anı") ve §5 Plan 27 görev (1); K5; D1–D5. Var olan hiçbir hesabın davranışı değişmez: `timeline.rs` (`lon_of`, `sidereal`), `chart.rs` (`finite`), `settings.rs` ve `zodiac.rs` yalnızca okunur.

**Files:**
- Create: `crates/jyotish-core/src/varsha/mod.rs`, `crates/jyotish-core/src/varsha/solar_return.rs`
- Modify: `crates/jyotish-core/src/lib.rs` (tek satır; paylaşılan dosya)
- Test: `solar_return.rs` `mod tests` (5)

**Interfaces:**
- Consumes (hepsi var; `pub(crate)` olanlar aynı crate'ten okunur): `crate::timeline::{lon_of, sidereal}`; `crate::chart::finite`; `crate::settings::{CalcSettings, JD_UT_RANGE}`; `crate::zodiac::normalize`; `ephemeris::{Body, Ephemeris}`; testte `crate::fake::FakeEph`.
- Produces (`jyotish_core::varsha` yolunda):
  - `VARSHA_METHOD: &str = "varsha.sidereal_return_v1"`, `MAX_VARSHA_AGE: u8 = 120`.
  - `SIDEREAL_YEAR_DAYS: f64 = 365.256_363`, `RETURN_BRACKET_DAYS: f64 = 3.0`, `RETURN_RESOLUTION_DAYS: f64` (0,01 saniye).
  - `solar_return_jd<E: Ephemeris + ?Sized>(eph: &mut E, settings: &CalcSettings, natal_sun_sidereal_lon: f64, birth_jd_ut: f64, age: u8) -> Result<f64, CoreError>`: yaş 0 doğum anıdır.
  - `varsha_span(eph, settings, natal_sun_sidereal_lon, birth_jd_ut, age) -> Result<(f64, f64), CoreError>`: `[dönüş, sonraki dönüş)`.
  - `completed_years_at(eph, settings, natal_sun_sidereal_lon, birth_jd_ut, at_jd_ut) -> Result<u8, CoreError>`: verilen anda süren yılın yaşı.
  - Hatalar: yaş > 120 `InvalidDashaInput("age")`; an aralık dışı ya da sonlu değil `InvalidJulianDay`; boylam sonlu değil `NonFiniteResult("varsha")`; Güneş aralıkta dönmüyor `PanchangSearch("solar_return")`.

- [ ] **Step 1: Tabanları ölç**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
git log -1 --format=%h
git status --short
rustc --version
[ -d app/node_modules ] || pnpm --dir app install --frozen-lockfile
git status --short
cargo test --workspace 2>&1 | grep -E "^test result|Running"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
```
Expected: iki `git status --short` de boş (bağımlılık kurulumu kilit dosyasını değiştirmez; `app/node_modules` izlenmez); `rustc 1.100.0` ya da yenisi; `jyotish-core` lib `466 passed`, `avastha_golden` `5 passed`, golden `8 passed`, `modules_golden` `17 passed`; `atlas` `build` 6, `moment_fixed_lmt` 11, `moment_iana` 13, `search` 8, `shipped` 3; `store` lib `122 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `58 passed`, `avastha` 1, `bhava` 1, `bindings` 1, `compute_chart` 1, `modules` 1, `natal_extras` 1, `open_client` 1, `ui_fixture` 2; Vitest `97 passed (97)`, `973 passed (973)`; `✓ built`. `git log` çıktısı `9727c55` değilse (`main` ilerlediyse ya da paralel bir plan birleştiyse) ölçülen değerler ilerleme notlarına yazılır ve sayaç zinciri o tabandan, aynı farklarla yürütülür. Okunan commit Task 8'de `BASE` olarak kullanılır: not edilir. E2e ve görsel testler bu adımda koşulmaz (kontrolörün tabanı: e2e 33, görsel 15).

- [ ] **Step 2: Testler: dönüş anı (±1″), yılın aralığı, doğrulama, süren yıl; modül kaydı**

Önce modül kaydı (testlerin derlenebilmesi için), sonra test gövdesi. Testler sahte ephemeris kullanır: Güneş sabit hızla yürür ve hız tam bir sidereal yılda 360° olacak biçimde seçilir, böylece her dönüşün anı elle bilinir (doğum + yaş × 365,256363 gün). Testler: dönüşte Güneş'in doğum boylamından bir yay saniyesinden az ayrıştığı ve geride durmadığı (iki zodyak kipinde, 1, 2, 37 ve 120 yaşlarında); yaş 0'ın doğum anı olduğu ve ardışık yılların boşluksuz olduğu; ortalama yıldan sapan bir Güneş'in payın içinde bulunduğu, payın dışına düşenin hata olduğu; yaşın, anın ve boylamın denetimi; süren yılın dönüşleri izlediği.

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 612a167..dca0f15 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -52,6 +52,7 @@ pub mod shadbala;
 pub mod timeline;
 pub mod transit;
 pub mod varga;
+pub mod varsha;
 pub mod yogas;
 pub mod zodiac;
 
````

**Yeni dosya:** `crates/jyotish-core/src/varsha/mod.rs`

````rust
//! Varshaphala (yol haritası §4.14, Plan 27): Tajika yıllık haritası. Yıl, Güneş'in doğumdaki
//! sidereal boylamına döndüğü anda başlar (`solar_return`); o anın haritası yıllık haritadır.
//! Muntha, doğum lagnasının burcundan yılda bir burç ilerleyen noktadır (`muntha`); mudda dasa,
//! Vimshottari paylarının bir yıla sığdırılmasıdır (`mudda`).
//!
//! Kaynak: Tajika geleneğinin yıllık harita bölümleri; kurallar özgün olarak uygulanır, alıntı
//! yapılmaz. Okuma seçimleri ve sahibin onayını bekleyen varyant noktaları
//! `tests/golden/modules/varsha/REVIEW.md`'dedir. Dönüş her zodyak kipinde sidereal boylamdandır;
//! muntha rasi çerçevesindedir (K5). Yorum ve hüküm yoktur: modül yalnızca anı, haritayı, burcu ve
//! dönemleri döner. Adlar `jyotish_core` kökünden yeniden dışa aktarılmaz.

mod solar_return;

pub use solar_return::{
    completed_years_at, solar_return_jd, varsha_span, RETURN_BRACKET_DAYS, RETURN_RESOLUTION_DAYS,
    SIDEREAL_YEAR_DAYS,
};

/// `VarshaReport::method`: bu modüldeki okumaların kimliği.
pub const VARSHA_METHOD: &str = "varsha.sidereal_return_v1";

/// Yıllık harita istenebilen en büyük yaş (tamamlanan yıl): Vimshottari'nin 120 yılı.
pub const MAX_VARSHA_AGE: u8 = 120;
````

**Yeni dosya (şimdilik yalnızca test modülü):** `crates/jyotish-core/src/varsha/solar_return.rs`

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::settings::ZodiacMode;
    use crate::zodiac::angular_distance;

    const JD: f64 = 2_451_545.0;
    /// Bir yay saniyesi, derece.
    const ARCSEC: f64 = 1.0 / 3600.0;
    /// Sahte Güneş'in hızı: tam bir sidereal yılda 360°.
    const SUN_SPEED: f64 = 360.0 / SIDEREAL_YEAR_DAYS;

    fn eph() -> FakeEph {
        FakeEph::default().with(Body::Sun, 10.0, SUN_SPEED)
    }

    fn settings(zodiac: ZodiacMode) -> CalcSettings {
        CalcSettings {
            zodiac,
            ..CalcSettings::default()
        }
    }

    /// Sahte Güneş'in `jd` anındaki sidereal boylamı (tropikal 10°, ayanamsa 24°).
    fn sun_at(eph: &mut FakeEph, jd: f64) -> f64 {
        lon_of(eph, jd, Body::Sun, sidereal(&CalcSettings::default())).unwrap()
    }

    #[test]
    fn the_return_brings_the_sidereal_sun_back_within_one_arcsecond() {
        // Doğumda sidereal Güneş 10° − 24° = 346°. Her dönüş tam bir sidereal yıl sonradır.
        for zodiac in [
            ZodiacMode::Sidereal,
            ZodiacMode::TropicalRasiSiderealNakshatra,
        ] {
            let mut eph = eph();
            let natal = sun_at(&mut eph, JD);
            assert!((natal - 346.0).abs() < 1e-9, "{natal}");
            for age in [1_u8, 2, 37, 120] {
                let jd = solar_return_jd(&mut eph, &settings(zodiac), natal, JD, age).unwrap();
                let back = sun_at(&mut eph, jd);
                assert!(
                    angular_distance(back, natal) < ARCSEC,
                    "{zodiac:?} age {age}: {back} vs {natal}"
                );
                // Güneş doğum boylamına varmıştır: yay eksi değildir.
                assert!(arc_from(natal, back) >= 0.0, "{zodiac:?} age {age}");
                let want = JD + f64::from(age) * SIDEREAL_YEAR_DAYS;
                assert!((jd - want).abs() < 1e-6, "{zodiac:?} age {age}: {jd}");
            }
        }
    }

    #[test]
    fn age_zero_is_the_birth_moment_and_the_span_runs_to_the_next_return() {
        let mut eph = eph();
        let natal = sun_at(&mut eph, JD);
        let s = CalcSettings::default();
        assert_eq!(solar_return_jd(&mut eph, &s, natal, JD, 0).unwrap(), JD);
        let (start, end) = varsha_span(&mut eph, &s, natal, JD, 0).unwrap();
        assert_eq!(start, JD);
        assert!((end - start - SIDEREAL_YEAR_DAYS).abs() < 1e-6, "{end}");
        // Ardışık yıllar boşluksuzdur: bir yılın sonu sonrakinin başıdır.
        let (next_start, next_end) = varsha_span(&mut eph, &s, natal, JD, 1).unwrap();
        assert_eq!(next_start, end);
        assert!(next_end > next_start);
        // En büyük yaşın yılı da kapanır (121. dönüş yalnızca yılın sonu olarak sorulur).
        let (last_start, last_end) = varsha_span(&mut eph, &s, natal, JD, MAX_VARSHA_AGE).unwrap();
        assert!((last_end - last_start - SIDEREAL_YEAR_DAYS).abs() < 1e-6);
    }

    #[test]
    fn a_sun_off_the_mean_year_is_found_inside_the_bracket_and_a_stray_one_is_an_error() {
        // Hız 0,9856°/gün: yıl 365,2597 gün; 120 yılda tahminden 0,4 gün sapar (pay 3 gün).
        let mut slow = FakeEph::default().with(Body::Sun, 10.0, 0.9856);
        let natal = sun_at(&mut slow, JD);
        let s = CalcSettings::default();
        let jd = solar_return_jd(&mut slow, &s, natal, JD, 120).unwrap();
        assert!((jd - (JD + 120.0 * 360.0 / 0.9856)).abs() < 1e-6, "{jd}");
        // Hız 0,98°/gün: yıl 367,35 gün; 30 yılda sapma 62 gün, dönüş aralığın dışındadır.
        let mut stray = FakeEph::default();
        let natal = sun_at(&mut stray, JD);
        assert_eq!(
            solar_return_jd(&mut stray, &s, natal, JD, 30),
            Err(CoreError::PanchangSearch("solar_return"))
        );
    }

    #[test]
    fn age_moment_and_longitude_are_validated() {
        let mut eph = eph();
        let s = CalcSettings::default();
        assert_eq!(
            solar_return_jd(&mut eph, &s, 346.0, JD, MAX_VARSHA_AGE + 1),
            Err(CoreError::InvalidDashaInput("age"))
        );
        assert_eq!(
            varsha_span(&mut eph, &s, 346.0, JD, MAX_VARSHA_AGE + 1),
            Err(CoreError::InvalidDashaInput("age"))
        );
        assert!(matches!(
            solar_return_jd(&mut eph, &s, 346.0, f64::NAN, 1),
            Err(CoreError::InvalidJulianDay(_))
        ));
        assert_eq!(
            solar_return_jd(&mut eph, &s, f64::INFINITY, JD, 1),
            Err(CoreError::NonFiniteResult("varsha"))
        );
        // Hesap aralığının sonuna yakın doğum: dönüş aralığın dışına düşer.
        let late = *JD_UT_RANGE.end() - 100.0;
        assert!(matches!(
            solar_return_jd(&mut eph, &s, 346.0, late, 1),
            Err(CoreError::InvalidJulianDay(_))
        ));
        assert!(matches!(
            completed_years_at(&mut eph, &s, 346.0, JD, f64::NAN),
            Err(CoreError::InvalidJulianDay(_))
        ));
    }

    #[test]
    fn completed_years_follow_the_returns() {
        let mut eph = eph();
        let natal = sun_at(&mut eph, JD);
        let s = CalcSettings::default();
        let fifth = solar_return_jd(&mut eph, &s, natal, JD, 5).unwrap();
        let mut age_at = |at: f64| completed_years_at(&mut eph, &s, natal, JD, at).unwrap();
        assert_eq!(age_at(JD - 1.0), 0, "before birth");
        assert_eq!(age_at(JD), 0);
        assert_eq!(age_at(fifth - 1e-3), 4);
        assert_eq!(age_at(fifth), 5, "the return moment opens the year");
        assert_eq!(age_at(fifth + 200.0), 5);
        assert_eq!(age_at(JD + 300.0 * SIDEREAL_YEAR_DAYS), MAX_VARSHA_AGE);
    }
}
````

- [ ] **Step 3: Testlerin derlenmediğini gör**

Run: `cargo test -p jyotish-core --lib varsha 2>&1 | grep -E "^error" | sort | uniq -c | head`
Expected: derleme hatası: `error[E0425]: cannot find function` (`solar_return_jd`, `varsha_span`, `completed_years_at`, `arc_from`: henüz yoklar) ve `cannot find type `CalcSettings``; son satır `could not compile `jyotish-core` (lib test)`. Test koşmaz.

- [ ] **Step 4: Dönüş anı**

**Dosyanın başına eklenir** (test modülünün üstüne, arada bir boş satırla): `crates/jyotish-core/src/varsha/solar_return.rs`

````rust
//! Dönüş anı: Güneş'in doğumdaki sidereal boylamına `age` kez döndüğü an. Yıllık haritanın ve
//! mudda dasanın başlangıcıdır.
//!
//! Yöntem: ilk tahmin doğum + `age` × sidereal yıldır; Güneş hep ileri yürüdüğü için tahminin
//! iki yanındaki dar aralıkta (`RETURN_BRACKET_DAYS`) işaretli yay tek bir kez sıfırdan geçer ve
//! geçiş ikiye bölmeyle bulunur (`RETURN_RESOLUTION_DAYS`). Dönen an, Güneş'in doğum boylamına
//! varmış olduğu ilk örnektir: o anda yay sıfır ya da çok küçük bir artıdır.
//!
//! Boylam her zodyak kipinde sidereal sorulur (seçili ayanamsayla): tropikal dönüş her yıl
//! yaklaşık yirmi dakika erken gelir ve yıllık lagnayı değiştirir.

use ephemeris::{Body, Ephemeris};

use crate::chart::finite;
use crate::settings::{CalcSettings, JD_UT_RANGE};
use crate::timeline::{lon_of, sidereal};
use crate::zodiac::normalize;
use crate::CoreError;

use super::MAX_VARSHA_AGE;

/// Sidereal yılın ortalama uzunluğu, gün: dönüş aramasının ilk tahmini.
pub const SIDEREAL_YEAR_DAYS: f64 = 365.256_363;
/// Tahminin iki yanında aranan pay, gün. Gerçek dönüş tahminden birkaç dakikadan çok sapmaz;
/// pay geniş tutulmuştur.
pub const RETURN_BRACKET_DAYS: f64 = 3.0;
/// Dönüş anının çözünürlüğü: 0,01 saniye. Güneş bu sürede yaklaşık 0,0004″ yürür.
pub const RETURN_RESOLUTION_DAYS: f64 = 0.01 / 86_400.0;
/// İkiye bölme sınırı (6 gün / 2^40; pay bırakılmıştır).
const MAX_BISECTIONS: u32 = 40;

/// İşaretli yay `lon − target`, [−180, 180): Güneş hedefin gerisindeyse eksi.
fn arc_from(target: f64, lon: f64) -> f64 {
    normalize(lon - target + 180.0) - 180.0
}

/// `n`. dönüşün anı; `n` denetlenmez (yılın sonu için `MAX_VARSHA_AGE + 1` de sorulur).
fn return_no<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    natal_sun_sidereal_lon: f64,
    birth_jd_ut: f64,
    n: u32,
) -> Result<f64, CoreError> {
    // NaN ve sonsuz da aralık dışındadır.
    if !JD_UT_RANGE.contains(&birth_jd_ut) {
        return Err(CoreError::InvalidJulianDay(birth_jd_ut));
    }
    let target = normalize(finite(natal_sun_sidereal_lon, "varsha")?);
    if n == 0 {
        return Ok(birth_jd_ut);
    }
    let guess = birth_jd_ut + f64::from(n) * SIDEREAL_YEAR_DAYS;
    let (mut lo, mut hi) = (guess - RETURN_BRACKET_DAYS, guess + RETURN_BRACKET_DAYS);
    if !(JD_UT_RANGE.contains(&lo) && JD_UT_RANGE.contains(&hi)) {
        return Err(CoreError::InvalidJulianDay(guess));
    }
    let zodiac = sidereal(settings);
    let arc = |eph: &mut E, jd: f64| {
        Ok::<_, CoreError>(arc_from(target, lon_of(eph, jd, Body::Sun, zodiac)?))
    };
    // Aralığın başında Güneş hedefin gerisinde, sonunda hedefte ya da ötesinde olmalıdır.
    if !(arc(eph, lo)? < 0.0 && arc(eph, hi)? >= 0.0) {
        return Err(CoreError::PanchangSearch("solar_return"));
    }
    let mut steps = 0;
    while hi - lo > RETURN_RESOLUTION_DAYS && steps < MAX_BISECTIONS {
        let mid = 0.5 * (lo + hi);
        if arc(eph, mid)? < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
        steps += 1;
    }
    Ok(hi)
}

fn check_age(age: u8) -> Result<(), CoreError> {
    if age > MAX_VARSHA_AGE {
        return Err(CoreError::InvalidDashaInput("age"));
    }
    Ok(())
}

/// Güneş'in `natal_sun_sidereal_lon` boylamına doğumdan sonra `age`. kez döndüğü an (UT Jülyen
/// günü). `age` tamamlanan yıldır: 0 doğum anının kendisidir, 1 ilk doğum günü dönüşüdür.
/// `age > MAX_VARSHA_AGE` `InvalidDashaInput("age")`; an hesap aralığının dışına düşerse
/// `InvalidJulianDay`; Güneş aranan aralıkta dönmüyorsa (yalnızca bozuk ya da sahte ephemeris)
/// `PanchangSearch("solar_return")`.
pub fn solar_return_jd<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    natal_sun_sidereal_lon: f64,
    birth_jd_ut: f64,
    age: u8,
) -> Result<f64, CoreError> {
    check_age(age)?;
    return_no(
        eph,
        settings,
        natal_sun_sidereal_lon,
        birth_jd_ut,
        u32::from(age),
    )
}

/// `age` yaşının yılı: `[dönüş, sonraki dönüş)`. Yılın uzunluğu iki dönüşün arasıdır (yaklaşık
/// 365,2564 gün).
pub fn varsha_span<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    natal_sun_sidereal_lon: f64,
    birth_jd_ut: f64,
    age: u8,
) -> Result<(f64, f64), CoreError> {
    check_age(age)?;
    let n = u32::from(age);
    let start = return_no(eph, settings, natal_sun_sidereal_lon, birth_jd_ut, n)?;
    let end = return_no(eph, settings, natal_sun_sidereal_lon, birth_jd_ut, n + 1)?;
    Ok((start, end))
}

/// `at_jd_ut` anında süren yılın yaşı: dönüşü `at_jd_ut`'den sonra olmayan en büyük yaş. Doğumdan
/// önceki an 0, `MAX_VARSHA_AGE`. dönüşten sonraki her an `MAX_VARSHA_AGE` verir.
pub fn completed_years_at<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    natal_sun_sidereal_lon: f64,
    birth_jd_ut: f64,
    at_jd_ut: f64,
) -> Result<u8, CoreError> {
    if !at_jd_ut.is_finite() {
        return Err(CoreError::InvalidJulianDay(at_jd_ut));
    }
    let returned = |eph: &mut E, age: u8| {
        solar_return_jd(eph, settings, natal_sun_sidereal_lon, birth_jd_ut, age)
            .map(|jd| jd <= at_jd_ut)
    };
    // Doğum anı da burada denetlenir.
    if !returned(eph, 0)? {
        return Ok(0);
    }
    let years = ((at_jd_ut - birth_jd_ut) / SIDEREAL_YEAR_DAYS).floor();
    let mut age = years.clamp(0.0, f64::from(MAX_VARSHA_AGE)) as u8;
    // Tahmin en çok bir yıl sapar: gerçek dönüşlerle düzeltilir.
    while age < MAX_VARSHA_AGE && returned(eph, age + 1)? {
        age += 1;
    }
    while age > 0 && !returned(eph, age)? {
        age -= 1;
    }
    Ok(age)
}
````

- [ ] **Step 5: Testler ve kapılar**

```bash
cargo fmt --all
cargo test -p jyotish-core --lib varsha 2>&1 | grep -E "^test result"
cargo test -p jyotish-core 2>&1 | grep -E "^test result"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
```
Expected: `5 passed`; lib `471 passed`, `avastha_golden` `5 passed`, golden `8 passed`, `modules_golden` `17 passed`; fmt ve clippy çıktısız; sayaç `0`. (clippy kapanışın `let mut` ile bağlanmasını reddeder: `arc` kapanışı `let` ile yazılıdır, kod bloğundaki yazım korunur.) Sayaç `0` değilse ve düşen test bir bütçe iddiasıysa Global Constraints'in "Bütçe" maddesi uygulanır.

- [ ] **Step 6: Commit**

```bash
git add crates/jyotish-core/src/varsha crates/jyotish-core/src/lib.rs
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): varsha module with the sidereal solar return moment" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: üç dosya (`3 files changed, 315 insertions(+)`); yasaklı ad taraması çıktısız; `git status --short` boş.

---

### Task 2: Çekirdek: mudda dasa ve ikinci yazım

Yol haritası §4.14 ("Mudda dasa") ve §5 Plan 27 görev (4); D9. Saf hesaptır: ephemeris, harita ve saat görmez. `dasha.rs` yalnızca okunur (`DashaPeriod` tipi; testte `vimshottari_years`, `VIMSHOTTARI_ORDER`, `VIMSHOTTARI_TOTAL_YEARS`).

**Files:**
- Create: `crates/jyotish-core/src/varsha/mudda.rs`
- Modify: `crates/jyotish-core/src/varsha/mod.rs`
- Test: `mudda.rs` `mod tests` (7)

**Interfaces:**
- Consumes (hepsi var): `crate::dasha::DashaPeriod` (`lords`, `level`, `start_jd_ut`, `end_jd_ut`, `has_children`); `crate::graha::Graha`.
- Produces (`jyotish_core::varsha`):
  - `MUDDA_ORDER: [Graha; 9]` (Güneş, Ay, Mars, Rahu, Jüpiter, Satürn, Merkür, Ketu, Venüs), `MUDDA_TOTAL_DAYS: f64 = 360.0`, `MUDDA_LEVELS: u8 = 2`, `MUDDA_OFFSET: u32 = 2`.
  - `mudda_days(lord: Graha) -> f64` (18, 30, 21, 54, 48, 57, 51, 21, 60).
  - `mudda_remainder(birth_nakshatra: u8, age: u8) -> u8` (0..=8), `mudda_first_lord(birth_nakshatra: u8, age: u8) -> Graha`.
  - `MuddaDasha { birth_nakshatra: u8, age: u8, remainder: u8, first_lord: Graha, periods: Vec<DashaPeriod> }`: 9 + 81 dönem, ön-sırada.
  - `mudda_dasha(birth_nakshatra: u8, age: u8, start_jd_ut: f64, end_jd_ut: f64) -> Result<MuddaDasha, CoreError>`; hatalar `InvalidDashaInput("nakshatra" | "range")`.

- [ ] **Step 1: Testler: paylar, ilk lord kuralı ve ikinci yazım, dönemler, alt dönemler, doğrulama; modül kaydı**

Testler: dokuz payın toplamının 360 olduğu ve her payın Vimshottari yılından türediği (ikinci yazım: `yıl × 360 / 120`); ilk lord kuralının on el örneği (türetmeler yorumlarda; artan 0 ve eksiye düşen toplam dahil); ikinci yazım (doğum nakshatrasının Vimshottari lordu her yıl sırada bir adım ilerler) 27 nakshatra × 121 yaşta; 360 günlük yılda dönem sınırlarının tam gün olduğu; dönüşten dönüşe yılın aynı paylarla ve boşluksuz bölündüğü; alt dönemlerin üst dönemin lordundan başlayıp onu doldurduğu; girdilerin denetimi.

````diff
diff --git a/crates/jyotish-core/src/varsha/mod.rs b/crates/jyotish-core/src/varsha/mod.rs
index 662ca02..d96f18a 100644
--- a/crates/jyotish-core/src/varsha/mod.rs
+++ b/crates/jyotish-core/src/varsha/mod.rs
@@ -9,8 +9,13 @@
 //! muntha rasi çerçevesindedir (K5). Yorum ve hüküm yoktur: modül yalnızca anı, haritayı, burcu ve
 //! dönemleri döner. Adlar `jyotish_core` kökünden yeniden dışa aktarılmaz.
 
+mod mudda;
 mod solar_return;
 
+pub use mudda::{
+    mudda_dasha, mudda_days, mudda_first_lord, mudda_remainder, MuddaDasha, MUDDA_LEVELS,
+    MUDDA_OFFSET, MUDDA_ORDER, MUDDA_TOTAL_DAYS,
+};
 pub use solar_return::{
     completed_years_at, solar_return_jd, varsha_span, RETURN_BRACKET_DAYS, RETURN_RESOLUTION_DAYS,
     SIDEREAL_YEAR_DAYS,
````

**Yeni dosya (şimdilik yalnızca test modülü):** `crates/jyotish-core/src/varsha/mudda.rs`

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dasha::{vimshottari_years, VIMSHOTTARI_ORDER, VIMSHOTTARI_TOTAL_YEARS};

    fn level(d: &MuddaDasha, n: u8) -> Vec<&DashaPeriod> {
        d.periods.iter().filter(|p| p.level == n).collect()
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn the_nine_shares_total_360_days_and_are_three_days_per_vimshottari_year() {
        assert_eq!(
            MUDDA_ORDER.map(mudda_days),
            [18.0, 30.0, 21.0, 54.0, 48.0, 57.0, 51.0, 21.0, 60.0]
        );
        let total: f64 = MUDDA_ORDER.iter().map(|&g| mudda_days(g)).sum();
        assert_eq!(total, MUDDA_TOTAL_DAYS);
        // İkinci yazım: pay Vimshottari yılının 120'de biridir; 360 günde yıl başına üç gün.
        for lord in MUDDA_ORDER {
            assert_eq!(
                mudda_days(lord),
                vimshottari_years(lord) * MUDDA_TOTAL_DAYS / VIMSHOTTARI_TOTAL_YEARS,
                "{lord:?}"
            );
        }
        // Sıra, Vimshottari sırasının Güneş'ten başlayan yazımıdır.
        for (i, lord) in MUDDA_ORDER.into_iter().enumerate() {
            assert_eq!(lord, VIMSHOTTARI_ORDER[(i + 2) % 9]);
            assert_eq!(order_index(lord), i);
        }
    }

    #[test]
    fn the_first_lord_follows_the_remainder_rule() {
        // Krittika (3), tamamlanan yıl 0: 3 + 0 − 2 = 1 → artan 1 → Güneş.
        assert_eq!(mudda_remainder(3, 0), 1);
        assert_eq!(mudda_first_lord(3, 0), Graha::Sun);
        // Ashwini (1), yıl 0: 1 + 0 − 2 = −1; 9 eklenir: 8 → Ketu.
        assert_eq!(mudda_remainder(1, 0), 8);
        assert_eq!(mudda_first_lord(1, 0), Graha::Ketu);
        // Bharani (2), yıl 0: 0 → artan 0 dokuzuncu lorddur: Venüs.
        assert_eq!(mudda_remainder(2, 0), 0);
        assert_eq!(mudda_first_lord(2, 0), Graha::Venus);
        // Swati (15), yıl 26: 15 + 26 − 2 = 39 = 4 × 9 + 3 → Mars.
        assert_eq!(mudda_remainder(15, 26), 3);
        assert_eq!(mudda_first_lord(15, 26), Graha::Mars);
        // Revati (27), yıl 120: 27 + 120 − 2 = 145 = 16 × 9 + 1 → Güneş.
        assert_eq!(mudda_first_lord(27, 120), Graha::Sun);
        // Rohini (4), yıl 3: 4 + 3 − 2 = 5 → Jüpiter. Ardra (6), yıl 0: 4 → Rahu.
        assert_eq!(mudda_first_lord(4, 3), Graha::Jupiter);
        assert_eq!(mudda_first_lord(6, 0), Graha::Rahu);
        // Pushya (8), yıl 0: 6 → Satürn. Ashlesha (9), yıl 0: 7 → Merkür. Rohini, yıl 0: 2 → Ay.
        assert_eq!(mudda_first_lord(8, 0), Graha::Saturn);
        assert_eq!(mudda_first_lord(9, 0), Graha::Mercury);
        assert_eq!(mudda_first_lord(4, 0), Graha::Moon);
    }

    #[test]
    fn second_formulation_the_birth_nakshatra_lord_advanced_one_step_a_year() {
        // Yıl 0'da ilk lord doğum nakshatrasının Vimshottari lordudur ve her yıl sırada bir
        // adım ilerler. Kural ile bu yürüyüş her nakshatra ve her yaşta aynı lordu verir.
        for nakshatra in 1..=27_u8 {
            let mut walk = (usize::from(nakshatra) - 1) % 9;
            for age in 0..=120_u8 {
                assert_eq!(
                    mudda_first_lord(nakshatra, age),
                    VIMSHOTTARI_ORDER[walk],
                    "nakshatra {nakshatra}, age {age}"
                );
                walk = (walk + 1) % 9;
            }
        }
    }

    #[test]
    fn nine_periods_fill_the_year_in_order_from_the_first_lord() {
        // 360 günlük yıl: sınırlar tam gündür. Swati, yıl 26: ilk lord Mars.
        let d = mudda_dasha(15, 26, 1000.0, 1360.0).unwrap();
        assert_eq!(
            (d.birth_nakshatra, d.age, d.remainder, d.first_lord),
            (15, 26, 3, Graha::Mars)
        );
        assert_eq!(d.periods.len(), 9 + 81);
        let top = level(&d, 1);
        let lords: Vec<_> = top.iter().map(|p| p.lords[0]).collect();
        assert_eq!(
            lords,
            [
                Graha::Mars,
                Graha::Rahu,
                Graha::Jupiter,
                Graha::Saturn,
                Graha::Mercury,
                Graha::Ketu,
                Graha::Venus,
                Graha::Sun,
                Graha::Moon
            ]
        );
        // Mars 21, Rahu 54, Jüpiter 48, Satürn 57, Merkür 51, Ketu 21, Venüs 60, Güneş 18, Ay 30.
        let ends: Vec<_> = top.iter().map(|p| p.end_jd_ut).collect();
        assert_eq!(
            ends,
            [1021.0, 1075.0, 1123.0, 1180.0, 1231.0, 1252.0, 1312.0, 1330.0, 1360.0]
        );
        assert_eq!(top[0].start_jd_ut, 1000.0);
        for w in top.windows(2) {
            assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
        }
        assert!(top
            .iter()
            .all(|p| p.has_children && p.lords.len() == 1 && p.level == 1));
    }

    #[test]
    fn the_year_between_two_returns_is_split_in_the_same_shares() {
        // Dönüşten dönüşe yıl 365,2564 gündür: her dönem aynı oranda uzar, yıl boşluksuz dolar.
        let (start, end) = (2_461_000.25, 2_461_365.506_363);
        let d = mudda_dasha(3, 0, start, end).unwrap();
        let top = level(&d, 1);
        assert_eq!(top[0].lords, [Graha::Sun]);
        assert!(close(
            top[0].end_jd_ut - top[0].start_jd_ut,
            (end - start) * 18.0 / 360.0
        ));
        assert_eq!(top[0].start_jd_ut, start);
        assert_eq!(top[8].end_jd_ut, end);
        let sum: f64 = top.iter().map(|p| p.end_jd_ut - p.start_jd_ut).sum();
        assert!(close(sum, end - start));
    }

    #[test]
    fn sub_periods_start_from_the_parent_lord_and_fill_the_parent() {
        let d = mudda_dasha(15, 26, 1000.0, 1360.0).unwrap();
        // Ön-sıra: her dönemin ardından dokuz alt dönemi.
        for (i, chunk) in d.periods.chunks(10).enumerate() {
            let parent = &chunk[0];
            assert_eq!(parent.level, 1, "chunk {i}");
            let kids = &chunk[1..];
            assert!(kids
                .iter()
                .all(|k| k.level == 2 && !k.has_children && k.lords[0] == parent.lords[0]));
            assert_eq!(kids[0].lords[1], parent.lords[0], "starts from the parent");
            assert_eq!(kids[0].start_jd_ut, parent.start_jd_ut);
            assert_eq!(kids[8].end_jd_ut, parent.end_jd_ut);
            for w in kids.windows(2) {
                assert_eq!(w[0].end_jd_ut, w[1].start_jd_ut);
            }
        }
        // Mars dönemi 21 gün (1000 → 1021): Mars–Mars 21 × 21 / 360 = 1,225 gün.
        let mars_mars = &d.periods[1];
        assert_eq!(mars_mars.lords, [Graha::Mars, Graha::Mars]);
        assert!(close(mars_mars.end_jd_ut, 1001.225));
        // Mars–Rahu 21 × 54 / 360 = 3,15 gün: 1001,225 → 1004,375.
        assert_eq!(d.periods[2].lords, [Graha::Mars, Graha::Rahu]);
        assert!(close(d.periods[2].end_jd_ut, 1004.375));
    }

    #[test]
    fn nakshatra_and_range_are_validated() {
        for bad in [0_u8, 28] {
            assert_eq!(
                mudda_dasha(bad, 0, 0.0, 360.0),
                Err(CoreError::InvalidDashaInput("nakshatra"))
            );
        }
        for (start, end) in [
            (10.0, 10.0),
            (10.0, 5.0),
            (f64::NAN, 5.0),
            (0.0, f64::INFINITY),
        ] {
            assert_eq!(
                mudda_dasha(1, 0, start, end),
                Err(CoreError::InvalidDashaInput("range"))
            );
        }
    }
}
````

- [ ] **Step 2: Testlerin derlenmediğini gör**

Run: `cargo test -p jyotish-core --lib varsha 2>&1 | grep -E "^error" | sort | uniq -c | head`
Expected: derleme hatası: `error[E0425]: cannot find function` (`mudda_dasha`, `mudda_first_lord`, `mudda_remainder`, `mudda_days`) ve `cannot find value` (`MUDDA_ORDER`, `MUDDA_TOTAL_DAYS`): henüz yoklar. Test koşmaz.

- [ ] **Step 3: Mudda dasa**

**Dosyanın başına eklenir** (test modülünün üstüne, arada bir boş satırla): `crates/jyotish-core/src/varsha/mudda.rs`

````rust
//! Mudda dasa: Vimshottari paylarının bir yıla sığdırılması. Dokuz dönem yılın başında (dönüş
//! anında) başlar ve sonraki dönüşte biter; her dönem yılın `gün(lord) / 360` kadarını alır.
//!
//! İlk dönemin lordu doğum nakshatrasından ve tamamlanan yıldan bulunur: doğum nakshatrasının
//! sırası (Ashwini 1) ile tamamlanan yıl toplanır, 2 çıkarılır ve 9'a bölünür; bölümden artan
//! sayı `MUDDA_ORDER` içindeki sırayı verir (1 Güneş … 8 Ketu, 0 Venüs). Sonraki dönemler aynı
//! sırayla, çevrimsel olarak yürür. İlk dönem tamdır: Ay'ın nakshatrada geçtiği yay için kesinti
//! yapılmaz.
//!
//! Alt dönemler (ikinci seviye) üst dönemin lordundan başlar ve üst dönemi aynı paylarla böler.

use serde::{Deserialize, Serialize};

use crate::dasha::DashaPeriod;
use crate::graha::Graha;
use crate::CoreError;

/// Mudda dasa lordlarının sırası: Vimshottari sırasının Güneş'ten başlayan yazımı.
pub const MUDDA_ORDER: [Graha; 9] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Rahu,
    Graha::Jupiter,
    Graha::Saturn,
    Graha::Mercury,
    Graha::Ketu,
    Graha::Venus,
];

/// Dokuz dönemin gün toplamı: payların paydası.
pub const MUDDA_TOTAL_DAYS: f64 = 360.0;
/// Raporun taşıdığı seviye sayısı: dönem ve alt dönem.
pub const MUDDA_LEVELS: u8 = 2;
/// İlk lord kuralında toplamdan çıkarılan sayı.
pub const MUDDA_OFFSET: u32 = 2;

/// Lordun 360 günlük yıldaki günü (Vimshottari yılının üç katı).
pub fn mudda_days(lord: Graha) -> f64 {
    match lord {
        Graha::Sun => 18.0,
        Graha::Moon => 30.0,
        Graha::Mars => 21.0,
        Graha::Rahu => 54.0,
        Graha::Jupiter => 48.0,
        Graha::Saturn => 57.0,
        Graha::Mercury => 51.0,
        Graha::Ketu => 21.0,
        Graha::Venus => 60.0,
    }
}

fn order_index(lord: Graha) -> usize {
    match lord {
        Graha::Sun => 0,
        Graha::Moon => 1,
        Graha::Mars => 2,
        Graha::Rahu => 3,
        Graha::Jupiter => 4,
        Graha::Saturn => 5,
        Graha::Mercury => 6,
        Graha::Ketu => 7,
        Graha::Venus => 8,
    }
}

/// Bölümden artan: `(doğum nakshatrası + tamamlanan yıl − 2)`'nin 9'a bölümünden artan, 0..=8.
/// `birth_nakshatra` 1..=27 olmalıdır (çağıran denetler).
pub fn mudda_remainder(birth_nakshatra: u8, age: u8) -> u8 {
    // En küçük toplam 1 + 0 = 1'dir; 9 eklemek çıkarmayı eksiye düşürmez ve artanı değiştirmez.
    ((u32::from(birth_nakshatra) + u32::from(age) + 9 - MUDDA_OFFSET) % 9) as u8
}

/// Yılın ilk mudda dasa lordu: artan 1 Güneş, 2 Ay, 3 Mars, 4 Rahu, 5 Jüpiter, 6 Satürn,
/// 7 Merkür, 8 Ketu, 0 Venüs.
pub fn mudda_first_lord(birth_nakshatra: u8, age: u8) -> Graha {
    let remainder = usize::from(mudda_remainder(birth_nakshatra, age));
    MUDDA_ORDER[(remainder + 8) % 9]
}

/// Bir yılın mudda dasası.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct MuddaDasha {
    /// Doğum nakshatrası (Ay), 1..=27.
    pub birth_nakshatra: u8,
    /// Tamamlanan yıl.
    pub age: u8,
    /// İlk lord kuralının bölümden artanı, 0..=8.
    pub remainder: u8,
    pub first_lord: Graha,
    /// Dönemler ve alt dönemleri, ön-sırada (her dönemin ardından dokuz alt dönemi): 9 + 81.
    /// `lords` kökten yoldur; zamanlar UT Jülyen günüdür.
    pub periods: Vec<DashaPeriod>,
}

/// `[start, end)` aralığını `first`'ten başlayan dokuz döneme böler. Bitişler birikimli günden
/// hesaplanır; sonuncusu `end`'e tam eşittir.
fn subdivide(path: &[Graha], first: Graha, start: f64, end: f64) -> Vec<DashaPeriod> {
    let level = (path.len() + 1) as u8;
    let span = end - start;
    let first = order_index(first);
    let mut periods = Vec::with_capacity(9);
    let mut from = start;
    let mut days = 0.0;
    for k in 0..9 {
        let lord = MUDDA_ORDER[(first + k) % 9];
        days += mudda_days(lord);
        let to = if k == 8 {
            end
        } else {
            start + span * days / MUDDA_TOTAL_DAYS
        };
        let mut lords = path.to_vec();
        lords.push(lord);
        periods.push(DashaPeriod {
            lords,
            level,
            start_jd_ut: from,
            end_jd_ut: to,
            has_children: level < MUDDA_LEVELS,
        });
        from = to;
    }
    periods
}

/// `[start_jd_ut, end_jd_ut)` yılının mudda dasası: dokuz dönem ve seksen bir alt dönem.
/// `birth_nakshatra` 1..=27 değilse `InvalidDashaInput("nakshatra")`; uçlar sonlu ve sıralı
/// değilse `InvalidDashaInput("range")`.
pub fn mudda_dasha(
    birth_nakshatra: u8,
    age: u8,
    start_jd_ut: f64,
    end_jd_ut: f64,
) -> Result<MuddaDasha, CoreError> {
    if !(1..=27).contains(&birth_nakshatra) {
        return Err(CoreError::InvalidDashaInput("nakshatra"));
    }
    // NaN karşılaştırması yanlıştır: sonlu olmayan uçlar da burada düşer.
    if !(start_jd_ut.is_finite() && end_jd_ut.is_finite() && start_jd_ut < end_jd_ut) {
        return Err(CoreError::InvalidDashaInput("range"));
    }
    let first_lord = mudda_first_lord(birth_nakshatra, age);
    let mut periods = Vec::with_capacity(9 + 81);
    for period in subdivide(&[], first_lord, start_jd_ut, end_jd_ut) {
        let children = subdivide(
            &period.lords,
            first_lord_of(&period),
            period.start_jd_ut,
            period.end_jd_ut,
        );
        periods.push(period);
        periods.extend(children);
    }
    Ok(MuddaDasha {
        birth_nakshatra,
        age,
        remainder: mudda_remainder(birth_nakshatra, age),
        first_lord,
        periods,
    })
}

/// Alt dönemler üst dönemin lordundan başlar.
fn first_lord_of(period: &DashaPeriod) -> Graha {
    *period.lords.last().expect("a period has a lord")
}
````

- [ ] **Step 4: Testler ve kapılar**

```bash
cargo fmt --all
cargo test -p jyotish-core --lib varsha 2>&1 | grep -E "^test result"
cargo test -p jyotish-core 2>&1 | grep -E "^test result"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
```
Expected: `12 passed`; lib `478 passed`, `avastha_golden` `5 passed`, golden `8 passed`, `modules_golden` `17 passed`; fmt ve clippy çıktısız; sayaç `0`. Bir el örneği tutmazsa önce yorumdaki türetme ve D9 yeniden okunur; beklenti motor çıktısına bakarak değiştirilmez.

- [ ] **Step 5: Commit**

```bash
git add crates/jyotish-core/src/varsha
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): mudda dasa with the first-lord rule and two levels of periods" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: iki dosya (`2 files changed, 357 insertions(+)`); tarama çıktısız; `git status --short` boş.

---

### Task 3: Çekirdek: muntha, yılın lagna olguları, Varshaphala raporu ve toplayıcı

Yol haritası §4.14 ("Muntha", "Dönüş anı: yer") ve §5 Plan 27 görev (2) ile (3); K1–K5; D6–D8, D10, D11. `chart.rs` yalnızca okunur (`compute_chart`, `compute_chart_for`, `check_moment`, `Chart`, `ChartWarning`); `dignity.rs`'ten `sign_lord` okunur.

**Files:**
- Create: `crates/jyotish-core/src/varsha/muntha.rs`, `crates/jyotish-core/src/varsha/collect.rs`
- Modify: `crates/jyotish-core/src/varsha/mod.rs`
- Test: `muntha.rs` `mod tests` (3), `collect.rs` `mod tests` (6)

**Interfaces:**
- Consumes (hepsi var): `crate::chart::{check_moment, compute_chart, compute_chart_for, Chart, ChartWarning}`; `crate::dignity::sign_lord`; `crate::positions::ChartInput`; `crate::zodiac::Sign` (`from_index`, `index`, `house_from`); Task 1'in `varsha_span` ve `completed_years_at`'i; Task 2'nin `mudda_dasha` ve `MuddaDasha`'sı.
- Produces (`jyotish_core::varsha`):
  - `muntha_sign(natal_lagna: Sign, age: u8) -> Sign`.
  - `Muntha { sign: Sign, lord: Graha, natal_house: u8, annual_house: u8 }`.
  - `YearFacts { lagna_sign: Sign, lagna_lord: Graha, natal_lagna_sign: Sign, natal_lagna_lord: Graha, muntha: Muntha }`, `year_facts(natal_lagna: Sign, annual_lagna: Sign, age: u8) -> YearFacts`.
  - `VarshaYear { Age(u8), At(f64) }` (IPC tipi değildir).
  - `VarshaReport { age: u8, start_jd_ut: f64, end_jd_ut: f64, time_known: bool, chart: Option<Chart>, facts: Option<YearFacts>, mudda: MuddaDasha, warnings: Vec<ChartWarning>, method: String }`.
  - `compute_varsha<E: Ephemeris + ?Sized>(eph: &mut E, input: &ChartInput, year: VarshaYear, place: Option<(f64, f64)>) -> Result<VarshaReport, CoreError>`: `place` `(enlem, boylam)`; `None` doğum yeridir.

- [ ] **Step 1: Testler: muntha ve ikinci yazım, yılın olguları; rapor, yer, saat bilinmiyor, süren yıl, tropikal kip, sayı prashnası; modül kaydı**

Muntha testleri: altı el örneği (türetmeler yorumlarda), burç burç yürüyen sayaçla ikinci yazım (12 lagna × 121 yaş), lordlar ve iki ev. Toplayıcı testleri sahte ephemeris kullanır: Güneş tam bir sidereal yılda döner, lagna günde bir tur atar (yıllık lagna doğum lagnasından ayrışsın diye); beklenen burçların türetmesi yorumlardadır.

````diff
diff --git a/crates/jyotish-core/src/varsha/mod.rs b/crates/jyotish-core/src/varsha/mod.rs
index d96f18a..9726ccf 100644
--- a/crates/jyotish-core/src/varsha/mod.rs
+++ b/crates/jyotish-core/src/varsha/mod.rs
@@ -9,13 +9,17 @@
 //! muntha rasi çerçevesindedir (K5). Yorum ve hüküm yoktur: modül yalnızca anı, haritayı, burcu ve
 //! dönemleri döner. Adlar `jyotish_core` kökünden yeniden dışa aktarılmaz.
 
+mod collect;
 mod mudda;
+mod muntha;
 mod solar_return;
 
+pub use collect::{compute_varsha, VarshaReport, VarshaYear};
 pub use mudda::{
     mudda_dasha, mudda_days, mudda_first_lord, mudda_remainder, MuddaDasha, MUDDA_LEVELS,
     MUDDA_OFFSET, MUDDA_ORDER, MUDDA_TOTAL_DAYS,
 };
+pub use muntha::{muntha_sign, year_facts, Muntha, YearFacts};
 pub use solar_return::{
     completed_years_at, solar_return_jd, varsha_span, RETURN_BRACKET_DAYS, RETURN_RESOLUTION_DAYS,
     SIDEREAL_YEAR_DAYS,
````

**Yeni dosya (şimdilik yalnızca test modülü):** `crates/jyotish-core/src/varsha/muntha.rs`

````rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn muntha_starts_in_the_lagna_sign_and_moves_one_sign_a_year() {
        assert_eq!(muntha_sign(Sign::Taurus, 0), Sign::Taurus);
        assert_eq!(muntha_sign(Sign::Taurus, 1), Sign::Gemini);
        // Boğa'dan 11 burç sonrası Koç'tur; 12. yılda başa döner.
        assert_eq!(muntha_sign(Sign::Taurus, 11), Sign::Aries);
        assert_eq!(muntha_sign(Sign::Taurus, 12), Sign::Taurus);
        // Balık lagnası, yıl 26: 26 = 2 × 12 + 2 → iki burç ötesi Boğa.
        assert_eq!(muntha_sign(Sign::Pisces, 26), Sign::Taurus);
        // En büyük yaş: 120 = 10 × 12 → lagna burcu.
        assert_eq!(muntha_sign(Sign::Leo, 120), Sign::Leo);
    }

    #[test]
    fn second_formulation_walking_sign_by_sign() {
        // Her yıl bir burç yürüyen sayaç, kapalı yazımla her lagna ve her yaşta aynıdır.
        for lagna in Sign::ALL {
            let mut walk = lagna;
            for age in 0..=120_u8 {
                assert_eq!(muntha_sign(lagna, age), walk, "{lagna:?} age {age}");
                assert_eq!(
                    muntha_sign(lagna, age).house_from(lagna),
                    age % 12 + 1,
                    "{lagna:?} age {age}: natal house"
                );
                walk = Sign::from_index(walk.index() + 1);
            }
        }
    }

    #[test]
    fn year_facts_give_the_lords_and_both_houses() {
        // Doğum lagnası Boğa, yıl 3: muntha Aslan (lordu Güneş), doğum lagnasından 4. ev.
        // Yıllık lagna Kova: Aslan, Kova'dan 7. evdir. Kova'nın lordu Satürn, Boğa'nınki Venüs.
        let f = year_facts(Sign::Taurus, Sign::Aquarius, 3);
        assert_eq!(
            f,
            YearFacts {
                lagna_sign: Sign::Aquarius,
                lagna_lord: Graha::Saturn,
                natal_lagna_sign: Sign::Taurus,
                natal_lagna_lord: Graha::Venus,
                muntha: Muntha {
                    sign: Sign::Leo,
                    lord: Graha::Sun,
                    natal_house: 4,
                    annual_house: 7,
                },
            }
        );
        // Yıl 0, yıllık lagna doğum lagnası: muntha iki haritada da 1. evdedir.
        let birth = year_facts(Sign::Scorpio, Sign::Scorpio, 0);
        assert_eq!(
            (
                birth.muntha.sign,
                birth.muntha.natal_house,
                birth.muntha.annual_house
            ),
            (Sign::Scorpio, 1, 1)
        );
        assert_eq!(birth.muntha.lord, Graha::Mars);
    }
}
````

**Yeni dosya (şimdilik yalnızca test modülü):** `crates/jyotish-core/src/varsha/collect.rs`

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::settings::{CalcSettings, ZodiacMode};
    use crate::varsha::{solar_return_jd, MAX_VARSHA_AGE, SIDEREAL_YEAR_DAYS};
    use crate::zodiac::{angular_distance, Sign};
    use ephemeris::{Body, Precision};

    const JD: f64 = 2_451_545.0;
    const ARCSEC: f64 = 1.0 / 3600.0;

    /// Güneş tam bir sidereal yılda döner; lagna günde bir tur atar (yıllık lagna doğum
    /// lagnasından ayrışsın diye).
    fn eph() -> FakeEph {
        FakeEph {
            asc_speed: 360.0,
            ..FakeEph::default().with(Body::Sun, 10.0, 360.0 / SIDEREAL_YEAR_DAYS)
        }
    }

    fn input(settings: CalcSettings, time_known: bool) -> ChartInput {
        ChartInput {
            jd_ut: JD,
            lat: 41.0,
            lon: 29.0,
            settings,
            time_known,
            lagna_number: None,
        }
    }

    #[test]
    fn the_report_carries_the_year_the_annual_chart_muntha_and_mudda() {
        let mut eph = eph();
        let i = input(CalcSettings::default(), true);
        let r = compute_varsha(&mut eph, &i, VarshaYear::Age(3), None).unwrap();
        assert_eq!((r.age, r.time_known), (3, true));
        assert_eq!(r.method, VARSHA_METHOD);
        assert!(r.warnings.is_empty());
        // Yıl üçüncü dönüşte başlar ve bir sidereal yıl sürer.
        assert!((r.start_jd_ut - (JD + 3.0 * SIDEREAL_YEAR_DAYS)).abs() < 1e-6);
        assert!((r.end_jd_ut - r.start_jd_ut - SIDEREAL_YEAR_DAYS).abs() < 1e-6);
        // Yıllık harita dönüş anının ve doğum yerinin haritasıdır; Güneş doğum boylamındadır.
        let natal = compute_chart_for(&mut eph, &i).unwrap();
        let chart = r.chart.as_ref().expect("annual chart");
        assert_eq!(
            (chart.jd_ut, chart.geo_lat, chart.geo_lon),
            (r.start_jd_ut, 41.0, 29.0)
        );
        assert_eq!(chart.meta.settings, i.settings);
        assert!(
            angular_distance(chart.grahas[0].sidereal_lon, natal.grahas[0].sidereal_lon) < ARCSEC
        );
        // Doğum lagnası sidereal 46° Boğa; yıl 3: muntha Aslan. Yıllık lagna: 70° + 360° ×
        // 0,769089 gün = 346,87° tropikal, 322,87° sidereal: Kova. Aslan, Kova'dan 7. evdir.
        let facts = r.facts.expect("facts");
        assert_eq!(facts, year_facts(Sign::Taurus, Sign::Aquarius, 3));
        assert_eq!(chart.lagna.sign, Sign::Aquarius);
        assert_eq!(
            (
                facts.muntha.sign,
                facts.muntha.natal_house,
                facts.muntha.annual_house
            ),
            (Sign::Leo, 4, 7)
        );
        // Ay sidereal 71°: 6. nakshatra (Ardra). 6 + 3 − 2 = 7 → Merkür; yıl dokuz dönemle dolar.
        assert_eq!(
            (r.mudda.birth_nakshatra, r.mudda.age, r.mudda.first_lord),
            (6, 3, Graha::Mercury)
        );
        assert_eq!(r.mudda.periods[0].start_jd_ut, r.start_jd_ut);
        assert_eq!(r.mudda.periods.last().unwrap().end_jd_ut, r.end_jd_ut);
    }

    #[test]
    fn the_place_moves_the_annual_chart_only_and_is_validated() {
        let mut eph = eph();
        let i = input(CalcSettings::default(), true);
        let birth = compute_varsha(&mut eph, &i, VarshaYear::Age(3), None).unwrap();
        let moved = compute_varsha(&mut eph, &i, VarshaYear::Age(3), Some((-33.9, 151.2))).unwrap();
        let chart = moved.chart.as_ref().expect("annual chart");
        assert_eq!((chart.geo_lat, chart.geo_lon), (-33.9, 151.2));
        // Dönüş anı ve mudda dasa yere bağlı değildir.
        assert_eq!(
            (moved.start_jd_ut, moved.end_jd_ut),
            (birth.start_jd_ut, birth.end_jd_ut)
        );
        assert_eq!(moved.mudda, birth.mudda);
        for time_known in [true, false] {
            let i = input(CalcSettings::default(), time_known);
            assert_eq!(
                compute_varsha(&mut eph, &i, VarshaYear::Age(3), Some((91.0, 0.0))),
                Err(CoreError::InvalidLocation {
                    lat: 91.0,
                    lon: 0.0
                }),
                "time_known {time_known}"
            );
        }
        assert_eq!(
            compute_varsha(&mut eph, &i, VarshaYear::Age(MAX_VARSHA_AGE + 1), None),
            Err(CoreError::InvalidDashaInput("age"))
        );
    }

    #[test]
    fn without_a_birth_time_there_is_no_annual_chart_and_no_muntha() {
        let mut eph = eph();
        let known = compute_varsha(
            &mut eph,
            &input(CalcSettings::default(), true),
            VarshaYear::Age(3),
            None,
        )
        .unwrap();
        let r = compute_varsha(
            &mut eph,
            &input(CalcSettings::default(), false),
            VarshaYear::Age(3),
            None,
        )
        .unwrap();
        assert!(!r.time_known);
        assert_eq!((&r.chart, r.facts), (&None, None));
        // Yıl ve mudda dasa yine döner (yaklaşık sayılır; işareti arayüz koyar).
        assert_eq!(
            (r.age, r.start_jd_ut, r.end_jd_ut),
            (3, known.start_jd_ut, known.end_jd_ut)
        );
        assert_eq!(r.mudda, known.mudda);
    }

    #[test]
    fn the_running_year_is_found_from_a_moment() {
        let mut eph = eph();
        let i = input(CalcSettings::default(), true);
        let sun = compute_chart_for(&mut eph, &i).unwrap().grahas[0].sidereal_lon;
        let fifth = solar_return_jd(&mut eph, &i.settings, sun, JD, 5).unwrap();
        let mut age_at = |jd: f64| {
            compute_varsha(&mut eph, &i, VarshaYear::At(jd), None)
                .unwrap()
                .age
        };
        assert_eq!(age_at(JD - 10.0), 0, "before birth: the birth year");
        assert_eq!(age_at(fifth - 0.5), 4);
        assert_eq!(age_at(fifth), 5);
        assert_eq!(age_at(fifth + 100.0), 5);
        // `At` ile bulunan yılın raporu, aynı yaşın raporudur.
        let by_moment = compute_varsha(&mut eph, &i, VarshaYear::At(fifth + 100.0), None).unwrap();
        let by_age = compute_varsha(&mut eph, &i, VarshaYear::Age(5), None).unwrap();
        assert_eq!(by_moment, by_age);
    }

    #[test]
    fn tropical_rasi_mode_reads_the_muntha_in_the_rasi_frame_and_the_return_sidereally() {
        // Tropikal lagna 70°: İkizler; yıl 3: muntha Başak. Yıllık lagna tropikal 346,87°: Balık;
        // Başak, Balık'tan 7. evdir. Dönüş anı ve doğum nakshatrası sidereal boylamdandır:
        // sidereal kipin raporuyla aynıdır.
        let mut eph = eph();
        let sidereal = compute_varsha(
            &mut eph,
            &input(CalcSettings::default(), true),
            VarshaYear::Age(3),
            None,
        )
        .unwrap();
        let settings = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..CalcSettings::default()
        };
        let r = compute_varsha(&mut eph, &input(settings, true), VarshaYear::Age(3), None).unwrap();
        assert_eq!(r.facts, Some(year_facts(Sign::Gemini, Sign::Pisces, 3)));
        assert_eq!(r.facts.unwrap().muntha.sign, Sign::Virgo);
        assert_eq!(
            (r.start_jd_ut, r.end_jd_ut),
            (sidereal.start_jd_ut, sidereal.end_jd_ut)
        );
        assert_eq!(r.mudda, sidereal.mudda);
    }

    #[test]
    fn number_prashna_counts_the_muntha_from_the_number_lagna_and_low_precision_is_carried() {
        // Sayı 30 → doğum lagnası Yengeç (`number_lagna_lon`); yıl 3: muntha Terazi. Yıllık harita
        // dönüş anının kendi lagnasıyla hesaplanır (Kova): sayı yıllık haritaya taşınmaz.
        let mut eph = eph();
        let numbered = ChartInput {
            lagna_number: Some(30),
            ..input(CalcSettings::default(), true)
        };
        let r = compute_varsha(&mut eph, &numbered, VarshaYear::Age(3), None).unwrap();
        assert_eq!(r.facts, Some(year_facts(Sign::Cancer, Sign::Aquarius, 3)));
        assert_eq!(r.facts.unwrap().muntha.sign, Sign::Libra);
        assert_eq!(
            compute_varsha(
                &mut eph,
                &ChartInput {
                    lagna_number: Some(0),
                    ..numbered
                },
                VarshaYear::Age(3),
                None
            ),
            Err(CoreError::InvalidPrashnaInput("number"))
        );
        // Moshier yedeği: uyarı rapora taşınır.
        let mut moshier = FakeEph {
            precision: Precision::Moshier,
            ..self::eph()
        };
        let r = compute_varsha(
            &mut moshier,
            &input(CalcSettings::default(), false),
            VarshaYear::Age(1),
            None,
        )
        .unwrap();
        assert_eq!(r.warnings, [ChartWarning::LowPrecision]);
    }
}
````

- [ ] **Step 2: Testlerin derlenmediğini gör**

Run: `cargo test -p jyotish-core --lib varsha 2>&1 | grep -E "^error" | sort | uniq -c | head`
Expected: derleme hatası: `error[E0425]: cannot find function` (`compute_varsha`, `muntha_sign`, `year_facts`) ve `error[E0422]: cannot find struct` (`Muntha`, `YearFacts`, `ChartInput`): henüz yoklar. Test koşmaz.

- [ ] **Step 3: Muntha ve yılın olguları**

**Dosyanın başına eklenir** (test modülünün üstüne, arada bir boş satırla): `crates/jyotish-core/src/varsha/muntha.rs`

````rust
//! Muntha ve yılın lagnaya bağlı olguları. Muntha doğumda lagnanın burcundadır ve tamamlanan her
//! yılda bir burç ilerler: `age` yaşında doğum lagnasının burcundan `age` burç ötededir. Burçlar
//! rasi çerçevesindedir (K5); evler burç evidir (K4).

use serde::{Deserialize, Serialize};

use crate::dignity::sign_lord;
use crate::graha::Graha;
use crate::zodiac::Sign;

/// Munthanın burcu: doğum lagnasının burcu + tamamlanan yıl (on ikide bir başa döner).
pub fn muntha_sign(natal_lagna: Sign, age: u8) -> Sign {
    Sign::from_index(natal_lagna.index() + usize::from(age))
}

/// Yılın munthası.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Muntha {
    pub sign: Sign,
    /// Muntha burcunun lordu.
    pub lord: Graha,
    /// Doğum lagnasından sayılan burç evi, 1..=12.
    pub natal_house: u8,
    /// Yıllık haritanın lagnasından sayılan burç evi, 1..=12.
    pub annual_house: u8,
}

/// Yılın lagnaya bağlı olguları; doğum saati bilinmiyorsa hiçbiri verilmez (K2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct YearFacts {
    /// Yıllık haritanın lagna burcu ve lordu.
    pub lagna_sign: Sign,
    pub lagna_lord: Graha,
    /// Doğum haritasının lagna burcu ve lordu.
    pub natal_lagna_sign: Sign,
    pub natal_lagna_lord: Graha,
    pub muntha: Muntha,
}

/// Doğum lagnası, yıllık lagna ve tamamlanan yıldan yılın olguları.
pub fn year_facts(natal_lagna: Sign, annual_lagna: Sign, age: u8) -> YearFacts {
    let sign = muntha_sign(natal_lagna, age);
    YearFacts {
        lagna_sign: annual_lagna,
        lagna_lord: sign_lord(annual_lagna),
        natal_lagna_sign: natal_lagna,
        natal_lagna_lord: sign_lord(natal_lagna),
        muntha: Muntha {
            sign,
            lord: sign_lord(sign),
            natal_house: sign.house_from(natal_lagna),
            annual_house: sign.house_from(annual_lagna),
        },
    }
}
````

- [ ] **Step 4: Rapor ve toplayıcı**

**Dosyanın başına eklenir** (test modülünün üstüne, arada bir boş satırla): `crates/jyotish-core/src/varsha/collect.rs`

````rust
//! Varshaphala raporu ve toplayıcısı (K1, K3): doğum haritası, dönüş anı, yıllık harita, muntha
//! ve mudda dasa. Rapor istek üzerine hesaplanır ve saklanmaz.

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::chart::{check_moment, compute_chart, compute_chart_for, Chart, ChartWarning};
use crate::graha::Graha;
use crate::positions::ChartInput;
use crate::CoreError;

use super::mudda::{mudda_dasha, MuddaDasha};
use super::muntha::{year_facts, YearFacts};
use super::solar_return::{completed_years_at, varsha_span};
use super::VARSHA_METHOD;

/// İstenen yıl.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VarshaYear {
    /// Tamamlanan yıl: 0 doğum yılıdır, 1 ilk dönüşle başlayan yıldır.
    Age(u8),
    /// Bu anda (UT Jülyen günü) süren yıl.
    At(f64),
}

/// `compute_varsha` sonucu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct VarshaReport {
    /// Tamamlanan yıl: yıl, `age`. dönüşle başlar.
    pub age: u8,
    /// Yılın başı: Güneş'in doğumdaki sidereal boylamına döndüğü an.
    pub start_jd_ut: f64,
    /// Yılın sonu: sonraki dönüş.
    pub end_jd_ut: f64,
    /// Yanlışsa doğum saati bilinmiyordur: yıllık harita ve lagnaya bağlı olgular verilmez;
    /// dönüş anı ve mudda dasa yaklaşıktır (Güneş'in ve Ay'ın doğumdaki yeri gün içinde değişir).
    pub time_known: bool,
    /// Yıllık harita: dönüş anında, istenen yerde, doğum haritasının ayarlarıyla. Doğum saati
    /// bilinmiyorsa `None`.
    pub chart: Option<Chart>,
    /// Yılın lagnaya bağlı olguları (yıllık lagna, doğum lagnası, muntha); doğum saati
    /// bilinmiyorsa `None`.
    pub facts: Option<YearFacts>,
    pub mudda: MuddaDasha,
    /// Doğum haritasının ya da yıllık haritanın konum duyarlılığı uyarısı (`LowPrecision`).
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

/// `compute_varsha` komutunun hesabı. Doğum haritası `compute_chart_for` ile hesaplanır (sayı
/// prashnasında doğum lagnası sayıdan gelir); yıl Güneş'in doğumdaki sidereal boylamına
/// dönüşüyle başlar; yıllık harita dönüş anında ve `place` yerinde (`None`: doğum yeri; `Some`:
/// kuzey + enlem, doğu + boylam) aynı ayarlarla hesaplanır. `input.time_known` yanlışsa yıllık
/// harita ve `facts` boştur. Ayar, an ve konum denetimi `compute_chart`'ınkidir; yaş
/// `InvalidDashaInput("age")`, yer `InvalidLocation` ile reddedilir.
pub fn compute_varsha<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
    year: VarshaYear,
    place: Option<(f64, f64)>,
) -> Result<VarshaReport, CoreError> {
    let natal = compute_chart_for(eph, input)?;
    // Yer, doğum saati bilinmese de denetlenir: geçersiz yer sessizce geçmez.
    let (lat, lon) = place.unwrap_or((input.lat, input.lon));
    check_moment(input.jd_ut, lat, lon)?;
    let sun = natal.grahas[Graha::Sun as usize].sidereal_lon;
    let age = match year {
        VarshaYear::Age(age) => age,
        VarshaYear::At(jd) => completed_years_at(eph, &input.settings, sun, input.jd_ut, jd)?,
    };
    let (start_jd_ut, end_jd_ut) = varsha_span(eph, &input.settings, sun, input.jd_ut, age)?;
    let mudda = mudda_dasha(
        natal.grahas[Graha::Moon as usize].nakshatra,
        age,
        start_jd_ut,
        end_jd_ut,
    )?;
    let low = |chart: &Chart| chart.meta.warnings.contains(&ChartWarning::LowPrecision);
    let mut low_precision = low(&natal);
    let (chart, facts) = if input.time_known {
        let annual = compute_chart(eph, start_jd_ut, lat, lon, &input.settings)?;
        low_precision |= low(&annual);
        let facts = year_facts(natal.lagna.sign, annual.lagna.sign, age);
        (Some(annual), Some(facts))
    } else {
        (None, None)
    };
    Ok(VarshaReport {
        age,
        start_jd_ut,
        end_jd_ut,
        time_known: input.time_known,
        chart,
        facts,
        mudda,
        warnings: if low_precision {
            vec![ChartWarning::LowPrecision]
        } else {
            Vec::new()
        },
        method: VARSHA_METHOD.to_owned(),
    })
}
````

- [ ] **Step 5: Testler ve kapılar**

```bash
cargo fmt --all
cargo test -p jyotish-core --lib varsha 2>&1 | grep -E "^test result"
cargo test -p jyotish-core 2>&1 | grep -E "^test result"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
```
Expected: `21 passed`; lib `487 passed`, `avastha_golden` `5 passed`, golden `8 passed`, `modules_golden` `17 passed`; fmt ve clippy çıktısız; sayaç `0`. Bir toplayıcı testi tutmazsa önce yorumdaki türetme (sahte ephemeris'in lagnası: tropikal 70° + 360° × geçen günün kesri) yeniden yapılır; beklenti motor çıktısına bakarak değiştirilmez.

- [ ] **Step 6: Commit**

```bash
git add crates/jyotish-core/src/varsha
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): muntha, year facts and the varsha report collector" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: üç dosya (`3 files changed, 455 insertions(+)`); tarama çıktısız; `git status --short` boş.

---

### Task 4: El örnekleri: muntha ve mudda dasa

Yol haritası §5 Plan 27 görev (5); ileri modüller spec'i §13, K9; D12. On dört dosya ve onları yükleyen ayrı bir test dosyası. Örnekler kural metninden türetilmiştir (`derivation`); motor çıktısı kopyalanmaz. `modules_golden.rs` ve ortak girdi biçimi (`lagna_lon`, `grahas`) kullanılmaz: bu örneklerin girdisi iki lagna boylamı ya da Ay'ın boylamıdır.

**Files:**
- Create: `crates/jyotish-core/tests/varsha_golden.rs`; `tests/golden/modules/varsha/muntha/` altında beş, `tests/golden/modules/varsha/mudda/` altında dokuz JSON dosyası
- Test: `varsha_golden.rs` (3)

**Interfaces:**
- Consumes: Task 2 ve Task 3'ün `jyotish_core::varsha::{mudda_dasha, muntha_sign, year_facts, MuddaDasha, Muntha, YearFacts, MUDDA_ORDER}`'ı; kökten `jyotish_core::{nakshatra_pada, DashaPeriod, Graha, Sign}` (hepsi var).
- Produces: `muntha/` örnek biçimi: `input { natal_lagna_lon, annual_lagna_lon, age }`, `expected { sign, lord, natal_house, annual_house, lagna_sign, lagna_lord, natal_lagna_sign, natal_lagna_lord }`. `mudda/` örnek biçimi: `input { moon_lon, age, year_days }`, `expected { birth_nakshatra, remainder, first_lord, periods: [[lord, bitiş günü] × 9], first_sub_periods: aynı biçim ya da null }`. Bitişler yılın başından gün sayısıdır, altı ondalığa yuvarlanmıştır; tolerans 1e-6 gündür.

- [ ] **Step 1: Test dosyası**

Üç test: muntha örnekleri (motorla ve örneğin kendi aritmetiğiyle), mudda örnekleri (aynı biçimde) ve kapsam (dokuz lordun her biri bir örnekte yılı açar; dokuz artan değerinin hepsi; toplamın eksiye düştüğü durum; 360 günlük ve dönüşten dönüşe yıl; en az üç örnekte alt dönemler; doğum yılı, tam on iki yıl, Balık'tan Koç'a dönen sayım ve yüzden büyük yaş).

**Yeni dosya:** `crates/jyotish-core/tests/varsha_golden.rs`

````rust
//! Varsha el örnekleri (yol haritası §4.14; ileri modüller spec'i §13): beklenen değerler
//! kuraldan elle türetilir (`derivation`), başka yazılımın çıktısı olamaz (K9). İki alt klasör:
//! `tests/golden/modules/varsha/muntha/` (muntha ve lagna lordları) ve
//! `tests/golden/modules/varsha/mudda/` (ilk lord kuralı ve dönem bitişleri). Dönüş anı bir
//! tablo değildir: gerçek motorla `app/src-tauri/tests/varsha.rs`'te denetlenir. Okuma kuralları
//! ve onay bekleyen varyant noktaları `varsha/REVIEW.md`'dedir.

use std::path::PathBuf;

use jyotish_core::varsha::{
    mudda_dasha, muntha_sign, year_facts, MuddaDasha, Muntha, YearFacts, MUDDA_ORDER,
};
use jyotish_core::{nakshatra_pada, DashaPeriod, Graha, Sign};
use serde::de::DeserializeOwned;
use serde::Deserialize;

/// Örnek silmek testi düşürür.
const MIN_MUNTHA: usize = 5;
const MIN_MUDDA: usize = 9;
/// Örneklerdeki gün değerleri altı ondalığa yuvarlanmıştır.
const DAY_TOLERANCE: f64 = 1e-6;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture<X, I> {
    id: String,
    source: String,
    rule_ref: String,
    derivation: Vec<String>,
    input: I,
    expected: X,
}

/// Bir dizinin el örnekleri, ortak denetimleriyle: en az sayı, dosya adı, kaynak, gönderme ve
/// türetme adımları.
fn load<X: DeserializeOwned, I: DeserializeOwned>(series: &str, min: usize) -> Vec<Fixture<X, I>> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/golden/modules/varsha")
        .join(series);
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|e| e.expect("dir entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    assert!(
        paths.len() >= min,
        "{series}: {} hand fixtures, at least {min} required",
        paths.len()
    );
    paths
        .into_iter()
        .map(|p| {
            let text =
                std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            let f: Fixture<X, I> =
                serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            assert_eq!(
                p.file_stem().and_then(|s| s.to_str()),
                Some(f.id.as_str()),
                "id must equal the file name"
            );
            // Başka yazılımın çıktısı fixture olamaz (K9).
            assert_eq!(f.source, "hand", "{}: only hand-derived fixtures", f.id);
            assert!(!f.rule_ref.trim().is_empty(), "{}: rule_ref", f.id);
            assert!(
                !f.derivation.is_empty() && f.derivation.iter().all(|s| !s.trim().is_empty()),
                "{}: derivation steps",
                f.id
            );
            f
        })
        .collect()
}

/// Sidereal sayılan iki lagna boylamı (derece) ve tamamlanan yıl.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MunthaInput {
    natal_lagna_lon: f64,
    annual_lagna_lon: f64,
    age: u8,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MunthaExpected {
    sign: Sign,
    lord: Graha,
    natal_house: u8,
    annual_house: u8,
    lagna_sign: Sign,
    lagna_lord: Graha,
    natal_lagna_sign: Sign,
    natal_lagna_lord: Graha,
}

/// Ay'ın sidereal boylamı (derece), tamamlanan yıl ve yılın uzunluğu (gün). Yıl 0. günde
/// başlar: beklenen bitişler yılın başından gün sayısıdır.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MuddaInput {
    moon_lon: f64,
    age: u8,
    year_days: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MuddaExpected {
    birth_nakshatra: u8,
    remainder: u8,
    first_lord: Graha,
    /// Dokuz dönem: lord ve bitiş günü.
    periods: Vec<(Graha, f64)>,
    /// İlk dönemin dokuz alt dönemi: alt lord ve bitiş günü; yazılmadıysa `null`.
    first_sub_periods: Option<Vec<(Graha, f64)>>,
}

fn muntha_fixtures() -> Vec<Fixture<MunthaExpected, MunthaInput>> {
    load("muntha", MIN_MUNTHA)
}

fn mudda_fixtures() -> Vec<Fixture<MuddaExpected, MuddaInput>> {
    load("mudda", MIN_MUDDA)
}

/// Dönemlerin son lordu ve bitişi, sırayla.
fn ends(periods: &[&DashaPeriod]) -> Vec<(Graha, f64)> {
    periods
        .iter()
        .map(|p| (*p.lords.last().expect("a lord"), p.end_jd_ut))
        .collect()
}

fn assert_ends(id: &str, what: &str, got: &[(Graha, f64)], want: &[(Graha, f64)]) {
    assert_eq!(got.len(), 9, "{id}: {what} has nine periods");
    assert_eq!(want.len(), 9, "{id}: {what} lists nine periods");
    for (k, (g, w)) in got.iter().zip(want).enumerate() {
        assert_eq!(g.0, w.0, "{id}: {what} lord {k}");
        assert!(
            (g.1 - w.1).abs() <= DAY_TOLERANCE,
            "{id}: {what} {:?} ends at {} (expected {})",
            w.0,
            g.1,
            w.1
        );
    }
}

#[test]
fn muntha_hand_fixtures() {
    for f in muntha_fixtures() {
        let (i, x) = (&f.input, &f.expected);
        let (natal, annual) = (Sign::of(i.natal_lagna_lon), Sign::of(i.annual_lagna_lon));
        assert_eq!(muntha_sign(natal, i.age), x.sign, "{}", f.id);
        assert_eq!(
            year_facts(natal, annual, i.age),
            YearFacts {
                lagna_sign: x.lagna_sign,
                lagna_lord: x.lagna_lord,
                natal_lagna_sign: x.natal_lagna_sign,
                natal_lagna_lord: x.natal_lagna_lord,
                muntha: Muntha {
                    sign: x.sign,
                    lord: x.lord,
                    natal_house: x.natal_house,
                    annual_house: x.annual_house,
                },
            },
            "{}",
            f.id
        );
        // Örneğin kendi aritmetiği, motordan bağımsız: doğum lagnasından sayılan ev, tamamlanan
        // yılın 12'ye bölümünden artanın bir fazlasıdır.
        assert_eq!(x.natal_house, i.age % 12 + 1, "{}: natal house", f.id);
        assert_eq!(x.sign.house_from(natal), x.natal_house, "{}", f.id);
    }
}

#[test]
fn mudda_hand_fixtures() {
    for f in mudda_fixtures() {
        let (i, x) = (&f.input, &f.expected);
        let nakshatra = nakshatra_pada(i.moon_lon).0;
        assert_eq!(nakshatra, x.birth_nakshatra, "{}: birth nakshatra", f.id);
        let d: MuddaDasha = mudda_dasha(nakshatra, i.age, 0.0, i.year_days)
            .unwrap_or_else(|e| panic!("{}: {e}", f.id));
        assert_eq!(
            (d.remainder, d.first_lord),
            (x.remainder, x.first_lord),
            "{}",
            f.id
        );
        let top: Vec<&DashaPeriod> = d.periods.iter().filter(|p| p.level == 1).collect();
        assert_ends(&f.id, "year", &ends(&top), &x.periods);
        if let Some(subs) = &x.first_sub_periods {
            let kids: Vec<&DashaPeriod> = d.periods[1..10].iter().collect();
            assert!(kids.iter().all(|k| k.level == 2), "{}", f.id);
            assert_ends(&f.id, "first period", &ends(&kids), subs);
            // Alt dönemler üst dönemin lorduyla başlar ve üst dönemin bitişinde biter.
            assert_eq!(subs[0].0, x.first_lord, "{}", f.id);
            assert!(
                (subs[8].1 - x.periods[0].1).abs() <= DAY_TOLERANCE,
                "{}",
                f.id
            );
        }
        // Örneğin kendi aritmetiği, motordan bağımsız: artan, toplamın 9'a bölümünden artandır;
        // ilk lord sıradaki o sayıdır (0 dokuzuncudur); dönemler sırayı çevrimsel izler ve son
        // dönem yılın sonunda biter.
        let total = i32::from(x.birth_nakshatra) + i32::from(i.age) - 2;
        assert_eq!(i32::from(x.remainder), total.rem_euclid(9), "{}", f.id);
        let first = (usize::from(x.remainder) + 8) % 9;
        assert_eq!(x.first_lord, MUDDA_ORDER[first], "{}", f.id);
        for (k, (lord, _)) in x.periods.iter().enumerate() {
            assert_eq!(*lord, MUDDA_ORDER[(first + k) % 9], "{}: order {k}", f.id);
        }
        assert!(
            (x.periods[8].1 - i.year_days).abs() <= DAY_TOLERANCE,
            "{}: the last period closes the year",
            f.id
        );
    }
}

#[test]
fn fixtures_cover_the_required_cases() {
    let mudda = mudda_fixtures();
    // Dokuz lordun her biri bir örnekte yılı açar; dokuz artan değerinin hepsi görülür.
    for (k, lord) in MUDDA_ORDER.into_iter().enumerate() {
        assert!(
            mudda.iter().any(|f| f.expected.first_lord == lord),
            "{lord:?} opens no fixture year"
        );
        assert!(
            mudda
                .iter()
                .any(|f| usize::from(f.expected.remainder) == (k + 1) % 9),
            "remainder {} is in no fixture",
            (k + 1) % 9
        );
    }
    // Toplamın eksiye düştüğü durum (Ashwini, doğum yılı), 360 günlük yıl, dönüşten dönüşe yıl
    // ve alt dönemler.
    assert!(mudda
        .iter()
        .any(|f| f.expected.birth_nakshatra == 1 && f.input.age == 0));
    assert!(mudda.iter().any(|f| f.input.year_days == 360.0));
    assert!(mudda.iter().any(|f| f.input.year_days != 360.0));
    assert!(
        mudda
            .iter()
            .filter(|f| f.expected.first_sub_periods.is_some())
            .count()
            >= 3
    );
    let muntha = muntha_fixtures();
    // Doğum yılı, tam on iki yıl, Balık'tan Koç'a dönen sayım ve on ikiden büyük yaş.
    assert!(muntha.iter().any(|f| f.input.age == 0));
    assert!(muntha
        .iter()
        .any(|f| f.input.age > 0 && f.input.age % 12 == 0));
    assert!(muntha
        .iter()
        .any(|f| f.expected.sign.index() < f.expected.natal_lagna_sign.index()));
    assert!(muntha.iter().any(|f| f.input.age > 100));
}
````

- [ ] **Step 2: Testlerin düştüğünü gör**

Run: `cargo test -p jyotish-core --test varsha_golden 2>&1 | grep -E "^test |^test result"`
Expected: üç test de `FAILED` (`cannot read …/tests/golden/modules/varsha/muntha` ya da `…/mudda`: örnek klasörleri henüz yok); `test result: FAILED. 0 passed; 3 failed`.

- [ ] **Step 3: On dört el örneği**

Beş muntha örneği:

**Yeni dosya:** `tests/golden/modules/varsha/muntha/birth-year-muntha-in-the-lagna.json`

````json
{
  "id": "birth-year-muntha-in-the-lagna",
  "source": "hand",
  "rule_ref": "Tajika yıllık harita: muntha (alıntı yapılmaz); okuma kuralları varsha/REVIEW.md",
  "derivation": [
    "Doğum yılı (tamamlanan yıl 0): yıllık harita doğum haritasıdır ve muntha lagnadadır.",
    "Doğum lagnası 215,25° ÷ 30° → 8. burç (Akrep). Yıllık lagna 215,25° ÷ 30° → 8. burç (Akrep).",
    "Tamamlanan yıl 0 = 0 × 12 + 0: muntha Akrep burcundan 0 burç ötededir → Akrep.",
    "Muntha lordu: Akrep burcunun lordu Mars.",
    "Evler: Akrep, Akrep burcundan sayınca 1. ev; Akrep burcundan sayınca 1. ev.",
    "Lagna lordları: yıllık lagna Akrep → Mars; doğum lagnası Akrep → Mars."
  ],
  "input": {"natal_lagna_lon": 215.25, "annual_lagna_lon": 215.25, "age": 0},
  "expected": {
    "sign": "Scorpio",
    "lord": "Mars",
    "natal_house": 1,
    "annual_house": 1,
    "lagna_sign": "Scorpio",
    "lagna_lord": "Mars",
    "natal_lagna_sign": "Scorpio",
    "natal_lagna_lord": "Mars"
  }
}
````

**Yeni dosya:** `tests/golden/modules/varsha/muntha/oldest-age-and-zero-degree-lagna.json`

````json
{
  "id": "oldest-age-and-zero-degree-lagna",
  "source": "hand",
  "rule_ref": "Tajika yıllık harita: muntha (alıntı yapılmaz); okuma kuralları varsha/REVIEW.md",
  "derivation": [
    "Lagna 0° Koç'un başıdır; 119 = 9 × 12 + 11: muntha lagnanın bir önceki burcundadır.",
    "Doğum lagnası 0° ÷ 30° → 1. burç (Koç). Yıllık lagna 359,5° ÷ 30° → 12. burç (Balık).",
    "Tamamlanan yıl 119 = 9 × 12 + 11: muntha Koç burcundan 11 burç ötededir → Balık.",
    "Muntha lordu: Balık burcunun lordu Jüpiter.",
    "Evler: Balık, Koç burcundan sayınca 12. ev; Balık burcundan sayınca 1. ev.",
    "Lagna lordları: yıllık lagna Balık → Jüpiter; doğum lagnası Koç → Mars."
  ],
  "input": {"natal_lagna_lon": 0.0, "annual_lagna_lon": 359.5, "age": 119},
  "expected": {
    "sign": "Pisces",
    "lord": "Jupiter",
    "natal_house": 12,
    "annual_house": 1,
    "lagna_sign": "Pisces",
    "lagna_lord": "Jupiter",
    "natal_lagna_sign": "Aries",
    "natal_lagna_lord": "Mars"
  }
}
````

**Yeni dosya:** `tests/golden/modules/varsha/muntha/pisces-lagna-wraps-past-aries.json`

````json
{
  "id": "pisces-lagna-wraps-past-aries",
  "source": "hand",
  "rule_ref": "Tajika yıllık harita: muntha (alıntı yapılmaz); okuma kuralları varsha/REVIEW.md",
  "derivation": [
    "Sayım Balık'tan Koç'a döner: 26 yıl iki tam tur ve iki burçtur.",
    "Doğum lagnası 345,5° ÷ 30° → 12. burç (Balık). Yıllık lagna 100° ÷ 30° → 4. burç (Yengeç).",
    "Tamamlanan yıl 26 = 2 × 12 + 2: muntha Balık burcundan 2 burç ötededir → Boğa.",
    "Muntha lordu: Boğa burcunun lordu Venüs.",
    "Evler: Boğa, Balık burcundan sayınca 3. ev; Yengeç burcundan sayınca 11. ev.",
    "Lagna lordları: yıllık lagna Yengeç → Ay; doğum lagnası Balık → Jüpiter."
  ],
  "input": {"natal_lagna_lon": 345.5, "annual_lagna_lon": 100.0, "age": 26},
  "expected": {
    "sign": "Taurus",
    "lord": "Venus",
    "natal_house": 3,
    "annual_house": 11,
    "lagna_sign": "Cancer",
    "lagna_lord": "Moon",
    "natal_lagna_sign": "Pisces",
    "natal_lagna_lord": "Jupiter"
  }
}
````

**Yeni dosya:** `tests/golden/modules/varsha/muntha/taurus-lagna-third-year-leo.json`

````json
{
  "id": "taurus-lagna-third-year-leo",
  "source": "hand",
  "rule_ref": "Tajika yıllık harita: muntha (alıntı yapılmaz); okuma kuralları varsha/REVIEW.md",
  "derivation": [
    "Doğum lagnası Boğa, tamamlanan yıl 3; yıllık lagna Kova.",
    "Doğum lagnası 46° ÷ 30° → 2. burç (Boğa). Yıllık lagna 322,87° ÷ 30° → 11. burç (Kova).",
    "Tamamlanan yıl 3 = 0 × 12 + 3: muntha Boğa burcundan 3 burç ötededir → Aslan.",
    "Muntha lordu: Aslan burcunun lordu Güneş.",
    "Evler: Aslan, Boğa burcundan sayınca 4. ev; Kova burcundan sayınca 7. ev.",
    "Lagna lordları: yıllık lagna Kova → Satürn; doğum lagnası Boğa → Venüs."
  ],
  "input": {"natal_lagna_lon": 46.0, "annual_lagna_lon": 322.87, "age": 3},
  "expected": {
    "sign": "Leo",
    "lord": "Sun",
    "natal_house": 4,
    "annual_house": 7,
    "lagna_sign": "Aquarius",
    "lagna_lord": "Saturn",
    "natal_lagna_sign": "Taurus",
    "natal_lagna_lord": "Venus"
  }
}
````

**Yeni dosya:** `tests/golden/modules/varsha/muntha/twelfth-return-back-to-the-lagna.json`

````json
{
  "id": "twelfth-return-back-to-the-lagna",
  "source": "hand",
  "rule_ref": "Tajika yıllık harita: muntha (alıntı yapılmaz); okuma kuralları varsha/REVIEW.md",
  "derivation": [
    "On iki yılda muntha doğum lagnasının burcuna döner. Lagna 29,999° hâlâ Koç'tur; yıllık lagna 180° Terazi'nin başıdır.",
    "Doğum lagnası 29,999° ÷ 30° → 1. burç (Koç). Yıllık lagna 180° ÷ 30° → 7. burç (Terazi).",
    "Tamamlanan yıl 12 = 1 × 12 + 0: muntha Koç burcundan 0 burç ötededir → Koç.",
    "Muntha lordu: Koç burcunun lordu Mars.",
    "Evler: Koç, Koç burcundan sayınca 1. ev; Terazi burcundan sayınca 7. ev.",
    "Lagna lordları: yıllık lagna Terazi → Venüs; doğum lagnası Koç → Mars."
  ],
  "input": {"natal_lagna_lon": 29.999, "annual_lagna_lon": 180.0, "age": 12},
  "expected": {
    "sign": "Aries",
    "lord": "Mars",
    "natal_house": 1,
    "annual_house": 7,
    "lagna_sign": "Libra",
    "lagna_lord": "Venus",
    "natal_lagna_sign": "Aries",
    "natal_lagna_lord": "Mars"
  }
}
````

Dokuz mudda örneği:

**Yeni dosya:** `tests/golden/modules/varsha/mudda/ardra-age-9-rahu-first.json`

````json
{
  "id": "ardra-age-9-rahu-first",
  "source": "hand",
  "rule_ref": "Tajika yıllık harita: mudda dasa (alıntı yapılmaz); okuma kuralları ve tablolar varsha/REVIEW.md",
  "derivation": [
    "Dokuz yılda sıra başa döner: ilk lord doğum yılındakiyle aynıdır (Ardra: Rahu).",
    "Doğum nakshatrası: Ay 71° ÷ 13°20′ → 6. nakshatra (Ardra).",
    "İlk lord: 6 + 9 − 2 = 13 = 1 × 9 + 4: artan 4 → 4. lord Rahu.",
    "Yıl 365,256363 gün; her dönem yılın gün(lord)/360 kadarıdır, bitişler birikimli günden.",
    "Rahu: 54 gün; birikimli 54/360 → bitiş 54,788454. gün.",
    "Jüpiter: 48 gün; birikimli 102/360 → bitiş 103,489303. gün.",
    "Satürn: 57 gün; birikimli 159/360 → bitiş 161,32156. gün.",
    "Merkür: 51 gün; birikimli 210/360 → bitiş 213,066212. gün.",
    "Ketu: 21 gün; birikimli 231/360 → bitiş 234,372833. gün.",
    "Venüs: 60 gün; birikimli 291/360 → bitiş 295,248893. gün.",
    "Güneş: 18 gün; birikimli 309/360 → bitiş 313,511712. gün.",
    "Ay: 30 gün; birikimli 339/360 → bitiş 343,949742. gün.",
    "Mars: 21 gün; birikimli 360/360 → bitiş 365,256363. gün."
  ],
  "input": {"moon_lon": 71.0, "age": 9, "year_days": 365.256363},
  "expected": {
    "birth_nakshatra": 6,
    "remainder": 4,
    "first_lord": "Rahu",
    "periods": [["Rahu", 54.788454], ["Jupiter", 103.489303], ["Saturn", 161.32156], ["Mercury", 213.066212], ["Ketu", 234.372833], ["Venus", 295.248893], ["Sun", 313.511712], ["Moon", 343.949742], ["Mars", 365.256363]],
    "first_sub_periods": null
  }
}
````

**Yeni dosya:** `tests/golden/modules/varsha/mudda/ashwini-birth-year-negative-count.json`

````json
{
  "id": "ashwini-birth-year-negative-count",
  "source": "hand",
  "rule_ref": "Tajika yıllık harita: mudda dasa (alıntı yapılmaz); okuma kuralları ve tablolar varsha/REVIEW.md",
  "derivation": [
    "Toplam eksiye düşer (1 + 0 − 2 = −1): bölmede −1 = −1 × 9 + 8, artan 8.",
    "Doğum nakshatrası: Ay 5° ÷ 13°20′ → 1. nakshatra (Ashwini).",
    "İlk lord: 1 + 0 − 2 = -1 = -1 × 9 + 8: artan 8 → 8. lord Ketu.",
    "Yıl 365,256363 gün; her dönem yılın gün(lord)/360 kadarıdır, bitişler birikimli günden.",
    "Ketu: 21 gün; birikimli 21/360 → bitiş 21,306621. gün.",
    "Venüs: 60 gün; birikimli 81/360 → bitiş 82,182682. gün.",
    "Güneş: 18 gün; birikimli 99/360 → bitiş 100,4455. gün.",
    "Ay: 30 gün; birikimli 129/360 → bitiş 130,88353. gün.",
    "Mars: 21 gün; birikimli 150/360 → bitiş 152,190151. gün.",
    "Rahu: 54 gün; birikimli 204/360 → bitiş 206,978606. gün.",
    "Jüpiter: 48 gün; birikimli 252/360 → bitiş 255,679454. gün.",
    "Satürn: 57 gün; birikimli 309/360 → bitiş 313,511712. gün.",
    "Merkür: 51 gün; birikimli 360/360 → bitiş 365,256363. gün."
  ],
  "input": {"moon_lon": 5.0, "age": 0, "year_days": 365.256363},
  "expected": {
    "birth_nakshatra": 1,
    "remainder": 8,
    "first_lord": "Ketu",
    "periods": [["Ketu", 21.306621], ["Venus", 82.182682], ["Sun", 100.4455], ["Moon", 130.88353], ["Mars", 152.190151], ["Rahu", 206.978606], ["Jupiter", 255.679454], ["Saturn", 313.511712], ["Mercury", 365.256363]],
    "first_sub_periods": null
  }
}
````

**Yeni dosya:** `tests/golden/modules/varsha/mudda/bharani-birth-year-remainder-zero.json`

````json
{
  "id": "bharani-birth-year-remainder-zero",
  "source": "hand",
  "rule_ref": "Tajika yıllık harita: mudda dasa (alıntı yapılmaz); okuma kuralları ve tablolar varsha/REVIEW.md",
  "derivation": [
    "Artan 0: dokuzuncu lord. Doğum yılında ilk lord doğum nakshatrasının Vimshottari lordudur (Bharani: Venüs).",
    "Doğum nakshatrası: Ay 20° ÷ 13°20′ → 2. nakshatra (Bharani).",
    "İlk lord: 2 + 0 − 2 = 0 = 0 × 9 + 0: artan 0 → dokuzuncu lord Venüs.",
    "Yıl 360 gün; her dönem yılın gün(lord)/360 kadarıdır, bitişler birikimli günden.",
    "Venüs: 60 gün; birikimli 60/360 → bitiş 60. gün.",
    "Güneş: 18 gün; birikimli 78/360 → bitiş 78. gün.",
    "Ay: 30 gün; birikimli 108/360 → bitiş 108. gün.",
    "Mars: 21 gün; birikimli 129/360 → bitiş 129. gün.",
    "Rahu: 54 gün; birikimli 183/360 → bitiş 183. gün.",
    "Jüpiter: 48 gün; birikimli 231/360 → bitiş 231. gün.",
    "Satürn: 57 gün; birikimli 288/360 → bitiş 288. gün.",
    "Merkür: 51 gün; birikimli 339/360 → bitiş 339. gün.",
    "Ketu: 21 gün; birikimli 360/360 → bitiş 360. gün.",
    "İlk dönemin (Venüs, 0 → 60) alt dönemleri Venüs ile başlar ve dönemi aynı paylarla böler:",
    "Venüs–Venüs: birikimli 60/360 × 60 → bitiş 10. gün.",
    "Venüs–Güneş: birikimli 78/360 × 60 → bitiş 13. gün.",
    "Venüs–Ay: birikimli 108/360 × 60 → bitiş 18. gün.",
    "Venüs–Mars: birikimli 129/360 × 60 → bitiş 21,5. gün.",
    "Venüs–Rahu: birikimli 183/360 × 60 → bitiş 30,5. gün.",
    "Venüs–Jüpiter: birikimli 231/360 × 60 → bitiş 38,5. gün.",
    "Venüs–Satürn: birikimli 288/360 × 60 → bitiş 48. gün.",
    "Venüs–Merkür: birikimli 339/360 × 60 → bitiş 56,5. gün.",
    "Venüs–Ketu: birikimli 360/360 × 60 → bitiş 60. gün."
  ],
  "input": {"moon_lon": 20.0, "age": 0, "year_days": 360.0},
  "expected": {
    "birth_nakshatra": 2,
    "remainder": 0,
    "first_lord": "Venus",
    "periods": [["Venus", 60.0], ["Sun", 78.0], ["Moon", 108.0], ["Mars", 129.0], ["Rahu", 183.0], ["Jupiter", 231.0], ["Saturn", 288.0], ["Mercury", 339.0], ["Ketu", 360.0]],
    "first_sub_periods": [["Venus", 10.0], ["Sun", 13.0], ["Moon", 18.0], ["Mars", 21.5], ["Rahu", 30.5], ["Jupiter", 38.5], ["Saturn", 48.0], ["Mercury", 56.5], ["Ketu", 60.0]]
  }
}
````

**Yeni dosya:** `tests/golden/modules/varsha/mudda/krittika-birth-year-sun-first.json`

````json
{
  "id": "krittika-birth-year-sun-first",
  "source": "hand",
  "rule_ref": "Tajika yıllık harita: mudda dasa (alıntı yapılmaz); okuma kuralları ve tablolar varsha/REVIEW.md",
  "derivation": [
    "Dönüşten dönüşe yıl (365,256363 gün): dönemler aynı oranda uzar.",
    "Doğum nakshatrası: Ay 30° ÷ 13°20′ → 3. nakshatra (Krittika).",
    "İlk lord: 3 + 0 − 2 = 1 = 0 × 9 + 1: artan 1 → 1. lord Güneş.",
    "Yıl 365,256363 gün; her dönem yılın gün(lord)/360 kadarıdır, bitişler birikimli günden.",
    "Güneş: 18 gün; birikimli 18/360 → bitiş 18,262818. gün.",
    "Ay: 30 gün; birikimli 48/360 → bitiş 48,700848. gün.",
    "Mars: 21 gün; birikimli 69/360 → bitiş 70,00747. gün.",
    "Rahu: 54 gün; birikimli 123/360 → bitiş 124,795924. gün.",
    "Jüpiter: 48 gün; birikimli 171/360 → bitiş 173,496772. gün.",
    "Satürn: 57 gün; birikimli 228/360 → bitiş 231,32903. gün.",
    "Merkür: 51 gün; birikimli 279/360 → bitiş 283,073681. gün.",
    "Ketu: 21 gün; birikimli 300/360 → bitiş 304,380303. gün.",
    "Venüs: 60 gün; birikimli 360/360 → bitiş 365,256363. gün.",
    "İlk dönemin (Güneş, 0 → 18,262818) alt dönemleri Güneş ile başlar ve dönemi aynı paylarla böler:",
    "Güneş–Güneş: birikimli 18/360 × 18,262818 → bitiş 0,913141. gün.",
    "Güneş–Ay: birikimli 48/360 × 18,262818 → bitiş 2,435042. gün.",
    "Güneş–Mars: birikimli 69/360 × 18,262818 → bitiş 3,500373. gün.",
    "Güneş–Rahu: birikimli 123/360 × 18,262818 → bitiş 6,239796. gün.",
    "Güneş–Jüpiter: birikimli 171/360 × 18,262818 → bitiş 8,674839. gün.",
    "Güneş–Satürn: birikimli 228/360 × 18,262818 → bitiş 11,566451. gün.",
    "Güneş–Merkür: birikimli 279/360 × 18,262818 → bitiş 14,153684. gün.",
    "Güneş–Ketu: birikimli 300/360 × 18,262818 → bitiş 15,219015. gün.",
    "Güneş–Venüs: birikimli 360/360 × 18,262818 → bitiş 18,262818. gün."
  ],
  "input": {"moon_lon": 30.0, "age": 0, "year_days": 365.256363},
  "expected": {
    "birth_nakshatra": 3,
    "remainder": 1,
    "first_lord": "Sun",
    "periods": [["Sun", 18.262818], ["Moon", 48.700848], ["Mars", 70.00747], ["Rahu", 124.795924], ["Jupiter", 173.496772], ["Saturn", 231.32903], ["Mercury", 283.073681], ["Ketu", 304.380303], ["Venus", 365.256363]],
    "first_sub_periods": [["Sun", 0.913141], ["Moon", 2.435042], ["Mars", 3.500373], ["Rahu", 6.239796], ["Jupiter", 8.674839], ["Saturn", 11.566451], ["Mercury", 14.153684], ["Ketu", 15.219015], ["Venus", 18.262818]]
  }
}
````

**Yeni dosya:** `tests/golden/modules/varsha/mudda/magha-age-62-mercury-first.json`

````json
{
  "id": "magha-age-62-mercury-first",
  "source": "hand",
  "rule_ref": "Tajika yıllık harita: mudda dasa (alıntı yapılmaz); okuma kuralları ve tablolar varsha/REVIEW.md",
  "derivation": [
    "Nakshatra sınırı: 120° tam Magha'nın (10.) başıdır.",
    "Doğum nakshatrası: Ay 120° ÷ 13°20′ → 10. nakshatra (Magha).",
    "İlk lord: 10 + 62 − 2 = 70 = 7 × 9 + 7: artan 7 → 7. lord Merkür.",
    "Yıl 360 gün; her dönem yılın gün(lord)/360 kadarıdır, bitişler birikimli günden.",
    "Merkür: 51 gün; birikimli 51/360 → bitiş 51. gün.",
    "Ketu: 21 gün; birikimli 72/360 → bitiş 72. gün.",
    "Venüs: 60 gün; birikimli 132/360 → bitiş 132. gün.",
    "Güneş: 18 gün; birikimli 150/360 → bitiş 150. gün.",
    "Ay: 30 gün; birikimli 180/360 → bitiş 180. gün.",
    "Mars: 21 gün; birikimli 201/360 → bitiş 201. gün.",
    "Rahu: 54 gün; birikimli 255/360 → bitiş 255. gün.",
    "Jüpiter: 48 gün; birikimli 303/360 → bitiş 303. gün.",
    "Satürn: 57 gün; birikimli 360/360 → bitiş 360. gün."
  ],
  "input": {"moon_lon": 120.0, "age": 62, "year_days": 360.0},
  "expected": {
    "birth_nakshatra": 10,
    "remainder": 7,
    "first_lord": "Mercury",
    "periods": [["Mercury", 51.0], ["Ketu", 72.0], ["Venus", 132.0], ["Sun", 150.0], ["Moon", 180.0], ["Mars", 201.0], ["Rahu", 255.0], ["Jupiter", 303.0], ["Saturn", 360.0]],
    "first_sub_periods": null
  }
}
````

**Yeni dosya:** `tests/golden/modules/varsha/mudda/punarvasu-age-18-jupiter-first.json`

````json
{
  "id": "punarvasu-age-18-jupiter-first",
  "source": "hand",
  "rule_ref": "Tajika yıllık harita: mudda dasa (alıntı yapılmaz); okuma kuralları ve tablolar varsha/REVIEW.md",
  "derivation": [
    "Punarvasu (7), tamamlanan yıl 18.",
    "Doğum nakshatrası: Ay 85° ÷ 13°20′ → 7. nakshatra (Punarvasu).",
    "İlk lord: 7 + 18 − 2 = 23 = 2 × 9 + 5: artan 5 → 5. lord Jüpiter.",
    "Yıl 360 gün; her dönem yılın gün(lord)/360 kadarıdır, bitişler birikimli günden.",
    "Jüpiter: 48 gün; birikimli 48/360 → bitiş 48. gün.",
    "Satürn: 57 gün; birikimli 105/360 → bitiş 105. gün.",
    "Merkür: 51 gün; birikimli 156/360 → bitiş 156. gün.",
    "Ketu: 21 gün; birikimli 177/360 → bitiş 177. gün.",
    "Venüs: 60 gün; birikimli 237/360 → bitiş 237. gün.",
    "Güneş: 18 gün; birikimli 255/360 → bitiş 255. gün.",
    "Ay: 30 gün; birikimli 285/360 → bitiş 285. gün.",
    "Mars: 21 gün; birikimli 306/360 → bitiş 306. gün.",
    "Rahu: 54 gün; birikimli 360/360 → bitiş 360. gün."
  ],
  "input": {"moon_lon": 85.0, "age": 18, "year_days": 360.0},
  "expected": {
    "birth_nakshatra": 7,
    "remainder": 5,
    "first_lord": "Jupiter",
    "periods": [["Jupiter", 48.0], ["Saturn", 105.0], ["Mercury", 156.0], ["Ketu", 177.0], ["Venus", 237.0], ["Sun", 255.0], ["Moon", 285.0], ["Mars", 306.0], ["Rahu", 360.0]],
    "first_sub_periods": null
  }
}
````

**Yeni dosya:** `tests/golden/modules/varsha/mudda/revati-age-116-saturn-first.json`

````json
{
  "id": "revati-age-116-saturn-first",
  "source": "hand",
  "rule_ref": "Tajika yıllık harita: mudda dasa (alıntı yapılmaz); okuma kuralları ve tablolar varsha/REVIEW.md",
  "derivation": [
    "Son nakshatra ve ileri yaş: Revati (27), tamamlanan yıl 116.",
    "Doğum nakshatrası: Ay 359,9° ÷ 13°20′ → 27. nakshatra (Revati).",
    "İlk lord: 27 + 116 − 2 = 141 = 15 × 9 + 6: artan 6 → 6. lord Satürn.",
    "Yıl 365,256363 gün; her dönem yılın gün(lord)/360 kadarıdır, bitişler birikimli günden.",
    "Satürn: 57 gün; birikimli 57/360 → bitiş 57,832257. gün.",
    "Merkür: 51 gün; birikimli 108/360 → bitiş 109,576909. gün.",
    "Ketu: 21 gün; birikimli 129/360 → bitiş 130,88353. gün.",
    "Venüs: 60 gün; birikimli 189/360 → bitiş 191,759591. gün.",
    "Güneş: 18 gün; birikimli 207/360 → bitiş 210,022409. gün.",
    "Ay: 30 gün; birikimli 237/360 → bitiş 240,460439. gün.",
    "Mars: 21 gün; birikimli 258/360 → bitiş 261,76706. gün.",
    "Rahu: 54 gün; birikimli 312/360 → bitiş 316,555515. gün.",
    "Jüpiter: 48 gün; birikimli 360/360 → bitiş 365,256363. gün."
  ],
  "input": {"moon_lon": 359.9, "age": 116, "year_days": 365.256363},
  "expected": {
    "birth_nakshatra": 27,
    "remainder": 6,
    "first_lord": "Saturn",
    "periods": [["Saturn", 57.832257], ["Mercury", 109.576909], ["Ketu", 130.88353], ["Venus", 191.759591], ["Sun", 210.022409], ["Moon", 240.460439], ["Mars", 261.76706], ["Rahu", 316.555515], ["Jupiter", 365.256363]],
    "first_sub_periods": null
  }
}
````

**Yeni dosya:** `tests/golden/modules/varsha/mudda/rohini-age-36-moon-first.json`

````json
{
  "id": "rohini-age-36-moon-first",
  "source": "hand",
  "rule_ref": "Tajika yıllık harita: mudda dasa (alıntı yapılmaz); okuma kuralları ve tablolar varsha/REVIEW.md",
  "derivation": [
    "Otuz altı yıl dört tam çevrimdir: ilk lord doğum yılındakiyle aynıdır (Rohini: Ay).",
    "Doğum nakshatrası: Ay 45° ÷ 13°20′ → 4. nakshatra (Rohini).",
    "İlk lord: 4 + 36 − 2 = 38 = 4 × 9 + 2: artan 2 → 2. lord Ay.",
    "Yıl 360 gün; her dönem yılın gün(lord)/360 kadarıdır, bitişler birikimli günden.",
    "Ay: 30 gün; birikimli 30/360 → bitiş 30. gün.",
    "Mars: 21 gün; birikimli 51/360 → bitiş 51. gün.",
    "Rahu: 54 gün; birikimli 105/360 → bitiş 105. gün.",
    "Jüpiter: 48 gün; birikimli 153/360 → bitiş 153. gün.",
    "Satürn: 57 gün; birikimli 210/360 → bitiş 210. gün.",
    "Merkür: 51 gün; birikimli 261/360 → bitiş 261. gün.",
    "Ketu: 21 gün; birikimli 282/360 → bitiş 282. gün.",
    "Venüs: 60 gün; birikimli 342/360 → bitiş 342. gün.",
    "Güneş: 18 gün; birikimli 360/360 → bitiş 360. gün."
  ],
  "input": {"moon_lon": 45.0, "age": 36, "year_days": 360.0},
  "expected": {
    "birth_nakshatra": 4,
    "remainder": 2,
    "first_lord": "Moon",
    "periods": [["Moon", 30.0], ["Mars", 51.0], ["Rahu", 105.0], ["Jupiter", 153.0], ["Saturn", 210.0], ["Mercury", 261.0], ["Ketu", 282.0], ["Venus", 342.0], ["Sun", 360.0]],
    "first_sub_periods": null
  }
}
````

**Yeni dosya:** `tests/golden/modules/varsha/mudda/swati-age-26-mars-first.json`

````json
{
  "id": "swati-age-26-mars-first",
  "source": "hand",
  "rule_ref": "Tajika yıllık harita: mudda dasa (alıntı yapılmaz); okuma kuralları ve tablolar varsha/REVIEW.md",
  "derivation": [
    "360 günlük yıl: bitişler tam gündür.",
    "Doğum nakshatrası: Ay 199,47° ÷ 13°20′ → 15. nakshatra (Swati).",
    "İlk lord: 15 + 26 − 2 = 39 = 4 × 9 + 3: artan 3 → 3. lord Mars.",
    "Yıl 360 gün; her dönem yılın gün(lord)/360 kadarıdır, bitişler birikimli günden.",
    "Mars: 21 gün; birikimli 21/360 → bitiş 21. gün.",
    "Rahu: 54 gün; birikimli 75/360 → bitiş 75. gün.",
    "Jüpiter: 48 gün; birikimli 123/360 → bitiş 123. gün.",
    "Satürn: 57 gün; birikimli 180/360 → bitiş 180. gün.",
    "Merkür: 51 gün; birikimli 231/360 → bitiş 231. gün.",
    "Ketu: 21 gün; birikimli 252/360 → bitiş 252. gün.",
    "Venüs: 60 gün; birikimli 312/360 → bitiş 312. gün.",
    "Güneş: 18 gün; birikimli 330/360 → bitiş 330. gün.",
    "Ay: 30 gün; birikimli 360/360 → bitiş 360. gün.",
    "İlk dönemin (Mars, 0 → 21) alt dönemleri Mars ile başlar ve dönemi aynı paylarla böler:",
    "Mars–Mars: birikimli 21/360 × 21 → bitiş 1,225. gün.",
    "Mars–Rahu: birikimli 75/360 × 21 → bitiş 4,375. gün.",
    "Mars–Jüpiter: birikimli 123/360 × 21 → bitiş 7,175. gün.",
    "Mars–Satürn: birikimli 180/360 × 21 → bitiş 10,5. gün.",
    "Mars–Merkür: birikimli 231/360 × 21 → bitiş 13,475. gün.",
    "Mars–Ketu: birikimli 252/360 × 21 → bitiş 14,7. gün.",
    "Mars–Venüs: birikimli 312/360 × 21 → bitiş 18,2. gün.",
    "Mars–Güneş: birikimli 330/360 × 21 → bitiş 19,25. gün.",
    "Mars–Ay: birikimli 360/360 × 21 → bitiş 21. gün."
  ],
  "input": {"moon_lon": 199.47, "age": 26, "year_days": 360.0},
  "expected": {
    "birth_nakshatra": 15,
    "remainder": 3,
    "first_lord": "Mars",
    "periods": [["Mars", 21.0], ["Rahu", 75.0], ["Jupiter", 123.0], ["Saturn", 180.0], ["Mercury", 231.0], ["Ketu", 252.0], ["Venus", 312.0], ["Sun", 330.0], ["Moon", 360.0]],
    "first_sub_periods": [["Mars", 1.225], ["Rahu", 4.375], ["Jupiter", 7.175], ["Saturn", 10.5], ["Mercury", 13.475], ["Ketu", 14.7], ["Venus", 18.2], ["Sun", 19.25], ["Moon", 21.0]]
  }
}
````

- [ ] **Step 4: Testler ve kapılar**

```bash
cargo fmt --all
cargo test -p jyotish-core --test varsha_golden 2>&1 | grep -E "^test result"
cargo test -p jyotish-core 2>&1 | grep -E "^test result"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
ls tests/golden/modules/varsha/muntha | wc -l
ls tests/golden/modules/varsha/mudda | wc -l
```
Expected: `3 passed`; lib `487 passed`, `avastha_golden` `5 passed`, golden `8 passed`, `modules_golden` `17 passed`, `varsha_golden` `3 passed`; fmt ve clippy çıktısız; sayaç `0`; `5` ve `9`. Bir örnek motorla uyuşmazsa **hiçbir beklenen değer değiştirilmez**: önce dosyanın bu plandaki metinle bire bir aynı olduğu denetlenir, sonra `derivation` adımları kuraldan (D8, D9) yeniden okunur; fark sürüyorsa ilerleme notlarına yazılır ve görev durdurulur.

- [ ] **Step 5: Commit**

```bash
git add crates/jyotish-core/tests/varsha_golden.rs tests/golden/modules/varsha
git diff --cached --stat | tail -1
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "test(golden): varsha hand fixtures for the muntha and the mudda dasa" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: `15 files changed, 670 insertions(+)`; tarama çıktısız; `git status --short` boş.

---

### Task 5: Uygulama: `compute_varsha` ve `save_varsha_chart` komutları, bağlamalar ve fikstür

Yol haritası §5 Plan 27 görev (2) ("komutu, yer seçimiyle") ve (7) ("olay haritası olarak kaydetme"); K1–K3; D5, D7, D11, D13, D14, D21. Rust tarafı bu görevde biter; arayüz Task 6'dadır (üretilen bağlamalar tek başına arayüz testlerini bozmaz: Vitest bu görevin sonunda taban sayısındadır).

**Files:**
- Create: `app/src-tauri/src/varsha_commands.rs`, `app/src-tauri/tests/varsha.rs`
- Modify: `app/src-tauri/src/lib.rs` (üç satır; paylaşılan dosya), `app/src-tauri/tests/ui_fixture.rs` (bir blok ve bir anahtar; paylaşılan dosya)
- Generate: `app/ui/bindings.ts`, `app/ui/test/fixtures/delhi.json` (yalnızca iki komutla)
- Test: `tests/varsha.rs` (1)

**Interfaces:**
- Consumes (hepsi var): `jyotish_core::varsha::{compute_varsha, VarshaReport, VarshaYear}` (Task 3); `crate::muhurta_commands::MuhurtaPlace { lat, lon, tz: TzChoice }`; `crate::client_commands::{blocking, lock, StoreState}`; `crate::commands::CommandError`; `atlas::{offset_at, Timestamp, TzSpec}`; `store::{resolve_birth, BirthData, ChartKind, ChartRecord, FoldChoice, HomePlace, NewChart, Store, TimeCertainty, TzChoice}` ve `Store`'un `chart`, `calc_settings`, `prashna_meta`, `app_prefs`, `add_chart`, `save_calc_settings` yöntemleri; `ephemeris::{Engine, SwissEph}`.
- Produces (`jyotish_desk_lib::varsha_commands`):
  - `VARSHA_LABEL: &str = "Varshaphala"`; `VarshaView { report: VarshaReport, offset_secs: i32 }`.
  - `compute_varsha_blocking(engine: &Engine, input: ChartInput, age: Option<u8>, place: MuhurtaPlace, now: Timestamp) -> Result<VarshaView, CommandError>` ve komut `compute_varsha(input, age, place)`.
  - `save_varsha_chart_blocking(engine: &Engine, store: &Mutex<Store>, chart_id: u32, age: u8, home: bool) -> Result<ChartRecord, CommandError>` ve komut `save_varsha_chart(chart_id, age, home)`.
  - Arayüz bağlamaları (üretilir): `commands.computeVarsha(input, age, place)`, `commands.saveVarshaChart(chartId, age, home)`; tipler `VarshaView`, `VarshaReport`, `YearFacts`, `Muntha`, `MuddaDasha`.
  - Fikstür: `varsha = { current, next, time_unknown, mudda_order, max_age }`.

- [ ] **Step 1: Rust testleri: gerçek motorla ±1″ ve çapraz denetim, kayıt, hatalar ve bütçe; fikstür**

Tek test (süreçte tek motor): beş golden anın her birinde 0, 1, 2, 30 ve 80 yaşları için dönüşte Güneş'in doğumdaki sidereal boylamından bir yay saniyesinden az ayrıştığı, yılın bir sidereal yıl sürdüğü, ardışık yılların boşluksuz olduğu, munthanın ve mudda dasanın ilk lordunun kuraldan çıktığı; olay aramasının (`event_search`, `TransitToNatal`) aynı dönüşü bir saniye içinde bulduğu; ev sisteminin sonucu değiştirmediği; doğum saati bilinmeyen girdide yıllık haritanın ve `facts`'in boş olduğu; yaş verilmeyince `now` anında süren yılın (26) döndüğü; başka yerin yalnızca yıllık haritayı değiştirdiği; hata kodları; kaydın (tür, etiket, yer, saat dilimi, ayar kopyası, açılmama, bulunulan yer, saati bilinmeyen harita) ve bütçenin denetimi.

**Yeni dosya:** `app/src-tauri/tests/varsha.rs`

````rust
//! Varshaphala komut gövdeleri, gerçek `Engine` ile uçtan uca (yol haritası §4.14; ileri modüller
//! spec'i §13 "Komut katmanı", §15 bütçeleri). Tek test: süreçte yalnızca bir SE örneği
//! açılabilir. Ayrı dosyadır: `tests/modules.rs` paralel planlarla paylaşılmaz.

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use atlas::Timestamp;
use ephemeris::{Engine, SwissEph};
use jyotish_core::varsha::{
    mudda_first_lord, muntha_sign, VarshaYear, MAX_VARSHA_AGE, SIDEREAL_YEAR_DAYS, VARSHA_METHOD,
};
use jyotish_core::{
    angular_distance, event_search, CalcSettings, ChartInput, EventSpec, Graha, HouseSystem,
    Positions, SkyEventKind,
};
use jyotish_desk_lib::client_commands::create_client_blocking;
use jyotish_desk_lib::commands::compute_chart_blocking;
use jyotish_desk_lib::muhurta_commands::MuhurtaPlace;
use jyotish_desk_lib::varsha_commands::{
    compute_varsha_blocking, save_varsha_chart_blocking, VARSHA_LABEL,
};
use serde_json::Value;
use store::{
    BirthData, ChartKind, HomePlace, NewChart, NewClient, Place, Store, TimeCertainty, TzChoice,
};

/// Bir yay saniyesi, derece.
const ARCSEC: f64 = 1.0 / 3600.0;
/// Bir saniye, gün.
const SECOND: f64 = 1.0 / 86_400.0;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// `tests/golden/*.json` fixture'larının anları (ad, girdi).
fn golden_inputs() -> Vec<(String, ChartInput)> {
    let dir = root().join("../../tests/golden");
    let mut out: Vec<(String, ChartInput)> = std::fs::read_dir(&dir)
        .expect("golden dir")
        .map(|e| e.expect("dir entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .map(|p| {
            let v: Value =
                serde_json::from_str(&std::fs::read_to_string(&p).expect("read fixture"))
                    .expect("parse fixture");
            let num = |k: &str| v["input"][k].as_f64().expect("number");
            let input = ChartInput {
                jd_ut: num("jd_ut"),
                lat: num("lat"),
                lon: num("lon"),
                settings: serde_json::from_value(v["settings"].clone()).expect("settings"),
                time_known: true,
                lagna_number: None,
            };
            (v["id"].as_str().expect("id").to_owned(), input)
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

fn median_of_15(mut run: impl FnMut()) -> (Duration, Duration) {
    run();
    let mut times: Vec<Duration> = (0..15)
        .map(|_| {
            let t = Instant::now();
            run();
            t.elapsed()
        })
        .collect();
    times.sort();
    (times[times.len() / 2], times[times.len() - 1])
}

/// Girdinin kendi yeri, UTC ofsetiyle.
fn birth_place(input: &ChartInput) -> MuhurtaPlace {
    MuhurtaPlace {
        lat: input.lat,
        lon: input.lon,
        tz: TzChoice::Fixed(0),
    }
}

/// tests/golden/new-delhi-2000-01-01.json: 17:30 IST = JD 2451545.0 (deneme verisi).
fn delhi() -> BirthData {
    BirthData {
        date: "2000-01-01".into(),
        time: Some("17:30:00".into()),
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
fn varsha_commands_run_on_the_engine_thread_and_map_errors() {
    let engine = Engine::spawn(root().join("../../data/ephe")).expect("spawn engine");
    // Fikstürün sabit saati: 2026-11-25 10:00 UTC.
    let now = Timestamp::from_second(1_795_600_800).expect("timestamp");
    let inputs = golden_inputs();
    assert!(inputs.len() >= 5, "{} golden moments", inputs.len());
    for (id, input) in &inputs {
        let natal = compute_chart_blocking(
            &engine,
            input.jd_ut,
            input.lat,
            input.lon,
            input.settings.clone(),
        )
        .expect("natal chart");
        let sun = natal.grahas[Graha::Sun as usize].sidereal_lon;
        let moon_nakshatra = natal.grahas[Graha::Moon as usize].nakshatra;
        let mut previous_end = None;
        for age in [0_u8, 1, 2, 30, 80] {
            let view =
                compute_varsha_blocking(&engine, input.clone(), Some(age), birth_place(input), now)
                    .unwrap_or_else(|e| panic!("{id} age {age}: {}", e.code));
            let r = &view.report;
            assert_eq!((r.age, r.time_known), (age, true), "{id}");
            assert_eq!(r.method, VARSHA_METHOD, "{id}");
            assert_eq!(view.offset_secs, 0, "{id}: a fixed UTC place");

            // Dönüş anı (±1″): yıllık haritanın Güneş'i doğumdaki sidereal boylamındadır.
            let chart = r.chart.as_ref().expect("annual chart");
            assert_eq!(chart.jd_ut, r.start_jd_ut, "{id} age {age}");
            assert_eq!(chart.meta.settings, input.settings, "{id}");
            let back = chart.grahas[Graha::Sun as usize].sidereal_lon;
            assert!(
                angular_distance(back, sun) < ARCSEC,
                "{id} age {age}: the Sun is at {back}, born at {sun}"
            );
            // Yıl bir sidereal yıl sürer (gerçek Güneş'te birkaç dakika oynar).
            let days = r.end_jd_ut - r.start_jd_ut;
            assert!(
                (days - SIDEREAL_YEAR_DAYS).abs() < 0.02,
                "{id} age {age}: {days} days"
            );
            if age == 0 {
                assert_eq!(r.start_jd_ut, input.jd_ut, "{id}: the birth year");
            }
            // Ardışık yıllar boşluksuzdur.
            if let (Some(end), 1 | 2) = (previous_end, age) {
                assert_eq!(r.start_jd_ut, end, "{id} age {age}");
            }
            previous_end = Some(r.end_jd_ut);

            // Muntha ve lagna lordları: iki haritanın lagnasından.
            let facts = r.facts.expect("facts");
            assert_eq!(
                facts.muntha.sign,
                muntha_sign(natal.lagna.sign, age),
                "{id}"
            );
            assert_eq!(facts.muntha.natal_house, age % 12 + 1, "{id}");
            assert_eq!(
                (facts.lagna_sign, facts.natal_lagna_sign),
                (chart.lagna.sign, natal.lagna.sign),
                "{id}"
            );
            // Mudda dasa: ilk lord kuraldan; dokuz dönem yılı boşluksuz doldurur.
            assert_eq!(r.mudda.birth_nakshatra, moon_nakshatra, "{id}");
            assert_eq!(
                r.mudda.first_lord,
                mudda_first_lord(moon_nakshatra, age),
                "{id}"
            );
            let top: Vec<_> = r.mudda.periods.iter().filter(|p| p.level == 1).collect();
            assert_eq!((top.len(), r.mudda.periods.len()), (9, 90), "{id}");
            assert_eq!(
                (top[0].start_jd_ut, top[8].end_jd_ut),
                (r.start_jd_ut, r.end_jd_ut),
                "{id}"
            );
        }

        // İkinci yazım: Plan 22'nin olay araması aynı dönüşü `TransitToNatal` olarak bulur
        // (sabit adımlı yürüyücü, 0,05 sn çözünürlük). İki bağımsız arama bir saniyede buluşur.
        let view =
            compute_varsha_blocking(&engine, input.clone(), Some(30), birth_place(input), now)
                .expect("age 30");
        let start = view.report.start_jd_ut;
        let (settings, positions) = (input.settings.clone(), Positions::from(&natal));
        let events = engine
            .call(move |se: &mut SwissEph| {
                let spec = EventSpec {
                    from_jd_ut: start - 5.0,
                    to_jd_ut: start + 5.0,
                    kinds: vec![SkyEventKind::TransitToNatal],
                    grahas: vec![Graha::Sun],
                };
                event_search(se, &settings, &spec, Some(&positions))
            })
            .expect("engine")
            .expect("event search");
        let returns: Vec<f64> = events
            .iter()
            .filter(|e| e.grahas == [Graha::Sun, Graha::Sun])
            .map(|e| e.jd_ut)
            .collect();
        assert_eq!(returns.len(), 1, "{id}: one return in ten days");
        assert!(
            (returns[0] - start).abs() < SECOND,
            "{id}: event search {} vs {start}",
            returns[0]
        );

        // Ev sistemi dönüş anını, munthayı ve mudda dasayı değiştirmez (burç evleri, K4).
        for house_system in [HouseSystem::Sripati, HouseSystem::Placidus] {
            let other = ChartInput {
                settings: CalcSettings {
                    house_system,
                    ..input.settings.clone()
                },
                ..input.clone()
            };
            let o = compute_varsha_blocking(&engine, other, Some(30), birth_place(input), now)
                .expect("house system")
                .report;
            assert_eq!(
                (o.start_jd_ut, o.end_jd_ut, o.facts, &o.mudda),
                (
                    view.report.start_jd_ut,
                    view.report.end_jd_ut,
                    view.report.facts,
                    &view.report.mudda
                ),
                "{id}: {house_system:?}"
            );
        }

        // Doğum saati bilinmiyorsa yıllık harita ve muntha yoktur; yıl ve mudda dasa aynıdır.
        let unknown = compute_varsha_blocking(
            &engine,
            ChartInput {
                time_known: false,
                ..input.clone()
            },
            Some(30),
            birth_place(input),
            now,
        )
        .expect("time unknown")
        .report;
        assert!(!unknown.time_known, "{id}");
        assert_eq!((&unknown.chart, unknown.facts), (&None, None), "{id}");
        assert_eq!(
            (unknown.start_jd_ut, &unknown.mudda),
            (view.report.start_jd_ut, &view.report.mudda),
            "{id}"
        );
    }

    let delhi_input = inputs
        .iter()
        .find(|(id, _)| id.starts_with("new-delhi"))
        .map(|(_, input)| input.clone())
        .expect("new-delhi golden");
    let delhi_place = MuhurtaPlace {
        lat: delhi_input.lat,
        lon: delhi_input.lon,
        tz: TzChoice::Iana("Asia/Kolkata".into()),
    };
    // Yaş verilmezse `now` anında süren yıl: 2000-01-01 doğumlu harita 2026-11-25'te 26 yılını
    // tamamlamıştır. Ofset yerin dilimindendir (IST, +05:30).
    let running =
        compute_varsha_blocking(&engine, delhi_input.clone(), None, delhi_place.clone(), now)
            .expect("running year");
    assert_eq!((running.report.age, running.offset_secs), (26, 19_800));
    let now_jd = now.as_second() as f64 / 86_400.0 + 2_440_587.5;
    assert!(running.report.start_jd_ut <= now_jd && now_jd < running.report.end_jd_ut);
    let by_age = compute_varsha_blocking(
        &engine,
        delhi_input.clone(),
        Some(26),
        delhi_place.clone(),
        now,
    )
    .expect("age 26");
    assert_eq!(running, by_age);
    // Çekirdeğin `At` yolu komutun `now` yoluyla aynı yılı verir.
    let (i, p) = (delhi_input.clone(), (delhi_input.lat, delhi_input.lon));
    let core = engine
        .call(move |se: &mut SwissEph| {
            jyotish_core::varsha::compute_varsha(se, &i, VarshaYear::At(now_jd), Some(p))
        })
        .expect("engine")
        .expect("core");
    assert_eq!(core, running.report);

    // Başka yer: dönüş anı aynıdır, yıllık harita o yerdedir ve lagnası başkadır.
    let sydney = MuhurtaPlace {
        lat: -33.8688,
        lon: 151.2093,
        tz: TzChoice::Iana("Australia/Sydney".into()),
    };
    let moved = compute_varsha_blocking(&engine, delhi_input.clone(), Some(26), sydney, now)
        .expect("moved");
    assert_eq!(moved.report.start_jd_ut, by_age.report.start_jd_ut);
    assert_eq!(moved.report.mudda, by_age.report.mudda);
    let (here, there) = (
        by_age.report.chart.as_ref().expect("chart"),
        moved.report.chart.as_ref().expect("chart"),
    );
    assert_eq!((there.geo_lat, there.geo_lon), (-33.8688, 151.2093));
    assert_ne!(there.lagna.sidereal_lon, here.lagna.sidereal_lon);
    // Sidney yaz saatinde: +11:00.
    assert_eq!(moved.offset_secs, 39_600);

    // Hatalar: `compute_chart`'ın kodları; yaş `invalid_dasha_input`, çözülemeyen dilim
    // `invalid_input`. Hiçbiri yeni bir koddur.
    let code = |input: ChartInput, age: Option<u8>, place: MuhurtaPlace| {
        compute_varsha_blocking(&engine, input, age, place, now)
            .unwrap_err()
            .code
    };
    assert_eq!(
        code(
            delhi_input.clone(),
            Some(MAX_VARSHA_AGE + 1),
            delhi_place.clone()
        ),
        "invalid_dasha_input"
    );
    assert_eq!(
        code(
            delhi_input.clone(),
            Some(26),
            MuhurtaPlace {
                lat: 91.0,
                ..delhi_place.clone()
            }
        ),
        "invalid_location"
    );
    assert_eq!(
        code(
            delhi_input.clone(),
            Some(26),
            MuhurtaPlace {
                tz: TzChoice::Iana("Nowhere/Nothing".into()),
                ..delhi_place.clone()
            }
        ),
        "invalid_input"
    );
    assert_eq!(
        code(
            ChartInput {
                settings: CalcSettings {
                    schema_version: 1,
                    ..CalcSettings::default()
                },
                ..delhi_input.clone()
            },
            Some(26),
            delhi_place.clone()
        ),
        "unsupported_schema_version"
    );
    assert_eq!(
        code(
            ChartInput {
                lagna_number: Some(0),
                ..delhi_input.clone()
            },
            Some(26),
            delhi_place.clone()
        ),
        "invalid_prashna_input"
    );
    // Sayı prashnası: muntha sayının lagnasından sayılır (30 → Yengeç; 26 = 2 × 12 + 2: Başak).
    let numbered = compute_varsha_blocking(
        &engine,
        ChartInput {
            lagna_number: Some(30),
            ..delhi_input.clone()
        },
        Some(26),
        delhi_place.clone(),
        now,
    )
    .expect("number prashna")
    .report;
    assert_eq!(
        numbered.facts.expect("facts").muntha.sign,
        jyotish_core::Sign::Virgo
    );

    // Olay haritası olarak kaydetme: var olan `add_chart` yolu, kaynak haritanın ayarlarıyla.
    let clock = Box::new(|| Timestamp::from_second(1_795_600_800).expect("timestamp"));
    let store = Mutex::new(Store::open_in_memory().expect("store").with_clock(clock));
    let new_client = |name: &str, birth: BirthData| NewClient {
        name: name.into(),
        gender: None,
        chart: NewChart {
            label: String::new(),
            kind: ChartKind::Natal,
            birth,
        },
    };
    let opened = create_client_blocking(&engine, &store, new_client("Örnek Danışan", delhi()))
        .expect("create client");
    // Kaynak haritaya varsayılandan ayrı bir ayar yazılır: kopyalandığı görülsün.
    let sripati = CalcSettings {
        house_system: HouseSystem::Sripati,
        ..opened.calc_settings.clone()
    };
    store
        .lock()
        .expect("store")
        .save_calc_settings(opened.chart_record.id, &sripati)
        .expect("save settings");
    let saved = save_varsha_chart_blocking(&engine, &store, opened.chart_record.id, 26, false)
        .expect("save");
    assert_eq!(saved.kind, ChartKind::Event);
    assert_eq!(saved.client_id, opened.client.id);
    assert_eq!(saved.birth.place, opened.chart_record.birth.place);
    assert_eq!(saved.birth.tz, opened.chart_record.birth.tz);
    assert_eq!(saved.birth.time_certainty, TimeCertainty::Exact);
    assert_eq!(saved.label, format!("{VARSHA_LABEL} {}", saved.birth.date));
    // 26 sidereal yıl 9496,665 gündür; 2000-01-01'den 2026-01-01'e 9497 gün vardır: dönüş doğum
    // saatinden (12:00 UT) yaklaşık sekiz saat öncedir, 04 UT dolayında; Delhi'de 09 suları.
    assert_eq!(saved.birth.date, "2026-01-01");
    let clock = saved.birth.time.as_deref().expect("a time");
    assert!(clock.starts_with("09:") && clock.len() == 8, "{clock}");
    // Kayıt saniyeye yuvarlanır: çözülen an dönüş anından yarım saniyeden çok ayrışmaz.
    assert!(
        (saved.resolved.jd_ut - by_age.report.start_jd_ut).abs() <= 0.5 * SECOND + 1e-9,
        "{} vs {}",
        saved.resolved.jd_ut,
        by_age.report.start_jd_ut
    );
    {
        let s = store.lock().expect("store");
        assert_eq!(
            s.calc_settings(saved.id).expect("settings").settings(),
            &sripati
        );
        assert_eq!(s.charts_of(opened.client.id).expect("charts").len(), 2);
        // Harita açılmaz: danışanın son haritası kaynak haritadır.
        assert_eq!(
            s.client(opened.client.id).expect("client").last_chart_id,
            Some(opened.chart_record.id)
        );
    }
    let save_code = |chart_id: u32, age: u8, home: bool| {
        save_varsha_chart_blocking(&engine, &store, chart_id, age, home)
            .unwrap_err()
            .code
    };
    // Bulunulan yer seçilmemiş; olmayan harita; sınır dışı yaş.
    assert_eq!(save_code(opened.chart_record.id, 26, true), "invalid_input");
    assert_eq!(save_code(9_999, 26, false), "not_found");
    assert_eq!(
        save_code(opened.chart_record.id, MAX_VARSHA_AGE + 1, false),
        "invalid_dasha_input"
    );
    // Bulunulan yer seçilince yıllık harita orada, oranın duvar saatiyle kaydedilir.
    {
        let s = store.lock().expect("store");
        let mut prefs = s.app_prefs().expect("prefs");
        prefs.home_place = Some(HomePlace {
            place: Place {
                name: "Sydney".into(),
                country: "Australia".into(),
                admin1: "New South Wales".into(),
                lat: -33.8688,
                lon: 151.2093,
                geoname_id: None,
            },
            tz: TzChoice::Iana("Australia/Sydney".into()),
        });
        s.set_app_prefs(&prefs).expect("set prefs");
    }
    let at_home = save_varsha_chart_blocking(&engine, &store, opened.chart_record.id, 26, true)
        .expect("save at home");
    assert_eq!(at_home.birth.place.name, "Sydney");
    assert_eq!(at_home.birth.tz, TzChoice::Iana("Australia/Sydney".into()));
    assert!((at_home.resolved.jd_ut - saved.resolved.jd_ut).abs() <= SECOND);
    assert_eq!(at_home.resolved.offset_secs, 39_600);
    assert_eq!(at_home.birth.date, "2026-01-01");
    assert!(at_home
        .birth
        .time
        .as_deref()
        .is_some_and(|t| t.starts_with("14:")));
    // Doğum saati bilinmeyen haritanın yıllık haritası yoktur: kaydedilmez.
    let mut no_time = delhi();
    no_time.time = None;
    no_time.time_certainty = TimeCertainty::Unknown;
    let unknown = create_client_blocking(&engine, &store, new_client("Saatsiz Danışan", no_time))
        .expect("create client without a birth time");
    let err = save_varsha_chart_blocking(&engine, &store, unknown.chart_record.id, 26, false)
        .unwrap_err();
    assert_eq!(
        (
            err.code.as_str(),
            err.params.get("field").map(String::as_str)
        ),
        ("invalid_input", Some("birth.time"))
    );
    assert_eq!(
        store
            .lock()
            .expect("store")
            .charts_of(unknown.client.id)
            .expect("charts")
            .len(),
        1
    );

    // Performans: doğum haritası, iki dönüş araması ve yıllık harita dahil ≤ 20 ms.
    let (median, max) = median_of_15(|| {
        compute_varsha_blocking(
            &engine,
            delhi_input.clone(),
            Some(26),
            delhi_place.clone(),
            now,
        )
        .expect("varsha");
    });
    eprintln!("compute_varsha median {median:?}, max {max:?}");
    assert!(
        median <= Duration::from_millis(20),
        "compute_varsha median {median:?}"
    );
}
````

Fikstür (paylaşılan dosya; blok avastha hesaplarından sonra, anahtar `"avastha_ids"`'ten sonra):

````diff
diff --git a/app/src-tauri/tests/ui_fixture.rs b/app/src-tauri/tests/ui_fixture.rs
index 8d05736..0f3b195 100644
--- a/app/src-tauri/tests/ui_fixture.rs
+++ b/app/src-tauri/tests/ui_fixture.rs
@@ -224,6 +224,46 @@ fn ui_fixture_is_up_to_date() {
         },
     )
     .expect("avasthas without a birth time");
+    // Varshaphala (Plan 27). Açık haritanın girdisiyle, doğum yerinde (Delhi, Asia/Kolkata) ve
+    // fikstürün sabit saatinde: süren yıl (yaş verilmeden; 26), sonraki yıl (27) ve doğum saati
+    // bilinmeyen girdiyle süren yıl (yıllık harita ve muntha yok). Kendi bloğundadır: ortak `use`
+    // listesine ve öteki hesaplara dokunmaz.
+    let varsha = {
+        use jyotish_core::varsha::{MAX_VARSHA_AGE, MUDDA_ORDER};
+        use jyotish_desk_lib::varsha_commands::compute_varsha_blocking;
+        let now = Timestamp::from_second(1_795_600_800).expect("timestamp");
+        let place = MuhurtaPlace {
+            lat: module_input.lat,
+            lon: module_input.lon,
+            tz: opened.chart_record.birth.tz.clone(),
+        };
+        let view_of = |input: ChartInput, age: Option<u8>| {
+            compute_varsha_blocking(&engine, input, age, place.clone(), now).expect("varsha")
+        };
+        let current = view_of(module_input.clone(), None);
+        assert_eq!(
+            current.report.age, 26,
+            "the fixture clock is in the 27th year"
+        );
+        assert!(current.report.chart.is_some() && current.report.facts.is_some());
+        let next = view_of(module_input.clone(), Some(27));
+        assert_eq!(next.report.start_jd_ut, current.report.end_jd_ut);
+        let time_unknown = view_of(
+            ChartInput {
+                time_known: false,
+                ..module_input.clone()
+            },
+            None,
+        );
+        assert!(time_unknown.report.chart.is_none() && time_unknown.report.facts.is_none());
+        json!({
+            "current": current,
+            "next": next,
+            "time_unknown": time_unknown,
+            "mudda_order": MUDDA_ORDER,
+            "max_age": MAX_VARSHA_AGE,
+        })
+    };
     let yoga_catalog: Vec<Value> = YOGA_CATALOG
         .iter()
         .map(|r| json!({ "id": r.id, "group": r.group }))
@@ -595,6 +635,7 @@ fn ui_fixture_is_up_to_date() {
             "lajjitadi": Lajjitadi::ALL,
             "shayanadi": Shayanadi::ALL,
         },
+        "varsha": varsha,
         "compat": {
             "partner_uuid": partner.chart_record.uuid,
             "brief": partner_brief,
````

- [ ] **Step 2: Testlerin derlenmediğini gör**

Run: `cargo test -p jyotish-desk --test varsha --test ui_fixture 2>&1 | grep -E "^error" | sort | uniq -c | head`
Expected: derleme hatası: iki test dosyasında da `error[E0432]: unresolved import `jyotish_desk_lib::varsha_commands`` (modül henüz yok). Test koşmaz.

- [ ] **Step 3: Komutlar ve kayıt**

**Yeni dosya:** `app/src-tauri/src/varsha_commands.rs`

````rust
//! Varshaphala komutları (yol haritası §4.14, Plan 27): yıllık harita, muntha ve mudda dasa;
//! yıllık haritayı olay haritası olarak kaydetme. İleri modül komutlarının kalıbı: hesap motor
//! iş parçacığında, komut gövdesi `blocking` havuzunda, hata `CommandError`. Rapor `Chart`'a
//! eklenmez ve saklanmaz (K1); kayıt var olan olay haritası yolunu kullanır (`ChartKind::Event`,
//! `Store::add_chart`), veritabanı şeması değişmez (Y6). "Şimdi" Rust'ta okunur (C4).
//!
//! Girdi doğum verisidir, yer kişisel veridir: loglanmaz. Bu dosyanın ürettiği hatalar yalnızca
//! alan adı taşır (`invalid_input`).

use std::sync::{Arc, Mutex};

use atlas::{Timestamp, TzSpec};
use ephemeris::{Engine, SwissEph};
use jyotish_core::varsha::{VarshaReport, VarshaYear};
use jyotish_core::ChartInput;
use serde::Serialize;
use specta::Type;
use store::{
    BirthData, ChartKind, ChartRecord, FoldChoice, HomePlace, NewChart, Store, TimeCertainty,
    TzChoice,
};
use tauri::State;

use crate::client_commands::{blocking, lock, StoreState};
use crate::commands::CommandError;
use crate::muhurta_commands::MuhurtaPlace;

/// Kaydedilen yıllık haritanın etiketinin başı. Klasik terimdir: üç dilde aynı yazılır (K11).
pub const VARSHA_LABEL: &str = "Varshaphala";
const UNIX_EPOCH_JD: f64 = 2_440_587.5;

/// Rapor ve yıllık haritanın yerinin yıl başındaki UTC ofseti (anlar bu ofsetle gösterilir).
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct VarshaView {
    pub report: VarshaReport,
    pub offset_secs: i32,
}

fn invalid(field: &str) -> CommandError {
    CommandError::new("invalid_input", &[("field", field.to_owned())])
}

fn tz_spec(tz: &TzChoice) -> TzSpec {
    match tz {
        TzChoice::Iana(id) => TzSpec::Iana(id.trim().to_owned()),
        TzChoice::Fixed(secs) => TzSpec::Fixed(*secs),
        TzChoice::Lmt => TzSpec::Lmt,
    }
}

/// Jülyen gününün en yakın saniyesi.
fn timestamp_of(jd_ut: f64) -> Result<Timestamp, CommandError> {
    let secs = (jd_ut - UNIX_EPOCH_JD) * 86_400.0;
    if !(secs.is_finite() && secs.abs() < 1e13) {
        return Err(invalid("varsha.moment"));
    }
    Timestamp::from_second(secs.round() as i64).map_err(|_| invalid("varsha.moment"))
}

/// Saat diliminin `at` anındaki UTC ofseti; çözülemeyen dilim yalnızca alan adıyla döner.
fn offset_at(tz: &TzChoice, lon: f64, at: Timestamp) -> Result<i32, CommandError> {
    atlas::offset_at(&tz_spec(tz), lon, at).map_err(|_| invalid("varsha.place"))
}

/// `compute_varsha` gövdesi; `now` testte sabitlenir. `age` boşsa `now` anında süren yıl
/// hesaplanır. Bloklar.
pub fn compute_varsha_blocking(
    engine: &Engine,
    input: ChartInput,
    age: Option<u8>,
    place: MuhurtaPlace,
    now: Timestamp,
) -> Result<VarshaView, CommandError> {
    let year = match age {
        Some(age) => VarshaYear::Age(age),
        None => VarshaYear::At(now.as_second() as f64 / 86_400.0 + UNIX_EPOCH_JD),
    };
    let at = (place.lat, place.lon);
    let report = engine.call(move |se: &mut SwissEph| {
        jyotish_core::varsha::compute_varsha(se, &input, year, Some(at))
    })??;
    let offset_secs = offset_at(&place.tz, place.lon, timestamp_of(report.start_jd_ut)?)?;
    Ok(VarshaView {
        report,
        offset_secs,
    })
}

/// Varshaphala: `age` yaşının (tamamlanan yıl; `null` ise şu anda süren yılın) yıllık haritası,
/// munthası ve mudda dasası. `input` açık haritanın girdisidir; `place` yıllık haritanın yeridir
/// (doğum yeri ya da bulunulan yer) ve saat dilimi anların gösterileceği ofseti verir. Doğum
/// saati bilinmiyorsa (`time_known == false`) yıllık harita ve muntha `null` döner. Girdi ve yer
/// loglanmaz.
#[tauri::command]
#[specta::specta]
pub async fn compute_varsha(
    engine: State<'_, Arc<Engine>>,
    input: ChartInput,
    age: Option<u8>,
    place: MuhurtaPlace,
) -> Result<VarshaView, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || compute_varsha_blocking(&engine, input, age, place, Timestamp::now())).await
}

/// Dönüş anının `home` yerindeki duvar saatiyle doğum verisi. Duvar saati yaz saati örtüşmesine
/// düşüyorsa dönüş anını veren taraf seçilir; öteki her durumda katlama seçimi yazılmaz.
fn event_birth(home: &HomePlace, at: Timestamp) -> Result<BirthData, CommandError> {
    let offset = offset_at(&home.tz, home.place.lon, at)?;
    let local = Timestamp::from_second(at.as_second() + i64::from(offset))
        .map_err(|_| invalid("varsha.moment"))?;
    let birth = |fold: Option<FoldChoice>| BirthData {
        date: local.strftime("%Y-%m-%d").to_string(),
        time: Some(local.strftime("%H:%M:%S").to_string()),
        time_certainty: TimeCertainty::Exact,
        place: home.place.clone(),
        tz: home.tz.clone(),
        fold,
        notes: String::new(),
    };
    // Yerel ortalama saatte ofset saniyeye yuvarlanır: çözülen an bir saniyeye dek ayrışabilir.
    let hits = |fold: Option<FoldChoice>| {
        store::resolve_birth(&birth(fold))
            .ok()
            .and_then(|(_, r)| r.utc.parse::<Timestamp>().ok())
            .is_some_and(|t| (t.as_second() - at.as_second()).abs() <= 1)
    };
    [None, Some(FoldChoice::Earlier), Some(FoldChoice::Later)]
        .into_iter()
        .find(|fold| hits(*fold))
        .map(birth)
        .ok_or_else(|| invalid("varsha.moment"))
}

/// `save_varsha_chart` gövdesi; bloklar. Kaynak harita, ayarları ve (istenirse) bulunulan yer
/// veritabanından okunur: arayüzden yer ya da an alınmaz.
pub fn save_varsha_chart_blocking(
    engine: &Engine,
    store: &Mutex<Store>,
    chart_id: u32,
    age: u8,
    home: bool,
) -> Result<ChartRecord, CommandError> {
    let (source, settings, number, home_place) = {
        let s = lock(store)?;
        let source = s.chart(chart_id)?;
        let settings = s.calc_settings(chart_id)?.settings().clone();
        let number = s.prashna_meta(chart_id)?.and_then(|p| p.number);
        (source, settings, number, s.app_prefs()?.home_place)
    };
    // Doğum saati bilinmiyorsa yıllık harita hesaplanmaz (K2): kaydedilecek harita yoktur.
    if source.birth.time_certainty == TimeCertainty::Unknown {
        return Err(invalid("birth.time"));
    }
    let place = if home {
        home_place.ok_or_else(|| invalid("home_place"))?
    } else {
        HomePlace {
            place: source.birth.place.clone(),
            tz: source.birth.tz.clone(),
        }
    };
    let input = ChartInput {
        jd_ut: source.resolved.jd_ut,
        lat: source.birth.place.lat,
        lon: source.birth.place.lon,
        settings: settings.clone(),
        time_known: true,
        lagna_number: number,
    };
    let at = (place.place.lat, place.place.lon);
    let report = engine.call(move |se: &mut SwissEph| {
        jyotish_core::varsha::compute_varsha(se, &input, VarshaYear::Age(age), Some(at))
    })??;
    let birth = event_birth(&place, timestamp_of(report.start_jd_ut)?)?;
    let chart = NewChart {
        label: format!("{VARSHA_LABEL} {}", birth.date),
        kind: ChartKind::Event,
        birth,
    };
    let mut s = lock(store)?;
    let saved = s.add_chart(source.client_id, &chart)?;
    // Yeni harita genel varsayılanı alır; yıllık harita kaynak haritanın ayarlarıyla
    // hesaplanmıştır: aynı ayar yazılır ki kayıt açılınca aynı harita çıksın.
    s.save_calc_settings(saved.id, &settings)?;
    Ok(s.chart(saved.id)?)
}

/// `age` yaşının yıllık haritasını kaynak haritanın danışanına olay haritası olarak kaydeder
/// (`ChartKind::Event`; etiket `Varshaphala <tarih>`). Yer doğum yeridir; `home` doğruysa
/// tercihlerdeki bulunulan yerdir. Kaynak haritanın ayarları yeni haritaya kopyalanır. Harita
/// açılmaz: dönen kayıt yalnızca bildirilir. Doğum saati bilinmeyen harita ve seçilmemiş
/// bulunulan yer `invalid_input` ile reddedilir.
#[tauri::command]
#[specta::specta]
pub async fn save_varsha_chart(
    engine: State<'_, Arc<Engine>>,
    store: State<'_, StoreState>,
    chart_id: u32,
    age: u8,
    home: bool,
) -> Result<ChartRecord, CommandError> {
    let (engine, store) = (Arc::clone(&engine), Arc::clone(&store));
    blocking(move || save_varsha_chart_blocking(&engine, &store, chart_id, age, home)).await
}
````

Kayıt (paylaşılan dosya; üç satır):

````diff
diff --git a/app/src-tauri/src/lib.rs b/app/src-tauri/src/lib.rs
index e704350..bda14e8 100644
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -12,6 +12,7 @@ pub mod search_commands;
 pub mod store_commands;
 pub mod transit_commands;
 pub mod update_commands;
+pub mod varsha_commands;
 
 use std::sync::{Arc, Mutex};
 
@@ -36,6 +37,8 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             bhava_commands::compute_bhava_table,
             bhava_commands::compute_sudarshana,
             commands::dasha_children,
+            varsha_commands::compute_varsha,
+            varsha_commands::save_varsha_chart,
             commands::print_window,
             avastha_commands::compute_avasthas,
             module_commands::compute_drishti,
````

- [ ] **Step 4: Rust testleri; bağlamaları ve fikstürü yeniden üret (C10)**

```bash
cargo fmt --all
cargo test -p jyotish-desk --test varsha -- --nocapture 2>&1 | grep -E "^test result|compute_varsha median"
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep -E "^test result"
cargo test -p jyotish-desk --test bindings --test ui_fixture 2>&1 | grep -E "^test result"
git status --short
git diff --stat -- app/ui/bindings.ts app/ui/test/fixtures/delhi.json
grep -cE "computeVarsha|saveVarshaChart" app/ui/bindings.ts
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
```
Expected: `varsha` `1 passed` ve ölçüm satırı (`compute_varsha median …`; bu plan yazılırken 2,0 ms, bütçe 20 ms); iki üretim koşusu da `ok`; yeniden üretimden sonraki denetim koşusunda `bindings` `1 passed` ve `ui_fixture` `2 passed`; `git status --short` altı dosya gösterir (`app/src-tauri/src/lib.rs`, `app/src-tauri/tests/ui_fixture.rs`, `app/ui/bindings.ts`, `app/ui/test/fixtures/delhi.json` değişmiş; `app/src-tauri/src/varsha_commands.rs`, `app/src-tauri/tests/varsha.rs` yeni); `bindings.ts` 89 satır kazanır, `delhi.json` tek satırı değişir; `grep -c` `2`; yasaklı dizi sayacı `0`. `tests/varsha.rs`'teki bir beklenti tutmazsa önce yorumdaki türetme okunur (özellikle kaydın tarihi: Global Constraints "El örnekleri"); beklenti motor çıktısına bakarak değiştirilmez.

- [ ] **Step 5: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
```
Expected: fmt ve clippy çıktısız; sayaç `0`; Vitest `97 passed (97)`, `973 passed (973)` (taban: arayüz henüz yeni bağlamaları kullanmıyor); `✓ built`.

- [ ] **Step 6: Commit**

```bash
git add app/src-tauri/src/varsha_commands.rs app/src-tauri/src/lib.rs app/src-tauri/tests/varsha.rs app/src-tauri/tests/ui_fixture.rs app/ui/bindings.ts app/ui/test/fixtures/delhi.json
git diff --cached --stat | tail -1
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(app): compute_varsha and save_varsha_chart commands, bindings and the UI fixture" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: `6 files changed, 879 insertions(+), 1 deletion(-)`; tarama çıktısız; `git status --short` boş.

---

### Task 6: Arayüz: Varshaphala paneli (yıl seçici, yer, yılın olguları, yıllık harita, muntha, kaydetme) ve metinleri

Yol haritası §5 Plan 27 görev (6) ("Varshaphala paneli: yıl seçici, harita, muntha") ve (7) ("i18n, saat bilinmiyor, olay haritası olarak kaydetme"); K11, K12; D7, D10, D14–D16, D18, D20. Arayüz hesap yapmaz: her değer Task 5'in komutlarından gelir.

**Files:**
- Create: `app/ui/modules/useVarsha.ts`, `app/ui/varsha/yearStore.ts`, `app/ui/varsha/YearControls.tsx`, `app/ui/varsha/zone.ts`, `app/ui/panels/varsha.css`, `app/ui/panels/VarshaPanel.tsx`, `app/ui/panels/VarshaPanel.test.tsx`
- Modify (hepsi paylaşılan dosya): `app/ui/api.ts`, `app/ui/api.test.ts`, `app/ui/test/fixture.ts`, `app/ui/layout/model.ts`, `app/ui/layout/model.test.ts`, `app/ui/panels/registry.ts`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`
- Test: `VarshaPanel.test.tsx` (17); `api.test.ts` ve `model.test.ts`'in var olan testleri yeni satırlarla

**Interfaces:**
- Consumes (hepsi var): bağlamalardan `VarshaView`, `Chart`, `HomePlace`, `AppPrefs`, `OpenedClient`, `Sign`, `CommandError` (Task 5); `modules/useModule` (`useModule`, `useLastData`), `modules/input` (`chartInput`, `inputKey`), `modules/muhurta` (`chartPlace`, `muhurtaPlace`, `placeLabel`); `chart/adapter` (`chartDataFromChart`, `signIndex`), `chart/ChartSvg`, `chart/types` (`CHART_STYLES`, `isChartStyle`, `RETRO_MARK`, `showsRetro`, `ChartData`); `highlight/store`; `session/store` (`useSession`: `opened`, `touch`); `settings/prefs` (`usePrefs`); `view/store` (`useLayoutStore`); `panels/bala` (`BalaScroll`), `panels/format` (`formatDegree`, `formatJdMinute`), `panels/labels` (`chartLabels`, `nakshatraName`, `VARA_NAMES`), `panels/PinButton`, `panels/states`, `panels/HomePlaceDialog`; `birth/model` (`formatFixedOffset`); var olan i18n anahtarları `panel.noChart`, `chart.style`, `chartStyle.*`, `chart.retro`, `table.lagnaRow`, `badgeHint.LowPrecision`, `muhurta.place.pickHome`, `muhurta.place.changeHome`, `grahaName.*`, `sign.*`.
- Produces:
  - `api.computeVarsha(input, age: number | null, place: MuhurtaPlace)`, `api.saveVarshaChart(chartId: number, age: number, home: boolean)`.
  - `varsha/yearStore.ts`: `MAX_VARSHA_AGE = 120`, `isVarshaAge(age)`, `useVarshaAge(chartId): { age: number | null; setAge(age: number | null): void }`, `resetVarshaAge()` (testler için).
  - `varsha/YearControls.tsx`: `VarshaYearControls({ age, current, onChange })`. `varsha/zone.ts`: `zoneLabel(offsetSecs)` → `"UTC+05:30"`.
  - `modules/useVarsha.ts`: `VARSHA_PLACES = ["birth", "home"]`, `varshaPlaceChoice(settings)`, `varshaPlace(choice, opened, prefs): HomePlace | null`, `useVarsha(opened, age, place): ModuleState<VarshaView>`.
  - `panels/VarshaPanel.tsx`: `VarshaPanel`, `MUNTHA_MARK = "Mu"`. Panel türü `varsha` (`MODULE_PANELS`: 7 × 24). Panel ayarları: `place` (`"birth"` | `"home"`), `style` (harita biçimi).
  - i18n: `panel.varsha` ve `varsha` bloğu (üç dilde; `varsha.mudda.*` Task 7'de kullanılır). Fikstür tipi: `UiFixture.varsha`.

- [ ] **Step 1: Arayüz testleri: panel, sarmalayıcılar, katalog; fikstür tipi**

Panel testleri (17): süren yılın doğum yeriyle istenmesi ve olgular tablosu; yıllık haritanın çizimi, muntha işareti ve tablo karşılığı; yıl adımları ve "bu yıl"; yaş alanı ve sınırları; ortak yıl deposu; yer seçimi (bulunulan yer yokken açıklama; varken o yerle istek ve kayıt); kaydetme (bir kez, bildirim, liste yenilemesi, yıl değişince sıfırlanma) ve kayıt hatası; doğum saati bilinmeyen harita; düşük hassasiyet; açık harita yok; hata ve yeniden deneme (odak dahil); yükleme evreleri; ortak vurgu ve kaydırma kapları; kayıt, katalog ve üç dildeki adlar.

**Yeni dosya:** `app/ui/panels/VarshaPanel.test.tsx`

````tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeVarsha: vi.fn(), saveVarshaChart: vi.fn(), setAppPrefs: vi.fn(), searchPlaces: vi.fn() } }));

import { api, type Result } from "../api";
import type { AppPrefs, HomePlace, VarshaView } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import { MODULE_PANELS, type Panel } from "../layout/model";
import { useSession } from "../session/store";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { MAX_VARSHA_AGE, resetVarshaAge, useVarshaAge } from "../varsha/yearStore";
import { zoneLabel } from "../varsha/zone";
import { useLayoutStore } from "../view/store";
import { PANEL_REGISTRY } from "./registry";
import { MUNTHA_MARK, VarshaPanel } from "./VarshaPanel";

const FX = fixture();
const ok = (view: VarshaView = fixture().varsha.current): Result<VarshaView> => ({ status: "ok", data: view });
const failure = { status: "error" as const, error: { code: "calc", params: { detail: "x" } } };
const DELHI = { lat: 28.6139, lon: 77.209, tz: { Iana: "Asia/Kolkata" } };
const SYDNEY: HomePlace = { place: { name: "Sydney", country: "Australia", admin1: "New South Wales", lat: -33.8688, lon: 151.2093, geoname_id: null }, tz: { Iana: "Australia/Sydney" } };
const LAYOUT = (settings: Record<string, unknown>) => {
  const panel: Panel = { id: "vp", kind: "varsha", x: 0, y: 0, w: 7, h: 24, settings };
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
};

let revision = 0;
/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  revision = useSession().revision;
  return <VarshaPanel panel={active.panels[0]} />;
}

interface Options {
  opened?: ReturnType<typeof fixture>["opened"] | null;
  prefs?: AppPrefs | null;
}
function renderPanel(settings: Record<string, unknown> = {}, { opened = fixture().opened, prefs = FX.prefs.prefs }: Options = {}) {
  return renderWithStores(<LivePanel />, { layout: LAYOUT(settings), opened, prefs });
}

const facts = () => screen.findByRole("table", { name: "Varshaphala: yılın olguları" });
const positions = () => screen.getByRole("table", { name: "Yıllık harita konumları" });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const rows = (table: HTMLElement) => within(table).getAllByRole("row").slice(1).map(cells);
const ageInput = () => screen.getByRole("spinbutton", { name: "Yaş (tamamlanan yıl)" });
const savedSettings = (persistence: ReturnType<typeof renderPanel>["persistence"]) =>
  persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings;

beforeEach(() => {
  resetVarshaAge();
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  // Fikstürde iki yıl vardır: 27 yaşı `next`, öteki her istek süren yıl (26).
  vi.mocked(api.computeVarsha).mockImplementation(async (_input, age) => ok(age === 27 ? fixture().varsha.next : fixture().varsha.current));
  vi.mocked(api.saveVarshaChart).mockImplementation(async () => ({ status: "ok", data: { ...fixture().opened.chart_record, id: 31, kind: "Event", label: "Varshaphala 2026-01-01" } }));
  vi.mocked(api.setAppPrefs).mockResolvedValue({ status: "ok", data: null });
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "ok", data: FX.places });
});

test("asks for the running year at the birth place and lists the facts of the year", async () => {
  renderPanel();
  const table = await facts();
  const opened = fixture().opened;
  expect(api.computeVarsha).toHaveBeenCalledTimes(1);
  // Yaş gönderilmez: süren yılı Rust bulur (arayüz saat okumaz). Yer doğum yeridir.
  expect(api.computeVarsha).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null }, null, DELHI);
  expect(within(table).getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Olgu", "Değer"]);
  // Delhi fikstürü, 2026-11-25'te: 26 yıl tamamlanmış. Doğum lagnası İkizler; 26 = 2 × 12 + 2:
  // muntha Aslan. Yıllık lagna Oğlak: Aslan, Oğlak'tan 8. evdir.
  expect(rows(table)).toEqual([
    ["Yıl", "27. yıl (tamamlanan yıl 26)"],
    ["Dönüş anı", "2026-01-01 09:28 (UTC+05:30)"],
    ["Yıl sonu (sonraki dönüş)", "2027-01-01 15:41 (UTC+05:30)"],
    ["Harita yeri", "New Delhi, Delhi, India"],
    ["Vara", "Guruvara"],
    ["Yıl lagnası", "Oğl 22°04′ · lordu Satürn"],
    ["Muntha", "Asl · yıllık haritada 8. ev · doğum haritasında 3. ev · lordu Güneş"],
    ["Doğum lagnası", "İki · lordu Merkür"],
    ["Mudda dasanın ilk lordu", "Mars"],
  ]);
  expect(ageInput()).toHaveValue(26);
  expect(screen.getByRole("button", { name: "Bu yıl" })).toHaveAttribute("aria-pressed", "true");
  expect(screen.getByRole("combobox", { name: "Yer" })).toHaveValue("birth");
  expect(screen.getByText(/^Dönüş her kipte sidereal boylamdandır/)).toBeInTheDocument();
  expect(screen.getByText("Yöntem: varsha.sidereal_return_v1")).toBeInTheDocument();
  expect(screen.queryByText(/Doğum saati bilinmiyor/)).toBeNull();
});

test("draws the annual chart with the muntha mark and gives its table equivalent", async () => {
  const { container, persistence } = renderPanel();
  await facts();
  const svg = screen.getByRole("group", { name: "Yıllık harita, 27. yıl" });
  expect(svg).toHaveAttribute("data-style", "north");
  // Yıllık haritanın dokuz grahası ve muntha işareti (Aslan).
  expect(svg.querySelectorAll("[data-graha]")).toHaveLength(9);
  expect(container.querySelector(".chart-mark")).toHaveTextContent(MUNTHA_MARK);
  expect(screen.getByText("Muntha Asl burcunda (çizimde “Mu”).")).toHaveClass("visually-hidden");
  const table = rows(positions());
  expect(table).toHaveLength(10);
  expect(table[0]).toEqual(["Lagna", "Oğl", "22°04′", "Shravana", "1"]);
  expect(table[1]).toEqual(["Güneş", "Yay", "16°30′", "Purva Ashadha", "12"]);
  // Retro: görünen "R", okunan sözcük. Düğümlerde işaret yoktur.
  expect(table[5]).toEqual(["Jüpiter Rretro", "İki", "27°06′", "Punarvasu", "6"]);
  expect(table[8][0]).toBe("Rahu");
  // Biçim panelin ayarıdır ve düzenle saklanır.
  fireEvent.change(screen.getByRole("combobox", { name: "Biçim" }), { target: { value: "south" } });
  expect(screen.getByRole("group", { name: "Yıllık harita, 27. yıl" })).toHaveAttribute("data-style", "south");
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ style: "south" }));
  // Biçim değişimi raporu yeniden istemez.
  expect(api.computeVarsha).toHaveBeenCalledTimes(1);
});

test("stepping the year asks Rust for that age; this-year goes back to the running year", async () => {
  renderPanel();
  await facts();
  fireEvent.click(screen.getByRole("button", { name: "Sonraki yıl" }));
  expect(ageInput()).toHaveValue(27);
  // Yeni yıl gelene dek önceki yılın tablosu yerinde durur.
  expect(screen.getByRole("table", { name: "Varshaphala: yılın olguları" })).toBeInTheDocument();
  await waitFor(() => expect(rows(screen.getByRole("table", { name: "Varshaphala: yılın olguları" }))[0]).toEqual(["Yıl", "28. yıl (tamamlanan yıl 27)"]));
  expect(vi.mocked(api.computeVarsha).mock.calls.map((c) => c[1])).toEqual([null, 27]);
  expect(screen.getByRole("button", { name: "Bu yıl" })).toHaveAttribute("aria-pressed", "false");
  // 27 yaşı: muntha Başak, yıl lagnası Boğa; mudda dasa Rahu ile başlar.
  const table = rows(screen.getByRole("table", { name: "Varshaphala: yılın olguları" }));
  expect(table[6]).toEqual(["Muntha", "Baş · yıllık haritada 5. ev · doğum haritasında 4. ev · lordu Merkür"]);
  expect(table[8]).toEqual(["Mudda dasanın ilk lordu", "Rahu"]);
  expect(screen.getByRole("group", { name: "Yıllık harita, 28. yıl" })).toBeInTheDocument();

  fireEvent.click(screen.getByRole("button", { name: "Önceki yıl" }));
  expect(ageInput()).toHaveValue(26);
  await waitFor(() => expect(vi.mocked(api.computeVarsha).mock.calls.map((c) => c[1])).toEqual([null, 27, 26]));
  fireEvent.click(screen.getByRole("button", { name: "Bu yıl" }));
  expect(screen.getByRole("button", { name: "Bu yıl" })).toHaveAttribute("aria-pressed", "true");
  // Kanca her anahtar değişiminde ister: dördüncü istek yaşsızdır.
  await waitFor(() => expect(vi.mocked(api.computeVarsha).mock.calls.map((c) => c[1])).toEqual([null, 27, 26, null]));
});

test("the age field takes a typed age and ignores values outside 0…120", async () => {
  renderPanel();
  await facts();
  fireEvent.change(ageInput(), { target: { value: "45" } });
  expect(ageInput()).toHaveValue(45);
  await waitFor(() => expect(vi.mocked(api.computeVarsha).mock.calls.at(-1)?.[1]).toBe(45));
  const before = vi.mocked(api.computeVarsha).mock.calls.length;
  for (const bad of ["121", "-1", "2.5", ""]) fireEvent.change(ageInput(), { target: { value: bad } });
  expect(ageInput()).toHaveValue(45);
  expect(api.computeVarsha).toHaveBeenCalledTimes(before);
  // Uçlarda adım düğmeleri kapalıdır (odağı tutar, yılı değiştirmez).
  fireEvent.change(ageInput(), { target: { value: "0" } });
  expect(screen.getByRole("button", { name: "Önceki yıl" })).toHaveAttribute("aria-disabled", "true");
  fireEvent.click(screen.getByRole("button", { name: "Önceki yıl" }));
  expect(ageInput()).toHaveValue(0);
  fireEvent.change(ageInput(), { target: { value: String(MAX_VARSHA_AGE) } });
  expect(screen.getByRole("button", { name: "Sonraki yıl" })).toHaveAttribute("aria-disabled", "true");
  fireEvent.click(screen.getByRole("button", { name: "Sonraki yıl" }));
  expect(ageInput()).toHaveValue(MAX_VARSHA_AGE);
  expect(MAX_VARSHA_AGE).toBe(FX.varsha.max_age);
});

test("the selected year belongs to the open chart and is shared between consumers", async () => {
  const seen: Record<string, number | null> = {};
  function Probe({ name, chartId }: { name: string; chartId: number }) {
    seen[name] = useVarshaAge(chartId).age;
    return null;
  }
  const chartId = fixture().opened.chart_record.id;
  renderWithStores(
    <>
      <LivePanel />
      <Probe name="same" chartId={chartId} />
      <Probe name="other" chartId={chartId + 1} />
    </>,
    { layout: LAYOUT({}) },
  );
  await facts();
  expect(seen).toEqual({ same: null, other: null });
  fireEvent.click(screen.getByRole("button", { name: "Sonraki yıl" }));
  await waitFor(() => expect(seen).toEqual({ same: 27, other: null }));
});

test("the current place is a panel setting; without one the panel says so and asks nothing", async () => {
  const { persistence } = renderPanel({}, { prefs: { ...FX.prefs.prefs, home_place: null } });
  await facts();
  fireEvent.change(screen.getByRole("combobox", { name: "Yer" }), { target: { value: "home" } });
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ place: "home" }));
  expect(screen.getByText("Bulunulan yer seçilmedi: yıllık harita için bir yer seçin ya da “Doğum yeri”ne dönün.")).toBeInTheDocument();
  expect(screen.queryByRole("table")).toBeNull();
  expect(api.computeVarsha).toHaveBeenCalledTimes(1);
  // Bulunulan yer penceresi buradan açılır.
  fireEvent.click(screen.getByRole("button", { name: "Bulunulan yeri seç" }));
  expect(screen.getByRole("dialog", { name: "Bulunulan yer" })).toBeInTheDocument();
});

test("with a current place the annual chart is asked for that place and saved there", async () => {
  renderPanel({ place: "home" }, { prefs: { ...FX.prefs.prefs, home_place: SYDNEY } });
  const table = await facts();
  expect(vi.mocked(api.computeVarsha).mock.calls[0][2]).toEqual({ lat: -33.8688, lon: 151.2093, tz: { Iana: "Australia/Sydney" } });
  expect(rows(table)[3]).toEqual(["Harita yeri", "Sydney, New South Wales, Australia"]);
  expect(screen.getByRole("button", { name: "Bulunulan yeri değiştir" })).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Olay haritası olarak kaydet" }));
  await screen.findByText("Olay haritası kaydedildi: Varshaphala 2026-01-01");
  expect(api.saveVarshaChart).toHaveBeenCalledWith(fixture().opened.chart_record.id, 26, true);
});

test("saving stores the annual chart once, reports its label and refreshes the lists", async () => {
  renderPanel();
  await facts();
  const before = revision;
  const save = screen.getByRole("button", { name: "Olay haritası olarak kaydet" });
  expect(screen.getByRole("status")).toHaveTextContent("");
  fireEvent.click(save);
  expect(await screen.findByText("Olay haritası kaydedildi: Varshaphala 2026-01-01")).toBe(screen.getByRole("status"));
  expect(api.saveVarshaChart).toHaveBeenCalledTimes(1);
  expect(api.saveVarshaChart).toHaveBeenCalledWith(fixture().opened.chart_record.id, 26, false);
  // Harita açılmaz; listeler yenilenir.
  expect(revision).toBe(before + 1);
  // Aynı yıl ikinci kez kaydedilmez.
  expect(save).toHaveAttribute("aria-disabled", "true");
  fireEvent.click(save);
  expect(api.saveVarshaChart).toHaveBeenCalledTimes(1);
  // Başka yıla geçince bildirim o yıla taşınmaz ve kayıt yeniden yapılabilir.
  fireEvent.click(screen.getByRole("button", { name: "Sonraki yıl" }));
  await screen.findByRole("group", { name: "Yıllık harita, 28. yıl" });
  expect(screen.getByRole("status")).toHaveTextContent("");
  expect(screen.getByRole("button", { name: "Olay haritası olarak kaydet" })).not.toHaveAttribute("aria-disabled");
});

test("a failed save is reported in the panel and can be tried again", async () => {
  vi.mocked(api.saveVarshaChart).mockResolvedValueOnce({ status: "error", error: { code: "store_sqlite", params: { detail: "x" } } });
  renderPanel();
  await facts();
  const save = screen.getByRole("button", { name: "Olay haritası olarak kaydet" });
  fireEvent.click(save);
  expect(await screen.findByRole("alert")).toBeInTheDocument();
  expect(save).not.toHaveAttribute("aria-disabled");
  fireEvent.click(save);
  await screen.findByText("Olay haritası kaydedildi: Varshaphala 2026-01-01");
  expect(screen.queryByRole("alert")).toBeNull();
  expect(api.saveVarshaChart).toHaveBeenCalledTimes(2);
});

test("unknown birth time is sent to Rust; there is no annual chart, no muntha and nothing to save", async () => {
  vi.mocked(api.computeVarsha).mockResolvedValue(ok(fixture().varsha.time_unknown));
  renderPanel({}, { opened: unknownTimeOpened() });
  const table = await facts();
  expect(vi.mocked(api.computeVarsha).mock.calls[0][0].time_known).toBe(false);
  expect(screen.getByText("Doğum saati bilinmiyor: yıllık harita ve muntha hesaplanmadı (ikisi de lagna ister). Dönüş anı ve mudda dasa yaklaşıktır (≈).")).toBeInTheDocument();
  expect(rows(table).map((r) => r[0])).toEqual(["Yıl", "Dönüş anı", "Yıl sonu (sonraki dönüş)", "Mudda dasanın ilk lordu"]);
  expect(screen.getAllByRole("table")).toHaveLength(1);
  expect(screen.queryByRole("group", { name: /Yıllık harita/ })).toBeNull();
  expect(screen.queryByRole("button", { name: "Olay haritası olarak kaydet" })).toBeNull();
  expect(screen.queryByRole("combobox", { name: "Biçim" })).toBeNull();
});

test("a low-precision warning from the report is shown as text", async () => {
  const view = fixture().varsha.current;
  view.report.warnings = ["LowPrecision"];
  vi.mocked(api.computeVarsha).mockResolvedValue(ok(view));
  renderPanel();
  await facts();
  expect(screen.getByText("Güneş veya Ay yedek ephemeris ile hesaplandı.")).toBeInTheDocument();
});

test("without an open chart nothing is requested", () => {
  renderPanel({}, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeVarsha).not.toHaveBeenCalled();
});

test("an error is shown in the panel, the year controls stay, and retry asks again", async () => {
  vi.mocked(api.computeVarsha).mockResolvedValueOnce(failure);
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.queryByRole("table")).toBeNull();
  expect(screen.getByRole("button", { name: "Bu yıl" })).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await facts();
  expect(api.computeVarsha).toHaveBeenCalledTimes(2);
  expect(screen.queryByRole("alert")).toBeNull();
});

test("retry from the keyboard keeps the focus in the panel while the error box is replaced", async () => {
  vi.mocked(api.computeVarsha).mockResolvedValueOnce(failure);
  renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Varshaphala">
      <LivePanel />
    </section>,
    { layout: LAYOUT({}) },
  );
  const retry = await screen.findByRole("button", { name: "Yeniden dene" });
  retry.focus();
  fireEvent.click(retry);
  await facts();
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Varshaphala" }));
});

test("loading: nothing under a second, then a skeleton, then the tables", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<VarshaView>) => void;
    vi.mocked(api.computeVarsha).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderPanel();
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    act(() => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    act(() => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    await act(async () => resolve(ok()));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getAllByRole("table")).toHaveLength(2);
  } finally {
    vi.useRealTimers();
  }
});

test("shared highlight: a pinned graha marks its row; both tables scroll by keyboard", async () => {
  renderPanel();
  await facts();
  const saturn = within(positions()).getByRole("button", { name: "Satürn" });
  fireEvent.click(saturn);
  expect(saturn).toHaveAttribute("aria-pressed", "true");
  expect(positions().querySelectorAll("tr.is-highlighted")).toHaveLength(1);
  expect(positions().querySelector('tr[data-graha="Saturn"]')).toHaveClass("is-highlighted");
  for (const name of ["Varshaphala: yılın olguları", "Yıllık harita konumları"]) {
    expect(screen.getByRole("group", { name })).toHaveAttribute("tabindex", "0");
  }
});

test("the panel is registered, opens from the palette list, and its texts exist in three languages", () => {
  expect(PANEL_REGISTRY.varsha.component).toBe(VarshaPanel);
  expect(MODULE_PANELS.find((m) => m.kind === "varsha")).toEqual({ kind: "varsha", w: 7, h: 24 });
  // Klasik terimler üç dilde aynı yazılır (K11).
  for (const lang of [tr, en, fr]) {
    expect(lang.panel.varsha).toBe("Varshaphala");
    expect(lang.varsha.muntha).toBe("Muntha");
    expect(lang.varsha.vara).toBe("Vara");
  }
  expect([tr.varsha.mudda.system, en.varsha.mudda.system, fr.varsha.mudda.system]).toEqual(["Mudda dasa", "Mudda dasha", "Mudda dasha"]);
  // Fransızca metinler tipografik kesme işareti kullanır (C12).
  expect(JSON.stringify(fr.varsha)).not.toContain("'");
  expect(zoneLabel(19_800)).toBe("UTC+05:30");
  expect(zoneLabel(-12_600)).toBe("UTC-03:30");
  expect(zoneLabel(0)).toBe("UTC+00:00");
});
````

Sarmalayıcı tablosu, katalog beklentileri ve fikstür tipi (paylaşılan dosyalar):

````diff
diff --git a/app/ui/api.test.ts b/app/ui/api.test.ts
index 3f5d97b..92d5203 100644
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -41,6 +41,8 @@ vi.mock("./bindings", () => ({
     computeVimshopaka: vi.fn(),
     computeYogas: vi.fn(),
     computeAvasthas: vi.fn(),
+    computeVarsha: vi.fn(),
+    saveVarshaChart: vi.fn(),
     computeCompat: vi.fn(),
     chartInput: vi.fn(),
     chartBrief: vi.fn(),
@@ -151,6 +153,8 @@ test("every wrapper forwards its arguments to the command of the same name", asy
     ["computeVimshopaka", [{ jd_ut: 1, time_known: true }]],
     ["computeYogas", [{ jd_ut: 1, time_known: false }]],
     ["computeAvasthas", [{ jd_ut: 1, time_known: false }]],
+    ["computeVarsha", [{ jd_ut: 1, time_known: true }, null, { lat: 28.6, lon: 77.2, tz: "Lmt" }]],
+    ["saveVarshaChart", [3, 26, false]],
     ["computeCompat", [{ jd_ut: 1, time_known: true }, { jd_ut: 2, time_known: false }]],
     ["chartInput", ["00000000-0000-4000-8000-000000000012"]],
     ["chartBrief", ["00000000-0000-4000-8000-000000000012"]],
diff --git a/app/ui/layout/model.test.ts b/app/ui/layout/model.test.ts
index 4fc9d72..d81e501 100644
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -51,6 +51,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
     "sudarshana",
     "yogas",
     "avastha",
+    "varsha",
     "compat",
     "muhurta_day",
     "muhurta_search",
@@ -65,7 +66,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
 });
 
 test("module panels are known kinds, listed once, with a size the grid accepts", () => {
-  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "bhava_table", "sudarshana", "yogas", "avastha", "compat", "muhurta_day", "muhurta_search", "prashna_facts", "event_search", "navatara", "vimshopaka"]);
+  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "bhava_table", "sudarshana", "yogas", "avastha", "varsha", "compat", "muhurta_day", "muhurta_search", "prashna_facts", "event_search", "navatara", "vimshopaka"]);
   for (const m of MODULE_PANELS) {
     expect(isKnownPanelKind(m.kind)).toBe(true);
     expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
diff --git a/app/ui/test/fixture.ts b/app/ui/test/fixture.ts
index c5d6cf8..b6ddb70 100644
--- a/app/ui/test/fixture.ts
+++ b/app/ui/test/fixture.ts
@@ -56,6 +56,7 @@ import type {
   TithiGroup,
   TransitView,
   Varga,
+  VarshaView,
   VimshopakaReport,
   YogaGroup,
   YogaReport,
@@ -127,6 +128,13 @@ export interface UiFixture {
   avasthas_time_unknown: AvasthaReport;
   /** Rust `Baladi::ALL`, `Jagradadi::ALL`, `Lajjitadi::ALL` ve `Shayanadi::ALL`. */
   avastha_ids: { baladi: Baladi[]; jagradadi: Jagradadi[]; lajjitadi: Lajjitadi[]; shayanadi: Shayanadi[] };
+  /**
+   * Varshaphala (Plan 27): açık haritanın girdisiyle, doğum yerinde ve fikstürün sabit saatinde
+   * `compute_varsha` dönüşleri. `current` yaş verilmeden (süren yıl: 26), `next` 27 yaşıyla,
+   * `time_unknown` `time_known: false` ile (yıllık harita ve muntha yok). `mudda_order` Rust
+   * `MUDDA_ORDER`, `max_age` Rust `MAX_VARSHA_AGE`.
+   */
+  varsha: { current: VarshaView; next: VarshaView; time_unknown: VarshaView; mudda_order: Graha[]; max_age: number };
   /**
    * Uyum: ikinci danışan ("Eş Danışan", İstanbul 1987-03-17) eş haritadır. `input` ve `brief`
    * store'un `chart_input` ve `chart_brief` dönüşleridir. `report` açık harita damat rolündeyken,
````

- [ ] **Step 2: Testlerin düştüğünü gör**

Run: `pnpm --dir app test 2>&1 | grep -E "Test Files|Tests |FAIL" | sort | uniq`
Expected: `Test Files  3 failed | 95 passed (98)`, `Tests  3 failed | 970 passed (973)`: `ui/panels/VarshaPanel.test.tsx` yüklenemez (içe aktardığı modüller henüz yok), `api.test.ts`'in sarmalayıcı testi ve `model.test.ts`'in iki katalog testi düşer.

- [ ] **Step 3: Sarmalayıcılar, yıl deposu, yıl seçici, kanca, panel, katalog ve metinler**

Sarmalayıcılar (paylaşılan dosya):

````diff
diff --git a/app/ui/api.ts b/app/ui/api.ts
index 8f186b3..94b2a8c 100644
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -117,6 +117,11 @@ export const api = {
   // Avasthalar: doğum saati bilinmiyorsa rapor yine döner; Lajjita değerlendirilmemiş, Shayanadi
   // hesaplanmamıştır (`time_known: false`). O gün gün doğumu yoksa yalnızca Shayanadi boştur.
   computeAvasthas: (input: ChartInput) => wrap(() => commands.computeAvasthas(input)),
+  // Varshaphala (Plan 27): `age` tamamlanan yıldır, `null` ise süren yılı Rust bulur; `place`
+  // yıllık haritanın yeridir. Kayıt komutu yeri ve anı veritabanından okur: yalnızca kaynak
+  // haritanın kimliği, yaş ve "bulunulan yer" seçimi gider. Girdi ve yer loglanmaz.
+  computeVarsha: (input: ChartInput, age: number | null, place: MuhurtaPlace) => wrap(() => commands.computeVarsha(input, age, place)),
+  saveVarshaChart: (chartId: number, age: number, home: boolean) => wrap(() => commands.saveVarshaChart(chartId, age, home)),
   // Uyum: `groom` damat rolünün, `bride` gelin rolünün girdisidir. Eş haritanın girdisi ve kısa
   // kimliği store'dan okunur; harita yoksa (silinmiş eş) ikisi de `null` döner. Girdiler doğum
   // verisidir, eş bağı ilişki bilgisidir: loglanmaz.
````

**Yeni dosya:** `app/ui/varsha/yearStore.ts`

````ts
// Seçili Varshaphala yılı: Varshaphala paneli ile dasa ağacının mudda görünümü aynı yılı
// gösterir. Seçim açık haritaya aittir ve yalnızca bellekte durur (yol haritası Y6: yeni
// kalıcı veri yok): başka harita açılınca ya da uygulama yeniden başlayınca "bu yıl"a döner.
// `null` "bu yıl"dır: süren yılı Rust bulur, arayüz saat okumaz (C4).
import { useCallback, useSyncExternalStore } from "react";

/** Rust `MAX_VARSHA_AGE`: istenebilen en büyük yaş (tamamlanan yıl). */
export const MAX_VARSHA_AGE = 120;

let selected: { chartId: number; age: number } | null = null;
const listeners = new Set<() => void>();

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function write(next: { chartId: number; age: number } | null): void {
  selected = next;
  for (const listener of listeners) listener();
}

/** Yaş istenebilir mi: 0…`MAX_VARSHA_AGE` arasında bir tamsayı. */
export function isVarshaAge(age: number): boolean {
  return Number.isInteger(age) && age >= 0 && age <= MAX_VARSHA_AGE;
}

export interface VarshaAge {
  /** Seçili yaş; `null` "bu yıl"dır. */
  age: number | null;
  /** Geçersiz yaş yok sayılır; `null` "bu yıl"a döner. */
  setAge(age: number | null): void;
}

/** `chartId` haritasının seçili yılı; seçim başka bir haritaya aitse "bu yıl". */
export function useVarshaAge(chartId: number): VarshaAge {
  const age = useSyncExternalStore(subscribe, () => (selected !== null && selected.chartId === chartId ? selected.age : null));
  const setAge = useCallback(
    (next: number | null) => {
      if (next === null) write(null);
      else if (isVarshaAge(next)) write({ chartId, age: next });
    },
    [chartId],
  );
  return { age, setAge };
}

/** Testler için: seçimi unutur. */
export function resetVarshaAge(): void {
  write(null);
}
````

**Yeni dosya:** `app/ui/varsha/zone.ts`

````ts
// Varshaphala anlarının gösterildiği ofsetin yazımı. Ofset Rust'tan gelir (yıllık haritanın
// yerinin yıl başındaki UTC ofseti); burada yalnızca biçimlenir.
import { formatFixedOffset } from "../birth/model";

/** "UTC+05:30", "UTC-03:30". */
export function zoneLabel(offsetSecs: number): string {
  return `UTC${formatFixedOffset(offsetSecs)}`;
}
````

**Yeni dosya:** `app/ui/varsha/YearControls.tsx`

````tsx
// Varshaphala yıl seçicisi: önceki ve sonraki yıl, yaşın kendisi ve "bu yıl". Varshaphala paneli
// ve dasa ağacının mudda görünümü aynı denetimi kullanır. `age` gösterilen yaştır (seçili yaş ya
// da Rust'ın bulduğu süren yıl); henüz bilinmiyorsa `null`. Burada tarih hesabı yoktur (C4).
import { useId } from "react";
import { useTranslation } from "react-i18next";
import { isVarshaAge, MAX_VARSHA_AGE } from "./yearStore";

export function VarshaYearControls({ age, current, onChange }: { age: number | null; /** "Bu yıl" seçili. */ current: boolean; onChange(age: number | null): void }) {
  const { t } = useTranslation();
  const id = useId();
  const step = (delta: number) => {
    if (age !== null && isVarshaAge(age + delta)) onChange(age + delta);
  };
  const atMin = age === null || age <= 0;
  const atMax = age === null || age >= MAX_VARSHA_AGE;
  return (
    <>
      {/* Kapalı düğme yerel `disabled` değildir: odağı tutar, tıklamak yılı değiştirmez. */}
      <button type="button" aria-disabled={atMin || undefined} onClick={() => step(-1)}>
        {t("varsha.previous")}
      </button>
      <label htmlFor={`${id}-age`}>{t("varsha.age")}</label>
      <input
        id={`${id}-age`}
        className="varsha-age"
        type="number"
        inputMode="numeric"
        min={0}
        max={MAX_VARSHA_AGE}
        step={1}
        value={age ?? ""}
        onChange={(e) => {
          const next = Number(e.target.value);
          if (e.target.value.trim() !== "" && isVarshaAge(next)) onChange(next);
        }}
      />
      <button type="button" aria-disabled={atMax || undefined} onClick={() => step(1)}>
        {t("varsha.next")}
      </button>
      <button type="button" aria-pressed={current} onClick={() => onChange(null)}>
        {t("varsha.current")}
      </button>
    </>
  );
}
````

**Yeni dosya:** `app/ui/modules/useVarsha.ts`

````ts
// Varshaphala raporunu (`compute_varsha`) isteyen kanca ve yıllık haritanın yer seçimi. Sonuç
// saklanmaz ve arayüzde hesap yoktur (K1, C4); panel açıkken istenir. `age === null` "bu yıl"dır:
// süren yılı Rust bulur. Doğum saati bilinmiyorsa rapor yine döner: yıllık harita ve muntha
// `null`'dır (K2). Yer kişisel veridir: loglanmaz.
import { api } from "../api";
import type { AppPrefs, HomePlace, OpenedClient, VarshaView } from "../bindings";
import { chartInput, inputKey } from "./input";
import { chartPlace, muhurtaPlace } from "./muhurta";
import { useModule, type ModuleState } from "./useModule";

/** Yıllık haritanın yeri: doğum yeri ya da tercihlerdeki bulunulan yer. */
export const VARSHA_PLACES = ["birth", "home"] as const;
export type VarshaPlaceChoice = (typeof VARSHA_PLACES)[number];

/** Panelin yer seçimi; tanınmayan değer ve eksik anahtar doğum yeridir. */
export function varshaPlaceChoice(settings: Record<string, unknown>): VarshaPlaceChoice {
  return settings.place === "home" ? "home" : "birth";
}

/** Seçime göre yer; bulunulan yer seçilmemişse `null`. */
export function varshaPlace(choice: VarshaPlaceChoice, opened: OpenedClient, prefs: AppPrefs | null): HomePlace | null {
  return choice === "home" ? (prefs?.home_place ?? null) : chartPlace(opened);
}

/** `place === null`: yer seçilmemiş, komut çağrılmaz. */
export function useVarsha(opened: OpenedClient | null, age: number | null, place: HomePlace | null): ModuleState<VarshaView> {
  const at = place === null ? null : muhurtaPlace(place);
  const key = opened !== null && at !== null ? `${inputKey(opened)}|${age ?? "now"}|${JSON.stringify(at)}` : null;
  return useModule(key, () => api.computeVarsha(chartInput(opened as OpenedClient), age, at as NonNullable<typeof at>));
}
````

**Yeni dosya:** `app/ui/panels/varsha.css`

````css
/* Varshaphala paneli ve mudda dasa görünümü. Renk token'lardan gelir (C9). */
.varsha-age {
  width: 5ch;
}
.varsha-figure {
  width: 100%;
  max-width: 26rem;
  aspect-ratio: 1;
  margin: 0 auto;
}
.varsha-save {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px 12px;
}
.varsha-save p {
  margin: 0;
  color: var(--text-2);
}
.varsha-summary {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin: 0;
  color: var(--text-2);
  font-size: var(--fs-table);
}
````

**Yeni dosya:** `app/ui/panels/VarshaPanel.tsx`

````tsx
// Varshaphala paneli (yol haritası §4.14): seçilen yılın dönüş anı, yıllık haritası ve munthası.
// Değerler `compute_varsha`'dan gelir; burada hesap yoktur (C4). Yıl seçimi ortak depodadır
// (`varsha/yearStore`): dasa ağacının mudda görünümü aynı yılı gösterir. Yer seçimi panelin
// ayarıdır ve düzenle saklanır. Doğum saati bilinmiyorsa yıllık harita ve muntha Rust'tan boş
// gelir (K2): çizilmez, yerine bir açıklama çıkar. Yıllık harita var olan olay haritası yoluyla
// kaydedilebilir (`save_varsha_chart`); kayıt açılmaz, yalnızca bildirilir.
import { useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { Chart, CommandError, OpenedClient, Sign, VarshaView } from "../bindings";
import { chartDataFromChart, signIndex } from "../chart/adapter";
import { ChartSvg } from "../chart/ChartSvg";
import { CHART_STYLES, isChartStyle, RETRO_MARK, showsRetro, type ChartData } from "../chart/types";
import { useHighlight } from "../highlight/store";
import { errorMessage } from "../i18n/errors";
import type { Panel } from "../layout/model";
import { inputKey } from "../modules/input";
import { placeLabel } from "../modules/muhurta";
import { useLastData } from "../modules/useModule";
import { useVarsha, VARSHA_PLACES, varshaPlace, varshaPlaceChoice, type VarshaPlaceChoice } from "../modules/useVarsha";
import { useSession } from "../session/store";
import { usePrefs } from "../settings/prefs";
import { VarshaYearControls } from "../varsha/YearControls";
import { useVarshaAge } from "../varsha/yearStore";
import { zoneLabel } from "../varsha/zone";
import { useLayoutStore } from "../view/store";
import { BalaScroll } from "./bala";
import { formatDegree, formatJdMinute } from "./format";
import { HomePlaceDialog } from "./HomePlaceDialog";
import { chartLabels, nakshatraName, VARA_NAMES } from "./labels";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";
import "./varsha.css";

/** Munthanın çizimdeki işareti (transliterasyon; dile göre değişmez, K11). */
export const MUNTHA_MARK = "Mu";

export function VarshaPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <Varsha panel={panel} opened={opened} />;
}

function Varsha({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { prefs } = usePrefs();
  const { setPanelSettings } = useLayoutStore();
  const { age: selected, setAge } = useVarshaAge(opened.chart_record.id);
  const id = useId();
  const [editingHome, setEditingHome] = useState(false);
  const choice = varshaPlaceChoice(panel.settings);
  const place = varshaPlace(choice, opened, prefs);
  const module = useVarsha(opened, selected, place);
  // Yıl değişirken önceki yılın görünümü yerinde durur; başka harita ya da yer eskiyi unutturur.
  const view = useLastData(module, place === null ? null : `${inputKey(opened)}|${choice}`);
  const phase = usePanelLoadPhase(module.loading);
  return (
    <div className="module-panel">
      <div className="module-toolbar">
        <VarshaYearControls age={selected ?? view?.report.age ?? null} current={selected === null} onChange={setAge} />
        <label htmlFor={`${id}-place`}>{t("varsha.place.label")}</label>
        <select id={`${id}-place`} value={choice} onChange={(e) => setPanelSettings(panel.id, { place: varshaPlaceChoice({ place: e.target.value }) })}>
          {VARSHA_PLACES.map((c) => (
            <option key={c} value={c}>
              {t(`varsha.place.${c}`)}
            </option>
          ))}
        </select>
        {choice === "home" && (
          <button type="button" onClick={() => setEditingHome(true)}>
            {t(place ? "muhurta.place.changeHome" : "muhurta.place.pickHome")}
          </button>
        )}
      </div>
      {place === null && (
        <p className="panel-notice" role="note">
          {t("varsha.place.missingHome")}
        </p>
      )}
      {module.error && <PanelError error={module.error} onRetry={module.retry} />}
      {place !== null && module.error === null && view === null && phase === "skeleton" && <PanelSkeleton />}
      {place !== null && module.error === null && view !== null && <Report key={`${view.report.age}|${choice}`} panel={panel} opened={opened} view={view} choice={choice} placeName={placeLabel(place)} />}
      {editingHome && <HomePlaceDialog opened={opened} onClose={() => setEditingHome(false)} />}
    </div>
  );
}

function Report({ panel, opened, view, choice, placeName }: { panel: Panel; opened: OpenedClient; view: VarshaView; choice: VarshaPlaceChoice; placeName: string }) {
  const { t } = useTranslation();
  const { report, offset_secs: offset } = view;
  const captionId = useId();
  const number = report.age + 1;
  const zone = zoneLabel(offset);
  const moment = (jd: number | null) => t("varsha.moment", { moment: formatJdMinute(jd, offset), zone });
  const sign = (s: Sign) => t(`sign.${signIndex(s)}`);
  const { facts, chart } = report;
  const rows: { key: string; label: string; value: string }[] = [
    { key: "year", label: t("varsha.year"), value: t("varsha.yearValue", { number, age: report.age }) },
    { key: "start", label: t("varsha.start"), value: moment(report.start_jd_ut) },
    { key: "end", label: t("varsha.end"), value: moment(report.end_jd_ut) },
  ];
  if (chart !== null) {
    rows.push({ key: "place", label: t("varsha.placeRow"), value: placeName });
    rows.push({ key: "vara", label: t("varsha.vara"), value: VARA_NAMES[chart.panchang.vara] });
  }
  if (facts !== null && chart !== null) {
    rows.push({
      key: "lagna",
      label: t("varsha.lagna"),
      value: t("varsha.lagnaValue", { sign: sign(facts.lagna_sign), degree: chart.lagna.deg_in_sign === null ? "—" : formatDegree(chart.lagna.deg_in_sign), lord: t(`grahaName.${facts.lagna_lord}`) }),
    });
    rows.push({
      key: "muntha",
      label: t("varsha.muntha"),
      value: t("varsha.munthaValue", { sign: sign(facts.muntha.sign), annual: facts.muntha.annual_house, natal: facts.muntha.natal_house, lord: t(`grahaName.${facts.muntha.lord}`) }),
    });
    rows.push({ key: "natal-lagna", label: t("varsha.natalLagna"), value: t("varsha.natalLagnaValue", { sign: sign(facts.natal_lagna_sign), lord: t(`grahaName.${facts.natal_lagna_lord}`) }) });
  }
  rows.push({ key: "mudda", label: t("varsha.muddaFirst"), value: t(`grahaName.${report.mudda.first_lord}`) });
  return (
    <>
      {!report.time_known && (
        <p className="panel-notice" role="note">
          {t("varsha.timeUnknown")}
        </p>
      )}
      {report.warnings.includes("LowPrecision") && (
        <p className="panel-notice" role="note">
          {t("badgeHint.LowPrecision")}
        </p>
      )}
      <BalaScroll labelledBy={captionId}>
        <table className="module-table">
          <caption id={captionId} className="visually-hidden">
            {t("varsha.caption")}
          </caption>
          <thead>
            <tr>
              <th scope="col">{t("varsha.fact")}</th>
              <th scope="col">{t("varsha.value")}</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.key} data-fact={r.key}>
                <th scope="row">{r.label}</th>
                <td>{r.value}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </BalaScroll>
      {chart !== null && facts !== null && (
        <>
          <AnnualChart panel={panel} chart={chart} number={number} munthaSign={signIndex(facts.muntha.sign)} />
          <Positions chart={chart} />
          <SaveAsEvent opened={opened} age={report.age} home={choice === "home"} />
        </>
      )}
      <p className="panel-message">{t("varsha.reading")}</p>
      <p className="panel-message">{t("varsha.method", { method: report.method })}</p>
    </>
  );
}

/** Yıllık haritanın D1 çizimi; muntha burcu "Mu" ile işaretlenir. Biçim panelin ayarıdır. */
function AnnualChart({ panel, chart, number, munthaSign }: { panel: Panel; chart: Chart; number: number; munthaSign: number }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const highlight = useHighlight();
  const id = useId();
  const style = isChartStyle(panel.settings.style) ? panel.settings.style : "north";
  const base = chartDataFromChart(chart, "D1", { showLagna: true });
  if (base === null) return null;
  const data: ChartData = { ...base, marks: [{ sign: munthaSign, text: MUNTHA_MARK }] };
  return (
    <>
      <div className="module-toolbar">
        <label htmlFor={`${id}-style`}>{t("chart.style")}</label>
        <select id={`${id}-style`} value={style} onChange={(e) => isChartStyle(e.target.value) && setPanelSettings(panel.id, { style: e.target.value })}>
          {CHART_STYLES.map((s) => (
            <option key={s} value={s}>
              {t(`chartStyle.${s}`)}
            </option>
          ))}
        </select>
      </div>
      <div className="varsha-figure">
        <ChartSvg
          data={data}
          style={style}
          labels={{ ...chartLabels(t, "D1"), title: t("varsha.chartTitle", { number }) }}
          highlight={highlight.active}
          pressed={highlight.pinned}
          onHover={highlight.hover}
          onPin={highlight.togglePin}
        />
      </div>
      {/* Çizimdeki işaret `aria-hidden`'dır: muntha burada sözcükle okunur. */}
      <p className="visually-hidden">{t("varsha.munthaMark", { sign: t(`sign.${munthaSign}`), mark: MUNTHA_MARK })}</p>
    </>
  );
}

/** Çizimin tablo karşılığı: lagna ve dokuz grahanın yıllık haritadaki yeri. */
function Positions({ chart }: { chart: Chart }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const captionId = useId();
  const degree = (d: number | null) => (d === null ? "—" : formatDegree(d));
  return (
    <BalaScroll labelledBy={captionId}>
      <table className="module-table">
        <caption id={captionId} className="visually-hidden">
          {t("varsha.positions")}
        </caption>
        <thead>
          <tr>
            <th scope="col">{t("varsha.col.graha")}</th>
            <th scope="col">{t("varsha.col.sign")}</th>
            <th scope="col" className="num">
              {t("varsha.col.degree")}
            </th>
            <th scope="col">{t("varsha.col.nakshatra")}</th>
            <th scope="col" className="num">
              {t("varsha.col.house")}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr data-graha="Lagna">
            <th scope="row">{t("table.lagnaRow")}</th>
            <td>{t(`sign.${signIndex(chart.lagna.sign)}`)}</td>
            <td className="num">{degree(chart.lagna.deg_in_sign)}</td>
            <td>{nakshatraName(chart.lagna.nakshatra)}</td>
            <td className="num">1</td>
          </tr>
          {chart.grahas.map((g) => (
            <tr key={g.graha} data-graha={g.graha} className={active?.type === "graha" && active.id === g.graha ? "is-highlighted" : undefined}>
              <th scope="row">
                <PinButton target={{ type: "graha", id: g.graha }}>{t(`grahaName.${g.graha}`)}</PinButton>
                {showsRetro(g.graha, g.retrograde) && (
                  <>
                    {" "}
                    <span aria-hidden="true">{RETRO_MARK}</span>
                    <span className="visually-hidden">{t("chart.retro")}</span>
                  </>
                )}
              </th>
              <td>{t(`sign.${signIndex(g.sign)}`)}</td>
              <td className="num">{degree(g.deg_in_sign)}</td>
              <td>{nakshatraName(g.nakshatra)}</td>
              <td className="num">{g.house}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </BalaScroll>
  );
}

/**
 * "Olay haritası olarak kaydet": kaynak harita, yaş ve yer seçimi gider; an ve yer Rust'ta
 * çözülür. Kayıt açılmaz: etiketi bildirilir ve harita listeleri yenilenir. Bileşen yıl ve yer
 * başına yeniden kurulur (`key`): başka yılın bildirimi bu yıla taşınmaz.
 */
function SaveAsEvent({ opened, age, home }: { opened: OpenedClient; age: number; home: boolean }) {
  const { t, i18n } = useTranslation();
  const { touch } = useSession();
  const [state, setState] = useState<{ status: "idle" | "saving" } | { status: "saved"; label: string } | { status: "error"; error: CommandError }>({ status: "idle" });
  const busy = state.status === "saving" || state.status === "saved";
  const save = async () => {
    if (busy) return;
    setState({ status: "saving" });
    const r = await api.saveVarshaChart(opened.chart_record.id, age, home);
    if (r.status === "error") return setState({ status: "error", error: r.error });
    setState({ status: "saved", label: r.data.label });
    touch();
  };
  return (
    <div className="varsha-save">
      {/* Kayıttan sonra düğme odağı tutar ama yeniden kaydetmez: aynı yıl iki kez eklenmez. */}
      <button type="button" aria-disabled={busy || undefined} onClick={() => void save()}>
        {t("varsha.save")}
      </button>
      <p role="status">{state.status === "saved" ? t("varsha.saved", { label: state.label }) : ""}</p>
      {state.status === "error" && (
        <p role="alert" className="panel-notice">
          {errorMessage(t, i18n, state.error)}
        </p>
      )}
    </div>
  );
}
````

Katalog ve kayıt (paylaşılan dosyalar):

````diff
diff --git a/app/ui/layout/model.ts b/app/ui/layout/model.ts
index ac64dae..32db1f7 100644
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -22,6 +22,7 @@ export const PANEL_KINDS = [
   "sudarshana",
   "yogas",
   "avastha",
+  "varsha",
   "compat",
   "muhurta_day",
   "muhurta_search",
@@ -55,6 +56,8 @@ export const MODULE_PANELS = [
   { kind: "yogas", w: 7, h: 14 },
   // Avasthalar (Plan 24).
   { kind: "avastha", w: 7, h: 16 },
+  // Varshaphala (yol haritası Plan 27).
+  { kind: "varsha", w: 7, h: 24 },
   // Uyum (Plan 19).
   { kind: "compat", w: 7, h: 16 },
   // Muhurta (Plan 20).
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index e1380f2..58ac9ef 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -27,6 +27,7 @@ import { ShadbalaPanel } from "./ShadbalaPanel";
 import { SudarshanaPanel } from "./SudarshanaPanel";
 import type { PanelDefinition } from "./types";
 import { UnavailablePanel } from "./UnavailablePanel";
+import { VarshaPanel } from "./VarshaPanel";
 import { VimshopakaPanel } from "./VimshopakaPanel";
 import { YogasPanel } from "./YogasPanel";
 
@@ -61,6 +62,8 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   yogas: { titleKey: "panel.yogas", component: YogasPanel },
   // Avasthalar (Plan 24).
   avastha: { titleKey: "panel.avastha", component: AvasthaPanel },
+  // Varshaphala (yol haritası Plan 27).
+  varsha: { titleKey: "panel.varsha", component: VarshaPanel },
   // Uyum (Plan 19).
   compat: { titleKey: "panel.compat", component: CompatPanel },
   // Muhurta (Plan 20).
````

Metinler (paylaşılan dosyalar; üç dilde aynı anahtarlar, `panel` bloğuna bir satır ve `shortcuts` bloğundan önce yeni `varsha` bloğu):

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index f8e66fe..a850d72 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -84,6 +84,7 @@ export const en: Messages = {
     bhava_bala: "Bhava bala",
     yogas: "Yogas",
     avastha: "Avasthas",
+    varsha: "Varshaphala",
     compat: "Compatibility",
     muhurta_day: "Day panchang",
     muhurta_search: "Muhurta search",
@@ -1106,6 +1107,51 @@ export const en: Messages = {
       outside: "The transit moment is outside the Vimshottari cycle.",
     },
   },
+  varsha: {
+    year: "Year",
+    age: "Age (completed years)",
+    previous: "Previous year",
+    next: "Next year",
+    current: "This year",
+    place: {
+      label: "Place",
+      birth: "Birth place",
+      home: "Current place",
+      missingHome: "No current place is set: choose one for the annual chart, or go back to “Birth place”.",
+    },
+    caption: "Varshaphala: facts of the year",
+    fact: "Fact",
+    value: "Value",
+    yearValue: "Year {{number}} (completed years {{age}})",
+    start: "Return moment",
+    end: "End of the year (next return)",
+    moment: "{{moment}} ({{zone}})",
+    placeRow: "Chart place",
+    vara: "Vara",
+    lagna: "Lagna of the year",
+    lagnaValue: "{{sign}} {{degree}} · lord {{lord}}",
+    muntha: "Muntha",
+    munthaValue: "{{sign}} · house {{annual}} in the annual chart · house {{natal}} in the birth chart · lord {{lord}}",
+    natalLagna: "Birth lagna",
+    natalLagnaValue: "{{sign}} · lord {{lord}}",
+    muddaFirst: "First lord of the mudda dasha",
+    chartTitle: "Annual chart, year {{number}}",
+    munthaMark: "The muntha is in {{sign}} (“{{mark}}” in the drawing).",
+    positions: "Annual chart positions",
+    col: { graha: "Graha", sign: "Sign", degree: "Degree", nakshatra: "Nakshatra", house: "House" },
+    timeUnknown: "Birth time unknown: the annual chart and the muntha are not computed (both need the lagna). The return moment and the mudda dasha are approximate (≈).",
+    reading: "The return is taken on the sidereal longitude in every mode (selected ayanamsa). The muntha is as many signs past the sign of the birth lagna as there are completed years; signs are in the rasi frame.",
+    save: "Save as an event chart",
+    saved: "Event chart saved: {{label}}",
+    method: "Method: {{method}}",
+    mudda: {
+      system: "Mudda dasha",
+      summary: "Year {{number}} (completed years {{age}}): {{start}} – {{end}} ({{zone}})",
+      first: "First lord {{lord}}: birth nakshatra {{nakshatra}} ({{name}}), completed years {{age}}, remainder {{remainder}}.",
+      unverified: "Unverified table: the first-lord rule and the period shares of the mudda dasha await comparison with a printed source.",
+      approx: "Birth time unknown: the return moment, the period dates and the first lord are approximate (≈).",
+    },
+  },
   shortcuts: {
     title: "Keyboard shortcuts",
     open: "Shortcuts",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 67cc873..ab65c17 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -84,6 +84,7 @@ export const fr: Messages = {
     bhava_bala: "Bhava bala",
     yogas: "Yogas",
     avastha: "Avasthas",
+    varsha: "Varshaphala",
     compat: "Compatibilité",
     muhurta_day: "Panchang du jour",
     muhurta_search: "Recherche de muhurta",
@@ -1106,6 +1107,51 @@ export const fr: Messages = {
       outside: "Le moment du transit est hors du cycle Vimshottari.",
     },
   },
+  varsha: {
+    year: "Année",
+    age: "Âge (années révolues)",
+    previous: "Année précédente",
+    next: "Année suivante",
+    current: "Cette année",
+    place: {
+      label: "Lieu",
+      birth: "Lieu de naissance",
+      home: "Lieu actuel",
+      missingHome: "Aucun lieu actuel n’est défini : choisissez-en un pour le thème annuel, ou revenez à « Lieu de naissance ».",
+    },
+    caption: "Varshaphala : faits de l’année",
+    fact: "Fait",
+    value: "Valeur",
+    yearValue: "Année {{number}} (années révolues {{age}})",
+    start: "Moment du retour",
+    end: "Fin de l’année (retour suivant)",
+    moment: "{{moment}} ({{zone}})",
+    placeRow: "Lieu du thème",
+    vara: "Vara",
+    lagna: "Lagna de l’année",
+    lagnaValue: "{{sign}} {{degree}} · maître {{lord}}",
+    muntha: "Muntha",
+    munthaValue: "{{sign}} · maison {{annual}} du thème annuel · maison {{natal}} du thème natal · maître {{lord}}",
+    natalLagna: "Lagna natal",
+    natalLagnaValue: "{{sign}} · maître {{lord}}",
+    muddaFirst: "Premier maître de la mudda dasha",
+    chartTitle: "Thème annuel, année {{number}}",
+    munthaMark: "Le muntha est en {{sign}} (« {{mark}} » sur le dessin).",
+    positions: "Positions du thème annuel",
+    col: { graha: "Graha", sign: "Signe", degree: "Degré", nakshatra: "Nakshatra", house: "Maison" },
+    timeUnknown: "Heure de naissance inconnue : le thème annuel et le muntha ne sont pas déterminés (les deux demandent le lagna). Le moment du retour et la mudda dasha sont approximatifs (≈).",
+    reading: "Le retour est pris sur la longitude sidérale dans tous les modes (ayanamsa choisi). Le muntha se trouve à autant de signes du signe du lagna natal qu’il y a d’années révolues ; les signes sont dans le repère rasi.",
+    save: "Enregistrer comme thème d’événement",
+    saved: "Thème d’événement enregistré : {{label}}",
+    method: "Méthode : {{method}}",
+    mudda: {
+      system: "Mudda dasha",
+      summary: "Année {{number}} (années révolues {{age}}) : {{start}} – {{end}} ({{zone}})",
+      first: "Premier maître {{lord}} : nakshatra natal {{nakshatra}} ({{name}}), années révolues {{age}}, reste {{remainder}}.",
+      unverified: "Table non vérifiée : la règle du premier maître et les parts des périodes de la mudda dasha attendent la comparaison avec une source imprimée.",
+      approx: "Heure de naissance inconnue : le moment du retour, les dates des périodes et le premier maître sont approximatifs (≈).",
+    },
+  },
   shortcuts: {
     title: "Raccourcis clavier",
     open: "Raccourcis",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 3c2e07b..6b0f509 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -83,6 +83,7 @@ export const tr = {
     bhava_bala: "Bhava bala",
     yogas: "Yogalar",
     avastha: "Avasthalar",
+    varsha: "Varshaphala",
     compat: "Uyum",
     muhurta_day: "Gün panchangı",
     muhurta_search: "Muhurta araması",
@@ -1105,6 +1106,51 @@ export const tr = {
       outside: "Transit anı Vimshottari döngüsünün dışında.",
     },
   },
+  varsha: {
+    year: "Yıl",
+    age: "Yaş (tamamlanan yıl)",
+    previous: "Önceki yıl",
+    next: "Sonraki yıl",
+    current: "Bu yıl",
+    place: {
+      label: "Yer",
+      birth: "Doğum yeri",
+      home: "Bulunulan yer",
+      missingHome: "Bulunulan yer seçilmedi: yıllık harita için bir yer seçin ya da “Doğum yeri”ne dönün.",
+    },
+    caption: "Varshaphala: yılın olguları",
+    fact: "Olgu",
+    value: "Değer",
+    yearValue: "{{number}}. yıl (tamamlanan yıl {{age}})",
+    start: "Dönüş anı",
+    end: "Yıl sonu (sonraki dönüş)",
+    moment: "{{moment}} ({{zone}})",
+    placeRow: "Harita yeri",
+    vara: "Vara",
+    lagna: "Yıl lagnası",
+    lagnaValue: "{{sign}} {{degree}} · lordu {{lord}}",
+    muntha: "Muntha",
+    munthaValue: "{{sign}} · yıllık haritada {{annual}}. ev · doğum haritasında {{natal}}. ev · lordu {{lord}}",
+    natalLagna: "Doğum lagnası",
+    natalLagnaValue: "{{sign}} · lordu {{lord}}",
+    muddaFirst: "Mudda dasanın ilk lordu",
+    chartTitle: "Yıllık harita, {{number}}. yıl",
+    munthaMark: "Muntha {{sign}} burcunda (çizimde “{{mark}}”).",
+    positions: "Yıllık harita konumları",
+    col: { graha: "Graha", sign: "Burç", degree: "Derece", nakshatra: "Nakshatra", house: "Ev" },
+    timeUnknown: "Doğum saati bilinmiyor: yıllık harita ve muntha hesaplanmadı (ikisi de lagna ister). Dönüş anı ve mudda dasa yaklaşıktır (≈).",
+    reading: "Dönüş her kipte sidereal boylamdandır (seçili ayanamsa). Muntha, doğum lagnasının burcundan tamamlanan yıl kadar burç ileridedir; burçlar rasi çerçevesindedir.",
+    save: "Olay haritası olarak kaydet",
+    saved: "Olay haritası kaydedildi: {{label}}",
+    method: "Yöntem: {{method}}",
+    mudda: {
+      system: "Mudda dasa",
+      summary: "{{number}}. yıl (tamamlanan yıl {{age}}): {{start}} – {{end}} ({{zone}})",
+      first: "İlk lord {{lord}}: doğum nakshatrası {{nakshatra}} ({{name}}), tamamlanan yıl {{age}}, artan {{remainder}}.",
+      unverified: "Doğrulanmamış tablo: mudda dasanın ilk lord kuralı ve dönem payları basılı kaynakla karşılaştırılmayı bekliyor.",
+      approx: "Doğum saati bilinmiyor: dönüş anı, dönem tarihleri ve ilk lord yaklaşıktır (≈).",
+    },
+  },
   shortcuts: {
     title: "Klavye kısayolları",
     open: "Kısayollar",
````

- [ ] **Step 4: Testler ve kapılar**

```bash
pnpm --dir app exec vitest run ui/panels/VarshaPanel.test.tsx 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
cargo fmt --all --check
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
```
Expected: panel dosyası `1 passed (1)`, `17 passed (17)`; tamamı `98 passed (98)`, `990 passed (990)`; `✓ built`; fmt çıktısız; sayaç `0`. Bir panel testi metin yüzünden düşerse önce i18n bloğunun bu plandaki metinle bire bir aynı olduğu denetlenir (tırnak ve kesme işaretleri dahil: Türkçe ve İngilizce metinlerde “ ”, Fransızca metinlerde « » ve ’).

- [ ] **Step 5: Commit**

```bash
git add app/ui/api.ts app/ui/api.test.ts app/ui/test/fixture.ts app/ui/layout/model.ts app/ui/layout/model.test.ts app/ui/panels/registry.ts app/ui/panels/VarshaPanel.tsx app/ui/panels/VarshaPanel.test.tsx app/ui/panels/varsha.css app/ui/modules/useVarsha.ts app/ui/varsha app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts
git diff --cached --stat | tail -1
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): Varshaphala panel with the year selector, annual chart, muntha and save as event chart" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: `16 files changed, 972 insertions(+), 1 deletion(-)`; tarama çıktısız; `git status --short` boş.

---

### Task 7: Arayüz: dasa ağacında mudda dasa; uçtan uca akışlar ve görsel testler

Yol haritası §5 Plan 27 görev (6) ("var olan dasa panellerinde mudda görünümü"); D16–D19, D22. Dasa ağacı paneli üçüncü bir sistem kazanır; dasa şeridi değişmez. Uçtan uca ve görsel testlerin kodu bu görevde yazılır; **yürüten ajan bunları koşmaz** (`pnpm build` tiplerini denetler), kontrolör Step 5'te koşar.

**Files:**
- Create: `app/ui/dasha/muddaSystem.tsx`, `app/ui/dasha/MuddaDashaTree.tsx`, `app/ui/dasha/MuddaDashaTree.test.tsx`, `app/e2e/varsha.desk.spec.ts`, `app/e2e/varsha.visual.spec.ts`
- Modify (paylaşılan dosyalar): `app/ui/dasha/DashaTreePanel.tsx`, `app/ui/panels/registry.ts`, `app/e2e/ipc.ts`
- Test: `MuddaDashaTree.test.tsx` (8); iki uçtan uca akış ve iki görsel test (kontrolör)

**Interfaces:**
- Consumes: Task 6'nın `useVarsha`, `useVarshaAge`, `resetVarshaAge`, `VarshaYearControls`, `zoneLabel`, `VarshaPanel`'i ve `varsha.mudda.*`, `varsha.method` metinleri; var olan `dasha/PeriodTree` (`PeriodTree`, `PeriodLevel`), `dasha/tree` (`childrenOf`), `dasha/system` (`DASHA_SYSTEMS`, `dashaSystem`, `DashaSystem`), `modules/muhurta` (`chartPlace`), `panels/format` (`formatJdMinute`), `panels/labels` (`nakshatraName`), `panels/states`; i18n `dasha.system`, `dasha.systems.*`.
- Produces:
  - `dasha/muddaSystem.tsx`: `TREE_SYSTEMS = ["vimshottari", "chara", "mudda"]`, `TreeSystem`, `treeSystem(settings)`, `treeSystemName(system, t)`, `TreeSystemSelect({ value, onChange })`.
  - `dasha/MuddaDashaTree.tsx`: `MuddaDashaTree({ opened })`.
  - Dasa ağacı panelinin `system` ayarı üç değer alır; panelin erişilebilir adı "Dasa ağacı Mudda dasa" olur.
  - E2e taklidi: `compute_varsha` (yaş 27 → `fixture.varsha.next`; saat bilinmiyorsa `time_unknown`; öteki → `current`), `save_varsha_chart`.

- [ ] **Step 1: Testler: mudda ağacı, iki uçtan uca akış, iki görsel test; komutların taklidi**

Ağaç testleri (8): sistem listesi ve ayarın kaydı; süren yılın dokuz dönemi, ilk lord satırı ve "doğrulanmamış tablo" notu; alt dönemler, tek açık yol ve klavye; yılın Varshaphala paneliyle ortak oluşu; doğum saati bilinmeyen harita; hata ve yeniden deneme; yükleme evreleri; fikstürdeki sıranın Rust sırası oluşu.

**Yeni dosya:** `app/ui/dasha/MuddaDashaTree.test.tsx`

````tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeVarsha: vi.fn(), saveVarshaChart: vi.fn(), dashaChildren: vi.fn(), computeCharaDasha: vi.fn(), charaDashaChildren: vi.fn() } }));

import { api, type Result } from "../api";
import type { VarshaView } from "../bindings";
import { i18n } from "../i18n";
import type { Panel } from "../layout/model";
import { PANEL_REGISTRY } from "../panels/registry";
import { VarshaPanel } from "../panels/VarshaPanel";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { resetVarshaAge } from "../varsha/yearStore";
import { useLayoutStore } from "../view/store";
import { DashaTreePanel } from "./DashaTreePanel";
import { TREE_SYSTEMS, treeSystem } from "./muddaSystem";
import { dashaSystem } from "./system";

const FX = fixture();
const ok = (view: VarshaView = fixture().varsha.current): Result<VarshaView> => ({ status: "ok", data: view });
const failure = { status: "error" as const, error: { code: "calc", params: { detail: "x" } } };
const DELHI = { lat: 28.6139, lon: 77.209, tz: { Iana: "Asia/Kolkata" } };
const LAYOUT = (settings: Record<string, unknown>, withVarsha = false) => {
  const panels: Panel[] = [{ id: "dt", kind: "dasha_tree", x: 0, y: 0, w: 4, h: 12, settings }];
  if (withVarsha) panels.push({ id: "vp", kind: "varsha", x: 4, y: 0, w: 7, h: 24, settings: {} });
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels }] };
};

/** Panelleri düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanels() {
  const { active } = useLayoutStore();
  return (
    <>
      <section aria-label="tree">
        <DashaTreePanel panel={active.panels[0]} />
      </section>
      {active.panels[1] && (
        <section aria-label="varsha">
          <VarshaPanel panel={active.panels[1]} />
        </section>
      )}
    </>
  );
}

function renderTree(settings: Record<string, unknown> = { system: "mudda" }, { opened = fixture().opened, withVarsha = false } = {}) {
  return renderWithStores(<LivePanels />, { layout: LAYOUT(settings, withVarsha), opened });
}

const tree = () => screen.findByRole("tree", { name: "Mudda dasa" });
const items = (root: HTMLElement, level: number) => within(root).queryAllByRole("treeitem").filter((i) => i.getAttribute("aria-level") === String(level));
const click = (item: HTMLElement) => fireEvent.click(item.querySelector(".dasha-row")!);
const savedSettings = (persistence: ReturnType<typeof renderTree>["persistence"]) =>
  persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings;

beforeEach(() => {
  resetVarshaAge();
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.computeVarsha).mockImplementation(async (_input, age) => ok(age === 27 ? fixture().varsha.next : fixture().varsha.current));
});

test("the tree's system list offers the mudda dasha and the choice is saved with the layout", async () => {
  const { persistence } = renderTree({});
  const select = screen.getByRole("combobox", { name: "Sistem" });
  expect(within(select).getAllByRole("option").map((o) => o.textContent)).toEqual(["Vimshottari", "Chara dasha", "Mudda dasa"]);
  expect(select).toHaveValue("vimshottari");
  // Mudda dasa seçilmeden rapor istenmez (K1).
  expect(api.computeVarsha).not.toHaveBeenCalled();
  fireEvent.change(select, { target: { value: "mudda" } });
  await tree();
  await waitFor(() => expect(savedSettings(persistence)).toEqual({ system: "mudda" }));
  expect(api.computeVarsha).toHaveBeenCalledTimes(1);
  // Sistem adı panelin erişilebilir adına eklenir; dasa şeridi mudda dasayı tanımaz.
  expect(PANEL_REGISTRY.dasha_tree.nameDetail?.({ id: "dt", kind: "dasha_tree", x: 0, y: 0, w: 4, h: 12, settings: { system: "mudda" } }, i18n.t)).toBe("Mudda dasa");
  expect(TREE_SYSTEMS).toEqual(["vimshottari", "chara", "mudda"]);
  expect(treeSystem({ system: "chara" })).toBe("chara");
  expect(treeSystem({ system: "other" })).toBe("vimshottari");
  expect(dashaSystem({ system: "mudda" })).toBe("vimshottari");
});

test("lists the nine periods of the running year from Rust, with the first-lord line and the unverified note", async () => {
  renderTree();
  const root = await tree();
  const opened = fixture().opened;
  // Yaş gönderilmez (süren yılı Rust bulur); dönemler yere bağlı değildir: doğum yeri gider.
  expect(api.computeVarsha).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true, lagna_number: null }, null, DELHI);
  const top = items(root, 1);
  // Delhi fikstürü: Swati (15), tamamlanan yıl 26 → artan 3: Mars. Sıra Mars'tan sürer.
  expect(top.map((i) => i.querySelector(".dasha-lord")?.textContent)).toEqual(["Mars", "Rahu", "Jüpiter", "Satürn", "Merkür", "Ketu", "Venüs", "Güneş", "Ay"]);
  expect(top[0]).toHaveAccessibleName("Mars, 2026-01-01 09:28 – 2026-01-22 16:50");
  expect(top[1]).toHaveAccessibleName("Rahu, 2026-01-22 16:50 – 2026-03-18 11:46");
  expect(top[8].querySelector(".dasha-dates")?.textContent).toMatch(/ – 2027-01-01 15:41$/);
  expect(top[0]).toHaveAttribute("aria-expanded", "false");
  expect(items(root, 2)).toHaveLength(0);
  expect(screen.getByText("27. yıl (tamamlanan yıl 26): 2026-01-01 09:28 – 2027-01-01 15:41 (UTC+05:30)")).toBeInTheDocument();
  expect(screen.getByText("İlk lord Mars: doğum nakshatrası 15 (Swati), tamamlanan yıl 26, artan 3.")).toBeInTheDocument();
  expect(screen.getByText("Doğrulanmamış tablo: mudda dasanın ilk lord kuralı ve dönem payları basılı kaynakla karşılaştırılmayı bekliyor.")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: varsha.sidereal_return_v1")).toBeInTheDocument();
  expect(screen.queryByText(/Doğum saati bilinmiyor/)).toBeNull();
  expect(screen.getByRole("spinbutton", { name: "Yaş (tamamlanan yıl)" })).toHaveValue(26);
});

test("a period opens its nine sub-periods; one path is open at a time; the keyboard works", async () => {
  renderTree();
  const root = await tree();
  click(items(root, 1)[0]);
  let kids = items(root, 2);
  expect(kids).toHaveLength(9);
  // Alt dönemler üst dönemin lorduyla başlar ve üst dönemin bitişinde biter.
  expect(kids[0]).toHaveAccessibleName("Mars–Mars, 2026-01-01 09:28 – 2026-01-02 15:18");
  expect(kids[8]).toHaveAccessibleName("Mars–Ay, 2026-01-20 22:13 – 2026-01-22 16:50");
  expect(kids.every((k) => !k.hasAttribute("aria-expanded"))).toBe(true);
  expect(items(root, 1)[0]).toHaveAttribute("aria-expanded", "true");
  // Başka bir dönemi açmak öncekini kapatır.
  click(items(root, 1)[1]);
  kids = items(root, 2);
  expect(kids).toHaveLength(9);
  expect(kids[0]).toHaveAccessibleName("Rahu–Rahu, 2026-01-22 16:50 – 2026-01-30 22:04");
  expect(items(root, 1)[0]).toHaveAttribute("aria-expanded", "false");
  // Klavye: sol ok kapatır, sağ ok açar.
  const rahu = items(root, 1)[1];
  rahu.focus();
  fireEvent.keyDown(rahu, { key: "ArrowLeft" });
  expect(items(root, 2)).toHaveLength(0);
  fireEvent.keyDown(rahu, { key: "ArrowRight" });
  expect(items(root, 2)).toHaveLength(9);
  // Alt dönemler istekle yüklenmez: rapor bir kez istenmiştir.
  expect(api.computeVarsha).toHaveBeenCalledTimes(1);
});

test("the year is shared with the Varshaphala panel: stepping it in one moves both", async () => {
  renderTree({ system: "mudda" }, { withVarsha: true });
  await tree();
  const panel = screen.getByRole("region", { name: "varsha" });
  await within(panel).findByRole("table", { name: "Varshaphala: yılın olguları" });
  // Aynı girdi: iki panel aynı raporu ayrı ayrı ister (doğum yeri, yaşsız).
  expect(vi.mocked(api.computeVarsha).mock.calls.map((c) => c[1])).toEqual([null, null]);
  fireEvent.click(within(panel).getByRole("button", { name: "Sonraki yıl" }));
  // 27 yaşı: 15 + 27 − 2 = 40, artan 4: Rahu.
  await waitFor(() => expect(items(screen.getByRole("tree", { name: "Mudda dasa" }), 1)[0]).toHaveAccessibleName("Rahu, 2027-01-01 15:41 – 2027-02-25 10:35"));
  expect(screen.getByText("İlk lord Rahu: doğum nakshatrası 15 (Swati), tamamlanan yıl 27, artan 4.")).toBeInTheDocument();
  for (const input of screen.getAllByRole("spinbutton", { name: "Yaş (tamamlanan yıl)" })) expect(input).toHaveValue(27);
  // Ağaçtaki "Bu yıl" iki paneli de süren yıla döndürür.
  fireEvent.click(within(screen.getByRole("region", { name: "tree" })).getByRole("button", { name: "Bu yıl" }));
  await waitFor(() => expect(within(panel).getByRole("spinbutton", { name: "Yaş (tamamlanan yıl)" })).toHaveValue(26));
  for (const button of screen.getAllByRole("button", { name: "Bu yıl" })) expect(button).toHaveAttribute("aria-pressed", "true");
});

test("unknown birth time: the periods are listed and marked approximate", async () => {
  vi.mocked(api.computeVarsha).mockResolvedValue(ok(fixture().varsha.time_unknown));
  renderTree({ system: "mudda" }, { opened: unknownTimeOpened() });
  const root = await tree();
  expect(vi.mocked(api.computeVarsha).mock.calls[0][0].time_known).toBe(false);
  expect(items(root, 1)).toHaveLength(9);
  expect(screen.getByText("Doğum saati bilinmiyor: dönüş anı, dönem tarihleri ve ilk lord yaklaşıktır (≈).")).toBeInTheDocument();
});

test("an error is shown with the year controls, and retry asks again", async () => {
  vi.mocked(api.computeVarsha).mockResolvedValueOnce(failure);
  renderTree();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.queryByRole("tree")).toBeNull();
  expect(screen.getByRole("button", { name: "Bu yıl" })).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await tree();
  expect(api.computeVarsha).toHaveBeenCalledTimes(2);
});

test("loading: nothing under a second, then a skeleton, then the tree", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<VarshaView>) => void;
    vi.mocked(api.computeVarsha).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderTree();
    act(() => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    act(() => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    await act(async () => resolve(ok()));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getByRole("tree", { name: "Mudda dasa" })).toBeInTheDocument();
  } finally {
    vi.useRealTimers();
  }
});

test("the fixture's mudda order is Rust's and the nine top periods follow it cyclically", () => {
  const { mudda } = FX.varsha.current.report;
  const order = FX.varsha.mudda_order;
  expect(order).toEqual(["Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury", "Ketu", "Venus"]);
  const top = mudda.periods.filter((p) => p.level === 1).map((p) => p.lords[0]);
  const first = order.indexOf(mudda.first_lord);
  expect(top).toEqual(order.map((_, k) => order[(first + k) % 9]));
  expect(mudda.periods).toHaveLength(90);
});
````

Uçtan uca akışlar (**[kontrolör]** koşar):

**Yeni dosya:** `app/e2e/varsha.desk.spec.ts`

````ts
// Varshaphala akışları (Plan 27). Ayrı dosyadır: `desk.spec.ts` paralel planlarla paylaşılır; "e2e"
// projesi adı `desk.spec.ts` ile biten her dosyayı koşar.
import { expect, test, type Page } from "@playwright/test";
import { ipcCalls, mockIpc } from "./ipc";

type VarshaCall = { input: { jd_ut: number; time_known: boolean; settings: { schema_version: number } }; age: number | null; place: { lat: number; lon: number; tz: unknown } };

async function togglePanel(page: Page) {
  await page.keyboard.press("Meta+k");
  const palette = page.getByRole("dialog", { name: "Komut paleti" });
  await palette.getByRole("combobox").fill("varshaphala");
  await palette.getByRole("option", { name: /Varshaphala panelini aç\/kapat/ }).click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
}

test("varshaphala: the annual chart, the muntha, year stepping, save as an event chart and the mudda dasha tree", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  const calls = async () => (await ipcCalls(page, "compute_varsha")) as VarshaCall[];
  // Araştırma düzeni: dasa ağacı burada durur. Rapor panel açılmadan istenmez (K1).
  await page.keyboard.press("Meta+Digit2");
  await expect(page.getByRole("region", { name: "Dasa ağacı Vimshottari", exact: true })).toBeVisible();
  expect(await calls()).toHaveLength(0);

  await togglePanel(page);
  const panel = page.getByRole("region", { name: "Varshaphala" });
  await expect(panel).toBeFocused();
  await expect(panel).toBeInViewport();
  const facts = panel.getByRole("table", { name: "Varshaphala: yılın olguları" });
  await expect(facts).toBeVisible();
  // Yaş gönderilmez: süren yılı Rust bulur. Yer açık haritanın doğum yeridir.
  expect(await calls()).toHaveLength(1);
  expect((await calls())[0]).toMatchObject({ input: { jd_ut: 2451545, time_known: true, settings: { schema_version: 2 } }, age: null, place: { lat: 28.6139, lon: 77.209, tz: { Iana: "Asia/Kolkata" } } });

  // Delhi fikstürü: 26 yıl tamamlanmış; muntha Aslan, yıl lagnası Oğlak.
  await expect(facts.locator('tr[data-fact="year"] td')).toHaveText("27. yıl (tamamlanan yıl 26)");
  await expect(facts.locator('tr[data-fact="start"] td')).toHaveText("2026-01-01 09:28 (UTC+05:30)");
  await expect(facts.locator('tr[data-fact="muntha"] td')).toHaveText("Asl · yıllık haritada 8. ev · doğum haritasında 3. ev · lordu Güneş");
  await expect(facts.locator('tr[data-fact="lagna"] td')).toHaveText("Oğl 22°04′ · lordu Satürn");
  await expect(panel.getByRole("spinbutton", { name: "Yaş (tamamlanan yıl)" })).toHaveValue("26");
  const chart = panel.getByRole("group", { name: "Yıllık harita, 27. yıl" });
  await expect(chart.locator("[data-graha]")).toHaveCount(9);
  await expect(chart.locator(".chart-mark")).toHaveText("Mu");
  const positions = panel.getByRole("table", { name: "Yıllık harita konumları" });
  await expect(positions.locator("tbody tr")).toHaveCount(10);
  await expect(positions.locator('tbody tr[data-graha="Sun"]').locator("th, td")).toHaveText(["Güneş", "Yay", "16°30′", "Purva Ashadha", "12"]);
  await expect(panel.getByText("Yöntem: varsha.sidereal_return_v1")).toBeVisible();
  // Tabloda denetim az olabilir: kaydırma kabı klavye odağı alır.
  const scroll = panel.getByRole("group", { name: "Varshaphala: yılın olguları" });
  await scroll.focus();
  await expect(scroll).toBeFocused();

  // Ortak vurgu: yıllık haritanın tablosundaki graha, doğum haritasındaki grahayı da vurgular.
  await positions.getByRole("button", { name: "Satürn" }).hover();
  await expect(page.getByRole("region", { name: "Harita D1", exact: true }).locator('svg [data-graha="Saturn"]').first()).toHaveClass(/chart-planet--highlight/);

  // Sonraki yıl: 27 yaşı istenir; muntha bir burç ilerler.
  await panel.getByRole("button", { name: "Sonraki yıl" }).click();
  await expect(facts.locator('tr[data-fact="year"] td')).toHaveText("28. yıl (tamamlanan yıl 27)");
  await expect(facts.locator('tr[data-fact="muntha"] td')).toHaveText("Baş · yıllık haritada 5. ev · doğum haritasında 4. ev · lordu Merkür");
  await expect(panel.getByRole("group", { name: "Yıllık harita, 28. yıl" })).toBeVisible();
  expect((await calls()).map((c) => c.age)).toEqual([null, 27]);

  // Olay haritası olarak kaydetme: kaynak haritanın kimliği, yaş ve yer seçimi gider; an ve yer
  // arayüzden gitmez. Harita açılmaz: başlık aynı danışanda ve aynı haritada durur.
  await panel.getByRole("button", { name: "Olay haritası olarak kaydet" }).click();
  await expect(panel.getByRole("status")).toHaveText("Olay haritası kaydedildi: Varshaphala 2027-01-01");
  expect(await ipcCalls(page, "save_varsha_chart")).toEqual([{ chartId: 1, age: 27, home: false }]);
  await expect(panel.getByRole("button", { name: "Olay haritası olarak kaydet" })).toHaveAttribute("aria-disabled", "true");
  await expect(panel.getByRole("group", { name: "Yıllık harita, 28. yıl" })).toBeVisible();

  // Dasa ağacında mudda dasa: aynı yılı gösterir (27 yaşı: ilk lord Rahu).
  const tree = page.getByRole("region", { name: "Dasa ağacı Vimshottari", exact: true });
  await tree.getByRole("combobox", { name: "Sistem" }).selectOption("mudda");
  const mudda = page.getByRole("region", { name: "Dasa ağacı Mudda dasa", exact: true });
  const items = mudda.getByRole("tree", { name: "Mudda dasa" }).getByRole("treeitem");
  await expect(items).toHaveCount(9);
  await expect(items.first()).toHaveAccessibleName("Rahu, 2027-01-01 15:41 – 2027-02-25 10:35");
  await expect(mudda.getByText("İlk lord Rahu: doğum nakshatrası 15 (Swati), tamamlanan yıl 27, artan 4.")).toBeVisible();
  await expect(mudda.getByText(/^Doğrulanmamış tablo:/)).toBeVisible();
  // Bir dönem dokuz alt dönemini açar (istek yapılmaz).
  const before = (await calls()).length;
  await items.first().locator(".dasha-row").click();
  await expect(mudda.getByRole("treeitem")).toHaveCount(18);
  await expect(mudda.getByRole("treeitem").nth(1)).toHaveAccessibleName("Rahu–Rahu, 2027-01-01 15:41 – 2027-01-09 20:55");
  expect((await calls()).length).toBe(before);
  // Ağaçtaki "Bu yıl" Varshaphala panelini de süren yıla döndürür.
  await mudda.getByRole("button", { name: "Bu yıl" }).click();
  await expect(panel.getByRole("spinbutton", { name: "Yaş (tamamlanan yıl)" })).toHaveValue("26");
  await expect(facts.locator('tr[data-fact="year"] td')).toHaveText("27. yıl (tamamlanan yıl 26)");
  await expect(mudda.getByRole("treeitem").first()).toHaveAccessibleName("Mars, 2026-01-01 09:28 – 2026-01-22 16:50");
  // Sistem seçimi düzenle birlikte kaydedilir.
  await expect
    .poll(async () => {
      const saved = (await ipcCalls(page, "save_view_state")).at(-1) as { viewState: { panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: { system?: string } }[] }[] } } } | undefined;
      const layout = saved?.viewState.panel_layout;
      return layout?.layouts
        .find((l) => l.id === layout.active_layout_id)
        ?.panels.filter((p) => p.kind === "dasha_tree")
        .map((p) => p.settings.system);
    })
    .toEqual(["mudda"]);

  // Aynı komut paneli kaldırır; odak gövdeye düşmez.
  await togglePanel(page);
  await expect(panel).toHaveCount(0);
  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
});

test("varshaphala with an unknown birth time: no annual chart and no muntha; the mudda dasha is approximate", async ({ page }) => {
  await mockIpc(page, { session: true, unknownTime: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+Digit2");
  await togglePanel(page);
  const panel = page.getByRole("region", { name: "Varshaphala" });
  await expect(panel.getByText(/^Doğum saati bilinmiyor: yıllık harita ve muntha hesaplanmadı/)).toBeVisible();
  const calls = (await ipcCalls(page, "compute_varsha")) as VarshaCall[];
  expect(calls.map((c) => c.input.time_known)).toEqual([false]);
  await expect(panel.getByRole("table")).toHaveCount(1);
  await expect(panel.locator("tr[data-fact]")).toHaveCount(4);
  await expect(panel.locator('tr[data-fact="muntha"]')).toHaveCount(0);
  await expect(panel.locator("svg.chart")).toHaveCount(0);
  await expect(panel.getByRole("button", { name: "Olay haritası olarak kaydet" })).toHaveCount(0);

  const tree = page.getByRole("region", { name: "Dasa ağacı Vimshottari", exact: true });
  await tree.getByRole("combobox", { name: "Sistem" }).selectOption("mudda");
  const mudda = page.getByRole("region", { name: "Dasa ağacı Mudda dasa", exact: true });
  await expect(mudda.getByRole("treeitem")).toHaveCount(9);
  await expect(mudda.getByText("Doğum saati bilinmiyor: dönüş anı, dönem tarihleri ve ilk lord yaklaşıktır (≈).")).toBeVisible();
});
````

Görsel testler (**[kontrolör]** koşar ve iki temeli üretir):

**Yeni dosya:** `app/e2e/varsha.visual.spec.ts`

````ts
// Varshaphala görsel regresyonu (Plan 27; yalnızca yerel: `pnpm e2e:visual`). Ayrı dosyadır:
// `visual.spec.ts` paralel planlarla paylaşılır; "visual" projesi adı `visual.spec.ts` ile biten
// her dosyayı koşar. Temel görüntüler `e2e/varsha.visual.spec.ts-snapshots/` altındadır (darwin
// son ekli) ve yerelde üretilir: `pnpm --dir app exec playwright test --project=visual
// varsha.visual --update-snapshots`.
import { expect, test } from "@playwright/test";
import { mockIpc } from "./ipc";

// Yıllık harita (gerçek Delhi haritasının 27. yılı), muntha işaretiyle; açık ve koyu tema.
for (const theme of ["light", "dark"] as const) {
  test(`varshaphala annual chart ${theme}`, async ({ page }) => {
    await mockIpc(page, { session: true });
    await page.addInitScript((t) => localStorage.setItem("jyotish-desk.theme.v1", t), theme);
    await page.goto("/");
    await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
    // Kenar çubuğu kapalı: temel görüntü haritanın kendisidir, pano genişliğine bağlı değil.
    await page.keyboard.press("Meta+Backslash");
    await expect(page.getByRole("complementary", { name: "Danışanlar" })).toHaveCount(0);
    await page.keyboard.press("Meta+k");
    const palette = page.getByRole("dialog", { name: "Komut paleti" });
    await palette.getByRole("combobox").fill("varshaphala");
    await palette.getByRole("option", { name: /Varshaphala panelini aç\/kapat/ }).click();
    const svg = page.getByRole("region", { name: "Varshaphala" }).locator("svg.chart");
    await expect(svg.locator("[data-graha]")).toHaveCount(9);
    await expect(svg.locator(".chart-mark")).toHaveText("Mu");
    await svg.scrollIntoViewIfNeeded();
    await expect(svg).toHaveScreenshot(`varsha-${theme}.png`);
  });
}
````

Komutların taklidi (paylaşılan dosya):

````diff
diff --git a/app/e2e/ipc.ts b/app/e2e/ipc.ts
index 751f48f..b5354d5 100644
--- a/app/e2e/ipc.ts
+++ b/app/e2e/ipc.ts
@@ -153,6 +153,14 @@ export async function mockIpc(page: Page, { session, unknownTime = false, partne
         compute_yogas: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.yogas : fixture.yogas_time_unknown),
         // Avasthalar: fikstürdeki iki gerçek sonuç (doğum saati bilinen ve bilinmeyen girdi).
         compute_avasthas: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.avasthas : fixture.avasthas_time_unknown),
+        // Varshaphala (Plan 27): fikstürdeki gerçek sonuçlar. Doğum saati bilinmiyorsa Rust yıllık
+        // haritayı ve munthayı boş döner; 27 yaşı sonraki yıldır, öteki her istek (yaşsız istek
+        // dahil) süren yıldır. Kayıt, açık haritanın kaydından türetilmiş bir olay haritası döner.
+        compute_varsha: (args) => {
+          if (!(args.input as { time_known: boolean }).time_known) return fixture.varsha.time_unknown;
+          return args.age === 27 ? fixture.varsha.next : fixture.varsha.current;
+        },
+        save_varsha_chart: (args) => ({ ...fixture.opened.chart_record, id: 900 + (args.age as number), uuid: "00000000-0000-4000-8000-000000000031", kind: "Event", label: `Varshaphala ${args.age === 27 ? 2027 : 2026}-01-01` }),
         // Uyum: eş haritanın girdisi ve kısa kimliği; başka kimlik `null` döner (harita yok).
         // Rapor yeniden hesaplanmaz, rollerden seçilir: açık harita damat rolündeyse `report`,
         // gelin rolündeyse `report_swapped`; gelin rolündeki eşin doğum saati bilinmiyorsa
````

- [ ] **Step 2: Testlerin düştüğünü gör**

Run: `pnpm --dir app test 2>&1 | grep -E "Test Files|Tests |FAIL" | sort | uniq`
Expected: `Test Files  1 failed | 98 passed (99)`, `Tests  990 passed (990)`: `ui/dasha/MuddaDashaTree.test.tsx` yüklenemez (`./muddaSystem` henüz yok); öteki bütün testler geçer.

- [ ] **Step 3: Mudda sistemi, mudda ağacı ve panele bağlanması**

**Yeni dosya:** `app/ui/dasha/muddaSystem.tsx`

````tsx
// Dasa ağacının sistem seçimi: Vimshottari, chara dasha ve mudda dasa (yol haritası §4.14).
// Mudda dasa yalnızca ağaçta vardır: dasa şeridi transit anına bağlıdır ve `system.tsx`'teki iki
// sistemi sunmayı sürdürür. Ayar panelindir ve düzenle kaydedilir; tanınmayan değer Vimshottari
// sayılır.
import type { TFunction } from "i18next";
import { useId } from "react";
import { useTranslation } from "react-i18next";
import { DASHA_SYSTEMS, dashaSystem, type DashaSystem } from "./system";

export const TREE_SYSTEMS = [...DASHA_SYSTEMS, "mudda"] as const;
export type TreeSystem = DashaSystem | "mudda";

export function treeSystem(settings: Record<string, unknown>): TreeSystem {
  return settings.system === "mudda" ? "mudda" : dashaSystem(settings);
}

/** Sistemin görünen adı. Mudda dasanın adı `varsha` bloğundadır. */
export function treeSystemName(system: TreeSystem, t: TFunction): string {
  return system === "mudda" ? t("varsha.mudda.system") : t(`dasha.systems.${system}`);
}

/** "Sistem" seçicisi: görünür etiketli `select`; dasa ağacının araç çubuğuna konur. */
export function TreeSystemSelect({ value, onChange }: { value: TreeSystem; onChange(system: TreeSystem): void }) {
  const { t } = useTranslation();
  const id = useId();
  return (
    <>
      <label htmlFor={`${id}-system`}>{t("dasha.system")}</label>
      <select id={`${id}-system`} value={value} onChange={(e) => onChange(treeSystem({ system: e.target.value }))}>
        {TREE_SYSTEMS.map((s) => (
          <option key={s} value={s}>
            {treeSystemName(s, t)}
          </option>
        ))}
      </select>
    </>
  );
}
````

**Yeni dosya:** `app/ui/dasha/MuddaDashaTree.tsx`

````tsx
// Mudda dasa ağacı (yol haritası §4.14): seçilen yılın dokuz dönemi ve alt dönemleri. Dönemler,
// ilk lord ve bölümden artan `compute_varsha`'dan gelir ve panel açıkken istenir (K1); burada
// hesap yoktur (C4). Yıl, Varshaphala paneliyle ortak depodadır (`varsha/yearStore`): iki panel
// aynı yılı gösterir. Dönemler yere bağlı değildir; istek doğum yeriyle yapılır ve anlar doğum
// yerinin yıl başındaki ofsetiyle yazılır. Açık dönem panelin kendi durumudur (kaydedilmez).
// Kural bellekten aktarılmıştır: sahibin onayına dek "doğrulanmamış tablo" notu durur (Y4).
import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { DashaPeriod, Graha, OpenedClient, VarshaView } from "../bindings";
import { inputKey } from "../modules/input";
import { chartPlace } from "../modules/muhurta";
import { useLastData } from "../modules/useModule";
import { useVarsha } from "../modules/useVarsha";
import { formatJdMinute } from "../panels/format";
import { nakshatraName } from "../panels/labels";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "../panels/states";
import "../panels/varsha.css";
import { VarshaYearControls } from "../varsha/YearControls";
import { useVarshaAge } from "../varsha/yearStore";
import { zoneLabel } from "../varsha/zone";
import { PeriodTree, type PeriodLevel } from "./PeriodTree";
import { childrenOf } from "./tree";

export function MuddaDashaTree({ opened }: { opened: OpenedClient }) {
  const { age: selected, setAge } = useVarshaAge(opened.chart_record.id);
  const module = useVarsha(opened, selected, chartPlace(opened));
  // Yıl değişirken önceki yılın ağacı yerinde durur; başka harita eskiyi unutturur.
  const view = useLastData(module, inputKey(opened));
  const phase = usePanelLoadPhase(module.loading);
  return (
    <>
      <div className="module-toolbar">
        <VarshaYearControls age={selected ?? view?.report.age ?? null} current={selected === null} onChange={setAge} />
      </div>
      {module.error && <PanelError error={module.error} onRetry={module.retry} />}
      {module.error === null && view === null && phase === "skeleton" && <PanelSkeleton />}
      {module.error === null && view !== null && <Tree key={view.report.age} view={view} />}
    </>
  );
}

function Tree({ view }: { view: VarshaView }) {
  const { t } = useTranslation();
  const { report, offset_secs: offset } = view;
  const { mudda } = report;
  // Tek açık yol: bir dönemi açmak ötekini kapatır.
  const [open, setOpen] = useState<Graha | null>(null);
  const at = (jd: number | null) => formatJdMinute(jd, offset);
  const dates = (p: DashaPeriod) => `${at(p.start_jd_ut)} – ${at(p.end_jd_ut)}`;
  const name = (g: Graha) => t(`grahaName.${g}`);

  const level = (path: readonly Graha[]): PeriodLevel => ({
    status: "ok",
    nodes: childrenOf(mudda.periods, path).map((p) => {
      const lord = p.lords[p.lords.length - 1];
      const isOpen = p.has_children && open === lord;
      return {
        key: p.lords.join(","),
        level: p.level,
        name: name(lord),
        label: `${p.lords.map(name).join("–")}, ${dates(p)}`,
        dates: dates(p),
        expandable: p.has_children,
        open: isOpen,
        kids: isOpen ? level(p.lords) : undefined,
        toggle: () => setOpen(isOpen ? null : lord),
      };
    }),
  });

  return (
    <>
      <p className="panel-notice" role="note">
        {t("varsha.mudda.unverified")}
      </p>
      {!report.time_known && (
        <p className="panel-notice" role="note">
          {t("varsha.mudda.approx")}
        </p>
      )}
      <p className="varsha-summary">
        <span>{t("varsha.mudda.summary", { number: report.age + 1, age: report.age, start: at(report.start_jd_ut), end: at(report.end_jd_ut), zone: zoneLabel(offset) })}</span>
        <span>{t("varsha.mudda.first", { lord: name(mudda.first_lord), nakshatra: mudda.birth_nakshatra, name: nakshatraName(mudda.birth_nakshatra), age: mudda.age, remainder: mudda.remainder })}</span>
      </p>
      <PeriodTree label={t("varsha.mudda.system")} roots={level([])} />
      <p className="panel-message">{t("varsha.method", { method: report.method })}</p>
    </>
  );
}
````

Dasa ağacı paneli ve panelin adı (paylaşılan dosyalar):

````diff
diff --git a/app/ui/dasha/DashaTreePanel.tsx b/app/ui/dasha/DashaTreePanel.tsx
index 59e4324..f96831b 100644
--- a/app/ui/dasha/DashaTreePanel.tsx
+++ b/app/ui/dasha/DashaTreePanel.tsx
@@ -7,8 +7,9 @@ import { useSession } from "../session/store";
 import { useLayoutStore, useViewStore } from "../view/store";
 import { CharaDashaTree } from "./CharaDashaTree";
 import "./dasha.css";
+import { MuddaDashaTree } from "./MuddaDashaTree";
+import { treeSystem, TreeSystemSelect } from "./muddaSystem";
 import { PeriodTree, type PeriodLevel } from "./PeriodTree";
-import { dashaSystem, SystemSelect } from "./system";
 import { childrenOf, formatJdDate, isOpen, MAX_DASHA_LEVEL, PRELOADED_LEVEL } from "./tree";
 import { pathKey, useLazyDasha, type LazyEntry } from "./useLazyDasha";
 
@@ -23,13 +24,14 @@ export function DashaTreePanel({ panel }: PanelProps) {
   const { opened } = useSession();
   const { setPanelSettings } = useLayoutStore();
   if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
-  const system = dashaSystem(panel.settings);
+  // Üçüncü sistem mudda dasadır (yol haritası §4.14): yalnızca ağaçta, `muddaSystem.tsx`.
+  const system = treeSystem(panel.settings);
   return (
     <div className="dasha-panel">
       <div className="module-toolbar">
-        <SystemSelect value={system} onChange={(next) => setPanelSettings(panel.id, { system: next })} />
+        <TreeSystemSelect value={system} onChange={(next) => setPanelSettings(panel.id, { system: next })} />
       </div>
-      {system === "chara" ? <CharaDashaTree opened={opened} /> : <VimshottariTree opened={opened} />}
+      {system === "mudda" ? <MuddaDashaTree opened={opened} /> : system === "chara" ? <CharaDashaTree opened={opened} /> : <VimshottariTree opened={opened} />}
     </div>
   );
 }
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index 58ac9ef..ad85584 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -1,7 +1,7 @@
 // Panel türü → bileşen (spec §3.3). Yeni panel türü yalnızca buraya eklenir.
 import { DashaStripPanel } from "../dasha/DashaStripPanel";
 import { DashaTreePanel } from "../dasha/DashaTreePanel";
-import { dashaSystem } from "../dasha/system";
+import { treeSystem, treeSystemName } from "../dasha/muddaSystem";
 import { isKnownPanelKind, type KnownPanelKind } from "../layout/model";
 import { NotesPanel } from "../notes/NotesPanel";
 import { ArgalaPanel } from "./ArgalaPanel";
@@ -36,7 +36,7 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   graha_table: { titleKey: "panel.graha_table", component: GrahaTablePanel },
   dasha_tree: {
     titleKey: "panel.dasha_tree",
-    nameDetail: (p, t) => t(`dasha.systems.${dashaSystem(p.settings)}`),
+    nameDetail: (p, t) => treeSystemName(treeSystem(p.settings), t),
     component: DashaTreePanel,
   },
   dasha_strip: { titleKey: "panel.dasha_strip", component: DashaStripPanel },
````

- [ ] **Step 4: Testler ve kapılar**

```bash
pnpm --dir app exec vitest run ui/dasha 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
cargo fmt --all --check
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
git status --short
```
Expected: `ui/dasha` altındaki bütün dosyalar geçer (var olan dasa testleri üç seçenekli sistem listesiyle değişmeden geçer; yeni dosya `8 passed`); tamamı `99 passed (99)`, `998 passed (998)`; `✓ built` (e2e dosyalarının tip denetimi dahil); fmt çıktısız; sayaç `0`; `git status --short` sekiz dosya gösterir ve aralarında `app/e2e/varsha.visual.spec.ts-snapshots/` ya da `app/test-results/` yoktur (yürüten ajan e2e ve görsel testleri koşmadı).

- [ ] **Step 5: [kontrolör] Uçtan uca akışlar ve görsel temeller**

Yürüten ajan bu adımı **atlar** ve ilerleme notlarına "Task 7 Step 5 kontrolöre bırakıldı" yazar; Step 6'ya geçer. Kontrolör, yürütme bittikten sonra (commit'lerden sonra da olabilir) özel portta koşar (Global Constraints "[kontrolör] E2e ve özel port"):

```bash
lsof -i :14527
pnpm --dir app exec playwright test -c playwright.p27.config.ts --project=e2e 2>&1 | tail -1
pnpm --dir app exec playwright test -c playwright.p27.config.ts --project=visual varsha.visual --update-snapshots 2>&1 | tail -1
pnpm --dir app exec playwright test -c playwright.p27.config.ts --project=visual 2>&1 | tail -1
ls app/e2e/varsha.visual.spec.ts-snapshots
rm -rf app/test-results
git status --short
```
Expected: `lsof` çıktısız (çıktı varsa yapılandırmadaki port değiştirilir); e2e `35 passed` (33 + iki yeni akış); ilk görsel koşu iki temeli yazar ve `2 passed`; ikinci görsel koşu `17 passed` (var olan on beş temel değişmedi; iki yeni temel kararlı); `ls` `varsha-dark-visual-darwin.png` ve `varsha-light-visual-darwin.png` gösterir. Kontrolör iki PNG'ye gözle bakar (Kuzey Hint biçiminde yıllık harita; dokuz graha; Aslan hücresinde "Mu"; koyu temada okunur) ve ayrı bir commit'le ekler: `git add app/e2e/varsha.visual.spec.ts-snapshots && git commit -m "test(visual): Varshaphala annual chart baselines" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"`. Geçici yapılandırma dosyası silinir ve commit edilmez.

- [ ] **Step 6: Commit**

```bash
git add app/ui/dasha/muddaSystem.tsx app/ui/dasha/MuddaDashaTree.tsx app/ui/dasha/MuddaDashaTree.test.tsx app/ui/dasha/DashaTreePanel.tsx app/ui/panels/registry.ts app/e2e/ipc.ts app/e2e/varsha.desk.spec.ts app/e2e/varsha.visual.spec.ts
git diff --cached --stat | tail -1
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): mudda dasha view in the dasha tree; Varshaphala end-to-end flows and visual spec" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: `8 files changed, 500 insertions(+), 6 deletions(-)`; tarama çıktısız; `git status --short` boş.

---

### Task 8: Kapılar, REVIEW.md ve elle denetim kapısı

Yol haritası §5 Plan 27 görev (8) ve "Sahibin denetimi"; Y3, Y4. Bu görev iki belge dosyası üretir (REVIEW kaydı ve el örnekleri kılavuzunun satırı), bütün kapıları koşar ve el örneklerini bağımsız olarak yeniden türetir. Kod değişmez; bulunan bir hata kendi düzeltme commit'iyle çözülür ve bu görev baştan koşulur. Aşağıda `BASE`, Task 1 Step 1'de `git log -1 --format=%h` ile okunan commit'tir (bu plan yazılırken `9727c55`).

**Files:**
- Create: `tests/golden/modules/varsha/REVIEW.md`
- Modify: `tests/golden/modules/README.md` (paylaşılan dosya; bir satır ve bir paragraf)

**Interfaces:**
- Consumes: Task 1–7'nin tamamı.
- Produces: okuma kurallarının, tabloların ve sahibin onayını bekleyen on iki varyant noktasının kaydı; yeşil kapılar; bağımsız yeniden türetmenin sonucu; kural ve gizlilik taramaları; sahibine devredilen **[insan]** ve kontrolöre bırakılan **[kontrolör]** maddeleri (ilerleme notlarına yazılır).

- [ ] **Step 1: REVIEW.md ve kılavuz satırı**

**Yeni dosya:** `tests/golden/modules/varsha/REVIEW.md`

````markdown
# Varshaphala el örnekleri: okuma kuralları, tablolar, onay bekleyen noktalar ve gözden geçirme kaydı

Kaynak: eksikler yol haritası §4.14 (ilk yarı) ve Plan 27'nin kararları; klasik dayanak Tajika
geleneğinin yıllık harita (varshaphala) bölümleridir (genel gönderme; alıntı yapılmaz). Bu
dizinde iki dizi durur:

- `muntha/`: iki lagna boylamı (doğum haritasının ve yıllık haritanın lagnası; sidereal sayılır)
  ve tamamlanan yıl. Her dosya munthanın burcunu, lordunu, iki haritadaki evini ve iki lagnanın
  lordunu yazar.
- `mudda/`: Ay'ın sidereal boylamı, tamamlanan yıl ve yılın gün olarak uzunluğu. Her dosya doğum
  nakshatrasını, ilk lord kuralının bölümden artanını, ilk lordu ve dokuz dönemin bitişini (yılın
  başından gün sayısı) yazar; üç dosya ilk dönemin dokuz alt dönemini de yazar.

Dönüş anı bir tablo değildir ve burada örneği yoktur: gerçek motorla
`app/src-tauri/tests/varsha.rs`'te denetlenir (yıllık haritanın Güneş'i doğumdaki sidereal
boylamından bir yay saniyesinden az ayrışır; aynı dönüşü olay araması da bir saniye içinde bulur).

Ephemeris yoktur. Değerler kuraldan türetilmiştir ve `derivation` alanında adım adım yazılıdır;
motor çıktısı buraya kopyalanmaz (K9). Bir değer motorla uyuşmazsa önce kural yeniden okunur.
Testler: `crates/jyotish-core/tests/varsha_golden.rs`.

## Okuma kuralları

1. **Tamamlanan yıl.** Yıllar doğumdan sayılır: tamamlanan yıl 0 doğum yılıdır (doğumdan ilk
   dönüşe), 1 ilk dönüşle başlayan yıldır. "N. yıl" diye yazılan sayı tamamlanan yılın bir
   fazlasıdır. En büyük tamamlanan yıl 120'dir.
2. **Dönüş anı.** Yıl, Güneş'in doğumdaki boylamına döndüğü anda başlar. Boylam her zodyak kipinde
   sidereal boylamdır (haritanın seçili ayanamsasıyla). Yılın sonu sonraki dönüştür; yılın
   uzunluğu iki dönüşün arasıdır (yaklaşık 365,2564 gün).
3. **Yıllık harita.** Dönüş anının haritasıdır: doğum haritasının ayarlarıyla (ayanamsa, düğüm, ev
   sistemi, zodyak kipi) ve seçilen yerde hesaplanır. Yer doğum yeridir; astrolog isterse bulunulan
   yeri seçer. Yer dönüş anını değiştirmez, yalnızca lagnayı ve evleri değiştirir.
4. **Muntha.** Doğumda lagnanın burcundadır ve tamamlanan her yılda bir burç ilerler: burcu, doğum
   lagnasının burcundan tamamlanan yıl kadar burç ötesidir (on ikide bir başa döner). Lordu o
   burcun lordudur. Evi burç evidir: doğum lagnasından ve yıllık lagnadan ayrı ayrı sayılır.
   Burçlar rasi çerçevesindedir. Munthanın yıl içindeki derece ilerleyişi hesaplanmaz.
5. **Lagna lordları.** Yıllık lagnanın ve doğum lagnasının lordu, burçlarının lordudur (aşağıdaki
   tablo). Yıl lordu seçilmez (Plan 28).
6. **Mudda dasanın ilk lordu.** Doğum nakshatrasının sırası (Ay'ın sidereal boylamından; Ashwini 1,
   Revati 27) ile tamamlanan yıl toplanır, 2 çıkarılır ve 9'a bölünür. Bölümden artan sayı mudda
   sırasındaki lordu verir: 1 Güneş, 2 Ay, 3 Mars, 4 Rahu, 5 Jüpiter, 6 Satürn, 7 Merkür, 8 Ketu,
   0 Venüs. Toplam eksiye düşerse (Ashwini ve doğum yılı: −1) artan yine 0–8 arasında okunur
   (−1 = −1 × 9 + 8).
7. **Dönemler.** Dokuz dönem ilk lorddan başlar ve mudda sırasını çevrimsel izler. İlk dönem yılın
   başında (dönüş anında) başlar ve tamdır: Ay'ın nakshatrada geçtiği yay için kesinti yapılmaz.
   Her dönem yılın `gün(lord) / 360` kadarını alır; yıl dönüşten dönüşe sayıldığı için dokuz dönem
   yılı boşluksuz doldurur ve son dönem sonraki dönüşte biter.
8. **Alt dönemler.** Her dönemin dokuz alt dönemi o dönemin lordundan başlar, aynı sırayı izler ve
   dönemi aynı paylarla böler.
9. **Doğum saati bilinmiyorsa** yıllık harita, muntha ve lagna lordları verilmez (hepsi lagna
   ister). Dönüş anı ve mudda dasa öğle konumlarından hesaplanır ve yaklaşık sayılır.
10. **Sayı prashnası.** Doğum lagnası sayıdan gelir ve muntha o lagnadan sayılır; yıllık harita
    dönüş anının kendi lagnasıyla hesaplanır.

## Tablolar

Burç lordları (uygulamanın ortak tablosu; düğümler burç lordu değildir):

| Burç | Koç | Boğa | İkizler | Yengeç | Aslan | Başak | Terazi | Akrep | Yay | Oğlak | Kova | Balık |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Lord | Mars | Venüs | Merkür | Ay | Güneş | Merkür | Venüs | Mars | Jüpiter | Satürn | Satürn | Jüpiter |

Mudda dasa: sıra, 360 günlük yıldaki gün ve ilk lord kuralının artanı. Günler Vimshottari
yıllarının üç katıdır (120 yıl → 360 gün).

| Sıra | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
|---|---|---|---|---|---|---|---|---|---|
| Lord | Güneş | Ay | Mars | Rahu | Jüpiter | Satürn | Merkür | Ketu | Venüs |
| Gün | 18 | 30 | 21 | 54 | 48 | 57 | 51 | 21 | 60 |
| Artan | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 0 |

Nakshatra bölümü: her nakshatra 13°20′'dır; sıra `tam kısım(boylam ÷ 13°20′) + 1`. Sınır sonraki
nakshatraya aittir (120° tam Magha'nın, 10. nakshatranın başıdır).

## Sahibin onayını bekleyen kurallar ve bilinen varyant noktaları

Kurallar bellekten aktarılmıştır (yol haritası Y4). Her satır basılı bir Tajika kaynağıyla
karşılaştırılmayı bekler; onaya dek dasa ağacının mudda görünümü "doğrulanmamış tablo" notunu
taşır. Yürüten ajan bu noktaları başka bir yazılımla ya da çevrimiçi bir kaynakla karşılaştırmaz.

| No | Nokta | Bu plandaki okuma | Bilinen öteki okuma | Durum |
|---|---|---|---|---|
| 1 | Dönüşün çerçevesi | Sidereal boylama dönüş, seçili ayanamsayla; her zodyak kipinde | Tropikal boylama dönüş (her yıl yaklaşık yirmi dakika erken gelir; lagnayı değiştirir) | Bekliyor |
| 2 | Yıllık haritanın yeri | Doğum yeri; astrolog bulunulan yeri seçebilir | Yalnızca doğum yeri; yalnızca o yıl yaşanan yer | Bekliyor |
| 3 | Munthanın sayımı | Doğum lagnasının burcu + tamamlanan yıl | Süren yılın sayısı (bir burç ileri) | Bekliyor |
| 4 | Munthanın çerçevesi | Rasi çerçevesi (tropikal-rasi kipinde tropikal lagna burcu) | Her kipte sidereal lagna burcu | Bekliyor |
| 5 | Munthanın derecesi | Hesaplanmaz; yalnızca burç | Lagnanın derecesiyle başlar, ayda 2°30′ ilerler | Bekliyor |
| 6 | İlk lord kuralının terimleri | Doğum nakshatrası + tamamlanan yıl − 2 | Süren yılın sayısı ile (çıkarılan sayı 3 olur; sonuç aynıdır) ya da çıkarmadan (iki lord kayar) | Bekliyor |
| 7 | Artanın lorda eşlenmesi | 1 Güneş … 8 Ketu, 0 Venüs | Artan 0 ilk lord sayılır (sıra bir kayar) | Bekliyor |
| 8 | Mudda sırası ve günler | Güneş 18, Ay 30, Mars 21, Rahu 54, Jüpiter 48, Satürn 57, Merkür 51, Ketu 21, Venüs 60 | — (toplam 360; sıra Vimshottari sırasıdır) | Bekliyor |
| 9 | İlk dönemin kesintisi | Yok: ilk dönem tamdır | Ay'ın doğum nakshatrasında geçtiği yay oranında ilk dönem kısalır | Bekliyor |
| 10 | Yılın uzunluğu | Dönüşten dönüşe; dönemler bu sürenin payıdır | 360 takvim günü (yılın sonunda beş gün boş); Güneş'in her 1° yolu bir gün | Bekliyor |
| 11 | Alt dönemler | Üst dönemin lordundan başlar; aynı paylar | Alt dönem verilmez | Bekliyor |
| 12 | Doğum saati bilinmeyen harita | Yıllık harita ve muntha verilmez; dönüş anı ve mudda dasa yaklaşık | Hiçbiri verilmez | Bekliyor |

Denetleyen (sahibi), tarih ve kullanılan basılı kaynak:

| Denetleyen | Tarih | Kaynak | Sonuç |
|---|---|---|---|
| — | — | — | Bekliyor: sahibine devredildi (Plan 27 Task 8) |

## Gözden geçirme kaydı

Gözden geçiren (örnekleri yazan dışında biri) her örnekte yalnızca `input`'tan başlar, motoru
çalıştırmadan ve Rust kaynağını okumadan, yukarıdaki okuma kurallarından ve tablolardan munthayı
ve lordları (`muntha/`) ya da doğum nakshatrasını, artanı, ilk lordu ve dönem bitişlerini
(`mudda/`) yeniden hesaplar; sonucunu `expected` ile karşılaştırır ve aşağıya adını, tarihini ve
bulduğu farkları yazar.

| Gözden geçiren | Tarih | Sonuç |
|---|---|---|
| Plan 27'yi yazan oturum (örneklerin yazarı) | 2026-11-28 | Örnekler, motorun kaynağından ayrı yazılmış bir türetmeyle kural metninden üretildi ve motorla karşılaştırıldı: `muntha/` beş örnek; `mudda/` dokuz örnek, 81 dönem bitişi ve üç örnekte 27 alt dönem bitişi; fark yok. Sınır: türetmeyi ve motoru aynı yazar yazdı; kuralın okunuşundaki ortak bir yanlış ikisinde de aynıdır. Bu kayıt aritmetiği ve kuralların uygulanışını doğrular, okuma seçimlerini (yukarıdaki onay tablosu) değil. |
| — | — | Bekliyor: Plan 27 Task 8 Step 4 (yürüten ajanın bağımsız yeniden türetmesi) |
````

Kılavuz (paylaşılan dosya; tablonun sonuna bir satır, Avastha paragrafından sonra bir paragraf):

````diff
diff --git a/tests/golden/modules/README.md b/tests/golden/modules/README.md
index ebf72ad..386ed15 100644
--- a/tests/golden/modules/README.md
+++ b/tests/golden/modules/README.md
@@ -57,6 +57,7 @@ doğum anı ve 365,25 günlük yıl kullanır, tarihleri el yıllarının biriki
 | `muhurta` | `eighth_min`, `rahu`, `yamaganda`, `gulika` (`segment`, `from_min`, `to_min`), `abhijit` (`from_min`, `to_min`), `abhijit_weekday_exception`, `hora_min`, `hora_lords` (24), `tithi_group`, `nakshatra_nature`, `tara` (`number`, `favourable`), `chandra` (`house`, `favourable`) | 7 |
 | `prashna` | `lagna` (`sign`, `deg_in_sign`, `kind`, `rising`, `lord`, dört `kendras`; lagna yoksa `null`), `focus` (`house`, `sign`, `lord`, `same_graha`, `links`, iki ev sayısı, `occupants`; odak evi ya da lagna yoksa `null`), `moon` (`sign`, `house`, `nakshatra`, `pada`, `tithi`, `paksha`, `aspected_by`) | 14 |
 | `avastha` (alt klasörler: `lajjitadi`, `shayanadi`) | `lajjitadi/`: `baladi`, `jagradadi`, `lajjitadi` (yedi graha; durum ve kuralı sağlayan grahalar), `absent`; `shayanadi/`: `basis` (`birth_nakshatra`, `ghatis`, `lagna_sign_number`), `grahas` (yedi graha: `state`, `nakshatra`, `navamsa`, `total`); Shayanadi hesaplanmıyorsa `expected` `null` | 7 + 6 |
+| `varsha` (alt klasörler: `muntha`, `mudda`) | `muntha/`: `sign`, `lord`, `natal_house`, `annual_house`, `lagna_sign`, `lagna_lord`, `natal_lagna_sign`, `natal_lagna_lord`; `mudda/`: `birth_nakshatra`, `remainder`, `first_lord`, `periods` (dokuz dönem: lord ve bitiş günü), `first_sub_periods` (ilk dönemin dokuz alt dönemi ya da `null`) | 5 + 9 |
 
 Shadbala örnekleri çalışma kâğıdıdır (tasarım belgesi §7 "Test"): `shadbala/WORKSHEET.md` her
 kalemin aritmetiğini adım adım yazar, JSON dosyaları aynı girdiyi ve sonuçları taşır. Tolerans
@@ -112,6 +113,18 @@ Shayanadi durumunun ve artanın sıfır olduğu durumun örneklerde bulunduğunu
 kuralları, motorun tabloları, sahibin onayını bekleyen varyant noktaları ve gözden geçirme kaydı
 `avastha/REVIEW.md`'dedir.
 
+Varshaphala örnekleri (yol haritası §4.14, Plan 27) ayrı bir test dosyasıyla yüklenir
+(`crates/jyotish-core/tests/varsha_golden.rs`) ve iki alt klasörde durur; ortak girdiyi
+kullanmazlar. `muntha/` örneklerinin girdisi iki lagna boylamı ve tamamlanan yıldır
+(`natal_lagna_lon`, `annual_lagna_lon`, `age`); `mudda/` örneklerinin girdisi Ay'ın sidereal
+boylamı, tamamlanan yıl ve yılın gün olarak uzunluğudur (`moon_lon`, `age`, `year_days`) ve
+bitişler yılın başından gün sayısıdır. Ayrı bir test dokuz lordun her birinin bir örnekte yılı
+açtığını, dokuz artan değerinin hepsinin, toplamın eksiye düştüğü durumun, 360 günlük ve dönüşten
+dönüşe yılın, alt dönemlerin, doğum yılının, tam on iki yılın ve Balık'tan Koç'a dönen sayımın
+örneklerde bulunduğunu sabitler. Dönüş anının örneği yoktur: gerçek motorla
+`app/src-tauri/tests/varsha.rs`'te denetlenir. Okuma kuralları, tablolar, sahibin onayını bekleyen
+varyant noktaları ve gözden geçirme kaydı `varsha/REVIEW.md`'dedir.
+
 Doğum anı ekleri (yol haritası §4.1–§4.3, Plan 23) ile bhava tablosu ve Sudarshana chakra
 (§4.5–§4.6, Plan 25) bu dizinde JSON örneği taşımaz: `natal_extras/` ve `bhava/` klasörlerinde
 yalnızca `REVIEW.md` vardır ve hiçbir test bu iki klasörü yüklemez. El örnekleri modüllerin birim
````

- [ ] **Step 2: Rust kapıları**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
BASE=9727c55
uptime
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
cargo test -p jyotish-desk --test varsha -- --nocapture 2>&1 | grep -E "compute_varsha median"
cargo deny check 2>&1 | tail -1
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p jyotish-core -p atlas -p store -p jyotish-desk --all-targets 2>&1 | tail -1
git diff $BASE..HEAD --stat -- deny.toml Cargo.toml Cargo.lock app/package.json app/pnpm-lock.yaml app/vite.config.ts app/playwright.config.ts app/src-tauri/capabilities app/src-tauri/tauri.conf.json app/src-tauri/Cargo.toml crates/jyotish-core/Cargo.toml crates/store crates/atlas crates/ephemeris crates/jyotish-core/src/chart.rs crates/jyotish-core/src/positions.rs crates/jyotish-core/src/settings.rs crates/jyotish-core/src/dasha.rs crates/jyotish-core/src/timeline.rs crates/jyotish-core/src/search crates/jyotish-core/src/dignity.rs crates/jyotish-core/src/error.rs crates/jyotish-core/tests/modules_golden.rs app/src-tauri/src/commands.rs app/src-tauri/src/module_commands.rs app/src-tauri/src/muhurta_commands.rs app/src-tauri/src/client_commands.rs app/src-tauri/tests/modules.rs
git diff $BASE..HEAD --stat -- crates/jyotish-core/src/lib.rs app/src-tauri/src/lib.rs | tail -1
git diff $BASE..HEAD -U0 -- crates/jyotish-core/src/lib.rs | grep -c '^+pub use'
ls crates/jyotish-core/proptest-regressions 2>/dev/null
```
Expected: fmt ve clippy çıktısız; testler: `jyotish-core` lib `487 passed`, `avastha_golden` `5 passed`, golden `8 passed`, `modules_golden` `17 passed`, `varsha_golden` `3 passed`; `atlas` `build` 6, `moment_fixed_lmt` 11, `moment_iana` 13, `search` 8, `shipped` 3; `store` lib `122 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `58 passed`, `avastha` 1, `bhava` 1, `bindings` 1, `compute_chart` 1, `modules` 1, `natal_extras` 1, `open_client` 1, `ui_fixture` 2, `varsha` `1 passed`; sayaç `0` (düşen bir bütçe iddiasıysa `uptime` çıktısına bakılır ve Global Constraints'in "Bütçe" maddesi uygulanır); ölçüm satırı bütçenin içinde (D13); deny `advisories ok, bans ok, licenses ok, sources ok`; MSRV `Finished` (1.93 araç zinciri kurulu değilse bu satır atlanır ve notlara yazılır); ilk `git diff --stat` çıktısız (bağımlılıklar, yetenek dosyası, veritabanı, var olan hesap modülleri, ortak komut ve test dosyaları değişmedi); iki `lib.rs` birlikte `2 files changed, 4 insertions(+)`; `pub use` sayacı `0` (kökten dışa aktarım yok); `ls` çıktısız. `BASE` Task 1 Step 1'de başka bir commit okunduysa o yazılır ve taban sayaçları o ölçümdendir.

- [ ] **Step 3: Arayüz kapıları**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
node --test "scripts/release/*.test.mjs" 2>&1 | grep -E "ℹ (pass|fail)"
git status --short
```
Expected: `Test Files  99 passed (99)`, `Tests  998 passed (998)`; `✓ built`; betik testleri `pass 14`, `fail 0`; `git status --short` yalnızca Step 1'in iki dosyasını gösterir. **[kontrolör]** E2e ve görsel testler Task 7 Step 5'teki komutlarla koşulur: e2e `35 passed`, görsel `17 passed`.

- [ ] **Step 4: Bağımsız yeniden türetme**

Yol haritası §8: "el örneğini yazanın kuralı yanlış anlaması motoru da yanlış doğrular; ayrı gözden geçirmede yeniden türetme zorunludur". Yürüten ajan örnekleri yazan oturum değildir ve bu denetimi yapabilir. Kural: **yalnızca** `tests/golden/modules/varsha/REVIEW.md`'nin "Okuma kuralları" ve "Tablolar" bölümleri ile her JSON'un `input` alanı okunur; `expected` ve `derivation` karşılaştırmadan önce okunmaz, Rust kaynağına bakılmaz, motor çalıştırılmaz.

1. Depo **dışında** (ör. `/tmp` altında; `git add` edilmez) atılacak bir betik yazılır: tabloları REVIEW'den kendi yazar, on dört dosyanın `input`'unu okur ve `muntha/` için munthanın burcunu, lordunu, iki evini ve iki lagnanın burcu ile lordunu; `mudda/` için doğum nakshatrasını, artanı, ilk lordu, dokuz dönemin lordunu ve bitiş gününü, `first_sub_periods` `null` olmayan dosyalarda ilk dönemin dokuz alt döneminin lordunu ve bitiş gününü hesaplar.
2. Sonuç `expected` ile karşılaştırılır (gün değerlerinde tolerans 1e-6): `muntha/` 5 dosya × 8 alan; `mudda/` 9 dosya × 9 dönem = 81 bitiş ve 3 dosya × 9 alt dönem = 27 bitiş.
3. Hepsi uyuşuyorsa REVIEW.md'nin "Gözden geçirme kaydı" tablosundaki son satır (`| — | — | Bekliyor: Plan 27 Task 8 Step 4 …`) şu satırla değiştirilir (tarih koşunun günüdür):

   ````markdown
   | Plan 27 Task 8'i yürüten ajan (örnekleri yazan oturum dışında) | <tarih> | 14/14 uyuştu: beş muntha örneği; dokuz mudda örneğinde 81 dönem ve 27 alt dönem bitişi, fark yok. Hesap yalnızca `input`'tan ve bu dosyanın kural ve tablolarından başlar, motoru çalıştırmaz. Sınır: yürüten ajan aynı planın kodunu da yazdı; bu kayıt kuralların uygulanışını ve aritmetiği doğrular, okuma seçimlerini (onay tablosu) değil. |
   ````

4. Bir fark varsa **hiçbir beklenen değer ve hiçbir kural değiştirilmez**: fark (dosya, alan, iki değer ve kuralın hangi cümlesinden çıktığı) ilerleme notlarına yazılır, tablodaki satır `Fark bulundu: <kısa özet>; sahibine devredildi` diye doldurulur ve görev kapıların sonucuyla birlikte sahibine bildirilir. REVIEW'deki kural metni belirsiz bulunduysa bu da bir bulgudur.

```bash
grep -c "Bekliyor: Plan 27 Task 8 Step 4" tests/golden/modules/varsha/REVIEW.md
grep -inE 'k(a|ā|aa)la|\bcala\b' tests/golden/modules/varsha/REVIEW.md tests/golden/modules/README.md
git status --short
```
Expected: sayaç `0` (satır dolduruldu); tarama çıktısız; `git status --short` yalnızca iki belge dosyasını gösterir (betik depoda değildir).

- [ ] **Step 5: Commit**

```bash
git add tests/golden/modules/varsha/REVIEW.md tests/golden/modules/README.md
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "docs(golden): varsha reading rules, variant points awaiting approval and the review record" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: iki dosya; tarama çıktısız; `git status --short` boş.

- [ ] **Step 6: Kural ve gizlilik taramaları (tabandan bu yana)**

```bash
BASE=9727c55
git diff $BASE..HEAD -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git log $BASE..HEAD --format=%B | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff $BASE..HEAD --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
git diff $BASE..HEAD --name-only | grep -vE '^(app|crates|tests)/'
echo "trailers: $(git log $BASE..HEAD --format=%B | grep -c '^Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>$') commits: $(git rev-list --count $BASE..HEAD) other: $(git log $BASE..HEAD --format=%B | grep '^Co-Authored-By: ' | grep -vc 'Claude Opus 5.5 <noreply@anthropic.com>$')"
git diff $BASE..HEAD -U0 -- app/ui | grep '^+' | grep -E 'console\.(log|info|warn|error|debug)'
git diff $BASE..HEAD -U0 -- crates app/src-tauri/src | grep '^+' | grep -E 'println!|eprintln!|dbg!|log::|tracing::'
git diff $BASE..HEAD -U0 -- app/ui | grep '^+' | grep -E '\b(alert|confirm|prompt)\(|window\.(print|open)\('
git diff $BASE..HEAD -U0 -- 'app/ui/**/*.css' | grep '^+' | grep -E '#[0-9a-fA-F]{3,8}\b|rgb\('
grep -rnE '#\[serde\(default' crates/jyotish-core/src/varsha app/src-tauri/src/varsha_commands.rs
grep -rnE 'XMLHttpRequest|WebSocket|\bfetch\(' app/ui/panels/VarshaPanel.tsx app/ui/modules/useVarsha.ts app/ui/varsha app/ui/dasha/MuddaDashaTree.tsx app/ui/dasha/muddaSystem.tsx
grep -rnE 'localStorage|sessionStorage|new Date\(|Date\.now' app/ui/panels/VarshaPanel.tsx app/ui/modules/useVarsha.ts app/ui/varsha app/ui/dasha/MuddaDashaTree.tsx app/ui/dasha/muddaSystem.tsx
grep -cE 'varsha_commands::(compute_varsha|save_varsha_chart),' app/src-tauri/src/lib.rs
grep -oE 'CommandError::new\("[a-z_]+"' app/src-tauri/src/varsha_commands.rs | sort -u
grep -rn 'varsha' crates/store/src | head -1
git diff $BASE..HEAD --stat -- app/ui/chart app/ui/print app/ui/shortcuts.ts app/ui/App.tsx app/ui/test/render.tsx app/e2e/visual.spec.ts-snapshots app/e2e/desk.spec.ts app/e2e/visual.spec.ts app/ui/panels/panels.test.tsx app/ui/panels/panels.css app/ui/panels/labels.ts app/ui/dasha/system.tsx app/ui/dasha/DashaStripPanel.tsx app/ui/dasha/dasha.css app/ui/i18n/i18n.test.ts
git ls-files | grep -c '^\.superpowers/'
git ls-files | grep -c 'playwright\.p27\.config'
git tag --points-at HEAD
```
Expected (`BASE` Task 1 Step 1'de okunan commit'tir; `9727c55` değilse o yazılır, yoksa aradaki başka commit'lerin dosyaları da taranır): `trailers:` satırında ilk iki sayı eşit (sekiz ve sekiz; inceleme düzeltmesi commit'i ya da kontrolörün görsel temel commit'i varsa ikisi birlikte büyür) ve `other: 0` (başka bir imza satırı yok); `grep -ciE` satırı `0`; komut sayacı `2`; `CommandError::new` satırı tek bir kod yazar: `CommandError::new("invalid_input"` (yeni hata kodu yok); `.superpowers` ve geçici yapılandırma sayaçları `0`. Bunların dışındaki her komut çıktısızdır (çıktısız `grep` 1 ile çıkar; beklenen budur): yasaklı ad yok (üretilen fikstürde de), `app/`, `crates/` ve `tests/` dışında dosya yok, arayüzde `console` ve Rust kaynağında log çağrısı yok (girdi, yer ve rapor loglanmaz: C1; `tests/varsha.rs`'teki ölçüm satırı test dosyasıdır ve bu taramanın dışındadır), yasak çağrı yok, yeni stilde hex ya da `rgb(` yok (C9), IPC tiplerinde `serde(default)` yok, panelde ağ, tarayıcı deposu ve saat çağrısı yok (C4), veritabanı katmanında varsha yok (Y6), harita çizimi, baskı, kısayollar, uygulama kökü, test sağlayıcıları, var olan görsel temeller ve bilerek dokunulmayan ortak dosyalar değişmedi, etiket (git tag) yok.

- [ ] **Step 7: Elle denetim kapısı**

Yürüten ajan gerçek uygulamayı **çalıştırmaz** (Global Constraints). Her madde ilerleme notlarına "geçti", "devredildi" ya da bulgu olarak yazılır; bulgu varsa düzeltme commit'i ve bu görev baştan. Her maddenin başında, otomatik bir karşılığı olup olmadığı yazılıdır. **[otomatik]** maddelerin karşılığı testlerde vardır ve yürüten ajan bunları testin adını göstererek "geçti" sayar; **[kontrolör]** maddeler uçtan uca ve görsel testlere dayanır ve kontrolöre bırakılır; **[insan]** maddeler yalnızca basılı kaynağa ya da gerçek uygulamaya bakan bir insan tarafından doğrulanabilir ve sahibine devredilir (bu devir sonraki planı engellemez: Y3; hiçbir şey yayımlanmaz).

- [ ] **[otomatik]** Dönüş anı: Güneş doğumdaki sidereal boylamından bir yay saniyesinden az ayrışır ve geride durmaz; yaş 0 doğum anıdır; ardışık yıllar boşluksuzdur; tropikal-rasi kipinde de dönüş sidereal boylamdandır. (`solar_return.rs › the_return_brings_…`, `› age_zero_is_the_birth_moment_…`; `tests/varsha.rs`: beş golden an × beş yaş, gerçek motorla)
- [ ] **[otomatik]** İkinci yazım: olay aramasının `TransitToNatal` anı ile bu planın dönüş anı beş golden anda bir saniye içinde buluşur. (`tests/varsha.rs`)
- [ ] **[otomatik]** Yaşın, anın ve boylamın denetimi; ortalama yıldan sapan Güneş'in payın içinde bulunması, payın dışına düşenin hata olması; süren yılın dönüşleri izlemesi. (`solar_return.rs › age_moment_and_longitude_are_validated`, `› a_sun_off_the_mean_year_…`, `› completed_years_follow_the_returns`)
- [ ] **[otomatik]** Mudda dasa: dokuz pay 360 eder ve Vimshottari yılının üç katıdır; ilk lord kuralının on el örneği; ikinci yazım (doğum nakshatrasının lordu her yıl bir adım ilerler) 27 × 121 girdide; dönemler yılı boşluksuz doldurur; alt dönemler üst dönemin lordundan başlar. (`mudda.rs` yedi test; `varsha_golden › mudda_hand_fixtures`, `› fixtures_cover_the_required_cases`: dokuz lord, dokuz artan)
- [ ] **[otomatik]** Muntha: lagna burcundan yılda bir burç; on ikide bir başa döner; ikinci yazım 12 × 121 girdide; lordlar ve iki ev. (`muntha.rs` üç test; `varsha_golden › muntha_hand_fixtures`)
- [ ] **[otomatik]** Rapor: yıllık harita dönüş anının ve istenen yerin haritasıdır, doğum haritasının ayarlarıyla; yer dönüş anını ve mudda dasayı değiştirmez; doğum saati bilinmiyorsa yıllık harita ve `facts` yoktur; sayı prashnasında muntha sayının lagnasından sayılır; düşük hassasiyet uyarısı taşınır. (`collect.rs` altı test; `tests/varsha.rs`)
- [ ] **[otomatik]** Komutlar: yaş verilmeyince süren yıl Rust'ta bulunur; ofset yerin dilimindendir; hatalar var olan kodlara eşlenir; ortanca süre ≤ 20 ms. Kayıt: `Event` türü, `Varshaphala <tarih>` etiketi, yerin duvar saati, kaynak haritanın ayarları, harita açılmaz; bulunulan yer seçilmemişken ve doğum saati bilinmeyen haritada reddedilir. (`tests/varsha.rs`: `assert!`)
- [ ] **[otomatik]** Panel: istek açık haritanın girdisiyle, yaşsız ve doğum yeriyle bir kez yapılır; dokuz olgu satırı; yıllık harita dokuz graha ve muntha işaretiyle çizilir, tablo karşılığı on satırdır; biçim ayarı düzenle kaydedilir; okuma notu ve yöntem kimliği yazılıdır. (`VarshaPanel.test.tsx` ilk iki test)
- [ ] **[otomatik]** Yıl seçici: önceki ve sonraki yıl, yaş alanı (0–120 dışı yok sayılır), "bu yıl"; uçlarda düğmeler `aria-disabled`; seçim açık haritaya aittir ve iki panel arasında ortaktır. (`VarshaPanel.test.tsx › stepping the year …`, `› the age field …`, `› the selected year …`; `MuddaDashaTree.test.tsx › the year is shared …`)
- [ ] **[otomatik]** Yer: seçim panel ayarıdır; bulunulan yer yokken açıklama çıkar ve istek yapılmaz; varken istek ve kayıt o yerle yapılır. (`VarshaPanel.test.tsx › the current place …`, `› with a current place …`)
- [ ] **[otomatik]** Kaydetme: bir kez kaydeder, etiketi `role="status"` ile bildirir, listeleri yeniler, haritayı açmaz; yıl değişince bildirim taşınmaz; hata `role="alert"` ile bildirilir ve yeniden denenebilir. (`VarshaPanel.test.tsx › saving stores …`, `› a failed save …`)
- [ ] **[otomatik]** Panel durumları: doğum saati bilinmeyen harita (açıklama; harita, muntha ve kaydetme yok); düşük hassasiyet uyarısı; açık harita yok; hata ve yeniden deneme; klavyeyle yeniden denemede odak panelde durur; yükleme evreleri; ortak vurgu ve kaydırma kapları. (`VarshaPanel.test.tsx` yedi test)
- [ ] **[otomatik]** Mudda görünümü: dasa ağacının sistem listesinde üçüncü seçenektir ve seçim düzenle kaydedilir; seçilmeden rapor istenmez; dokuz dönem, ilk lord satırı, "doğrulanmamış tablo" notu ve yöntem kimliği; alt dönemler istek yapmadan açılır, tek açık yol, klavye; doğum saati bilinmeyen haritada yaklaşık notu; hata ve yükleme. Dasa şeridi mudda dasayı tanımaz. (`MuddaDashaTree.test.tsx` sekiz test; var olan dasa testleri değişmeden geçer)
- [ ] **[otomatik]** Klasik terimler üç dilde aynıdır; Fransızca metinlerde düz kesme işareti yoktur; üç dilin anahtar kümeleri ve yer tutucuları eşittir; arayüzün `MAX_VARSHA_AGE` değeri Rust'ınkidir. (`VarshaPanel.test.tsx` son test ve yaş alanı testi; `i18n.test.ts`)
- [ ] **[otomatik]** Bağımlılıklar, veritabanı ve var olan hesap modülleri değişmedi; kökten dışa aktarım yok; yeni hata kodu yok. (Step 2 ve Step 6'nın `git diff --stat` ve sayaç satırları)
- [ ] **[otomatik]** El örnekleri motorun kaynağına bakılmadan yeniden türetildi ve uyuştu. (Step 4; REVIEW.md "Gözden geçirme kaydı")
- [ ] **[kontrolör]** Uçtan uca: panel paletten açılır, odağı alır ve görünür alana gelir; olgular, çizim ve tablo; ortak vurgu doğum haritasına ulaşır; sonraki yıl; kaydetme komutu yalnızca kimlik, yaş ve yer seçimi taşır; dasa ağacında mudda dasa aynı yılı gösterir ve "bu yıl" iki paneli birlikte döndürür; sistem seçimi düzenle kaydedilir; panel aynı komutla çıkarılır ve odak gövdeye düşmez; doğum saati bilinmeyen harita. (`varsha.desk.spec.ts` iki akış; e2e `35 passed`)
- [ ] **[kontrolör]** Görsel: yıllık haritanın iki temeli üretildi, gözle denetlendi ve commit edildi; var olan on beş temel değişmedi. (`varsha.visual.spec.ts`; görsel `17 passed`)
- [ ] **[insan]** **Basılı kaynak denetimi (asıl kapı).** `tests/golden/modules/varsha/REVIEW.md`'nin onay tablosundaki on iki nokta basılı bir Tajika kaynağıyla karşılaştırılır: dönüşün sidereal mi tropikal mi alındığı; yıllık haritanın yeri; munthanın sayımı (tamamlanan yıl), çerçevesi ve derecesi; mudda dasanın ilk lord kuralı (terimler ve artanın lorda eşlenmesi), sırası ve günleri, ilk dönemin kesintisi, yılın uzunluğu ve alt dönemler. Sonuç tablonun altındaki "Denetleyen" satırına yazılır. Onaydan sonra `varsha.mudda.unverified` notu ayrı bir commit'le çıkarılır.
- [ ] **[insan]** Gerçek uygulamada, **yalnızca deneme danışanlarıyla**: palette "Varshaphala panelini aç/kapat" paneli açar; süren yılın yaşı danışanın doğum tarihiyle tutarlıdır (son doğum günü dönüşü geçtiyse yaş, geçmediyse bir eksiği); dönüş anı doğum gününün bir gün çevresindedir; yıllık haritada Güneş doğum haritasındaki burcunda ve derecesindedir (graha tablosuyla karşılaştırılır).
- [ ] **[insan]** Muntha elle sağlanır: doğum lagnasının burcundan tamamlanan yıl kadar burç sayılır; "Sonraki yıl" munthayı bir burç ilerletir; çizimdeki "Mu" işareti o burcun hücresindedir.
- [ ] **[insan]** Yer: "Bulunulan yer" seçilince (yer seçiliyken) dönüş anı aynı anın o yerdeki duvar saatiyle yazılır, yıl lagnası değişir, muntha burcu değişmez; bulunulan yer yokken açıklama ve "Bulunulan yeri seç" düğmesi görünür ve pencere açılır. Seçim uygulama yeniden başlatılınca yerindedir.
- [ ] **[insan]** "Olay haritası olarak kaydet" bir harita ekler: bildirimde etiket görünür, kenar çubuğunda danışanın haritaları arasında `Varshaphala <tarih>` çıkar, açık harita değişmez. Kaydedilen harita açılınca Güneş doğum haritasındaki derecesindedir, lagnası panelde görülen yıl lagnasıdır ve ayar rozetleri kaynak haritanınkiyle aynıdır. Deneme kaydı sonra silinir.
- [ ] **[insan]** Dasa ağacında "Sistem: Mudda dasa" dokuz dönemi listeler; ilk dönem dönüş anında başlar, son dönem yıl sonunda biter; bir dönemin günü tablodaki payla tutarlıdır (ör. Güneş dönemi yaklaşık 18 gün); bir dönem açılınca dokuz alt dönem görünür. Varshaphala panelinde yıl değişince ağaç da değişir.
- [ ] **[insan]** Doğum saati bilinmeyen bir deneme haritasında panel açıklamayı gösterir, harita ve muntha çizilmez, kaydetme düğmesi yoktur; mudda görünümü yaklaşık notunu taşır.
- [ ] **[insan]** Panel açık ve koyu temada, Türkçe, İngilizce ve Fransızca dilde okunur; dar panelde tablolar yatay kayar; yıl seçici ve yer seçimi klavyeyle kullanılır; kapalı düğmeler renk olmadan da ayırt edilir.
- [ ] **[insan]** VoiceOver: tablolar başlıklarıyla duyurulur; yıl alanı "Yaş (tamamlanan yıl)" diye okunur; kayıt bildirimi kendiliğinden okunur; çizimdeki muntha işareti yerine "Muntha … burcunda" cümlesi okunur; mudda ağacı ağaç olarak duyurulur ve ok tuşlarıyla gezilir.
- [ ] **[insan]** Bütün denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, doğum anı, yer, koordinat ya da rapor içeriği görünmez.

Kapı geçtiyse ilerleme notlarına `Plan 27 Task 8 elle denetim: otomatik maddeler geçti, kontrolör maddeleri kontrolöre, insan maddeleri sahibine devredildi (<tarih>)` yazılır. Bu adım commit üretmez.

---

## Başarı ölçütleri

- Seçilen yılın dönüş anı, yıllık haritası, munthası ve mudda dasası tek komutla, saf parçalardan ve doğum haritasının kendi ayarıyla hesaplanır; sonuç saklanmaz ve `Chart`'a eklenmez.
- Dönüşte Güneş doğumdaki sidereal boylamından bir yay saniyesinden az ayrışır (gerçek motorla, beş golden anda) ve uygulamanın olay araması aynı anı bir saniye içinde bulur.
- Mudda dasanın ilk lord kuralı iki yazımla her nakshatra ve her yaşta aynı lordu verir; dokuz dönem yılı boşluksuz doldurur; artan ve dönem sınırları raporda durur.
- Doğum saati bilinmiyorsa yıllık harita ve muntha Rust'ta boş döner; panel ve mudda görünümü bunu ayrı ayrı söyler.
- Yıllık harita var olan olay haritası yoluyla, kaynak haritanın ayarlarıyla ve veritabanı şeması değişmeden kaydedilir; arayüzden an ya da yer alınmaz.
- Okuma seçimleri `varsha.sidereal_return_v1` kimliğine bağlıdır ve REVIEW.md'de on iki varyant noktasıyla yazılıdır; onaya dek mudda görünümü "doğrulanmamış tablo" notunu taşır.
- Paralel planlarla paylaşılan her dosyadaki değişiklik ekleme türündedir ve "Paylaşılan dosyalar" tablosundadır; `jyotish_core` kökünden hiçbir ad dışa aktarılmaz; ortak komut, test ve stil dosyalarına dokunulmamıştır.
- Sayaçlar: `jyotish-core` lib 487, `varsha_golden` 3, `jyotish-desk` `varsha` 1, Vitest 99 dosya / 998 test, e2e 35, görsel 17; öteki bütün sayaçlar tabandaki gibidir.

## Bu planın kapsamadıkları

- Yıl lordu (beş aday, panchavargiya bala, hadda tablosu), Tajika bakışları ve sahamlar: Plan 28. Tajika yogaları, tri-pataki chakra ve yıllık yorum: yol haritası §4.14 ve §7.
- İkinci okumalar ve bunların ayarı: tropikal dönüş, süren yıldan sayılan muntha, munthanın derecesi ve aylık ilerleyişi, mudda dasada ilk dönemin kesintisi, 360 takvim günlük yıl (Y5: tek seçenekli okuma ayar olmaz). Sahibi REVIEW'deki bir noktayı değiştirirse kural ayrı bir commit'le değişir ve yöntem kimliği `varsha.sidereal_return_v2` olur.
- Dasa şeridinde mudda görünümü ve "transit anındaki mudda dönemi" (ayrı bir "andaki yol" komutu ister); mudda ağacında süren dönemin işaretlenmesi; üçüncü seviye mudda dönemleri.
- Aylık ve günlük dönüş haritaları (masa-phala, dina-phala); yıllık haritanın bölüm haritaları, ashtakavargası ve öteki modül panelleri (kaydedilen olay haritası açılınca hepsi kullanılır).
- Yıllık harita için atlastan üçüncü bir özel yer; kaydedilen haritanın otomatik açılması; aynı yılın ikinci kez kaydedilmesinin engellenmesi (panel yalnızca aynı oturumdaki çift tıklamayı engeller).
- Panelin hazır düzenlere eklenmesi; baskı düzeni (K10); doğum haritasının üstüne yıllık halka.
- Yorum metni, AI paneli; Windows ve Android.

## Spec'e geri bildirim (yol haritası farkları ve boşlukları)

Kararlar bölümünde işaretlenen maddeler ve yol haritasının açık bıraktığı yerler; yol haritası bir sonraki düzenlemede bunlara göre güncellenmelidir.

1. **(D4) Dönüş anının bulunuşu.** §3 "dönüş anı olay aramasında bulunuyor" der ve §5 Plan 27'yi "Plan 22 (dönüş anı)"na bağlar. Bu plan olay aramasının yürüyücüsünü çağırmaz: tek bir anı kendi dar aramasıyla bulur (tahmin + ikiye bölme, 0,01 saniye) ve olay aramasını yalnızca testte ikinci yazım olarak kullanır. §4.14'e yöntem ve çözünürlük yazılmalıdır.
2. **(D17) Mudda görünümünün yeri.** §5 (6) "var olan dasa panellerinde" der; görünüm yalnızca dasa ağacındadır. Dasa şeridi transit anının dönem yolunu Rust'tan alır; mudda için ayrı bir komut gerekir ve sonraki işe bırakılmıştır.
3. **(D23) Görev taslağı.** §5 Plan 27'nin sırası: mudda dasa çekirdeği toplayıcıdan önce gelir; panelin metinleri, "saat bilinmiyor" durumu ve kaydetme düğmesi panelle aynı görevdedir; (7) dasa ağacının mudda görünümü ile uçtan uca ve görsel testlerdir.
4. **(D2) Yaşın tanımı.** §4.14 "tamamlanan yıl" der ama aralığı ve "N. yıl" yazımını tanımlamaz: tamamlanan yıl 0–120, gösterilen yıl numarası bir fazlası.
5. **(D7) Yer seçimi.** §4.14 iki seçeneği "karar ve `method`" diye bırakır; ikisi de uygulandı (panel ayarı, varsayılan doğum yeri). Y5'in "ikinci okuma ancak uygulanınca ayar olur" kuralının burada uygulandığı yazılmalıdır.
6. **(D8) Yıl başlığı olguları.** §5 (3) "yıl başlığı olguları"nı saymaz: yıllık lagnanın ve doğum lagnasının burcu ve lordu, munthanın lordu ve iki evi. Bunların Plan 28'in beş adayından üçü olduğu yazılmalıdır.
7. **(D9) Mudda dasanın okuması.** §4.14 iki varyant noktasını sayar ama seçimi yazmaz: ilk lord kuralı (doğum nakshatrası + tamamlanan yıl − 2; artan 1 Güneş … 0 Venüs), ilk dönemin tam oluşu, yılın dönüşten dönüşe sayılması, iki seviye ve raporun artanı taşıması.
8. **(D10) Rapor biçimi.** "Saat bilinmiyorsa `None`" yalnızca muntha için yazılıdır; rapor `None` olmaz, `time_known: false` ile döner; yıllık harita ve lagna olguları boştur, dönüş anı ve mudda dasa yaklaşık olarak döner. Sayı prashnası durumu eklenmelidir.
9. **(D14) Kaydetme.** "Var olan akış"ın ne olduğu yazılmalıdır: `Store::add_chart` ve `ChartKind::Event`; etiket, ayar kopyası, haritanın açılmaması ve arayüzden an ya da yer alınmaması.
10. **(D13) Bütçe.** `compute_varsha` için hedef yazılmamıştır: ortanca ≤ 20 ms (ölçüm 2,0 ms).
11. **(D18) Anların ofseti.** Uzak yıllardaki anların hangi ofsetle yazılacağı tanımsızdır: yerin yıl başındaki ofseti, yanında yazılarak. Aynı soru Plan 26 (Sankranti listesi) ve Plan 29 (takvim) için de geçerlidir ve yol haritasında tek bir yerde tanımlanmalıdır.
12. **Y4'ün notu.** Bu planda not yalnızca mudda görünümündedir; "okuma seçimi" (dönüşün çerçevesi, yer, muntha sayımı) ile "bellekten tablo" (mudda payları ve kuralı) ayrımı Y4'e yazılmalıdır.

## Self-Review

1. **Kapsam:** yol haritası §4.14'ün ilk yarısının ve §5 Plan 27'nin her maddesi "Kapsam" tablosunda bir göreve bağlıdır. Taslaktan bilerek ayrılan yerler Kararlar'da işaretlidir (D4, D17, D23) ve geri bildirim listesindedir. Sekiz görev vardır; son görev kapılar, REVIEW.md ve **[otomatik]**/**[kontrolör]**/**[insan]** maddeleridir.
2. **Yer tutucu taraması:** planda "TBD", "TODO", "sonra doldurulur", "uygun hata işleme ekle" ya da kodu olmayan kod adımı yoktur; her kod adımı tam dosya ya da uygulanabilir `diff` taşır. `<tarih>` ve `<kısa özet>` yürütme anında yazılan değerlerdir (Task 8 Step 4 ve Step 7); `BASE` Task 1 Step 1'de okunur. İki görsel temel PNG'si bu belgede yoktur: kontrolör üretir (D22).
3. **Tip ve ad tutarlılığı:** `solar_return_jd`, `varsha_span`, `completed_years_at` Task 1'de; `MuddaDasha`, `mudda_dasha`, `mudda_first_lord`, `mudda_remainder`, `MUDDA_ORDER` Task 2'de; `Muntha`, `YearFacts`, `year_facts`, `VarshaYear`, `VarshaReport`, `compute_varsha` Task 3'te; `VarshaView`, `compute_varsha_blocking`, `save_varsha_chart_blocking` Task 5'te tanımlanır ve sonraki görevlerde aynı adlarla kullanılır. Komut adları Rust'ta `compute_varsha` ve `save_varsha_chart`, arayüzde `computeVarsha` ve `saveVarshaChart`'tır; panel türü `varsha`, i18n bloğu `varsha`, fikstür alanı `varsha`, dasa ağacının üçüncü sistemi `mudda`'dır. Arayüzün `MAX_VARSHA_AGE` değeri ve mudda sırası fikstürdeki Rust değerleriyle testte karşılaştırılır.
4. **Dersler:** hiçbir ad kökten dışa aktarılmaz (geçen birleştirmedeki ad çatışması); tek okuma `method` kimliğine bağlıdır ve varyantlar REVIEW'dedir (Y4, Y5); el örneği motorla uyuşmazsa kural yeniden okunur (bu plan yazılırken bir kez; Global Constraints); arayüz saat okumaz ve bölme yapmaz, süren yıl ve artan Rust'tan gelir (C4); lagnaya bağlı çıktı Rust'ta boşaltılır (K2); yeniden denemede odak gövdeye düşmez (panel testi); bütçe `assert!` ile doğrulanır ve yük altında düşen bütçe iddiası kodla değil yeniden koşuyla çözülür; paylaşılan dosyalara yalnızca ekleme yapılır ve liste tamdır; üretilen dosyalar yalnızca iki komutla güncellenir (C10); gerçek uygulama çalıştırılmaz; e2e özel portta ve yalnızca kontrolör tarafından koşulur.
5. **Koşulmuş kod:** bu plandaki her kod bloğu, `9727c55`'in bir kopyasında sekiz ayrı commit olarak uygulanmış ağaçtan alınmıştır; sonra bu belgenin kendisi ikinci bir dalda baştan uygulanmış (yeni dosyalar yazılarak, `diff` blokları `git apply` ile, "başına eklenir" blokları eklenerek, üretilen iki dosya iki komutla üretilerek) ve her görevin sonunda ağacın ilk uygulamadaki commit'le bayt bayt aynı olduğu doğrulanmıştır. Son ağaçta `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, Vitest (99 / 998), `pnpm build`, e2e (35; özel portta), görsel (17; iki yeni temel üretilip ikinci koşuda karşılaştırılarak), `cargo deny check`, 1.93 ile `cargo check` ve yayın betiği testleri (14) koşulmuş ve geçmiştir; `cargo test --workspace`'in bir koşusunda, makine başka çalışma ağaçlarının derlemesiyle yüklüyken (yük ortalaması 100'ün üstü) var olan `tests/modules.rs`'in bir bütçe iddiası düşmüş, test tek başına yeniden koşulunca geçmiştir (bu planın kodu o testte yoktur). "Testlerin düştüğünü gör" adımlarının beklentileri ikinci uygulamada koşularak alınmıştır. **[insan]** maddeleri, Task 8 Step 4'ün bağımsız yeniden türetmesi ve yeni bir çalışma ağacındaki bağımlılık kurulumu koşulmamıştır. Mudda dasanın kuralı ve payları, dönüşün çerçevesi ve munthanın sayımı basılı kaynakla karşılaştırılmamıştır (Y4).
6. **Yasaklı ad:** bu belge ve bütün kod blokları `grep -inE 'k(a|ā|aa)la|\bcala\b'` ile taranmıştır; çıktı boştur.
