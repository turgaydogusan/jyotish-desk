# Plan 26 — Geçiş araçları: nakshatra lordları, Sankranti listesi, efemeris tablosu, adımlı ve oynatılan geçiş

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Yol haritasının dördüncü planı (kalem 4.7–4.10); var olan hiçbir hesabın davranışı değişmez. (1) **Nakshatra lordları:** geçiş tablosunda her zaman, graha tablosunda isteğe bağlı bir "Nakshatra lordu" sütunu; olay aramasının nakshatra girişi satırlarında girilen nakshatranın lordu; Ay'ın önümüzdeki nakshatra girişleri için olay aramasında hazır seçim. (2) **Sankranti listesi:** seçilen yılın on iki güneş girişi, giriş anının tithisi, varası ve Ay nakshatrasıyla; yeni `sankranti` paneli. (3) **Efemeris tablosu:** seçilen ayın her günü için dokuz grahanın burcu, derecesi, retro işareti ve isteğe bağlı nakshatrası; o gün içindeki burç girişi ve yön dönüşü işaretli; yeni `ephemeris` paneli. (4) **Adımlı ve oynatılan geçiş:** transit anını saat, gün, hafta, ay ya da yıl adımıyla ileri-geri yürüten düğmeler ve oynat-duraklat; geçiş halkası ve tablo transit deposundan birlikte güncellenir. Yorum, hüküm ve sıralama yoktur.

**Architecture:**
- **Tek yeni çekirdek modülü: `crates/jyotish-core/src/transit_tools/`.** Üç dosya: `lords.rs` (nakshatra lordları tablosu), `sankranti.rs` (sidereal güneş girişleri ve giriş anının panchangı), `ephemeris_table.rs` (verilen anların dokuz grahası ve iki işaret). Var olan modüllerden yalnızca okunur: `timeline` yürüyücüsü, `panchang::compute_panchang`, `chart::raw_grahas`. **Var olan hiçbir Rust tipi değişmez:** `GrahaPosition`, `TransitGraha`, `SkyEvent`, `Chart`, `TransitAt` ve `CoreError` olduğu gibidir.
- **Nakshatra lordu alan değil, Rust'ın sabit tablosudur.** `NAKSHATRA_LORDS` bağlama dosyasına `tauri-specta` sabiti olarak çıkar; arayüz nakshatra numarasının satırını okur (`panels/nakshatraLord.ts`). Lord nakshatra numarasından tümüyle belirlidir; üç paylaşılan tipe alan eklemek eşzamanlı planların testlerini ve `delhi.json`'ı oynatırdı (D2; yol haritası farkı 1).
- **Komutlar yeni bir modül dizinindedir: `app/src-tauri/src/transit_tools_commands/`** (`sankranti.rs`, `ephemeris.rs`, `step.rs`). Sankranti ve efemeris motor iş parçacığında hesaplanır; adım saf hesaptır. "Bu yıl", "bu ay", önceki ve sonraki ay ile adım aritmetiği (ay sonu, yaz saati) Rust'tadır: arayüz tarih hesabı yapmaz (C4). Yeni hata kodu yoktur: hepsi `invalid_transit_input { field }` verir.
- **Sonuç saklanmaz ve `Chart`'a eklenmez (K1); veritabanı şeması değişmez (Y6).** Yıl, ay, adım birimi, hız ve oynatma durumu oturumluktur; yer seçimi ve efemerisin "Nakshatra" kutusu panelin ayarıdır ve düzenle birlikte saklanır.
- **Oynatma iş biriktirmez.** Sonraki adım, önceki an yerleşince (adım isteği bitmiş, o anın transitleri gelmiş) planlanır; an başka bir yoldan değişmişse, harita değişmişse ya da bileşen kalkmışsa geç yanıt düşer. Sistem "hareketi azalt" isterse oynatma sunulmaz.
- **Eşzamanlı planlarla çakışmayı küçük tutmak.** Plan 27 (Varshaphala I) ve Plan 29 (geçiş takvimi) aynı tabandan, ayrı çalışma ağaçlarında yazılıp uygulanır. Bu plan yeni kodu yeni dosyalara koyar; fikstürleri kendi dosyalarındadır (`sankranti.json`, `ephemeris.json`: `delhi.json`, `ui_fixture.rs` ve `fixture.ts` değişmez); akışları kendi `*.desk.spec.ts` dosyalarındadır (`desk.spec.ts` değişmez). Paylaşılan on bir dosyadaki her değişiklik yalnızca ektir ve "Paylaşılan dosyalar" bölümünde çapasıyla yazılıdır.
- **Testler:** çekirdekte el örnekleri ve ikinci yazımlar (tablo ↔ fonksiyon ↔ doğum dasası; giriş anı ↔ aritmetik ↔ burç başı; efemeris satırı ↔ transit hesabı); uygulamada gerçek motorla çapraz denetim (Sankranti ↔ ephemeris boylamı ↔ olay araması ↔ v1 panchangı; efemeris işaretleri ↔ `timeline` dizileri) ve bütçeler; adım aritmetiğinin takvim örnekleri; arayüzde Vitest (sahte zamanlayıcıyla oynatma dahil) ve dört uçtan uca akış.

**Tech Stack:** Rust 1.93+ (stable; yerelde ve CI'da 1.100), Tauri 2.12, tauri-specta 2.0.0-rc.29, serde, React 19, TypeScript 7, Vitest 5, Playwright 1.65.1. **Yeni Rust ya da npm bağımlılığı yok.**

**Spec:** `docs/superpowers/specs/2026-11-27-eksikler-yol-haritasi.md` (aşağıda "yol haritası"): §1 (taşınan kurallar), §2 (Y1–Y8), §4.7–§4.10, §5 "Plan 26" (görev taslağı ve sahibin denetimi), §8 (tek ephemeris iş parçacığı), Ek A. İleri modüller spec'i (`2026-11-26-alt-proje-3-ileri-moduller-design.md`, "İM spec"): §2 (K1–K12), §13, §14, §15. UI spec: `2026-11-24-ui-design.md` §3.3–§3.4, §5, §6, §8. Yol haritası bir tasarım belgesi değildir: bu planın tasarımı Architecture ve Kararlar bölümleridir.

**Ön koşul:** Taban `main` commit `9727c55`'tir (Plan 23, 24 ve 25 birleşmiş). Kod blokları ve sayaçlar o ağaca göre yazılmış ve deponun bir kopyasında, her görev ayrı commit olarak uygulanıp ölçülmüştür. `main` ilerlediyse Task 1 Step 1 tabanları yeniden ölçer ve sayaç zinciri o ölçümden, aynı farklarla yürütülür; bir `diff` bloğunun bağlamı tutmazsa önce "Paylaşılan dosyalar" bölümündeki çapa okunur. Önceki planların **[insan]** maddeleri bu planı engellemez (Y3).

**Yürüten ajan için (bu plan kendi başına yeterlidir):** Her görevde adımlar sırayla uygulanır. "**Yeni dosya:**" bloğu dosyanın tam içeriğidir. `diff` bloğu depo kökünde bir dosyaya kaydedilip `git apply <dosya>` ile ya da elle uygulanır. Üretilen dosyalar (`app/ui/bindings.ts`, `app/ui/test/fixtures/sankranti.json`, `app/ui/test/fixtures/ephemeris.json`) hiçbir zaman elle düzenlenmez; adımlardaki komutlarla üretilir. Gerçek uygulama hiçbir adımda çalıştırılmaz. E2e yalnızca özel portta koşar (14326; Task 1 Step 1).

## Kararlar

Sahibi ürün kararlarını devretti; muhafazakâr seçenek seçildi ve her karar bir satır gerekçeyle yazıldı. "Yol haritası farkı" diye işaretlenenler yol haritasının görev taslağından bilerek ayrılan yerlerdir ve belgenin sonunda aynı numarayla toplanmıştır. D6–D8, D12–D14, D22–D23 ve D25–D26 sahibin onayını bekleyen okuma seçimleridir ve `tests/golden/modules/transit_tools/REVIEW.md`'de varyantlarıyla yazılıdır. D17'deki ölçümler ve sayaç zinciri bu plan yazılırken deponun bir kopyasında alınmıştır (macOS 27.2, Rust 1.100, hata ayıklama derlemesi; makine yük altındaydı).

- **D1 — Çekirdek modülü `crates/jyotish-core/src/transit_tools/`'tır (`lords.rs`, `sankranti.rs`, `ephemeris_table.rs`); `lib.rs` kaydı dosyanın sonunda, ayrı bir bloktadır.** Gerekçe: `rustfmt` aynı gruptaki `mod` ve `use` satırlarını alfabetik sıraya zorlar; Plan 25'in bloğu dosyanın başındadır. Sondaki blok, eşzamanlı planların alfabetik gruplardaki ve baştaki ekleriyle aynı satırlara dokunmaz.
- **D2 — Nakshatra lordu Rust tiplerine alan olarak eklenmez. `NAKSHATRA_LORDS: [Graha; 27]` çekirdekte sabittir, `specta_builder()` onu bağlama dosyasına sabit olarak yazar ve arayüz `nakshatraLord(n)` ile satırı okur.** Gerekçe: lord, nakshatra numarasının tek değerli bir özelliğidir (nakshatra adı gibi); tablo Rust'ındır ve üretilen dosyayla gelir, yani arayüz lordu türetmez (C4). Alan eklemek `GrahaPosition`, `TransitGraha` ve `SkyEvent`'i değiştirirdi: Plan 29 `SkyEvent` üzerine kurulur ve o tipin elle yazılmış her örneği (Rust ve TypeScript testleri) birleştirmede derlenmezdi; `delhi.json`'ın her satırı da oynardı. Test, tabloyu `natal_extras::nakshatra_lord` ve `birth_dasha` ile 27 nakshatrada karşılaştırır. **Yol haritası farkı 1.**
- **D3 — Geçiş tablosunda "Nakshatra lordu" sütunu her zaman görünür (nakshatradan hemen sonra). Graha tablosunda sütun isteğe bağlıdır (`settings.nakshatraLord`), varsayılan olarak gizlidir ve `columns` listesinin dışındadır; lagna satırı da lordunu yazar; doğum saati bilinmiyorsa Ay'ın lordu "≈" taşır.** Gerekçe: graha tablosunun varsayılan sütunları ve baskı tablosu (`columns` listesini kullanır) değişmez (K10); kalıp "Karaka" ve "Baladi" sütunlarının kalıbıdır. Komut çağrılmaz.
- **D4 — Olay aramasında lord yalnızca nakshatra girişi satırında, var olan "Burç · nakshatra" hücresinde yazılır ("Rohini · lordu Ay"); sütun eklenmez.** Gerekçe: yol haritası "nakshatra girişi satırlarına" der; yeni sütun `EventSearchPanel.test.tsx` ve `desk.spec.ts`'in var olan satırlarını değiştirirdi (ikisi de paylaşılan dosyadır). Yeni ay ve dolunay satırı olduğu gibidir.
- **D5 — "Ay'ın önümüzdeki nakshatra girişleri" olay aramasının hazır seçimidir: düğme tür seçimini nakshatra girişine, graha seçimini Ay'a çevirir ve aramayı başlatmaz.** Gerekçe: liste zaten olay aramasının verdiği listedir ve artık lordu taşır; ayrı komut ve ayrı liste aynı bilgiyi ikinci kez üretirdi. Kullanıcı aralığı seçip "Ara"ya basar. **Yol haritası farkı 2.**
- **D6 — Sankranti listesi her zodyak kipinde seçili ayanamsayla sidereal boylamdan kurulur; yöntem kimliği `sankranti.sidereal_v1`'dir. Harita tropikal burç kullanıyorsa panel bunu sözle söyler.** Gerekçe: yol haritası §4.8'in önerisi (K5'ten bilinçli ayrılma). Olay aramasının Güneş burç girişleri rasi çerçevesinde durmayı sürdürür.
- **D7 — Giriş anının tithisi, varası ve Ay nakshatrası v1'in `compute_panchang` hesabıdır: vara gün doğumunda değişir ve seçilen yerin gün doğumuna göre okunur. Gün doğumu olmayan günde liste `SunriseUnavailable`, yedek ephemeris kullanıldıysa `LowPrecision` uyarısı taşır.** Gerekçe: uygulamanın tek vara kuralı; ikinci bir kural (takvim günü) aynı an için iki ayrı vara gösterirdi.
- **D8 — Yıl, seçilen yerin takvim yılıdır (1 Ocak 00:00'dan sonraki 1 Ocak 00:00'a; yerin saat diliminde); yıl verilmezse yerin bugünkü yılıdır ve Rust'ta okunur. Yıl aralığı 1–2999'dur (`TOOLS_YEAR_RANGE`).** Gerekçe: "seçilen yılın on iki güneş girişi"; dört basamakla yazılabilen yıllar. Girişin kutlu süresi kapsam dışıdır.
- **D9 — `sankranti_moments` (yalnızca anlar ve burçlar) ayrı bir fonksiyondur ve kökten dışa aktarılır.** Gerekçe: Plan 29 (takvim) ve Plan 30 (mundane) girişin gününü işaretlemek için panchang hesabına gerek duymaz; küçük ve ucuz bir API yeter.
- **D10 — Sankranti görünümü ayrı bir paneldir (`sankranti`, 7 × 16; paletten). Yıl kutusu, "Yılı göster" ve "Bu yıl"; beş sütun (giriş anı, girilen burç, tithi, vara, Ay nakshatrası); satırdaki an transit tarihini o ana götürür.** Gerekçe: liste rasi çerçevesindeki olay aramasından başka bir çerçevededir ve panchang sütunları taşır; olay araması paneline sığdırmak iki çerçeveyi tek tabloda karıştırırdı.
- **D11 — Sankranti ve Efemeris panellerinin varsayılan yeri bulunulan yerdir (muhurta panelleriyle aynı seçici: bulunulan yer, haritanın doğum yeri, özel yer); yer seçilmemişse istek gitmez ve panel nedenini söyler.** Gerekçe: yol haritası §4.8 ve §4.9 "bulunulan yer"; var olan `PlaceControls` yeniden kullanılır.
- **D12 — Efemeris satırının anı, seçilen yerin o günkü yerel gece yarısıdır (duvar saatiyle 00:00); 00:00 yoksa (yaz saati gece yarısında başlıyorsa) günün başladığı andır. Yöntem kimliği `ephemeris_table.v1`'dir; 00:00 UT okuması uygulanmaz.** Gerekçe: yol haritası §4.9'un önerisi; Y5.
- **D13 — Tablo bir takvim ayıdır (28–31 satır). Burç ve derece rasi çerçevesindedir (haritanın zodyak kipini izler, K5), nakshatra ve pada sidereal boylamdandır; ayanamsa ve düğüm türü açık haritanın ayarıdır.** Gerekçe: transit tablosuyla aynı okuma; paneller açık harita olmadan çizilmediği için "harita açık değilken hangi ayar" sorusu doğmaz.
- **D14 — Giriş ve dönüş işaretleri ardışık iki satırın karşılaştırmasıdır: `ingress` girilen burcu, `station` dönülen yönü taşır (yalnızca `STATION_GRAHAS`); çekirdek `n + 1` an alır ve `n` satır döner. Kesin an olay aramasındadır.** Gerekçe: yol haritası "dönüş ve giriş günlerini işaretlemek"; satır başına bir `timeline` yürüyüşü ayda dokuz graha için gereksiz iştir. Gerçek motor testi işaretleri `timeline` dizilerinin sınırlarıyla üç ayda karşılaştırır.
- **D15 — Efemeris paneli `ephemeris` türüdür (12 × 22). Önceki ay, sonraki ay ve "Bu ay" düğmeleri Rust'ın döndürdüğü `prev`, `next` ve boş ayla çalışır; ay ve yıl kutusu doğrudan bir aya gider. "Nakshatra" kutusu panelin ayarıdır. İşaretler simge ve sözdür ("→ burç", "SR", "SD"; ekran okuyucuya tam cümle) ve anlamları tablonun altında yazılıdır.** Gerekçe: C4 (ay aritmetiği Rust'ta) ve C9 (renk tek başına anlam taşımaz).
- **D16 — Efemerisin günü, transit tarihini o günün başına götürür.** Gerekçe: Sankranti ve olay araması satırlarıyla aynı davranış.
- **D17 — Bütçe: bir yılın Sankranti listesi ve bir ayın efemeris tablosu, hata ayıklama derlemesinde ≤ 100 ms; testte `assert!` ile doğrulanır.** Ölçüm (yük altında): `sankranti_list` 7–8 ms, `ephemeris_month` 2–4 ms. Gerekçe: İM spec §15'in sınıfları; pay, yük altındaki dalgalanma içindir. Bütçe testi düşerse makine boşken yeniden koşulur; bütçe gevşetilmez.
- **D18 — El örnekleri Rust birim testleridir; `tests/golden/modules/` altına JSON eklenmez ve `modules_golden.rs` değişmez. Gerçek motor denetimleri iki yeni entegrasyon testindedir (`tests/sankranti.rs`, `tests/ephemeris_table.rs`: ayrı süreç, kendi motoru) ve bu testler arayüz fikstürlerini de üretir (`UPDATE_FIXTURES=1`).** Gerekçe: Plan 22–25'in yolu; `ui_fixture.rs`, `delhi.json` ve `fixture.ts` üç eşzamanlı planın ortak dosyası olurdu.
- **D19 — Yıl, ay, adım birimi, hız ve oynatma durumu oturumluktur; düzenle birlikte yalnızca yer seçimi ve "Nakshatra" kutusu saklanır.** Gerekçe: transit anı da oturumluktur (Plan 11 D8); olay aramasının tarih aralığı da saklanmaz.
- **D20 — Yeni hata kodu yoktur: `error.rs`, `commands.rs`, `errors.*` metinleri ve `i18n.test.ts` değişmez. Yeni girdi hataları `invalid_transit_input { field }` verir (alanlar: `range`, `place`, `year`, `month`, `moments`, `jd_ut`, `count`, `tz`); paneller alan adını arayüz dilinde yazar.** Gerekçe: yer, an ve dilim hataya girmez (İM spec §14).
- **D21 — Yeni görsel temel (PNG) ve yeni baskı düzeni yoktur (K10); var olan on beş temel değişmez. İki küçük stil dosyası eklenir (`panels/ephemeris.css`, `transit/stepper.css`); renk yalnızca token'lardan gelir.** Gerekçe: geçiş tablosu ve yeni paneller görsel temellerde yer almaz.
- **D22 — Adım birimleri saat, gün, hafta, ay ve yıldır; varsayılan gündür. Saat adımı mutlaktır (3 600 saniye); gün, hafta, ay ve yıl adımı tarayıcının saat dilimindeki duvar saatini korur.** Gerekçe: "yarın aynı saatte" beklentisi; transit tarih kutuları da tarayıcının dilimindedir.
- **D23 — Varılan ayda o gün yoksa ayın son günü alınır; duvar saati yaz saati boşluğuna düşerse boşluk kadar ileri, örtüşmeye düşerse önceki an alınır. Aralık 1–2999 yıllarıdır; dışına çıkan adım hata verir ve an değişmez.** Gerekçe: transit anının v1 kuralları (Plan 11 D7); sessizce başka bir güne kaymak yerine tanımlı bir kural.
- **D24 — Adım komutu tek bir istek yapısı alır (`TransitStepRequest`); varılan an depoya var olan `TransitAt::Jd` ile yazılır. Bir adım isteği sürerken yeni adım başlatılmaz; denetimler yalnızca Transit tablosu panelindedir.** Gerekçe: dizgi bağımsız değişken eklenmez (C2′ listesi değişmez); `TransitAt`, `store.tsx` ve `TransitDate.tsx` değişmez; halka ve tablo aynı depodan okur.
- **D25 — Oynatma yalnızca ileridir; üç hız: Yavaş 2 sn, Orta 1 sn (varsayılan), Hızlı 0,4 sn (iki adım arasındaki bekleme). Elle seçilen an oynatmayı durdurmaz; hata, başka harita, panelin kapanması ve "hareketi azalt" durdurur.** Gerekçe: en yavaş hız tabloyu okutur, en hızlısı motoru ve gözü yormaz; sahibin ekranda denetleyeceği seçimdir.
- **D26 — "Hareketi azalt" açıkken oynatma düğmesi ve hız seçici çizilmez; yerinde nedeni yazan bir satır durur.** Gerekçe: yol haritası §4.10 "oynatma yerine yalnızca adım"; gizlenen denetimin nedeni görünür olmalıdır.
- **D27 — Oynatma düğmesinin adı durumu söyler ("Oynat" ya da "Duraklat"); `aria-pressed` kullanılmaz.** Gerekçe: değişen ad ile basılı durumun birlikte okunması çelişir.
- **D28 — Adlandırma: bu planın getirdiği adların hiçbiri yasak diziyi içermez. Türkçe metinlerde "durur", "öteki", "sürer", "düşer", "geri" yazılır; `-ka` ile biten sözcükler çoğul yazılmaz (süre birimleri "dk", "sn").** Gerekçe: v1 spec §10, İM spec Ek A, yol haritası Y8.

## Global Constraints

- Lisans: AGPL-3.0-or-later. Yeni bağımlılık yok: `Cargo.toml`, `Cargo.lock`, `deny.toml`, `app/package.json`, `app/pnpm-lock.yaml`, `app/vite.config.ts` ve `app/playwright.config.ts` değişmez.
- **Yasaklı ad kuralı (v1 spec §10; İM spec Ek A; Y8):** o maddedeki dört ad ve türevleri hiçbir isimde, yorumda, metinde, dosya adında ya da commit mesajında kullanılmaz; kural harf dizisi düzeyindedir (D28). Her commit'ten önce eklenen satırlar şu komutla taranır ve çıktı boş olmalıdır: `git diff --cached -U0 -- . ':!app/ui/test/fixtures' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'`.
- **Clean-room ve yorum metni yok:** kurallar klasik tanımdan özgün olarak uygulanır; başka ürünün adı, düzeni, metni ya da çıktısı kullanılmaz. Listeler ve tablolar olgudur: hüküm, sıralama ve açıklama metni eklenmez. **Yürüten ajan sonuçları başka bir yazılımla ya da çevrimiçi kaynakla karşılaştırmaz** (K9).
- **Çerçeveler (K4, K5; D6, D13):** Sankranti her kipte sidereal; efemerisin burcu ve derecesi rasi çerçevesinde; nakshatra, pada ve tithi sidereal boylamdan.
- **Gizlilik (İM spec §14):** yer, an, saat dilimi ve hesap ayarı hiçbir yerde loglanmaz; hatalar yalnızca alan adını taşır; hiçbir modül ağ isteği yapmaz; testler ve fikstürler yalnızca deneme verisi kullanır.
- **Yeniden okuma:** başka bir yıl ya da ay okunurken tablo DOM'dan çıkarılıp yeniden kurulmaz (`useLastData`) ve odak gövdeye düşmez; hatadan sonraki okuma hata öncesi veriyi göstermez. Liste ve tablo panel açılmadan istenmez.
- **El örnekleri (K9):** beklenen değer kuraldan elle türetilir ve test yorumunda yazılıdır. Bir el örneği motorla uyuşmazsa önce kural yeniden okunur; **beklenen değer motor çıktısına bakarak düzeltilmez.** (Bu plan yazılırken çekirdeğin 17 testi ve adım aritmetiğinin takvim örnekleri ilk koşuda geçti. Üç düzeltme test yazımındandı: yaz saati sınırındaki satırın ofseti sınır anında kayan noktayla belirsizdir ve komşu günlerden okunur; sahte zamanlayıcıda anın yerleşmesi aralıktan önce işlenir; adım sayısının `abs`'i `i32::MIN`'de taşıyordu ve test bunu buldu, kod düzeltildi.)
- Kapılar: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `pnpm --dir app test`, `pnpm --dir app build` her görev sonunda yeşil; e2e Task 1, 3, 5, 6, 7 ve 8'de. `cargo test --workspace` çıktısı uzun olduğu için başarısızlık şu sayaçla denetlenir ve `0` yazmalıdır: `cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."`.
- **Toolchain:** `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"` (oturum başına bir kez). Node 24+, pnpm 11.8.0. Depo kısa bir yolda durmalıdır (Swiss Ephemeris 242 bayttan uzun ephemeris yolunu reddeder).
- **Gerçek uygulama çalıştırılmaz:** `pnpm tauri dev` hiçbir adımda koşulmaz; üretilen `.app` çalıştırılmaz; `~/Library/Application Support/io.github.turgaydogusan.jyotishdesk` dizinine dokunulmaz. `pkill -f` kullanılmaz.
- **E2e özel portta koşar: 14326.** Port 1421 eşzamanlı çalışma ağaçlarınındır ve `playwright.config.ts` var olan sunucuyu yeniden kullanır: 1421'de koşan bir test başka ağacın arayüzünü sınayabilir. Task 1 Step 1 `app/playwright.port.config.ts` dosyasını kurar (izlenmez; yerel dışlama listesindedir; Task 8 siler) ve bütün e2e komutları `-c playwright.port.config.ts` ile koşar. Her koşudan önce `lsof -i :14326` çıktısız olmalıdır. `pnpm --dir app e2e` ve `e2e:visual` betikleri (1421) bu planda kullanılmaz.
- **Tabanlar (`9727c55`):** `jyotish-core` lib 466, `avastha_golden` 5, golden 8, `modules_golden` 17; `atlas` `build` 6, `moment_fixed_lmt` 11, `moment_iana` 13, `search` 8, `shipped` 3; `store` lib 122 (+1 `#[ignore]`); `jyotish-desk` lib 58, `avastha` 1, `bhava` 1, `bindings` 1, `compute_chart` 1, `modules` 1, `natal_extras` 1, `open_client` 1, `ui_fixture` 2; Vitest `97 passed (97)`, `973 passed (973)`; e2e 33; görsel 15 (yerel, darwin); yayın betiği testleri 14.
- **Sayaç zinciri** (her satır görev bitince; boş hücre "önceki satırla aynı").

  | Görev | `jyotish-core` lib | `jyotish-desk` lib | `jyotish-desk` yeni testler | Vitest dosya / test | e2e |
  |---|---|---|---|---|---|
  | Taban | 466 | 58 | (yok) | 97 / 973 | 33 |
  | Task 1 | 469 | | | 100 / 986 | |
  | Task 2 | 476 | | `sankranti` 1 | | |
  | Task 3 | | | | 101 / 996 | 34 |
  | Task 4 | 483 | 59 | `ephemeris_table` 1 | | |
  | Task 5 | | | | 102 / 1006 | 35 |
  | Task 6 | | 64 | | 103 / 1017 | |
  | Task 7 | | | | 104 / 1028 | 37 |

  Değişmeyenler: `avastha_golden` 5, golden 8, `modules_golden` 17; `atlas`, `ephemeris` ve `store` testleri; `jyotish-desk` `avastha`, `bhava`, `bindings`, `compute_chart`, `modules`, `natal_extras`, `open_client`, `ui_fixture`; görsel 15; yayın betiği testleri 14. `jyotish-core` lib 17 test kazanır (`lords.rs` 3, `sankranti.rs` 7, `ephemeris_table.rs` 7); `jyotish-desk` lib 6 (`ephemeris.rs` 1, `step.rs` 5). Vitest yedi dosya ve 55 test kazanır: `nakshatraLord.test.ts` 3, `GrahaTableNakshatraLord.test.tsx` 5, `EventSearchLords.test.tsx` 4, `TransitTablePanel.test.tsx` +1, `SankrantiPanel.test.tsx` 10, `EphemerisPanel.test.tsx` 10, `TransitStepper.test.tsx` 11, `TransitPlayback.test.tsx` 11.
- **Taşınan kurallar (Plan 09 C1–C14, Plan 10 C2′, C15–C17):**
  - **C1** Gizlilik maddesi; `no-console.test.ts` durur. Yeni `CommandError` kodu yoktur (D20).
  - **C2′** Hiçbir IPC komutu arayüzden dosya yolu ya da adres almaz. Bu planın üç komutu dizgi bağımsız değişken almaz (saat dilimi istek yapısının alanıdır): `api.test.ts`'in dizgi bağımsız değişken listesi değişmez.
  - **C3** Şema sürümleri değişmez; `crates/store` hiç değişmez (Y6).
  - **C4** Arayüz astrolojik hesap ve tarih hesabı yapmaz: lord tablosu, giriş anları, panchang olguları, konumlar, işaretler, "bu yıl", "bu ay", önceki ve sonraki ay ile adımın vardığı an Rust'tan gelir. Tablodan satır seçmek, kimliği adına çevirmek, dereceyi ve tarihi biçimlemek (`Intl`), yıl kutusunun metnini sayıya çevirmek ve zamanlayıcı kurmak gösterimdir.
  - **C5/C17** Otomatik kayıtlar değişmez; panel ayarları var olan yoldan kaydedilir. **C6** Düzen ayrıştırıcıları değişmez; iki panel türü kataloğa eklenir.
  - **C7/C16** Yeni kısayol yoktur. **C8** Yeni pencere yoktur.
  - **C9** Renk yalnızca token'lardan gelir ve tek başına anlam taşımaz.
  - **C10** Üretilen dosyalar elle düzenlenmez; birleştirme çakışmasında da yeniden üretilir.
  - **C11** Görsel PNG temelleri yereldir; bu planda değişmezler (D21).
  - **C12** Her yeni i18n anahtarı üç dilde vardır; Fransızca metinler tipografik kesme işareti (’) kullanır.
  - **C13** Yasaklı ad taraması her commit'te; ilerleme notları, port dosyası ve `.superpowers/` hiçbir zaman `git add` edilmez.
  - **C14** `compute_panchang` ve gün doğumu toleransı değişmez. **C15** `alert`, `confirm`, `prompt`, `window.print()` ve `window.open()` çağrılmaz.
- **Erişilebilirlik:** WCAG AA. Tablolar `caption`, `scope="col"` ve `scope="row"` taşır; geniş tabloların kaydırma kabı klavye odağı alır (`BalaScroll`); kısaltmayla yazılan graha tam adıyla okunur; işaretler sözle okunur; hata `role="alert"`'tir; kullanılamayan düğme yerel `disabled` değildir (odakta durur, `aria-disabled` taşır); her işlem klavyeyle yapılır; "hareketi azalt" gözetilir.
- Commit mesajları İngilizcedir ve tam olarak `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` satırıyla biter. Her görev tek commit'tir (gerekirse inceleme düzeltmesi ayrı commit). `git push`, `git tag`, yıkıcı git komutu ve `.superpowers/` commit'i yoktur.
- Yeni bir Rust dosyasının testleri ve uygulaması ayrı adımlarda verilir: önce dosya yalnızca `#[cfg(test)] mod tests` bloğuyla oluşturulur, sonra uygulama **o bloğun üstüne**, arada bir boş satırla eklenir; `cargo fmt --all --check` birleştirmeyi doğrular.

## Paylaşılan dosyalar

Plan 27 ve Plan 29 aynı tabandan, ayrı çalışma ağaçlarında yürür. Aşağıdaki on bir dosyaya (ve üretilen `bindings.ts`'e) onlar da dokunabilir. Bu plandaki her değişiklik **yalnızca ektir**: tabana göre hiçbir satır silinmez ya da yeniden sarılmaz (tek istisna `model.test.ts`'in tek satırlık listesi). Task 8 Step 3 bunu sayaçla doğrular. Çapalar, öteki planların büyük olasılıkla ekleyeceği yerlerden (listelerin sonu, dosya sonu) bilerek uzak seçilmiştir.

| Dosya | Bu planın eki | Çapa | Görev |
|---|---|---|---|
| `crates/jyotish-core/src/lib.rs` | ayrı blok: yorum, `pub mod transit_tools;`, tek `pub use transit_tools::{…}` | dosyanın sonu (`pub use zodiac::{…};` satırından sonra, boş satırla) | 1, 2, 4 |
| `app/src-tauri/src/lib.rs` | `pub mod transit_tools_commands;`; `.constant("NAKSHATRA_LORDS", …)` (yorumuyla iki satır); komut listesinde üç satır | `mod`: `transit_commands`'tan sonra; sabit: `.disable_serde_phases()` satırından sonra; komutlar: `transit_commands::transits_at,` satırından sonra | 1, 2, 4, 6 |
| `app/ui/bindings.ts` | **üretilir** (C10): bir sabit, üç komut, on bir tip | elle birleştirilmez | 1, 2, 4, 6 |
| `app/ui/api.ts` | iki tip içe aktarımı (`MonthId`, `TransitStepRequest`); üç sarmalayıcı | sarmalayıcılar `transitsAt` satırından sonra | 2, 4, 6 |
| `app/ui/api.test.ts` | taklit listesinde ve çağrı tablosunda üçer satır | `transitsAt` satırlarından sonra | 2, 4, 6 |
| `app/ui/layout/model.ts` | `PANEL_KINDS`'e `"sankranti"`, `"ephemeris"`; `MODULE_PANELS`'e iki satır ve bir yorum | `event_search` satırlarından sonra (listelerin sonu değil) | 3, 5 |
| `app/ui/layout/model.test.ts` | iki listede aynı iki tür | `event_search`'ten sonra; **tek satırlık `MODULE_PANELS` beklentisi her planın değiştirdiği satırdır** | 3, 5 |
| `app/ui/panels/registry.ts`, `panels.test.tsx` | iki içe aktarım; iki kayıt; iki beklenti | `event_search` satırlarından sonra | 3, 5 |
| `app/ui/i18n/tr.ts`, `en.ts`, `fr.ts` | `panel.sankranti`, `panel.ephemeris`; `table.nakshatraLord`; `eventSearch.lordOf`, `eventSearch.moonPreset`; `transit.nakshatraLord`; üç yeni üst bölüm (`sankranti`, `ephemeris`, `transitStep`) | `panel.*`: `transit_table` satırından sonra; `table`: `baladi`'den sonra; `eventSearch`: `method`'dan sonra; `transit`: `caption`'dan sonra; üst bölümler: üst düzey `transit` bölümünden hemen önce | 1, 3, 5, 6, 7 |
| `app/e2e/ipc.ts` | `FIXTURE.transit_tools` (üç satır); üç komutun taklidi | fikstür: `export const FIXTURE` satırından sonra; taklitler: `transits_at` satırından sonra | 3, 5, 6 |
| `tests/golden/modules/README.md` | bir paragraf | "Sonraki alt projeler…" satırından hemen önce | 8 |

Paylaşılmayan ama var olan dört dosya da değişir: `app/ui/transit/TransitTablePanel.tsx` (+test), `app/ui/panels/GrahaTablePanel.tsx`, `app/ui/panels/EventSearchPanel.tsx`. Bilerek **dokunulmayanlar:** `crates/jyotish-core/src/graha.rs`, `chart.rs`, `transit.rs`, `timeline.rs`, `panchang.rs`, `error.rs`, `search/`, `muhurta/`, `natal_extras/`; `crates/jyotish-core/tests/`; `app/src-tauri/src/commands.rs`, `transit_commands.rs`, `muhurta_commands.rs`, `search_commands.rs`; `app/src-tauri/tests/ui_fixture.rs`; `app/ui/test/fixtures/delhi.json`, `app/ui/test/fixture.ts`; `app/ui/transit/store.tsx`, `TransitDate.tsx`, `transit.css`; `app/ui/panels/panels.css`, `EventSearchPanel.test.tsx`, `GrahaTablePanel.test.tsx`, `grahaColumns.ts`; `app/ui/i18n/i18n.test.ts`; `app/ui/App.test.tsx`; `app/e2e/desk.spec.ts`, `visual.spec.ts`; `app/playwright.config.ts`; `crates/store`.

**Birleştirme kuralı.** (1) Çakışan her ek listesinde iki taraf da tutulur; sıra plan numarasıdır (26, 27, 29). (2) `bindings.ts` elle birleştirilmez: taraflardan biri alınır ve `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings` ile yeniden üretilir. Bu planın iki fikstürü yalnızca bu planındır; yine de birleştirmeden sonra `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test sankranti --test ephemeris_table` ile yeniden üretilip farkın boş olduğu görülür. (3) `model.test.ts`'in iki listesi `model.ts`'teki sırayla yeniden yazılır. (4) Birleştirmeden sonra bütün kapılar yeniden koşulur; sayaçlar planların farklarının toplamıdır. (5) Öteki plan `SkyEvent`, `TransitGraha` ya da `GrahaPosition`'a alan eklediyse bu planın kodu derlenmeyi sürdürür (bu plan o tiplerin örneğini elle kurmaz; tek istisna `EventSearchLords.test.tsx`'in `row` yardımcısıdır ve fikstür satırını yayarak kurar).

## Sonraki işlere verilen API

Bu planın ürettiği ve sonraki işlerin tüketeceği adlar. Yıldızlı Rust adları `jyotish_core` kökünden dışa aktarılır; imzalar görevlerde tam olarak verilir. **Plan 29 bu planla aynı tabandan yazıldığı için bu adları ancak birleşmeden sonra kullanabilir; API bilerek küçük tutulmuştur.**

| API | Tüketen | Not |
|---|---|---|
| `NAKSHATRA_LORDS`\* ve arayüzde `nakshatraLord(n)` (`panels/nakshatraLord.ts`) | Plan 29 (takvim hücresinde Ay nakshatrasının lordu), Plan 30, Plan 31 | Tip değişikliği gerektirmez: nakshatra numarası taşıyan her yüzey lordu okur |
| `sankranti_moments(eph, settings, from, to) -> Vec<(f64, Sign)>`\* | Plan 29 (ay ızgarasında Sankranti günü), Plan 30 (girişten mundane harita) | Yalnızca anlar ve burçlar; panchang hesaplamaz; aralık ≤ 400 gün; her kipte sidereal |
| `sankranti_list`\*, `Sankranti`\*, `SankrantiList`\*, `SANKRANTI_METHOD`\* ve `sankranti_list_blocking` | Plan 30 ("olayı harita olarak aç"), AI paneli | Giriş anının tithisi, varası, Ay nakshatrası |
| `ephemeris_table(eph, settings, moments) -> EphemerisTable`\*, `EphemerisRow`\*, `EphemerisCell`\* | Plan 29 (isterse günün başındaki konumlar ve o günün giriş/dönüş işaretleri) | Anları çağıran verir (`n + 1` an → `n` satır); takvim kendi gün sınırını seçebilir |
| `transit_tools_commands::ephemeris::MonthId` (`prev`, `next`) ve `TOOLS_YEAR_RANGE` | Plan 29 (ay gezinmesi) | Ay aritmetiği Rust'ta; arayüz `prev` ve `next`'i geri gönderir |
| `transit_step` komutu, `useTransitStepper`, `stepBase` | Plan 29 (takvimden güne gitme), Plan 31, Plan 32 | Duvar saatini koruyan adım; depo `setAt({ Jd })` ile güncellenir |
| Arayüz: `parseYear`, `YEAR_MIN`, `YEAR_MAX`; `formatMonth`, `formatDay` | Plan 29 | Yıl kutusu ve ay/gün biçimlemesi |

Plan 29'un bu plana bağlı olmayan yolu da açıktır: takvim, olay aramasının `SignIngress` (Güneş) olaylarını kullanabilir; yalnız o olaylar rasi çerçevesindedir ve tropikal-rasi kipinde Sankranti değildir. Birleşmeden sonra `sankranti_moments`'e geçmek tek fonksiyon çağrısıdır.

## Dosya Yapısı (bu plan)

```
crates/jyotish-core/
  src/transit_tools/mod.rs              modül kaydı ve dışa aktarımlar (yeni)
  src/transit_tools/lords.rs            NAKSHATRA_LORDS; el örnekleri (yeni)
  src/transit_tools/sankranti.rs        sidereal güneş girişleri, giriş anının panchangı (yeni)
  src/transit_tools/ephemeris_table.rs  anların dokuz grahası, giriş ve dönüş işaretleri (yeni)
  src/lib.rs                            sondaki ayrı blok (ek)
app/src-tauri/
  src/transit_tools_commands/mod.rs       modül kaydı, yıl aralığı, hata yardımcısı (yeni)
  src/transit_tools_commands/sankranti.rs sankranti_list (yeni)
  src/transit_tools_commands/ephemeris.rs ephemeris_month, MonthId (yeni)
  src/transit_tools_commands/step.rs      transit_step; takvim el örnekleri (yeni)
  src/lib.rs                              modül, sabit ve üç komut (ek)
  tests/sankranti.rs                      gerçek motor, bütçe, fikstür (yeni)
  tests/ephemeris_table.rs                gerçek motor, bütçe, fikstür (yeni)
app/ui/
  bindings.ts                             üretilir
  test/fixtures/sankranti.json, ephemeris.json   üretilir (yeni)
  test/sankrantiFixture.ts, ephemerisFixture.ts  fikstür yükleyicileri (yeni)
  api.ts, api.test.ts                     üç sarmalayıcı (ek)
  modules/useSankranti.ts, useEphemeris.ts       kancalar (yeni)
  panels/nakshatraLord.ts (+test)         Rust tablosundan lord (yeni)
  panels/SankrantiPanel.tsx (+test)       (yeni)
  panels/EphemerisPanel.tsx (+test), ephemeris.css   (yeni)
  panels/GrahaTableNakshatraLord.test.tsx, EventSearchLords.test.tsx   (yeni)
  panels/GrahaTablePanel.tsx, EventSearchPanel.tsx   lord sütunu; lord ve hazır seçim
  panels/registry.ts, panels.test.tsx     iki tür (ek)
  layout/model.ts (+test)                 sankranti ve ephemeris türleri (ek)
  transit/TransitTablePanel.tsx (+test)   lord sütunu; adım denetimleri
  transit/useTransitStepper.ts, TransitStepper.tsx (+test), stepper.css   (yeni)
  transit/useTransitPlayback.ts, TransitPlayback.tsx (+test)              (yeni)
  i18n/{tr,en,fr}.ts                      metinler (ek)
app/e2e/
  ipc.ts                                  fikstürler ve üç komutun taklidi (ek)
  sankranti.desk.spec.ts, ephemeris.desk.spec.ts, transit-step.desk.spec.ts   (yeni)
tests/golden/modules/transit_tools/REVIEW.md   okuma kuralları, onay ve elle denetim kaydı (yeni)
tests/golden/modules/README.md                 bir paragraf (ek)
```

### Kapsam

| Yol haritası maddesi | Yer |
|---|---|
| §4.7 Geçiş tablosuna ve graha tablosuna nakshatra lordu sütunu | Task 1 (D2, D3) |
| §4.7 Olay aramasının nakshatra girişi satırlarına lord; Ay'ın önümüzdeki nakshatra girişleri | Task 1 (D4, D5) |
| §4.7 Lord arayüzde türetilmez (C4) | Task 1 (D2: Rust tablosu, bağlama sabiti) |
| §4.8 Yılın on iki güneş girişi: yerel saat, girilen burç, tithi, vara, Ay nakshatrası | Task 2 (D6–D9), Task 3 |
| §4.8 Tropikal haritada da sidereal; kutlu süre kapsam dışı | Task 2 (D6), Task 3 (not) |
| §4.8 Görünüm: hazır seçim ya da ayrı panel | Task 3 (D10: ayrı panel) |
| §4.9 Ay için gün gün dokuz graha; isteğe bağlı nakshatra; sabit saat | Task 4 (D12, D13), Task 5 |
| §4.9 Dönüş ve giriş günlerinin işaretlenmesi | Task 4 (D14), Task 5 (D15) |
| §4.9 Geniş tablonun erişilebilirliği ve yatay kaydırma; CSV ve baskı yok | Task 5 (D15, D21) |
| §4.10 Adım düğmeleri (saat, gün, hafta, ay, yıl); adım aritmetiği Rust'ta | Task 6 (D22–D24) |
| §4.10 Oynat-durdur; eski yanıtın düşürülmesi; harita açmanın beklememesi; azaltılmış hareket; panel kapanınca zamanlayıcının durması | Task 6 (düşen yanıt), Task 7 (D25–D27) |
| §5 (8) Kapılar ve elle denetim; "Sahibin denetimi" | Task 8 |
| §8 Tek ephemeris iş parçacığı: ayrı testler | Task 6, Task 7 (sahte zamanlayıcı testleri), Task 4 (bütçe) |
| K12 Paletten ekleme; uçtan uca akışlar | Task 3, 5, 7 |


---

### Task 1: Tabanlar; nakshatra lordları tablosu ve üç yüzeyde lord (geçiş tablosu, graha tablosu, olay araması)

Yol haritası §4.7 ve §5 "Plan 26" görev taslağının (1). maddesi; D1–D5. Var olan hiçbir Rust tipi değişmez (`graha.rs`, `chart.rs`, `transit.rs`, `search/events.rs` dokunulmaz): tablo çekirdekte sabittir, bağlama dosyasına sabit olarak çıkar ve arayüz yalnızca satır seçer.

**Files:**
- Create: `crates/jyotish-core/src/transit_tools/mod.rs`, `crates/jyotish-core/src/transit_tools/lords.rs`, `app/ui/panels/nakshatraLord.ts`, `app/ui/panels/nakshatraLord.test.ts`, `app/ui/panels/GrahaTableNakshatraLord.test.tsx`, `app/ui/panels/EventSearchLords.test.tsx`
- Modify: `crates/jyotish-core/src/lib.rs`, `app/src-tauri/src/lib.rs`, `app/ui/bindings.ts` (üretilir), `app/ui/transit/TransitTablePanel.tsx`, `app/ui/transit/TransitTablePanel.test.tsx`, `app/ui/panels/GrahaTablePanel.tsx`, `app/ui/panels/EventSearchPanel.tsx`, `app/ui/i18n/tr.ts`, `en.ts`, `fr.ts`
- Test: `lords.rs` `mod tests` (3); `nakshatraLord.test.ts` (3); `GrahaTableNakshatraLord.test.tsx` (5); `EventSearchLords.test.tsx` (4); `TransitTablePanel.test.tsx` (+1)

**Interfaces:**
- Consumes (hepsi var): `graha::Graha`; testte `dasha::{birth_dasha, VIMSHOTTARI_ORDER}`, `natal_extras::nakshatra_lord`, `zodiac::NAKSHATRA_SPAN_DEG`; `tauri_specta::Builder::constant`.
- Produces:
  - Rust: `jyotish_core::NAKSHATRA_LORDS: [Graha; 27]` (`NAKSHATRA_LORDS[n - 1]`, n. nakshatranın lordu).
  - Bağlama: `export const NAKSHATRA_LORDS = [...] as const` (`bindings.ts`).
  - Arayüz: `nakshatraLord(n: number | null): Graha | null` (`panels/nakshatraLord.ts`); panel ayarı `graha_table.settings.nakshatraLord: boolean`; i18n `transit.nakshatraLord`, `table.nakshatraLord`, `eventSearch.lordOf`, `eventSearch.moonPreset`.

- [ ] **Step 1: Tabanları ölç ve özel e2e portunu kur**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
git log -1 --format=%h
git status --short
rustc --version
cargo test --workspace 2>&1 | grep -E "^test result|Running"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
cat > app/playwright.port.config.ts <<'EOF'
// Geçici: e2e'yi özel bir portta koşturur (1421 başka çalışma ağaçlarınındır). Commit edilmez.
import base from "./playwright.config";

const PORT = 14326;
export default {
  ...base,
  use: { ...base.use, baseURL: `http://localhost:${PORT}` },
  webServer: { command: `pnpm exec vite --port ${PORT} --strictPort`, url: `http://localhost:${PORT}`, reuseExistingServer: false },
};
EOF
echo "app/playwright.port.config.ts" >> "$(git rev-parse --git-common-dir)/info/exclude"
lsof -i :14326
pnpm --dir app exec playwright test --project=e2e -c playwright.port.config.ts 2>&1 | tail -1
rm -rf app/test-results
git status --short
```
Expected: ilk `git status --short` boş; `rustc 1.100.0` ya da yenisi; `jyotish-core` lib `466 passed`, `avastha_golden` `5 passed`, golden `8 passed`, `modules_golden` `17 passed`; `atlas` `build` 6, `moment_fixed_lmt` 11, `moment_iana` 13, `search` 8, `shipped` 3; `store` lib `122 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `58 passed`, `avastha` 1, `bhava` 1, `bindings` 1, `compute_chart` 1, `modules` 1, `natal_extras` 1, `open_client` 1, `ui_fixture` 2; Vitest `97 passed (97)`, `973 passed (973)`; `lsof` çıktısız (14326 boş); e2e `33 passed`; son `git status --short` boş (port dosyası yerel dışlama listesindedir ve hiçbir zaman `git add` edilmez). Sayılar farklıysa (`main` ilerlediyse) ölçülen değerler ilerleme notlarına yazılır ve sayaç zinciri o tabandan, aynı farklarla yürütülür.

- [ ] **Step 2: Modül kaydı ve çekirdek testleri: tablonun biçimi, el örnekleri ve iki ikinci yazım**

`lib.rs`'teki blok bilerek dosyanın sonundadır ve boş satırla ayrılmıştır (D1, "Paylaşılan dosyalar"); alfabetik `pub mod` grubuna taşınmaz.

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 612a167..7a978ca 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -163,3 +163,8 @@ pub use yogas::{
     YOGA_GRAHAS,
 };
 pub use zodiac::{angular_distance, deg_in_sign, nakshatra_pada, normalize, Sign};
+
+// Geçiş araçları (yol haritası Plan 26). Ayrı blok, dosyanın sonunda: eşzamanlı planların
+// alfabetik gruplardaki ve dosya başındaki ekleriyle aynı satırlara dokunmaz.
+pub mod transit_tools;
+pub use transit_tools::NAKSHATRA_LORDS;
````

**Yeni dosya:** `crates/jyotish-core/src/transit_tools/mod.rs`

````rust
//! Geçiş araçları (yol haritası Plan 26; §4.7–§4.9): nakshatra lordları tablosu, Sankranti
//! listesi ve efemeris tablosu. Hepsi var olan konum hesabının yeni görünümüdür: yeni astronomi
//! yoktur. Sonuçlar `Chart`'a eklenmez ve saklanmaz; panel açıkken istenir (K1).

pub mod lords;

pub use lords::NAKSHATRA_LORDS;
````

**Yeni dosya:** `crates/jyotish-core/src/transit_tools/lords.rs` (şimdilik yalnızca test bloğu)

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dasha::{birth_dasha, VIMSHOTTARI_ORDER};
    use crate::natal_extras::nakshatra_lord;
    use crate::zodiac::NAKSHATRA_SPAN_DEG;

    #[test]
    fn the_table_is_the_vimshottari_order_three_times() {
        assert_eq!(NAKSHATRA_LORDS.len(), 27);
        for (i, lord) in NAKSHATRA_LORDS.iter().enumerate() {
            assert_eq!(*lord, VIMSHOTTARI_ORDER[i % 9], "nakshatra {}", i + 1);
        }
        // Her graha üç nakshatranın lordudur.
        for graha in Graha::ALL {
            let n = NAKSHATRA_LORDS.iter().filter(|l| **l == graha).count();
            assert_eq!(n, 3, "{graha:?}");
        }
    }

    #[test]
    fn hand_examples() {
        // 1 Ashwini Ketu; 4 Rohini Ay; 8 Pushya Satürn; 10 Magha Ketu; 14 Chitra Mars;
        // 18 Jyeshtha Merkür; 19 Mula Ketu; 22 Shravana Ay; 27 Revati Merkür.
        let expected = [
            (1, Graha::Ketu),
            (4, Graha::Moon),
            (8, Graha::Saturn),
            (10, Graha::Ketu),
            (14, Graha::Mars),
            (18, Graha::Mercury),
            (19, Graha::Ketu),
            (22, Graha::Moon),
            (27, Graha::Mercury),
        ];
        for (nakshatra, lord) in expected {
            assert_eq!(
                NAKSHATRA_LORDS[nakshatra - 1],
                lord,
                "nakshatra {nakshatra}"
            );
        }
    }

    #[test]
    fn the_table_agrees_with_the_function_and_the_dasha_seed() {
        for n in 1..=27_u8 {
            let lord = NAKSHATRA_LORDS[usize::from(n) - 1];
            // İkinci yazım: hesap yapan kodun kullandığı fonksiyon.
            assert_eq!(lord, nakshatra_lord(n), "nakshatra {n}");
            // Üçüncü yazım: nakshatranın ortasındaki bir Ay'ın doğum dasası.
            let mid = (f64::from(n) - 0.5) * NAKSHATRA_SPAN_DEG;
            assert_eq!(lord, birth_dasha(mid).0, "nakshatra {n}");
        }
    }
}
````

- [ ] **Step 3: Testlerin derlenmediğini gör**

```bash
cargo test -p jyotish-core --lib transit_tools 2>&1 | grep -E "^error" | head -3
```
Expected: `error[E0432]` ya da `error[E0425]` (`NAKSHATRA_LORDS` yok).

- [ ] **Step 4: Tablo**

**`crates/jyotish-core/src/transit_tools/lords.rs`: uygulama, test bloğunun üstüne (arada bir boş satır)**

````rust
//! Nakshatra lordları tablosu (yol haritası §4.7).
//!
//! Yirmi yedi nakshatranın Vimshottari lordu, nakshatra sırasıyla: Ashwini Ketu, Bharani Venüs,
//! Krittika Güneş … dokuzlu sıra üç kez yinelenir. Tablo, arayüze bağlama dosyasıyla sabit
//! olarak çıkar: geçiş tablosu, graha tablosu ve olay araması nakshatra numarasının lordunu
//! buradan okur (C4: tablo Rust'ındır, arayüz yalnızca satırı seçer). Hesap yapan Rust kodu
//! `natal_extras::nakshatra_lord`'u çağırmayı sürdürür; testler ikisinin aynı olduğunu sabitler.

use crate::graha::Graha;

/// `NAKSHATRA_LORDS[n - 1]`, n. nakshatranın (1..=27) lordudur.
pub const NAKSHATRA_LORDS: [Graha; 27] = [
    Graha::Ketu,
    Graha::Venus,
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Rahu,
    Graha::Jupiter,
    Graha::Saturn,
    Graha::Mercury,
    Graha::Ketu,
    Graha::Venus,
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Rahu,
    Graha::Jupiter,
    Graha::Saturn,
    Graha::Mercury,
    Graha::Ketu,
    Graha::Venus,
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Rahu,
    Graha::Jupiter,
    Graha::Saturn,
    Graha::Mercury,
];
````

```bash
cargo fmt --all --check
cargo test -p jyotish-core --lib transit_tools 2>&1 | grep -E "^test result"
```
Expected: fmt çıktısız; `3 passed`.

- [ ] **Step 5: Tabloyu bağlama dosyasına sabit olarak çıkar (C10)**

````diff
diff --git a/app/src-tauri/src/lib.rs b/app/src-tauri/src/lib.rs
index e704350..7538e67 100644
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -28,6 +28,8 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
     tauri_specta::Builder::<tauri::Wry>::new()
         .error_handling(tauri_specta::ErrorHandlingMode::Result)
         .disable_serde_phases()
+        // Nakshatra lordları tablosu (Plan 26): arayüz lordu bu sabitten okur, türetmez (C4).
+        .constant("NAKSHATRA_LORDS", jyotish_core::NAKSHATRA_LORDS)
         .commands(tauri_specta::collect_commands![
             commands::engine_status,
             commands::compute_chart,
````

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
grep -c '^export const NAKSHATRA_LORDS = \["Ketu","Venus","Sun","Moon","Mars","Rahu","Jupiter","Saturn","Mercury","Ketu"' app/ui/bindings.ts
```
Expected: iki kez `1 passed`; `1`. `bindings.ts` elle düzenlenmez.

- [ ] **Step 6: Arayüz testleri: sabit, üç yüzey ve hazır seçim**

Beklenen lordlar kuraldan elle türetilmiştir (n. nakshatranın lordu dokuzlu sıranın ((n − 1) mod 9). adıdır); türetme test yorumlarındadır. Üç yeni test dosyası ayrıdır: `GrahaTablePanel.test.tsx` ve `EventSearchPanel.test.tsx` değişmez.

**Yeni dosya:** `app/ui/panels/nakshatraLord.test.ts`

````ts
import { expect, test } from "vitest";
import { NAKSHATRA_LORDS } from "../bindings";
import { fixture } from "../test/fixture";
import { nakshatraLord } from "./nakshatraLord";

test("the lord is read from the Rust table: nine lords, three times, in the Vimshottari order", () => {
  expect(NAKSHATRA_LORDS).toHaveLength(27);
  expect(NAKSHATRA_LORDS.slice(0, 9)).toEqual(["Ketu", "Venus", "Sun", "Moon", "Mars", "Rahu", "Jupiter", "Saturn", "Mercury"]);
  for (let n = 1; n <= 27; n++) expect(nakshatraLord(n)).toBe(NAKSHATRA_LORDS[(n - 1) % 9]);
  expect([nakshatraLord(1), nakshatraLord(4), nakshatraLord(27)]).toEqual(["Ketu", "Moon", "Mercury"]);
});

test("a number outside 1..27 has no lord", () => {
  for (const n of [0, 28, -1, 1.5, Number.NaN, null]) expect(nakshatraLord(n)).toBeNull();
});

test("the table agrees with the lords Rust wrote into the fixture", () => {
  // Navatara satırları ve bhava noktaları lordu Rust'tan taşır: aynı tablo, iki ayrı yoldan.
  for (const row of fixture().natal_extras.navatara.rows) expect(nakshatraLord(row.nakshatra)).toBe(row.lord);
  for (const row of fixture().bhava.table.rows) expect(nakshatraLord(row.start.nakshatra)).toBe(row.start.nakshatra_lord);
});
````

**Yeni dosya:** `app/ui/panels/GrahaTableNakshatraLord.test.tsx`

````tsx
// Graha tablosunun isteğe bağlı "Nakshatra lordu" sütunu (Plan 26). Ayrı dosyadır:
// `GrahaTablePanel.test.tsx` eşzamanlı planlarla paylaşılır.
import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { computeJaimini: vi.fn(), computeAvasthas: vi.fn() } }));

import { api } from "../api";
import type { Panel } from "../layout/model";
import { fixture, unknownTimeOpened } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { GrahaTablePanel } from "./GrahaTablePanel";

beforeEach(() => {
  vi.mocked(api.computeJaimini).mockReset();
  vi.mocked(api.computeAvasthas).mockReset();
});

function renderTable(settings: Record<string, unknown> = {}, opened = fixture().opened) {
  const panel: Panel = { id: "t", kind: "graha_table", x: 0, y: 0, w: 6, h: 8, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<GrahaTablePanel panel={panel} />, { layout, opened });
}

const headers = () => screen.getAllByRole("columnheader").map((h) => h.textContent);
const lord = (row: string) => document.querySelector(`[data-nakshatra-lord="${row}"]`)!;

test("the nakshatra lord column is off by default", () => {
  renderTable();
  expect(headers()).not.toContain("Nakshatra lordu");
  expect(document.querySelector("[data-nakshatra-lord]")).toBeNull();
});

test("the column names the lord of each graha's nakshatra and of the lagna's, without a command", () => {
  renderTable({ columns: ["nakshatra"], nakshatraLord: true });
  expect(headers()).toEqual(["Graha", "Nakshatra", "Nakshatra lordu"]);
  // Delhi fikstürünün nakshatraları, `Graha::ALL` sırasında: 20, 15, 23, 19, 1, 17, 2, 8, 22;
  // lagna 6. Lord, dokuzlu sıranın ((n − 1) mod 9). adıdır (Ketu, Venüs, Güneş, Ay, Mars, Rahu,
  // Jüpiter, Satürn, Merkür): 20 → 1 Venüs; 15 → 5 Rahu; 23 → 4 Mars; 19 → 0 Ketu; 1 → 0 Ketu;
  // 17 → 7 Satürn; 2 → 1 Venüs; 8 → 7 Satürn; 22 → 3 Ay; lagna 6 → 5 Rahu.
  expect(fixture().opened.chart.grahas.map((g) => g.nakshatra)).toEqual([20, 15, 23, 19, 1, 17, 2, 8, 22]);
  expect(fixture().opened.chart.lagna.nakshatra).toBe(6);
  expect(["Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu"].map((g) => lord(g).textContent)).toEqual([
    "Venüs",
    "Rahu",
    "Mars",
    "Ketu",
    "Ketu",
    "Satürn",
    "Venüs",
    "Satürn",
    "Ay",
  ]);
  expect(lord("lagna")).toHaveTextContent(/^Rahu$/);
  expect(api.computeJaimini).not.toHaveBeenCalled();
  expect(api.computeAvasthas).not.toHaveBeenCalled();
});

test("the column sits after the karaka and Baladi columns", () => {
  vi.mocked(api.computeJaimini).mockImplementation(async () => ({ status: "ok", data: fixture().jaimini }));
  vi.mocked(api.computeAvasthas).mockImplementation(async () => ({ status: "ok", data: fixture().avasthas }));
  renderTable({ columns: ["sign"], karaka: true, baladi: true, nakshatraLord: true });
  expect(headers()).toEqual(["Graha", "Burç", "Karaka", "Baladi", "Nakshatra lordu"]);
});

test("unknown birth time: the Moon's lord is marked approximate and the lagna row is hidden", () => {
  renderTable({ columns: ["nakshatra"], nakshatraLord: true }, unknownTimeOpened());
  expect(lord("Moon")).toHaveTextContent("≈yaklaşık Rahu");
  expect(lord("Sun")).toHaveTextContent(/^Venüs$/);
  expect(document.querySelector('[data-nakshatra-lord="lagna"]')).toBeNull();
});

test("the checkbox writes settings.nakshatraLord and leaves the other settings alone", async () => {
  const { persistence } = renderTable({ columns: ["sign", "house"], karaka: false });
  const summary = screen.getByText("Sütunlar", { selector: "summary" });
  fireEvent.click(summary);
  fireEvent.click(within(summary.closest("details")!).getByRole("checkbox", { name: "Nakshatra lordu" }));
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  const saved = persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!;
  expect(saved.panels[0].settings).toEqual({ columns: ["sign", "house"], karaka: false, nakshatraLord: true });
});
````

**Yeni dosya:** `app/ui/panels/EventSearchLords.test.tsx`

````tsx
// Olay aramasının nakshatra lordu ve hazır seçimi (Plan 26). Ayrı dosyadır:
// `EventSearchPanel.test.tsx` eşzamanlı planlarla paylaşılır.
import { act, fireEvent, screen, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { eventSearch: vi.fn(), eventSearchCancel: vi.fn(), searchPlaces: vi.fn(), setAppPrefs: vi.fn() } }));

import { api, type Result } from "../api";
import type { EventSearchView, SkyEventView } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import type { Panel } from "../layout/model";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useLayoutStore } from "../view/store";
import { EventSearchPanel } from "./EventSearchPanel";

const FX = fixture();
const VIEW = FX.search.events;
const ok = (view: EventSearchView): Result<EventSearchView> => ({ status: "ok", data: view });
/** Fikstürün ilk olayının anı ve ofsetiyle, verilen türde ve nakshatrada bir satır. */
const row = (kind: SkyEventView["event"]["kind"], grahas: SkyEventView["event"]["grahas"], nakshatra: number | null): SkyEventView => ({
  ...VIEW.events[0],
  event: { ...VIEW.events[0].event, kind, grahas, sign: null, nakshatra, house: null },
});

function LivePanel() {
  const { active } = useLayoutStore();
  return <EventSearchPanel panel={active.panels[0]} />;
}
function renderPanel(settings: Record<string, unknown> = {}) {
  const panel: Panel = { id: "es", kind: "event_search", x: 0, y: 0, w: 7, h: 18, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<LivePanel />, { layout, opened: fixture().opened, prefs: FX.prefs.prefs });
}
const run = () => screen.getByRole("button", { name: "Ara" });
const settle = () => act(async () => {});
const where = () => within(screen.getByRole("table", { name: "Olaylar" })).getAllByRole("row").slice(1).map((r) => r.querySelectorAll("td")[1].textContent);

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.eventSearchCancel).mockResolvedValue({ status: "ok", data: null });
});

test("a nakshatra ingress row names the lord of the nakshatra entered; other rows do not", async () => {
  // 4 Rohini: dokuzlu sıranın (4 − 1) mod 9 = 3. adı, Ay. 19 Mula: 18 mod 9 = 0, Ketu.
  // 27 Revati: 26 mod 9 = 8, Merkür. Yeni ay satırı nakshatra taşır ama giriş değildir.
  const events = [row("NakshatraIngress", ["Moon"], 4), row("NakshatraIngress", ["Moon"], 19), row("NakshatraIngress", ["Saturn"], 27), row("NewMoon", ["Sun", "Moon"], 18)];
  vi.mocked(api.eventSearch).mockResolvedValue(ok({ ...VIEW, events }));
  renderPanel();
  fireEvent.click(run());
  await settle();
  expect(where()).toEqual(["Rohini · lordu Ay", "Mula · lordu Ketu", "Revati · lordu Merkür", "Jyeshtha"]);
  // Sütun eklenmez: lord, var olan "Burç · nakshatra" hücresindedir.
  expect(within(screen.getByRole("table", { name: "Olaylar" })).getAllByRole("columnheader")).toHaveLength(3);
});

test("the fixture's rows are unchanged: none of them is a nakshatra ingress", async () => {
  vi.mocked(api.eventSearch).mockResolvedValue(ok(VIEW));
  renderPanel();
  fireEvent.click(run());
  await settle();
  expect(VIEW.events.some((e) => e.event.kind === "NakshatraIngress")).toBe(false);
  for (const text of where()) expect(text).not.toContain("lordu");
});

test("the preset selects the Moon's nakshatra ingresses and does not start a search", async () => {
  vi.mocked(api.eventSearch).mockResolvedValue(ok(VIEW));
  renderPanel();
  fireEvent.click(screen.getByRole("button", { name: "Hazır seçim: Ay'ın nakshatra girişleri" }));
  await settle();
  expect(api.eventSearch).not.toHaveBeenCalled();
  const form = screen.getByRole("form", { name: "Olay araması ayarları" });
  const checked = (group: string) =>
    within(within(form).getByRole("group", { name: group }))
      .getAllByRole("checkbox", { checked: true })
      .map((c) => c.parentElement?.textContent);
  expect(checked("Olay türleri")).toEqual(["Nakshatra girişi"]);
  expect(checked("Grahalar")).toEqual(["Ay"]);
  fireEvent.click(run());
  await settle();
  expect(vi.mocked(api.eventSearch).mock.calls.at(-1)![0]).toMatchObject({ kinds: ["NakshatraIngress"], grahas: ["Moon"] });
});

test("the lord texts exist in the three languages", () => {
  expect([tr.eventSearch.lordOf, en.eventSearch.lordOf, fr.eventSearch.lordOf]).toEqual(["lordu {{graha}}", "lord {{graha}}", "maître {{graha}}"]);
  expect([tr.transit.nakshatraLord, en.transit.nakshatraLord, fr.transit.nakshatraLord]).toEqual(["Nakshatra lordu", "Nakshatra lord", "Maître du nakshatra"]);
  for (const m of [tr, en, fr]) expect(m.table.nakshatraLord).toBe(m.transit.nakshatraLord);
});
````

````diff
diff --git a/app/ui/transit/TransitTablePanel.test.tsx b/app/ui/transit/TransitTablePanel.test.tsx
index 6d44519..c6deb90 100644
--- a/app/ui/transit/TransitTablePanel.test.tsx
+++ b/app/ui/transit/TransitTablePanel.test.tsx
@@ -15,7 +15,7 @@ const panel: Panel = { id: "tt", kind: "transit_table", x: 0, y: 0, w: 6, h: 8,
 beforeEach(() => vi.mocked(api.transitsAt).mockResolvedValue({ status: "ok", data: fixture().transit }));
 afterEach(() => vi.mocked(api.transitsAt).mockReset());
 
-test("nine rows: sign, degree, nakshatra, natal house and the retro mark (none on the nodes)", async () => {
+test("nine rows: sign, degree, nakshatra, its lord, natal house and the retro mark (none on the nodes)", async () => {
   renderWithStores(<TransitTablePanel panel={panel} />);
   const table = await screen.findByRole("table");
   const rows = within(table).getAllByRole("row").slice(1);
@@ -23,16 +23,29 @@ test("nine rows: sign, degree, nakshatra, natal house and the retro mark (none o
   const tr = fixture().transit.snapshot.grahas;
   rows.forEach((row, i) => {
     const cells = within(row).getAllByRole("cell");
-    expect(cells[3]).toHaveTextContent(String(tr[i].natal_house));
+    expect(cells[4]).toHaveTextContent(String(tr[i].natal_house));
     const nodes = tr[i].graha === "Rahu" || tr[i].graha === "Ketu";
-    expect(cells[4]).toHaveTextContent(tr[i].retrograde && !nodes ? "R" : "");
+    expect(cells[5]).toHaveTextContent(tr[i].retrograde && !nodes ? "R" : "");
   });
 });
 
+test("the nakshatra lord column names the lord of each transit nakshatra", async () => {
+  renderWithStores(<TransitTablePanel panel={panel} />);
+  const table = await screen.findByRole("table");
+  expect(within(table).getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Graha", "Burç", "Derece", "Nakshatra", "Nakshatra lordu", "Natal ev", "Retro"]);
+  // Fikstür (2026-11-25 10:00 UT) nakshatraları, `Graha::ALL` sırasında: 17, 4, 10, 16, 10, 14,
+  // 26, 23, 10. Lord, dokuzlu sıranın ((n − 1) mod 9). adıdır: Ketu, Venüs, Güneş, Ay, Mars,
+  // Rahu, Jüpiter, Satürn, Merkür. 17 → 7 Satürn; 4 → 3 Ay; 10 → 0 Ketu; 16 → 6 Jüpiter;
+  // 14 → 4 Mars; 26 → 7 Satürn; 23 → 4 Mars.
+  expect(fixture().transit.snapshot.grahas.map((g) => g.nakshatra)).toEqual([17, 4, 10, 16, 10, 14, 26, 23, 10]);
+  const lords = [...table.querySelectorAll("[data-nakshatra-lord]")].map((c) => c.textContent);
+  expect(lords).toEqual(["Satürn", "Ay", "Ketu", "Jüpiter", "Ketu", "Mars", "Satürn", "Mars", "Ketu"]);
+});
+
 test("unknown birth time hides the natal house column values", async () => {
   renderWithStores(<TransitTablePanel panel={panel} />, { opened: unknownTimeOpened() });
   const table = await screen.findByRole("table");
-  for (const row of within(table).getAllByRole("row").slice(1)) expect(within(row).getAllByRole("cell")[3]).toHaveTextContent("—");
+  for (const row of within(table).getAllByRole("row").slice(1)) expect(within(row).getAllByRole("cell")[4]).toHaveTextContent("—");
 });
 
 test("apply sends the wall-clock time in the browser's time zone; now goes back to now", async () => {
````

```bash
pnpm --dir app exec vitest run ui/panels/nakshatraLord.test.ts ui/panels/GrahaTableNakshatraLord.test.tsx ui/panels/EventSearchLords.test.tsx ui/transit/TransitTablePanel.test.tsx 2>&1 | grep -E "Test Files|Tests "
```
Expected: dört dosya da düşer (`nakshatraLord` modülü ve sütunlar yok).

- [ ] **Step 7: Yardımcı, üç yüzey ve üç dilde metinler**

**Yeni dosya:** `app/ui/panels/nakshatraLord.ts`

````ts
// Nakshatra lordu (yol haritası Plan 26, §4.7). Tablo Rust'ındır (`jyotish_core::NAKSHATRA_LORDS`)
// ve bağlama dosyasıyla sabit olarak gelir: burada yalnızca satır seçilir, lord türetilmez (C4).
import { NAKSHATRA_LORDS, type Graha } from "../bindings";

/** n. nakshatranın (1..=27) lordu; aralık dışı ya da tam olmayan numara `null`. */
export function nakshatraLord(n: number | null): Graha | null {
  if (n === null || !Number.isInteger(n) || n < 1 || n > NAKSHATRA_LORDS.length) return null;
  return NAKSHATRA_LORDS[n - 1];
}
````

````diff
diff --git a/app/ui/panels/EventSearchPanel.tsx b/app/ui/panels/EventSearchPanel.tsx
index 456a1e0..6badedc 100644
--- a/app/ui/panels/EventSearchPanel.tsx
+++ b/app/ui/panels/EventSearchPanel.tsx
@@ -5,6 +5,9 @@
 // Olay bir olgudur: "iyi" ya da "kötü" hükmü ve açıklama metni yoktur. Retro düz `R`'dir.
 // Arama durumu ve sonuç sayısı hep DOM'da duran tek bir `role="status"` bölgesindedir. Liste,
 // başlatıldığı isteğin kopyasına bağlıdır (D24): yeni arama başlayınca önceki liste kalkar.
+// Nakshatra girişi satırı girilen nakshatranın lordunu da yazar (Plan 26): lord Rust'ın tablosundan
+// okunur. "Hazır seçim" düğmesi tür ve graha seçimini Ay'ın nakshatra girişlerine çevirir; aramayı
+// başlatmaz.
 import { useEffect, useId, useLayoutEffect, useRef, useState, type FormEvent, type RefObject } from "react";
 import { useTranslation } from "react-i18next";
 import type { CommandError, EventSearchSpec, Graha, OpenedClient, SkyEventKind, SkyEventView } from "../bindings";
@@ -23,6 +26,7 @@ import { useTransitControl } from "../transit/store";
 import { useLayoutStore } from "../view/store";
 import { formatJdClock, formatJdDate, formatJdMinute } from "./format";
 import { nakshatraName } from "./labels";
+import { nakshatraLord } from "./nakshatraLord";
 import { PlaceControls, PlaceMissing, usePanelPlace } from "./MuhurtaPlace";
 import "./panels.css";
 import { PanelError, usePanelLoadPhase } from "./states";
@@ -147,6 +151,9 @@ function Search({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
             {t("eventSearch.run")}
           </button>
           {running && <StopButton onStop={search.stop} fallback={runButton} />}
+          <button type="button" onClick={() => setPanelSettings(panel.id, { kinds: ["NakshatraIngress"], grahas: ["Moon"] })}>
+            {t("eventSearch.moonPreset")}
+          </button>
         </div>
         {!ready && <div id={`${id}-why`}>{why !== null ? <p className="chart-toolbar-hint">{t(`eventSearch.why.${why}`)}</p> : <PlaceMissing choice={place.choice} />}</div>}
       </form>
@@ -227,7 +234,15 @@ function Results({ search, stale, statusId }: { search: EventSearch; stale: bool
         return t(`eventSearch.event.${e.kind}`, { graha: name(a) });
     }
   };
-  const where = ({ event: e }: SkyEventView) => [e.sign === null ? null : t(`sign.${signIndex(e.sign)}`), e.nakshatra === null ? null : nakshatraName(e.nakshatra)].filter((x) => x !== null).join(" · ");
+  // Yalnızca nakshatra girişi satırında: girilen nakshatranın lordu.
+  const lord = ({ event: e }: SkyEventView) => {
+    const graha = e.kind === "NakshatraIngress" ? nakshatraLord(e.nakshatra) : null;
+    return graha === null ? null : t("eventSearch.lordOf", { graha: name(graha) });
+  };
+  const where = (row: SkyEventView) => {
+    const e = row.event;
+    return [e.sign === null ? null : t(`sign.${signIndex(e.sign)}`), e.nakshatra === null ? null : nakshatraName(e.nakshatra), lord(row)].filter((x) => x !== null).join(" · ");
+  };
   const day = (jd: number | null, offset: number) => formatJdDate(jd, offset);
   // Kesilen liste son olayın anına dek taranmıştır: an, o olayın kendi ofsetiyle yazılır.
   const until = formatJdMinute(view.scanned_to_jd_ut, view.events.at(-1)?.offset_secs ?? view.to_offset_secs);
diff --git a/app/ui/panels/GrahaTablePanel.tsx b/app/ui/panels/GrahaTablePanel.tsx
index 8f5e89c..73d389f 100644
--- a/app/ui/panels/GrahaTablePanel.tsx
+++ b/app/ui/panels/GrahaTablePanel.tsx
@@ -14,6 +14,7 @@ import { BALADI_NAMES } from "./avasthaLabels";
 import { APPROX_COLUMNS, grahaCellText, lagnaCellText, NUMERIC_COLUMNS as NUMERIC } from "./grahaCells";
 import { GRAHA_COLUMNS, grahaColumns, toggleColumn, type GrahaColumn } from "./grahaColumns";
 import { KARAKA_ROLES } from "./labels";
+import { nakshatraLord } from "./nakshatraLord";
 import "./panels.css";
 import { PanelError } from "./states";
 import type { PanelProps } from "./types";
@@ -44,7 +45,9 @@ function grahaCell(t: TFunction, g: GrahaPosition, column: GrahaColumn, approx:
  * chara karaka rolünü `compute_jaimini`'den okur; açık değilken komut çağrılmaz. Sütun
  * `columns` listesinde değildir: baskı tablosu o listeyi kullanır ve değişmez (K10). İsteğe
  * bağlı "Baladi" sütunu (`settings.baladi`) aynı kalıptır: değer `compute_avasthas`'tan okunur,
- * açık değilken komut çağrılmaz; Rahu, Ketu ve lagna satırında boştur.
+ * açık değilken komut çağrılmaz; Rahu, Ketu ve lagna satırında boştur. İsteğe bağlı "Nakshatra
+ * lordu" sütunu (`settings.nakshatraLord`; Plan 26) komut çağırmaz: lord, satırın nakshatra
+ * numarasıyla Rust'ın tablosundan okunur (`nakshatraLord`); lagna satırında da doludur.
  */
 export function GrahaTablePanel({ panel }: PanelProps) {
   const { t } = useTranslation();
@@ -55,6 +58,11 @@ export function GrahaTablePanel({ panel }: PanelProps) {
   const jaimini = useJaimini(opened, karakaOn);
   const baladiOn = panel.settings.baladi === true;
   const avasthas = useAvasthas(opened, baladiOn);
+  const lordOn = panel.settings.nakshatraLord === true;
+  const lordName = (nakshatra: number) => {
+    const lord = nakshatraLord(nakshatra);
+    return lord === null ? "—" : t(`grahaName.${lord}`);
+  };
 
   if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
   if (opened.chart.grahas.length === 0) return <p className="panel-message">{t("table.empty")}</p>;
@@ -91,6 +99,10 @@ export function GrahaTablePanel({ panel }: PanelProps) {
             <input type="checkbox" checked={baladiOn} onChange={(e) => setPanelSettings(panel.id, { baladi: e.target.checked })} />
             {t("table.baladi")}
           </label>
+          <label>
+            <input type="checkbox" checked={lordOn} onChange={(e) => setPanelSettings(panel.id, { nakshatraLord: e.target.checked })} />
+            {t("table.nakshatraLord")}
+          </label>
         </fieldset>
       </details>
       {jaimini.error && <PanelError error={jaimini.error} onRetry={jaimini.retry} />}
@@ -106,6 +118,7 @@ export function GrahaTablePanel({ panel }: PanelProps) {
             ))}
             {karakaOn && <th scope="col">{t("table.karaka")}</th>}
             {baladiOn && <th scope="col">{t("table.baladi")}</th>}
+            {lordOn && <th scope="col">{t("table.nakshatraLord")}</th>}
           </tr>
         </thead>
         <tbody>
@@ -119,6 +132,7 @@ export function GrahaTablePanel({ panel }: PanelProps) {
               ))}
               {karakaOn && <td />}
               {baladiOn && <td />}
+              {lordOn && <td data-nakshatra-lord="lagna">{lordName(opened.chart.lagna.nakshatra)}</td>}
             </tr>
           )}
           {opened.chart.grahas.map((g) => {
@@ -172,6 +186,14 @@ export function GrahaTablePanel({ panel }: PanelProps) {
                     )}
                   </td>
                 )}
+                {lordOn && (
+                  <td data-nakshatra-lord={g.graha}>
+                    {/* Ay'ın nakshatrası yaklaşık dereceden çıkar: lordu da "≈" taşır (K2). */}
+                    <Approx on={approx} label={t("table.approx")}>
+                      {lordName(g.nakshatra)}
+                    </Approx>
+                  </td>
+                )}
               </tr>
             );
           })}
diff --git a/app/ui/transit/TransitTablePanel.tsx b/app/ui/transit/TransitTablePanel.tsx
index aea8a0e..ed76bf9 100644
--- a/app/ui/transit/TransitTablePanel.tsx
+++ b/app/ui/transit/TransitTablePanel.tsx
@@ -1,10 +1,12 @@
 // Transit tablosu (UI spec §3.3). Konumlar ve natal evler Rust'tan (`transits_at`); saat
 // bilinmiyorsa natal ev gösterilmez (spec §7). Retro düz "R", düğümlerde yok (spec §2.3).
+// Nakshatra lordu Rust'ın tablosundan okunur (`nakshatraLord`; Plan 26).
 import { useTranslation } from "react-i18next";
 import { signIndex } from "../chart/adapter";
 import { RETRO_MARK, showsRetro, type GrahaId } from "../chart/types";
 import { formatDegree } from "../panels/format";
 import { nakshatraName } from "../panels/labels";
+import { nakshatraLord } from "../panels/nakshatraLord";
 import { PanelError, usePanelLoadPhase } from "../panels/states";
 import type { PanelProps } from "../panels/types";
 import { timeUnknown } from "../session/birth";
@@ -21,6 +23,10 @@ export function TransitTablePanel(_: PanelProps) {
   if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
   const unknown = timeUnknown(opened);
   const view = transit.view;
+  const lordName = (nakshatra: number) => {
+    const lord = nakshatraLord(nakshatra);
+    return lord === null ? "—" : t(`grahaName.${lord}`);
+  };
   return (
     <div className="transit-panel">
       <TransitDate />
@@ -36,6 +42,7 @@ export function TransitTablePanel(_: PanelProps) {
                 <th scope="col">{t("transit.col.sign")}</th>
                 <th scope="col">{t("transit.col.degree")}</th>
                 <th scope="col">{t("transit.col.nakshatra")}</th>
+                <th scope="col">{t("transit.nakshatraLord")}</th>
                 <th scope="col">{t("transit.col.house")}</th>
                 <th scope="col">{t("transit.col.retro")}</th>
               </tr>
@@ -49,6 +56,7 @@ export function TransitTablePanel(_: PanelProps) {
                   <td>
                     {nakshatraName(g.nakshatra)} {g.pada}
                   </td>
+                  <td data-nakshatra-lord={g.graha}>{lordName(g.nakshatra)}</td>
                   <td className="num">{unknown ? "—" : g.natal_house}</td>
                   <td>{showsRetro(g.graha as GrahaId, g.retrograde) ? RETRO_MARK : ""}</td>
                 </tr>
````

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index f8e66fe..b1ee1ae 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -855,6 +855,8 @@ export const en: Messages = {
       HouseFromNatalMoon: "{{graha}}: house {{house}} from the natal Moon",
     },
     method: "Method: {{method}}. Moments are in the time zone of the selected place; the moment in a row changes the transit date.",
+    lordOf: "lord {{graha}}",
+    moonPreset: "Preset: the Moon's nakshatra ingresses",
   },
   bhavaTable: {
     caption: "Bhava table: {{point}} point, nakshatra, lords and arc of the twelve bhavas",
@@ -920,6 +922,7 @@ export const en: Messages = {
     approx: "approximate",
     karaka: "Karaka",
     baladi: "Baladi",
+    nakshatraLord: "Nakshatra lord",
   },
   errors: {
     dasha_empty: "The sub-periods of this period could not be loaded.",
@@ -1381,6 +1384,7 @@ export const en: Messages = {
     shownNow: "Now: {{when}}",
     shown: "Shown moment: {{when}}",
     caption: "Transit positions, {{when}}",
+    nakshatraLord: "Nakshatra lord",
     col: { graha: "Graha", sign: "Sign", degree: "Degree", nakshatra: "Nakshatra", house: "Natal house", retro: "Retro" },
   },
 };
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 67cc873..8f57fcb 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -855,6 +855,8 @@ export const fr: Messages = {
       HouseFromNatalMoon: "{{graha}} : maison {{house}} depuis la Lune natale",
     },
     method: "Méthode : {{method}}. Les instants sont dans le fuseau du lieu choisi ; l’instant d’une ligne change la date de transit.",
+    lordOf: "maître {{graha}}",
+    moonPreset: "Préréglage : entrées de la Lune dans les nakshatras",
   },
   bhavaTable: {
     caption: "Table des bhavas : {{point}}, nakshatra, maîtres et arc des douze bhavas",
@@ -920,6 +922,7 @@ export const fr: Messages = {
     approx: "approximatif",
     karaka: "Karaka",
     baladi: "Baladi",
+    nakshatraLord: "Maître du nakshatra",
   },
   errors: {
     dasha_empty: "Les sous-périodes de cette période n’ont pas pu être chargées.",
@@ -1381,6 +1384,7 @@ export const fr: Messages = {
     shownNow: "Maintenant : {{when}}",
     shown: "Moment affiché : {{when}}",
     caption: "Positions des transits, {{when}}",
+    nakshatraLord: "Maître du nakshatra",
     col: { graha: "Graha", sign: "Signe", degree: "Degré", nakshatra: "Nakshatra", house: "Maison natale", retro: "Rétro" },
   },
 };
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 3c2e07b..fdb980d 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -854,6 +854,8 @@ export const tr = {
       HouseFromNatalMoon: "{{graha}}: doğum Ay'ından {{house}}. ev",
     },
     method: "Yöntem: {{method}}. Anlar seçilen yerin saat dilimindedir; satırdaki an transit tarihini değiştirir.",
+    lordOf: "lordu {{graha}}",
+    moonPreset: "Hazır seçim: Ay'ın nakshatra girişleri",
   },
   bhavaTable: {
     caption: "Bhava tablosu: on iki bhavanın {{point}} noktası, nakshatrası, lordları ve yayı",
@@ -919,6 +921,7 @@ export const tr = {
     approx: "yaklaşık",
     karaka: "Karaka",
     baladi: "Baladi",
+    nakshatraLord: "Nakshatra lordu",
   },
   errors: {
     dasha_empty: "Bu dönemin alt dönemleri alınamadı.",
@@ -1380,6 +1383,7 @@ export const tr = {
     shownNow: "Şimdi: {{when}}",
     shown: "Gösterilen an: {{when}}",
     caption: "Transit konumları, {{when}}",
+    nakshatraLord: "Nakshatra lordu",
     col: { graha: "Graha", sign: "Burç", degree: "Derece", nakshatra: "Nakshatra", house: "Natal ev", retro: "Retro" },
   },
 };
````

- [ ] **Step 8: Testler ve kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -1
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
cargo test -p jyotish-core --lib 2>&1 | grep -E "^test result"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
lsof -i :14326
pnpm --dir app exec playwright test --project=e2e -c playwright.port.config.ts 2>&1 | tail -1
rm -rf app/test-results
```
Expected: fmt çıktısız; clippy `Finished`; sayaç `0`; `469 passed`; Vitest `100 passed (100)`, `986 passed (986)`; `✓ built`; `lsof` çıktısız; e2e `33 passed` (var olan akışlar değişmedi: olay tablosuna sütun eklenmedi, graha tablosunun sütunu varsayılan olarak gizli).

- [ ] **Step 9: Commit**

```bash
git add crates/jyotish-core/src/lib.rs crates/jyotish-core/src/transit_tools app/src-tauri/src/lib.rs app/ui/bindings.ts app/ui/panels/nakshatraLord.ts app/ui/panels/nakshatraLord.test.ts app/ui/panels/GrahaTableNakshatraLord.test.tsx app/ui/panels/EventSearchLords.test.tsx app/ui/panels/GrahaTablePanel.tsx app/ui/panels/EventSearchPanel.tsx app/ui/transit/TransitTablePanel.tsx app/ui/transit/TransitTablePanel.test.tsx app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(transit): nakshatra lords table from Rust; lord column in the transit and graha tables and on nakshatra ingress rows" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: on altı dosya; yasaklı ad taraması çıktısız; `git status --short` boş.


---

### Task 2: Sankranti çekirdeği ve komutu: yıl listesi, giriş anının panchangı, gerçek motorla çapraz denetim ve fikstür

Yol haritası §4.8 ve görev taslağının (2). maddesi; D6–D9, D17, D18. `timeline.rs` ve `panchang.rs` yalnızca okunur. Komut yeni bir modül dizinindedir; gerçek motor testi yeni bir test dosyasıdır ve arayüz fikstürünü de üretir (`delhi.json` ve `ui_fixture.rs` değişmez).

**Files:**
- Create: `crates/jyotish-core/src/transit_tools/sankranti.rs`, `app/src-tauri/src/transit_tools_commands/mod.rs`, `app/src-tauri/src/transit_tools_commands/sankranti.rs`, `app/src-tauri/tests/sankranti.rs`, `app/ui/test/fixtures/sankranti.json` (üretilir), `tests/golden/modules/transit_tools/REVIEW.md`
- Modify: `crates/jyotish-core/src/transit_tools/mod.rs`, `crates/jyotish-core/src/lib.rs`, `app/src-tauri/src/lib.rs`, `app/ui/bindings.ts` (üretilir), `app/ui/api.ts`, `app/ui/api.test.ts`
- Test: `sankranti.rs` `mod tests` (7); `tests/sankranti.rs` (1)

**Interfaces:**
- Consumes: `timeline::{walk, lon_of, sidereal, GRAHA_STEP_DAYS}` (kasa içi), `panchang::compute_panchang`, `chart::ChartWarning`; uygulamada `muhurta_commands::{day_start_jd, offset_at, today, MuhurtaPlace, J2000_JD}` (kasa içi), `client_commands::blocking`.
- Produces (çekirdek adları `jyotish_core` kökünden de çıkar):
  - `SANKRANTI_METHOD = "sankranti.sidereal_v1"`, `SANKRANTI_MAX_RANGE_DAYS = 400.0`.
  - `Sankranti { jd_ut: f64, sign: Sign, tithi: u8, paksha: Paksha, vara: Vara, nakshatra: u8, pada: u8 }`.
  - `SankrantiList { rows: Vec<Sankranti>, warnings: Vec<ChartWarning>, method: String }`.
  - `sankranti_moments(eph, settings, from_jd_ut, to_jd_ut) -> Result<Vec<(f64, Sign)>, CoreError>` (yalnızca anlar ve burçlar; panchang hesaplamaz).
  - `sankranti_list(eph, settings, geo_lat, geo_lon, from_jd_ut, to_jd_ut) -> Result<SankrantiList, CoreError>`.
  - Uygulama: `transit_tools_commands::TOOLS_YEAR_RANGE` (1..=2999); `sankranti::{SankrantiView { sankranti, offset_secs }, SankrantiListView { year, rows, from_jd_ut, to_jd_ut, warnings, method }, sankranti_list_blocking(engine, year: Option<i32>, place, settings, now), sankranti_list}`; `api.sankrantiList(year, place, settings)`.
  - Hata: `invalid_transit_input { field }`; alanlar `range`, `place`, `year`.

- [ ] **Step 1: Kayıt ve testler: on iki giriş, iki el örneği, ikinci yazım, çerçeve, uyarılar, girdi denetimi ve gerçek motor**

El örnekleri sahte gökten elle türetilmiştir (Güneş sidereal 346°, günde 0,98°; Ay 71°, günde 13,2°); türetme test yorumlarındadır.

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 7a978ca..6244ea0 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -167,4 +167,7 @@ pub use zodiac::{angular_distance, deg_in_sign, nakshatra_pada, normalize, Sign}
 // Geçiş araçları (yol haritası Plan 26). Ayrı blok, dosyanın sonunda: eşzamanlı planların
 // alfabetik gruplardaki ve dosya başındaki ekleriyle aynı satırlara dokunmaz.
 pub mod transit_tools;
-pub use transit_tools::NAKSHATRA_LORDS;
+pub use transit_tools::{
+    sankranti_list, sankranti_moments, Sankranti, SankrantiList, NAKSHATRA_LORDS,
+    SANKRANTI_MAX_RANGE_DAYS, SANKRANTI_METHOD,
+};
diff --git a/crates/jyotish-core/src/transit_tools/mod.rs b/crates/jyotish-core/src/transit_tools/mod.rs
index e727d43..db6acb6 100644
--- a/crates/jyotish-core/src/transit_tools/mod.rs
+++ b/crates/jyotish-core/src/transit_tools/mod.rs
@@ -3,5 +3,10 @@
 //! yoktur. Sonuçlar `Chart`'a eklenmez ve saklanmaz; panel açıkken istenir (K1).
 
 pub mod lords;
+pub mod sankranti;
 
 pub use lords::NAKSHATRA_LORDS;
+pub use sankranti::{
+    sankranti_list, sankranti_moments, Sankranti, SankrantiList, SANKRANTI_MAX_RANGE_DAYS,
+    SANKRANTI_METHOD,
+};
````

**Yeni dosya:** `crates/jyotish-core/src/transit_tools/sankranti.rs` (şimdilik yalnızca test bloğu)

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::graha::Graha;
    use crate::settings::ZodiacMode;
    use crate::timeline::graha_sign_series;
    use crate::zodiac::normalize;

    const JD: f64 = 2_451_545.0;
    /// Sahte gök: Güneş `JD` anında tropikal 10°, günde 0,98°; ayanamsa 24° → sidereal 346°.
    /// Sidereal k. burç sınırına (k × 30°) varış: (14 + 30k) / 0,98 gün sonra.
    fn entry_day(k: u32) -> f64 {
        (14.0 + 30.0 * f64::from(k)) / 0.98
    }

    fn list(settings: &CalcSettings, from: f64, to: f64) -> Vec<Sankranti> {
        let list = sankranti_list(&mut FakeEph::default(), settings, 41.0, 29.0, from, to).unwrap();
        assert_eq!(list.method, "sankranti.sidereal_v1");
        assert!(list.warnings.is_empty());
        list.rows
    }

    #[test]
    fn a_year_has_twelve_entries_at_the_moments_the_arithmetic_gives() {
        // 366 günde on iki sınır: 14,29; 44,90; 75,51 … 351,02. gün (on üçüncüsü 381,63).
        let rows = list(&CalcSettings::default(), JD, JD + 366.0);
        assert_eq!(rows.len(), 12);
        for (k, row) in rows.iter().enumerate() {
            let expected = JD + entry_day(k as u32);
            assert!(
                (row.jd_ut - expected).abs() < 1e-6,
                "{k}: {} vs {expected}",
                row.jd_ut
            );
            // İlk giriş Koç'tur (sidereal 346° Balık'tadır); sıra burç sırasıdır.
            assert_eq!(row.sign, Sign::from_index(k % 12), "{k}");
        }
        assert_eq!(rows[0].sign, Sign::Aries);
        assert_eq!(rows[11].sign, Sign::Pisces);
        // Girişler zaman sırasındadır ve aralığın içindedir.
        assert!(rows.windows(2).all(|w| w[0].jd_ut < w[1].jd_ut));
        assert!(rows.iter().all(|r| JD < r.jd_ut && r.jd_ut < JD + 366.0));
    }

    #[test]
    fn an_entry_carries_the_tithi_vara_and_moon_nakshatra_of_its_moment() {
        // İlk giriş: t = 14 / 0,98 = 14,285714 gün. Ay `JD` anında sidereal 71°, günde 13,2°:
        // 71 + 13,2 × 14,285714 = 259,571°. Güneş 0°'dedir: uzanım 259,571°.
        // Tithi: 259,571 / 12 = 21,63 → 22. tithi, Krishna paksha (22 > 15).
        // Nakshatra: 259,571 / 13,3333 = 19,47 → 20. nakshatra (Purva Ashadha); nakshatranın
        // başı 253,333°, içinde 6,238°; 6,238 / 3,3333 = 1,87 → 2. pada.
        // Vara: an JD 2451559,2857. Sahte gün doğumu her gün n + 0,67'dedir: andan önceki son
        // doğuş 2451558,67. Yerel gün (29° D): floor(2451558,67 + 0,5 + 29/360) = 2451559;
        // 2451545 (2000-01-01) Cumartesi'dir ve 14 gün sonrası yine Cumartesi'dir.
        let first = &list(&CalcSettings::default(), JD, JD + 30.0)[0];
        assert_eq!(first.sign, Sign::Aries);
        assert_eq!((first.tithi, first.paksha), (22, Paksha::Krishna));
        assert_eq!((first.nakshatra, first.pada), (20, 2));
        assert_eq!(first.vara, Vara::Saturday);
        // İkinci giriş (Boğa): t = 44 / 0,98 = 44,897959 gün; Ay 71 + 592,653 = 663,653 →
        // 303,653°; Güneş 30°: uzanım 273,653° → 273,653 / 12 = 22,80 → 23. tithi.
        // Nakshatra: 303,653 / 13,3333 = 22,77 → 23. (Dhanishta); başı 293,333°, içinde
        // 10,320°; 10,320 / 3,3333 = 3,10 → 4. pada.
        // Vara: an 2451589,898; önceki doğuş 2451589,67; yerel gün floor(2451590,2506) =
        // 2451590 = 2451545 + 45; 45 = 6 × 7 + 3: Cumartesi + 3 = Salı.
        let rows = list(&CalcSettings::default(), JD, JD + 60.0);
        assert_eq!(rows.len(), 2);
        let second = &rows[1];
        assert_eq!(second.sign, Sign::Taurus);
        assert_eq!((second.tithi, second.paksha), (23, Paksha::Krishna));
        assert_eq!((second.nakshatra, second.pada), (23, 4));
        assert_eq!(second.vara, Vara::Tuesday);
    }

    #[test]
    fn the_sun_crosses_the_sidereal_sign_start_at_the_entry() {
        // İkinci yazım: giriş anının bir saniye öncesinde Güneş önceki burcun sonunda, bir
        // saniye sonrasında girilen burcun başındadır.
        let mut eph = FakeEph::default();
        let settings = CalcSettings::default();
        let zodiac = sidereal(&settings);
        let second = 1.0 / 86_400.0;
        for row in list(&settings, JD, JD + 366.0) {
            let before = lon_of(&mut eph, row.jd_ut - second, Body::Sun, zodiac).unwrap();
            let after = lon_of(&mut eph, row.jd_ut + second, Body::Sun, zodiac).unwrap();
            let start = 30.0 * row.sign.index() as f64;
            assert_eq!(Sign::of(after), row.sign);
            assert_eq!(
                Sign::of(before),
                Sign::from_index((row.sign.index() + 11) % 12)
            );
            assert!(normalize(after - start) < 1e-4, "{:?}", row.sign);
            assert!(normalize(start - before) < 1e-4, "{:?}", row.sign);
        }
    }

    #[test]
    fn the_list_is_sidereal_in_the_tropical_rasi_mode_too() {
        let sidereal_mode = CalcSettings::default();
        let tropical = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..CalcSettings::default()
        };
        // Liste zodyak kipine bakmaz.
        assert_eq!(
            list(&tropical, JD, JD + 366.0),
            list(&sidereal_mode, JD, JD + 366.0)
        );
        // Rasi çerçevesindeki (tropikal) Güneş burç girişleri başka anlardır: tropikal 10°'den
        // 30°'ye 20 / 0,98 = 20,41 gün; sidereal giriş 14,29. gündedir.
        let series = graha_sign_series(
            &mut FakeEph::default(),
            &tropical,
            Graha::Sun,
            JD,
            JD + 30.0,
        )
        .unwrap();
        assert_eq!(series.len(), 2);
        assert!((series[1].start_jd_ut - (JD + 20.0 / 0.98)).abs() < 1e-6);
        let first = &list(&tropical, JD, JD + 30.0)[0];
        assert!((first.jd_ut - (JD + 14.0 / 0.98)).abs() < 1e-6);
    }

    #[test]
    fn spans_join_without_a_gap_and_a_span_without_an_entry_is_empty() {
        let s = CalcSettings::default();
        assert!(list(&s, JD, JD + 10.0).is_empty());
        let mut joined = list(&s, JD, JD + 200.0);
        joined.extend(list(&s, JD + 200.0, JD + 366.0));
        assert_eq!(joined, list(&s, JD, JD + 366.0));
    }

    #[test]
    fn warnings_report_the_fallback_ephemeris_and_a_day_without_sunrise() {
        let s = CalcSettings::default();
        let mut moshier = FakeEph {
            precision: Precision::Moshier,
            ..FakeEph::default()
        };
        let low = sankranti_list(&mut moshier, &s, 41.0, 29.0, JD, JD + 30.0).unwrap();
        assert_eq!(low.warnings, [ChartWarning::LowPrecision]);
        // Gün doğumu yok: gün 06:00 yerel ortalama saatte başlar. İlk giriş JD 2451559,2857:
        // 0,25 gün öncesi 2451559,0357; yerel gün floor(2451559,0357 + 0,5 + 29/360) =
        // 2451559, yine Cumartesi. Öteki olgular değişmez.
        let mut polar = FakeEph {
            polar: true,
            ..FakeEph::default()
        };
        let dark = sankranti_list(&mut polar, &s, 41.0, 29.0, JD, JD + 30.0).unwrap();
        assert_eq!(dark.warnings, [ChartWarning::SunriseUnavailable]);
        assert_eq!(dark.rows, list(&s, JD, JD + 30.0));
        assert_eq!(dark.rows[0].vara, Vara::Saturday);
    }

    #[test]
    fn bad_input_names_the_field_and_never_carries_the_place() {
        let s = CalcSettings::default();
        let mut eph = FakeEph::default();
        let range = Err(CoreError::InvalidTransitInput("range"));
        for (from, to) in [
            (JD, JD),
            (JD + 1.0, JD),
            (JD, JD + SANKRANTI_MAX_RANGE_DAYS + 1.0),
            (f64::NAN, JD),
            (JD, f64::INFINITY),
            (1.0, 2.0),
        ] {
            assert_eq!(
                sankranti_list(&mut eph, &s, 41.0, 29.0, from, to),
                range,
                "{from} {to}"
            );
        }
        // Yer aralıktan önce denetlenir ve hata koordinatı taşımaz.
        for (lat, lon) in [(91.0, 29.0), (41.0, 181.0), (f64::NAN, 29.0)] {
            assert_eq!(
                sankranti_list(&mut eph, &s, lat, lon, JD + 1.0, JD),
                Err(CoreError::InvalidTransitInput("place"))
            );
        }
        let old = CalcSettings {
            schema_version: 1,
            ..CalcSettings::default()
        };
        assert_eq!(
            sankranti_list(&mut eph, &old, 41.0, 29.0, JD, JD + 30.0),
            Err(CoreError::UnsupportedSchemaVersion {
                found: 1,
                supported: 2
            })
        );
    }
}
````

**Yeni dosya:** `app/src-tauri/tests/sankranti.rs`

````rust
//! Sankranti listesi komutu, gerçek `Engine` ile uçtan uca (yol haritası Plan 26, §4.8) ve
//! arayüz testlerinin fikstürü: `app/ui/test/fixtures/sankranti.json`.
//! Güncellemek için: `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test sankranti`.
//! Tek test: süreçte yalnızca bir SE örneği açılabilir. Çapraz denetimler ephemeris'in kendi
//! boylamına, olay aramasının Güneş burç girişlerine ve v1'in panchangına dayanır; beklenen
//! değerler başka bir yazılımdan alınmaz (K9).

use std::path::PathBuf;
use std::time::{Duration, Instant};

use atlas::Timestamp;
use ephemeris::{Body, Engine, Ephemeris, SiderealMode, SwissEph, Zodiac};
use jyotish_core::{
    event_chunks, event_search_chunk, CalcSettings, ChartWarning, EventSpec, Graha, Sign,
    SkyEventKind, ZodiacMode, SANKRANTI_METHOD,
};
use jyotish_desk_lib::commands::compute_chart_blocking;
use jyotish_desk_lib::muhurta_commands::MuhurtaPlace;
use jyotish_desk_lib::transit_tools_commands::sankranti::{
    sankranti_list_blocking, SankrantiListView,
};
use serde_json::{json, Value};
use store::TzChoice;

/// Bütçe (D17): bir yılın listesi, hata ayıklama derlemesinde.
const BUDGET: Duration = Duration::from_millis(100);
const UNIX_EPOCH_JD: f64 = 2_440_587.5;

fn delhi() -> MuhurtaPlace {
    MuhurtaPlace {
        lat: 28.6139,
        lon: 77.2090,
        tz: TzChoice::Iana("Asia/Kolkata".into()),
    }
}

/// Kutup dairesinin kuzeyi: aralık ortasında gün doğmaz (tests/golden/tromso-2050-12-21.json yeri).
fn tromso() -> MuhurtaPlace {
    MuhurtaPlace {
        lat: 69.6492,
        lon: 18.9553,
        tz: TzChoice::Iana("Europe/Oslo".into()),
    }
}

/// 2026-11-24T00:00:00Z.
fn now() -> Timestamp {
    Timestamp::from_second(1_795_478_400).expect("timestamp")
}

/// Anın yerdeki takvim günü (`YYYY-MM-DD`), satırın kendi ofsetiyle.
fn local_date(jd_ut: f64, offset_secs: i32) -> String {
    let secs = ((jd_ut - UNIX_EPOCH_JD) * 86_400.0).floor() as i64 + i64::from(offset_secs);
    Timestamp::from_second(secs)
        .expect("timestamp")
        .strftime("%Y-%m-%d")
        .to_string()
}

fn list(
    engine: &Engine,
    year: Option<i32>,
    place: MuhurtaPlace,
    settings: CalcSettings,
) -> SankrantiListView {
    sankranti_list_blocking(engine, year, place, settings, now()).expect("sankranti list")
}

#[test]
fn sankranti_command_runs_on_the_engine_thread_and_maps_errors() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let engine = Engine::spawn(root.join("../../data/ephe")).expect("spawn engine");
    let settings = CalcSettings::default();

    let started = Instant::now();
    let view = list(&engine, Some(2026), delhi(), settings.clone());
    let spent = started.elapsed();
    eprintln!("sankranti_list(2026): {spent:?}");
    assert!(spent <= BUDGET, "sankranti_list took {spent:?}");

    assert_eq!((view.year, view.method.as_str()), (2026, SANKRANTI_METHOD));
    assert!(view.warnings.is_empty());
    // Yıl yerin diliminde 1 Ocak 00:00'dan sonraki 1 Ocak 00:00'a: 2026 artık yıl değildir.
    assert!((view.to_jd_ut - view.from_jd_ut - 365.0).abs() < 1e-9);
    assert_eq!(local_date(view.from_jd_ut, 19_800), "2026-01-01");

    // On iki giriş, burç sırasıyla; yılın ilki Oğlak'tır (Makara). Türetme: Güneş tropikal
    // 270°'ye Aralık gündönümünde (21–22 Aralık) varır; Lahiri ayanamsası 2026'da yaklaşık
    // 24,2°'dir, yani sidereal 270° tropikal 294,2°'dir. Güneş Ocak'ta günde yaklaşık 1,02°
    // yürür: 24,2 / 1,02 ≈ 23,7 gün → 14 Ocak dolayı. Pencere: 13–16 Ocak.
    assert_eq!(view.rows.len(), 12);
    for (k, row) in view.rows.iter().enumerate() {
        assert_eq!(
            row.sankranti.sign,
            Sign::from_index((Sign::Capricorn.index() + k) % 12),
            "row {k}"
        );
        assert_eq!(row.offset_secs, 19_800, "row {k}");
        assert!(view.from_jd_ut <= row.sankranti.jd_ut && row.sankranti.jd_ut < view.to_jd_ut);
    }
    let first = local_date(view.rows[0].sankranti.jd_ut, view.rows[0].offset_secs);
    assert!(
        ("2026-01-13".."2026-01-17").contains(&first.as_str()),
        "{first}"
    );
    // Ardışık girişler 29–32 gün arayla gelir (Güneş'in hızı yıl içinde değişir).
    for w in view.rows.windows(2) {
        let gap = w[1].sankranti.jd_ut - w[0].sankranti.jd_ut;
        assert!((29.0..32.0).contains(&gap), "{gap}");
    }

    // İkinci yazım 1: ephemeris'in sidereal Güneş boylamı giriş anında burcun başındadır.
    let moments: Vec<(f64, Sign)> = view
        .rows
        .iter()
        .map(|r| (r.sankranti.jd_ut, r.sankranti.sign))
        .collect();
    let lons = {
        let moments = moments.clone();
        engine
            .call(move |se: &mut SwissEph| {
                moments
                    .iter()
                    .map(|(jd, _)| {
                        se.body(*jd, Body::Sun, Zodiac::Sidereal(SiderealMode::Lahiri))
                            .map(|p| p.lon)
                    })
                    .collect::<Result<Vec<f64>, _>>()
            })
            .expect("engine")
            .expect("sun")
    };
    for ((_, sign), lon) in moments.iter().zip(&lons) {
        let start = 30.0 * sign.index() as f64;
        let off = (lon - start + 180.0).rem_euclid(360.0) - 180.0;
        assert!(off.abs() < 1e-5, "{sign:?}: {off}");
    }

    // İkinci yazım 2: olay aramasının Güneş burç girişleri (sidereal kipte rasi çerçevesi
    // sidereal çerçevedir) aynı anları verir.
    let spec = EventSpec {
        from_jd_ut: view.from_jd_ut,
        to_jd_ut: view.to_jd_ut,
        kinds: vec![SkyEventKind::SignIngress],
        grahas: vec![Graha::Sun],
    };
    let events = {
        let (spec, settings) = (spec.clone(), settings.clone());
        engine
            .call(move |se: &mut SwissEph| {
                let mut out = Vec::new();
                for chunk in event_chunks(&spec) {
                    out.extend(event_search_chunk(se, &settings, &spec, None, chunk)?);
                }
                Ok::<_, jyotish_core::CoreError>(out)
            })
            .expect("engine")
            .expect("events")
    };
    assert_eq!(events.len(), 12);
    for (event, (jd, sign)) in events.iter().zip(&moments) {
        assert_eq!(event.sign, Some(*sign));
        assert!((event.jd_ut - jd).abs() < 2.0 / 86_400.0, "{sign:?}");
    }

    // Giriş anının tithisi, varası ve Ay nakshatrası v1'in o an ve yer için hesapladığı
    // haritanın panchangıdır.
    for row in &view.rows {
        let s = &row.sankranti;
        let chart = compute_chart_blocking(&engine, s.jd_ut, 28.6139, 77.2090, settings.clone())
            .expect("chart");
        let p = &chart.panchang;
        assert_eq!((s.tithi, s.paksha), (p.tithi.no, p.tithi.paksha));
        assert_eq!(s.vara, p.vara);
        assert_eq!((s.nakshatra, s.pada), (p.nakshatra.no, p.nakshatra.pada));
        assert_eq!(
            (s.nakshatra, s.pada),
            (chart.grahas[1].nakshatra, chart.grahas[1].pada)
        );
    }

    // Liste zodyak kipine bakmaz: tropikal-rasi kipinde de sidereal girişlerdir.
    let tropical = CalcSettings {
        zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
        ..settings.clone()
    };
    let tropical_view = list(&engine, Some(2026), delhi(), tropical);
    assert_eq!(tropical_view, view);

    // Yıl verilmezse yerin bugünkü yılıdır. 2026-12-31 19:00 UT Delhi'de 1 Ocak 2027 00:30'dur.
    assert_eq!(list(&engine, None, delhi(), settings.clone()).year, 2026);
    let new_year = Timestamp::from_second(1_798_743_600).expect("timestamp");
    let next =
        sankranti_list_blocking(&engine, None, delhi(), settings.clone(), new_year).expect("list");
    assert_eq!(next.year, 2027);

    // Kutup gecesi: Yay girişi (Aralık ortası) Tromsø'da gün doğmayan bir güne düşer.
    let polar = list(&engine, Some(2026), tromso(), settings.clone());
    assert_eq!(polar.rows.len(), 12);
    assert_eq!(polar.warnings, [ChartWarning::SunriseUnavailable]);
    // Anlar yere bağlı değildir; yılın uçları yerin dilimine göre kayar.
    for (a, b) in polar.rows.iter().zip(&view.rows) {
        assert_eq!(a.sankranti.sign, b.sankranti.sign);
        assert!((a.sankranti.jd_ut - b.sankranti.jd_ut).abs() < 1e-6);
    }

    // Hatalar yalnızca alan adını taşır; yer hataya girmez.
    let bad_place = MuhurtaPlace {
        lat: 95.0,
        ..delhi()
    };
    let e = sankranti_list_blocking(&engine, Some(2026), bad_place, settings.clone(), now())
        .unwrap_err();
    assert_eq!(e.code, "invalid_transit_input");
    assert_eq!(e.params.len(), 1);
    assert_eq!(e.params["field"], "place");
    let bad_zone = MuhurtaPlace {
        tz: TzChoice::Iana("Mars/Olympus".into()),
        ..delhi()
    };
    let e = sankranti_list_blocking(&engine, Some(2026), bad_zone, settings.clone(), now())
        .unwrap_err();
    assert_eq!(
        (e.code.as_str(), e.params["field"].as_str()),
        ("invalid_transit_input", "place")
    );
    for year in [0, -44, 3000, i32::MAX] {
        let e = sankranti_list_blocking(&engine, Some(year), delhi(), settings.clone(), now())
            .unwrap_err();
        assert_eq!(e.code, "invalid_transit_input", "{year}");
        assert_eq!(e.params.len(), 1);
        assert_eq!(e.params["field"], "year");
    }
    let old = CalcSettings {
        schema_version: 1,
        ..settings.clone()
    };
    let e = sankranti_list_blocking(&engine, Some(2026), delhi(), old, now()).unwrap_err();
    assert_eq!(e.code, "unsupported_schema_version");

    // Fikstür: Delhi 2026 ve Tromsø 2026 (gün doğumu uyarısıyla).
    let fresh = json!({ "place": delhi(), "list": view, "list_polar": polar });
    let path = root.join("../ui/test/fixtures/sankranti.json");
    if std::env::var_os("UPDATE_FIXTURES").is_some() {
        let text = serde_json::to_string(&fresh).expect("serialize fixture") + "\n";
        std::fs::write(&path, text).expect("write fixture");
        return;
    }
    let committed: Value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(Value::Null);
    assert!(
        approx_eq(&committed, &fresh),
        "app/ui/test/fixtures/sankranti.json is stale; run: UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test sankranti"
    );
}

/// Yapısal eşitlik; sayılar göreli 1e-9 toleransla, diğer değerler birebir (`ui_fixture.rs`
/// ile aynı kural).
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
````

- [ ] **Step 2: Testlerin derlenmediğini gör**

```bash
cargo test -p jyotish-core --lib transit_tools 2>&1 | grep -E "^error" | head -3
```
Expected: `error[E0432]` (çözülemeyen `sankranti` adları).

- [ ] **Step 3: Çekirdek ve komut**

**`crates/jyotish-core/src/transit_tools/sankranti.rs`: uygulama, test bloğunun üstüne (arada bir boş satır)**

````rust
//! Sankranti listesi (yol haritası §4.8): bir aralıkta Güneş'in sidereal burç girişleri ve her
//! giriş anının tithisi, varası ve Ay nakshatrası.
//!
//! Sankranti sidereal bir kavramdır: liste her zodyak kipinde, seçili ayanamsayla sidereal
//! boylamdan kurulur. Tropikal-rasi kipinde olay aramasının Güneş burç girişleri (rasi çerçevesi,
//! K5) başka anlardır; bu liste onlardan bilerek ayrılır. Giriş anı `timeline` yürüyücüsüyle
//! bulunur (çözünürlük 0,05 sn); giriş anının panchangı v1'in `compute_panchang`'ıdır: vara gün
//! doğumunda değişir ve yerin gün doğumuna göre okunur. Girişin kutlu süresi hesaplanmaz.

use ephemeris::{Body, Ephemeris, Precision};
use serde::{Deserialize, Serialize};

use crate::chart::ChartWarning;
use crate::panchang::{compute_panchang, Paksha, Vara};
use crate::settings::{CalcSettings, JD_UT_RANGE};
use crate::timeline::{lon_of, sidereal, walk, GRAHA_STEP_DAYS};
use crate::zodiac::Sign;
use crate::CoreError;

/// Yöntem kimliği (K7, Y5): sidereal giriş, giriş anının panchangı.
pub const SANKRANTI_METHOD: &str = "sankranti.sidereal_v1";
/// Bir çağrının en uzun aralığı, gün: bir takvim yılı ve pay.
pub const SANKRANTI_MAX_RANGE_DAYS: f64 = 400.0;

/// Güneş'in bir sidereal burca girişi ve o anın olguları.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Sankranti {
    pub jd_ut: f64,
    /// Girilen sidereal burç.
    pub sign: Sign,
    /// Giriş anının tithisi, 1..=30.
    pub tithi: u8,
    pub paksha: Paksha,
    /// Giriş anını içeren günün varası (gün, yerin gün doğumunda başlar).
    pub vara: Vara,
    /// Giriş anında Ay'ın nakshatrası (1..=27) ve padası (1..=4).
    pub nakshatra: u8,
    pub pada: u8,
}

/// Bir aralığın Sankranti listesi.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SankrantiList {
    /// Zaman sırasıyla.
    pub rows: Vec<Sankranti>,
    /// `LowPrecision`: bir giriş Moshier yedeğiyle hesaplandı. `SunriseUnavailable`: bir giriş
    /// gününde gün doğumu yok; o satırın varası 06:00 yerel ortalama saatte başlayan gündendir.
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

fn invalid(field: &'static str) -> CoreError {
    CoreError::InvalidTransitInput(field)
}

/// `[from_jd_ut, to_jd_ut]` aralığında Güneş'in sidereal burç girişleri: (an, girilen burç),
/// zaman sırasıyla. Aralığın ilk anındaki burç giriş sayılmaz.
pub fn sankranti_moments<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<Vec<(f64, Sign)>, CoreError> {
    settings.validate()?;
    // NaN ve sonsuz da aralık dışındadır.
    let in_range = JD_UT_RANGE.contains(&from_jd_ut) && JD_UT_RANGE.contains(&to_jd_ut);
    if !in_range || from_jd_ut >= to_jd_ut || to_jd_ut - from_jd_ut > SANKRANTI_MAX_RANGE_DAYS {
        return Err(invalid("range"));
    }
    let zodiac = sidereal(settings);
    let series = walk(eph, from_jd_ut, to_jd_ut, GRAHA_STEP_DAYS, |eph, jd| {
        Ok(Sign::of(lon_of(eph, jd, Body::Sun, zodiac)?))
    })?;
    Ok(series
        .windows(2)
        .map(|w| (w[1].start_jd_ut, w[1].value))
        .collect())
}

/// Aralıktaki Sankranti listesi. `geo_lat` (kuzey +) ve `geo_lon` (doğu +) varanın okunduğu
/// yerdir. Girdi hatası yalnızca alan adını taşır (`range`, `place`): yer hataya girmez.
pub fn sankranti_list<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    geo_lat: f64,
    geo_lon: f64,
    from_jd_ut: f64,
    to_jd_ut: f64,
) -> Result<SankrantiList, CoreError> {
    settings.validate()?;
    if !((-90.0..=90.0).contains(&geo_lat) && (-180.0..=180.0).contains(&geo_lon)) {
        return Err(invalid("place"));
    }
    let zodiac = sidereal(settings);
    let (mut low_precision, mut no_sunrise) = (false, false);
    let mut rows = Vec::new();
    for (jd_ut, sign) in sankranti_moments(eph, settings, from_jd_ut, to_jd_ut)? {
        let p = compute_panchang(eph, jd_ut, geo_lat, geo_lon, settings)?;
        no_sunrise |= p.sunrise_jd_ut.is_none();
        // Hassasiyet Güneş ve Ay'dan okunur (`compute_chart` ile aynı kural).
        for body in [Body::Sun, Body::Moon] {
            low_precision |= eph.body(jd_ut, body, zodiac)?.precision == Precision::Moshier;
        }
        rows.push(Sankranti {
            jd_ut,
            sign,
            tithi: p.tithi.no,
            paksha: p.tithi.paksha,
            vara: p.vara,
            nakshatra: p.nakshatra.no,
            pada: p.nakshatra.pada,
        });
    }
    let mut warnings = Vec::new();
    if low_precision {
        warnings.push(ChartWarning::LowPrecision);
    }
    if no_sunrise {
        warnings.push(ChartWarning::SunriseUnavailable);
    }
    Ok(SankrantiList {
        rows,
        warnings,
        method: SANKRANTI_METHOD.to_owned(),
    })
}
````

**Yeni dosya:** `app/src-tauri/src/transit_tools_commands/mod.rs`

````rust
//! Geçiş araçlarının komutları (yol haritası Plan 26): Sankranti listesi. Kalıp muhurta
//! komutlarıyla aynıdır: hesap motor iş parçacığında, komut gövdesi `blocking` havuzunda, hata
//! `CommandError`; sonuç saklanmaz (K1). "Bu yıl" Rust'ta okunur; arayüz tarih hesabı yapmaz
//! (C4).
//!
//! Yer kişisel veridir: loglanmaz ve hata parametresine girmez. Bu modülün ürettiği hatalar
//! var olan `invalid_transit_input` kodunu ve yalnızca alan adını taşır.

pub mod sankranti;

use crate::commands::CommandError;

/// Yıl girdisinin aralığı (yerin takvim yılı). Hesap aralığı (`JD_UT_RANGE`) daha geniştir;
/// sınır, tarihin dört basamakla yazılabildiği yıllardır.
pub const TOOLS_YEAR_RANGE: std::ops::RangeInclusive<i32> = 1..=2999;

pub(crate) fn invalid(field: &str) -> CommandError {
    CommandError::new("invalid_transit_input", &[("field", field.to_owned())])
}
````

**Yeni dosya:** `app/src-tauri/src/transit_tools_commands/sankranti.rs`

````rust
//! Sankranti listesi komutu (yol haritası §4.8): yerin bir takvim yılındaki güneş girişleri.
//! Yıl, yerin saat dilimindeki 1 Ocak 00:00'dan sonraki 1 Ocak 00:00'a dektir; saatler yerin
//! diliminde gösterilsin diye her satır o andaki UTC ofsetini taşır.

use std::sync::Arc;

use atlas::Timestamp;
use ephemeris::{Engine, SwissEph};
use jyotish_core::{CalcSettings, ChartWarning, Sankranti};
use serde::Serialize;
use specta::Type;
use tauri::State;

use super::{invalid, TOOLS_YEAR_RANGE};
use crate::client_commands::blocking;
use crate::commands::CommandError;
use crate::muhurta_commands::{day_start_jd, offset_at, today, MuhurtaPlace, J2000_JD};

/// Bir Sankranti ve o andaki UTC ofseti (saat, yerin diliminde gösterilir).
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct SankrantiView {
    pub sankranti: Sankranti,
    pub offset_secs: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct SankrantiListView {
    /// Listenin yılı (yıl verilmediyse yerin bugünkü yılı).
    pub year: i32,
    /// Zaman sırasıyla; olağan bir yılda on iki satır.
    pub rows: Vec<SankrantiView>,
    /// Yılın uçları: yerin 1 Ocak 00:00'ı ve sonraki 1 Ocak 00:00'ı.
    pub from_jd_ut: f64,
    pub to_jd_ut: f64,
    /// `LowPrecision` ya da `SunriseUnavailable`.
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

/// Yerin takvim yılının başı (1 Ocak 00:00 duvar saati).
fn year_start_jd(year: i32, place: &MuhurtaPlace) -> Result<f64, CommandError> {
    day_start_jd(&format!("{year:04}-01-01"), place, "year").map_err(|_| invalid("year"))
}

/// `sankranti_list` gövdesi; `now` testte sabitlenir. Bloklar.
pub fn sankranti_list_blocking(
    engine: &Engine,
    year: Option<i32>,
    place: MuhurtaPlace,
    settings: CalcSettings,
    now: Timestamp,
) -> Result<SankrantiListView, CommandError> {
    // Yer, yıldan ve motordan önce denetlenir: geçersiz yer hangi yılla sorulursa sorulsun
    // `place`'tir.
    offset_at(&place, J2000_JD).map_err(|_| invalid("place"))?;
    let year = match year {
        Some(y) => y,
        None => {
            let date: atlas::Date = today(&place, now)
                .map_err(|_| invalid("place"))?
                .parse()
                .map_err(|_| invalid("year"))?;
            i32::from(date.year())
        }
    };
    if !TOOLS_YEAR_RANGE.contains(&year) {
        return Err(invalid("year"));
    }
    let from_jd_ut = year_start_jd(year, &place)?;
    let to_jd_ut = year_start_jd(year + 1, &place)?;
    let (lat, lon) = (place.lat, place.lon);
    let list = engine.call(move |se: &mut SwissEph| {
        jyotish_core::sankranti_list(se, &settings, lat, lon, from_jd_ut, to_jd_ut)
    })??;
    let rows = list
        .rows
        .into_iter()
        .map(|sankranti| {
            Ok(SankrantiView {
                offset_secs: offset_at(&place, sankranti.jd_ut).map_err(|_| invalid("place"))?,
                sankranti,
            })
        })
        .collect::<Result<Vec<_>, CommandError>>()?;
    Ok(SankrantiListView {
        year,
        rows,
        from_jd_ut,
        to_jd_ut,
        warnings: list.warnings,
        method: list.method,
    })
}

/// Sankranti listesi: yerin `year` takvim yılında Güneş'in sidereal burç girişleri (haritanın
/// zodyak kipi ne olursa olsun seçili ayanamsayla sidereal) ve her giriş anının tithisi, varası
/// ve Ay nakshatrası. `year` boşsa yerin bugünkü yılı ("bugün" Rust'ta okunur). Yer loglanmaz.
#[tauri::command]
#[specta::specta]
pub async fn sankranti_list(
    engine: State<'_, Arc<Engine>>,
    year: Option<i32>,
    place: MuhurtaPlace,
    settings: CalcSettings,
) -> Result<SankrantiListView, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || sankranti_list_blocking(&engine, year, place, settings, Timestamp::now()))
        .await
}
````

Komut satırı `transit_commands::transits_at,` satırından hemen sonradır ("Paylaşılan dosyalar").

````diff
diff --git a/app/src-tauri/src/lib.rs b/app/src-tauri/src/lib.rs
index 7538e67..efccb2c 100644
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -11,6 +11,7 @@ pub mod prashna_commands;
 pub mod search_commands;
 pub mod store_commands;
 pub mod transit_commands;
+pub mod transit_tools_commands;
 pub mod update_commands;
 
 use std::sync::{Arc, Mutex};
@@ -35,6 +36,7 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             commands::compute_chart,
             commands::compute_transits,
             transit_commands::transits_at,
+            transit_tools_commands::sankranti::sankranti_list,
             bhava_commands::compute_bhava_table,
             bhava_commands::compute_sudarshana,
             commands::dasha_children,
````

- [ ] **Step 4: Rust testleri, fikstür ve bağlamalar (C10)**

```bash
cargo fmt --all --check
cargo test -p jyotish-core --lib transit_tools 2>&1 | grep -E "^test result"
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test sankranti 2>&1 | grep -E "^test result"
cargo test -p jyotish-desk --test sankranti -- --nocapture 2>&1 | grep -E "^sankranti_list|^test result"
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
grep -c "sankrantiList:" app/ui/bindings.ts
```
Expected: fmt çıktısız; `10 passed`; `1 passed`; ölçüm satırı (bu plan yazılırken 7–8 ms; bütçe 100 ms) ve `1 passed`; `1 passed`; `1`. Fikstür ve bağlamalar elle düzenlenmez. Bir el örneği düşerse önce kural yeniden okunur; beklenen değer motor çıktısına bakarak düzeltilmez.

- [ ] **Step 5: Sarmalayıcı**

Sarmalayıcı ve iki test satırı `transitsAt` satırlarından hemen sonradır.

````diff
diff --git a/app/ui/api.test.ts b/app/ui/api.test.ts
index 3f5d97b..b2e3e8a 100644
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -66,6 +66,7 @@ vi.mock("./bindings", () => ({
     saveCalcSettings: vi.fn(),
     updateClient: vi.fn(),
     transitsAt: vi.fn(),
+    sankrantiList: vi.fn(),
     keepStoredMoment: vi.fn(),
     appInfo: vi.fn(),
     checkForUpdate: vi.fn(),
@@ -176,6 +177,7 @@ test("every wrapper forwards its arguments to the command of the same name", asy
     ["saveCalcSettings", [3, { ayanamsa: "Lahiri" }]],
     ["updateClient", [2, "A", null]],
     ["transitsAt", [{ kind: "now" }, { ayanamsa: "Lahiri" }, [], { birth_jd_ut: 1 }]],
+    ["sankrantiList", [2026, { lat: 28.6, lon: 77.2, tz: { Iana: "Asia/Kolkata" } }, { ayanamsa: "Lahiri" }]],
     ["keepStoredMoment", [3]],
     ["appInfo", []],
     ["checkForUpdate", []],
diff --git a/app/ui/api.ts b/app/ui/api.ts
index 8f186b3..c5626c6 100644
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -70,6 +70,9 @@ export const api = {
   getAppPrefs: () => wrap(() => commands.getAppPrefs()),
   setAppPrefs: (prefs: AppPrefs) => wrap(() => commands.setAppPrefs(prefs)),
   transitsAt: (at: TransitAt, settings: CalcSettings, natalBhavas: Bhavas, seed: DashaSeed) => wrap(() => commands.transitsAt(at, settings, natalBhavas, seed)),
+  // Geçiş araçları (yol haritası Plan 26). `year` boşsa yerin bugünkü yılıdır ("bugün" Rust'ta
+  // okunur). Yer kişisel veridir: loglanmaz.
+  sankrantiList: (year: number | null, place: MuhurtaPlace, settings: CalcSettings) => wrap(() => commands.sankrantiList(year, place, settings)),
   deleteClient: (clientId: number) => wrap(() => commands.deleteClient(clientId)),
   deleteChart: (chartId: number) => wrap(() => commands.deleteChart(chartId)),
   listClients: (limit: number) => wrap(() => commands.listClients(limit)),
````

- [ ] **Step 6: REVIEW kaydı: nakshatra lordları ve Sankranti'nin okuma kuralları**

**Yeni dosya:** `tests/golden/modules/transit_tools/REVIEW.md`

````markdown
# Geçiş araçları: okuma kuralları, onay bekleyen seçimler ve gözden geçirme kaydı

Kaynak: yol haritası (`docs/superpowers/specs/2026-11-27-eksikler-yol-haritasi.md`) §4.7–§4.10 ve
Plan 26'nın kararları. Çekirdek: `crates/jyotish-core/src/transit_tools/`; komutlar:
`app/src-tauri/src/transit_tools_commands/`.

Bu dizinde JSON örnek yoktur. El örnekleri birim testleridir (`lords.rs`, `sankranti.rs`,
`ephemeris_table.rs` ve `transit_tools_commands/step.rs` içindeki `mod tests`); her beklenen
değerin türetmesi testin yorumunda adım adım yazılıdır. Değerler kuraldan elle türetilmiştir;
motor çıktısı ve başka bir yazılımın çıktısı buraya ve testlere kopyalanmaz (K9). Bir değer
motorla uyuşmazsa önce kural yeniden okunur; beklenen değer motor çıktısına bakarak düzeltilmez.

**Bu planda bellekten yazılan klasik tablo yoktur.** Tek tablo (nakshatra lordları) v1'in
Vimshottari sırasının üç kez yazılmış biçimidir ve testte o sırayla karşılaştırılır; paneller
"doğrulanmamış tablo" notu taşımaz. Aşağıdaki "onay bekleyen" maddeler okuma seçimleridir (Y5).

## 1. Nakshatra lordları

1. **Tablo.** `NAKSHATRA_LORDS`: yirmi yedi nakshatranın lordu, nakshatra sırasıyla. Dokuzlu
   sıra (Ketu, Venüs, Güneş, Ay, Mars, Rahu, Jüpiter, Satürn, Merkür) üç kez yinelenir: n.
   nakshatranın lordu sıranın ((n − 1) mod 9). adıdır.
2. **Arayüz lordu türetmez.** Tablo bağlama dosyasına sabit olarak çıkar (`bindings.ts`,
   `NAKSHATRA_LORDS`); arayüz nakshatra numarasının satırını okur (`panels/nakshatraLord.ts`).
   `GrahaPosition`, `TransitGraha` ve `SkyEvent` tiplerine alan eklenmez.
3. **Üç yüzey.** Geçiş tablosunda "Nakshatra lordu" sütunu her zaman görünür. Graha tablosunda
   sütun isteğe bağlıdır (`settings.nakshatraLord`), varsayılan sütunlar ve baskı tablosu değişmez;
   lagna satırı da lordunu yazar; doğum saati bilinmiyorsa Ay'ın lordu "≈" taşır. Olay aramasında
   yalnızca nakshatra girişi satırı, girilen nakshatranın lordunu aynı hücrede yazar.
4. **Ay'ın önümüzdeki nakshatra girişleri** olay aramasının hazır seçimidir (tür: nakshatra
   girişi; graha: Ay); ayrı komut ve ayrı liste yoktur.

### 1.1 Onay bekleyen noktalar (sahibi)

- [ ] Graha tablosunda sütunun varsayılan olarak gizli olması (öteki okuma: varsayılan sütun).
- [ ] Olay aramasında lordun yalnızca nakshatra girişi satırında yazılması (öteki okuma: yeni ay
      ve dolunay satırında Ay'ın nakshatra lordu da yazılır).

## 2. Sankranti listesi (yöntem `sankranti.sidereal_v1`)

### 2.1 Okuma kuralları

1. **Giriş.** Sankranti, Güneş'in sidereal bir burca girdiği andır: sidereal boylamın 30°'nin
   katından geçişi. An `timeline` yürüyücüsüyle 0,05 saniye çözünürlükte bulunur.
2. **Çerçeve.** Liste her zodyak kipinde seçili ayanamsayla sidereal boylamdan kurulur.
   Tropikal-rasi kipinde haritanın burçları tropikaldir (K5) ve olay aramasının Güneş burç
   girişleri tropikal girişlerdir; Sankranti listesi bundan bilerek ayrılır.
3. **Yıl.** Yerin takvim yılı: yerin saat diliminde 1 Ocak 00:00'dan sonraki 1 Ocak 00:00'a.
   Olağan bir yılda on iki giriş vardır; yılın ilki Oğlak (Makara) girişidir.
4. **Giriş anının olguları.** Tithi (Ay − Güneş uzanımının 12°'lik dilimi) ve paksha, Ay'ın
   nakshatrası ve padası, vara. Üçü de v1'in `compute_panchang` hesabıdır. Vara gün doğumunda
   değişir ve seçilen yerin gün doğumuna göre okunur: gün doğumundan önceki bir giriş önceki
   günün varasını taşır.
5. **Gün doğumu olmayan gün.** Vara, günü 06:00 yerel ortalama saatte başlatarak okunur ve liste
   `SunriseUnavailable` uyarısını taşır (v1'in panchang kuralı).
6. **Saat.** Anlar seçilen yerin saat diliminde gösterilir; her satır kendi anının UTC ofsetini
   taşır (yaz saati yıl içinde değişebilir).
7. **Kapsam dışı.** Girişin kutlu süresi (punya dilimi), girişin yönü ve adı (ör. burca göre
   özel adlar) hesaplanmaz ve yazılmaz.

### 2.2 Onay bekleyen noktalar (sahibi)

- [ ] Listenin tropikal-rasi kipinde de sidereal verilmesi (K5'ten bilinçli ayrılma).
- [ ] Varanın gün doğumuna göre okunması (öteki okuma: yerin takvim günü, gece yarısından).
- [ ] Yılın, seçilen yerin takvim yılı olması (öteki okuma: Mesha girişinden Mesha girişine).
````

- [ ] **Step 7: Kapılar**

```bash
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -1
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
cargo test -p jyotish-core --lib 2>&1 | grep -E "^test result"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
grep -inE 'k(a|ā|aa)la|\bcala\b' tests/golden/modules/transit_tools/REVIEW.md
```
Expected: clippy `Finished`; sayaç `0`; `476 passed`; Vitest `100 passed (100)`, `986 passed (986)` (sarmalayıcı var olan tabloya girer); `✓ built`; tarama çıktısız.

- [ ] **Step 8: Commit**

```bash
git add crates/jyotish-core/src/lib.rs crates/jyotish-core/src/transit_tools app/src-tauri/src/lib.rs app/src-tauri/src/transit_tools_commands app/src-tauri/tests/sankranti.rs app/ui/bindings.ts app/ui/test/fixtures/sankranti.json app/ui/api.ts app/ui/api.test.ts tests/golden/modules/transit_tools/REVIEW.md
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(transit): sankranti list core and command with the panchang of each ingress, sidereal in every zodiac mode" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: on iki dosya; yasaklı ad taraması çıktısız; `git status --short` boş.


---

### Task 3: Arayüz: Sankranti paneli

Yol haritası §4.8 ve görev taslağının (3). maddesi; D10, D11, D19–D21. Görünüm ayrı bir paneldir (`sankranti`; paletten eklenir, K12). Yeni stil dosyası yoktur.

**Files:**
- Create: `app/ui/test/sankrantiFixture.ts`, `app/ui/modules/useSankranti.ts`, `app/ui/panels/SankrantiPanel.tsx`, `app/ui/panels/SankrantiPanel.test.tsx`, `app/e2e/sankranti.desk.spec.ts`
- Modify: `app/ui/layout/model.ts`, `app/ui/layout/model.test.ts`, `app/ui/panels/registry.ts`, `app/ui/panels/panels.test.tsx`, `app/ui/i18n/tr.ts`, `en.ts`, `fr.ts`, `app/e2e/ipc.ts`
- Test: `SankrantiPanel.test.tsx` (10); e2e +1

**Interfaces:**
- Consumes: `api.sankrantiList`; `usePanelPlace`, `PlaceControls`, `PlaceMissing` (`panels/MuhurtaPlace.tsx`); `useModule`, `useLastData`; `useTransitControl`; `BalaScroll`; `formatJdMinute`; `tithiInPaksha`, `tithiName`, `VARA_NAMES`, `nakshatraName`.
- Produces: `useSankrantiList(opened, place, year, scope)`; `SankrantiPanel`; `parseYear(text)`, `YEAR_MIN`, `YEAR_MAX` (Task 5 de kullanır); `sankrantiFixture()`; panel türü `sankranti` (7 × 16); i18n `panel.sankranti`, `sankranti.*`; e2e taklidi `sankranti_list` ve `FIXTURE.transit_tools.sankranti`.

- [ ] **Step 1: Testler: yer, bu yıl, yıl kutusu, yeniden okuma, transit tarihi, uyarılar, hata ve diller**

**Yeni dosya:** `app/ui/test/sankrantiFixture.ts`

````ts
// Sankranti fikstürü (app/src-tauri/tests/sankranti.rs üretir): Delhi için 2026 yılının
// `sankranti_list` dönüşü ve aynı yılın Tromsø listesi (bir giriş gün doğmayan bir güne düşer).
// Ayrı dosyadır: `delhi.json` ve `fixture.ts` eşzamanlı planlarla paylaşılır.
import type { MuhurtaPlace, SankrantiListView } from "../bindings";
import raw from "./fixtures/sankranti.json?raw";

export interface SankrantiFixture {
  /** Listenin yeri: Delhi, `Asia/Kolkata`. */
  place: MuhurtaPlace;
  /** `sankranti_list(2026, place, varsayılan ayarlar)`: on iki giriş, uyarı yok. */
  list: SankrantiListView;
  /** Aynı yıl, Tromsø: anlar aynıdır; `SunriseUnavailable` uyarısı vardır. */
  list_polar: SankrantiListView;
}

const FIXTURE = JSON.parse(raw) as SankrantiFixture;

/** Her çağrıda derin kopya: testler sonucu değiştirebilir. */
export function sankrantiFixture(): SankrantiFixture {
  return structuredClone(FIXTURE);
}
````

**Yeni dosya:** `app/ui/panels/SankrantiPanel.test.tsx`

````tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { sankrantiList: vi.fn(), searchPlaces: vi.fn(), setAppPrefs: vi.fn() } }));

import { api, type Result } from "../api";
import type { OpenedClient, SankrantiListView } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import type { Panel } from "../layout/model";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { sankrantiFixture } from "../test/sankrantiFixture";
import { useTransitControl, type Transit } from "../transit/store";
import { useLayoutStore } from "../view/store";
import { parseYear, SankrantiPanel, YEAR_MAX, YEAR_MIN } from "./SankrantiPanel";

const SX = sankrantiFixture();
const ok = (view: SankrantiListView): Result<SankrantiListView> => ({ status: "ok", data: view });

let transit: Transit;
function LivePanel() {
  const { active } = useLayoutStore();
  transit = useTransitControl();
  return <SankrantiPanel panel={active.panels[0]} />;
}
/** Varsayılan: yer haritanın doğum yeridir (fikstürde bulunulan yer seçilmemiştir). */
function renderPanel(settings: Record<string, unknown> = { place: "chart" }, opened: OpenedClient = fixture().opened) {
  const panel: Panel = { id: "sk", kind: "sankranti", x: 0, y: 0, w: 7, h: 16, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<LivePanel />, { layout, opened });
}
const rows = () => within(screen.getByRole("table")).getAllByRole("row").slice(1);
const cells = (row: HTMLElement) => [...row.querySelectorAll("th, td")].map((c) => c.textContent);
const yearBox = () => screen.getByRole("spinbutton", { name: "Yıl" });
const settle = () => act(async () => {});

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.sankrantiList).mockImplementation(async (year) => ok({ ...sankrantiFixture().list, year: year ?? 2026 }));
});

test("without a place nothing is asked and the panel says how to pick one", () => {
  renderPanel({});
  expect(screen.getByRole("combobox", { name: "Yer" })).toHaveDisplayValue("Bulunulan yer");
  expect(screen.getByRole("note")).toHaveTextContent(/Bulunulan yer seçilmedi/);
  expect(api.sankrantiList).not.toHaveBeenCalled();
  expect(screen.queryByRole("table")).toBeNull();
});

test("this year's list is asked with the place and the chart's settings; twelve rows in the place's time", async () => {
  renderPanel();
  const table = await screen.findByRole("table", { name: "Sankranti listesi, 2026: 12 giriş" });
  const opened = fixture().opened;
  const { lat, lon } = opened.chart_record.birth.place;
  expect(api.sankrantiList).toHaveBeenCalledTimes(1);
  // Yıl boştur: "bu yıl" Rust'ta okunur. Ayar haritanın hesap ayarıdır.
  expect(vi.mocked(api.sankrantiList).mock.calls[0]).toEqual([null, { lat, lon, tz: opened.chart_record.birth.tz }, opened.chart.meta.settings]);
  expect(within(table).getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Giriş anı", "Girilen burç", "Tithi", "Vara", "Ay nakshatrası"]);
  expect(rows()).toHaveLength(12);
  // İlk satır (fikstür, Delhi +05:30): Oğlak girişi 2026-01-14 15:07. Tithi 26, Krishna
  // pakshanın 26 − 15 = 11. tithisi (Ekadashi); 17. nakshatra Anuradha, 3. pada; Çarşamba.
  const first = SX.list.rows[0];
  expect([first.sankranti.sign, first.sankranti.tithi, first.sankranti.nakshatra, first.sankranti.pada, first.sankranti.vara, first.offset_secs]).toEqual(["Capricorn", 26, 17, 3, "Wednesday", 19_800]);
  expect(cells(rows()[0])).toEqual(["2026-01-14 15:07", "Oğl", "Krishna 11 · Ekadashi", "Budhavara · Çarşamba", "Anuradha · pada 3"]);
  // Burç sırası: Oğlak'tan Yay'a.
  expect(rows().map((r) => cells(r)[1])).toEqual(["Oğl", "Kov", "Bal", "Koç", "Boğ", "İki", "Yen", "Asl", "Baş", "Ter", "Akr", "Yay"]);
  expect(yearBox()).toHaveValue(2026);
  expect(screen.getByRole("button", { name: "Bu yıl" })).toHaveAttribute("aria-pressed", "true");
  expect(screen.getByText(/Yöntem: sankranti\.sidereal_v1/)).toBeInTheDocument();
  // Harita sidereal: çerçeve notu yok; uyarı yok.
  expect(screen.queryByRole("note")).toBeNull();
});

test("a typed year is asked on submit; an invalid year is not sent", async () => {
  renderPanel();
  await screen.findByRole("table");
  fireEvent.change(yearBox(), { target: { value: "2027" } });
  // Yazarken istek gitmez.
  expect(api.sankrantiList).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByRole("button", { name: "Yılı göster" }));
  await screen.findByRole("table", { name: "Sankranti listesi, 2027: 12 giriş" });
  expect(vi.mocked(api.sankrantiList).mock.calls.at(-1)![0]).toBe(2027);
  expect(screen.getByRole("button", { name: "Bu yıl" })).toHaveAttribute("aria-pressed", "false");
  for (const bad of ["0", "3000", "", "20.5"]) {
    fireEvent.change(yearBox(), { target: { value: bad } });
    expect(yearBox()).toHaveAttribute("aria-invalid", "true");
    fireEvent.click(screen.getByRole("button", { name: "Yılı göster" }));
  }
  await settle();
  expect(api.sankrantiList).toHaveBeenCalledTimes(2);
  // "Bu yıl" yılı boşaltır ve kutu Rust'ın döndürdüğü yılı gösterir.
  fireEvent.click(screen.getByRole("button", { name: "Bu yıl" }));
  await screen.findByRole("table", { name: "Sankranti listesi, 2026: 12 giriş" });
  expect(vi.mocked(api.sankrantiList).mock.calls.at(-1)![0]).toBeNull();
  expect(yearBox()).toHaveValue(2026);
  expect(yearBox()).not.toHaveAttribute("aria-invalid");
});

test("the table stays while another year is read", async () => {
  renderPanel();
  await screen.findByRole("table");
  let resolve!: (r: Result<SankrantiListView>) => void;
  vi.mocked(api.sankrantiList).mockImplementationOnce(() => new Promise((r) => (resolve = r)));
  fireEvent.change(yearBox(), { target: { value: "2030" } });
  fireEvent.click(screen.getByRole("button", { name: "Yılı göster" }));
  await settle();
  expect(screen.getByRole("table", { name: "Sankranti listesi, 2026: 12 giriş" })).toBeInTheDocument();
  await act(async () => resolve(ok({ ...sankrantiFixture().list, year: 2030 })));
  expect(screen.getByRole("table", { name: "Sankranti listesi, 2030: 12 giriş" })).toBeInTheDocument();
});

test("the moment in a row moves the transit date", async () => {
  renderPanel();
  await screen.findByRole("table");
  const button = within(rows()[3]).getByRole("button");
  expect(button).toHaveAccessibleName(/transit tarihini bu ana götür/);
  fireEvent.click(button);
  expect(transit.at).toEqual({ Jd: SX.list.rows[3].sankranti.jd_ut });
});

test("warnings are written out: a day without sunrise and the fallback ephemeris", async () => {
  vi.mocked(api.sankrantiList).mockResolvedValue(ok({ ...sankrantiFixture().list_polar, warnings: ["LowPrecision", "SunriseUnavailable"] }));
  renderPanel();
  await screen.findByRole("table");
  expect(SX.list_polar.warnings).toEqual(["SunriseUnavailable"]);
  const notes = screen.getAllByRole("note").map((n) => n.textContent);
  expect(notes).toEqual([expect.stringMatching(/^Düşük hassasiyet: /), expect.stringMatching(/^Gün doğumu yok: Girişlerden en az birinin gününde bu yerde gün doğmuyor/)]);
});

test("a chart with tropical signs is told that the list is sidereal", async () => {
  const opened = fixture().opened;
  opened.chart.meta.settings.zodiac = "TropicalRasiSiderealNakshatra";
  renderPanel({ place: "chart" }, opened);
  await screen.findByRole("table");
  expect(screen.getByRole("note")).toHaveTextContent("Harita tropikal burç kullanıyor; bu liste her durumda seçili ayanamsayla sidereal girişleri verir.");
  expect(vi.mocked(api.sankrantiList).mock.calls[0][2].zodiac).toBe("TropicalRasiSiderealNakshatra");
});

test("an input error names the field in the interface language and can be retried", async () => {
  vi.mocked(api.sankrantiList).mockResolvedValueOnce({ status: "error", error: { code: "invalid_transit_input", params: { field: "year" } } });
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Transit isteği geçersiz: yıl.");
  expect(screen.queryByRole("table")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await waitFor(() => expect(screen.getByRole("table")).toBeInTheDocument());
  expect(screen.queryByRole("alert")).toBeNull();
});

test("the year box accepts whole years in the range only", () => {
  expect([YEAR_MIN, YEAR_MAX]).toEqual([1, 2999]);
  expect(["1", "2026", " 2999 ", "0001"].map(parseYear)).toEqual([1, 2026, 2999, 1]);
  expect(["0", "3000", "", "20.5", "-5", "2e3", "12345", "yıl"].map(parseYear)).toEqual([null, null, null, null, null, null, null, null]);
});

test("the texts exist in the three languages; French uses the typographic apostrophe", () => {
  expect([tr.panel.sankranti, en.panel.sankranti, fr.panel.sankranti]).toEqual(["Sankranti", "Sankranti", "Sankranti"]);
  for (const m of [tr, en, fr]) {
    expect(Object.keys(m.sankranti).sort()).toEqual(Object.keys(tr.sankranti).sort());
    expect(m.sankranti.method).toContain("{{method}}");
    expect(m.sankranti.caption).toContain("{{year}}");
  }
  expect(JSON.stringify(fr.sankranti)).not.toContain("'");
  expect(fr.sankranti.show).toBe("Afficher l’année");
});
````

`model.test.ts`'in tek satırlık `MODULE_PANELS` beklentisi bu planın değiştirdiği tek var olan satırdır.

````diff
diff --git a/app/ui/layout/model.test.ts b/app/ui/layout/model.test.ts
index 4fc9d72..b73fd97 100644
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -56,6 +56,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
     "muhurta_search",
     "prashna_facts",
     "event_search",
+    "sankranti",
     "navatara",
     "vimshopaka",
   ]);
@@ -65,7 +66,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
 });
 
 test("module panels are known kinds, listed once, with a size the grid accepts", () => {
-  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "bhava_table", "sudarshana", "yogas", "avastha", "compat", "muhurta_day", "muhurta_search", "prashna_facts", "event_search", "navatara", "vimshopaka"]);
+  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "bhava_table", "sudarshana", "yogas", "avastha", "compat", "muhurta_day", "muhurta_search", "prashna_facts", "event_search", "sankranti", "navatara", "vimshopaka"]);
   for (const m of MODULE_PANELS) {
     expect(isKnownPanelKind(m.kind)).toBe(true);
     expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
diff --git a/app/ui/panels/panels.test.tsx b/app/ui/panels/panels.test.tsx
index 03d8c17..179f7e2 100644
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -25,6 +25,7 @@ import { MuhurtaSearchPanel } from "./MuhurtaSearchPanel";
 import { NavataraPanel } from "./NavataraPanel";
 import { PanchangPanel } from "./PanchangPanel";
 import { panelDefinition, PANEL_REGISTRY } from "./registry";
+import { SankrantiPanel } from "./SankrantiPanel";
 import { ShadbalaPanel } from "./ShadbalaPanel";
 import { SudarshanaPanel } from "./SudarshanaPanel";
 import { UnavailablePanel } from "./UnavailablePanel";
@@ -66,6 +67,7 @@ test("registry covers every catalog kind", () => {
   expect(PANEL_REGISTRY.muhurta_day.component).toBe(MuhurtaDayPanel);
   expect(PANEL_REGISTRY.muhurta_search.component).toBe(MuhurtaSearchPanel);
   expect(PANEL_REGISTRY.event_search.component).toBe(EventSearchPanel);
+  expect(PANEL_REGISTRY.sankranti.component).toBe(SankrantiPanel);
   expect(PANEL_REGISTRY.navatara.component).toBe(NavataraPanel);
   expect(PANEL_REGISTRY.vimshopaka.component).toBe(VimshopakaPanel);
   // Kataloğun her türü uygulanmıştır: yer tutucu yoktur.
````

**Yeni dosya:** `app/e2e/sankranti.desk.spec.ts`

````ts
// Sankranti akışı (Plan 26). Ayrı dosyadır: `desk.spec.ts` eşzamanlı planlarla paylaşılır; "e2e"
// projesi adı `desk.spec.ts` ile biten her dosyayı koşar.
import { expect, test } from "@playwright/test";
import { FIXTURE, ipcCalls, mockIpc } from "./ipc";

type SankrantiCall = { year: number | null; place: { lat: number; lon: number; tz: unknown }; settings: { schema_version: number } };

test("sankranti: a palette panel that lists the year's twelve ingresses in the place's time and moves the transit date", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  const calls = async () => (await ipcCalls(page, "sankranti_list")) as SankrantiCall[];
  // Liste panel açılmadan istenmez (ileri modüller spec'i K1).
  expect(await calls()).toHaveLength(0);

  await page.keyboard.press("Meta+k");
  const palette = page.getByRole("dialog", { name: "Komut paleti" });
  await palette.getByRole("combobox").fill("sankranti");
  await palette.getByRole("option", { name: /Sankranti panelini aç\/kapat/ }).click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  const panel = page.getByRole("region", { name: "Sankranti" });
  await expect(panel).toBeFocused();
  await expect(panel).toBeInViewport();

  // Bulunulan yer seçilmemiştir: liste istenmez, panel nedenini söyler.
  await expect(panel.getByRole("note")).toContainText("Bulunulan yer seçilmedi");
  expect(await calls()).toHaveLength(0);
  await panel.getByRole("combobox", { name: "Yer" }).selectOption("chart");

  const list = FIXTURE.transit_tools.sankranti.list;
  const table = panel.getByRole("table", { name: `Sankranti listesi, ${list.year}: 12 giriş` });
  await expect(table.locator("tbody tr")).toHaveCount(12);
  const sent = await calls();
  expect(sent).toHaveLength(1);
  // Yıl boştur ("bu yıl" Rust'ta okunur); yer haritanın doğum yeridir.
  expect(sent[0].year).toBeNull();
  expect(sent[0].place).toMatchObject({ lat: 28.6139, lon: 77.209, tz: { Iana: "Asia/Kolkata" } });
  expect(sent[0].settings.schema_version).toBe(2);
  // Yılın ilk girişi Oğlak'tır; saat yerin dilimindedir (+05:30).
  await expect(table.locator("tbody tr").first().locator("th, td")).toHaveText(["2026-01-14 15:07", "Oğl", "Krishna 11 · Ekadashi", "Budhavara · Çarşamba", "Anuradha · pada 3"]);
  await expect(panel.getByRole("spinbutton", { name: "Yıl" })).toHaveValue("2026");

  // Başka bir yıl: istek yılı taşır.
  await panel.getByRole("spinbutton", { name: "Yıl" }).fill("2031");
  await panel.getByRole("button", { name: "Yılı göster" }).click();
  await expect(panel.getByRole("table", { name: "Sankranti listesi, 2031: 12 giriş" })).toBeVisible();
  expect((await calls()).at(-1)!.year).toBe(2031);

  // Satırdaki an transit tarihini o ana götürür; liste ve odak yerinde durur.
  const before = (await ipcCalls(page, "transits_at")).length;
  const first = panel.getByRole("table").locator("tbody tr").first().getByRole("button");
  await first.focus();
  await page.keyboard.press("Enter");
  await expect.poll(async () => (await ipcCalls(page, "transits_at")).length).toBe(before + 1);
  const moved = (await ipcCalls(page, "transits_at")).at(-1) as { at: { Jd: number } };
  expect(moved.at).toEqual({ Jd: list.rows[0].sankranti.jd_ut });
  await expect(panel.getByRole("table").locator("tbody tr")).toHaveCount(12);
  await expect(first).toBeFocused();
});
````

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/panels/SankrantiPanel.test.tsx ui/layout/model.test.ts ui/panels/panels.test.tsx 2>&1 | grep -E "Test Files|Tests "
```
Expected: üç dosya da düşer.

- [ ] **Step 3: Kanca, panel, katalog, metinler ve e2e taklidi**

**Yeni dosya:** `app/ui/modules/useSankranti.ts`

````ts
// Sankranti listesini (`sankranti_list`) isteyen kanca (yol haritası Plan 26, §4.8). Sonuç
// saklanmaz ve arayüzde hesap yoktur (K1, C4); panel açıkken istenir. `year === null` yerin
// bugünkü yılıdır: "bugün" Rust'ta okunur. Yeni bir yıl okunurken aynı panelin son listesi
// gösterilmeyi sürdürür (tablo ve odak yerinde durur). Yer kişisel veridir: anahtar ve
// girdiler loglanmaz.
import { api } from "../api";
import type { HomePlace, OpenedClient, SankrantiListView } from "../bindings";
import { muhurtaPlace } from "./muhurta";
import { useLastData, useModule, type ModuleState } from "./useModule";

export interface SankrantiState extends ModuleState<SankrantiListView> {
  /** Gösterilecek liste: yeni yanıt gelene dek bir önceki. */
  shown: SankrantiListView | null;
}

/** `place === null`: yer seçilmemiş, komut çağrılmaz. `scope` panelin kimliğidir. */
export function useSankrantiList(opened: OpenedClient, place: HomePlace | null, year: number | null, scope: string): SankrantiState {
  const settings = opened.chart.meta.settings;
  const key = place !== null ? JSON.stringify([opened.chart_record.id, muhurtaPlace(place), year, settings]) : null;
  const state = useModule(key, () => api.sankrantiList(year, muhurtaPlace(place as HomePlace), settings));
  const shown = useLastData(state, key === null ? null : `${opened.chart_record.id}:${scope}`);
  return { ...state, shown };
}
````

**Yeni dosya:** `app/ui/panels/SankrantiPanel.tsx`

````tsx
// Sankranti paneli (yol haritası Plan 26, §4.8): seçilen yılın güneş girişleri ve her giriş anının
// tithisi, varası ve Ay nakshatrası. Liste `sankranti_list`'ten gelir ve panel açıkken istenir
// (K1); burada hesap yoktur (C4). Girişler haritanın zodyak kipi ne olursa olsun sidereal
// burçlaradır; harita tropikal burç kullanıyorsa panel bunu sözle söyler. Yıl saklanmaz: boş
// yıl "bu yıl"dır ve Rust'ta okunur. Saatler seçilen yerin dilimindedir (varsayılan: bulunulan
// yer); satırdaki an transit tarihini o ana götürür. Liste olgudur: hüküm ve açıklama yoktur.
import { useId, useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import type { CommandError, OpenedClient, SankrantiListView } from "../bindings";
import { signIndex } from "../chart/adapter";
import type { Panel } from "../layout/model";
import { useSankrantiList } from "../modules/useSankranti";
import { useSession } from "../session/store";
import { useTransitControl } from "../transit/store";
import { BalaScroll } from "./bala";
import { formatJdMinute } from "./format";
import { nakshatraName, tithiInPaksha, tithiName, VARA_NAMES } from "./labels";
import { PlaceControls, PlaceMissing, usePanelPlace } from "./MuhurtaPlace";
import "./panels.css";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

/** Rust `TOOLS_YEAR_RANGE`: yıl kutusunun sınırları. */
export const YEAR_MIN = 1;
export const YEAR_MAX = 2999;

/** Yıl kutusunun metni → yıl; tam sayı değilse ya da aralık dışındaysa `null`. */
export function parseYear(text: string): number | null {
  if (!/^\d{1,4}$/.test(text.trim())) return null;
  const year = Number(text.trim());
  return year >= YEAR_MIN && year <= YEAR_MAX ? year : null;
}

export function SankrantiPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <Body panel={panel} opened={opened} />;
}

/** `invalid_transit_input`'un alan adını arayüz dilinde yazar; öteki hatalar olduğu gibi durur. */
function fieldError(error: CommandError, name: (field: string) => string): CommandError {
  const field = error.params.field;
  if (error.code !== "invalid_transit_input" || typeof field !== "string") return error;
  return { ...error, params: { ...error.params, field: name(field) } };
}

function Body({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t, i18n } = useTranslation();
  const id = useId();
  const place = usePanelPlace(panel, opened);
  // `null`: bu yıl (Rust okur). Yıl oturumluktur; düzenle birlikte saklanmaz.
  const [year, setYear] = useState<number | null>(null);
  const list = useSankrantiList(opened, place.place, year, panel.id);
  const view = list.shown;
  const phase = usePanelLoadPhase(list.loading);
  const [text, setText] = useState("");
  const [editing, setEditing] = useState(false);
  // Gösterilen yıl değişince kutu onu gösterir; kullanıcı yazarken değişmez. Eşitleme çizim
  // sırasında yapılır (`TransitDate` ile aynı kalıp).
  const target = editing || view === null ? null : String(view.year);
  const [synced, setSynced] = useState(target);
  if (target !== synced) {
    setSynced(target);
    if (target !== null) setText(target);
  }
  const typed = parseYear(text);
  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (typed === null) return;
    setEditing(false);
    setYear(typed);
  };
  const shownError = list.error === null ? null : fieldError(list.error, (f) => (i18n.exists(`sankranti.field.${f}`) ? t(`sankranti.field.${f}`) : f));

  return (
    <div className="module-panel">
      <form className="muhurta-form" aria-label={t("sankranti.form")} onSubmit={submit}>
        <div className="module-toolbar">
          <PlaceControls panel={panel} opened={opened} at={place} />
        </div>
        <div className="module-toolbar">
          <label htmlFor={`${id}-year`}>{t("sankranti.year")}</label>
          <input
            id={`${id}-year`}
            type="number"
            inputMode="numeric"
            min={YEAR_MIN}
            max={YEAR_MAX}
            step={1}
            value={text}
            aria-describedby={`${id}-hint`}
            aria-invalid={editing && typed === null ? true : undefined}
            onChange={(e) => {
              setEditing(true);
              setText(e.target.value);
            }}
          />
          <button type="submit">{t("sankranti.show")}</button>
          <button
            type="button"
            aria-pressed={year === null}
            onClick={() => {
              setEditing(false);
              setYear(null);
            }}
          >
            {t("sankranti.thisYear")}
          </button>
          <span id={`${id}-hint`} className="chart-toolbar-hint">
            {t("sankranti.yearHint", { min: YEAR_MIN, max: YEAR_MAX })}
          </span>
        </div>
      </form>
      {opened.chart.meta.settings.zodiac !== "Sidereal" && (
        <p className="panel-notice" role="note">
          {t("sankranti.siderealNote")}
        </p>
      )}
      {place.place === null ? (
        <PlaceMissing choice={place.choice} />
      ) : shownError ? (
        <PanelError error={shownError} onRetry={list.retry} />
      ) : view === null ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : (
        <List view={view} />
      )}
    </div>
  );
}

function List({ view }: { view: SankrantiListView }) {
  const { t } = useTranslation();
  const transit = useTransitControl();
  const captionId = useId();
  return (
    <>
      {view.warnings.includes("LowPrecision") && (
        <p className="panel-notice" role="note">
          <strong>{t("badge.LowPrecision")}</strong>: {t("badgeHint.LowPrecision")}
        </p>
      )}
      {view.warnings.includes("SunriseUnavailable") && (
        <p className="panel-notice" role="note">
          <strong>{t("badge.SunriseUnavailable")}</strong>: {t("sankranti.noSunrise")}
        </p>
      )}
      <BalaScroll labelledBy={captionId}>
        <table className="module-table">
          <caption id={captionId}>{t("sankranti.caption", { year: view.year, n: view.rows.length })}</caption>
          <thead>
            <tr>
              <th scope="col">{t("sankranti.col.when")}</th>
              <th scope="col">{t("sankranti.col.sign")}</th>
              <th scope="col">{t("sankranti.col.tithi")}</th>
              <th scope="col">{t("sankranti.col.vara")}</th>
              <th scope="col">{t("sankranti.col.nakshatra")}</th>
            </tr>
          </thead>
          <tbody>
            {view.rows.map(({ sankranti: s, offset_secs }) => {
              const jd = s.jd_ut;
              const when = formatJdMinute(jd, offset_secs);
              return (
                <tr key={s.sign} data-sign={s.sign}>
                  <th scope="row">
                    {jd === null ? (
                      when
                    ) : (
                      <button type="button" className="link-button" aria-label={t("sankranti.goTo", { when })} onClick={() => transit.setAt({ Jd: jd })}>
                        {when}
                      </button>
                    )}
                  </th>
                  <td>{t(`sign.${signIndex(s.sign)}`)}</td>
                  <td>
                    {s.paksha} {tithiInPaksha(s.tithi)} · {tithiName(s.tithi)}
                  </td>
                  <td>
                    {VARA_NAMES[s.vara]} · {t(`panchang.weekday.${s.vara}`)}
                  </td>
                  <td>
                    {nakshatraName(s.nakshatra)} · {t("panchang.pada", { pada: s.pada })}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </BalaScroll>
      <p className="panel-message">{t("sankranti.method", { method: view.method })}</p>
    </>
  );
}
````

Katalog ekleri `event_search` satırlarından hemen sonradır; `panel.sankranti` metni `transit_table` satırından sonra, `sankranti` bölümü üst düzey `transit` bölümünden hemen öncedir.

````diff
diff --git a/app/ui/layout/model.ts b/app/ui/layout/model.ts
index ac64dae..ba26fb1 100644
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -27,6 +27,7 @@ export const PANEL_KINDS = [
   "muhurta_search",
   "prashna_facts",
   "event_search",
+  "sankranti",
   "navatara",
   "vimshopaka",
 ] as const;
@@ -64,6 +65,8 @@ export const MODULE_PANELS = [
   { kind: "prashna_facts", w: 7, h: 20 },
   // Olay araması (Plan 22).
   { kind: "event_search", w: 7, h: 18 },
+  // Geçiş araçları (yol haritası Plan 26).
+  { kind: "sankranti", w: 7, h: 16 },
   // Doğum anı ekleri (Plan 23).
   { kind: "navatara", w: 7, h: 16 },
   { kind: "vimshopaka", w: 7, h: 12 },
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index e1380f2..aabfdee 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -23,6 +23,7 @@ import { NavataraPanel } from "./NavataraPanel";
 import { PanchangPanel } from "./PanchangPanel";
 import { EventSearchPanel } from "./EventSearchPanel";
 import { PrashnaFactsPanel } from "./PrashnaFactsPanel";
+import { SankrantiPanel } from "./SankrantiPanel";
 import { ShadbalaPanel } from "./ShadbalaPanel";
 import { SudarshanaPanel } from "./SudarshanaPanel";
 import type { PanelDefinition } from "./types";
@@ -70,6 +71,8 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   prashna_facts: { titleKey: "panel.prashna_facts", component: PrashnaFactsPanel },
   // Olay araması (Plan 22).
   event_search: { titleKey: "panel.event_search", component: EventSearchPanel },
+  // Geçiş araçları (yol haritası Plan 26).
+  sankranti: { titleKey: "panel.sankranti", component: SankrantiPanel },
   // Doğum anı ekleri (Plan 23).
   navatara: { titleKey: "panel.navatara", component: NavataraPanel },
   vimshopaka: { titleKey: "panel.vimshopaka", component: VimshopakaPanel },
````

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index b1ee1ae..1477789 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -70,6 +70,7 @@ export const en: Messages = {
     dasha_tree: "Dasha tree",
     dasha_strip: "Dasha strip",
     transit_table: "Transit table",
+    sankranti: "Sankranti",
     notes: "Notes",
     birth_info: "Birth details",
     panchang: "Panchang",
@@ -1375,6 +1376,20 @@ export const en: Messages = {
     notes: "Release notes for {{version}}",
     unconfigured: "This build has no update check (no signing key is embedded).",
   },
+  sankranti: {
+    form: "Sankranti list settings",
+    year: "Year",
+    show: "Show year",
+    thisYear: "This year",
+    yearHint: "Type a year from {{min}} to {{max}}. The year is the calendar year of the selected place.",
+    caption: "Sankranti list, {{year}}: {{n}} ingresses",
+    col: { when: "Moment of ingress", sign: "Sign entered", tithi: "Tithi", vara: "Vara", nakshatra: "Moon's nakshatra" },
+    goTo: "{{when}}: move the transit date to this moment",
+    siderealNote: "The chart uses tropical signs; this list always gives the sidereal ingresses with the selected ayanamsa.",
+    noSunrise: "The sun does not rise at this place on the day of at least one ingress: that row's vara is read from a day starting at 06:00 local mean time.",
+    field: { year: "year", place: "place" },
+    method: "Method: {{method}}. Ingresses are into sidereal signs; the tithi, vara and nakshatra are those of the moment of ingress. Times are in the selected place's time zone; the moment in a row changes the transit date.",
+  },
   transit: {
     date: "Transit moment",
     dateLabel: "Date",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 8f57fcb..a16ad87 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -70,6 +70,7 @@ export const fr: Messages = {
     dasha_tree: "Arbre des dashas",
     dasha_strip: "Bande des dashas",
     transit_table: "Tableau des transits",
+    sankranti: "Sankranti",
     notes: "Notes",
     birth_info: "Données de naissance",
     panchang: "Panchang",
@@ -1375,6 +1376,20 @@ export const fr: Messages = {
     notes: "Notes de version {{version}}",
     unconfigured: "Cette version ne vérifie pas les mises à jour (aucune clé de signature intégrée).",
   },
+  sankranti: {
+    form: "Réglages de la liste des sankranti",
+    year: "Année",
+    show: "Afficher l’année",
+    thisYear: "Cette année",
+    yearHint: "Saisissez une année de {{min}} à {{max}}. L’année est l’année civile du lieu choisi.",
+    caption: "Liste des sankranti, {{year}} : {{n}} entrées",
+    col: { when: "Moment de l’entrée", sign: "Signe d’entrée", tithi: "Tithi", vara: "Vara", nakshatra: "Nakshatra de la Lune" },
+    goTo: "{{when}} : placer la date de transit à ce moment",
+    siderealNote: "Le thème utilise les signes tropicaux ; cette liste donne toujours les entrées sidérales avec l’ayanamsa choisi.",
+    noSunrise: "Le soleil ne se lève pas en ce lieu le jour d’au moins une entrée : le vara de cette ligne est lu d’un jour commençant à 06:00, heure locale moyenne.",
+    field: { year: "année", place: "lieu" },
+    method: "Méthode : {{method}}. Les entrées se font dans les signes sidéraux ; le tithi, le vara et le nakshatra sont ceux du moment de l’entrée. Les heures sont dans le fuseau du lieu choisi ; le moment d’une ligne change la date de transit.",
+  },
   transit: {
     date: "Moment du transit",
     dateLabel: "Date",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index fdb980d..ce49bc2 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -69,6 +69,7 @@ export const tr = {
     dasha_tree: "Dasa ağacı",
     dasha_strip: "Dasa şeridi",
     transit_table: "Transit tablosu",
+    sankranti: "Sankranti",
     notes: "Notlar",
     birth_info: "Doğum bilgisi",
     panchang: "Panchang",
@@ -1374,6 +1375,20 @@ export const tr = {
     notes: "{{version}} sürüm notları",
     unconfigured: "Bu derlemede güncelleme denetimi yok (imza anahtarı gömülmemiş).",
   },
+  sankranti: {
+    form: "Sankranti listesi ayarları",
+    year: "Yıl",
+    show: "Yılı göster",
+    thisYear: "Bu yıl",
+    yearHint: "{{min}}–{{max}} arası bir yıl yazın. Yıl, seçilen yerin takvim yılıdır.",
+    caption: "Sankranti listesi, {{year}}: {{n}} giriş",
+    col: { when: "Giriş anı", sign: "Girilen burç", tithi: "Tithi", vara: "Vara", nakshatra: "Ay nakshatrası" },
+    goTo: "{{when}}: transit tarihini bu ana götür",
+    siderealNote: "Harita tropikal burç kullanıyor; bu liste her durumda seçili ayanamsayla sidereal girişleri verir.",
+    noSunrise: "Girişlerden en az birinin gününde bu yerde gün doğmuyor: o satırın varası 06:00 yerel ortalama saatte başlayan günden okundu.",
+    field: { year: "yıl", place: "yer" },
+    method: "Yöntem: {{method}}. Girişler sidereal burçlaradır; tithi, vara ve nakshatra giriş anınındır. Anlar seçilen yerin saat dilimindedir; satırdaki an transit tarihini değiştirir.",
+  },
   transit: {
     date: "Transit anı",
     dateLabel: "Tarih",
````

````diff
diff --git a/app/e2e/ipc.ts b/app/e2e/ipc.ts
index 751f48f..550d59c 100644
--- a/app/e2e/ipc.ts
+++ b/app/e2e/ipc.ts
@@ -5,6 +5,8 @@ import { readFileSync } from "node:fs";
 import type { Page } from "@playwright/test";
 
 export const FIXTURE = JSON.parse(readFileSync(new URL("../ui/test/fixtures/delhi.json", import.meta.url), "utf8"));
+// Geçiş araçlarının fikstürleri (Plan 26) ayrı dosyalardadır; taklit onları `fixture.transit_tools` altında görür.
+FIXTURE.transit_tools = { sankranti: JSON.parse(readFileSync(new URL("../ui/test/fixtures/sankranti.json", import.meta.url), "utf8")) };
 
 export interface IpcOptions {
   /** `restore_session` dönüşü: `true` Delhi haritası, `false` oturum yok. */
@@ -115,6 +117,9 @@ export async function mockIpc(page: Page, { session, unknownTime = false, partne
         dasha_children: (args) =>
           JSON.stringify(args.path) === JSON.stringify(fixture.dasha_children.path) ? fixture.dasha_children.periods : [],
         transits_at: () => fixture.transit,
+        // Geçiş araçları (Plan 26): fikstürdeki gerçek sonuçlar. Liste yıla göre yeniden hesaplanmaz:
+        // istenen yıl yanıtın `year` alanına yazılır.
+        sankranti_list: (args) => ({ ...fixture.transit_tools.sankranti.list, year: args.year ?? fixture.transit_tools.sankranti.list.year }),
         // Bakış tablosu yeniden hesaplanmaz: fikstürdeki gerçek sonuç döner.
         compute_drishti: () => fixture.drishti,
         // Bhava tablosu ve Sudarshana chakra (Plan 25): fikstürdeki gerçek sonuçlar. Tablo
````

- [ ] **Step 4: Testler, akış ve kapılar**

```bash
pnpm --dir app exec vitest run ui/panels/SankrantiPanel.test.tsx 2>&1 | grep -E "Tests |FAIL"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
lsof -i :14326
pnpm --dir app exec playwright test --project=e2e -c playwright.port.config.ts 2>&1 | tail -1
rm -rf app/test-results
```
Expected: `10 passed`; Vitest `101 passed (101)`, `996 passed (996)`; `✓ built`; `lsof` çıktısız; e2e `34 passed`.

- [ ] **Step 5: Commit**

```bash
git add app/ui/test/sankrantiFixture.ts app/ui/modules/useSankranti.ts app/ui/panels/SankrantiPanel.tsx app/ui/panels/SankrantiPanel.test.tsx app/ui/layout/model.ts app/ui/layout/model.test.ts app/ui/panels/registry.ts app/ui/panels/panels.test.tsx app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts app/e2e/ipc.ts app/e2e/sankranti.desk.spec.ts
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): sankranti panel with the year's ingresses in the place's time and rows that move the transit date" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: on üç dosya; yasaklı ad taraması çıktısız; `git status --short` boş.


---

### Task 4: Efemeris çekirdeği ve komutu: satırlar, giriş ve dönüş işaretleri, gerçek motorla çapraz denetim ve fikstür

Yol haritası §4.9 ve görev taslağının (4). maddesi; D12–D14, D17, D18. Konum hesabı transit hesabının yoludur (`chart::raw_grahas`); `transit.rs` değişmez.

**Files:**
- Create: `crates/jyotish-core/src/transit_tools/ephemeris_table.rs`, `app/src-tauri/src/transit_tools_commands/ephemeris.rs`, `app/src-tauri/tests/ephemeris_table.rs`, `app/ui/test/fixtures/ephemeris.json` (üretilir)
- Modify: `crates/jyotish-core/src/transit_tools/mod.rs`, `crates/jyotish-core/src/lib.rs`, `app/src-tauri/src/transit_tools_commands/mod.rs`, `app/src-tauri/src/lib.rs`, `app/ui/bindings.ts` (üretilir), `app/ui/api.ts`, `app/ui/api.test.ts`, `tests/golden/modules/transit_tools/REVIEW.md`
- Test: `ephemeris_table.rs` `mod tests` (7); `transit_tools_commands/ephemeris.rs` `mod tests` (1); `tests/ephemeris_table.rs` (1)

**Interfaces:**
- Consumes: `chart::{finite, rasi_frame, raw_grahas, to_sidereal, ChartWarning}` (kasa içi), `timeline::{Motion, STATION_GRAHAS}`; uygulamada `muhurta_commands` yardımcıları ve `atlas::Date`.
- Produces (çekirdek adları kökten de çıkar):
  - `EPHEMERIS_METHOD = "ephemeris_table.v1"`, `EPHEMERIS_MAX_ROWS = 31`, `EPHEMERIS_MAX_STEP_DAYS = 2.0`.
  - `EphemerisCell { graha, lon, sign, deg_in_sign, retrograde, nakshatra, pada, ingress: Option<Sign>, station: Option<Motion> }`, `EphemerisRow { jd_ut, cells }`, `EphemerisTable { rows, warnings, method }`.
  - `ephemeris_table(eph, settings, moments: &[f64]) -> Result<EphemerisTable, CoreError>`: `n + 1` an `n` satır verir; sonuncu an yalnızca son satırın işaretlerini kapatır.
  - Uygulama: `ephemeris::{MonthId { year: i32, month: u8 }, EphemerisDayView { date, offset_secs, row }, EphemerisMonthView { month, prev, next, days, warnings, method }, ephemeris_month_blocking(engine, month: Option<MonthId>, place, settings, now), ephemeris_month}`; `api.ephemerisMonth(month, place, settings)`.
  - Hata alanları: `moments` (çekirdek), `month`, `place`.

- [ ] **Step 1: Kayıt ve testler: satır, transit hesabıyla ikinci yazım, çerçeve, iki işaret, uyarı, girdi denetimi, ay adımı ve gerçek motor**

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 6244ea0..ec86a44 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -168,6 +168,7 @@ pub use zodiac::{angular_distance, deg_in_sign, nakshatra_pada, normalize, Sign}
 // alfabetik gruplardaki ve dosya başındaki ekleriyle aynı satırlara dokunmaz.
 pub mod transit_tools;
 pub use transit_tools::{
-    sankranti_list, sankranti_moments, Sankranti, SankrantiList, NAKSHATRA_LORDS,
-    SANKRANTI_MAX_RANGE_DAYS, SANKRANTI_METHOD,
+    ephemeris_table, sankranti_list, sankranti_moments, EphemerisCell, EphemerisRow,
+    EphemerisTable, Sankranti, SankrantiList, EPHEMERIS_MAX_ROWS, EPHEMERIS_MAX_STEP_DAYS,
+    EPHEMERIS_METHOD, NAKSHATRA_LORDS, SANKRANTI_MAX_RANGE_DAYS, SANKRANTI_METHOD,
 };
diff --git a/crates/jyotish-core/src/transit_tools/mod.rs b/crates/jyotish-core/src/transit_tools/mod.rs
index db6acb6..4de1469 100644
--- a/crates/jyotish-core/src/transit_tools/mod.rs
+++ b/crates/jyotish-core/src/transit_tools/mod.rs
@@ -2,9 +2,14 @@
 //! listesi ve efemeris tablosu. Hepsi var olan konum hesabının yeni görünümüdür: yeni astronomi
 //! yoktur. Sonuçlar `Chart`'a eklenmez ve saklanmaz; panel açıkken istenir (K1).
 
+pub mod ephemeris_table;
 pub mod lords;
 pub mod sankranti;
 
+pub use ephemeris_table::{
+    ephemeris_table, EphemerisCell, EphemerisRow, EphemerisTable, EPHEMERIS_MAX_ROWS,
+    EPHEMERIS_MAX_STEP_DAYS, EPHEMERIS_METHOD,
+};
 pub use lords::NAKSHATRA_LORDS;
 pub use sankranti::{
     sankranti_list, sankranti_moments, Sankranti, SankrantiList, SANKRANTI_MAX_RANGE_DAYS,
````

**Yeni dosya:** `crates/jyotish-core/src/transit_tools/ephemeris_table.rs` (şimdilik yalnızca test bloğu)

````rust
#[cfg(test)]
mod tests {
    use ephemeris::{
        Ayanamsa, Body, BodyPosition, EphError, HouseSystem, Houses, Precision, RiseSet,
        SiderealMode, SunDisc, Zodiac,
    };

    use super::*;
    use crate::fake::FakeEph;
    use crate::settings::ZodiacMode;
    use crate::transit::compute_transits;
    use crate::zodiac::normalize;
    use crate::{compute_chart, Bhavas};

    const JD: f64 = 2_451_545.0;

    /// `JD`'den başlayan `rows` günlük tablo: `rows + 1` an.
    fn days(rows: usize) -> Vec<f64> {
        (0..=rows).map(|i| JD + i as f64).collect()
    }

    fn table(settings: &CalcSettings, rows: usize) -> EphemerisTable {
        ephemeris_table(&mut FakeEph::default(), settings, &days(rows)).unwrap()
    }

    fn cell(row: &EphemerisRow, graha: Graha) -> &EphemerisCell {
        row.cells.iter().find(|c| c.graha == graha).unwrap()
    }

    fn natal_bhavas(settings: &CalcSettings) -> Bhavas {
        compute_chart(&mut FakeEph::default(), JD, 41.0, 29.0, settings)
            .unwrap()
            .bhavas
    }

    #[test]
    fn a_row_is_the_nine_grahas_of_its_moment() {
        // Sahte gök (sidereal = tropikal − 24°): Güneş `JD` anında 346°, günde 0,98°; Ay 71°,
        // günde 13,2°; Jüpiter 226°, günde −0,1° (retro).
        let t = table(&CalcSettings::default(), 10);
        assert_eq!(t.method, "ephemeris_table.v1");
        assert!(t.warnings.is_empty());
        assert_eq!(t.rows.len(), 10);
        for (i, row) in t.rows.iter().enumerate() {
            assert_eq!(row.jd_ut, JD + i as f64);
            let grahas: Vec<Graha> = row.cells.iter().map(|c| c.graha).collect();
            assert_eq!(grahas, Graha::ALL);
        }
        // Satır 0: Güneş 346° = Balık 16°; Ay 71° = İkizler 11°, 71 / 13,3333 = 5,3 → 6.
        // nakshatra (Ardra), içinde 71 − 66,667 = 4,333°, 4,333 / 3,3333 = 1,3 → 2. pada.
        let r0 = &t.rows[0];
        let sun = cell(r0, Graha::Sun);
        assert_eq!(sun.sign, Sign::Pisces);
        assert!((sun.lon - 346.0).abs() < 1e-9 && (sun.deg_in_sign - 16.0).abs() < 1e-9);
        let moon = cell(r0, Graha::Moon);
        assert_eq!((moon.sign, moon.nakshatra, moon.pada), (Sign::Gemini, 6, 2));
        assert!((moon.deg_in_sign - 11.0).abs() < 1e-9);
        // Satır 3: Ay 71 + 39,6 = 110,6° = Yengeç 20,6°; 110,6 / 13,3333 = 8,3 → 9. nakshatra
        // (Ashlesha), içinde 110,6 − 106,667 = 3,933°, 3,933 / 3,3333 = 1,18 → 2. pada.
        let moon = cell(&t.rows[3], Graha::Moon);
        assert_eq!((moon.sign, moon.nakshatra, moon.pada), (Sign::Cancer, 9, 2));
        assert!((moon.deg_in_sign - 20.6).abs() < 1e-9);
        // Retro: yalnızca hızı eksi olanlar (Jüpiter ve ortalama düğümler).
        let retro: Vec<Graha> = r0
            .cells
            .iter()
            .filter(|c| c.retrograde)
            .map(|c| c.graha)
            .collect();
        assert_eq!(retro, [Graha::Jupiter, Graha::Rahu, Graha::Ketu]);
        // Ketu, Rahu'nun tam karşısındadır.
        let (rahu, ketu) = (cell(r0, Graha::Rahu), cell(r0, Graha::Ketu));
        assert!((ketu.lon - normalize(rahu.lon + 180.0)).abs() < 1e-9);
    }

    #[test]
    fn rows_agree_with_the_transit_snapshot_of_the_same_moment() {
        // İkinci yazım: v1'in transit hesabı aynı anda aynı konumu verir, iki zodyak kipinde.
        for zodiac in [
            ZodiacMode::Sidereal,
            ZodiacMode::TropicalRasiSiderealNakshatra,
        ] {
            let s = CalcSettings {
                zodiac,
                ..CalcSettings::default()
            };
            let bhavas = natal_bhavas(&s);
            for row in table(&s, 31).rows {
                let snap =
                    compute_transits(&mut FakeEph::default(), row.jd_ut, &s, &bhavas).unwrap();
                for (c, g) in row.cells.iter().zip(&snap.grahas) {
                    let ctx = format!("{:?} {zodiac:?} {}", c.graha, row.jd_ut);
                    assert_eq!(c.graha, g.graha, "{ctx}");
                    assert_eq!(
                        (c.lon, c.sign, c.deg_in_sign),
                        (g.lon, g.sign, g.deg_in_sign)
                    );
                    assert_eq!((c.nakshatra, c.pada), (g.nakshatra, g.pada), "{ctx}");
                    assert_eq!(c.retrograde, g.retrograde, "{ctx}");
                }
            }
        }
    }

    #[test]
    fn the_tropical_rasi_mode_reads_signs_from_the_tropical_longitude() {
        let tropical = CalcSettings {
            zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
            ..CalcSettings::default()
        };
        // Güneş tropikal 10° = Koç 10°; sidereal 346°: 346 / 13,3333 = 25,95 → 26. nakshatra
        // (Uttara Bhadrapada), içinde 346 − 333,333 = 12,667°, 12,667 / 3,3333 = 3,8 → 4. pada.
        let sun = table(&tropical, 1).rows[0].cells[0].clone();
        assert_eq!((sun.sign, sun.nakshatra, sun.pada), (Sign::Aries, 26, 4));
        assert!((sun.lon - 10.0).abs() < 1e-9);
        // Sidereal kipte aynı an: Balık 16°, aynı nakshatra.
        let sun = table(&CalcSettings::default(), 1).rows[0].cells[0].clone();
        assert_eq!((sun.sign, sun.nakshatra, sun.pada), (Sign::Pisces, 26, 4));
    }

    #[test]
    fn the_ingress_mark_names_the_sign_entered_before_the_next_moment() {
        let t = table(&CalcSettings::default(), 31);
        // Güneş: 346 + 0,98 g = 360 → g = 14,29: 14. satırla 15. an arasında Koç'a girer.
        for (i, row) in t.rows.iter().enumerate() {
            let expected = (i == 14).then_some(Sign::Aries);
            assert_eq!(cell(row, Graha::Sun).ingress, expected, "row {i}");
        }
        // Ay: kuraldan, satır satır. g. günde boylam 71 + 13,2 g; burç floor(boylam / 30) mod 12.
        // İlk girişler: 90° g = 1,44 (satır 1, Yengeç); 120° g = 3,71 (satır 3, Aslan);
        // 150° g = 5,98 (satır 5, Başak); 180° g = 8,26 (satır 8, Terazi).
        let sign_at = |day: usize| Sign::of(normalize(71.0 + 13.2 * day as f64));
        let mut marked = Vec::new();
        for (i, row) in t.rows.iter().enumerate() {
            let expected = (sign_at(i) != sign_at(i + 1)).then_some(sign_at(i + 1));
            assert_eq!(cell(row, Graha::Moon).ingress, expected, "row {i}");
            if expected.is_some() {
                marked.push(i);
            }
        }
        assert_eq!(&marked[..4], [1, 3, 5, 8]);
        assert_eq!(cell(&t.rows[1], Graha::Moon).ingress, Some(Sign::Cancer));
        // Son satırın işareti kapatan andan okunur: 31 satır, 32 an.
        assert_eq!(t.rows.len(), 31);
        // Retro graha geriye girer: Jüpiter 226° − 0,1 g; 210°'ye 160 günde varır (bu ayda yok).
        assert!(t
            .rows
            .iter()
            .all(|r| cell(r, Graha::Jupiter).ingress.is_none()));
    }

    /// Mars'ın hızı `turn` anından sonra eksi olan sahte gök: dönüş işaretinin el örneği.
    struct Turning {
        inner: FakeEph,
        turn: f64,
    }

    impl Ephemeris for Turning {
        fn body(
            &mut self,
            jd_ut: f64,
            body: Body,
            zodiac: Zodiac,
        ) -> Result<BodyPosition, EphError> {
            let mut p = self.inner.body(jd_ut, body, zodiac)?;
            if body == Body::Mars && jd_ut > self.turn {
                p.lon_speed = -p.lon_speed.abs();
            }
            if body == Body::Jupiter && jd_ut > self.turn + 3.0 {
                p.lon_speed = p.lon_speed.abs();
            }
            Ok(p)
        }

        fn ayanamsa(&mut self, jd_ut: f64, mode: SiderealMode) -> Result<Ayanamsa, EphError> {
            self.inner.ayanamsa(jd_ut, mode)
        }

        fn houses(
            &mut self,
            jd_ut: f64,
            geo_lat: f64,
            geo_lon: f64,
            system: HouseSystem,
            zodiac: Zodiac,
        ) -> Result<Houses, EphError> {
            self.inner.houses(jd_ut, geo_lat, geo_lon, system, zodiac)
        }

        fn sun_rise_set(
            &mut self,
            after_jd_ut: f64,
            event: RiseSet,
            geo_lat: f64,
            geo_lon: f64,
            disc: SunDisc,
        ) -> Result<Option<f64>, EphError> {
            self.inner
                .sun_rise_set(after_jd_ut, event, geo_lat, geo_lon, disc)
        }

        fn swe_version(&self) -> String {
            self.inner.swe_version()
        }
    }

    #[test]
    fn the_station_mark_names_the_direction_turned_to_before_the_next_moment() {
        // Mars 4,5. günde retroya döner: 4. satır (4. gün) düz, 5. an retro → 4. satır
        // "retroya döner". Jüpiter 7,5. günde düze döner: 7. satır retro, 8. an düz → 7. satır
        // "düze döner". Düğümlerin yönü işaretlenmez.
        let mut eph = Turning {
            inner: FakeEph::default(),
            turn: JD + 4.5,
        };
        let t = ephemeris_table(&mut eph, &CalcSettings::default(), &days(12)).unwrap();
        for (i, row) in t.rows.iter().enumerate() {
            let mars = cell(row, Graha::Mars);
            assert_eq!(mars.retrograde, i >= 5, "row {i}");
            assert_eq!(
                mars.station,
                (i == 4).then_some(Motion::Retrograde),
                "row {i}"
            );
            let jupiter = cell(row, Graha::Jupiter);
            assert_eq!(jupiter.retrograde, i <= 7, "row {i}");
            assert_eq!(
                jupiter.station,
                (i == 7).then_some(Motion::Direct),
                "row {i}"
            );
            for graha in [Graha::Sun, Graha::Moon, Graha::Rahu, Graha::Ketu] {
                assert_eq!(cell(row, graha).station, None, "row {i} {graha:?}");
            }
        }
    }

    #[test]
    fn the_fallback_ephemeris_is_reported() {
        let mut eph = FakeEph {
            precision: Precision::Moshier,
            ..FakeEph::default()
        };
        let t = ephemeris_table(&mut eph, &CalcSettings::default(), &days(3)).unwrap();
        assert_eq!(t.warnings, [ChartWarning::LowPrecision]);
    }

    #[test]
    fn bad_moments_name_the_field() {
        let s = CalcSettings::default();
        let mut eph = FakeEph::default();
        let bad = Err(CoreError::InvalidTransitInput("moments"));
        let cases: Vec<Vec<f64>> = vec![
            vec![],
            vec![JD],
            days(EPHEMERIS_MAX_ROWS + 1),
            vec![JD, JD],
            vec![JD + 1.0, JD],
            vec![JD, JD + 1.0, JD + 0.5],
            vec![JD, f64::NAN],
            vec![JD, f64::INFINITY],
            vec![1.0, 2.0],
            vec![JD, JD + EPHEMERIS_MAX_STEP_DAYS + 0.01],
        ];
        for moments in cases {
            assert_eq!(ephemeris_table(&mut eph, &s, &moments), bad, "{moments:?}");
        }
        // En uzun tablo: 31 satır, 32 an.
        assert_eq!(table(&s, EPHEMERIS_MAX_ROWS).rows.len(), EPHEMERIS_MAX_ROWS);
        let old = CalcSettings {
            schema_version: 1,
            ..CalcSettings::default()
        };
        assert_eq!(
            ephemeris_table(&mut eph, &old, &days(2)),
            Err(CoreError::UnsupportedSchemaVersion {
                found: 1,
                supported: 2
            })
        );
    }
}
````

````diff
diff --git a/app/src-tauri/src/lib.rs b/app/src-tauri/src/lib.rs
index efccb2c..68dfeea 100644
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -37,6 +37,7 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             commands::compute_transits,
             transit_commands::transits_at,
             transit_tools_commands::sankranti::sankranti_list,
+            transit_tools_commands::ephemeris::ephemeris_month,
             bhava_commands::compute_bhava_table,
             bhava_commands::compute_sudarshana,
             commands::dasha_children,
diff --git a/app/src-tauri/src/transit_tools_commands/mod.rs b/app/src-tauri/src/transit_tools_commands/mod.rs
index ed6764a..f5566c8 100644
--- a/app/src-tauri/src/transit_tools_commands/mod.rs
+++ b/app/src-tauri/src/transit_tools_commands/mod.rs
@@ -1,11 +1,12 @@
-//! Geçiş araçlarının komutları (yol haritası Plan 26): Sankranti listesi. Kalıp muhurta
-//! komutlarıyla aynıdır: hesap motor iş parçacığında, komut gövdesi `blocking` havuzunda, hata
-//! `CommandError`; sonuç saklanmaz (K1). "Bu yıl" Rust'ta okunur; arayüz tarih hesabı yapmaz
-//! (C4).
+//! Geçiş araçlarının komutları (yol haritası Plan 26): Sankranti listesi ve efemeris tablosu.
+//! Kalıp muhurta komutlarıyla aynıdır: hesap motor iş parçacığında, komut gövdesi `blocking`
+//! havuzunda, hata `CommandError`; sonuç saklanmaz (K1). "Bu yıl", "bu ay", önceki ve sonraki
+//! ay Rust'ta okunur; arayüz tarih hesabı yapmaz (C4).
 //!
 //! Yer kişisel veridir: loglanmaz ve hata parametresine girmez. Bu modülün ürettiği hatalar
 //! var olan `invalid_transit_input` kodunu ve yalnızca alan adını taşır.
 
+pub mod ephemeris;
 pub mod sankranti;
 
 use crate::commands::CommandError;
````

**Yeni dosya:** `app/src-tauri/src/transit_tools_commands/ephemeris.rs` (şimdilik yalnızca test bloğu)

````rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn months_step_across_the_year_and_stop_at_the_range_ends() {
        let m = |year, month| MonthId { year, month };
        assert_eq!(m(2026, 11).prev(), Some(m(2026, 10)));
        assert_eq!(m(2026, 11).next(), Some(m(2026, 12)));
        assert_eq!(m(2026, 12).next(), Some(m(2027, 1)));
        assert_eq!(m(2027, 1).prev(), Some(m(2026, 12)));
        // Yıl aralığının uçları: 1 ve 2999.
        assert_eq!(m(1, 1).prev(), None);
        assert_eq!(m(1, 1).next(), Some(m(1, 2)));
        assert_eq!(m(2999, 12).next(), None);
        assert_eq!(m(2999, 12).prev(), Some(m(2999, 11)));
        for bad in [m(0, 5), m(3000, 1), m(2026, 0), m(2026, 13)] {
            assert!(!bad.is_valid(), "{bad:?}");
        }
        assert_eq!(m(2024, 2).first_day().unwrap().days_in_month(), 29);
        assert_eq!(m(2026, 2).first_day().unwrap().days_in_month(), 28);
    }
}
````

**Yeni dosya:** `app/src-tauri/tests/ephemeris_table.rs`

````rust
//! Efemeris tablosu komutu, gerçek `Engine` ile uçtan uca (yol haritası Plan 26, §4.9) ve arayüz
//! testlerinin fikstürü: `app/ui/test/fixtures/ephemeris.json`.
//! Güncellemek için: `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ephemeris_table`.
//! Tek test: süreçte yalnızca bir SE örneği açılabilir. Çapraz denetimler v1'in transit hesabına
//! (konumlar) ve `timeline` dizilerine (giriş ve dönüş işaretleri) dayanır; beklenen değerler
//! başka bir yazılımdan alınmaz (K9).

use std::path::PathBuf;
use std::time::{Duration, Instant};

use atlas::Timestamp;
use ephemeris::{Engine, SwissEph};
use jyotish_core::{
    graha_motion_series, graha_sign_series, CalcSettings, ChartWarning, CoreError, Graha, Motion,
    NodeKind, Sign, ZodiacMode, EPHEMERIS_METHOD, STATION_GRAHAS,
};
use jyotish_desk_lib::commands::{compute_chart_blocking, compute_transits_blocking};
use jyotish_desk_lib::muhurta_commands::MuhurtaPlace;
use jyotish_desk_lib::transit_tools_commands::ephemeris::{
    ephemeris_month_blocking, EphemerisMonthView, MonthId,
};
use serde_json::{json, Value};
use store::TzChoice;

/// Bütçe (D17): bir ayın tablosu, hata ayıklama derlemesinde.
const BUDGET: Duration = Duration::from_millis(100);

fn delhi() -> MuhurtaPlace {
    MuhurtaPlace {
        lat: 28.6139,
        lon: 77.2090,
        tz: TzChoice::Iana("Asia/Kolkata".into()),
    }
}

/// 2026-11-24T00:00:00Z.
fn now() -> Timestamp {
    Timestamp::from_second(1_795_478_400).expect("timestamp")
}

fn month(year: i32, month: u8) -> MonthId {
    MonthId { year, month }
}

fn view(
    engine: &Engine,
    m: Option<MonthId>,
    place: MuhurtaPlace,
    settings: &CalcSettings,
) -> EphemerisMonthView {
    ephemeris_month_blocking(engine, m, place, settings.clone(), now()).expect("ephemeris month")
}

/// Ayın anları: satırların anları ve son satırı kapatan an (sonraki ayın ilk satırı).
fn moments(
    engine: &Engine,
    v: &EphemerisMonthView,
    place: MuhurtaPlace,
    settings: &CalcSettings,
) -> Vec<f64> {
    let mut out: Vec<f64> = v.days.iter().map(|d| d.row.jd_ut).collect();
    let next = view(engine, v.next, place, settings);
    out.push(next.days[0].row.jd_ut);
    out
}

/// `timeline` dizisinin sınırlarının düştüğü satırlar: (satır, yeni değer).
fn rows_of<T: Copy>(series: &[jyotish_core::Interval<T>], moments: &[f64]) -> Vec<(usize, T)> {
    series
        .windows(2)
        .map(|w| {
            let t = w[1].start_jd_ut;
            let row = moments.partition_point(|m| *m <= t) - 1;
            (row, w[1].value)
        })
        .collect()
}

/// Tablonun konumları v1'in transit hesabıyla, işaretleri `timeline` dizileriyle aynıdır.
/// Dönen sayı, tablodaki dönüş işaretlerinin sayısıdır.
fn cross_check(
    engine: &Engine,
    v: &EphemerisMonthView,
    place: MuhurtaPlace,
    settings: &CalcSettings,
) -> usize {
    assert_eq!(v.method, EPHEMERIS_METHOD);
    let bhavas = compute_chart_blocking(engine, 2_451_545.0, 28.6139, 77.2090, settings.clone())
        .expect("chart")
        .bhavas;
    for day in &v.days {
        let snap =
            compute_transits_blocking(engine, day.row.jd_ut, settings.clone(), bhavas.clone())
                .expect("transits");
        assert_eq!(day.row.cells.len(), 9);
        for (c, g) in day.row.cells.iter().zip(&snap.grahas) {
            let ctx = format!("{} {:?}", day.date, c.graha);
            assert_eq!(c.graha, g.graha, "{ctx}");
            assert_eq!(
                (c.lon, c.sign, c.deg_in_sign),
                (g.lon, g.sign, g.deg_in_sign),
                "{ctx}"
            );
            assert_eq!(
                (c.nakshatra, c.pada, c.retrograde),
                (g.nakshatra, g.pada, g.retrograde),
                "{ctx}"
            );
        }
    }
    let moments = moments(engine, v, place, settings);
    let (from, to) = (moments[0], *moments.last().expect("moments"));
    let mut stations = 0;
    for (index, graha) in Graha::ALL.into_iter().enumerate() {
        let s = settings.clone();
        let (signs, motions) = engine
            .call(move |se: &mut SwissEph| {
                let signs = graha_sign_series(se, &s, graha, from, to)?;
                let motions = if STATION_GRAHAS.contains(&graha) {
                    graha_motion_series(se, &s, graha, from, to)?
                } else {
                    Vec::new()
                };
                Ok::<_, CoreError>((signs, motions))
            })
            .expect("engine")
            .expect("series");
        let marked: Vec<(usize, Sign)> = v
            .days
            .iter()
            .enumerate()
            .filter_map(|(i, d)| d.row.cells[index].ingress.map(|sign| (i, sign)))
            .collect();
        assert_eq!(
            marked,
            rows_of(&signs, &moments),
            "{graha:?} ingress, {:?}",
            v.month
        );
        let turned: Vec<(usize, Motion)> = v
            .days
            .iter()
            .enumerate()
            .filter_map(|(i, d)| d.row.cells[index].station.map(|m| (i, m)))
            .collect();
        assert_eq!(
            turned,
            rows_of(&motions, &moments),
            "{graha:?} station, {:?}",
            v.month
        );
        stations += turned.len();
    }
    stations
}

#[test]
fn ephemeris_command_runs_on_the_engine_thread_and_maps_errors() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let engine = Engine::spawn(root.join("../../data/ephe")).expect("spawn engine");
    let settings = CalcSettings::default();

    let started = Instant::now();
    let november = view(&engine, Some(month(2026, 11)), delhi(), &settings);
    let spent = started.elapsed();
    eprintln!("ephemeris_month(2026-11): {spent:?}");
    assert!(spent <= BUDGET, "ephemeris_month took {spent:?}");

    // Kasım 30 gündür; her satır yerin gece yarısındadır (Delhi +05:30: 18:30 UT).
    assert_eq!(november.month, month(2026, 11));
    assert_eq!(
        (november.prev, november.next),
        (Some(month(2026, 10)), Some(month(2026, 12)))
    );
    assert_eq!(november.days.len(), 30);
    assert!(november.warnings.is_empty());
    for (i, day) in november.days.iter().enumerate() {
        assert_eq!(day.date, format!("2026-11-{:02}", i + 1));
        assert_eq!(day.offset_secs, 19_800);
        let local = day.row.jd_ut + 0.5 + f64::from(day.offset_secs) / 86_400.0;
        assert!(
            (local - local.round()).abs() < 1e-9,
            "{}: {local}",
            day.date
        );
    }
    // Güneş ayda bir burç değiştirir; Ay 30 günde 13 ya da 14 burca girer; Güneş ve Ay retro
    // olmaz. Bunlar tablonun kendi içinde tutarlılığıdır; kesin denetim aşağıdadır.
    let ingresses = |g: usize| {
        november
            .days
            .iter()
            .filter(|d| d.row.cells[g].ingress.is_some())
            .count()
    };
    assert_eq!(ingresses(0), 1);
    assert!((13..=14).contains(&ingresses(1)), "{}", ingresses(1));
    assert!(november
        .days
        .iter()
        .all(|d| !d.row.cells[0].retrograde && !d.row.cells[1].retrograde));

    // Konumlar ve işaretler, üç ay ve iki zodyak kipi; gerçek düğümle bir ay daha.
    let tropical = CalcSettings {
        zodiac: ZodiacMode::TropicalRasiSiderealNakshatra,
        ..settings.clone()
    };
    let true_node = CalcSettings {
        node: NodeKind::True,
        ..settings.clone()
    };
    let mut stations = 0;
    for m in [month(2026, 11), month(2026, 12), month(2027, 1)] {
        for s in [&settings, &tropical] {
            let v = view(&engine, Some(m), delhi(), s);
            stations += cross_check(&engine, &v, delhi(), s);
        }
    }
    cross_check(
        &engine,
        &view(&engine, Some(month(2026, 11)), delhi(), &true_node),
        delhi(),
        &true_node,
    );
    // Üç ayda en az bir dönüş vardır (iki kipte iki kez sayılır).
    assert!(stations >= 2, "{stations}");

    // Ay verilmezse yerin bugünkü ayıdır. 2026-11-30 19:00 UT Delhi'de 1 Aralık 00:30'dur.
    assert_eq!(
        view(&engine, None, delhi(), &settings).month,
        month(2026, 11)
    );
    let month_end = Timestamp::from_second(1_796_065_200).expect("timestamp");
    let december = ephemeris_month_blocking(&engine, None, delhi(), settings.clone(), month_end)
        .expect("month");
    assert_eq!(december.month, month(2026, 12));
    assert_eq!(
        (december.days.len(), december.next),
        (31, Some(month(2027, 1)))
    );
    // Şubat: 2024 artık yıldır.
    assert_eq!(
        view(&engine, Some(month(2024, 2)), delhi(), &settings)
            .days
            .len(),
        29
    );

    // Yaz saati gece yarısında başlayan yer: São Paulo'da 2017-10-15 00:00 yoktur (saat 01:00'e
    // atlar). O günün satırı günün başladığı andadır; önceki günle arası 23 saattir.
    let sao_paulo = MuhurtaPlace {
        lat: -23.5475,
        lon: -46.6361,
        tz: TzChoice::Iana("America/Sao_Paulo".into()),
    };
    let october = view(&engine, Some(month(2017, 10)), sao_paulo, &settings);
    assert_eq!(october.days.len(), 31);
    let gaps: Vec<f64> = october
        .days
        .windows(2)
        .map(|w| (w[1].row.jd_ut - w[0].row.jd_ut) * 24.0)
        .collect();
    assert!((gaps[13] - 23.0).abs() < 1e-6, "{}", gaps[13]);
    assert!(gaps
        .iter()
        .enumerate()
        .all(|(i, g)| i == 13 || (g - 24.0).abs() < 1e-6));
    // Geçişten önceki ve sonraki günlerin ofseti: −03:00 ve −02:00. Geçiş anının kendisi (15
    // Ekim'in satırı) sınırdadır; ofseti burada sabitlenmez.
    assert_eq!(
        (october.days[13].offset_secs, october.days[15].offset_secs),
        (-10_800, -7_200)
    );
    assert_eq!(october.days[14].date, "2017-10-15");

    // Ephemeris dosyalarının dışındaki yıl: yedek ephemeris uyarısı.
    let far = view(&engine, Some(month(2500, 1)), delhi(), &settings);
    assert_eq!(far.warnings, [ChartWarning::LowPrecision]);

    // Hatalar yalnızca alan adını taşır; yer hataya girmez.
    let bad_place = MuhurtaPlace {
        lon: 200.0,
        ..delhi()
    };
    let e = ephemeris_month_blocking(
        &engine,
        Some(month(2026, 11)),
        bad_place,
        settings.clone(),
        now(),
    )
    .unwrap_err();
    assert_eq!(e.code, "invalid_transit_input");
    assert_eq!(e.params.len(), 1);
    assert_eq!(e.params["field"], "place");
    for bad in [
        month(0, 1),
        month(3000, 1),
        month(2026, 0),
        month(2026, 13),
        month(i32::MIN, 255),
    ] {
        let e = ephemeris_month_blocking(&engine, Some(bad), delhi(), settings.clone(), now())
            .unwrap_err();
        assert_eq!(e.code, "invalid_transit_input", "{bad:?}");
        assert_eq!(e.params.len(), 1);
        assert_eq!(e.params["field"], "month");
    }
    let old = CalcSettings {
        schema_version: 1,
        ..settings.clone()
    };
    let e =
        ephemeris_month_blocking(&engine, Some(month(2026, 11)), delhi(), old, now()).unwrap_err();
    assert_eq!(e.code, "unsupported_schema_version");

    // Fikstür: Delhi, Kasım 2026.
    let fresh = json!({ "place": delhi(), "month": november });
    let path = root.join("../ui/test/fixtures/ephemeris.json");
    if std::env::var_os("UPDATE_FIXTURES").is_some() {
        let text = serde_json::to_string(&fresh).expect("serialize fixture") + "\n";
        std::fs::write(&path, text).expect("write fixture");
        return;
    }
    let committed: Value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(Value::Null);
    assert!(
        approx_eq(&committed, &fresh),
        "app/ui/test/fixtures/ephemeris.json is stale; run: UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ephemeris_table"
    );
}

/// Yapısal eşitlik; sayılar göreli 1e-9 toleransla, diğer değerler birebir (`ui_fixture.rs`
/// ile aynı kural).
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
````

- [ ] **Step 2: Testlerin derlenmediğini gör**

```bash
cargo test -p jyotish-core --lib transit_tools 2>&1 | grep -E "^error" | head -3
```
Expected: `error[E0432]` (çözülemeyen `ephemeris_table` adları).

- [ ] **Step 3: Çekirdek ve komut**

**`crates/jyotish-core/src/transit_tools/ephemeris_table.rs`: uygulama, test bloğunun üstüne (arada bir boş satır)**

````rust
//! Efemeris tablosu (yol haritası §4.9): verilen anlarda dokuz grahanın konumu, satır satır.
//!
//! Konum hesabı transit hesabıyla aynıdır (`chart::raw_grahas`): boylam, burç ve burç içi derece
//! rasi çerçevesindedir (K5); nakshatra ve pada sidereal boylamdandır. Çağıran anları verir
//! (uygulama katmanı: bir ayın günleri, yerin gece yarısında) ve son satırı kapatan bir an daha
//! ekler: her satır, bir sonraki ana dek olan değişimi iki işaretle taşır. `ingress`: graha o
//! süre içinde başka bir burca girer (girilen burç). `station`: graha o süre içinde yön
//! değiştirir (retroya ya da düze döner). İşaretler iki ardışık örneğin karşılaştırmasıdır;
//! kesin an olay aramasındadır. Aynı süre içinde gidip dönen bir değişim (durağan grahanın burç
//! sınırını iki kez geçmesi) görülmez.

use ephemeris::Ephemeris;
use serde::{Deserialize, Serialize};

use crate::chart::{finite, rasi_frame, raw_grahas, to_sidereal, ChartWarning};
use crate::graha::Graha;
use crate::settings::{CalcSettings, JD_UT_RANGE};
use crate::timeline::{Motion, STATION_GRAHAS};
use crate::zodiac::{deg_in_sign, nakshatra_pada, Sign};
use crate::CoreError;

/// Yöntem kimliği (K7).
pub const EPHEMERIS_METHOD: &str = "ephemeris_table.v1";
/// Bir tablonun en çok satırı: bir ay.
pub const EPHEMERIS_MAX_ROWS: usize = 31;
/// Ardışık iki an arasındaki en uzun süre, gün. İşaretler satır aralığının bir günü pek
/// aşmamasına dayanır (Ay iki günde bir burçtan fazla yürümez).
pub const EPHEMERIS_MAX_STEP_DAYS: f64 = 2.0;

/// Bir grahanın bir satırdaki konumu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct EphemerisCell {
    pub graha: Graha,
    /// Rasi çerçevesinde boylam, [0, 360).
    pub lon: f64,
    pub sign: Sign,
    pub deg_in_sign: f64,
    /// Boylam hızı < 0 (v1'in retro tanımı).
    pub retrograde: bool,
    /// Sidereal boylamdan.
    pub nakshatra: u8,
    pub pada: u8,
    /// Sonraki ana dek girilen burç; burç değişmiyorsa `None`.
    pub ingress: Option<Sign>,
    /// Sonraki ana dek dönülen yön; yalnızca `STATION_GRAHAS` için, yön değişmiyorsa `None`.
    pub station: Option<Motion>,
}

/// Bir anın dokuz grahası.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct EphemerisRow {
    pub jd_ut: f64,
    /// Her zaman 9 eleman, `Graha::ALL` sırasında.
    pub cells: Vec<EphemerisCell>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct EphemerisTable {
    /// Verilen anların sonuncusu dışında her an için bir satır.
    pub rows: Vec<EphemerisRow>,
    /// Yalnızca `LowPrecision` olabilir.
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

fn invalid(field: &'static str) -> CoreError {
    CoreError::InvalidTransitInput(field)
}

/// Bir anın dokuz hücresi (işaretsiz) ve Güneş ya da Ay'ın Moshier yedeğiyle hesaplanıp
/// hesaplanmadığı.
fn sample<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    jd_ut: f64,
) -> Result<(Vec<EphemerisCell>, bool), CoreError> {
    let ayanamsa = eph.ayanamsa(jd_ut, settings.ayanamsa.sidereal_mode())?;
    finite(ayanamsa.deg, "ayanamsa")?;
    let (raw, low_precision) = raw_grahas(eph, jd_ut, rasi_frame(settings), settings.node)?;
    let cells = raw
        .iter()
        .map(|r| {
            let (nakshatra, pada) =
                nakshatra_pada(to_sidereal(settings.zodiac, r.lon, ayanamsa.deg));
            EphemerisCell {
                graha: r.graha,
                lon: r.lon,
                sign: Sign::of(r.lon),
                deg_in_sign: deg_in_sign(r.lon),
                retrograde: r.speed < 0.0,
                nakshatra,
                pada,
                ingress: None,
                station: None,
            }
        })
        .collect();
    Ok((cells, low_precision))
}

/// `moments` anlarının efemeris tablosu. Anlar artan sıradadır; sonuncusu yalnızca son satırın
/// işaretlerini kapatır ve satır olmaz: `n + 1` an `n` satır verir (1 ≤ n ≤ `EPHEMERIS_MAX_ROWS`).
/// Girdi hatası yalnızca alan adını taşır (`moments`).
pub fn ephemeris_table<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    moments: &[f64],
) -> Result<EphemerisTable, CoreError> {
    settings.validate()?;
    let count_ok = (2..=EPHEMERIS_MAX_ROWS + 1).contains(&moments.len());
    // NaN ve sonsuz da aralık dışındadır.
    let in_range = moments.iter().all(|jd| JD_UT_RANGE.contains(jd));
    let ordered = moments
        .windows(2)
        .all(|w| w[0] < w[1] && w[1] - w[0] <= EPHEMERIS_MAX_STEP_DAYS);
    if !(count_ok && in_range && ordered) {
        return Err(invalid("moments"));
    }
    let mut low_precision = false;
    let mut samples = Vec::with_capacity(moments.len());
    for &jd_ut in moments {
        let (cells, low) = sample(eph, settings, jd_ut)?;
        low_precision |= low;
        samples.push(cells);
    }
    let rows = moments[..moments.len() - 1]
        .iter()
        .enumerate()
        .map(|(i, &jd_ut)| {
            let cells = samples[i]
                .iter()
                .zip(&samples[i + 1])
                .map(|(now, next)| EphemerisCell {
                    ingress: (next.sign != now.sign).then_some(next.sign),
                    station: (STATION_GRAHAS.contains(&now.graha)
                        && next.retrograde != now.retrograde)
                        .then_some(if next.retrograde {
                            Motion::Retrograde
                        } else {
                            Motion::Direct
                        }),
                    ..now.clone()
                })
                .collect();
            EphemerisRow { jd_ut, cells }
        })
        .collect();
    Ok(EphemerisTable {
        rows,
        warnings: if low_precision {
            vec![ChartWarning::LowPrecision]
        } else {
            Vec::new()
        },
        method: EPHEMERIS_METHOD.to_owned(),
    })
}
````

**`app/src-tauri/src/transit_tools_commands/ephemeris.rs`: uygulama, test bloğunun üstüne (arada bir boş satır)**

````rust
//! Efemeris tablosu komutu (yol haritası §4.9): yerin bir takvim ayında, her günün başında
//! (yerin duvar saatiyle 00:00) dokuz grahanın konumu. Ay verilmezse yerin bugünkü ayıdır;
//! önceki ve sonraki ay Rust'ta hesaplanır (arayüz tarih hesabı yapmaz, C4).

use std::sync::Arc;

use atlas::Timestamp;
use ephemeris::{Engine, SwissEph};
use jyotish_core::{CalcSettings, ChartWarning, EphemerisRow};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use super::{invalid, TOOLS_YEAR_RANGE};
use crate::client_commands::blocking;
use crate::commands::CommandError;
use crate::muhurta_commands::{day_start_jd, offset_at, today, MuhurtaPlace, J2000_JD};

/// Bir takvim ayı (yerin takvimi): `month` 1..=12.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct MonthId {
    pub year: i32,
    pub month: u8,
}

impl MonthId {
    fn is_valid(self) -> bool {
        TOOLS_YEAR_RANGE.contains(&self.year) && (1..=12).contains(&self.month)
    }

    /// Önceki ay; yıl aralığının dışına çıkıyorsa `None`.
    pub fn prev(self) -> Option<MonthId> {
        let m = if self.month == 1 {
            MonthId {
                year: self.year - 1,
                month: 12,
            }
        } else {
            MonthId {
                month: self.month - 1,
                ..self
            }
        };
        m.is_valid().then_some(m)
    }

    /// Sonraki ay; yıl aralığının dışına çıkıyorsa `None`.
    pub fn next(self) -> Option<MonthId> {
        let m = self.following();
        m.is_valid().then_some(m)
    }

    /// Sonraki ay, aralık denetimi olmadan (son satırı kapatan an için).
    fn following(self) -> MonthId {
        if self.month == 12 {
            MonthId {
                year: self.year + 1,
                month: 1,
            }
        } else {
            MonthId {
                month: self.month + 1,
                ..self
            }
        }
    }

    fn first_day(self) -> Result<atlas::Date, CommandError> {
        let (year, month) = (
            i16::try_from(self.year).map_err(|_| invalid("month"))?,
            i8::try_from(self.month).map_err(|_| invalid("month"))?,
        );
        atlas::Date::new(year, month, 1).map_err(|_| invalid("month"))
    }
}

/// Bir günün satırı: yerin takvim günü (`YYYY-MM-DD`), o günün başındaki UTC ofseti ve konumlar.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct EphemerisDayView {
    pub date: String,
    pub offset_secs: i32,
    pub row: EphemerisRow,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct EphemerisMonthView {
    /// Tablonun ayı (ay verilmediyse yerin bugünkü ayı).
    pub month: MonthId,
    pub prev: Option<MonthId>,
    pub next: Option<MonthId>,
    /// Ayın günleri, sırayla (28–31 satır).
    pub days: Vec<EphemerisDayView>,
    /// Yalnızca `LowPrecision` olabilir.
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

/// `ephemeris_month` gövdesi; `now` testte sabitlenir. Bloklar.
pub fn ephemeris_month_blocking(
    engine: &Engine,
    month: Option<MonthId>,
    place: MuhurtaPlace,
    settings: CalcSettings,
    now: Timestamp,
) -> Result<EphemerisMonthView, CommandError> {
    // Yer, aydan ve motordan önce denetlenir.
    offset_at(&place, J2000_JD).map_err(|_| invalid("place"))?;
    let month = match month {
        Some(m) => m,
        None => {
            let date: atlas::Date = today(&place, now)
                .map_err(|_| invalid("place"))?
                .parse()
                .map_err(|_| invalid("month"))?;
            MonthId {
                year: i32::from(date.year()),
                month: date.month().unsigned_abs(),
            }
        }
    };
    if !month.is_valid() {
        return Err(invalid("month"));
    }
    let first = month.first_day()?;
    let start_of = |date: atlas::Date| {
        day_start_jd(&date.to_string(), &place, "month").map_err(|_| invalid("month"))
    };
    let mut dates = Vec::with_capacity(31);
    let mut moments = Vec::with_capacity(32);
    let mut day = first;
    for _ in 0..first.days_in_month() {
        moments.push(start_of(day)?);
        dates.push(day);
        day = day.tomorrow().map_err(|_| invalid("month"))?;
    }
    // Son satırı kapatan an: sonraki ayın ilk gününün başı.
    moments.push(start_of(month.following().first_day()?)?);
    let table = {
        let moments = moments.clone();
        engine.call(move |se: &mut SwissEph| {
            jyotish_core::ephemeris_table(se, &settings, &moments)
        })??
    };
    let days = dates
        .into_iter()
        .zip(table.rows)
        .map(|(date, row)| {
            Ok(EphemerisDayView {
                date: date.to_string(),
                offset_secs: offset_at(&place, row.jd_ut).map_err(|_| invalid("place"))?,
                row,
            })
        })
        .collect::<Result<Vec<_>, CommandError>>()?;
    Ok(EphemerisMonthView {
        month,
        prev: month.prev(),
        next: month.next(),
        days,
        warnings: table.warnings,
        method: table.method,
    })
}

/// Efemeris tablosu: yerin `month` takvim ayında her günün başındaki (00:00 duvar saati) dokuz
/// graha: burç, derece, retro, nakshatra ve pada; o gün içindeki burç girişi ve yön dönüşü
/// işaretli. `month` boşsa yerin bugünkü ayı ("bugün" Rust'ta okunur). `settings` açık haritanın
/// hesap ayarıdır (ayanamsa, düğüm türü, zodyak kipi). Yer loglanmaz.
#[tauri::command]
#[specta::specta]
pub async fn ephemeris_month(
    engine: State<'_, Arc<Engine>>,
    month: Option<MonthId>,
    place: MuhurtaPlace,
    settings: CalcSettings,
) -> Result<EphemerisMonthView, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || ephemeris_month_blocking(&engine, month, place, settings, Timestamp::now()))
        .await
}
````

- [ ] **Step 4: Rust testleri, fikstür ve bağlamalar (C10)**

```bash
cargo fmt --all --check
cargo test -p jyotish-core --lib transit_tools 2>&1 | grep -E "^test result"
cargo test -p jyotish-desk --lib transit_tools 2>&1 | grep -E "^test result"
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ephemeris_table 2>&1 | grep -E "^test result"
cargo test -p jyotish-desk --test ephemeris_table -- --nocapture 2>&1 | grep -E "^ephemeris_month|^test result"
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
grep -c "ephemerisMonth:" app/ui/bindings.ts
```
Expected: fmt çıktısız; `17 passed`; `1 passed`; `1 passed`; ölçüm satırı (bu plan yazılırken 2–4 ms; bütçe 100 ms) ve `1 passed`; `1 passed`; `1`. Gerçek motor testi işaretleri `timeline` dizilerinin sınırlarıyla üç ayda, iki zodyak kipinde ve gerçek düğümle karşılaştırır; uyuşmazlıkta önce kural (REVIEW §3.1 madde 3–5) okunur.

- [ ] **Step 5: Sarmalayıcı ve REVIEW kaydına efemeris bölümü**

````diff
diff --git a/app/ui/api.test.ts b/app/ui/api.test.ts
index b2e3e8a..3665ede 100644
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -67,6 +67,7 @@ vi.mock("./bindings", () => ({
     updateClient: vi.fn(),
     transitsAt: vi.fn(),
     sankrantiList: vi.fn(),
+    ephemerisMonth: vi.fn(),
     keepStoredMoment: vi.fn(),
     appInfo: vi.fn(),
     checkForUpdate: vi.fn(),
@@ -178,6 +179,7 @@ test("every wrapper forwards its arguments to the command of the same name", asy
     ["updateClient", [2, "A", null]],
     ["transitsAt", [{ kind: "now" }, { ayanamsa: "Lahiri" }, [], { birth_jd_ut: 1 }]],
     ["sankrantiList", [2026, { lat: 28.6, lon: 77.2, tz: { Iana: "Asia/Kolkata" } }, { ayanamsa: "Lahiri" }]],
+    ["ephemerisMonth", [{ year: 2026, month: 11 }, { lat: 28.6, lon: 77.2, tz: { Iana: "Asia/Kolkata" } }, { ayanamsa: "Lahiri" }]],
     ["keepStoredMoment", [3]],
     ["appInfo", []],
     ["checkForUpdate", []],
diff --git a/app/ui/api.ts b/app/ui/api.ts
index c5626c6..9d7d526 100644
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -18,6 +18,7 @@ import {
   type EventSearchSpec,
   type Gender,
   type Graha,
+  type MonthId,
   type MuhurtaPageCursor,
   type MuhurtaPlace,
   type MuhurtaQuery,
@@ -73,6 +74,8 @@ export const api = {
   // Geçiş araçları (yol haritası Plan 26). `year` boşsa yerin bugünkü yılıdır ("bugün" Rust'ta
   // okunur). Yer kişisel veridir: loglanmaz.
   sankrantiList: (year: number | null, place: MuhurtaPlace, settings: CalcSettings) => wrap(() => commands.sankrantiList(year, place, settings)),
+  // `month` boşsa yerin bugünkü ayıdır; önceki ve sonraki ay yanıtla gelir (arayüz tarih hesabı yapmaz).
+  ephemerisMonth: (month: MonthId | null, place: MuhurtaPlace, settings: CalcSettings) => wrap(() => commands.ephemerisMonth(month, place, settings)),
   deleteClient: (clientId: number) => wrap(() => commands.deleteClient(clientId)),
   deleteChart: (chartId: number) => wrap(() => commands.deleteChart(chartId)),
   listClients: (limit: number) => wrap(() => commands.listClients(limit)),
````

````diff
diff --git a/tests/golden/modules/transit_tools/REVIEW.md b/tests/golden/modules/transit_tools/REVIEW.md
index 7f1f74d..e859c04 100644
--- a/tests/golden/modules/transit_tools/REVIEW.md
+++ b/tests/golden/modules/transit_tools/REVIEW.md
@@ -62,3 +62,33 @@ Vimshottari sırasının üç kez yazılmış biçimidir ve testte o sırayla ka
 - [ ] Listenin tropikal-rasi kipinde de sidereal verilmesi (K5'ten bilinçli ayrılma).
 - [ ] Varanın gün doğumuna göre okunması (öteki okuma: yerin takvim günü, gece yarısından).
 - [ ] Yılın, seçilen yerin takvim yılı olması (öteki okuma: Mesha girişinden Mesha girişine).
+
+## 3. Efemeris tablosu (yöntem `ephemeris_table.v1`)
+
+### 3.1 Okuma kuralları
+
+1. **Satır.** Seçilen yerin takvim ayının her günü için bir satır; satırın anı o günün başıdır:
+   yerin duvar saatiyle 00:00. Yaz saati gece yarısında başlıyorsa (00:00 yoksa) satır günün
+   başladığı andadır; o gün önceki satırdan 23 saat sonradır.
+2. **Konum.** Dokuz grahanın boylamı transit hesabıyla aynıdır: burç ve burç içi derece rasi
+   çerçevesindedir (haritanın zodyak kipini izler, K5); nakshatra ve pada sidereal boylamdandır.
+   Ayanamsa ve düğüm türü açık haritanın ayarıdır. Retro, boylam hızının eksi olmasıdır; Rahu ve
+   Ketu'da retro işareti yazılmaz.
+3. **Giriş işareti.** Graha, satırın anı ile bir sonraki satırın anı arasında başka bir burca
+   giriyorsa satır girilen burcu taşır. Ayın son gününün işareti, sonraki ayın ilk gününün
+   başındaki konumdan okunur.
+4. **Dönüş işareti.** Mars, Merkür, Jüpiter, Venüs ve Satürn için: graha aynı süre içinde retroya
+   ya da düze dönüyorsa satır dönülen yönü taşır. Düğümlerin yönü işaretlenmez.
+5. **İşaretlerin sınırı.** İşaret, ardışık iki satırın karşılaştırmasıdır: kesin an olay
+   aramasındadır. Bir gün içinde burç sınırını geçip geri dönen durağan bir graha işaret almaz.
+6. **Aralık.** Bir ay (28–31 satır); yıl 1–2999. Ay verilmezse yerin bugünkü ayıdır. Ephemeris
+   dosyalarının dışındaki yıllarda (1800 öncesi, 2400 ve sonrası) tablo `LowPrecision` uyarısını
+   taşır.
+7. **Kapsam dışı.** CSV ve baskı (K10); enlem, hız ve deklinasyon sütunları; yıl görünümü;
+   00:00 UT satırı.
+
+### 3.2 Onay bekleyen noktalar (sahibi)
+
+- [ ] Satır saatinin seçilen yerin yerel gece yarısı olması (öteki okuma: 00:00 UT).
+- [ ] Varsayılan yerin bulunulan yer olması (öteki okuma: haritanın doğum yeri).
+- [ ] Burç ve derecenin haritanın zodyak kipini izlemesi (tropikal-rasi kipinde tropikal).
````

- [ ] **Step 6: Kapılar**

```bash
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -1
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
cargo test -p jyotish-core --lib 2>&1 | grep -E "^test result"
cargo test -p jyotish-desk --lib 2>&1 | grep -E "^test result"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
```
Expected: clippy `Finished`; sayaç `0`; `483 passed`; `59 passed`; Vitest `101 passed (101)`, `996 passed (996)`; `✓ built`.

- [ ] **Step 7: Commit**

```bash
git add crates/jyotish-core/src/lib.rs crates/jyotish-core/src/transit_tools app/src-tauri/src/lib.rs app/src-tauri/src/transit_tools_commands app/src-tauri/tests/ephemeris_table.rs app/ui/bindings.ts app/ui/test/fixtures/ephemeris.json app/ui/api.ts app/ui/api.test.ts tests/golden/modules/transit_tools/REVIEW.md
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(transit): ephemeris table core and month command with ingress and station marks" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: on iki dosya; yasaklı ad taraması çıktısız; `git status --short` boş.


---

### Task 5: Arayüz: Efemeris paneli

Yol haritası §4.9 ve görev taslağının (5). maddesi; D15, D16, D19–D21. Tek yeni stil dosyası `panels/ephemeris.css`'tir (renk yalnızca token'lardan).

**Files:**
- Create: `app/ui/test/ephemerisFixture.ts`, `app/ui/modules/useEphemeris.ts`, `app/ui/panels/ephemeris.css`, `app/ui/panels/EphemerisPanel.tsx`, `app/ui/panels/EphemerisPanel.test.tsx`, `app/e2e/ephemeris.desk.spec.ts`
- Modify: `app/ui/layout/model.ts`, `app/ui/layout/model.test.ts`, `app/ui/panels/registry.ts`, `app/ui/panels/panels.test.tsx`, `app/ui/i18n/tr.ts`, `en.ts`, `fr.ts`, `app/e2e/ipc.ts`
- Test: `EphemerisPanel.test.tsx` (10); e2e +1

**Interfaces:**
- Consumes: `api.ephemerisMonth`; `parseYear`, `YEAR_MIN`, `YEAR_MAX` (Task 3); yer denetimleri; `useModule`, `useLastData`; `useTransitControl`; `BalaScroll`; `formatDegree`; `GRAHA_IDS`, `RETRO_MARK`, `showsRetro`.
- Produces: `useEphemerisMonth(opened, place, month, scope)`; `EphemerisPanel`; `formatMonth(month, language)`, `formatDay(date, language)`; `INGRESS_MARK`, `STATION_MARK`; `ephemerisFixture()`; panel türü `ephemeris` (12 × 22); panel ayarı `nakshatra: boolean`; i18n `panel.ephemeris`, `ephemeris.*`; e2e taklidi `ephemeris_month`.

- [ ] **Step 1: Testler: yer, bu ay, işaretler, nakshatra kutusu, ay adımları, aralığın ucu, yeniden okuma, uyarı ve hata, biçimleme ve diller**

**Yeni dosya:** `app/ui/test/ephemerisFixture.ts`

````ts
// Efemeris fikstürü (app/src-tauri/tests/ephemeris_table.rs üretir): Delhi için Kasım 2026'nın
// `ephemeris_month` dönüşü (30 satır; dört burç girişi ve iki dönüş işareti Ay dışındaki
// grahalardadır). Ayrı dosyadır: `delhi.json` ve `fixture.ts` eşzamanlı planlarla paylaşılır.
import type { EphemerisMonthView, MuhurtaPlace } from "../bindings";
import raw from "./fixtures/ephemeris.json?raw";

export interface EphemerisFixture {
  /** Tablonun yeri: Delhi, `Asia/Kolkata`. */
  place: MuhurtaPlace;
  /** `ephemeris_month({ year: 2026, month: 11 }, place, varsayılan ayarlar)`. */
  month: EphemerisMonthView;
}

const FIXTURE = JSON.parse(raw) as EphemerisFixture;

/** Her çağrıda derin kopya: testler sonucu değiştirebilir. */
export function ephemerisFixture(): EphemerisFixture {
  return structuredClone(FIXTURE);
}
````

**Yeni dosya:** `app/ui/panels/EphemerisPanel.test.tsx`

````tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { ephemerisMonth: vi.fn(), searchPlaces: vi.fn(), setAppPrefs: vi.fn() } }));

import { api, type Result } from "../api";
import type { EphemerisMonthView, MonthId } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import type { Panel } from "../layout/model";
import { ephemerisFixture } from "../test/ephemerisFixture";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useTransitControl, type Transit } from "../transit/store";
import { useLayoutStore } from "../view/store";
import { EphemerisPanel, formatDay, formatMonth } from "./EphemerisPanel";
import { formatDegree } from "./format";

const EX = ephemerisFixture();
const ok = (view: EphemerisMonthView): Result<EphemerisMonthView> => ({ status: "ok", data: view });
const NOVEMBER: MonthId = { year: 2026, month: 11 };

let transit: Transit;
function LivePanel() {
  const { active } = useLayoutStore();
  transit = useTransitControl();
  return <EphemerisPanel panel={active.panels[0]} />;
}
/** Varsayılan: yer haritanın doğum yeridir (fikstürde bulunulan yer seçilmemiştir). */
function renderPanel(settings: Record<string, unknown> = { place: "chart" }) {
  const panel: Panel = { id: "ep", kind: "ephemeris", x: 0, y: 0, w: 12, h: 22, settings };
  const layout = { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
  return renderWithStores(<LivePanel />, { layout, opened: fixture().opened });
}
const rows = () => within(screen.getByRole("table")).getAllByRole("row").slice(1);
const row = (date: string) => document.querySelector<HTMLElement>(`tr[data-date="${date}"]`)!;
const cellOf = (date: string, graha: string) => row(date).querySelector<HTMLElement>(`td[data-graha="${graha}"]`)!;
const settle = () => act(async () => {});
const lastMonth = () => vi.mocked(api.ephemerisMonth).mock.calls.at(-1)![0];

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  // Tablo yeniden hesaplanmaz: istenen ay yanıtın `month` alanına yazılır.
  vi.mocked(api.ephemerisMonth).mockImplementation(async (month) => ok({ ...ephemerisFixture().month, month: month ?? NOVEMBER }));
});

test("without a place nothing is asked and the panel says how to pick one", () => {
  renderPanel({});
  expect(screen.getByRole("note")).toHaveTextContent(/Bulunulan yer seçilmedi/);
  expect(api.ephemerisMonth).not.toHaveBeenCalled();
  expect(screen.queryByRole("table")).toBeNull();
});

test("this month's table is asked with the place and the chart's settings; one row a day, nine grahas", async () => {
  renderPanel();
  const table = await screen.findByRole("table", { name: /^Kasım 2026: her günün başında/ });
  const opened = fixture().opened;
  const { lat, lon } = opened.chart_record.birth.place;
  expect(api.ephemerisMonth).toHaveBeenCalledTimes(1);
  // Ay boştur: "bu ay" Rust'ta okunur.
  expect(vi.mocked(api.ephemerisMonth).mock.calls[0]).toEqual([null, { lat, lon, tz: opened.chart_record.birth.tz }, opened.chart.meta.settings]);
  // Sütun başlıkları kısaltmadır; ekran okuyucu tam adı okur.
  expect(within(table).getAllByRole("columnheader").map((h) => h.textContent)).toEqual(["Gün", "GüGüneş", "AyAy", "MaMars", "MeMerkür", "JüJüpiter", "VeVenüs", "SaSatürn", "RaRahu", "KeKetu"]);
  expect(rows()).toHaveLength(30);
  // 1 Kasım 2026 Pazar'dır.
  const first = EX.month.days[0];
  expect(first.date).toBe("2026-11-01");
  expect(within(rows()[0]).getByRole("rowheader")).toHaveTextContent("01 Paz");
  // Güneş Terazi'dedir; derece Rust'ın değeridir, yalnızca biçimlenir.
  const sun = first.row.cells[0];
  expect([sun.graha, sun.sign, sun.retrograde]).toEqual(["Sun", "Libra", false]);
  expect(cellOf("2026-11-01", "Sun")).toHaveTextContent(new RegExp(`^Ter ${formatDegree(sun.deg_in_sign!)}$`));
  // Merkür o gün retrodur: düz "R" ve ekran okuyucu için sözü. Rahu ve Ketu'da "R" yazılmaz.
  expect(first.row.cells[3].retrograde && first.row.cells[7].retrograde).toBe(true);
  expect(cellOf("2026-11-01", "Mercury")).toHaveTextContent(/ Rretro$/);
  expect(cellOf("2026-11-01", "Rahu")).not.toHaveTextContent(/R/);
  // Nakshatra kutusu kapalıdır.
  expect(document.querySelector(".ephemeris-nakshatra")).toBeNull();
  expect(screen.getByRole("button", { name: "Bu ay" })).toHaveAttribute("aria-pressed", "true");
  expect(screen.getByText(/Yöntem: ephemeris_table\.v1/)).toBeInTheDocument();
  expect(screen.getByText(/→ burç: graha o gün içinde bu burca girer\. SR: o gün retroya döner; SD: o gün düze döner/)).toBeInTheDocument();
});

test("ingress and station marks are symbols with words, on the rows Rust marked", async () => {
  renderPanel();
  await screen.findByRole("table");
  // Fikstürde Ay dışındaki girişler: 6 Kasım Venüs Başak, 12 Kasım Mars Aslan, 16 Kasım Güneş
  // Akrep, 22 Kasım Venüs Terazi; dönüşler: 13 Kasım Merkür ve 14 Kasım Venüs düze döner.
  const marked = (kind: "ingress" | "station") =>
    EX.month.days.flatMap((d) => d.row.cells.filter((c) => c.graha !== "Moon" && c[kind] !== null).map((c) => `${d.date} ${c.graha} ${c[kind]}`));
  expect(marked("ingress")).toEqual(["2026-11-06 Venus Virgo", "2026-11-12 Mars Leo", "2026-11-16 Sun Scorpio", "2026-11-22 Venus Libra"]);
  expect(marked("station")).toEqual(["2026-11-13 Mercury Direct", "2026-11-14 Venus Direct"]);
  const mark = (date: string, graha: string, kind: string) => cellOf(date, graha).querySelector(`[data-mark="${kind}"]`);
  expect(mark("2026-11-16", "Sun", "ingress")).toHaveTextContent("→ Akro gün Akr burcuna girer");
  expect(mark("2026-11-12", "Mars", "ingress")).toHaveTextContent("→ Aslo gün Asl burcuna girer");
  expect(mark("2026-11-13", "Mercury", "station")).toHaveTextContent("SDo gün düze döner");
  expect(mark("2026-11-15", "Sun", "ingress")).toBeNull();
  expect(mark("2026-11-13", "Sun", "station")).toBeNull();
  // İşaret sayısı Rust'ın işaretlediği hücre sayısıdır (Ay dahil).
  const count = (kind: "ingress" | "station") => EX.month.days.reduce((n, d) => n + d.row.cells.filter((c) => c[kind] !== null).length, 0);
  expect(document.querySelectorAll('[data-mark="ingress"]')).toHaveLength(count("ingress"));
  expect(document.querySelectorAll('[data-mark="station"]')).toHaveLength(2);
  // Retroya dönüş işareti: sözü ve simgesi.
  const view = ephemerisFixture().month;
  view.days[2].row.cells[2].station = "Retrograde";
  vi.mocked(api.ephemerisMonth).mockResolvedValue(ok(view));
  fireEvent.click(screen.getByRole("button", { name: "Bu ay" }));
  fireEvent.click(screen.getByRole("button", { name: "Sonraki ay" }));
  await waitFor(() => expect(mark("2026-11-03", "Mars", "station")).toHaveTextContent("SRo gün retroya döner"));
});

test("the nakshatra box adds the nakshatra and pada to every cell and is saved with the layout", async () => {
  const { persistence } = renderPanel();
  await screen.findByRole("table");
  fireEvent.click(screen.getByRole("checkbox", { name: "Nakshatra" }));
  await waitFor(() => expect(document.querySelectorAll(".ephemeris-nakshatra")).toHaveLength(30 * 9));
  // 1 Kasım: Güneş 15. nakshatrada (Swati), 3. padada.
  const sun = EX.month.days[0].row.cells[0];
  expect([sun.nakshatra, sun.pada]).toEqual([15, 3]);
  expect(cellOf("2026-11-01", "Sun").querySelector(".ephemeris-nakshatra")).toHaveTextContent(/^Swati 3$/);
  await waitFor(() => expect(persistence.saved.length).toBeGreaterThan(0));
  const saved = persistence.saved.at(-1)!.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!;
  expect(saved.panels[0].settings).toEqual({ place: "chart", nakshatra: true });
  // Kutu yeni bir istek başlatmaz: tablo aynıdır.
  expect(api.ephemerisMonth).toHaveBeenCalledTimes(1);
});

test("previous, next and this month use the months Rust returned; a chosen month is asked on submit", async () => {
  renderPanel();
  await screen.findByRole("table");
  expect([EX.month.prev, EX.month.next]).toEqual([
    { year: 2026, month: 10 },
    { year: 2026, month: 12 },
  ]);
  fireEvent.click(screen.getByRole("button", { name: "Sonraki ay" }));
  await screen.findByRole("table", { name: /^Aralık 2026:/ });
  expect(lastMonth()).toEqual({ year: 2026, month: 12 });
  expect(screen.getByRole("button", { name: "Bu ay" })).toHaveAttribute("aria-pressed", "false");
  fireEvent.click(screen.getByRole("button", { name: "Önceki ay" }));
  await settle();
  expect(lastMonth()).toEqual({ year: 2026, month: 10 });
  // Ay ve yıl kutuları gösterilen ayı izler; seçim "Ayı göster" ile gider.
  expect(screen.getByRole("combobox", { name: "Ay" })).toHaveDisplayValue("Ekim");
  expect(screen.getByRole("spinbutton", { name: "Yıl" })).toHaveValue(2026);
  fireEvent.change(screen.getByRole("combobox", { name: "Ay" }), { target: { value: "3" } });
  fireEvent.change(screen.getByRole("spinbutton", { name: "Yıl" }), { target: { value: "2030" } });
  const asked = vi.mocked(api.ephemerisMonth).mock.calls.length;
  fireEvent.click(screen.getByRole("button", { name: "Ayı göster" }));
  await screen.findByRole("table", { name: /^Mart 2030:/ });
  expect(vi.mocked(api.ephemerisMonth).mock.calls).toHaveLength(asked + 1);
  expect(lastMonth()).toEqual({ year: 2030, month: 3 });
  // Geçersiz yıl gönderilmez.
  fireEvent.change(screen.getByRole("spinbutton", { name: "Yıl" }), { target: { value: "3000" } });
  expect(screen.getByRole("spinbutton", { name: "Yıl" })).toHaveAttribute("aria-invalid", "true");
  fireEvent.click(screen.getByRole("button", { name: "Ayı göster" }));
  await settle();
  expect(vi.mocked(api.ephemerisMonth).mock.calls).toHaveLength(asked + 1);
  fireEvent.click(screen.getByRole("button", { name: "Bu ay" }));
  await screen.findByRole("table", { name: /^Kasım 2026:/ });
  expect(lastMonth()).toBeNull();
  expect(screen.getByRole("spinbutton", { name: "Yıl" })).toHaveValue(2026);
});

test("at the end of the range the step button stays focusable and does nothing", async () => {
  vi.mocked(api.ephemerisMonth).mockResolvedValue(ok({ ...ephemerisFixture().month, month: { year: 1, month: 1 }, prev: null }));
  renderPanel();
  await screen.findByRole("table");
  const prev = screen.getByRole("button", { name: "Önceki ay" });
  expect(prev).toHaveAttribute("aria-disabled", "true");
  expect(prev).not.toBeDisabled();
  fireEvent.click(prev);
  await settle();
  expect(api.ephemerisMonth).toHaveBeenCalledTimes(1);
  expect(screen.getByRole("button", { name: "Sonraki ay" })).not.toHaveAttribute("aria-disabled");
});

test("the table stays while another month is read; a day moves the transit date", async () => {
  renderPanel();
  await screen.findByRole("table");
  let resolve!: (r: Result<EphemerisMonthView>) => void;
  vi.mocked(api.ephemerisMonth).mockImplementationOnce(() => new Promise((r) => (resolve = r)));
  fireEvent.click(screen.getByRole("button", { name: "Sonraki ay" }));
  await settle();
  expect(screen.getByRole("table", { name: /^Kasım 2026:/ })).toBeInTheDocument();
  await act(async () => resolve(ok({ ...ephemerisFixture().month, month: { year: 2026, month: 12 } })));
  expect(screen.getByRole("table", { name: /^Aralık 2026:/ })).toBeInTheDocument();
  const day = within(rows()[5]).getByRole("button");
  expect(day).toHaveAccessibleName("2026-11-06: transit tarihini bu günün başına götür");
  fireEvent.click(day);
  expect(transit.at).toEqual({ Jd: EX.month.days[5].row.jd_ut });
});

test("the fallback ephemeris is written out; an input error names the field and can be retried", async () => {
  vi.mocked(api.ephemerisMonth).mockResolvedValueOnce({ status: "error", error: { code: "invalid_transit_input", params: { field: "month" } } });
  vi.mocked(api.ephemerisMonth).mockResolvedValueOnce(ok({ ...ephemerisFixture().month, warnings: ["LowPrecision"] }));
  renderPanel();
  expect(await screen.findByRole("alert")).toHaveTextContent("Transit isteği geçersiz: ay.");
  expect(screen.queryByRole("table")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await screen.findByRole("table");
  expect(screen.getByRole("note")).toHaveTextContent(/^Düşük hassasiyet: /);
  expect(screen.queryByRole("alert")).toBeNull();
});

test("months and days are written in the interface language", () => {
  expect([formatMonth(NOVEMBER, "tr"), formatMonth(NOVEMBER, "en"), formatMonth(NOVEMBER, "fr")]).toEqual(["Kasım 2026", "November 2026", "novembre 2026"]);
  // İki basamaklı yıl 1900'lere taşınmaz.
  expect(formatMonth({ year: 50, month: 1 }, "en")).toBe("January 50");
  expect(formatDay("2026-11-01", "tr")).toBe("01 Paz");
  expect(formatDay("2026-11-16", "tr")).toBe("16 Pzt");
  expect(formatDay("2026-11-01", "en")).toMatch(/Sun/);
  expect(formatDay("bozuk", "tr")).toBe("bozuk");
});

test("the texts exist in the three languages; French uses the typographic apostrophe", () => {
  expect([tr.panel.ephemeris, en.panel.ephemeris, fr.panel.ephemeris]).toEqual(["Efemeris", "Ephemeris", "Éphémérides"]);
  for (const m of [tr, en, fr]) {
    expect(Object.keys(m.ephemeris).sort()).toEqual(Object.keys(tr.ephemeris).sort());
    expect(Object.keys(m.ephemeris.station).sort()).toEqual(["Direct", "Retrograde"]);
    expect(m.ephemeris.legend).toContain("{{ingress}}");
    expect(m.ephemeris.legend).toContain("{{station}}");
  }
  expect(JSON.stringify(fr.ephemeris)).not.toContain("'");
  expect(fr.ephemeris.legend).toContain("recherche d’événements");
});
````

````diff
diff --git a/app/ui/layout/model.test.ts b/app/ui/layout/model.test.ts
index b73fd97..d6e968d 100644
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -57,6 +57,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
     "prashna_facts",
     "event_search",
     "sankranti",
+    "ephemeris",
     "navatara",
     "vimshopaka",
   ]);
@@ -66,7 +67,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
 });
 
 test("module panels are known kinds, listed once, with a size the grid accepts", () => {
-  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "bhava_table", "sudarshana", "yogas", "avastha", "compat", "muhurta_day", "muhurta_search", "prashna_facts", "event_search", "sankranti", "navatara", "vimshopaka"]);
+  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "bhava_table", "sudarshana", "yogas", "avastha", "compat", "muhurta_day", "muhurta_search", "prashna_facts", "event_search", "sankranti", "ephemeris", "navatara", "vimshopaka"]);
   for (const m of MODULE_PANELS) {
     expect(isKnownPanelKind(m.kind)).toBe(true);
     expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
diff --git a/app/ui/panels/panels.test.tsx b/app/ui/panels/panels.test.tsx
index 179f7e2..17cb8d9 100644
--- a/app/ui/panels/panels.test.tsx
+++ b/app/ui/panels/panels.test.tsx
@@ -17,6 +17,7 @@ import { BirthInfoPanel } from "./BirthInfoPanel";
 import { ChartPanel } from "./ChartPanel";
 import { CompatPanel } from "./CompatPanel";
 import { DrishtiPanel } from "./DrishtiPanel";
+import { EphemerisPanel } from "./EphemerisPanel";
 import { GrahaTablePanel } from "./GrahaTablePanel";
 import { JaiminiKarakasPanel } from "./JaiminiKarakasPanel";
 import { MuhurtaDayPanel } from "./MuhurtaDayPanel";
@@ -68,6 +69,7 @@ test("registry covers every catalog kind", () => {
   expect(PANEL_REGISTRY.muhurta_search.component).toBe(MuhurtaSearchPanel);
   expect(PANEL_REGISTRY.event_search.component).toBe(EventSearchPanel);
   expect(PANEL_REGISTRY.sankranti.component).toBe(SankrantiPanel);
+  expect(PANEL_REGISTRY.ephemeris.component).toBe(EphemerisPanel);
   expect(PANEL_REGISTRY.navatara.component).toBe(NavataraPanel);
   expect(PANEL_REGISTRY.vimshopaka.component).toBe(VimshopakaPanel);
   // Kataloğun her türü uygulanmıştır: yer tutucu yoktur.
````

**Yeni dosya:** `app/e2e/ephemeris.desk.spec.ts`

````ts
// Efemeris akışı (Plan 26). Ayrı dosyadır: `desk.spec.ts` eşzamanlı planlarla paylaşılır; "e2e"
// projesi adı `desk.spec.ts` ile biten her dosyayı koşar.
import { expect, test } from "@playwright/test";
import { FIXTURE, ipcCalls, mockIpc } from "./ipc";

type EphemerisCall = { month: { year: number; month: number } | null; place: { lat: number; lon: number; tz: unknown }; settings: { schema_version: number } };

test("ephemeris: a palette panel with a month of daily positions, month steps from Rust, marks in words and a day that moves the transit date", async ({ page }) => {
  await mockIpc(page, { session: true });
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  const calls = async () => (await ipcCalls(page, "ephemeris_month")) as EphemerisCall[];
  // Tablo panel açılmadan istenmez (ileri modüller spec'i K1).
  expect(await calls()).toHaveLength(0);

  await page.keyboard.press("Meta+k");
  const palette = page.getByRole("dialog", { name: "Komut paleti" });
  await palette.getByRole("combobox").fill("efemeris");
  await palette.getByRole("option", { name: /Efemeris panelini aç\/kapat/ }).click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
  const panel = page.getByRole("region", { name: "Efemeris" });
  await expect(panel).toBeFocused();
  await expect(panel).toBeInViewport();

  // Bulunulan yer seçilmemiştir: tablo istenmez, panel nedenini söyler.
  await expect(panel.getByRole("note")).toContainText("Bulunulan yer seçilmedi");
  expect(await calls()).toHaveLength(0);
  await panel.getByRole("combobox", { name: "Yer" }).selectOption("chart");

  const month = FIXTURE.transit_tools.ephemeris.month;
  const table = panel.getByRole("table", { name: /^Kasım 2026: her günün başında/ });
  await expect(table.locator("tbody tr")).toHaveCount(month.days.length);
  const sent = await calls();
  expect(sent).toHaveLength(1);
  // Ay boştur ("bu ay" Rust'ta okunur); yer haritanın doğum yeridir.
  expect(sent[0].month).toBeNull();
  expect(sent[0].place).toMatchObject({ lat: 28.6139, lon: 77.209, tz: { Iana: "Asia/Kolkata" } });
  expect(sent[0].settings.schema_version).toBe(2);
  // On sütun: gün ve dokuz graha. 1 Kasım 2026 Pazar'dır.
  await expect(table.locator("thead th")).toHaveCount(10);
  await expect(table.locator('tr[data-date="2026-11-01"] th')).toHaveText("01 Paz");
  // 16 Kasım: Güneş o gün Akrep'e girer; işaret simge ve sözdür.
  const sun = table.locator('tr[data-date="2026-11-16"] td[data-graha="Sun"]');
  await expect(sun.locator('[data-mark="ingress"]')).toContainText("→ Akr");
  await expect(sun.locator('[data-mark="ingress"]')).toContainText("o gün Akr burcuna girer");
  await expect(table.locator('tr[data-date="2026-11-13"] td[data-graha="Mercury"] [data-mark="station"]')).toContainText("o gün düze döner");

  // Nakshatra kutusu yeni istek başlatmaz; hücrelere nakshatra ve pada eklenir.
  await expect(panel.locator(".ephemeris-nakshatra")).toHaveCount(0);
  await panel.getByRole("checkbox", { name: "Nakshatra" }).check();
  await expect(table.locator('tr[data-date="2026-11-01"] td[data-graha="Sun"] .ephemeris-nakshatra')).toHaveText("Swati 3");
  expect(await calls()).toHaveLength(1);

  // Sonraki ay: istek, Rust'ın döndürdüğü `next` ayını taşır; odak düğmede durur.
  const next = panel.getByRole("button", { name: "Sonraki ay" });
  await next.focus();
  await page.keyboard.press("Enter");
  await expect(panel.getByRole("table", { name: /^Aralık 2026:/ })).toBeVisible();
  expect((await calls()).at(-1)!.month).toEqual({ year: 2026, month: 12 });
  await expect(next).toBeFocused();

  // Gün, transit tarihini o günün başına götürür.
  const before = (await ipcCalls(page, "transits_at")).length;
  await panel.getByRole("table").locator('tr[data-date="2026-11-06"] th').getByRole("button").click();
  await expect.poll(async () => (await ipcCalls(page, "transits_at")).length).toBe(before + 1);
  const moved = (await ipcCalls(page, "transits_at")).at(-1) as { at: { Jd: number } };
  expect(moved.at).toEqual({ Jd: month.days[5].row.jd_ut });
});
````

- [ ] **Step 2: Testlerin düştüğünü gör**

```bash
pnpm --dir app exec vitest run ui/panels/EphemerisPanel.test.tsx ui/layout/model.test.ts ui/panels/panels.test.tsx 2>&1 | grep -E "Test Files|Tests "
```
Expected: üç dosya da düşer.

- [ ] **Step 3: Kanca, stil, panel, katalog, metinler ve e2e taklidi**

**Yeni dosya:** `app/ui/modules/useEphemeris.ts`

````ts
// Efemeris tablosunu (`ephemeris_month`) isteyen kanca (yol haritası Plan 26, §4.9). Sonuç
// saklanmaz ve arayüzde hesap yoktur (K1, C4); panel açıkken istenir. `month === null` yerin
// bugünkü ayıdır: "bugün" Rust'ta okunur; önceki ve sonraki ay yanıtla gelir. Yeni bir ay
// okunurken aynı panelin son tablosu gösterilmeyi sürdürür (tablo ve odak yerinde durur). Yer
// kişisel veridir: anahtar ve girdiler loglanmaz.
import { api } from "../api";
import type { EphemerisMonthView, HomePlace, MonthId, OpenedClient } from "../bindings";
import { muhurtaPlace } from "./muhurta";
import { useLastData, useModule, type ModuleState } from "./useModule";

export interface EphemerisState extends ModuleState<EphemerisMonthView> {
  /** Gösterilecek tablo: yeni yanıt gelene dek bir önceki. */
  shown: EphemerisMonthView | null;
}

/** `place === null`: yer seçilmemiş, komut çağrılmaz. `scope` panelin kimliğidir. */
export function useEphemerisMonth(opened: OpenedClient, place: HomePlace | null, month: MonthId | null, scope: string): EphemerisState {
  const settings = opened.chart.meta.settings;
  const key = place !== null ? JSON.stringify([opened.chart_record.id, muhurtaPlace(place), month, settings]) : null;
  const state = useModule(key, () => api.ephemerisMonth(month, muhurtaPlace(place as HomePlace), settings));
  const shown = useLastData(state, key === null ? null : `${opened.chart_record.id}:${scope}`);
  return { ...state, shown };
}
````

**Yeni dosya:** `app/ui/panels/ephemeris.css`

````css
/* Efemeris paneli: dar hücreler ve hücre içi ikinci satır. Renk token'lardan gelir (C9). */
.ephemeris-table th,
.ephemeris-table td {
  white-space: nowrap;
}
.ephemeris-table td {
  font-variant-numeric: tabular-nums;
}
.ephemeris-mark {
  color: var(--text-2);
}
.ephemeris-nakshatra {
  display: block;
  color: var(--text-2);
}
.ephemeris-month {
  font-weight: var(--fw-medium);
}
````

**Yeni dosya:** `app/ui/panels/EphemerisPanel.tsx`

````tsx
// Efemeris paneli (yol haritası Plan 26, §4.9): seçilen ayın her günü için dokuz grahanın burcu,
// derecesi ve retro işareti; isteğe bağlı nakshatra ve pada. Tablo `ephemeris_month`'tan gelir
// ve panel açıkken istenir (K1); burada hesap yoktur (C4): "bu ay", önceki ve sonraki ay Rust'tan
// gelir. Satırın anı günün başıdır (seçilen yerin saatiyle 00:00); gün, transit tarihini o ana
// götürür. O gün içindeki burç girişi ve yön dönüşü işaretlidir; işaret renkle değil, simge ve
// sözle verilir (C9). Ay saklanmaz; "Nakshatra" kutusu panelin ayarıdır ve düzenle birlikte
// saklanır. Retro düz `R`'dir ve düğümlerde yazılmaz. CSV ve baskı yoktur (K10).
import { useId, useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import type { CommandError, EphemerisCell, EphemerisMonthView, MonthId, OpenedClient } from "../bindings";
import { signIndex } from "../chart/adapter";
import { GRAHA_IDS, RETRO_MARK, showsRetro, type GrahaId } from "../chart/types";
import type { Panel } from "../layout/model";
import { useEphemerisMonth } from "../modules/useEphemeris";
import { useSession } from "../session/store";
import { useTransitControl } from "../transit/store";
import { useLayoutStore } from "../view/store";
import { BalaScroll } from "./bala";
import "./ephemeris.css";
import { formatDegree } from "./format";
import { nakshatraName } from "./labels";
import { PlaceControls, PlaceMissing, usePanelPlace } from "./MuhurtaPlace";
import "./panels.css";
import { parseYear, YEAR_MAX, YEAR_MIN } from "./SankrantiPanel";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "./states";
import type { PanelProps } from "./types";

/** Burç girişi ve yön dönüşü işaretleri; anlamları tablonun altında yazılıdır. */
export const INGRESS_MARK = "→";
export const STATION_MARK = "S";
const MONTHS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12] as const;

/** Ayın adı ve yılı arayüz dilinde ("Kasım 2026"); biçimlemedir, hesap değil. */
export function formatMonth(month: MonthId, language: string): string {
  const d = new Date(Date.UTC(2000, month.month - 1, 1));
  // `Date.UTC` 0–99 arası yılı 1900'lere taşır: yıl ayrıca yazılır.
  d.setUTCFullYear(month.year);
  return new Intl.DateTimeFormat(language, { month: "long", year: "numeric", timeZone: "UTC" }).format(d);
}

/** Ayın adı ("Kasım"). */
function monthName(month: number, language: string): string {
  return new Intl.DateTimeFormat(language, { month: "long", timeZone: "UTC" }).format(new Date(Date.UTC(2000, month - 1, 1)));
}

/** `YYYY-MM-DD` → gün ve hafta günü ("01 Paz"); biçimlemedir. Okunamayan tarih olduğu gibi yazılır. */
export function formatDay(date: string, language: string): string {
  const d = new Date(`${date}T00:00:00Z`);
  if (Number.isNaN(d.getTime())) return date;
  return new Intl.DateTimeFormat(language, { day: "2-digit", weekday: "short", timeZone: "UTC" }).format(d);
}

export function EphemerisPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  return <Body panel={panel} opened={opened} />;
}

/** `invalid_transit_input`'un alan adını arayüz dilinde yazar; öteki hatalar olduğu gibi durur. */
function fieldError(error: CommandError, name: (field: string) => string): CommandError {
  const field = error.params.field;
  if (error.code !== "invalid_transit_input" || typeof field !== "string") return error;
  return { ...error, params: { ...error.params, field: name(field) } };
}

function Body({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t, i18n } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  const place = usePanelPlace(panel, opened);
  const nakshatra = panel.settings.nakshatra === true;
  // `null`: bu ay (Rust okur). Ay oturumluktur; düzenle birlikte saklanmaz.
  const [month, setMonth] = useState<MonthId | null>(null);
  const table = useEphemerisMonth(opened, place.place, month, panel.id);
  const view = table.shown;
  const phase = usePanelLoadPhase(table.loading);
  const [year, setYear] = useState("");
  const [monthNo, setMonthNo] = useState(1);
  const [editing, setEditing] = useState(false);
  // Gösterilen ay değişince kutular onu gösterir; kullanıcı seçerken değişmez (`TransitDate`
  // ile aynı kalıp: eşitleme çizim sırasında).
  const target = editing || view === null ? null : `${view.month.year}-${view.month.month}`;
  const [synced, setSynced] = useState(target);
  if (target !== synced) {
    setSynced(target);
    if (view !== null && target !== null) {
      setYear(String(view.month.year));
      setMonthNo(view.month.month);
    }
  }
  const typed = parseYear(year);
  const go = (next: MonthId | null) => {
    setEditing(false);
    setMonth(next);
  };
  const submit = (e: FormEvent) => {
    e.preventDefault();
    if (typed === null) return;
    go({ year: typed, month: monthNo });
  };
  const shownError = table.error === null ? null : fieldError(table.error, (f) => (i18n.exists(`ephemeris.field.${f}`) ? t(`ephemeris.field.${f}`) : f));
  const prev = view?.prev ?? null;
  const next = view?.next ?? null;

  return (
    <div className="module-panel">
      <form className="muhurta-form" aria-label={t("ephemeris.form")} onSubmit={submit}>
        <div className="module-toolbar">
          <PlaceControls panel={panel} opened={opened} at={place} />
        </div>
        <div className="module-toolbar">
          {/* Kullanılamazken yerel `disabled` değildir: odak düğmede durur (ay değişirken de). */}
          <button type="button" aria-disabled={prev === null ? true : undefined} onClick={() => prev !== null && go(prev)}>
            {t("ephemeris.prev")}
          </button>
          <button type="button" aria-disabled={next === null ? true : undefined} onClick={() => next !== null && go(next)}>
            {t("ephemeris.next")}
          </button>
          <button type="button" aria-pressed={month === null} onClick={() => go(null)}>
            {t("ephemeris.thisMonth")}
          </button>
          <label htmlFor={`${id}-month`}>{t("ephemeris.month")}</label>
          <select
            id={`${id}-month`}
            value={monthNo}
            onChange={(e) => {
              setEditing(true);
              setMonthNo(Number(e.target.value));
            }}
          >
            {MONTHS.map((m) => (
              <option key={m} value={m}>
                {monthName(m, i18n.language)}
              </option>
            ))}
          </select>
          <label htmlFor={`${id}-year`}>{t("ephemeris.year")}</label>
          <input
            id={`${id}-year`}
            type="number"
            inputMode="numeric"
            min={YEAR_MIN}
            max={YEAR_MAX}
            step={1}
            value={year}
            aria-invalid={editing && typed === null ? true : undefined}
            onChange={(e) => {
              setEditing(true);
              setYear(e.target.value);
            }}
          />
          <button type="submit">{t("ephemeris.show")}</button>
          <label>
            <input type="checkbox" checked={nakshatra} onChange={(e) => setPanelSettings(panel.id, { nakshatra: e.target.checked })} />
            {t("ephemeris.nakshatra")}
          </label>
        </div>
      </form>
      {place.place === null ? (
        <PlaceMissing choice={place.choice} />
      ) : shownError ? (
        <PanelError error={shownError} onRetry={table.retry} />
      ) : view === null ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : (
        <Table view={view} nakshatra={nakshatra} />
      )}
    </div>
  );
}

function Cell({ cell, nakshatra }: { cell: EphemerisCell; nakshatra: boolean }) {
  const { t } = useTranslation();
  const retro = showsRetro(cell.graha as GrahaId, cell.retrograde);
  return (
    <td data-graha={cell.graha}>
      {t(`sign.${signIndex(cell.sign)}`)} {cell.deg_in_sign === null ? "—" : formatDegree(cell.deg_in_sign)}
      {retro && (
        <>
          {" "}
          <span aria-hidden="true">{RETRO_MARK}</span>
          <span className="visually-hidden">{t("chart.retro")}</span>
        </>
      )}
      {cell.ingress !== null && (
        <span className="ephemeris-mark" data-mark="ingress">
          {" "}
          <span aria-hidden="true">
            {INGRESS_MARK} {t(`sign.${signIndex(cell.ingress)}`)}
          </span>
          <span className="visually-hidden">{t("ephemeris.ingress", { sign: t(`sign.${signIndex(cell.ingress)}`) })}</span>
        </span>
      )}
      {cell.station !== null && (
        <span className="ephemeris-mark" data-mark="station">
          {" "}
          <span aria-hidden="true">
            {STATION_MARK}
            {cell.station === "Retrograde" ? "R" : "D"}
          </span>
          <span className="visually-hidden">{t(`ephemeris.station.${cell.station}`)}</span>
        </span>
      )}
      {nakshatra && (
        <span className="ephemeris-nakshatra">
          {nakshatraName(cell.nakshatra)} {cell.pada}
        </span>
      )}
    </td>
  );
}

function Table({ view, nakshatra }: { view: EphemerisMonthView; nakshatra: boolean }) {
  const { t, i18n } = useTranslation();
  const transit = useTransitControl();
  const captionId = useId();
  const title = formatMonth(view.month, i18n.language);
  return (
    <>
      {view.warnings.includes("LowPrecision") && (
        <p className="panel-notice" role="note">
          <strong>{t("badge.LowPrecision")}</strong>: {t("badgeHint.LowPrecision")}
        </p>
      )}
      <BalaScroll labelledBy={captionId}>
        <table className="module-table ephemeris-table">
          <caption id={captionId}>
            <span className="ephemeris-month">{title}</span>: {t("ephemeris.caption")}
          </caption>
          <thead>
            <tr>
              <th scope="col">{t("ephemeris.day")}</th>
              {GRAHA_IDS.map((g) => (
                <th key={g} scope="col">
                  <abbr title={t(`grahaName.${g}`)} aria-hidden="true">
                    {t(`graha.${g}`)}
                  </abbr>
                  <span className="visually-hidden">{t(`grahaName.${g}`)}</span>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {view.days.map((day) => {
              const jd = day.row.jd_ut;
              const label = formatDay(day.date, i18n.language);
              return (
                <tr key={day.date} data-date={day.date}>
                  <th scope="row">
                    {jd === null ? (
                      label
                    ) : (
                      <button type="button" className="link-button" aria-label={t("ephemeris.goTo", { date: day.date })} onClick={() => transit.setAt({ Jd: jd })}>
                        {label}
                      </button>
                    )}
                  </th>
                  {day.row.cells.map((cell) => (
                    <Cell key={cell.graha} cell={cell} nakshatra={nakshatra} />
                  ))}
                </tr>
              );
            })}
          </tbody>
        </table>
      </BalaScroll>
      <p className="panel-message">{t("ephemeris.legend", { ingress: INGRESS_MARK, station: STATION_MARK })}</p>
      <p className="panel-message">{t("ephemeris.method", { method: view.method })}</p>
    </>
  );
}
````

````diff
diff --git a/app/ui/layout/model.ts b/app/ui/layout/model.ts
index ba26fb1..c52d052 100644
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -28,6 +28,7 @@ export const PANEL_KINDS = [
   "prashna_facts",
   "event_search",
   "sankranti",
+  "ephemeris",
   "navatara",
   "vimshopaka",
 ] as const;
@@ -67,6 +68,7 @@ export const MODULE_PANELS = [
   { kind: "event_search", w: 7, h: 18 },
   // Geçiş araçları (yol haritası Plan 26).
   { kind: "sankranti", w: 7, h: 16 },
+  { kind: "ephemeris", w: 12, h: 22 },
   // Doğum anı ekleri (Plan 23).
   { kind: "navatara", w: 7, h: 16 },
   { kind: "vimshopaka", w: 7, h: 12 },
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index aabfdee..16a8df2 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -15,6 +15,7 @@ import { ChartPanel, chartVarga } from "./ChartPanel";
 import { CompatPanel } from "./CompatPanel";
 import { TransitTablePanel } from "../transit/TransitTablePanel";
 import { DrishtiPanel } from "./DrishtiPanel";
+import { EphemerisPanel } from "./EphemerisPanel";
 import { GrahaTablePanel } from "./GrahaTablePanel";
 import { JaiminiKarakasPanel } from "./JaiminiKarakasPanel";
 import { MuhurtaDayPanel } from "./MuhurtaDayPanel";
@@ -73,6 +74,7 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   event_search: { titleKey: "panel.event_search", component: EventSearchPanel },
   // Geçiş araçları (yol haritası Plan 26).
   sankranti: { titleKey: "panel.sankranti", component: SankrantiPanel },
+  ephemeris: { titleKey: "panel.ephemeris", component: EphemerisPanel },
   // Doğum anı ekleri (Plan 23).
   navatara: { titleKey: "panel.navatara", component: NavataraPanel },
   vimshopaka: { titleKey: "panel.vimshopaka", component: VimshopakaPanel },
````

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index 1477789..1158bf4 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -71,6 +71,7 @@ export const en: Messages = {
     dasha_strip: "Dasha strip",
     transit_table: "Transit table",
     sankranti: "Sankranti",
+    ephemeris: "Ephemeris",
     notes: "Notes",
     birth_info: "Birth details",
     panchang: "Panchang",
@@ -1390,6 +1391,24 @@ export const en: Messages = {
     field: { year: "year", place: "place" },
     method: "Method: {{method}}. Ingresses are into sidereal signs; the tithi, vara and nakshatra are those of the moment of ingress. Times are in the selected place's time zone; the moment in a row changes the transit date.",
   },
+  ephemeris: {
+    form: "Ephemeris table settings",
+    prev: "Previous month",
+    next: "Next month",
+    thisMonth: "This month",
+    month: "Month",
+    year: "Year",
+    show: "Show month",
+    nakshatra: "Nakshatra",
+    caption: "sign and degree of the nine grahas at the start of each day (00:00, time of the selected place)",
+    day: "Day",
+    goTo: "{{date}}: move the transit date to the start of this day",
+    ingress: "enters {{sign}} that day",
+    station: { Retrograde: "turns retrograde that day", Direct: "turns direct that day" },
+    legend: "{{ingress}} sign: the graha enters this sign during that day. {{station}}R: turns retrograde that day; {{station}}D: turns direct that day. The exact moments are in the event search.",
+    field: { month: "month", place: "place" },
+    method: "Method: {{method}}. Sign and degree follow the chart's zodiac mode; the nakshatra is from the sidereal longitude. A day moves the transit date to the start of that day.",
+  },
   transit: {
     date: "Transit moment",
     dateLabel: "Date",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index a16ad87..2ba95dc 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -71,6 +71,7 @@ export const fr: Messages = {
     dasha_strip: "Bande des dashas",
     transit_table: "Tableau des transits",
     sankranti: "Sankranti",
+    ephemeris: "Éphémérides",
     notes: "Notes",
     birth_info: "Données de naissance",
     panchang: "Panchang",
@@ -1390,6 +1391,24 @@ export const fr: Messages = {
     field: { year: "année", place: "lieu" },
     method: "Méthode : {{method}}. Les entrées se font dans les signes sidéraux ; le tithi, le vara et le nakshatra sont ceux du moment de l’entrée. Les heures sont dans le fuseau du lieu choisi ; le moment d’une ligne change la date de transit.",
   },
+  ephemeris: {
+    form: "Réglages de la table des éphémérides",
+    prev: "Mois précédent",
+    next: "Mois suivant",
+    thisMonth: "Ce mois-ci",
+    month: "Mois",
+    year: "Année",
+    show: "Afficher le mois",
+    nakshatra: "Nakshatra",
+    caption: "signe et degré des neuf grahas au début de chaque jour (00:00, heure du lieu choisi)",
+    day: "Jour",
+    goTo: "{{date}} : placer la date de transit au début de ce jour",
+    ingress: "entre en {{sign}} ce jour-là",
+    station: { Retrograde: "devient rétrograde ce jour-là", Direct: "redevient direct ce jour-là" },
+    legend: "{{ingress}} signe : le graha entre dans ce signe au cours de la journée. {{station}}R : devient rétrograde ce jour-là ; {{station}}D : redevient direct ce jour-là. Les moments exacts sont dans la recherche d’événements.",
+    field: { month: "mois", place: "lieu" },
+    method: "Méthode : {{method}}. Le signe et le degré suivent le mode zodiacal du thème ; le nakshatra vient de la longitude sidérale. Un jour place la date de transit au début de ce jour.",
+  },
   transit: {
     date: "Moment du transit",
     dateLabel: "Date",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index ce49bc2..0892c99 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -70,6 +70,7 @@ export const tr = {
     dasha_strip: "Dasa şeridi",
     transit_table: "Transit tablosu",
     sankranti: "Sankranti",
+    ephemeris: "Efemeris",
     notes: "Notlar",
     birth_info: "Doğum bilgisi",
     panchang: "Panchang",
@@ -1389,6 +1390,24 @@ export const tr = {
     field: { year: "yıl", place: "yer" },
     method: "Yöntem: {{method}}. Girişler sidereal burçlaradır; tithi, vara ve nakshatra giriş anınındır. Anlar seçilen yerin saat dilimindedir; satırdaki an transit tarihini değiştirir.",
   },
+  ephemeris: {
+    form: "Efemeris tablosu ayarları",
+    prev: "Önceki ay",
+    next: "Sonraki ay",
+    thisMonth: "Bu ay",
+    month: "Ay",
+    year: "Yıl",
+    show: "Ayı göster",
+    nakshatra: "Nakshatra",
+    caption: "her günün başında (00:00, seçilen yerin saati) dokuz grahanın burcu ve derecesi",
+    day: "Gün",
+    goTo: "{{date}}: transit tarihini bu günün başına götür",
+    ingress: "o gün {{sign}} burcuna girer",
+    station: { Retrograde: "o gün retroya döner", Direct: "o gün düze döner" },
+    legend: "{{ingress}} burç: graha o gün içinde bu burca girer. {{station}}R: o gün retroya döner; {{station}}D: o gün düze döner. Kesin anlar olay aramasındadır.",
+    field: { month: "ay", place: "yer" },
+    method: "Yöntem: {{method}}. Burç ve derece haritanın zodyak kipindedir; nakshatra sidereal boylamdandır. Gün, transit tarihini o günün başına götürür.",
+  },
   transit: {
     date: "Transit anı",
     dateLabel: "Tarih",
````

````diff
diff --git a/app/e2e/ipc.ts b/app/e2e/ipc.ts
index 550d59c..f26fdc9 100644
--- a/app/e2e/ipc.ts
+++ b/app/e2e/ipc.ts
@@ -7,6 +7,7 @@ import type { Page } from "@playwright/test";
 export const FIXTURE = JSON.parse(readFileSync(new URL("../ui/test/fixtures/delhi.json", import.meta.url), "utf8"));
 // Geçiş araçlarının fikstürleri (Plan 26) ayrı dosyalardadır; taklit onları `fixture.transit_tools` altında görür.
 FIXTURE.transit_tools = { sankranti: JSON.parse(readFileSync(new URL("../ui/test/fixtures/sankranti.json", import.meta.url), "utf8")) };
+FIXTURE.transit_tools.ephemeris = JSON.parse(readFileSync(new URL("../ui/test/fixtures/ephemeris.json", import.meta.url), "utf8"));
 
 export interface IpcOptions {
   /** `restore_session` dönüşü: `true` Delhi haritası, `false` oturum yok. */
@@ -120,6 +121,8 @@ export async function mockIpc(page: Page, { session, unknownTime = false, partne
         // Geçiş araçları (Plan 26): fikstürdeki gerçek sonuçlar. Liste yıla göre yeniden hesaplanmaz:
         // istenen yıl yanıtın `year` alanına yazılır.
         sankranti_list: (args) => ({ ...fixture.transit_tools.sankranti.list, year: args.year ?? fixture.transit_tools.sankranti.list.year }),
+        // Tablo aya göre yeniden hesaplanmaz: istenen ay yanıtın `month` alanına yazılır.
+        ephemeris_month: (args) => ({ ...fixture.transit_tools.ephemeris.month, month: args.month ?? fixture.transit_tools.ephemeris.month.month }),
         // Bakış tablosu yeniden hesaplanmaz: fikstürdeki gerçek sonuç döner.
         compute_drishti: () => fixture.drishti,
         // Bhava tablosu ve Sudarshana chakra (Plan 25): fikstürdeki gerçek sonuçlar. Tablo
````

- [ ] **Step 4: Testler, akış ve kapılar**

```bash
pnpm --dir app exec vitest run ui/panels/EphemerisPanel.test.tsx 2>&1 | grep -E "Tests |FAIL"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
git diff -U0 -- 'app/ui/**/*.css' | grep '^+' | grep -E '#[0-9a-fA-F]{3,8}\b|rgb\('
git status --short app/ui/panels/ephemeris.css
grep -cE '#[0-9a-fA-F]{3,8}\b|rgb\(' app/ui/panels/ephemeris.css
lsof -i :14326
pnpm --dir app exec playwright test --project=e2e -c playwright.port.config.ts 2>&1 | tail -1
rm -rf app/test-results
```
Expected: `10 passed`; Vitest `102 passed (102)`, `1006 passed (1006)`; `✓ built`; renk taraması çıktısız; `?? app/ui/panels/ephemeris.css`; `0`; `lsof` çıktısız; e2e `35 passed`.

- [ ] **Step 5: Commit**

```bash
git add app/ui/test/ephemerisFixture.ts app/ui/modules/useEphemeris.ts app/ui/panels/ephemeris.css app/ui/panels/EphemerisPanel.tsx app/ui/panels/EphemerisPanel.test.tsx app/ui/layout/model.ts app/ui/layout/model.test.ts app/ui/panels/registry.ts app/ui/panels/panels.test.tsx app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts app/e2e/ipc.ts app/e2e/ephemeris.desk.spec.ts
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): ephemeris panel with a month of daily positions, marks in words and month steps from Rust" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: on dört dosya; yasaklı ad taraması çıktısız; `git status --short` boş.


---

### Task 6: Geçiş adımı: Rust'ta adım aritmetiği ve adım düğmeleri

Yol haritası §4.10 ve görev taslağının (6). maddesi; D22–D24. Komut saf hesaptır (motora gitmez). `transit/store.tsx` ve `TransitDate.tsx` değişmez: varılan an depoya var olan `setAt({ Jd })` ile yazılır.

**Files:**
- Create: `app/src-tauri/src/transit_tools_commands/step.rs`, `app/ui/transit/useTransitStepper.ts`, `app/ui/transit/TransitStepper.tsx`, `app/ui/transit/TransitStepper.test.tsx`, `app/ui/transit/stepper.css`
- Modify: `app/src-tauri/src/transit_tools_commands/mod.rs`, `app/src-tauri/src/lib.rs`, `app/ui/bindings.ts` (üretilir), `app/ui/api.ts`, `app/ui/api.test.ts`, `app/ui/transit/TransitTablePanel.tsx`, `app/ui/i18n/tr.ts`, `en.ts`, `fr.ts`, `app/e2e/ipc.ts`, `tests/golden/modules/transit_tools/REVIEW.md`
- Test: `step.rs` `mod tests` (5); `TransitStepper.test.tsx` (11)

**Interfaces:**
- Consumes: `transit_commands::{transit_moment, TransitAt}` (kasa içi), `atlas::{offset_at, is_known_time_zone, Date, Timestamp, TzSpec}`, `jyotish_core::JD_UT_RANGE`; arayüzde `useTransit`, `browserTimeZone`, `PanelError`.
- Produces:
  - Rust: `step::{StepUnit { Hour, Day, Week, Month, Year }, StepUnit::ALL, MAX_STEP_COUNT = 12, TransitStepRequest { jd_ut, unit, count, tz }, TransitStepView { jd_ut, utc }, transit_step_body(&request), transit_step}`. Hata alanları: `jd_ut`, `count`, `tz`, `range`.
  - Arayüz: `api.transitStep(request)`; `useTransitStepper(): TransitStepper { unit, setUnit, ready, busy, error, step(count), retry, transit }`; `stepBase(transit)`; `STEP_UNITS`, `DEFAULT_STEP_UNIT`; `TransitStepper` bileşeni (Transit tablosu panelinde, tarih kutularının altında); i18n `transitStep.*`; e2e taklidi `transit_step`.

- [ ] **Step 1: Kayıt ve Rust testleri: saat, gün ve hafta, ay ve yıl, boşluk ve örtüşme, girdi denetimi**

Beklenen anlar takvimden elle türetilmiştir (Berlin'de yaz saati 2026'da 29 Mart'ta başlar, 25 Ekim'de biter); türetme test yorumlarındadır.

````diff
diff --git a/app/src-tauri/src/lib.rs b/app/src-tauri/src/lib.rs
index 68dfeea..6746ce0 100644
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -38,6 +38,7 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             transit_commands::transits_at,
             transit_tools_commands::sankranti::sankranti_list,
             transit_tools_commands::ephemeris::ephemeris_month,
+            transit_tools_commands::step::transit_step,
             bhava_commands::compute_bhava_table,
             bhava_commands::compute_sudarshana,
             commands::dasha_children,
diff --git a/app/src-tauri/src/transit_tools_commands/mod.rs b/app/src-tauri/src/transit_tools_commands/mod.rs
index f5566c8..ee2e195 100644
--- a/app/src-tauri/src/transit_tools_commands/mod.rs
+++ b/app/src-tauri/src/transit_tools_commands/mod.rs
@@ -1,4 +1,5 @@
-//! Geçiş araçlarının komutları (yol haritası Plan 26): Sankranti listesi ve efemeris tablosu.
+//! Geçiş araçlarının komutları (yol haritası Plan 26): Sankranti listesi, efemeris tablosu ve
+//! geçiş anının adımı.
 //! Kalıp muhurta komutlarıyla aynıdır: hesap motor iş parçacığında, komut gövdesi `blocking`
 //! havuzunda, hata `CommandError`; sonuç saklanmaz (K1). "Bu yıl", "bu ay", önceki ve sonraki
 //! ay Rust'ta okunur; arayüz tarih hesabı yapmaz (C4).
@@ -8,6 +9,7 @@
 
 pub mod ephemeris;
 pub mod sankranti;
+pub mod step;
 
 use crate::commands::CommandError;
 
````

**Yeni dosya:** `app/src-tauri/src/transit_tools_commands/step.rs` (şimdilik yalnızca test bloğu)

````rust
#[cfg(test)]
mod tests {
    use super::*;

    /// RFC 3339 UTC yazımından Jülyen günü.
    fn jd(utc: &str) -> f64 {
        let t: Timestamp = utc.parse().expect("timestamp");
        UNIX_EPOCH_JD + t.as_second() as f64 / 86_400.0
    }

    fn step(utc: &str, unit: StepUnit, count: i32, tz: &str) -> Result<String, CommandError> {
        let request = TransitStepRequest {
            jd_ut: jd(utc),
            unit,
            count,
            tz: tz.to_owned(),
        };
        let view = transit_step_body(&request)?;
        // İki alan aynı andır.
        assert!((view.jd_ut - jd(&view.utc)).abs() < 1e-9, "{}", view.utc);
        Ok(view.utc)
    }

    fn ok(utc: &str, unit: StepUnit, count: i32, tz: &str) -> String {
        step(utc, unit, count, tz).expect("step")
    }

    fn field(utc: &str, unit: StepUnit, count: i32, tz: &str) -> String {
        let e = step(utc, unit, count, tz).unwrap_err();
        assert_eq!(e.code, "invalid_transit_input");
        // Hata yalnızca alan adını taşır.
        assert_eq!(e.params.len(), 1);
        e.params["field"].clone()
    }

    const BERLIN: &str = "Europe/Berlin";
    const ISTANBUL: &str = "Europe/Istanbul";

    #[test]
    fn an_hour_step_is_exactly_3600_seconds_in_any_zone() {
        assert_eq!(
            ok("2026-11-25T10:00:00Z", StepUnit::Hour, 1, ISTANBUL),
            "2026-11-25T11:00:00Z"
        );
        assert_eq!(
            ok("2026-11-25T10:00:00Z", StepUnit::Hour, -3, BERLIN),
            "2026-11-25T07:00:00Z"
        );
        // Yaz saati geçişinden geçerken de mutlaktır: Berlin 2026-03-29 01:30 CET (00:30 UT)
        // + 1 saat = 01:30 UT (duvar saati 03:30 CEST).
        assert_eq!(
            ok("2026-03-29T00:30:00Z", StepUnit::Hour, 1, BERLIN),
            "2026-03-29T01:30:00Z"
        );
    }

    #[test]
    fn day_and_week_steps_keep_the_wall_clock() {
        // İstanbul 2016'dan beri hep +03:00: bir gün tam 24 saattir. 13:45:30 → 13:45:30.
        assert_eq!(
            ok("2026-11-25T10:45:30Z", StepUnit::Day, 1, ISTANBUL),
            "2026-11-26T10:45:30Z"
        );
        assert_eq!(
            ok("2026-11-25T10:45:30Z", StepUnit::Day, -2, ISTANBUL),
            "2026-11-23T10:45:30Z"
        );
        // Berlin'de yaz saati 29 Mart 2026 Pazar günü başlar (Mart'ın son Pazar'ı).
        // 28 Mart 12:00 CET = 11:00 UT; bir gün sonrası 29 Mart 12:00 CEST = 10:00 UT: 23 saat.
        assert_eq!(
            ok("2026-03-28T11:00:00Z", StepUnit::Day, 1, BERLIN),
            "2026-03-29T10:00:00Z"
        );
        // Geri: 29 Mart 12:00 CEST → 28 Mart 12:00 CET.
        assert_eq!(
            ok("2026-03-29T10:00:00Z", StepUnit::Day, -1, BERLIN),
            "2026-03-28T11:00:00Z"
        );
        // Yaz saati 25 Ekim 2026 Pazar günü biter. 22 Ekim 12:00 CEST = 10:00 UT; bir hafta
        // sonrası 29 Ekim 12:00 CET = 11:00 UT: yedi gün ve bir saat.
        assert_eq!(
            ok("2026-10-22T10:00:00Z", StepUnit::Week, 1, BERLIN),
            "2026-10-29T11:00:00Z"
        );
        assert_eq!(
            ok("2026-11-25T10:00:00Z", StepUnit::Week, -2, "UTC"),
            "2026-11-11T10:00:00Z"
        );
    }

    #[test]
    fn month_and_year_steps_clamp_to_the_last_day_of_the_month() {
        // UTC diliminde duvar saati UT'dir: türetme yalnızca takvimdir.
        // 31 Ocak + 1 ay: Şubat 2026 28 gündür (2026 artık yıl değildir).
        assert_eq!(
            ok("2026-01-31T12:00:00Z", StepUnit::Month, 1, "UTC"),
            "2026-02-28T12:00:00Z"
        );
        // 2024 artık yıldır: 29 Şubat.
        assert_eq!(
            ok("2024-01-31T12:00:00Z", StepUnit::Month, 1, "UTC"),
            "2024-02-29T12:00:00Z"
        );
        // 31 Mart − 1 ay = 28 Şubat; 31 Mayıs + 1 ay = 30 Haziran.
        assert_eq!(
            ok("2026-03-31T12:00:00Z", StepUnit::Month, -1, "UTC"),
            "2026-02-28T12:00:00Z"
        );
        assert_eq!(
            ok("2026-05-31T12:00:00Z", StepUnit::Month, 1, "UTC"),
            "2026-06-30T12:00:00Z"
        );
        // Yıl sınırı: Aralık + 1 ay = Ocak; Ocak − 1 ay = Aralık.
        assert_eq!(
            ok("2026-12-15T12:00:00Z", StepUnit::Month, 1, "UTC"),
            "2027-01-15T12:00:00Z"
        );
        assert_eq!(
            ok("2026-01-15T12:00:00Z", StepUnit::Month, -1, "UTC"),
            "2025-12-15T12:00:00Z"
        );
        // On iki ay bir yıldır.
        assert_eq!(
            ok("2026-11-25T10:00:00Z", StepUnit::Month, 12, "UTC"),
            ok("2026-11-25T10:00:00Z", StepUnit::Year, 1, "UTC")
        );
        // 29 Şubat ± 1 yıl: 28 Şubat.
        assert_eq!(
            ok("2024-02-29T12:00:00Z", StepUnit::Year, 1, "UTC"),
            "2025-02-28T12:00:00Z"
        );
        assert_eq!(
            ok("2024-02-29T12:00:00Z", StepUnit::Year, -1, "UTC"),
            "2023-02-28T12:00:00Z"
        );
        // Ay adımı geri alınamayabilir: 31 Ocak → 28 Şubat → 28 Ocak.
        assert_eq!(
            ok("2026-02-28T12:00:00Z", StepUnit::Month, -1, "UTC"),
            "2026-01-28T12:00:00Z"
        );
        // Duvar saati dilimde korunur: Berlin 15 Ocak 12:00 CET (11:00 UT) + 6 ay = 15 Temmuz
        // 12:00 CEST (10:00 UT).
        assert_eq!(
            ok("2026-01-15T11:00:00Z", StepUnit::Month, 6, BERLIN),
            "2026-07-15T10:00:00Z"
        );
    }

    #[test]
    fn a_wall_clock_in_a_gap_moves_forward_and_one_in_an_overlap_takes_the_earlier_moment() {
        // Boşluk: Berlin'de 29 Mart 2026 02:00–03:00 yoktur. 28 Mart 02:30 CET (01:30 UT) + 1 gün
        // = 29 Mart 02:30 (yok) → boşluk kadar ileri, 03:30 CEST = 01:30 UT.
        assert_eq!(
            ok("2026-03-28T01:30:00Z", StepUnit::Day, 1, BERLIN),
            "2026-03-29T01:30:00Z"
        );
        // Ay adımıyla boşluğa düşmek: 28 Şubat 02:30 CET (01:30 UT) + 1 ay = 28 Mart 02:30 CET
        // (boşluk 29'undadır: 28 Mart olağandır) → 01:30 UT.
        assert_eq!(
            ok("2026-02-28T01:30:00Z", StepUnit::Month, 1, BERLIN),
            "2026-03-28T01:30:00Z"
        );
        // 1 Mart 02:30 CET + 28 gün (dört hafta) = 29 Mart 02:30 (yok) → 03:30 CEST = 01:30 UT.
        assert_eq!(
            ok("2026-03-01T01:30:00Z", StepUnit::Week, 4, BERLIN),
            "2026-03-29T01:30:00Z"
        );
        // Örtüşme: Berlin'de 25 Ekim 2026 02:00–03:00 iki kez yaşanır. 24 Ekim 02:30 CEST
        // (00:30 UT) + 1 gün = 25 Ekim 02:30 → önceki an (CEST), 00:30 UT.
        assert_eq!(
            ok("2026-10-24T00:30:00Z", StepUnit::Day, 1, BERLIN),
            "2026-10-25T00:30:00Z"
        );
    }

    #[test]
    fn bad_input_names_the_field_and_carries_nothing_else() {
        let at = "2026-11-25T10:00:00Z";
        assert_eq!(field(at, StepUnit::Day, 0, "UTC"), "count");
        assert_eq!(field(at, StepUnit::Day, MAX_STEP_COUNT + 1, "UTC"), "count");
        assert_eq!(
            field(at, StepUnit::Hour, -MAX_STEP_COUNT - 1, "UTC"),
            "count"
        );
        assert_eq!(field(at, StepUnit::Day, i32::MIN, "UTC"), "count");
        assert_eq!(field(at, StepUnit::Day, 1, "Mars/Olympus"), "tz");
        assert_eq!(field(at, StepUnit::Day, 1, "  "), "tz");
        assert_eq!(field(at, StepUnit::Hour, 1, ""), "tz");
        for bad in [f64::NAN, f64::INFINITY, 0.0, 1e300] {
            let request = TransitStepRequest {
                jd_ut: bad,
                unit: StepUnit::Day,
                count: 1,
                tz: "UTC".into(),
            };
            let e = transit_step_body(&request).unwrap_err();
            assert_eq!(
                (e.code.as_str(), e.params["field"].as_str()),
                ("invalid_transit_input", "jd_ut")
            );
        }
        // Yıl aralığının uçları (1–2999).
        assert_eq!(
            field("2999-06-15T12:00:00Z", StepUnit::Year, 1, "UTC"),
            "range"
        );
        assert_eq!(
            field("2999-12-15T12:00:00Z", StepUnit::Month, 1, "UTC"),
            "range"
        );
        assert_eq!(
            field("2999-12-31T12:00:00Z", StepUnit::Day, 1, "UTC"),
            "range"
        );
        assert_eq!(
            field("0001-01-01T12:00:00Z", StepUnit::Week, -1, "UTC"),
            "range"
        );
        assert_eq!(
            ok("2998-06-15T12:00:00Z", StepUnit::Year, 1, "UTC"),
            "2999-06-15T12:00:00Z"
        );
        // En büyük adım sayısı geçerlidir.
        assert_eq!(
            ok(at, StepUnit::Day, MAX_STEP_COUNT, "UTC"),
            "2026-12-07T10:00:00Z"
        );
        assert_eq!(StepUnit::ALL.len(), 5);
    }
}
````

- [ ] **Step 2: Testlerin derlenmediğini gör**

```bash
cargo test -p jyotish-desk --lib transit_tools 2>&1 | grep -E "^error" | head -3
```
Expected: `error[E0433]` ya da `error[E0425]` (`step` modülünün adları yok).

- [ ] **Step 3: Adım aritmetiği, bağlamalar ve sarmalayıcı**

Adım sayısı `abs` ile denetlenmez: `i32::MIN` taşar (bu plan yazılırken `bad_input_names_the_field…` testi bu taşmayı buldu; aşağıdaki kod düzeltilmiş biçimdir).

**`app/src-tauri/src/transit_tools_commands/step.rs`: uygulama, test bloğunun üstüne (arada bir boş satır)**

````rust
//! Geçiş anının adımı (yol haritası §4.10): bir andan bir birim ileri ya da geri. Adım aritmetiği
//! burada çözülür; arayüz tarih hesabı yapmaz (C4).
//!
//! Saat adımı mutlaktır (tam 3 600 saniye). Gün, hafta, ay ve yıl adımı duvar saatiyledir: an,
//! verilen IANA dilimindeki takvim tarihine ve saatine çevrilir, takvimde yürünür ve aynı duvar
//! saati yeniden çözülür. Yaz saati geçişinden geçen bir gün adımı bu yüzden 23 ya da 25 saat
//! sürer. Varılan ayda o gün yoksa ayın son günü alınır (31 Ocak + 1 ay = 28 ya da 29 Şubat).
//! Varılan duvar saati yaz saati boşluğuna düşerse boşluk kadar ileri alınır (v1'in önerdiği
//! saat); örtüşmeye düşerse önceki an alınır (transit anının kuralı, Plan 11 D7).
//!
//! Komut saf hesaptır: motora gitmez. Hata yalnızca alan adını taşır; an ve dilim hataya girmez.

use atlas::{Timestamp, TzSpec};
use jyotish_core::JD_UT_RANGE;
use serde::{Deserialize, Serialize};
use specta::Type;

use super::{invalid, TOOLS_YEAR_RANGE};
use crate::commands::CommandError;
use crate::transit_commands::{transit_moment, TransitAt};

const UNIX_EPOCH_JD: f64 = 2_440_587.5;
const DAY_SECS: i64 = 86_400;
/// Bir çağrının en büyük adım sayısı (ileri ya da geri).
pub const MAX_STEP_COUNT: i32 = 12;

/// Adım birimi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
pub enum StepUnit {
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl StepUnit {
    pub const ALL: [StepUnit; 5] = [
        StepUnit::Hour,
        StepUnit::Day,
        StepUnit::Week,
        StepUnit::Month,
        StepUnit::Year,
    ];
}

/// Adımın girdisi. `jd_ut` gösterilen transit anıdır (`TransitView.snapshot.jd_ut`); `count`
/// adım sayısıdır (eksi: geri; sıfır olamaz); `tz` duvar saatinin okunduğu IANA dilimidir
/// (tarayıcının dilimi; yol ya da adres değildir).
#[derive(Debug, Clone, PartialEq, Deserialize, Type)]
pub struct TransitStepRequest {
    pub jd_ut: f64,
    pub unit: StepUnit,
    pub count: i32,
    pub tz: String,
}

/// Varılan an; arayüz onu `TransitAt::Jd` olarak geri verir.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct TransitStepView {
    pub jd_ut: f64,
    /// Anın UTC yazımı (RFC 3339).
    pub utc: String,
}

/// Takvim tarihine `months` ay ekler; varılan ayda o gün yoksa ayın son günü alınır.
fn add_months(date: atlas::Date, months: i32) -> Result<atlas::Date, CommandError> {
    let total = i32::from(date.year()) * 12 + i32::from(date.month()) - 1 + months;
    let (year, month) = (total.div_euclid(12), total.rem_euclid(12) + 1);
    if !TOOLS_YEAR_RANGE.contains(&year) {
        return Err(invalid("range"));
    }
    let first = atlas::Date::new(
        i16::try_from(year).map_err(|_| invalid("range"))?,
        i8::try_from(month).map_err(|_| invalid("range"))?,
        1,
    )
    .map_err(|_| invalid("range"))?;
    let day = date.day().min(first.days_in_month());
    atlas::Date::new(first.year(), first.month(), day).map_err(|_| invalid("range"))
}

/// Duvar saatini çözer; boşluğa düşen saat, v1'in önerdiği saate (boşluk kadar ileri) alınır.
fn resolve(date: &str, time: &str, tz: &str) -> Result<(f64, String), CommandError> {
    let local = |date: &str, time: &str| TransitAt::Local {
        date: date.to_owned(),
        time: time.to_owned(),
        tz: tz.to_owned(),
    };
    // `now` yalnızca `TransitAt::Now` için okunur; burada kullanılmaz.
    let now = Timestamp::UNIX_EPOCH;
    match transit_moment(&local(date, time), now) {
        Ok(moment) => Ok(moment),
        Err(e) if e.code == "nonexistent_local_time" => {
            let suggestion = e.params.get("suggestion").ok_or_else(|| invalid("range"))?;
            let (date, time) = suggestion.split_once('T').ok_or_else(|| invalid("range"))?;
            transit_moment(&local(date, time), now).map_err(|_| invalid("range"))
        }
        Err(e) if e.code == "unknown_time_zone" => Err(invalid("tz")),
        Err(_) => Err(invalid("range")),
    }
}

/// `transit_step` gövdesi.
pub fn transit_step_body(request: &TransitStepRequest) -> Result<TransitStepView, CommandError> {
    let TransitStepRequest {
        jd_ut,
        unit,
        count,
        tz,
    } = request;
    // NaN ve sonsuz da aralık dışındadır.
    if !JD_UT_RANGE.contains(jd_ut) {
        return Err(invalid("jd_ut"));
    }
    // `abs` kullanılmaz: `i32::MIN` taşar.
    if *count == 0 || !(-MAX_STEP_COUNT..=MAX_STEP_COUNT).contains(count) {
        return Err(invalid("count"));
    }
    let tz = tz.trim();
    if tz.is_empty() || !atlas::is_known_time_zone(tz) {
        return Err(invalid("tz"));
    }
    let secs = ((jd_ut - UNIX_EPOCH_JD) * 86_400.0).round() as i64;
    let (jd_ut, utc) = if *unit == StepUnit::Hour {
        let at = Timestamp::from_second(secs + i64::from(*count) * 3_600)
            .map_err(|_| invalid("range"))?;
        (
            UNIX_EPOCH_JD + at.as_second() as f64 / 86_400.0,
            at.to_string(),
        )
    } else {
        let at = Timestamp::from_second(secs).map_err(|_| invalid("range"))?;
        // Yer boylamı 0: transit anının duvar saati de böyle çözülür (`transit_moment`).
        let offset =
            atlas::offset_at(&TzSpec::Iana(tz.to_owned()), 0.0, at).map_err(|_| invalid("tz"))?;
        // Duvar saati, UTC'ymiş gibi yazılan bir an olarak: takvim ve saat buradan okunur.
        let wall = secs + i64::from(offset);
        let days = match unit {
            StepUnit::Day => i64::from(*count),
            StepUnit::Week => 7 * i64::from(*count),
            _ => 0,
        };
        let stepped =
            Timestamp::from_second(wall + days * DAY_SECS).map_err(|_| invalid("range"))?;
        let time = stepped.strftime("%H:%M:%S").to_string();
        let mut date: atlas::Date = stepped
            .strftime("%Y-%m-%d")
            .to_string()
            .parse()
            .map_err(|_| invalid("range"))?;
        date = match unit {
            StepUnit::Month => add_months(date, *count)?,
            StepUnit::Year => add_months(date, 12 * *count)?,
            _ => date,
        };
        if !TOOLS_YEAR_RANGE.contains(&i32::from(date.year())) {
            return Err(invalid("range"));
        }
        resolve(&date.to_string(), &time, tz)?
    };
    if !JD_UT_RANGE.contains(&jd_ut) {
        return Err(invalid("range"));
    }
    Ok(TransitStepView { jd_ut, utc })
}

/// Geçiş anının adımı: `request.jd_ut` anından `count` birim sonraki (eksi: önceki) an. Saat adımı
/// mutlaktır; gün, hafta, ay ve yıl adımı `request.tz` dilimindeki duvar saatini korur. Saf
/// hesaptır, motora gitmez. An ve dilim loglanmaz.
#[tauri::command]
#[specta::specta]
pub fn transit_step(request: TransitStepRequest) -> Result<TransitStepView, CommandError> {
    transit_step_body(&request)
}
````

```bash
cargo fmt --all --check
cargo test -p jyotish-desk --lib transit_tools 2>&1 | grep -E "^test result"
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
grep -c "transitStep:" app/ui/bindings.ts
```
Expected: fmt çıktısız; `6 passed`; `1 passed`; `1`.

````diff
diff --git a/app/ui/api.test.ts b/app/ui/api.test.ts
index 3665ede..bb93eb1 100644
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -68,6 +68,7 @@ vi.mock("./bindings", () => ({
     transitsAt: vi.fn(),
     sankrantiList: vi.fn(),
     ephemerisMonth: vi.fn(),
+    transitStep: vi.fn(),
     keepStoredMoment: vi.fn(),
     appInfo: vi.fn(),
     checkForUpdate: vi.fn(),
@@ -180,6 +181,7 @@ test("every wrapper forwards its arguments to the command of the same name", asy
     ["transitsAt", [{ kind: "now" }, { ayanamsa: "Lahiri" }, [], { birth_jd_ut: 1 }]],
     ["sankrantiList", [2026, { lat: 28.6, lon: 77.2, tz: { Iana: "Asia/Kolkata" } }, { ayanamsa: "Lahiri" }]],
     ["ephemerisMonth", [{ year: 2026, month: 11 }, { lat: 28.6, lon: 77.2, tz: { Iana: "Asia/Kolkata" } }, { ayanamsa: "Lahiri" }]],
+    ["transitStep", [{ jd_ut: 2461369.5, unit: "Day", count: 1, tz: "Europe/Istanbul" }]],
     ["keepStoredMoment", [3]],
     ["appInfo", []],
     ["checkForUpdate", []],
diff --git a/app/ui/api.ts b/app/ui/api.ts
index 9d7d526..f875a4c 100644
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -34,6 +34,7 @@ import {
   type Sign,
   type TagChartsRequest,
   type TransitAt,
+  type TransitStepRequest,
   type ViewState,
 } from "./bindings";
 
@@ -76,6 +77,8 @@ export const api = {
   sankrantiList: (year: number | null, place: MuhurtaPlace, settings: CalcSettings) => wrap(() => commands.sankrantiList(year, place, settings)),
   // `month` boşsa yerin bugünkü ayıdır; önceki ve sonraki ay yanıtla gelir (arayüz tarih hesabı yapmaz).
   ephemerisMonth: (month: MonthId | null, place: MuhurtaPlace, settings: CalcSettings) => wrap(() => commands.ephemerisMonth(month, place, settings)),
+  // Adım aritmetiği Rust'tadır; `request.tz` tarayıcının IANA dilimidir (yol ya da adres değil, C2′).
+  transitStep: (request: TransitStepRequest) => wrap(() => commands.transitStep(request)),
   deleteClient: (clientId: number) => wrap(() => commands.deleteClient(clientId)),
   deleteChart: (chartId: number) => wrap(() => commands.deleteChart(chartId)),
   listClients: (limit: number) => wrap(() => commands.listClients(limit)),
````

Komut dizgi bağımsız değişken almaz (dilim, istek yapısının alanıdır): `api.test.ts`'in dizgi bağımsız değişken listesi değişmez (C2′).

- [ ] **Step 4: Arayüz testleri: adım isteği, çıkış anı, bekleyen transit, süren istek, düşen yanıt, hata ve diller**

**Yeni dosya:** `app/ui/transit/TransitStepper.test.tsx`

````tsx
import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { transitsAt: vi.fn(), transitStep: vi.fn() } }));

import { api, type Result } from "../api";
import type { TransitStepRequest, TransitStepView, TransitView } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import type { Panel } from "../layout/model";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { browserTimeZone, useTransitControl, type Transit } from "./store";
import { TransitStepper } from "./TransitStepper";
import { TransitTablePanel } from "./TransitTablePanel";
import { DEFAULT_STEP_UNIT, STEP_UNITS, stepBase } from "./useTransitStepper";

const NOW = fixture().transit;
const J0 = NOW.snapshot.jd_ut!;
/** Taklit adım: birim ne olursa olsun adım başına bir gün (aritmetik Rust'tadır). */
const stepped = (r: TransitStepRequest): Result<TransitStepView> => ({ status: "ok", data: { jd_ut: r.jd_ut! + r.count, utc: "" } });
/** Taklit transit: `{ Jd }` anının yanıtı o anı taşır. */
const viewAt = (at: unknown): Result<TransitView> => {
  const view = fixture().transit;
  if (typeof at === "object" && at !== null && "Jd" in at) view.snapshot.jd_ut = (at as { Jd: number }).Jd;
  return { status: "ok", data: view };
};

let transit: Transit;
function Probe() {
  transit = useTransitControl();
  return <TransitStepper />;
}
const group = () => screen.getByRole("group", { name: "Transit anını adımla" });
const back = () => within(group()).getByRole("button", { name: "Bir adım geri" });
const forward = () => within(group()).getByRole("button", { name: "Bir adım ileri" });
const settle = () => act(async () => {});
const requests = () => vi.mocked(api.transitStep).mock.calls.map((c) => c[0]);

beforeEach(() => {
  vi.mocked(api.transitsAt).mockReset();
  vi.mocked(api.transitsAt).mockImplementation(async (at) => viewAt(at));
  vi.mocked(api.transitStep).mockReset();
  vi.mocked(api.transitStep).mockImplementation(async (r) => stepped(r));
});

test("a step asks Rust with the shown moment, the unit and the browser's zone, then moves the transit moment", async () => {
  renderWithStores(<Probe />);
  await waitFor(() => expect(forward()).not.toHaveAttribute("aria-disabled"));
  expect(screen.getByRole("combobox", { name: "Adım birimi" })).toHaveDisplayValue("Gün");
  expect(within(screen.getByRole("combobox", { name: "Adım birimi" })).getAllByRole("option").map((o) => o.textContent)).toEqual(["Saat", "Gün", "Hafta", "Ay", "Yıl"]);
  fireEvent.click(forward());
  await settle();
  // An "şimdi"dir: çıkış, gösterilen yanıtın anıdır.
  expect(requests()).toEqual([{ jd_ut: J0, unit: "Day", count: 1, tz: browserTimeZone() }]);
  expect(transit.at).toEqual({ Jd: J0 + 1 });
  expect(vi.mocked(api.transitsAt).mock.calls.at(-1)![0]).toEqual({ Jd: J0 + 1 });
});

test("back steps by minus one and the unit select chooses the unit Rust steps by", async () => {
  renderWithStores(<Probe />);
  await waitFor(() => expect(back()).not.toHaveAttribute("aria-disabled"));
  fireEvent.change(screen.getByRole("combobox", { name: "Adım birimi" }), { target: { value: "Month" } });
  fireEvent.click(back());
  await settle();
  expect(requests()).toEqual([{ jd_ut: J0, unit: "Month", count: -1, tz: browserTimeZone() }]);
  expect(transit.at).toEqual({ Jd: J0 - 1 });
});

test("the next step starts from the stepped moment even before its transits arrive", async () => {
  renderWithStores(<Probe />);
  await waitFor(() => expect(forward()).not.toHaveAttribute("aria-disabled"));
  // Bundan sonraki transit yanıtları gelmez: çıkış anı deponun `{ Jd }` anından okunur.
  vi.mocked(api.transitsAt).mockImplementation(() => new Promise(() => {}));
  fireEvent.click(forward());
  await settle();
  fireEvent.click(forward());
  await settle();
  fireEvent.click(back());
  await settle();
  expect(requests().map((r) => [r.jd_ut, r.count])).toEqual([
    [J0, 1],
    [J0 + 1, 1],
    [J0 + 2, -1],
  ]);
  expect(transit.at).toEqual({ Jd: J0 + 1 });
  expect(forward()).not.toHaveAttribute("aria-disabled");
});

test("before the first transits arrive the buttons stay focusable and do nothing", async () => {
  vi.mocked(api.transitsAt).mockImplementation(() => new Promise(() => {}));
  renderWithStores(<Probe />);
  await settle();
  for (const button of [back(), forward()]) {
    expect(button).toHaveAttribute("aria-disabled", "true");
    expect(button).not.toBeDisabled();
    fireEvent.click(button);
  }
  await settle();
  expect(api.transitStep).not.toHaveBeenCalled();
});

test("a click while a step is being asked is ignored", async () => {
  renderWithStores(<Probe />);
  await waitFor(() => expect(forward()).not.toHaveAttribute("aria-disabled"));
  let resolve!: (r: Result<TransitStepView>) => void;
  vi.mocked(api.transitStep).mockImplementationOnce(() => new Promise((r) => (resolve = r)));
  fireEvent.click(forward());
  fireEvent.click(forward());
  fireEvent.click(back());
  await settle();
  expect(api.transitStep).toHaveBeenCalledTimes(1);
  await act(async () => resolve({ status: "ok", data: { jd_ut: J0 + 1, utc: "" } }));
  expect(transit.at).toEqual({ Jd: J0 + 1 });
  // İstek bitti: yeni adım atılır.
  fireEvent.click(forward());
  await settle();
  expect(api.transitStep).toHaveBeenCalledTimes(2);
});

test("a reply that arrives after the moment was changed another way is dropped", async () => {
  renderWithStores(<Probe />);
  await waitFor(() => expect(forward()).not.toHaveAttribute("aria-disabled"));
  let resolve!: (r: Result<TransitStepView>) => void;
  vi.mocked(api.transitStep).mockImplementationOnce(() => new Promise((r) => (resolve = r)));
  fireEvent.click(forward());
  await settle();
  // Kullanıcı bu arada başka bir an seçti (dasa şeridi, olay satırı).
  act(() => transit.setAt({ Jd: 2_451_545 }));
  await act(async () => resolve({ status: "ok", data: { jd_ut: J0 + 1, utc: "" } }));
  expect(transit.at).toEqual({ Jd: 2_451_545 });
});

test("a reply that arrives after the stepper is gone is dropped without an update", async () => {
  const { unmount } = renderWithStores(<Probe />);
  await waitFor(() => expect(forward()).not.toHaveAttribute("aria-disabled"));
  let resolve!: (r: Result<TransitStepView>) => void;
  vi.mocked(api.transitStep).mockImplementationOnce(() => new Promise((r) => (resolve = r)));
  fireEvent.click(forward());
  await settle();
  const calls = vi.mocked(api.transitsAt).mock.calls.length;
  unmount();
  await act(async () => resolve({ status: "ok", data: { jd_ut: J0 + 1, utc: "" } }));
  expect(vi.mocked(api.transitsAt).mock.calls).toHaveLength(calls);
});

test("an input error names the field in the interface language and the step can be retried", async () => {
  renderWithStores(<Probe />);
  await waitFor(() => expect(forward()).not.toHaveAttribute("aria-disabled"));
  vi.mocked(api.transitStep).mockResolvedValueOnce({ status: "error", error: { code: "invalid_transit_input", params: { field: "range" } } });
  fireEvent.click(back());
  expect(await screen.findByRole("alert")).toHaveTextContent("Transit isteği geçersiz: tarih aralığı (1–2999 yılları).");
  expect(transit.at).toBe("Now");
  fireEvent.click(screen.getByRole("button", { name: "Yeniden dene" }));
  await waitFor(() => expect(transit.at).toEqual({ Jd: J0 - 1 }));
  expect(screen.queryByRole("alert")).toBeNull();
  expect(requests().map((r) => r.count)).toEqual([-1, -1]);
});

test("the step base is the chosen Julian day, or the shown moment once it has arrived", () => {
  const view = fixture().transit;
  expect(stepBase({ at: { Jd: 5 }, view: null, loading: true })).toBe(5);
  expect(stepBase({ at: "Now", view, loading: false })).toBe(J0);
  expect(stepBase({ at: "Now", view, loading: true })).toBeNull();
  expect(stepBase({ at: { Local: { date: "2026-12-01", time: "09:30", tz: "UTC" } }, view: null, loading: false })).toBeNull();
  expect([DEFAULT_STEP_UNIT, STEP_UNITS]).toEqual(["Day", ["Hour", "Day", "Week", "Month", "Year"]]);
});

test("the transit table panel carries the stepper under the date boxes and steps the table", async () => {
  const panel: Panel = { id: "tt", kind: "transit_table", x: 0, y: 0, w: 6, h: 8, settings: {} };
  renderWithStores(<TransitTablePanel panel={panel} />);
  await screen.findByRole("table");
  const form = screen.getByRole("form", { name: "Transit anı" });
  expect(form.compareDocumentPosition(group()) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  expect(group().compareDocumentPosition(screen.getByRole("table")) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  fireEvent.click(forward());
  await waitFor(() => expect(vi.mocked(api.transitsAt).mock.calls.at(-1)![0]).toEqual({ Jd: J0 + 1 }));
  expect(screen.getByRole("button", { name: "Şimdi" })).toHaveAttribute("aria-pressed", "false");
  expect(screen.getByRole("table")).toBeInTheDocument();
});

test("the step texts exist in the three languages", () => {
  for (const m of [tr, en, fr]) {
    expect(Object.keys(m.transitStep.units)).toEqual([...STEP_UNITS]);
    expect(Object.keys(m.transitStep.field).sort()).toEqual(["count", "jd_ut", "range", "tz"]);
  }
  expect([tr.transitStep.forward, en.transitStep.forward, fr.transitStep.forward]).toEqual(["Bir adım ileri", "One step forward", "Un pas en avant"]);
  expect(JSON.stringify(fr.transitStep)).not.toContain("'");
});
````

```bash
pnpm --dir app exec vitest run ui/transit/TransitStepper.test.tsx 2>&1 | grep -E "Test Files|Tests "
```
Expected: dosya düşer (`TransitStepper` yok).

- [ ] **Step 5: Kanca, bileşen, panel, metinler, e2e taklidi ve REVIEW kaydı**

**Yeni dosya:** `app/ui/transit/useTransitStepper.ts`

````ts
// Transit anını adım adım yürüten kanca (yol haritası Plan 26, §4.10). Adım aritmetiği Rust'tadır
// (`transit_step`: ay sonu, yaz saati); burada tarih hesabı yoktur (C4). Adımın çıkış anı transit
// deposunun anıdır: an bir Jülyen günüyse o, değilse ("şimdi", duvar saati) gösterilen yanıtın
// anı. Varılan an depoya `{ Jd }` olarak yazılır; halka ve tablo depodan birlikte güncellenir.
//
// Eşzamanlılık: bir adım isteği sürerken yeni adım başlatılmaz. İstek sürerken an başka bir
// yoldan değişirse (Şimdi, tarih kutusu, dasa şeridi), harita değişirse ya da bileşen kalkarsa
// geç gelen yanıt düşürülür. An ve dilim loglanmaz.
import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "../api";
import type { CommandError, StepUnit, TransitAt } from "../bindings";
import { useSession } from "../session/store";
import { browserTimeZone, useTransit, type Transit } from "./store";

/** Rust `StepUnit::ALL`. */
export const STEP_UNITS = ["Hour", "Day", "Week", "Month", "Year"] as const satisfies readonly StepUnit[];
export const DEFAULT_STEP_UNIT: StepUnit = "Day";

export interface TransitStepper {
  unit: StepUnit;
  setUnit(unit: StepUnit): void;
  /** Adımın çıkacağı an biliniyor. */
  ready: boolean;
  /** Bir adım isteği sürüyor. */
  busy: boolean;
  error: CommandError | null;
  /** `count` birim ileri (eksi: geri). An ilerlediyse `true`; düşen ya da başlamayan adım `false`. */
  step(count: number): Promise<boolean>;
  /** Hatayla biten son adımı yeniden dener. */
  retry(): void;
  /** Deponun kendisi: oynatma, yanıtın gelişini buradan izler. */
  transit: Transit;
}

function isJd(at: TransitAt): at is { Jd: number } {
  return typeof at === "object" && "Jd" in at && typeof at.Jd === "number";
}

/** Adımın çıkış anı; yanıt beklenirken ve an bir Jülyen günü değilken `null`. */
export function stepBase(transit: Pick<Transit, "at" | "view" | "loading">): number | null {
  if (isJd(transit.at)) return transit.at.Jd;
  if (transit.loading) return null;
  return transit.view?.snapshot.jd_ut ?? null;
}

export function useTransitStepper(): TransitStepper {
  const transit = useTransit();
  const { opened } = useSession();
  const chartId = opened?.chart_record.id ?? null;
  const [unit, setUnit] = useState<StepUnit>(DEFAULT_STEP_UNIT);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<CommandError | null>(null);
  const base = stepBase(transit);
  // Son çizimin değerleri: süren istek, yanıt geldiğinde güncel anı buradan okur.
  const latest = useRef({ base, unit, setAt: transit.setAt });
  latest.current = { base, unit, setAt: transit.setAt };
  const inFlight = useRef(false);
  const generation = useRef(0);
  const lastCount = useRef(1);

  // Başka harita ya da bileşenin kalkması: süren adımın yanıtı düşer.
  useEffect(() => {
    setError(null);
    setBusy(false);
    inFlight.current = false;
    return () => {
      generation.current += 1;
    };
  }, [chartId]);

  const step = useCallback(async (count: number): Promise<boolean> => {
    const from = latest.current.base;
    if (from === null || inFlight.current) return false;
    const mine = generation.current;
    inFlight.current = true;
    lastCount.current = count;
    setBusy(true);
    const r = await api.transitStep({ jd_ut: from, unit: latest.current.unit, count, tz: browserTimeZone() });
    if (generation.current !== mine) return false;
    inFlight.current = false;
    setBusy(false);
    if (r.status === "error") {
      setError(r.error);
      return false;
    }
    setError(null);
    // İstek sürerken an başka bir yoldan değişti: eski adım uygulanmaz.
    if (latest.current.base !== from || r.data.jd_ut === null) return false;
    latest.current.setAt({ Jd: r.data.jd_ut });
    return true;
  }, []);

  const retry = useCallback(() => {
    setError(null);
    void step(lastCount.current);
  }, [step]);

  return { unit, setUnit, ready: base !== null, busy, error, step, retry, transit };
}
````

**Yeni dosya:** `app/ui/transit/stepper.css`

````css
/* Transit adım ve oynatma denetimleri. Renk token'lardan gelir (C9). */
.transit-stepper {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
}
.transit-stepper-note {
  margin: 0;
  color: var(--text-2);
  font-size: 12px;
}
````

**Yeni dosya:** `app/ui/transit/TransitStepper.tsx`

````tsx
// Transit adım denetimleri (yol haritası Plan 26, §4.10): birim seçici ve bir adım geri, bir adım
// ileri düğmeleri. Adımın hesabı Rust'tadır (`useTransitStepper`). Düğmeler kullanılamazken yerel
// `disabled` değildir: odakta durur, tıklama bir şey yapmaz. Hata yalnızca alan adını taşır; ad
// arayüz dilinde yazılır.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { CommandError, StepUnit } from "../bindings";
import { PanelError } from "../panels/states";
import "./stepper.css";
import { STEP_UNITS, useTransitStepper } from "./useTransitStepper";

/** `invalid_transit_input`'un alan adını arayüz dilinde yazar; öteki hatalar olduğu gibi durur. */
function fieldError(error: CommandError, name: (field: string) => string): CommandError {
  const field = error.params.field;
  if (error.code !== "invalid_transit_input" || typeof field !== "string") return error;
  return { ...error, params: { ...error.params, field: name(field) } };
}

export function TransitStepper() {
  const { t, i18n } = useTranslation();
  const stepper = useTransitStepper();
  const id = useId();
  const blocked = !stepper.ready;
  const shownError = stepper.error === null ? null : fieldError(stepper.error, (f) => (i18n.exists(`transitStep.field.${f}`) ? t(`transitStep.field.${f}`) : f));
  return (
    <>
      <div className="transit-stepper" role="group" aria-label={t("transitStep.group")}>
        <label htmlFor={`${id}-unit`}>{t("transitStep.unit")}</label>
        <select id={`${id}-unit`} value={stepper.unit} onChange={(e) => stepper.setUnit(e.target.value as StepUnit)}>
          {STEP_UNITS.map((u) => (
            <option key={u} value={u}>
              {t(`transitStep.units.${u}`)}
            </option>
          ))}
        </select>
        <button type="button" aria-disabled={blocked ? true : undefined} onClick={() => void stepper.step(-1)}>
          {t("transitStep.back")}
        </button>
        <button type="button" aria-disabled={blocked ? true : undefined} onClick={() => void stepper.step(1)}>
          {t("transitStep.forward")}
        </button>
      </div>
      {shownError && <PanelError error={shownError} onRetry={stepper.retry} />}
    </>
  );
}
````

````diff
diff --git a/app/ui/transit/TransitTablePanel.tsx b/app/ui/transit/TransitTablePanel.tsx
index ed76bf9..18fcdbe 100644
--- a/app/ui/transit/TransitTablePanel.tsx
+++ b/app/ui/transit/TransitTablePanel.tsx
@@ -1,6 +1,7 @@
 // Transit tablosu (UI spec §3.3). Konumlar ve natal evler Rust'tan (`transits_at`); saat
 // bilinmiyorsa natal ev gösterilmez (spec §7). Retro düz "R", düğümlerde yok (spec §2.3).
-// Nakshatra lordu Rust'ın tablosundan okunur (`nakshatraLord`; Plan 26).
+// Nakshatra lordu Rust'ın tablosundan okunur (`nakshatraLord`; Plan 26). Tarih kutularının
+// altındaki adım denetimleri anı birim birim yürütür (`TransitStepper`).
 import { useTranslation } from "react-i18next";
 import { signIndex } from "../chart/adapter";
 import { RETRO_MARK, showsRetro, type GrahaId } from "../chart/types";
@@ -13,6 +14,7 @@ import { timeUnknown } from "../session/birth";
 import { useSession } from "../session/store";
 import { browserTimeZone, formatMoment, useTransit } from "./store";
 import { TransitDate } from "./TransitDate";
+import { TransitStepper } from "./TransitStepper";
 import "./transit.css";
 
 export function TransitTablePanel(_: PanelProps) {
@@ -30,6 +32,7 @@ export function TransitTablePanel(_: PanelProps) {
   return (
     <div className="transit-panel">
       <TransitDate />
+      <TransitStepper />
       {transit.error ? (
         <PanelError error={transit.error} onRetry={transit.retry} />
       ) : (
````

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index 1158bf4..a27b103 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -1409,6 +1409,14 @@ export const en: Messages = {
     field: { month: "month", place: "place" },
     method: "Method: {{method}}. Sign and degree follow the chart's zodiac mode; the nakshatra is from the sidereal longitude. A day moves the transit date to the start of that day.",
   },
+  transitStep: {
+    group: "Step the transit moment",
+    unit: "Step unit",
+    units: { Hour: "Hour", Day: "Day", Week: "Week", Month: "Month", Year: "Year" },
+    back: "One step back",
+    forward: "One step forward",
+    field: { jd_ut: "moment", count: "step count", tz: "time zone", range: "date range (years 1–2999)" },
+  },
   transit: {
     date: "Transit moment",
     dateLabel: "Date",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 2ba95dc..68b5445 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -1409,6 +1409,14 @@ export const fr: Messages = {
     field: { month: "mois", place: "lieu" },
     method: "Méthode : {{method}}. Le signe et le degré suivent le mode zodiacal du thème ; le nakshatra vient de la longitude sidérale. Un jour place la date de transit au début de ce jour.",
   },
+  transitStep: {
+    group: "Avancer le moment du transit pas à pas",
+    unit: "Unité du pas",
+    units: { Hour: "Heure", Day: "Jour", Week: "Semaine", Month: "Mois", Year: "Année" },
+    back: "Un pas en arrière",
+    forward: "Un pas en avant",
+    field: { jd_ut: "moment", count: "nombre de pas", tz: "fuseau horaire", range: "plage de dates (années 1–2999)" },
+  },
   transit: {
     date: "Moment du transit",
     dateLabel: "Date",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 0892c99..31addfd 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -1408,6 +1408,14 @@ export const tr = {
     field: { month: "ay", place: "yer" },
     method: "Yöntem: {{method}}. Burç ve derece haritanın zodyak kipindedir; nakshatra sidereal boylamdandır. Gün, transit tarihini o günün başına götürür.",
   },
+  transitStep: {
+    group: "Transit anını adımla",
+    unit: "Adım birimi",
+    units: { Hour: "Saat", Day: "Gün", Week: "Hafta", Month: "Ay", Year: "Yıl" },
+    back: "Bir adım geri",
+    forward: "Bir adım ileri",
+    field: { jd_ut: "an", count: "adım sayısı", tz: "saat dilimi", range: "tarih aralığı (1–2999 yılları)" },
+  },
   transit: {
     date: "Transit anı",
     dateLabel: "Tarih",
````

````diff
diff --git a/app/e2e/ipc.ts b/app/e2e/ipc.ts
index f26fdc9..7aef4c4 100644
--- a/app/e2e/ipc.ts
+++ b/app/e2e/ipc.ts
@@ -123,6 +123,13 @@ export async function mockIpc(page: Page, { session, unknownTime = false, partne
         sankranti_list: (args) => ({ ...fixture.transit_tools.sankranti.list, year: args.year ?? fixture.transit_tools.sankranti.list.year }),
         // Tablo aya göre yeniden hesaplanmaz: istenen ay yanıtın `month` alanına yazılır.
         ephemeris_month: (args) => ({ ...fixture.transit_tools.ephemeris.month, month: args.month ?? fixture.transit_tools.ephemeris.month.month }),
+        // Adım aritmetiği taklit edilmez (Rust testlerindedir): her birim sabit bir gün sayısıdır.
+        transit_step: (args) => {
+          const request = args.request as { jd_ut: number; unit: "Hour" | "Day" | "Week" | "Month" | "Year"; count: number };
+          const days = { Hour: 1 / 24, Day: 1, Week: 7, Month: 30, Year: 365 }[request.unit];
+          const jd = request.jd_ut + request.count * days;
+          return { jd_ut: jd, utc: new Date((jd - 2440587.5) * 86_400_000).toISOString() };
+        },
         // Bakış tablosu yeniden hesaplanmaz: fikstürdeki gerçek sonuç döner.
         compute_drishti: () => fixture.drishti,
         // Bhava tablosu ve Sudarshana chakra (Plan 25): fikstürdeki gerçek sonuçlar. Tablo
````

````diff
diff --git a/tests/golden/modules/transit_tools/REVIEW.md b/tests/golden/modules/transit_tools/REVIEW.md
index e859c04..fdffadd 100644
--- a/tests/golden/modules/transit_tools/REVIEW.md
+++ b/tests/golden/modules/transit_tools/REVIEW.md
@@ -92,3 +92,32 @@ Vimshottari sırasının üç kez yazılmış biçimidir ve testte o sırayla ka
 - [ ] Satır saatinin seçilen yerin yerel gece yarısı olması (öteki okuma: 00:00 UT).
 - [ ] Varsayılan yerin bulunulan yer olması (öteki okuma: haritanın doğum yeri).
 - [ ] Burç ve derecenin haritanın zodyak kipini izlemesi (tropikal-rasi kipinde tropikal).
+
+## 4. Geçiş anının adımı ve oynatma
+
+### 4.1 Okuma kuralları
+
+1. **Birimler.** Saat, gün, hafta, ay, yıl. Varsayılan birim gündür. Her tıklama bir adımdır.
+2. **Saat adımı mutlaktır:** tam 3 600 saniye; yaz saati geçişinde de böyledir.
+3. **Gün, hafta, ay ve yıl adımı duvar saatini korur.** An, tarayıcının saat dilimindeki takvim
+   tarihine ve saatine çevrilir; takvimde yürünür; aynı duvar saati yeniden çözülür. Yaz saati
+   geçişinden geçen bir gün adımı 23 ya da 25 saat sürer.
+4. **Ay sonu.** Varılan ayda o gün yoksa ayın son günü alınır: 31 Ocak + 1 ay = 28 Şubat (artık
+   yılda 29 Şubat); 29 Şubat ± 1 yıl = 28 Şubat. Bu yüzden ay adımı her zaman geri alınamaz:
+   31 Ocak → 28 Şubat → 28 Ocak.
+5. **Yaz saati boşluğu.** Varılan duvar saati yoksa (ör. 02:30, saatlerin 02:00'den 03:00'e
+   atladığı gün) boşluk kadar ileri alınır: 03:30. Örtüşmede (aynı duvar saati iki kez) önceki an
+   alınır. İkisi de transit anının v1 kuralıdır.
+6. **Aralık.** 1–2999 yılları; dışına çıkan adım `invalid_transit_input { field: "range" }` verir
+   ve an değişmez.
+7. **Çıkış anı.** Transit deposunun anıdır: an bir Jülyen günüyse o, "şimdi" ya da yazılmış bir
+   duvar saatiyse gösterilen yanıtın anı. İlk adımdan sonra an "şimdi" olmaktan çıkar.
+8. **Eşzamanlılık.** Bir adım isteği sürerken yeni adım başlatılmaz. İstek sürerken an başka bir
+   yoldan değişirse, harita değişirse ya da panel kapanırsa geç gelen yanıt düşürülür.
+
+### 4.2 Onay bekleyen noktalar (sahibi)
+
+- [ ] Gün ve hafta adımının duvar saatini koruması (öteki okuma: tam 24 ve 168 saat).
+- [ ] Duvar saatinin tarayıcının diliminde okunması (öteki okuma: haritanın ya da bulunulan
+      yerin dilimi).
+- [ ] Ay sonunda ayın son gününe düşme ve boşlukta ileri alma.
````

- [ ] **Step 6: Testler ve kapılar**

```bash
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -1
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
cargo test -p jyotish-desk --lib 2>&1 | grep -E "^test result"
pnpm --dir app exec vitest run ui/transit 2>&1 | grep -E "Tests |FAIL"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
lsof -i :14326
pnpm --dir app exec playwright test --project=e2e -c playwright.port.config.ts 2>&1 | tail -1
rm -rf app/test-results
```
Expected: clippy `Finished`; sayaç `0`; `64 passed`; `22 passed`; Vitest `103 passed (103)`, `1017 passed (1017)`; `✓ built`; `lsof` çıktısız; e2e `35 passed` (var olan transit akışı değişmedi).

- [ ] **Step 7: Commit**

```bash
git add app/src-tauri/src/lib.rs app/src-tauri/src/transit_tools_commands app/ui/bindings.ts app/ui/api.ts app/ui/api.test.ts app/ui/transit/useTransitStepper.ts app/ui/transit/TransitStepper.tsx app/ui/transit/TransitStepper.test.tsx app/ui/transit/stepper.css app/ui/transit/TransitTablePanel.tsx app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts app/e2e/ipc.ts tests/golden/modules/transit_tools/REVIEW.md
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(transit): step the transit moment by hour, day, week, month or year with the arithmetic in Rust" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: on altı dosya; yasaklı ad taraması çıktısız; `git status --short` boş.


---

### Task 7: Oynat ve duraklat: yerleşen an başına tek adım, düşen yanıtlar, azaltılmış hareket ve uçtan uca akışlar

Yol haritası §4.10 ve görev taslağının (7). maddesi; D25–D27. Oynatma yeni komut getirmez: Task 6'nın adımını zamanlayıcıyla çağırır.

**Files:**
- Create: `app/ui/transit/useTransitPlayback.ts`, `app/ui/transit/TransitPlayback.tsx`, `app/ui/transit/TransitPlayback.test.tsx`, `app/e2e/transit-step.desk.spec.ts`
- Modify: `app/ui/transit/TransitStepper.tsx`, `app/ui/i18n/tr.ts`, `en.ts`, `fr.ts`, `tests/golden/modules/transit_tools/REVIEW.md`
- Test: `TransitPlayback.test.tsx` (11); e2e +2

**Interfaces:**
- Consumes: `TransitStepper` (Task 6), `useSession`, `window.matchMedia("(prefers-reduced-motion: reduce)")`.
- Produces: `useReducedMotion()`; `useTransitPlayback(stepper): TransitPlayback { available, playing, speed, setSpeed, toggle }`; `PLAY_SPEEDS`, `DEFAULT_PLAY_SPEED`, `PLAY_INTERVAL_MS` (2000, 1000, 400); `TransitPlayback` bileşeni; i18n `transitStep.play`, `pause`, `speed`, `speeds.*`, `reducedMotion`.

- [ ] **Step 1: Testler: aralık, bekleyen transit, hız ve birim, iki hata, bileşenin kalkması, başka harita, elle seçilen an, azaltılmış hareket ve diller**

Testler sahte zamanlayıcıyla koşar. Bir anın "yerleşmesi" (transit yanıtının gelmesi) zamanlayıcı ilerletilmeden önce işlenir: `advance(0)`.

**Yeni dosya:** `app/ui/transit/TransitPlayback.test.tsx`

````tsx
import { act, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { transitsAt: vi.fn(), transitStep: vi.fn() } }));

import { api, type Result } from "../api";
import type { TransitStepRequest, TransitStepView, TransitView } from "../bindings";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useSession, type Session } from "../session/store";
import { useTransitControl, type Transit } from "./store";
import { TransitStepper } from "./TransitStepper";
import { DEFAULT_PLAY_SPEED, PLAY_INTERVAL_MS, PLAY_SPEEDS } from "./useTransitPlayback";

const J0 = fixture().transit.snapshot.jd_ut!;
const stepped = (r: TransitStepRequest): Result<TransitStepView> => ({ status: "ok", data: { jd_ut: r.jd_ut! + r.count, utc: "" } });
const viewAt = (at: unknown): Result<TransitView> => {
  const view = fixture().transit;
  if (typeof at === "object" && at !== null && "Jd" in at) view.snapshot.jd_ut = (at as { Jd: number }).Jd;
  return { status: "ok", data: view };
};

let transit: Transit;
let session: Session;
function Probe() {
  transit = useTransitControl();
  session = useSession();
  return <TransitStepper />;
}
const group = () => screen.getByRole("group", { name: "Transit anını adımla" });
const play = () => within(group()).getByRole("button", { name: "Oynat" });
const pause = () => within(group()).getByRole("button", { name: "Duraklat" });
/** Zamanı ilerletir ve bekleyen yanıtları işler. */
const advance = (ms: number) => act(async () => void (await vi.advanceTimersByTimeAsync(ms)));
const steps = () => vi.mocked(api.transitStep).mock.calls.length;
const NORMAL = PLAY_INTERVAL_MS.normal;

/** `matchMedia` taklidi: "hareketi azalt" sorgusunun yanıtı ve değişim bildirimi. */
function mockReducedMotion(matches: boolean) {
  const listeners = new Set<() => void>();
  const query = {
    matches,
    addEventListener: (_: string, f: () => void) => listeners.add(f),
    removeEventListener: (_: string, f: () => void) => listeners.delete(f),
  };
  vi.stubGlobal("matchMedia", (q: string) => (q === "(prefers-reduced-motion: reduce)" ? query : { matches: false, addEventListener: () => {}, removeEventListener: () => {} }));
  return {
    set(next: boolean) {
      query.matches = next;
      for (const f of listeners) f();
    },
  };
}

beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(api.transitsAt).mockReset();
  vi.mocked(api.transitsAt).mockImplementation(async (at) => viewAt(at));
  vi.mocked(api.transitStep).mockReset();
  vi.mocked(api.transitStep).mockImplementation(async (r) => stepped(r));
});
afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

async function renderReady() {
  const utils = renderWithStores(<Probe />);
  await advance(0);
  expect(play()).not.toHaveAttribute("aria-disabled");
  return utils;
}

test("play steps forward once per interval, each step after the previous moment has settled; pause stops", async () => {
  await renderReady();
  expect([DEFAULT_PLAY_SPEED, NORMAL]).toEqual(["normal", 1000]);
  fireEvent.click(play());
  // Düğmenin adı durumu söyler.
  expect(pause()).toBeInTheDocument();
  await advance(NORMAL - 1);
  expect(steps()).toBe(0);
  await advance(1);
  expect(steps()).toBe(1);
  expect(vi.mocked(api.transitStep).mock.calls[0][0]).toMatchObject({ jd_ut: J0, unit: "Day", count: 1 });
  expect(transit.at).toEqual({ Jd: J0 + 1 });
  await advance(NORMAL);
  await advance(NORMAL);
  expect(steps()).toBe(3);
  expect(transit.at).toEqual({ Jd: J0 + 3 });
  fireEvent.click(pause());
  await advance(10 * NORMAL);
  expect(steps()).toBe(3);
  expect(play()).toBeInTheDocument();
});

test("no step is planned while the transits of the last moment are still being read", async () => {
  await renderReady();
  let resolve!: (r: Result<TransitView>) => void;
  fireEvent.click(play());
  // İlk adımdan sonraki transit yanıtı bekletilir: motor meşgul.
  vi.mocked(api.transitsAt).mockImplementationOnce(() => new Promise((r) => (resolve = r)));
  await advance(NORMAL);
  expect(steps()).toBe(1);
  await advance(20 * NORMAL);
  expect(steps()).toBe(1);
  // Yanıt gelince bir aralık sonra sıradaki adım atılır.
  await act(async () => resolve(viewAt({ Jd: J0 + 1 })));
  await advance(NORMAL - 1);
  expect(steps()).toBe(1);
  await advance(1);
  expect(steps()).toBe(2);
});

test("the speed select sets the wait between steps and the unit select the size of a step", async () => {
  await renderReady();
  const speed = within(group()).getByRole("combobox", { name: "Hız" });
  expect(within(speed).getAllByRole("option").map((o) => o.textContent)).toEqual(["Yavaş", "Orta", "Hızlı"]);
  expect([PLAY_SPEEDS, PLAY_INTERVAL_MS]).toEqual([["slow", "normal", "fast"], { slow: 2000, normal: 1000, fast: 400 }]);
  fireEvent.change(speed, { target: { value: "fast" } });
  fireEvent.change(within(group()).getByRole("combobox", { name: "Adım birimi" }), { target: { value: "Week" } });
  fireEvent.click(play());
  await advance(400);
  expect(steps()).toBe(1);
  expect(vi.mocked(api.transitStep).mock.calls[0][0].unit).toBe("Week");
  await advance(400);
  expect(steps()).toBe(2);
  fireEvent.change(speed, { target: { value: "slow" } });
  await advance(1999);
  expect(steps()).toBe(2);
  await advance(1);
  expect(steps()).toBe(3);
});

test("a step error stops playback and is shown", async () => {
  await renderReady();
  fireEvent.click(play());
  await advance(NORMAL);
  vi.mocked(api.transitStep).mockResolvedValueOnce({ status: "error", error: { code: "invalid_transit_input", params: { field: "range" } } });
  await advance(NORMAL);
  expect(steps()).toBe(2);
  expect(screen.getByRole("alert")).toHaveTextContent("Transit isteği geçersiz: tarih aralığı (1–2999 yılları).");
  expect(play()).toBeInTheDocument();
  await advance(10 * NORMAL);
  expect(steps()).toBe(2);
});

test("a transit error stops playback", async () => {
  await renderReady();
  fireEvent.click(play());
  vi.mocked(api.transitsAt).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  await advance(NORMAL);
  expect(steps()).toBe(1);
  expect(play()).toBeInTheDocument();
  await advance(10 * NORMAL);
  expect(steps()).toBe(1);
});

test("the timer stops when the stepper goes away", async () => {
  const { unmount } = await renderReady();
  fireEvent.click(play());
  await advance(NORMAL);
  expect(steps()).toBe(1);
  unmount();
  await advance(20 * NORMAL);
  expect(steps()).toBe(1);
});

test("opening another chart stops playback", async () => {
  await renderReady();
  fireEvent.click(play());
  await advance(NORMAL);
  expect(steps()).toBe(1);
  const other = fixture().opened;
  other.chart_record.id = 77;
  act(() => session.open(other));
  await advance(0);
  expect(play()).toBeInTheDocument();
  // Yeni haritada an "şimdi"ye döner ve oynatma kendiliğinden başlamaz.
  expect(transit.at).toBe("Now");
  await advance(20 * NORMAL);
  expect(steps()).toBe(1);
});

test("a moment chosen by hand during playback is the new starting point", async () => {
  await renderReady();
  fireEvent.click(play());
  await advance(NORMAL);
  expect(transit.at).toEqual({ Jd: J0 + 1 });
  act(() => transit.setAt({ Jd: 2_451_545 }));
  // Önce seçilen anın transitleri gelir (an yerleşir); aralık ondan sonra sayılır.
  await advance(0);
  expect(steps()).toBe(1);
  await advance(NORMAL);
  expect(steps()).toBe(2);
  expect(vi.mocked(api.transitStep).mock.calls.at(-1)![0].jd_ut).toBe(2_451_545);
  expect(transit.at).toEqual({ Jd: 2_451_546 });
});

test("with reduced motion playback is not offered and the step buttons still work", async () => {
  mockReducedMotion(true);
  renderWithStores(<Probe />);
  await advance(0);
  expect(within(group()).queryByRole("button", { name: "Oynat" })).toBeNull();
  expect(within(group()).queryByRole("combobox", { name: "Hız" })).toBeNull();
  expect(within(group()).getByRole("note")).toHaveTextContent("Sistemde “hareketi azalt” açık: oynatma sunulmuyor. Adım düğmeleri çalışır.");
  fireEvent.click(within(group()).getByRole("button", { name: "Bir adım ileri" }));
  await advance(0);
  expect(transit.at).toEqual({ Jd: J0 + 1 });
  await advance(20 * NORMAL);
  expect(steps()).toBe(1);
});

test("turning reduced motion on during playback stops it", async () => {
  const media = mockReducedMotion(false);
  await renderReady();
  fireEvent.click(play());
  await advance(NORMAL);
  expect(steps()).toBe(1);
  act(() => media.set(true));
  expect(within(group()).queryByRole("button", { name: "Duraklat" })).toBeNull();
  await advance(20 * NORMAL);
  expect(steps()).toBe(1);
  // Ayar kapanınca oynatma kendiliğinden başlamaz.
  act(() => media.set(false));
  expect(play()).toBeInTheDocument();
  await advance(20 * NORMAL);
  expect(steps()).toBe(1);
});

test("the playback texts exist in the three languages", () => {
  for (const m of [tr, en, fr]) expect(Object.keys(m.transitStep.speeds)).toEqual([...PLAY_SPEEDS]);
  expect([tr.transitStep.play, en.transitStep.play, fr.transitStep.play]).toEqual(["Oynat", "Play", "Lire"]);
  expect([tr.transitStep.pause, en.transitStep.pause, fr.transitStep.pause]).toEqual(["Duraklat", "Pause", "Mettre en pause"]);
  expect(JSON.stringify(fr.transitStep)).not.toContain("'");
  expect(fr.transitStep.reducedMotion).toContain("n’est pas proposée");
});
````

**Yeni dosya:** `app/e2e/transit-step.desk.spec.ts`

````ts
// Transit adımı ve oynatma akışları (Plan 26). Ayrı dosyadır: `desk.spec.ts` eşzamanlı planlarla
// paylaşılır; "e2e" projesi adı `desk.spec.ts` ile biten her dosyayı koşar.
import { expect, test, type Page } from "@playwright/test";
import { FIXTURE, ipcCalls, mockIpc } from "./ipc";

type StepCall = { request: { jd_ut: number; unit: string; count: number; tz: string } };

// Gösterilen anın yazımı tarayıcının dilimine bağlıdır: dilim sabitlenir.
test.use({ timezoneId: "Europe/Istanbul" });

/**
 * `transits_at` taklidini canlandırır: `{ Jd }` anının yanıtı o anı taşır (konumlar fikstürdeki
 * gibidir; arayüz hesap yapmaz). `mockIpc`'ten sonra çağrılır.
 */
async function liveTransits(page: Page) {
  await page.addInitScript(() => {
    type Invoke = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;
    const internals = (window as unknown as { __TAURI_INTERNALS__: { invoke: Invoke } }).__TAURI_INTERNALS__;
    const inner = internals.invoke;
    internals.invoke = async (cmd, args = {}) => {
      const reply = await inner(cmd, args);
      const at = args.at;
      if (cmd === "transits_at" && typeof at === "object" && at !== null && "Jd" in at) {
        const jd = (at as { Jd: number }).Jd;
        const view = reply as { utc: string; snapshot: { jd_ut: number } };
        view.snapshot.jd_ut = jd;
        view.utc = new Date(Math.round((jd - 2440587.5) * 86_400_000)).toISOString();
      }
      return reply;
    };
  });
}

async function openTransitTable(page: Page) {
  await mockIpc(page, { session: true });
  await liveTransits(page);
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+Digit3");
  const table = page.getByRole("region", { name: "Transit tablosu" });
  await expect(table.getByRole("table")).toBeVisible();
  return table;
}

const steps = async (page: Page) => (await ipcCalls(page, "transit_step")) as StepCall[];
const J0: number = FIXTURE.transit.snapshot.jd_ut;

test("transit steps: the step buttons move the moment by the chosen unit; play advances it until paused", async ({ page }) => {
  const table = await openTransitTable(page);
  const stepper = table.getByRole("group", { name: "Transit anını adımla" });
  const now = table.getByRole("button", { name: "Şimdi" });
  await expect(now).toHaveAttribute("aria-pressed", "true");
  expect(await steps(page)).toHaveLength(0);

  // Bir gün ileri: adım Rust'a gösterilen an, birim ve tarayıcının dilimiyle sorulur; varılan an
  // transit tarihi olur (fikstürün anı 2026-11-25 10:00 UT; İstanbul'da 13:00).
  const forward = stepper.getByRole("button", { name: "Bir adım ileri" });
  await forward.focus();
  await page.keyboard.press("Enter");
  await expect(table.getByRole("status")).toHaveText("Gösterilen an: 26 Kas 2026 13:00");
  expect((await steps(page))[0].request).toEqual({ jd_ut: J0, unit: "Day", count: 1, tz: "Europe/Istanbul" });
  await expect.poll(async () => ((await ipcCalls(page, "transits_at")) as { at: unknown }[]).at(-1)?.at).toEqual({ Jd: J0 + 1 });
  await expect(now).toHaveAttribute("aria-pressed", "false");
  // Odak düğmede durur: art arda adım klavyeyle atılır.
  await expect(forward).toBeFocused();
  await expect(table.getByLabel("Tarih")).toHaveValue("2026-11-26");

  // Bir hafta geri: çıkış anı bir önceki adımın vardığı andır.
  await stepper.getByRole("combobox", { name: "Adım birimi" }).selectOption("Week");
  await stepper.getByRole("button", { name: "Bir adım geri" }).click();
  await expect(table.getByRole("status")).toHaveText("Gösterilen an: 19 Kas 2026 13:00");
  expect((await steps(page))[1].request).toMatchObject({ jd_ut: J0 + 1, unit: "Week", count: -1 });

  // Oynat: an, duraklatılana dek aralıklarla ilerler.
  await stepper.getByRole("combobox", { name: "Adım birimi" }).selectOption("Day");
  await stepper.getByRole("combobox", { name: "Hız" }).selectOption("fast");
  await stepper.getByRole("button", { name: "Oynat" }).click();
  const pause = stepper.getByRole("button", { name: "Duraklat" });
  await expect(pause).toBeVisible();
  await expect.poll(async () => (await steps(page)).length, { timeout: 10_000 }).toBeGreaterThanOrEqual(5);
  await pause.click();
  await expect(stepper.getByRole("button", { name: "Oynat" })).toBeVisible();
  const played = await steps(page);
  // Her oynatma adımı bir öncekinin vardığı andan çıkar: adım atlanmaz, yinelenmez.
  for (let i = 3; i < played.length; i++) expect(played[i].request).toMatchObject({ jd_ut: played[i - 1].request.jd_ut + 1, unit: "Day", count: 1 });
  // Duraklatınca durur.
  await page.waitForTimeout(1_200);
  expect(await steps(page)).toHaveLength(played.length);
  // Tablo oynatma boyunca yerindedir.
  await expect(table.getByRole("table").locator("tbody tr")).toHaveCount(9);
});

test.describe("reduced motion", () => {
  test.use({ reducedMotion: "reduce" });

  test("transit steps: with reduced motion there is no play button, only steps", async ({ page }) => {
    const table = await openTransitTable(page);
    const stepper = table.getByRole("group", { name: "Transit anını adımla" });
    await expect(stepper.getByRole("note")).toContainText("oynatma sunulmuyor");
    await expect(stepper.getByRole("button", { name: "Oynat" })).toHaveCount(0);
    await expect(stepper.getByRole("combobox", { name: "Hız" })).toHaveCount(0);
    await stepper.getByRole("button", { name: "Bir adım ileri" }).click();
    await expect(table.getByRole("status")).toHaveText("Gösterilen an: 26 Kas 2026 13:00");
    expect(await steps(page)).toHaveLength(1);
  });
});
````

```bash
pnpm --dir app exec vitest run ui/transit/TransitPlayback.test.tsx 2>&1 | grep -E "Test Files|Tests "
```
Expected: dosya düşer (`useTransitPlayback` yok).

- [ ] **Step 2: Kanca, bileşen, metinler ve REVIEW kaydı**

**Yeni dosya:** `app/ui/transit/useTransitPlayback.ts`

````ts
// Transit anının oynatılması (yol haritası Plan 26, §4.10): seçili birimle, sabit aralıkla ileri
// adım. Oynatma yeni bir adımı ancak önceki an yerleşince planlar: adım isteği bitmiş ve o anın
// transitleri gelmiş olmalıdır. Böylece tek ephemeris iş parçacığında iş birikmez ve oynatma
// sürerken harita açmak beklemez. Zamanlayıcı bileşen kalkınca (panel kapanınca), harita
// değişince, bir adım ya da transit hatasında ve sistem "hareketi azalt" isteyince durur.
// "Hareketi azalt" açıkken oynatma sunulmaz; adım düğmeleri çalışır.
import { useCallback, useEffect, useState } from "react";
import { useSession } from "../session/store";
import type { TransitStepper } from "./useTransitStepper";

export const PLAY_SPEEDS = ["slow", "normal", "fast"] as const;
export type PlaySpeed = (typeof PLAY_SPEEDS)[number];
export const DEFAULT_PLAY_SPEED: PlaySpeed = "normal";
/** Bir an yerleştikten sonra sonraki adıma dek beklenen süre. */
export const PLAY_INTERVAL_MS: Record<PlaySpeed, number> = { slow: 2000, normal: 1000, fast: 400 };
const REDUCED_MOTION_QUERY = "(prefers-reduced-motion: reduce)";

function reducedMotionQuery(): MediaQueryList | null {
  return typeof window.matchMedia === "function" ? window.matchMedia(REDUCED_MOTION_QUERY) : null;
}

/** Sistem "hareketi azalt" istiyor; ayar değişince yeniden çizer. */
export function useReducedMotion(): boolean {
  const [reduced, setReduced] = useState(() => reducedMotionQuery()?.matches ?? false);
  useEffect(() => {
    const query = reducedMotionQuery();
    if (query === null) return;
    const onChange = () => setReduced(query.matches);
    onChange();
    query.addEventListener("change", onChange);
    return () => query.removeEventListener("change", onChange);
  }, []);
  return reduced;
}

export interface TransitPlayback {
  /** Oynatma sunuluyor ("hareketi azalt" kapalı). */
  available: boolean;
  playing: boolean;
  speed: PlaySpeed;
  setSpeed(speed: PlaySpeed): void;
  toggle(): void;
}

export function useTransitPlayback(stepper: TransitStepper): TransitPlayback {
  const reduced = useReducedMotion();
  const { opened } = useSession();
  const chartId = opened?.chart_record.id ?? null;
  const [playing, setPlaying] = useState(false);
  const [speed, setSpeed] = useState<PlaySpeed>(DEFAULT_PLAY_SPEED);
  const { step, transit } = stepper;
  // An yerleşti: çıkış anı biliniyor, adım isteği bitti ve o anın transitleri geldi.
  const settled = stepper.ready && !stepper.busy && !transit.loading;
  const failed = stepper.error !== null || transit.error !== null;
  const frame = transit.view?.snapshot.jd_ut ?? null;

  // Başka harita: oynatma durur.
  useEffect(() => setPlaying(false), [chartId]);
  // Hata ya da "hareketi azalt": oynatma durur.
  useEffect(() => {
    if (failed || reduced) setPlaying(false);
  }, [failed, reduced]);
  // Yerleşen her an için tek bir sonraki adım planlanır; temizlik zamanlayıcıyı kaldırır.
  useEffect(() => {
    if (!playing || !settled || failed || reduced) return;
    const timer = setTimeout(() => void step(1), PLAY_INTERVAL_MS[speed]);
    return () => clearTimeout(timer);
  }, [playing, settled, failed, reduced, frame, speed, step]);

  const toggle = useCallback(() => setPlaying((on) => !on), []);
  return { available: !reduced, playing: playing && !reduced, speed, setSpeed, toggle };
}
````

**Yeni dosya:** `app/ui/transit/TransitPlayback.tsx`

````tsx
// Transit oynatma denetimleri (yol haritası Plan 26, §4.10): oynat ya da duraklat ve hız seçici.
// Düğmenin adı durumu söyler ("Oynat" ya da "Duraklat"). Sistem "hareketi azalt" istiyorsa
// denetimler yerine nedeni yazan bir satır çizilir; adım düğmeleri çalışmayı sürdürür.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import { PLAY_SPEEDS, useTransitPlayback, type PlaySpeed } from "./useTransitPlayback";
import type { TransitStepper } from "./useTransitStepper";

export function TransitPlayback({ stepper }: { stepper: TransitStepper }) {
  const { t } = useTranslation();
  const playback = useTransitPlayback(stepper);
  const id = useId();
  if (!playback.available) {
    return (
      <p className="transit-stepper-note" role="note">
        {t("transitStep.reducedMotion")}
      </p>
    );
  }
  // Oynatma sürerken düğme hep kullanılabilir: duraklatmak her an mümkündür.
  const blocked = !stepper.ready && !playback.playing;
  return (
    <>
      <button type="button" data-playing={playback.playing ? "" : undefined} aria-disabled={blocked ? true : undefined} onClick={() => !blocked && playback.toggle()}>
        {t(playback.playing ? "transitStep.pause" : "transitStep.play")}
      </button>
      <label htmlFor={`${id}-speed`}>{t("transitStep.speed")}</label>
      <select id={`${id}-speed`} value={playback.speed} onChange={(e) => playback.setSpeed(e.target.value as PlaySpeed)}>
        {PLAY_SPEEDS.map((s) => (
          <option key={s} value={s}>
            {t(`transitStep.speeds.${s}`)}
          </option>
        ))}
      </select>
    </>
  );
}
````

````diff
diff --git a/app/ui/transit/TransitStepper.tsx b/app/ui/transit/TransitStepper.tsx
index c1f6c63..4973eb1 100644
--- a/app/ui/transit/TransitStepper.tsx
+++ b/app/ui/transit/TransitStepper.tsx
@@ -1,5 +1,5 @@
-// Transit adım denetimleri (yol haritası Plan 26, §4.10): birim seçici ve bir adım geri, bir adım
-// ileri düğmeleri. Adımın hesabı Rust'tadır (`useTransitStepper`). Düğmeler kullanılamazken yerel
+// Transit adım denetimleri (yol haritası Plan 26, §4.10): birim seçici, bir adım geri ve bir adım
+// ileri düğmeleri, yanında oynatma denetimleri (`TransitPlayback`). Adımın hesabı Rust'tadır (`useTransitStepper`). Düğmeler kullanılamazken yerel
 // `disabled` değildir: odakta durur, tıklama bir şey yapmaz. Hata yalnızca alan adını taşır; ad
 // arayüz dilinde yazılır.
 import { useId } from "react";
@@ -7,6 +7,7 @@ import { useTranslation } from "react-i18next";
 import type { CommandError, StepUnit } from "../bindings";
 import { PanelError } from "../panels/states";
 import "./stepper.css";
+import { TransitPlayback } from "./TransitPlayback";
 import { STEP_UNITS, useTransitStepper } from "./useTransitStepper";
 
 /** `invalid_transit_input`'un alan adını arayüz dilinde yazar; öteki hatalar olduğu gibi durur. */
@@ -39,6 +40,7 @@ export function TransitStepper() {
         <button type="button" aria-disabled={blocked ? true : undefined} onClick={() => void stepper.step(1)}>
           {t("transitStep.forward")}
         </button>
+        <TransitPlayback stepper={stepper} />
       </div>
       {shownError && <PanelError error={shownError} onRetry={stepper.retry} />}
     </>
````

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index a27b103..11b5add 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -1415,6 +1415,11 @@ export const en: Messages = {
     units: { Hour: "Hour", Day: "Day", Week: "Week", Month: "Month", Year: "Year" },
     back: "One step back",
     forward: "One step forward",
+    play: "Play",
+    pause: "Pause",
+    speed: "Speed",
+    speeds: { slow: "Slow", normal: "Medium", fast: "Fast" },
+    reducedMotion: "“Reduce motion” is on in the system: playback is not offered. The step buttons work.",
     field: { jd_ut: "moment", count: "step count", tz: "time zone", range: "date range (years 1–2999)" },
   },
   transit: {
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 68b5445..eb21b42 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -1415,6 +1415,11 @@ export const fr: Messages = {
     units: { Hour: "Heure", Day: "Jour", Week: "Semaine", Month: "Mois", Year: "Année" },
     back: "Un pas en arrière",
     forward: "Un pas en avant",
+    play: "Lire",
+    pause: "Mettre en pause",
+    speed: "Vitesse",
+    speeds: { slow: "Lente", normal: "Moyenne", fast: "Rapide" },
+    reducedMotion: "« Réduire les animations » est activé dans le système : la lecture n’est pas proposée. Les boutons de pas fonctionnent.",
     field: { jd_ut: "moment", count: "nombre de pas", tz: "fuseau horaire", range: "plage de dates (années 1–2999)" },
   },
   transit: {
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 31addfd..ac97c2e 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -1414,6 +1414,11 @@ export const tr = {
     units: { Hour: "Saat", Day: "Gün", Week: "Hafta", Month: "Ay", Year: "Yıl" },
     back: "Bir adım geri",
     forward: "Bir adım ileri",
+    play: "Oynat",
+    pause: "Duraklat",
+    speed: "Hız",
+    speeds: { slow: "Yavaş", normal: "Orta", fast: "Hızlı" },
+    reducedMotion: "Sistemde “hareketi azalt” açık: oynatma sunulmuyor. Adım düğmeleri çalışır.",
     field: { jd_ut: "an", count: "adım sayısı", tz: "saat dilimi", range: "tarih aralığı (1–2999 yılları)" },
   },
   transit: {
````

````diff
diff --git a/tests/golden/modules/transit_tools/REVIEW.md b/tests/golden/modules/transit_tools/REVIEW.md
index fdffadd..2aa5da8 100644
--- a/tests/golden/modules/transit_tools/REVIEW.md
+++ b/tests/golden/modules/transit_tools/REVIEW.md
@@ -121,3 +121,26 @@ Vimshottari sırasının üç kez yazılmış biçimidir ve testte o sırayla ka
 - [ ] Duvar saatinin tarayıcının diliminde okunması (öteki okuma: haritanın ya da bulunulan
       yerin dilimi).
 - [ ] Ay sonunda ayın son gününe düşme ve boşlukta ileri alma.
+
+### 4.3 Oynatma
+
+1. **Oynat** seçili birimle ileri adım atar; **Duraklat** durdurur. Geri oynatma yoktur.
+2. **Hız** iki adım arasındaki bekleme süresidir: Yavaş 2 sn, Orta 1 sn (varsayılan), Hızlı 0,4 sn.
+3. **Sıra.** Sonraki adım, önceki an yerleşince planlanır: adım isteği bitmiş ve o anın
+   transitleri gelmiş olmalıdır. Motor meşgulse oynatma bekler; iş birikmez ve oynatma sürerken
+   harita açmak beklemez.
+4. **Durma.** Duraklat düğmesi; panelin kapanması; başka bir haritanın açılması; adım ya da
+   transit hatası (aralığın sonu dahil); sistemin "hareketi azalt" ayarının açılması.
+5. **Elle seçilen an** (tarih kutuları, Şimdi, başka bir panelin satırı, adım düğmeleri) oynatmayı
+   durdurmaz: oynatma o andan sürer.
+6. **Hareketi azalt.** Sistem ayarı açıkken oynatma düğmesi ve hız seçici çizilmez; yerinde
+   nedeni yazan bir satır durur. Adım düğmeleri çalışır.
+7. Birim, hız ve oynatma durumu oturumluktur: düzenle birlikte saklanmaz (transit anı da
+   saklanmaz).
+
+### 4.4 Onay bekleyen noktalar (sahibi)
+
+- [ ] Üç hız ve süreleri (2 sn, 1 sn, 0,4 sn); varsayılan hız.
+- [ ] Geri oynatmanın olmaması.
+- [ ] Elle seçilen anın oynatmayı durdurmaması.
+- [ ] Denetimlerin yalnızca Transit tablosu panelinde durması (harita panelinde ayrı düğme yok).
````

- [ ] **Step 3: Testler, akışlar ve kapılar**

```bash
pnpm --dir app exec vitest run ui/transit 2>&1 | grep -E "Tests |FAIL"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
lsof -i :14326
pnpm --dir app exec playwright test --project=e2e -c playwright.port.config.ts 2>&1 | tail -1
pnpm --dir app exec playwright test --project=e2e -c playwright.port.config.ts 2>&1 | tail -1
rm -rf app/test-results
```
Expected: `33 passed`; Vitest `104 passed (104)`, `1028 passed (1028)`; `✓ built`; `lsof` çıktısız; e2e iki koşuda da `37 passed` (oynatma akışı zamana bağlıdır: iki koşu dalgalanmayı görünür kılar; bu plan yazılırken üç koşu da temizdi).

- [ ] **Step 4: Commit**

```bash
git add app/ui/transit/useTransitPlayback.ts app/ui/transit/TransitPlayback.tsx app/ui/transit/TransitPlayback.test.tsx app/ui/transit/TransitStepper.tsx app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts app/e2e/transit-step.desk.spec.ts tests/golden/modules/transit_tools/REVIEW.md
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(transit): play and pause the transit moment, one step per settled moment, no playback under reduced motion" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: dokuz dosya; yasaklı ad taraması çıktısız; `git status --short` boş.


---

### Task 8: Kapılar, REVIEW.md'nin elle denetim kaydı ve sahibine devir

Yol haritası görev taslağının (8). maddesi ve "Sahibin denetimi" paragrafı; Y2–Y4. Kod değişmez; bulunan bir hata kendi düzeltme commit'iyle çözülür ve bu görev baştan koşulur. Bu görevin iki dosyası belgedir. Gerçek uygulama çalıştırılmaz: **[insan]** maddeleri sahibine devredilir ve sonraki planı engellemez (Y3). `BASE`, Task 1 Step 1'de okunan commit'tir (bu plan yazılırken `9727c55`).

**Files:**
- Modify: `tests/golden/modules/transit_tools/REVIEW.md`, `tests/golden/modules/README.md`

**Interfaces:**
- Consumes: Task 1–7'nin tamamı.
- Produces: yeşil kapılar, kural ve gizlilik taramaları, `REVIEW.md` §5 (elle denetim kaydı) ve §6 (gözden geçirme kaydı tablosu, boş), golden `README.md`'de bir paragraf.

- [ ] **Step 1: Rust kapıları**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
BASE=9727c55
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -1
cargo test --workspace 2>&1 | grep -E "^test result|Running"
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
cargo test -p jyotish-desk --test sankranti --test ephemeris_table -- --nocapture 2>&1 | grep -E "^(sankranti_list|ephemeris_month)"
cargo deny check 2>&1 | tail -1
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p jyotish-core -p atlas -p store -p jyotish-desk --all-targets 2>&1 | tail -1
git diff $BASE..HEAD --stat -- deny.toml Cargo.toml Cargo.lock app/package.json app/pnpm-lock.yaml app/vite.config.ts app/playwright.config.ts app/src-tauri/capabilities app/src-tauri/tauri.conf.json app/src-tauri/Cargo.toml crates/store crates/atlas crates/ephemeris crates/jyotish-core/Cargo.toml crates/jyotish-core/src/graha.rs crates/jyotish-core/src/chart.rs crates/jyotish-core/src/transit.rs crates/jyotish-core/src/timeline.rs crates/jyotish-core/src/panchang.rs crates/jyotish-core/src/error.rs crates/jyotish-core/src/dasha.rs crates/jyotish-core/src/search crates/jyotish-core/src/muhurta crates/jyotish-core/src/natal_extras crates/jyotish-core/src/bhava crates/jyotish-core/tests app/src-tauri/src/commands.rs app/src-tauri/src/transit_commands.rs app/src-tauri/src/muhurta_commands.rs app/src-tauri/src/search_commands.rs app/src-tauri/tests/ui_fixture.rs app/ui/test/fixtures/delhi.json app/ui/test/fixture.ts app/ui/transit/store.tsx app/ui/transit/TransitDate.tsx app/ui/i18n/i18n.test.ts app/ui/App.test.tsx app/e2e/desk.spec.ts app/e2e/visual.spec.ts app/e2e/visual.spec.ts-snapshots app/ui/panels/EventSearchPanel.test.tsx app/ui/panels/GrahaTablePanel.test.tsx
```
Expected: fmt çıktısız; clippy `Finished`; testler: `jyotish-core` lib `483 passed`, `avastha_golden` 5, golden 8, `modules_golden` 17; `atlas` 6 / 11 / 13 / 8 / 3; `store` lib `122 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `64 passed`, `avastha` 1, `bhava` 1, `bindings` 1, `compute_chart` 1, `ephemeris_table` 1, `modules` 1, `natal_extras` 1, `open_client` 1, `sankranti` 1, `ui_fixture` 2; sayaç `0`; iki ölçüm satırı, ikisi de 100 ms'nin altında (D17; yük altında düşen bütçe testi makine boşken yeniden koşulur, bütçe gevşetilmez); deny `advisories ok, bans ok, licenses ok, sources ok`; MSRV `Finished`; `git diff --stat` çıktısız (bağımlılıklar, var olan tipler ve hesap modülleri, var olan komutlar, `delhi.json` ve üreticisi, transit deposu, paylaşılan test dosyaları ve görsel temeller değişmedi).

- [ ] **Step 2: Arayüz kapıları**

```bash
lsof -i :14326
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
pnpm --dir app exec playwright test --project=e2e -c playwright.port.config.ts 2>&1 | tail -1
pnpm --dir app exec playwright test --project=visual -c playwright.port.config.ts 2>&1 | tail -1
rm -rf app/test-results
node --test "scripts/release/*.test.mjs" 2>&1 | grep -E "ℹ (pass|fail)"
git status --short
```
Expected: `lsof` çıktısız; `Test Files  104 passed (104)`, `Tests  1028 passed (1028)`; `✓ built`; e2e `37 passed`; görsel `15 passed` (temel PNG'ler değişmedi: D21); betik testleri `pass 14`, `fail 0`; `git status --short` boş.

- [ ] **Step 3: Kural, gizlilik ve "yalnızca ek" taramaları (tabandan bu yana)**

```bash
BASE=9727c55
git diff $BASE..HEAD -U0 -- . ':!app/ui/test/fixtures' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git log $BASE..HEAD --format=%B | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff $BASE..HEAD --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/sankranti.json app/ui/test/fixtures/ephemeris.json
git diff $BASE..HEAD --name-only | grep -vE '^(app|crates|tests)/'
git ls-files | grep -E 'playwright\.port\.config|^\.superpowers'
echo "trailers: $(git log $BASE..HEAD --format=%B | grep -c '^Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>$') commits: $(git rev-list --count $BASE..HEAD)"
git diff $BASE..HEAD -U0 -- app/ui | grep '^+' | grep -E 'console\.(log|info|warn|error|debug)'
git diff $BASE..HEAD -U0 -- crates app/src-tauri/src | grep '^+' | grep -E 'println!|eprintln!|dbg!|log::|tracing::'
git diff $BASE..HEAD -U0 -- app/ui | grep '^+' | grep -E '\b(alert|confirm|prompt)\(|window\.(print|open)\('
grep -rnE '#\[serde\(default' crates/jyotish-core/src/transit_tools app/src-tauri/src/transit_tools_commands
grep -rnE 'XMLHttpRequest|WebSocket|\bfetch\(|localStorage|sessionStorage|Date\.now' app/ui/modules/useSankranti.ts app/ui/modules/useEphemeris.ts app/ui/panels/SankrantiPanel.tsx app/ui/panels/EphemerisPanel.tsx app/ui/panels/nakshatraLord.ts app/ui/transit/useTransitStepper.ts app/ui/transit/useTransitPlayback.ts app/ui/transit/TransitStepper.tsx app/ui/transit/TransitPlayback.tsx
grep -cE 'transit_tools_commands::(sankranti::sankranti_list|ephemeris::ephemeris_month|step::transit_step),' app/src-tauri/src/lib.rs
grep -rhoE 'CommandError::new\("[a-z_]+"' app/src-tauri/src/transit_tools_commands | sort -u
grep -rnE 'InvalidTransitInput\((format!|&format|String)' crates app/src-tauri/src
git diff $BASE..HEAD -U0 -- crates/jyotish-core/src/lib.rs app/src-tauri/src/lib.rs app/ui/api.ts app/ui/api.test.ts app/ui/layout/model.ts app/ui/panels/registry.ts app/ui/panels/panels.test.tsx app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts app/e2e/ipc.ts | grep -c '^-[^-]'
git diff $BASE..HEAD -U0 -- app/ui/layout/model.test.ts | grep -c '^-[^-]'
git tag --points-at HEAD
```
Expected: `trailers:` satırındaki iki sayı eşit; fikstür sayaçları `0`; komut sayacı `3`; `CommandError::new` satırı tektir: `CommandError::new("invalid_transit_input"` (yeni hata kodu yok: D20); "yalnızca ek" sayacı `0` (paylaşılan on bir dosyada tabana göre silinen ya da yeniden sarılan satır yok); `model.test.ts` sayacı `1` (tek satırlık `MODULE_PANELS` beklentisi). Bunların dışındaki her komut çıktısızdır (çıktısız `grep` 1 ile çıkar; beklenen budur): yasaklı ad yok; `app/`, `crates/` ve `tests/` dışında dosya yok; port dosyası ve `.superpowers/` izlenmiyor; arayüzde `console`, Rust'ta log çağrısı yok (`eprintln!` yalnızca `app/src-tauri/tests/` altındadır); yasak çağrı yok; IPC tiplerinde `serde(default)` yok; yeni yüzeyler ağ, depolama ve sistem saati kullanmaz (`new Date(…)` yalnızca biçimlemede, verilen değerle kullanılır); etiket yok.

- [ ] **Step 4: REVIEW.md: elle denetim kaydı; golden README'ye paragraf**

Kayıt `REVIEW.md`'nin sonuna eklenir. Her **[otomatik]** madde, adı geçen testler bu oturumda (Step 1 ve Step 2) yeşil görüldükten sonra `- [x]` yapılır; düşen bir test varsa madde işaretlenmez, düzeltme commit'i atılır ve bu görev baştan koşulur. **[insan]** maddeleri ve §1.1, §2.2, §3.2, §4.2, §4.4'teki onay noktaları **işaretlenmez**: yalnızca sahibi işaretler. Yürüten ajan sonuçları başka bir yazılımla ya da çevrimiçi kaynakla karşılaştırmaz; gerçek uygulamayı (`pnpm --dir app tauri dev`, üretilen `.app`) çalıştırmaz.

````diff
diff --git a/tests/golden/modules/transit_tools/REVIEW.md b/tests/golden/modules/transit_tools/REVIEW.md
index 2aa5da8..4b88798 100644
--- a/tests/golden/modules/transit_tools/REVIEW.md
+++ b/tests/golden/modules/transit_tools/REVIEW.md
@@ -144,3 +144,97 @@ Vimshottari sırasının üç kez yazılmış biçimidir ve testte o sırayla ka
 - [ ] Geri oynatmanın olmaması.
 - [ ] Elle seçilen anın oynatmayı durdurmaması.
 - [ ] Denetimlerin yalnızca Transit tablosu panelinde durması (harita panelinde ayrı düğme yok).
+
+## 5. Elle denetim kaydı (Plan 26 Task 8)
+
+Her maddenin başında, otomatik bir karşılığı olup olmadığı yazılıdır. **[otomatik]** maddelerin
+karşılığı testlerdedir; yürüten ajan testin adını göstererek işaretler. **[insan]** maddeler
+yalnızca gerçek uygulamaya (ya da isterse basılı bir efemerise) bakan bir insan tarafından
+doğrulanır ve sahibine devredilir; devir sonraki planı engellemez (Y3). Gerçek uygulamada yalnızca
+deneme danışanları kullanılır.
+
+- [ ] **[otomatik]** Nakshatra lordları tablosu Vimshottari sırasının üç kez yazılmış biçimidir;
+  hesap fonksiyonuyla ve doğum dasasının lorduyla 27 nakshatrada aynıdır; arayüzün okuduğu sabit,
+  fikstürdeki Rust lordlarıyla aynıdır. (`lords.rs` üç test; `nakshatraLord.test.ts` üç test;
+  `tests/bindings.rs`)
+- [ ] **[otomatik]** Geçiş tablosu her satırda nakshatra lordunu yazar; graha tablosunda sütun
+  isteğe bağlıdır, varsayılan olarak gizlidir, lagna satırını da doldurur ve doğum saati
+  bilinmiyorsa Ay'ın lordu "≈" taşır; olay aramasında yalnızca nakshatra girişi satırı lordu
+  yazar ve sütun sayısı değişmez; hazır seçim aramayı başlatmaz. (`TransitTablePanel.test.tsx ›
+  the nakshatra lord column …`; `GrahaTableNakshatraLord.test.tsx` beş test;
+  `EventSearchLords.test.tsx` dört test)
+- [ ] **[otomatik]** Sankranti: bir yılda on iki giriş, aritmetiğin verdiği anlarda ve burç
+  sırasıyla; giriş anında Güneş burcun başındadır; iki el örneği tithiyi, varayı ve Ay'ın
+  nakshatrasını verir; liste tropikal-rasi kipinde de sidereal girişlerdir; aralıklar boşluksuz
+  birleşir. (`sankranti.rs` yedi test)
+- [ ] **[otomatik]** Sankranti komutu gerçek motorla: 2026 için on iki giriş, ilki Ocak ortasında
+  Oğlak; anlar ephemeris'in sidereal Güneş boylamıyla ve olay aramasının Güneş burç girişleriyle
+  aynıdır; tithi, vara ve nakshatra v1'in panchangıdır; yıl verilmezse yerin bugünkü yılıdır;
+  kutup gecesindeki giriş `SunriseUnavailable` uyarısı taşır; hata yalnızca alan adını taşır; bir
+  yıl 100 ms'nin altındadır. (`tests/sankranti.rs`)
+- [ ] **[otomatik]** Efemeris tablosu: satır, anının dokuz grahasıdır ve v1'in transit hesabıyla
+  iki zodyak kipinde aynıdır; giriş işareti girilen burcu, dönüş işareti dönülen yönü taşır;
+  düğümlerin yönü işaretlenmez; kapatan an satır olmaz; bozuk anlar alan adıyla reddedilir.
+  (`ephemeris_table.rs` yedi test)
+- [ ] **[otomatik]** Efemeris komutu gerçek motorla: Kasım 2026 otuz satırdır ve her satır yerin
+  gece yarısındadır; konumlar transit komutuyla, işaretler `timeline` dizilerinin sınırlarıyla üç
+  ayda, iki zodyak kipinde ve gerçek düğümle aynıdır; ay verilmezse yerin bugünkü ayıdır; önceki
+  ve sonraki ay yıl sınırını geçer ve aralığın ucunda durur; yaz saati gece yarısında başlayan
+  yerde gün 23 saattir; dosyaların dışındaki yıl `LowPrecision` taşır; bir ay 100 ms'nin
+  altındadır. (`tests/ephemeris_table.rs`; `transit_tools_commands/ephemeris.rs ›
+  months_step_across_the_year_and_stop_at_the_range_ends`)
+- [ ] **[otomatik]** Adım aritmetiği: saat adımı tam 3 600 saniyedir; gün ve hafta adımı duvar
+  saatini korur (yaz saati geçişinde 23 saat; bir hafta ve bir saat); ay ve yıl adımı ayın son
+  gününe düşer; boşluğa düşen duvar saati ileri alınır, örtüşmede önceki an alınır; hata yalnızca
+  alan adını taşır; aralığın ucunda adım reddedilir. (`transit_tools_commands/step.rs` beş test)
+- [ ] **[otomatik]** Sankranti paneli: yer seçilmeden istek gitmez; "bu yıl" Rust'ta okunur; on
+  iki satır yerin saatiyle yazılır; yazılan yıl gönderilince istenir, geçersiz yıl gönderilmez;
+  başka yıl okunurken tablo yerinde durur; satırdaki an transit tarihini taşır; uyarılar ve
+  tropikal harita notu sözle yazılır; hata alan adını arayüz dilinde söyler.
+  (`SankrantiPanel.test.tsx` on test; e2e `sankranti: …`)
+- [ ] **[otomatik]** Efemeris paneli: bir gün bir satırdır; sütun başlıkları ekran okuyucuya tam
+  adı okur; retro düz "R"dir ve düğümlerde yazılmaz; giriş ve dönüş işaretleri simge ve sözdür ve
+  Rust'ın işaretlediği hücrelerdedir; "Nakshatra" kutusu istek başlatmaz ve düzenle birlikte
+  saklanır; önceki, sonraki ve "bu ay" Rust'ın döndürdüğü aylarla çalışır; aralığın ucundaki
+  düğme odakta durur ve bir şey yapmaz. (`EphemerisPanel.test.tsx` on test; e2e `ephemeris: …`)
+- [ ] **[otomatik]** Adım düğmeleri: adım Rust'a gösterilen an, birim ve tarayıcının dilimiyle
+  sorulur; sonraki adım, transitleri gelmeden de varılan andan çıkar; istek sürerken tıklama yok
+  sayılır; an başka bir yoldan değişmişse ya da bileşen kalkmışsa geç yanıt düşer; hata alan
+  adını söyler ve yeniden denenir. (`TransitStepper.test.tsx` on bir test; e2e `transit steps: …`)
+- [ ] **[otomatik]** Oynatma: aralık başına bir adım; önceki anın transitleri gelmeden adım
+  planlanmaz; hız ve birim seçilir; adım ya da transit hatası, bileşenin kalkması, başka harita
+  ve "hareketi azalt" oynatmayı durdurur; elle seçilen an yeni çıkış noktasıdır; "hareketi azalt"
+  açıkken oynatma düğmesi yoktur ve adım düğmeleri çalışır. (`TransitPlayback.test.tsx` on bir
+  test; e2e `transit steps: …` iki akış)
+- [ ] **[otomatik]** İki yeni panel paletten eklenir ve çıkarılır; liste ve tablo panel açılmadan
+  istenmez; var olan on beş görsel temel değişmez. (`panels.test.tsx`, `model.test.ts`; üç e2e
+  dosyası; `pnpm e2e:visual`)
+- [ ] **[otomatik]** Her metin üç dilde vardır; Fransızca metinlerde düz kesme işareti yoktur.
+  (`i18n.test.ts`; beş panel ve bileşen testinin dil testleri)
+- [ ] **[insan]** Sankranti listesinin tropikal haritada da sidereal verilmesi onaylanır (2.2);
+  varanın gün doğumuna göre okunması onaylanır.
+- [ ] **[insan]** Efemeris satırının saati (yerel gece yarısı ya da 00:00 UT) ve varsayılan yer
+  için karar verilir (3.2). İstenirse basılı bir efemerisle birkaç satır karşılaştırılır
+  (karşılaştırırken satır saati ve ayanamsa eşitlenir).
+- [ ] **[insan]** Gerçek uygulamada oynatma: üç hız okunur mu, varsayılan hız uygun mu; adım
+  birimleri yeterli mi (4.4). Harita panelindeki geçiş halkası ile tablo birlikte ilerler;
+  oynatma sürerken başka bir danışan açmak beklemez.
+- [ ] **[insan]** Gerçek uygulamada macOS "Hareketi azalt" (Erişilebilirlik › Ekran) açılınca
+  oynatma düğmesi gider ve satır görünür; oynatma sürerken açılınca oynatma durur.
+- [ ] **[insan]** Gerçek uygulamada Efemeris paneli: on iki sütunluk panelde tablo okunur; dar
+  panelde yana kayar; "Nakshatra" açıkken satır yüksekliği kabul edilir; açık ve koyu temada, üç
+  dilde okunur.
+- [ ] **[insan]** Gerçek uygulamada geçiş tablosu: yeni "Nakshatra lordu" sütunuyla tablo
+  varsayılan panel genişliğine sığar; graha tablosunda sütun "Sütunlar"dan açılır.
+- [ ] **[insan]** VoiceOver: iki yeni tablo başlıklarıyla duyurulur; efemeris işaretleri sözle
+  okunur; oynatma sürerken "Gösterilen an" duyurusu rahatsız edici değildir (ediciyse karar
+  olarak yazılır).
+- [ ] **[insan]** Denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, doğum verisi,
+  koordinat, yer ya da dosya yolu görünmez.
+
+## 6. Gözden geçirme kaydı
+
+| Tarih | Gözden geçiren | Kapsam | Sonuç |
+|---|---|---|---|
+| | | El örneklerinin kuraldan yeniden türetilmesi (Sankranti'nin iki girişi, efemerisin satır ve işaret örnekleri, adım aritmetiğinin takvim örnekleri) | |
+| | | Sahibinin okuma seçimleri onayı (1.1, 2.2, 3.2, 4.2, 4.4) | |
````

````diff
diff --git a/tests/golden/modules/README.md b/tests/golden/modules/README.md
index ebf72ad..271825c 100644
--- a/tests/golden/modules/README.md
+++ b/tests/golden/modules/README.md
@@ -122,4 +122,13 @@ Gerçek motorla çapraz denetim `app/src-tauri/tests/natal_extras.rs` ve
 sahibin onayını bekleyen noktalar, denetim maddeleri ve gözden geçirme kaydı
 `natal_extras/REVIEW.md` ve `bhava/REVIEW.md`'dedir.
 
+Geçiş araçları (yol haritası §4.7–§4.10, Plan 26) da bu dizinde JSON örneği taşımaz:
+`transit_tools/` klasöründe yalnızca `REVIEW.md` vardır ve hiçbir test bu klasörü yüklemez. El
+örnekleri birim testleridir (`crates/jyotish-core/src/transit_tools/` ve
+`app/src-tauri/src/transit_tools_commands/step.rs` içindeki `mod tests`). Gerçek motorla çapraz
+denetim `app/src-tauri/tests/sankranti.rs` ve `app/src-tauri/tests/ephemeris_table.rs`
+dosyalarındadır; bu iki test arayüz fikstürlerini de üretir
+(`app/ui/test/fixtures/sankranti.json`, `ephemeris.json`). Okuma kuralları, sahibin onayını
+bekleyen seçimler ve denetim maddeleri `transit_tools/REVIEW.md`'dedir.
+
 Sonraki alt projeler kendi klasörünü ve `expected` yapısını ekler.
````

```bash
sed -i '' 's/^- \[ \] \*\*\[otomatik\]\*\*/- [x] **[otomatik]**/' tests/golden/modules/transit_tools/REVIEW.md
grep -c '^- \[x\] \*\*\[otomatik\]\*\*' tests/golden/modules/transit_tools/REVIEW.md
grep -c '^- \[ \] \*\*\[insan\]\*\*' tests/golden/modules/transit_tools/REVIEW.md
grep -c '^- \[ \] ' tests/golden/modules/transit_tools/REVIEW.md
grep -inE 'k(a|ā|aa)la|\bcala\b' tests/golden/modules/transit_tools/REVIEW.md tests/golden/modules/README.md
```
Expected: `13` (on üç otomatik maddenin hepsi işaretli); `8` (sekiz insan maddesi açık); `23` (sekiz insan maddesi ve on beş onay noktası açık); son komut çıktısız. `sed` yalnızca Step 1 ve Step 2 yeşil görüldüyse koşulur.

- [ ] **Step 5: Commit, port dosyasının silinmesi ve ilerleme notu**

```bash
git add tests/golden/modules/transit_tools/REVIEW.md tests/golden/modules/README.md
git diff --cached --stat
git diff --cached -U0 -- . ':!app/ui/test/fixtures' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "docs(review): manual check record for the transit tools; human items handed to the owner" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
rm -f app/playwright.port.config.ts
rm -rf app/test-results app/playwright-report
git status --short
```
Expected: iki dosya; yasaklı ad taraması çıktısız; `git status --short` boş.

İlerleme notlarına (depo dışında; `git add` edilmez) `Plan 26 Task 8: kapılar geçti; otomatik maddeler işaretlendi, insan maddeleri ve okuma seçimleri sahibine devredildi (<tarih>)` yazılır. Sahibine devredilenler, yol haritasının "Sahibin denetimi" paragrafındaki üç başlıktır: Sankranti'nin tropikal haritada da sidereal verilmesi; efemeris satırının saati (yerel gece yarısı ya da 00:00 UT); oynatma hızları ve adım birimleri (ekranda). Bir seçim değişirse önce `REVIEW.md` ve test, sonra kod değişir ve ilgili yöntem kimliği `v2` olur.

---

## Başarı ölçütleri

- Geçiş tablosu her grahanın nakshatra lordunu yazar; graha tablosunda sütun "Sütunlar"dan açılır; olay aramasının nakshatra girişi satırı lordu taşır; hazır seçim Ay'ın nakshatra girişlerini seçer. Var olan hiçbir Rust tipi, `delhi.json` ve var olan hiçbir test satırı (iki `TransitTablePanel.test.tsx` satırı dışında) değişmemiştir.
- Sankranti paneli seçilen yerin yılı için on iki girişi yerin saatiyle, tithi, vara ve Ay nakshatrasıyla listeler; liste her zodyak kipinde sidereal girişlerdir ve gerçek motorda ephemeris'in boylamıyla, olay aramasıyla ve v1'in panchangıyla uyuşur.
- Efemeris paneli bir ayın her günü için dokuz grahayı yazar; giriş ve dönüş işaretleri `timeline` dizilerinin sınırlarıyla uyuşur; ay gezinmesi Rust'ın döndürdüğü aylarla çalışır.
- Adım düğmeleri anı beş birimle ileri-geri yürütür; ay sonu ve yaz saati kuralları testlerle sabittir; oynatma yerleşen an başına tek adım atar, hatada, başka haritada, panel kapanınca ve "hareketi azalt"ta durur.
- Kapılar yeşildir: `jyotish-core` lib 483, `jyotish-desk` lib 64, iki yeni entegrasyon testi, Vitest 104 / 1028, e2e 37, görsel 15; `cargo deny` ve MSRV temizdir; yasaklı ad taraması boştur.
- `REVIEW.md`'de on üç **[otomatik]** madde işaretli, sekiz **[insan]** maddesi ve on beş onay noktası sahibine devredilmiştir.

## Bu planın kapsamadıkları

- Sankranti'nin kutlu süresi (punya dilimi), girişin özel adları ve yönü; Mesha girişinden başlayan yıl.
- Efemerisin CSV'si ve baskısı (K10); enlem, hız ve deklinasyon sütunları; yıl görünümü; 00:00 UT satırı; birden çok ayın tek tabloda gösterilmesi.
- Geri oynatma; oynatmanın harita panelindeki ayrı düğmesi; adım biriminin ve hızın düzenle birlikte saklanması; dakika adımı; klavye kısayolları (C7).
- Olay aramasında yeni ay ve dolunay satırına lord; olay tablosuna ayrı lord sütunu; KP alt lordu.
- Ay'ın nakshatra girişleri için ayrı komut ve ayrı liste (D5).
- Geçiş takvimi (Plan 29), tutulmalar ve "olayı harita olarak aç" (Plan 30).

## Yol haritasına geri bildirim (farklar ve boşluklar)

1. **§4.7 "lord … Rust tipine alan olarak eklenir".** Bu plan alan eklemez; lord, Rust'ın sabit tablosundan bağlama dosyasıyla gelir (D2). Neden: Plan 27 ve 29 aynı anda aynı tabandan yazılıyor ve `SkyEvent`, `TransitGraha`, `GrahaPosition` üçünün de girdisi; alan eklemek birleştirmede derleme ve fikstür çakışması üretirdi. C4'ün amacı (arayüz lordu kendi tablosundan türetmesin) korunur. Sahibi alanı yine de isterse, üç plan birleştikten sonra tek bir küçük iş olarak eklenebilir; arayüz o zaman da `nakshatraLord`'u bırakıp alanı okur.
2. **§4.7 "Ay'ın önümüzdeki nakshatra girişleri lordlarıyla".** Ayrı liste yerine olay aramasının hazır seçimi (D5). Yol haritası §3 de aynı bilgiyi olay aramasının verdiğini yazar.
3. **§4.9 "`timeline` (dönüş ve giriş günlerini işaretlemek için)".** İşaretler `timeline` yürüyüşüyle değil, ardışık iki satırın karşılaştırmasıyla üretilir; `timeline` gerçek motor testinde ikinci yazımdır (D14).
4. **§4.9 "harita açık değilken hangi ayarın kullanıldığı".** Paneller açık harita olmadan çizilmez; soru doğmaz (D13). Harita seçmeden açılan bir efemeris istenirse panel kataloğu kararıyla birlikte ele alınmalıdır (yol haritası §8 "panel sayısı").
5. **§4.10 bütçe ve hız yazmaz.** D17 (100 ms) ve D25 (2 sn, 1 sn, 0,4 sn) bu planın seçimidir.
6. **§5 görev taslağı (1) "olay satırı … ve sütunlar".** Olay tablosuna sütun eklenmedi (D4): paylaşılan iki test dosyasının var olan satırlarını değiştirirdi.
7. **E2e portu.** Önceki planlar 1421'i paylaşıyordu; eşzamanlı çalışma ağaçlarında bu, bir ağacın testlerinin ötekinin arayüzünü sınamasına yol açabilir. Bu plan geçici bir yapılandırma dosyasıyla özel port kullanır; kalıcı çözüm `playwright.config.ts`'in portu bir ortam değişkeninden okumasıdır (paylaşılan dosya olduğu için bu planda yapılmadı).

## Self-Review

1. **Kapsam:** yol haritası §4.7–§4.10'un her cümlesi "Kapsam" tablosunda bir göreve ve karara bağlıdır; kapsam dışı bırakılanlar "Bu planın kapsamadıkları"nda ve `REVIEW.md`'de yazılıdır.
2. **Yer tutucu yok:** her adım tam kod ya da tam `diff` taşır; "benzer biçimde" diye geçilen adım yoktur. Üretilen üç dosya komutla üretilir.
3. **Ad tutarlılığı:** `NAKSHATRA_LORDS`, `sankranti_list`, `sankranti_moments`, `ephemeris_table`, `MonthId`, `TransitStepRequest`, `StepUnit`, `useTransitStepper`, `stepBase`, `useTransitPlayback`, `parseYear`, `formatMonth` adları Interfaces bölümlerinde, kodda, testlerde ve "Sonraki işlere verilen API" tablosunda aynıdır.
4. **Eşzamanlılık:** paylaşılan on bir dosyadaki ekler çapalarıyla listelidir; Task 8 Step 3 "yalnızca ek" sayacını ve dokunulmayan dosyaların farkını denetler.
5. **Yeniden oynatma:** bu belgenin blokları, deponun `9727c55` kopyasında baştan sona uygulanmıştır: her görevin sonunda üretilen dosyalar komutla yeniden üretilmiş ve ağaç, deneme commit'iyle bayt bayt karşılaştırılmıştır (fark boş). Sayaçlar o koşulardan okunmuştur. Deneme kopyasının derleme çıktıları silinmiştir.
6. **Yasaklı ad:** bu belge `grep -inE 'k(a|ā|aa)la|\bcala\b'` ile taranmıştır ve çıktı boştur.
