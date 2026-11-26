# Plan 17 — Alt proje 3.4: Shadbala ve Bhava bala (altı güç, zaman lordları, on iki bhava; iki panel)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Yedi grahanın Shadbala'sını (sthana, dig, zaman gücü, cheshta, naisargika, drik; toplam, gerekli en az değere oran, sıra, graha savaşı işareti) ve on iki bhavanın Bhava bala'sını vermek: `jyotish-core`'da saf `shadbala` modülü, `compute_shadbala` toplayıcısı ve komutu, `shadbala` ve `bhava_bala` panelleri (ayrıntı ve birim ayarlarıyla).

**Architecture:**
- **Saf çekirdek, ince toplayıcı (spec K3).** `jyotish-core`'a `shadbala` modülü eklenir: `sthana`, `temporal`, `cheshta`, `bhava` alt modülleri ve rapor (`mod.rs`). Hesap yalnızca `Positions` ve `ShadbalaAstro` görür; el örnekleri ephemeris olmadan kurulur. `collect` alt modülü temel haritayı hesaplar, gün doğumlarını, MC'yi, tropikal ve ortalama boylamları ve iki güneş girişinin varasını toplar, saf fonksiyona verir.
- **Tek komut, Plan 14'ün kalıbıyla (spec K1, K2).** `compute_shadbala(input: ChartInput) -> Option<ShadbalaReport>`; motor iş parçacığı, `blocking` havuzu, `CommandError`. `Chart` ve `open_client` değişmez; sonuç saklanmaz. Doğum saati bilinmiyorsa ya da o gün Güneş doğmuyor ya da batmıyorsa rapor Rust'ta `None` döner.
- **Arayüz hesap yapmaz.** Bütün kalemler, toplamlar, oranlar ve sıralar Rust'tan gelir. İki panel aynı raporu okur (`useShadbala`); virupa → rupa gösterimi yalnızca birim çevirisidir. Paneller paletten eklenir (K12).
- **Kalıcı biçim:** yalnızca panel ayarları (`detail`, `unit`). Şema sürümleri, SQLite şeması, `CalcSettings` ve `ViewState` değişmez.
- **Testler:** kalem düzeyinde el örnekleri (her dosyanın `mod tests`'i), çalışma kâğıdı (`tests/golden/modules/shadbala/`: `WORKSHEET.md` ve dört JSON, kaynak yalnızca `hand`), elle yazılmış tablolar (bağımsız ikinci yazım), proptest değişmezleri, sahte ephemeris'le toplayıcı testleri, gerçek motorla entegrasyon testi (v1 haritasıyla çapraz denetim, astronomik olgular, bütçe), Vitest (iki panel), Playwright e2e.

**Tech Stack:** Rust 1.93+ (stable; yerelde ve CI'da 1.100), Tauri 2.12, tauri-specta 2.0.0-rc.29, serde, proptest, React 19, TypeScript 7, Vitest 5, Playwright 1.65.1. **Yeni Rust ya da npm bağımlılığı yok.**

**Spec:** `docs/superpowers/specs/2026-11-26-alt-proje-3-ileri-moduller-design.md` (Plan 16'nın geri bildirimi işlenmiş hâli): §2 (K1–K12), §4 (sphuta drishti, `Positions`), §7 (Alt proje 3.4), §13 (ortak test stratejisi), §14 (gizlilik), §15 (bütçeler), §17 (riskler), Ek A (adlandırma). UI spec: `2026-11-24-ui-design.md` §3.3, §3.4, §5, §6, §8.

**Ön koşul:** Taban, Plan 16'nın son inceleme düzeltmelerini taşıyan `main` ucudur: commit `4d435e7` (Plan 16 son inceleme `1832760` ve clippy 1.100 düzeltmesi). `main` bu commit'ten ilerlediyse Task 1 adım 1 tabanları yeniden ölçer ve sayaç zinciri o ölçümden yürütülür (farklar aynıdır). Plan 16'nın elle denetim kapısının yalnızca insana ait maddeleri sahibine devredilmiştir; bu planı engellemez.

## Kararlar

Sahibi kararları devretti; muhafazakâr seçenek seçildi ve her karar bir satır gerekçeyle yazıldı. "Spec boşluğu" ya da "Spec kusuru" diye işaretlenenler spec'te düzeltilmesi gereken yerlerdir ve belgenin sonundaki "Spec'e geri bildirim" bölümünde aynı numarayla toplanmıştır. D8, D9, D17, D18 ve sayaç zinciri bu plan yazılırken deponun bir kopyasında ölçülmüştür (macOS 27.2, Rust 1.100).

- **D1 — Modül `crates/jyotish-core/src/shadbala/` dizinidir: `mod.rs` (tipler, dig, naisargika, drik, graha savaşı, rapor), `sthana.rs`, `temporal.rs`, `cheshta.rs`, `bhava.rs`, `collect.rs`. `method = "shadbala.bphs_v1"` (`SHADBALA_METHOD`). Üçüncü güç kodda, metinde ve arayüzde yalnızca "zaman gücü" / `temporal` adıyla anılır (Ek A).** Gerekçe: spec §7 tek modül ve tek komut verir; kalem başına dosya, kalem düzeyindeki el örneklerini kuralın yanında tutar.
- **D2 — Saf fonksiyon `shadbala(positions, astro) -> Option<ShadbalaReport>`'tır: lagna yoksa ya da `astro` tutarsızsa (`ShadbalaAstro::is_consistent`: değerler sonlu; doğuş ≤ doğum < sonraki doğuş; doğuş < batış < sonraki doğuş) `None`. `ShadbalaAstro` alanları: `jd_ut`, `sunrise_jd_ut` (doğumdan önceki son doğuş), `sunset_jd_ut` (ondan sonraki ilk batış), `next_sunrise_jd_ut`, `mc_lon` (sidereal), `tropical_lons: [f64; 7]`, `mean: MeanLongitudes { sun, mercury, venus, mars, jupiter, saturn }`, `weekday`, `year_entry_weekday`, `month_entry_weekday` (`Vara`). Tip IPC'ye çıkmaz; bilinmeyen alanı reddeder (el örnekleri onu doğrudan okur).** Gerekçe: spec `ShadbalaAstro`'yu yalnızca sayar ("önceki ve sonraki gün doğumu ile batımı, MC, …"); hangi doğuş ve batışın gerektiği hesaptan çıkar: gündüz üçte birleri (doğuş → batış), gece üçte birleri (batış → sonraki doğuş), saat dilimleri (doğuş → sonraki doğuş). Önceki günün batışı hiçbir kalemde kullanılmaz. **Spec boşluğu 4.**
- **D3 — `compute_shadbala` şu durumlarda `None` döner: doğum saati bilinmiyor; doğum gününün doğuşu, batışı ya da sonraki doğuşu yok (kutup günü ya da gecesi). Ayar, an ve konum denetimi (`compute_chart`) her durumda yapılır. `None` uyarı taşıyamaz: arayüz nedeni kendi bildiğinden okur (doğum saati bilinmiyorsa onu, biliniyorsa gün doğumunun yokluğunu söyler; `Chart.meta.warnings` zaten `SunriseUnavailable` taşır ve harita başlığındaki rozette görünür). Dolu rapor temel haritanın yalnızca `LowPrecision` uyarısını taşır (Plan 15'in `JaiminiReport` kuralı).** Gerekçe: spec §7 "`None` ve ilgili uyarı" der; dönüş tipi `Option<ShadbalaReport>` olduğu için `None`'ın yanında uyarı alanı yoktur. Tipi değiştirmek (ör. zarf) AI paketleyicisinin sözleşmesini büyütürdü. **Spec kusuru 1.**
- **D4 — Çerçeveler (K5): burca ve burç içi dereceye dayalı kalemler rasi çerçevesindedir (uccha, saptavargaja, ojayugma, kendradi, drekkana, bhavanın burç türü); açı farkına dayalı kalemler sidereal boylamlarla hesaplanır ve çerçeveden bağımsızdır (dig, drik, bhava drishti, paksha, graha savaşı); ayana ve cheshta tropikal boylam kullanır. Grahanın rasi çerçevesindeki boylamı `rasi_lon(point) = burç başı + deg_in_sign`'dır. `Positions` lagnanın rasi çerçevesindeki derecesini taşımaz; bhava ortasının derecesi `lagna_lon + (rasi_lon(Güneş) − Güneş.sidereal_lon)` ile bulunur (sidereal kipte ve sentetik konumlarda fark tam 0'dır, tropikal-rasi kipinde ayanamsadır).** Gerekçe: spec §7 kalemlerin çerçevesini yazmaz; v1 dignity ve vargaları rasi çerçevesinde hesaplar (K5). **Spec boşluğu 5.**
- **D5 — Saptavargaja: (1) moolatrikona 45 yalnızca D1'de ve v1'in moolatrikona derece aralığında (`dignity::moolatrikona`); (2) varga burcu grahanın kendi burcuysa 30 (yedi vargada da; D2'de Güneş için Aslan, Ay için Yengeç); (3) değilse varga burcunun lorduyla bileşik ilişki: çok dost 20, dost 15, nötr 10, düşman 4, çok düşman 2. Geçici ilişki her vargada D1 burçlarından alınır (`temporal_friend(grahanın D1 burcu, lordun D1 burcu)`; aynı burç düşmandır). Yücelme ve düşüş burcu ayrıca sayılmaz (uccha bala onu ölçer).** Gerekçe: spec "moolatrikona 45 (yalnızca D1)" ve "ilişki v1'deki bileşik ilişkidir" der; moolatrikonanın burç mu derece aralığı mı olduğu ve yücelme burcundaki grahanın değeri yazılmamış. v1'in dignity sırası aynı tanımları kullanır. **Spec boşluğu 6.**
- **D6 — Gündüz–gece gücünde `m`, doğum anının doğum gününün görünür öğlesine (doğuş ile batışın ortası) saat cinsinden uzaklığıdır; 12 saatten büyükse `24 − m` alınır (ardışık görünür öğleler 24 saat arayla sayılır). Üçte birler ve saat dilimleri yarı açık aralıklardır (`[başlangıç, bitiş)`): tam batış anı gecedir, tam sınırdaki an sonraki dilimdedir. Saat dilimleri doğuştan sonraki doğuşa 24 eşit parçadır (gün 24 saat değilse dilimler de eşit uzar ya da kısalır).** Gerekçe: spec "en yakın görünür öğle" der; şafaktan önce doğanlarda en yakın öğle ertesi günün öğlesidir ve onun için ikinci bir batış gerekirdi. Ardışık görünür öğlelerin 24 saatten farkı 30 saniyenin altındadır (en çok 0,04 virupa). **Spec boşluğu 7.**
- **D7 — Yıl ve ay lordu: Güneş'in doğumdan önceki son sidereal 0° girişi ve bulunduğu sidereal burcun başına girişi `sun_entry_before` ile bulunur (ilk tahmin doğum anındaki hızdan, sonra sekiz sabit Newton adımı; her adım `Ephemeris::body`'nin boylamını ve hızını kullanır). Giriş her zodyak kipinde sidereal çerçevededir (ayarın ayanamsasıyla). Giriş anının varası doğum yerinde, v1'in vara kuralıyla bulunur (`vara_at`: o andan önceki son gün doğumunun yerel hafta günü; gün doğumu yoksa 06:00 yerel ortalama saat). Bunun için `panchang.rs`'e iki `pub(crate)` yardımcı eklenir (`vara_at`, `following_sunrise`) ve `day_sunrise` `pub(crate)` olur; `compute_panchang`'ın davranışı değişmez.** Gerekçe: spec "girişin gerçekleştiği varanın lordu" der; varanın hangi yerde ölçüldüğü ve tropikal-rasi kipinde hangi burcun kastedildiği yazılmamış. Doğum yeri, doğum gününün varasıyla aynı kuralı verir. **Spec boşluğu 8.**
- **D8 — Ortalama boylam katsayıları (tropikal, tarihin ortalama ekinoksu; `L0 + L1 × T`, `T` J2000'den Jülyen yüzyılı): Güneş 280,466457 + 36000,7698278 T (Dünya'nın güneş merkezli ortalama boylamı + 180°); Merkür 252,250906 + 149474,0722491 T; Venüs 181,979801 + 58519,2130302 T; Mars 355,433000 + 19141,6964471 T; Jüpiter 34,351519 + 3036,3027748 T; Satürn 50,077444 + 1223,5110686 T. Kaynak: J. Meeus, *Astronomical Algorithms* (2. baskı), Tablo 31.A (VSOP87 ortalama öğeleri), yalnızca sabit ve doğrusal terim. `T` UT'den hesaplanır (dinamik zamanla fark en hızlı öğede 0,003°'nin altındadır). Doğrulama gerçek motorla, 1900–2100 arasında yapılır: gerçek jeosantrik boylam − ortalama boylam farkının ortalaması Güneş'te ve Mars'ta ±0,05° içindedir; Jüpiter'de ±0,1°, Satürn'de ±0,3° sınırı uygulanır; Merkür ve Venüs'ün güneş merkezli ortalama boylamı alt kavuşum anlarında Güneş'in karşısıyla karşılaştırılır (Venüs ±0,05°, Merkür ±0,5°).** Gerekçe: spec "±0,05° içinde doğrulanır" der; ölçüm: Güneş −0,007°, Mars −0,027°, Jüpiter +0,055°, Satürn −0,212°; Venüs −0,008°, Merkür +0,26° (630 alt kavuşum). Jüpiter ve Satürn'ün karşılıklı uzun dönemli tedirginliği (~900 yıl) 200 yılda ortalanmaz; `Ephemeris` trait'i güneş merkezli konum vermediği için iç gezegenlerin ortalama boylamı doğrudan ölçülemez. Katsayılar yayımlanmış değerlerdir, ölçüme uydurulmaz. **Spec kusuru 2.**
- **D9 — Cheshta: dairesel orta nokta kısa yayın ortasıdır (`circular_midpoint`; tam karşıt iki boylamda yay −180° sayılır). Güneş'in cheshtası ayana balanın iki katı alınmamış hâli, Ay'ınki `uzanım / 3`'tür. Astronomik çapraz denetim spec'in iki koşuluna (dış gezegen karşı konumda ≥ 55, kavuşumda ≤ 5) iç gezegenleri ekler: alt kavuşumda ≥ 50, üst kavuşumda ≤ 10.** Gerekçe: ölçüm (1900–2100): Mars 57,7–60,0 ve 0,0–1,5; Jüpiter 58,8–60,0 ve 0,0–1,2; Satürn 59,5–60,0 ve 0,0–1,8; Merkür alt kavuşumda 52,3–60,0, üst kavuşumda 0,0–8,3 (yörünge dış merkezliği ortalama boylamı gerçek boylamdan 23°'ye kadar ayırır); Venüs 59,8–60,0 ve 0,0–0,6. Spec iç gezegenler için denetim yazmaz. **Spec boşluğu 9.**
- **D10 — Naisargika bala 60'ın yedide birinin katlarıdır (`60 × k / 7`; Güneş 7, Ay 6, Venüs 5, Jüpiter 4, Merkür 3, Mars 2, Satürn 1); spec'in iki ondalıklı değerleri bunların yuvarlanmışıdır.** Gerekçe: yuvarlanmış sabitler toplamda 0,005'e kadar fark üretir; çalışma kâğıdının toleransı 0,01'dir. **Spec boşluğu 10.**
- **D11 — Drik bala harfiyen uygulanır: diğer altı grahanın her biri için `±bakış / 4` (K6 sınıfına göre artı ya da eksi), ardından Merkür'ün ve Jüpiter'in bakış değeri bir kez daha, tamamıyla ve artı işaretle eklenir (Merkür kötücül sayılsa da). Bhava drishti aynı formüldür ve yedi grahanın hepsi sayılır.** Gerekçe: spec "(Σ iyicil − Σ kötücül) / 4, artı Merkür'ün ve Jüpiter'in bakış değerlerinin tamamı" der; çeyreğin üstüne mi yerine mi eklendiği ve kötücül Merkür'ün işareti yazılmamış. Harfiyen okuma tek fonksiyondadır (`drik_bala_at`). **Spec boşluğu 11.**
- **D12 — Graha savaşı: `GrahaWar { with, winner }`. Beş grahadan (Mars, Merkür, Jüpiter, Venüs, Satürn) ikisinin sidereal boylam farkı en çok 1° ise (sınır dahil) ikisi de işaretlenir. Üç graha 1° içindeyse her grahanın karşısındaki, boylamca en yakın olandır (eşitlikte `WAR_GRAHAS` sırasında önce gelen). Kazanan ekliptik enlemi büyük olandır; enlemler eşitse `WAR_GRAHAS` sırasında önce gelen. Güç aktarımı yoktur.** Gerekçe: spec `war: Some { with, winner }` yazar ama tipi, sınırı, üçlü durumu ve eşit enlemi tanımlamaz (sentetik konumlarda enlem 0'dır; kural belirli olmalı). **Spec boşluğu 12.**
- **D13 — Sıra toplam virupaya göredir (1 en güçlü); eşit toplamda `SHADBALA_GRAHAS` sırasında önce gelen öndedir. Bhava sırası da toplamadır; eşitlikte küçük numaralı bhava öndedir. `ratio = total / required`, `rupas = total / 60`.** Gerekçe: spec "oranı ve yedi graha arasındaki sırayı verir" der; sıranın toplama mı orana mı göre olduğu yazılmamış. Toplam klasik karşılaştırmadır; oran ayrı sütundur. **Spec boşluğu 13.**
- **D14 — Bhava bala: bhavanın burcu lagna burcundan sayılan rasi evinin burcudur (`house_sign`), derecesi lagnanın rasi çerçevesindeki derecesidir (D4); Yay ve Oğlak 15°'de bölünür (ilk yarı `[0°, 15°)`). Bhava ortaları 30° arayla durduğu için bhava dig bala 0, 10, …, 60 değerlerini alır. Lord `house_lord`'dur (Akrep: Mars, Kova: Satürn).** Gerekçe: spec §7 "eşit bhava"; K4.
- **D15 — El örnekleri çalışma kâğıdıdır: `tests/golden/modules/shadbala/WORKSHEET.md` (her kalemin aritmetiği) ve dört JSON (üç tam örnek, bir "saat bilinmiyor" örneği, `expected: null`). Düzeneğin girdisi iki isteğe bağlı alan kazanır: `astro` (`ShadbalaAstro`; zamanlar gün kesridir) ve `lats` (ekliptik enlemler). `expected`: yedi grahanın 20 kalemi, `rank` ve `war`; `time_lords`; on iki bhava. Tolerans 0,01 virupa (`WORKSHEET_TOL`). Ayrı bir test örneklerin kapsaması gereken durumları sabitler (gündüz, gece ve şafak öncesi doğum; büyüyen ve küçülen Ay; enlemle ve eşit enlemle biten savaş; negatif drik; Akrep ve Yay lagna; oranın iki yanı).** Gerekçe: spec §7 "Çalışma kâğıdı … ve eşlik eden JSON"; §13 şeması `astro` girdisini ve boş sonucu tanımlamaz. **Spec boşluğu 14.**
- **D16 — Bağımsız ikinci yazım (§13 katman 3): saat lordları testte elle yazılmış 7 × 24 tabloyla (`HAND_HORAS`), motorda döngüyle durur; bhava burç türleri elle yazılmış 12 satırlık tabloyla; naisargika ve gerekli değerler elle yazılmış listelerle karşılaştırılır.** Gerekçe: §13 "tablo olarak yazılabilen kurallar … dilim sıraları".
- **D17 — "Uçtan uca: beş `swetest` fixture anı" denetiminde beş anın dördü rapor verir; Tromsø 2050-12-21 kutup gecesindedir ve raporu `None`'dır (hata değildir). Test rapor sayısını (`4`) ve boş raporun yalnızca `SunriseUnavailable` uyarılı haritada görüldüğünü sabitler.** Gerekçe: spec "beş anda hata yok ve property'ler sağlanır" der; beş andan biri gün doğumsuzdur. **Spec kusuru 3.**
- **D18 — Bütçe entegrasyon testinde ölçülür: `compute_shadbala` ≤ 40 ms (§15; temel harita, iki giriş araması ve üç gün doğumu araması dahil).** Gerekçe: ölçüm (hata ayıklama derlemesi): ortanca 0,69–0,74 ms.
- **D19 — İki panel türü eklenir: `shadbala` (8 × 12) ve `bhava_bala` (6 × 14); ikisi de `MODULE_PANELS`'e girer ve paletten eklenip çıkarılır (K12). Ortak kanca `useShadbala`'dır; rapor oturumda paylaşılmaz, her panel kendi isteğini yapar (Plan 15 D16 sürer).** Gerekçe: komut 1 ms'nin altındadır; önbellek bayatlama riski getirirdi.
- **D20 — Sayı biçimi: virupa bir ondalık; rupa ve oran iki ondalık; eksi işareti tipografiktir (−). Rupa gösterimi kalemlerde `virupa / 60`'tır (birim çevirisi, gösterim); toplamın rupası Rust'ın `rupas` alanıdır. Oran 1'in altındaysa hücrede `<1` metni ve ekran okuyucu için "gerekli değerin altında" sözü durur; `ratio < 1` karşılaştırması gösterim eşiğidir. Yatay çubuk çizilmez.** Gerekçe: spec "bir ondalık" der; oranı bir ondalıkla yazmak 0,96'yı `1.0 <1` gösterirdi ve rupa değerleri (0–10) bir ondalıkta ayırt edilemezdi. Çubuk spec'te isteğe bağlıdır ("gösterilirse"). **Spec boşluğu 15.**
- **D21 — Shadbala paneli: özet tablosu her zaman görünür; "Kalemler" görünümü altına iki tablo ekler (sthana balanın beş kalemi, zaman gücünün sekiz kalemi). Altında zaman lordları listesi, birim açıklaması ve "Yöntem: shadbala.bphs_v1" satırı durur. Graha savaşı satırda "savaş: Ve" metniyle yazılır (ekran okuyucuya tam adlar ve kazanan okunur). Rapor yoksa tablo yerine bir satır açıklama çıkar (doğum saati bilinmiyor ya da gün doğumu yok; D3); araç çubuğu yükleme, hata ve "rapor yok" durumlarında da görünür. Klasik terimler üç dilde aynı yazılır (K11); "zaman gücü" çevrilir (Ek A).** Gerekçe: spec "alt kalemleri açılır" der ama düzeni vermez; 24 sütunluk tek tablo dar panelde okunmaz. **Spec boşluğu 16.**
- **D22 — Yeni görsel temel (PNG) ve yeni baskı düzeni yoktur (K10). `pnpm build` Vite'ın 500 kB parça uyarısını basmayı sürdürür (ölçüm: 531,7 kB) ve kapılar `✓ built` satırını arar (Plan 15 D22).** Gerekçe: on görsel temel harita öğelerini ve baskı sayfalarını çeker; modül panelleri bu görüntülerde yoktur (ölçüm: `e2e:visual` 10 geçti, temel dosyalar değişmedi).


## Global Constraints

- Lisans: AGPL-3.0-or-later. Yeni bağımlılık yok: `Cargo.toml`, `Cargo.lock`, `deny.toml`, `app/package.json`, `app/pnpm-lock.yaml` ve `app/vite.config.ts` değişmez; `cargo deny check` Task 8'de `advisories ok, bans ok, licenses ok, sources ok` verir.
- **Yasaklı ad kuralı (v1 spec §10 "İsim kuralları", ilk madde; ileri modüller spec'i Ek A):** o maddedeki dört ad ve türevleri hiçbir isimde, yorumda, metinde, dosya adında ya da commit mesajında kullanılmaz. Kural harf dizisi düzeyindedir: Türkçe sözcüklerin içinde de geçemez (yerine ör. "diğer", "ötekiler", "durur", "geri", "sürer", "tutar" yazılır). **Bu planda özel dikkat:** Shadbala'nın üçüncü gücünün klasik adı yasak diziyi içerir; bu güç her yerde "zaman gücü" / "temporal strength" / "force temporelle" ve `temporal` diye anılır, klasik adı hiçbir yere yazılmaz. `-ka` ile biten bir sözcüğe (`naisargika`, `karaka`) `-lar` çoğul eki ya da `L` ile başlayan bir ad bitişik yazılınca (camelCase'te de) yasak dizi oluşur; böyle birleşimler kurulmaz. Her commit adımı eklenen satırları, dosya adlarını ve commit mesajını `grep -inE 'k(a|ā|aa)la|\bcala\b'` ile tarar; çıktı boş olmalıdır (C13). Bu belgede de eşleşme yoktur.
- **Clean-room ve yorum metni yok:** yöntem klasik kuraldan özgün olarak uygulanır; başka ürünün adı, düzeni, metni ya da çıktısı kullanılmaz; kaynak gösterimi geneldir ("BPHS, graha gücü bölümü") ve alıntı yapılmaz. Paneller hesaplanmış değerleri ve klasik etiketleri gösterir (kalem adı, virupa, oran, sıra); bir gücün ne anlama geldiğini anlatan metin eklenmez. "Gerekli değerin altında" bir olgudur (karşılaştırma), yorum değildir.
- **Evler rasi evidir (K4), rasi çerçevesi `settings.zodiac`'ı izler (K5).** Kendradi bala ve bhava lordları lagna burcundan sayılan bütün burç evleridir; `CalcSettings.house_system` hiçbir kalemi etkilemez. MC ve lagna astronomik açılardır.
- **Doğum saati bilinmiyorsa rapor Rust'ta `None` döner (K2, D3).** Arayüzün gizlemesine güvenilmez.
- **El örnekleri (§13, K9):** `source` yalnızca `hand`'dir; beklenen değer kuraldan elle türetilir ve çalışma kâğıdında adım adım yazılır. Bir el örneği motorla uyuşmazsa önce kural yeniden okunur; **beklenen değer motor çıktısına bakarak düzeltilmez.** Gözden geçirme döngüsünün zorunlu maddesi: motoru yazan dışında bir gözden geçiren çalışma kâğıdını kuraldan yeniden hesaplar ve kâğıttaki tabloya adını yazar (Task 8 Step 5).
- Kapılar: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` (1.100), `cargo test --workspace`, `pnpm --dir app test`, `pnpm --dir app build` her görev sonunda yeşil; `pnpm --dir app e2e` Task 7 ve 8'de. `tsc` `strict`. `pnpm build` çıktısında aranan satır `✓ built`'tir (D22).
- **Toolchain:** `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"` (oturum başına bir kez). Node 24+, pnpm 11.8.0. Depo kısa bir yolda durmalıdır: Swiss Ephemeris 242 bayttan uzun ephemeris yolunu reddeder (çalışma ağacı uzun bir yoldaysa `/tmp` altında kısa bir yola klonlanır).
- **Çalışan `pnpm tauri dev` durdurulmaz** (port 1420); e2e 1421'i kullanır. `pkill -f` kullanılmaz. Üretilen `.app` çalıştırılmaz. `~/Library/Application Support/io.github.turgaydogusan.jyotishdesk` dizinine dokunulmaz.
- Yalnızca macOS hedeflenir. Yorum metni ve AI paneli bu planın dışındadır. Yogalar ve sonrası (3.5+) bu planın dışındadır.
- **Tabanlar (`4d435e7`):** `jyotish-core` lib 191, golden 8, `modules_golden` 7; `store` lib 86 (+1 `#[ignore]`); `jyotish-desk` lib 46, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; Vitest `Test Files  70 passed (70)`, `Tests  653 passed (653)`; e2e 15; görsel 10 (yerel, darwin); yayın betiği testleri 14.
- **Sayaç zinciri** (her satır görev bitince; boş hücre "önceki satırla aynı"). Bu plan yazılırken `4d435e7`'nin bir kopyasında her görev ayrı commit olarak uygulanıp ölçülmüştür.

  | Görev | `jyotish-core` lib | `modules_golden` | `jyotish-desk` `modules` | Vitest dosya / test | e2e |
  |---|---|---|---|---|---|
  | Taban | 191 | 7 | 1 | 70 / 653 | 15 |
  | Task 1 | 215 | | | | |
  | Task 2 | 231 | 9 | | | |
  | Task 3 | 237 | | | | |
  | Task 4 | | | 1 | 70 / 653 | |
  | Task 5 | | | | 71 / 665 | |
  | Task 6 | | | | 72 / 674 | |
  | Task 7 | | | | | 16 |
  | Final inceleme | 239 | | | 72 / 677 | |

  Değişmeyenler: `jyotish-core` golden 8; `store` lib 86 (+1 ign.); `jyotish-desk` lib 46, `bindings` 1, `compute_chart` 1, `open_client` 1, `ui_fixture` 2; görsel 10; yayın betiği testleri 14. `jyotish-core` lib Task 1'de 24 birim testi (sthana 8, zaman gücü 8, cheshta 4, `mod.rs` 4), Task 2'de 16 test (bhava 3, rapor 3, proptest 10), Task 3'te 6 toplayıcı testi kazanır. `modules_golden` iki test kazanır (`shadbala_hand_fixtures`, `shadbala_fixtures_cover_the_required_cases`). `jyotish-desk` `modules` tek test olarak durur (süreçte tek SE örneği); Task 4 ona `shadbala_checks`'i ekler. Task 4'te Vitest sayısı değişmez (`api.test.ts`'in tablosu büyür).
- **Taşınan kurallar (Plan 09 C1–C14, Plan 10 C2′, C15–C17), her görevin gereksinimidir:**
  - **C1** Gizlilik (spec §14): danışan adları, doğum verisi, not gövdeleri, yer bilgisi, dosya yolları ve `CommandError.params` loglanmaz; `no-console.test.ts` durur ve izin listesi büyümez. `ChartInput` ve `inputKey` doğum verisinden türer: loglanmaz, hata parametresine girmez. Hiçbir modül ağ isteği yapmaz; `ShadbalaReport` veritabanına yazılmaz. Yeni hata kodu yoktur (giriş aramasının yalnızca bozuk ephemeris'te görülen hatası mevcut `NonFiniteResult("sun_entry")`'dir; parametresi alan adıdır).
  - **C2′** Hiçbir IPC komutu arayüzden dosya yolu ya da adres almaz. Yeni komutun tek bağımsız değişkeni `ChartInput`'tur; `api.test.ts`'in dizgi bağımsız değişken listesi değişmez.
  - **C3** Göç V1 donmuştur; `crates/store/src/schema.rs` değişmez. `CalcSettings`, `AppPrefs` ve `ViewState` tipleri ve şema sürümleri değişmez.
  - **C4** Arayüz astrolojik hesap yapmaz: kalemler, toplamlar, oran, sıra, savaş ve zaman lordları Rust'tan gelir. Virupa → rupa çevirisi, sayıların biçimlenmesi ve `<1` işaretinin eşiği gösterimdir (D20).
  - **C5/C17** Otomatik kayıtlar ve boşaltılmaları değişmez: panel ayarları düzenle birlikte, mevcut debounce ve `save_view_state` kaydıyla yazılır. **C6** Düzen ayrıştırıcıları (`parseLayoutState`, Rust `check_panel_layout`) değişmez; yeni panel türleri kataloğa eklenir.
  - **C7** Kısayollar yalnızca `shortcuts.ts`'tedir; bu plan kısayol eklemez (K12). **C8** Her pencere `shell/Dialog`'u kullanır (bu plan pencere eklemez).
  - **C9** Renk yalnızca token'lardan gelir (hex yok) ve tek başına anlam taşımaz: oranın 1'in altında olması `<1` metniyle, savaş "savaş: …" metniyle, vurgu zemin ve yazı ağırlığıyla, sabitleme `aria-pressed` ile gösterilir.
  - **C10** Üretilen dosyalar elle düzenlenmez: `app/ui/bindings.ts` ve `app/ui/test/fixtures/delhi.json` Task 4'te komutla yeniden üretilir.
  - **C11** Görsel PNG temelleri yereldir; bu planda değişmeleri beklenmez (D22).
  - **C12** Her `CommandError.code` üç dilde çevrilidir; bu plan kod eklemez. Her yeni i18n anahtarı üç dilde vardır (`i18n.test.ts` anahtar kümelerini ve yer tutucuları karşılaştırır).
  - **C13** Yasaklı ad taraması her commit'te; depo dışındaki ilerleme notları ve `.superpowers/` hiçbir zaman `git add` edilmez.
  - **C14** Plan 07'nin gün doğumu toleransı değişmez; gün doğumu ve batımı v1'in tanımıyla (`settings.sunrise_def`) okunur. **C15** `alert`, `confirm`, `prompt`, `window.print()` ve `window.open()` çağrılmaz. **C16** ⌘ kısayolları değişmez.
- **Retro** her yerde düz `R`'dir ve Rahu ile Ketu'da gösterilmez (UI spec §2.3). Shadbala ve Bhava bala panelleri retro göstermez; Rahu ve Ketu'nun gücü hesaplanmaz (spec §7 kapsam dışı).
- **Erişilebilirlik:** WCAG AA. Tablolar `caption`, `scope="col"` ve `scope="row"` taşır; seçicilerin görünür etiketi vardır; `<1` ve savaş işaretlerinin sözcük karşılığı ekran okuyucuya okunur; yükleme iskeleti `aria-busy`, hata `role="alert"` ve "Yeniden dene" düğmesi taşır; "Yeniden dene" hata kutusunu değiştirirken odak panelde durur; graha, bhava ve lord adları ortak vurgu düğmesidir (`PinButton`, `aria-pressed`).
- Commit mesajları İngilizcedir ve deponun `Co-Authored-By:` satırıyla biter (yürütücünün oturum yönergesindeki satır). Her görev tek commit'tir (gerekirse inceleme düzeltmesi ayrı commit). `git push`, `git tag`, yıkıcı git komutu ve `.superpowers/` commit'i yoktur.
- Kod blokları `4d435e7` ağacına göre yazılmış ve bu plan yazılırken deponun bir kopyasında koşulmuştur (Self-Review madde 5). Yeni dosyalar tam içerikle verilir ("**Yeni dosya:**"). Değişen dosyalar `diff` bloğuyla verilir: blok bir dosyaya konup depo kökünde `git apply <dosya>` ile ya da elle uygulanır; bağlam tutmazsa önceki görevin adımı denetlenir, elle uydurma yapılmaz. Bir dosyanın test ve uygulama değişiklikleri ayrı adımlarda ayrı bloklardır. Dosya ve diff blokları dört ters tırnakla çevrilidir (içlerinde üç ters tırnak geçebilir); komut blokları üç ters tırnaklıdır. `app/ui/bindings.ts` ve `app/ui/test/fixtures/delhi.json` diff olarak verilmez; adımlardaki komutlar üretir.

## Sonraki alt projelere verilen API

Bu planın ürettiği ve sonraki işlerin tüketeceği adlar. Rust adları `jyotish_core`'dan dışa aktarılır; imzalar görevlerde tam olarak verilir.

| API | Tüketen | Not |
|---|---|---|
| `shadbala(positions, astro) -> Option<ShadbalaReport>`, `ShadbalaReport`, `GrahaShadbala`, `SthanaBala`, `TemporalBala`, `TimeLords`, `BhavaBala`, `GrahaWar`, `SHADBALA_METHOD`, `SHADBALA_GRAHAS` | AI paketleyicisi; 3.9'un arama koşulları (güç sırası) | Lagna yoksa `None` |
| `compute_shadbala(eph, input)`, `compute_shadbala_astro(eph, input, chart)`, `ShadbalaAstro`; Tauri komutu `compute_shadbala` | Arayüz; AI paneli | Saat bilinmiyorsa ya da gün doğumu yoksa `None` |
| `sun_entry_before(eph, zodiac, jd_ut, target_deg)` | 3.7 `timeline` (güneş girişleri), 3.9 olay araması | Newton; sabit adım sayısı |
| `vara_lord(vara)`, `hora_lord(weekday_lord, index)`, `hora_index(astro)`, `HORA_CYCLE` | 3.7 Muhurta (gün dilimleri, saat lordları) | |
| `mean_longitudes(jd_ut)`, `MeanLongitudes`, `circular_midpoint` | Başka bir hareket ölçüsü eklenirse | Tropikal, tarihin ekinoksu |
| `drik_bala_at(target_lon, skip, positions)`, `dig_bala`, `uccha_bala`, `graha_war`, `naisargika_bala`, `required_bala` | 3.5 yogalar (güç koşulu isteyen kurallar), 3.8 Prashna olguları | Saf; `Positions` yeter |
| `panchang::vara_at`, `panchang::following_sunrise` (`pub(crate)`) | 3.7 `timeline` | v1 vara kuralı |
| Arayüz: `modules/useShadbala.ts`; `panels/bala.tsx` (`balaUnit`, `formatBala`, `formatFixed`, `UnitSelect`, `BalaUnavailable`) | Güç tablosu gösteren sonraki paneller | |

## Dosya Yapısı (bu plan)

```
crates/jyotish-core/
  src/shadbala/mod.rs             tipler, dig, naisargika, drik, graha savaşı, rapor (yeni)
  src/shadbala/sthana.rs          uccha, saptavargaja, ojayugma, kendradi, drekkana (yeni)
  src/shadbala/temporal.rs        zaman gücü kalemleri ve zaman lordları (yeni)
  src/shadbala/cheshta.rs         ortalama boylamlar ve cheshta (yeni)
  src/shadbala/bhava.rs           bhava bala (yeni)
  src/shadbala/collect.rs         toplayıcı: giriş araması, astronomik girdi, compute_shadbala (yeni)
  src/panchang.rs                 vara_at, following_sunrise; day_sunrise pub(crate)
  src/lib.rs                      modül kaydı ve dışa aktarımlar
  src/proptests.rs                shadbala_properties
  tests/modules_golden.rs         çalışma kâğıdı düzeneği ve kapsam testi
tests/golden/modules/
  README.md                       shadbala satırı; astro ve lats girdisi
  shadbala/WORKSHEET.md           çalışma kâğıdı (yeni)
  shadbala/*.json                 4 el örneği (yeni)
app/src-tauri/
  src/module_commands.rs          compute_shadbala
  src/lib.rs                      komut kaydı
  tests/modules.rs                shadbala entegrasyonu, astronomik çapraz denetim, bütçe
  tests/ui_fixture.rs             fikstüre shadbala
app/ui/
  bindings.ts                     üretilir
  test/fixtures/delhi.json        üretilir
  test/fixture.ts                 UiFixture.shadbala
  api.ts, api.test.ts             computeShadbala
  modules/useShadbala.ts          ortak kanca (yeni)
  panels/bala.tsx                 birim ayarı, sayı biçimi, "rapor yok" açıklaması (yeni)
  panels/ShadbalaPanel.tsx (+test)           (yeni)
  panels/BhavaBalaPanel.tsx (+test)          (yeni)
  panels/registry.ts, panels.test.tsx        iki panel türü
  panels/panels.css
  layout/model.ts (+test)         PANEL_KINDS, MODULE_PANELS
  i18n/{tr,en,fr}.ts              panel.shadbala, panel.bhava_bala, bala, shadbala, bhavaBala
app/e2e/{ipc.ts,desk.spec.ts}     komutun taklidi; iki panelin akışı
```

### Kapsam

| Spec maddesi | Yer |
|---|---|
| §7 Sthana bala: uccha, saptavargaja, ojayugma, kendradi, drekkana | Task 1 (`sthana.rs`; D4, D5) |
| §7 Dig bala | Task 1 (`dig_bala`) |
| §7 Zaman gücü: gündüz–gece, paksha, tribhaga, yıl/ay/gün/saat lordu, ayana | Task 1 (`temporal.rs`; D6) |
| §7 Graha savaşı işareti | Task 1 (`graha_war`; D12) |
| §7 Cheshta bala; ortalama boylamlar | Task 1 (`cheshta.rs`; D8, D9) |
| §7 Naisargika bala; drik bala | Task 1 (D10, D11) |
| §7 Toplam, gerekli en az değer, oran, sıra | Task 2 (`shadbala`; D13) |
| §7 Bhava bala: bhavadhipati, dig, drishti; sıra | Task 2 (`bhava.rs`; D14) |
| §7 Veri modeli; saf fonksiyon ve `ShadbalaAstro` | Task 1, 2 (D2) |
| §7 Yıl ve ay girişi; toplayıcı; `None` durumları | Task 3 (D3, D7) |
| §7 `compute_shadbala` komutu; K1, K2, K8 | Task 4 |
| §7 Arayüz: Shadbala paneli (ayrıntı, birim, zaman lordları, `<1`, savaş metni) | Task 5 (D19–D21) |
| §7 Arayüz: Bhava bala paneli | Task 6 |
| §7 Test: çalışma kâğıdı (≥ 3 girdi, tolerans 0,01) | Task 2 (D15); Task 8 Step 5 |
| §7 Test: kalem düzeyinde el örnekleri (her kalem ≥ 4 durum) | Task 1, 2 (`mod tests`) |
| §7 Test: property'ler | Task 2 (`shadbala_properties`) |
| §7 Test: astronomik çapraz denetim | Task 4 (`sky_facts`; D8, D9) |
| §7 Test: uçtan uca beş golden anı | Task 4 (D17) |
| §13 Bağımsız ikinci yazım | Task 1, 2 (D16) |
| §13 Komut katmanı; §15 bütçe (≤ 40 ms) | Task 4 (D18) |
| §13 Panel durumları: boş, yükleniyor, hata, dolu, "saat bilinmiyor" | Task 5, 6 |
| §14 Gizlilik | C1; Task 8 taramaları |
| K12 Paletten ekleme | Task 5, 6; Task 7 |
| Uçtan uca akış | Task 7 |
| Son doğrulama, elle denetim, çalışma kâğıdının gözden geçirilmesi | Task 8 |

---

### Task 1: Çekirdek: kalemler (sthana, dig, zaman gücü, cheshta, naisargika, drik, graha savaşı)

Spec §7 "Yöntem — altı güç", "Veri modeli"; §13 katman 1 (kalem düzeyinde el örnekleri) ve katman 3 (elle yazılmış tablolar). Hiçbir mevcut davranış değişmez; yalnızca yeni modül eklenir. Rapor (`shadbala`), bhava bala, çalışma kâğıdı ve değişmezler Task 2'dedir; toplayıcı Task 3'tedir.

**Files:**
- Create: `crates/jyotish-core/src/shadbala/mod.rs`, `crates/jyotish-core/src/shadbala/sthana.rs`, `crates/jyotish-core/src/shadbala/temporal.rs`, `crates/jyotish-core/src/shadbala/cheshta.rs`
- Modify: `crates/jyotish-core/src/lib.rs`
- Test: dört dosyanın `mod tests`'i (24 test)

**Interfaces:**
- Consumes (v1, Plan 14): `Positions` (`point`, `sign_of`, `house_of`, `synthetic`, `without_lagna`), `GrahaPoint`, `Graha`, `Sign`, `Vara`, `ChartWarning`, `dignity::{compound_relation, debilitation_point, moolatrikona, natural_relation, own_signs, sign_lord, temporal_friend, Dignity}`, `varga::{varga_position, Varga}`, `drishti::{sphuta_drishti, ASPECTING_GRAHAS}`, `nature::{nature, Nature}`, `zodiac::{angular_distance, normalize}`.
- Produces (hepsi `jyotish_core`'dan dışa aktarılır):
  - Tipler: `SthanaBala`, `TemporalBala`, `GrahaWar { with, winner }`, `GrahaShadbala`, `TimeLords`, `BhavaBala`, `ShadbalaReport` (spec §7'deki alanlarla), `ShadbalaAstro` (D2; `is_consistent()`), `MeanLongitudes`.
  - Sabitler: `SHADBALA_METHOD`, `SHADBALA_GRAHAS: [Graha; 7]`, `VIRUPAS_PER_RUPA`, `WAR_GRAHAS`, `GRAHA_WAR_ORB_DEG`, `SAPTAVARGA: [Varga; 7]`, `HORA_CYCLE`, `DAY_TRIBHAGA`, `NIGHT_TRIBHAGA`, `AYANA_OBLIQUITY_DEG`, `YEAR_LORD_BALA`, `MONTH_LORD_BALA`, `WEEKDAY_LORD_BALA`, `HORA_LORD_BALA`, `MEAN_SUN` … `MEAN_SATURN`.
  - Sthana: `rasi_lon(&GrahaPoint) -> f64`, `uccha_bala(graha, rasi_lon) -> f64`, `saptavargaja_points(graha, varga, &Positions) -> f64`, `saptavargaja_bala(graha, &Positions) -> f64`, `ojayugma_bala(graha, rasi_lon) -> f64`, `kendradi_bala(house: u8) -> f64`, `drekkana_bala(graha, deg_in_sign) -> f64`, `sthana_bala(graha, &Positions) -> Option<SthanaBala>`.
  - Zaman gücü: `vara_lord(Vara) -> Graha`, `hours_from_noon(&ShadbalaAstro) -> f64`, `day_night_bala(graha, m) -> f64`, `paksha_elongation(&Positions) -> f64`, `paksha_bala(graha, &Positions) -> f64`, `is_daytime(&ShadbalaAstro) -> bool`, `tribhaga_lord(&ShadbalaAstro) -> Graha`, `hora_index(&ShadbalaAstro) -> u8`, `hora_lord(weekday_lord, index) -> Graha`, `time_lords(&ShadbalaAstro) -> TimeLords`, `ayana_declination(tropical_lon) -> f64`, `ayana_bala(graha, tropical_lon) -> f64`, `temporal_bala(graha, &Positions, &ShadbalaAstro, &TimeLords) -> TemporalBala`.
  - Cheshta: `mean_longitudes(jd_ut) -> MeanLongitudes`, `circular_midpoint(a, b) -> f64`, `cheshta_from(slow, true_lon, apex) -> f64`, `cheshta_bala(graha, &Positions, &ShadbalaAstro) -> f64`.
  - `mod.rs`: `required_bala(graha) -> f64`, `naisargika_bala(graha) -> f64`, `dig_bala(graha, lon, lagna_lon, mc_lon) -> f64`, `drik_bala_at(target_lon, skip: Option<Graha>, &Positions) -> f64`, `drik_bala(graha, &Positions) -> f64`, `graha_war(graha, &Positions) -> Option<GrahaWar>`.

- [ ] **Step 1: Tabanları ölç**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
git log -1 --format=%h
git status --short
rustc --version
cargo test --workspace 2>&1 | grep -E "^test result|Running"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
```
Expected: `git status --short` boş; `rustc 1.100.0` ya da yenisi; `jyotish-core` lib `191 passed`, golden `8 passed`, `modules_golden` `7 passed`; `store` lib `86 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `46 passed`, `bindings` 1, `compute_chart` 1, `modules` 1, `open_client` 1, `ui_fixture` 2; Vitest `70 passed (70)`, `653 passed (653)`. Commit `4d435e7` değilse ölçülen sayılar ilerleme notlarına taban olarak yazılır ve zincir oradan yürütülür (farklar aynıdır). (Vitest'te `NotesPanel.test.tsx › New note adds an empty note…` yük altında seyrek olarak düşebilir; bu plandan bağımsızdır, yeniden koşulur ve ilerleme notlarına yazılır.)

- [ ] **Step 2: Modülü kaydet (derleme başarısız olur)**

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 00a0538..0b7286a 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -4,7 +4,8 @@
 //! Kapsam: ayanamsa, grahalar, lagna, bhavalar, nakshatra, dignity, yanma (M2), 16 varga,
 //! Vimshottari dasha, panchang, ashtakavarga ve transit (M3). İleri modüllerin ortak temeli
 //! (Alt proje 3.1): `positions` (ortak girdi), `lordship`, `nature` ve `drishti`. Jaimini
-//! çekirdeği (Alt proje 3.2) ve chara dasha (Alt proje 3.3): `jaimini`.
+//! çekirdeği (Alt proje 3.2) ve chara dasha (Alt proje 3.3): `jaimini`. Shadbala ve Bhava bala
+//! (Alt proje 3.4): `shadbala`.
 
 pub mod ashtakavarga;
 mod chart;
@@ -25,6 +26,7 @@ pub mod positions;
 #[cfg(test)]
 mod proptests;
 pub mod settings;
+pub mod shadbala;
 pub mod transit;
 pub mod varga;
 pub mod zodiac;
@@ -72,6 +74,16 @@ pub use settings::{
     AdvancedSettings, AyanamsaSystem, CalcSettings, CombustionOrbs, HouseSystem, KarakaScheme,
     NodeKind, SunriseDef, ZodiacMode, CALC_SETTINGS_SCHEMA_VERSION, JD_UT_RANGE,
 };
+pub use shadbala::{
+    ayana_bala, ayana_declination, cheshta_bala, cheshta_from, circular_midpoint, day_night_bala,
+    dig_bala, drekkana_bala, drik_bala, drik_bala_at, graha_war, hora_index, hora_lord,
+    hours_from_noon, is_daytime, kendradi_bala, mean_longitudes, naisargika_bala, ojayugma_bala,
+    paksha_bala, paksha_elongation, rasi_lon, required_bala, saptavargaja_bala,
+    saptavargaja_points, sthana_bala, temporal_bala, time_lords, tribhaga_lord, uccha_bala,
+    vara_lord, BhavaBala, GrahaShadbala, GrahaWar, MeanLongitudes, ShadbalaAstro, ShadbalaReport,
+    SthanaBala, TemporalBala, TimeLords, GRAHA_WAR_ORB_DEG, HORA_CYCLE, SAPTAVARGA,
+    SHADBALA_GRAHAS, SHADBALA_METHOD, VIRUPAS_PER_RUPA, WAR_GRAHAS,
+};
 pub use transit::{compute_transits, TransitGraha, TransitSnapshot};
 pub use varga::{
     all_vargas, is_vargottama, varga_chart, varga_position, Varga, VargaChart, VargaPlacement,
````

- [ ] **Step 3: Derlemenin başarısız olduğunu doğrula**

```bash
cargo test -p jyotish-core --lib shadbala 2>&1 | grep -E "^error"
```
Expected: `error[E0583]: file not found for module `shadbala`` ve `could not compile`. Sonraki dört adım modülü yazar; her dosyanın `mod tests` bölümü o dosyadaki kalemlerin el örnekleridir (kuraldan elle türetilmiştir; her beklenen değerin gerekçesi yanındaki yorumdadır). Dosyayı yazarken önce testleri okuyun: bir test düşerse önce kural (spec §7 ve ilgili D-kararı) yeniden okunur, beklenen değer motor çıktısına bakarak değiştirilmez.

- [ ] **Step 4: Tipler, dig, naisargika, drik ve graha savaşı**

**Yeni dosya:** `crates/jyotish-core/src/shadbala/mod.rs`

````rust
//! Shadbala ve Bhava bala (ileri modüller spec'i §7). Yedi grahanın altı gücü (sthana, dig,
//! zaman gücü, cheshta, naisargika, drik), toplamı, gerekli en az değere oranı ve sırası; on
//! iki bhavanın gücü. Kaynak: BPHS'nin graha gücü ve bhava gücü bölümleri. Birim virupadır
//! (60 virupa = 1 rupa).
//!
//! Hesap fonksiyonları ephemeris görmez (K3): yalnızca `Positions` ve `ShadbalaAstro` alır.
//! Ek A: üçüncü güç yalnızca "zaman gücü" (`temporal`) adıyla anılır.

mod cheshta;
mod sthana;
mod temporal;

use serde::{Deserialize, Serialize};

use crate::chart::ChartWarning;
use crate::drishti::{sphuta_drishti, ASPECTING_GRAHAS};
use crate::graha::Graha;
use crate::nature::{nature, Nature};
use crate::panchang::Vara;
use crate::positions::Positions;
use crate::zodiac::{angular_distance, normalize};

pub use cheshta::{
    cheshta_bala, cheshta_from, circular_midpoint, mean_longitudes, MeanLongitudes, MEAN_JUPITER,
    MEAN_MARS, MEAN_MERCURY, MEAN_SATURN, MEAN_SUN, MEAN_VENUS,
};
pub use sthana::{
    drekkana_bala, kendradi_bala, ojayugma_bala, rasi_lon, saptavargaja_bala, saptavargaja_points,
    sthana_bala, uccha_bala, SAPTAVARGA,
};
pub use temporal::{
    ayana_bala, ayana_declination, day_night_bala, hora_index, hora_lord, hours_from_noon,
    is_daytime, paksha_bala, paksha_elongation, temporal_bala, time_lords, tribhaga_lord,
    vara_lord, AYANA_OBLIQUITY_DEG, DAY_TRIBHAGA, HORA_CYCLE, HORA_LORD_BALA, MONTH_LORD_BALA,
    NIGHT_TRIBHAGA, WEEKDAY_LORD_BALA, YEAR_LORD_BALA,
};

/// `ShadbalaReport::method`: bu modüldeki yöntemin kimliği.
pub const SHADBALA_METHOD: &str = "shadbala.bphs_v1";
/// Shadbala hesaplanan yedi graha; `ShadbalaReport::grahas` bu sıradadır (`Graha::ALL`'un ilk
/// yedisi).
pub const SHADBALA_GRAHAS: [Graha; 7] = ASPECTING_GRAHAS;
/// 60 virupa = 1 rupa.
pub const VIRUPAS_PER_RUPA: f64 = 60.0;
/// Graha savaşına girebilen beş graha (Güneş ve Ay girmez).
pub const WAR_GRAHAS: [Graha; 5] = [
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];
/// Graha savaşı: iki grahanın boylam farkı en çok bu kadar derecedir (sınır dahil).
pub const GRAHA_WAR_ORB_DEG: f64 = 1.0;

/// Sthana bala (konum gücü) kalemleri, virupa.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SthanaBala {
    pub uccha: f64,
    pub saptavargaja: f64,
    pub ojayugma: f64,
    pub kendradi: f64,
    pub drekkana: f64,
    pub total: f64,
}

/// Zaman gücü kalemleri, virupa. `ayana` Güneş'te iki katıdır.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TemporalBala {
    pub day_night: f64,
    pub paksha: f64,
    pub tribhaga: f64,
    pub year_lord: f64,
    pub month_lord: f64,
    pub weekday_lord: f64,
    pub hora_lord: f64,
    pub ayana: f64,
    pub total: f64,
}

/// Graha savaşı işareti: `with` karşıdaki graha, `winner` kazanan. Güç aktarımı uygulanmaz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GrahaWar {
    pub with: Graha,
    pub winner: Graha,
}

/// Bir grahanın Shadbala sonucu, virupa.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct GrahaShadbala {
    pub graha: Graha,
    pub sthana: SthanaBala,
    pub dig: f64,
    pub temporal: TemporalBala,
    pub cheshta: f64,
    pub naisargika: f64,
    /// Negatif olabilir.
    pub drik: f64,
    /// Altı gücün toplamı.
    pub total: f64,
    /// `total / 60`.
    pub rupas: f64,
    /// Klasik gerekli en az değer, virupa.
    pub required: f64,
    /// `total / required`.
    pub ratio: f64,
    /// Yedi graha arasında toplamın sırası: 1 en güçlü.
    pub rank: u8,
    pub war: Option<GrahaWar>,
}

/// Zaman gücünde değer alan beş lord.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct TimeLords {
    pub year: Graha,
    pub month: Graha,
    pub weekday: Graha,
    pub hora: Graha,
    pub tribhaga: Graha,
}

/// Bir bhavanın gücü, virupa.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct BhavaBala {
    /// 1..=12.
    pub house: u8,
    pub lord: Graha,
    /// Bhavadhipati bala: lordun Shadbala toplamı.
    pub lord_bala: f64,
    pub dig: f64,
    /// Negatif olabilir.
    pub drishti: f64,
    pub total: f64,
    /// On iki bhava arasında toplamın sırası: 1 en güçlü.
    pub rank: u8,
}

/// `compute_shadbala` sonucu. AI yorum paneli için sözleşmedir.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ShadbalaReport {
    /// `SHADBALA_GRAHAS` sırasında.
    pub grahas: [GrahaShadbala; 7],
    pub time_lords: TimeLords,
    /// 1.–12. bhava.
    pub bhavas: [BhavaBala; 12],
    pub method: String,
    pub warnings: Vec<ChartWarning>,
}

/// Saf hesabın astronomik girdisi (K3): toplayıcı ephemeris'ten üretir, el örnekleri elle
/// yazar. Zamanlar UT Jülyen günüdür; hesap yalnızca farklarını kullanır.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShadbalaAstro {
    /// Doğum anı.
    pub jd_ut: f64,
    /// Doğumdan önceki son gün doğumu (doğum anı dahil).
    pub sunrise_jd_ut: f64,
    /// O gün doğumundan sonraki ilk gün batımı.
    pub sunset_jd_ut: f64,
    /// Sonraki gün doğumu (doğumdan sonra).
    pub next_sunrise_jd_ut: f64,
    /// MC'nin sidereal boylamı (astronomik açı; ev sistemi ayarından bağımsız).
    pub mc_lon: f64,
    /// Yedi grahanın tropikal jeosantrik boylamı, `SHADBALA_GRAHAS` sırasında.
    pub tropical_lons: [f64; 7],
    /// Doğum anındaki tropikal ortalama boylamlar.
    pub mean: MeanLongitudes,
    /// Doğum gününün varası (gün doğumunda değişir).
    pub weekday: Vara,
    /// Güneş'in doğumdan önceki son sidereal Koç girişinin gerçekleştiği vara.
    pub year_entry_weekday: Vara,
    /// Güneş'in bulunduğu sidereal burca girişinin gerçekleştiği vara.
    pub month_entry_weekday: Vara,
}

impl ShadbalaAstro {
    /// Değerler sonlu ve sıralı mı: doğuş ≤ doğum < sonraki doğuş, doğuş < batış < sonraki
    /// doğuş.
    pub fn is_consistent(&self) -> bool {
        let m = &self.mean;
        let finite = [
            self.jd_ut,
            self.sunrise_jd_ut,
            self.sunset_jd_ut,
            self.next_sunrise_jd_ut,
            self.mc_lon,
            m.sun,
            m.mercury,
            m.venus,
            m.mars,
            m.jupiter,
            m.saturn,
        ]
        .into_iter()
        .chain(self.tropical_lons)
        .all(f64::is_finite);
        finite
            && self.sunrise_jd_ut <= self.jd_ut
            && self.jd_ut < self.next_sunrise_jd_ut
            && self.sunrise_jd_ut < self.sunset_jd_ut
            && self.sunset_jd_ut < self.next_sunrise_jd_ut
    }
}

/// Klasik gerekli en az Shadbala, virupa. Düğümler 0.
pub fn required_bala(graha: Graha) -> f64 {
    match graha {
        Graha::Sun | Graha::Jupiter => 390.0,
        Graha::Moon => 360.0,
        Graha::Mars | Graha::Saturn => 300.0,
        Graha::Mercury => 420.0,
        Graha::Venus => 330.0,
        Graha::Rahu | Graha::Ketu => 0.0,
    }
}

/// Naisargika bala (doğal güç): sabit, 60'ın yedide birinin katları. Güneş 60; Ay 51,43; Venüs
/// 42,86; Jüpiter 34,29; Merkür 25,71; Mars 17,14; Satürn 8,57. Düğümler 0.
pub fn naisargika_bala(graha: Graha) -> f64 {
    let sevenths = match graha {
        Graha::Sun => 7.0,
        Graha::Moon => 6.0,
        Graha::Venus => 5.0,
        Graha::Jupiter => 4.0,
        Graha::Mercury => 3.0,
        Graha::Mars => 2.0,
        Graha::Saturn => 1.0,
        Graha::Rahu | Graha::Ketu => 0.0,
    };
    60.0 * sevenths / 7.0
}

/// Dig bala (yön gücü): güçsüz noktadan açısal uzaklık (0°–180°) / 3. Güçsüz nokta: Güneş ve
/// Mars için IC (MC + 180°), Ay ve Venüs için MC, Merkür ve Jüpiter için alçalan (lagna +
/// 180°), Satürn için lagna. Üç boylam da aynı çerçevededir (sidereal). Düğümler 0.
pub fn dig_bala(graha: Graha, lon: f64, lagna_lon: f64, mc_lon: f64) -> f64 {
    let powerless = match graha {
        Graha::Sun | Graha::Mars => mc_lon + 180.0,
        Graha::Moon | Graha::Venus => mc_lon,
        Graha::Mercury | Graha::Jupiter => lagna_lon + 180.0,
        Graha::Saturn => lagna_lon,
        Graha::Rahu | Graha::Ketu => return 0.0,
    };
    angular_distance(lon, normalize(powerless)) / 3.0
}

/// Bir noktaya gelen bakış gücü: yedi grahanın (`skip` dışında) sphuta drishti değerleriyle
/// `(Σ iyicil − Σ kötücül) / 4`, artı Merkür'ün ve Jüpiter'in bakış değerlerinin tamamı.
/// İyicil/kötücül sınıfı `nature`'dır (K6). `target_lon` sidereal boylamdır.
pub fn drik_bala_at(target_lon: f64, skip: Option<Graha>, positions: &Positions) -> f64 {
    let mut sum = 0.0;
    for graha in SHADBALA_GRAHAS {
        if Some(graha) == skip {
            continue;
        }
        let value = sphuta_drishti(graha, positions.point(graha).sidereal_lon, target_lon);
        sum += match nature(graha, positions) {
            Nature::Benefic => value,
            Nature::Malefic => -value,
        } / 4.0;
        if matches!(graha, Graha::Mercury | Graha::Jupiter) {
            sum += value;
        }
    }
    sum
}

/// Drik bala (bakış gücü): grahaya diğer altı grahadan gelen bakışlar. Negatif olabilir.
pub fn drik_bala(graha: Graha, positions: &Positions) -> f64 {
    drik_bala_at(positions.point(graha).sidereal_lon, Some(graha), positions)
}

/// Graha savaşı: `graha` beş grahadan biriyse ve bir başkası `GRAHA_WAR_ORB_DEG` içindeyse
/// işaretlenir. Karşıdaki graha en yakın olandır (eşitlikte `WAR_GRAHAS` sırasında önce
/// gelen); kazanan ekliptik enlemi daha kuzeyde olandır (eşit enlemde `WAR_GRAHAS` sırasında
/// önce gelen).
pub fn graha_war(graha: Graha, positions: &Positions) -> Option<GrahaWar> {
    let place = |g: Graha| WAR_GRAHAS.iter().position(|w| *w == g);
    let own = place(graha)?;
    let me = positions.point(graha);
    let mut nearest: Option<(Graha, f64)> = None;
    for other in WAR_GRAHAS {
        if other == graha {
            continue;
        }
        let gap = angular_distance(me.sidereal_lon, positions.point(other).sidereal_lon);
        if gap <= GRAHA_WAR_ORB_DEG && nearest.is_none_or(|(_, best)| gap < best) {
            nearest = Some((other, gap));
        }
    }
    let (with, _) = nearest?;
    let other = positions.point(with);
    let wins = me.lat > other.lat || (me.lat == other.lat && Some(own) < place(with));
    Some(GrahaWar {
        with,
        winner: if wins { graha } else { with },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;

    /// `Graha::ALL` sırası: Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Rahu, Ketu.
    const BASE: [f64; 9] = [10.0, 100.0, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0];

    fn sky(lons: [f64; 9]) -> Positions {
        Positions::synthetic(Some(0.0), lons)
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn dig_is_zero_at_the_powerless_point_and_sixty_opposite() {
        // Lagna 0°, MC 270°: IC 90°, alçalan 180°.
        for (g, powerless) in [
            (Sun, 90.0),
            (Mars, 90.0),
            (Moon, 270.0),
            (Venus, 270.0),
            (Mercury, 180.0),
            (Jupiter, 180.0),
            (Saturn, 0.0),
        ] {
            assert!(close(dig_bala(g, powerless, 0.0, 270.0), 0.0), "{g:?}");
            assert!(
                close(dig_bala(g, powerless + 180.0, 0.0, 270.0), 60.0),
                "{g:?}"
            );
            assert!(
                close(dig_bala(g, powerless + 90.0, 0.0, 270.0), 30.0),
                "{g:?}"
            );
            assert!(
                close(dig_bala(g, powerless - 45.0, 0.0, 270.0), 15.0),
                "{g:?}"
            );
        }
        // MC lagnadan 90° geride olmak zorunda değildir: Güneş MC'de (250°) 60 alır.
        assert!(close(dig_bala(Sun, 250.0, 0.0, 250.0), 60.0));
        assert!(close(dig_bala(Moon, 70.0, 0.0, 250.0), 60.0));
        assert_eq!(dig_bala(Rahu, 10.0, 0.0, 270.0), 0.0);
    }

    #[test]
    fn naisargika_is_fixed_and_required_values_are_the_classical_ones() {
        let rounded: Vec<f64> = SHADBALA_GRAHAS
            .into_iter()
            .map(|g| (naisargika_bala(g) * 100.0).round() / 100.0)
            .collect();
        // Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn.
        assert_eq!(rounded, [60.0, 51.43, 17.14, 25.71, 34.29, 42.86, 8.57]);
        let required: Vec<f64> = SHADBALA_GRAHAS.into_iter().map(required_bala).collect();
        assert_eq!(required, [390.0, 360.0, 300.0, 420.0, 390.0, 330.0, 300.0]);
        assert_eq!((naisargika_bala(Rahu), required_bala(Ketu)), (0.0, 0.0));
    }

    #[test]
    fn drik_adds_a_quarter_of_each_aspect_and_all_of_mercury_and_jupiter() {
        // Hedef 0°. Yalnızca Jüpiter bakar: 180° geride (tam bakış 60); diğerleri hedefin 10°
        // gerisinde (açı 10° → 0) durur.
        let mut lons = [350.0; 9];
        lons[Jupiter as usize] = 180.0;
        let p = sky(lons);
        // İyicil: 60 / 4 = 15; Jüpiter'in tamamı: +60 → 75.
        assert!(close(drik_bala_at(0.0, None, &p), 75.0));
        // Yalnızca Satürn bakar (açı 180° → 60): kötücül, −15.
        let mut lons = [350.0; 9];
        lons[Saturn as usize] = 180.0;
        assert!(close(drik_bala_at(0.0, None, &sky(lons)), -15.0));
        // Yalnızca Venüs bakar: +15. Yalnızca Güneş: −15.
        let mut lons = [350.0; 9];
        lons[Venus as usize] = 180.0;
        assert!(close(drik_bala_at(0.0, None, &sky(lons)), 15.0));
        let mut lons = [350.0; 9];
        lons[Sun as usize] = 180.0;
        assert!(close(drik_bala_at(0.0, None, &sky(lons)), -15.0));
        // Yalnız Merkür (iyicil): 15 + 60 = 75. Merkür Satürn'le aynı burçtaysa kötücüldür:
        // Merkür −15 + 60, Satürn −15 → 30.
        let mut lons = [350.0; 9];
        lons[Mercury as usize] = 180.0;
        assert!(close(drik_bala_at(0.0, None, &sky(lons)), 75.0));
        lons[Saturn as usize] = 180.0;
        assert!(close(drik_bala_at(0.0, None, &sky(lons)), 30.0));
        // Büyüyen Ay iyicil (+15), küçülen Ay kötücül (−15). Güneş 350°'de.
        let mut lons = [350.0; 9];
        lons[Moon as usize] = 120.0; // uzanım 130°: büyüyor; açı 240° → (300 − 240) / 2 = 30
        assert!(close(drik_bala_at(0.0, None, &sky(lons)), 7.5));
        lons[Moon as usize] = 240.0; // uzanım 250°: küçülüyor; açı 120° → 30
        assert!(close(drik_bala_at(0.0, None, &sky(lons)), -7.5));
        // Grahanın kendisi sayılmaz: Jüpiter'e yalnızca kendisi "bakıyorsa" 0.
        let mut lons = [170.0; 9];
        lons[Jupiter as usize] = 180.0;
        assert!(close(drik_bala(Jupiter, &sky(lons)), 0.0));
    }

    #[test]
    fn graha_war_marks_both_sides_and_the_northern_one_wins() {
        // Mars 100°, Venüs 100,6°: savaş. Enlem: Mars +1,0, Venüs +0,5 → Mars kazanır.
        let mut lons = BASE;
        lons[Mars as usize] = 100.0;
        lons[Venus as usize] = 100.6;
        let mut p = sky(lons);
        p.grahas[Mars as usize].lat = 1.0;
        p.grahas[Venus as usize].lat = 0.5;
        let war = Some(GrahaWar {
            with: Venus,
            winner: Mars,
        });
        assert_eq!(graha_war(Mars, &p), war);
        assert_eq!(
            graha_war(Venus, &p),
            Some(GrahaWar {
                with: Mars,
                winner: Mars
            })
        );
        // Güneyde olan kaybeder.
        p.grahas[Mars as usize].lat = -1.0;
        assert_eq!(graha_war(Mars, &p).map(|w| w.winner), Some(Venus));
        // Eşit enlem: `WAR_GRAHAS` sırasında önce gelen (Mars).
        p.grahas[Mars as usize].lat = 0.5;
        assert_eq!(graha_war(Venus, &p).map(|w| w.winner), Some(Mars));
        // Sınır dahil: tam 1° savaş, 1°'den büyük değil.
        let mut lons = BASE;
        lons[Mars as usize] = 100.0;
        lons[Venus as usize] = 101.0;
        assert!(graha_war(Mars, &sky(lons)).is_some());
        lons[Venus as usize] = 101.001;
        assert_eq!(graha_war(Mars, &sky(lons)), None);
        // 0°'nin iki yanı: 359,7° ile 0,2°.
        let mut lons = BASE;
        lons[Jupiter as usize] = 359.7;
        lons[Saturn as usize] = 0.2;
        assert_eq!(graha_war(Saturn, &sky(lons)).map(|w| w.with), Some(Jupiter));
        // Güneş, Ay ve düğümler savaşa girmez, yakın olsalar da.
        let mut lons = BASE;
        lons[Sun as usize] = 100.0;
        lons[Moon as usize] = 100.1;
        lons[Rahu as usize] = 100.2;
        lons[Mars as usize] = 100.3;
        let p = sky(lons);
        for g in [Sun, Moon, Rahu, Ketu, Mars] {
            assert_eq!(graha_war(g, &p), None, "{g:?}");
        }
        // Üç graha 1° içinde: karşıdaki en yakın olandır.
        let mut lons = BASE;
        lons[Mars as usize] = 100.0;
        lons[Mercury as usize] = 100.9;
        lons[Jupiter as usize] = 100.5;
        let p = sky(lons);
        assert_eq!(graha_war(Mars, &p).map(|w| w.with), Some(Jupiter));
        assert_eq!(graha_war(Mercury, &p).map(|w| w.with), Some(Jupiter));
        assert_eq!(graha_war(Jupiter, &p).map(|w| w.with), Some(Mercury));
    }
}
````

- [ ] **Step 5: Sthana bala**

**Yeni dosya:** `crates/jyotish-core/src/shadbala/sthana.rs`

````rust
//! Sthana bala (konum gücü): uccha, saptavargaja, ojayugma, kendradi ve drekkana kalemleri
//! (ileri modüller spec'i §7, "1. Sthana bala"). Kaynak: BPHS, graha gücü bölümü.
//!
//! Burca ve burç içi dereceye dayalı her kural rasi çerçevesindedir (K5); evler lagna
//! burcundan sayılan rasi evleridir (K4). Birim virupadır.

use crate::dignity::{
    compound_relation, debilitation_point, moolatrikona, natural_relation, own_signs, sign_lord,
    temporal_friend, Dignity,
};
use crate::graha::Graha;
use crate::positions::{GrahaPoint, Positions};
use crate::varga::{varga_position, Varga};
use crate::zodiac::{angular_distance, Sign};

use super::SthanaBala;

/// Saptavargaja balada kullanılan yedi varga.
pub const SAPTAVARGA: [Varga; 7] = [
    Varga::D1,
    Varga::D2,
    Varga::D3,
    Varga::D7,
    Varga::D9,
    Varga::D12,
    Varga::D30,
];

/// Grahanın rasi çerçevesindeki boylamı, [0, 360): burç başı + burç içi derece. `Sidereal`
/// kipinde sidereal boylamın, tropikal-rasi kipinde tropikal boylamın kendisidir.
pub fn rasi_lon(point: &GrahaPoint) -> f64 {
    point.rasi_sign.index() as f64 * 30.0 + point.deg_in_sign
}

/// Tek (oja) burç: Koç, İkizler, Aslan, Terazi, Yay, Kova.
fn is_odd_sign(sign: Sign) -> bool {
    sign.index().is_multiple_of(2)
}

/// Uccha bala: düşüş noktasından açısal uzaklık (0°–180°) / 3. Yücelme noktasında 60, düşüş
/// noktasında 0. `rasi_lon` rasi çerçevesindeki boylamdır. Düğümler 0 verir.
pub fn uccha_bala(graha: Graha, rasi_lon: f64) -> f64 {
    match debilitation_point(graha) {
        Some((sign, deg)) => angular_distance(rasi_lon, sign.index() as f64 * 30.0 + deg) / 3.0,
        None => 0.0,
    }
}

/// Bileşik ilişkinin saptavargaja değeri. `compound_relation` yalnızca bu beş değeri üretir.
fn relation_points(relation: Dignity) -> f64 {
    match relation {
        Dignity::GreatFriend => 20.0,
        Dignity::Friend => 15.0,
        Dignity::Neutral => 10.0,
        Dignity::Enemy => 4.0,
        Dignity::GreatEnemy => 2.0,
        Dignity::Exalted | Dignity::Moolatrikona | Dignity::OwnSign | Dignity::Debilitated => 0.0,
    }
}

/// Grahanın tek bir vargadaki saptavargaja değeri: moolatrikona 45 (yalnızca D1; v1'in
/// moolatrikona derece aralığı), kendi burcu 30, sonra varga burcunun lorduyla bileşik ilişki:
/// çok dost 20, dost 15, nötr 10, düşman 4, çok düşman 2. Geçici ilişki her vargada D1
/// burçlarından alınır. Düğümler 0 verir.
pub fn saptavargaja_points(graha: Graha, varga: Varga, positions: &Positions) -> f64 {
    if graha.is_node() {
        return 0.0;
    }
    let point = positions.point(graha);
    let (sign, _) = varga_position(varga, rasi_lon(point));
    if varga == Varga::D1 {
        if let Some((mt_sign, from, to)) = moolatrikona(graha) {
            if sign == mt_sign && point.deg_in_sign >= from && point.deg_in_sign < to {
                return 45.0;
            }
        }
    }
    if own_signs(graha).contains(&sign) {
        return 30.0;
    }
    let lord = sign_lord(sign);
    relation_points(compound_relation(
        natural_relation(graha, lord),
        temporal_friend(point.rasi_sign, positions.sign_of(lord)),
    ))
}

/// Saptavargaja bala: yedi varganın değerlerinin toplamı (14–225).
pub fn saptavargaja_bala(graha: Graha, positions: &Positions) -> f64 {
    SAPTAVARGA
        .into_iter()
        .map(|varga| saptavargaja_points(graha, varga, positions))
        .sum()
}

/// Ojayugma bala: Ay ve Venüs çift burçta, diğer beş graha tek burçta 15; aynı kural navamsa
/// burcu için ayrıca 15 (0, 15 ya da 30). Düğümler 0 verir.
pub fn ojayugma_bala(graha: Graha, rasi_lon: f64) -> f64 {
    if graha.is_node() {
        return 0.0;
    }
    let wants_even = matches!(graha, Graha::Moon | Graha::Venus);
    [Varga::D1, Varga::D9]
        .into_iter()
        .filter(|&varga| is_odd_sign(varga_position(varga, rasi_lon).0) != wants_even)
        .count() as f64
        * 15.0
}

/// Kendradi bala: kendra (1, 4, 7, 10) 60; panaphara (2, 5, 8, 11) 30; apoklima (3, 6, 9, 12)
/// 15. `house` lagnadan sayılan rasi evidir, 1..=12.
pub fn kendradi_bala(house: u8) -> f64 {
    match house % 3 {
        1 => 60.0,
        2 => 30.0,
        _ => 15.0,
    }
}

/// Drekkana bala: eril grahalar (Güneş, Mars, Jüpiter) burcun ilk 10°'sinde, nötrler (Merkür,
/// Satürn) ikinci 10°'de, dişiller (Ay, Venüs) son 10°'de 15; değilse 0. Düğümler 0 verir.
pub fn drekkana_bala(graha: Graha, deg_in_sign: f64) -> f64 {
    let part = match graha {
        Graha::Sun | Graha::Mars | Graha::Jupiter => 0,
        Graha::Mercury | Graha::Saturn => 1,
        Graha::Moon | Graha::Venus => 2,
        Graha::Rahu | Graha::Ketu => return 0.0,
    };
    let third = if deg_in_sign < 10.0 {
        0
    } else if deg_in_sign < 20.0 {
        1
    } else {
        2
    };
    if part == third {
        15.0
    } else {
        0.0
    }
}

/// Beş kalem ve toplamı. Kendradi lagnaya bağlıdır: doğum saati bilinmiyorsa `None`.
pub fn sthana_bala(graha: Graha, positions: &Positions) -> Option<SthanaBala> {
    let house = positions.house_of(graha)?;
    let point = positions.point(graha);
    let lon = rasi_lon(point);
    let uccha = uccha_bala(graha, lon);
    let saptavargaja = saptavargaja_bala(graha, positions);
    let ojayugma = ojayugma_bala(graha, lon);
    let kendradi = kendradi_bala(house);
    let drekkana = drekkana_bala(graha, point.deg_in_sign);
    Some(SthanaBala {
        uccha,
        saptavargaja,
        ojayugma,
        kendradi,
        drekkana,
        total: uccha + saptavargaja + ojayugma + kendradi + drekkana,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use Graha::*;

    /// `Graha::ALL` sırası: Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn, Rahu, Ketu.
    const BASE: [f64; 9] = [10.0, 100.0, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0];

    fn with(changes: &[(Graha, f64)]) -> Positions {
        let mut lons = BASE;
        for &(g, lon) in changes {
            lons[g as usize] = lon;
        }
        Positions::synthetic(Some(0.0), lons)
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn uccha_is_sixty_at_exaltation_and_zero_at_debilitation() {
        // Yücelme noktaları: Güneş Koç 10°, Ay Boğa 3°, Mars Oğlak 28°, Merkür Başak 15°,
        // Jüpiter Yengeç 5°, Venüs Balık 27°, Satürn Terazi 20°.
        for (g, exalt) in [
            (Sun, 10.0),
            (Moon, 33.0),
            (Mars, 298.0),
            (Mercury, 165.0),
            (Jupiter, 95.0),
            (Venus, 357.0),
            (Saturn, 200.0),
        ] {
            assert!(close(uccha_bala(g, exalt), 60.0), "{g:?} exalted");
            assert!(
                close(uccha_bala(g, exalt + 180.0), 0.0),
                "{g:?} debilitated"
            );
            // Çeyrek uzaklıkta yarım değer; 30° ötede 50 ya da 10.
            assert!(close(uccha_bala(g, exalt + 90.0), 30.0), "{g:?} quarter");
            assert!(close(uccha_bala(g, exalt - 90.0), 30.0), "{g:?} quarter");
            assert!(close(uccha_bala(g, exalt + 30.0), 50.0), "{g:?} thirty");
            assert!(close(uccha_bala(g, exalt + 150.0), 10.0), "{g:?} near fall");
        }
        // Ara değer: Güneş Aslan 15° (135°); düşüş noktası 190°; uzaklık 55° → 18,333…
        assert!(close(uccha_bala(Sun, 135.0), 55.0 / 3.0));
        assert_eq!(uccha_bala(Rahu, 100.0), 0.0);
        assert_eq!(uccha_bala(Ketu, 100.0), 0.0);
    }

    #[test]
    fn saptavargaja_points_by_dignity_and_compound_relation() {
        // Moolatrikona yalnızca D1'de ve derece aralığında: Güneş Aslan 0–20°.
        let p = with(&[(Sun, 130.0)]);
        assert_eq!(saptavargaja_points(Sun, Varga::D1, &p), 45.0);
        // Aslan 25°: aralığın dışı, kendi burcu.
        let p = with(&[(Sun, 145.0)]);
        assert_eq!(saptavargaja_points(Sun, Varga::D1, &p), 30.0);
        // Ay Boğa 2°: moolatrikona aralığından (3–30°) önce; Boğa'nın lordu Venüs. Doğal: Ay →
        // Venüs nötr. Geçici: Venüs Boğa'da, Ay'la aynı burçta (1. burç) → düşman. Nötr +
        // düşman = düşman: 4.
        let p = with(&[(Moon, 32.0)]);
        assert_eq!(saptavargaja_points(Moon, Varga::D1, &p), 4.0);
        // Ay Boğa 10°: moolatrikona.
        let p = with(&[(Moon, 40.0)]);
        assert_eq!(saptavargaja_points(Moon, Varga::D1, &p), 45.0);
        // Mars Akrep'te (220°): kendi burcu, moolatrikona değil.
        assert_eq!(saptavargaja_points(Mars, Varga::D1, &with(&[])), 30.0);

        // Bileşik ilişkinin beş değeri, D1'de. Taban: Güneş Koç 10° (lordu Mars, Akrep'te).
        // Doğal: Güneş → Mars dost. Geçici: Akrep, Koç'tan 8. burç → düşman. Dost + düşman =
        // nötr: 10.
        assert_eq!(saptavargaja_points(Sun, Varga::D1, &with(&[])), 10.0);
        // Mars Boğa'ya (2. burç) alınırsa geçici dost olur: dost + dost = çok dost: 20.
        assert_eq!(
            saptavargaja_points(Sun, Varga::D1, &with(&[(Mars, 40.0)])),
            20.0
        );
        // Merkür İkizler 5° (65°) kendi burcundadır; Yengeç 5°'ye (95°) alınırsa lord Ay'dır
        // (Yengeç 10°'de, aynı burçta): doğal Merkür → Ay düşman, geçici düşman → çok düşman: 2.
        assert_eq!(
            saptavargaja_points(Mercury, Varga::D1, &with(&[(Mercury, 95.0)])),
            2.0
        );
        // Venüs Yay'da (250°): lord Jüpiter de Yay'da (aynı burç → geçici düşman). Doğal Venüs
        // → Jüpiter nötr: düşman: 4.
        assert_eq!(
            saptavargaja_points(Venus, Varga::D1, &with(&[(Venus, 255.0)])),
            4.0
        );
        // Venüs Oğlak'ta (275°): lord Satürn Kova'da (2. burç → geçici dost). Doğal Venüs →
        // Satürn dost: çok dost: 20.
        assert_eq!(
            saptavargaja_points(Venus, Varga::D1, &with(&[(Venus, 275.0)])),
            20.0
        );
        // Jüpiter Boğa'da (45°): lord Venüs de Boğa'da (40°) → geçici düşman; doğal Jüpiter →
        // Venüs düşman: çok düşman: 2. Venüs İkizler'e (70°, 2. burç) alınırsa nötr: 10.
        assert_eq!(
            saptavargaja_points(Jupiter, Varga::D1, &with(&[(Jupiter, 45.0)])),
            2.0
        );
        assert_eq!(
            saptavargaja_points(Jupiter, Varga::D1, &with(&[(Jupiter, 45.0), (Venus, 70.0)])),
            10.0
        );
        // Mars Yengeç'te (95°): lord Ay Yengeç'te (aynı burç → geçici düşman); doğal Mars → Ay
        // dost: nötr 10. Ay Aslan'a (125°, 2. burç) alınırsa çok dost: 20.
        assert_eq!(
            saptavargaja_points(Mars, Varga::D1, &with(&[(Mars, 95.0)])),
            10.0
        );
        assert_eq!(
            saptavargaja_points(Mars, Varga::D1, &with(&[(Mars, 95.0), (Moon, 125.0)])),
            20.0
        );
        // Satürn Yay'da (245°): lord Jüpiter Yay'da → geçici düşman; doğal Satürn → Jüpiter
        // nötr: düşman 4. Jüpiter Oğlak'a (275°) alınırsa dost: 15.
        assert_eq!(
            saptavargaja_points(Saturn, Varga::D1, &with(&[(Saturn, 245.0)])),
            4.0
        );
        assert_eq!(
            saptavargaja_points(
                Saturn,
                Varga::D1,
                &with(&[(Saturn, 245.0), (Jupiter, 275.0)])
            ),
            15.0
        );
        assert_eq!(saptavargaja_points(Rahu, Varga::D1, &with(&[])), 0.0);
    }

    #[test]
    fn saptavargaja_uses_the_varga_sign_and_the_d1_temporal_relation() {
        let p = with(&[]);
        // Güneş Koç 10°: hora (D2) tek burcun ilk yarısı → Aslan: kendi burcu 30; D2'de
        // moolatrikona sayılmaz.
        assert_eq!(saptavargaja_points(Sun, Varga::D2, &p), 30.0);
        // Güneş Koç 10°: navamsa 4. parça (10°–13°20′) → Koç'tan 4.: Yengeç; lord Ay (D1'de
        // Yengeç, Koç'tan 4. burç → geçici dost). Doğal Güneş → Ay dost: çok dost 20.
        assert_eq!(saptavargaja_points(Sun, Varga::D9, &p), 20.0);
        // Güneş Koç 10°: D3 ikinci parça → Koç'tan 5.: Aslan: kendi burcu 30.
        assert_eq!(saptavargaja_points(Sun, Varga::D3, &p), 30.0);
        // Güneş Koç 10°: D12 5. parça (10°–12°30′) → Aslan: 30.
        assert_eq!(saptavargaja_points(Sun, Varga::D12, &p), 30.0);
        // Güneş Koç 10°: D7 3. parça (8°34′–12°51′) → tek burçta kendinden: İkizler; lord Merkür
        // (D1'de İkizler, Koç'tan 3. burç → geçici dost). Doğal Güneş → Merkür nötr: dost 15.
        assert_eq!(saptavargaja_points(Sun, Varga::D7, &p), 15.0);
        // Güneş Koç 10°: trimsamsa, tek burç 10°–18° → Yay; lord Jüpiter (D1'de Yay, Koç'tan
        // 9. burç → geçici düşman). Doğal dost: nötr 10.
        assert_eq!(saptavargaja_points(Sun, Varga::D30, &p), 10.0);
        // Toplam: D1 10 + D2 30 + D3 30 + D7 15 + D9 20 + D12 30 + D30 10 = 145.
        assert_eq!(saptavargaja_bala(Sun, &p), 145.0);
        assert_eq!(saptavargaja_bala(Ketu, &p), 0.0);
    }

    #[test]
    fn ojayugma_counts_the_sign_and_the_navamsa_sign() {
        // Koç 1° (tek burç; navamsa Koç, tek): eriller 30, Ay ve Venüs 0.
        for g in [Sun, Mars, Mercury, Jupiter, Saturn] {
            assert_eq!(ojayugma_bala(g, 1.0), 30.0, "{g:?}");
        }
        for g in [Moon, Venus] {
            assert_eq!(ojayugma_bala(g, 1.0), 0.0, "{g:?}");
        }
        // Koç 5° (tek burç; navamsa 2. parça: Boğa, çift): herkes 15.
        for g in [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn] {
            assert_eq!(ojayugma_bala(g, 5.0), 15.0, "{g:?}");
        }
        // Boğa 1° (çift burç; navamsa Oğlak, çift): Ay ve Venüs 30, diğerleri 0.
        for g in [Moon, Venus] {
            assert_eq!(ojayugma_bala(g, 31.0), 30.0, "{g:?}");
        }
        for g in [Sun, Mars, Mercury, Jupiter, Saturn] {
            assert_eq!(ojayugma_bala(g, 31.0), 0.0, "{g:?}");
        }
        // Boğa 5° (çift burç; navamsa 2. parça: Kova, tek): herkes 15.
        for g in [Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn] {
            assert_eq!(ojayugma_bala(g, 35.0), 15.0, "{g:?}");
        }
        assert_eq!(ojayugma_bala(Rahu, 1.0), 0.0);
    }

    #[test]
    fn kendradi_by_house_group() {
        let got: Vec<f64> = (1..=12).map(kendradi_bala).collect();
        assert_eq!(
            got,
            [60.0, 30.0, 15.0, 60.0, 30.0, 15.0, 60.0, 30.0, 15.0, 60.0, 30.0, 15.0]
        );
    }

    #[test]
    fn drekkana_by_gender_and_third_of_the_sign() {
        // (burç içi derece, eril, nötr, dişil)
        for (deg, male, neuter, female) in [
            (0.0, 15.0, 0.0, 0.0),
            (9.999, 15.0, 0.0, 0.0),
            (10.0, 0.0, 15.0, 0.0),
            (19.999, 0.0, 15.0, 0.0),
            (20.0, 0.0, 0.0, 15.0),
            (29.999, 0.0, 0.0, 15.0),
        ] {
            for g in [Sun, Mars, Jupiter] {
                assert_eq!(drekkana_bala(g, deg), male, "{g:?} at {deg}");
            }
            for g in [Mercury, Saturn] {
                assert_eq!(drekkana_bala(g, deg), neuter, "{g:?} at {deg}");
            }
            for g in [Moon, Venus] {
                assert_eq!(drekkana_bala(g, deg), female, "{g:?} at {deg}");
            }
        }
        assert_eq!(drekkana_bala(Rahu, 5.0), 0.0);
    }

    #[test]
    fn sthana_sums_the_five_items_and_needs_a_lagna() {
        let p = with(&[]);
        // Güneş Koç 10°, lagna Koç: uccha 60, saptavargaja 145, ojayugma (Koç tek; navamsa
        // Yengeç çift) 15, kendradi (1. ev) 60, drekkana (ikinci 10°: eril için yok) 0.
        let s = sthana_bala(Sun, &p).expect("lagna");
        assert_eq!(
            (s.uccha, s.saptavargaja, s.ojayugma, s.kendradi, s.drekkana),
            (60.0, 145.0, 15.0, 60.0, 0.0)
        );
        assert_eq!(s.total, 280.0);
        assert_eq!(sthana_bala(Sun, &p.without_lagna()), None);
    }

    #[test]
    fn rasi_longitude_rebuilds_the_synthetic_longitude_exactly() {
        let p = with(&[]);
        for (g, lon) in Graha::ALL.into_iter().zip(BASE) {
            assert_eq!(rasi_lon(p.point(g)), lon, "{g:?}");
        }
    }
}
````

- [ ] **Step 6: Zaman gücü**

**Yeni dosya:** `crates/jyotish-core/src/shadbala/temporal.rs`

````rust
//! Zaman gücü (ileri modüller spec'i §7, "3. Zaman gücü"; Ek A: bu güç yalnızca `temporal`
//! adıyla anılır): gündüz–gece gücü, paksha bala, tribhaga bala, yıl, ay, gün ve saat lordu,
//! ayana bala. Kaynak: BPHS, graha gücü bölümü.
//!
//! Fonksiyonlar ephemeris görmez: gün doğumu, batımı ve giriş varaları `ShadbalaAstro` ile
//! verilir (K3). Zamanlar UT Jülyen günüdür; birim virupadır.

use crate::graha::Graha;
use crate::nature::{nature, Nature};
use crate::panchang::Vara;
use crate::positions::Positions;
use crate::zodiac::angular_distance;

use super::{ShadbalaAstro, TemporalBala, TimeLords, SHADBALA_GRAHAS};

/// Ekliptiğin eğimi, ayana bala için: 23°27′.
pub const AYANA_OBLIQUITY_DEG: f64 = 23.45;
/// Yıl, ay, gün ve saat lordunun aldığı değerler.
pub const YEAR_LORD_BALA: f64 = 15.0;
pub const MONTH_LORD_BALA: f64 = 30.0;
pub const WEEKDAY_LORD_BALA: f64 = 45.0;
pub const HORA_LORD_BALA: f64 = 60.0;

/// Saat lordlarının döngüsü: yörünge hızına göre yavaştan hızlıya.
pub const HORA_CYCLE: [Graha; 7] = [
    Graha::Saturn,
    Graha::Jupiter,
    Graha::Mars,
    Graha::Sun,
    Graha::Venus,
    Graha::Mercury,
    Graha::Moon,
];
/// Gündüzün üç diliminin lordları (doğuş → batış).
pub const DAY_TRIBHAGA: [Graha; 3] = [Graha::Mercury, Graha::Sun, Graha::Saturn];
/// Gecenin üç diliminin lordları (batış → sonraki doğuş).
pub const NIGHT_TRIBHAGA: [Graha; 3] = [Graha::Moon, Graha::Venus, Graha::Mars];

/// Varanın lordu: Pazar Güneş, Pazartesi Ay, Salı Mars, Çarşamba Merkür, Perşembe Jüpiter,
/// Cuma Venüs, Cumartesi Satürn.
pub fn vara_lord(vara: Vara) -> Graha {
    match vara {
        Vara::Sunday => Graha::Sun,
        Vara::Monday => Graha::Moon,
        Vara::Tuesday => Graha::Mars,
        Vara::Wednesday => Graha::Mercury,
        Vara::Thursday => Graha::Jupiter,
        Vara::Friday => Graha::Venus,
        Vara::Saturday => Graha::Saturn,
    }
}

/// Doğum anının en yakın görünür öğleye uzaklığı, saat, [0, 12]. Görünür öğle, günün gün
/// doğumu ile batımının orta noktasıdır; ardışık öğleler 24 saat arayla sayılır (12 saatten
/// büyük uzaklık 24'ten çıkarılır).
pub fn hours_from_noon(astro: &ShadbalaAstro) -> f64 {
    let noon = 0.5 * (astro.sunrise_jd_ut + astro.sunset_jd_ut);
    let m = ((astro.jd_ut - noon) * 24.0).abs().rem_euclid(24.0);
    if m > 12.0 {
        24.0 - m
    } else {
        m
    }
}

/// Gündüz–gece gücü (`day_night`). `m`: öğleye uzaklık, saat (0–12). Güneş, Jüpiter ve Venüs
/// öğlede 60, gece yarısında 0; Ay, Mars ve Satürn tersine; Merkür her zaman 60. Düğümler 0.
pub fn day_night_bala(graha: Graha, m: f64) -> f64 {
    match graha {
        Graha::Sun | Graha::Jupiter | Graha::Venus => 60.0 * (1.0 - m / 12.0),
        Graha::Moon | Graha::Mars | Graha::Saturn => 60.0 * m / 12.0,
        Graha::Mercury => 60.0,
        Graha::Rahu | Graha::Ketu => 0.0,
    }
}

/// Ay'ın Güneş'ten uzanımı, 0°–180°'ye indirgenmiş (yeni ayda 0, dolunayda 180).
pub fn paksha_elongation(positions: &Positions) -> f64 {
    angular_distance(
        positions.point(Graha::Moon).sidereal_lon,
        positions.point(Graha::Sun).sidereal_lon,
    )
}

/// Paksha bala. `e` = `paksha_elongation`. Ay: `2 × e / 3`; diğerleri doğal sınıfına göre
/// (K6): iyicil `e / 3`, kötücül `(180 − e) / 3`. Düğümler 0.
pub fn paksha_bala(graha: Graha, positions: &Positions) -> f64 {
    let e = paksha_elongation(positions);
    match graha {
        Graha::Rahu | Graha::Ketu => 0.0,
        Graha::Moon => 2.0 * e / 3.0,
        _ => match nature(graha, positions) {
            Nature::Benefic => e / 3.0,
            Nature::Malefic => (180.0 - e) / 3.0,
        },
    }
}

/// Doğum gündüz mü: gün doğumundan (dahil) gün batımına (hariç).
pub fn is_daytime(astro: &ShadbalaAstro) -> bool {
    astro.jd_ut < astro.sunset_jd_ut
}

/// `[start, end)` aralığının `parts` eşit diliminden `t` anını içerenin sırası, 0'dan.
fn part_index(t: f64, start: f64, end: f64, parts: usize) -> usize {
    let x = (t - start) / (end - start) * parts as f64;
    // NaN ve negatifler 0'a, aralığın sonu son dilime düşer.
    (x.max(0.0) as usize).min(parts - 1)
}

/// İçinde bulunulan tribhaga diliminin lordu: gündüz Merkür, Güneş, Satürn; gece Ay, Venüs,
/// Mars.
pub fn tribhaga_lord(astro: &ShadbalaAstro) -> Graha {
    if is_daytime(astro) {
        DAY_TRIBHAGA[part_index(astro.jd_ut, astro.sunrise_jd_ut, astro.sunset_jd_ut, 3)]
    } else {
        NIGHT_TRIBHAGA[part_index(astro.jd_ut, astro.sunset_jd_ut, astro.next_sunrise_jd_ut, 3)]
    }
}

/// Doğum anını içeren saat diliminin (hora) sırası, 0..=23: gün doğumundan sonraki gün
/// doğumuna 24 eşit dilim.
pub fn hora_index(astro: &ShadbalaAstro) -> u8 {
    part_index(
        astro.jd_ut,
        astro.sunrise_jd_ut,
        astro.next_sunrise_jd_ut,
        24,
    ) as u8
}

/// `index`. saat diliminin lordu: ilk dilim gün lordunundur, sonrakiler `HORA_CYCLE`
/// sırasında ilerler.
pub fn hora_lord(weekday_lord: Graha, index: u8) -> Graha {
    let start = HORA_CYCLE
        .iter()
        .position(|g| *g == weekday_lord)
        .unwrap_or(0);
    HORA_CYCLE[(start + usize::from(index)) % 7]
}

/// Beş zaman lordu.
pub fn time_lords(astro: &ShadbalaAstro) -> TimeLords {
    let weekday = vara_lord(astro.weekday);
    TimeLords {
        year: vara_lord(astro.year_entry_weekday),
        month: vara_lord(astro.month_entry_weekday),
        weekday,
        hora: hora_lord(weekday, hora_index(astro)),
        tribhaga: tribhaga_lord(astro),
    }
}

/// Tropikal boylamın ekliptik üzerindeki dik açıklığı, derece: `asin(sin 23°27′ × sin λ)`.
pub fn ayana_declination(tropical_lon: f64) -> f64 {
    (AYANA_OBLIQUITY_DEG.to_radians().sin() * tropical_lon.to_radians().sin())
        .asin()
        .to_degrees()
}

/// Ayana bala (Güneş'te iki katı alınmamış hâli). `δ` = `ayana_declination`. Güneş, Mars,
/// Jüpiter, Venüs: `(24 + δ) / 48 × 60`; Ay, Satürn: `(24 − δ) / 48 × 60`; Merkür:
/// `(24 + |δ|) / 48 × 60`. Düğümler 0.
pub fn ayana_bala(graha: Graha, tropical_lon: f64) -> f64 {
    let d = ayana_declination(tropical_lon);
    let signed = match graha {
        Graha::Sun | Graha::Mars | Graha::Jupiter | Graha::Venus => d,
        Graha::Moon | Graha::Saturn => -d,
        Graha::Mercury => d.abs(),
        Graha::Rahu | Graha::Ketu => return 0.0,
    };
    (24.0 + signed) / 48.0 * 60.0
}

fn lord_bala(graha: Graha, lord: Graha, value: f64) -> f64 {
    if graha == lord {
        value
    } else {
        0.0
    }
}

/// Bir grahanın zaman gücü kalemleri ve toplamı. Güneş'in ayana balası iki katıdır; Jüpiter
/// tribhaga balayı her zaman alır.
pub fn temporal_bala(
    graha: Graha,
    positions: &Positions,
    astro: &ShadbalaAstro,
    lords: &TimeLords,
) -> TemporalBala {
    // `SHADBALA_GRAHAS`, `Graha::ALL`'un ilk yedisidir: sıra numarası aynıdır.
    debug_assert_eq!(SHADBALA_GRAHAS.get(graha as usize), Some(&graha));
    let day_night = day_night_bala(graha, hours_from_noon(astro));
    let paksha = paksha_bala(graha, positions);
    let tribhaga = if graha == Graha::Jupiter || graha == lords.tribhaga {
        60.0
    } else {
        0.0
    };
    let year_lord = lord_bala(graha, lords.year, YEAR_LORD_BALA);
    let month_lord = lord_bala(graha, lords.month, MONTH_LORD_BALA);
    let weekday_lord = lord_bala(graha, lords.weekday, WEEKDAY_LORD_BALA);
    let hora_lord = lord_bala(graha, lords.hora, HORA_LORD_BALA);
    let tropical = astro
        .tropical_lons
        .get(graha as usize)
        .copied()
        .unwrap_or(0.0);
    let ayana = ayana_bala(graha, tropical) * if graha == Graha::Sun { 2.0 } else { 1.0 };
    TemporalBala {
        day_night,
        paksha,
        tribhaga,
        year_lord,
        month_lord,
        weekday_lord,
        hora_lord,
        ayana,
        total: day_night
            + paksha
            + tribhaga
            + year_lord
            + month_lord
            + weekday_lord
            + hora_lord
            + ayana,
    }
}

#[cfg(test)]
mod tests {
    use super::super::MeanLongitudes;
    use super::*;
    use Graha::*;

    /// Gün doğumu 06:00, batımı 18:00, sonraki doğuş 06:00 (JD kesirleri: 0,25 / 0,75 / 1,25).
    fn astro_at(jd_ut: f64) -> ShadbalaAstro {
        ShadbalaAstro {
            jd_ut,
            sunrise_jd_ut: 0.25,
            sunset_jd_ut: 0.75,
            next_sunrise_jd_ut: 1.25,
            mc_lon: 270.0,
            tropical_lons: [0.0; 7],
            mean: MeanLongitudes {
                sun: 0.0,
                mercury: 0.0,
                venus: 0.0,
                mars: 0.0,
                jupiter: 0.0,
                saturn: 0.0,
            },
            weekday: Vara::Sunday,
            year_entry_weekday: Vara::Monday,
            month_entry_weekday: Vara::Tuesday,
        }
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn hours_from_noon_fold_into_twelve() {
        // (an, öğleye uzaklık): öğle 0,5.
        for (jd, m) in [
            (0.5, 0.0),
            (0.25, 6.0),
            (0.75, 6.0),
            (1.0, 12.0),
            (0.625, 3.0),
            // 03:00 (sonraki sabaha karşı): öğleden 15 saat sonra; en yakın öğleye 9 saat.
            (1.125, 9.0),
            (1.249, 24.0 - 0.749 * 24.0),
        ] {
            assert!(close(hours_from_noon(&astro_at(jd)), m), "{jd}");
        }
        // Eşit olmayan gün: doğuş 0,30, batış 0,80 → öğle 0,55.
        let mut a = astro_at(0.55);
        a.sunrise_jd_ut = 0.30;
        a.sunset_jd_ut = 0.80;
        assert!(close(hours_from_noon(&a), 0.0));
    }

    #[test]
    fn day_night_strength_at_noon_midnight_and_between() {
        // Öğle: gündüz güçlüler 60, gece güçlüler 0.
        for g in [Sun, Jupiter, Venus] {
            assert_eq!(day_night_bala(g, 0.0), 60.0, "{g:?}");
            assert_eq!(day_night_bala(g, 12.0), 0.0, "{g:?}");
            assert_eq!(day_night_bala(g, 6.0), 30.0, "{g:?}");
            assert_eq!(day_night_bala(g, 3.0), 45.0, "{g:?}");
        }
        for g in [Moon, Mars, Saturn] {
            assert_eq!(day_night_bala(g, 0.0), 0.0, "{g:?}");
            assert_eq!(day_night_bala(g, 12.0), 60.0, "{g:?}");
            assert_eq!(day_night_bala(g, 6.0), 30.0, "{g:?}");
            assert_eq!(day_night_bala(g, 3.0), 15.0, "{g:?}");
        }
        for m in [0.0, 3.0, 6.0, 12.0] {
            assert_eq!(day_night_bala(Mercury, m), 60.0);
            assert_eq!(day_night_bala(Rahu, m), 0.0);
        }
    }

    /// Güneş 10°'de; Ay `moon`'da; Merkür yalnız (iyicil).
    fn sky(moon: f64) -> Positions {
        Positions::synthetic(
            Some(0.0),
            [10.0, moon, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0],
        )
    }

    #[test]
    fn paksha_strength_at_new_moon_full_moon_and_quarters() {
        // Dolunay (uzanım 180°): Ay 120, iyiciller 60, kötücüller 0.
        let full = sky(190.0);
        assert!(close(paksha_elongation(&full), 180.0));
        assert!(close(paksha_bala(Moon, &full), 120.0));
        for g in [Mercury, Jupiter, Venus] {
            assert!(close(paksha_bala(g, &full), 60.0), "{g:?}");
        }
        for g in [Sun, Mars, Saturn] {
            assert!(close(paksha_bala(g, &full), 0.0), "{g:?}");
        }
        // Yeni ay (uzanım 0°): Ay 0, iyiciller 0, kötücüller 60.
        let new = sky(10.0);
        assert!(close(paksha_bala(Moon, &new), 0.0));
        assert!(close(paksha_bala(Jupiter, &new), 0.0));
        assert!(close(paksha_bala(Saturn, &new), 60.0));
        // Büyüyen çeyrek (90°) ve küçülen çeyrek (270° → 90°): Ay 60, herkes 30.
        for moon in [100.0, 280.0] {
            let quarter = sky(moon);
            assert!(close(paksha_elongation(&quarter), 90.0));
            assert!(close(paksha_bala(Moon, &quarter), 60.0));
            assert!(close(paksha_bala(Venus, &quarter), 30.0));
            assert!(close(paksha_bala(Mars, &quarter), 30.0));
        }
        // Ara değer: uzanım 60° → Ay 40, iyicil 20, kötücül 40.
        let sixty = sky(70.0);
        assert!(close(paksha_bala(Moon, &sixty), 40.0));
        assert!(close(paksha_bala(Jupiter, &sixty), 20.0));
        assert!(close(paksha_bala(Sun, &sixty), 40.0));
        // Merkür kötücülle aynı burçta (ikisi de Aslan'da; uzanım 60°): kötücül sayılır, 40.
        // Satürn Kova'ya alınınca Merkür yalnızdır: iyicil, 20.
        let mut lons = [10.0, 70.0, 220.0, 125.0, 250.0, 40.0, 130.0, 200.0, 310.0];
        assert!(close(
            paksha_bala(Mercury, &Positions::synthetic(Some(0.0), lons)),
            40.0
        ));
        lons[6] = 300.0;
        assert!(close(
            paksha_bala(Mercury, &Positions::synthetic(Some(0.0), lons)),
            20.0
        ));
        assert_eq!(paksha_bala(Rahu, &full), 0.0);
    }

    #[test]
    fn tribhaga_lords_of_the_six_parts() {
        // Gündüz 06:00–18:00: 4'er saatlik dilimler; gece 18:00–06:00: 4'er saatlik dilimler.
        for (jd, lord) in [
            (0.25, Mercury), // 06:00
            (0.25 + 3.9 / 24.0, Mercury),
            (0.25 + 4.01 / 24.0, Sun), // 10:00'dan hemen sonra
            (0.5, Sun),
            (0.25 + 8.01 / 24.0, Saturn), // 14:00'ten hemen sonra
            (0.749, Saturn),
            (0.75, Moon), // 18:00: gece başlar
            (0.75 + 3.9 / 24.0, Moon),
            (0.75 + 4.01 / 24.0, Venus), // 22:00'den hemen sonra
            (1.0, Venus),
            (0.75 + 8.01 / 24.0, Mars), // 02:00'den hemen sonra
            (1.249, Mars),
        ] {
            let a = astro_at(jd);
            assert_eq!(tribhaga_lord(&a), lord, "{jd}");
            assert_eq!(is_daytime(&a), jd < 0.75, "{jd}");
        }
    }

    /// Elle yazılmış saat lordu tablosu (bağımsız ikinci yazım): her varanın 24 saat dilimi.
    const HAND_HORAS: [(Vara, [Graha; 24]); 7] = [
        (
            Vara::Sunday,
            [
                Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon,
                Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun,
                Venus, Mercury,
            ],
        ),
        (
            Vara::Monday,
            [
                Moon, Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun,
                Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn,
                Jupiter,
            ],
        ),
        (
            Vara::Tuesday,
            [
                Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon,
                Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun,
                Venus,
            ],
        ),
        (
            Vara::Wednesday,
            [
                Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter,
                Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon,
                Saturn,
            ],
        ),
        (
            Vara::Thursday,
            [
                Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus,
                Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter,
                Mars, Sun,
            ],
        ),
        (
            Vara::Friday,
            [
                Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn,
                Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus,
                Mercury, Moon,
            ],
        ),
        (
            Vara::Saturday,
            [
                Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun,
                Venus, Mercury, Moon, Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon, Saturn,
                Jupiter, Mars,
            ],
        ),
    ];

    #[test]
    fn hora_lords_match_the_hand_table() {
        for (i, (vara, row)) in HAND_HORAS.into_iter().enumerate() {
            let day_lord = vara_lord(vara);
            assert_eq!(
                row[0], day_lord,
                "{vara:?}: the first hora is the day lord's"
            );
            for (k, want) in row.into_iter().enumerate() {
                assert_eq!(hora_lord(day_lord, k as u8), want, "{vara:?} hora {k}");
            }
            // 25. dilim sonraki günün ilk dilimidir: sonraki varanın lordu.
            let next = HAND_HORAS[(i + 1) % 7].0;
            assert_eq!(hora_lord(day_lord, 24), vara_lord(next), "{vara:?}");
        }
        // Dilim sırası: 06:00'dan saat başına bir dilim.
        for (jd, index) in [
            (0.25, 0),
            (0.25 + 0.99 / 24.0, 0),
            (0.25 + 1.0 / 24.0 + 1e-9, 1),
            (0.5, 6),
            (0.75, 12),
            (1.0, 18),
            (1.249, 23),
        ] {
            assert_eq!(hora_index(&astro_at(jd)), index, "{jd}");
        }
        // Doğuşlar arası 24 saat değilse dilimler de eşit uzar: 25 saatlik günde ortası 12.
        let mut long = astro_at(0.25 + 12.5 / 24.0);
        long.next_sunrise_jd_ut = 0.25 + 25.0 / 24.0;
        assert_eq!(hora_index(&long), 12);
    }

    #[test]
    fn time_lords_come_from_the_three_weekdays_and_the_clock() {
        // Pazar, öğle (7. saat dilimi, sıra 6): Pazar tablosunda Mars; tribhaga Güneş.
        let lords = time_lords(&astro_at(0.5));
        assert_eq!(
            lords,
            TimeLords {
                year: Moon,
                month: Mars,
                weekday: Sun,
                hora: Mars,
                tribhaga: Sun
            }
        );
    }

    #[test]
    fn ayana_strength_at_the_solstices_and_equinoxes() {
        // Ekinokslarda δ = 0: herkes 30. Gündönümlerinde δ = ±23,45.
        let north = (24.0 + 23.45) / 48.0 * 60.0; // 59,3125
        let south = (24.0 - 23.45) / 48.0 * 60.0; // 0,6875
        assert!(close(north, 59.3125) && close(south, 0.6875));
        assert!(close(ayana_declination(90.0), 23.45));
        assert!(close(ayana_declination(270.0), -23.45));
        assert!(ayana_declination(0.0).abs() < 1e-12);
        for g in [Sun, Mars, Jupiter, Venus] {
            assert!(close(ayana_bala(g, 0.0), 30.0), "{g:?}");
            assert!(close(ayana_bala(g, 180.0), 30.0), "{g:?}");
            assert!(close(ayana_bala(g, 90.0), north), "{g:?}");
            assert!(close(ayana_bala(g, 270.0), south), "{g:?}");
        }
        for g in [Moon, Saturn] {
            assert!(close(ayana_bala(g, 0.0), 30.0), "{g:?}");
            assert!(close(ayana_bala(g, 90.0), south), "{g:?}");
            assert!(close(ayana_bala(g, 270.0), north), "{g:?}");
        }
        // Merkür her iki yarıda da kuzey gibi sayılır.
        assert!(close(ayana_bala(Mercury, 0.0), 30.0));
        assert!(close(ayana_bala(Mercury, 90.0), north));
        assert!(close(ayana_bala(Mercury, 270.0), north));
        // Ara değer: λ = 30° → sin δ = sin 23,45° × 0,5 = 0,198974 → δ = 11,4770°
        let d = ayana_declination(30.0);
        assert!((d - 11.4770).abs() < 1e-4, "{d}");
        assert!((ayana_bala(Sun, 30.0) - (24.0 + d) / 48.0 * 60.0).abs() < 1e-12);
        assert_eq!(ayana_bala(Ketu, 90.0), 0.0);
    }

    #[test]
    fn temporal_sums_the_eight_items_and_doubles_the_ayana_of_the_sun() {
        // Pazar öğle; Güneş tropikal 90°'de; dolunay.
        let mut a = astro_at(0.5);
        a.tropical_lons = [90.0, 270.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let p = sky(190.0);
        let lords = time_lords(&a);
        let sun = temporal_bala(Sun, &p, &a, &lords);
        // Gündüz–gece 60; paksha (kötücül, dolunay) 0; tribhaga (Güneş dilimi) 60; gün lordu
        // 45; ayana 2 × 59,3125.
        assert_eq!(
            (sun.day_night, sun.paksha, sun.tribhaga, sun.weekday_lord),
            (60.0, 0.0, 60.0, 45.0)
        );
        assert_eq!(
            (sun.year_lord, sun.month_lord, sun.hora_lord),
            (0.0, 0.0, 0.0)
        );
        assert!(close(sun.ayana, 118.625));
        assert!(close(sun.total, 60.0 + 60.0 + 45.0 + 118.625));
        // Ay: yıl lordu 15; paksha 120; gündüz–gece 0; ayana (güney, Ay için güçlü) 59,3125.
        let moon = temporal_bala(Moon, &p, &a, &lords);
        assert_eq!(
            (moon.year_lord, moon.tribhaga, moon.day_night),
            (15.0, 0.0, 0.0)
        );
        assert!(close(moon.paksha, 120.0) && close(moon.ayana, 59.3125));
        assert!(close(moon.total, 15.0 + 120.0 + 59.3125));
        // Mars: ay lordu 30 ve saat lordu 60; ayana 30; paksha 0; gündüz–gece 0.
        let mars = temporal_bala(Mars, &p, &a, &lords);
        assert_eq!((mars.month_lord, mars.hora_lord), (30.0, 60.0));
        assert!(close(mars.total, 30.0 + 60.0 + 30.0));
        // Jüpiter tribhaga balayı her zaman alır.
        let jupiter = temporal_bala(Jupiter, &p, &a, &lords);
        assert_eq!(jupiter.tribhaga, 60.0);
        assert!(close(jupiter.total, 60.0 + 60.0 + 60.0 + 30.0));
    }
}
````

- [ ] **Step 7: Ortalama boylamlar ve cheshta bala**

**Yeni dosya:** `crates/jyotish-core/src/shadbala/cheshta.rs`

````rust
//! Cheshta bala (hareket gücü) ve ortalama boylamlar (ileri modüller spec'i §7, "4. Cheshta
//! bala"). Kaynak: BPHS, graha gücü bölümü; hesap adımlarında yaygın hesap geleneği.
//!
//! Ortalama boylamlar J2000 çağlı doğrusal ortalama öğelerdir (`L0 + L1 × T`; tropikal,
//! tarihin ekinoksu). Katsayılar: J. Meeus, *Astronomical Algorithms* (2. baskı), Tablo 31.A
//! (VSOP87 ortalama öğeleri, tarihin ortalama ekinoksu), yalnızca sabit ve doğrusal terim.
//! Gerçek boylam tropikal jeosantrik boylamdır (`ShadbalaAstro::tropical_lons`).

use serde::{Deserialize, Serialize};

use crate::graha::Graha;
use crate::positions::Positions;
use crate::zodiac::normalize;

use super::temporal::{ayana_bala, paksha_elongation};
use super::ShadbalaAstro;

/// J2000.0 (JD 2451545,0) ve bir Jülyen yüzyılı, gün.
const J2000: f64 = 2_451_545.0;
const JULIAN_CENTURY_DAYS: f64 = 36_525.0;

/// Ortalama boylam öğeleri: (J2000'deki değer, yüzyıl başına ilerleme), derece.
pub const MEAN_SUN: (f64, f64) = (280.466_457, 36_000.769_827_8);
pub const MEAN_MERCURY: (f64, f64) = (252.250_906, 149_474.072_249_1);
pub const MEAN_VENUS: (f64, f64) = (181.979_801, 58_519.213_030_2);
pub const MEAN_MARS: (f64, f64) = (355.433_000, 19_141.696_447_1);
pub const MEAN_JUPITER: (f64, f64) = (34.351_519, 3_036.302_774_8);
pub const MEAN_SATURN: (f64, f64) = (50.077_444, 1_223.511_068_6);

/// Tropikal ortalama boylamlar, derece, [0, 360). `sun` jeosantrik ortalama Güneş'tir (Dünya'nın
/// güneş merkezli ortalama boylamı + 180°); diğerleri güneş merkezli ortalama boylamlardır.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeanLongitudes {
    pub sun: f64,
    pub mercury: f64,
    pub venus: f64,
    pub mars: f64,
    pub jupiter: f64,
    pub saturn: f64,
}

/// `jd_ut` anındaki ortalama boylamlar. UT ile dinamik zaman arasındaki fark (≈ 1 dk) en hızlı
/// öğede (Merkür, 4,1°/gün) 0,003°'den küçüktür ve yok sayılır.
pub fn mean_longitudes(jd_ut: f64) -> MeanLongitudes {
    let t = (jd_ut - J2000) / JULIAN_CENTURY_DAYS;
    let at = |(l0, l1): (f64, f64)| normalize(l0 + l1 * t);
    MeanLongitudes {
        sun: at(MEAN_SUN),
        mercury: at(MEAN_MERCURY),
        venus: at(MEAN_VENUS),
        mars: at(MEAN_MARS),
        jupiter: at(MEAN_JUPITER),
        saturn: at(MEAN_SATURN),
    }
}

/// İki boylamın dairesel orta noktası: `a`'dan `b`'ye kısa yayın ortası, [0, 360). Tam karşıt
/// iki boylamda yay `−180°` sayılır ve `a − 90°` döner (gerçek gökyüzünde ortalama ve gerçek
/// boylam hiçbir zaman karşıt olmaz).
pub fn circular_midpoint(a: f64, b: f64) -> f64 {
    // `a`'dan `b`'ye işaretli kısa yay, [-180, 180).
    let arc = normalize(b - a + 180.0) - 180.0;
    normalize(a + arc / 2.0)
}

/// Cheshta formülü: `ort` = `slow` ile `true_lon`'un dairesel orta noktası; `K` = (`apex` −
/// `ort`) mod 360, 180°'den büyükse `360 − K`; güç `K / 3` (0–60).
///
/// Dış gezegenlerde `slow` gezegenin ortalama boylamı, `apex` ortalama Güneş'tir; Merkür ve
/// Venüs'te `slow` ortalama Güneş, `apex` gezegenin güneş merkezli ortalama boylamıdır.
pub fn cheshta_from(slow: f64, true_lon: f64, apex: f64) -> f64 {
    let k = normalize(apex - circular_midpoint(slow, true_lon));
    (if k > 180.0 { 360.0 - k } else { k }) / 3.0
}

/// Grahanın cheshta balası. Güneş: ayana bala (iki katı alınmamış). Ay: paksha balanın iki
/// katı alınmamış hâli (`uzanım / 3`). Düğümler 0.
pub fn cheshta_bala(graha: Graha, positions: &Positions, astro: &ShadbalaAstro) -> f64 {
    let mean = &astro.mean;
    let true_lon = astro
        .tropical_lons
        .get(graha as usize)
        .copied()
        .unwrap_or(0.0);
    match graha {
        Graha::Sun => ayana_bala(Graha::Sun, true_lon),
        Graha::Moon => paksha_elongation(positions) / 3.0,
        Graha::Mars => cheshta_from(mean.mars, true_lon, mean.sun),
        Graha::Jupiter => cheshta_from(mean.jupiter, true_lon, mean.sun),
        Graha::Saturn => cheshta_from(mean.saturn, true_lon, mean.sun),
        Graha::Mercury => cheshta_from(mean.sun, true_lon, mean.mercury),
        Graha::Venus => cheshta_from(mean.sun, true_lon, mean.venus),
        Graha::Rahu | Graha::Ketu => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panchang::Vara;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn mean_longitudes_at_the_epoch_and_one_century_later() {
        let m = mean_longitudes(J2000);
        assert_eq!(
            (m.sun, m.mercury, m.venus, m.mars, m.jupiter, m.saturn),
            (
                280.466_457,
                252.250_906,
                181.979_801,
                355.433,
                34.351_519,
                50.077_444
            )
        );
        // Bir yüzyıl sonra: L0 + L1, tam turlar atılır (elle: 36000,7698278 = 100 tur + 0,7698278).
        let c = mean_longitudes(J2000 + JULIAN_CENTURY_DAYS);
        assert!((c.sun - 281.236_284_8).abs() < 1e-6, "{}", c.sun);
        // Mars: 355,433 + 19141,6964471 = 19497,1294471 = 54 × 360 + 57,1294471.
        assert!((c.mars - 57.129_447_1).abs() < 1e-6, "{}", c.mars);
        // Satürn: 50,077444 + 1223,5110686 = 1273,5885126 = 3 × 360 + 193,5885126.
        assert!((c.saturn - 193.588_512_6).abs() < 1e-6, "{}", c.saturn);
        // Günlük ilerleme: Güneş 0,98565°, Merkür 4,09234°, Venüs 1,60215°.
        let d = mean_longitudes(J2000 + 1.0);
        assert!((normalize(d.sun - m.sun) - 0.985_647).abs() < 1e-6);
        assert!((normalize(d.mercury - m.mercury) - 4.092_377).abs() < 1e-6);
        assert!((normalize(d.venus - m.venus) - 1.602_169).abs() < 1e-6);
        // J2000'den önce de [0, 360) içindedir.
        let before = mean_longitudes(J2000 - 400.0 * 365.25);
        for v in [
            before.sun,
            before.mercury,
            before.venus,
            before.mars,
            before.jupiter,
            before.saturn,
        ] {
            assert!((0.0..360.0).contains(&v), "{v}");
        }
    }

    #[test]
    fn circular_midpoint_takes_the_short_arc() {
        assert!(close(circular_midpoint(10.0, 50.0), 30.0));
        assert!(close(circular_midpoint(50.0, 10.0), 30.0));
        assert!(close(circular_midpoint(350.0, 30.0), 10.0));
        assert!(close(circular_midpoint(30.0, 350.0), 10.0));
        assert!(close(circular_midpoint(340.0, 350.0), 345.0));
        assert!(close(circular_midpoint(77.0, 77.0), 77.0));
        // Tam karşıt: yay −180° sayılır, orta nokta a − 90°.
        assert!(close(circular_midpoint(0.0, 180.0), 270.0));
    }

    #[test]
    fn cheshta_is_full_at_opposition_and_zero_at_conjunction() {
        // Dış gezegen: ortalama ve gerçek boylam 100°. Güneş karşıda (280°): K = 180 → 60.
        assert!(close(cheshta_from(100.0, 100.0, 280.0), 60.0));
        // Güneş'le kavuşumda (100°): K = 0 → 0.
        assert!(close(cheshta_from(100.0, 100.0, 100.0), 0.0));
        // Dörtlükte (Güneş 190° ya da 10°): K = 90 → 30.
        assert!(close(cheshta_from(100.0, 100.0, 190.0), 30.0));
        assert!(close(cheshta_from(100.0, 100.0, 10.0), 30.0));
        // Ara değer: ortalama 100°, gerçek 110° → orta 105°; Güneş 225° → K = 120 → 40.
        assert!(close(cheshta_from(100.0, 110.0, 225.0), 40.0));
        // 0°'yi geçen orta nokta: ortalama 355°, gerçek 5° → orta 0°; Güneş 300° → K = 300 →
        // 360 − 300 = 60 → 20.
        assert!(close(cheshta_from(355.0, 5.0, 300.0), 20.0));
        // İç gezegen: ortalama Güneş 100°, gerçek boylam 100°. Güneş merkezli ortalama boylam
        // 280° (Dünya ile Güneş arasında, alt kavuşum): 60. 100° (üst kavuşum): 0.
        assert!(close(cheshta_from(100.0, 100.0, 280.0), 60.0));
        assert!(close(cheshta_from(100.0, 100.0, 100.0), 0.0));
    }

    #[test]
    fn cheshta_by_graha() {
        let p = Positions::synthetic(
            Some(0.0),
            [10.0, 70.0, 220.0, 65.0, 250.0, 40.0, 300.0, 130.0, 310.0],
        );
        let astro = ShadbalaAstro {
            jd_ut: 0.5,
            sunrise_jd_ut: 0.25,
            sunset_jd_ut: 0.75,
            next_sunrise_jd_ut: 1.25,
            mc_lon: 270.0,
            // Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn (tropikal).
            tropical_lons: [90.0, 150.0, 200.0, 80.0, 280.0, 130.0, 10.0],
            mean: MeanLongitudes {
                sun: 100.0,
                mercury: 250.0,
                venus: 100.0,
                mars: 220.0,
                jupiter: 280.0,
                saturn: 350.0,
            },
            weekday: Vara::Sunday,
            year_entry_weekday: Vara::Sunday,
            month_entry_weekday: Vara::Sunday,
        };
        // Güneş: ayana bala, tropikal 90° → 59,3125 (iki katı alınmaz).
        assert!(close(cheshta_bala(Graha::Sun, &p, &astro), 59.3125));
        // Ay: uzanım 60° → 20.
        assert!(close(cheshta_bala(Graha::Moon, &p, &astro), 20.0));
        // Mars: orta(220, 200) = 210; K = (100 − 210) mod 360 = 250 → 110 → 36,666…
        assert!(close(cheshta_bala(Graha::Mars, &p, &astro), 110.0 / 3.0));
        // Jüpiter: orta(280, 280) = 280; K = (100 − 280) mod 360 = 180 → 60.
        assert!(close(cheshta_bala(Graha::Jupiter, &p, &astro), 60.0));
        // Satürn: orta(350, 10) = 0; K = 100 → 33,333…
        assert!(close(cheshta_bala(Graha::Saturn, &p, &astro), 100.0 / 3.0));
        // Merkür: orta(ortalama Güneş 100, gerçek 80) = 90; K = (250 − 90) = 160 → 53,333…
        assert!(close(cheshta_bala(Graha::Mercury, &p, &astro), 160.0 / 3.0));
        // Venüs: orta(100, 130) = 115; K = (100 − 115) mod 360 = 345 → 15 → 5.
        assert!(close(cheshta_bala(Graha::Venus, &p, &astro), 5.0));
        assert_eq!(cheshta_bala(Graha::Rahu, &p, &astro), 0.0);
    }
}
````

- [ ] **Step 8: Testleri koş**

```bash
cargo test -p jyotish-core 2>&1 | grep -E "^test result|Running"
cargo test -p jyotish-core --lib shadbala 2>&1 | grep -E "^test result"
```
Expected: lib `215 passed` (191 + 24), golden `8 passed`, `modules_golden` `7 passed`; ikinci komut `24 passed`.

- [ ] **Step 9: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
```
Expected: fmt ve clippy çıktısız; `jyotish-core` lib `215 passed`; diğer Rust satırları tabandaki gibi.

- [ ] **Step 10: Commit**

```bash
git add crates/jyotish-core
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): shadbala item strengths: sthana, dig, temporal, cheshta, naisargika, drik and the graha war mark"
```
Expected: iki `grep` çıktısız.

---

### Task 2: Çekirdek: rapor, bhava bala, çalışma kâğıdı ve değişmezler

Spec §7 "Toplam ve eşik", "Yöntem — Bhava bala", "Test" (çalışma kâğıdı, property'ler); §13 katman 1, 2 ve 3; D2, D13–D16.

**Files:**
- Create: `crates/jyotish-core/src/shadbala/bhava.rs`, `tests/golden/modules/shadbala/WORKSHEET.md`, `tests/golden/modules/shadbala/*.json` (4 dosya)
- Modify: `crates/jyotish-core/src/shadbala/mod.rs`, `crates/jyotish-core/src/lib.rs`, `crates/jyotish-core/src/proptests.rs`, `crates/jyotish-core/tests/modules_golden.rs`, `tests/golden/modules/README.md`
- Test: `bhava.rs` ve `mod.rs` içindeki `mod tests`, `proptests.rs` (`shadbala_properties`), `tests/modules_golden.rs`

**Interfaces:**
- Consumes: Task 1'in tamamı; `lordship::{house_lord, house_sign}`; `modules_golden.rs` düzeneği (`Fixture<X>`, `Input`, `load`); `foundation_properties::positions()` üreteci.
- Produces (hepsi `jyotish_core`'dan dışa aktarılır):
  - `pub fn shadbala(positions: &Positions, astro: &ShadbalaAstro) -> Option<ShadbalaReport>`
  - `pub enum BhavaSignKind { Human, Quadruped, Water, Insect }` (`powerless_house() -> u8`), `pub fn bhava_sign_kind(sign: Sign, deg: f64) -> BhavaSignKind`, `pub fn bhava_midpoint(lagna_lon: f64, house: u8) -> f64`, `pub fn bhava_dig_bala(house: u8, kind: BhavaSignKind) -> f64`, `pub fn bhava_balas(positions: &Positions, grahas: &[GrahaShadbala; 7]) -> Option<[BhavaBala; 12]>`
  - `pub(crate) fn ranks<const N: usize>(totals: [f64; N]) -> [u8; N]`
  - El örneği düzeneği: `Input.astro: Option<ShadbalaAstro>`, `Input.lats`; `ShadbalaExpected { grahas, time_lords, bhavas }`; `MIN_SHADBALA = 4`; `WORKSHEET_TOL = 0.01`.

- [ ] **Step 1: Çalışma kâğıdını ve el örneklerini yaz (kaynak: `hand`)**

Üç tam örnek ve doğum saati bilinmeyen bir örnek. Girdiler tam derecelerdedir ve varga, drekkana ve moolatrikona sınırlarından uzak seçilmiştir; zamanlar gün kesridir (0,25 = 06:00). Çalışma kâğıdı her kalemin aritmetiğini yazar; JSON dosyaları aynı girdiyi ve dört ondalığa yuvarlanmış sonuçları taşır (tolerans 0,01 virupa).

**Yeni dosya:** `tests/golden/modules/shadbala/WORKSHEET.md`

````markdown
# Shadbala ve Bhava bala — çalışma kâğıdı

Kaynak: ileri modüller tasarım belgesi §7 (yöntem) ve §4 (sphuta drishti tablosu, K6 doğal sınıf).
Her örneğin girdisi aynı adlı JSON dosyasındadır; aşağıda her kalemin aritmetiği adım adım yazılıdır.
Değerler kuraldan elle türetilmiştir; motor çıktısı buraya kopyalanmaz (K9). Tolerans 0,01 virupa.
Zamanlar gün kesridir (0,25 = 06:00); hesap yalnızca farklarını kullanır. Boylamlar sidereal derecedir;
sentetik konumlarda rasi çerçevesi sidereal sayılır. Bölüm sınırında duran derece üst bölüme aittir.

Gözden geçiren (motoru yazan dışında biri) her satırı kuraldan yeniden hesaplar ve aşağıya adını,
tarihini ve bulduğu farkları yazar. Bir satır motorla uyuşmazsa önce kural yeniden okunur.

| Gözden geçiren | Tarih | Sonuç |
|---|---|---|
| | | |

## Örnek 1 — öğle doğumu, dolunaya yakın Ay, Merkür kötücül

Dosya: `day-birth-at-noon-near-full-moon.json`

Burçlar: Güneş Koç 11°, Ay Başak 22°, Mars Oğlak 26°, Merkür Koç 24°, Jüpiter Yengeç 7°, Venüs Balık 21°, Satürn Terazi 16°; lagna Koç 8°.
Doğal sınıf (K6): Ay − Güneş = 161° → Ay büyüyor, iyicil. Merkür'ün burcundaki diğer grahalar: Güneş → Merkür kötücül.

**Sthana bala**

- Güneş uccha: düşüş noktası 190°; uzaklık 179° / 3 = 59,6667.
- Güneş saptavargaja: D1 Koç (lord Mars: doğal dost, D1'de 10. burçta → geçici dost) 20; D2 Aslan (kendi burcu) 30; D3 Aslan (kendi burcu) 30; D7 İkizler (lord Merkür: doğal nötr, D1'de 1. burçta → geçici düşman) 4; D9 Yengeç (lord Ay: doğal dost, D1'de 6. burçta → geçici düşman) 10; D12 Aslan (kendi burcu) 30; D30 Yay (lord Jüpiter: doğal dost, D1'de 4. burçta → geçici dost) 20. Toplam 144.
- Güneş: ojayugma (burç Koç tek, navamsa Yengeç çift) 15; kendradi (1. ev) 60; drekkana (2. on derece) 0. Sthana = 59,6667 + 144 + 15 + 60 + 0 = 278,6667.
- Ay uccha: düşüş noktası 213°; uzaklık 41° / 3 = 13,6667.
- Ay saptavargaja: D1 Başak (lord Merkür: doğal dost, D1'de 8. burçta → geçici düşman) 10; D2 Aslan (lord Güneş: doğal dost, D1'de 8. burçta → geçici düşman) 10; D3 Boğa (lord Venüs: doğal nötr, D1'de 7. burçta → geçici düşman) 4; D7 Aslan (lord Güneş: doğal dost, D1'de 8. burçta → geçici düşman) 10; D9 Yengeç (kendi burcu) 30; D12 Boğa (lord Venüs: doğal nötr, D1'de 7. burçta → geçici düşman) 4; D30 Oğlak (lord Satürn: doğal nötr, D1'de 2. burçta → geçici dost) 15. Toplam 83.
- Ay: ojayugma (burç Başak çift, navamsa Yengeç çift) 30; kendradi (6. ev) 15; drekkana (3. on derece) 15. Sthana = 13,6667 + 83 + 30 + 15 + 15 = 156,6667.
- Mars uccha: düşüş noktası 118°; uzaklık 178° / 3 = 59,3333.
- Mars saptavargaja: D1 Oğlak (lord Satürn: doğal nötr, D1'de 10. burçta → geçici dost) 15; D2 Aslan (lord Güneş: doğal dost, D1'de 4. burçta → geçici dost) 20; D3 Başak (lord Merkür: doğal düşman, D1'de 4. burçta → geçici dost) 10; D7 Oğlak (lord Satürn: doğal nötr, D1'de 10. burçta → geçici dost) 15; D9 Aslan (lord Güneş: doğal dost, D1'de 4. burçta → geçici dost) 20; D12 Akrep (kendi burcu) 30; D30 Akrep (kendi burcu) 30. Toplam 140.
- Mars: ojayugma (burç Oğlak çift, navamsa Aslan tek) 15; kendradi (10. ev) 60; drekkana (3. on derece) 0. Sthana = 59,3333 + 140 + 15 + 60 + 0 = 274,3333.
- Merkür uccha: düşüş noktası 345°; uzaklık 39° / 3 = 13.
- Merkür saptavargaja: D1 Koç (lord Mars: doğal nötr, D1'de 10. burçta → geçici dost) 15; D2 Yengeç (lord Ay: doğal düşman, D1'de 6. burçta → geçici düşman) 2; D3 Yay (lord Jüpiter: doğal nötr, D1'de 4. burçta → geçici dost) 15; D7 Başak (kendi burcu) 30; D9 Akrep (lord Mars: doğal nötr, D1'de 10. burçta → geçici dost) 15; D12 Oğlak (lord Satürn: doğal nötr, D1'de 7. burçta → geçici düşman) 4; D30 İkizler (kendi burcu) 30. Toplam 111.
- Merkür: ojayugma (burç Koç tek, navamsa Akrep çift) 15; kendradi (1. ev) 60; drekkana (3. on derece) 0. Sthana = 13 + 111 + 15 + 60 + 0 = 199.
- Jüpiter uccha: düşüş noktası 275°; uzaklık 178° / 3 = 59,3333.
- Jüpiter saptavargaja: D1 Yengeç (lord Ay: doğal dost, D1'de 3. burçta → geçici dost) 20; D2 Yengeç (lord Ay: doğal dost, D1'de 3. burçta → geçici dost) 20; D3 Yengeç (lord Ay: doğal dost, D1'de 3. burçta → geçici dost) 20; D7 Kova (lord Satürn: doğal nötr, D1'de 4. burçta → geçici dost) 15; D9 Başak (lord Merkür: doğal düşman, D1'de 10. burçta → geçici dost) 10; D12 Başak (lord Merkür: doğal düşman, D1'de 10. burçta → geçici dost) 10; D30 Başak (lord Merkür: doğal düşman, D1'de 10. burçta → geçici dost) 10. Toplam 105.
- Jüpiter: ojayugma (burç Yengeç çift, navamsa Başak çift) 0; kendradi (4. ev) 60; drekkana (1. on derece) 15. Sthana = 59,3333 + 105 + 0 + 60 + 15 = 239,3333.
- Venüs uccha: düşüş noktası 177°; uzaklık 174° / 3 = 58.
- Venüs saptavargaja: D1 Balık (lord Jüpiter: doğal nötr, D1'de 5. burçta → geçici düşman) 4; D2 Aslan (lord Güneş: doğal düşman, D1'de 2. burçta → geçici dost) 10; D3 Akrep (lord Mars: doğal nötr, D1'de 11. burçta → geçici dost) 15; D7 Oğlak (lord Satürn: doğal dost, D1'de 8. burçta → geçici düşman) 10; D9 Oğlak (lord Satürn: doğal dost, D1'de 8. burçta → geçici düşman) 10; D12 Akrep (lord Mars: doğal nötr, D1'de 11. burçta → geçici dost) 15; D30 Oğlak (lord Satürn: doğal dost, D1'de 8. burçta → geçici düşman) 10. Toplam 74.
- Venüs: ojayugma (burç Balık çift, navamsa Oğlak çift) 30; kendradi (12. ev) 15; drekkana (3. on derece) 15. Sthana = 58 + 74 + 30 + 15 + 15 = 192.
- Satürn uccha: düşüş noktası 20°; uzaklık 176° / 3 = 58,6667.
- Satürn saptavargaja: D1 Terazi (lord Venüs: doğal dost, D1'de 6. burçta → geçici düşman) 10; D2 Yengeç (lord Ay: doğal düşman, D1'de 12. burçta → geçici dost) 10; D3 Kova (kendi burcu) 30; D7 Oğlak (kendi burcu) 30; D9 Kova (kendi burcu) 30; D12 Koç (lord Mars: doğal düşman, D1'de 4. burçta → geçici dost) 10; D30 Yay (lord Jüpiter: doğal nötr, D1'de 10. burçta → geçici dost) 15. Toplam 135.
- Satürn: ojayugma (burç Terazi tek, navamsa Kova tek) 30; kendradi (7. ev) 60; drekkana (2. on derece) 15. Sthana = 58,6667 + 135 + 30 + 60 + 15 = 298,6667.

**Dig bala**

- Güneş: güçsüz nokta 96°; uzaklık 85° / 3 = 28,3333.
- Ay: güçsüz nokta 276°; uzaklık 104° / 3 = 34,6667.
- Mars: güçsüz nokta 96°; uzaklık 160° / 3 = 53,3333.
- Merkür: güçsüz nokta 188°; uzaklık 164° / 3 = 54,6667.
- Jüpiter: güçsüz nokta 188°; uzaklık 91° / 3 = 30,3333.
- Venüs: güçsüz nokta 276°; uzaklık 75° / 3 = 25.
- Satürn: güçsüz nokta 8°; uzaklık 172° / 3 = 57,3333.

**Zaman gücü**

- Öğle = (0,25 + 0,75) / 2 = 0,5; doğum 0,5; fark 0 saat. m = 0.
- Gündüz–gece: Güneş, Jüpiter, Venüs 60 × (1 − 0/12) = 60; Ay, Mars, Satürn 60 × 0/12 = 0; Merkür 60.
- Paksha: uzanım e = 161°. Ay 2e/3 = 107,3333; iyiciller (Jüpiter, Venüs) e/3 = 53,6667; kötücüller (Güneş, Mars, Merkür, Satürn) (180 − e)/3 = 6,3333.
- Tribhaga: gündüz; (0,5 − 0,25) / (0,75 − 0,25) × 3 = 1,5 → 2. dilim: Güneş. Güneş ve Jüpiter 60.
- Lordlar: yıl girişi Thursday → Jüpiter 15; ay girişi Tuesday → Mars 30; gün Sunday → Güneş 45; saat dilimi (0,5 − 0,25) / (1,25 − 0,25) × 24 = 6 → sıra 6 (7. dilim); Güneş'ten başlayıp Satürn → Jüpiter → Mars → Güneş → Venüs → Merkür → Ay döngüsünde 6 adım (6 mod 7 = 6) → Mars 60.
- Güneş ayana: λ = 35°; δ = asin(sin 23,45° × sin λ) = 13,1943°; (24 + 13,1943) / 48 × 60 = 46,4929; Güneş'te iki katı: 92,9857.
- Ay ayana: λ = 196°; δ = asin(sin 23,45° × sin λ) = -6,2974°; (24 + 6,2974) / 48 × 60 = 37,8718.
- Mars ayana: λ = 320°; δ = asin(sin 23,45° × sin λ) = -14,8208°; (24 − 14,8208) / 48 × 60 = 11,474.
- Merkür ayana: λ = 48°; δ = asin(sin 23,45° × sin λ) = 17,2015°; (24 + 17,2015) / 48 × 60 = 51,5019.
- Jüpiter ayana: λ = 121°; δ = asin(sin 23,45° × sin λ) = 19,9444°; (24 + 19,9444) / 48 × 60 = 54,9305.
- Venüs ayana: λ = 15°; δ = asin(sin 23,45° × sin λ) = 5,9118°; (24 + 5,9118) / 48 × 60 = 37,3897.
- Satürn ayana: λ = 220°; δ = asin(sin 23,45° × sin λ) = -14,8208°; (24 + 14,8208) / 48 × 60 = 48,526.
- Güneş zaman gücü = 60 + 6,3333 + 60 + 0 + 0 + 45 + 0 + 92,9857 = 264,3191.
- Ay zaman gücü = 0 + 107,3333 + 0 + 0 + 0 + 0 + 0 + 37,8718 = 145,2051.
- Mars zaman gücü = 0 + 6,3333 + 0 + 0 + 30 + 0 + 60 + 11,474 = 107,8074.
- Merkür zaman gücü = 60 + 6,3333 + 0 + 0 + 0 + 0 + 0 + 51,5019 = 117,8352.
- Jüpiter zaman gücü = 60 + 53,6667 + 60 + 15 + 0 + 0 + 0 + 54,9305 = 243,5972.
- Venüs zaman gücü = 60 + 53,6667 + 0 + 0 + 0 + 0 + 0 + 37,3897 = 151,0564.
- Satürn zaman gücü = 0 + 6,3333 + 0 + 0 + 0 + 0 + 0 + 48,526 = 54,8593.

**Cheshta bala**

- Güneş: ayana (iki katı alınmamış) 46,4929. Ay: e/3 = 53,6667.
- Mars: orta(ortalama Mars 310°, gerçek 320°) = 315°; K = (ortalama Güneş 33° − 315°) mod 360 = 78°; 78 / 3 = 26.
- Jüpiter: orta(ortalama Jüpiter 125°, gerçek 121°) = 123°; K = (ortalama Güneş 33° − 123°) mod 360 = 270° → 360 − 270 = 90°; 90 / 3 = 30.
- Satürn: orta(ortalama Satürn 215°, gerçek 220°) = 217,5°; K = (ortalama Güneş 33° − 217,5°) mod 360 = 175,5°; 175,5 / 3 = 58,5.
- Merkür: orta(ortalama Güneş 33°, gerçek 48°) = 40,5°; K = (Merkür'ün güneş merkezli ortalaması 100° − 40,5°) mod 360 = 59,5°; 59,5 / 3 = 19,8333.
- Venüs: orta(ortalama Güneş 33°, gerçek 15°) = 24°; K = (Venüs'ün güneş merkezli ortalaması 300° − 24°) mod 360 = 276° → 360 − 276 = 84°; 84 / 3 = 28.

**Naisargika bala**

- Güneş 60 × 7/7 = 60; Ay 60 × 6/7 = 51,4286; Mars 60 × 2/7 = 17,1429; Merkür 60 × 3/7 = 25,7143; Jüpiter 60 × 4/7 = 34,2857; Venüs 60 × 5/7 = 42,8571; Satürn 60 × 1/7 = 8,5714.

**Drik bala**

- Güneş (11°): Ay açı 199° → 50,5 (iyicil: 12,625); Mars açı 75° → 30 (kötücül: -7,5); Jüpiter açı 274° → 13 (iyicil: 3,25, tamamı +13); Satürn açı 175° → 50 (kötücül: -12,5). Drik = 8,875.
- Ay (172°): Güneş açı 161° → 22 (kötücül: -5,5); Mars açı 236° → 47 (kötücül: -11,75); Merkür açı 148° → 2 (kötücül: -0,5, tamamı +2); Jüpiter açı 75° → 30 (iyicil: 7,5, tamamı +30); Venüs açı 181° → 59,5 (iyicil: 14,875). Drik = 36,625.
- Mars (296°): Güneş açı 285° → 7,5 (kötücül: -1,875); Ay açı 124° → 26 (iyicil: 6,5); Merkür açı 272° → 14 (kötücül: -3,5, tamamı +14); Jüpiter açı 199° → 50,5 (iyicil: 12,625, tamamı +50,5); Satürn açı 100° → 40 (kötücül: -10). Drik = 68,25.
- Merkür (24°): Ay açı 212° → 44 (iyicil: 11); Mars açı 88° → 43 (kötücül: -10,75); Jüpiter açı 287° → 6,5 (iyicil: 1,625, tamamı +6,5); Venüs açı 33° → 1,5 (iyicil: 0,375); Satürn açı 188° → 56 (kötücül: -14). Drik = -5,25.
- Jüpiter (97°): Güneş açı 86° → 41 (kötücül: -10,25); Ay açı 285° → 7,5 (iyicil: 1,875); Mars açı 161° → 22 (kötücül: -5,5); Merkür açı 73° → 28 (kötücül: -7, tamamı +28); Venüs açı 106° → 37 (iyicil: 9,25); Satürn açı 261° → 19,5 (kötücül: -4,875). Drik = 11,5.
- Venüs (351°): Ay açı 179° → 58 (iyicil: 14,5); Mars açı 55° → 12,5 (kötücül: -3,125); Jüpiter açı 254° → 53 (iyicil: 13,25, tamamı +53); Satürn açı 155° → 10 (kötücül: -2,5). Drik = 75,125.
- Satürn (196°): Güneş açı 185° → 57,5 (kötücül: -14,375); Mars açı 260° → 20 (kötücül: -5); Merkür açı 172° → 44 (kötücül: -11, tamamı +44); Jüpiter açı 99° → 40,5 (iyicil: 10,125, tamamı +40,5); Venüs açı 205° → 47,5 (iyicil: 11,875). Drik = 76,125.

**Toplam, oran, sıra, savaş**

- Güneş: 278,6667 + 28,3333 + 264,3191 + 46,4929 + 60 + 8,875 = 686,6869 virupa = 11,4448 rupa; gerekli 390; oran 1,7607; sıra 1.
- Ay: 156,6667 + 34,6667 + 145,2051 + 53,6667 + 51,4286 + 36,625 = 478,2587 virupa = 7,971 rupa; gerekli 360; oran 1,3285; sıra 6.
- Mars: 274,3333 + 53,3333 + 107,8074 + 26 + 17,1429 + 68,25 = 546,8669 virupa = 9,1144 rupa; gerekli 300; oran 1,8229; sıra 4.
- Merkür: 199 + 54,6667 + 117,8352 + 19,8333 + 25,7143 + -5,25 = 411,7995 virupa = 6,8633 rupa; gerekli 420; oran 0,9805; sıra 7.
- Jüpiter: 239,3333 + 30,3333 + 243,5972 + 30 + 34,2857 + 11,5 = 589,0496 virupa = 9,8175 rupa; gerekli 390; oran 1,5104; sıra 2.
- Venüs: 192 + 25 + 151,0564 + 28 + 42,8571 + 75,125 = 514,0385 virupa = 8,5673 rupa; gerekli 330; oran 1,5577; sıra 5.
- Satürn: 298,6667 + 57,3333 + 54,8593 + 58,5 + 8,5714 + 76,125 = 554,0557 virupa = 9,2343 rupa; gerekli 300; oran 1,8469; sıra 3.

**Bhava bala**

- 1. bhava: orta 8° (Koç 8°, dört ayaklı; güçsüz bhava 4, 3 bhava = 90° → dig 30); lord Mars 546,8669; drishti: Ay açı 196° → 52 (iyicil: 13); Mars açı 72° → 27 (kötücül: -6,75); Jüpiter açı 271° → 14,5 (iyicil: 3,625, tamamı +14,5); Satürn açı 172° → 44 (kötücül: -11) = 13,375. Toplam 590,2419.
- 2. bhava: orta 38° (Boğa 8°, dört ayaklı; güçsüz bhava 4, 2 bhava = 60° → dig 20); lord Venüs 514,0385; drishti: Ay açı 226° → 37 (iyicil: 9,25); Mars açı 102° → 54 (kötücül: -13,5); Venüs açı 47° → 8,5 (iyicil: 2,125); Satürn açı 202° → 49 (kötücül: -12,25) = -14,375. Toplam 519,6635.
- 3. bhava: orta 68° (İkizler 8°, insan; güçsüz bhava 7, 4 bhava = 120° → dig 40); lord Merkür 411,7995; drishti: Güneş açı 57° → 13,5 (kötücül: -3,375); Ay açı 256° → 22 (iyicil: 5,5); Mars açı 132° → 18 (kötücül: -4,5); Merkür açı 44° → 7 (kötücül: -1,75, tamamı +7); Venüs açı 77° → 32 (iyicil: 8); Satürn açı 232° → 34 (kötücül: -8,5) = 2,375. Toplam 454,1745.
- 4. bhava: orta 98° (Yengeç 8°, su; güçsüz bhava 10, 6 bhava = 180° → dig 60); lord Ay 478,2587; drishti: Güneş açı 87° → 42 (kötücül: -10,5); Ay açı 286° → 7 (iyicil: 1,75); Mars açı 162° → 24 (kötücül: -6); Merkür açı 74° → 29 (kötücül: -7,25, tamamı +29); Venüs açı 107° → 36,5 (iyicil: 9,125); Satürn açı 262° → 19 (kötücül: -4,75) = 11,375. Toplam 549,6337.
- 5. bhava: orta 128° (Aslan 8°, dört ayaklı; güçsüz bhava 4, 1 bhava = 30° → dig 10); lord Güneş 686,6869; drishti: Güneş açı 117° → 31,5 (kötücül: -7,875); Mars açı 192° → 54 (kötücül: -13,5); Merkür açı 104° → 38 (kötücül: -9,5, tamamı +38); Jüpiter açı 31° → 0,5 (iyicil: 0,125, tamamı +0,5); Venüs açı 137° → 13 (iyicil: 3,25); Satürn açı 292° → 49 (kötücül: -12,25) = -1,25. Toplam 695,4369.
- 6. bhava: orta 158° (Başak 8°, insan; güçsüz bhava 7, 1 bhava = 30° → dig 10); lord Merkür 411,7995; drishti: Güneş açı 147° → 3 (kötücül: -0,75); Mars açı 222° → 54 (kötücül: -13,5); Merkür açı 134° → 16 (kötücül: -4, tamamı +16); Jüpiter açı 61° → 16 (iyicil: 4, tamamı +16); Venüs açı 167° → 34 (iyicil: 8,5) = 26,25. Toplam 448,0495.
- 7. bhava: orta 188° (Terazi 8°, insan; güçsüz bhava 7, 0 bhava = 0° → dig 0); lord Venüs 514,0385; drishti: Güneş açı 177° → 54 (kötücül: -13,5); Mars açı 252° → 24 (kötücül: -6); Merkür açı 164° → 28 (kötücül: -7, tamamı +28); Jüpiter açı 91° → 44,5 (iyicil: 11,125, tamamı +44,5); Venüs açı 197° → 51,5 (iyicil: 12,875) = 70. Toplam 584,0385.
- 8. bhava: orta 218° (Akrep 8°, Akrep; güçsüz bhava 1, 5 bhava = 150° → dig 50); lord Mars 546,8669; drishti: Güneş açı 207° → 46,5 (kötücül: -11,625); Ay açı 46° → 8 (iyicil: 2); Mars açı 282° → 9 (kötücül: -2,25); Merkür açı 194° → 53 (kötücül: -13,25, tamamı +53); Jüpiter açı 121° → 59 (iyicil: 14,75, tamamı +59); Venüs açı 227° → 36,5 (iyicil: 9,125) = 110,75. Toplam 707,6169.
- 9. bhava: orta 248° (Yay 8°, insan; güçsüz bhava 7, 2 bhava = 60° → dig 20); lord Jüpiter 589,0496; drishti: Güneş açı 237° → 31,5 (kötücül: -7,875); Ay açı 76° → 31 (iyicil: 7,75); Merkür açı 224° → 38 (kötücül: -9,5, tamamı +38); Jüpiter açı 151° → 2 (iyicil: 0,5, tamamı +2); Venüs açı 257° → 21,5 (iyicil: 5,375); Satürn açı 52° → 11 (kötücül: -2,75) = 33,5. Toplam 642,5496.
- 10. bhava: orta 278° (Oğlak 8°, dört ayaklı; güçsüz bhava 4, 6 bhava = 180° → dig 60); lord Satürn 554,0557; drishti: Güneş açı 267° → 16,5 (kötücül: -4,125); Ay açı 106° → 37 (iyicil: 9,25); Merkür açı 254° → 23 (kötücül: -5,75, tamamı +23); Jüpiter açı 181° → 59,5 (iyicil: 14,875, tamamı +59,5); Venüs açı 287° → 6,5 (iyicil: 1,625); Satürn açı 82° → 60 (kötücül: -15) = 83,375. Toplam 697,4307.
- 11. bhava: orta 308° (Kova 8°, insan; güçsüz bhava 7, 4 bhava = 120° → dig 40); lord Satürn 554,0557; drishti: Güneş açı 297° → 1,5 (kötücül: -0,375); Ay açı 136° → 14 (iyicil: 3,5); Merkür açı 284° → 8 (kötücül: -2, tamamı +8); Jüpiter açı 211° → 44,5 (iyicil: 11,125, tamamı +44,5); Satürn açı 112° → 34 (kötücül: -8,5) = 56,25. Toplam 650,3057.
- 12. bhava: orta 338° (Balık 8°, su; güçsüz bhava 10, 2 bhava = 60° → dig 20); lord Jüpiter 589,0496; drishti: Ay açı 166° → 32 (iyicil: 8); Mars açı 42° → 6 (kötücül: -1,5); Jüpiter açı 241° → 59,5 (iyicil: 14,875, tamamı +59,5); Satürn açı 142° → 8 (kötücül: -2) = 78,875. Toplam 687,9246.
- Sıra (büyükten küçüğe): 8, 10, 5, 12, 11, 9, 1, 7, 4, 2, 3, 6.

## Örnek 2 — gece doğumu, eşit olmayan gün, küçülen Ay, Mars–Venüs savaşı, Akrep lagna

Dosya: `night-birth-graha-war-waning-moon.json`

Burçlar: Güneş Akrep 13°, Ay Yengeç 11°, Mars İkizler 4°, Merkür Akrep 26°, Jüpiter Oğlak 11°, Venüs İkizler 4,5°, Satürn Koç 4°; lagna Akrep 17°.
Doğal sınıf (K6): Ay − Güneş = 238° → Ay küçülüyor, kötücül. Merkür'ün burcundaki diğer grahalar: Güneş → Merkür kötücül.

**Sthana bala**

- Güneş uccha: düşüş noktası 190°; uzaklık 33° / 3 = 11.
- Güneş saptavargaja: D1 Akrep (lord Mars: doğal dost, D1'de 8. burçta → geçici düşman) 10; D2 Yengeç (lord Ay: doğal dost, D1'de 9. burçta → geçici düşman) 10; D3 Balık (lord Jüpiter: doğal dost, D1'de 3. burçta → geçici dost) 20; D7 Aslan (kendi burcu) 30; D9 Terazi (lord Venüs: doğal düşman, D1'de 8. burçta → geçici düşman) 2; D12 Koç (lord Mars: doğal dost, D1'de 8. burçta → geçici düşman) 10; D30 Balık (lord Jüpiter: doğal dost, D1'de 3. burçta → geçici dost) 20. Toplam 102.
- Güneş: ojayugma (burç Akrep çift, navamsa Terazi tek) 15; kendradi (1. ev) 60; drekkana (2. on derece) 0. Sthana = 11 + 102 + 15 + 60 + 0 = 188.
- Ay uccha: düşüş noktası 213°; uzaklık 112° / 3 = 37,3333.
- Ay saptavargaja: D1 Yengeç (kendi burcu) 30; D2 Yengeç (kendi burcu) 30; D3 Akrep (lord Mars: doğal nötr, D1'de 12. burçta → geçici dost) 15; D7 Balık (lord Jüpiter: doğal nötr, D1'de 7. burçta → geçici düşman) 4; D9 Terazi (lord Venüs: doğal nötr, D1'de 12. burçta → geçici dost) 15; D12 Akrep (lord Mars: doğal nötr, D1'de 12. burçta → geçici dost) 15; D30 Başak (lord Merkür: doğal dost, D1'de 5. burçta → geçici düşman) 10. Toplam 119.
- Ay: ojayugma (burç Yengeç çift, navamsa Terazi tek) 15; kendradi (9. ev) 15; drekkana (2. on derece) 0. Sthana = 37,3333 + 119 + 15 + 15 + 0 = 186,3333.
- Mars uccha: düşüş noktası 118°; uzaklık 54° / 3 = 18.
- Mars saptavargaja: D1 İkizler (lord Merkür: doğal düşman, D1'de 6. burçta → geçici düşman) 2; D2 Aslan (lord Güneş: doğal dost, D1'de 6. burçta → geçici düşman) 10; D3 İkizler (lord Merkür: doğal düşman, D1'de 6. burçta → geçici düşman) 2; D7 İkizler (lord Merkür: doğal düşman, D1'de 6. burçta → geçici düşman) 2; D9 Akrep (kendi burcu) 30; D12 Yengeç (lord Ay: doğal dost, D1'de 2. burçta → geçici dost) 20; D30 Koç (kendi burcu) 30. Toplam 96.
- Mars: ojayugma (burç İkizler tek, navamsa Akrep çift) 15; kendradi (8. ev) 30; drekkana (1. on derece) 15. Sthana = 18 + 96 + 15 + 30 + 15 = 174.
- Merkür uccha: düşüş noktası 345°; uzaklık 109° / 3 = 36,3333.
- Merkür saptavargaja: D1 Akrep (lord Mars: doğal nötr, D1'de 8. burçta → geçici düşman) 4; D2 Aslan (lord Güneş: doğal dost, D1'de 1. burçta → geçici düşman) 10; D3 Yengeç (lord Ay: doğal düşman, D1'de 9. burçta → geçici düşman) 2; D7 Akrep (lord Mars: doğal nötr, D1'de 8. burçta → geçici düşman) 4; D9 Kova (lord Satürn: doğal nötr, D1'de 6. burçta → geçici düşman) 4; D12 Başak (kendi burcu) 30; D30 Akrep (lord Mars: doğal nötr, D1'de 8. burçta → geçici düşman) 4. Toplam 58.
- Merkür: ojayugma (burç Akrep çift, navamsa Kova tek) 15; kendradi (1. ev) 60; drekkana (3. on derece) 0. Sthana = 36,3333 + 58 + 15 + 60 + 0 = 169,3333.
- Jüpiter uccha: düşüş noktası 275°; uzaklık 6° / 3 = 2.
- Jüpiter saptavargaja: D1 Oğlak (lord Satürn: doğal nötr, D1'de 4. burçta → geçici dost) 15; D2 Yengeç (lord Ay: doğal dost, D1'de 7. burçta → geçici düşman) 10; D3 Boğa (lord Venüs: doğal düşman, D1'de 6. burçta → geçici düşman) 2; D7 Başak (lord Merkür: doğal düşman, D1'de 11. burçta → geçici dost) 10; D9 Koç (lord Mars: doğal dost, D1'de 6. burçta → geçici düşman) 10; D12 Boğa (lord Venüs: doğal düşman, D1'de 6. burçta → geçici düşman) 2; D30 Başak (lord Merkür: doğal düşman, D1'de 11. burçta → geçici dost) 10. Toplam 59.
- Jüpiter: ojayugma (burç Oğlak çift, navamsa Koç tek) 15; kendradi (3. ev) 15; drekkana (2. on derece) 0. Sthana = 2 + 59 + 15 + 15 + 0 = 91.
- Venüs uccha: düşüş noktası 177°; uzaklık 112,5° / 3 = 37,5.
- Venüs saptavargaja: D1 İkizler (lord Merkür: doğal dost, D1'de 6. burçta → geçici düşman) 10; D2 Aslan (lord Güneş: doğal düşman, D1'de 6. burçta → geçici düşman) 2; D3 İkizler (lord Merkür: doğal dost, D1'de 6. burçta → geçici düşman) 10; D7 Yengeç (lord Ay: doğal düşman, D1'de 2. burçta → geçici dost) 10; D9 Akrep (lord Mars: doğal nötr, D1'de 1. burçta → geçici düşman) 4; D12 Yengeç (lord Ay: doğal düşman, D1'de 2. burçta → geçici dost) 10; D30 Koç (lord Mars: doğal nötr, D1'de 1. burçta → geçici düşman) 4. Toplam 50.
- Venüs: ojayugma (burç İkizler tek, navamsa Akrep çift) 15; kendradi (8. ev) 30; drekkana (1. on derece) 0. Sthana = 37,5 + 50 + 15 + 30 + 0 = 132,5.
- Satürn uccha: düşüş noktası 20°; uzaklık 16° / 3 = 5,3333.
- Satürn saptavargaja: D1 Koç (lord Mars: doğal düşman, D1'de 3. burçta → geçici dost) 10; D2 Aslan (lord Güneş: doğal düşman, D1'de 8. burçta → geçici düşman) 2; D3 Koç (lord Mars: doğal düşman, D1'de 3. burçta → geçici dost) 10; D7 Koç (lord Mars: doğal düşman, D1'de 3. burçta → geçici dost) 10; D9 Boğa (lord Venüs: doğal dost, D1'de 3. burçta → geçici dost) 20; D12 Boğa (lord Venüs: doğal dost, D1'de 3. burçta → geçici dost) 20; D30 Koç (lord Mars: doğal düşman, D1'de 3. burçta → geçici dost) 10. Toplam 82.
- Satürn: ojayugma (burç Koç tek, navamsa Boğa çift) 15; kendradi (6. ev) 15; drekkana (1. on derece) 0. Sthana = 5,3333 + 82 + 15 + 15 + 0 = 117,3333.

**Dig bala**

- Güneş: güçsüz nokta 320°; uzaklık 97° / 3 = 32,3333.
- Ay: güçsüz nokta 140°; uzaklık 39° / 3 = 13.
- Mars: güçsüz nokta 320°; uzaklık 104° / 3 = 34,6667.
- Merkür: güçsüz nokta 47°; uzaklık 171° / 3 = 57.
- Jüpiter: güçsüz nokta 47°; uzaklık 126° / 3 = 42.
- Venüs: güçsüz nokta 140°; uzaklık 75,5° / 3 = 25,1667.
- Satürn: güçsüz nokta 227°; uzaklık 137° / 3 = 45,6667.

**Zaman gücü**

- Öğle = (0,3 + 0,7) / 2 = 0,5; doğum 0,95; fark 10,8 saat. m = 10,8.
- Gündüz–gece: Güneş, Jüpiter, Venüs 60 × (1 − 10,8/12) = 6; Ay, Mars, Satürn 60 × 10,8/12 = 54; Merkür 60.
- Paksha: uzanım e = 122°. Ay 2e/3 = 81,3333; iyiciller (Jüpiter, Venüs) e/3 = 40,6667; kötücüller (Güneş, Mars, Merkür, Satürn) (180 − e)/3 = 19,3333.
- Tribhaga: gece; (0,95 − 0,7) / (1,31 − 0,7) × 3 = 1,2295 → 2. dilim: Venüs. Venüs ve Jüpiter 60.
- Lordlar: yıl girişi Saturday → Satürn 15; ay girişi Wednesday → Merkür 30; gün Wednesday → Merkür 45; saat dilimi (0,95 − 0,3) / (1,31 − 0,3) × 24 = 15,4455 → sıra 15 (16. dilim); Merkür'ten başlayıp Satürn → Jüpiter → Mars → Güneş → Venüs → Merkür → Ay döngüsünde 15 adım (15 mod 7 = 1) → Ay 60.
- Güneş ayana: λ = 247°; δ = asin(sin 23,45° × sin λ) = -21,4884°; (24 − 21,4884) / 48 × 60 = 3,1394; Güneş'te iki katı: 6,2789.
- Ay ayana: λ = 125°; δ = asin(sin 23,45° × sin λ) = 19,025°; (24 − 19,025) / 48 × 60 = 6,2188.
- Mars ayana: λ = 88°; δ = asin(sin 23,45° × sin λ) = 23,4349°; (24 + 23,4349) / 48 × 60 = 59,2936.
- Merkür ayana: λ = 260°; δ = asin(sin 23,45° × sin λ) = -23,073°; (24 + 23,073) / 48 × 60 = 58,8412.
- Jüpiter ayana: λ = 305°; δ = asin(sin 23,45° × sin λ) = -19,025°; (24 − 19,025) / 48 × 60 = 6,2188.
- Venüs ayana: λ = 88,5°; δ = asin(sin 23,45° × sin λ) = 23,4415°; (24 + 23,4415) / 48 × 60 = 59,3019.
- Satürn ayana: λ = 28°; δ = asin(sin 23,45° × sin λ) = 10,7676°; (24 − 10,7676) / 48 × 60 = 16,5405.
- Güneş zaman gücü = 6 + 19,3333 + 0 + 0 + 0 + 0 + 0 + 6,2789 = 31,6122.
- Ay zaman gücü = 54 + 81,3333 + 0 + 0 + 0 + 0 + 60 + 6,2188 = 201,5521.
- Mars zaman gücü = 54 + 19,3333 + 0 + 0 + 0 + 0 + 0 + 59,2936 = 132,6269.
- Merkür zaman gücü = 60 + 19,3333 + 0 + 0 + 30 + 45 + 0 + 58,8412 = 213,1745.
- Jüpiter zaman gücü = 6 + 40,6667 + 60 + 0 + 0 + 0 + 0 + 6,2188 = 112,8854.
- Venüs zaman gücü = 6 + 40,6667 + 60 + 0 + 0 + 0 + 0 + 59,3019 = 165,9685.
- Satürn zaman gücü = 54 + 19,3333 + 0 + 15 + 0 + 0 + 0 + 16,5405 = 104,8739.

**Cheshta bala**

- Güneş: ayana (iki katı alınmamış) 3,1394. Ay: e/3 = 40,6667.
- Mars: orta(ortalama Mars 70°, gerçek 88°) = 79°; K = (ortalama Güneş 246° − 79°) mod 360 = 167°; 167 / 3 = 55,6667.
- Jüpiter: orta(ortalama Jüpiter 300°, gerçek 305°) = 302,5°; K = (ortalama Güneş 246° − 302,5°) mod 360 = 303,5° → 360 − 303,5 = 56,5°; 56,5 / 3 = 18,8333.
- Satürn: orta(ortalama Satürn 31°, gerçek 28°) = 29,5°; K = (ortalama Güneş 246° − 29,5°) mod 360 = 216,5° → 360 − 216,5 = 143,5°; 143,5 / 3 = 47,8333.
- Merkür: orta(ortalama Güneş 246°, gerçek 260°) = 253°; K = (Merkür'ün güneş merkezli ortalaması 40° − 253°) mod 360 = 147°; 147 / 3 = 49.
- Venüs: orta(ortalama Güneş 246°, gerçek 88,5°) = 167,25°; K = (Venüs'ün güneş merkezli ortalaması 150° − 167,25°) mod 360 = 342,75° → 360 − 342,75 = 17,25°; 17,25 / 3 = 5,75.

**Naisargika bala**

- Güneş 60 × 7/7 = 60; Ay 60 × 6/7 = 51,4286; Mars 60 × 2/7 = 17,1429; Merkür 60 × 3/7 = 25,7143; Jüpiter 60 × 4/7 = 34,2857; Venüs 60 × 5/7 = 42,8571; Satürn 60 × 1/7 = 8,5714.

**Drik bala**

- Güneş (223°): Ay açı 122° → 28 (kötücül: -7); Mars açı 159° → 18 (kötücül: -4,5); Venüs açı 158,5° → 17 (iyicil: 4,25); Satürn açı 219° → 40,5 (kötücül: -10,125). Drik = -17,375.
- Ay (101°): Güneş açı 238° → 31 (kötücül: -7,75); Mars açı 37° → 3,5 (kötücül: -0,875); Merkür açı 225° → 37,5 (kötücül: -9,375, tamamı +37,5); Jüpiter açı 180° → 60 (iyicil: 15, tamamı +60); Venüs açı 36,5° → 3,25 (iyicil: 0,8125); Satürn açı 97° → 41,5 (kötücül: -10,375). Drik = 84,9375.
- Mars (64°): Güneş açı 201° → 49,5 (kötücül: -12,375); Merkür açı 188° → 56 (kötücül: -14, tamamı +56); Jüpiter açı 143° → 37 (iyicil: 9,25, tamamı +37); Satürn açı 60° → 60 (kötücül: -15). Drik = 60,875.
- Merkür (236°): Ay açı 135° → 15 (kötücül: -3,75); Mars açı 172° → 44 (kötücül: -11); Venüs açı 171,5° → 43 (iyicil: 10,75); Satürn açı 232° → 34 (kötücül: -8,5). Drik = -12,5.
- Jüpiter (281°): Güneş açı 58° → 14 (kötücül: -3,5); Ay açı 180° → 60 (kötücül: -15); Mars açı 217° → 56,5 (kötücül: -14,125); Merkür açı 45° → 7,5 (kötücül: -1,875, tamamı +7,5); Venüs açı 216,5° → 41,75 (iyicil: 10,4375); Satürn açı 277° → 56,5 (kötücül: -14,125). Drik = -30,6875.
- Venüs (64,5°): Güneş açı 201,5° → 49,25 (kötücül: -12,3125); Merkür açı 188,5° → 55,75 (kötücül: -13,9375, tamamı +55,75); Jüpiter açı 143,5° → 36,5 (iyicil: 9,125, tamamı +36,5); Satürn açı 60,5° → 60 (kötücül: -15). Drik = 60,125.
- Satürn (4°): Güneş açı 141° → 9 (kötücül: -2,25); Ay açı 263° → 18,5 (kötücül: -4,625); Merkür açı 128° → 22 (kötücül: -5,5, tamamı +22); Jüpiter açı 83° → 38 (iyicil: 9,5, tamamı +38); Venüs açı 299,5° → 0,25 (iyicil: 0,0625). Drik = 57,1875.

**Toplam, oran, sıra, savaş**

- Güneş: 188 + 32,3333 + 31,6122 + 3,1394 + 60 + -17,375 = 297,71 virupa = 4,9618 rupa; gerekli 390; oran 0,7634; sıra 6.
- Ay: 186,3333 + 13 + 201,5521 + 40,6667 + 51,4286 + 84,9375 = 577,9182 virupa = 9,632 rupa; gerekli 360; oran 1,6053; sıra 1.
- Mars: 174 + 34,6667 + 132,6269 + 55,6667 + 17,1429 + 60,875 = 474,9781 virupa = 7,9163 rupa; gerekli 300; oran 1,5833; sıra 3; savaş: Venüs ile (boylam farkı 0,5°; enlemler 0,8° ve -0,3°), kazanan Mars.
- Merkür: 169,3333 + 57 + 213,1745 + 49 + 25,7143 + -12,5 = 501,7221 virupa = 8,362 rupa; gerekli 420; oran 1,1946; sıra 2.
- Jüpiter: 91 + 42 + 112,8854 + 18,8333 + 34,2857 + -30,6875 = 268,317 virupa = 4,4719 rupa; gerekli 390; oran 0,688; sıra 7.
- Venüs: 132,5 + 25,1667 + 165,9685 + 5,75 + 42,8571 + 60,125 = 432,3673 virupa = 7,2061 rupa; gerekli 330; oran 1,3102; sıra 4; savaş: Mars ile (boylam farkı 0,5°; enlemler -0,3° ve 0,8°), kazanan Mars.
- Satürn: 117,3333 + 45,6667 + 104,8739 + 47,8333 + 8,5714 + 57,1875 = 381,4661 virupa = 6,3578 rupa; gerekli 300; oran 1,2716; sıra 5.

**Bhava bala**

- 1. bhava: orta 227° (Akrep 17°, Akrep; güçsüz bhava 1, 0 bhava = 0° → dig 0); lord Mars 474,9781; drishti: Ay açı 126° → 24 (kötücül: -6); Mars açı 163° → 26 (kötücül: -6,5); Venüs açı 162,5° → 25 (iyicil: 6,25); Satürn açı 223° → 38,5 (kötücül: -9,625) = -15,875. Toplam 459,1031.
- 2. bhava: orta 257° (Yay 17°, dört ayaklı; güçsüz bhava 4, 2 bhava = 60° → dig 20); lord Jüpiter 268,317; drishti: Güneş açı 34° → 2 (kötücül: -0,5); Ay açı 156° → 12 (kötücül: -3); Mars açı 193° → 53,5 (kötücül: -13,375); Venüs açı 192,5° → 53,75 (iyicil: 13,4375); Satürn açı 253° → 23,5 (kötücül: -5,875) = -9,3125. Toplam 279,0045.
- 3. bhava: orta 287° (Oğlak 17°, su; güçsüz bhava 10, 5 bhava = 150° → dig 50); lord Satürn 381,4661; drishti: Güneş açı 64° → 19 (kötücül: -4,75); Ay açı 186° → 57 (kötücül: -14,25); Mars açı 223° → 53,5 (kötücül: -13,375); Merkür açı 51° → 10,5 (kötücül: -2,625, tamamı +10,5); Venüs açı 222,5° → 38,75 (iyicil: 9,6875); Satürn açı 283° → 53,5 (kötücül: -13,375) = -28,1875. Toplam 403,2786.
- 4. bhava: orta 317° (Kova 17°, insan; güçsüz bhava 7, 3 bhava = 90° → dig 30); lord Satürn 381,4661; drishti: Güneş açı 94° → 43 (kötücül: -10,75); Ay açı 216° → 42 (kötücül: -10,5); Mars açı 253° → 23,5 (kötücül: -5,875); Merkür açı 81° → 36 (kötücül: -9, tamamı +36); Jüpiter açı 36° → 3 (iyicil: 0,75, tamamı +3); Venüs açı 252,5° → 23,75 (iyicil: 5,9375) = 9,5625. Toplam 421,0286.
- 5. bhava: orta 347° (Balık 17°, su; güçsüz bhava 10, 5 bhava = 150° → dig 50); lord Jüpiter 268,317; drishti: Güneş açı 124° → 26 (kötücül: -6,5); Ay açı 246° → 27 (kötücül: -6,75); Mars açı 283° → 8,5 (kötücül: -2,125); Merkür açı 111° → 34,5 (kötücül: -8,625, tamamı +34,5); Jüpiter açı 66° → 21 (iyicil: 5,25, tamamı +21); Venüs açı 282,5° → 8,75 (iyicil: 2,1875) = 38,9375. Toplam 357,2545.
- 6. bhava: orta 17° (Koç 17°, dört ayaklı; güçsüz bhava 4, 2 bhava = 60° → dig 20); lord Mars 474,9781; drishti: Güneş açı 154° → 8 (kötücül: -2); Ay açı 276° → 12 (kötücül: -3); Merkür açı 141° → 9 (kötücül: -2,25, tamamı +9); Jüpiter açı 96° → 42 (iyicil: 10,5, tamamı +42) = 54,25. Toplam 549,2281.
- 7. bhava: orta 47° (Boğa 17°, dört ayaklı; güçsüz bhava 4, 3 bhava = 90° → dig 30); lord Venüs 432,3673; drishti: Güneş açı 184° → 58 (kötücül: -14,5); Merkür açı 171° → 42 (kötücül: -10,5, tamamı +42); Jüpiter açı 126° → 54 (iyicil: 13,5, tamamı +54); Satürn açı 43° → 6,5 (kötücül: -1,625) = 82,875. Toplam 545,2423.
- 8. bhava: orta 77° (İkizler 17°, insan; güçsüz bhava 7, 1 bhava = 30° → dig 10); lord Merkür 501,7221; drishti: Güneş açı 214° → 43 (kötücül: -10,75); Merkür açı 201° → 49,5 (kötücül: -12,375, tamamı +49,5); Jüpiter açı 156° → 12 (iyicil: 3, tamamı +12); Satürn açı 73° → 60 (kötücül: -15) = 26,375. Toplam 538,0971.
- 9. bhava: orta 107° (Yengeç 17°, su; güçsüz bhava 10, 1 bhava = 30° → dig 10); lord Ay 577,9182; drishti: Güneş açı 244° → 28 (kötücül: -7); Mars açı 43° → 6,5 (kötücül: -1,625); Merkür açı 231° → 34,5 (kötücül: -8,625, tamamı +34,5); Jüpiter açı 186° → 57 (iyicil: 14,25, tamamı +57); Venüs açı 42,5° → 6,25 (iyicil: 1,5625); Satürn açı 103° → 38,5 (kötücül: -9,625) = 80,4375. Toplam 668,3557.
- 10. bhava: orta 137° (Aslan 17°, dört ayaklı; güçsüz bhava 4, 6 bhava = 180° → dig 60); lord Güneş 297,71; drishti: Güneş açı 274° → 13 (kötücül: -3,25); Ay açı 36° → 3 (kötücül: -0,75); Mars açı 73° → 28 (kötücül: -7); Merkür açı 261° → 19,5 (kötücül: -4,875, tamamı +19,5); Jüpiter açı 216° → 42 (iyicil: 10,5, tamamı +42); Venüs açı 72,5° → 27,5 (iyicil: 6,875); Satürn açı 133° → 17 (kötücül: -4,25) = 58,75. Toplam 416,46.
- 11. bhava: orta 167° (Başak 17°, insan; güçsüz bhava 7, 4 bhava = 120° → dig 40); lord Merkür 501,7221; drishti: Ay açı 66° → 21 (kötücül: -5,25); Mars açı 103° → 53,5 (kötücül: -13,375); Merkür açı 291° → 4,5 (kötücül: -1,125, tamamı +4,5); Jüpiter açı 246° → 57 (iyicil: 14,25, tamamı +57); Venüs açı 102,5° → 38,75 (iyicil: 9,6875); Satürn açı 163° → 26 (kötücül: -6,5) = 59,1875. Toplam 600,9096.
- 12. bhava: orta 197° (Terazi 17°, insan; güçsüz bhava 7, 5 bhava = 150° → dig 50); lord Venüs 432,3673; drishti: Ay açı 96° → 42 (kötücül: -10,5); Mars açı 133° → 17 (kötücül: -4,25); Jüpiter açı 276° → 12 (iyicil: 3, tamamı +12); Venüs açı 132,5° → 17,5 (iyicil: 4,375); Satürn açı 193° → 53,5 (kötücül: -13,375) = -8,75. Toplam 473,6173.
- Sıra (büyükten küçüğe): 9, 11, 6, 7, 8, 12, 1, 4, 10, 3, 5, 2.

## Örnek 3 — şafaktan önce doğum (öğleye uzaklık 12 saati aşar), moolatrikona grahalar, eşit enlemli Merkür–Jüpiter savaşı, Yay lagna (ilk yarı)

Dosya: `pre-dawn-birth-moolatrikona-equal-latitude-war.json`

Burçlar: Güneş Kova 7°, Ay Balık 7°, Mars Koç 7°, Merkür Oğlak 13°, Jüpiter Oğlak 13,5°, Venüs Terazi 6°, Satürn Kova 16°; lagna Yay 9°.
Doğal sınıf (K6): Ay − Güneş = 30° → Ay büyüyor, iyicil. Merkür'ün burcundaki diğer grahalar: Jüpiter → Merkür iyicil.

**Sthana bala**

- Güneş uccha: düşüş noktası 190°; uzaklık 117° / 3 = 39.
- Güneş saptavargaja: D1 Kova (lord Satürn: doğal düşman, D1'de 1. burçta → geçici düşman) 2; D2 Aslan (kendi burcu) 30; D3 Kova (lord Satürn: doğal düşman, D1'de 1. burçta → geçici düşman) 2; D7 Balık (lord Jüpiter: doğal dost, D1'de 12. burçta → geçici dost) 20; D9 Yay (lord Jüpiter: doğal dost, D1'de 12. burçta → geçici dost) 20; D12 Koç (lord Mars: doğal dost, D1'de 3. burçta → geçici dost) 20; D30 Kova (lord Satürn: doğal düşman, D1'de 1. burçta → geçici düşman) 2. Toplam 96.
- Güneş: ojayugma (burç Kova tek, navamsa Yay tek) 30; kendradi (3. ev) 15; drekkana (1. on derece) 15. Sthana = 39 + 96 + 30 + 15 + 15 = 195.
- Ay uccha: düşüş noktası 213°; uzaklık 124° / 3 = 41,3333.
- Ay saptavargaja: D1 Balık (lord Jüpiter: doğal nötr, D1'de 11. burçta → geçici dost) 15; D2 Yengeç (kendi burcu) 30; D3 Balık (lord Jüpiter: doğal nötr, D1'de 11. burçta → geçici dost) 15; D7 Terazi (lord Venüs: doğal nötr, D1'de 8. burçta → geçici düşman) 4; D9 Başak (lord Merkür: doğal dost, D1'de 11. burçta → geçici dost) 20; D12 Boğa (lord Venüs: doğal nötr, D1'de 8. burçta → geçici düşman) 4; D30 Başak (lord Merkür: doğal dost, D1'de 11. burçta → geçici dost) 20. Toplam 108.
- Ay: ojayugma (burç Balık çift, navamsa Başak çift) 30; kendradi (4. ev) 60; drekkana (1. on derece) 0. Sthana = 41,3333 + 108 + 30 + 60 + 0 = 239,3333.
- Mars uccha: düşüş noktası 118°; uzaklık 111° / 3 = 37.
- Mars saptavargaja: D1 Koç (moolatrikona) 45; D2 Aslan (lord Güneş: doğal dost, D1'de 11. burçta → geçici dost) 20; D3 Koç (kendi burcu) 30; D7 Boğa (lord Venüs: doğal nötr, D1'de 7. burçta → geçici düşman) 4; D9 İkizler (lord Merkür: doğal düşman, D1'de 10. burçta → geçici dost) 10; D12 İkizler (lord Merkür: doğal düşman, D1'de 10. burçta → geçici dost) 10; D30 Kova (lord Satürn: doğal nötr, D1'de 11. burçta → geçici dost) 15. Toplam 134.
- Mars: ojayugma (burç Koç tek, navamsa İkizler tek) 30; kendradi (5. ev) 30; drekkana (1. on derece) 15. Sthana = 37 + 134 + 30 + 30 + 15 = 246.
- Merkür uccha: düşüş noktası 345°; uzaklık 62° / 3 = 20,6667.
- Merkür saptavargaja: D1 Oğlak (lord Satürn: doğal nötr, D1'de 2. burçta → geçici dost) 15; D2 Yengeç (lord Ay: doğal düşman, D1'de 3. burçta → geçici dost) 10; D3 Boğa (lord Venüs: doğal dost, D1'de 10. burçta → geçici dost) 20; D7 Terazi (lord Venüs: doğal dost, D1'de 10. burçta → geçici dost) 20; D9 Koç (lord Mars: doğal nötr, D1'de 4. burçta → geçici dost) 15; D12 İkizler (kendi burcu) 30; D30 Balık (lord Jüpiter: doğal nötr, D1'de 1. burçta → geçici düşman) 4. Toplam 114.
- Merkür: ojayugma (burç Oğlak çift, navamsa Koç tek) 15; kendradi (2. ev) 30; drekkana (2. on derece) 15. Sthana = 20,6667 + 114 + 15 + 30 + 15 = 194,6667.
- Jüpiter uccha: düşüş noktası 275°; uzaklık 8,5° / 3 = 2,8333.
- Jüpiter saptavargaja: D1 Oğlak (lord Satürn: doğal nötr, D1'de 2. burçta → geçici dost) 15; D2 Yengeç (lord Ay: doğal dost, D1'de 3. burçta → geçici dost) 20; D3 Boğa (lord Venüs: doğal düşman, D1'de 10. burçta → geçici dost) 10; D7 Terazi (lord Venüs: doğal düşman, D1'de 10. burçta → geçici dost) 10; D9 Boğa (lord Venüs: doğal düşman, D1'de 10. burçta → geçici dost) 10; D12 İkizler (lord Merkür: doğal düşman, D1'de 1. burçta → geçici düşman) 2; D30 Balık (kendi burcu) 30. Toplam 97.
- Jüpiter: ojayugma (burç Oğlak çift, navamsa Boğa çift) 0; kendradi (2. ev) 30; drekkana (2. on derece) 0. Sthana = 2,8333 + 97 + 0 + 30 + 0 = 129,8333.
- Venüs uccha: düşüş noktası 177°; uzaklık 9° / 3 = 3.
- Venüs saptavargaja: D1 Terazi (moolatrikona) 45; D2 Aslan (lord Güneş: doğal düşman, D1'de 5. burçta → geçici düşman) 2; D3 Terazi (kendi burcu) 30; D7 Akrep (lord Mars: doğal nötr, D1'de 7. burçta → geçici düşman) 4; D9 Akrep (lord Mars: doğal nötr, D1'de 7. burçta → geçici düşman) 4; D12 Yay (lord Jüpiter: doğal nötr, D1'de 4. burçta → geçici dost) 15; D30 Kova (lord Satürn: doğal dost, D1'de 5. burçta → geçici düşman) 10. Toplam 110.
- Venüs: ojayugma (burç Terazi tek, navamsa Akrep çift) 15; kendradi (11. ev) 30; drekkana (1. on derece) 0. Sthana = 3 + 110 + 15 + 30 + 0 = 158.
- Satürn uccha: düşüş noktası 20°; uzaklık 64° / 3 = 21,3333.
- Satürn saptavargaja: D1 Kova (moolatrikona) 45; D2 Yengeç (lord Ay: doğal düşman, D1'de 2. burçta → geçici dost) 10; D3 İkizler (lord Merkür: doğal dost, D1'de 12. burçta → geçici dost) 20; D7 Boğa (lord Venüs: doğal dost, D1'de 9. burçta → geçici düşman) 10; D9 Kova (kendi burcu) 30; D12 Aslan (lord Güneş: doğal düşman, D1'de 1. burçta → geçici düşman) 2; D30 Yay (lord Jüpiter: doğal nötr, D1'de 12. burçta → geçici dost) 15. Toplam 132.
- Satürn: ojayugma (burç Kova tek, navamsa Kova tek) 30; kendradi (3. ev) 15; drekkana (2. on derece) 15. Sthana = 21,3333 + 132 + 30 + 15 + 15 = 213,3333.

**Dig bala**

- Güneş: güçsüz nokta 350°; uzaklık 43° / 3 = 14,3333.
- Ay: güçsüz nokta 170°; uzaklık 167° / 3 = 55,6667.
- Mars: güçsüz nokta 350°; uzaklık 17° / 3 = 5,6667.
- Merkür: güçsüz nokta 69°; uzaklık 146° / 3 = 48,6667.
- Jüpiter: güçsüz nokta 69°; uzaklık 145,5° / 3 = 48,5.
- Venüs: güçsüz nokta 170°; uzaklık 16° / 3 = 5,3333.
- Satürn: güçsüz nokta 249°; uzaklık 67° / 3 = 22,3333.

**Zaman gücü**

- Öğle = (0,27 + 0,73) / 2 = 0,5; doğum 1,2; fark 16,8 saat → 24 − 16,8 = 7,2 saat. m = 7,2.
- Gündüz–gece: Güneş, Jüpiter, Venüs 60 × (1 − 7,2/12) = 24; Ay, Mars, Satürn 60 × 7,2/12 = 36; Merkür 60.
- Paksha: uzanım e = 30°. Ay 2e/3 = 20; iyiciller (Merkür, Jüpiter, Venüs) e/3 = 10; kötücüller (Güneş, Mars, Satürn) (180 − e)/3 = 50.
- Tribhaga: gece; (1,2 − 0,73) / (1,27 − 0,73) × 3 = 2,6111 → 3. dilim: Mars. Mars ve Jüpiter 60.
- Lordlar: yıl girişi Monday → Ay 15; ay girişi Sunday → Güneş 30; gün Friday → Venüs 45; saat dilimi (1,2 − 0,27) / (1,27 − 0,27) × 24 = 22,32 → sıra 22 (23. dilim); Venüs'ten başlayıp Satürn → Jüpiter → Mars → Güneş → Venüs → Merkür → Ay döngüsünde 22 adım (22 mod 7 = 1) → Merkür 60.
- Güneş ayana: λ = 331°; δ = asin(sin 23,45° × sin λ) = -11,1238°; (24 − 11,1238) / 48 × 60 = 16,0953; Güneş'te iki katı: 32,1905.
- Ay ayana: λ = 1°; δ = asin(sin 23,45° × sin λ) = 0,3979°; (24 − 0,3979) / 48 × 60 = 29,5026.
- Mars ayana: λ = 31°; δ = asin(sin 23,45° × sin λ) = 11,8271°; (24 + 11,8271) / 48 × 60 = 44,7839.
- Merkür ayana: λ = 307°; δ = asin(sin 23,45° × sin λ) = -18,5309°; (24 + 18,5309) / 48 × 60 = 53,1636.
- Jüpiter ayana: λ = 307,5°; δ = asin(sin 23,45° × sin λ) = -18,4039°; (24 − 18,4039) / 48 × 60 = 6,9951.
- Venüs ayana: λ = 210°; δ = asin(sin 23,45° × sin λ) = -11,477°; (24 − 11,477) / 48 × 60 = 15,6538.
- Satürn ayana: λ = 340°; δ = asin(sin 23,45° × sin λ) = -7,8226°; (24 + 7,8226) / 48 × 60 = 39,7783.
- Güneş zaman gücü = 24 + 50 + 0 + 0 + 30 + 0 + 0 + 32,1905 = 136,1905.
- Ay zaman gücü = 36 + 20 + 0 + 15 + 0 + 0 + 0 + 29,5026 = 100,5026.
- Mars zaman gücü = 36 + 50 + 60 + 0 + 0 + 0 + 0 + 44,7839 = 190,7839.
- Merkür zaman gücü = 60 + 10 + 0 + 0 + 0 + 0 + 60 + 53,1636 = 183,1636.
- Jüpiter zaman gücü = 24 + 10 + 60 + 0 + 0 + 0 + 0 + 6,9951 = 100,9951.
- Venüs zaman gücü = 24 + 10 + 0 + 0 + 0 + 45 + 0 + 15,6538 = 94,6538.
- Satürn zaman gücü = 36 + 50 + 0 + 0 + 0 + 0 + 0 + 39,7783 = 125,7783.

**Cheshta bala**

- Güneş: ayana (iki katı alınmamış) 16,0953. Ay: e/3 = 10.
- Mars: orta(ortalama Mars 20°, gerçek 31°) = 25,5°; K = (ortalama Güneş 329° − 25,5°) mod 360 = 303,5° → 360 − 303,5 = 56,5°; 56,5 / 3 = 18,8333.
- Jüpiter: orta(ortalama Jüpiter 310°, gerçek 307,5°) = 308,75°; K = (ortalama Güneş 329° − 308,75°) mod 360 = 20,25°; 20,25 / 3 = 6,75.
- Satürn: orta(ortalama Satürn 338°, gerçek 340°) = 339°; K = (ortalama Güneş 329° − 339°) mod 360 = 350° → 360 − 350 = 10°; 10 / 3 = 3,3333.
- Merkür: orta(ortalama Güneş 329°, gerçek 307°) = 318°; K = (Merkür'ün güneş merkezli ortalaması 180° − 318°) mod 360 = 222° → 360 − 222 = 138°; 138 / 3 = 46.
- Venüs: orta(ortalama Güneş 329°, gerçek 210°) = 269,5°; K = (Venüs'ün güneş merkezli ortalaması 260° − 269,5°) mod 360 = 350,5° → 360 − 350,5 = 9,5°; 9,5 / 3 = 3,1667.

**Naisargika bala**

- Güneş 60 × 7/7 = 60; Ay 60 × 6/7 = 51,4286; Mars 60 × 2/7 = 17,1429; Merkür 60 × 3/7 = 25,7143; Jüpiter 60 × 4/7 = 34,2857; Venüs 60 × 5/7 = 42,8571; Satürn 60 × 1/7 = 8,5714.

**Drik bala**

- Güneş (307°): Venüs açı 121° → 29 (iyicil: 7,25). Drik = 7,25.
- Ay (337°): Merkür açı 54° → 12 (iyicil: 3, tamamı +12); Jüpiter açı 53,5° → 11,75 (iyicil: 2,9375, tamamı +11,75); Venüs açı 151° → 2 (iyicil: 0,5). Drik = 30,1875.
- Mars (7°): Güneş açı 60° → 15 (kötücül: -3,75); Merkür açı 84° → 39 (iyicil: 9,75, tamamı +39); Jüpiter açı 83,5° → 38,5 (iyicil: 9,625, tamamı +38,5); Venüs açı 181° → 59,5 (iyicil: 14,875); Satürn açı 51° → 10,5 (kötücül: -2,625). Drik = 105,375.
- Merkür (283°): Mars açı 276° → 12 (kötücül: -3); Venüs açı 97° → 41,5 (iyicil: 10,375). Drik = 7,375.
- Jüpiter (283,5°): Mars açı 276,5° → 11,75 (kötücül: -2,9375); Venüs açı 97,5° → 41,25 (iyicil: 10,3125). Drik = 7,375.
- Venüs (186°): Güneş açı 239° → 30,5 (kötücül: -7,625); Ay açı 209° → 45,5 (iyicil: 11,375); Mars açı 179° → 58 (kötücül: -14,5); Merkür açı 263° → 18,5 (iyicil: 4,625, tamamı +18,5); Jüpiter açı 262,5° → 48,75 (iyicil: 12,1875, tamamı +48,75); Satürn açı 230° → 35 (kötücül: -8,75). Drik = 64,5625.
- Satürn (316°): Merkür açı 33° → 1,5 (iyicil: 0,375, tamamı +1,5); Jüpiter açı 32,5° → 1,25 (iyicil: 0,3125, tamamı +1,25); Venüs açı 130° → 20 (iyicil: 5). Drik = 8,4375.

**Toplam, oran, sıra, savaş**

- Güneş: 195 + 14,3333 + 136,1905 + 16,0953 + 60 + 7,25 = 428,8691 virupa = 7,1478 rupa; gerekli 390; oran 1,0997; sıra 4.
- Ay: 239,3333 + 55,6667 + 100,5026 + 10 + 51,4286 + 30,1875 = 487,1187 virupa = 8,1186 rupa; gerekli 360; oran 1,3531; sıra 3.
- Mars: 246 + 5,6667 + 190,7839 + 18,8333 + 17,1429 + 105,375 = 583,8017 virupa = 9,73 rupa; gerekli 300; oran 1,946; sıra 1.
- Merkür: 194,6667 + 48,6667 + 183,1636 + 46 + 25,7143 + 7,375 = 505,5862 virupa = 8,4264 rupa; gerekli 420; oran 1,2038; sıra 2; savaş: Jüpiter ile (boylam farkı 0,5°; enlemler 0° ve 0°), kazanan Merkür.
- Jüpiter: 129,8333 + 48,5 + 100,9951 + 6,75 + 34,2857 + 7,375 = 327,7392 virupa = 5,4623 rupa; gerekli 390; oran 0,8404; sıra 7; savaş: Merkür ile (boylam farkı 0,5°; enlemler 0° ve 0°), kazanan Merkür.
- Venüs: 158 + 5,3333 + 94,6538 + 3,1667 + 42,8571 + 64,5625 = 368,5734 virupa = 6,1429 rupa; gerekli 330; oran 1,1169; sıra 6.
- Satürn: 213,3333 + 22,3333 + 125,7783 + 3,3333 + 8,5714 + 8,4375 = 381,7872 virupa = 6,3631 rupa; gerekli 300; oran 1,2726; sıra 5.

**Bhava bala**

- 1. bhava: orta 249° (Yay 9°, insan; güçsüz bhava 7, 6 bhava = 180° → dig 60); lord Jüpiter 327,7392; drishti: Ay açı 272° → 14 (iyicil: 3,5); Mars açı 242° → 29 (kötücül: -7,25); Venüs açı 63° → 18 (iyicil: 4,5); Satürn açı 293° → 48,5 (kötücül: -12,125) = -11,375. Toplam 376,3642.
- 2. bhava: orta 279° (Oğlak 9°, dört ayaklı; güçsüz bhava 4, 2 bhava = 60° → dig 20); lord Satürn 381,7872; drishti: Mars açı 272° → 14 (kötücül: -3,5); Venüs açı 93° → 43,5 (iyicil: 10,875) = 7,375. Toplam 409,1622.
- 3. bhava: orta 309° (Kova 9°, insan; güçsüz bhava 7, 4 bhava = 120° → dig 40); lord Satürn 381,7872; drishti: Venüs açı 123° → 27 (iyicil: 6,75) = 6,75. Toplam 428,5372.
- 4. bhava: orta 339° (Balık 9°, su; güçsüz bhava 10, 6 bhava = 180° → dig 60); lord Jüpiter 327,7392; drishti: Güneş açı 32° → 1 (kötücül: -0,25); Merkür açı 56° → 13 (iyicil: 3,25, tamamı +13); Jüpiter açı 55,5° → 12,75 (iyicil: 3,1875, tamamı +12,75); Venüs açı 153° → 6 (iyicil: 1,5) = 33,4375. Toplam 421,1767.
- 5. bhava: orta 9° (Koç 9°, dört ayaklı; güçsüz bhava 4, 1 bhava = 30° → dig 10); lord Mars 583,8017; drishti: Güneş açı 62° → 17 (kötücül: -4,25); Ay açı 32° → 1 (iyicil: 0,25); Merkür açı 86° → 41 (iyicil: 10,25, tamamı +41); Jüpiter açı 85,5° → 40,5 (iyicil: 10,125, tamamı +40,5); Venüs açı 183° → 58,5 (iyicil: 14,625); Satürn açı 53° → 11,5 (kötücül: -2,875) = 109,625. Toplam 703,4267.
- 6. bhava: orta 39° (Boğa 9°, dört ayaklı; güçsüz bhava 4, 2 bhava = 60° → dig 20); lord Venüs 368,5734; drishti: Güneş açı 92° → 44 (kötücül: -11); Ay açı 62° → 17 (iyicil: 4,25); Mars açı 32° → 1 (kötücül: -0,25); Merkür açı 116° → 32 (iyicil: 8, tamamı +32); Jüpiter açı 115,5° → 32,25 (iyicil: 8,0625, tamamı +32,25); Venüs açı 213° → 43,5 (iyicil: 10,875); Satürn açı 83° → 60 (kötücül: -15) = 69,1875. Toplam 457,7609.
- 7. bhava: orta 69° (İkizler 9°, insan; güçsüz bhava 7, 0 bhava = 0° → dig 0); lord Merkür 505,5862; drishti: Güneş açı 122° → 28 (kötücül: -7); Ay açı 92° → 44 (iyicil: 11); Mars açı 62° → 17 (kötücül: -4,25); Merkür açı 146° → 4 (iyicil: 1, tamamı +4); Jüpiter açı 145,5° → 34,5 (iyicil: 8,625, tamamı +34,5); Venüs açı 243° → 28,5 (iyicil: 7,125); Satürn açı 113° → 33,5 (kötücül: -8,375) = 46,625. Toplam 552,2112.
- 8. bhava: orta 99° (Yengeç 9°, su; güçsüz bhava 10, 2 bhava = 60° → dig 20); lord Ay 487,1187; drishti: Güneş açı 152° → 4 (kötücül: -1); Ay açı 122° → 28 (iyicil: 7); Mars açı 92° → 59 (kötücül: -14,75); Merkür açı 176° → 52 (iyicil: 13, tamamı +52); Jüpiter açı 175,5° → 51 (iyicil: 12,75, tamamı +51); Venüs açı 273° → 13,5 (iyicil: 3,375); Satürn açı 143° → 7 (kötücül: -1,75) = 121,625. Toplam 628,7437.
- 9. bhava: orta 129° (Aslan 9°, dört ayaklı; güçsüz bhava 4, 5 bhava = 150° → dig 50); lord Güneş 428,8691; drishti: Güneş açı 182° → 59 (kötücül: -14,75); Ay açı 152° → 4 (iyicil: 1); Mars açı 122° → 28 (kötücül: -7); Merkür açı 206° → 47 (iyicil: 11,75, tamamı +47); Jüpiter açı 205,5° → 47,25 (iyicil: 11,8125, tamamı +47,25); Satürn açı 173° → 46 (kötücül: -11,5) = 85,5625. Toplam 564,4316.
- 10. bhava: orta 159° (Başak 9°, insan; güçsüz bhava 7, 3 bhava = 90° → dig 30); lord Merkür 505,5862; drishti: Güneş açı 212° → 44 (kötücül: -11); Ay açı 182° → 59 (iyicil: 14,75); Mars açı 152° → 4 (kötücül: -1); Merkür açı 236° → 32 (iyicil: 8, tamamı +32); Jüpiter açı 235,5° → 32,25 (iyicil: 8,0625, tamamı +32,25); Satürn açı 203° → 48,5 (kötücül: -12,125) = 70,9375. Toplam 606,5237.
- 11. bhava: orta 189° (Terazi 9°, insan; güçsüz bhava 7, 4 bhava = 120° → dig 40); lord Venüs 368,5734; drishti: Güneş açı 242° → 29 (kötücül: -7,25); Ay açı 212° → 44 (iyicil: 11); Mars açı 182° → 59 (kötücül: -14,75); Merkür açı 266° → 17 (iyicil: 4,25, tamamı +17); Jüpiter açı 265,5° → 47,25 (iyicil: 11,8125, tamamı +47,25); Satürn açı 233° → 33,5 (kötücül: -8,375) = 60,9375. Toplam 469,5109.
- 12. bhava: orta 219° (Akrep 9°, Akrep; güçsüz bhava 1, 1 bhava = 30° → dig 10); lord Mars 583,8017; drishti: Güneş açı 272° → 14 (kötücül: -3,5); Ay açı 242° → 29 (iyicil: 7,25); Mars açı 212° → 59 (kötücül: -14,75); Merkür açı 296° → 2 (iyicil: 0,5, tamamı +2); Jüpiter açı 295,5° → 2,25 (iyicil: 0,5625, tamamı +2,25); Venüs açı 33° → 1,5 (iyicil: 0,375); Satürn açı 263° → 18,5 (kötücül: -4,625) = -9,9375. Toplam 583,8642.
- Sıra (büyükten küçüğe): 5, 8, 10, 12, 9, 7, 11, 6, 3, 4, 2, 1.
````

**Yeni dosya:** `tests/golden/modules/shadbala/day-birth-at-noon-near-full-moon.json`

````json
{
  "id": "day-birth-at-noon-near-full-moon",
  "source": "hand",
  "rule_ref": "BPHS, graha gücü ve bhava gücü bölümleri; ileri modüller tasarım belgesi §7",
  "derivation": [
    "Adım adım aritmetik: WORKSHEET.md, \"Örnek 1 — öğle doğumu, dolunaya yakın Ay, Merkür kötücül\".",
    "Güneş: 278,6667 + 28,3333 + 264,3191 + 46,4929 + 60 + 8,875 = 686,6869 virupa = 11,4448 rupa; gerekli 390; oran 1,7607; sıra 1.",
    "Ay: 156,6667 + 34,6667 + 145,2051 + 53,6667 + 51,4286 + 36,625 = 478,2587 virupa = 7,971 rupa; gerekli 360; oran 1,3285; sıra 6.",
    "Mars: 274,3333 + 53,3333 + 107,8074 + 26 + 17,1429 + 68,25 = 546,8669 virupa = 9,1144 rupa; gerekli 300; oran 1,8229; sıra 4.",
    "Merkür: 199 + 54,6667 + 117,8352 + 19,8333 + 25,7143 + -5,25 = 411,7995 virupa = 6,8633 rupa; gerekli 420; oran 0,9805; sıra 7.",
    "Jüpiter: 239,3333 + 30,3333 + 243,5972 + 30 + 34,2857 + 11,5 = 589,0496 virupa = 9,8175 rupa; gerekli 390; oran 1,5104; sıra 2.",
    "Venüs: 192 + 25 + 151,0564 + 28 + 42,8571 + 75,125 = 514,0385 virupa = 8,5673 rupa; gerekli 330; oran 1,5577; sıra 5.",
    "Satürn: 298,6667 + 57,3333 + 54,8593 + 58,5 + 8,5714 + 76,125 = 554,0557 virupa = 9,2343 rupa; gerekli 300; oran 1,8469; sıra 3.",
    "Sıra (büyükten küçüğe): 8, 10, 5, 12, 11, 9, 1, 7, 4, 2, 3, 6."
  ],
  "input": {
    "lagna_lon": 8.0,
    "grahas": {"Sun": 11.0, "Moon": 172.0, "Mars": 296.0, "Mercury": 24.0, "Jupiter": 97.0, "Venus": 351.0, "Saturn": 196.0, "Rahu": 47.0, "Ketu": 227.0},
    "astro": {
      "jd_ut": 0.5, "sunrise_jd_ut": 0.25, "sunset_jd_ut": 0.75, "next_sunrise_jd_ut": 1.25, "mc_lon": 276.0,
      "tropical_lons": [35.0, 196.0, 320.0, 48.0, 121.0, 15.0, 220.0],
      "mean": {"sun": 33.0, "mercury": 100.0, "venus": 300.0, "mars": 310.0, "jupiter": 125.0, "saturn": 215.0},
      "weekday": "Sunday", "year_entry_weekday": "Thursday", "month_entry_weekday": "Tuesday"
    }
  },
  "expected": {
    "grahas": {
      "Sun": {"uccha": 59.6667, "saptavargaja": 144, "ojayugma": 15, "kendradi": 60, "drekkana": 0, "sthana": 278.6667, "dig": 28.3333, "day_night": 60.0, "paksha": 6.3333, "tribhaga": 60, "year_lord": 0, "month_lord": 0, "weekday_lord": 45, "hora_lord": 0, "ayana": 92.9857, "temporal": 264.3191, "cheshta": 46.4929, "naisargika": 60.0, "drik": 8.875, "total": 686.6869, "rank": 1, "war": null},
      "Moon": {"uccha": 13.6667, "saptavargaja": 83, "ojayugma": 30, "kendradi": 15, "drekkana": 15, "sthana": 156.6667, "dig": 34.6667, "day_night": 0.0, "paksha": 107.3333, "tribhaga": 0, "year_lord": 0, "month_lord": 0, "weekday_lord": 0, "hora_lord": 0, "ayana": 37.8718, "temporal": 145.2051, "cheshta": 53.6667, "naisargika": 51.4286, "drik": 36.625, "total": 478.2587, "rank": 6, "war": null},
      "Mars": {"uccha": 59.3333, "saptavargaja": 140, "ojayugma": 15, "kendradi": 60, "drekkana": 0, "sthana": 274.3333, "dig": 53.3333, "day_night": 0.0, "paksha": 6.3333, "tribhaga": 0, "year_lord": 0, "month_lord": 30, "weekday_lord": 0, "hora_lord": 60, "ayana": 11.474, "temporal": 107.8074, "cheshta": 26.0, "naisargika": 17.1429, "drik": 68.25, "total": 546.8669, "rank": 4, "war": null},
      "Mercury": {"uccha": 13.0, "saptavargaja": 111, "ojayugma": 15, "kendradi": 60, "drekkana": 0, "sthana": 199.0, "dig": 54.6667, "day_night": 60, "paksha": 6.3333, "tribhaga": 0, "year_lord": 0, "month_lord": 0, "weekday_lord": 0, "hora_lord": 0, "ayana": 51.5019, "temporal": 117.8352, "cheshta": 19.8333, "naisargika": 25.7143, "drik": -5.25, "total": 411.7995, "rank": 7, "war": null},
      "Jupiter": {"uccha": 59.3333, "saptavargaja": 105, "ojayugma": 0, "kendradi": 60, "drekkana": 15, "sthana": 239.3333, "dig": 30.3333, "day_night": 60.0, "paksha": 53.6667, "tribhaga": 60, "year_lord": 15, "month_lord": 0, "weekday_lord": 0, "hora_lord": 0, "ayana": 54.9305, "temporal": 243.5972, "cheshta": 30.0, "naisargika": 34.2857, "drik": 11.5, "total": 589.0496, "rank": 2, "war": null},
      "Venus": {"uccha": 58.0, "saptavargaja": 74, "ojayugma": 30, "kendradi": 15, "drekkana": 15, "sthana": 192.0, "dig": 25.0, "day_night": 60.0, "paksha": 53.6667, "tribhaga": 0, "year_lord": 0, "month_lord": 0, "weekday_lord": 0, "hora_lord": 0, "ayana": 37.3897, "temporal": 151.0564, "cheshta": 28.0, "naisargika": 42.8571, "drik": 75.125, "total": 514.0385, "rank": 5, "war": null},
      "Saturn": {"uccha": 58.6667, "saptavargaja": 135, "ojayugma": 30, "kendradi": 60, "drekkana": 15, "sthana": 298.6667, "dig": 57.3333, "day_night": 0.0, "paksha": 6.3333, "tribhaga": 0, "year_lord": 0, "month_lord": 0, "weekday_lord": 0, "hora_lord": 0, "ayana": 48.526, "temporal": 54.8593, "cheshta": 58.5, "naisargika": 8.5714, "drik": 76.125, "total": 554.0557, "rank": 3, "war": null}
    },
    "time_lords": {"year": "Jupiter", "month": "Mars", "weekday": "Sun", "hora": "Mars", "tribhaga": "Sun"},
    "bhavas": [
      {"house": 1, "lord": "Mars", "lord_bala": 546.8669, "dig": 30.0, "drishti": 13.375, "total": 590.2419, "rank": 7},
      {"house": 2, "lord": "Venus", "lord_bala": 514.0385, "dig": 20.0, "drishti": -14.375, "total": 519.6635, "rank": 10},
      {"house": 3, "lord": "Mercury", "lord_bala": 411.7995, "dig": 40.0, "drishti": 2.375, "total": 454.1745, "rank": 11},
      {"house": 4, "lord": "Moon", "lord_bala": 478.2587, "dig": 60.0, "drishti": 11.375, "total": 549.6337, "rank": 9},
      {"house": 5, "lord": "Sun", "lord_bala": 686.6869, "dig": 10.0, "drishti": -1.25, "total": 695.4369, "rank": 3},
      {"house": 6, "lord": "Mercury", "lord_bala": 411.7995, "dig": 10.0, "drishti": 26.25, "total": 448.0495, "rank": 12},
      {"house": 7, "lord": "Venus", "lord_bala": 514.0385, "dig": 0.0, "drishti": 70.0, "total": 584.0385, "rank": 8},
      {"house": 8, "lord": "Mars", "lord_bala": 546.8669, "dig": 50.0, "drishti": 110.75, "total": 707.6169, "rank": 1},
      {"house": 9, "lord": "Jupiter", "lord_bala": 589.0496, "dig": 20.0, "drishti": 33.5, "total": 642.5496, "rank": 6},
      {"house": 10, "lord": "Saturn", "lord_bala": 554.0557, "dig": 60.0, "drishti": 83.375, "total": 697.4307, "rank": 2},
      {"house": 11, "lord": "Saturn", "lord_bala": 554.0557, "dig": 40.0, "drishti": 56.25, "total": 650.3057, "rank": 5},
      {"house": 12, "lord": "Jupiter", "lord_bala": 589.0496, "dig": 20.0, "drishti": 78.875, "total": 687.9246, "rank": 4}
    ]
  }
}
````

**Yeni dosya:** `tests/golden/modules/shadbala/night-birth-graha-war-waning-moon.json`

````json
{
  "id": "night-birth-graha-war-waning-moon",
  "source": "hand",
  "rule_ref": "BPHS, graha gücü ve bhava gücü bölümleri; ileri modüller tasarım belgesi §7",
  "derivation": [
    "Adım adım aritmetik: WORKSHEET.md, \"Örnek 2 — gece doğumu, eşit olmayan gün, küçülen Ay, Mars–Venüs savaşı, Akrep lagna\".",
    "Güneş: 188 + 32,3333 + 31,6122 + 3,1394 + 60 + -17,375 = 297,71 virupa = 4,9618 rupa; gerekli 390; oran 0,7634; sıra 6.",
    "Ay: 186,3333 + 13 + 201,5521 + 40,6667 + 51,4286 + 84,9375 = 577,9182 virupa = 9,632 rupa; gerekli 360; oran 1,6053; sıra 1.",
    "Mars: 174 + 34,6667 + 132,6269 + 55,6667 + 17,1429 + 60,875 = 474,9781 virupa = 7,9163 rupa; gerekli 300; oran 1,5833; sıra 3; savaş: Venüs ile (boylam farkı 0,5°; enlemler 0,8° ve -0,3°), kazanan Mars.",
    "Merkür: 169,3333 + 57 + 213,1745 + 49 + 25,7143 + -12,5 = 501,7221 virupa = 8,362 rupa; gerekli 420; oran 1,1946; sıra 2.",
    "Jüpiter: 91 + 42 + 112,8854 + 18,8333 + 34,2857 + -30,6875 = 268,317 virupa = 4,4719 rupa; gerekli 390; oran 0,688; sıra 7.",
    "Venüs: 132,5 + 25,1667 + 165,9685 + 5,75 + 42,8571 + 60,125 = 432,3673 virupa = 7,2061 rupa; gerekli 330; oran 1,3102; sıra 4; savaş: Mars ile (boylam farkı 0,5°; enlemler -0,3° ve 0,8°), kazanan Mars.",
    "Satürn: 117,3333 + 45,6667 + 104,8739 + 47,8333 + 8,5714 + 57,1875 = 381,4661 virupa = 6,3578 rupa; gerekli 300; oran 1,2716; sıra 5.",
    "Sıra (büyükten küçüğe): 9, 11, 6, 7, 8, 12, 1, 4, 10, 3, 5, 2."
  ],
  "input": {
    "lagna_lon": 227.0,
    "grahas": {"Sun": 223.0, "Moon": 101.0, "Mars": 64.0, "Mercury": 236.0, "Jupiter": 281.0, "Venus": 64.5, "Saturn": 4.0, "Rahu": 133.0, "Ketu": 313.0},
    "lats": {"Mars": 0.8, "Venus": -0.3},
    "astro": {
      "jd_ut": 0.95, "sunrise_jd_ut": 0.3, "sunset_jd_ut": 0.7, "next_sunrise_jd_ut": 1.31, "mc_lon": 140.0,
      "tropical_lons": [247.0, 125.0, 88.0, 260.0, 305.0, 88.5, 28.0],
      "mean": {"sun": 246.0, "mercury": 40.0, "venus": 150.0, "mars": 70.0, "jupiter": 300.0, "saturn": 31.0},
      "weekday": "Wednesday", "year_entry_weekday": "Saturday", "month_entry_weekday": "Wednesday"
    }
  },
  "expected": {
    "grahas": {
      "Sun": {"uccha": 11.0, "saptavargaja": 102, "ojayugma": 15, "kendradi": 60, "drekkana": 0, "sthana": 188.0, "dig": 32.3333, "day_night": 6.0, "paksha": 19.3333, "tribhaga": 0, "year_lord": 0, "month_lord": 0, "weekday_lord": 0, "hora_lord": 0, "ayana": 6.2789, "temporal": 31.6122, "cheshta": 3.1394, "naisargika": 60.0, "drik": -17.375, "total": 297.71, "rank": 6, "war": null},
      "Moon": {"uccha": 37.3333, "saptavargaja": 119, "ojayugma": 15, "kendradi": 15, "drekkana": 0, "sthana": 186.3333, "dig": 13.0, "day_night": 54.0, "paksha": 81.3333, "tribhaga": 0, "year_lord": 0, "month_lord": 0, "weekday_lord": 0, "hora_lord": 60, "ayana": 6.2188, "temporal": 201.5521, "cheshta": 40.6667, "naisargika": 51.4286, "drik": 84.9375, "total": 577.9182, "rank": 1, "war": null},
      "Mars": {"uccha": 18.0, "saptavargaja": 96, "ojayugma": 15, "kendradi": 30, "drekkana": 15, "sthana": 174.0, "dig": 34.6667, "day_night": 54.0, "paksha": 19.3333, "tribhaga": 0, "year_lord": 0, "month_lord": 0, "weekday_lord": 0, "hora_lord": 0, "ayana": 59.2936, "temporal": 132.6269, "cheshta": 55.6667, "naisargika": 17.1429, "drik": 60.875, "total": 474.9781, "rank": 3, "war": {"with": "Venus", "winner": "Mars"}},
      "Mercury": {"uccha": 36.3333, "saptavargaja": 58, "ojayugma": 15, "kendradi": 60, "drekkana": 0, "sthana": 169.3333, "dig": 57.0, "day_night": 60, "paksha": 19.3333, "tribhaga": 0, "year_lord": 0, "month_lord": 30, "weekday_lord": 45, "hora_lord": 0, "ayana": 58.8412, "temporal": 213.1745, "cheshta": 49.0, "naisargika": 25.7143, "drik": -12.5, "total": 501.7221, "rank": 2, "war": null},
      "Jupiter": {"uccha": 2.0, "saptavargaja": 59, "ojayugma": 15, "kendradi": 15, "drekkana": 0, "sthana": 91.0, "dig": 42.0, "day_night": 6.0, "paksha": 40.6667, "tribhaga": 60, "year_lord": 0, "month_lord": 0, "weekday_lord": 0, "hora_lord": 0, "ayana": 6.2188, "temporal": 112.8854, "cheshta": 18.8333, "naisargika": 34.2857, "drik": -30.6875, "total": 268.317, "rank": 7, "war": null},
      "Venus": {"uccha": 37.5, "saptavargaja": 50, "ojayugma": 15, "kendradi": 30, "drekkana": 0, "sthana": 132.5, "dig": 25.1667, "day_night": 6.0, "paksha": 40.6667, "tribhaga": 60, "year_lord": 0, "month_lord": 0, "weekday_lord": 0, "hora_lord": 0, "ayana": 59.3019, "temporal": 165.9685, "cheshta": 5.75, "naisargika": 42.8571, "drik": 60.125, "total": 432.3673, "rank": 4, "war": {"with": "Mars", "winner": "Mars"}},
      "Saturn": {"uccha": 5.3333, "saptavargaja": 82, "ojayugma": 15, "kendradi": 15, "drekkana": 0, "sthana": 117.3333, "dig": 45.6667, "day_night": 54.0, "paksha": 19.3333, "tribhaga": 0, "year_lord": 15, "month_lord": 0, "weekday_lord": 0, "hora_lord": 0, "ayana": 16.5405, "temporal": 104.8739, "cheshta": 47.8333, "naisargika": 8.5714, "drik": 57.1875, "total": 381.4661, "rank": 5, "war": null}
    },
    "time_lords": {"year": "Saturn", "month": "Mercury", "weekday": "Mercury", "hora": "Moon", "tribhaga": "Venus"},
    "bhavas": [
      {"house": 1, "lord": "Mars", "lord_bala": 474.9781, "dig": 0.0, "drishti": -15.875, "total": 459.1031, "rank": 7},
      {"house": 2, "lord": "Jupiter", "lord_bala": 268.317, "dig": 20.0, "drishti": -9.3125, "total": 279.0045, "rank": 12},
      {"house": 3, "lord": "Saturn", "lord_bala": 381.4661, "dig": 50.0, "drishti": -28.1875, "total": 403.2786, "rank": 10},
      {"house": 4, "lord": "Saturn", "lord_bala": 381.4661, "dig": 30.0, "drishti": 9.5625, "total": 421.0286, "rank": 8},
      {"house": 5, "lord": "Jupiter", "lord_bala": 268.317, "dig": 50.0, "drishti": 38.9375, "total": 357.2545, "rank": 11},
      {"house": 6, "lord": "Mars", "lord_bala": 474.9781, "dig": 20.0, "drishti": 54.25, "total": 549.2281, "rank": 3},
      {"house": 7, "lord": "Venus", "lord_bala": 432.3673, "dig": 30.0, "drishti": 82.875, "total": 545.2423, "rank": 4},
      {"house": 8, "lord": "Mercury", "lord_bala": 501.7221, "dig": 10.0, "drishti": 26.375, "total": 538.0971, "rank": 5},
      {"house": 9, "lord": "Moon", "lord_bala": 577.9182, "dig": 10.0, "drishti": 80.4375, "total": 668.3557, "rank": 1},
      {"house": 10, "lord": "Sun", "lord_bala": 297.71, "dig": 60.0, "drishti": 58.75, "total": 416.46, "rank": 9},
      {"house": 11, "lord": "Mercury", "lord_bala": 501.7221, "dig": 40.0, "drishti": 59.1875, "total": 600.9096, "rank": 2},
      {"house": 12, "lord": "Venus", "lord_bala": 432.3673, "dig": 50.0, "drishti": -8.75, "total": 473.6173, "rank": 6}
    ]
  }
}
````

**Yeni dosya:** `tests/golden/modules/shadbala/pre-dawn-birth-moolatrikona-equal-latitude-war.json`

````json
{
  "id": "pre-dawn-birth-moolatrikona-equal-latitude-war",
  "source": "hand",
  "rule_ref": "BPHS, graha gücü ve bhava gücü bölümleri; ileri modüller tasarım belgesi §7",
  "derivation": [
    "Adım adım aritmetik: WORKSHEET.md, \"Örnek 3 — şafaktan önce doğum (öğleye uzaklık 12 saati aşar), moolatrikona grahalar, eşit enlemli Merkür–Jüpiter savaşı, Yay lagna (ilk yarı)\".",
    "Güneş: 195 + 14,3333 + 136,1905 + 16,0953 + 60 + 7,25 = 428,8691 virupa = 7,1478 rupa; gerekli 390; oran 1,0997; sıra 4.",
    "Ay: 239,3333 + 55,6667 + 100,5026 + 10 + 51,4286 + 30,1875 = 487,1187 virupa = 8,1186 rupa; gerekli 360; oran 1,3531; sıra 3.",
    "Mars: 246 + 5,6667 + 190,7839 + 18,8333 + 17,1429 + 105,375 = 583,8017 virupa = 9,73 rupa; gerekli 300; oran 1,946; sıra 1.",
    "Merkür: 194,6667 + 48,6667 + 183,1636 + 46 + 25,7143 + 7,375 = 505,5862 virupa = 8,4264 rupa; gerekli 420; oran 1,2038; sıra 2; savaş: Jüpiter ile (boylam farkı 0,5°; enlemler 0° ve 0°), kazanan Merkür.",
    "Jüpiter: 129,8333 + 48,5 + 100,9951 + 6,75 + 34,2857 + 7,375 = 327,7392 virupa = 5,4623 rupa; gerekli 390; oran 0,8404; sıra 7; savaş: Merkür ile (boylam farkı 0,5°; enlemler 0° ve 0°), kazanan Merkür.",
    "Venüs: 158 + 5,3333 + 94,6538 + 3,1667 + 42,8571 + 64,5625 = 368,5734 virupa = 6,1429 rupa; gerekli 330; oran 1,1169; sıra 6.",
    "Satürn: 213,3333 + 22,3333 + 125,7783 + 3,3333 + 8,5714 + 8,4375 = 381,7872 virupa = 6,3631 rupa; gerekli 300; oran 1,2726; sıra 5.",
    "Sıra (büyükten küçüğe): 5, 8, 10, 12, 9, 7, 11, 6, 3, 4, 2, 1."
  ],
  "input": {
    "lagna_lon": 249.0,
    "grahas": {"Sun": 307.0, "Moon": 337.0, "Mars": 7.0, "Mercury": 283.0, "Jupiter": 283.5, "Venus": 186.0, "Saturn": 316.0, "Rahu": 21.0, "Ketu": 201.0},
    "astro": {
      "jd_ut": 1.2, "sunrise_jd_ut": 0.27, "sunset_jd_ut": 0.73, "next_sunrise_jd_ut": 1.27, "mc_lon": 170.0,
      "tropical_lons": [331.0, 1.0, 31.0, 307.0, 307.5, 210.0, 340.0],
      "mean": {"sun": 329.0, "mercury": 180.0, "venus": 260.0, "mars": 20.0, "jupiter": 310.0, "saturn": 338.0},
      "weekday": "Friday", "year_entry_weekday": "Monday", "month_entry_weekday": "Sunday"
    }
  },
  "expected": {
    "grahas": {
      "Sun": {"uccha": 39.0, "saptavargaja": 96, "ojayugma": 30, "kendradi": 15, "drekkana": 15, "sthana": 195.0, "dig": 14.3333, "day_night": 24.0, "paksha": 50.0, "tribhaga": 0, "year_lord": 0, "month_lord": 30, "weekday_lord": 0, "hora_lord": 0, "ayana": 32.1905, "temporal": 136.1905, "cheshta": 16.0953, "naisargika": 60.0, "drik": 7.25, "total": 428.8691, "rank": 4, "war": null},
      "Moon": {"uccha": 41.3333, "saptavargaja": 108, "ojayugma": 30, "kendradi": 60, "drekkana": 0, "sthana": 239.3333, "dig": 55.6667, "day_night": 36.0, "paksha": 20.0, "tribhaga": 0, "year_lord": 15, "month_lord": 0, "weekday_lord": 0, "hora_lord": 0, "ayana": 29.5026, "temporal": 100.5026, "cheshta": 10.0, "naisargika": 51.4286, "drik": 30.1875, "total": 487.1187, "rank": 3, "war": null},
      "Mars": {"uccha": 37.0, "saptavargaja": 134, "ojayugma": 30, "kendradi": 30, "drekkana": 15, "sthana": 246.0, "dig": 5.6667, "day_night": 36.0, "paksha": 50.0, "tribhaga": 60, "year_lord": 0, "month_lord": 0, "weekday_lord": 0, "hora_lord": 0, "ayana": 44.7839, "temporal": 190.7839, "cheshta": 18.8333, "naisargika": 17.1429, "drik": 105.375, "total": 583.8017, "rank": 1, "war": null},
      "Mercury": {"uccha": 20.6667, "saptavargaja": 114, "ojayugma": 15, "kendradi": 30, "drekkana": 15, "sthana": 194.6667, "dig": 48.6667, "day_night": 60, "paksha": 10.0, "tribhaga": 0, "year_lord": 0, "month_lord": 0, "weekday_lord": 0, "hora_lord": 60, "ayana": 53.1636, "temporal": 183.1636, "cheshta": 46.0, "naisargika": 25.7143, "drik": 7.375, "total": 505.5862, "rank": 2, "war": {"with": "Jupiter", "winner": "Mercury"}},
      "Jupiter": {"uccha": 2.8333, "saptavargaja": 97, "ojayugma": 0, "kendradi": 30, "drekkana": 0, "sthana": 129.8333, "dig": 48.5, "day_night": 24.0, "paksha": 10.0, "tribhaga": 60, "year_lord": 0, "month_lord": 0, "weekday_lord": 0, "hora_lord": 0, "ayana": 6.9951, "temporal": 100.9951, "cheshta": 6.75, "naisargika": 34.2857, "drik": 7.375, "total": 327.7392, "rank": 7, "war": {"with": "Mercury", "winner": "Mercury"}},
      "Venus": {"uccha": 3.0, "saptavargaja": 110, "ojayugma": 15, "kendradi": 30, "drekkana": 0, "sthana": 158.0, "dig": 5.3333, "day_night": 24.0, "paksha": 10.0, "tribhaga": 0, "year_lord": 0, "month_lord": 0, "weekday_lord": 45, "hora_lord": 0, "ayana": 15.6538, "temporal": 94.6538, "cheshta": 3.1667, "naisargika": 42.8571, "drik": 64.5625, "total": 368.5734, "rank": 6, "war": null},
      "Saturn": {"uccha": 21.3333, "saptavargaja": 132, "ojayugma": 30, "kendradi": 15, "drekkana": 15, "sthana": 213.3333, "dig": 22.3333, "day_night": 36.0, "paksha": 50.0, "tribhaga": 0, "year_lord": 0, "month_lord": 0, "weekday_lord": 0, "hora_lord": 0, "ayana": 39.7783, "temporal": 125.7783, "cheshta": 3.3333, "naisargika": 8.5714, "drik": 8.4375, "total": 381.7872, "rank": 5, "war": null}
    },
    "time_lords": {"year": "Moon", "month": "Sun", "weekday": "Venus", "hora": "Mercury", "tribhaga": "Mars"},
    "bhavas": [
      {"house": 1, "lord": "Jupiter", "lord_bala": 327.7392, "dig": 60.0, "drishti": -11.375, "total": 376.3642, "rank": 12},
      {"house": 2, "lord": "Saturn", "lord_bala": 381.7872, "dig": 20.0, "drishti": 7.375, "total": 409.1622, "rank": 11},
      {"house": 3, "lord": "Saturn", "lord_bala": 381.7872, "dig": 40.0, "drishti": 6.75, "total": 428.5372, "rank": 9},
      {"house": 4, "lord": "Jupiter", "lord_bala": 327.7392, "dig": 60.0, "drishti": 33.4375, "total": 421.1767, "rank": 10},
      {"house": 5, "lord": "Mars", "lord_bala": 583.8017, "dig": 10.0, "drishti": 109.625, "total": 703.4267, "rank": 1},
      {"house": 6, "lord": "Venus", "lord_bala": 368.5734, "dig": 20.0, "drishti": 69.1875, "total": 457.7609, "rank": 8},
      {"house": 7, "lord": "Mercury", "lord_bala": 505.5862, "dig": 0.0, "drishti": 46.625, "total": 552.2112, "rank": 6},
      {"house": 8, "lord": "Moon", "lord_bala": 487.1187, "dig": 20.0, "drishti": 121.625, "total": 628.7437, "rank": 2},
      {"house": 9, "lord": "Sun", "lord_bala": 428.8691, "dig": 50.0, "drishti": 85.5625, "total": 564.4316, "rank": 5},
      {"house": 10, "lord": "Mercury", "lord_bala": 505.5862, "dig": 30.0, "drishti": 70.9375, "total": 606.5237, "rank": 3},
      {"house": 11, "lord": "Venus", "lord_bala": 368.5734, "dig": 40.0, "drishti": 60.9375, "total": 469.5109, "rank": 7},
      {"house": 12, "lord": "Mars", "lord_bala": 583.8017, "dig": 10.0, "drishti": -9.9375, "total": 583.8642, "rank": 4}
    ]
  }
}
````

**Yeni dosya:** `tests/golden/modules/shadbala/time-unknown-no-report.json`

````json
{
  "id": "time-unknown-no-report",
  "source": "hand",
  "rule_ref": "İleri modüller tasarım belgesi K2 ve §7 (veri modeli)",
  "derivation": [
    "Lagna yok (doğum saati bilinmiyor).",
    "Kendradi bala, dig bala, gündüz–gece gücü, tribhaga, saat lordu ve bhava bala doğum saatine bağlıdır.",
    "K2: lagnaya bağlı çıktı gizlenmez, hesaplanmaz → rapor yok (null)."
  ],
  "input": {
    "lagna_lon": null,
    "grahas": {"Sun": 11.0, "Moon": 172.0, "Mars": 296.0, "Mercury": 24.0, "Jupiter": 97.0, "Venus": 351.0, "Saturn": 196.0, "Rahu": 47.0, "Ketu": 227.0},
    "astro": {
      "jd_ut": 0.5, "sunrise_jd_ut": 0.25, "sunset_jd_ut": 0.75, "next_sunrise_jd_ut": 1.25, "mc_lon": 276.0,
      "tropical_lons": [35.0, 196.0, 320.0, 48.0, 121.0, 15.0, 220.0],
      "mean": {"sun": 33.0, "mercury": 100.0, "venus": 300.0, "mars": 310.0, "jupiter": 125.0, "saturn": 215.0},
      "weekday": "Sunday", "year_entry_weekday": "Thursday", "month_entry_weekday": "Tuesday"
    }
  },
  "expected": null
}
````

````diff
diff --git a/tests/golden/modules/README.md b/tests/golden/modules/README.md
index c5a8e81..38ba27f 100644
--- a/tests/golden/modules/README.md
+++ b/tests/golden/modules/README.md
@@ -32,6 +32,12 @@ ileri modüller tasarım belgesi §13 (dört katmanlı test stratejisinin ilk ka
 
 `lagna_lon: null` doğum saati bilinmeyen haritadır: lagnaya bağlı her beklenen değer `null` olur.
 
+Shadbala örnekleri girdiye iki alan daha ekler: `astro` (saf hesabın astronomik girdisi:
+doğum anı, gün doğumu, batımı ve sonraki gün doğumu gün kesri olarak; MC'nin sidereal boylamı;
+yedi grahanın tropikal boylamı; ortalama boylamlar; doğum gününün ve iki güneş girişinin
+varası) ve isteğe bağlı `lats` (ekliptik enlemler; yazılmayan graha 0°'dir). Diğer modüllerde
+bu alanlar yazılmaz.
+
 Ayara bağlı modüller (Jaimini, chara dasha) girdiye `advanced` bloğunu ekler:
 `"advanced": {"karaka_scheme": "Eight", "jaimini_co_lords": true}`. Diğer modüllerde bu alan
 yazılmaz. Chara dasha örneklerinde doğum anı ve yıl uzunluğu girdide yoktur: test sabit bir
@@ -46,4 +52,11 @@ doğum anı ve 365,25 günlük yıl kullanır, tarihleri el yıllarının biriki
 | `jaimini` | `karakas` (rol sırasında: `role`, `graha`, `effective_deg`, `tie`), `karakamsha` (`atmakaraka`, `sign`, dokuz grahanın `houses_in_d1` ve `houses_in_d9` değeri), `co_lords` (Akrep ve Kova: `lord`, belirleyici ölçüt `step`), `arudhas` (on iki pada: `house`, `sign`, `lord`, `exception`; saat bilinmiyorsa `null`), `argala` (en az iki referans burç: dört çiftin iki graha listesi ve durumu, `third_house_malefic_argala`) | 8 |
 | `chara_dasha` | `forward`, `order` (dasha sırası, lagnadan), `years` (ilk çevrim, `order` sırasında), `second_cycle` (ikinci çevrimde üretilen dashalar: `sign`, `years`), `sub_periods` (ilk iki dashanın alt dönem sırası); saat bilinmiyorsa `expected` `null` | 7 |
 
+| `shadbala` | `grahas` (yedi graha: sthana balanın beş kalemi ve `sthana`, `dig`, zaman gücünün sekiz kalemi ve `temporal`, `cheshta`, `naisargika`, `drik`, `total`, `rank`, `war`), `time_lords`, `bhavas` (on iki bhava: `house`, `lord`, `lord_bala`, `dig`, `drishti`, `total`, `rank`); saat bilinmiyorsa `expected` `null` | 4 |
+
+Shadbala örnekleri çalışma kâğıdıdır (tasarım belgesi §7 "Test"): `shadbala/WORKSHEET.md` her
+kalemin aritmetiğini adım adım yazar, JSON dosyaları aynı girdiyi ve sonuçları taşır. Tolerans
+0,01 virupadır. Kâğıdı, motoru yazan dışında bir gözden geçiren kuraldan yeniden hesaplar ve
+kâğıdın başındaki tabloya adını yazar.
+
 Sonraki alt projeler kendi klasörünü ve `expected` yapısını ekler.
````

- [ ] **Step 2: Başarısız testleri yaz (çalışma kâğıdı düzeneği ve değişmezler)**

````diff
diff --git a/crates/jyotish-core/tests/modules_golden.rs b/crates/jyotish-core/tests/modules_golden.rs
index 6f575e3..15d378b 100644
--- a/crates/jyotish-core/tests/modules_golden.rs
+++ b/crates/jyotish-core/tests/modules_golden.rs
@@ -9,9 +9,10 @@ use std::path::PathBuf;
 use jyotish_core::{
     argala_rows, arudha_padas, chara_dasha, chara_dasha_order, chara_dasha_sign_years,
     chara_karakas, co_lord_choice, drishti_table, house_lord, houses_owned, jaimini_lord,
-    jaimini_report, karakamsha, kuja_dosha, moon_is_waxing, nature, normalize, sign_dasha_children,
-    AdvancedSettings, ArgalaStatus, Graha, KarakaRole, KujaDosha, Nature, Positions, Sign,
-    SignDashaPeriod, ASPECTING_GRAHAS, CHARA_DASHA_METHOD, DRISHTI_METHOD, JAIMINI_METHOD,
+    jaimini_report, karakamsha, kuja_dosha, moon_is_waxing, nature, normalize, shadbala,
+    sign_dasha_children, AdvancedSettings, ArgalaStatus, Graha, GrahaWar, KarakaRole, KujaDosha,
+    Nature, Positions, ShadbalaAstro, Sign, SignDashaPeriod, TimeLords, ASPECTING_GRAHAS,
+    CHARA_DASHA_METHOD, DRISHTI_METHOD, JAIMINI_METHOD, SHADBALA_GRAHAS, SHADBALA_METHOD,
 };
 use serde::de::DeserializeOwned;
 use serde::Deserialize;
@@ -21,8 +22,11 @@ const MIN_FOUNDATION: usize = 6;
 const MIN_DRISHTI: usize = 4;
 const MIN_JAIMINI: usize = 8;
 const MIN_CHARA_DASHA: usize = 7;
+const MIN_SHADBALA: usize = 4;
 /// El hesabı ile motor arasındaki en büyük fark, virupa.
 const VIRUPA_TOL: f64 = 1e-9;
+/// Shadbala çalışma kâğıdının toleransı, virupa (ileri modüller spec'i §7 "Test").
+const WORKSHEET_TOL: f64 = 0.01;
 
 #[derive(Debug, Deserialize)]
 #[serde(deny_unknown_fields)]
@@ -36,7 +40,9 @@ struct Fixture<X> {
 }
 
 /// Sentetik konumlar: sidereal boylamlar, derece. `lagna_lon: null` doğum saati bilinmeyen
-/// haritadır. `advanced` yalnızca ayara bağlı modüllerde (Jaimini, chara dasha) verilir.
+/// haritadır. `advanced` yalnızca ayara bağlı modüllerde (Jaimini, chara dasha) verilir;
+/// `astro` (saf hesabın astronomik girdisi) ve `lats` (ekliptik enlemler; yazılmayan graha 0°)
+/// yalnızca Shadbala örneklerinde.
 #[derive(Debug, Deserialize)]
 #[serde(deny_unknown_fields)]
 struct Input {
@@ -44,15 +50,23 @@ struct Input {
     grahas: BTreeMap<Graha, f64>,
     #[serde(default)]
     advanced: Option<AdvancedSettings>,
+    #[serde(default)]
+    lats: Option<BTreeMap<Graha, f64>>,
+    #[serde(default)]
+    astro: Option<ShadbalaAstro>,
 }
 
 impl Input {
     fn positions(&self, id: &str) -> Positions {
         assert_eq!(self.grahas.len(), 9, "{id}: all nine grahas are required");
-        Positions::synthetic(
+        let mut positions = Positions::synthetic(
             self.lagna_lon,
             std::array::from_fn(|i| self.grahas[&Graha::ALL[i]]),
-        )
+        );
+        for (graha, lat) in self.lats.iter().flatten() {
+            positions.grahas[*graha as usize].lat = *lat;
+        }
+        positions
     }
 }
 
@@ -779,3 +793,223 @@ fn chara_dasha_fixtures_cover_the_required_cases() {
     );
     assert!(any(&|(x, _, _)| total(x) > 120), "ends after 120 years");
 }
+
+/// Bir grahanın çalışma kâğıdındaki satırı: bütün kalemler, virupa.
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct ExpectedGrahaBala {
+    uccha: f64,
+    saptavargaja: f64,
+    ojayugma: f64,
+    kendradi: f64,
+    drekkana: f64,
+    sthana: f64,
+    dig: f64,
+    day_night: f64,
+    paksha: f64,
+    tribhaga: f64,
+    year_lord: f64,
+    month_lord: f64,
+    weekday_lord: f64,
+    hora_lord: f64,
+    ayana: f64,
+    temporal: f64,
+    cheshta: f64,
+    naisargika: f64,
+    drik: f64,
+    total: f64,
+    rank: u8,
+    war: Option<GrahaWar>,
+}
+
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct ExpectedBhavaBala {
+    house: u8,
+    lord: Graha,
+    lord_bala: f64,
+    dig: f64,
+    drishti: f64,
+    total: f64,
+    rank: u8,
+}
+
+/// Shadbala el örneği (çalışma kâğıdı). Doğum saati bilinmeyen haritada `expected` `null`'dır.
+#[derive(Debug, Deserialize)]
+#[serde(deny_unknown_fields)]
+struct ShadbalaExpected {
+    grahas: BTreeMap<Graha, ExpectedGrahaBala>,
+    time_lords: TimeLords,
+    bhavas: Vec<ExpectedBhavaBala>,
+}
+
+fn shadbala_fixtures() -> Vec<(Fixture<Option<ShadbalaExpected>>, Positions, ShadbalaAstro)> {
+    load::<Option<ShadbalaExpected>>("shadbala", MIN_SHADBALA)
+        .into_iter()
+        .map(|(mut f, p)| {
+            let astro = f
+                .input
+                .astro
+                .take()
+                .unwrap_or_else(|| panic!("{}: shadbala fixtures state the astro block", f.id));
+            (f, p, astro)
+        })
+        .collect()
+}
+
+#[test]
+fn shadbala_hand_fixtures() {
+    for (f, p, astro) in shadbala_fixtures() {
+        let id = &f.id;
+        assert!(astro.is_consistent(), "{id}: astro block");
+        let got = shadbala(&p, &astro);
+        let Some(x) = &f.expected else {
+            assert!(p.lagna_sign.is_none(), "{id}: null only without a lagna");
+            assert_eq!(got, None, "{id}: no report without a lagna");
+            continue;
+        };
+        let r = got.unwrap_or_else(|| panic!("{id}: a report with a lagna"));
+        assert_eq!(r.method, SHADBALA_METHOD, "{id}");
+        assert_eq!(r.time_lords, x.time_lords, "{id}: time lords");
+        assert_eq!(x.grahas.len(), 7, "{id}: seven grahas");
+        let near = |got: f64, want: f64, what: &str| {
+            assert!(
+                (got - want).abs() <= WORKSHEET_TOL,
+                "{id}: {what}: engine {got}, worksheet {want}"
+            );
+        };
+        for (row, g) in r.grahas.iter().zip(SHADBALA_GRAHAS) {
+            let w = &x.grahas[&g];
+            let item = |name: &str| format!("{g:?} {name}");
+            near(row.sthana.uccha, w.uccha, &item("uccha"));
+            near(
+                row.sthana.saptavargaja,
+                w.saptavargaja,
+                &item("saptavargaja"),
+            );
+            near(row.sthana.ojayugma, w.ojayugma, &item("ojayugma"));
+            near(row.sthana.kendradi, w.kendradi, &item("kendradi"));
+            near(row.sthana.drekkana, w.drekkana, &item("drekkana"));
+            near(row.sthana.total, w.sthana, &item("sthana"));
+            near(row.dig, w.dig, &item("dig"));
+            near(row.temporal.day_night, w.day_night, &item("day_night"));
+            near(row.temporal.paksha, w.paksha, &item("paksha"));
+            near(row.temporal.tribhaga, w.tribhaga, &item("tribhaga"));
+            near(row.temporal.year_lord, w.year_lord, &item("year_lord"));
+            near(row.temporal.month_lord, w.month_lord, &item("month_lord"));
+            near(
+                row.temporal.weekday_lord,
+                w.weekday_lord,
+                &item("weekday_lord"),
+            );
+            near(row.temporal.hora_lord, w.hora_lord, &item("hora_lord"));
+            near(row.temporal.ayana, w.ayana, &item("ayana"));
+            near(row.temporal.total, w.temporal, &item("temporal"));
+            near(row.cheshta, w.cheshta, &item("cheshta"));
+            near(row.naisargika, w.naisargika, &item("naisargika"));
+            near(row.drik, w.drik, &item("drik"));
+            near(row.total, w.total, &item("total"));
+            assert_eq!(row.rank, w.rank, "{id}: rank of {g:?}");
+            assert_eq!(row.war, w.war, "{id}: war of {g:?}");
+        }
+        assert_eq!(x.bhavas.len(), 12, "{id}: twelve bhavas");
+        for (b, w) in r.bhavas.iter().zip(&x.bhavas) {
+            let item = |name: &str| format!("bhava {} {name}", w.house);
+            assert_eq!((b.house, b.lord), (w.house, w.lord), "{id}: bhava lord");
+            near(b.lord_bala, w.lord_bala, &item("lord_bala"));
+            near(b.dig, w.dig, &item("dig"));
+            near(b.drishti, w.drishti, &item("drishti"));
+            near(b.total, w.total, &item("total"));
+            assert_eq!(b.rank, w.rank, "{id}: rank of bhava {}", w.house);
+        }
+    }
+}
+
+/// Çalışma kâğıdının kapsaması gereken durumlar (ileri modüller spec'i §7 "Test": en az üç
+/// sentetik girdi): örnekler silinir ya da değiştirilirse kapsam sessizce daralmasın. Denetim
+/// motorun fonksiyonlarını değil, örneklerin girdisini ve el değerlerini okur.
+#[test]
+fn shadbala_fixtures_cover_the_required_cases() {
+    let all = shadbala_fixtures();
+    let full: Vec<(&ShadbalaExpected, &Positions, &ShadbalaAstro)> = all
+        .iter()
+        .filter_map(|(f, p, a)| f.expected.as_ref().map(|x| (x, p, a)))
+        .collect();
+    assert!(full.len() >= 3, "{} worked examples", full.len());
+    assert!(
+        all.iter().any(|(f, _, _)| f.expected.is_none()),
+        "birth time unknown"
+    );
+    let any = |pred: &dyn Fn(&ShadbalaExpected, &Positions, &ShadbalaAstro) -> bool| {
+        full.iter().any(|(x, p, a)| pred(x, p, a))
+    };
+    // Gündüz ve gece doğumu; öğleye uzaklığın 12 saati aştığı (şafaktan önce) doğum.
+    assert!(any(&|_, _, a| a.jd_ut < a.sunset_jd_ut), "day birth");
+    assert!(any(&|_, _, a| a.jd_ut >= a.sunset_jd_ut), "night birth");
+    assert!(
+        any(&|_, _, a| {
+            let noon = 0.5 * (a.sunrise_jd_ut + a.sunset_jd_ut);
+            (a.jd_ut - noon) * 24.0 > 12.0
+        }),
+        "more than twelve hours after noon"
+    );
+    // Büyüyen ve küçülen Ay (Ay − Güneş uzanımı 180°'nin altında ve üstünde).
+    let elongation = |p: &Positions| {
+        normalize(p.point(Graha::Moon).sidereal_lon - p.point(Graha::Sun).sidereal_lon)
+    };
+    assert!(any(&|_, p, _| elongation(p) < 180.0), "waxing Moon");
+    assert!(any(&|_, p, _| elongation(p) > 180.0), "waning Moon");
+    // Graha savaşı: enlemle kazanılan ve eşit enlemli.
+    let war = |equal_lat: bool| {
+        any(&|x, p, _| {
+            x.grahas.iter().any(|(g, row)| {
+                row.war
+                    .is_some_and(|w| (p.point(*g).lat == p.point(w.with).lat) == equal_lat)
+            })
+        })
+    };
+    assert!(war(false), "a war decided by latitude");
+    assert!(war(true), "a war between equal latitudes");
+    assert!(
+        any(&|x, _, _| x.grahas.values().all(|row| row.war.is_none())),
+        "an example without a war"
+    );
+    // Negatif drik bala; moolatrikona (D1'de 45 puan ancak toplam ≥ 45 ile görünür); her
+    // kendradi değeri; bhava dig balanın uç değerleri.
+    assert!(
+        any(&|x, _, _| x.grahas.values().any(|row| row.drik < 0.0)),
+        "a negative drik bala"
+    );
+    for value in [15.0, 30.0, 60.0] {
+        assert!(
+            any(&|x, _, _| x.grahas.values().any(|row| row.kendradi == value)),
+            "kendradi {value}"
+        );
+    }
+    for value in [0.0, 60.0] {
+        assert!(
+            any(&|x, _, _| x.bhavas.iter().any(|b| b.dig == value)),
+            "bhava dig {value}"
+        );
+    }
+    // Akrep lagna (güçsüz bhavası kendisi) ve Yay ya da Oğlak'ın bölünen yarıları.
+    assert!(
+        any(&|_, p, _| p.lagna_sign == Some(Sign::Scorpio)),
+        "Scorpio lagna"
+    );
+    assert!(
+        any(&|_, p, _| p.lagna_sign == Some(Sign::Sagittarius)),
+        "Sagittarius lagna"
+    );
+    // Oranı 1'in altında ve üstünde olan grahalar (gerekli değer el tablosundan).
+    let required = [390.0, 360.0, 300.0, 420.0, 390.0, 330.0, 300.0];
+    let ratio = |above: bool| {
+        any(&|x, _, _| {
+            SHADBALA_GRAHAS
+                .iter()
+                .zip(required)
+                .any(|(g, need)| (x.grahas[g].total >= need) == above)
+        })
+    };
+    assert!(ratio(true) && ratio(false), "ratios on both sides of one");
+}
````

````diff
diff --git a/crates/jyotish-core/src/proptests.rs b/crates/jyotish-core/src/proptests.rs
index 3c9d6b9..f4ea42d 100644
--- a/crates/jyotish-core/src/proptests.rs
+++ b/crates/jyotish-core/src/proptests.rs
@@ -850,3 +850,288 @@ mod chara_dasha_properties {
         }
     }
 }
+
+mod shadbala_properties {
+    use proptest::prelude::*;
+
+    use super::foundation_properties::positions;
+    use crate::graha::Graha;
+    use crate::lordship::house_lord;
+    use crate::nature::{nature, Nature};
+    use crate::panchang::Vara;
+    use crate::positions::Positions;
+    use crate::shadbala::{
+        ayana_bala, bhava_dig_bala, bhava_sign_kind, cheshta_from, day_night_bala, dig_bala,
+        drik_bala, graha_war, hora_index, hora_lord, naisargika_bala, paksha_bala, required_bala,
+        shadbala, uccha_bala, vara_lord, MeanLongitudes, ShadbalaAstro, GRAHA_WAR_ORB_DEG,
+        SHADBALA_GRAHAS, SHADBALA_METHOD, WAR_GRAHAS,
+    };
+    use crate::zodiac::{angular_distance, Sign};
+
+    const TOL: f64 = 1e-9;
+
+    fn vara() -> impl Strategy<Value = Vara> {
+        (0usize..7).prop_map(|i| Vara::ALL[i])
+    }
+
+    /// Tutarlı bir astronomik girdi: doğuş, gündüz uzunluğu, doğuşlar arası süre ve doğumun
+    /// gün içindeki yeri.
+    fn astro() -> impl Strategy<Value = ShadbalaAstro> {
+        (
+            (0.0f64..1.0, 0.05f64..0.95, 0.97f64..1.03, 0.0f64..0.999_999),
+            0.0f64..360.0,
+            proptest::array::uniform7(0.0f64..360.0),
+            proptest::array::uniform6(0.0f64..360.0),
+            (vara(), vara(), vara()),
+        )
+            .prop_map(
+                |((rise, day, span, at), mc, tropical, mean, (weekday, year, month))| {
+                    ShadbalaAstro {
+                        jd_ut: rise + span * at,
+                        sunrise_jd_ut: rise,
+                        sunset_jd_ut: rise + span * day,
+                        next_sunrise_jd_ut: rise + span,
+                        mc_lon: mc,
+                        tropical_lons: tropical,
+                        mean: MeanLongitudes {
+                            sun: mean[0],
+                            mercury: mean[1],
+                            venus: mean[2],
+                            mars: mean[3],
+                            jupiter: mean[4],
+                            saturn: mean[5],
+                        },
+                        weekday,
+                        year_entry_weekday: year,
+                        month_entry_weekday: month,
+                    }
+                },
+            )
+    }
+
+    fn graha7() -> impl Strategy<Value = Graha> {
+        (0usize..7).prop_map(|i| SHADBALA_GRAHAS[i])
+    }
+
+    /// Tam bir graha `value` alır, diğerleri 0.
+    fn one_lord(values: [f64; 7], value: f64) -> bool {
+        values.iter().filter(|v| **v == value).count() == 1
+            && values.iter().filter(|v| **v == 0.0).count() == 6
+    }
+
+    proptest! {
+        #[test]
+        fn uccha_of_opposite_points_adds_to_sixty(g in graha7(), lon in 0.0f64..360.0) {
+            let (a, b) = (uccha_bala(g, lon), uccha_bala(g, lon + 180.0));
+            prop_assert!((0.0..=60.0).contains(&a));
+            prop_assert!((a + b - 60.0).abs() < TOL, "{} + {}", a, b);
+        }
+
+        #[test]
+        fn dig_of_opposite_points_adds_to_sixty(
+            g in graha7(),
+            lon in 0.0f64..360.0,
+            lagna in 0.0f64..360.0,
+            mc in 0.0f64..360.0,
+        ) {
+            let (a, b) = (dig_bala(g, lon, lagna, mc), dig_bala(g, lon + 180.0, lagna, mc));
+            prop_assert!((0.0..=60.0).contains(&a));
+            prop_assert!((a + b - 60.0).abs() < TOL, "{} + {}", a, b);
+        }
+
+        #[test]
+        fn day_and_night_strong_grahas_add_to_sixty(m in 0.0f64..=12.0) {
+            for day in [Graha::Sun, Graha::Jupiter, Graha::Venus] {
+                for night in [Graha::Moon, Graha::Mars, Graha::Saturn] {
+                    let (a, b) = (day_night_bala(day, m), day_night_bala(night, m));
+                    prop_assert!((0.0..=60.0).contains(&a) && (0.0..=60.0).contains(&b));
+                    prop_assert!((a + b - 60.0).abs() < TOL);
+                }
+            }
+            prop_assert_eq!(day_night_bala(Graha::Mercury, m), 60.0);
+        }
+
+        #[test]
+        fn benefic_and_malefic_paksha_add_to_sixty(p in positions()) {
+            // Jüpiter her zaman iyicil, Satürn her zaman kötücüldür.
+            let (b, m) = (paksha_bala(Graha::Jupiter, &p), paksha_bala(Graha::Saturn, &p));
+            prop_assert!((b + m - 60.0).abs() < TOL, "{} + {}", b, m);
+            let moon = paksha_bala(Graha::Moon, &p);
+            prop_assert!((0.0..=120.0).contains(&moon));
+            prop_assert!((moon - 2.0 * b).abs() < TOL);
+            for g in SHADBALA_GRAHAS {
+                if g == Graha::Moon {
+                    continue;
+                }
+                let want = match nature(g, &p) {
+                    Nature::Benefic => b,
+                    Nature::Malefic => m,
+                };
+                prop_assert!((paksha_bala(g, &p) - want).abs() < TOL, "{:?}", g);
+            }
+        }
+
+        #[test]
+        fn ayana_stays_in_range_and_mirrors_across_the_equator(lon in 0.0f64..360.0) {
+            // Aynı boylamda kuzeyde güçlenen ile güneyde güçlenenin toplamı 60'tır.
+            let (north, south) = (ayana_bala(Graha::Sun, lon), ayana_bala(Graha::Moon, lon));
+            prop_assert!((north + south - 60.0).abs() < TOL);
+            for g in SHADBALA_GRAHAS {
+                let v = ayana_bala(g, lon);
+                prop_assert!((0.6875 - TOL..=59.3125 + TOL).contains(&v), "{:?} {}", g, v);
+            }
+            prop_assert!(ayana_bala(Graha::Mercury, lon) >= 30.0 - TOL);
+        }
+
+        #[test]
+        fn cheshta_stays_in_range_and_opposite_apexes_add_to_sixty(
+            slow in 0.0f64..360.0,
+            true_lon in 0.0f64..360.0,
+            apex in 0.0f64..360.0,
+        ) {
+            let (a, b) = (
+                cheshta_from(slow, true_lon, apex),
+                cheshta_from(slow, true_lon, apex + 180.0),
+            );
+            prop_assert!((0.0..=60.0).contains(&a));
+            prop_assert!((a + b - 60.0).abs() < 1e-6, "{} + {}", a, b);
+        }
+
+        #[test]
+        fn the_twenty_fifth_hora_is_the_next_day_lord(i in 0usize..7, a in astro()) {
+            let today = vara_lord(Vara::ALL[i]);
+            let tomorrow = vara_lord(Vara::ALL[(i + 1) % 7]);
+            prop_assert_eq!(hora_lord(today, 0), today);
+            prop_assert_eq!(hora_lord(today, 24), tomorrow);
+            prop_assert!(hora_index(&a) < 24);
+        }
+
+        #[test]
+        fn war_is_mutual_and_only_within_the_orb(p in positions(), near in 0.0f64..2.0) {
+            // Venüs'ü Mars'ın `near` derece ilerisine koy: savaş olasılığı yüksek olsun.
+            let mut lons: [f64; 9] = std::array::from_fn(|i| p.grahas[i].sidereal_lon);
+            lons[Graha::Venus as usize] = lons[Graha::Mars as usize] + near;
+            let p = Positions::synthetic(p.lagna_lon, lons);
+            for g in Graha::ALL {
+                let Some(war) = graha_war(g, &p) else { continue; };
+                prop_assert!(WAR_GRAHAS.contains(&g) && WAR_GRAHAS.contains(&war.with));
+                prop_assert!(war.with != g);
+                prop_assert!(war.winner == g || war.winner == war.with);
+                let gap = angular_distance(
+                    p.point(g).sidereal_lon,
+                    p.point(war.with).sidereal_lon,
+                );
+                prop_assert!(gap <= GRAHA_WAR_ORB_DEG);
+                // Karşı taraf da savaştadır (kendi en yakınıyla).
+                prop_assert!(graha_war(war.with, &p).is_some());
+            }
+            for g in [Graha::Sun, Graha::Moon, Graha::Rahu, Graha::Ketu] {
+                prop_assert_eq!(graha_war(g, &p), None);
+            }
+        }
+
+        #[test]
+        fn report_needs_a_lagna_and_every_item_is_in_its_range(p in positions(), a in astro()) {
+            let report = shadbala(&p, &a);
+            prop_assert_eq!(report.is_some(), p.lagna_sign.is_some());
+            let Some(r) = report else { return Ok(()); };
+            let lagna = p.lagna_sign.expect("lagna");
+            prop_assert_eq!(r.method.as_str(), SHADBALA_METHOD);
+            prop_assert!(r.warnings.is_empty());
+
+            let mut ranks = [false; 7];
+            for (row, g) in r.grahas.iter().zip(SHADBALA_GRAHAS) {
+                prop_assert_eq!(row.graha, g);
+                let s = &row.sthana;
+                prop_assert!((0.0..=60.0).contains(&s.uccha));
+                prop_assert!((14.0..=225.0).contains(&s.saptavargaja), "{}", s.saptavargaja);
+                prop_assert!([0.0, 15.0, 30.0].contains(&s.ojayugma));
+                prop_assert!([15.0, 30.0, 60.0].contains(&s.kendradi));
+                prop_assert!([0.0, 15.0].contains(&s.drekkana));
+                let sum = s.uccha + s.saptavargaja + s.ojayugma + s.kendradi + s.drekkana;
+                prop_assert!((s.total - sum).abs() < TOL);
+                prop_assert!((0.0..=60.0).contains(&row.dig));
+                let t = &row.temporal;
+                prop_assert!((0.0..=60.0).contains(&t.day_night));
+                let paksha_max = if g == Graha::Moon { 120.0 } else { 60.0 };
+                prop_assert!((0.0..=paksha_max).contains(&t.paksha));
+                let ayana_max = if g == Graha::Sun { 120.0 } else { 60.0 };
+                prop_assert!((0.0..=ayana_max).contains(&t.ayana));
+                let sum = t.day_night + t.paksha + t.tribhaga + t.year_lord + t.month_lord
+                    + t.weekday_lord + t.hora_lord + t.ayana;
+                prop_assert!((t.total - sum).abs() < TOL);
+                prop_assert!((0.0..=60.0).contains(&row.cheshta));
+                prop_assert_eq!(row.naisargika, naisargika_bala(g));
+                // Altı bakış: her biri ±15; Merkür ve Jüpiter'in tamamı en çok 60'ar.
+                prop_assert!((-90.0..=210.0).contains(&row.drik), "{}", row.drik);
+                prop_assert_eq!(row.drik, drik_bala(g, &p));
+                let total = s.total + row.dig + t.total + row.cheshta + row.naisargika + row.drik;
+                prop_assert!((row.total - total).abs() < TOL);
+                prop_assert!((row.rupas * 60.0 - row.total).abs() < 1e-6);
+                prop_assert_eq!(row.required, required_bala(g));
+                prop_assert!((row.ratio * row.required - row.total).abs() < 1e-6);
+                prop_assert_eq!(row.war, graha_war(g, &p));
+                prop_assert!((1..=7).contains(&row.rank));
+                ranks[usize::from(row.rank) - 1] = true;
+            }
+            prop_assert!(ranks.iter().all(|x| *x), "ranks are a permutation of 1..=7");
+            for x in &r.grahas {
+                for y in &r.grahas {
+                    if x.total > y.total {
+                        prop_assert!(x.rank < y.rank);
+                    }
+                }
+            }
+
+            // Tribhaga: Jüpiter her zaman 60; diğer altı grahadan tam biri 60, ötekiler 0.
+            let pick = |f: fn(&crate::shadbala::TemporalBala) -> f64| -> [f64; 7] {
+                std::array::from_fn(|i| f(&r.grahas[i].temporal))
+            };
+            let tribhaga = pick(|t| t.tribhaga);
+            prop_assert_eq!(tribhaga[Graha::Jupiter as usize], 60.0);
+            prop_assert!(r.time_lords.tribhaga != Graha::Jupiter);
+            let others = tribhaga
+                .iter()
+                .enumerate()
+                .filter(|(i, v)| *i != Graha::Jupiter as usize && **v == 60.0)
+                .count();
+            prop_assert_eq!(others, 1);
+            prop_assert!(tribhaga.iter().all(|v| *v == 0.0 || *v == 60.0));
+            // Yıl, ay, gün ve saat lordu: tam bir graha 15, 30, 45 ve 60 alır.
+            prop_assert!(one_lord(pick(|t| t.year_lord), 15.0));
+            prop_assert!(one_lord(pick(|t| t.month_lord), 30.0));
+            prop_assert!(one_lord(pick(|t| t.weekday_lord), 45.0));
+            prop_assert!(one_lord(pick(|t| t.hora_lord), 60.0));
+            prop_assert_eq!(r.time_lords.weekday, vara_lord(a.weekday));
+            prop_assert_eq!(r.time_lords.year, vara_lord(a.year_entry_weekday));
+            prop_assert_eq!(r.time_lords.month, vara_lord(a.month_entry_weekday));
+
+            // Bhavalar.
+            let mut ranks = [false; 12];
+            for (i, b) in r.bhavas.iter().enumerate() {
+                let house = i as u8 + 1;
+                prop_assert_eq!(b.house, house);
+                prop_assert_eq!(b.lord, house_lord(lagna, house));
+                prop_assert_eq!(b.lord_bala, r.grahas[b.lord as usize].total);
+                prop_assert!([0.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0].contains(&b.dig));
+                // Yedi bakış: her biri ±15; Merkür ve Jüpiter'in tamamı en çok 60'ar.
+                prop_assert!((-105.0..=225.0).contains(&b.drishti), "{}", b.drishti);
+                prop_assert!((b.total - (b.lord_bala + b.dig + b.drishti)).abs() < TOL);
+                prop_assert!((1..=12).contains(&b.rank));
+                ranks[usize::from(b.rank) - 1] = true;
+            }
+            prop_assert!(ranks.iter().all(|x| *x), "bhava ranks are a permutation of 1..=12");
+        }
+
+        #[test]
+        fn bhava_dig_of_opposite_houses_adds_to_sixty(
+            sign in 0usize..12,
+            deg in 0.0f64..30.0,
+            house in 1u8..=6,
+        ) {
+            let kind = bhava_sign_kind(Sign::from_index(sign), deg);
+            prop_assert_eq!(bhava_dig_bala(kind.powerless_house(), kind), 0.0);
+            prop_assert_eq!(bhava_dig_bala(house, kind) + bhava_dig_bala(house + 6, kind), 60.0);
+        }
+    }
+}
````

- [ ] **Step 3: Testlerin başarısız olduğunu doğrula**

```bash
cargo test -p jyotish-core 2>&1 | grep -E "^error" | sort | uniq -c
```
Expected: `error[E0432]: unresolved imports `crate::shadbala::bhava_dig_bala`, `crate::shadbala::bhava_sign_kind`, `crate::shadbala::shadbala``, `modules_golden` için `shadbala` fonksiyonunun bulunamadığını söyleyen hatalar (`E0432` ya da `E0423`) ve iki `could not compile` satırı.

- [ ] **Step 4: Bhava bala**

**Yeni dosya:** `crates/jyotish-core/src/shadbala/bhava.rs`

````rust
//! Bhava bala (ileri modüller spec'i §7, "Yöntem — Bhava bala"): bhavadhipati bala, bhava dig
//! bala ve bhava drishti bala. Kaynak: BPHS, bhava gücü bölümü.
//!
//! Bhavalar eşittir: bhava ortası `lagna + 30° × (h − 1)`; `CalcSettings.house_system`
//! bunları etkilemez. Bhavanın burcu rasi çerçevesindedir (K5) ve lagna burcundan sayılan
//! rasi evinin burcuyla aynıdır (K4).

use crate::graha::Graha;
use crate::lordship::{house_lord, house_sign};
use crate::positions::Positions;
use crate::zodiac::{angular_distance, deg_in_sign, normalize, Sign};

use super::sthana::rasi_lon;
use super::{drik_bala_at, ranks, BhavaBala, GrahaShadbala};

/// Bhava dig bala için burç türü.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BhavaSignKind {
    /// İnsan burçları: İkizler, Başak, Terazi, Kova, Yay'ın ilk yarısı.
    Human,
    /// Dört ayaklılar: Koç, Boğa, Aslan, Yay'ın ikinci yarısı, Oğlak'ın ilk yarısı.
    Quadruped,
    /// Su burçları: Yengeç, Balık, Oğlak'ın ikinci yarısı.
    Water,
    /// Akrep.
    Insect,
}

impl BhavaSignKind {
    /// Türün güçsüz olduğu bhava: bu bhavanın ortasında dig bala 0'dır.
    pub fn powerless_house(self) -> u8 {
        match self {
            Self::Human => 7,
            Self::Quadruped => 4,
            Self::Water => 10,
            Self::Insect => 1,
        }
    }
}

/// `sign` burcunun `deg` derecesindeki noktanın türü. Yay ve Oğlak 15°'de bölünür (ilk yarı
/// `[0°, 15°)`).
pub fn bhava_sign_kind(sign: Sign, deg: f64) -> BhavaSignKind {
    let first_half = deg < 15.0;
    match sign {
        Sign::Gemini | Sign::Virgo | Sign::Libra | Sign::Aquarius => BhavaSignKind::Human,
        Sign::Aries | Sign::Taurus | Sign::Leo => BhavaSignKind::Quadruped,
        Sign::Cancer | Sign::Pisces => BhavaSignKind::Water,
        Sign::Scorpio => BhavaSignKind::Insect,
        Sign::Sagittarius => {
            if first_half {
                BhavaSignKind::Human
            } else {
                BhavaSignKind::Quadruped
            }
        }
        Sign::Capricorn => {
            if first_half {
                BhavaSignKind::Quadruped
            } else {
                BhavaSignKind::Water
            }
        }
    }
}

/// `house`. bhavanın ortası: `lagna_lon + 30° × (house − 1)`, [0, 360). `lagna_lon` hangi
/// çerçevedeyse sonuç da o çerçevededir.
pub fn bhava_midpoint(lagna_lon: f64, house: u8) -> f64 {
    normalize(lagna_lon + 30.0 * (f64::from(house) - 1.0))
}

/// Bhava dig bala: bhava ortasının, burç türünün güçsüz olduğu bhavanın ortasına açısal
/// uzaklığı / 3. Bhava ortaları 30° arayla durduğu için değer 0, 10, …, 60'tır.
pub fn bhava_dig_bala(house: u8, kind: BhavaSignKind) -> f64 {
    angular_distance(
        bhava_midpoint(0.0, house),
        bhava_midpoint(0.0, kind.powerless_house()),
    ) / 3.0
}

/// Rasi çerçevesi ile sidereal boylam arasındaki fark, derece: `Sidereal` kipinde ve sentetik
/// konumlarda tam 0, tropikal-rasi kipinde ayanamsadır. Bütün grahalarda aynıdır; Güneş'ten
/// okunur.
fn frame_offset(positions: &Positions) -> f64 {
    let sun = positions.point(Graha::Sun);
    normalize(rasi_lon(sun) - sun.sidereal_lon)
}

/// On iki bhavanın gücü ve sırası. `grahas` yedi grahanın Shadbala sonucudur
/// (`SHADBALA_GRAHAS` sırasında). Doğum saati bilinmiyorsa `None`.
pub fn bhava_balas(positions: &Positions, grahas: &[GrahaShadbala; 7]) -> Option<[BhavaBala; 12]> {
    let lagna_lon = positions.lagna_lon?;
    let lagna_sign = positions.lagna_sign?;
    // Lagnanın rasi çerçevesindeki burç içi derecesi: her bhava ortasının derecesi de budur.
    let deg = deg_in_sign(lagna_lon + frame_offset(positions));
    let mut bhavas: [BhavaBala; 12] = std::array::from_fn(|i| {
        let house = i as u8 + 1;
        let lord = house_lord(lagna_sign, house);
        // Burç lordu her zaman yedi grahadan biridir.
        let lord_bala = grahas[lord as usize].total;
        let dig = bhava_dig_bala(house, bhava_sign_kind(house_sign(lagna_sign, house), deg));
        let drishti = drik_bala_at(bhava_midpoint(lagna_lon, house), None, positions);
        BhavaBala {
            house,
            lord,
            lord_bala,
            dig,
            drishti,
            total: lord_bala + dig + drishti,
            rank: 0,
        }
    });
    let order = ranks(std::array::from_fn::<f64, 12, _>(|i| bhavas[i].total));
    for (b, rank) in bhavas.iter_mut().zip(order) {
        b.rank = rank;
    }
    Some(bhavas)
}

#[cfg(test)]
mod tests {
    use super::*;
    use BhavaSignKind::*;

    #[test]
    fn sign_kinds_with_the_two_split_signs() {
        // Elle yazılmış tablo: (burç, ilk yarının türü, ikinci yarının türü).
        let table = [
            (Sign::Aries, Quadruped, Quadruped),
            (Sign::Taurus, Quadruped, Quadruped),
            (Sign::Gemini, Human, Human),
            (Sign::Cancer, Water, Water),
            (Sign::Leo, Quadruped, Quadruped),
            (Sign::Virgo, Human, Human),
            (Sign::Libra, Human, Human),
            (Sign::Scorpio, Insect, Insect),
            (Sign::Sagittarius, Human, Quadruped),
            (Sign::Capricorn, Quadruped, Water),
            (Sign::Aquarius, Human, Human),
            (Sign::Pisces, Water, Water),
        ];
        for (sign, first, second) in table {
            assert_eq!(bhava_sign_kind(sign, 0.0), first, "{sign:?}");
            assert_eq!(bhava_sign_kind(sign, 14.999), first, "{sign:?}");
            assert_eq!(bhava_sign_kind(sign, 15.0), second, "{sign:?}");
            assert_eq!(bhava_sign_kind(sign, 29.999), second, "{sign:?}");
        }
        assert_eq!(
            [Human, Quadruped, Water, Insect].map(BhavaSignKind::powerless_house),
            [7, 4, 10, 1]
        );
    }

    #[test]
    fn bhava_midpoints_step_by_thirty_degrees() {
        assert_eq!(bhava_midpoint(100.0, 1), 100.0);
        assert_eq!(bhava_midpoint(100.0, 4), 190.0);
        assert_eq!(bhava_midpoint(100.0, 10), 10.0);
        assert_eq!(bhava_midpoint(350.0, 2), 20.0);
    }

    #[test]
    fn bhava_dig_is_zero_at_the_powerless_house_and_sixty_opposite() {
        // İnsan burcu: 7. bhavada 0, 1. bhavada 60; 4. ve 10. bhavada 30.
        assert_eq!(bhava_dig_bala(7, Human), 0.0);
        assert_eq!(bhava_dig_bala(1, Human), 60.0);
        assert_eq!(bhava_dig_bala(4, Human), 30.0);
        assert_eq!(bhava_dig_bala(10, Human), 30.0);
        assert_eq!(bhava_dig_bala(8, Human), 10.0);
        // Dört ayaklı: 4. bhavada 0, 10. bhavada 60.
        assert_eq!(bhava_dig_bala(4, Quadruped), 0.0);
        assert_eq!(bhava_dig_bala(10, Quadruped), 60.0);
        assert_eq!(bhava_dig_bala(1, Quadruped), 30.0);
        assert_eq!(bhava_dig_bala(12, Quadruped), 40.0);
        // Su burcu: 10. bhavada 0, 4. bhavada 60.
        assert_eq!(bhava_dig_bala(10, Water), 0.0);
        assert_eq!(bhava_dig_bala(4, Water), 60.0);
        assert_eq!(bhava_dig_bala(12, Water), 20.0);
        // Akrep: 1. bhavada 0, 7. bhavada 60.
        assert_eq!(bhava_dig_bala(1, Insect), 0.0);
        assert_eq!(bhava_dig_bala(7, Insect), 60.0);
        assert_eq!(bhava_dig_bala(3, Insect), 20.0);
        // Her tür için on iki değer 0, 10, …, 60 kümesindedir ve karşıt bhavaların toplamı 60'tır.
        for kind in [Human, Quadruped, Water, Insect] {
            for h in 1..=6u8 {
                let (a, b) = (bhava_dig_bala(h, kind), bhava_dig_bala(h + 6, kind));
                assert_eq!(a + b, 60.0, "{kind:?} {h}");
                assert_eq!(a % 10.0, 0.0, "{kind:?} {h}");
            }
        }
    }
}
````

- [ ] **Step 5: Rapor, sıra ve dışa aktarımlar**

````diff
diff --git a/crates/jyotish-core/src/shadbala/mod.rs b/crates/jyotish-core/src/shadbala/mod.rs
index d6dee20..97d702d 100644
--- a/crates/jyotish-core/src/shadbala/mod.rs
+++ b/crates/jyotish-core/src/shadbala/mod.rs
@@ -3,9 +3,10 @@
 //! iki bhavanın gücü. Kaynak: BPHS'nin graha gücü ve bhava gücü bölümleri. Birim virupadır
 //! (60 virupa = 1 rupa).
 //!
-//! Hesap fonksiyonları ephemeris görmez (K3): yalnızca `Positions` ve `ShadbalaAstro` alır.
-//! Ek A: üçüncü güç yalnızca "zaman gücü" (`temporal`) adıyla anılır.
+//! Hesap fonksiyonları ephemeris görmez (K3): `shadbala` yalnızca `Positions` ve
+//! `ShadbalaAstro` alır. Ek A: üçüncü güç yalnızca "zaman gücü" (`temporal`) adıyla anılır.
 
+mod bhava;
 mod cheshta;
 mod sthana;
 mod temporal;
@@ -20,6 +21,7 @@ use crate::panchang::Vara;
 use crate::positions::Positions;
 use crate::zodiac::{angular_distance, normalize};
 
+pub use bhava::{bhava_balas, bhava_dig_bala, bhava_midpoint, bhava_sign_kind, BhavaSignKind};
 pub use cheshta::{
     cheshta_bala, cheshta_from, circular_midpoint, mean_longitudes, MeanLongitudes, MEAN_JUPITER,
     MEAN_MARS, MEAN_MERCURY, MEAN_SATURN, MEAN_SUN, MEAN_VENUS,
@@ -305,6 +307,72 @@ pub fn graha_war(graha: Graha, positions: &Positions) -> Option<GrahaWar> {
     })
 }
 
+/// Toplamların sırası: en büyük 1. Eşit toplamlarda dizide önce gelen öndedir.
+pub(crate) fn ranks<const N: usize>(totals: [f64; N]) -> [u8; N] {
+    let mut order: [usize; N] = std::array::from_fn(|i| i);
+    // `sort_by` kararlıdır: eşitler dizi sırasını korur.
+    order.sort_by(|a, b| totals[*b].total_cmp(&totals[*a]));
+    let mut out = [0u8; N];
+    for (rank, index) in order.into_iter().enumerate() {
+        out[index] = rank as u8 + 1;
+    }
+    out
+}
+
+/// Saf hesap: konumlardan ve astronomik girdiden Shadbala ve Bhava bala. Lagna yoksa (doğum
+/// saati bilinmiyor) ya da `astro` tutarsızsa `None`. `warnings` boştur; toplayıcı doldurur.
+pub fn shadbala(positions: &Positions, astro: &ShadbalaAstro) -> Option<ShadbalaReport> {
+    let lagna_lon = positions.lagna_lon?;
+    if !astro.is_consistent() {
+        return None;
+    }
+    let lords = time_lords(astro);
+    let mut rows = Vec::with_capacity(7);
+    for graha in SHADBALA_GRAHAS {
+        let sthana = sthana_bala(graha, positions)?;
+        let dig = dig_bala(
+            graha,
+            positions.point(graha).sidereal_lon,
+            lagna_lon,
+            astro.mc_lon,
+        );
+        let temporal = temporal_bala(graha, positions, astro, &lords);
+        let cheshta = cheshta_bala(graha, positions, astro);
+        let naisargika = naisargika_bala(graha);
+        let drik = drik_bala(graha, positions);
+        let total = sthana.total + dig + temporal.total + cheshta + naisargika + drik;
+        let required = required_bala(graha);
+        rows.push(GrahaShadbala {
+            graha,
+            sthana,
+            dig,
+            temporal,
+            cheshta,
+            naisargika,
+            drik,
+            total,
+            rupas: total / VIRUPAS_PER_RUPA,
+            required,
+            ratio: total / required,
+            rank: 0,
+            war: graha_war(graha, positions),
+        });
+    }
+    let order = ranks(std::array::from_fn::<f64, 7, _>(|i| rows[i].total));
+    for (row, rank) in rows.iter_mut().zip(order) {
+        row.rank = rank;
+    }
+    let grahas: [GrahaShadbala; 7] = rows.try_into().ok()?;
+    let bhavas = bhava_balas(positions, &grahas)?;
+    Some(ShadbalaReport {
+        grahas,
+        time_lords: lords,
+        bhavas,
+        method: SHADBALA_METHOD.to_owned(),
+        warnings: Vec::new(),
+    })
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
@@ -317,6 +385,28 @@ mod tests {
         Positions::synthetic(Some(0.0), lons)
     }
 
+    fn astro() -> ShadbalaAstro {
+        ShadbalaAstro {
+            jd_ut: 0.5,
+            sunrise_jd_ut: 0.25,
+            sunset_jd_ut: 0.75,
+            next_sunrise_jd_ut: 1.25,
+            mc_lon: 270.0,
+            tropical_lons: [34.0, 124.0, 244.0, 89.0, 274.0, 64.0, 324.0],
+            mean: MeanLongitudes {
+                sun: 30.0,
+                mercury: 200.0,
+                venus: 100.0,
+                mars: 240.0,
+                jupiter: 270.0,
+                saturn: 320.0,
+            },
+            weekday: Vara::Sunday,
+            year_entry_weekday: Vara::Thursday,
+            month_entry_weekday: Vara::Friday,
+        }
+    }
+
     fn close(a: f64, b: f64) -> bool {
         (a - b).abs() < 1e-9
     }
@@ -464,4 +554,91 @@ mod tests {
         assert_eq!(graha_war(Mercury, &p).map(|w| w.with), Some(Jupiter));
         assert_eq!(graha_war(Jupiter, &p).map(|w| w.with), Some(Mercury));
     }
+
+    #[test]
+    fn ranks_order_by_total_and_keep_array_order_on_ties() {
+        assert_eq!(ranks([10.0, 30.0, 20.0]), [3, 1, 2]);
+        assert_eq!(ranks([5.0, 5.0, 7.0, 5.0]), [2, 3, 1, 4]);
+        assert_eq!(ranks([-1.0, -3.0]), [1, 2]);
+    }
+
+    #[test]
+    fn report_sums_the_six_strengths_and_ranks_grahas_and_bhavas() {
+        let p = sky(BASE);
+        let a = astro();
+        let r = shadbala(&p, &a).expect("lagna and astro");
+        assert_eq!(r.method, SHADBALA_METHOD);
+        assert!(r.warnings.is_empty());
+        assert_eq!(r.time_lords, time_lords(&a));
+        let mut seen = [false; 7];
+        for (row, g) in r.grahas.iter().zip(SHADBALA_GRAHAS) {
+            assert_eq!(row.graha, g);
+            let sum = row.sthana.total
+                + row.dig
+                + row.temporal.total
+                + row.cheshta
+                + row.naisargika
+                + row.drik;
+            assert!(close(row.total, sum), "{g:?}");
+            assert!(close(row.rupas, row.total / 60.0), "{g:?}");
+            assert_eq!(row.required, required_bala(g));
+            assert!(close(row.ratio, row.total / row.required), "{g:?}");
+            assert_eq!(row.sthana, sthana_bala(g, &p).expect("lagna"));
+            assert_eq!(row.war, None);
+            seen[usize::from(row.rank) - 1] = true;
+        }
+        assert!(seen.iter().all(|s| *s), "ranks are a permutation of 1..=7");
+        // Sıra toplamı izler.
+        for a in &r.grahas {
+            for b in &r.grahas {
+                if a.total > b.total {
+                    assert!(a.rank < b.rank, "{:?} before {:?}", a.graha, b.graha);
+                }
+            }
+        }
+        // Bhavalar: lordun toplamı + dig + drishti; sıra 1..=12.
+        let mut seen = [false; 12];
+        for (i, b) in r.bhavas.iter().enumerate() {
+            assert_eq!(usize::from(b.house), i + 1);
+            assert_eq!(b.lord_bala, r.grahas[b.lord as usize].total);
+            assert!(close(b.total, b.lord_bala + b.dig + b.drishti));
+            seen[usize::from(b.rank) - 1] = true;
+        }
+        assert!(seen.iter().all(|s| *s), "bhava ranks are a permutation");
+        // Lagna Koç 0°: 1. bhava dört ayaklı (güçsüz bhava 4; 90° uzakta) → 30; 8. bhava Akrep
+        // (güçsüz bhava 1; 210° → kısa yay 150°) → 50.
+        assert_eq!(r.bhavas[0].dig, 30.0);
+        assert_eq!(r.bhavas[7].dig, 50.0);
+    }
+
+    #[test]
+    fn no_report_without_a_lagna_or_with_inconsistent_times() {
+        let p = sky(BASE);
+        assert_eq!(shadbala(&p.clone().without_lagna(), &astro()), None);
+        // Doğum, sonraki gün doğumundan sonra.
+        let late = ShadbalaAstro {
+            jd_ut: 1.3,
+            ..astro()
+        };
+        assert!(!late.is_consistent());
+        assert_eq!(shadbala(&p, &late), None);
+        // Batış doğuştan önce.
+        let no_day = ShadbalaAstro {
+            sunset_jd_ut: 0.2,
+            ..astro()
+        };
+        assert_eq!(shadbala(&p, &no_day), None);
+        let nan = ShadbalaAstro {
+            mc_lon: f64::NAN,
+            ..astro()
+        };
+        assert_eq!(shadbala(&p, &nan), None);
+        assert!(astro().is_consistent());
+        // Doğum tam gün doğumunda: tutarlı.
+        let at_rise = ShadbalaAstro {
+            jd_ut: 0.25,
+            ..astro()
+        };
+        assert!(at_rise.is_consistent());
+    }
 }
````

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 0b7286a..89d6e21 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -75,14 +75,15 @@ pub use settings::{
     NodeKind, SunriseDef, ZodiacMode, CALC_SETTINGS_SCHEMA_VERSION, JD_UT_RANGE,
 };
 pub use shadbala::{
-    ayana_bala, ayana_declination, cheshta_bala, cheshta_from, circular_midpoint, day_night_bala,
-    dig_bala, drekkana_bala, drik_bala, drik_bala_at, graha_war, hora_index, hora_lord,
-    hours_from_noon, is_daytime, kendradi_bala, mean_longitudes, naisargika_bala, ojayugma_bala,
-    paksha_bala, paksha_elongation, rasi_lon, required_bala, saptavargaja_bala,
-    saptavargaja_points, sthana_bala, temporal_bala, time_lords, tribhaga_lord, uccha_bala,
-    vara_lord, BhavaBala, GrahaShadbala, GrahaWar, MeanLongitudes, ShadbalaAstro, ShadbalaReport,
-    SthanaBala, TemporalBala, TimeLords, GRAHA_WAR_ORB_DEG, HORA_CYCLE, SAPTAVARGA,
-    SHADBALA_GRAHAS, SHADBALA_METHOD, VIRUPAS_PER_RUPA, WAR_GRAHAS,
+    ayana_bala, ayana_declination, bhava_balas, bhava_dig_bala, bhava_midpoint, bhava_sign_kind,
+    cheshta_bala, cheshta_from, circular_midpoint, day_night_bala, dig_bala, drekkana_bala,
+    drik_bala, drik_bala_at, graha_war, hora_index, hora_lord, hours_from_noon, is_daytime,
+    kendradi_bala, mean_longitudes, naisargika_bala, ojayugma_bala, paksha_bala, paksha_elongation,
+    rasi_lon, required_bala, saptavargaja_bala, saptavargaja_points, shadbala, sthana_bala,
+    temporal_bala, time_lords, tribhaga_lord, uccha_bala, vara_lord, BhavaBala, BhavaSignKind,
+    GrahaShadbala, GrahaWar, MeanLongitudes, ShadbalaAstro, ShadbalaReport, SthanaBala,
+    TemporalBala, TimeLords, GRAHA_WAR_ORB_DEG, HORA_CYCLE, SAPTAVARGA, SHADBALA_GRAHAS,
+    SHADBALA_METHOD, VIRUPAS_PER_RUPA, WAR_GRAHAS,
 };
 pub use transit::{compute_transits, TransitGraha, TransitSnapshot};
 pub use varga::{
````

- [ ] **Step 6: Testleri koş**

```bash
cargo test -p jyotish-core 2>&1 | grep -E "^test result|Running"
cargo test -p jyotish-core --lib shadbala 2>&1 | grep -E "^test result"
```
Expected: lib `231 passed` (215 + 3 bhava testi + 3 rapor testi + 10 proptest), golden `8 passed`, `modules_golden` `9 passed`; ikinci komut `40 passed`. Üç tam örnek motorun ilk koşusunda 0,01 virupa içinde geçmelidir; bir kalem düşerse önce kural (spec §7 ve D4–D14) ve çalışma kâğıdının o satırı yeniden okunur, beklenen değer motor çıktısına bakarak değiştirilmez.

- [ ] **Step 7: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
grep -rn '"source"' tests/golden/modules | grep -v '"source": "hand"'
ls tests/golden/modules/shadbala/*.json | wc -l
```
Expected: fmt ve clippy çıktısız; `jyotish-core` lib `231 passed`, `modules_golden` `9 passed`; diğer Rust satırları tabandaki gibi; `grep` çıktısız; son komut `4` yazar.

- [ ] **Step 8: Commit**

```bash
git add crates/jyotish-core tests/golden/modules
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): shadbala report with ranks and bhava bala; worksheet fixtures and invariants"
```
Expected: iki `grep` çıktısız.

---

### Task 3: Çekirdek: toplayıcı (`compute_shadbala`), güneş girişleri ve astronomik girdi

Spec K3, §7 "Zaman gücü" (yıl ve ay lordu), "Veri modeli ve IPC"; D3, D7. Testler sahte ephemeris'le (`fake::FakeEph`) koşar; gerçek motorla çapraz denetim Task 4'tedir.

**Files:**
- Create: `crates/jyotish-core/src/shadbala/collect.rs`
- Modify: `crates/jyotish-core/src/shadbala/mod.rs`, `crates/jyotish-core/src/lib.rs`, `crates/jyotish-core/src/panchang.rs`
- Test: `collect.rs` içindeki `mod tests` (6 test)

**Interfaces:**
- Consumes: Task 1 ve 2; `compute_chart`, `Chart` (`panchang.sunrise_jd_ut`, `panchang.sunset_jd_ut`, `panchang.vara`, `ayanamsa_deg`, `grahas`, `meta.warnings`), `chart::finite` (`pub(crate)`), `ChartInput`, `CalcSettings::{sunrise_def, ayanamsa}`, `ephemeris::{Body, Ephemeris, HouseSystem, Zodiac}`, `panchang::{day_sunrise, local_weekday}`, `fake::FakeEph`.
- Produces:
  - `panchang.rs` (`pub(crate)`): `day_sunrise` (görünürlük değişir), `following_sunrise(eph, sunrise, geo_lat, geo_lon, disc) -> Result<Option<f64>, CoreError>`, `vara_at(eph, jd_ut, geo_lat, geo_lon, disc) -> Result<Vara, CoreError>`.
  - `pub fn sun_entry_before<E: Ephemeris + ?Sized>(eph: &mut E, zodiac: Zodiac, jd_ut: f64, target_deg: f64) -> Result<f64, CoreError>`
  - `pub fn compute_shadbala_astro<E: Ephemeris + ?Sized>(eph: &mut E, input: &ChartInput, chart: &Chart) -> Result<Option<ShadbalaAstro>, CoreError>`
  - `pub fn compute_shadbala<E: Ephemeris + ?Sized>(eph: &mut E, input: &ChartInput) -> Result<Option<ShadbalaReport>, CoreError>`

- [ ] **Step 1: Alt modülü kaydet (derleme başarısız olur)**

````diff
diff --git a/crates/jyotish-core/src/shadbala/mod.rs b/crates/jyotish-core/src/shadbala/mod.rs
index 97d702d..ccf2d7b 100644
--- a/crates/jyotish-core/src/shadbala/mod.rs
+++ b/crates/jyotish-core/src/shadbala/mod.rs
@@ -4,10 +4,13 @@
 //! (60 virupa = 1 rupa).
 //!
 //! Hesap fonksiyonları ephemeris görmez (K3): `shadbala` yalnızca `Positions` ve
-//! `ShadbalaAstro` alır. Ek A: üçüncü güç yalnızca "zaman gücü" (`temporal`) adıyla anılır.
+//! `ShadbalaAstro` alır. `compute_shadbala` (`collect` alt modülü) temel haritayı ve gereken
+//! astronomik değerleri toplayıp ona verir. Ek A: üçüncü güç yalnızca "zaman gücü"
+//! (`temporal`) adıyla anılır.
 
 mod bhava;
 mod cheshta;
+mod collect;
 mod sthana;
 mod temporal;
 
@@ -26,6 +29,7 @@ pub use cheshta::{
     cheshta_bala, cheshta_from, circular_midpoint, mean_longitudes, MeanLongitudes, MEAN_JUPITER,
     MEAN_MARS, MEAN_MERCURY, MEAN_SATURN, MEAN_SUN, MEAN_VENUS,
 };
+pub use collect::{compute_shadbala, compute_shadbala_astro, sun_entry_before};
 pub use sthana::{
     drekkana_bala, kendradi_bala, ojayugma_bala, rasi_lon, saptavargaja_bala, saptavargaja_points,
     sthana_bala, uccha_bala, SAPTAVARGA,
@@ -156,8 +160,8 @@ pub struct ShadbalaReport {
     pub warnings: Vec<ChartWarning>,
 }
 
-/// Saf hesabın astronomik girdisi (K3): toplayıcı ephemeris'ten üretir, el örnekleri elle
-/// yazar. Zamanlar UT Jülyen günüdür; hesap yalnızca farklarını kullanır.
+/// Saf hesabın astronomik girdisi (K3): toplayıcı `compute_shadbala_astro` ile üretir, el
+/// örnekleri elle yazar. Zamanlar UT Jülyen günüdür; hesap yalnızca farklarını kullanır.
 #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
 #[serde(deny_unknown_fields)]
 pub struct ShadbalaAstro {
````

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 89d6e21..584ca19 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -76,14 +76,15 @@ pub use settings::{
 };
 pub use shadbala::{
     ayana_bala, ayana_declination, bhava_balas, bhava_dig_bala, bhava_midpoint, bhava_sign_kind,
-    cheshta_bala, cheshta_from, circular_midpoint, day_night_bala, dig_bala, drekkana_bala,
-    drik_bala, drik_bala_at, graha_war, hora_index, hora_lord, hours_from_noon, is_daytime,
-    kendradi_bala, mean_longitudes, naisargika_bala, ojayugma_bala, paksha_bala, paksha_elongation,
-    rasi_lon, required_bala, saptavargaja_bala, saptavargaja_points, shadbala, sthana_bala,
-    temporal_bala, time_lords, tribhaga_lord, uccha_bala, vara_lord, BhavaBala, BhavaSignKind,
-    GrahaShadbala, GrahaWar, MeanLongitudes, ShadbalaAstro, ShadbalaReport, SthanaBala,
-    TemporalBala, TimeLords, GRAHA_WAR_ORB_DEG, HORA_CYCLE, SAPTAVARGA, SHADBALA_GRAHAS,
-    SHADBALA_METHOD, VIRUPAS_PER_RUPA, WAR_GRAHAS,
+    cheshta_bala, cheshta_from, circular_midpoint, compute_shadbala, compute_shadbala_astro,
+    day_night_bala, dig_bala, drekkana_bala, drik_bala, drik_bala_at, graha_war, hora_index,
+    hora_lord, hours_from_noon, is_daytime, kendradi_bala, mean_longitudes, naisargika_bala,
+    ojayugma_bala, paksha_bala, paksha_elongation, rasi_lon, required_bala, saptavargaja_bala,
+    saptavargaja_points, shadbala, sthana_bala, sun_entry_before, temporal_bala, time_lords,
+    tribhaga_lord, uccha_bala, vara_lord, BhavaBala, BhavaSignKind, GrahaShadbala, GrahaWar,
+    MeanLongitudes, ShadbalaAstro, ShadbalaReport, SthanaBala, TemporalBala, TimeLords,
+    GRAHA_WAR_ORB_DEG, HORA_CYCLE, SAPTAVARGA, SHADBALA_GRAHAS, SHADBALA_METHOD, VIRUPAS_PER_RUPA,
+    WAR_GRAHAS,
 };
 pub use transit::{compute_transits, TransitGraha, TransitSnapshot};
 pub use varga::{
````

- [ ] **Step 2: Derlemenin başarısız olduğunu doğrula**

```bash
cargo test -p jyotish-core --lib shadbala::collect 2>&1 | grep -E "^error"
```
Expected: `error[E0583]: file not found for module `collect`` ve `could not compile`.

- [ ] **Step 3: Panchang yardımcıları**

`compute_panchang` değişmez; yalnızca iki yardımcı eklenir ve `day_sunrise` modül dışından çağrılabilir olur.

````diff
diff --git a/crates/jyotish-core/src/panchang.rs b/crates/jyotish-core/src/panchang.rs
index 5348d4e..2ef78fe 100644
--- a/crates/jyotish-core/src/panchang.rs
+++ b/crates/jyotish-core/src/panchang.rs
@@ -280,7 +280,7 @@ fn next_boundary<E: Ephemeris + ?Sized>(
 }
 
 /// Doğumdan önceki son gün doğumu, `[jd_ut − SUNRISE_WINDOW_DAYS, jd_ut]` içinde.
-fn day_sunrise<E: Ephemeris + ?Sized>(
+pub(crate) fn day_sunrise<E: Ephemeris + ?Sized>(
     eph: &mut E,
     jd_ut: f64,
     geo_lat: f64,
@@ -329,6 +329,38 @@ fn day_sunset<E: Ephemeris + ?Sized>(
     }
 }
 
+/// `sunrise`'tan sonraki ilk gün doğumu; arama ufkunda yoksa `None`. Shadbala'nın saat
+/// dilimleri ve gece üçte birleri için (ileri modüller spec'i §7).
+pub(crate) fn following_sunrise<E: Ephemeris + ?Sized>(
+    eph: &mut E,
+    sunrise: f64,
+    geo_lat: f64,
+    geo_lon: f64,
+    disc: SunDisc,
+) -> Result<Option<f64>, CoreError> {
+    let from = sunrise + RISE_STEP_DAYS;
+    match eph.sun_rise_set(from, RiseSet::Rise, geo_lat, geo_lon, disc)? {
+        Some(t) if !t.is_finite() => Err(CoreError::NonFiniteResult("sunrise")),
+        Some(t) if t > from => Ok(Some(t)),
+        _ => Ok(None),
+    }
+}
+
+/// `jd_ut` anının varası: vara o andan önceki son gün doğumunda başlar; gün doğumu yoksa gün
+/// 06:00 yerel ortalama saatte başlıyor sayılır (`compute_panchang` ile aynı kural).
+pub(crate) fn vara_at<E: Ephemeris + ?Sized>(
+    eph: &mut E,
+    jd_ut: f64,
+    geo_lat: f64,
+    geo_lon: f64,
+    disc: SunDisc,
+) -> Result<Vara, CoreError> {
+    Ok(match day_sunrise(eph, jd_ut, geo_lat, geo_lon, disc)? {
+        Some(r) => local_weekday(r, geo_lon),
+        None => local_weekday(jd_ut - FALLBACK_DAY_START, geo_lon),
+    })
+}
+
 /// Doğum anının panchangı (spec §5). Güneş ve Ay her zodyak kipinde sidereal sorulur.
 /// Gün doğumu `settings.sunrise_def` tanımıyla, doğum yerinde (`geo_lat` kuzey +, `geo_lon`
 /// doğu +) ve deniz seviyesinde hesaplanır.
````

- [ ] **Step 4: Toplayıcı**

Testlerdeki hafta günleri elle türetilmiştir: JD 2451545,0 (2000-01-01 12:00 UT) Cumartesi'dir; sahte Güneş 0,98°/gün hızla doğrusal ilerler (sidereal 346°), yani ay girişi 16 / 0,98 = 16,3265 gün, yıl girişi 346 / 0,98 = 353,0612 gün öncedir.

**Yeni dosya:** `crates/jyotish-core/src/shadbala/collect.rs`

````rust
//! Shadbala toplayıcısı (ileri modüller spec'i K3, §7 "Veri modeli ve IPC"): temel haritayı
//! hesaplar, saf hesabın astronomik girdisini (`ShadbalaAstro`) ephemeris'ten toplar ve
//! `shadbala`'ya verir.

use ephemeris::{Body, Ephemeris, HouseSystem, Zodiac};

use crate::chart::{compute_chart, finite, Chart, ChartWarning};
use crate::graha::Graha;
use crate::panchang::{following_sunrise, vara_at};
use crate::positions::{ChartInput, Positions};
use crate::zodiac::normalize;
use crate::CoreError;

use super::cheshta::mean_longitudes;
use super::{shadbala, ShadbalaAstro, ShadbalaReport, SHADBALA_GRAHAS};

/// Giriş aramasının sabit adım sayısı (Newton). Güneş'in hızı %3,4'ten az değişir: ilk tahmin
/// en çok ~12° şaşar ve her adım hatanın karesiyle küçülür; sekiz adım fazlasıyla yeter.
const ENTRY_STEPS: usize = 8;

/// Güneş'in `jd_ut`'den önceki son anı ki `zodiac` çerçevesindeki boylamı `target_deg` olsun
/// (burç girişi). `jd_ut` anında Güneş tam hedefteyse o an döner.
pub fn sun_entry_before<E: Ephemeris + ?Sized>(
    eph: &mut E,
    zodiac: Zodiac,
    jd_ut: f64,
    target_deg: f64,
) -> Result<f64, CoreError> {
    let now = eph.body(jd_ut, Body::Sun, zodiac)?;
    let speed = finite(now.lon_speed, "sun_entry")?;
    if speed <= 0.0 {
        return Err(CoreError::NonFiniteResult("sun_entry"));
    }
    // Hedeften bu yana kat edilen yay, [0, 360).
    let travelled = normalize(finite(now.lon, "sun_entry")? - target_deg);
    let mut t = jd_ut - travelled / speed;
    for _ in 0..ENTRY_STEPS {
        let p = eph.body(t, Body::Sun, zodiac)?;
        let speed = finite(p.lon_speed, "sun_entry")?;
        if speed <= 0.0 {
            return Err(CoreError::NonFiniteResult("sun_entry"));
        }
        // Hedefe işaretli uzaklık, [-180, 180).
        let off = normalize(finite(p.lon, "sun_entry")? - target_deg + 180.0) - 180.0;
        t -= off / speed;
    }
    finite(t, "sun_entry")
}

/// Saf hesabın astronomik girdisi. `chart`, `input` ile hesaplanmış temel haritadır. Doğum
/// gününün gün doğumu, batımı ya da sonraki gün doğumu yoksa (kutup günü ya da gecesi)
/// `None`.
pub fn compute_shadbala_astro<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
    chart: &Chart,
) -> Result<Option<ShadbalaAstro>, CoreError> {
    let settings = &input.settings;
    let disc = settings.sunrise_def.sun_disc();
    let zodiac = Zodiac::Sidereal(settings.ayanamsa.sidereal_mode());
    let (Some(sunrise), Some(sunset)) = (chart.panchang.sunrise_jd_ut, chart.panchang.sunset_jd_ut)
    else {
        return Ok(None);
    };
    let Some(next_sunrise) = following_sunrise(eph, sunrise, input.lat, input.lon, disc)? else {
        return Ok(None);
    };
    // MC astronomik açıdır; hangi ev sistemi sorulursa sorulsun aynıdır.
    let houses = eph.houses(
        input.jd_ut,
        input.lat,
        input.lon,
        HouseSystem::WholeSign,
        zodiac,
    )?;
    let mc_lon = finite(houses.mc, "mc")?;

    // Tropikal boylam: sidereal + ayanamsa (v1'in sidereal dönüşümünün tersi).
    let tropical_lons = std::array::from_fn(|i| {
        normalize(chart.grahas[SHADBALA_GRAHAS[i] as usize].sidereal_lon + chart.ayanamsa_deg)
    });

    // Yıl girişi: sidereal 0° (Koç); ay girişi: Güneş'in bulunduğu sidereal burcun başı.
    let sun = chart.grahas[Graha::Sun as usize].sidereal_lon;
    let month_start = (sun / 30.0).floor().clamp(0.0, 11.0) * 30.0;
    let year_entry = sun_entry_before(eph, zodiac, input.jd_ut, 0.0)?;
    let month_entry = sun_entry_before(eph, zodiac, input.jd_ut, month_start)?;
    let year_entry_weekday = vara_at(eph, year_entry, input.lat, input.lon, disc)?;
    let month_entry_weekday = vara_at(eph, month_entry, input.lat, input.lon, disc)?;

    let astro = ShadbalaAstro {
        jd_ut: input.jd_ut,
        sunrise_jd_ut: sunrise,
        sunset_jd_ut: sunset,
        next_sunrise_jd_ut: next_sunrise,
        mc_lon,
        tropical_lons,
        mean: mean_longitudes(input.jd_ut),
        weekday: chart.panchang.vara,
        year_entry_weekday,
        month_entry_weekday,
    };
    Ok(astro.is_consistent().then_some(astro))
}

/// `compute_shadbala` komutunun hesabı. Doğum saati bilinmiyorsa (`input.time_known` yanlış)
/// ya da doğum yerinde o gün gün doğumu ya da batımı yoksa `None` döner; ayar, an ve konum
/// denetimi (`compute_chart`) her durumda yapılır. Temel haritanın konum duyarlılığı uyarısı
/// (`LowPrecision`) rapora taşınır.
pub fn compute_shadbala<E: Ephemeris + ?Sized>(
    eph: &mut E,
    input: &ChartInput,
) -> Result<Option<ShadbalaReport>, CoreError> {
    let chart = compute_chart(eph, input.jd_ut, input.lat, input.lon, &input.settings)?;
    if !input.time_known {
        return Ok(None);
    }
    let Some(astro) = compute_shadbala_astro(eph, input, &chart)? else {
        return Ok(None);
    };
    let Some(mut report) = shadbala(&Positions::from(&chart), &astro) else {
        return Ok(None);
    };
    report.warnings = chart
        .meta
        .warnings
        .iter()
        .copied()
        .filter(|w| *w == ChartWarning::LowPrecision)
        .collect();
    Ok(Some(report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::panchang::Vara;
    use crate::settings::{CalcSettings, ZodiacMode};
    use ephemeris::{Precision, SiderealMode};

    const JD: f64 = 2_451_545.0;
    const LAHIRI: Zodiac = Zodiac::Sidereal(SiderealMode::Lahiri);

    fn input(settings: CalcSettings, time_known: bool) -> ChartInput {
        ChartInput {
            jd_ut: JD,
            lat: 41.0,
            lon: 29.0,
            settings,
            time_known,
        }
    }

    fn astro_of(eph: &mut FakeEph, input: &ChartInput) -> Option<ShadbalaAstro> {
        let chart = compute_chart(eph, input.jd_ut, input.lat, input.lon, &input.settings).unwrap();
        compute_shadbala_astro(eph, input, &chart).unwrap()
    }

    #[test]
    fn sun_entry_is_found_on_a_linear_sun() {
        // FakeEph: Güneş tropikal 10°, ayanamsa 24° → sidereal 346°; hız 0,98°/gün.
        let mut eph = FakeEph::default();
        // Ay girişi (Balık, 330°): 16° geride → 16 / 0,98 gün önce.
        let month = sun_entry_before(&mut eph, LAHIRI, JD, 330.0).unwrap();
        assert!((month - (JD - 16.0 / 0.98)).abs() < 1e-9, "{month}");
        // Yıl girişi (Koç, 0°): 346° geride.
        let year = sun_entry_before(&mut eph, LAHIRI, JD, 0.0).unwrap();
        assert!((year - (JD - 346.0 / 0.98)).abs() < 1e-9, "{year}");
        // Giriş anında boylam hedeftedir.
        let at = eph.body(year, Body::Sun, LAHIRI).unwrap().lon;
        assert!(at.min(360.0 - at) < 1e-9, "{at}");
        // Güneş tam hedefteyse an doğumun kendisidir.
        let now = sun_entry_before(&mut eph, LAHIRI, JD, 346.0).unwrap();
        assert!((now - JD).abs() < 1e-9);
        // Geri giden ya da duran bir "Güneş" hata verir (yalnızca bozuk ephemeris'te).
        let mut still = FakeEph::default().with(Body::Sun, 10.0, 0.0);
        assert_eq!(
            sun_entry_before(&mut still, LAHIRI, JD, 0.0),
            Err(CoreError::NonFiniteResult("sun_entry"))
        );
    }

    #[test]
    fn astro_is_collected_from_the_chart_and_the_ephemeris() {
        let mut eph = FakeEph::default();
        let a = astro_of(&mut eph, &input(CalcSettings::default(), true)).expect("astro");
        // FakeEph: doğuş her gün n + 0,67, batış n + 0,15.
        assert_eq!(a.jd_ut, JD);
        assert!((a.sunrise_jd_ut - 2_451_544.67).abs() < 1e-9);
        assert!((a.sunset_jd_ut - 2_451_545.15).abs() < 1e-9);
        assert!((a.next_sunrise_jd_ut - 2_451_545.67).abs() < 1e-9);
        // MC: tropikal lagna 70° + 270° = 340°; sidereal 316°.
        assert!((a.mc_lon - 316.0).abs() < 1e-9);
        // Tropikal boylamlar ephemeris'in tropikal değerleridir (sidereal + ayanamsa).
        let want = [10.0, 95.0, 200.0, 20.0, 250.0, 40.0, 300.0];
        for (got, want) in a.tropical_lons.into_iter().zip(want) {
            assert!((got - want).abs() < 1e-9, "{got} {want}");
        }
        assert_eq!(a.mean, mean_longitudes(JD));
        // JD 2451545,0 (2000-01-01 12:00 UT) Cumartesi'dir; doğuş 2451544,67 aynı yerel günde.
        assert_eq!(a.weekday, Vara::Saturday);
        // Ay girişi JD − 16,3265 = 2451528,6735: o günün doğuşu 2451528,67; 16 gün önce
        // (Cumartesi − 16 gün) Perşembe.
        assert_eq!(a.month_entry_weekday, Vara::Thursday);
        // Yıl girişi JD − 353,0612 = 2451191,9388: doğuş 2451191,67; 353 gün önce
        // (353 = 50 × 7 + 3; Cumartesi − 3 gün) Çarşamba.
        assert_eq!(a.year_entry_weekday, Vara::Wednesday);
        assert!(a.is_consistent());
    }

    #[test]
    fn report_is_the_pure_function_on_the_collected_input() {
        let mut eph = FakeEph::default();
        let i = input(CalcSettings::default(), true);
        let report = compute_shadbala(&mut eph, &i).unwrap().expect("report");
        let chart = compute_chart(&mut eph, i.jd_ut, i.lat, i.lon, &i.settings).unwrap();
        let astro = compute_shadbala_astro(&mut eph, &i, &chart)
            .unwrap()
            .expect("astro");
        assert_eq!(
            Some(&report),
            shadbala(&Positions::from(&chart), &astro).as_ref()
        );
        assert!(report.warnings.is_empty());
        // Cumartesi; doğum doğuştan 0,33 gün sonra: 8. saat dilimi (sıra 7) → Cumartesi
        // dizisinde Satürn. Gündüz 0,48 gün; 0,33 / 0,16 = 2,06 → üçüncü dilim: Satürn.
        let lords = report.time_lords;
        assert_eq!(
            (lords.weekday, lords.hora, lords.tribhaga),
            (Graha::Saturn, Graha::Saturn, Graha::Saturn)
        );
        assert_eq!((lords.year, lords.month), (Graha::Mercury, Graha::Jupiter));
        // Sidereal lagna 46° (Boğa 16°): 1. bhava dört ayaklı, güçsüz bhavası 4. → 30.
        assert_eq!(report.bhavas[0].dig, 30.0);
    }

    #[test]
    fn tropical_rasi_mode_uses_tropical_signs_and_sidereal_angles() {
        let mut eph = FakeEph::default();
        let settings = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..CalcSettings::default()
        };
        let report = compute_shadbala(&mut eph, &input(settings, true))
            .unwrap()
            .expect("report");
        // Güneş rasi çerçevesinde Koç 10°: yücelme noktası, uccha 60.
        assert!((report.grahas[0].sthana.uccha - 60.0).abs() < 1e-9);
        // Lagna rasi çerçevesinde İkizler 10° (tropikal 70°): 1. bhava insan burcu → 60.
        assert_eq!(report.bhavas[0].dig, 60.0);
        assert_eq!(report.bhavas[0].lord, Graha::Mercury);
        // Açı farkları çerçeveden bağımsızdır: dig bala sidereal kipteki ile aynıdır.
        let sidereal = compute_shadbala(&mut eph, &input(CalcSettings::default(), true))
            .unwrap()
            .expect("report");
        for (a, b) in report.grahas.iter().zip(&sidereal.grahas) {
            assert!((a.dig - b.dig).abs() < 1e-9, "{:?}", a.graha);
            assert!((a.drik - b.drik).abs() < 1e-9, "{:?}", a.graha);
            assert!((a.cheshta - b.cheshta).abs() < 1e-9, "{:?}", a.graha);
            assert_eq!(a.temporal, b.temporal, "{:?}", a.graha);
        }
    }

    #[test]
    fn no_report_without_a_birth_time_or_a_sunrise_but_errors_are_still_reported() {
        let mut eph = FakeEph::default();
        let s = CalcSettings::default();
        assert_eq!(
            compute_shadbala(&mut eph, &input(s.clone(), false)),
            Ok(None)
        );
        // Kutup günü ya da gecesi: doğuş ve batış yok.
        let mut polar = FakeEph {
            polar: true,
            ..FakeEph::default()
        };
        assert_eq!(
            compute_shadbala(&mut polar, &input(s.clone(), true)),
            Ok(None)
        );
        // Doğum gününün doğuşu var, sonraki doğuş arama ufkunda yok.
        let mut last = FakeEph {
            sunrises: Some(vec![2_451_544.67]),
            ..FakeEph::default()
        };
        assert_eq!(astro_of(&mut last, &input(s.clone(), true)), None);
        assert_eq!(
            compute_shadbala(&mut last, &input(s.clone(), true)),
            Ok(None)
        );
        // Ayar, an ve konum denetimi saat bilinmese de yapılır.
        let mut bad = input(s.clone(), false);
        bad.lat = 91.0;
        assert_eq!(
            compute_shadbala(&mut eph, &bad),
            Err(CoreError::InvalidLocation {
                lat: 91.0,
                lon: 29.0
            })
        );
        let old = CalcSettings {
            schema_version: 1,
            ..s
        };
        assert_eq!(
            compute_shadbala(&mut eph, &input(old, false)),
            Err(CoreError::UnsupportedSchemaVersion {
                found: 1,
                supported: 2
            })
        );
    }

    #[test]
    fn only_the_low_precision_warning_is_carried() {
        let mut eph = FakeEph {
            precision: Precision::Moshier,
            porphyry_fallback: true,
            ..FakeEph::default()
        };
        let settings = CalcSettings {
            house_system: crate::settings::HouseSystem::Placidus,
            ..CalcSettings::default()
        };
        let report = compute_shadbala(&mut eph, &input(settings, true))
            .unwrap()
            .expect("report");
        assert_eq!(report.warnings, [ChartWarning::LowPrecision]);
    }
}
````

- [ ] **Step 5: Testleri koş**

```bash
cargo test -p jyotish-core 2>&1 | grep -E "^test result|Running"
cargo test -p jyotish-core --lib shadbala::collect 2>&1 | grep -E "^test result"
cargo test -p jyotish-core --lib panchang 2>&1 | grep -E "^test result"
```
Expected: lib `237 passed` (231 + 6), golden `8 passed`, `modules_golden` `9 passed`; ikinci komut `6 passed`; üçüncü komut hatasız (panchang testleri değişmeden geçer).

- [ ] **Step 6: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
git diff --stat -- crates/jyotish-core/src/chart.rs crates/jyotish-core/src/settings.rs
```
Expected: fmt ve clippy çıktısız; `jyotish-core` lib `237 passed`; golden `8 passed` (v1 haritası değişmedi); diğer Rust satırları tabandaki gibi; son komut çıktısız (`Chart` ve ayar tipi değişmedi: K1).

- [ ] **Step 7: Commit**

```bash
git add crates/jyotish-core
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): compute_shadbala collector: sunrise span, MC, tropical and mean longitudes, year and month entry weekdays"
```
Expected: iki `grep` çıktısız.

---

### Task 4: Uygulama: `compute_shadbala` komutu; bağlamalar, sarmalayıcı ve fikstür; astronomik çapraz denetim

Spec §7 "Veri modeli ve IPC", "Test" (astronomik çapraz denetim, uçtan uca), K1, K2, K8; §13 "Komut katmanı" ve katman 4; §15 bütçesi; D8, D9, D17, D18. Komut Plan 14'ün ortak kalıbını izler: motor iş parçacığı, `blocking` havuzu, `CommandError`.

**Files:**
- Modify: `app/src-tauri/src/module_commands.rs`, `app/src-tauri/src/lib.rs`, `app/src-tauri/tests/modules.rs`, `app/src-tauri/tests/ui_fixture.rs`, `app/ui/api.ts`, `app/ui/api.test.ts`, `app/ui/test/fixture.ts`, `app/e2e/ipc.ts`
- Regenerate: `app/ui/bindings.ts`, `app/ui/test/fixtures/delhi.json`
- Test: `app/src-tauri/tests/modules.rs` (`shadbala_checks`), `app/ui/api.test.ts`

**Interfaces:**
- Consumes: `jyotish_core::{compute_shadbala, compute_shadbala_astro, shadbala, sun_entry_before, cheshta_from, mean_longitudes, vara_lord, compute_panchang, ShadbalaReport, SHADBALA_GRAHAS, SHADBALA_METHOD}`; `ephemeris::{Body, Engine, Ephemeris, SwissEph, Zodiac}`; `client_commands::blocking`; `commands::{CommandError, compute_chart_blocking}`; `tests/modules.rs`'teki `golden_inputs`, `median_of_15`.
- Produces:
  - `pub fn compute_shadbala_blocking(engine: &Engine, input: ChartInput) -> Result<Option<ShadbalaReport>, CommandError>`
  - Tauri komutu `compute_shadbala(input: ChartInput) -> Option<ShadbalaReport>`; TS `commands.computeShadbala`, `api.computeShadbala(input: ChartInput): Promise<Result<ShadbalaReport | null>>`
  - TS tipleri (üretilir): `ShadbalaReport`, `GrahaShadbala`, `SthanaBala`, `TemporalBala`, `TimeLords`, `BhavaBala`, `GrahaWar` (her `f64` alanı `number | null`'dır: specta sonlu olmayan sayıyı böyle tipler).
  - `UiFixture.shadbala: ShadbalaReport`; e2e taklidinde `compute_shadbala`.

- [ ] **Step 1: Başarısız testleri yaz**

`shadbala_checks` beş golden anında komutu çağırır, v1 haritası ve toplanan astronomik girdiyle çapraz denetler, giriş anlarında Güneş'in boylamını ve giriş varasını (v1'in `compute_panchang`'ıyla) doğrular; `sky_facts` 1900–2100 arasında gerçek karşı konum ve kavuşum anlarını bulur ve ortalama boylamların uzun dönem ortalamasını ölçer.

````diff
diff --git a/app/src-tauri/tests/modules.rs b/app/src-tauri/tests/modules.rs
index 8ae9caa..326a2dd 100644
--- a/app/src-tauri/tests/modules.rs
+++ b/app/src-tauri/tests/modules.rs
@@ -4,18 +4,20 @@
 use std::path::PathBuf;
 use std::time::{Duration, Instant};
 
-use ephemeris::Engine;
+use ephemeris::{Body, Engine, Ephemeris, SwissEph, Zodiac};
 use jyotish_core::{
-    chara_dasha, co_lords, graha_aspect_signs, house_lord, house_sign, jaimini_report,
-    rashi_drishti_signs, sign_dasha_children, sign_dasha_path_at, sphuta_drishti, AdvancedSettings,
-    CalcSettings, ChartInput, ChartWarning, Graha, KarakaRole, KarakaScheme, Positions, Sign,
-    SignDashaPeriod, Varga, ASPECTING_GRAHAS, CHARA_DASHA_METHOD, DRISHTI_METHOD, JAIMINI_METHOD,
-    SEVEN_KARAKA_ROLES,
+    chara_dasha, cheshta_from, co_lords, compute_panchang, compute_shadbala_astro,
+    graha_aspect_signs, house_lord, house_sign, jaimini_report, mean_longitudes,
+    rashi_drishti_signs, shadbala, sign_dasha_children, sign_dasha_path_at, sphuta_drishti,
+    sun_entry_before, vara_lord, AdvancedSettings, CalcSettings, ChartInput, ChartWarning, Graha,
+    KarakaRole, KarakaScheme, Positions, Sign, SignDashaPeriod, Varga, ASPECTING_GRAHAS,
+    CHARA_DASHA_METHOD, DRISHTI_METHOD, JAIMINI_METHOD, SEVEN_KARAKA_ROLES, SHADBALA_GRAHAS,
+    SHADBALA_METHOD,
 };
 use jyotish_desk_lib::commands::compute_chart_blocking;
 use jyotish_desk_lib::module_commands::{
     chara_dasha_at_blocking, chara_dasha_children_blocking, compute_chara_dasha_blocking,
-    compute_drishti_blocking, compute_jaimini_blocking,
+    compute_drishti_blocking, compute_jaimini_blocking, compute_shadbala_blocking,
 };
 use serde_json::Value;
 
@@ -58,6 +60,7 @@ fn module_commands_run_on_the_engine_thread_and_map_errors() {
     drishti_checks(&engine, &inputs);
     jaimini_checks(&engine, &inputs);
     chara_dasha_checks(&engine, &inputs);
+    shadbala_checks(&engine, &inputs);
 }
 
 fn drishti_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
@@ -642,3 +645,371 @@ fn chara_dasha_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
         "chara_dasha_at median {median:?}"
     );
 }
+
+/// (−180°, 180°] aralığına indirgenmiş açı farkı.
+fn wrap180(x: f64) -> f64 {
+    (x + 180.0).rem_euclid(360.0) - 180.0
+}
+
+/// Ephemeris'ten ölçülen astronomik olgular (ileri modüller spec'i §7 "Astronomik çapraz
+/// denetim" ve "Ortalama boylam katsayıları"), 1900–2100 arasında.
+#[derive(Debug)]
+struct SkyFacts {
+    /// (ad, karşı konumlardaki en küçük cheshta, kavuşumlardaki en büyük cheshta, olay sayısı)
+    outer: Vec<(&'static str, f64, f64, usize)>,
+    /// (ad, alt kavuşumlardaki en küçük cheshta, üst kavuşumlardaki en büyük cheshta, olay
+    /// sayısı, alt kavuşumlarda ortalama boylamın Güneş'in karşısından ortalama farkı)
+    inner: Vec<(&'static str, f64, f64, usize, f64)>,
+    /// (ad, gerçek jeosantrik boylam − ortalama boylam farkının ortalaması)
+    mean_offsets: Vec<(&'static str, f64)>,
+}
+
+fn sky_facts(se: &mut SwissEph) -> SkyFacts {
+    // 1900-01-01 … 2100-01-01 (paketlenen ephemeris dosyalarının içinde).
+    const FROM: f64 = 2_415_020.5;
+    const TO: f64 = 2_488_069.5;
+    fn at(se: &mut SwissEph, t: f64, body: Body) -> (f64, f64) {
+        let p = se.body(t, body, Zodiac::Tropical).expect("body");
+        (p.lon, p.lon_speed)
+    }
+    let mean_of = |name: &str, t: f64| {
+        let m = mean_longitudes(t);
+        match name {
+            "sun" => m.sun,
+            "mercury" => m.mercury,
+            "venus" => m.venus,
+            "mars" => m.mars,
+            "jupiter" => m.jupiter,
+            _ => m.saturn,
+        }
+    };
+
+    // Uzun dönem ortalaması: gerçek − ortalama.
+    let mut mean_offsets = Vec::new();
+    for (name, body) in [
+        ("sun", Body::Sun),
+        ("mars", Body::Mars),
+        ("jupiter", Body::Jupiter),
+        ("saturn", Body::Saturn),
+    ] {
+        let (mut sum, mut n, mut t) = (0.0, 0.0, FROM);
+        while t < TO {
+            sum += wrap180(at(se, t, body).0 - mean_of(name, t));
+            n += 1.0;
+            t += 3.7;
+        }
+        mean_offsets.push((name, sum / n));
+    }
+
+    // Güneş'le kavuşum (`shift` 0) ya da karşı konum (`shift` 180) anları: iki günlük
+    // adımlarla işaret değişimi, sonra ikiye bölme.
+    let events = |se: &mut SwissEph, body: Body, shift: f64| -> Vec<f64> {
+        let gap =
+            |se: &mut SwissEph, t: f64| wrap180(at(se, t, body).0 - at(se, t, Body::Sun).0 - shift);
+        let mut out = Vec::new();
+        let mut t = FROM;
+        let mut prev = gap(se, t);
+        while t < TO {
+            let next_t = t + 2.0;
+            let cur = gap(se, next_t);
+            if prev.abs() < 30.0 && cur.abs() < 30.0 && (prev < 0.0) != (cur < 0.0) {
+                let (mut lo, mut hi) = (t, next_t);
+                for _ in 0..40 {
+                    let mid = 0.5 * (lo + hi);
+                    if (gap(se, mid) < 0.0) == (prev < 0.0) {
+                        lo = mid;
+                    } else {
+                        hi = mid;
+                    }
+                }
+                out.push(0.5 * (lo + hi));
+            }
+            prev = cur;
+            t = next_t;
+        }
+        out
+    };
+
+    let mut outer = Vec::new();
+    for (name, body) in [
+        ("mars", Body::Mars),
+        ("jupiter", Body::Jupiter),
+        ("saturn", Body::Saturn),
+    ] {
+        let cheshta = |se: &mut SwissEph, t: f64| {
+            cheshta_from(mean_of(name, t), at(se, t, body).0, mean_of("sun", t))
+        };
+        let oppositions = events(se, body, 180.0);
+        let conjunctions = events(se, body, 0.0);
+        let low = oppositions
+            .iter()
+            .map(|t| cheshta(se, *t))
+            .fold(f64::MAX, f64::min);
+        let high = conjunctions
+            .iter()
+            .map(|t| cheshta(se, *t))
+            .fold(f64::MIN, f64::max);
+        outer.push((name, low, high, oppositions.len().min(conjunctions.len())));
+    }
+
+    let mut inner = Vec::new();
+    for (name, body) in [("mercury", Body::Mercury), ("venus", Body::Venus)] {
+        let (mut low, mut high) = (f64::MAX, f64::MIN);
+        let (mut inferior, mut offset) = (0usize, 0.0);
+        for t in events(se, body, 0.0) {
+            let (lon, speed) = at(se, t, body);
+            let value = cheshta_from(mean_of("sun", t), lon, mean_of(name, t));
+            // Alt kavuşumda gezegen geri gider; üst kavuşumda ileri.
+            if speed < 0.0 {
+                low = low.min(value);
+                inferior += 1;
+                // Alt kavuşumda güneş merkezli boylam Güneş'in jeosantrik boylamının karşısıdır.
+                offset += wrap180(mean_of(name, t) - at(se, t, Body::Sun).0 - 180.0);
+            } else {
+                high = high.max(value);
+            }
+        }
+        inner.push((name, low, high, inferior, offset / inferior as f64));
+    }
+    SkyFacts {
+        outer,
+        inner,
+        mean_offsets,
+    }
+}
+
+fn shadbala_checks(engine: &Engine, inputs: &[(String, ChartInput)]) {
+    let mut reports = 0;
+    for (id, input) in inputs {
+        let report = compute_shadbala_blocking(engine, input.clone())
+            .unwrap_or_else(|e| panic!("{id}: {}", e.code));
+        let chart = compute_chart_blocking(
+            engine,
+            input.jd_ut,
+            input.lat,
+            input.lon,
+            input.settings.clone(),
+        )
+        .expect("chart");
+        // Doğum saati bilinmiyorsa rapor yoktur (K2).
+        let unknown = ChartInput {
+            time_known: false,
+            ..input.clone()
+        };
+        assert_eq!(
+            compute_shadbala_blocking(engine, unknown).expect("time unknown"),
+            None,
+            "{id}"
+        );
+        let Some(report) = report else {
+            // Rapor ancak gün doğumu ya da batımı yoksa boştur (kutup gecesi ya da gündüzü).
+            assert!(
+                chart
+                    .meta
+                    .warnings
+                    .contains(&ChartWarning::SunriseUnavailable),
+                "{id}: no report although the Sun rises and sets"
+            );
+            continue;
+        };
+        reports += 1;
+        assert_eq!(report.method, SHADBALA_METHOD, "{id}");
+
+        // Çapraz denetim: v1'de golden fixture'larla doğrulanmış haritanın konumlarından ve
+        // toplanan astronomik girdiden aynı rapor çıkar.
+        let (astro, year_entry, month_entry, year_vara, month_vara) = {
+            let (input, chart) = (input.clone(), chart.clone());
+            engine
+                .call(move |se: &mut SwissEph| {
+                    let astro = compute_shadbala_astro(se, &input, &chart)
+                        .expect("astro")
+                        .expect("sunrise and sunset");
+                    let zodiac = Zodiac::Sidereal(input.settings.ayanamsa.sidereal_mode());
+                    let sun = chart.grahas[Graha::Sun as usize].sidereal_lon;
+                    let month_start = (sun / 30.0).floor() * 30.0;
+                    let year = sun_entry_before(se, zodiac, input.jd_ut, 0.0).expect("year entry");
+                    let month = sun_entry_before(se, zodiac, input.jd_ut, month_start)
+                        .expect("month entry");
+                    let sun_at =
+                        |se: &mut SwissEph, t: f64| se.body(t, Body::Sun, zodiac).expect("sun").lon;
+                    let (year_lon, month_lon) = (sun_at(se, year), sun_at(se, month));
+                    // Giriş anının varası, v1'in panchang hesabıyla.
+                    let mut vara = |t: f64| {
+                        compute_panchang(se, t, input.lat, input.lon, &input.settings)
+                            .expect("panchang")
+                            .vara
+                    };
+                    let varas = (vara(year), vara(month));
+                    (
+                        astro,
+                        (year, year_lon),
+                        (month, wrap180(month_lon - month_start)),
+                        varas.0,
+                        varas.1,
+                    )
+                })
+                .expect("engine")
+        };
+        let want = shadbala(&Positions::from(&chart), &astro).expect("lagna");
+        assert_eq!(report.grahas, want.grahas, "{id}");
+        assert_eq!(report.bhavas, want.bhavas, "{id}");
+        assert_eq!(report.time_lords, want.time_lords, "{id}");
+
+        // Yıl girişi: Güneş'in sidereal boylamı 0° ± 1″; giriş doğumdan önce, en çok bir yıl.
+        const ARCSECOND: f64 = 1.0 / 3600.0;
+        assert!(
+            wrap180(year_entry.1).abs() < ARCSECOND,
+            "{id}: Sun at the year entry is at {}",
+            year_entry.1
+        );
+        let back = input.jd_ut - year_entry.0;
+        assert!((0.0..366.5).contains(&back), "{id}: year entry {back} days");
+        assert!(month_entry.1.abs() < ARCSECOND, "{id}: month entry");
+        let back = input.jd_ut - month_entry.0;
+        assert!((0.0..32.5).contains(&back), "{id}: month entry {back} days");
+        assert_eq!(astro.year_entry_weekday, year_vara, "{id}");
+        assert_eq!(astro.month_entry_weekday, month_vara, "{id}");
+        assert_eq!(astro.weekday, chart.panchang.vara, "{id}");
+        assert_eq!(Some(astro.sunrise_jd_ut), chart.panchang.sunrise_jd_ut);
+        assert_eq!(report.time_lords.weekday, vara_lord(chart.panchang.vara));
+        assert_eq!(report.time_lords.year, vara_lord(year_vara), "{id}");
+        assert_eq!(report.time_lords.month, vara_lord(month_vara), "{id}");
+        // Tropikal boylam, ephemeris'in tropikal çerçevedeki değeridir.
+        let jd = input.jd_ut;
+        let tropical_sun = engine
+            .call(move |se: &mut SwissEph| se.body(jd, Body::Sun, Zodiac::Tropical))
+            .expect("engine")
+            .expect("sun")
+            .lon;
+        assert!(
+            wrap180(astro.tropical_lons[0] - tropical_sun).abs() < 1e-6,
+            "{id}: tropical Sun {} vs {tropical_sun}",
+            astro.tropical_lons[0]
+        );
+
+        // Değişmezler gerçek haritalarda da sağlanır.
+        let mut ranks: Vec<u8> = report.grahas.iter().map(|g| g.rank).collect();
+        ranks.sort_unstable();
+        assert_eq!(ranks, [1, 2, 3, 4, 5, 6, 7], "{id}");
+        let mut ranks: Vec<u8> = report.bhavas.iter().map(|b| b.rank).collect();
+        ranks.sort_unstable();
+        assert_eq!(ranks, (1..=12).collect::<Vec<u8>>(), "{id}");
+        for (row, g) in report.grahas.iter().zip(SHADBALA_GRAHAS) {
+            assert_eq!(row.graha, g, "{id}");
+            let t = &row.temporal;
+            assert!((0.0..=60.0).contains(&row.dig), "{id}: {g:?} dig");
+            assert!((0.0..=60.0).contains(&row.cheshta), "{id}: {g:?} cheshta");
+            assert!((0.0..=60.0).contains(&t.day_night), "{id}: {g:?}");
+            assert_eq!(t.year_lord == 15.0, report.time_lords.year == g, "{id}");
+            assert_eq!(t.month_lord == 30.0, report.time_lords.month == g, "{id}");
+            assert_eq!(t.weekday_lord == 45.0, report.time_lords.weekday == g);
+            assert_eq!(t.hora_lord == 60.0, report.time_lords.hora == g, "{id}");
+            assert_eq!(
+                t.tribhaga == 60.0,
+                g == Graha::Jupiter || report.time_lords.tribhaga == g,
+                "{id}"
+            );
+            let total =
+                row.sthana.total + row.dig + t.total + row.cheshta + row.naisargika + row.drik;
+            assert!((row.total - total).abs() < 1e-9, "{id}: {g:?} total");
+        }
+        for b in &report.bhavas {
+            assert_eq!(b.lord, house_lord(chart.lagna.sign, b.house), "{id}");
+            assert_eq!(b.lord_bala, report.grahas[b.lord as usize].total, "{id}");
+        }
+    }
+    // Beş andan biri kutup gecesindedir (Tromsø, Aralık): raporu yoktur.
+    assert_eq!(reports, inputs.len() - 1, "reports for the golden moments");
+
+    // Hatalar `compute_chart` ile aynı kodlara eşlenir; saat bilinmese de denetlenir.
+    let delhi = inputs
+        .iter()
+        .find(|(id, _)| id == "new-delhi-2000-01-01")
+        .expect("delhi fixture")
+        .1
+        .clone();
+    let bad_place = ChartInput {
+        lat: 91.0,
+        time_known: false,
+        ..delhi.clone()
+    };
+    assert_eq!(
+        compute_shadbala_blocking(engine, bad_place)
+            .unwrap_err()
+            .code,
+        "invalid_location"
+    );
+    let old_settings = ChartInput {
+        settings: CalcSettings {
+            schema_version: 1,
+            ..CalcSettings::default()
+        },
+        ..delhi.clone()
+    };
+    assert_eq!(
+        compute_shadbala_blocking(engine, old_settings)
+            .unwrap_err()
+            .code,
+        "unsupported_schema_version"
+    );
+
+    // Astronomik çapraz denetim (spec §7): dış gezegen karşı konumdayken cheshta ≥ 55,
+    // kavuşumdayken ≤ 5. İç gezegenlerde alt kavuşumda ≥ 50, üst kavuşumda ≤ 10 (Merkür'ün
+    // yörünge dış merkezliği ortalama boylamı gerçek boylamdan 23°'ye kadar ayırır).
+    let facts = engine.call(sky_facts).expect("engine");
+    eprintln!("{facts:?}");
+    for (name, at_opposition, at_conjunction, count) in &facts.outer {
+        assert!(*count >= 90, "{name}: {count} oppositions and conjunctions");
+        assert!(
+            *at_opposition >= 55.0,
+            "{name}: cheshta {at_opposition} at an opposition"
+        );
+        assert!(
+            *at_conjunction <= 5.0,
+            "{name}: cheshta {at_conjunction} at a conjunction"
+        );
+    }
+    for (name, at_inferior, at_superior, count, offset) in &facts.inner {
+        assert!(*count >= 120, "{name}: {count} inferior conjunctions");
+        assert!(
+            *at_inferior >= 50.0,
+            "{name}: cheshta {at_inferior} at an inferior conjunction"
+        );
+        assert!(
+            *at_superior <= 10.0,
+            "{name}: cheshta {at_superior} at a superior conjunction"
+        );
+        // Güneş merkezli ortalama boylamın denetimi: alt kavuşumlar yörüngeye yayıldığı için
+        // fark ortalamada sıfırdır. Merkür'de örnekleme gürültüsü büyüktür.
+        let limit = if *name == "mercury" { 0.5 } else { 0.05 };
+        assert!(
+            offset.abs() <= limit,
+            "{name}: mean heliocentric longitude is off by {offset}"
+        );
+    }
+    // Ortalama boylam katsayıları (spec §7, §17): gerçek boylamın uzun dönem ortalaması
+    // ortalama boylamdır. Güneş ve Mars ±0,05° içindedir; Jüpiter ve Satürn'de ~900 yıllık
+    // karşılıklı tedirginlik (büyük eşitsizlik) 200 yılda ortalanmaz (D-kararı; ölçüm: +0,055°
+    // ve −0,212°).
+    for (name, offset) in &facts.mean_offsets {
+        let limit = match *name {
+            "jupiter" => 0.1,
+            "saturn" => 0.3,
+            _ => 0.05,
+        };
+        assert!(
+            offset.abs() <= limit,
+            "{name}: long-term mean of true minus mean longitude is {offset}"
+        );
+    }
+
+    // Performans: spec §15 "compute_shadbala ≤ 40 ms" (temel harita, yıl ve ay girişi dahil).
+    let (median, max) = median_of_15(|| {
+        compute_shadbala_blocking(engine, delhi.clone()).expect("shadbala");
+    });
+    eprintln!("compute_shadbala median {median:?}, max {max:?}");
+    assert!(
+        median <= Duration::from_millis(40),
+        "compute_shadbala median {median:?}"
+    );
+}
````

````diff
diff --git a/app/src-tauri/tests/ui_fixture.rs b/app/src-tauri/tests/ui_fixture.rs
index e60289e..58afd56 100644
--- a/app/src-tauri/tests/ui_fixture.rs
+++ b/app/src-tauri/tests/ui_fixture.rs
@@ -1,7 +1,7 @@
 //! Arayüz testlerinin (vitest ve Playwright) gerçek veri fikstürü:
 //! `app/ui/test/fixtures/delhi.json`. Gerçek motor, store ve atlasla üretilir; saat sabit,
 //! UUID'ler sabit dizgiyle değiştirilir. Arayüz kendi haritasını hesaplamadığı için
-//! (spec §4) testler bu dosyadaki `OpenedClient`'ı çizer.
+//! (spec §4) testler bu dosyadaki `OpenedClient`'ı ve modül sonuçlarını çizer.
 //! Güncellemek için: `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture`.
 //! Denetim sayılarda göreli 1e-9 toleranslıdır: CI derleyicisi ve libm son basamakta
 //! farklı sonuç verebilir. Motoru tek test açar: süreçte yalnızca bir SE örneği olabilir.
@@ -17,7 +17,7 @@ use jyotish_desk_lib::client_commands::create_client_blocking;
 use jyotish_desk_lib::commands::dasha_children;
 use jyotish_desk_lib::module_commands::{
     chara_dasha_at_blocking, chara_dasha_children_blocking, compute_chara_dasha_blocking,
-    compute_drishti_blocking, compute_jaimini_blocking,
+    compute_drishti_blocking, compute_jaimini_blocking, compute_shadbala_blocking,
 };
 use jyotish_desk_lib::transit_commands::{transits_at_blocking, TransitAt};
 use serde_json::{json, Value};
@@ -123,11 +123,15 @@ fn ui_fixture_is_up_to_date() {
     // Transit anındaki alt dönemin 3. seviyesi: dasa şeridinin "pratyantar" görünümü.
     let chara_at_children = chara_dasha_children_blocking(
         &engine,
-        module_input,
+        module_input.clone(),
         chara_at.signs[..2].to_vec(),
         chara_at.cycle,
     )
     .expect("chara dasha children at the transit moment");
+    // Shadbala ve Bhava bala: aynı girdiyle (doğum saati biliniyor, Güneş doğup batıyor).
+    let shadbala = compute_shadbala_blocking(&engine, module_input)
+        .expect("shadbala")
+        .expect("time known and a sunrise");
     // Kaydedilen doğum verisi girdinin kanonik biçimidir (Plan 10 kapsam dışı 21).
     assert_eq!(opened.chart_record.birth, delhi());
     let prefs = store
@@ -149,6 +153,7 @@ fn ui_fixture_is_up_to_date() {
         "chara_dasha_children": { "path": chara_path, "cycle": 1, "periods": chara_children },
         "chara_dasha_at": chara_at,
         "chara_dasha_at_children": chara_at_children,
+        "shadbala": shadbala,
     });
     let text = serde_json::to_string(&value)
         .expect("serialize")
````

````diff
diff --git a/app/ui/api.test.ts b/app/ui/api.test.ts
index a7aaef8..a3206e3 100644
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -33,6 +33,7 @@ vi.mock("./bindings", () => ({
     computeCharaDasha: vi.fn(),
     charaDashaChildren: vi.fn(),
     charaDashaAt: vi.fn(),
+    computeShadbala: vi.fn(),
     searchPlaces: vi.fn(),
     previewMoment: vi.fn(),
     getAppPrefs: vi.fn(),
@@ -117,6 +118,7 @@ test("every wrapper forwards its arguments to the command of the same name", asy
     ["computeCharaDasha", [{ jd_ut: 1, time_known: true }]],
     ["charaDashaChildren", [{ jd_ut: 1, time_known: true }, ["Aries", "Taurus"], 2]],
     ["charaDashaAt", [{ jd_ut: 1, time_known: true }, 2451545.5]],
+    ["computeShadbala", [{ jd_ut: 1, time_known: true }]],
     ["searchPlaces", ["delhi", 8]],
     ["previewMoment", [{ date: "2000-01-01" }]],
     ["getAppPrefs", []],
````

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
cargo test -p jyotish-desk --test modules --test ui_fixture 2>&1 | grep -E "^error" | sort | uniq -c
pnpm --dir app test 2>&1 | grep -E "Tests |TypeError"
```
Expected: ilk komutta iki `error[E0432]: unresolved import `jyotish_desk_lib::module_commands::compute_shadbala_blocking`` ve iki `could not compile`; ikincide `TypeError: call is not a function` ve `1 failed | 652 passed (653)`.

- [ ] **Step 3: Komutu yaz ve kaydet**

````diff
diff --git a/app/src-tauri/src/module_commands.rs b/app/src-tauri/src/module_commands.rs
index 08c426e..bed1562 100644
--- a/app/src-tauri/src/module_commands.rs
+++ b/app/src-tauri/src/module_commands.rs
@@ -7,7 +7,8 @@ use std::sync::Arc;
 
 use ephemeris::{Engine, SwissEph};
 use jyotish_core::{
-    CharaDasha, ChartInput, DrishtiTable, JaiminiReport, Sign, SignDashaPath, SignDashaPeriod,
+    CharaDasha, ChartInput, DrishtiTable, JaiminiReport, ShadbalaReport, Sign, SignDashaPath,
+    SignDashaPeriod,
 };
 use tauri::State;
 
@@ -134,3 +135,26 @@ pub async fn chara_dasha_at(
     let engine = Arc::clone(&engine);
     blocking(move || chara_dasha_at_blocking(&engine, input, jd_ut)).await
 }
+
+/// `compute_shadbala` komutunun gövdesi; bloklar.
+pub fn compute_shadbala_blocking(
+    engine: &Engine,
+    input: ChartInput,
+) -> Result<Option<ShadbalaReport>, CommandError> {
+    let report =
+        engine.call(move |se: &mut SwissEph| jyotish_core::compute_shadbala(se, &input))??;
+    Ok(report)
+}
+
+/// Shadbala ve Bhava bala: yedi grahanın altı gücü, toplamı, gerekli değere oranı ve sırası;
+/// zaman lordları; on iki bhavanın gücü. Doğum saati bilinmiyorsa (`time_known == false`) ya da
+/// doğum yerinde o gün gün doğumu ya da batımı yoksa `null` döner.
+#[tauri::command]
+#[specta::specta]
+pub async fn compute_shadbala(
+    engine: State<'_, Arc<Engine>>,
+    input: ChartInput,
+) -> Result<Option<ShadbalaReport>, CommandError> {
+    let engine = Arc::clone(&engine);
+    blocking(move || compute_shadbala_blocking(&engine, input)).await
+}
````

````diff
diff --git a/app/src-tauri/src/lib.rs b/app/src-tauri/src/lib.rs
index 2d9f9f3..d347d44 100644
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -34,6 +34,7 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             module_commands::compute_chara_dasha,
             module_commands::chara_dasha_children,
             module_commands::chara_dasha_at,
+            module_commands::compute_shadbala,
             client_commands::open_client,
             client_commands::restore_session,
             client_commands::create_client,
````

- [ ] **Step 4: Sarmalayıcı, fikstür tipi ve e2e taklidi; üretilen dosyaları yeniden üret**

````diff
diff --git a/app/ui/api.ts b/app/ui/api.ts
index ff9ea85..7c4f834 100644
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -86,6 +86,9 @@ export const api = {
   computeCharaDasha: (input: ChartInput) => wrap(() => commands.computeCharaDasha(input)),
   charaDashaChildren: (input: ChartInput, path: Sign[], cycle: number) => wrap(() => commands.charaDashaChildren(input, path, cycle)),
   charaDashaAt: (input: ChartInput, jdUt: number) => wrap(() => commands.charaDashaAt(input, jdUt)),
+  // Shadbala ve Bhava bala: `null` doğum saati bilinmeyen ya da o gün Güneş'in doğmadığı ya da
+  // batmadığı haritadır.
+  computeShadbala: (input: ChartInput) => wrap(() => commands.computeShadbala(input)),
   searchPlaces: (query: string, limit: number) => wrap(() => commands.searchPlaces(query, limit)),
   previewMoment: (birth: BirthData) => wrap(() => commands.previewMoment(birth)),
   appInfo: () => wrap(() => commands.appInfo()),
````

````diff
diff --git a/app/ui/test/fixture.ts b/app/ui/test/fixture.ts
index 6433569..65129e4 100644
--- a/app/ui/test/fixture.ts
+++ b/app/ui/test/fixture.ts
@@ -1,7 +1,7 @@
 // Gerçek veri fikstürü (app/src-tauri/tests/ui_fixture.rs üretir): Delhi 2000-01-01 17:30
 // için `create_client` dönüşü, ilk 3. seviye dönemin alt dönemleri, "new delhi" atlas araması,
-// an önizlemesi, bir DST boşluğu hatası, aynı sabit saatte transit, bakış tablosu, Jaimini raporu
-// ve chara dasha. Arayüz hesap yapmadığı için testler bunu çizer.
+// an önizlemesi, bir DST boşluğu hatası, aynı sabit saatte transit, bakış tablosu, Jaimini raporu,
+// chara dasha ve Shadbala raporu. Arayüz hesap yapmadığı için testler bunu çizer.
 import type {
   CharaDasha,
   CommandError,
@@ -13,6 +13,7 @@ import type {
   OpenedClient,
   PlaceHit,
   PrefsLoad,
+  ShadbalaReport,
   Sign,
   SignDashaPath,
   SignDashaPeriod,
@@ -42,6 +43,8 @@ export interface UiFixture {
   chara_dasha_at: SignDashaPath;
   /** `chara_dasha_children`: transit anındaki alt dönemin (`chara_dasha_at` yolunun ilk iki burcu) 3. seviyesi. */
   chara_dasha_at_children: SignDashaPeriod[];
+  /** `compute_shadbala`, aynı girdiyle: yedi graha, zaman lordları, on iki bhava. */
+  shadbala: ShadbalaReport;
 }
 
 const FIXTURE = JSON.parse(raw) as UiFixture;
````

````diff
diff --git a/app/e2e/ipc.ts b/app/e2e/ipc.ts
index 7bb6ab6..d823b43 100644
--- a/app/e2e/ipc.ts
+++ b/app/e2e/ipc.ts
@@ -110,6 +110,8 @@ export async function mockIpc(page: Page, { session, unknownTime = false }: IpcO
           return [];
         },
         chara_dasha_at: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.chara_dasha_at : null),
+        // Shadbala: fikstürdeki gerçek sonuç; saat bilinmiyorsa Rust `null` döner.
+        compute_shadbala: (args) => ((args.input as { time_known: boolean }).time_known ? fixture.shadbala : null),
         print_window: () => null,
         // Güncelleme: sahte yayın; ağ yoktur. Kurulum yalnızca denetimin bildirdiği sürümü kabul eder.
         app_info: () => ({ version: "0.9.0", updaterConfigured: true }),
````

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep -E "^test result"
git diff --numstat -- app/ui/bindings.ts app/ui/test/fixtures/delhi.json
grep -c "computeShadbala" app/ui/bindings.ts
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
```
Expected: iki `ok`; `bindings.ts` 93 satır kazanır (bir komut ve yedi tip); `delhi.json` tek satır değişir; `grep -c` `1`, son komut `0` yazar. Fikstürdeki rapor (Delhi 2000-01-01, İkizler lagna 16°20′, varsayılan ayarlar) sonraki görevlerin panel testlerinin beklediği değerleri taşır: toplamlar (virupa) Güneş 505,3; Ay 413,5; Mars 412,6; Merkür 454,1; Jüpiter 533,1; Venüs 371,0; Satürn 353,7; sıra Jüpiter, Güneş, Merkür, Ay, Mars, Venüs, Satürn; bütün oranlar 1'in üstünde (1,08–1,38); savaş yok; zaman lordları: yıl Merkür, ay Jüpiter, gün Satürn, saat Güneş, tribhaga Satürn; bhava sırası 7, 1, 3, 10, 4, 2, 6, 11, 5, 12, 9, 8.

- [ ] **Step 5: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
cargo test -p jyotish-desk --test modules -- --nocapture 2>&1 | grep -E "median|SkyFacts"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
```
Expected: fmt ve clippy çıktısız; `jyotish-desk` `modules` `1 passed`, `bindings` `1 passed`, `ui_fixture` `2 passed`; diğer Rust satırları Task 3'teki gibi; altı `median` satırı (`compute_drishti`, `compute_jaimini`, `compute_chara_dasha`, `chara_dasha_children`, `chara_dasha_at`, `compute_shadbala`), hepsi bütçesinin altında (ölçüm: 0,5–0,8 ms; `compute_shadbala` 0,69–0,74 ms); bir `SkyFacts` satırı (ölçüm: `outer` Mars 57,75 / 1,53 / 94; Jüpiter 58,85 / 1,19 / 183; Satürn 59,46 / 1,77 / 193; `inner` Merkür 52,34 / 8,26 / 630 / +0,260; Venüs 59,83 / 0,57 / 125 / −0,008; `mean_offsets` Güneş −0,0071; Mars −0,0271; Jüpiter +0,0549; Satürn −0,2120); Vitest `70 passed (70)`, `653 passed (653)` (`api.test.ts`'in tablosu büyür, test sayısı değişmez; "dizgi bağımsız değişken" testi değişmeden geçer: C2′); `✓ built`.

- [ ] **Step 6: Commit**

```bash
git add app/src-tauri/src app/src-tauri/tests app/ui/bindings.ts app/ui/test app/ui/api.ts app/ui/api.test.ts app/e2e/ipc.ts
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(app): compute_shadbala command; bindings, wrapper and fixture; astronomical cross-checks on the engine"
```
Expected: iki `grep` çıktısız.

---

### Task 5: Arayüz: Shadbala paneli, ortak kanca ve ortak parçalar

Spec §7 "Arayüz" (Shadbala satırı ve üç madde), K1, K11, K12; §13 "Arayüz" (beş durum, ortak seçim, klavye, i18n); UI spec §3.3, §3.4, §5, §8; D19–D21.

**Files:**
- Create: `app/ui/modules/useShadbala.ts`, `app/ui/panels/bala.tsx`, `app/ui/panels/ShadbalaPanel.tsx`, `app/ui/panels/ShadbalaPanel.test.tsx`
- Modify: `app/ui/layout/model.ts`, `app/ui/layout/model.test.ts`, `app/ui/panels/registry.ts`, `app/ui/panels/panels.test.tsx`, `app/ui/panels/panels.css`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`
- Test: `ShadbalaPanel.test.tsx` (12 test), `model.test.ts`, `panels.test.tsx`, `i18n.test.ts` (değişmeden)

**Interfaces:**
- Consumes: `api.computeShadbala`, `ShadbalaReport` ve alt tipleri (Task 4); `modules/input.ts` (`chartInput`, `inputKey`), `modules/useModule.ts`; `panels/states.tsx` (`PanelError`, `PanelSkeleton`, `usePanelLoadPhase`), `panels/PinButton.tsx`; `highlight/store`; `view/store` (`useLayoutStore().setPanelSettings`); `session/birth.ts` (`timeUnknown`); `test/fixture.ts`, `test/render.tsx`.
- Produces:
  - `useShadbala(opened: OpenedClient | null): ModuleState<ShadbalaReport | null>`
  - `panels/bala.tsx`: `BALA_UNITS`, `BalaUnit`, `VIRUPAS_PER_RUPA`, `balaUnit(settings)`, `formatFixed(value: number | null, digits)`, `formatBala(virupa: number | null, unit)`, `UnitSelect({ panel })`, `BalaUnavailable({ opened })`
  - `ShadbalaPanel`, `SHADBALA_DETAILS`, `ShadbalaDetail`, `shadbalaDetail(settings)`, `BELOW_MARK = "<1"`
  - Panel türü `shadbala` (`PANEL_KINDS`, `MODULE_PANELS` `{ w: 8, h: 12 }`, `PANEL_REGISTRY`); panel ayarları `detail: "summary" | "items"`, `unit: "virupa" | "rupa"`
  - i18n: `panel.shadbala`, `bala.*`, `shadbala.*`

- [ ] **Step 1: Başarısız testleri yaz**

**Yeni dosya:** `app/ui/panels/ShadbalaPanel.test.tsx`

````tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeShadbala: vi.fn() } }));

import { api, type Result } from "../api";
import type { ShadbalaReport } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useLayoutStore } from "../view/store";
import { balaUnit, formatBala, formatFixed } from "./bala";
import { BELOW_MARK, ShadbalaPanel, shadbalaDetail } from "./ShadbalaPanel";

const ok = (report: ShadbalaReport | null = fixture().shadbala): Result<ShadbalaReport | null> => ({ status: "ok", data: report });
const LAYOUT = (settings: Record<string, unknown>) => {
  const panel: Panel = { id: "sb", kind: "shadbala", x: 0, y: 0, w: 8, h: 12, settings };
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
};

/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  return <ShadbalaPanel panel={active.panels[0]} />;
}

function renderPanel(settings: Record<string, unknown> = {}, opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  return renderWithStores(<LivePanel />, { layout: LAYOUT(settings), opened });
}

const CAPTION = "Shadbala: yedi grahanın altı gücü, toplamı, gerekli en az değeri, oranı ve sırası";
const summary = () => screen.findByRole("table", { name: CAPTION });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const bodyRows = (table: HTMLElement) => within(table).getAllByRole("row").slice(1).map(cells);

beforeEach(() => {
  vi.mocked(api.computeShadbala).mockReset();
  vi.mocked(api.computeShadbala).mockImplementation(async () => ok());
});

test("summary: seven grahas with the six strengths, total, required value, ratio and rank; time lords below", async () => {
  renderPanel();
  const table = await summary();
  const opened = fixture().opened;
  expect(api.computeShadbala).toHaveBeenCalledTimes(1);
  expect(api.computeShadbala).toHaveBeenCalledWith({ jd_ut: 2451545, lat: 28.6139, lon: 77.209, settings: opened.chart.meta.settings, time_known: true });
  expect(cells(within(table).getAllByRole("row")[0])).toEqual(["Graha", "Sthana", "Dig", "Zaman gücü", "Cheshta", "Naisargika", "Drik", "Toplam", "Gerekli", "Oran", "Sıra"]);
  // Delhi fikstürü, virupa, bir ondalık; oran iki ondalık.
  expect(bodyRows(table)).toEqual([
    ["Güneş", "216.2", "34.3", "137.9", "1.2", "60.0", "55.8", "505.3", "390.0", "1.30", "2"],
    ["Ay", "141.5", "44.7", "113.3", "19.0", "51.4", "43.5", "413.5", "360.0", "1.15", "4"],
    ["Mars", "248.0", "50.2", "81.2", "20.4", "17.1", "−4.3", "412.6", "300.0", "1.38", "5"],
    ["Merkür", "182.3", "2.8", "175.3", "8.0", "25.7", "60.0", "454.1", "420.0", "1.08", "3"],
    ["Jüpiter", "233.8", "35.0", "185.8", "36.4", "34.3", "7.8", "533.1", "390.0", "1.37", "1"],
    ["Venüs", "168.6", "38.6", "57.9", "26.3", "42.9", "36.7", "371.0", "330.0", "1.12", "6"],
    ["Satürn", "108.2", "19.9", "182.8", "41.6", "8.6", "−7.4", "353.7", "300.0", "1.18", "7"],
  ]);
  // Özet görünümünde alt kalem tabloları yoktur.
  expect(screen.getAllByRole("table")).toHaveLength(1);
  const lords = screen.getByRole("list", { name: "Zaman lordları" });
  expect(within(lords).getAllByRole("listitem").map((li) => li.textContent)).toEqual([
    "Yıl lordu: Merkür",
    "Ay lordu: Jüpiter",
    "Gün lordu: Satürn",
    "Saat lordu: Güneş",
    "Tribhaga lordu: Satürn",
  ]);
  expect(screen.getByText("Değerler virupa cinsindendir (60 virupa = 1 rupa).")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: shadbala.bphs_v1")).toBeInTheDocument();
  // Fikstürde bütün oranlar 1'in üstündedir: işaret ve açıklaması yoktur.
  expect(screen.queryByText(BELOW_MARK)).toBeNull();
  expect(screen.queryByText(/gerekli en az değerin altında/)).toBeNull();
});

test("items view adds the items of sthana bala and of the temporal strength", async () => {
  renderPanel({ detail: "items" });
  await summary();
  expect(screen.getByRole("combobox", { name: "Ayrıntı" })).toHaveDisplayValue("Kalemler");
  const sthana = screen.getByRole("table", { name: "Sthana bala kalemleri" });
  expect(cells(within(sthana).getAllByRole("row")[0])).toEqual(["Graha", "Uccha", "Saptavargaja", "Ojayugma", "Kendradi", "Drekkana", "Toplam"]);
  expect(bodyRows(sthana)).toEqual([
    ["Güneş", "22.2", "104.0", "30.0", "60.0", "0.0", "216.2"],
    ["Ay", "4.5", "92.0", "15.0", "30.0", "0.0", "141.5"],
    ["Mars", "58.0", "145.0", "15.0", "15.0", "15.0", "248.0"],
    ["Merkür", "32.3", "60.0", "30.0", "60.0", "0.0", "182.3"],
    ["Jüpiter", "28.8", "130.0", "30.0", "30.0", "15.0", "233.8"],
    ["Venüs", "13.6", "110.0", "30.0", "15.0", "0.0", "168.6"],
    ["Satürn", "1.2", "32.0", "30.0", "30.0", "15.0", "108.2"],
  ]);
  const temporal = screen.getByRole("table", { name: "Zaman gücü kalemleri" });
  expect(cells(within(temporal).getAllByRole("row")[0])).toEqual(["Graha", "Gündüz–gece", "Paksha", "Tribhaga", "Yıl lordu", "Ay lordu", "Gün lordu", "Saat lordu", "Ayana", "Toplam"]);
  expect(bodyRows(temporal)).toEqual([
    ["Güneş", "34.5", "41.0", "0.0", "0.0", "0.0", "0.0", "60.0", "2.4", "137.9"],
    ["Ay", "25.5", "38.0", "0.0", "0.0", "0.0", "0.0", "0.0", "49.8", "113.3"],
    ["Mars", "25.5", "41.0", "0.0", "0.0", "0.0", "0.0", "0.0", "14.8", "81.2"],
    ["Merkür", "60.0", "41.0", "0.0", "15.0", "0.0", "0.0", "0.0", "59.3", "175.3"],
    ["Jüpiter", "34.5", "19.0", "60.0", "0.0", "30.0", "0.0", "0.0", "42.2", "185.8"],
    ["Venüs", "34.5", "19.0", "0.0", "0.0", "0.0", "0.0", "0.0", "4.4", "57.9"],
    ["Satürn", "25.5", "41.0", "60.0", "0.0", "0.0", "45.0", "0.0", "11.3", "182.8"],
  ]);
});

test("detail and unit are saved with the layout, redraw the panel and ask for nothing again", async () => {
  const { persistence } = renderPanel();
  const table = await summary();
  fireEvent.change(screen.getByRole("combobox", { name: "Birim" }), { target: { value: "rupa" } });
  // Rupa: iki ondalık; toplam Rust'ın `rupas` alanıdır; oran ve sıra birimden bağımsızdır.
  await waitFor(() => expect(bodyRows(table)[0]).toEqual(["Güneş", "3.60", "0.57", "2.30", "0.02", "1.00", "0.93", "8.42", "6.50", "1.30", "2"]));
  expect(bodyRows(table)[2]).toEqual(["Mars", "4.13", "0.84", "1.35", "0.34", "0.29", "−0.07", "6.88", "5.00", "1.38", "5"]);
  expect(screen.getByText("Değerler rupa cinsindendir (1 rupa = 60 virupa).")).toBeInTheDocument();
  fireEvent.change(screen.getByRole("combobox", { name: "Ayrıntı" }), { target: { value: "items" } });
  await screen.findByRole("table", { name: "Sthana bala kalemleri" });
  await waitFor(() =>
    expect(persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ unit: "rupa", detail: "items" }),
  );
  expect(api.computeShadbala).toHaveBeenCalledTimes(1);
});

test("setting readers and number formats: unknown values fall back; minus is typographic; no negative zero", () => {
  expect([shadbalaDetail({}), shadbalaDetail({ detail: "all" }), shadbalaDetail({ detail: "items" })]).toEqual(["summary", "summary", "items"]);
  expect([balaUnit({}), balaUnit({ unit: "shashtiamsa" }), balaUnit({ unit: "rupa" })]).toEqual(["virupa", "virupa", "rupa"]);
  expect([formatBala(505.34, "virupa"), formatBala(505.34, "rupa"), formatBala(-7.35, "virupa"), formatBala(-7.35, "rupa")]).toEqual(["505.3", "8.42", "−7.3", "−0.12"]);
  expect([formatBala(-0.04, "virupa"), formatBala(-0.2, "rupa"), formatBala(0, "virupa")]).toEqual(["0.0", "0.00", "0.0"]);
  expect([formatBala(null, "virupa"), formatFixed(null, 2), formatFixed(Number.NaN, 2), formatFixed(Number.POSITIVE_INFINITY, 1)]).toEqual(["—", "—", "—", "—"]);
});

test("a ratio below one carries the text mark and a legend; a graha war is written as text", async () => {
  const report = fixture().shadbala;
  report.grahas[5].ratio = 0.94;
  report.grahas[2].war = { with: "Venus", winner: "Mars" };
  report.grahas[5].war = { with: "Mars", winner: "Mars" };
  vi.mocked(api.computeShadbala).mockResolvedValue(ok(report));
  renderPanel();
  const rows = within(await summary()).getAllByRole("row");
  const venus = rows[6];
  expect(venus.querySelectorAll("td")[8].textContent).toBe(`0.94 ${BELOW_MARK}gerekli değerin altında`);
  // İşaret ekran okuyucudan gizlidir; anlamı sözcükle okunur.
  expect(within(venus).getByText(BELOW_MARK)).toHaveAttribute("aria-hidden", "true");
  expect(within(venus).getByText("gerekli değerin altında")).toHaveClass("visually-hidden");
  expect(rows[1].querySelectorAll("td")[8].textContent).toBe("1.30");
  expect(screen.getByText(`${BELOW_MARK} toplam, klasik gerekli en az değerin altında.`)).toBeInTheDocument();
  // Savaş: kısa metin görünür, tam adlar ve kazanan ekran okuyucuya okunur.
  const mars = rows[3];
  expect(within(mars).getByText("savaş: Ve")).toHaveAttribute("aria-hidden", "true");
  expect(within(mars).getByText("Venüs ile graha savaşında; kazanan Mars")).toHaveClass("visually-hidden");
  expect(within(venus).getByText("savaş: Ma")).toBeInTheDocument();
  expect(within(rows[1]).queryByText(/savaş/)).toBeNull();
});

test("unknown birth time: Rust returns no report and the panel says why; the toolbar stays", async () => {
  vi.mocked(api.computeShadbala).mockResolvedValue(ok(null));
  renderPanel({}, unknownTimeOpened());
  expect(await screen.findByText("Doğum saati bilinmiyor: Shadbala ve Bhava bala doğum saatine bağlı olduğu için hesaplanmıyor.")).toBeInTheDocument();
  expect(vi.mocked(api.computeShadbala).mock.calls[0][0].time_known).toBe(false);
  expect(screen.queryByRole("table")).toBeNull();
  expect(screen.getByRole("combobox", { name: "Ayrıntı" })).toBeInTheDocument();
  expect(screen.getByRole("combobox", { name: "Birim" })).toBeInTheDocument();
});

test("no sunrise or sunset: with a known birth time the missing report is explained by the Sun", async () => {
  vi.mocked(api.computeShadbala).mockResolvedValue(ok(null));
  renderPanel();
  expect(
    await screen.findByText("Doğum yerinde o gün gün doğumu ya da batımı yok: zaman gücü ölçülemediği için Shadbala ve Bhava bala hesaplanmıyor."),
  ).toBeInTheDocument();
  expect(vi.mocked(api.computeShadbala).mock.calls[0][0].time_known).toBe(true);
  expect(screen.queryByRole("table")).toBeNull();
});

test("without an open chart nothing is requested", () => {
  renderPanel({}, null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeShadbala).not.toHaveBeenCalled();
});

test("an error is shown in the panel with the toolbar and retry asks again", async () => {
  vi.mocked(api.computeShadbala).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  expect(screen.getByRole("combobox", { name: "Birim" })).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await summary();
  expect(api.computeShadbala).toHaveBeenCalledTimes(2);
});

test("retry from the keyboard keeps the focus in the panel while the error box is replaced", async () => {
  vi.mocked(api.computeShadbala).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Shadbala">
      <LivePanel />
    </section>,
    { layout: LAYOUT({}), opened: fixture().opened },
  );
  const retry = await screen.findByRole("button", { name: "Yeniden dene" });
  retry.focus();
  expect(document.activeElement).toBe(retry);
  fireEvent.click(retry);
  await summary();
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).not.toBe(document.body);
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Shadbala" }));
});

test("loading: nothing under a second, then a skeleton, then the table", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<ShadbalaReport | null>) => void;
    vi.mocked(api.computeShadbala).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderPanel();
    act(() => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    // Yüklenirken "rapor yok" açıklaması gösterilmez.
    expect(container.querySelector(".panel-notice")).toBeNull();
    act(() => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    await act(async () => resolve(ok()));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getByRole("table")).toBeInTheDocument();
  } finally {
    vi.useRealTimers();
  }
});

test("shared highlight: a graha marks its row in every table; a time lord pins the same graha", async () => {
  renderPanel({ detail: "items" });
  const table = await summary();
  const marked = () => [...document.querySelectorAll("tr.is-highlighted")].map((r) => (r as HTMLElement).dataset.graha);
  const jupiter = within(table).getByRole("button", { name: "Jüpiter" });
  fireEvent.click(jupiter);
  expect(jupiter).toHaveAttribute("aria-pressed", "true");
  expect(marked()).toEqual(["Jupiter", "Jupiter", "Jupiter"]);
  // Ay lordu Jüpiter: aynı hedef, o da basılı görünür.
  const lords = screen.getByRole("list", { name: "Zaman lordları" });
  expect(within(lords).getByRole("button", { name: "Jüpiter" })).toHaveAttribute("aria-pressed", "true");
  fireEvent.click(within(lords).getAllByRole("button", { name: "Satürn" })[0]);
  expect(marked()).toEqual(["Saturn", "Saturn", "Saturn"]);
  fireEvent.keyDown(window, { key: "Escape" });
  expect(marked()).toEqual([]);
});
````

````diff
diff --git a/app/ui/layout/model.test.ts b/app/ui/layout/model.test.ts
index 6b43845..b8a6bce 100644
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -39,6 +39,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
     "jaimini_karakas",
     "arudha",
     "argala",
+    "shadbala",
   ]);
   expect(isKnownPanelKind("chart")).toBe(true);
   expect(isKnownPanelKind("drishti")).toBe(true);
@@ -46,7 +47,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
 });
 
 test("module panels are known kinds, listed once, with a size the grid accepts", () => {
-  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala"]);
+  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala"]);
   for (const m of MODULE_PANELS) {
     expect(isKnownPanelKind(m.kind)).toBe(true);
     expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
````

````diff
diff --git a/app/ui/panels/panels.test.tsx b/app/ui/panels/panels.test.tsx
index 78d899d..a4bd113 100644
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -18,6 +18,7 @@ import { GrahaTablePanel } from "./GrahaTablePanel";
 import { JaiminiKarakasPanel } from "./JaiminiKarakasPanel";
 import { PanchangPanel } from "./PanchangPanel";
 import { panelDefinition, PANEL_REGISTRY } from "./registry";
+import { ShadbalaPanel } from "./ShadbalaPanel";
 import { UnavailablePanel } from "./UnavailablePanel";
 
 const chartPanel = (settings: Record<string, unknown> = { varga: "D1", style: "north" }): Panel => ({
@@ -46,6 +47,7 @@ test("registry covers every catalog kind", () => {
   expect(PANEL_REGISTRY.jaimini_karakas.component).toBe(JaiminiKarakasPanel);
   expect(PANEL_REGISTRY.arudha.component).toBe(ArudhaPanel);
   expect(PANEL_REGISTRY.argala.component).toBe(ArgalaPanel);
+  expect(PANEL_REGISTRY.shadbala.component).toBe(ShadbalaPanel);
   // Kataloğun her türü uygulanmıştır: yer tutucu yoktur.
   for (const kind of PANEL_KINDS) expect(PANEL_REGISTRY[kind].component, kind).not.toBe(UnavailablePanel);
   expect(panelDefinition("yoga_list")).toMatchObject({ titleKey: "panel.unknown", component: UnavailablePanel });
````

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests |Failed to resolve"
```
Expected: `Failed to resolve import "./bala"` (`ShadbalaPanel.test.tsx`) ve `Failed to resolve import "./ShadbalaPanel"` (`panels.test.tsx`); `Test Files  3 failed | 68 passed (71)`, `Tests  2 failed | 640 passed (642)` (iki dosya yüklenemez; `model.test.ts`'in iki testi düşer).

- [ ] **Step 3: Ortak kanca ve ortak parçalar**

**Yeni dosya:** `app/ui/modules/useShadbala.ts`

````ts
// Shadbala raporunu (`compute_shadbala`) isteyen ortak kanca. Shadbala ve Bhava bala panelleri
// aynı raporu okur. Rapor saklanmaz ve arayüzde hesap yoktur (K1, C4); her panel kendi isteğini
// yapar (komut 1 ms'nin altındadır). `data === null` (yükleme bittiğinde): doğum saati
// bilinmiyor ya da doğum yerinde o gün Güneş doğmuyor ya da batmıyor.
import { api } from "../api";
import type { OpenedClient, ShadbalaReport } from "../bindings";
import { chartInput, inputKey } from "./input";
import { useModule, type ModuleState } from "./useModule";

export function useShadbala(opened: OpenedClient | null): ModuleState<ShadbalaReport | null> {
  return useModule(opened !== null ? inputKey(opened) : null, () => api.computeShadbala(chartInput(opened as OpenedClient)));
}
````

**Yeni dosya:** `app/ui/panels/bala.tsx`

````tsx
// Shadbala ve Bhava bala panellerinin ortak parçaları (ileri modüller spec'i §7 "Arayüz"):
// birim ayarı, sayı biçimi ve "rapor yok" açıklaması. Değerler Rust'tan virupa olarak gelir;
// rupa gösterimi yalnızca birim çevirisidir (60 virupa = 1 rupa), astrolojik hesap değildir (C4).
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { OpenedClient } from "../bindings";
import type { Panel } from "../layout/model";
import { timeUnknown } from "../session/birth";
import { useLayoutStore } from "../view/store";

export const BALA_UNITS = ["virupa", "rupa"] as const;
export type BalaUnit = (typeof BALA_UNITS)[number];
/** Rust `VIRUPAS_PER_RUPA`. */
export const VIRUPAS_PER_RUPA = 60;
const DASH = "—";

/** Panelin birim ayarı; tanınmayan değer ve eksik anahtar virupadır. */
export function balaUnit(settings: Record<string, unknown>): BalaUnit {
  return settings.unit === "rupa" ? "rupa" : "virupa";
}

/** İşaretli sabit ondalık: eksi işareti tipografik (−); boş ya da sonlu olmayan değer çizgi. */
export function formatFixed(value: number | null, digits: number): string {
  if (value === null || !Number.isFinite(value)) return DASH;
  const text = Math.abs(value).toFixed(digits);
  // "−0.0" yazılmaz.
  return value < 0 && Number(text) !== 0 ? `−${text}` : text;
}

/** Virupa değeri seçili birimde: virupa bir ondalık, rupa iki ondalık. */
export function formatBala(virupa: number | null, unit: BalaUnit): string {
  if (virupa === null) return DASH;
  return unit === "rupa" ? formatFixed(virupa / VIRUPAS_PER_RUPA, 2) : formatFixed(virupa, 1);
}

/** Birim seçicisi; değer panelin `unit` ayarıdır ve düzenle birlikte saklanır. */
export function UnitSelect({ panel }: { panel: Panel }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  return (
    <>
      <label htmlFor={id}>{t("bala.unit")}</label>
      <select id={id} value={balaUnit(panel.settings)} onChange={(e) => setPanelSettings(panel.id, { unit: e.target.value })}>
        {BALA_UNITS.map((u) => (
          <option key={u} value={u}>
            {t(`bala.units.${u}`)}
          </option>
        ))}
      </select>
    </>
  );
}

/**
 * Rust rapor döndürmedi (`null`): doğum saati bilinmiyorsa onu, değilse gün doğumunun ya da
 * batımının olmadığını söyler. İkinci durumda harita başlığındaki uyarı rozeti de görünür.
 */
export function BalaUnavailable({ opened }: { opened: OpenedClient }) {
  const { t } = useTranslation();
  return <p className="panel-notice">{t(timeUnknown(opened) ? "bala.timeUnknown" : "bala.noSunrise")}</p>;
}
````

- [ ] **Step 4: Panel, panel türü, kayıt, stil ve metinler**

**Yeni dosya:** `app/ui/panels/ShadbalaPanel.tsx`

````tsx
// Shadbala paneli (ileri modüller spec'i §7): yedi grahanın altı gücü, toplamı, gerekli en az
// değeri, oranı ve sırası; "Kalemler" görünümünde sthana balanın ve zaman gücünün alt kalemleri;
// altta zaman lordları. Değerler `compute_shadbala`'dan gelir ve panel açıkken istenir (K1);
// burada hesap yoktur (C4). Oran 1'in altındaysa hücrede `<1` metin işareti durur; graha savaşı
// düz metinle yazılır; renk anlam taşımaz (C9).
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { Graha, GrahaShadbala, OpenedClient, ShadbalaReport, SthanaBala, TemporalBala, TimeLords } from "../bindings";
import { useHighlight } from "../highlight/store";
import type { Panel } from "../layout/model";
import { useShadbala } from "../modules/useShadbala";
import { useSession } from "../session/store";
import { useLayoutStore } from "../view/store";
import { BalaUnavailable, balaUnit, formatBala, formatFixed, UnitSelect, type BalaUnit } from "./bala";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

export const SHADBALA_DETAILS = ["summary", "items"] as const;
export type ShadbalaDetail = (typeof SHADBALA_DETAILS)[number];
/** Oranı 1'in altında olan grahanın metin işareti. */
export const BELOW_MARK = "<1";

/** Panelin ayrıntı ayarı; tanınmayan değer ve eksik anahtar özettir. */
export function shadbalaDetail(settings: Record<string, unknown>): ShadbalaDetail {
  return settings.detail === "items" ? "items" : "summary";
}

const STHANA_ITEMS = ["uccha", "saptavargaja", "ojayugma", "kendradi", "drekkana"] as const satisfies readonly (keyof SthanaBala)[];
const TEMPORAL_ITEMS = [
  ["day_night", "dayNight"],
  ["paksha", "paksha"],
  ["tribhaga", "tribhaga"],
  ["year_lord", "yearLord"],
  ["month_lord", "monthLord"],
  ["weekday_lord", "weekdayLord"],
  ["hora_lord", "horaLord"],
  ["ayana", "ayana"],
] as const satisfies readonly (readonly [keyof TemporalBala, string])[];
const TIME_LORDS = ["year", "month", "weekday", "hora", "tribhaga"] as const satisfies readonly (keyof TimeLords)[];

export function ShadbalaPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <Shadbala panel={panel} opened={opened} />;
}

function Shadbala({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  const module = useShadbala(opened);
  const phase = usePanelLoadPhase(module.loading);
  const detail = shadbalaDetail(panel.settings);
  const unit = balaUnit(panel.settings);
  return (
    <div className="module-panel">
      {/* Araç çubuğu yükleme, hata ve "rapor yok" durumlarında da kullanılabilir. */}
      <div className="module-toolbar">
        <label htmlFor={`${id}-detail`}>{t("shadbala.detail")}</label>
        <select id={`${id}-detail`} value={detail} onChange={(e) => setPanelSettings(panel.id, { detail: e.target.value })}>
          {SHADBALA_DETAILS.map((d) => (
            <option key={d} value={d}>
              {t(`shadbala.details.${d}`)}
            </option>
          ))}
        </select>
        <UnitSelect panel={panel} />
      </div>
      {module.error ? (
        <PanelError error={module.error} onRetry={module.retry} />
      ) : module.loading ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : module.data === null ? (
        <BalaUnavailable opened={opened} />
      ) : (
        <Report report={module.data} detail={detail} unit={unit} />
      )}
    </div>
  );
}

function Report({ report, detail, unit }: { report: ShadbalaReport; detail: ShadbalaDetail; unit: BalaUnit }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const on = (g: Graha) => active?.type === "graha" && active.id === g;
  const num = (v: number | null) => <td className="num">{formatBala(v, unit)}</td>;
  const name = (row: GrahaShadbala) => (
    <th scope="row">
      <PinButton target={{ type: "graha", id: row.graha }}>{t(`grahaName.${row.graha}`)}</PinButton>
      {row.war && (
        <span className="bala-war">
          {" "}
          <span aria-hidden="true">{t("shadbala.war", { with: t(`graha.${row.war.with}`) })}</span>
          <span className="visually-hidden">{t("shadbala.warFull", { with: t(`grahaName.${row.war.with}`), winner: t(`grahaName.${row.war.winner}`) })}</span>
        </span>
      )}
    </th>
  );
  // Karşılaştırma gösterim eşiğidir: oran Rust'tan gelir.
  const below = (row: GrahaShadbala) => row.ratio !== null && row.ratio < 1;
  const anyBelow = report.grahas.some(below);

  return (
    <>
      <div className="bala-scroll">
        <table className="module-table">
          <caption className="visually-hidden">{t("shadbala.caption")}</caption>
          <thead>
            <tr>
              <th scope="col">{t("shadbala.graha")}</th>
              {(["sthana", "dig", "temporal", "cheshta", "naisargika", "drik"] as const).map((k) => (
                <th key={k} scope="col" className="num">
                  {t(`shadbala.${k}`)}
                </th>
              ))}
              <th scope="col" className="num">
                {t("bala.total")}
              </th>
              <th scope="col" className="num">
                {t("shadbala.required")}
              </th>
              <th scope="col" className="num">
                {t("shadbala.ratio")}
              </th>
              <th scope="col" className="num">
                {t("bala.rank")}
              </th>
            </tr>
          </thead>
          <tbody>
            {report.grahas.map((row) => (
              <tr key={row.graha} data-graha={row.graha} className={on(row.graha) ? "is-highlighted" : undefined}>
                {name(row)}
                {num(row.sthana.total)}
                {num(row.dig)}
                {num(row.temporal.total)}
                {num(row.cheshta)}
                {num(row.naisargika)}
                {num(row.drik)}
                {/* Rupa toplamı Rust'ın `rupas` alanıdır. */}
                <td className="num">{unit === "rupa" ? formatFixed(row.rupas, 2) : formatBala(row.total, unit)}</td>
                {num(row.required)}
                <td className="num">
                  {formatFixed(row.ratio, 2)}
                  {below(row) && (
                    <>
                      {" "}
                      <span aria-hidden="true">{BELOW_MARK}</span>
                      <span className="visually-hidden">{t("shadbala.below")}</span>
                    </>
                  )}
                </td>
                <td className="num">{row.rank}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {detail === "items" && (
        <>
          <div className="bala-scroll">
            <table className="module-table">
              <caption>{t("shadbala.sthanaCaption")}</caption>
              <thead>
                <tr>
                  <th scope="col">{t("shadbala.graha")}</th>
                  {STHANA_ITEMS.map((k) => (
                    <th key={k} scope="col" className="num">
                      {t(`shadbala.${k}`)}
                    </th>
                  ))}
                  <th scope="col" className="num">
                    {t("bala.total")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {report.grahas.map((row) => (
                  <tr key={row.graha} data-graha={row.graha} className={on(row.graha) ? "is-highlighted" : undefined}>
                    <th scope="row">{t(`grahaName.${row.graha}`)}</th>
                    {STHANA_ITEMS.map((k) => (
                      <td key={k} className="num">
                        {formatBala(row.sthana[k], unit)}
                      </td>
                    ))}
                    {num(row.sthana.total)}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <div className="bala-scroll">
            <table className="module-table">
              <caption>{t("shadbala.temporalCaption")}</caption>
              <thead>
                <tr>
                  <th scope="col">{t("shadbala.graha")}</th>
                  {TEMPORAL_ITEMS.map(([k, label]) => (
                    <th key={k} scope="col" className="num">
                      {t(`shadbala.${label}`)}
                    </th>
                  ))}
                  <th scope="col" className="num">
                    {t("bala.total")}
                  </th>
                </tr>
              </thead>
              <tbody>
                {report.grahas.map((row) => (
                  <tr key={row.graha} data-graha={row.graha} className={on(row.graha) ? "is-highlighted" : undefined}>
                    <th scope="row">{t(`grahaName.${row.graha}`)}</th>
                    {TEMPORAL_ITEMS.map(([k]) => (
                      <td key={k} className="num">
                        {formatBala(row.temporal[k], unit)}
                      </td>
                    ))}
                    {num(row.temporal.total)}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </>
      )}
      <ul className="bala-lords" aria-label={t("shadbala.lords")}>
        {TIME_LORDS.map((k) => (
          <li key={k} data-lord={k}>
            {t(`shadbala.lord.${k}`)}: <PinButton target={{ type: "graha", id: report.time_lords[k] }}>{t(`grahaName.${report.time_lords[k]}`)}</PinButton>
          </li>
        ))}
      </ul>
      {anyBelow && <p className="panel-message">{t("shadbala.belowLegend", { mark: BELOW_MARK })}</p>}
      <p className="panel-message">{t(`bala.legend.${unit}`)}</p>
      <p className="panel-message">{t("bala.method", { method: report.method })}</p>
    </>
  );
}
````

````diff
diff --git a/app/ui/layout/model.ts b/app/ui/layout/model.ts
index 2a303aa..e7bda13 100644
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -1,7 +1,7 @@
 // Pano düzeni veri modeli (spec §3.2). Kalıcı biçim `ViewState.panel_layout` ile aynıdır;
 // M4'teki store komutu gelene kadar localStorage'da tutulur (bkz. persistence.ts).
 
-/** Panel kataloğu: v1 (UI spec §3.3) ve ileri modül panelleri (ileri modüller spec'i §4, §5). */
+/** Panel kataloğu: v1 (UI spec §3.3) ve ileri modül panelleri (ileri modüller spec'i §4, §5, §7). */
 export const PANEL_KINDS = [
   "chart",
   "graha_table",
@@ -16,6 +16,7 @@ export const PANEL_KINDS = [
   "jaimini_karakas",
   "arudha",
   "argala",
+  "shadbala",
 ] as const;
 export type KnownPanelKind = (typeof PANEL_KINDS)[number];
 
@@ -31,6 +32,8 @@ export const MODULE_PANELS = [
   { kind: "jaimini_karakas", w: 5, h: 14 },
   { kind: "arudha", w: 4, h: 12 },
   { kind: "argala", w: 6, h: 9 },
+  // Shadbala ve Bhava bala (Plan 17).
+  { kind: "shadbala", w: 8, h: 12 },
 ] as const satisfies readonly { kind: KnownPanelKind; w: number; h: number }[];
 export type ModulePanelKind = (typeof MODULE_PANELS)[number]["kind"];
 
````

````diff
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index 0f20aca..1dbd343 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -13,6 +13,7 @@ import { DrishtiPanel } from "./DrishtiPanel";
 import { GrahaTablePanel } from "./GrahaTablePanel";
 import { JaiminiKarakasPanel } from "./JaiminiKarakasPanel";
 import { PanchangPanel } from "./PanchangPanel";
+import { ShadbalaPanel } from "./ShadbalaPanel";
 import type { PanelDefinition } from "./types";
 import { UnavailablePanel } from "./UnavailablePanel";
 
@@ -32,6 +33,8 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   jaimini_karakas: { titleKey: "panel.jaimini_karakas", component: JaiminiKarakasPanel },
   arudha: { titleKey: "panel.arudha", component: ArudhaPanel },
   argala: { titleKey: "panel.argala", component: ArgalaPanel },
+  // Shadbala ve Bhava bala (Plan 17).
+  shadbala: { titleKey: "panel.shadbala", component: ShadbalaPanel },
 };
 
 const UNKNOWN: PanelDefinition = { titleKey: "panel.unknown", component: UnavailablePanel };
````

````diff
diff --git a/app/ui/panels/panels.css b/app/ui/panels/panels.css
index d9de0a7..6e495d5 100644
--- a/app/ui/panels/panels.css
+++ b/app/ui/panels/panels.css
@@ -218,3 +218,23 @@
   margin: 4px 0 0;
   font-weight: var(--fw-medium);
 }
+.bala-scroll {
+  overflow-x: auto;
+}
+.bala-scroll caption {
+  text-align: left;
+  padding: 6px 6px 2px;
+  color: var(--text-2);
+}
+.bala-war {
+  color: var(--text-2);
+  font-weight: var(--fw-regular);
+}
+.bala-lords {
+  display: flex;
+  flex-wrap: wrap;
+  gap: 2px 14px;
+  margin: 4px 0 0;
+  padding: 0;
+  list-style: none;
+}
````

````diff
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index e8691c9..23556a9 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -65,6 +65,7 @@ export const tr = {
     jaimini_karakas: "Chara karaka",
     arudha: "Arudha padalar",
     argala: "Argala",
+    shadbala: "Shadbala",
     unknown: "Bilinmeyen panel",
     loading: "Yükleniyor",
     unavailable: "Bu panel sonraki bir sürümde gelecek.",
@@ -158,6 +159,51 @@ export const tr = {
     thirdHouse: "3. evde üç ya da daha çok doğal kötücül var: 3. ev kötücül argalası.",
     note: "Durum yalnızca graha sayısına bakar; güç karşılaştırması yapılmaz.",
   },
+  bala: {
+    unit: "Birim",
+    units: { virupa: "Virupa", rupa: "Rupa" },
+    total: "Toplam",
+    rank: "Sıra",
+    legend: { virupa: "Değerler virupa cinsindendir (60 virupa = 1 rupa).", rupa: "Değerler rupa cinsindendir (1 rupa = 60 virupa)." },
+    method: "Yöntem: {{method}}",
+    timeUnknown: "Doğum saati bilinmiyor: Shadbala ve Bhava bala doğum saatine bağlı olduğu için hesaplanmıyor.",
+    noSunrise: "Doğum yerinde o gün gün doğumu ya da batımı yok: zaman gücü ölçülemediği için Shadbala ve Bhava bala hesaplanmıyor.",
+  },
+  shadbala: {
+    detail: "Ayrıntı",
+    details: { summary: "Özet", items: "Kalemler" },
+    caption: "Shadbala: yedi grahanın altı gücü, toplamı, gerekli en az değeri, oranı ve sırası",
+    graha: "Graha",
+    sthana: "Sthana",
+    dig: "Dig",
+    temporal: "Zaman gücü",
+    cheshta: "Cheshta",
+    naisargika: "Naisargika",
+    drik: "Drik",
+    required: "Gerekli",
+    ratio: "Oran",
+    below: "gerekli değerin altında",
+    belowLegend: "{{mark}} toplam, klasik gerekli en az değerin altında.",
+    war: "savaş: {{with}}",
+    warFull: "{{with}} ile graha savaşında; kazanan {{winner}}",
+    sthanaCaption: "Sthana bala kalemleri",
+    uccha: "Uccha",
+    saptavargaja: "Saptavargaja",
+    ojayugma: "Ojayugma",
+    kendradi: "Kendradi",
+    drekkana: "Drekkana",
+    temporalCaption: "Zaman gücü kalemleri",
+    dayNight: "Gündüz–gece",
+    paksha: "Paksha",
+    tribhaga: "Tribhaga",
+    yearLord: "Yıl lordu",
+    monthLord: "Ay lordu",
+    weekdayLord: "Gün lordu",
+    horaLord: "Saat lordu",
+    ayana: "Ayana",
+    lords: "Zaman lordları",
+    lord: { year: "Yıl lordu", month: "Ay lordu", weekday: "Gün lordu", hora: "Saat lordu", tribhaga: "Tribhaga lordu" },
+  },
   arudha: {
     caption: "Arudha padalar: on iki evin padası, burcu ve evin lordu",
     pada: "Pada",
````

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index 54a8533..07d2d84 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -66,6 +66,7 @@ export const en: Messages = {
     jaimini_karakas: "Chara karaka",
     arudha: "Arudha padas",
     argala: "Argala",
+    shadbala: "Shadbala",
     unknown: "Unknown panel",
     loading: "Loading",
     unavailable: "This panel arrives in a later version.",
@@ -159,6 +160,51 @@ export const en: Messages = {
     thirdHouse: "Three or more natural malefics in the 3rd house: third-house malefic argala.",
     note: "The status looks at the number of grahas only; strength is not compared.",
   },
+  bala: {
+    unit: "Unit",
+    units: { virupa: "Virupa", rupa: "Rupa" },
+    total: "Total",
+    rank: "Rank",
+    legend: { virupa: "Values are in virupas (60 virupas = 1 rupa).", rupa: "Values are in rupas (1 rupa = 60 virupas)." },
+    method: "Method: {{method}}",
+    timeUnknown: "Birth time unknown: Shadbala and Bhava bala depend on the birth time and are not computed.",
+    noSunrise: "The Sun does not rise or set at the birth place on that day: the temporal strength cannot be measured, so Shadbala and Bhava bala are not computed.",
+  },
+  shadbala: {
+    detail: "Detail",
+    details: { summary: "Summary", items: "Items" },
+    caption: "Shadbala: the six strengths of the seven grahas, total, required minimum, ratio and rank",
+    graha: "Graha",
+    sthana: "Sthana",
+    dig: "Dig",
+    temporal: "Temporal",
+    cheshta: "Cheshta",
+    naisargika: "Naisargika",
+    drik: "Drik",
+    required: "Required",
+    ratio: "Ratio",
+    below: "below the required value",
+    belowLegend: "{{mark}} the total is below the classical required minimum.",
+    war: "war: {{with}}",
+    warFull: "in a graha war with {{with}}; winner {{winner}}",
+    sthanaCaption: "Items of sthana bala",
+    uccha: "Uccha",
+    saptavargaja: "Saptavargaja",
+    ojayugma: "Ojayugma",
+    kendradi: "Kendradi",
+    drekkana: "Drekkana",
+    temporalCaption: "Items of the temporal strength",
+    dayNight: "Day–night",
+    paksha: "Paksha",
+    tribhaga: "Tribhaga",
+    yearLord: "Year lord",
+    monthLord: "Month lord",
+    weekdayLord: "Weekday lord",
+    horaLord: "Hora lord",
+    ayana: "Ayana",
+    lords: "Time lords",
+    lord: { year: "Year lord", month: "Month lord", weekday: "Weekday lord", hora: "Hora lord", tribhaga: "Tribhaga lord" },
+  },
   arudha: {
     caption: "Arudha padas: the pada of each of the twelve houses, its sign and the lord of the house",
     pada: "Pada",
````

````diff
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 5fc1cc2..3d38609 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -66,6 +66,7 @@ export const fr: Messages = {
     jaimini_karakas: "Chara karaka",
     arudha: "Arudha padas",
     argala: "Argala",
+    shadbala: "Shadbala",
     unknown: "Panneau inconnu",
     loading: "Chargement",
     unavailable: "Ce panneau arrivera dans une version ultérieure.",
@@ -159,6 +160,51 @@ export const fr: Messages = {
     thirdHouse: "Trois maléfiques naturels ou plus en maison 3 : argala maléfique de la 3e maison.",
     note: "L'état ne tient compte que du nombre de grahas ; la force n'est pas comparée.",
   },
+  bala: {
+    unit: "Unité",
+    units: { virupa: "Virupa", rupa: "Rupa" },
+    total: "Total",
+    rank: "Rang",
+    legend: { virupa: "Valeurs en virupas (60 virupas = 1 rupa).", rupa: "Valeurs en rupas (1 rupa = 60 virupas)." },
+    method: "Méthode : {{method}}",
+    timeUnknown: "Heure de naissance inconnue : le Shadbala et le Bhava bala dépendent de l'heure de naissance et ne sont pas calculés.",
+    noSunrise: "Le Soleil ne se lève pas ou ne se couche pas au lieu de naissance ce jour-là : la force temporelle ne peut pas être mesurée ; le Shadbala et le Bhava bala ne sont pas calculés.",
+  },
+  shadbala: {
+    detail: "Détail",
+    details: { summary: "Résumé", items: "Éléments" },
+    caption: "Shadbala : les six forces des sept grahas, total, minimum requis, rapport et rang",
+    graha: "Graha",
+    sthana: "Sthana",
+    dig: "Dig",
+    temporal: "Temporelle",
+    cheshta: "Cheshta",
+    naisargika: "Naisargika",
+    drik: "Drik",
+    required: "Requis",
+    ratio: "Rapport",
+    below: "sous la valeur requise",
+    belowLegend: "{{mark}} le total est sous le minimum requis classique.",
+    war: "guerre : {{with}}",
+    warFull: "en guerre de grahas avec {{with}} ; vainqueur {{winner}}",
+    sthanaCaption: "Éléments du sthana bala",
+    uccha: "Uccha",
+    saptavargaja: "Saptavargaja",
+    ojayugma: "Ojayugma",
+    kendradi: "Kendradi",
+    drekkana: "Drekkana",
+    temporalCaption: "Éléments de la force temporelle",
+    dayNight: "Jour–nuit",
+    paksha: "Paksha",
+    tribhaga: "Tribhaga",
+    yearLord: "Maître de l'année",
+    monthLord: "Maître du mois",
+    weekdayLord: "Maître du jour",
+    horaLord: "Maître de l'heure",
+    ayana: "Ayana",
+    lords: "Maîtres du temps",
+    lord: { year: "Maître de l'année", month: "Maître du mois", weekday: "Maître du jour", hora: "Maître de l'heure", tribhaga: "Maître du tribhaga" },
+  },
   arudha: {
     caption: "Arudha padas : le pada de chacune des douze maisons, son signe et le maître de la maison",
     pada: "Pada",
````

- [ ] **Step 5: Testleri koş**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
```
Expected: `Test Files  71 passed (71)`, `Tests  665 passed (665)` (653 + 12). `i18n.test.ts` üç dilin anahtar kümelerini ve yer tutucularını değişmeden doğrular.

- [ ] **Step 6: Kapılar**

```bash
pnpm --dir app build 2>&1 | grep "✓ built"
git diff -U0 -- 'app/ui/**/*.css' | grep '^+' | grep -E '#[0-9a-fA-F]{3,8}\b|rgb\('
cargo test -p jyotish-desk --test bindings --test ui_fixture 2>&1 | grep -E "^test result"
```
Expected: `✓ built` (`tsc` hatasız); `grep` çıktısız (C9); `bindings` `1 passed`, `ui_fixture` `2 passed` (üretilen dosyalar değişmedi).

- [ ] **Step 7: Commit**

```bash
git add app/ui
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): shadbala panel with summary and items views, virupa or rupa unit and the time lords; shared report hook"
```
Expected: iki `grep` çıktısız.

---

### Task 6: Arayüz: Bhava bala paneli

Spec §7 "Arayüz" (Bhava bala satırı), K12; §13 "Arayüz"; D14, D19, D20. Panel Task 5'in kancasını ve ortak parçalarını kullanır.

**Files:**
- Create: `app/ui/panels/BhavaBalaPanel.tsx`, `app/ui/panels/BhavaBalaPanel.test.tsx`
- Modify: `app/ui/layout/model.ts`, `app/ui/layout/model.test.ts`, `app/ui/panels/registry.ts`, `app/ui/panels/panels.test.tsx`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`
- Test: `BhavaBalaPanel.test.tsx` (9 test), `model.test.ts`, `panels.test.tsx`

**Interfaces:**
- Consumes: `useShadbala`, `panels/bala.tsx` (Task 5); `chart/adapter.ts` (`signIndex`); `PinButton`; `highlight/store` (`house`, `sign`, `graha` hedefleri).
- Produces: `BhavaBalaPanel`; panel türü `bhava_bala` (`MODULE_PANELS` `{ w: 6, h: 14 }`); panel ayarı `unit`; i18n `panel.bhava_bala`, `bhavaBala.*`.

- [ ] **Step 1: Başarısız testleri yaz**

**Yeni dosya:** `app/ui/panels/BhavaBalaPanel.test.tsx`

````tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeShadbala: vi.fn() } }));

import { api, type Result } from "../api";
import type { ShadbalaReport } from "../bindings";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useLayoutStore } from "../view/store";
import { BhavaBalaPanel } from "./BhavaBalaPanel";

const ok = (report: ShadbalaReport | null = fixture().shadbala): Result<ShadbalaReport | null> => ({ status: "ok", data: report });
const LAYOUT = (settings: Record<string, unknown>) => {
  const panel: Panel = { id: "bb", kind: "bhava_bala", x: 0, y: 0, w: 6, h: 14, settings };
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
};

/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  return <BhavaBalaPanel panel={active.panels[0]} />;
}

function renderPanel(settings: Record<string, unknown> = {}, opened: ReturnType<typeof fixture>["opened"] | null = fixture().opened) {
  return renderWithStores(<LivePanel />, { layout: LAYOUT(settings), opened });
}

const table = () => screen.findByRole("table", { name: "Bhava bala: on iki bhavanın lordu, üç kalemi, toplamı ve sırası" });
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);

beforeEach(() => {
  vi.mocked(api.computeShadbala).mockReset();
  vi.mocked(api.computeShadbala).mockImplementation(async () => ok());
});

test("lists the twelve bhavas with lord, three items, total and rank from the shadbala report", async () => {
  renderPanel();
  const rows = within(await table()).getAllByRole("row");
  expect(api.computeShadbala).toHaveBeenCalledTimes(1);
  expect(vi.mocked(api.computeShadbala).mock.calls[0][0].time_known).toBe(true);
  expect(cells(rows[0])).toEqual(["Bhava", "Lord", "Bhavadhipati", "Dig", "Drishti", "Toplam", "Sıra"]);
  // Delhi fikstürü, İkizler lagna 16°: 1. bhava insan burcu (dig 60), 10. bhava Balık (dig 0).
  expect(rows.slice(1).map(cells)).toEqual([
    ["1", "Merkür", "454.1", "60.0", "58.5", "572.6", "2"],
    ["2", "Ay", "413.5", "40.0", "47.4", "500.9", "6"],
    ["3", "Güneş", "505.3", "10.0", "49.5", "564.8", "3"],
    ["4", "Merkür", "454.1", "30.0", "28.2", "512.3", "5"],
    ["5", "Venüs", "371.0", "20.0", "44.8", "435.8", "9"],
    ["6", "Mars", "412.6", "50.0", "33.4", "496.0", "7"],
    ["7", "Jüpiter", "533.1", "30.0", "55.9", "619.0", "1"],
    ["8", "Satürn", "353.7", "20.0", "4.2", "377.9", "12"],
    ["9", "Satürn", "353.7", "20.0", "4.8", "378.4", "11"],
    ["10", "Jüpiter", "533.1", "0.0", "22.5", "555.6", "4"],
    ["11", "Mars", "412.6", "50.0", "−7.2", "455.4", "8"],
    ["12", "Venüs", "371.0", "40.0", "10.6", "421.6", "10"],
  ]);
  // Bhava numarası tam adıyla okunur.
  expect(within(rows[1]).getByRole("button", { name: "1. bhava" })).toHaveTextContent("1");
  expect(screen.getByText(/^Bhavalar eşittir: bhava ortası lagnadan 30°'lik adımlarla sayılır/)).toBeInTheDocument();
  expect(screen.getByText("Değerler virupa cinsindendir (60 virupa = 1 rupa).")).toBeInTheDocument();
  expect(screen.getByText("Yöntem: shadbala.bphs_v1")).toBeInTheDocument();
});

test("the unit is saved with the layout and redraws the values in rupas", async () => {
  const { persistence } = renderPanel();
  const t = await table();
  fireEvent.change(screen.getByRole("combobox", { name: "Birim" }), { target: { value: "rupa" } });
  await waitFor(() => expect(cells(within(t).getAllByRole("row")[1])).toEqual(["1", "Merkür", "7.57", "1.00", "0.98", "9.54", "2"]));
  expect(cells(within(t).getAllByRole("row")[2])).toEqual(["2", "Ay", "6.89", "0.67", "0.79", "8.35", "6"]);
  expect(screen.getByText("Değerler rupa cinsindendir (1 rupa = 60 virupa).")).toBeInTheDocument();
  await waitFor(() => expect(persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ unit: "rupa" }));
  expect(api.computeShadbala).toHaveBeenCalledTimes(1);
});

test("unknown birth time: Rust returns no report and the panel says why; the toolbar stays", async () => {
  vi.mocked(api.computeShadbala).mockResolvedValue(ok(null));
  renderPanel({}, unknownTimeOpened());
  expect(await screen.findByText("Doğum saati bilinmiyor: Shadbala ve Bhava bala doğum saatine bağlı olduğu için hesaplanmıyor.")).toBeInTheDocument();
  expect(vi.mocked(api.computeShadbala).mock.calls[0][0].time_known).toBe(false);
  expect(screen.queryByRole("table")).toBeNull();
  expect(screen.getByRole("combobox", { name: "Birim" })).toBeInTheDocument();
});

test("no sunrise or sunset: with a known birth time the missing report is explained by the Sun", async () => {
  vi.mocked(api.computeShadbala).mockResolvedValue(ok(null));
  renderPanel();
  expect(
    await screen.findByText("Doğum yerinde o gün gün doğumu ya da batımı yok: zaman gücü ölçülemediği için Shadbala ve Bhava bala hesaplanmıyor."),
  ).toBeInTheDocument();
  expect(screen.queryByRole("table")).toBeNull();
});

test("without an open chart nothing is requested", () => {
  renderPanel({}, null);
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.computeShadbala).not.toHaveBeenCalled();
});

test("an error is shown in the panel and retry asks again", async () => {
  vi.mocked(api.computeShadbala).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await table();
  expect(api.computeShadbala).toHaveBeenCalledTimes(2);
});

test("retry from the keyboard keeps the focus in the panel while the error box is replaced", async () => {
  vi.mocked(api.computeShadbala).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Bhava bala">
      <LivePanel />
    </section>,
    { layout: LAYOUT({}), opened: fixture().opened },
  );
  const retry = await screen.findByRole("button", { name: "Yeniden dene" });
  retry.focus();
  fireEvent.click(retry);
  await table();
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Bhava bala" }));
});

test("loading: nothing under a second, then a skeleton, then the table", async () => {
  vi.useFakeTimers();
  try {
    let resolve!: (r: Result<ShadbalaReport | null>) => void;
    vi.mocked(api.computeShadbala).mockReturnValueOnce(new Promise((r) => (resolve = r)));
    const { container } = renderPanel();
    act(() => vi.advanceTimersByTime(999));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(container.querySelector(".panel-notice")).toBeNull();
    act(() => vi.advanceTimersByTime(1));
    expect(container.querySelector(".panel-skeleton")).toHaveAttribute("aria-busy", "true");
    await act(async () => resolve(ok()));
    expect(container.querySelector(".panel-skeleton")).toBeNull();
    expect(screen.getByRole("table")).toBeInTheDocument();
  } finally {
    vi.useRealTimers();
  }
});

test("shared highlight: a bhava marks its row, a graha every bhava it rules", async () => {
  renderPanel();
  const t = await table();
  const marked = () => [...document.querySelectorAll("tr.is-highlighted")].map((r) => (r as HTMLElement).dataset.house);
  const seventh = within(t).getByRole("button", { name: "7. bhava" });
  fireEvent.click(seventh);
  expect(seventh).toHaveAttribute("aria-pressed", "true");
  expect(marked()).toEqual(["7"]);
  // Satürn iki bhavanın lordudur: 8 ve 9.
  fireEvent.click(within(t).getAllByRole("button", { name: "Satürn" })[0]);
  expect(marked()).toEqual(["8", "9"]);
  fireEvent.keyDown(window, { key: "Escape" });
  expect(marked()).toEqual([]);
});
````

````diff
diff --git a/app/ui/layout/model.test.ts b/app/ui/layout/model.test.ts
index b8a6bce..888282a 100644
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -40,6 +40,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
     "arudha",
     "argala",
     "shadbala",
+    "bhava_bala",
   ]);
   expect(isKnownPanelKind("chart")).toBe(true);
   expect(isKnownPanelKind("drishti")).toBe(true);
@@ -47,7 +48,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
 });
 
 test("module panels are known kinds, listed once, with a size the grid accepts", () => {
-  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala"]);
+  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala"]);
   for (const m of MODULE_PANELS) {
     expect(isKnownPanelKind(m.kind)).toBe(true);
     expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
````

````diff
diff --git a/app/ui/panels/panels.test.tsx b/app/ui/panels/panels.test.tsx
index a4bd113..6a2a807 100644
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -11,6 +11,7 @@ import { TransitTablePanel } from "../transit/TransitTablePanel";
 import { ArgalaPanel } from "./ArgalaPanel";
 import { ArudhaPanel } from "./ArudhaPanel";
 import { AshtakavargaPanel } from "./AshtakavargaPanel";
+import { BhavaBalaPanel } from "./BhavaBalaPanel";
 import { BirthInfoPanel } from "./BirthInfoPanel";
 import { ChartPanel } from "./ChartPanel";
 import { DrishtiPanel } from "./DrishtiPanel";
@@ -48,6 +49,7 @@ test("registry covers every catalog kind", () => {
   expect(PANEL_REGISTRY.arudha.component).toBe(ArudhaPanel);
   expect(PANEL_REGISTRY.argala.component).toBe(ArgalaPanel);
   expect(PANEL_REGISTRY.shadbala.component).toBe(ShadbalaPanel);
+  expect(PANEL_REGISTRY.bhava_bala.component).toBe(BhavaBalaPanel);
   // Kataloğun her türü uygulanmıştır: yer tutucu yoktur.
   for (const kind of PANEL_KINDS) expect(PANEL_REGISTRY[kind].component, kind).not.toBe(UnavailablePanel);
   expect(panelDefinition("yoga_list")).toMatchObject({ titleKey: "panel.unknown", component: UnavailablePanel });
````

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests |Failed to resolve"
```
Expected: iki `Failed to resolve import "./BhavaBalaPanel"`; `Test Files  3 failed | 69 passed (72)`, `Tests  2 failed | 652 passed (654)`.

- [ ] **Step 3: Panel, tür, kayıt ve metinler**

**Yeni dosya:** `app/ui/panels/BhavaBalaPanel.tsx`

````tsx
// Bhava bala paneli (ileri modüller spec'i §7): on iki bhavanın lordu, üç kalemi (bhavadhipati,
// dig, drishti), toplamı ve sırası. Değerler Shadbala paneliyle aynı rapordan
// (`compute_shadbala`) gelir; burada hesap yoktur (C4). Bhavalar eşittir ve lagna burcundan
// sayılan rasi evleriyle aynı burçtadır; ev sistemi ayarından bağımsızdır.
import { useTranslation } from "react-i18next";
import type { OpenedClient, ShadbalaReport } from "../bindings";
import { signIndex } from "../chart/adapter";
import { useHighlight } from "../highlight/store";
import type { Panel } from "../layout/model";
import { useShadbala } from "../modules/useShadbala";
import { useSession } from "../session/store";
import { BalaUnavailable, balaUnit, formatBala, UnitSelect, type BalaUnit } from "./bala";
import "./panels.css";
import { PinButton } from "./PinButton";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

export function BhavaBalaPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <BhavaBala panel={panel} opened={opened} />;
}

function BhavaBala({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const module = useShadbala(opened);
  const phase = usePanelLoadPhase(module.loading);
  return (
    <div className="module-panel">
      {/* Araç çubuğu yükleme, hata ve "rapor yok" durumlarında da kullanılabilir. */}
      <div className="module-toolbar">
        <UnitSelect panel={panel} />
      </div>
      {module.error ? (
        <PanelError error={module.error} onRetry={module.retry} />
      ) : module.loading ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : module.data === null ? (
        <BalaUnavailable opened={opened} />
      ) : (
        <Bhavas report={module.data} unit={balaUnit(panel.settings)} opened={opened} />
      )}
    </div>
  );
}

function Bhavas({ report, unit, opened }: { report: ShadbalaReport; unit: BalaUnit; opened: OpenedClient }) {
  const { t } = useTranslation();
  const { active } = useHighlight();
  const lagna = signIndex(opened.chart.lagna.sign);
  // Burç vurgusu D1 evidir: lagna burcundan sayılan burç (gösterim eşlemesi, hesap değil).
  const activeHouse = active?.type === "house" ? active.house : active?.type === "sign" ? ((active.sign - lagna + 12) % 12) + 1 : null;
  return (
    <>
      <div className="bala-scroll">
        <table className="module-table">
          <caption className="visually-hidden">{t("bhavaBala.caption")}</caption>
          <thead>
            <tr>
              <th scope="col">{t("bhavaBala.bhava")}</th>
              <th scope="col">{t("bhavaBala.lord")}</th>
              {(["lordBala", "dig", "drishti"] as const).map((k) => (
                <th key={k} scope="col" className="num">
                  {t(`bhavaBala.${k}`)}
                </th>
              ))}
              <th scope="col" className="num">
                {t("bala.total")}
              </th>
              <th scope="col" className="num">
                {t("bala.rank")}
              </th>
            </tr>
          </thead>
          <tbody>
            {report.bhavas.map((b) => {
              const on = activeHouse === b.house || (active?.type === "graha" && active.id === b.lord);
              return (
                <tr key={b.house} data-house={b.house} className={on ? "is-highlighted" : undefined}>
                  <th scope="row">
                    <PinButton target={{ type: "house", house: b.house }} label={t("bhavaBala.houseLabel", { house: b.house })}>
                      {b.house}
                    </PinButton>
                  </th>
                  <td>
                    <PinButton target={{ type: "graha", id: b.lord }}>{t(`grahaName.${b.lord}`)}</PinButton>
                  </td>
                  <td className="num">{formatBala(b.lord_bala, unit)}</td>
                  <td className="num">{formatBala(b.dig, unit)}</td>
                  <td className="num">{formatBala(b.drishti, unit)}</td>
                  <td className="num">{formatBala(b.total, unit)}</td>
                  <td className="num">{b.rank}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      <p className="panel-message">{t("bhavaBala.note")}</p>
      <p className="panel-message">{t(`bala.legend.${unit}`)}</p>
      <p className="panel-message">{t("bala.method", { method: report.method })}</p>
    </>
  );
}
````

````diff
diff --git a/app/ui/layout/model.ts b/app/ui/layout/model.ts
index e7bda13..96e1b99 100644
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -17,6 +17,7 @@ export const PANEL_KINDS = [
   "arudha",
   "argala",
   "shadbala",
+  "bhava_bala",
 ] as const;
 export type KnownPanelKind = (typeof PANEL_KINDS)[number];
 
@@ -34,6 +35,7 @@ export const MODULE_PANELS = [
   { kind: "argala", w: 6, h: 9 },
   // Shadbala ve Bhava bala (Plan 17).
   { kind: "shadbala", w: 8, h: 12 },
+  { kind: "bhava_bala", w: 6, h: 14 },
 ] as const satisfies readonly { kind: KnownPanelKind; w: number; h: number }[];
 export type ModulePanelKind = (typeof MODULE_PANELS)[number]["kind"];
 
````

````diff
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index 1dbd343..006dcd9 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -6,6 +6,7 @@ import { NotesPanel } from "../notes/NotesPanel";
 import { ArgalaPanel } from "./ArgalaPanel";
 import { ArudhaPanel } from "./ArudhaPanel";
 import { AshtakavargaPanel } from "./AshtakavargaPanel";
+import { BhavaBalaPanel } from "./BhavaBalaPanel";
 import { BirthInfoPanel } from "./BirthInfoPanel";
 import { ChartPanel, chartVarga } from "./ChartPanel";
 import { TransitTablePanel } from "../transit/TransitTablePanel";
@@ -35,6 +36,7 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   argala: { titleKey: "panel.argala", component: ArgalaPanel },
   // Shadbala ve Bhava bala (Plan 17).
   shadbala: { titleKey: "panel.shadbala", component: ShadbalaPanel },
+  bhava_bala: { titleKey: "panel.bhava_bala", component: BhavaBalaPanel },
 };
 
 const UNKNOWN: PanelDefinition = { titleKey: "panel.unknown", component: UnavailablePanel };
````

````diff
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 23556a9..952cd1b 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -66,6 +66,7 @@ export const tr = {
     arudha: "Arudha padalar",
     argala: "Argala",
     shadbala: "Shadbala",
+    bhava_bala: "Bhava bala",
     unknown: "Bilinmeyen panel",
     loading: "Yükleniyor",
     unavailable: "Bu panel sonraki bir sürümde gelecek.",
@@ -204,6 +205,16 @@ export const tr = {
     lords: "Zaman lordları",
     lord: { year: "Yıl lordu", month: "Ay lordu", weekday: "Gün lordu", hora: "Saat lordu", tribhaga: "Tribhaga lordu" },
   },
+  bhavaBala: {
+    caption: "Bhava bala: on iki bhavanın lordu, üç kalemi, toplamı ve sırası",
+    bhava: "Bhava",
+    houseLabel: "{{house}}. bhava",
+    lord: "Lord",
+    lordBala: "Bhavadhipati",
+    dig: "Dig",
+    drishti: "Drishti",
+    note: "Bhavalar eşittir: bhava ortası lagnadan 30°'lik adımlarla sayılır ve ev sistemi ayarından bağımsızdır. Bhavadhipati, lordun Shadbala toplamıdır.",
+  },
   arudha: {
     caption: "Arudha padalar: on iki evin padası, burcu ve evin lordu",
     pada: "Pada",
````

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index 07d2d84..a31c4c4 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -67,6 +67,7 @@ export const en: Messages = {
     arudha: "Arudha padas",
     argala: "Argala",
     shadbala: "Shadbala",
+    bhava_bala: "Bhava bala",
     unknown: "Unknown panel",
     loading: "Loading",
     unavailable: "This panel arrives in a later version.",
@@ -205,6 +206,16 @@ export const en: Messages = {
     lords: "Time lords",
     lord: { year: "Year lord", month: "Month lord", weekday: "Weekday lord", hora: "Hora lord", tribhaga: "Tribhaga lord" },
   },
+  bhavaBala: {
+    caption: "Bhava bala: lord, three items, total and rank of the twelve bhavas",
+    bhava: "Bhava",
+    houseLabel: "Bhava {{house}}",
+    lord: "Lord",
+    lordBala: "Bhavadhipati",
+    dig: "Dig",
+    drishti: "Drishti",
+    note: "The bhavas are equal: each midpoint is counted from the lagna in steps of 30° and does not depend on the house system setting. Bhavadhipati is the Shadbala total of the lord.",
+  },
   arudha: {
     caption: "Arudha padas: the pada of each of the twelve houses, its sign and the lord of the house",
     pada: "Pada",
````

````diff
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 3d38609..a433953 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -67,6 +67,7 @@ export const fr: Messages = {
     arudha: "Arudha padas",
     argala: "Argala",
     shadbala: "Shadbala",
+    bhava_bala: "Bhava bala",
     unknown: "Panneau inconnu",
     loading: "Chargement",
     unavailable: "Ce panneau arrivera dans une version ultérieure.",
@@ -205,6 +206,16 @@ export const fr: Messages = {
     lords: "Maîtres du temps",
     lord: { year: "Maître de l'année", month: "Maître du mois", weekday: "Maître du jour", hora: "Maître de l'heure", tribhaga: "Maître du tribhaga" },
   },
+  bhavaBala: {
+    caption: "Bhava bala : maître, trois éléments, total et rang des douze bhavas",
+    bhava: "Bhava",
+    houseLabel: "Bhava {{house}}",
+    lord: "Maître",
+    lordBala: "Bhavadhipati",
+    dig: "Dig",
+    drishti: "Drishti",
+    note: "Les bhavas sont égaux : chaque milieu est compté depuis le lagna par pas de 30° et ne dépend pas du système de maisons choisi. Bhavadhipati est le total Shadbala du maître.",
+  },
   arudha: {
     caption: "Arudha padas : le pada de chacune des douze maisons, son signe et le maître de la maison",
     pada: "Pada",
````

- [ ] **Step 4: Testleri koş**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
```
Expected: `Test Files  72 passed (72)`, `Tests  674 passed (674)` (665 + 9).

- [ ] **Step 5: Kapılar**

```bash
pnpm --dir app build 2>&1 | grep "✓ built"
```
Expected: `✓ built`.

- [ ] **Step 6: Commit**

```bash
git add app/ui
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): bhava bala panel on the shared shadbala report"
```
Expected: iki `grep` çıktısız.

---

### Task 7: Uçtan uca: Shadbala ve Bhava bala

Spec K1 (panel açılmadan istek yok), K12 (paletten ekleme ve çıkarma; odak), §13 (ortak seçimin yayılması). IPC taklidi Task 4'te eklenmiştir (`compute_shadbala`).

**Files:**
- Modify: `app/e2e/desk.spec.ts`

**Interfaces:**
- Consumes: `mockIpc`, `ipcCalls` (`app/e2e/ipc.ts`); fikstürdeki `shadbala` raporu; Task 5 ve 6'nın panelleri.
- Produces: e2e testi `shadbala and bhava bala: two palette panels on one command; units, items view and the shared highlight`.

- [ ] **Step 1: Testi yaz**

````diff
diff --git a/app/e2e/desk.spec.ts b/app/e2e/desk.spec.ts
index 15f61d8..f92e19f 100644
--- a/app/e2e/desk.spec.ts
+++ b/app/e2e/desk.spec.ts
@@ -516,3 +516,83 @@ test("chara dasha: the system selector in the dasha tree and strip; each system
   await expect(items.nth(1)).toBeFocused();
   await expect(strip.getByRole("list")).not.toHaveAttribute("aria-busy");
 });
+
+test("shadbala and bhava bala: two palette panels on one command; units, items view and the shared highlight", async ({ page }) => {
+  await mockIpc(page, { session: true });
+  await page.goto("/");
+  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
+  const toggle = async (query: string, name: RegExp) => {
+    await page.keyboard.press("Meta+k");
+    const palette = page.getByRole("dialog", { name: "Komut paleti" });
+    await palette.getByRole("combobox").fill(query);
+    await palette.getByRole("option", { name }).click();
+    await expect(page.getByRole("dialog")).toHaveCount(0);
+  };
+  type ShadbalaCall = { input: { jd_ut: number; time_known: boolean; settings: { schema_version: number } } };
+  const calls = async () => (await ipcCalls(page, "compute_shadbala")) as ShadbalaCall[];
+  // Rapor panel açılmadan istenmez (ileri modüller spec'i K1).
+  expect(await calls()).toHaveLength(0);
+
+  await toggle("shadbala", /Shadbala panelini aç\/kapat/);
+  const shadbala = page.getByRole("region", { name: "Shadbala" });
+  await expect(shadbala).toBeFocused();
+  await expect(shadbala).toBeInViewport();
+  const summary = shadbala.getByRole("table", { name: /^Shadbala: yedi grahanın altı gücü/ });
+  await expect(summary).toBeVisible();
+  expect(await calls()).toHaveLength(1);
+  expect((await calls())[0].input).toMatchObject({ jd_ut: 2451545, time_known: true, settings: { schema_version: 2 } });
+  // Jüpiter en güçlü grahadır (sıra 1): toplam 533,1 virupa, oran 1,37.
+  const jupiter = summary.locator('tbody tr[data-graha="Jupiter"]');
+  await expect(jupiter.locator("td")).toHaveText(["233.8", "35.0", "185.8", "36.4", "34.3", "7.8", "533.1", "390.0", "1.37", "1"]);
+  await expect(shadbala.getByRole("list", { name: "Zaman lordları" }).getByRole("listitem")).toHaveText([
+    "Yıl lordu: Merkür",
+    "Ay lordu: Jüpiter",
+    "Gün lordu: Satürn",
+    "Saat lordu: Güneş",
+    "Tribhaga lordu: Satürn",
+  ]);
+  // Birim ve ayrıntı: yeniden hesap istenmez.
+  await shadbala.getByRole("combobox", { name: "Birim" }).selectOption("rupa");
+  await expect(jupiter.locator("td").nth(6)).toHaveText("8.89");
+  await shadbala.getByRole("combobox", { name: "Ayrıntı" }).selectOption("items");
+  await expect(shadbala.getByRole("table", { name: "Sthana bala kalemleri" })).toBeVisible();
+  await expect(shadbala.getByRole("table", { name: "Zaman gücü kalemleri" }).locator('tbody tr[data-graha="Saturn"] td').nth(5)).toHaveText("0.75");
+  expect(await calls()).toHaveLength(1);
+
+  // Bhava bala kendi isteğini yapar (her panel bir kez): aynı girdi, aynı rapor.
+  await toggle("bhava", /Bhava bala panelini aç\/kapat/);
+  const bhava = page.getByRole("region", { name: "Bhava bala" });
+  await expect(bhava).toBeFocused();
+  const seventh = bhava.locator('tbody tr[data-house="7"]');
+  await expect(seventh.locator("td")).toHaveText(["Jüpiter", "533.1", "30.0", "55.9", "619.0", "1"]);
+  expect(await calls()).toHaveLength(2);
+  expect((await calls())[1].input).toEqual((await calls())[0].input);
+
+  // Ortak vurgu: Bhava bala'daki lord, Shadbala satırını ve haritadaki grahayı vurgular.
+  await seventh.getByRole("button", { name: "Jüpiter" }).hover();
+  await expect(jupiter).toHaveClass(/is-highlighted/);
+  await expect(page.locator('svg [data-graha="Jupiter"]').first()).toHaveClass(/chart-planet--highlight/);
+  await expect(bhava.locator('tbody tr[data-house="10"]')).toHaveClass(/is-highlighted/);
+
+  // Paneller ve ayarları düzenle birlikte kaydedilir.
+  await expect
+    .poll(async () => {
+      const saved = (await ipcCalls(page, "save_view_state")).at(-1) as { viewState: { panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: unknown }[] }[] } } } | undefined;
+      const layout = saved?.viewState.panel_layout;
+      return layout?.layouts
+        .find((l) => l.id === layout.active_layout_id)
+        ?.panels.filter((p) => p.kind === "shadbala" || p.kind === "bhava_bala")
+        .map((p) => [p.kind, p.settings]);
+    })
+    .toEqual([
+      ["shadbala", { unit: "rupa", detail: "items" }],
+      ["bhava_bala", {}],
+    ]);
+
+  // Aynı komut paneli kaldırır; odak gövdeye düşmez; diğer panel yeniden istemez.
+  await toggle("shadbala", /Shadbala panelini aç\/kapat/);
+  await expect(shadbala).toHaveCount(0);
+  await expect(bhava).toBeVisible();
+  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
+  expect(await calls()).toHaveLength(2);
+});
````

- [ ] **Step 2: Koş**

```bash
lsof -i :1421
pnpm --dir app build 2>&1 | grep "✓ built"
pnpm --dir app e2e 2>&1 | tail -1
rm -rf app/test-results
```
Expected: `lsof` çıktısız (port boş; 1420'deki `tauri dev` durdurulmaz); `✓ built` (`tsc -p e2e` hatasız); `16 passed`.

- [ ] **Step 3: Commit**

```bash
git add app/e2e/desk.spec.ts
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "test(e2e): shadbala and bhava bala panels from the palette; units, items view and the shared highlight"
```
Expected: `grep` çıktısız.

---

### Task 8: Son doğrulama, elle denetim kapısı ve çalışma kâğıdının gözden geçirilmesi

Kod değişmez; bulunan bir hata kendi düzeltme commit'iyle çözülür ve bu görev baştan koşulur. Tek dosya değişikliği, gözden geçirenin çalışma kâğıdındaki tabloya yazdığı satırdır (Step 5).

**Files:**
- Modify: `tests/golden/modules/shadbala/WORKSHEET.md` (yalnızca gözden geçiren tablosu; Step 5). İlerleme notları depo dışıdır; `git add` edilmez.

**Interfaces:**
- Consumes: Task 1–7'nin tamamı.
- Produces: yeşil kapılar, kural taramaları, elle denetim sonucu, sahibine devredilen maddeler ve çalışma kâğıdının bağımsız gözden geçirmesi (kâğıda ve ilerleme notlarına yazılır).

- [ ] **Step 1: Rust kapıları**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
cargo test -p jyotish-desk --test modules -- --nocapture 2>&1 | grep -E "median|SkyFacts"
cargo deny check 2>&1 | tail -1
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p jyotish-core -p store -p jyotish-desk --all-targets 2>&1 | tail -1
git diff 4d435e7..HEAD --stat -- deny.toml Cargo.toml Cargo.lock app/package.json app/pnpm-lock.yaml app/vite.config.ts app/src-tauri/capabilities app/src-tauri/tauri.conf.json crates/store crates/jyotish-core/src/settings.rs crates/jyotish-core/src/chart.rs
```
Expected: fmt ve clippy çıktısız; testler: `jyotish-core` lib `237 passed`, golden `8 passed`, `modules_golden` `9 passed`; `store` lib `86 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `46 passed`, `bindings` `1 passed`, `compute_chart` `1 passed`, `modules` `1 passed`, `open_client` `1 passed`, `ui_fixture` `2 passed`; altı `median` satırı, `compute_shadbala` 40 ms'nin altında; `SkyFacts` satırı Task 4 Step 5'teki değerlerle; deny `advisories ok, bans ok, licenses ok, sources ok`; MSRV `Finished`; son `git diff --stat` çıktısız (bağımlılık, yetenek, yapılandırma, `store`, ayar tipi ve `Chart` değişmedi: K1, C3).

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
Expected: `lsof` çıktısız; `Test Files  72 passed (72)`, `Tests  674 passed (674)`; `✓ built`; e2e `16 passed`; görsel `10 passed` (temel PNG'ler değişmedi: D22); betik testleri `pass 14`, `fail 0`; `git status --short` boş.

- [ ] **Step 3: Kural taramaları (`4d435e7`'den bu yana)**

```bash
git diff 4d435e7..HEAD -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git log 4d435e7..HEAD --format=%B | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff 4d435e7..HEAD --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
git diff 4d435e7..HEAD --name-only | grep -vE '^(app|crates|scripts|tests|docs)/'
echo "trailers: $(git log 4d435e7..HEAD --format=%B | grep -c '^Co-Authored-By: ') commits: $(git rev-list --count 4d435e7..HEAD)"
git diff 4d435e7..HEAD -U0 -- app/ui | grep '^+' | grep -E 'console\.(log|info|warn|error|debug)'
git diff 4d435e7..HEAD -U0 -- app/ui | grep '^+' | grep -E '\b(alert|confirm|prompt)\(|window\.(print|open)\('
git diff 4d435e7..HEAD -U0 -- 'app/ui/**/*.css' | grep '^+' | grep -E '#[0-9a-fA-F]{3,8}\b|rgb\('
grep -rn '"source"' tests/golden/modules | grep -v '"source": "hand"'
ls tests/golden/modules/shadbala/*.json | wc -l
grep -rnE 'serde\(default' crates/jyotish-core/src
grep -rnE 'XMLHttpRequest|WebSocket|\bfetch\(' app/ui/modules app/ui/panels
git diff 4d435e7..HEAD --stat -- app/ui/print app/e2e/visual.spec.ts-snapshots
git tag --points-at HEAD
```
Expected: `trailers:` satırındaki iki sayı eşit; `grep -ciE` satırı `0`, `ls … | wc -l` satırı `4` yazar; bunların dışındaki her komut çıktısızdır (çıktısız `grep` 1 ile çıkar; beklenen budur): yasaklı ad yok (üretilen fikstürde de; üçüncü gücün klasik adı hiçbir yerde geçmez), beklenmeyen yerde dosya yok, `console` ve yasak çağrı yok, yeni stilde hex ya da `rgb(` yok (C9), el örneklerinde `hand` dışında kaynak yok (K9), çekirdek tiplerinde `serde(default)` yok, modül ve panel kodunda ağ çağrısı yok (C1), baskı ve görsel temeller değişmedi (K10, D22), ajan etiket atmamış.

- [ ] **Step 4: Elle denetim kapısı (`pnpm --dir app tauri dev`)**

Gerçek uygulamada, deneme danışanlarıyla (gerçek danışan verisi kullanılmaz). Her madde ilerleme notlarına "geçti" ya da bulgu olarak yazılır; bulgu varsa düzeltme commit'i ve bu görev baştan. Her maddenin başında, otomatik bir karşılığı olup olmadığı yazılıdır. **[otomatik]** maddelerin karşılığı testlerde vardır ve yürütücü ajan bunları testin adını göstererek "geçti" sayabilir; **[insan]** maddeler yalnızca gerçek uygulamaya bakan bir insan tarafından doğrulanabilir ve sahibi yoksa ona devredilir (bu devir `git push`'u engellemez; hiçbir şey yayımlanmaz).

- [ ] **[otomatik]** Komut paletinde "Shadbala panelini aç/kapat" ve "Bhava bala panelini aç/kapat" komutları yalnızca harita açıkken vardır; panel etkin düzene eklenir ve odak ona geçer; aynı komut paneli çıkarır ve odak gövdeye düşmez; rapor panel açılmadan istenmez. (e2e `shadbala and bhava bala: …`; `Palette.test.tsx`, `layout/model.test.ts`)
- [ ] **[otomatik]** Shadbala paneli yedi grahayı altı güç, toplam, gerekli, oran ve sırayla gösterir; "Ayrıntı → Kalemler" sthana balanın ve zaman gücünün alt kalemlerini ekler; "Birim → Rupa" değerleri iki ondalıkla yazar; iki ayar düzenle birlikte saklanır ve yeniden hesap istemez; altta beş zaman lordu durur. (`ShadbalaPanel.test.tsx`, e2e)
- [ ] **[otomatik]** Oranı 1'in altında olan grahada hücre `<1` metnini taşır ve altında açıklama satırı çıkar; savaştaki graha "savaş: …" metniyle yazılır; renk tek başına anlam taşımaz. (`ShadbalaPanel.test.tsx › a ratio below one …`)
- [ ] **[otomatik]** Bhava bala paneli on iki bhavayı lord, üç kalem, toplam ve sırayla gösterir; birim ayarı saklanır. (`BhavaBalaPanel.test.tsx`, e2e)
- [ ] **[otomatik]** Doğum saati "bilinmiyor" olan haritada iki panel de tablo yerine tek satır açıklama gösterir; araç çubuğu yerindedir; Rust raporu boş döndürür. Gün doğumu olmayan haritada açıklama Güneş'i söyler. (iki panel testi; çekirdek `no_report_without_a_birth_time_or_a_sunrise_…`; Rust `modules` testi: Tromsø)
- [ ] **[otomatik]** Hata durumunda seçiciler görünür durur; "Yeniden dene" klavyeyle kullanılınca odak panelde durur; yükleme 1 saniyeyi geçerse iskelet çıkar. (iki panel testi)
- [ ] **[otomatik]** Ortak vurgu: Bhava bala'daki bir lord, Shadbala'daki satırı ve haritadaki grahayı vurgular; bir bhava kendi satırını, bir graha lordu olduğu bhavaları vurgular. (iki panel testi, e2e)
- [ ] **[insan]** Gerçek motorla ayarlar: ayanamsa ya da "Gün doğumu tanımı" değişince iki panel yeniden hesaplanır (kendradi, saptavargaja, saat lordu değişebilir); ev sistemi Placidus'a alınınca Shadbala ve Bhava bala değerleri değişmez (K4, D14). (e2e taklidi raporu yeniden hesaplamaz; bu yalnızca gerçek motorla görülür.)
- [ ] **[insan]** Kutup dairesinin kuzeyinde yaz ya da kış gündönümüne yakın bir deneme haritasında iki panel "gün doğumu ya da batımı yok" açıklamasını, harita başlığı da uyarı rozetini gösterir.
- [ ] **[insan]** İki panel açık ve koyu temada, Türkçe, İngilizce ve Fransızca dilde okunur; özet tablosu dar panelde yatay kaydırılır ve taşmaz; "Kalemler" görünümünde üç tablonun başlıkları okunur; sayılar sağa hizalı ve eş genişliklidir.
- [ ] **[insan]** VoiceOver: tablo başlığı ve sütun adları okunur; `<1` işareti "gerekli değerin altında" diye, savaş tam adlarla ve kazananla okunur; "Ayrıntı" ve "Birim" seçicileri adlarıyla duyurulur; bhava numarası "7. bhava" diye okunur.
- [ ] **[insan]** Bilinen bir haritada (astrologun kendi elindeki bir örnek) kalemler astrologun beklediğiyle karşılaştırılır. Uyuşmazlık bulgu olarak yazılır; yöntem farkıysa (saptavargaja değerleri, yıl ve ay lordunun güneş girişinden bulunması, cheshtanın ortalama öğelerle hesabı, drik balanın okunuşu, sıranın toplama göre verilmesi) "Bu planın kapsamadıkları"na ve spec §17'ye göre sınıflanır.
- [ ] **[insan]** Bütün denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, doğum verisi, konum ya da dosya yolu görünmez.

Kapı geçtiyse ilerleme notlarına `Plan 17 Task 8 elle denetim: geçti (<tarih>)` yazılır; **[insan]** maddeleri devredildiyse `Plan 17 Task 8 elle denetim: otomatik maddeler geçti, insan maddeleri sahibine devredildi (<tarih>)` yazılır. Bu adım commit üretmez.

- [ ] **Step 5: Çalışma kâğıdının bağımsız gözden geçirilmesi (gözden geçirme maddesi)**

Spec §7: "kâğıdı motoru yazan dışında bir gözden geçiren doğrular"; §13: "`derivation` ayrı bir gözden geçirmede kuraldan yeniden hesaplanır." Gözden geçiren (motoru yazan dışında bir ajan ya da insan), `tests/golden/modules/shadbala/` altındaki üç tam örneğin her birinde yalnızca `input`'tan başlar ve motoru çalıştırmadan, Rust kaynağını okumadan `WORKSHEET.md`'nin satırlarını spec §7 ve §4'ten yeniden hesaplar: (1) burçlar, burç içi dereceler ve K6 sınıfı (Ay'ın büyümesi, Merkür'ün burcundaki grahalar); (2) sthana balanın beş kalemi: düşüş noktasından uzaklık; yedi varganın burcu (D2, D3, D7, D9, D12, D30 kuralları) ve D5'in sırası (moolatrikona, kendi burcu, doğal ve D1'den geçici ilişki); tek ve çift burç; ev grubu; on derecelik dilim; (3) dig bala: dört güçsüz nokta; (4) zaman gücü: öğleye uzaklık ve 24'ten çıkarma (D6); uzanım ve sınıfa göre paksha; üçte bir; üç varanın lordu ve saat diliminin sırası (lord, Satürn → Jüpiter → Mars → Güneş → Venüs → Merkür → Ay döngüsünden sayılır); `asin(sin 23,45° × sin λ)` ve işareti; (5) cheshta: dairesel orta nokta ve `K`; (6) naisargika; (7) drik: §4 tablosundan açı başına değer, özel bakış eklemesi, çeyrek ve Merkür ile Jüpiter'in tamamı (D11); (8) toplam, oran, sıra ve savaş (D12, D13); (9) bhava bala: burç türü, güçsüz bhava, bhava ortasına bakışlar. Sonra kendi değerlerini kâğıtla ve JSON'daki `expected` ile karşılaştırır (tolerans 0,01 virupa); farklar yazılır. Fark varsa önce kural yeniden okunur; düzeltme hangi tarafta olursa olsun ayrı commit'tir ve beklenen değer motor çıktısına bakarak değiştirilmez. Ayrıca `temporal.rs`'teki `HAND_HORAS` tablosunun 7 × 24 hücresi ve `bhava.rs`'teki burç türü tablosu aynı kuraldan satır satır denetlenir.

Gözden geçiren, kâğıdın başındaki tabloya adını (ya da ajan kimliğini), tarihi ve sonucu yazar:

```bash
git add tests/golden/modules/shadbala/WORKSHEET.md
git diff --cached --stat
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "docs(golden): shadbala worksheet re-derived by an independent reviewer"
cargo test -p jyotish-core --test modules_golden 2>&1 | grep -E "^test result"
```
Expected: yalnızca `WORKSHEET.md` değişir (tablo satırı; fark bulunduysa ilgili aritmetik satırları ayrı commit'tedir); `grep` çıktısız; `9 passed`. Sonuç ilerleme notlarına `Plan 17 çalışma kâğıdı yeniden türetildi: 3/3 (<tarih>, <gözden geçiren>)` olarak yazılır.

---

## Başarı ölçütleri

- `jyotish-core` Shadbala'yı ve Bhava bala'yı verir: sthana balanın beş kalemi, dig, zaman gücünün sekiz kalemi ve beş zaman lordu, cheshta, naisargika, drik, toplam, rupa, gerekli değer, oran, sıra, graha savaşı işareti; on iki bhavanın üç kalemi, toplamı ve sırası. Hepsi kalem düzeyinde el örnekleri, üç örnekli çalışma kâğıdı (kaynak `hand`, tolerans 0,01 virupa), elle yazılmış tablolar ve proptest değişmezleriyle doğrulanır; lagna yoksa rapor `None`'dır.
- Toplayıcı gün doğumlarını, MC'yi, tropikal ve ortalama boylamları ve iki güneş girişinin varasını toplar; gerçek motorla giriş anında Güneş 0° ± 1″'dedir, dış gezegenlerde karşı konumda cheshta ≥ 55 ve kavuşumda ≤ 5'tir, ortalama boylam katsayıları D8'in sınırları içindedir.
- `compute_shadbala` ortak kalıpla çalışır (motor iş parçacığı, `blocking`, `CommandError`), dört golden anında v1 haritasıyla çapraz denetlenir, kutup gecesindeki beşinci anda ve doğum saati bilinmeyen girdide `None` döner ve bütçesinin içindedir (≤ 40 ms).
- İki panel paletten eklenir; Shadbala paneli özet ve kalem görünümlerini, virupa ve rupa birimlerini, zaman lordlarını, `<1` işaretini ve savaş metnini gösterir; Bhava bala paneli on iki bhavayı gösterir. Boş, yükleniyor, hata, dolu ve "saat bilinmiyor" durumları testlidir; renk tek başına anlam taşımaz; yorum metni yoktur.
- `Chart`, `open_client`, `CalcSettings`, `ViewState`, `store`, baskı düzenleri ve görsel temeller değişmemiştir; v1 golden testleri ve panchang testleri değişmeden geçer.
- Kapılar: `jyotish-core` lib 237, `modules_golden` 9, `store` lib 86 (+1 ign.), `jyotish-desk` `modules` 1; Vitest 72/674; e2e 16; görsel 10; `cargo deny` temiz; diğer sayılar tabandaki gibi. Elle denetim kapısının **[otomatik]** maddeleri testlerle doğrulanmış, **[insan]** maddeleri yapılmış ya da sahibine devredilmiştir; çalışma kâğıdı bağımsız olarak yeniden türetilmiş ve kâğıda yazılmıştır.

## Bu planın kapsamadıkları

- **Spec'in 3.4 için kapsam dışı saydıkları:** graha savaşında güç aktarımı, ishta ve kashta phala, vimshopaka bala, ahargana temelli yıl ve ay lordu, sekiz hareket durumuna dayalı cheshta, bhava balanın yerleşim ve gündüz/gece ek düzeltmeleri, avasthalar, Rahu ve Ketu gücü.
- **Sonraki alt projeler (3.5+):** yogalar, uyum, Muhurta, Prashna, arama. Güç sırasının arama koşulu olması 3.9'un, saat lordlarının gün çizelgesinde gösterimi 3.7'nin işidir.
- İkinci bir yöntem (ör. 22,5 / 7,5 dizili saptavargaja, ahargana temelli lordlar, orana göre sıra): K7'ye göre ancak ikinci seçenek uygulanınca ayar olur; bu planda ayar eklenmez.
- Yatay çubuk gösterimi (D20) ve kalemlerin tek geniş tabloda birleştirilmesi (D21).
- Shadbala ve Bhava bala'nın baskı düzeni (K10); raporun oturum içi paylaşılan önbelleği (D19) ve veritabanında saklanması (spec §16).
- "Rapor yok" nedeninin Rust'tan ayrı bir alanla dönmesi (D3: tip spec'teki gibi korunur).
- İç gezegenlerin güneş merkezli konumunun ephemeris'ten okunması (`Ephemeris` trait'i jeosantrik konum verir; D8).
- Paket parçalama (D22: 500 kB uyarısı).
- Yorum metinleri ve AI yorum paneli; Windows ve Android.
- v1'den taşınan seyrek test dalgalanması (`NotesPanel.test.tsx › New note adds an empty note…`, yük altında): bu planın dosyalarına dokunmaz; ayrı bir bakım işidir.

## Spec'e geri bildirim (spec kusurları ve boşlukları)

Plan yazılırken bulunan ve Kararlar'da çözülen maddeler. "Kusur" spec'in kendi içinde çelişen, harfiyen uygulanamayan ya da ölçümle tutmayan yeridir; "boşluk" yazılmamış ayrıntıdır.

1. **Kusur — §7 IPC: "doğum saati bilinmiyorsa ya da gün doğumu yoksa `None` ve ilgili uyarı".** Dönüş tipi `Option<ShadbalaReport>`'tır; `None` uyarı taşıyamaz ve `warnings` alanı raporun içindedir. Çözüm (D3): `None`; arayüz nedeni doğum saati bilgisinden ve haritanın mevcut `SunriseUnavailable` uyarısından okur. Spec ya "uyarı haritanın uyarısıdır" demeli ya da dönüşü nedeni taşıyan bir tipe çevirmeli.
2. **Kusur — §7 Cheshta ve §17: "katsayılar … ephemeris'ten uzun dönem ortalamasıyla ±0,05° içinde doğrulanır".** Ölçüm (1900–2100, gerçek − ortalama): Güneş −0,007°, Mars −0,027°, Jüpiter +0,055°, Satürn −0,212°. Jüpiter ve Satürn'ün ~900 yıllık karşılıklı tedirginliği iki yüzyılda ortalanmaz; sınır bu iki gezegende tutmaz. Ayrıca Merkür ve Venüs'ün güneş merkezli ortalama boylamı, yalnızca jeosantrik konum veren `Ephemeris` trait'iyle doğrudan ölçülemez. Çözüm (D8): Güneş ve Mars ±0,05°; Jüpiter ±0,1°, Satürn ±0,3°; iç gezegenler alt kavuşum anlarında (Venüs ±0,05°, Merkür ±0,5°). Spec sınırı gezegen başına yazmalı ve iç gezegenlerin yöntemini vermeli.
3. **Kusur — §7 Test: "beş `swetest` fixture anında hata yok ve property'ler sağlanır".** Beş andan biri (Tromsø 2050-12-21) kutup gecesindedir; spec'in kendi kuralına göre raporu `None`'dır ve property'ler ona uygulanamaz. Çözüm (D17): dört rapor, bir `None`. Spec sayıyı düzeltmeli ya da Shadbala için gün doğumlu beşinci bir an eklemeli.
4. **Boşluk — §7 `ShadbalaAstro`.** Alanlar sayılmış ama tip verilmemiş; "önceki ve sonraki gün doğumu ile batımı"nın hangi dördü kastettiği, saf fonksiyonun lagna yokken ne döndüğü ve tutarsız zamanlarda davranışı yazılmamış. Çözüm (D2): doğuş, batış, sonraki doğuş; `Option`; `is_consistent`.
5. **Boşluk — §7 ve K5: kalemlerin çerçevesi.** Hangi kalemin rasi çerçevesinde, hangisinin sidereal ya da tropikal boylamla hesaplandığı yazılmamış. `Positions` lagnanın rasi çerçevesindeki derecesini taşımaz (yalnızca sidereal boylam ve rasi burcu); tropikal-rasi kipinde bhava ortasının Yay ve Oğlak'taki yarısı bu dereceye bağlıdır. Çözüm (D4): çerçeve farkı Güneş'ten okunur. Spec §4 `Positions`'a `lagna_deg_in_sign` eklemeyi değerlendirmeli.
6. **Boşluk — §7 Saptavargaja.** Moolatrikonanın burç mu derece aralığı mı olduğu, yücelme ve düşüş burcundaki grahanın değeri, kuralların sırası ve "D1'den geçici" ilişkinin lordla aynı burçtaki durumu yazılmamış. Çözüm (D5).
7. **Boşluk — §7 Gündüz–gece gücü ve dilimler.** "En yakın görünür öğle" şafaktan önceki doğumlarda ertesi günün öğlesidir (ikinci bir batış gerektirir); dilim sınırlarının hangi dilime ait olduğu ve günün 24 saat olmadığı durumda saat dilimlerinin uzunluğu yazılmamış. Çözüm (D6).
8. **Boşluk — §7 Yıl ve ay lordu.** Giriş varasının hangi yerde ölçüldüğü (doğum yeri mi, sabit bir boylam mı), tropikal-rasi kipinde hangi burç girişinin kastedildiği ve arama yöntemi yazılmamış. Çözüm (D7): doğum yeri, sidereal giriş, Newton. Bu, spec §17'nin "en çok tartışılabilir varyant" notuna eklenmeli.
9. **Boşluk — §7 Cheshta ve astronomik çapraz denetim.** "Dairesel orta nokta"nın karşıt boylamlardaki tanımı ve iç gezegenler için denetim koşulu yok (yalnızca dış gezegenler yazılmış). Çözüm (D9): alt kavuşumda ≥ 50, üst kavuşumda ≤ 10.
10. **Boşluk — §7 Naisargika.** İki ondalıklı sabitler mi, 60'ın yedide birleri mi kullanılacağı yazılmamış (fark 0,005'e kadar; tolerans 0,01). Çözüm (D10).
11. **Boşluk — §7 Drik bala.** Merkür ve Jüpiter'in "tamamı"nın çeyreğin üstüne mi yerine mi eklendiği ve kötücül sayılan Merkür'ün işareti yazılmamış. Çözüm (D11): harfiyen okuma (çeyrek artı tamamı, artı işaretle).
12. **Boşluk — §7 Graha savaşı.** `GrahaWar` tipi veri modelinde yok; 1° sınırının dahil olup olmadığı, üç grahanın birlikte durduğu ve enlemlerin eşit olduğu durum yazılmamış. Çözüm (D12).
13. **Boşluk — §7 "yedi graha arasındaki sırayı verir".** Sıranın toplama mı orana mı göre olduğu ve eşitlik kuralı yazılmamış; bhava sırası için de aynı. Çözüm (D13): toplam.
14. **Boşluk — §7 Test ve §13 şeması.** Çalışma kâğıdının JSON girdisi (`astro`, enlemler), boş sonucun gösterimi ve "gözden geçiren doğrular" kaydının nereye yazılacağı tanımsız. Çözüm (D15; Task 8 Step 5: kâğıttaki tablo).
15. **Boşluk — §7 Arayüz: "bir ondalık".** Oranı bir ondalıkla yazmak `<1` işaretiyle çelişen hücreler üretir (0,96 → `1.0 <1`); rupa değerleri bir ondalıkta ayırt edilemez. Yalnızca toplamın rupası (`rupas`) Rust'tan gelir; kalemlerin rupa gösterimi birim çevirisidir. Çözüm (D20).
16. **Boşluk — §7 Arayüz: düzen ve durumlar.** "Alt kalemleri açılır" düzeni, zaman lordları satırının biçimi, rapor yokken (iki ayrı neden) gösterilecek metin ve yöntem kimliğinin gösterimi yazılmamış. Çözüm (D21).

## Self-Review

**1. Spec kapsamı.**

| Gereksinim (ileri modüller spec'i) | Görev |
|---|---|
| §7 Uccha bala | Task 1 (`uccha_bala`) |
| §7 Saptavargaja bala (BPHS değerleri; 22,5 dizisi uygulanmaz) | Task 1 (`saptavargaja_points`, `saptavargaja_bala`; D5) |
| §7 Ojayugma, kendradi, drekkana | Task 1 (`ojayugma_bala`, `kendradi_bala`, `drekkana_bala`) |
| §7 Dig bala; lagna ve MC astronomik açı | Task 1 (`dig_bala`), Task 3 (`mc_lon`) |
| §7 Gündüz–gece gücü | Task 1 (`hours_from_noon`, `day_night_bala`; D6) |
| §7 Paksha bala (K6 sınıfı) | Task 1 (`paksha_bala`) |
| §7 Tribhaga bala; Jüpiter her zaman 60 | Task 1 (`tribhaga_lord`, `temporal_bala`) |
| §7 Yıl, ay, gün ve saat lordu | Task 1 (`time_lords`, `hora_lord`, `HAND_HORAS`), Task 3 (girişler; D7) |
| §7 Ayana bala; Güneş'te iki katı | Task 1 (`ayana_bala`, `temporal_bala`) |
| §7 Graha savaşı; güç aktarımı yok | Task 1 (`graha_war`; D12) |
| §7 Cheshta bala; J2000 doğrusal ortalama öğeler | Task 1 (`cheshta_bala`, `mean_longitudes`; D8, D9), Task 4 (doğrulama) |
| §7 Naisargika; drik | Task 1 (`naisargika_bala`, `drik_bala`; D10, D11) |
| §7 Toplam, gerekli, oran, sıra | Task 2 (`shadbala`; D13) |
| §7 Bhava bala (eşit bhava; üç kalem; sıra) | Task 2 (`bhava_balas`; D14) |
| §7 Veri modeli (alan adları spec'tekiyle birebir; `GrahaWar` eklenir) | Task 1 |
| §7 Saf fonksiyon ve `ShadbalaAstro`; toplayıcı | Task 2, Task 3 (D2) |
| §7 Komut; saat bilinmiyorsa ya da gün doğumu yoksa `None` | Task 3, Task 4 (D3) |
| §7 Arayüz: Shadbala (ayrıntı, birim, zaman lordları, `<1`, savaş metni, `tabular-nums`) | Task 5 |
| §7 Arayüz: Bhava bala (birim) | Task 6 |
| §7 Test: çalışma kâğıdı | Task 2; Task 8 Step 5 |
| §7 Test: kalem düzeyinde el örnekleri (uç değerler ve ara değer) | Task 1, 2 (`mod tests`) |
| §7 Test: property'ler (dokuz madde) | Task 2 (`shadbala_properties`) |
| §7 Test: astronomik çapraz denetim | Task 4 (`sky_facts`; yıl girişi 0° ± 1″) |
| §7 Test: uçtan uca | Task 4 (D17) |
| §7 Gizlilik, performans (≤ 40 ms) | C1; Task 4 (ölçüm), Task 8 |
| K1, K2, K3 | Task 2–4 |
| K4, K5 | D4, D14; Task 3 (`tropical_rasi_mode_…` testi) |
| K6 | Task 1 (`nature` çağrıları; kendi listesi yok) |
| K7 (ayar yok; `method`) | D1; "Bu planın kapsamadıkları" |
| K8, §14 | D3 (yeni kod yok), C1 |
| K9, §13 (el örnekleri, değişmezler, ikinci yazım, çapraz denetim, komut katmanı) | Task 1–4; Task 8 Step 5 |
| K10, K11, K12 | D22; klasik terimler üç dilde aynı (D21); Task 5–7 |
| Ek A | D1; C13 taramaları |

Boşluk: yok. Spec'in istemediği ama planın eklediği parçalar: iç gezegenler için kavuşum denetimi (D9), doğum saati bilinmeyen el örneği ve kapsam testi (D15), giriş varasının v1 panchangıyla çapraz denetimi (Task 4), `ShadbalaAstro::is_consistent` (D2), yöntem ve birim satırları (D21).

**2. Yer tutucu taraması.** "TODO", "TBD", "daha sonra", "uygun hata işleme" yok. Task 8'deki `<tarih>` ve `<gözden geçiren>` denetimin yapıldığı günün ve yapanın yazılacağı yerdir. Her kod adımı tam dosya ya da `4d435e7` ağacına (ve önceki görevlere) karşı üretilmiş diff taşır.

**3. Tip tutarlılığı.**
- `ShadbalaReport { grahas, time_lords, bhavas, method, warnings }`, `GrahaShadbala { graha, sthana, dig, temporal, cheshta, naisargika, drik, total, rupas, required, ratio, rank, war }`, `SthanaBala`, `TemporalBala`, `TimeLords` ve `BhavaBala` Task 1'de Rust'ta tanımlanır (alan adları spec §7'dekiyle aynıdır); TS bağlaması Task 4'te üretilir; `ui_fixture.rs`, `tests/modules.rs`, iki panel ve e2e aynı adları okur.
- `ShadbalaAstro` alanları (`jd_ut`, `sunrise_jd_ut`, `sunset_jd_ut`, `next_sunrise_jd_ut`, `mc_lon`, `tropical_lons`, `mean { sun, mercury, venus, mars, jupiter, saturn }`, `weekday`, `year_entry_weekday`, `month_entry_weekday`) Task 1'deki tipte, Task 2'deki JSON dosyalarında ve proptest üretecinde, Task 3'teki toplayıcıda ve Task 4'teki testte aynıdır.
- El örneği alanları (`uccha` … `drekkana`, `sthana`, `dig`, `day_night` … `ayana`, `temporal`, `cheshta`, `naisargika`, `drik`, `total`, `rank`, `war`; `time_lords`; `bhavas[].{house, lord, lord_bala, dig, drishti, total, rank}`) düzenekteki `ShadbalaExpected`, README satırı ve üç JSON dosyasında aynıdır.
- `SHADBALA_METHOD = "shadbala.bphs_v1"` Rust'ta, `modules_golden.rs`'te, `tests/modules.rs`'te, panel testlerindeki "Yöntem: shadbala.bphs_v1" metninde ve spec §7'de aynıdır.
- Komut adları: Rust `compute_shadbala`; gövde `compute_shadbala_blocking`; TS `computeShadbala`; e2e taklidindeki anahtar `compute_shadbala`; bağımsız değişken adı `input`.
- Fikstür anahtarı `shadbala` `ui_fixture.rs`, `test/fixture.ts`, panel testleri ve `e2e/ipc.ts`'te aynıdır.
- Panel türleri `shadbala` ve `bhava_bala`: `PANEL_KINDS`, `MODULE_PANELS`, `PANEL_REGISTRY`, `panel.*` i18n anahtarları, testlerdeki `Panel.kind` değerleri ve e2e'de kaydedilen düzen aynı adları taşır. Panel ayar anahtarları: `shadbala` `detail` ve `unit`; `bhava_bala` `unit`.
- `balaUnit`, `formatBala`, `formatFixed`, `UnitSelect` ve `BalaUnavailable` Task 5'te `bala.tsx`'te tanımlanır; iki panel ve `ShadbalaPanel.test.tsx` oradan okur. `useShadbala` iki panelin tek veri kaynağıdır.
- i18n: `bala.*` iki panelde ortak; `shadbala.*` yalnızca Shadbala panelinde; `bhavaBala.*` yalnızca Bhava bala panelinde; metinler testlerde birebir geçer.

**4. Kural denetimi.** C1 (`ChartInput` loglanmaz; yeni hata kodu yok), C2′ (komut dizgi almaz), C3 (`store`, ayar tipi ve görünüm kaydı değişmez), C4 (bütün değerler Rust'tan; birim çevirisi ve biçimleme gösterim), C5 (panel ayarları mevcut kayıt yoluyla), C7 (kısayol eklenmez), C9 (`<1` ve "savaş: …" metinleri; yeni stilde hex yok), C10 (iki üretilen dosya komutla), C12 (her yeni anahtar üç dilde; yeni hata kodu yok), C13 (her commit'te tarama), C14 (gün doğumu v1 tanımıyla), K9 (yalnızca `hand`), K10 (baskı ve görsel temeller değişmez) görevlerin adımlarında ve Task 8 Step 3'te denetlenir.

**5. Koşulan doğrulama.** Bu plandaki kod, deponun `4d435e7` üzerindeki bir kopyasında (kısa bir yolda, Rust 1.100) yedi commit olarak uygulanmış ve her commit'te ölçülmüştür: `jyotish-core` lib 191 → 215 → 231 → 237; `modules_golden` 7 → 9; `jyotish-desk` `modules` 1 (ortancalar: `compute_shadbala` 0,69–0,74 ms; diğer beş komut 0,54–0,79 ms); Vitest 653 → 653 → 665 → 674 (dosya 70 → 71 → 72); e2e 15 → 16. Son durumda `cargo fmt --check`, `clippy -D warnings` (1.100), `cargo test --workspace`, `cargo deny check` (dört `ok`), 1.93 ile `cargo check`, `pnpm build` (531,7 kB; D22), görsel 10 (temel dosyalar değişmeden) ve yayın betiği testleri 14 yeşildir; Task 8 Step 3'ün taramaları beklendiği gibidir. "Testlerin başarısız olduğunu doğrula" ve "derlemenin başarısız olduğunu doğrula" adımlarındaki çıktılar da aynı kopyada, bir önceki görevin ağacına yalnızca o adımın değişiklikleri konarak ölçülmüştür. Ayrıca bu belgedeki bloklar (17 yeni dosya ve 33 diff) belgeden okunup sırayla temiz bir `4d435e7` ağacına uygulanmış ve her görevin sonunda o commit'in ağacıyla birebir aynı sonucu vermiştir (üretilen iki dosyayı adımlardaki komutlar üretir).

Çalışma kâğıdının değerleri bu plan yazılırken motordan bağımsız bir hesapla türetilmiştir: kural metninden (spec §7, §4) başka bir dilde, tablolarla yazılmış ayrı bir hesap her satırın aritmetiğini dökmüş, JSON dosyaları o hesabın sonuçlarından kurulmuştur. Motor, üç örneğin bütün değerlerinde (örnek başına 7 × 20 kalem, 7 sıra, 7 savaş alanı, 5 zaman lordu ve 12 × 6 bhava değeri) ilk koşuda 0,01 virupa içinde uyuşmuş; kapsam testi de ilk koşuda geçmiştir. Hiçbir beklenen değer motor çıktısına bakılarak değiştirilmemiştir. İki türetmeyi de aynı yazar yazmıştır: kuralın aynı yanlış okunuşu ikisine de girmiş olabilir. Task 8 Step 5 bunun için vardır ve bu belgedeki doğrulamayla yerine getirilmiş sayılmaz.

Kalem düzeyindeki birim testlerinde ilk koşuda üç el değeri düştü; üçü de kural yeniden okunarak (motor çıktısı kopyalanmadan) düzeltildi: (1) ayana balanın ara değerinde el aritmetiği yanlıştı (`asin(0,397949 × 0,5)` 11,4758° değil 11,4770°'dir); (2) "kötücülle aynı burçtaki Merkür" örneğinde büyüyen Ay da aynı burca konmuştu (K6'ya göre iyicil varsa Merkür iyicildir; örnek yanlıştı, kural doğruydu) ve grahalar başka bir burca alındı; (3) tribhaga örnekleri tam dilim sınırına konmuştu (06:00 + 4 saat, kayan noktada sınırın bir adım altına düşer); örnekler sınırın 0,01 saat ötesine alındı (D6 sınırın hangi dilime ait olduğunu tanımlar, kayan nokta yuvarlamasını değil). Toplayıcı testlerindeki hafta günleri (Perşembe, Çarşamba) ve saat lordu tablosu ilk koşuda geçti. Yazım sırasında kodda düzeltilenler: clippy 1.100 `index % 2 == 0` yerine `is_multiple_of(2)` istedi; specta `f64` alanlarını `number | null` olarak tiplediği için biçimleyiciler `null` kabul edecek biçimde yazıldı; iki yorum satırındaki yasak dizi (Türkçe bir sözcüğün içinde) tarama ile bulundu ve sözcük değiştirildi. D8'in ölçümü ilk tasarımı değiştirdi: spec'in tek sınırı (±0,05°) Jüpiter ve Satürn'de tutmadığı için sınırlar gezegen başına yazıldı; katsayılara dokunulmadı. **Koşulmayanlar:** `pnpm tauri dev` ile gerçek uygulama ve gerçek veritabanı (Task 8 Step 4'ün **[insan]** maddeleri; özellikle gerçek motorla ayar değişiminin panellere yansıması ve kutup haritası); bir astrologun elindeki örnekle karşılaştırma (spec §17: yıl ve ay lordu yöntemi beta öncesinde teyit edilmeli); çalışma kâğıdının ikinci bir kişi tarafından yeniden türetilmesi (Task 8 Step 5); uzak CI (hiçbir şey gönderilmedi).
