# Plan 29 — Transit takvimi (ay ızgarası: takvim günleri, gün doğumundaki tithi ve Ay nakshatrası, günün olayları; `month_calendar` komutu, Transit takvimi paneli, klavye gezinmesi, gün ayrıntısı ve olay süzgeci)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Bu plan Claude dışı bir kodlama ajanı tarafından, ayrı bir git çalışma ağacında da yürütülebilir: gereken her yol, kod ve komut bu belgededir; başka bir oturumun bağlamına gönderme yoktur.

**Goal:** Olay aramasının düz listesini bir ay ızgarasına çevirmek (yol haritası §4.11). Her hücre seçilen yerin bir takvim günüdür ve üç şey taşır: ayın günü; o günün gün doğumundaki tithi ve Ay nakshatrası; o güne düşen gök olayları (burç ve nakshatra girişleri, R başlangıcı ve bitişi, yeni ay, dolunay; istenirse açık haritanın doğum boylamlarına gelişler ve Ay'ın kendi girişleri). Üç katman: `jyotish-core`'da saf `calendar` modülü (takvim günleri, gün doğumu olguları, olayların günlere dağıtımı, sınır); uygulamada tek komut (`month_calendar`: ayı yedi günlük dilimlerle hesaplar); arayüzde paletten açılan `calendar` paneli (ay ızgarası, ay seçimi, klavye gezinmesi, gün ayrıntısı, gün panchangına geçiş, olay süzgeci). Yorum, puan ve "uygun gün" hükmü yoktur: takvim yalnızca olguları sayar.

**Architecture:**
- **Yeni astronomi yoktur.** Olayları var olan olay araması çekirdeği bulur (`search::events::event_search_chunk`, `SkyEventKind`); tithi ve nakshatra v1'in panchang fonksiyonlarıyla okunur (`tithi_no`, `elongation`, `paksha`, `nakshatra_pada`); gün doğumu `Ephemeris::sun_rise_set` çağrısıdır. Var olan hiçbir hesabın davranışı değişmez: `search/`, `timeline.rs`, `panchang.rs` ve `muhurta/` yalnızca okunur.
- **Saf çekirdek, saat dilimi dışarıda (K3).** Çekirdek takvim günlerini `DaySpan` listesi olarak alır (`[başlangıç, bitiş)` JD çiftleri); günlerin sınırını (yerel gece yarısı) uygulama katmanı atlasın saat dilimi kurallarıyla çözer. Böylece yaz saati günleri (23 ve 25 saat) ve takvimden atlanan gün çekirdekte özel durum değildir.
- **Modül `Chart`'a eklenmez (K1).** Ay istek üzerine hesaplanır ve saklanmaz; veritabanı şeması değişmez (Y6).
- **Arayüz hesap yapmaz (C4).** "Bugün", ayın günleri, her günün hafta günü, önceki ve sonraki ay, gün panchangı paneline verilecek an Rust'tan gelir. Arayüz günleri hafta sütunlarına dizer, kimlikleri adlarına çevirir ve klavye odağını yürütür.
- **Dilimli yükleme.** Komut ayı en çok yedi günlük dilimlerle hesaplar; her dilim ayrı bir motor işidir ve aralarında motor iş parçacığı bırakılır. Eski isteğin geç gelen yanıtı arayüzde düşer (`useModule`).
- **Plan 26'ya bağımlılık yoktur.** Bu plan yalnızca bugün `main`'de olan koda dayanır (Kararlar D2).
- **Paralel planlarla çatışmayı azaltan yerleşim.** Plan 26 ve Plan 27 başka çalışma ağaçlarında aynı anda yürür. Bu plan yeni kodu yeni dosyalara koyar (çekirdekte `calendar/`, uygulamada `calendar_commands.rs` ve `tests/calendar.rs`, arayüzde `app/ui/calendar/` klasörü, kendi fikstür tipi, e2e ve görsel dosyaları) ve ortak dosyalara yalnızca küçük, ekleme türünde dokunur; bunların tam listesi "Paylaşılan dosyalar" bölümündedir. `jyotish_core` kökünde yeni `pub use` yoktur.
- **Testler:** çekirdekte sahte gökyüzüyle el örnekleri (gün doğumu olguları, gün sınırları, olayların günleri, süzgeç, sınır, dilimlemenin ikinci yazımı); uygulamada gerçek motorla çapraz denetim (gün panchangı ve olay aramasıyla), yaz saati, kutup dairesi, atlanan gün, hatalar ve bütçe; arayüzde Vitest (model, kanca, panel) ve yazılmış ama yürüten ajanın koşmadığı uçtan uca ve görsel akışlar.

**Tech Stack:** Rust 1.93+ (stable; yerelde ve CI'da 1.100), Tauri 2.12, tauri-specta 2.0.0-rc.29, serde, React 19, TypeScript 7, Vitest 5, Playwright 1.65.1. **Yeni Rust ya da npm bağımlılığı yok.**

**Spec:** `docs/superpowers/specs/2026-11-27-eksikler-yol-haritasi.md`: §1 (değişmeyen kurallar), §2 (Y1–Y8), §3 (geçiş takvimi satırı), §4.11, §5 "Plan 29", §7, §8, Ek A. Taşınan kurallar: `2026-11-26-alt-proje-3-ileri-moduller-design.md` §2 (K1–K12), §10 (bulunulan yer, gün panchangı), §12.2 (olay araması), §13 (ortak test stratejisi), §14 (gizlilik), §15 (bütçeler), Ek A. UI spec: `2026-11-24-ui-design.md` §3.3–§3.4 (paneller), §5 (yükleme evreleri), §8 (panel durumları).

**Ön koşul:** Plan 22 (olay araması), Plan 20 (gün panchangı, bulunulan yer) ve Plan 23–25 birleşmiştir. Bu plan yazılırken `main` commit `9727c55`'teydi; kod blokları ve sayaçlar o ağaca göre yazılmış ve ölçülmüştür. `main` bu commit'ten ilerlediyse Task 1 Step 1 tabanları yeniden ölçer ve sayaç zinciri o ölçümden yürütülür (farklar aynıdır); bir `diff` bloğunun bağlamı tutmazsa önce o satırlara dokunan commit okunur. `BASE` her zaman Task 1 Step 1'de okunan commit'tir. Plan 26 ve Plan 27 bu planın ön koşulu değildir ve bu planı beklemez.

## Kararlar

Sahibi kararları devretti; muhafazakâr seçenek seçildi ve her karar bir satır gerekçeyle yazıldı. "Yol haritası farkı" ya da "Yol haritası boşluğu" diye işaretlenenler yol haritasında düzeltilmesi gereken yerlerdir ve belgenin sonundaki "Spec'e geri bildirim" bölümünde toplanmıştır. Ölçümler ve sayaç zinciri bu plan yazılırken deponun bir kopyasında, her görev ayrı commit olarak uygulanarak alınmıştır (macOS 27.2, Rust 1.100, hata ayıklama derlemesi).

- **D1 — Çekirdek modülü `crates/jyotish-core/src/calendar/`'dır: `mod.rs` (kayıt, yöntem kimliği), `grid.rs` (takvim günleri, gün doğumu olguları, dilimler), `events.rs` (süzgeç, olayların günlere dağıtımı, sınır, dilim ve ay toplayıcısı). Yöntem kimliği `CALENDAR_METHOD = "calendar.month_v1"`'dir. Adlar `jyotish_core` kökünden yeniden dışa aktarılmaz; tüketiciler `jyotish_core::calendar::…` yolunu kullanır.** Gerekçe: K7 her sonucun yöntem kimliği taşımasını ister. Kökte yeniden dışa aktarım `lib.rs`'in uzun `pub use` bloğuna satır ekler ve paralel planlarla çatışır; `DaySpan` gibi genel bir adı başka bir plan da kökten çıkarmak isteyebilir.
- **D2 — Plan 26'ya bağımlılık yoktur. Yol haritası Plan 29'u Plan 26'ya bağlar ("nakshatra lordu alanı ve Sankranti çekirdeği"); bu plan ikisini de kullanmaz: Güneş'in burç girişi olay aramasının `SignIngress` olayıdır ve takvimde öteki girişler gibi durur; nakshatra lordu takvimde yazılmaz.** Gerekçe: iki plan aynı tabandan aynı anda yürür. Plan 26 birleştikten sonra lord sütunu ya da Sankranti işareti ayrı, küçük bir iş olarak eklenebilir. **Yol haritası farkı 1.**
- **D3 — Hücre takvim günüdür: seçilen yerin duvar saatiyle gece yarısından sonraki gece yarısına (`[başlangıç, bitiş)`). Hücrenin tithisi ve Ay nakshatrası, o takvim gününün içindeki gün doğumunda geçerli olan değerdir (o gün başlayan panchang gününün ilk değeri). Olay, anını içeren takvim günündedir.** Gerekçe: yol haritası §4.11 "hücre hangisini gösterir" kararını bu plana bırakır. Izgara bir duvar takvimidir ve astrolog randevuyu takvim gününe yazar; gün doğumundaki tithi ise panchang takvimlerinin günün tithisi diye bastığı değerdir. Panchang gününü hücre yapmak gece yarısından sonraki olayı önceki günün hücresine düşürürdü. Öteki okuma REVIEW'de 1. ve 2. noktadır ve sahibin kararını bekler.
- **D4 — Günlerin sınırını uygulama katmanı çözer (`muhurta_commands::day_start_jd`, atlasın saat dilimi kuralları) ve çekirdeğe `DaySpan` listesi verir. Yaz saati gününde hücre 23 ya da 25 saattir. Saat diliminin takvimden atladığı gün (Samoa, 30 Aralık 2011) listede yer almaz; öteki günler ardışık durur. Çekirdek listeyi denetler: 1–31 gün, her gün en çok 30 saat, günler boşluksuz ardışık.** Gerekçe: çekirdek saat dilimi bilmez (K3). Atlanan gün gerçek bir uç durumdur ve "ay hesaplanamadı" hatası vermek yerine o hücreyi boş bırakmak doğrudur.
- **D5 — O takvim gününde Güneş doğmuyorsa (kutup gecesi ya da gündüzü) olgular 06:00 yerel ortalama saatte okunur (günün ortasının düştüğü yerel ortalama tarihin 06:00'sı), `sunrise_jd_ut` `None`'dır ve hücre `*` işareti ile "gün doğumu yok" sözünü taşır. Günün içinde iki doğuş varsa ilki alınır.** Gerekçe: v1'in ve gün panchangı panelinin kuralı (06:00 yerel ortalama saat); aynı kural, "Gün panchangını aç" ile açılan panelin aynı günü göstermesini sağlar (gerçek motorla sınanır). Hücreyi boş bırakmak Tromsø'da iki ay boş ızgara demektir. REVIEW 3. nokta.
- **D6 — Takvim yedi olay türünü gösterebilir (`CALENDAR_KINDS`): `SignIngress`, `NakshatraIngress`, `StationRetrograde`, `StationDirect`, `NewMoon`, `FullMoon`, `TransitToNatal`. `Conjunction` ve `HouseFromNatalMoon` takvimde yoktur; süzgeçte verilirse hata `kinds`'tir.** Gerekçe: yol haritası §4.11'in saydığı türler tam olarak bunlardır. Doğum Ay'ından ev değişimi Jüpiter ve Satürn'ün burç girişidir ve o satırda zaten durur; kavuşum olay araması panelindedir.
- **D7 — Varsayılan türler altıdır (`CALENDAR_DEFAULT_KINDS`: doğum boylamına geliş dışındakiler) ve Ay ayrı bir seçimdir (`CalendarFilter.moon`, varsayılan kapalı): kapalıyken aranan grahalar Ay dışındaki sekiz grahadır; açılınca Ay'ın burç ve nakshatra girişleri ile doğum boylamlarına gelişleri de aranır. Yeni ay ve dolunay bu seçime bağlı değildir.** Gerekçe: ölçüm (Delhi, Kasım 2026): varsayılan süzgeç 13 olay, bütün türler ve Ay 71 olay verir (bir günde en çok 5). Ay tek başına olay sayısını üçe katlar ve hücre Ay'ın nakshatrasını zaten taşır. Yol haritası "varsayılan olay türleri"ni sahibin denetimine bırakır: REVIEW 4. nokta.
- **D8 — Doğum boylamına gelişler açık haritayı ister. Tür seçili değilse açık haritanın girdisi komuta gönderilmez; tür seçili ama girdi yoksa tür sessizce düşer ve yanıt `natal: false` taşır. Doğum saati bilinmiyorsa girdi `time_known: false` ile gider (boylamlar öğlen içindir) ve panel bunu bir notla söyler.** Gerekçe: olay araması panelinin kuralı (doğum verisi yalnızca gerektiğinde gider); süzgeç panelin ayarıdır ve haritadan bağımsız saklanır, bu yüzden haritasız istek hata olmamalıdır.
- **D9 — Sınırlar: bir istek en çok 31 gündür (`CALENDAR_MAX_DAYS`); bir ay en çok 300 olay taşır (`CALENDAR_MAX_EVENTS`), fazlası kesilir ve yanıt `truncated: true` olur; bir hücre en çok üç olay yazar (`CELL_EVENTS`, arayüz sabiti) ve gerisini "+N olay" diye sayar; gün ayrıntısı günün bütün olaylarını listeler.** Gerekçe: yol haritası §4.11 "ay başına olay sayısının sınırı". 300, ölçülen en dolu ayın (71) dört katından büyüktür: gerçek bir ayda kesilme beklenmez, sınır bozuk girdiye ve gelecekteki tür eklemelerine karşı emniyettir. Hücredeki üç satır 84 pikselin sığdırdığıdır.
- **D10 — Komut ayı en çok yedi günlük dilimlerle hesaplar (`CALENDAR_SLICE_DAYS`); her dilim ayrı bir motor işidir. Durdurma kaydı (olay aramasının `SearchRegistry`'si gibi) yoktur: eski isteğin geç gelen yanıtı arayüzde düşer. Bütçe: en ağır ay (bütün türler, Ay, açık harita) ortanca ≤ 150 ms (ölçüm: 12–17 ms; dilim başına 2–4 ms) ve testte `assert!` ile doğrulanır.** Gerekçe: yol haritası §8 "dilimleme ve eski yanıtın düşürülmesi her birinde ayrı test ister". Ölçülen süre bir durdurma kaydını gereksiz kılar: art arda on ay istemek motoru toplam 0,2 saniye meşgul eder ve araya giren harita hesabı bir dilimden uzun beklemez. Kayıt `lib.rs`'e yeni bir yönetilen durum ve arayüze bir durdurma komutu eklerdi. **Yol haritası boşluğu 2.**
- **D11 — Komut `month_calendar(request: CalendarRequest) -> CalendarView`'dur ve yeni bir dosyadadır: `app/src-tauri/src/calendar_commands.rs`. `request.month` boşsa yerin bugünkü ayıdır ("bugün" Rust'ta okunur). Yanıt ayı (`month`), önceki ve sonraki ayı (`prev`, `next`; aralığın ucunda `None`), bugünün gününü (`today`; başka aydaysa `None`), günleri, `truncated`, `natal`, uyarıları ve yöntem kimliğini taşır. Her gün tarihini, ayın gününü, takvim tarihinin hafta gününü (`Vara`), sınırlarını, olgularını, ofsetini, gün panchangı paneline verilecek anı (`panchang_jd_ut`) ve olaylarını (`SkyEventView`: olay ve kendi ofseti) taşır. Komut listesine `muhurta_commands::muhurta_search`'ten hemen sonra eklenir.** Gerekçe: arayüz tarih hesabı yapmaz (C4): ay ilerletme, hafta günü ve "bugün" Rust'tadır. Ayrı dosya `search_commands.rs` ve `muhurta_commands.rs` ile çatışmayı önler; ikisinin `pub(crate)` yardımcıları (`day_start_jd`, `offset_at`, `today`) ve `SkyEventView` tipi yalnızca okunur.
- **D12 — Yıl aralığı 1–2999'dur (`CALENDAR_YEARS`). Paketlenen ephemeris dosyalarının dışındaki aylar (1800'den önce, 2399'dan sonra) hesaplanır ve yanıt `LowPrecision` uyarısını taşır; panel var olan rozet metnini gösterir.** Gerekçe: hesap aralığının (`JD_UT_RANGE`) içindedir ve öteki paneller de aralık dışını uyarıyla hesaplar; dört haneli yıl alanı yeterlidir.
- **D13 — Girdi hatasının kodu `invalid_calendar_input`'tur ve yalnızca alan adı taşır (`place`, `month`, `kinds`, `range`). Yeni `CoreError` türü yoktur: çekirdek `InvalidSearchInput(alan)` döner, komut dosyası bunu ve muhurta yardımcılarının alan adlı hatasını takvimin koduna çevirir. `error.rs`, `commands.rs` ve `i18n.test.ts`'in hata listesi değişmez; kodun üç dildeki metni panel testinde sabitlenir.** Gerekçe: "Arama isteği geçersiz" metni takvimde yanıltıcı olurdu; yeni `CoreError` türü üç ortak dosyaya dokunurdu.
- **D14 — Hafta başlangıcı arayüz diline bağlıdır: Türkçe ve Fransızcada Pazartesi, İngilizcede Pazar (`WEEK_START`, `app/ui/calendar/model.ts`). Günün sütununu Rust'ın verdiği hafta günü belirler; ay adları ve kısa gün adları i18n kaynağındadır (`Intl` kullanılmaz).** Gerekçe: yol haritası "üç dilde hafta başlangıcı" der. Tablo `Language` tipine bağlıdır: yeni bir dil eklenince derleme hafta başlangıcını sorar. i18n'deki adlar testlerde ve WebKit'te aynıdır. REVIEW 7. nokta.
- **D15 — Panel `calendar` türüdür (`MODULE_PANELS`: 9 × 24; paletten eklenir ve çıkarılır, K12) ve hazır düzenlere eklenmez. Katalogda `muhurta_search`'ten hemen sonra durur. Türkçe adı "Transit takvimi"dir (İngilizce "Transit calendar", Fransızca "Calendrier des transits").** Gerekçe: arayüz "transit" sözcüğünü zaten kullanır ("Transit tablosu", "transit tarihi"); yol haritasındaki "geçiş takvimi" belge dilidir. Katalogdaki yer, paralel planların listenin sonuna ya da `event_search` çevresine yapacağı eklemeyle aynı satırlara düşmemek içindir. REVIEW 9. nokta.
- **D16 — Varsayılan yer bulunulan yerdir (gün panchangı gibi); seçim panelin ayarıdır (`place`: `home`, `chart`, `custom`) ve var olan `PlaceControls` bileşeniyle yapılır. Bulunulan yer seçilmemişse panel yer ister ve komut çağrılmaz.** Gerekçe: takvim astrologun kendi günlerini gösterir; yol haritası §4.11 bağımlılık olarak `home_place`'i sayar. REVIEW 6. nokta.
- **D17 — Ay seçimi: önceki ve sonraki ay düğmeleri (Rust'ın verdiği aylar), ay listesi, yıl alanı (Enter ya da alandan çıkışla uygulanır) ve "Bu ay". Gösterilen ay saklanmaz: panel her açılışta ve her yeni haritada bugünün ayını gösterir. "Bu ay" her basışta yeniden okur.** Gerekçe: transit anı da oturumluktur (Plan 11 D7). Yıl alanı her tuşta istek göndermez. Yeniden okuma, gece yarısından sonra "bugün"ün yenilenmesi içindir.
- **D18 — Hücre: ayın günü (bugünse "bugün" sözü ve çerçeve); tithinin kısa yazımı ve adı ("K1 Pratipada"; tam yazım ekran okuyucuya: "Krishna 1 · Pratipada"); Ay nakshatrasının adı; en çok üç olay, saatiyle ve kısa yazısıyla ("19:43 Gü → Akr"; tam adı ekran okuyucuya). Olay adları olay araması panelinin var olan metinlerinden okunur (`eventSearch.kind.*`, `eventSearch.event.*`).** Gerekçe: 100–130 piksellik hücreye sığan en çok bilgi; renk tek başına anlam taşımaz (C9). Var olan metinleri okumak iki panelin aynı olayı aynı adla yazmasını sağlar.
- **D19 — Izgara ARIA `grid`'dir ve tek bir sekme durağıdır (gezici `tabindex`). Ok tuşları günler arasında gezer; Home ve End haftanın, Ctrl ya da Cmd ile ayın ucuna gider; Page Up ve Page Down ayı değiştirir ve odak aynı günde (o gün yoksa ayın son gününde) durur; Enter, boşluk ya da tıklama günü seçer. Odak seçimi değiştirmez. Seçim yokken ayrıntı bugünündür (bu aydaysa). Izgara odaktayken DOM'dan çıkarsa odak panelin çerçevesine taşınır.** Gerekçe: yol haritası §4.11 "ızgaranın klavye ve ekran okuyucu erişimi"; WAI-ARIA takvim ızgarası kalıbı. Seçimin odağı izlememesi, ayrıntı başlığının (duyurulan bölge) her ok tuşunda okunmasını önler.
- **D20 — Gün ayrıntısı ızgaranın altındadır: günün başlığı, gün doğumu saati ve olguları, "Gün panchangını aç" düğmesi ve günün bütün olaylarının tablosu. Düğme transit tarihini `panchang_jd_ut`'ye götürür (olguların okunduğu andan bir dakika sonrası: an kesinlikle o gün doğumuyla başlayan günün içindedir) ve Gün panchangı paneli düzende yoksa ekler. Olayın saati transit tarihini o ana götürür.** Gerekçe: yol haritası (5) "gün ayrıntısı, gün panchangına geçiş". Gün panchangı paneli tarihini transit anından okur; geçiş var olan depoyla yapılır, yeni bağ kurulmaz. Anın Rust'tan gelmesi arayüzün "bir dakika ekle" hesabı yapmasını önler.
- **D21 — Süzgeç panelin ayarıdır ve düzenle birlikte saklanır: `kinds` (tür listesi; anahtar yoksa varsayılan) ve `moon`. Türler açılır kapanır bir bölümdedir; özeti seçili sayıyı söyler.** Gerekçe: olay araması panelinin kalıbı (`CheckSet`); C5/C17 otomatik kayıt yolu değişmez.
- **D22 — Arayüz fikstürü bir alan kazanır: `calendar` (iki istek, dört ay ve Rust sabitleri). Alanın tipi ayrı bir dosyadadır (`app/ui/test/calendarFixture.ts`); `fixture.ts` değişmez. E2e taklidi ayı isteğe göre fikstürden seçer.** Gerekçe: arayüz hesap yapmaz, testler gerçek motorun ürettiği veriyi çizer; `fixture.ts`'in içe aktarım listesi paralel planların çatıştığı yerdir.
- **D23 — Uçtan uca ve görsel akışlar yeni dosyalardadır: `app/e2e/calendar.desk.spec.ts` (üç akış) ve `app/e2e/calendar.visual.spec.ts` (üç temel). Yürüten ajan bunları yazar ama koşmaz; kontrolör Task 8'in "[kontrolör]" adımlarında özel bir portta koşar ve üç temel PNG'yi üretir.** Gerekçe: Playwright projeleri dosya adının sonuna bakar (`desk.spec.ts`, `visual.spec.ts`); ortak iki dosya ve yapılandırma değişmez.
- **D24 — REVIEW kaydı `tests/golden/modules/calendar/REVIEW.md`'dir; bu klasörde JSON örneği yoktur. El örnekleri çekirdeğin birim testleridir (sahte gökyüzüyle; türetme test yorumlarında).** Gerekçe: Plan 23 ve 25'in kalıbı. Takvim bellekten aktarılmış tablo taşımaz; Y4'ün "doğrulanmamış tablo" notu gerekmez.
- **D25 — Görev sırası yol haritasının taslağından ayrılır: i18n metinleri ve boş, hata durumları panelle birlikte Task 4'tedir (panel metinsiz commit edilemez); doğum olayları ve süzgeç Task 6'dadır; Task 7 uçtan uca ve görsel akışların kodudur.** Gerekçe: her görev yeşil kapılarla tek commit'tir. **Yol haritası farkı 3.**
- **D26 — Adlandırma: bu planın getirdiği adların hiçbiri yasak diziyi içermez. Türkçe metinlerde dizi şu sözcüklerle de oluşur ve yerlerine ötekiler yazılır: geriye duran pay için "pay", çok dolu için "aşırı dolu", "dakika" sözcüğüne çoğul eki bitişik yazılmaz.** Gerekçe: Y8, v1 spec §10 ve Ek A; yasak harf dizisi düzeyindedir.

## Global Constraints

- Lisans: AGPL-3.0-or-later. Yeni bağımlılık yok: `Cargo.toml`, `Cargo.lock`, `deny.toml`, `app/package.json`, `app/pnpm-lock.yaml`, `app/vite.config.ts` ve `app/playwright.config.ts` değişmez; `cargo deny check` Task 8'de `advisories ok, bans ok, licenses ok, sources ok` verir.
- **Yasaklı ad kuralı (v1 spec §10 "İsim kuralları", ilk madde; ileri modüller spec'i Ek A; yol haritası Y8):** o maddedeki dört ad ve türevleri hiçbir isimde, yorumda, metinde, dosya adında ya da commit mesajında kullanılmaz. Kural harf dizisi düzeyindedir: Türkçe sözcüklerin içinde de geçemez (D26). Her commit'ten önce eklenen satırlar şu komutla taranır ve çıktı boş olmalıdır: `git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'`.
- **Clean-room ve yorum metni yok:** başka ürünün adı, düzeni, metni ya da çıktısı kullanılmaz. Takvim olgu döner: **"uygun gün", "iyi/kötü" hükmü, puan, renkli derecelendirme ve bir olayın ne anlama geldiğini anlatan metin eklenmez.** Tithi ve nakshatra adları çevrilmez ve açıklanmaz.
- **Var olan hesap değişmez:** `crates/jyotish-core/src/search/`, `timeline.rs`, `panchang.rs`, `muhurta/`, `error.rs`, `app/src-tauri/src/search_commands.rs`, `muhurta_commands.rs` ve `commands.rs` yalnızca okunur. Yürüten ajan olay aramasının bulduğu bir anı "düzeltmez"; şüphe duyduğu yeri ilerleme notlarına yazar.
- **Gizlilik (ileri modüller spec'i §14):** (1) Girdi (yer, ay, süzgeç, açık haritanın doğum verisi) ve takvim hiçbir yerde loglanmaz. (2) Hata yalnızca alan adı taşır; yer ve tarih hata parametresine girmez. (3) Takvim saklanmaz ve dışa aktarıma girmez. (4) Hiçbir modül ağ isteği yapmaz. (5) Doğum verisi yalnızca doğum türü seçiliyken komuta gider (D8). (6) Testler ve fikstürler yalnızca deneme adları ve sentetik konumlar kullanır ("Örnek Danışan"); gerçek kişi verisi yoktur.
- **Bütçe testte doğrulanır** (D10): `assert!` ile; `eprintln!` yalnızca ölçümü görünür kılar.
- **El örnekleri (§13, K9):** beklenen değer kuraldan türetilir ve test yorumunda yazılıdır. Bir el örneği motorla uyuşmazsa önce kural yeniden okunur; **beklenen değer motor çıktısına bakarak düzeltilmez.**
- Kapılar: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` (1.100), `cargo test --workspace`, `pnpm --dir app test`, `pnpm --dir app build` her görev sonunda yeşil; `cargo deny check` Task 8'de. `tsc` `strict`. `pnpm build` çıktısında aranan satır `✓ built`'tir (Vite'ın 500 kB parça uyarısı sürer). Tek bir Vitest dosyası `pnpm --dir app exec vitest run ui/…` ile koşulur: yol `app/` dizinine göredir. `cargo test --workspace` çıktısı uzun olduğu için başarısızlık şu sayaçla denetlenir ve `0` yazmalıdır: `cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."`.
- **Yürüten ajan uçtan uca ve görsel testleri koşmaz.** `pnpm --dir app e2e` ve `pnpm --dir app e2e:visual` bu planın görev kapılarında yoktur; Task 7'nin iki dosyası `pnpm --dir app build` içindeki `tsc --noEmit -p e2e` ile tip denetiminden geçer. Kontrolör Task 8'in "[kontrolör]" adımlarını sonradan koşar. Yürüten ajan 1421 portunda sunucu açmaz.
- **Toolchain:** `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"` (oturum başına bir kez; her yeni kabukta yeniden). Node 24+, pnpm 11.8.0. Depo ya da çalışma ağacı kısa bir yolda durmalıdır: Swiss Ephemeris 242 bayttan uzun ephemeris yolunu reddeder.
- **Çalışma ağacı.** Yeni bir git çalışma ağacında `app/node_modules` ve `target/` yoktur: Task 1 Step 1 bağımlılıkları kilit dosyasından kurar (`pnpm --dir app install --frozen-lockfile`; kilit dosyası değişmez) ve ilk `cargo test` her şeyi derler (birkaç dakika). Bütün komutlar çalışma ağacının kökünden verilir.
- **Üretilen dosyalar (C10) elle düzenlenmez** ve yalnızca şu iki komutla güncellenir: `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings` (`app/ui/bindings.ts`) ve `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture` (`app/ui/test/fixtures/delhi.json`). İkisi de Task 3 Step 4'te koşulur.
- **Gerçek uygulama çalıştırılmaz.** Yürüten ajan `pnpm tauri dev` ya da `pnpm --dir app tauri dev` **koşmaz**, üretilen `.app`'i açmaz ve `~/Library/Application Support/io.github.turgaydogusan.jyotishdesk` dizinine dokunmaz. Kapı testlerdir (Y3); gerçek uygulamaya bakan maddeler Task 8'de **[insan]** diye işaretlidir ve sahibine devredilir. Başkasının çalışan `tauri dev`'i (port 1420) ya da Vite sunucusu (1421) durdurulmaz; `pkill -f` kullanılmaz.
- Yalnızca macOS hedeflenir. Yorum metni ve AI paneli bu planın dışındadır (Y7).
- **Tabanlar (`9727c55`):** `jyotish-core` lib 466, `avastha_golden` 5, golden 8, `modules_golden` 17; `atlas` `build` 6, `moment_fixed_lmt` 11, `moment_iana` 13, `search` 8, `shipped` 3; `ephemeris` lib 1, `engine` 7, `env_override` 1, `ffi_smoke` 1, `golden` 1, `houses` 7, `rise_set` 5, `sanity` 15; `store` lib 122 (+1 `#[ignore]`); `jyotish-desk` lib 58, `avastha` 1, `bhava` 1, `bindings` 1, `compute_chart` 1, `modules` 1, `natal_extras` 1, `open_client` 1, `ui_fixture` 2; Vitest `Test Files  97 passed (97)`, `Tests  973 passed (973)`; e2e 33; görsel 15 (yerel, darwin); yayın betiği testleri 14.
- **Sayaç zinciri** (her satır görev bitince; boş hücre "önceki satırla aynı"). Bu plan yazılırken `9727c55`'in bir kopyasında her görev ayrı commit olarak uygulanıp ölçülmüştür. Son iki sütunu yalnızca kontrolör ölçer.

  | Görev | `jyotish-core` lib | `jyotish-desk` lib | `jyotish-desk` `calendar` | Vitest dosya / test | e2e (kontrolör) | görsel (kontrolör) |
  |---|---|---|---|---|---|---|
  | Taban | 466 | 58 | yok | 97 / 973 | 33 | 15 |
  | Task 1 | 472 | | | | | |
  | Task 2 | 479 | | | | | |
  | Task 3 | | 61 | 1 | 98 / 976 | | |
  | Task 4 | | | | 101 / 997 | | |
  | Task 5 | | | | 101 / 1003 | | |
  | Task 6 | | | | 101 / 1007 | | |
  | Task 7 | | | | | 36 | 18 |
  | Task 8 | | | | | | |

  Değişmeyenler: `avastha_golden` 5, golden 8, `modules_golden` 17; `atlas`, `ephemeris` ve `store` sayaçları; `jyotish-desk` `avastha` 1, `bhava` 1, `bindings` 1, `compute_chart` 1, `modules` 1, `natal_extras` 1, `open_client` 1, `ui_fixture` 2; yayın betiği testleri 14. `jyotish-core` lib Task 1'de 6 test (`calendar/grid.rs`), Task 2'de 7 test (`calendar/events.rs`) kazanır. `jyotish-desk` lib Task 3'te 3 test kazanır (`calendar_commands.rs`). Vitest Task 3'te bir dosya ve 3 test (`test/calendarFixture.test.ts`; `api.test.ts`'in tablosunda bir satır), Task 4'te üç dosya ve 21 test (`calendar/model.test.ts` 6, `calendar/useCalendar.test.tsx` 3, `calendar/CalendarPanel.test.tsx` 12), Task 5'te 6 test, Task 6'da 4 test kazanır (`CalendarPanel.test.tsx` sonunda 22 testtir). E2e üç akış (`calendar.desk.spec.ts`), görsel üç temel (`calendar.visual.spec.ts`) kazanır. Paralel bir plan önce birleştiyse tabanlar onun eklediği kadar büyüktür; farklar aynıdır.
- **Taşınan kurallar (Plan 09 C1–C14, Plan 10 C2′, C15–C17), her görevin gereksinimidir; bu planda değişen okumalar açıkça yazılıdır:**
  - **C1** Gizlilik: yukarıdaki "Gizlilik" maddesi; `no-console.test.ts` durur ve izin listesi büyümez.
  - **C2′** Hiçbir IPC komutu arayüzden dosya yolu ya da adres almaz. `month_calendar` tek bir yapı alır ve dizgi bağımsız değişkeni yoktur; `api.test.ts`'in dizgi bağımsız değişken listesi değişmez, yalnızca sarmalayıcı tablosu bir satır kazanır.
  - **C3** Var olan göçler ve şema sürümleri değişmez; `crates/store` bu planda hiç değişmez (Y6). `CalcSettings`, `ViewState` ve `AppPrefs` tipleri değişmez.
  - **C4** Arayüz astrolojik hesap ve tarih hesabı yapmaz: günler, hafta günleri, "bugün", önceki ve sonraki ay, olgular, olaylar ve gün panchangına verilecek an Rust'tan gelir. Günü hafta gününün sütununa koymak, kimliği adına çevirmek, JD'yi verilen ofsetle saat olarak yazmak (`formatJdClock`) ve klavye odağını komşu hücreye taşımak gösterimdir.
  - **C5/C17** Otomatik kayıtlar değişmez; panelin `place`, `custom`, `kinds` ve `moon` ayarları düzenle birlikte var olan yoldan kaydedilir. **C6** Düzen ayrıştırıcıları (`parseLayoutState`, Rust `check_panel_layout`) değişmez; bir panel türü kataloğa eklenir.
  - **C7, C16** Uygulama kısayolu eklenmez; `shortcuts.ts` değişmez. Izgaranın tuşları yalnızca ızgara odaktayken çalışır. **C8** Yeni pencere yoktur.
  - **C9** Renk yalnızca token'lardan gelir (hex yok) ve tek başına anlam taşımaz: bugün sözle ve çerçeveyle, seçili gün `aria-selected` ve zeminle, gün doğumu olmayan gün `*` ve sözle yazılır.
  - **C10** Üretilen dosyalar elle düzenlenmez (yukarıda). **C11** Görsel PNG temelleri yereldir; var olan on beş temel değişmez, üç yeni temeli kontrolör üretir (D23).
  - **C12** Her yeni i18n anahtarı üç dilde vardır (`i18n.test.ts` anahtar kümelerini ve yer tutucuları karşılaştırır). Fransızca metinler tipografik kesme işareti (’) kullanır; panel testi `calendar` bloğunda düz kesme işaretinin yokluğunu sabitler.
  - **C13** Yasaklı ad taraması her commit'te; depo dışındaki ilerleme notları ve `.superpowers/` hiçbir zaman `git add` edilmez.
  - **C14** Plan 07'nin gün doğumu toleransı ve `compute_panchang` değişmez (`panchang.rs`'e dokunulmaz). **C15** `alert`, `confirm`, `prompt`, `window.print()` ve `window.open()` çağrılmaz.
- **Erişilebilirlik:** WCAG AA. Izgara `role="grid"`'dir, adını başlığından (`caption`) ve açıklamasını tuş ipucundan alır; sütun başlıkları `scope="col"` ve tam gün adını taşır; hücrede kısaltmaların tam adı ekran okuyucuya ayrıca verilir; tek sekme durağı vardır ve odak halkası görünür; gün ayrıntısı adlı bir bölgedir ve başlığı duyurulur; hata `role="alert"`'tir; odak hiçbir durumda gövdeye düşmez; her işlem klavyeyle yapılır.
- Commit mesajları İngilizcedir ve tam olarak şu satırla biter: `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` (komutlarda ikinci `-m` olarak verilir; yürüten araç kendi imza satırını eklemez). Her görev tek commit'tir (gerekirse inceleme düzeltmesi ayrı commit). `git push`, `git tag`, yıkıcı git komutu ve `.superpowers/` commit'i yoktur.
- Kod blokları `9727c55` ağacına göre yazılmış ve bu plan yazılırken deponun bir kopyasında koşulmuştur (Self-Review madde 5). Yeni dosyalar tam içerikle verilir ("**Yeni dosya:**"). Değişen dosyalar `diff` bloğuyla verilir: blok bir dosyaya konup çalışma ağacının kökünde `git apply <dosya>` ile ya da elle uygulanır; bağlam tutmazsa önceki görevin adımı denetlenir, elle uydurma yapılmaz. Tek bir yerde var olan bir dosya baştan yazılır ("**Dosyanın yeni tam içeriği:**": dosyanın eski içeriği silinir). Yeni bir Rust dosyasının testleri ve uygulaması ayrı adımlarda verilir: önce dosya yalnızca `#[cfg(test)] mod tests` bloğuyla oluşturulur ("şimdilik yalnızca test bloğu"), sonra uygulama **o bloğun üstüne**, arada bir boş satırla eklenir; `cargo fmt --all --check` birleştirmeyi doğrular. Kod blokları dört ters tırnakla çevrilidir; çevreleyen satırlar dosyaya yazılmaz.

## Paylaşılan dosyalar

Plan 26 (geçiş araçları) ve Plan 27 (Varshaphala I) başka çalışma ağaçlarında aynı anda yürür. Aşağıdaki liste bu planın dokunduğu ve öteki planların da dokunması beklenen **bütün** dosyalardır; her satır değişikliğin boyutunu, yerini ve birleştirme çatışmasında ne yapılacağını söyler. Bu listenin dışındaki her dosya ya yenidir ya da yalnızca bu plana aittir.

| Dosya | Görev | Değişiklik | Çatışmada |
|---|---|---|---|
| `crates/jyotish-core/src/lib.rs` | 1 | Tek satır: `pub mod calendar;` (`pub mod avastha;`'dan sonra, abecesel yerinde). `pub use` bloğuna dokunulmaz. | İki tarafın satırları korunur. Plan 26 kendi bloğunu dosyanın sonuna koyar; bu satır abecesel listededir. |
| `app/src-tauri/src/lib.rs` | 3 | İki satır: `pub mod calendar_commands;` (`bhava_commands`'tan sonra) ve komut listesinde `muhurta_commands::muhurta_search,`'ten sonra `calendar_commands::month_calendar,`. | İki tarafın satırları korunur (`pub mod` satırları abecesel sırayla; `rustfmt` bunu ister). Sonra bağlamalar yeniden üretilir. |
| `app/src-tauri/tests/ui_fixture.rs` | 3 | Kendi içinde kapalı 72 satırlık blok (`let calendar = { … };`, kendi `use` satırlarıyla), `// Bhava tablosu ve Sudarshana chakra` yorumundan hemen önce; `json!` içinde `"search": {`'ten önce tek satır: `"calendar": calendar,`. Dosyanın başındaki `use` listelerine dokunulmaz. | İki tarafın blokları ve anahtarları korunur. Sonra fikstür yeniden üretilir. |
| `app/ui/bindings.ts` | 3 | Üretilir (96 satır ekleme). | Elle çözülmez: bir taraf alınır, sonra `UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings`. |
| `app/ui/test/fixtures/delhi.json` | 3 | Üretilir (tek satırlık dosya). | Elle çözülmez: bir taraf alınır, sonra `UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture`. |
| `app/ui/api.ts` | 3 | `muhurtaSearch`'ten sonra bir sarmalayıcı (dört satır). İçe aktarım listesine dokunulmaz (bağımsız değişkenin tipi `Parameters<typeof commands.monthCalendar>[0]` ile okunur). | İki tarafın sarmalayıcıları korunur. |
| `app/ui/api.test.ts` | 3 | İki satır: taklit listesinde ve sarmalayıcı tablosunda `monthCalendar` (`muhurtaSearch`'ten sonra). | İki tarafın satırları korunur. |
| `app/ui/layout/model.ts` | 4 | `PANEL_KINDS`'e `"muhurta_search"`'ten sonra `"calendar"`; `MODULE_PANELS`'e `muhurta_search`'ten sonra iki satır. | İki tarafın türleri korunur. |
| `app/ui/layout/model.test.ts` | 4 | Katalog listesine bir satır; **tek satırlık** `MODULE_PANELS` beklentisine `"calendar"` (`"muhurta_search"`'ten sonra). | Tek satırlık beklenti kesin çatışır: sonuç, `model.ts`'teki `MODULE_PANELS` sırasının aynısıdır (iki tarafın türleri birlikte). |
| `app/ui/panels/registry.ts` | 4 | Bir içe aktarım (`BirthInfoPanel`'den sonra) ve `muhurta_search`'ten sonra iki satır. | İki tarafın satırları korunur. |
| `app/ui/i18n/tr.ts`, `en.ts`, `fr.ts` | 4, 5, 6 | Üç ekleme: `panel` bloğunda `muhurta_search`'ten sonra `calendar`; `prashna` bloğundan önce yeni `calendar` bloğu (Task 4'te 41, Task 5'te 11, Task 6'da 3 satır); `errors` bloğunda `invalid_search_input`'tan sonra `invalid_calendar_input`. Var olan hiçbir satır değişmez. | İki tarafın anahtarları korunur; `i18n.test.ts` üç dilin eşitliğini denetler. |
| `app/e2e/ipc.ts` | 7 | `load_muhurta_filters` taklidinden önce on satır (`month_calendar`). | İki tarafın taklitleri korunur. |
| `tests/golden/modules/README.md` | 8 | Son cümleden önce bir paragraf (yedi satır). | İki tarafın paragrafları korunur. |

Bu belge teslim edilirken `main` `cba3643`'e ilerlemişti: aradaki iki commit Plan 26 ve Plan 27'nin plan belgeleridir (`docs/` altında iki dosya); kod, `diff` bağlamları ve sayaçlar değişmez. O iki planla karşılaştırıldı: katalogda Plan 26 `event_search`'ten, Plan 27 `avastha`'dan sonra ekler, bu plan `muhurta_search`'ten sonra; i18n'de üçü ayrı bloklar ekler; çekirdek `lib.rs`'te Plan 26 dosyanın sonuna, bu plan abecesel listeye yazar; Plan 26'nın `app/src-tauri/src/lib.rs`'teki `.constant(…)` satırı bu planın satırlarıyla aynı yerde değildir. Metin çatışması beklenen yerler: `model.test.ts`'in tek satırlık beklentisi, `app/src-tauri/src/lib.rs`'in `pub mod` listesi (komşu satırlar; hepsi korunur) ve üretilen iki dosya. Birleştirmeden sonra üretilen iki dosya iki komutla yeniden üretilir ve bütün kapılar yeniden koşulur (birleştirenin işidir, bu planın değil).

**Okunan ama değiştirilmeyen ortak yüzey (anlam çatışması riski).** Metin çatışması vermez ama öteki plan bu adları değiştirirse birleştirmeden sonra derleme ya da test düşer; birleştiren şuralara bakar:

- `SkyEvent`, `SkyEventKind`, `EventSpec` (`search/events.rs`): Plan 26 `SkyEvent`'e alan eklerse (nakshatra lordu) bu planın kodu derlenmeyi sürdürür: testler olayı alan alan kurmaz (gerçek bir olayın kopyasını değiştirir), arayüz testi fikstürdeki olaydan kurar. `EventSpec` alan kazanırsa `calendar/events.rs`'teki tek yapı kuruluşu (`CalendarFilter::event_spec`) ve `app/src-tauri/tests/calendar.rs`'teki `EventSearchSpec` kuruluşu güncellenir.
- `muhurta_commands::{day_start_jd, offset_at, today, MuhurtaPlace, J2000_JD}` ve `search_commands::SkyEventView`: imzaları değişirse `calendar_commands.rs` güncellenir.
- i18n anahtarları `eventSearch.kind.*`, `eventSearch.event.*`, `panchang.weekday.*`, `muhurta.place.*`, `badge.LowPrecision`, `badgeHint.LowPrecision`, `graha.*`, `grahaName.*`, `sign.*`: adları değişirse `app/ui/calendar/` altındaki okuyan satırlar güncellenir (panel testleri metni sabitler ve düşerek haber verir).
- Arayüz yardımcıları `modules/useModule.ts`, `modules/muhurta.ts` (`VARAS`, `muhurtaPlace`, `toggleMember`), `panels/MuhurtaPlace.tsx`, `panels/format.ts`, `panels/labels.ts`, `search/CheckSet.tsx`, `transit/store.tsx`.

Bilerek dokunulmayan ortak dosyalar: `crates/jyotish-core/src/error.rs`, `crates/jyotish-core/src/search/*`, `crates/jyotish-core/src/timeline.rs`, `crates/jyotish-core/tests/modules_golden.rs`, `app/src-tauri/src/commands.rs`, `app/src-tauri/src/search_commands.rs`, `app/src-tauri/src/muhurta_commands.rs`, `app/src-tauri/src/module_commands.rs`, `app/src-tauri/tests/modules.rs`, `app/ui/test/fixture.ts`, `app/ui/panels/panels.test.tsx`, `app/ui/panels/panels.css`, `app/ui/panels/labels.ts`, `app/ui/i18n/i18n.test.ts`, `app/e2e/desk.spec.ts`, `app/e2e/visual.spec.ts`, `app/playwright.config.ts`.

## Sonraki işlere verilen API

Bu planın ürettiği ve sonraki işlerin tüketeceği adlar. Rust adları `jyotish_core::calendar` yolundadır (D1); imzalar görevlerde tam olarak verilir.

| API | Tüketen | Not |
|---|---|---|
| `DaySpan`, `DayFacts`, `day_facts`, `sunrise_in`, `fallback_moment`, `check_spans`, `calendar_slices`, `CALENDAR_MAX_DAYS`, `CALENDAR_SLICE_DAYS`, `MAX_DAY_SPAN_DAYS` | Plan 30 (tutulmaların takvimde gösterimi), yıl görünümü | Saf; gün sınırlarını çağıran verir |
| `CalendarFilter`, `CalendarSpec`, `CalendarDay`, `CalendarSlice`, `assign_events`, `calendar_slice`, `calendar_month`, `CALENDAR_KINDS`, `CALENDAR_DEFAULT_KINDS`, `CALENDAR_MAX_EVENTS`, `CALENDAR_METHOD` | Plan 26 sonrası (Sankranti işareti, nakshatra lordu), Plan 30 (yeni olay türleri `CALENDAR_KINDS`'e eklenir) | Olaylar olay aramasından |
| `calendar_commands::{month_calendar, month_calendar_blocking, CalendarRequest, CalendarView, CalendarDayView, YearMonth, CALENDAR_YEARS}` | AI paneli, baskı | Hata kodu `invalid_calendar_input` |
| Arayüz: `calendar/model.ts`, `calendar/useCalendar.ts`, `calendar/CalendarGrid.tsx`, `calendar/DayDetail.tsx`, `calendar/EventFilter.tsx`, `calendar/CalendarPanel.tsx`; fikstürde `calendar` (`test/calendarFixture.ts`) | sonraki paneller | i18n: `calendar.*`, `panel.calendar`, `errors.invalid_calendar_input` |

## Dosya Yapısı (bu plan)

```
crates/jyotish-core/
  src/calendar/mod.rs              modül kaydı, CALENDAR_METHOD (yeni)
  src/calendar/grid.rs             DaySpan, DayFacts, gün doğumu olguları, dilimler (yeni)
  src/calendar/events.rs           CalendarFilter, dağıtım, sınır, dilim ve ay toplayıcısı (yeni)
  src/lib.rs                       pub mod calendar;
tests/golden/modules/
  calendar/REVIEW.md               okuma kuralları, sahibin kararını bekleyen noktalar (yeni)
  README.md                        bir paragraf
app/src-tauri/
  src/calendar_commands.rs         month_calendar (yeni)
  src/lib.rs                       modül ve komut kaydı
  tests/calendar.rs                gerçek motorla çapraz denetim, hatalar, bütçe (yeni)
  tests/ui_fixture.rs              fikstüre calendar
app/ui/
  bindings.ts                      üretilir
  test/fixtures/delhi.json         üretilir
  test/calendarFixture.ts (+test)  fikstürün calendar alanının tipi (yeni)
  api.ts, api.test.ts              bir sarmalayıcı
  calendar/model.ts (+test)        sabitler, hafta satırları, süzgeç, olay yazıları (yeni)
  calendar/useCalendar.ts (+test)  ay kancası (yeni)
  calendar/CalendarGrid.tsx        ay ızgarası, klavye (yeni)
  calendar/DayDetail.tsx           gün ayrıntısı (yeni)
  calendar/EventFilter.tsx         olay süzgeci (yeni)
  calendar/CalendarPanel.tsx (+test)  panel, ay seçimi (yeni)
  calendar/calendar.css            (yeni)
  panels/registry.ts
  layout/model.ts (+test)          calendar türü, MODULE_PANELS
  i18n/{tr,en,fr}.ts               panel.calendar, calendar, errors.invalid_calendar_input
app/e2e/
  ipc.ts                           month_calendar taklidi
  calendar.desk.spec.ts            üç akış (yeni; kontrolör koşar)
  calendar.visual.spec.ts          üç görsel temel (yeni; kontrolör koşar)
```

### Kapsam

| Yol haritası maddesi | Yer |
|---|---|
| §4.11 Ay ızgarası: her günün hücresinde gün doğumundaki tithi ve Ay nakshatrası | Task 1 (D3), Task 4 (D18) |
| §4.11 O güne düşen olaylar: burç ve nakshatra girişleri, dönüşler, yeni ay, dolunay | Task 2 (D6, D7) |
| §4.11 Harita açıksa doğum boylamına gelişler | Task 2, Task 6 (D8) |
| §4.11 Zor yer: gün sınırı (takvim günü ile panchang günü) | Task 1, Task 3 (D3, D4); REVIEW 1–2 |
| §4.11 Zor yer: kutup dairesinde gün doğumu olmayan günler | Task 1, Task 3, Task 4 (D5) |
| §4.11 Zor yer: ay başına olay sayısının sınırı | Task 2 (D9) |
| §4.11 Zor yer: ızgaranın klavye ve ekran okuyucu erişimi | Task 5 (D19) |
| §4.11 Zor yer: üç dilde hafta başlangıcı | Task 4 (D14) |
| §5 (1) Tabanlar ve ay ızgarası çekirdeği (günler, gün doğumu olguları) | Task 1 |
| §5 (2) Olayların günlere dağıtımı ve sınırlar | Task 2 |
| §5 (3) Komut ve dilimli yükleme | Task 3 (D10, D11) |
| §5 (4) Takvim paneli: ızgara ve hücre içeriği | Task 4 |
| §5 (5) Klavye gezinme, gün ayrıntısı, gün panchangına geçiş | Task 5 (D19, D20) |
| §5 (6) Harita açıkken doğuma göre olaylar ve süzgeç | Task 6 (D8, D21) |
| §5 (7) i18n (hafta başlangıcı, ay adları), boş ve hata durumları | Task 4 (D25); uçtan uca ve görsel akışlar Task 7 |
| §5 (8) Kapılar ve elle denetim; sahibin denetimi | Task 8 |
| §8 Tek ephemeris iş parçacığı: dilimleme ve eski yanıtın düşürülmesi | Task 3 (dilimler, bütçe), Task 4 (`useCalendar.test.tsx`) |
| §5 "Bağımlı olduğu: Plan 26" | Kullanılmaz (D2) |

---

### Task 1: Çekirdek: `calendar` modülü, takvim günleri ve gün doğumu olguları

Yol haritası §4.11 ve §5 Plan 29 görev (1); K3, K5; D1, D3–D5. Var olan hiçbir hesabın davranışı değişmez: `panchang.rs` (`elongation`, `tithi_no`, `paksha`), `timeline.rs` (`lon_of`, `sidereal`, `TIMELINE_RESOLUTION_DAYS`) ve `zodiac.rs` (`nakshatra_pada`) yalnızca okunur.

**Files:**
- Create: `crates/jyotish-core/src/calendar/mod.rs`, `crates/jyotish-core/src/calendar/grid.rs`
- Modify: `crates/jyotish-core/src/lib.rs` (tek satır; paylaşılan dosya)
- Test: `grid.rs` `mod tests` (6)

**Interfaces:**
- Consumes (hepsi var): `ephemeris::{Body, Ephemeris, RiseSet}`; `crate::panchang::{elongation, paksha, tithi_no, Paksha}`; `crate::timeline::{lon_of, sidereal, TIMELINE_RESOLUTION_DAYS}` (`pub(crate)`); `crate::zodiac::nakshatra_pada`; `crate::settings::{CalcSettings, JD_UT_RANGE}`; testlerde `crate::fake::FakeEph`.
- Produces (`jyotish_core::calendar` yolunda):
  - `CALENDAR_METHOD: &str = "calendar.month_v1"`.
  - `DaySpan { start_jd_ut: f64, end_jd_ut: f64 }` (`contains(jd_ut) -> bool`), `DayFacts { jd_ut: f64, sunrise_jd_ut: Option<f64>, tithi: u8, paksha: Paksha, nakshatra: u8 }`.
  - `CALENDAR_MAX_DAYS: usize = 31`, `CALENDAR_SLICE_DAYS: usize = 7`, `MAX_DAY_SPAN_DAYS: f64 = 1.25`.
  - `check_spans(spans: &[DaySpan]) -> Result<(), CoreError>`, `calendar_slices(n: usize) -> Vec<(usize, usize)>`, `sunrise_in(eph, settings, geo_lat, geo_lon, span) -> Result<Option<f64>, CoreError>`, `fallback_moment(span, geo_lon) -> f64`, `day_facts(eph, settings, geo_lat, geo_lon, span) -> Result<DayFacts, CoreError>`.
  - Hatalar `CoreError::InvalidSearchInput("range" | "place")`'tir (yeni hata türü yok, D13).

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
```
Expected: iki `git status --short` de boş (bağımlılık kurulumu kilit dosyasını değiştirmez; `app/node_modules` izlenmez); `rustc 1.100.0` ya da yenisi; `jyotish-core` lib `466 passed`, `avastha_golden` `5 passed`, golden `8 passed`, `modules_golden` `17 passed`; `atlas` `build` 6, `moment_fixed_lmt` 11, `moment_iana` 13, `search` 8, `shipped` 3; `store` lib `122 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `58 passed`, `avastha` 1, `bhava` 1, `bindings` 1, `compute_chart` 1, `modules` 1, `natal_extras` 1, `open_client` 1, `ui_fixture` 2; Vitest `97 passed (97)`, `973 passed (973)`. `git log` çıktısı `9727c55` değilse (`main` ilerlediyse ya da paralel bir plan birleştiyse) ölçülen değerler ilerleme notlarına yazılır ve sayaç zinciri o tabandan, aynı farklarla yürütülür. Okunan commit Task 8'de `BASE` olarak kullanılır: not edilir. Uçtan uca ve görsel testler koşulmaz (Global Constraints).

- [ ] **Step 2: Testler: gün doğumu olguları, gün sınırları, gün doğumu olmayan gün, dilimler; modül kaydı**

Önce modül kaydı (testlerin derlenebilmesi için), sonra test gövdesi. Sahte gökyüzü (`fake.rs`): sidereal Güneş J2000'de 346° (0,98°/gün), Ay 71° (13,2°/gün); gün doğumu her gün JD'nin 0,67 kesrindedir. Beklenen her değerin türetmesi test yorumundadır.

````diff
diff --git a/crates/jyotish-core/src/lib.rs b/crates/jyotish-core/src/lib.rs
index 612a167..2549f21 100644
--- a/crates/jyotish-core/src/lib.rs
+++ b/crates/jyotish-core/src/lib.rs
@@ -25,6 +25,7 @@ pub use bhava::{
 
 pub mod ashtakavarga;
 pub mod avastha;
+pub mod calendar;
 mod chart;
 mod combustion;
 pub mod dasha;
````

**Yeni dosya:** `crates/jyotish-core/src/calendar/mod.rs`

````rust
//! Geçiş takvimi (yol haritası §4.11, Plan 29): bir ayın takvim günleri, her günün gün
//! doğumundaki tithisi ve Ay nakshatrası ile o güne düşen gök olayları.
//!
//! - `grid`: takvim günleri (`DaySpan`), günün gün doğumu olguları (`DayFacts`) ve dilimler.
//! - `events`: takvimin olay süzgeci (`CalendarFilter`), olayların günlere dağıtımı ve sınır.
//!
//! Gün, yerin takvim günüdür: sınırlarını (yerel gece yarısı) çağıran verir, çünkü saat dilimi
//! bu kasada çözülmez. Olgular o günün içindeki gün doğumunda okunur; gün doğumu yoksa (kutup
//! gecesi ya da gündüzü) 06:00 yerel ortalama saatte. Olaylar olay aramasının çekirdeğinden
//! (`search::events`) gelir: yeni astronomi yoktur. Hepsi olgudur: yorum, puan ya da "uygun gün"
//! hükmü yoktur. Sonuç saklanmaz (K1). Adlar `jyotish_core` kökünden yeniden dışa aktarılmaz;
//! tüketiciler `jyotish_core::calendar::…` yolunu kullanır.

mod grid;

pub use grid::{
    calendar_slices, check_spans, day_facts, fallback_moment, sunrise_in, DayFacts, DaySpan,
    CALENDAR_MAX_DAYS, CALENDAR_SLICE_DAYS, MAX_DAY_SPAN_DAYS,
};

/// Yöntem kimliği (K7): takvim günü yerel gece yarısından gece yarısınadır; olgular günün
/// içindeki gün doğumunda okunur.
pub const CALENDAR_METHOD: &str = "calendar.month_v1";
````

**Yeni dosya:** `crates/jyotish-core/src/calendar/grid.rs` (şimdilik yalnızca test bloğu)

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;

    /// 2000-01-01 00:00 UT.
    const MIDNIGHT: f64 = 2_451_544.5;
    const SECOND: f64 = 1.0 / 86_400.0;

    /// `n` ardışık UT günü.
    fn days(n: usize) -> Vec<DaySpan> {
        (0..n)
            .map(|i| DaySpan {
                start_jd_ut: MIDNIGHT + i as f64,
                end_jd_ut: MIDNIGHT + (i + 1) as f64,
            })
            .collect()
    }

    fn facts(eph: &mut FakeEph, span: DaySpan, lon: f64) -> DayFacts {
        day_facts(eph, &CalcSettings::default(), 28.6, lon, span).unwrap()
    }

    fn field(r: Result<(), CoreError>) -> &'static str {
        match r {
            Err(CoreError::InvalidSearchInput(f)) => f,
            other => panic!("expected an invalid search input, got {other:?}"),
        }
    }

    /// Sahte ephemeris: gün doğumu her gün `n + 0,67` anındadır (04:05 UT); sidereal Güneş
    /// J2000'de 346° (0,98°/gün), Ay 71° (13,2°/gün).
    #[test]
    fn the_facts_of_a_day_are_read_at_the_sunrise_inside_it() {
        let mut eph = FakeEph::default();
        let spans = days(3);
        // 1. gün: doğuş 2451544,67 (J2000'den 0,33 gün önce). Güneş 346 − 0,3234 = 345,6766°;
        // Ay 71 − 4,356 = 66,644°. Uzanım 80,9674°: 72°–84° dilimi 7. tithidir (Shukla).
        // Ay 66,644° < 66°40′ (= 66,667°): 5. nakshatranın son yayındadır.
        let first = facts(&mut eph, spans[0], 77.2);
        assert_eq!(first.sunrise_jd_ut, Some(2_451_544.67));
        assert_eq!(first.jd_ut, 2_451_544.67);
        assert_eq!(
            (first.tithi, first.paksha, first.nakshatra),
            (7, Paksha::Shukla, 5)
        );
        // 2. gün: doğuş J2000'den 0,67 gün sonra. Güneş 346,6566°, Ay 79,844°. Uzanım
        // 93,1874°: 84°–96° dilimi 8. tithidir. Ay 66°40′–80° arasında: 6. nakshatra.
        let second = facts(&mut eph, spans[1], 77.2);
        assert_eq!(second.sunrise_jd_ut, Some(2_451_545.67));
        assert_eq!((second.tithi, second.nakshatra), (8, 6));
        // 3. gün: Güneş 347,6366°, Ay 93,044°. Uzanım 105,4074°: 96°–108° dilimi 9. tithidir.
        // Ay 80°–93°20′ dilimini 0,29 gün önce geçmiştir: 7. nakshatra.
        let third = facts(&mut eph, spans[2], 77.2);
        assert_eq!((third.tithi, third.nakshatra), (9, 7));
        // Disk tanımı hesap ayarından gelir.
        assert_eq!(
            eph.last_sun_disc,
            Some(CalcSettings::default().sunrise_def.sun_disc())
        );
    }

    #[test]
    fn the_waning_half_starts_after_the_fifteenth_tithi() {
        // Ay'ı Güneş'in 200° önüne koy: uzanım 200° − (0,33 × 12,22) ≈ 195,97°; 192°–204°
        // dilimi 17. tithidir (Krishna 2).
        let mut eph = FakeEph::default().with(Body::Moon, 210.0, 13.2);
        let got = facts(&mut eph, days(1)[0], 77.2);
        assert_eq!((got.tithi, got.paksha), (17, Paksha::Krishna));
    }

    #[test]
    fn a_sunrise_belongs_to_the_day_that_contains_it_and_midnight_opens_the_day() {
        let settings = CalcSettings::default();
        let span = days(1)[0];
        // Tam gece yarısındaki doğuş o güne aittir; günün bitişindeki doğuş sonraki güne.
        for (rise, want) in [
            (span.start_jd_ut, Some(span.start_jd_ut)),
            (span.end_jd_ut - SECOND, Some(span.end_jd_ut - SECOND)),
            (span.end_jd_ut, None),
            (span.start_jd_ut - SECOND, None),
        ] {
            let mut eph = FakeEph {
                sunrises: Some(vec![rise]),
                ..FakeEph::default()
            };
            assert_eq!(
                sunrise_in(&mut eph, &settings, 69.6, 18.9, span).unwrap(),
                want
            );
        }
        // İki doğuşlu günde ilki alınır.
        let mut eph = FakeEph {
            sunrises: Some(vec![span.start_jd_ut + 0.01, span.start_jd_ut + 0.99]),
            ..FakeEph::default()
        };
        assert_eq!(
            sunrise_in(&mut eph, &settings, 69.6, 18.9, span).unwrap(),
            Some(span.start_jd_ut + 0.01)
        );
    }

    #[test]
    fn a_day_without_a_sunrise_is_read_at_six_local_mean_time() {
        let mut eph = FakeEph {
            polar: true,
            ..FakeEph::default()
        };
        let span = days(1)[0];
        // Boylam 30° D: yerel ortalama saat UT + 2 saattir; 06:00 yerel 04:00 UT'dir.
        let got = facts(&mut eph, span, 30.0);
        assert_eq!(got.sunrise_jd_ut, None);
        assert!((got.jd_ut - (span.start_jd_ut + 4.0 / 24.0)).abs() < SECOND);
        // Güneş 346 − 0,98 / 3 = 345,6733°; Ay 71 − 4,4 = 66,6°: uzanım 80,93° (7. tithi),
        // 5. nakshatra.
        assert_eq!((got.tithi, got.nakshatra), (7, 5));
        // Boylam 0°: 06:00 UT. Batı boylamında (−90°: UT − 6 saat) 06:00 yerel 12:00 UT'dir.
        assert!((fallback_moment(span, 0.0) - (span.start_jd_ut + 0.25)).abs() < SECOND);
        assert!((fallback_moment(span, -90.0) - (span.start_jd_ut + 0.5)).abs() < SECOND);
        // An her durumda günün içindedir (duvar saati yerel ortalama saatten çok uzak olsa da).
        for lon in [-180.0, -135.0, 135.0, 180.0] {
            assert!(span.contains(fallback_moment(span, lon)), "{lon}");
        }
    }

    #[test]
    fn spans_are_one_to_thirty_one_gapless_days_of_plausible_length() {
        assert!(check_spans(&days(1)).is_ok());
        assert!(check_spans(&days(31)).is_ok());
        assert_eq!(field(check_spans(&[])), "range");
        assert_eq!(field(check_spans(&days(32))), "range");
        // 23 ve 25 saatlik günler (yaz saati geçişi) geçerlidir; 31 saatlik gün değildir.
        let day = |start: f64, hours: f64| DaySpan {
            start_jd_ut: start,
            end_jd_ut: start + hours / 24.0,
        };
        assert!(check_spans(&[day(MIDNIGHT, 23.0)]).is_ok());
        assert!(check_spans(&[day(MIDNIGHT, 25.0)]).is_ok());
        assert_eq!(field(check_spans(&[day(MIDNIGHT, 31.0)])), "range");
        assert_eq!(field(check_spans(&[day(MIDNIGHT, 0.0)])), "range");
        assert_eq!(field(check_spans(&[day(MIDNIGHT, -1.0)])), "range");
        // Günler boşluksuz ardışıktır.
        let mut gap = days(3);
        gap[2].start_jd_ut += SECOND;
        assert_eq!(field(check_spans(&gap)), "range");
        // Hesap aralığının dışı ve sonlu olmayan uçlar.
        assert_eq!(field(check_spans(&[day(600_000.0, 24.0)])), "range");
        assert_eq!(field(check_spans(&[day(f64::NAN, 24.0)])), "range");
        assert_eq!(field(check_spans(&[day(f64::INFINITY, 24.0)])), "range");
        // Yer denetimi alan adıyla döner.
        let bad = day_facts(
            &mut FakeEph::default(),
            &CalcSettings::default(),
            91.0,
            0.0,
            days(1)[0],
        );
        assert_eq!(field(bad.map(|_| ())), "place");
        assert_eq!(field(check_place(91.0, 0.0)), "place");
        assert_eq!(field(check_place(0.0, f64::NAN)), "place");
        assert!(check_place(-90.0, 180.0).is_ok());
    }

    #[test]
    fn a_month_is_cut_into_slices_of_at_most_seven_days() {
        assert_eq!(
            calendar_slices(31),
            [(0, 7), (7, 14), (14, 21), (21, 28), (28, 31)]
        );
        assert_eq!(calendar_slices(28), [(0, 7), (7, 14), (14, 21), (21, 28)]);
        assert_eq!(calendar_slices(1), [(0, 1)]);
        assert!(calendar_slices(0).is_empty());
    }
}
````

- [ ] **Step 3: Testlerin derlenmediğini gör**

Run: `cargo test -p jyotish-core --lib calendar 2>&1 | grep -E "^error" | sort | uniq -c | head`
Expected: derleme hatası: `mod.rs`'in `pub use grid::{…}` satırındaki adlar çözülemez (`error[E0432]: unresolved imports`) ve test bloğunun `use super::*` ile aradığı adlar bulunamaz (`E0425`, `E0433`). Hiçbir test koşmaz.

- [ ] **Step 4: Takvim günleri ve gün doğumu olguları**

**`crates/jyotish-core/src/calendar/grid.rs`: uygulama, test bloğunun üstüne eklenir (arada bir boş satır):**

````rust
//! Takvim günleri ve günün gün doğumu olguları (Plan 29 D3–D5).
//!
//! Hücre takvim günüdür (yerel gece yarısından gece yarısına); panchang günü ise gün doğumunda
//! başlar. İkisi şöyle bağlanır: hücrenin tithisi ve Ay nakshatrası, o takvim gününün içindeki
//! gün doğumunda geçerli olan değerdir (o gün başlayan panchang gününün ilk değeri). Gün içinde
//! değişen tithi hücrede yazılmaz; gün panchangı paneli bütün değişimleri saatleriyle verir.

use ephemeris::{Body, Ephemeris, RiseSet};
use serde::{Deserialize, Serialize};

use crate::panchang::{elongation, paksha, tithi_no, Paksha};
use crate::settings::{CalcSettings, JD_UT_RANGE};
use crate::timeline::{lon_of, sidereal, TIMELINE_RESOLUTION_DAYS};
use crate::zodiac::nakshatra_pada;
use crate::CoreError;

/// Bir istekteki en çok takvim günü: bir ay.
pub const CALENDAR_MAX_DAYS: usize = 31;
/// Bir motor işinde hesaplanan en çok gün: çağıran dilimler arasında motor iş parçacığını
/// bırakır.
pub const CALENDAR_SLICE_DAYS: usize = 7;
/// Bir takvim gününün en uzun süresi, gün: 24 saat ve saat dilimi geçişi için 6 saat pay.
pub const MAX_DAY_SPAN_DAYS: f64 = 1.25;
/// Gün doğumu yoksa olguların okunduğu yerel ortalama saat: 06:00 = 0,25 gün (v1 kuralı).
const FALLBACK_DAY_START: f64 = 0.25;

/// Bir takvim günü: `[start_jd_ut, end_jd_ut)`. Uçlar yerin duvar saatiyle gece yarılarıdır.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DaySpan {
    pub start_jd_ut: f64,
    pub end_jd_ut: f64,
}

impl DaySpan {
    pub fn contains(&self, jd_ut: f64) -> bool {
        self.start_jd_ut <= jd_ut && jd_ut < self.end_jd_ut
    }
}

/// Bir takvim gününün gün doğumu olguları.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct DayFacts {
    /// Olguların okunduğu an: günün içindeki gün doğumu; yoksa 06:00 yerel ortalama saat.
    pub jd_ut: f64,
    /// Günün içindeki (ilk) gün doğumu; o takvim gününde Güneş doğmuyorsa `None`.
    pub sunrise_jd_ut: Option<f64>,
    /// 1..=30.
    pub tithi: u8,
    pub paksha: Paksha,
    /// Ay'ın nakshatrası, 1..=27 (sidereal boylamdan, her zodyak kipinde).
    pub nakshatra: u8,
}

fn invalid(field: &'static str) -> CoreError {
    CoreError::InvalidSearchInput(field)
}

pub(crate) fn check_place(geo_lat: f64, geo_lon: f64) -> Result<(), CoreError> {
    if (-90.0..=90.0).contains(&geo_lat) && (-180.0..=180.0).contains(&geo_lon) {
        Ok(())
    } else {
        Err(invalid("place"))
    }
}

/// Gün listesinin denetimi: 1–31 gün, her gün hesap aralığında ve en çok `MAX_DAY_SPAN_DAYS`
/// uzunluğunda, günler boşluksuz ardışık. Hata yalnızca alan adını taşır (`range`).
pub fn check_spans(spans: &[DaySpan]) -> Result<(), CoreError> {
    if spans.is_empty() || spans.len() > CALENDAR_MAX_DAYS {
        return Err(invalid("range"));
    }
    for s in spans {
        // NaN ve sonsuz da aralık dışındadır.
        let in_range = JD_UT_RANGE.contains(&s.start_jd_ut) && JD_UT_RANGE.contains(&s.end_jd_ut);
        let days = s.end_jd_ut - s.start_jd_ut;
        if !in_range || days <= 0.0 || days > MAX_DAY_SPAN_DAYS {
            return Err(invalid("range"));
        }
    }
    if spans.windows(2).any(|w| w[0].end_jd_ut != w[1].start_jd_ut) {
        return Err(invalid("range"));
    }
    Ok(())
}

/// `n` günün dilimleri, sırayla: `[from, to)` gün sıraları, her biri en çok
/// `CALENDAR_SLICE_DAYS` gün.
pub fn calendar_slices(n: usize) -> Vec<(usize, usize)> {
    (0..n)
        .step_by(CALENDAR_SLICE_DAYS)
        .map(|from| (from, (from + CALENDAR_SLICE_DAYS).min(n)))
        .collect()
}

/// Takvim gününün içindeki ilk gün doğumu; o gün Güneş doğmuyorsa `None`.
pub fn sunrise_in<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    geo_lat: f64,
    geo_lon: f64,
    span: DaySpan,
) -> Result<Option<f64>, CoreError> {
    let disc = settings.sunrise_def.sun_disc();
    // Arama günün başından hemen önce başlar: tam gece yarısındaki doğuş o güne aittir.
    let from = span.start_jd_ut - TIMELINE_RESOLUTION_DAYS;
    match eph.sun_rise_set(from, RiseSet::Rise, geo_lat, geo_lon, disc)? {
        Some(t) if !t.is_finite() => Err(CoreError::NonFiniteResult("sunrise")),
        Some(t) if span.contains(t) => Ok(Some(t)),
        _ => Ok(None),
    }
}

/// Gün doğumu olmayan günde olguların okunduğu an: günün ortasının düştüğü yerel ortalama
/// tarihte 06:00 (gün panchangının kuralı), günün içine sıkıştırılmış.
pub fn fallback_moment(span: DaySpan, geo_lon: f64) -> f64 {
    let mid = 0.5 * (span.start_jd_ut + span.end_jd_ut);
    // Yerel gün sayısı `jd + 0,5 + boylam / 360`'tır.
    let local = mid + 0.5 + geo_lon / 360.0;
    let six = local.floor() - 0.5 - geo_lon / 360.0 + FALLBACK_DAY_START;
    six.clamp(span.start_jd_ut, span.end_jd_ut - TIMELINE_RESOLUTION_DAYS)
}

/// Bir takvim gününün gün doğumu olguları: tithi ve Ay nakshatrası, günün içindeki gün
/// doğumunda; gün doğumu yoksa `fallback_moment` anında. Güneş ve Ay her zodyak kipinde
/// sidereal sorulur. Konum denetimi alan adıyla döner (`place`); konum hata parametresine girmez.
pub fn day_facts<E: Ephemeris + ?Sized>(
    eph: &mut E,
    settings: &CalcSettings,
    geo_lat: f64,
    geo_lon: f64,
    span: DaySpan,
) -> Result<DayFacts, CoreError> {
    check_place(geo_lat, geo_lon)?;
    let sunrise_jd_ut = sunrise_in(eph, settings, geo_lat, geo_lon, span)?;
    let jd_ut = sunrise_jd_ut.unwrap_or_else(|| fallback_moment(span, geo_lon));
    let zodiac = sidereal(settings);
    let sun = lon_of(eph, jd_ut, Body::Sun, zodiac)?;
    let moon = lon_of(eph, jd_ut, Body::Moon, zodiac)?;
    let tithi = tithi_no(elongation(sun, moon));
    Ok(DayFacts {
        jd_ut,
        sunrise_jd_ut,
        tithi,
        paksha: paksha(tithi),
        nakshatra: nakshatra_pada(moon).0,
    })
}
````

- [ ] **Step 5: Testler ve kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -1
cargo test -p jyotish-core --lib calendar 2>&1 | grep -E "^test result"
cargo test -p jyotish-core --lib 2>&1 | grep -E "^test result"
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
```
Expected: fmt çıktısız; clippy `Finished`; `6 passed … 466 filtered out`; lib `472 passed`; sayaç `0`; Vitest `97 passed (97)`, `973 passed (973)`; `✓ built`.

- [ ] **Step 6: Commit**

```bash
git add crates/jyotish-core/src/calendar crates/jyotish-core/src/lib.rs
git diff --cached --stat | tail -1
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): calendar days and the sunrise facts of a civil day" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: `3 files changed, 348 insertions(+)`; tarama çıktısız; `git status --short` boş.

---

### Task 2: Çekirdek: takvimin olay süzgeci, olayların günlere dağıtımı ve sınırlar

Yol haritası §5 Plan 29 görev (2); D6–D9 ve D10'un çekirdek yarısı (dilim ve ay toplayıcısı). Olayları var olan `event_search_chunk` bulur; `search/events.rs` değişmez.

**Files:**
- Create: `crates/jyotish-core/src/calendar/events.rs`
- Modify: `crates/jyotish-core/src/calendar/mod.rs`
- Test: `events.rs` `mod tests` (7)

**Interfaces:**
- Consumes: Task 1'in `DaySpan`, `DayFacts`, `day_facts`, `check_spans`, `calendar_slices` ve `grid::check_place` (`pub(crate)`); `crate::search::events::{event_search_chunk, EventSpec, SkyEvent, SkyEventKind}`; `crate::positions::Positions`; `crate::chart::rasi_frame` (`pub(crate)`); `ephemeris::{Body, Ephemeris, Precision}`; testlerde `event_search` ve `FakeEph`.
- Produces (`jyotish_core::calendar` yolunda):
  - `CALENDAR_KINDS: [SkyEventKind; 7]`, `CALENDAR_DEFAULT_KINDS: [SkyEventKind; 6]`, `CALENDAR_MAX_EVENTS: usize = 300`.
  - `CalendarFilter { kinds: Vec<SkyEventKind>, moon: bool }` (`validate()`, `grahas() -> Vec<Graha>`, `event_spec(from_jd_ut, to_jd_ut, has_natal) -> Option<EventSpec>`).
  - `CalendarSpec { lat, lon, settings: CalcSettings, filter: CalendarFilter }` (`validate()`).
  - `CalendarDay { span: DaySpan, facts: DayFacts, events: Vec<SkyEvent> }`, `CalendarSlice { days: Vec<CalendarDay>, truncated: bool, low_precision: bool }`.
  - `assign_events(spans: &[DaySpan], events: Vec<SkyEvent>, limit: usize) -> (Vec<Vec<SkyEvent>>, bool)`.
  - `calendar_slice(eph, spec: &CalendarSpec, natal: Option<&Positions>, spans: &[DaySpan], limit: usize) -> Result<CalendarSlice, CoreError>` ve `calendar_month(eph, spec, natal, spans) -> Result<CalendarSlice, CoreError>`.

- [ ] **Step 1: Testler: süzgeç, dağıtım, olayların günleri, Ay seçimi, dilimlemenin ikinci yazımı, sınır, hatalar; modül kaydı**

Beklenen günler sahte gökyüzünün hızlarından elle türetilmiştir (yorumlarda): gün `i`, J2000'den `i − 0,5` … `i + 0,5` gün sonrasıdır. Testler `SkyEvent`'i alan alan kurmaz (gerçek bir olayın kopyasının anını değiştirir): tip alan kazanırsa test derlenmeyi sürdürür ("Paylaşılan dosyalar").

````diff
diff --git a/crates/jyotish-core/src/calendar/mod.rs b/crates/jyotish-core/src/calendar/mod.rs
index 072c9cb..0f405e8 100644
--- a/crates/jyotish-core/src/calendar/mod.rs
+++ b/crates/jyotish-core/src/calendar/mod.rs
@@ -11,8 +11,13 @@
 //! hükmü yoktur. Sonuç saklanmaz (K1). Adlar `jyotish_core` kökünden yeniden dışa aktarılmaz;
 //! tüketiciler `jyotish_core::calendar::…` yolunu kullanır.
 
+mod events;
 mod grid;
 
+pub use events::{
+    assign_events, calendar_month, calendar_slice, CalendarDay, CalendarFilter, CalendarSlice,
+    CalendarSpec, CALENDAR_DEFAULT_KINDS, CALENDAR_KINDS, CALENDAR_MAX_EVENTS,
+};
 pub use grid::{
     calendar_slices, check_spans, day_facts, fallback_moment, sunrise_in, DayFacts, DaySpan,
     CALENDAR_MAX_DAYS, CALENDAR_SLICE_DAYS, MAX_DAY_SPAN_DAYS,
````

**Yeni dosya:** `crates/jyotish-core/src/calendar/events.rs` (şimdilik yalnızca test bloğu)

````rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::FakeEph;
    use crate::search::events::event_search;

    /// 2000-01-01 00:00 UT.
    const MIDNIGHT: f64 = 2_451_544.5;
    const HALF_SECOND: f64 = 0.5 / 86_400.0;

    fn days(n: usize) -> Vec<DaySpan> {
        (0..n)
            .map(|i| DaySpan {
                start_jd_ut: MIDNIGHT + i as f64,
                end_jd_ut: MIDNIGHT + (i + 1) as f64,
            })
            .collect()
    }

    fn spec(kinds: &[SkyEventKind], moon: bool) -> CalendarSpec {
        CalendarSpec {
            lat: 28.6,
            lon: 77.2,
            settings: CalcSettings::default(),
            filter: CalendarFilter {
                kinds: kinds.to_vec(),
                moon,
            },
        }
    }

    fn month(spec: &CalendarSpec, natal: Option<&Positions>, n: usize) -> CalendarSlice {
        calendar_month(&mut FakeEph::default(), spec, natal, &days(n)).unwrap()
    }

    /// (gün sırası, tür, grahalar) üçlüleri, takvim sırasıyla.
    fn placed(slice: &CalendarSlice) -> Vec<(usize, SkyEventKind, Vec<Graha>)> {
        slice
            .days
            .iter()
            .enumerate()
            .flat_map(|(i, d)| d.events.iter().map(move |e| (i, e.kind, e.grahas.clone())))
            .collect()
    }

    /// Gerçek bir olayın (sahte gökyüzünün ilk dolunayı) verilen ana taşınmış kopyası. Olay
    /// alan alan kurulmaz: `SkyEvent` yeni bir alan kazanırsa bu test derlenmeyi sürdürür.
    fn event(jd_ut: f64) -> SkyEvent {
        let search = spec(&[SkyEventKind::FullMoon], false)
            .filter
            .event_spec(MIDNIGHT, MIDNIGHT + 31.0, false)
            .unwrap();
        let found = event_search(
            &mut FakeEph::default(),
            &CalcSettings::default(),
            &search,
            None,
        )
        .unwrap();
        SkyEvent {
            jd_ut,
            ..found[0].clone()
        }
    }

    fn field<T: std::fmt::Debug>(r: Result<T, CoreError>) -> &'static str {
        match r {
            Err(CoreError::InvalidSearchInput(f)) => f,
            other => panic!("expected an invalid search input, got {other:?}"),
        }
    }

    #[test]
    fn the_filter_takes_calendar_kinds_only_and_the_moon_is_a_separate_choice() {
        assert!(spec(&CALENDAR_KINDS, true).validate().is_ok());
        assert!(spec(&[], false).validate().is_ok());
        assert_eq!(
            field(spec(&[SkyEventKind::Conjunction], false).validate()),
            "kinds"
        );
        assert_eq!(
            field(spec(&[SkyEventKind::HouseFromNatalMoon], false).validate()),
            "kinds"
        );
        let twice = [SkyEventKind::NewMoon, SkyEventKind::NewMoon];
        assert_eq!(field(spec(&twice, false).validate()), "kinds");
        // Yer ve ayar da denetlenir.
        let mut far = spec(&[], false);
        far.lat = 95.0;
        assert_eq!(field(far.validate()), "place");
        // Varsayılan türler takvim türlerinin alt kümesidir ve açık harita istemez.
        assert!(CALENDAR_DEFAULT_KINDS
            .iter()
            .all(|k| CALENDAR_KINDS.contains(k) && !k.needs_natal()));

        // Ay kapalıyken sekiz graha; açıkken dokuzu.
        let off = spec(&CALENDAR_KINDS, false).filter;
        assert_eq!(off.grahas().len(), 8);
        assert!(!off.grahas().contains(&Graha::Moon));
        assert_eq!(spec(&[], true).filter.grahas(), Graha::ALL);

        // Arama girdisi: türler takvim sırasında; açık harita yoksa doğum türü düşer.
        let mixed = spec(
            &[
                SkyEventKind::TransitToNatal,
                SkyEventKind::FullMoon,
                SkyEventKind::SignIngress,
            ],
            false,
        )
        .filter;
        let with = mixed.event_spec(1.0, 2.0, true).unwrap();
        assert_eq!(
            with.kinds,
            [
                SkyEventKind::SignIngress,
                SkyEventKind::FullMoon,
                SkyEventKind::TransitToNatal
            ]
        );
        assert_eq!((with.from_jd_ut, with.to_jd_ut), (1.0, 2.0));
        let without = mixed.event_spec(1.0, 2.0, false).unwrap();
        assert_eq!(
            without.kinds,
            [SkyEventKind::SignIngress, SkyEventKind::FullMoon]
        );
        // Aranacak tür yoksa arama da yoktur.
        assert!(spec(&[], true).filter.event_spec(1.0, 2.0, true).is_none());
        let natal_only = spec(&[SkyEventKind::TransitToNatal], false).filter;
        assert!(natal_only.event_spec(1.0, 2.0, false).is_none());
        assert!(natal_only.event_spec(1.0, 2.0, true).is_some());
    }

    #[test]
    fn an_event_belongs_to_the_day_that_contains_its_moment() {
        let spans = days(3);
        let events = vec![
            // Aralıktan önce: atılır.
            event(MIDNIGHT - 0.25),
            // Tam gece yarısı: başlayan günündür.
            event(MIDNIGHT),
            event(MIDNIGHT + 0.5),
            event(MIDNIGHT + 1.0),
            event(MIDNIGHT + 2.999),
            // Son günün bitişi aralığın dışındadır.
            event(MIDNIGHT + 3.0),
        ];
        let (per_day, truncated) = assign_events(&spans, events.clone(), 10);
        assert!(!truncated);
        let counts: Vec<usize> = per_day.iter().map(Vec::len).collect();
        assert_eq!(counts, [2, 1, 1]);
        assert_eq!(per_day[1][0].jd_ut, MIDNIGHT + 1.0);
        // Sınır: ilk `limit` olay dağıtılır (atılanlar da sayılır) ve sonuç bunu söyler.
        let (per_day, truncated) = assign_events(&spans, events.clone(), 3);
        assert!(truncated);
        let counts: Vec<usize> = per_day.iter().map(Vec::len).collect();
        assert_eq!(counts, [2, 0, 0]);
        let (per_day, truncated) = assign_events(&spans, events, 6);
        assert!(!truncated);
        assert_eq!(per_day.iter().map(Vec::len).sum::<usize>(), 4);
        // Olay yoksa her gün boştur.
        let (per_day, truncated) = assign_events(&spans, Vec::new(), 0);
        assert!(!truncated && per_day.iter().all(Vec::is_empty) && per_day.len() == 3);
    }

    /// Sahte ephemeris'in J2000'deki (2. günün öğlesi) sidereal boylamları: Güneş 346°
    /// (0,98°/gün), Ay 71° (13,2), Mars 176° (0,6), Merkür 356° (1,2), Jüpiter 226° (−0,1),
    /// Venüs 16° (1,1), Satürn 276° (0,05). Gün sırası 0'dan başlar: gün `i`, J2000'den
    /// `i − 0,5` … `i + 0,5` gün sonrasıdır.
    #[test]
    fn sign_ingresses_and_lunations_land_on_their_days() {
        let found = month(
            &spec(
                &[
                    SkyEventKind::SignIngress,
                    SkyEventKind::NewMoon,
                    SkyEventKind::FullMoon,
                ],
                false,
            ),
            None,
            31,
        );
        assert_eq!(found.days.len(), 31);
        assert!(!found.truncated && !found.low_precision);
        // Merkür 356° → 0°: 4 / 1,2 = 3,33 gün (gün 3). Mars 176° → 180°: 4 / 0,6 = 6,67 gün
        // (gün 7). Dolunay: uzanım 85° → 180°, 95 / 12,22 = 7,77 gün (gün 8). Venüs 16° → 30°:
        // 14 / 1,1 = 12,73 gün (gün 13). Güneş 346° → 0°: 14 / 0,98 = 14,29 gün (gün 14). Yeni
        // ay: 275 / 12,22 = 22,50 gün (gün 23). Merkür 0° → 30°: 34 / 1,2 = 28,33 gün (gün 28).
        // Jüpiter, Satürn ve düğümler bu ayda burç değiştirmez; Ay kapalıdır.
        assert_eq!(
            placed(&found),
            [
                (3, SkyEventKind::SignIngress, vec![Graha::Mercury]),
                (7, SkyEventKind::SignIngress, vec![Graha::Mars]),
                (8, SkyEventKind::FullMoon, vec![Graha::Sun, Graha::Moon]),
                (13, SkyEventKind::SignIngress, vec![Graha::Venus]),
                (14, SkyEventKind::SignIngress, vec![Graha::Sun]),
                (23, SkyEventKind::NewMoon, vec![Graha::Sun, Graha::Moon]),
                (28, SkyEventKind::SignIngress, vec![Graha::Mercury]),
            ]
        );
        // Her olay kendi gününün içindedir ve günün olayları zaman sırasındadır.
        for d in &found.days {
            assert!(d.events.iter().all(|e| d.span.contains(e.jd_ut)));
            assert!(d.events.windows(2).all(|w| w[0].jd_ut <= w[1].jd_ut));
        }
        // Günlerin olguları olay seçiminden bağımsızdır.
        let bare = month(&spec(&[], false), None, 31);
        assert!(bare.days.iter().all(|d| d.events.is_empty()));
        assert!(!bare.truncated);
        let facts = |s: &CalendarSlice| s.days.iter().map(|d| d.facts).collect::<Vec<_>>();
        assert_eq!(facts(&bare), facts(&found));
    }

    #[test]
    fn the_moon_adds_its_own_ingresses_only_when_asked() {
        let kinds = [SkyEventKind::SignIngress, SkyEventKind::NakshatraIngress];
        let off = month(&spec(&kinds, false), None, 31);
        assert!(placed(&off).iter().all(|(_, _, g)| g[0] != Graha::Moon));
        let on = month(&spec(&kinds, true), None, 31);
        let moon = |kind: SkyEventKind| {
            placed(&on)
                .iter()
                .filter(|(_, k, g)| *k == kind && g[0] == Graha::Moon)
                .count()
        };
        // Ay 31 günde 409,2° yürür: 64,4°'den 473,6°'ye. Geçtiği burç sınırları 90°, 120°, …,
        // 450° (13 giriş); nakshatra sınırları 66°40′'tan 466°40′'a 13°20′ arayla (31 giriş).
        assert_eq!(moon(SkyEventKind::SignIngress), 13);
        assert_eq!(moon(SkyEventKind::NakshatraIngress), 31);
        // Öteki grahaların olayları iki sonuçta aynıdır.
        let others = |s: &CalendarSlice| {
            placed(s)
                .into_iter()
                .filter(|(_, _, g)| g[0] != Graha::Moon)
                .collect::<Vec<_>>()
        };
        assert_eq!(others(&on), others(&off));
    }

    #[test]
    fn the_sliced_month_finds_exactly_the_events_of_one_search() {
        let calendar = spec(&CALENDAR_KINDS, true);
        let natal = Positions::synthetic(
            None,
            [0.0, 100.0, 200.0, 200.0, 200.0, 200.0, 200.0, 200.0, 200.0],
        );
        let spans = days(31);
        let sliced = month(&calendar, Some(&natal), 31);
        let search = calendar
            .filter
            .event_spec(spans[0].start_jd_ut, spans[30].end_jd_ut, true)
            .unwrap();
        let whole = event_search(
            &mut FakeEph::default(),
            &calendar.settings,
            &search,
            Some(&natal),
        )
        .unwrap();
        let flat: Vec<&SkyEvent> = sliced.days.iter().flat_map(|d| &d.events).collect();
        // İkinci yazım: dilim sınırı olay kaybettirmez ve çoğaltmaz. Aynı andaki olayların sırası
        // dilime göre değişebilir; karşılaştırma tür ve graha üzerinden yapılır.
        assert_eq!(flat.len(), whole.len());
        assert!(!sliced.truncated);
        for w in &whole {
            assert!(
                flat.iter().any(|e| e.kind == w.kind
                    && e.grahas == w.grahas
                    && (e.jd_ut - w.jd_ut).abs() < HALF_SECOND),
                "{w:?}"
            );
        }
        // Güneş doğum Güneş'inin boylamına (0°) gün 14'te gelir: kendi boylamına dönüş.
        assert!(placed(&sliced).contains(&(
            14,
            SkyEventKind::TransitToNatal,
            vec![Graha::Sun, Graha::Sun]
        )));
        // Açık harita yoksa doğum türü sessizce düşer; öteki olaylar aynıdır.
        let without = month(&calendar, None, 31);
        let natal_free = |s: &CalendarSlice| {
            placed(s)
                .into_iter()
                .filter(|(_, k, _)| *k != SkyEventKind::TransitToNatal)
                .collect::<Vec<_>>()
        };
        assert!(placed(&without)
            .iter()
            .all(|(_, k, _)| *k != SkyEventKind::TransitToNatal));
        assert_eq!(natal_free(&without), natal_free(&sliced));
    }

    #[test]
    fn the_event_limit_cuts_the_list_and_says_so() {
        let calendar = spec(&[SkyEventKind::SignIngress], false);
        let spans = days(31);
        let cut = |limit: usize| {
            calendar_slice(&mut FakeEph::default(), &calendar, None, &spans, limit).unwrap()
        };
        // Beş girişin ilk ikisi: Merkür (gün 3) ve Mars (gün 7).
        let two = cut(2);
        assert!(two.truncated);
        assert_eq!(
            placed(&two).iter().map(|(i, _, _)| *i).collect::<Vec<_>>(),
            [3, 7]
        );
        // Tam sınırda kesilme yoktur.
        assert!(!cut(5).truncated);
        assert_eq!(placed(&cut(5)).len(), 5);
        // Pay bitmişse olay aranmaz; günlerin olguları yine döner.
        let none = cut(0);
        assert!(none.truncated && placed(&none).is_empty());
        assert_eq!(none.days.len(), 31);
        // Aranacak tür yoksa "kesildi" denmez.
        let bare =
            calendar_slice(&mut FakeEph::default(), &spec(&[], false), None, &spans, 0).unwrap();
        assert!(!bare.truncated);
        // Ayın sınırı gerçek bir ayın çok üstündedir: Ay açıkken bütün türler bile sığar.
        let full = month(&spec(&CALENDAR_KINDS, true), None, 31);
        assert!(!full.truncated);
        assert!(placed(&full).len() < CALENDAR_MAX_EVENTS / 2);
    }

    #[test]
    fn bad_input_is_named_by_field_and_low_precision_is_reported() {
        let calendar = spec(&[SkyEventKind::NewMoon], false);
        let run = |spec: &CalendarSpec, spans: &[DaySpan]| {
            calendar_month(&mut FakeEph::default(), spec, None, spans)
        };
        assert_eq!(field(run(&calendar, &[])), "range");
        assert_eq!(field(run(&calendar, &days(32))), "range");
        let mut far = calendar.clone();
        far.lon = 200.0;
        assert_eq!(field(run(&far, &days(3))), "place");
        let odd = spec(&[SkyEventKind::Conjunction], false);
        assert_eq!(field(run(&odd, &days(3))), "kinds");
        // Dilim de aynı denetimi yapar.
        let slice = calendar_slice(&mut FakeEph::default(), &odd, None, &days(3), 10);
        assert_eq!(field(slice), "kinds");

        let mut moshier = FakeEph {
            precision: Precision::Moshier,
            ..FakeEph::default()
        };
        let low = calendar_month(&mut moshier, &calendar, None, &days(8)).unwrap();
        assert!(low.low_precision);
        assert_eq!(low.days.len(), 8);
    }
}
````

- [ ] **Step 2: Testlerin derlenmediğini gör**

Run: `cargo test -p jyotish-core --lib calendar 2>&1 | grep -E "^error" | sort | uniq -c | head`
Expected: derleme hatası: `mod.rs`'in `pub use events::{…}` satırındaki adlar çözülemez (`E0432`) ve test bloğunun adları bulunamaz. Hiçbir test koşmaz.

- [ ] **Step 3: Süzgeç, dağıtım, dilim ve ay toplayıcısı**

**`crates/jyotish-core/src/calendar/events.rs`: uygulama, test bloğunun üstüne eklenir (arada bir boş satır):**

````rust
//! Takvimin olayları (Plan 29 D6–D9): olay aramasının çekirdeğinden gelen olayların takvim
//! günlerine dağıtımı, takvimin olay süzgeci ve olay sayısı sınırı. Yeni astronomi yoktur:
//! olayları `search::events::event_search_chunk` bulur.

use ephemeris::{Body, Ephemeris, Precision};
use serde::{Deserialize, Serialize};

use crate::chart::rasi_frame;
use crate::graha::Graha;
use crate::positions::Positions;
use crate::search::events::{event_search_chunk, EventSpec, SkyEvent, SkyEventKind};
use crate::settings::CalcSettings;
use crate::CoreError;

use super::grid::{calendar_slices, check_place, check_spans, day_facts, DayFacts, DaySpan};

/// Takvimde gösterilebilen olay türleri, bu sırayla. Kavuşum ve doğum Ay'ından ev değişimi
/// takvimde yoktur (ikincisi Jüpiter ve Satürn'ün burç girişidir ve o satırda zaten durur).
pub const CALENDAR_KINDS: [SkyEventKind; 7] = [
    SkyEventKind::SignIngress,
    SkyEventKind::NakshatraIngress,
    SkyEventKind::StationRetrograde,
    SkyEventKind::StationDirect,
    SkyEventKind::NewMoon,
    SkyEventKind::FullMoon,
    SkyEventKind::TransitToNatal,
];
/// Yeni panelin olay türleri: girişler, dönüşler, yeni ay ve dolunay. Doğum boylamına gelişler
/// ve Ay'ın kendi girişleri süzgeçten açılır.
pub const CALENDAR_DEFAULT_KINDS: [SkyEventKind; 6] = [
    SkyEventKind::SignIngress,
    SkyEventKind::NakshatraIngress,
    SkyEventKind::StationRetrograde,
    SkyEventKind::StationDirect,
    SkyEventKind::NewMoon,
    SkyEventKind::FullMoon,
];
/// Bir ayın taşıdığı en çok olay; fazlası kesilir ve sonuç bunu söyler.
pub const CALENDAR_MAX_EVENTS: usize = 300;

/// Takvimin olay süzgeci.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CalendarFilter {
    /// `CALENDAR_KINDS`'in bir alt kümesi; boş olabilir (yalnızca gün doğumu olguları).
    pub kinds: Vec<SkyEventKind>,
    /// Ay'ın girişleri ve doğum boylamlarına gelişleri de aranır. Kapalıyken graha seçimi Ay
    /// dışındaki sekiz grahadır; yeni ay ve dolunay bu seçime bağlı değildir.
    pub moon: bool,
}

fn invalid(field: &'static str) -> CoreError {
    CoreError::InvalidSearchInput(field)
}

impl CalendarFilter {
    /// Türler tekrarsızdır ve `CALENDAR_KINDS` içindedir; hata alan adını taşır (`kinds`).
    pub fn validate(&self) -> Result<(), CoreError> {
        let distinct = self
            .kinds
            .iter()
            .enumerate()
            .all(|(i, k)| !self.kinds[..i].contains(k));
        if distinct && self.kinds.iter().all(|k| CALENDAR_KINDS.contains(k)) {
            Ok(())
        } else {
            Err(invalid("kinds"))
        }
    }

    /// Aranan grahalar, `Graha::ALL` sırasında.
    pub fn grahas(&self) -> Vec<Graha> {
        Graha::ALL
            .into_iter()
            .filter(|g| self.moon || *g != Graha::Moon)
            .collect()
    }

    /// `[from, to)` için olay aramasının girdisi; aranacak tür yoksa `None`. Açık harita
    /// isteyen tür, harita verilmediyse sessizce düşer (süzgeç panelin ayarıdır ve haritadan
    /// bağımsız saklanır).
    pub fn event_spec(&self, from_jd_ut: f64, to_jd_ut: f64, has_natal: bool) -> Option<EventSpec> {
        let kinds: Vec<SkyEventKind> = CALENDAR_KINDS
            .into_iter()
            .filter(|k| self.kinds.contains(k) && (has_natal || !k.needs_natal()))
            .collect();
        if kinds.is_empty() {
            return None;
        }
        Some(EventSpec {
            from_jd_ut,
            to_jd_ut,
            kinds,
            grahas: self.grahas(),
        })
    }
}

/// Takvimin değişmeyen girdisi: yer (yalnızca gün doğumu için), hesap ayarı ve süzgeç.
#[derive(Debug, Clone, PartialEq)]
pub struct CalendarSpec {
    /// Kuzey +.
    pub lat: f64,
    /// Doğu +.
    pub lon: f64,
    pub settings: CalcSettings,
    pub filter: CalendarFilter,
}

impl CalendarSpec {
    pub fn validate(&self) -> Result<(), CoreError> {
        self.settings.validate()?;
        check_place(self.lat, self.lon)?;
        self.filter.validate()
    }
}

/// Bir takvim günü: sınırları, gün doğumu olguları ve o güne düşen olaylar (zaman sırasıyla).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct CalendarDay {
    pub span: DaySpan,
    pub facts: DayFacts,
    pub events: Vec<SkyEvent>,
}

/// Bir dilimin (ya da bütün ayın) günleri.
#[derive(Debug, Clone, PartialEq)]
pub struct CalendarSlice {
    pub days: Vec<CalendarDay>,
    /// Olay sayısı sınırı aştı: sınırdan sonraki olaylar listede yoktur.
    pub truncated: bool,
    /// Güneş Moshier hassasiyetinde hesaplandı (ephemeris dosyalarının aralığının dışı).
    pub low_precision: bool,
}

/// Olayları günlere dağıtır: olay, anını içeren günündür (`[start, end)`; tam gece yarısındaki
/// olay başlayan günündür). `events` zaman sırasında olmalıdır; ilk `limit` olay dağıtılır.
/// Dönüş: gün başına olaylar ve "kesildi" bilgisi. Hiçbir güne düşmeyen olay atılır.
pub fn assign_events(
    spans: &[DaySpan],
    events: Vec<SkyEvent>,
    limit: usize,
) -> (Vec<Vec<SkyEvent>>, bool) {
    let truncated = events.len() > limit;
    let mut out = vec![Vec::new(); spans.len()];
    for event in events.into_iter().take(limit) {
        let i = spans.partition_point(|s| s.end_jd_ut <= event.jd_ut);
        if spans.get(i).is_some_and(|s| s.contains(event.jd_ut)) {
            out[i].push(event);
        }
    }
    (out, truncated)
}

/// Bir dilimin günleri: her günün gün doğumu olguları ve o güne düşen olaylar. `spans` ardışık
/// takvim günleridir (çağıran `calendar_slices` ile böler); `limit` bu dilimin taşıyabileceği
/// en çok olaydır (ayın sınırından geriye duran pay). Pay bitmişse olay aranmaz ve sonuç
/// "kesildi" işaretlidir. `natal` açık haritanın konumlarıdır (yalnızca sidereal boylamlar
/// kullanılır; sonuca ve hataya girmez).
pub fn calendar_slice<E: Ephemeris + ?Sized>(
    eph: &mut E,
    spec: &CalendarSpec,
    natal: Option<&Positions>,
    spans: &[DaySpan],
    limit: usize,
) -> Result<CalendarSlice, CoreError> {
    spec.validate()?;
    check_spans(spans)?;
    let (from, to) = (spans[0].start_jd_ut, spans[spans.len() - 1].end_jd_ut);
    // Hassasiyet Güneş'ten okunur (`compute_chart` ile aynı kural).
    let low_precision = eph
        .body(from, Body::Sun, rasi_frame(&spec.settings))?
        .precision
        == Precision::Moshier;
    let facts: Vec<DayFacts> = spans
        .iter()
        .map(|s| day_facts(eph, &spec.settings, spec.lat, spec.lon, *s))
        .collect::<Result<_, _>>()?;
    let (events, truncated) = match spec.filter.event_spec(from, to, natal.is_some()) {
        None => (vec![Vec::new(); spans.len()], false),
        Some(_) if limit == 0 => (vec![Vec::new(); spans.len()], true),
        Some(search) => {
            let found = event_search_chunk(eph, &spec.settings, &search, natal, (from, to))?;
            assign_events(spans, found, limit)
        }
    };
    let days = spans
        .iter()
        .zip(facts)
        .zip(events)
        .map(|((span, facts), events)| CalendarDay {
            span: *span,
            facts,
            events,
        })
        .collect();
    Ok(CalendarSlice {
        days,
        truncated,
        low_precision,
    })
}

/// Bütün ay, dilimleri sırayla yürüterek (testler ve tek seferlik kullanım). Uygulama dilimleri
/// kendisi yürütür ve aralarında motor iş parçacığını bırakır.
pub fn calendar_month<E: Ephemeris + ?Sized>(
    eph: &mut E,
    spec: &CalendarSpec,
    natal: Option<&Positions>,
    spans: &[DaySpan],
) -> Result<CalendarSlice, CoreError> {
    spec.validate()?;
    check_spans(spans)?;
    let mut month = CalendarSlice {
        days: Vec::with_capacity(spans.len()),
        truncated: false,
        low_precision: false,
    };
    let mut left = CALENDAR_MAX_EVENTS;
    for (from, to) in calendar_slices(spans.len()) {
        let slice = calendar_slice(eph, spec, natal, &spans[from..to], left)?;
        left -= slice.days.iter().map(|d| d.events.len()).sum::<usize>();
        month.truncated |= slice.truncated;
        month.low_precision |= slice.low_precision;
        month.days.extend(slice.days);
    }
    Ok(month)
}
````

- [ ] **Step 4: Testler ve kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -1
cargo test -p jyotish-core --lib calendar 2>&1 | grep -E "^test result"
cargo test -p jyotish-core --lib 2>&1 | grep -E "^test result"
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
```
Expected: fmt çıktısız; clippy `Finished`; `13 passed … 466 filtered out`; lib `479 passed`; sayaç `0`; Vitest `97 passed (97)`, `973 passed (973)`; `✓ built`. Bir el örneği düşerse beklenen değer değiştirilmez: yorumdaki türetme ve kural yeniden okunur (Global Constraints).

- [ ] **Step 5: Commit**

```bash
git add crates/jyotish-core/src/calendar
git diff --cached --stat | tail -1
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(core): calendar events by day, the calendar filter and the event limit" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: `2 files changed, 586 insertions(+)`; tarama çıktısız; `git status --short` boş.

---

### Task 3: Uygulama: `month_calendar` komutu (dilimli), bağlamalar, fikstür ve sarmalayıcı

Yol haritası §5 Plan 29 görev (3); K1–K3; D4, D10–D13, D22. Komut yeni bir dosyadadır ve var olan komut dosyalarına dokunulmaz; `muhurta_commands`'ın `pub(crate)` yardımcıları ve `search_commands::SkyEventView` yalnızca okunur. Paylaşılan dosyalardaki her değişiklik ekleme türündedir ("Paylaşılan dosyalar" tablosu).

**Files:**
- Create: `app/src-tauri/src/calendar_commands.rs`, `app/src-tauri/tests/calendar.rs`, `app/ui/test/calendarFixture.ts`, `app/ui/test/calendarFixture.test.ts`
- Modify: `app/src-tauri/src/lib.rs`, `app/src-tauri/tests/ui_fixture.rs`, `app/ui/bindings.ts` (üretilir), `app/ui/test/fixtures/delhi.json` (üretilir), `app/ui/api.ts`, `app/ui/api.test.ts`
- Test: `calendar_commands.rs` `mod tests` (3); `tests/calendar.rs` (1); `ui_fixture` (var olan iki test, yeni alanla); `calendarFixture.test.ts` (3); `api.test.ts` (tabloda bir satır)

**Interfaces:**
- Consumes: Task 1–2'nin `jyotish_core::calendar::{calendar_slice, calendar_slices, CalendarFilter, CalendarSpec, DayFacts, DaySpan, CALENDAR_MAX_EVENTS, CALENDAR_METHOD, CALENDAR_KINDS, CALENDAR_DEFAULT_KINDS}`'i; `jyotish_core::{compute_positions, CalcSettings, ChartInput, ChartWarning, Positions, Vara}`; `atlas::{Date, Timestamp}`; `ephemeris::{Engine, SwissEph}`; `client_commands::blocking`; `commands::CommandError`; `muhurta_commands::{day_start_jd, offset_at, today, MuhurtaPlace, J2000_JD}`; `search_commands::SkyEventView`.
- Produces:
  - Rust: `calendar_commands::{YearMonth { year: i32, month: u8 }, CalendarRequest { month: Option<YearMonth>, place: MuhurtaPlace, filter: CalendarFilter, settings: CalcSettings, natal: Option<ChartInput> }, CalendarDayView { date, day, weekday: Vara, span, facts, offset_secs, panchang_jd_ut, events: Vec<SkyEventView> }, CalendarView { month, prev, next, today: Option<u8>, days, truncated, natal, warnings, method }, CALENDAR_YEARS, month_calendar_blocking(engine: &Engine, request: CalendarRequest, now: Timestamp) -> Result<CalendarView, CommandError>}`; Tauri komutu `month_calendar(request) -> CalendarView`.
  - Hata kodu `invalid_calendar_input` (`field`: `place`, `month`, `kinds`, `range`).
  - TypeScript (üretilir): `commands.monthCalendar`, `CalendarRequest`, `CalendarView`, `CalendarDayView`, `CalendarFilter`, `DayFacts`, `DaySpan`, `YearMonth`. Sarmalayıcı `api.monthCalendar(request)`.
  - Fikstür: `calendar: { request, request_full, month, month_full, month_next, month_polar, ids }`; tipi ve okuyucusu `calendarFixture()`.

- [ ] **Step 1: Rust testleri: ay adımı, hafta günü, hata çevirisi; gerçek motorla çapraz denetim, yaz saati, kutup dairesi, atlanan gün, hatalar ve bütçe; fikstür; kayıt**

Dört parça: komut kaydı; komut dosyasının birim testleri (şimdilik yalnızca test bloğu); gerçek motorla tek bir entegrasyon testi (süreçte tek motor açılabilir); fikstür bloğu. Entegrasyon testindeki gök olayları (2026 Kasım'ında yeni ay 9'unda, dolunay 24'ünde) gökbilim almanağı bilgisidir, motor çıktısından kopyalanmamıştır.

````diff
diff --git a/app/src-tauri/src/lib.rs b/app/src-tauri/src/lib.rs
index e704350..6f91a67 100644
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -1,6 +1,7 @@
 pub mod atlas_commands;
 pub mod avastha_commands;
 pub mod bhava_commands;
+pub mod calendar_commands;
 pub mod client_commands;
 pub mod commands;
 pub mod data_dir;
@@ -51,6 +52,7 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             natal_extras_commands::compute_vimshopaka,
             muhurta_commands::day_panchang,
             muhurta_commands::muhurta_search,
+            calendar_commands::month_calendar,
             prashna_commands::create_prashna,
             prashna_commands::update_prashna,
             prashna_commands::compute_prashna_facts,
````

**Yeni dosya:** `app/src-tauri/src/calendar_commands.rs` (şimdilik yalnızca test bloğu)

````rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn months_step_across_years_and_stop_at_the_ends_of_the_range() {
        let ym = |year: i32, month: u8| YearMonth { year, month };
        assert_eq!(ym(2026, 11).shifted(1), Some(ym(2026, 12)));
        assert_eq!(ym(2026, 12).shifted(1), Some(ym(2027, 1)));
        assert_eq!(ym(2027, 1).shifted(-1), Some(ym(2026, 12)));
        assert_eq!(ym(2026, 3).shifted(-1), Some(ym(2026, 2)));
        // Aralığın uçları.
        assert_eq!(ym(1, 1).shifted(-1), None);
        assert_eq!(ym(1, 1).shifted(1), Some(ym(1, 2)));
        assert_eq!(ym(2999, 12).shifted(1), None);
        assert_eq!(ym(2999, 12).shifted(-1), Some(ym(2999, 11)));
        assert!(!ym(0, 5).is_valid() && !ym(3000, 1).is_valid());
        assert!(!ym(2026, 0).is_valid() && !ym(2026, 13).is_valid());
        assert!(ym(1, 1).is_valid() && ym(2999, 12).is_valid());
    }

    #[test]
    fn the_weekday_of_a_civil_date_is_its_calendar_weekday() {
        // 2000-01-01 Cumartesi; 2026-11-01 Pazar; 2026-11-25 Çarşamba.
        let day = |y, m, d| weekday(civil(y, m, d).unwrap());
        assert_eq!(day(2000, 1, 1), Vara::Saturday);
        assert_eq!(day(2026, 11, 1), Vara::Sunday);
        assert_eq!(day(2026, 11, 25), Vara::Wednesday);
        // Var olmayan tarih kurulmaz.
        assert!(civil(2026, 2, 30).is_none());
        assert!(civil(40_000, 1, 1).is_none());
    }

    #[test]
    fn errors_name_the_field_and_carry_nothing_else() {
        let e = invalid("place");
        assert_eq!(e.code, "invalid_calendar_input");
        assert_eq!(e.params.len(), 1);
        assert_eq!(e.params["field"], "place");
        // Çekirdeğin ve muhurta yardımcılarının hatası takvimin koduna, sabit bir alan adıyla
        // çevrilir.
        let from = |code: &str, field: &str| {
            calendar_error(CommandError::new(code, &[("field", field.to_owned())]))
        };
        for (code, given, want) in [
            ("invalid_search_input", "kinds", "kinds"),
            ("invalid_search_input", "place", "place"),
            ("invalid_search_input", "range", "range"),
            ("invalid_search_input", "grahas", "range"),
            ("invalid_muhurta_input", "place", "place"),
            ("invalid_muhurta_input", "month", "month"),
            ("invalid_muhurta_input", "28.6139, 77.2090", "range"),
        ] {
            let e = from(code, given);
            assert_eq!(
                (e.code.as_str(), e.params["field"].as_str(), e.params.len()),
                ("invalid_calendar_input", want, 1)
            );
        }
        // Hesap ve motor hataları olduğu gibi geçer.
        let calc = calendar_error(CommandError::new("calc", &[("detail", "x".to_owned())]));
        assert_eq!(
            (calc.code.as_str(), calc.params["detail"].as_str()),
            ("calc", "x")
        );
    }
}
````

**Yeni dosya:** `app/src-tauri/tests/calendar.rs`

````rust
//! Geçiş takvimi komutunun gövdesi, gerçek `Engine` ile uçtan uca (yol haritası Plan 29; ileri
//! modüller spec'i §13 "Komut katmanı"). Tek test: süreçte yalnızca bir SE örneği açılabilir.
//! Çapraz denetimler var olan iki komuta dayanır: günün olguları gün panchangıyla
//! (`day_panchang`), olaylar olay aramasıyla (`event_search`) karşılaştırılır.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use atlas::Timestamp;
use ephemeris::Engine;
use jyotish_core::calendar::{
    CalendarFilter, CALENDAR_DEFAULT_KINDS, CALENDAR_KINDS, CALENDAR_MAX_EVENTS, CALENDAR_METHOD,
};
use jyotish_core::{
    CalcSettings, ChartInput, ChartWarning, Graha, Paksha, Sign, SkyEventKind, Vara,
};
use jyotish_desk_lib::calendar_commands::{
    month_calendar_blocking, CalendarRequest, CalendarView, YearMonth,
};
use jyotish_desk_lib::commands::CommandError;
use jyotish_desk_lib::muhurta_commands::{day_panchang_blocking, MuhurtaPlace};
use jyotish_desk_lib::search_commands::{event_search_blocking, EventSearchSpec, SearchRegistry};
use jyotish_desk_lib::transit_commands::TransitAt;
use store::TzChoice;

const SECOND: f64 = 1.0 / 86_400.0;
/// Bir ayın bütçesi (ortanca; hata ayıklama derlemesi): bütün türler, Ay ve açık harita.
const MONTH_BUDGET: Duration = Duration::from_millis(150);

fn place(lat: f64, lon: f64, zone: &str) -> MuhurtaPlace {
    MuhurtaPlace {
        lat,
        lon,
        tz: TzChoice::Iana(zone.into()),
    }
}

fn delhi() -> MuhurtaPlace {
    place(28.6139, 77.209, "Asia/Kolkata")
}

fn tromso() -> MuhurtaPlace {
    place(69.6492, 18.9553, "Europe/Oslo")
}

fn ym(year: i32, month: u8) -> Option<YearMonth> {
    Some(YearMonth { year, month })
}

fn filter(kinds: &[SkyEventKind], moon: bool) -> CalendarFilter {
    CalendarFilter {
        kinds: kinds.to_vec(),
        moon,
    }
}

/// Deneme haritası (Delhi, J2000 öğlesi UT); gerçek kişi değildir.
fn natal() -> ChartInput {
    ChartInput {
        jd_ut: 2_451_545.0,
        lat: 28.6139,
        lon: 77.209,
        settings: CalcSettings::default(),
        time_known: true,
        lagna_number: None,
    }
}

fn request(month: Option<YearMonth>, place: MuhurtaPlace) -> CalendarRequest {
    CalendarRequest {
        month,
        place,
        filter: filter(&CALENDAR_DEFAULT_KINDS, false),
        settings: CalcSettings::default(),
        natal: None,
    }
}

fn field(r: Result<CalendarView, CommandError>) -> String {
    let e = r.expect_err("an input error");
    assert_eq!(e.code, "invalid_calendar_input");
    assert_eq!(e.params.len(), 1, "only the field name");
    e.params["field"].clone()
}

#[test]
fn the_month_calendar_agrees_with_the_day_panchang_and_the_event_search() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let engine = Engine::spawn(root.join("../../data/ephe")).expect("spawn engine");
    // 2026-11-25 10:00 UT: Delhi'de 25 Kasım 2026 Çarşamba 15:30.
    let now = Timestamp::from_second(1_795_600_800).expect("timestamp");
    let run = |r: CalendarRequest| month_calendar_blocking(&engine, r, now);

    // 1. Ay verilmezse yerin bugünkü ayıdır; günler takvim günleridir.
    let november = run(request(None, delhi())).expect("this month");
    assert_eq!(
        november.month,
        YearMonth {
            year: 2026,
            month: 11
        }
    );
    assert_eq!(november.prev, ym(2026, 10));
    assert_eq!(november.next, ym(2026, 12));
    assert_eq!(november.today, Some(25));
    assert_eq!(november.method, CALENDAR_METHOD);
    assert!(!november.truncated && !november.natal && november.warnings.is_empty());
    assert_eq!(november.days.len(), 30);
    assert_eq!(november.days[0].date, "2026-11-01");
    assert_eq!(november.days[29].date, "2026-11-30");
    // 1 Kasım 2026 Pazar'dır; hafta günleri sırayla yürür.
    for (i, d) in november.days.iter().enumerate() {
        assert_eq!(usize::from(d.day), i + 1);
        assert_eq!(d.weekday, Vara::ALL[i % 7], "{}", d.date);
        // Delhi'de yaz saati yoktur: her gün 24 saattir ve ofset +05:30'dur.
        assert!((d.span.end_jd_ut - d.span.start_jd_ut - 1.0).abs() < SECOND);
        assert_eq!(d.offset_secs, 19_800);
        // Olgular günün içindeki gün doğumunda okunur.
        assert_eq!(d.facts.sunrise_jd_ut, Some(d.facts.jd_ut));
        assert!(d.span.start_jd_ut <= d.facts.jd_ut && d.facts.jd_ut < d.span.end_jd_ut);
        assert!(d.facts.jd_ut < d.panchang_jd_ut && d.panchang_jd_ut < d.span.end_jd_ut);
        assert!(d.events.iter().all(|e| {
            let jd = e.event.jd_ut;
            d.span.start_jd_ut <= jd && jd < d.span.end_jd_ut && e.offset_secs == 19_800
        }));
    }
    assert!(november
        .days
        .windows(2)
        .all(|w| w[0].span.end_jd_ut == w[1].span.start_jd_ut));
    // İstenen ay da aynı sonucu verir; başka bir ayda "bugün" yoktur.
    assert_eq!(
        run(request(ym(2026, 11), delhi())).expect("asked month"),
        november
    );
    let december = run(request(ym(2026, 12), delhi())).expect("next month");
    assert_eq!((december.days.len(), december.today), (31, None));
    assert_eq!(december.days[0].weekday, Vara::Tuesday);

    // 2. Çapraz denetim: hücrenin tithisi ve nakshatrası, o gün doğumuyla başlayan panchang
    // gününün ilk değeridir (`day_panchang`, aynı yer ve ayar).
    for d in &november.days {
        let view = day_panchang_blocking(
            &engine,
            TransitAt::Jd(d.panchang_jd_ut),
            now,
            delhi(),
            CalcSettings::default(),
            None,
        )
        .expect("day panchang");
        let sunrise = view.day.sunrise_jd_ut.expect("a sunrise in Delhi");
        assert!((sunrise - d.facts.jd_ut).abs() < SECOND, "{}", d.date);
        assert_eq!(view.day.tithis[0].value.no, d.facts.tithi, "{}", d.date);
        assert_eq!(
            view.day.tithis[0].value.paksha, d.facts.paksha,
            "{}",
            d.date
        );
        assert_eq!(
            view.day.nakshatras[0].value.no, d.facts.nakshatra,
            "{}",
            d.date
        );
        assert_eq!(view.day.vara, d.weekday, "{}", d.date);
    }
    // Bilinen gök olayları (gökbilim almanağı): 2026 Kasım'ında yeni ay 9 Kasım'da, dolunay
    // 24 Kasım'dadır (Delhi tarihiyle de). Dolunay günü gün doğumunda Ay büyümektedir; ertesi
    // günün tithisi Krishna'dır.
    let day_of = |view: &CalendarView, kind: SkyEventKind| -> Vec<String> {
        view.days
            .iter()
            .filter(|d| d.events.iter().any(|e| e.event.kind == kind))
            .map(|d| d.date.clone())
            .collect()
    };
    assert_eq!(day_of(&november, SkyEventKind::NewMoon), ["2026-11-09"]);
    assert_eq!(day_of(&november, SkyEventKind::FullMoon), ["2026-11-24"]);
    assert_eq!(november.days[23].facts.paksha, Paksha::Shukla);
    assert_eq!(november.days[24].facts.paksha, Paksha::Krishna);
    // Güneş ayda bir kez burç değiştirir: Kasım'da sidereal Akrep'e girer.
    let sun_ingress: Vec<_> = november
        .days
        .iter()
        .flat_map(|d| &d.events)
        .filter(|e| e.event.kind == SkyEventKind::SignIngress && e.event.grahas == [Graha::Sun])
        .collect();
    assert_eq!(sun_ingress.len(), 1);
    assert_eq!(sun_ingress[0].event.sign, Some(Sign::Scorpio));
    // Varsayılan süzgeçte Ay'ın kendi girişleri yoktur.
    assert!(november.days.iter().flat_map(|d| &d.events).all(|e| {
        e.event.kind == SkyEventKind::NewMoon
            || e.event.kind == SkyEventKind::FullMoon
            || e.event.grahas[0] != Graha::Moon
    }));

    // 3. Çapraz denetim: bütün türler, Ay ve açık haritayla takvimin olayları, aynı ay için
    // olay aramasının bulduklarıdır (dilimleme olay kaybettirmez ve çoğaltmaz).
    let full_request = CalendarRequest {
        filter: filter(&CALENDAR_KINDS, true),
        natal: Some(natal()),
        ..request(ym(2026, 11), delhi())
    };
    let full = run(full_request.clone()).expect("full month");
    assert!(full.natal && !full.truncated);
    let searched = event_search_blocking(
        &engine,
        &SearchRegistry::default(),
        EventSearchSpec {
            from_date: Some("2026-11-01".into()),
            to_date: Some("2026-11-30".into()),
            place: delhi(),
            kinds: CALENDAR_KINDS.to_vec(),
            grahas: Graha::ALL.to_vec(),
            settings: CalcSettings::default(),
            natal: Some(natal()),
        },
        1,
        now,
    )
    .expect("event search");
    let in_calendar: Vec<_> = full.days.iter().flat_map(|d| &d.events).collect();
    assert_eq!(in_calendar.len(), searched.events.len());
    assert!(in_calendar.len() < CALENDAR_MAX_EVENTS / 2);
    for want in &searched.events {
        assert!(
            in_calendar.iter().any(|e| e.event.kind == want.event.kind
                && e.event.grahas == want.event.grahas
                && (e.event.jd_ut - want.event.jd_ut).abs() < SECOND),
            "{:?}",
            want.event
        );
    }
    // Günlerin olguları süzgeçten bağımsızdır; açık harita verilse de tür seçili değilse
    // doğum olayları aranmaz.
    let facts = |v: &CalendarView| v.days.iter().map(|d| d.facts).collect::<Vec<_>>();
    assert_eq!(facts(&full), facts(&november));
    let unasked = run(CalendarRequest {
        natal: Some(natal()),
        ..request(ym(2026, 11), delhi())
    })
    .expect("natal input without the natal kind");
    assert!(!unasked.natal);
    assert_eq!(unasked.days, november.days);
    // Tür seçili ama harita yok: tür sessizce düşer.
    let no_chart = run(CalendarRequest {
        natal: None,
        ..full_request.clone()
    })
    .expect("natal kind without a chart");
    assert!(!no_chart.natal);
    assert!(no_chart
        .days
        .iter()
        .flat_map(|d| &d.events)
        .all(|e| e.event.kind != SkyEventKind::TransitToNatal));
    // Olay türü seçilmemiş takvim yalnızca olguları taşır.
    let bare = run(CalendarRequest {
        filter: filter(&[], false),
        ..request(ym(2026, 11), delhi())
    })
    .expect("no kinds");
    assert!(bare.days.iter().all(|d| d.events.is_empty()));
    assert_eq!(facts(&bare), facts(&november));

    // 4. Yaz saati: Berlin'de 29 Mart 2026 23 saat, 25 Ekim 2026 25 saattir; ofset gün içinde
    // değişir ve günler yine boşluksuz ardışıktır.
    let berlin = || place(52.52, 13.405, "Europe/Berlin");
    let march = run(request(ym(2026, 3), berlin())).expect("March in Berlin");
    let hours = |v: &CalendarView, day: usize| {
        (v.days[day - 1].span.end_jd_ut - v.days[day - 1].span.start_jd_ut) * 24.0
    };
    assert!((hours(&march, 29) - 23.0).abs() < 1e-6);
    assert!((hours(&march, 28) - 24.0).abs() < 1e-6);
    assert_eq!(
        (march.days[27].offset_secs, march.days[29].offset_secs),
        (3600, 7200)
    );
    let october = run(request(ym(2026, 10), berlin())).expect("October in Berlin");
    assert!((hours(&october, 25) - 25.0).abs() < 1e-6);
    assert_eq!((march.days.len(), october.days.len()), (31, 31));
    assert!(march
        .days
        .windows(2)
        .all(|w| w[0].span.end_jd_ut == w[1].span.start_jd_ut));

    // 5. Kutup dairesi: Tromsø'da Aralık'ta Güneş doğmaz; olgular 06:00 yerel ortalama saatte
    // okunur ve gün panchangıyla aynı günü gösterir. Ocak'ta Güneş ay içinde geri döner.
    let polar = run(request(ym(2026, 12), tromso())).expect("polar night");
    assert_eq!(polar.days.len(), 31);
    for d in &polar.days {
        assert_eq!(d.facts.sunrise_jd_ut, None, "{}", d.date);
        assert!(d.span.start_jd_ut <= d.facts.jd_ut && d.facts.jd_ut < d.span.end_jd_ut);
        // 18,96° D: yerel ortalama saat UT + 1 sa 16 dk; 06:00 yerel 04:44 UT, 05:44 Oslo saati.
        let local_hours = (d.facts.jd_ut - d.span.start_jd_ut) * 24.0;
        assert!(
            (local_hours - 5.736).abs() < 0.01,
            "{}: {local_hours}",
            d.date
        );
    }
    let polar_day = day_panchang_blocking(
        &engine,
        TransitAt::Jd(polar.days[14].panchang_jd_ut),
        now,
        tromso(),
        CalcSettings::default(),
        None,
    )
    .expect("polar day panchang");
    assert!(polar_day
        .day
        .warnings
        .contains(&ChartWarning::SunriseUnavailable));
    assert!((polar_day.day.day_start_jd_ut - polar.days[14].facts.jd_ut).abs() < SECOND);
    assert_eq!(polar_day.day.tithis[0].value.no, polar.days[14].facts.tithi);
    let january = run(request(ym(2027, 1), tromso())).expect("the Sun returns");
    let risen = january
        .days
        .iter()
        .filter(|d| d.facts.sunrise_jd_ut.is_some())
        .count();
    assert!(0 < risen && risen < 31, "{risen}");
    assert_eq!(january.days[0].facts.sunrise_jd_ut, None);
    assert!(january.days[30].facts.sunrise_jd_ut.is_some());
    // Gece yarısı güneşi: Haziran'da da doğuş yoktur.
    let june = run(request(ym(2026, 6), tromso())).expect("midnight sun");
    assert!(june.days.iter().all(|d| d.facts.sunrise_jd_ut.is_none()));

    // 6. Takvimden gün atlayan saat dilimi: Samoa 30 Aralık 2011'i atladı. O gün listede yoktur
    // ve günler yine ardışıktır.
    let apia = run(request(
        ym(2011, 12),
        place(-13.8333, -171.7667, "Pacific/Apia"),
    ))
    .expect("the month with a skipped day");
    assert_eq!(apia.days.len(), 30);
    assert!(apia.days.iter().all(|d| d.date != "2011-12-30"));
    assert_eq!(
        (apia.days[28].date.as_str(), apia.days[29].date.as_str()),
        ("2011-12-29", "2011-12-31")
    );
    assert!(apia
        .days
        .windows(2)
        .all(|w| w[0].span.end_jd_ut == w[1].span.start_jd_ut));

    // 7. Yıl aralığının uçları ve ephemeris dosyalarının dışı: sonuç döner ve düşük hassasiyet
    // uyarısını taşır.
    let first = run(request(ym(1, 1), delhi())).expect("the first month");
    assert_eq!((first.prev, first.next), (None, ym(1, 2)));
    assert_eq!(first.warnings, [ChartWarning::LowPrecision]);
    let last = run(request(ym(2999, 12), delhi())).expect("the last month");
    assert_eq!((last.prev, last.next), (ym(2999, 11), None));
    assert_eq!(last.days.len(), 31);

    // 8. Girdi hataları yalnızca alan adını taşır; yer aydan önce denetlenir.
    assert_eq!(field(run(request(ym(2026, 13), delhi()))), "month");
    assert_eq!(field(run(request(ym(2026, 0), delhi()))), "month");
    assert_eq!(field(run(request(ym(0, 5), delhi()))), "month");
    assert_eq!(field(run(request(ym(3000, 1), delhi()))), "month");
    let nowhere = place(91.0, 0.0, "Asia/Kolkata");
    assert_eq!(field(run(request(ym(2026, 13), nowhere))), "place");
    let no_zone = place(28.6, 77.2, "Nowhere/Zone");
    assert_eq!(field(run(request(None, no_zone))), "place");
    for kind in [SkyEventKind::Conjunction, SkyEventKind::HouseFromNatalMoon] {
        let bad = CalendarRequest {
            filter: filter(&[kind], false),
            ..request(None, delhi())
        };
        assert_eq!(field(run(bad)), "kinds");
    }
    let stale_settings = CalendarRequest {
        settings: CalcSettings {
            schema_version: 99,
            ..CalcSettings::default()
        },
        ..request(None, delhi())
    };
    assert_eq!(
        run(stale_settings).expect_err("settings").code,
        "unsupported_schema_version"
    );

    // 9. Bütçe: en ağır ay (bütün türler, Ay, açık harita), ortanca.
    let mut runs: Vec<Duration> = (0..5)
        .map(|_| {
            let start = Instant::now();
            run(full_request.clone()).expect("timed month");
            start.elapsed()
        })
        .collect();
    runs.sort();
    eprintln!("month_calendar median: {:?}", runs[2]);
    assert!(runs[2] <= MONTH_BUDGET, "{:?}", runs[2]);
}
````

````diff
diff --git a/app/src-tauri/tests/ui_fixture.rs b/app/src-tauri/tests/ui_fixture.rs
index 8d05736..80f8f0c 100644
--- a/app/src-tauri/tests/ui_fixture.rs
+++ b/app/src-tauri/tests/ui_fixture.rs
@@ -499,6 +499,78 @@ fn ui_fixture_is_up_to_date() {
             "the fixture range has a {kind:?}"
         );
     }
+    // Geçiş takvimi (Plan 29). Fikstürün sabit saatinde (2026-11-25, Delhi) dört ay: bulunulan
+    // yerin bugünkü ayı varsayılan süzgeçle; aynı ay bütün türler, Ay ve açık haritayla; sonraki
+    // ay; kutup dairesinde Güneş'in ay içinde geri döndüğü ay (Tromsø, Ocak 2027).
+    let calendar = {
+        use jyotish_core::calendar::{
+            CalendarFilter, CALENDAR_DEFAULT_KINDS, CALENDAR_KINDS, CALENDAR_MAX_EVENTS,
+        };
+        use jyotish_desk_lib::calendar_commands::{
+            month_calendar_blocking, CalendarRequest, YearMonth, CALENDAR_YEARS,
+        };
+        let request = CalendarRequest {
+            month: None,
+            place: muhurta_place.clone(),
+            filter: CalendarFilter {
+                kinds: CALENDAR_DEFAULT_KINDS.to_vec(),
+                moon: false,
+            },
+            settings: opened.calc_settings.clone(),
+            natal: None,
+        };
+        let month_of = |request: CalendarRequest| {
+            month_calendar_blocking(&engine, request, fixture_now).expect("month calendar")
+        };
+        let month = month_of(request.clone());
+        assert_eq!((month.days.len(), month.today), (30, Some(25)));
+        let request_full = CalendarRequest {
+            filter: CalendarFilter {
+                kinds: CALENDAR_KINDS.to_vec(),
+                moon: true,
+            },
+            natal: Some(module_input.clone()),
+            ..request.clone()
+        };
+        let month_full = month_of(request_full.clone());
+        assert!(month_full.natal && !month_full.truncated);
+        let month_next = month_of(CalendarRequest {
+            month: month.next,
+            ..request.clone()
+        });
+        assert_eq!(month_next.today, None);
+        let month_polar = month_of(CalendarRequest {
+            month: Some(YearMonth {
+                year: 2027,
+                month: 1,
+            }),
+            place: MuhurtaPlace {
+                lat: 69.6492,
+                lon: 18.9553,
+                tz: TzChoice::Iana("Europe/Oslo".into()),
+            },
+            ..request.clone()
+        });
+        assert!(month_polar
+            .days
+            .iter()
+            .any(|d| d.facts.sunrise_jd_ut.is_none()));
+        json!({
+            "request": request,
+            "request_full": request_full,
+            "month": month,
+            "month_full": month_full,
+            "month_next": month_next,
+            "month_polar": month_polar,
+            "ids": {
+                "kinds": CALENDAR_KINDS,
+                "default_kinds": CALENDAR_DEFAULT_KINDS,
+                "max_events": CALENDAR_MAX_EVENTS,
+                "min_year": CALENDAR_YEARS.start(),
+                "max_year": CALENDAR_YEARS.end(),
+            },
+        })
+    };
     // Bhava tablosu ve Sudarshana chakra (Plan 25). Açık haritanın girdisiyle (tam burç) tablo ve
     // chakra; aynı an ve yer için Sripati tablosu (bhava ortalarıyla) ile Placidus haritası ve
     // tablosu (chalit görünümünün verisi: üç graha burç evinden başka bir bhavadadır); kutup
@@ -626,6 +698,7 @@ fn ui_fixture_is_up_to_date() {
             "max_number": PRASHNA_NUMBER_MAX,
             "max_question_chars": MAX_QUESTION_CHARS,
         },
+        "calendar": calendar,
         "search": {
             "query": search_query,
             "page": search_page,
````

- [ ] **Step 2: Testlerin derlenmediğini gör**

Run: `cargo test -p jyotish-desk --lib calendar_commands 2>&1 | grep -E "^error" | sort | uniq -c | head`
Expected: derleme hatası: `lib.rs`'teki `calendar_commands::month_calendar` bulunamaz ve test bloğunun adları (`YearMonth`, `weekday`, `civil`, `invalid`, `calendar_error`) çözülemez. Hiçbir test koşmaz.

- [ ] **Step 3: Komut**

**`app/src-tauri/src/calendar_commands.rs`: uygulama, test bloğunun üstüne eklenir (arada bir boş satır):**

````rust
//! Geçiş takvimi komutu (yol haritası §4.11, Plan 29): bir ayın takvim günleri, her günün gün
//! doğumundaki tithisi ve Ay nakshatrası ile o güne düşen gök olayları. Ortak kalıp (K1–K3):
//! hesap motor iş parçacığında, komut gövdesi `blocking` havuzunda, hata `CommandError`; sonuç
//! saklanmaz. "Bugün" Rust'ta okunur; arayüz tarih hesabı yapmaz (C4): önceki ve sonraki ay da
//! buradan döner.
//!
//! Ay dilim dilim hesaplanır (en çok yedi gün): her dilim ayrı bir motor işidir ve aralarında
//! motor iş parçacığı bırakılır; araya giren harita hesabı bir dilimden uzun beklemez.
//!
//! Gizlilik: yer (bulunulan yer dahil) ve açık haritanın doğum verisi kişisel veridir;
//! loglanmaz ve hata parametresine girmez. Bu dosyanın ürettiği girdi hataları yalnızca alan
//! adı taşır (`invalid_calendar_input`).

use std::sync::Arc;

use atlas::Timestamp;
use ephemeris::{Engine, SwissEph};
use jyotish_core::calendar::{
    calendar_slice, calendar_slices, CalendarFilter, CalendarSpec, DayFacts, DaySpan,
    CALENDAR_MAX_EVENTS, CALENDAR_METHOD,
};
use jyotish_core::{CalcSettings, ChartInput, ChartWarning, Positions, Vara};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::client_commands::blocking;
use crate::commands::CommandError;
use crate::muhurta_commands::{day_start_jd, offset_at, today, MuhurtaPlace, J2000_JD};
use crate::search_commands::SkyEventView;

/// Takvimin kabul ettiği yıllar. Paketlenen ephemeris dosyaları 1800–2399'u kapsar; dışındaki
/// aylar Moshier hassasiyetindedir ve sonuç `LowPrecision` uyarısını taşır.
pub const CALENDAR_YEARS: std::ops::RangeInclusive<i32> = 1..=2999;
/// Gün panchangı paneline verilen an, olguların okunduğu andan bu kadar sonradır (bir dakika):
/// an kesinlikle o gün doğumuyla başlayan günün içindedir.
const PANCHANG_ANCHOR_DAYS: f64 = 60.0 / 86_400.0;

/// Bir takvim ayı.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct YearMonth {
    pub year: i32,
    /// 1..=12.
    pub month: u8,
}

/// Takvimin girdisi. `month` boşsa yerin bugünkü ayıdır. `place` günlerin sınırını (yerel gece
/// yarısı), gün doğumunu ve saatlerin gösterileceği dilimi belirler. `natal` açık haritanın
/// modül girdisidir (doğum boylamına gelişler için; o tür seçili değilse gönderilmez);
/// `settings` açık haritanın hesap ayarıdır.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct CalendarRequest {
    pub month: Option<YearMonth>,
    pub place: MuhurtaPlace,
    pub filter: CalendarFilter,
    pub settings: CalcSettings,
    pub natal: Option<ChartInput>,
}

/// Bir takvim günü. Saatler `offset_secs` ile gösterilir; olaylar kendi ofsetlerini taşır (yaz
/// saati gün içinde değişebilir).
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct CalendarDayView {
    /// `YYYY-MM-DD`.
    pub date: String,
    /// Ayın günü, 1..=31.
    pub day: u8,
    /// Takvim gününün hafta günü.
    pub weekday: Vara,
    pub span: DaySpan,
    /// Gün doğumundaki tithi ve Ay nakshatrası (gün doğumu yoksa 06:00 yerel ortalama saat).
    pub facts: DayFacts,
    /// `facts.jd_ut` anındaki UTC ofseti.
    pub offset_secs: i32,
    /// Gün panchangı paneline verilecek an: o gün doğumuyla başlayan günün içindedir.
    pub panchang_jd_ut: f64,
    /// Zaman sırasıyla.
    pub events: Vec<SkyEventView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct CalendarView {
    pub month: YearMonth,
    /// Önceki ve sonraki ay; yıl aralığının ucunda `None`.
    pub prev: Option<YearMonth>,
    pub next: Option<YearMonth>,
    /// Yerin bugünkü günü bu aydaysa ayın günü.
    pub today: Option<u8>,
    pub days: Vec<CalendarDayView>,
    /// Olay sayısı ayın sınırını (`CALENDAR_MAX_EVENTS`) aştı: sonraki olaylar listede yoktur.
    pub truncated: bool,
    /// Doğum boylamına gelişler arandı (tür seçili ve açık haritanın girdisi verildi).
    pub natal: bool,
    /// `LowPrecision` ya da boş.
    pub warnings: Vec<ChartWarning>,
    pub method: String,
}

fn invalid(field: &str) -> CommandError {
    CommandError::new("invalid_calendar_input", &[("field", field.to_owned())])
}

/// Çekirdeğin ve muhurta yardımcılarının alan adlı hatasını takvimin koduna çevirir; alan adı
/// sabit bir kümeden seçilir (değer taşınmaz). Öteki hatalar (hesap, motor) olduğu gibi geçer.
fn calendar_error(e: CommandError) -> CommandError {
    if e.code != "invalid_search_input" && e.code != "invalid_muhurta_input" {
        return e;
    }
    let field = match e.params.get("field").map(String::as_str) {
        Some("place") => "place",
        Some("kinds") => "kinds",
        Some("month") => "month",
        _ => "range",
    };
    invalid(field)
}

fn civil(year: i32, month: u8, day: u8) -> Option<atlas::Date> {
    atlas::Date::new(
        i16::try_from(year).ok()?,
        i8::try_from(month).ok()?,
        i8::try_from(day).ok()?,
    )
    .ok()
}

impl YearMonth {
    fn is_valid(self) -> bool {
        CALENDAR_YEARS.contains(&self.year) && (1..=12).contains(&self.month)
    }

    /// `step` ay sonrası (−1 ya da +1); yıl aralığının dışına çıkıyorsa `None`.
    fn shifted(self, step: i32) -> Option<YearMonth> {
        let index = self.year * 12 + i32::from(self.month) - 1 + step;
        let next = YearMonth {
            year: index.div_euclid(12),
            month: u8::try_from(index.rem_euclid(12)).ok()? + 1,
        };
        next.is_valid().then_some(next)
    }
}

/// Bir ayın takvim günleri: (tarih, günün başı). Yerin saat diliminde var olmayan gün (takvimden
/// atlanan gün) listede yer almaz. Son öğe sonraki ayın ilk günüdür (son günün bitişi).
fn month_days(
    month: YearMonth,
    place: &MuhurtaPlace,
) -> Result<Vec<(atlas::Date, f64)>, CommandError> {
    let first = civil(month.year, month.month, 1).ok_or_else(|| invalid("month"))?;
    let mut out = Vec::new();
    let mut date = first;
    for _ in 0..first.days_in_month() {
        // Çözülemeyen gün başı: o gün yerin takviminde yoktur (saat dilimi bir günü atlamıştır).
        if let Ok(start) = day_start_jd(&date.to_string(), place, "month") {
            out.push((date, start));
        }
        date = date.tomorrow().map_err(|_| invalid("month"))?;
    }
    if out.is_empty() {
        return Err(invalid("month"));
    }
    let end = day_start_jd(&date.to_string(), place, "month").map_err(calendar_error)?;
    out.push((date, end));
    Ok(out)
}

/// Takvim tarihinin hafta günü.
fn weekday(date: atlas::Date) -> Vara {
    Vara::ALL[usize::from(date.weekday().to_sunday_zero_offset().unsigned_abs())]
}

/// `month_calendar` gövdesi; `now` testte sabitlenir. Bloklar; dilimleri sırayla yürütür ve
/// her dilimden sonra motor iş parçacığını bırakır.
pub fn month_calendar_blocking(
    engine: &Engine,
    request: CalendarRequest,
    now: Timestamp,
) -> Result<CalendarView, CommandError> {
    // Yer, aydan ve motordan önce denetlenir: geçersiz yer hangi ayla sorulursa sorulsun
    // `place`'tir.
    offset_at(&request.place, J2000_JD).map_err(calendar_error)?;
    let spec = CalendarSpec {
        lat: request.place.lat,
        lon: request.place.lon,
        settings: request.settings.clone(),
        filter: request.filter.clone(),
    };
    // Süzgeç ve ayar denetimi motordan önce.
    spec.validate()
        .map_err(|e| calendar_error(CommandError::from(e)))?;
    let today = today(&request.place, now).map_err(calendar_error)?;
    let today: atlas::Date = today.parse().map_err(|_| invalid("range"))?;
    let month = match request.month {
        Some(m) => m,
        None => YearMonth {
            year: i32::from(today.year()),
            month: today.month().unsigned_abs(),
        },
    };
    if !month.is_valid() {
        return Err(invalid("month"));
    }
    let days = month_days(month, &request.place)?;
    let spans: Vec<DaySpan> = days
        .windows(2)
        .map(|w| DaySpan {
            start_jd_ut: w[0].1,
            end_jd_ut: w[1].1,
        })
        .collect();

    let natal: Option<Arc<Positions>> = match request.natal.clone() {
        Some(input) => Some(Arc::new(engine.call(move |se: &mut SwissEph| {
            jyotish_core::compute_positions(se, &input)
        })??)),
        None => None,
    };
    let searched_natal = natal.is_some()
        && request
            .filter
            .event_spec(0.0, 1.0, true)
            .is_some_and(|s| s.kinds.iter().any(|k| k.needs_natal()));
    let spec = Arc::new(spec);
    let spans = Arc::new(spans);
    let mut found = Vec::with_capacity(spans.len());
    let mut truncated = false;
    let mut low_precision = false;
    let mut left = CALENDAR_MAX_EVENTS;
    for (from, to) in calendar_slices(spans.len()) {
        let (c, s, n) = (Arc::clone(&spec), Arc::clone(&spans), natal.clone());
        let slice = engine
            .call(move |se: &mut SwissEph| {
                calendar_slice(se, &c, n.as_deref(), &s[from..to], left)
            })?
            .map_err(|e| calendar_error(CommandError::from(e)))?;
        left -= slice.days.iter().map(|d| d.events.len()).sum::<usize>();
        truncated |= slice.truncated;
        low_precision |= slice.low_precision;
        found.extend(slice.days);
    }

    let days = days
        .iter()
        .zip(found)
        .map(|((date, _), day)| {
            let events = day
                .events
                .into_iter()
                .map(|event| {
                    Ok(SkyEventView {
                        offset_secs: offset_at(&request.place, event.jd_ut)
                            .map_err(calendar_error)?,
                        event,
                    })
                })
                .collect::<Result<Vec<_>, CommandError>>()?;
            Ok(CalendarDayView {
                date: date.to_string(),
                day: date.day().unsigned_abs(),
                weekday: weekday(*date),
                span: day.span,
                facts: day.facts,
                offset_secs: offset_at(&request.place, day.facts.jd_ut).map_err(calendar_error)?,
                panchang_jd_ut: (day.facts.jd_ut + PANCHANG_ANCHOR_DAYS)
                    .min(0.5 * (day.facts.jd_ut + day.span.end_jd_ut)),
                events,
            })
        })
        .collect::<Result<Vec<_>, CommandError>>()?;
    let in_month =
        i32::from(today.year()) == month.year && today.month().unsigned_abs() == month.month;
    Ok(CalendarView {
        month,
        prev: month.shifted(-1),
        next: month.shifted(1),
        today: in_month.then(|| today.day().unsigned_abs()),
        days,
        truncated,
        natal: searched_natal,
        warnings: if low_precision {
            vec![ChartWarning::LowPrecision]
        } else {
            Vec::new()
        },
        method: CALENDAR_METHOD.to_owned(),
    })
}

/// Geçiş takvimi: bir ayın takvim günleri (yerin duvar saatiyle gece yarısından gece yarısına),
/// her günün gün doğumundaki tithisi ve Ay nakshatrası ile o güne düşen gök olayları. `month`
/// boşsa yerin bugünkü ayı döner ("bugün" Rust'ta okunur). Yer ve girdi loglanmaz.
#[tauri::command]
#[specta::specta]
pub async fn month_calendar(
    engine: State<'_, Arc<Engine>>,
    request: CalendarRequest,
) -> Result<CalendarView, CommandError> {
    let engine = Arc::clone(&engine);
    blocking(move || month_calendar_blocking(&engine, request, Timestamp::now())).await
}
````

- [ ] **Step 4: Rust testleri; bağlamaları ve fikstürü yeniden üret (C10)**

```bash
cargo fmt --all --check
cargo test -p jyotish-desk --lib calendar_commands 2>&1 | grep -E "^test result"
cargo test -p jyotish-desk --test calendar -- --nocapture 2>&1 | grep -E "month_calendar median|^test result"
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep -E "^test result"
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep -E "^test result"
cargo test -p jyotish-desk --test bindings --test ui_fixture 2>&1 | grep -E "^test result"
git diff --stat -- app/ui/bindings.ts app/ui/test/fixtures/delhi.json | cat
grep -c "monthCalendar" app/ui/bindings.ts
grep -ciE 'k(a|ā|aa)la' app/ui/test/fixtures/delhi.json
```
Expected: fmt çıktısız; birim testleri `3 passed`; entegrasyon testi `1 passed` ve `month_calendar median:` satırı 150 ms'nin altında (ölçüm: 12–17 ms; aşan değer testi düşürür); üretim komutları `1 passed` ve `2 passed`; denetim koşusu `1 passed` ve `2 passed`; `bindings.ts` 96 satır kazanır, `delhi.json` tek satırı değişir (`2 files changed, 97 insertions(+), 1 deletion(-)`); `monthCalendar` sayacı `1`; son sayaç `0`. Entegrasyon testi bir çapraz denetimde düşerse (takvimin tithisi gün panchangından, olayları olay aramasından ayrışırsa) var olan iki komut "düzeltilmez": fark ilerleme notlarına yazılır ve Task 1–2'nin adımları denetlenir.

- [ ] **Step 5: Arayüz testleri: fikstürün `calendar` alanı ve sarmalayıcı**

**Yeni dosya:** `app/ui/test/calendarFixture.test.ts`

````ts
import { calendarFixture } from "./calendarFixture";
import { fixture } from "./fixture";

test("the calendar fixture is a real month for the fixture place with the facts of every day", () => {
  const c = calendarFixture();
  expect(c.request.place).toEqual(fixture().muhurta.place);
  expect(c.request.month).toBeNull();
  expect(c.request.natal).toBeNull();
  expect(c.month.month).toEqual({ year: 2026, month: 11 });
  expect([c.month.prev, c.month.next]).toEqual([
    { year: 2026, month: 10 },
    { year: 2026, month: 12 },
  ]);
  expect(c.month.today).toBe(25);
  expect(c.month.days.map((d) => d.day)).toEqual(Array.from({ length: 30 }, (_, i) => i + 1));
  expect(c.month.days[0]).toMatchObject({ date: "2026-11-01", weekday: "Sunday" });
  // Bugünün gün doğumu olguları gün panchangı fikstürüyle aynıdır (aynı an, aynı yer).
  const today = c.month.days[24];
  const day = fixture().muhurta.day.day;
  expect(today.facts.sunrise_jd_ut).toBeCloseTo(day.sunrise_jd_ut as number, 6);
  expect([today.facts.tithi, today.facts.nakshatra]).toEqual([day.tithis[0].value.no, day.nakshatras[0].value.no]);
  expect(today.weekday).toBe(day.vara);
});

test("the four months cover the default filter, the full filter, the next month and a polar month", () => {
  const c = calendarFixture();
  const kinds = (view: typeof c.month) => new Set(view.days.flatMap((d) => d.events.map((e) => e.event.kind)));
  expect([...kinds(c.month)].every((k) => c.ids.default_kinds.includes(k))).toBe(true);
  expect(kinds(c.month_full).has("TransitToNatal")).toBe(true);
  expect([c.month.natal, c.month_full.natal]).toEqual([false, true]);
  expect(c.request_full.filter).toEqual({ kinds: c.ids.kinds, moon: true });
  expect(c.request_full.natal).not.toBeNull();
  expect(c.month_next.month).toEqual(c.month.next);
  expect(c.month_next.today).toBeNull();
  const risen = c.month_polar.days.filter((d) => d.facts.sunrise_jd_ut !== null).map((d) => d.day);
  expect(risen).toEqual([19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31]);
  expect(c.ids).toMatchObject({ max_events: 300, min_year: 1, max_year: 2999 });
  expect(c.ids.kinds).toHaveLength(7);
});

test("calendar fixture copies are independent", () => {
  const a = calendarFixture();
  a.month.days[0].events.push(a.month.days[8].events[0]);
  expect(calendarFixture().month.days[0].events).toHaveLength(0);
});
````

````diff
diff --git a/app/ui/api.test.ts b/app/ui/api.test.ts
index 3f5d97b..c03c923 100644
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -46,6 +46,7 @@ vi.mock("./bindings", () => ({
     chartBrief: vi.fn(),
     dayPanchang: vi.fn(),
     muhurtaSearch: vi.fn(),
+    monthCalendar: vi.fn(),
     loadMuhurtaFilters: vi.fn(),
     saveMuhurtaFilters: vi.fn(),
     createPrashna: vi.fn(),
@@ -156,6 +157,7 @@ test("every wrapper forwards its arguments to the command of the same name", asy
     ["chartBrief", ["00000000-0000-4000-8000-000000000012"]],
     ["dayPanchang", ["Now", { lat: 28.6, lon: 77.2, tz: "Lmt" }, { ayanamsa: "Lahiri" }, { nakshatra: 15, sign: "Libra" }]],
     ["muhurtaSearch", [{ from_date: null, to_date: null }, { ayanamsa: "Lahiri" }, { cursor: { chunk: 1 } }]],
+    ["monthCalendar", [{ month: { year: 2026, month: 11 }, filter: { kinds: ["NewMoon"], moon: false }, natal: null }]],
     ["loadMuhurtaFilters", []],
     ["saveMuhurtaFilters", [[{ name: "Yolculuk", criteria: {} }]]],
     ["createPrashna", [{ client_id: null, at: "Now", number: 54, question: "", focus_house: 7 }]],
````

- [ ] **Step 6: Testlerin düştüğünü gör**

Run: `pnpm --dir app exec vitest run ui/test/calendarFixture.test.ts ui/api.test.ts 2>&1 | grep -E "Test Files|Tests |Error:" | head -5`
Expected: `Test Files  2 failed (2)`, `Tests  1 failed | 6 passed (7)`: `calendarFixture.test.ts` içe aktarımını çözemez (`Failed to resolve import "./calendarFixture"`) ve `api.test.ts`'in sarmalayıcı tablosu `monthCalendar` satırında düşer (`TypeError: call is not a function`).

- [ ] **Step 7: Fikstür tipi ve sarmalayıcı**

**Yeni dosya:** `app/ui/test/calendarFixture.ts`

````ts
// Geçiş takviminin fikstürü (Plan 29): `delhi.json`'un `calendar` alanı. Ayrı dosyadır:
// `fixture.ts` paralel planlarla paylaşılır; bu dosya aynı fikstürü okur ve yalnızca takvimin
// alanını tipler. Değerler `ui_fixture.rs`'te gerçek motorla üretilir (arayüz hesap yapmaz).
import type { CalendarRequest, CalendarView, SkyEventKind } from "../bindings";
import { fixture } from "./fixture";

export interface CalendarFixture {
  /** Bulunulan yerin (Delhi) bugünkü ayı, varsayılan süzgeçle; açık haritanın girdisi gitmez. */
  request: CalendarRequest;
  /** Aynı ay: bütün türler, Ay ve açık haritanın girdisi. */
  request_full: CalendarRequest;
  /** `month_calendar(request)`: Kasım 2026; bugün ayın 25'idir. */
  month: CalendarView;
  /** `month_calendar(request_full)`: doğum boylamına gelişler ve Ay'ın girişleriyle. */
  month_full: CalendarView;
  /** Sonraki ay (Aralık 2026), varsayılan süzgeçle. */
  month_next: CalendarView;
  /** Tromsø, Ocak 2027: Güneş ayın 19'una dek doğmaz. */
  month_polar: CalendarView;
  /** Rust `CALENDAR_KINDS`, `CALENDAR_DEFAULT_KINDS`, `CALENDAR_MAX_EVENTS` ve `CALENDAR_YEARS`. */
  ids: { kinds: SkyEventKind[]; default_kinds: SkyEventKind[]; max_events: number; min_year: number; max_year: number };
}

/** Her çağrıda derin kopya (`fixture()` gibi). */
export function calendarFixture(): CalendarFixture {
  return (fixture() as unknown as { calendar: CalendarFixture }).calendar;
}
````

````diff
diff --git a/app/ui/api.ts b/app/ui/api.ts
index 8f186b3..dafd157 100644
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -128,6 +128,10 @@ export const api = {
   // aramayı durdurmak sonraki dilimi istememektir. Yer ve doğum Ay'ı kişisel veridir: loglanmaz.
   dayPanchang: (at: TransitAt, place: MuhurtaPlace, settings: CalcSettings, natal: NatalMoon | null) => wrap(() => commands.dayPanchang(at, place, settings, natal)),
   muhurtaSearch: (query: MuhurtaQuery, settings: CalcSettings, cursor: MuhurtaPageCursor | null) => wrap(() => commands.muhurtaSearch(query, settings, cursor)),
+  // Geçiş takvimi (Plan 29): bir ayın takvim günleri, gün doğumu olguları ve olayları. `month`
+  // boşsa yerin bugünkü ayıdır ("bugün" Rust'ta okunur); önceki ve sonraki ay yanıtta döner.
+  // İstek yer ve (doğum türü seçiliyse) açık haritanın doğum verisini taşır: loglanmaz.
+  monthCalendar: (request: Parameters<typeof commands.monthCalendar>[0]) => wrap(() => commands.monthCalendar(request)),
   // Kayıtlı süzgeçler: ad ve koşullar; yer, tarih ve kişi kaydedilmez. Kayıt listenin tamamını yazar.
   loadMuhurtaFilters: () => wrap(() => commands.loadMuhurtaFilters()),
   saveMuhurtaFilters: (filters: SavedMuhurtaFilter[]) => wrap(() => commands.saveMuhurtaFilters(filters)),
````

- [ ] **Step 8: Testler ve kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -1
cargo test -p jyotish-desk --lib 2>&1 | grep -E "^test result"
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
pnpm --dir app exec vitest run ui/test/calendarFixture.test.ts ui/api.test.ts 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
```
Expected: fmt çıktısız; clippy `Finished`; lib `61 passed`; sayaç `0`; iki dosya `2 passed (2)`, `13 passed (13)`; Vitest `98 passed (98)`, `976 passed (976)`; `✓ built`.

- [ ] **Step 9: Commit**

```bash
git add app/src-tauri/src/calendar_commands.rs app/src-tauri/src/lib.rs app/src-tauri/tests/calendar.rs app/src-tauri/tests/ui_fixture.rs app/ui/bindings.ts app/ui/test/fixtures/delhi.json app/ui/test/calendarFixture.ts app/ui/test/calendarFixture.test.ts app/ui/api.ts app/ui/api.test.ts
git diff --cached --stat | tail -1
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(app): month_calendar command in week slices, bindings and the calendar fixture" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: `10 files changed, 1014 insertions(+), 1 deletion(-)`; tarama çıktısız; `git status --short` boş.

---

### Task 4: Arayüz: Transit takvimi paneli (ay ızgarası, hücre içeriği, ay seçimi, yer, durumlar ve metinler)

Yol haritası §5 Plan 29 görev (4) ve (7) ("ızgara ve hücre içeriği", "i18n: hafta başlangıcı, ay adları", "boş ve hata durumları"); K11, K12; D14–D18, D25. Bu görevde ızgara düz bir tablodur ve kaydırma kabı klavye odağı alır; hücrelerin klavyesi ve gün ayrıntısı Task 5'tedir, süzgecin denetimleri Task 6'dadır (süzgeç bu görevde panel ayarından okunur ve varsayılanıyla istenir).

**Files:**
- Create: `app/ui/calendar/model.ts`, `app/ui/calendar/model.test.ts`, `app/ui/calendar/useCalendar.ts`, `app/ui/calendar/useCalendar.test.tsx`, `app/ui/calendar/CalendarGrid.tsx`, `app/ui/calendar/CalendarPanel.tsx`, `app/ui/calendar/CalendarPanel.test.tsx`, `app/ui/calendar/calendar.css`
- Modify: `app/ui/layout/model.ts`, `app/ui/layout/model.test.ts`, `app/ui/panels/registry.ts`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`
- Test: `model.test.ts` (6), `useCalendar.test.tsx` (3), `CalendarPanel.test.tsx` (12); `layout/model.test.ts` (iki beklenti)

**Interfaces:**
- Consumes: Task 3'ün `api.monthCalendar`, bağlama tipleri ve `calendarFixture()`; `modules/useModule` (`useModule`, `useLastData`), `modules/input` (`chartInput`), `modules/muhurta` (`VARAS`, `muhurtaPlace`), `panels/MuhurtaPlace` (`usePanelPlace`, `PlaceControls`, `PlaceMissing`), `panels/states` (`PanelError`, `PanelSkeleton`, `usePanelLoadPhase`), `panels/format` (`formatJdClock`), `panels/labels` (`nakshatraName`, `tithiInPaksha`, `tithiName`, `tithiShort`), `chart/adapter` (`signIndex`), `i18n` (`currentLanguage`, `Language`), `session/store`.
- Produces:
  - `calendar/model.ts`: `CALENDAR_KINDS`, `DEFAULT_CALENDAR_KINDS`, `MIN_YEAR`, `MAX_YEAR`, `CELL_EVENTS`, `WEEK_START`, `weekdayOrder(start)`, `weekRows(days, start)`, `parseFilter(settings)`, `eventShort(t, e)`, `eventLong(t, e)`, `eventWhere(t, e)`.
  - `calendar/useCalendar.ts`: `useCalendar(request: CalendarRequest | null, epoch: number, scope: string): CalendarState` (`shown`: yeni yanıt gelene dek bir önceki ay).
  - `calendar/CalendarGrid.tsx`: `CalendarGrid({ view })`, `NO_SUNRISE_MARK`.
  - `calendar/CalendarPanel.tsx`: `CalendarPanel({ panel })`; panel türü `calendar`; i18n `panel.calendar`, `calendar.*`, `errors.invalid_calendar_input`.

- [ ] **Step 1: Testler: model, kanca, panel ve katalog**

Fikstürün ayı Kasım 2026'dır (Delhi; bugün ayın 25'i, 1 Kasım Pazar). Beklenen metinler fikstürdeki gerçek değerlerdir: 25 Kasım gün doğumunda Krishna 1 (Pratipada), Ay Rohini'de; 13:41'de Merkür Vishakha'ya girer.

**Yeni dosya:** `app/ui/calendar/model.test.ts`

````ts
import { expect, test } from "vitest";
import type { CalendarDayView, SkyEvent } from "../bindings";
import { i18n, LANGUAGES } from "../i18n";
import { calendarFixture } from "../test/calendarFixture";
import { CALENDAR_KINDS, CELL_EVENTS, DEFAULT_CALENDAR_KINDS, eventLong, eventShort, eventWhere, MAX_YEAR, MIN_YEAR, parseFilter, WEEK_START, weekdayOrder, weekRows } from "./model";

const t = i18n.getFixedT("tr");
const days = (row: (CalendarDayView | null)[]) => row.map((d) => d?.day ?? null);

test("the kind lists and the year range are Rust's", () => {
  const { ids } = calendarFixture();
  expect([...CALENDAR_KINDS]).toEqual(ids.kinds);
  expect([...DEFAULT_CALENDAR_KINDS]).toEqual(ids.default_kinds);
  expect([MIN_YEAR, MAX_YEAR]).toEqual([ids.min_year, ids.max_year]);
  expect(CELL_EVENTS).toBe(3);
});

test("the week starts on Monday in Turkish and French and on Sunday in English", () => {
  expect(Object.keys(WEEK_START).sort()).toEqual([...LANGUAGES].sort());
  expect(WEEK_START).toEqual({ tr: "Monday", en: "Sunday", fr: "Monday" });
  expect(weekdayOrder("Monday")).toEqual(["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"]);
  expect(weekdayOrder("Sunday")).toEqual(["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"]);
});

test("days sit in the column of their weekday", () => {
  const month = calendarFixture().month.days;
  // 1 Kasım 2026 Pazar'dır. Pazartesi başlayan haftada ilk satırın yalnızca son hücresi doludur.
  const monday = weekRows(month, "Monday");
  expect(monday).toHaveLength(6);
  expect(days(monday[0])).toEqual([null, null, null, null, null, null, 1]);
  expect(days(monday[1])).toEqual([2, 3, 4, 5, 6, 7, 8]);
  expect(days(monday[5])).toEqual([30, null, null, null, null, null, null]);
  // Pazar başlayan haftada ay beş satırdır.
  const sunday = weekRows(month, "Sunday");
  expect(sunday).toHaveLength(5);
  expect(days(sunday[0])).toEqual([1, 2, 3, 4, 5, 6, 7]);
  expect(days(sunday[4])).toEqual([29, 30, null, null, null, null, null]);
  // Her gün tam bir kez yerleşir.
  expect(monday.flat().filter((d) => d !== null)).toHaveLength(30);
  expect(weekRows([], "Monday")).toEqual([]);
});

test("a day the time zone skipped leaves its cell empty and the rest stay in their columns", () => {
  // Takvimden atlanan gün (Rust o günü listeye koymaz): 11'i olmayan bir ay.
  const month = calendarFixture().month.days.filter((d) => d.day !== 11);
  const rows = weekRows(month, "Monday");
  expect(days(rows[2])).toEqual([9, 10, null, 12, 13, 14, 15]);
  expect(rows).toHaveLength(6);
});

test("the filter is read tolerantly from the panel settings", () => {
  expect(parseFilter({})).toEqual({ kinds: [...DEFAULT_CALENDAR_KINDS], moon: false });
  // Tanınmayan türler atılır; sıra Rust'ın sırasıdır.
  expect(parseFilter({ kinds: ["TransitToNatal", "Conjunction", "SignIngress", 7], moon: true })).toEqual({ kinds: ["SignIngress", "TransitToNatal"], moon: true });
  // Boş küme geçerli bir seçimdir (yalnızca olgular); bozuk değer varsayılandır.
  expect(parseFilter({ kinds: [] }).kinds).toEqual([]);
  expect(parseFilter({ kinds: "all", moon: "yes" })).toEqual({ kinds: [...DEFAULT_CALENDAR_KINDS], moon: false });
});

test("an event has a short cell text, a full name and a place", () => {
  // Fikstürdeki gerçek bir olaydan kurulur: `SkyEvent` yeni bir alan kazanırsa test derlenmeyi sürdürür.
  const base = calendarFixture().month.days[8].events[0].event;
  const event = (kind: SkyEvent["kind"], grahas: SkyEvent["grahas"], sign: SkyEvent["sign"] = null, nakshatra: number | null = null): SkyEvent => ({ ...base, kind, grahas, sign, nakshatra, house: null });
  const rows: [SkyEvent, string, string, string][] = [
    [event("SignIngress", ["Sun"], "Scorpio"), "Gü → Akr", "Güneş: burç girişi", "Akr"],
    [event("NakshatraIngress", ["Mercury"], null, 16), "Me → Vishakha", "Merkür: nakshatra girişi", "Vishakha"],
    [event("StationRetrograde", ["Jupiter"], "Leo"), "Jü R başlar", "Jüpiter: R başlıyor", "Asl"],
    [event("StationDirect", ["Venus"], "Virgo"), "Ve R biter", "Venüs: R bitiyor", "Baş"],
    [event("NewMoon", ["Sun", "Moon"], "Libra", 16), "Yeni ay", "Yeni ay", "Ter · Vishakha"],
    [event("FullMoon", ["Sun", "Moon"], "Taurus", 3), "Dolunay", "Dolunay", "Boğ · Krittika"],
    [event("TransitToNatal", ["Mars", "Moon"], "Libra"), "Ma → doğum Ay", "Mars, doğum Ay boylamında", "Ter"],
    [event("TransitToNatal", ["Sun", "Sun"], "Sagittarius"), "Gü dönüşü", "Güneş dönüşü", "Yay"],
  ];
  for (const [e, short, long, where] of rows) {
    expect([eventShort(t, e), eventLong(t, e), eventWhere(t, e)], e.kind).toEqual([short, long, where]);
  }
  // Burcu ya da nakshatrası gelmeyen giriş çizgiyle yazılır.
  expect(eventShort(t, event("SignIngress", ["Sun"]))).toBe("Gü → —");
  expect(eventWhere(t, event("SignIngress", ["Sun"]))).toBe("");
});
````

**Yeni dosya:** `app/ui/calendar/useCalendar.test.tsx`

````tsx
import { act, render, screen } from "@testing-library/react";
import { beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { monthCalendar: vi.fn() } }));

import { api, type Result } from "../api";
import type { CalendarRequest, CalendarView } from "../bindings";
import { calendarFixture } from "../test/calendarFixture";
import { useCalendar } from "./useCalendar";

const FX = calendarFixture();

/** Elle çözülen söz: yanıtın geliş sırasını test belirler. */
function deferred() {
  let resolve!: (r: Result<CalendarView>) => void;
  const promise = new Promise<Result<CalendarView>>((r) => (resolve = r));
  return { promise, resolve };
}

function Probe({ request, epoch = 0, scope = "1:p" }: { request: CalendarRequest | null; epoch?: number; scope?: string }) {
  const c = useCalendar(request, epoch, scope);
  return (
    <span data-testid="state">
      {c.loading ? "loading" : c.error ? `error:${c.error.code}` : "settled"}|{c.shown ? `${c.shown.month.year}-${c.shown.month.month}` : "none"}
    </span>
  );
}

const state = () => screen.getByTestId("state").textContent;
const asked = (month: CalendarRequest["month"]): CalendarRequest => ({ ...FX.request, month });

beforeEach(() => vi.mocked(api.monthCalendar).mockReset());

test("without a request nothing is asked; a request is asked once", async () => {
  vi.mocked(api.monthCalendar).mockResolvedValue({ status: "ok", data: FX.month });
  const { rerender } = render(<Probe request={null} />);
  expect(state()).toBe("settled|none");
  expect(api.monthCalendar).not.toHaveBeenCalled();
  rerender(<Probe request={FX.request} />);
  expect(state()).toBe("loading|none");
  await act(async () => {});
  expect(state()).toBe("settled|2026-11");
  expect(vi.mocked(api.monthCalendar).mock.calls).toEqual([[FX.request]]);
  // Aynı istekle yeniden çizim yeni istek göndermez.
  rerender(<Probe request={{ ...FX.request }} />);
  expect(api.monthCalendar).toHaveBeenCalledTimes(1);
});

test("the last month stays shown while the next one loads, and a late answer to an older request is dropped", async () => {
  const [first, december, january] = [deferred(), deferred(), deferred()];
  vi.mocked(api.monthCalendar).mockReturnValueOnce(first.promise).mockReturnValueOnce(december.promise).mockReturnValueOnce(january.promise);
  const { rerender } = render(<Probe request={asked(null)} />);
  await act(async () => first.resolve({ status: "ok", data: FX.month }));
  expect(state()).toBe("settled|2026-11");
  // İki hızlı adım: Aralık istenir, yanıtı gelmeden Ocak istenir.
  rerender(<Probe request={asked({ year: 2026, month: 12 })} />);
  expect(state()).toBe("loading|2026-11");
  rerender(<Probe request={asked({ year: 2027, month: 1 })} />);
  expect(state()).toBe("loading|2026-11");
  // Aralık'ın geç gelen yanıtı gösterilmez.
  await act(async () => december.resolve({ status: "ok", data: FX.month_next }));
  expect(state()).toBe("loading|2026-11");
  await act(async () => january.resolve({ status: "ok", data: FX.month_polar }));
  expect(state()).toBe("settled|2027-1");
  expect(api.monthCalendar).toHaveBeenCalledTimes(3);
});

test("a new epoch reads the same request again; an error forgets the shown month; another scope starts empty", async () => {
  vi.mocked(api.monthCalendar).mockResolvedValue({ status: "ok", data: FX.month });
  const { rerender } = render(<Probe request={FX.request} />);
  await act(async () => {});
  rerender(<Probe request={FX.request} epoch={1} />);
  expect(state()).toBe("loading|2026-11");
  await act(async () => {});
  expect(api.monthCalendar).toHaveBeenCalledTimes(2);

  vi.mocked(api.monthCalendar).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  rerender(<Probe request={asked({ year: 2026, month: 12 })} epoch={1} />);
  await act(async () => {});
  expect(state()).toBe("error:calc|none");

  // Başka harita ya da panel: önceki ay gösterilmez.
  rerender(<Probe request={FX.request} epoch={2} scope="2:p" />);
  expect(state()).toBe("loading|none");
  await act(async () => {});
  expect(state()).toBe("settled|2026-11");
});
````

**Yeni dosya:** `app/ui/calendar/CalendarPanel.test.tsx`

````tsx
import { act, fireEvent, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { monthCalendar: vi.fn(), setAppPrefs: vi.fn(), searchPlaces: vi.fn() } }));

import { api, type Result } from "../api";
import type { AppPrefs, CalendarRequest, CalendarView } from "../bindings";
import { setLanguage } from "../i18n";
import { en } from "../i18n/en";
import { fr } from "../i18n/fr";
import { tr } from "../i18n/tr";
import { MODULE_PANELS, type Panel } from "../layout/model";
import { PANEL_REGISTRY } from "../panels/registry";
import { calendarFixture } from "../test/calendarFixture";
import { fixture } from "../test/fixture";
import { renderWithStores } from "../test/render";
import { useLayoutStore } from "../view/store";
import { CalendarPanel } from "./CalendarPanel";

const FX = fixture();
const CAL = calendarFixture();
const ok = (view: CalendarView): Result<CalendarView> => ({ status: "ok", data: view });
const LAYOUT = (settings: Record<string, unknown>) => {
  const panel: Panel = { id: "cal", kind: "calendar", x: 0, y: 0, w: 9, h: 24, settings };
  return { schema_version: 1, active_layout_id: "user:t", layouts: [{ id: "user:t", name: "T", builtin: false, panels: [panel] }] };
};
const withHome = (): AppPrefs => ({ ...FX.prefs.prefs, home_place: FX.muhurta.home_place });

/** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
function LivePanel() {
  const { active } = useLayoutStore();
  return <CalendarPanel panel={active.panels[0]} />;
}

interface Options {
  opened?: ReturnType<typeof fixture>["opened"] | null;
  prefs?: AppPrefs | null;
}
function renderPanel(settings: Record<string, unknown> = {}, { opened = fixture().opened, prefs = withHome() }: Options = {}) {
  // Pano çerçevesi gibi: hata kutusu kalkınca odak panelin çerçevesine taşınır.
  return renderWithStores(
    <section className="panel" tabIndex={0} aria-label="Transit takvimi">
      <LivePanel />
    </section>,
    { layout: LAYOUT(settings), opened, prefs },
  );
}

/** Fikstürün ayları, isteğin ayına ve süzgecine göre (Rust'ın yerine). */
function answer(request: CalendarRequest): Result<CalendarView> {
  const { month } = request;
  if (month !== null && month.year === 2026 && month.month === 12) return ok(calendarFixture().month_next);
  if (month !== null && month.year === 2027 && month.month === 1) return ok(calendarFixture().month_polar);
  if (month !== null && (month.year !== 2026 || month.month !== 11)) return { status: "error", error: { code: "invalid_calendar_input", params: { field: "month" } } };
  return ok(request.filter.moon || request.filter.kinds.includes("TransitToNatal") ? calendarFixture().month_full : calendarFixture().month);
}

const grid = (name = "Kasım 2026 takvimi: gün doğumundaki tithi ve Ay nakshatrası, günün olayları") => screen.findByRole("table", { name });
const cell = (table: HTMLElement, date: string) => table.querySelector<HTMLElement>(`td[data-date="${date}"]`)!;
const lines = (td: HTMLElement) => [...td.children].map((c) => c.textContent);
const events = (td: HTMLElement) => [...td.querySelectorAll("li")].map((li) => li.textContent);
const requests = () => vi.mocked(api.monthCalendar).mock.calls.map(([r]) => r);

beforeEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
  vi.mocked(api.monthCalendar).mockImplementation(async (request) => answer(request));
  vi.mocked(api.setAppPrefs).mockResolvedValue({ status: "ok", data: null });
  vi.mocked(api.searchPlaces).mockResolvedValue({ status: "ok", data: FX.places });
});

afterEach(async () => {
  await act(async () => {
    await setLanguage("tr");
  });
});

test("asks for this month at the current place and draws one cell per calendar day", async () => {
  renderPanel();
  const table = await grid();
  // Bir istek: ay verilmez ("bugün" Rust'ta okunur), yerin koordinatı ve dilimi (ad gitmez),
  // varsayılan süzgeç, haritanın hesap ayarı; doğum verisi gitmez.
  expect(requests()).toEqual([{ month: null, place: FX.muhurta.place, filter: CAL.request.filter, settings: FX.opened.chart.meta.settings, natal: null }]);
  expect(requests()[0]).toEqual(CAL.request);
  expect(screen.getByText("New Delhi, Delhi, India")).toBeInTheDocument();

  // Türkçede hafta Pazartesi başlar; başlık kısaltmadır, tam ad ekran okuyucuya verilir.
  const heads = within(table).getAllByRole("columnheader");
  expect(heads.map((h) => h.querySelector('[aria-hidden="true"]')?.textContent)).toEqual(["Pzt", "Sal", "Çar", "Per", "Cum", "Cmt", "Paz"]);
  expect(heads.map((h) => h.getAttribute("abbr"))).toEqual(["Pazartesi", "Salı", "Çarşamba", "Perşembe", "Cuma", "Cumartesi", "Pazar"]);
  // 1 Kasım 2026 Pazar'dır: altı hafta satırı, otuz gün, on iki boş hücre.
  expect(table.querySelectorAll("tbody tr")).toHaveLength(6);
  expect(table.querySelectorAll("td[data-date]")).toHaveLength(30);
  expect(table.querySelectorAll("td.calendar-empty")).toHaveLength(12);
  expect([...table.querySelectorAll("tbody tr:first-child td")].map((td) => td.getAttribute("data-date"))).toEqual([null, null, null, null, null, null, "2026-11-01"]);

  // Hücre: ayın günü, gün doğumundaki tithi (kısa yazım; tam adı ekran okuyucuya) ve Ay nakshatrası.
  const first = cell(table, "2026-11-01");
  expect(lines(first)).toEqual(["1", "K7 SaptamiKrishna 7 · Saptami", "Pushya"]);
  expect(first.querySelector("ul")).toBeNull();
  // Yeni ay günü: gün doğumunda Amavasya; olay saatiyle yazılır.
  const amavasya = cell(table, "2026-11-09");
  expect(lines(amavasya).slice(0, 3)).toEqual(["9", "K15 AmavasyaKrishna 15 · Amavasya", "Swati"]);
  expect(events(amavasya)).toEqual(["12:32 Yeni ayYeni ay · Ter · Vishakha"]);
  // Girişler ve dönüşler: kısa yazı görünür, tam ad ekran okuyucuya verilir.
  expect(events(cell(table, "2026-11-06"))).toEqual(["01:04 Ve → BaşVenüs: burç girişi · Baş", "21:03 Gü → VishakhaGüneş: nakshatra girişi · Vishakha"]);
  expect(events(cell(table, "2026-11-13"))).toEqual(["21:24 Me R biterMerkür: R bitiyor · Ter"]);
  expect(events(cell(table, "2026-11-16"))).toEqual(["19:43 Gü → AkrGüneş: burç girişi · Akr"]);
  expect(cell(table, "2026-11-16").querySelector("li")).toHaveAttribute("data-event", "SignIngress");
  const short = cell(table, "2026-11-16").querySelector('li [aria-hidden="true"]');
  expect(short).toHaveTextContent("Gü → Akr");
  expect(lines(cell(table, "2026-11-24")).slice(1, 3)).toEqual(["S15 PurnimaShukla 15 · Purnima", "Krittika"]);

  // Bugün (Rust'ın verdiği gün) sözle ve çerçeveyle işaretlidir; yalnızca bir hücre.
  const today = cell(table, "2026-11-25");
  expect(lines(today)).toEqual(["25bugün", "K1 PratipadaKrishna 1 · Pratipada", "Rohini", "13:41 Me → VishakhaMerkür: nakshatra girişi · Vishakha"]);
  expect(table.querySelectorAll("td.is-today")).toHaveLength(1);
  expect(today).toHaveClass("is-today");

  expect(screen.getByText(/^Yöntem: calendar\.month_v1\. Hücre takvim günüdür/)).toBeInTheDocument();
  // Tabloda denetim yoktur: kaydırma kabı klavye odağı alır ve tablonun adını taşır.
  const scroll = screen.getByRole("group", { name: /^Kasım 2026 takvimi/ });
  expect(scroll).toHaveAttribute("tabindex", "0");
});

test("a cell lists at most three events and counts the rest", async () => {
  renderPanel({ kinds: CAL.ids.kinds, moon: true });
  const table = await grid();
  expect(requests()[0]).toEqual(CAL.request_full);
  // 24 Kasım: beş olay (Ay'ın iki nakshatra girişi, burç girişi, doğum boylamına geliş, dolunay).
  expect(events(cell(table, "2026-11-24"))).toEqual([
    "02:02 Ay → KrittikaAy: nakshatra girişi · Krittika",
    "07:24 Ay → BoğAy: burç girişi · Boğ",
    "11:09 Gü → doğum VeGüneş, doğum Venüs boylamında · Akr",
    "+2 olay",
  ]);
  // Üç olaylı günde sayaç yoktur.
  expect(events(cell(table, "2026-11-25"))).toHaveLength(3);
  expect(cell(table, "2026-11-25").querySelector(".calendar-more")).toBeNull();
});

test("the month is stepped with Rust's previous and next months, chosen by month and year, and brought back with This month", async () => {
  renderPanel();
  await grid();
  const nav = screen.getByRole("group", { name: "Ay seçimi" });
  const month = within(nav).getByRole("combobox", { name: "Ay" }) as HTMLSelectElement;
  // Yıl alanı gösterilen yıl değişince yeniden kurulur: her seferinde yeniden bulunur.
  const year = () => within(nav).getByRole("textbox", { name: "Yıl" }) as HTMLInputElement;
  expect([month.value, year().value]).toEqual(["11", "2026"]);
  expect([...month.options].map((o) => o.textContent)).toEqual(["Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran", "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık"]);

  // Sonraki ay: yanıttaki `next` istenir; arayüz ay hesabı yapmaz.
  await act(async () => within(nav).getByRole("button", { name: "Sonraki ay" }).click());
  expect(requests().at(-1)?.month).toEqual({ year: 2026, month: 12 });
  const december = await grid("Aralık 2026 takvimi: gün doğumundaki tithi ve Ay nakshatrası, günün olayları");
  expect(december.querySelectorAll("td[data-date]")).toHaveLength(31);
  // Başka bir ayda "bugün" yoktur.
  expect(december.querySelectorAll("td.is-today")).toHaveLength(0);
  expect([month.value, year().value]).toEqual(["12", "2026"]);
  // Aynı gün ve saatteki olaylar ayrı satırlardır (düğümlerin burç girişi).
  expect(events(cell(december, "2026-12-05")).slice(0, 2)).toEqual(["22:32 Ra → OğlRahu: burç girişi · Oğl", "22:32 Ke → YenKetu: burç girişi · Yen"]);

  // Ay alanı: gösterilen yılın seçilen ayı istenir.
  await act(async () => fireEvent.change(month, { target: { value: "1" } }));
  expect(requests().at(-1)?.month).toEqual({ year: 2026, month: 1 });
  await act(async () => {});
  // Bu testte o ayın yanıtı hatadır: alan adı arayüz dilinde yazılır ve ay seçimi istenen ayı
  // göstererek kullanılabilir durur.
  expect(screen.getByRole("alert")).toHaveTextContent("Takvim isteği geçersiz: ay (yıl 1–2999).");
  expect([month.value, month.disabled, year().value, year().disabled]).toEqual(["1", false, "2026", false]);
  await act(async () => within(nav).getByRole("button", { name: "Bu ay" }).click());
  expect(requests().at(-1)?.month).toBeNull();
  await grid();
  expect(screen.queryByRole("alert")).toBeNull();
  expect(month.value).toBe("11");

  // Yıl: geçerli değer Enter ile istenir; aralık dışı değer uygulanmaz ve gösterilen yıla döner.
  const before = requests().length;
  fireEvent.change(year(), { target: { value: "3000" } });
  expect(year()).toHaveAttribute("aria-invalid", "true");
  fireEvent.blur(year());
  expect(year().value).toBe("2026");
  expect(year()).not.toHaveAttribute("aria-invalid");
  fireEvent.change(year(), { target: { value: "20x6" } });
  await act(async () => fireEvent.submit(year().closest("form")!));
  expect(year().value).toBe("2026");
  expect(requests()).toHaveLength(before);

  // Önceki ay.
  await act(async () => within(nav).getByRole("button", { name: "Önceki ay" }).click());
  expect(requests().at(-1)?.month).toEqual({ year: 2026, month: 10 });
});

test("This month reads again even when the month is already the current one", async () => {
  renderPanel();
  await grid();
  expect(api.monthCalendar).toHaveBeenCalledTimes(1);
  await act(async () => screen.getByRole("button", { name: "Bu ay" }).click());
  // Aynı istek yeniden gider: gece yarısından sonra Rust'ın "bugün"ü değişmiş olabilir.
  expect(api.monthCalendar).toHaveBeenCalledTimes(2);
  expect(requests()[1]).toEqual(requests()[0]);
  await grid();
});

test("the year field asks for the typed year in the shown month", async () => {
  vi.mocked(api.monthCalendar).mockImplementation(async (request) => (request.month === null ? answer(request) : ok({ ...calendarFixture().month_next, month: request.month, today: null })));
  renderPanel();
  await grid();
  const year = screen.getByRole("textbox", { name: "Yıl" }) as HTMLInputElement;
  fireEvent.change(year, { target: { value: "2031" } });
  await act(async () => fireEvent.submit(year.closest("form")!));
  expect(requests().at(-1)?.month).toEqual({ year: 2031, month: 11 });
  await grid("Kasım 2031 takvimi: gün doğumundaki tithi ve Ay nakshatrası, günün olayları");
  expect((screen.getByRole("textbox", { name: "Yıl" }) as HTMLInputElement).value).toBe("2031");
});

test("the grid stays while the next month loads and the range ends disable a step", async () => {
  let release!: (r: Result<CalendarView>) => void;
  renderPanel();
  const table = await grid();
  vi.mocked(api.monthCalendar).mockImplementationOnce(() => new Promise((r) => (release = r)));
  await act(async () => screen.getByRole("button", { name: "Sonraki ay" }).click());
  // Yeni ay gelene dek eski ızgara durur ve panel meşgul işaretlidir.
  expect(table).toBeInTheDocument();
  expect(table.closest(".calendar")).toHaveAttribute("aria-busy", "true");
  // Aralığın ucunda sonraki ay yoktur: düğme odaklanır ama bir şey istemez.
  await act(async () => release(ok({ ...calendarFixture().month_next, next: null })));
  expect(screen.getByRole("table").closest(".calendar")).not.toHaveAttribute("aria-busy");
  const next = screen.getByRole("button", { name: "Sonraki ay" });
  expect(next).toHaveAttribute("aria-disabled", "true");
  expect(next).not.toBeDisabled();
  const before = requests().length;
  await act(async () => next.click());
  expect(requests()).toHaveLength(before);
});

test("without a current place the panel asks for one and nothing is computed; the chart's place can be used", async () => {
  const { persistence } = renderPanel({}, { prefs: FX.prefs.prefs });
  expect(screen.getByRole("note")).toHaveTextContent("Bulunulan yer seçilmedi.");
  expect(api.monthCalendar).not.toHaveBeenCalled();
  // Ay seçimi yanıt gelene dek kullanılamaz.
  expect(screen.getByRole("combobox", { name: "Ay" })).toBeDisabled();
  expect(screen.getByRole("textbox", { name: "Yıl" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Önceki ay" })).toHaveAttribute("aria-disabled", "true");
  await act(async () => fireEvent.change(screen.getByRole("combobox", { name: "Yer" }), { target: { value: "chart" } }));
  await grid();
  expect(requests()[0].place).toEqual(FX.muhurta.place);
  // Yer seçimi panelin ayarıdır ve düzenle birlikte kaydedilir.
  await act(async () => {});
  expect(persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings).toEqual({ place: "chart" });
});

test("days without a sunrise are marked and explained; low precision is said", async () => {
  vi.mocked(api.monthCalendar).mockImplementation(async () => ok({ ...calendarFixture().month_polar, warnings: ["LowPrecision"] }));
  renderPanel();
  const table = await grid("Ocak 2027 takvimi: gün doğumundaki tithi ve Ay nakshatrası, günün olayları");
  // Tromsø, Ocak 2027: Güneş ayın 19'una dek doğmaz.
  const dark = cell(table, "2027-01-18");
  expect(dark.querySelector(".calendar-tithi")).toHaveTextContent(/\*\s*gün doğumu yok$/);
  expect(dark.querySelector('.calendar-tithi [title="gün doğumu yok"]')).toHaveTextContent("*");
  expect(cell(table, "2027-01-19").querySelector(".calendar-tithi")).not.toHaveTextContent("*");
  expect(table.querySelectorAll('.calendar-tithi [title="gün doğumu yok"]')).toHaveLength(18);
  expect(screen.getByText("* Bu günde Güneş doğmuyor: tithi ve nakshatra 06:00 yerel ortalama saatte okundu.")).toBeInTheDocument();
  expect(screen.getByText("Düşük hassasiyet")).toBeInTheDocument();
  expect(screen.getByText(/Güneş veya Ay yedek ephemeris ile hesaplandı\./)).toBeInTheDocument();
});

test("a failure is shown with retry; the focus does not fall to the body", async () => {
  vi.mocked(api.monthCalendar).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
  renderPanel();
  const alert = await screen.findByRole("alert");
  expect(alert).toHaveTextContent("Hesap tamamlanamadı (x).");
  // Yer ve ay seçimi hata durumunda da yerindedir.
  expect(screen.getByRole("combobox", { name: "Yer" })).toBeInTheDocument();
  const retry = within(alert).getByRole("button", { name: "Yeniden dene" });
  retry.focus();
  await act(async () => retry.click());
  await grid();
  expect(api.monthCalendar).toHaveBeenCalledTimes(2);
  expect(retry).not.toBeInTheDocument();
  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Transit takvimi" }));
});

test("without an open chart there is only a message and nothing is asked", () => {
  renderPanel({}, { opened: null });
  expect(screen.getByText("Açık harita yok.")).toBeInTheDocument();
  expect(api.monthCalendar).not.toHaveBeenCalled();
  expect(screen.queryByRole("group", { name: "Ay seçimi" })).toBeNull();
});

test("the week starts on Sunday in English and on Monday in French; month and weekday names follow the language", async () => {
  await setLanguage("en");
  const { unmount } = renderPanel();
  const english = await grid("Calendar for November 2026: tithi and Moon nakshatra at sunrise, the events of the day");
  expect(within(english).getAllByRole("columnheader").map((h) => h.querySelector('[aria-hidden="true"]')?.textContent)).toEqual(["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]);
  expect(english.querySelectorAll("tbody tr")).toHaveLength(5);
  expect([...english.querySelectorAll("tbody tr:first-child td")].map((td) => td.getAttribute("data-date"))[0]).toBe("2026-11-01");
  expect(lines(cell(english, "2026-11-25"))).toEqual(["25today", "K1 PratipadaKrishna 1 · Pratipada", "Rohini", "13:41 Me → VishakhaMercury: nakshatra ingress · Vishakha"]);
  expect(events(cell(english, "2026-11-13"))).toEqual(["21:24 Me R endsMercury: R ends · Li"]);
  unmount();

  await setLanguage("fr");
  renderPanel();
  const french = await grid("Calendrier de novembre 2026 : tithi et nakshatra de la Lune au lever du Soleil, événements du jour");
  expect(within(french).getAllByRole("columnheader").map((h) => h.querySelector('[aria-hidden="true"]')?.textContent)).toEqual(["lun.", "mar.", "mer.", "jeu.", "ven.", "sam.", "dim."]);
  expect(french.querySelectorAll("tbody tr")).toHaveLength(6);
  expect(cell(french, "2026-11-25").querySelector(".calendar-today")).toHaveTextContent("aujourd’hui");
});

test("the panel is in the palette catalog, the names exist in three languages and French uses the typographic apostrophe", () => {
  expect(MODULE_PANELS.find((m) => m.kind === "calendar")).toEqual({ kind: "calendar", w: 9, h: 24 });
  expect(PANEL_REGISTRY.calendar).toEqual({ titleKey: "panel.calendar", component: CalendarPanel });
  expect([tr.panel.calendar, en.panel.calendar, fr.panel.calendar]).toEqual(["Transit takvimi", "Transit calendar", "Calendrier des transits"]);
  for (const lang of [tr, en, fr]) {
    expect(Object.keys(lang.calendar.month)).toEqual(["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12"]);
    expect(Object.keys(lang.calendar.weekdayShort).sort()).toEqual(Object.keys(lang.panchang.weekday).sort());
    expect(Object.keys(lang.calendar.short).sort()).toEqual([...CAL.ids.kinds, "return"].sort());
    expect(lang.errors.invalid_calendar_input).toContain("{{field}}");
  }
  expect(JSON.stringify(fr.calendar)).not.toContain("'");
});
````

````diff
diff --git a/app/ui/layout/model.test.ts b/app/ui/layout/model.test.ts
index 4fc9d72..3d48000 100644
--- a/app/ui/layout/model.test.ts
+++ b/app/ui/layout/model.test.ts
@@ -54,6 +54,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
     "compat",
     "muhurta_day",
     "muhurta_search",
+    "calendar",
     "prashna_facts",
     "event_search",
     "navatara",
@@ -65,7 +66,7 @@ test("catalog has the seven v1 panel kinds and the module panels", () => {
 });
 
 test("module panels are known kinds, listed once, with a size the grid accepts", () => {
-  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "bhava_table", "sudarshana", "yogas", "avastha", "compat", "muhurta_day", "muhurta_search", "prashna_facts", "event_search", "navatara", "vimshopaka"]);
+  expect(MODULE_PANELS.map((m) => m.kind)).toEqual(["panchang", "ashtakavarga", "drishti", "jaimini_karakas", "arudha", "argala", "shadbala", "bhava_bala", "bhava_table", "sudarshana", "yogas", "avastha", "compat", "muhurta_day", "muhurta_search", "calendar", "prashna_facts", "event_search", "navatara", "vimshopaka"]);
   for (const m of MODULE_PANELS) {
     expect(isKnownPanelKind(m.kind)).toBe(true);
     expect(m.w).toBeGreaterThanOrEqual(minSize(m.kind).w);
````

- [ ] **Step 2: Testlerin düştüğünü gör**

Run: `pnpm --dir app exec vitest run ui/calendar ui/layout/model.test.ts 2>&1 | grep -E "Test Files|Tests " | head -3`
Expected: `Test Files  4 failed (4)`, `Tests  2 failed | 23 passed (25)`: üç takvim test dosyası içe aktarımını çözemez (`./model`, `./useCalendar`, `./CalendarPanel`) ve `layout/model.test.ts`'in iki beklentisi düşer (katalog listesi ve `MODULE_PANELS` sırası). Çıktının başında `Failed Tests 2` satırı da görünür.

- [ ] **Step 3: Model, kanca, ızgara, panel, katalog ve metinler**

**Yeni dosya:** `app/ui/calendar/model.ts`

````ts
// Geçiş takviminin arayüz modeli (yol haritası §4.11, Plan 29): süzgecin panel ayarından
// okunması, günlerin hafta satırlarına dizilmesi ve olayın hücredeki kısa yazısı. Burada hesap
// yoktur (C4): günler, hafta günleri, olgular, olaylar, önceki ve sonraki ay Rust'tan gelir;
// buradaki her şey seçim listesi, yerleşim ve gösterimdir.
import type { TFunction } from "i18next";
import type { CalendarDayView, CalendarFilter, SkyEvent, SkyEventKind, Vara } from "../bindings";
import { signIndex } from "../chart/adapter";
import type { Language } from "../i18n";
import { VARAS } from "../modules/muhurta";
import { nakshatraName } from "../panels/labels";

/** Rust `CALENDAR_KINDS`: takvimde gösterilebilen olay türleri, süzgeçteki sırayla. */
export const CALENDAR_KINDS = ["SignIngress", "NakshatraIngress", "StationRetrograde", "StationDirect", "NewMoon", "FullMoon", "TransitToNatal"] as const satisfies readonly SkyEventKind[];
/** Rust `CALENDAR_DEFAULT_KINDS`: yeni panelin türleri. */
export const DEFAULT_CALENDAR_KINDS = ["SignIngress", "NakshatraIngress", "StationRetrograde", "StationDirect", "NewMoon", "FullMoon"] as const satisfies readonly SkyEventKind[];
/** Rust `CALENDAR_YEARS`. */
export const MIN_YEAR = 1;
export const MAX_YEAR = 2999;
/** Bir hücrede yazılan en çok olay; fazlası sayılır ve gün ayrıntısında listelenir. */
export const CELL_EVENTS = 3;

/** Haftanın ilk günü, arayüz diline göre: Türkçe ve Fransızcada Pazartesi, İngilizcede Pazar. */
export const WEEK_START: Record<Language, Vara> = { tr: "Monday", en: "Sunday", fr: "Monday" };

/** Sütun sırası: haftanın ilk gününden başlayan yedi gün. */
export function weekdayOrder(start: Vara): Vara[] {
  const first = VARAS.indexOf(start);
  return VARAS.map((_, i) => VARAS[(first + i) % 7]);
}

/**
 * Günlerin hafta satırları: her gün kendi hafta gününün sütununda durur, boş hücreler `null`'dır.
 * Sütunu Rust'ın verdiği hafta günü belirler; takvimden gün atlayan bir saat diliminde de
 * (ardışık iki günün hafta günü bir gün atlar) her gün doğru sütundadır.
 */
export function weekRows(days: readonly CalendarDayView[], start: Vara): (CalendarDayView | null)[][] {
  const order = weekdayOrder(start);
  const rows: (CalendarDayView | null)[][] = [];
  // İlk gün her zaman yeni bir satır açar.
  let last = 7;
  for (const day of days) {
    const col = order.indexOf(day.weekday);
    if (col <= last) rows.push([null, null, null, null, null, null, null]);
    rows[rows.length - 1][col] = day;
    last = col;
  }
  return rows;
}

/** Panel ayarındaki süzgecin toleranslı okuması: tanınmayan türler atılır; anahtar yoksa varsayılan. */
export function parseFilter(settings: Record<string, unknown>): CalendarFilter {
  const raw = settings.kinds;
  const kinds: SkyEventKind[] = Array.isArray(raw) ? CALENDAR_KINDS.filter((k) => raw.includes(k)) : [...DEFAULT_CALENDAR_KINDS];
  return { kinds, moon: settings.moon === true };
}

/** Olayın hücredeki kısa yazısı: graha ve burç kısaltmaları, nakshatra adı. */
export function eventShort(t: TFunction, e: SkyEvent): string {
  const [a, b] = e.grahas;
  const graha = t(`graha.${a}`);
  switch (e.kind) {
    case "SignIngress":
      return t("calendar.short.SignIngress", { graha, sign: e.sign === null ? "—" : t(`sign.${signIndex(e.sign)}`) });
    case "NakshatraIngress":
      return t("calendar.short.NakshatraIngress", { graha, nakshatra: e.nakshatra === null ? "—" : nakshatraName(e.nakshatra) });
    case "TransitToNatal":
      return a === b ? t("calendar.short.return", { graha }) : t("calendar.short.TransitToNatal", { a: graha, b: t(`graha.${b}`) });
    case "NewMoon":
    case "FullMoon":
      return t(`calendar.short.${e.kind}`);
    default:
      return t(`calendar.short.${e.kind}`, { graha });
  }
}

/** Olayın tam adı (olay araması panelinin metinleriyle): ekran okuyucu ve gün ayrıntısı için. */
export function eventLong(t: TFunction, e: SkyEvent): string {
  const [a, b] = e.grahas;
  const name = (g: string) => t(`grahaName.${g}`);
  switch (e.kind) {
    case "Conjunction":
      return t("eventSearch.event.Conjunction", { a: name(a), b: name(b) });
    case "TransitToNatal":
      return a === b ? t("eventSearch.event.return", { graha: name(a) }) : t("eventSearch.event.TransitToNatal", { a: name(a), b: name(b) });
    case "HouseFromNatalMoon":
      return t("eventSearch.event.HouseFromNatalMoon", { graha: name(a), house: e.house ?? "—" });
    case "NewMoon":
    case "FullMoon":
      return t(`eventSearch.event.${e.kind}`);
    default:
      return t(`eventSearch.event.${e.kind}`, { graha: name(a) });
  }
}

/** Olayın burcu ve nakshatrası: "Akrep · Anuradha"; ikisi de yoksa boş. */
export function eventWhere(t: TFunction, e: SkyEvent): string {
  return [e.sign === null ? null : t(`sign.${signIndex(e.sign)}`), e.nakshatra === null ? null : nakshatraName(e.nakshatra)].filter((x) => x !== null).join(" · ");
}
````

**Yeni dosya:** `app/ui/calendar/useCalendar.ts`

````ts
// Geçiş takvimini (`month_calendar`) isteyen kanca (yol haritası §4.11, Plan 29). Sonuç saklanmaz
// ve arayüzde hesap yoktur (K1, C4); panel açıkken istenir. Ay, yer, süzgeç ya da ayar değişince
// yeni ay istenir ve eski isteğin geç gelen yanıtı düşer; yeni ay okunurken aynı panelin son ayı
// gösterilmeyi sürdürür (ızgara ve odak yerinde durur). `epoch` aynı isteğin yeniden okunmasını
// ister ("Bu ay" yeniden seçildi: "bugün" Rust'ta okunur). İstek yer ve (doğum türü seçiliyse)
// doğum verisi taşır: anahtar yalnızca bellekte karşılaştırılır, loglanmaz.
import { api } from "../api";
import type { CalendarRequest, CalendarView } from "../bindings";
import { useLastData, useModule, type ModuleState } from "../modules/useModule";

export interface CalendarState extends ModuleState<CalendarView> {
  /** Gösterilecek ay: yeni yanıt gelene dek bir önceki. */
  shown: CalendarView | null;
}

/** `request === null`: yer seçilmemiş, komut çağrılmaz. `scope` haritanın ve panelin kimliğidir. */
export function useCalendar(request: CalendarRequest | null, epoch: number, scope: string): CalendarState {
  const key = request === null ? null : JSON.stringify([request, epoch]);
  const state = useModule(key, () => api.monthCalendar(request as CalendarRequest));
  const shown = useLastData(state, key === null ? null : scope);
  return { ...state, shown };
}
````

**Yeni dosya:** `app/ui/calendar/CalendarGrid.tsx`

````tsx
// Ay ızgarası (Plan 29): hafta günleri sütun, haftalar satırdır. Hücre takvim günüdür: ayın günü,
// gün doğumundaki tithi ve Ay nakshatrası ve o güne düşen olayların ilk birkaçı. Değerler
// `month_calendar`'dan gelir; burada hesap yoktur (C4). Kısaltmaların tam adı ekran okuyucuya
// ayrıca verilir; işaretler yazıdır (renk tek başına anlam taşımaz, C9).
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { CalendarDayView, CalendarView } from "../bindings";
import { currentLanguage } from "../i18n";
import { formatJdClock } from "../panels/format";
import { nakshatraName, tithiInPaksha, tithiName, tithiShort } from "../panels/labels";
import { CELL_EVENTS, eventLong, eventShort, eventWhere, WEEK_START, weekdayOrder, weekRows } from "./model";

/** Gün doğumu olmayan günün işareti (dipnotla açıklanır). */
export const NO_SUNRISE_MARK = "*";

export function CalendarGrid({ view }: { view: CalendarView }) {
  const { t } = useTranslation();
  const captionId = useId();
  const start = WEEK_START[currentLanguage()];
  const order = weekdayOrder(start);
  const rows = weekRows(view.days, start);
  const month = t(`calendar.month.${view.month.month}`);
  return (
    <div className="bala-scroll calendar-scroll" role="group" tabIndex={0} aria-labelledby={captionId}>
      <table className="calendar-grid">
        <caption id={captionId}>{t("calendar.caption", { month, year: view.month.year })}</caption>
        <thead>
          <tr>
            {order.map((v) => (
              <th key={v} scope="col" abbr={t(`panchang.weekday.${v}`)}>
                <span aria-hidden="true">{t(`calendar.weekdayShort.${v}`)}</span>
                <span className="visually-hidden">{t(`panchang.weekday.${v}`)}</span>
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, r) => (
            <tr key={row.find((d) => d !== null)?.date ?? r}>
              {row.map((day, c) => (day === null ? <td key={`empty-${c}`} className="calendar-empty" /> : <DayCell key={day.date} day={day} today={view.today === day.day} />))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function DayCell({ day, today }: { day: CalendarDayView; today: boolean }) {
  const { t } = useTranslation();
  const { facts } = day;
  const shown = day.events.slice(0, CELL_EVENTS);
  const more = day.events.length - shown.length;
  return (
    <td className={today ? "calendar-day is-today" : "calendar-day"} data-date={day.date}>
      <div className="calendar-day-head">
        <span className="calendar-day-number">{day.day}</span>
        {today && <span className="calendar-today">{t("calendar.today")}</span>}
      </div>
      <div className="calendar-tithi">
        <span aria-hidden="true">
          {tithiShort(facts.tithi)} {tithiName(facts.tithi)}
        </span>
        <span className="visually-hidden">{t("calendar.tithi", { paksha: facts.paksha, number: tithiInPaksha(facts.tithi), name: tithiName(facts.tithi) })}</span>
        {facts.sunrise_jd_ut === null && (
          <>
            <span aria-hidden="true" title={t("calendar.noSunrise")}>
              {NO_SUNRISE_MARK}
            </span>
            <span className="visually-hidden"> {t("calendar.noSunrise")}</span>
          </>
        )}
      </div>
      <div className="calendar-nakshatra">{nakshatraName(facts.nakshatra)}</div>
      {day.events.length > 0 && (
        <ul className="calendar-events">
          {shown.map((row, n) => {
            const clock = formatJdClock(row.event.jd_ut, row.offset_secs, null);
            const where = eventWhere(t, row.event);
            return (
              <li key={`${n}:${row.event.jd_ut}`} data-event={row.event.kind}>
                <span className="calendar-event-time">{clock}</span>{" "}
                <span aria-hidden="true">{eventShort(t, row.event)}</span>
                <span className="visually-hidden">{where === "" ? eventLong(t, row.event) : `${eventLong(t, row.event)} · ${where}`}</span>
              </li>
            );
          })}
          {more > 0 && <li className="calendar-more">{t("calendar.more", { n: more })}</li>}
        </ul>
      )}
    </td>
  );
}
````

**Yeni dosya:** `app/ui/calendar/CalendarPanel.tsx`

````tsx
// Transit takvimi paneli (yol haritası §4.11, Plan 29): seçilen yerin takvim ayı. Her hücre bir
// takvim günüdür: gün doğumundaki tithi ve Ay nakshatrası ile o güne düşen gök olayları. Ay,
// günler, olgular ve olaylar `month_calendar`'dan gelir; burada hesap yoktur (C4): "bugün",
// önceki ve sonraki ay da Rust'tan okunur. Yer panelin ayarıdır ve düzenle birlikte saklanır
// (varsayılan: bulunulan yer); gösterilen ay saklanmaz, panel her açılışta bugünün ayını gösterir.
// Takvim olgudur: "uygun gün" hükmü, puan ve açıklama metni yoktur.
import { useId, useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import type { CalendarRequest, CalendarView, CommandError, OpenedClient, YearMonth } from "../bindings";
import type { Panel } from "../layout/model";
import { chartInput } from "../modules/input";
import { muhurtaPlace } from "../modules/muhurta";
import { PlaceControls, PlaceMissing, usePanelPlace } from "../panels/MuhurtaPlace";
import "../panels/panels.css";
import { PanelError, PanelSkeleton, usePanelLoadPhase } from "../panels/states";
import type { PanelProps } from "../panels/types";
import { useSession } from "../session/store";
import "./calendar.css";
import { CalendarGrid, NO_SUNRISE_MARK } from "./CalendarGrid";
import { MAX_YEAR, MIN_YEAR, parseFilter } from "./model";
import { useCalendar } from "./useCalendar";

const MONTHS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12] as const;

export function CalendarPanel({ panel }: PanelProps) {
  const { t } = useTranslation();
  const { opened } = useSession();
  if (!opened) return <p className="panel-message">{t("panel.noChart")}</p>;
  // Başka harita: panel bugünün ayına döner.
  return <Calendar key={opened.chart_record.id} panel={panel} opened={opened} />;
}

function Calendar({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
  const { t, i18n } = useTranslation();
  const place = usePanelPlace(panel, opened);
  // `null`: yerin bugünkü ayı ("bugün" Rust'ta okunur).
  const [month, setMonth] = useState<YearMonth | null>(null);
  const [epoch, setEpoch] = useState(0);
  const filter = parseFilter(panel.settings);
  // Doğum verisi yalnızca doğum boylamına gelişler seçiliyse gönderilir.
  const natal = filter.kinds.includes("TransitToNatal");
  const request: CalendarRequest | null =
    place.place === null ? null : { month, place: muhurtaPlace(place.place), filter, settings: opened.chart.meta.settings, natal: natal ? chartInput(opened) : null };
  const module = useCalendar(request, epoch, `${opened.chart_record.id}:${panel.id}`);
  const phase = usePanelLoadPhase(module.loading);
  const view = module.shown;
  const goTo = (next: YearMonth | null) => {
    setMonth(next);
    // "Bu ay" her seçildiğinde yeniden okunur: Rust'ın saati ilerlemiş olabilir.
    if (next === null) setEpoch((n) => n + 1);
  };
  // Hata yalnızca alan adını taşır; ad arayüz dilinde yazılır.
  const shownError = module.error === null ? null : fieldError(module.error, (f) => (i18n.exists(`calendar.field.${f}`) ? t(`calendar.field.${f}`) : f));
  return (
    <div className="module-panel calendar" aria-busy={module.loading && view !== null ? true : undefined}>
      {/* Yer ve ay seçimi yükleme ve hata durumlarında da kullanılabilir. */}
      <div className="module-toolbar">
        <PlaceControls panel={panel} opened={opened} at={place} />
      </div>
      <MonthNav view={view} requested={month} onMonth={goTo} />
      {place.place === null ? (
        <PlaceMissing choice={place.choice} />
      ) : shownError ? (
        <PanelError error={shownError} onRetry={module.retry} />
      ) : view === null ? (
        phase === "skeleton" && <PanelSkeleton />
      ) : (
        <Month view={view} />
      )}
    </div>
  );
}

/** `invalid_calendar_input`'un alan adını verilen adla değiştirir; öteki hatalar olduğu gibi geçer. */
function fieldError(error: CommandError, name: (field: string) => string): CommandError {
  const field = error.params.field;
  if (error.code !== "invalid_calendar_input" || typeof field !== "string") return error;
  return { ...error, params: { ...error.params, field: name(field) } };
}

/**
 * Ay seçimi: önceki ve sonraki ay (Rust'ın verdiği aylar), ay ve yıl alanları, "Bu ay". Alanlar
 * istenen ayı yazar (istek hatayla bitse de başka bir ay seçilebilsin); ay istenmediyse ("bu
 * ay") yanıttaki ayı. İlk yanıt gelene dek kullanılamazlar. Kullanılamayan düğme yerel
 * `disabled` değildir: odaklanır ve basılınca bir şey yapmaz.
 */
function MonthNav({ view, requested, onMonth }: { view: CalendarView | null; requested: YearMonth | null; onMonth(month: YearMonth | null): void }) {
  const { t } = useTranslation();
  const id = useId();
  const shown = requested ?? view?.month ?? null;
  const step = (to: YearMonth | null | undefined, label: string, mark: string) => (
    <button type="button" aria-label={label} title={label} aria-disabled={to ? undefined : true} onClick={() => to && onMonth(to)}>
      <span aria-hidden="true">{mark}</span>
    </button>
  );
  return (
    <div className="module-toolbar calendar-nav" role="group" aria-label={t("calendar.nav")}>
      {step(view?.prev, t("calendar.prev"), "‹")}
      <label htmlFor={`${id}-month`} className="visually-hidden">
        {t("calendar.monthLabel")}
      </label>
      <select id={`${id}-month`} value={shown?.month ?? ""} disabled={shown === null} onChange={(e) => shown && onMonth({ year: shown.year, month: Number(e.target.value) })}>
        {shown === null && <option value="">—</option>}
        {MONTHS.map((m) => (
          <option key={m} value={m}>
            {t(`calendar.month.${m}`)}
          </option>
        ))}
      </select>
      <YearField key={shown?.year ?? "none"} id={`${id}-year`} shown={shown} onMonth={onMonth} />
      {step(view?.next, t("calendar.next"), "›")}
      <button type="button" onClick={() => onMonth(null)}>
        {t("calendar.thisMonth")}
      </button>
    </div>
  );
}

/** Yıl alanı: Enter ya da alandan çıkışla uygulanır; aralık dışı ya da tam sayı olmayan değer uygulanmaz. */
function YearField({ id, shown, onMonth }: { id: string; shown: YearMonth | null; onMonth(month: YearMonth): void }) {
  const { t } = useTranslation();
  const [text, setText] = useState(shown === null ? "" : String(shown.year));
  const year = /^\d{1,4}$/.test(text.trim()) ? Number(text.trim()) : null;
  const valid = year !== null && year >= MIN_YEAR && year <= MAX_YEAR;
  const apply = () => {
    if (shown === null) return;
    if (valid && year !== shown.year) onMonth({ year, month: shown.month });
    // Uygulanmayan değer gösterilen yıla döner.
    else if (!valid) setText(String(shown.year));
  };
  const submit = (e: FormEvent) => {
    e.preventDefault();
    apply();
  };
  return (
    <form className="calendar-year" onSubmit={submit}>
      <label htmlFor={id} className="visually-hidden">
        {t("calendar.yearLabel")}
      </label>
      <input id={id} type="text" inputMode="numeric" size={5} maxLength={4} value={text} disabled={shown === null} aria-invalid={shown !== null && !valid ? true : undefined} onChange={(e) => setText(e.target.value)} onBlur={apply} />
    </form>
  );
}

function Month({ view }: { view: CalendarView }) {
  const { t } = useTranslation();
  const noSunrise = view.days.some((d) => d.facts.sunrise_jd_ut === null);
  return (
    <>
      {view.warnings.includes("LowPrecision") && (
        <p className="panel-notice" role="note">
          <strong>{t("badge.LowPrecision")}</strong>: {t("badgeHint.LowPrecision")}
        </p>
      )}
      <CalendarGrid view={view} />
      {noSunrise && (
        <p className="panel-notice" role="note">
          {t("calendar.noSunriseNote", { mark: NO_SUNRISE_MARK })}
        </p>
      )}
      <p className="panel-message">{t("calendar.method", { method: view.method })}</p>
    </>
  );
}
````

**Yeni dosya:** `app/ui/calendar/calendar.css`

````css
/* Transit takvimi paneli: ay ızgarası. Renk token'lardan gelir (C9). */
.calendar-nav button[aria-disabled="true"] {
  color: var(--text-2);
}
.calendar-year {
  display: inline-flex;
}
.calendar-year input[aria-invalid="true"] {
  outline: 2px solid var(--danger);
  outline-offset: -1px;
}
.calendar-grid {
  width: 100%;
  min-width: 560px;
  table-layout: fixed;
  border-collapse: collapse;
  font-size: var(--fs-table);
}
.calendar-grid th {
  padding: 2px 4px;
  text-align: left;
  color: var(--text-2);
  font-weight: var(--fw-medium);
  border-bottom: 1px solid var(--line);
}
.calendar-grid td {
  height: 84px;
  padding: 2px 4px;
  vertical-align: top;
  border: 1px solid var(--line);
  overflow: hidden;
}
.calendar-grid td.calendar-empty {
  border-color: transparent;
  border-bottom-color: var(--line);
}
.calendar-day-head {
  display: flex;
  align-items: baseline;
  gap: 6px;
}
.calendar-day-number {
  font-size: var(--fs-body);
  font-weight: var(--fw-medium);
  font-variant-numeric: tabular-nums;
}
.calendar-day.is-today {
  border: 2px solid var(--ink);
}
.calendar-today {
  color: var(--ink-strong);
}
.calendar-tithi,
.calendar-nakshatra,
.calendar-events li {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.calendar-nakshatra {
  color: var(--text-2);
}
.calendar-events {
  list-style: none;
  margin: 2px 0 0;
  padding: 0;
}
.calendar-event-time {
  color: var(--text-2);
  font-variant-numeric: tabular-nums;
}
.calendar-more {
  color: var(--text-2);
}
````

````diff
diff --git a/app/ui/layout/model.ts b/app/ui/layout/model.ts
index ac64dae..96ff7a8 100644
--- a/app/ui/layout/model.ts
+++ b/app/ui/layout/model.ts
@@ -25,6 +25,7 @@ export const PANEL_KINDS = [
   "compat",
   "muhurta_day",
   "muhurta_search",
+  "calendar",
   "prashna_facts",
   "event_search",
   "navatara",
@@ -60,6 +61,8 @@ export const MODULE_PANELS = [
   // Muhurta (Plan 20).
   { kind: "muhurta_day", w: 7, h: 18 },
   { kind: "muhurta_search", w: 8, h: 18 },
+  // Transit takvimi (yol haritası Plan 29).
+  { kind: "calendar", w: 9, h: 24 },
   // Prashna (Plan 21).
   { kind: "prashna_facts", w: 7, h: 20 },
   // Olay araması (Plan 22).
diff --git a/app/ui/panels/registry.ts b/app/ui/panels/registry.ts
index e1380f2..cb16d7c 100644
--- a/app/ui/panels/registry.ts
+++ b/app/ui/panels/registry.ts
@@ -11,6 +11,7 @@ import { AvasthaPanel } from "./AvasthaPanel";
 import { BhavaBalaPanel } from "./BhavaBalaPanel";
 import { BhavaTablePanel } from "./BhavaTablePanel";
 import { BirthInfoPanel } from "./BirthInfoPanel";
+import { CalendarPanel } from "../calendar/CalendarPanel";
 import { ChartPanel, chartVarga } from "./ChartPanel";
 import { CompatPanel } from "./CompatPanel";
 import { TransitTablePanel } from "../transit/TransitTablePanel";
@@ -66,6 +67,8 @@ export const PANEL_REGISTRY: Record<KnownPanelKind, PanelDefinition> = {
   // Muhurta (Plan 20).
   muhurta_day: { titleKey: "panel.muhurta_day", component: MuhurtaDayPanel },
   muhurta_search: { titleKey: "panel.muhurta_search", component: MuhurtaSearchPanel },
+  // Transit takvimi (yol haritası Plan 29).
+  calendar: { titleKey: "panel.calendar", component: CalendarPanel },
   // Prashna (Plan 21).
   prashna_facts: { titleKey: "panel.prashna_facts", component: PrashnaFactsPanel },
   // Olay araması (Plan 22).
````

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index f8e66fe..d8bd953 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -87,6 +87,7 @@ export const en: Messages = {
     compat: "Compatibility",
     muhurta_day: "Day panchang",
     muhurta_search: "Muhurta search",
+    calendar: "Transit calendar",
     prashna_facts: "Prashna facts",
     event_search: "Event search",
     navatara: "Navatara",
@@ -585,6 +586,47 @@ export const en: Messages = {
     capped: "Windows shown: the first {{shown}} of {{n}}. Narrow the filter or shorten the range.",
     note: "The limbs and the lagna are for the moment the window starts. Windows are facts, not a verdict on suitability.",
   },
+  calendar: {
+    caption: "Calendar for {{month}} {{year}}: tithi and Moon nakshatra at sunrise, the events of the day",
+    month: {
+      "1": "January",
+      "2": "February",
+      "3": "March",
+      "4": "April",
+      "5": "May",
+      "6": "June",
+      "7": "July",
+      "8": "August",
+      "9": "September",
+      "10": "October",
+      "11": "November",
+      "12": "December",
+    },
+    weekdayShort: { Sunday: "Sun", Monday: "Mon", Tuesday: "Tue", Wednesday: "Wed", Thursday: "Thu", Friday: "Fri", Saturday: "Sat" },
+    nav: "Month selection",
+    prev: "Previous month",
+    next: "Next month",
+    thisMonth: "This month",
+    monthLabel: "Month",
+    yearLabel: "Year",
+    today: "today",
+    tithi: "{{paksha}} {{number}} · {{name}}",
+    noSunrise: "no sunrise",
+    noSunriseNote: "{{mark}} The Sun does not rise on this day: tithi and nakshatra are read at 06:00 local mean time.",
+    more: "+{{n}} more",
+    short: {
+      SignIngress: "{{graha}} → {{sign}}",
+      NakshatraIngress: "{{graha}} → {{nakshatra}}",
+      StationRetrograde: "{{graha}} R starts",
+      StationDirect: "{{graha}} R ends",
+      NewMoon: "New moon",
+      FullMoon: "Full moon",
+      TransitToNatal: "{{a}} → natal {{b}}",
+      return: "{{graha}} return",
+    },
+    field: { place: "place", month: "month (years 1–2999)", kinds: "event kinds", range: "date range" },
+    method: "Method: {{method}}. A cell is a calendar day (midnight to midnight); tithi and nakshatra are the values at that day’s sunrise. Times are in the chosen place’s time.",
+  },
   prashna: {
     notPrashna: "This chart is not a prashna. Press ⌘⇧P to cast a new one.",
     question: "Question",
@@ -965,6 +1007,7 @@ export const en: Messages = {
     invalid_muhurta_input: "Invalid muhurta request: {{field}}.",
     invalid_prashna_input: "Invalid prashna request: {{field}}.",
     invalid_search_input: "Invalid search request: {{field}}.",
+    invalid_calendar_input: "Invalid calendar request: {{field}}.",
     search_stale: "The search was stopped or replaced by a new one.",
     atlas_unavailable: "Place search is unavailable ({{reason}}). You can enter latitude, longitude and time zone by hand.",
     atlas_sqlite: "The place atlas could not be read ({{detail}}).",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 67cc873..6a5cc62 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -87,6 +87,7 @@ export const fr: Messages = {
     compat: "Compatibilité",
     muhurta_day: "Panchang du jour",
     muhurta_search: "Recherche de muhurta",
+    calendar: "Calendrier des transits",
     prashna_facts: "Faits du prashna",
     event_search: "Recherche d’événements",
     navatara: "Navatara",
@@ -585,6 +586,47 @@ export const fr: Messages = {
     capped: "Fenêtres affichées : les {{shown}} premières sur {{n}}. Resserrez le filtre ou raccourcissez l'intervalle.",
     note: "Les membres et le lagna sont ceux de l'instant où la fenêtre commence. Les fenêtres sont des faits, non un jugement de convenance.",
   },
+  calendar: {
+    caption: "Calendrier de {{month}} {{year}} : tithi et nakshatra de la Lune au lever du Soleil, événements du jour",
+    month: {
+      "1": "janvier",
+      "2": "février",
+      "3": "mars",
+      "4": "avril",
+      "5": "mai",
+      "6": "juin",
+      "7": "juillet",
+      "8": "août",
+      "9": "septembre",
+      "10": "octobre",
+      "11": "novembre",
+      "12": "décembre",
+    },
+    weekdayShort: { Sunday: "dim.", Monday: "lun.", Tuesday: "mar.", Wednesday: "mer.", Thursday: "jeu.", Friday: "ven.", Saturday: "sam." },
+    nav: "Choix du mois",
+    prev: "Mois précédent",
+    next: "Mois suivant",
+    thisMonth: "Ce mois-ci",
+    monthLabel: "Mois",
+    yearLabel: "Année",
+    today: "aujourd’hui",
+    tithi: "{{paksha}} {{number}} · {{name}}",
+    noSunrise: "pas de lever du Soleil",
+    noSunriseNote: "{{mark}} Le Soleil ne se lève pas ce jour-là : le tithi et le nakshatra sont lus à 06:00, heure locale moyenne.",
+    more: "+{{n}} autres",
+    short: {
+      SignIngress: "{{graha}} → {{sign}}",
+      NakshatraIngress: "{{graha}} → {{nakshatra}}",
+      StationRetrograde: "{{graha}} R débute",
+      StationDirect: "{{graha}} R finit",
+      NewMoon: "Nouvelle lune",
+      FullMoon: "Pleine lune",
+      TransitToNatal: "{{a}} → {{b}} natal",
+      return: "Retour de {{graha}}",
+    },
+    field: { place: "lieu", month: "mois (années 1–2999)", kinds: "types d’événements", range: "intervalle de dates" },
+    method: "Méthode : {{method}}. Une case est un jour civil (de minuit à minuit) ; le tithi et le nakshatra sont les valeurs au lever du Soleil de ce jour. Les heures sont celles du lieu choisi.",
+  },
   prashna: {
     notPrashna: "Ce thème n’est pas un prashna. Appuyez sur ⌘⇧P pour en établir un.",
     question: "Question",
@@ -965,6 +1007,7 @@ export const fr: Messages = {
     invalid_muhurta_input: "Requête de muhurta invalide : {{field}}.",
     invalid_prashna_input: "Requête de prashna invalide : {{field}}.",
     invalid_search_input: "Requête de recherche invalide : {{field}}.",
+    invalid_calendar_input: "Requête de calendrier invalide : {{field}}.",
     search_stale: "La recherche a été arrêtée ou remplacée par une nouvelle.",
     atlas_unavailable: "La recherche de lieux est indisponible ({{reason}}). Vous pouvez saisir la latitude, la longitude et le fuseau à la main.",
     atlas_sqlite: "L’atlas des lieux est illisible ({{detail}}).",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 3c2e07b..3582e0c 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -86,6 +86,7 @@ export const tr = {
     compat: "Uyum",
     muhurta_day: "Gün panchangı",
     muhurta_search: "Muhurta araması",
+    calendar: "Transit takvimi",
     prashna_facts: "Prashna olguları",
     event_search: "Olay araması",
     navatara: "Navatara",
@@ -584,6 +585,47 @@ export const tr = {
     capped: "İlk {{shown}} pencere gösteriliyor (toplam {{n}}). Süzgeci daraltın ya da aralığı kısaltın.",
     note: "Uzuvlar ve lagna pencerenin başlangıç anı içindir. Pencereler olgudur; uygunluk hükmü değildir.",
   },
+  calendar: {
+    caption: "{{month}} {{year}} takvimi: gün doğumundaki tithi ve Ay nakshatrası, günün olayları",
+    month: {
+      "1": "Ocak",
+      "2": "Şubat",
+      "3": "Mart",
+      "4": "Nisan",
+      "5": "Mayıs",
+      "6": "Haziran",
+      "7": "Temmuz",
+      "8": "Ağustos",
+      "9": "Eylül",
+      "10": "Ekim",
+      "11": "Kasım",
+      "12": "Aralık",
+    },
+    weekdayShort: { Sunday: "Paz", Monday: "Pzt", Tuesday: "Sal", Wednesday: "Çar", Thursday: "Per", Friday: "Cum", Saturday: "Cmt" },
+    nav: "Ay seçimi",
+    prev: "Önceki ay",
+    next: "Sonraki ay",
+    thisMonth: "Bu ay",
+    monthLabel: "Ay",
+    yearLabel: "Yıl",
+    today: "bugün",
+    tithi: "{{paksha}} {{number}} · {{name}}",
+    noSunrise: "gün doğumu yok",
+    noSunriseNote: "{{mark}} Bu günde Güneş doğmuyor: tithi ve nakshatra 06:00 yerel ortalama saatte okundu.",
+    more: "+{{n}} olay",
+    short: {
+      SignIngress: "{{graha}} → {{sign}}",
+      NakshatraIngress: "{{graha}} → {{nakshatra}}",
+      StationRetrograde: "{{graha}} R başlar",
+      StationDirect: "{{graha}} R biter",
+      NewMoon: "Yeni ay",
+      FullMoon: "Dolunay",
+      TransitToNatal: "{{a}} → doğum {{b}}",
+      return: "{{graha}} dönüşü",
+    },
+    field: { place: "yer", month: "ay (yıl 1–2999)", kinds: "olay türleri", range: "tarih aralığı" },
+    method: "Yöntem: {{method}}. Hücre takvim günüdür (gece yarısından gece yarısına); tithi ve nakshatra o günün gün doğumundaki değerdir. Saatler seçilen yerin saatiyledir.",
+  },
   prashna: {
     notPrashna: "Bu harita bir prashna değil. Yeni bir prashna kurmak için ⌘⇧P.",
     question: "Soru",
@@ -964,6 +1006,7 @@ export const tr = {
     invalid_muhurta_input: "Muhurta isteği geçersiz: {{field}}.",
     invalid_prashna_input: "Prashna isteği geçersiz: {{field}}.",
     invalid_search_input: "Arama isteği geçersiz: {{field}}.",
+    invalid_calendar_input: "Takvim isteği geçersiz: {{field}}.",
     search_stale: "Arama durduruldu ya da yerine yenisi başlatıldı.",
     atlas_unavailable: "Yer araması kullanılamıyor ({{reason}}). Enlem, boylam ve saat dilimini elle girebilirsiniz.",
     atlas_sqlite: "Yer atlası okunamadı ({{detail}}).",
````

- [ ] **Step 4: Testler ve kapılar**

```bash
pnpm --dir app exec vitest run ui/calendar ui/layout/model.test.ts ui/i18n 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
```
Expected: ilk satırlar `5 passed (5)`; Vitest `101 passed (101)`, `997 passed (997)`; `✓ built`; sayaç `0`.

- [ ] **Step 5: Commit**

```bash
git add app/ui/calendar app/ui/layout/model.ts app/ui/layout/model.test.ts app/ui/panels/registry.ts app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts
git diff --cached --stat | tail -1
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): transit calendar panel with the month grid, month navigation and the place" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: `14 files changed, 1075 insertions(+), 1 deletion(-)`; tarama çıktısız; `git status --short` boş.

---

### Task 5: Arayüz: klavye gezinmesi, gün ayrıntısı ve gün panchangına geçiş

Yol haritası §5 Plan 29 görev (5); D19, D20. Izgara ARIA `grid` olur ve tek sekme durağına iner; kaydırma kabı artık sekme durağı değildir (hücreler odak alır). Gün ayrıntısı yeni bir dosyadadır.

**Files:**
- Create: `app/ui/calendar/DayDetail.tsx`
- Modify: `app/ui/calendar/CalendarGrid.tsx` (baştan yazılır), `app/ui/calendar/CalendarPanel.tsx`, `app/ui/calendar/CalendarPanel.test.tsx`, `app/ui/calendar/calendar.css`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`
- Test: `CalendarPanel.test.tsx` (12 → 18)

**Interfaces:**
- Consumes: Task 4'ün dosyaları; `transit/store` (`useTransitControl`: `setAt`), `view/store` (`useLayoutStore`: `active`, `togglePanel`), `shell/focus` (`MAIN_ID`).
- Produces:
  - `CalendarGridProps { view, selected: string | null, onSelect(date), onPage(step: -1 | 1, day: number), focusDay: number | null, onFocused() }`; `CalendarGrid(props)`.
  - `DayDetail({ view, day })`.
  - i18n: `calendar.gridHint`, `pickDay`, `dayTitle`, `sunriseFacts`, `fallbackFacts`, `openDay`, `openDayHint`, `noEvents`, `events`, `col.*`, `goTo`.

- [ ] **Step 1: Testler: sekme durağı ve tuşlar, seçim ve ayrıntı, geçişler, Page Up ve Page Down, odağın korunması**

Var olan testlerde ızgara artık `grid` rolüyle aranır; altı yeni test eklenir.

````diff
diff --git a/app/ui/calendar/CalendarPanel.test.tsx b/app/ui/calendar/CalendarPanel.test.tsx
index 191eafa..9f42766 100644
--- a/app/ui/calendar/CalendarPanel.test.tsx
+++ b/app/ui/calendar/CalendarPanel.test.tsx
@@ -14,6 +14,7 @@ import { PANEL_REGISTRY } from "../panels/registry";
 import { calendarFixture } from "../test/calendarFixture";
 import { fixture } from "../test/fixture";
 import { renderWithStores } from "../test/render";
+import { useTransitControl, type Transit } from "../transit/store";
 import { useLayoutStore } from "../view/store";
 import { CalendarPanel } from "./CalendarPanel";
 
@@ -26,9 +27,13 @@ const LAYOUT = (settings: Record<string, unknown>) => {
 };
 const withHome = (): AppPrefs => ({ ...FX.prefs.prefs, home_place: FX.muhurta.home_place });
 
+let transit: Transit;
+let kinds: string[] = [];
 /** Paneli düzen deposundan okur: ayar değişince panel yeni ayarla yeniden çizilir (pano gibi). */
 function LivePanel() {
   const { active } = useLayoutStore();
+  transit = useTransitControl();
+  kinds = active.panels.map((p) => p.kind);
   return <CalendarPanel panel={active.panels[0]} />;
 }
 
@@ -55,7 +60,12 @@ function answer(request: CalendarRequest): Result<CalendarView> {
   return ok(request.filter.moon || request.filter.kinds.includes("TransitToNatal") ? calendarFixture().month_full : calendarFixture().month);
 }
 
-const grid = (name = "Kasım 2026 takvimi: gün doğumundaki tithi ve Ay nakshatrası, günün olayları") => screen.findByRole("table", { name });
+const grid = (name = "Kasım 2026 takvimi: gün doğumundaki tithi ve Ay nakshatrası, günün olayları") => screen.findByRole("grid", { name });
+const DECEMBER = "Aralık 2026 takvimi: gün doğumundaki tithi ve Ay nakshatrası, günün olayları";
+const key = (td: HTMLElement, k: string, init: Record<string, unknown> = {}) => fireEvent.keyDown(td, { key: k, ...init });
+const focused = () => (document.activeElement as HTMLElement | null)?.getAttribute("data-date");
+const detail = () => screen.getByRole("region", { name: /^\d+ \S+ \d+ \S+$/ });
+const rowsOf = (table: HTMLElement) => within(table).getAllByRole("row").slice(1).map((r) => [...r.querySelectorAll("th, td")].map((c) => c.textContent));
 const cell = (table: HTMLElement, date: string) => table.querySelector<HTMLElement>(`td[data-date="${date}"]`)!;
 const lines = (td: HTMLElement) => [...td.children].map((c) => c.textContent);
 const events = (td: HTMLElement) => [...td.querySelectorAll("li")].map((li) => li.textContent);
@@ -117,9 +127,6 @@ test("asks for this month at the current place and draws one cell per calendar d
   expect(today).toHaveClass("is-today");
 
   expect(screen.getByText(/^Yöntem: calendar\.month_v1\. Hücre takvim günüdür/)).toBeInTheDocument();
-  // Tabloda denetim yoktur: kaydırma kabı klavye odağı alır ve tablonun adını taşır.
-  const scroll = screen.getByRole("group", { name: /^Kasım 2026 takvimi/ });
-  expect(scroll).toHaveAttribute("tabindex", "0");
 });
 
 test("a cell lists at most three events and counts the rest", async () => {
@@ -151,7 +158,7 @@ test("the month is stepped with Rust's previous and next months, chosen by month
   // Sonraki ay: yanıttaki `next` istenir; arayüz ay hesabı yapmaz.
   await act(async () => within(nav).getByRole("button", { name: "Sonraki ay" }).click());
   expect(requests().at(-1)?.month).toEqual({ year: 2026, month: 12 });
-  const december = await grid("Aralık 2026 takvimi: gün doğumundaki tithi ve Ay nakshatrası, günün olayları");
+  const december = await grid(DECEMBER);
   expect(december.querySelectorAll("td[data-date]")).toHaveLength(31);
   // Başka bir ayda "bugün" yoktur.
   expect(december.querySelectorAll("td.is-today")).toHaveLength(0);
@@ -224,7 +231,7 @@ test("the grid stays while the next month loads and the range ends disable a ste
   expect(table.closest(".calendar")).toHaveAttribute("aria-busy", "true");
   // Aralığın ucunda sonraki ay yoktur: düğme odaklanır ama bir şey istemez.
   await act(async () => release(ok({ ...calendarFixture().month_next, next: null })));
-  expect(screen.getByRole("table").closest(".calendar")).not.toHaveAttribute("aria-busy");
+  expect(screen.getByRole("grid").closest(".calendar")).not.toHaveAttribute("aria-busy");
   const next = screen.getByRole("button", { name: "Sonraki ay" });
   expect(next).toHaveAttribute("aria-disabled", "true");
   expect(next).not.toBeDisabled();
@@ -233,6 +240,178 @@ test("the grid stays while the next month loads and the range ends disable a ste
   expect(requests()).toHaveLength(before);
 });
 
+test("the grid is one tab stop; arrows, Home, End and their Ctrl forms move the focus between the days", async () => {
+  renderPanel();
+  const table = await grid();
+  // Gezici sekme durağı: yalnızca bir hücre sekmeyle odaklanır (bugün); kaydırma kabı durak değildir.
+  expect([...table.querySelectorAll('td[tabindex="0"]')].map((td) => td.getAttribute("data-date"))).toEqual(["2026-11-25"]);
+  expect(table.querySelectorAll('td[tabindex="-1"]')).toHaveLength(29);
+  expect(table.closest(".calendar-scroll")).not.toHaveAttribute("tabindex");
+  expect(table).toHaveAccessibleDescription("Ok tuşları günler arasında gezer; Enter günü seçer; Page Up ve Page Down ayı değiştirir.");
+  const at = (date: string) => cell(table, date);
+  at("2026-11-25").focus();
+  // 25 Kasım Çarşamba'dır (Pazartesi başlayan haftanın üçüncü sütunu).
+  key(at("2026-11-25"), "ArrowRight");
+  expect(focused()).toBe("2026-11-26");
+  key(at("2026-11-26"), "ArrowLeft");
+  key(at("2026-11-25"), "ArrowLeft");
+  expect(focused()).toBe("2026-11-24");
+  key(at("2026-11-24"), "ArrowUp");
+  expect(focused()).toBe("2026-11-17");
+  key(at("2026-11-17"), "ArrowDown");
+  key(at("2026-11-24"), "ArrowDown");
+  // Son satırda Salı sütunu boştur: odak yerinde durur.
+  expect(focused()).toBe("2026-11-24");
+  key(at("2026-11-24"), "Home");
+  expect(focused()).toBe("2026-11-23");
+  key(at("2026-11-23"), "End");
+  expect(focused()).toBe("2026-11-29");
+  key(at("2026-11-29"), "Home", { ctrlKey: true });
+  expect(focused()).toBe("2026-11-01");
+  // Ayın ilk günü ilk satırın tek dolu hücresidir: sola ve yukarı gidilmez; Home ve End aynı gündür.
+  key(at("2026-11-01"), "ArrowLeft");
+  key(at("2026-11-01"), "ArrowUp");
+  key(at("2026-11-01"), "Home");
+  key(at("2026-11-01"), "End");
+  expect(focused()).toBe("2026-11-01");
+  key(at("2026-11-01"), "ArrowDown");
+  expect(focused()).toBe("2026-11-08");
+  key(at("2026-11-08"), "End", { metaKey: true });
+  expect(focused()).toBe("2026-11-30");
+  key(at("2026-11-30"), "ArrowRight");
+  expect(focused()).toBe("2026-11-30");
+  // Sekme durağı odağı izler; gezinme seçimi ve isteği değiştirmez.
+  expect([...table.querySelectorAll('td[tabindex="0"]')].map((td) => td.getAttribute("data-date"))).toEqual(["2026-11-30"]);
+  expect([...table.querySelectorAll('td[aria-selected="true"]')].map((td) => td.getAttribute("data-date"))).toEqual(["2026-11-25"]);
+  expect(api.monthCalendar).toHaveBeenCalledTimes(1);
+});
+
+test("the detail shows today until a day is chosen with Enter, Space or a click", async () => {
+  renderPanel();
+  const table = await grid();
+  // Seçim yokken ayrıntı bugünündür ve bugünün hücresi seçili işaretlidir.
+  expect(detail()).toHaveAccessibleName("25 Kasım 2026 Çarşamba");
+  expect(within(detail()).getByText("Gün doğumu 06:55 · Krishna 1 · Pratipada · Rohini")).toBeInTheDocument();
+  const eventsTable = within(detail()).getByRole("table", { name: "25 Kasım 2026 Çarşamba: günün olayları" });
+  expect(rowsOf(eventsTable)).toEqual([["13:41", "Merkür: nakshatra girişi", "Vishakha"]]);
+  expect(cell(table, "2026-11-25")).toHaveClass("is-selected");
+
+  // Enter: 6 Kasım (iki olay).
+  cell(table, "2026-11-06").focus();
+  key(cell(table, "2026-11-06"), "Enter");
+  expect(detail()).toHaveAccessibleName("6 Kasım 2026 Cuma");
+  expect(detail()).toHaveAttribute("data-detail", "2026-11-06");
+  expect(within(detail()).getByText("Gün doğumu 06:40 · Krishna 12 · Dwadashi · Hasta")).toBeInTheDocument();
+  expect(rowsOf(within(detail()).getByRole("table"))).toEqual([
+    ["01:04", "Venüs: burç girişi", "Baş"],
+    ["21:03", "Güneş: nakshatra girişi", "Vishakha"],
+  ]);
+  expect([...table.querySelectorAll('td[aria-selected="true"]')].map((td) => td.getAttribute("data-date"))).toEqual(["2026-11-06"]);
+  expect(cell(table, "2026-11-25")).not.toHaveClass("is-selected");
+  // Başlık duyurulur.
+  expect(within(detail()).getByText("6 Kasım 2026 Cuma")).toHaveAttribute("aria-live", "polite");
+
+  // Boşluk: olaysız bir gün.
+  key(cell(table, "2026-11-01"), " ");
+  expect(detail()).toHaveAccessibleName("1 Kasım 2026 Pazar");
+  expect(within(detail()).getByText("Seçili türlerde bu güne düşen olay yok.")).toBeInTheDocument();
+  expect(within(detail()).queryByRole("table")).toBeNull();
+
+  // Tıklama: gün seçilir ve sekme durağı o güne taşınır.
+  fireEvent.click(cell(table, "2026-11-16"));
+  expect(detail()).toHaveAccessibleName("16 Kasım 2026 Pazartesi");
+  expect(cell(table, "2026-11-16")).toHaveAttribute("tabindex", "0");
+  expect(api.monthCalendar).toHaveBeenCalledTimes(1);
+});
+
+test("a month without today has no detail until a day is chosen; a day without a sunrise says how it was read", async () => {
+  vi.mocked(api.monthCalendar).mockImplementation(async () => ok(calendarFixture().month_polar));
+  renderPanel();
+  const table = await grid("Ocak 2027 takvimi: gün doğumundaki tithi ve Ay nakshatrası, günün olayları");
+  expect(screen.getByText("Ayrıntısını görmek için bir gün seçin.")).toBeInTheDocument();
+  expect(table.querySelectorAll('td[aria-selected="true"]')).toHaveLength(0);
+  // Bugün yoksa sekme durağı ayın ilk günüdür.
+  expect(cell(table, "2027-01-01")).toHaveAttribute("tabindex", "0");
+  key(cell(table, "2027-01-18"), "Enter");
+  expect(detail()).toHaveAccessibleName("18 Ocak 2027 Pazartesi");
+  expect(within(detail()).getByText(/^Gün doğumu yok · 06:00 yerel ortalama saatte \(05:44\) /)).toBeInTheDocument();
+  key(cell(table, "2027-01-19"), "Enter");
+  expect(within(detail()).getByText(/^Gün doğumu 11:44 · /)).toBeInTheDocument();
+  expect(screen.queryByText("Ayrıntısını görmek için bir gün seçin.")).toBeNull();
+});
+
+test("an event's time moves the transit date to that moment; the day button moves it to the day and adds the day panchang panel", async () => {
+  renderPanel();
+  await grid();
+  expect(transit.at).toBe("Now");
+  const today = calendarFixture().month.days[24];
+  // Olayın saati: transit tarihi o ana gider.
+  await act(async () => within(detail()).getByRole("button", { name: "13:41: transit tarihini bu ana götür" }).click());
+  expect(transit.at).toEqual({ Jd: today.events[0].event.jd_ut });
+  // "Gün panchangını aç": Rust'ın verdiği an (o gün doğumuyla başlayan günün içinde) ve panel.
+  expect(kinds).toEqual(["calendar"]);
+  const open = within(detail()).getByRole("button", { name: "Gün panchangını aç" });
+  expect(open).toHaveAttribute("title", "Transit tarihini bu güne götürür; Gün panchangı paneli düzende yoksa ekler.");
+  await act(async () => open.click());
+  expect(transit.at).toEqual({ Jd: today.panchang_jd_ut });
+  expect(kinds).toEqual(["calendar", "muhurta_day"]);
+  // Panel zaten düzendeyse kaldırılmaz; an seçili günün anıdır.
+  const table = await grid();
+  key(cell(table, "2026-11-09"), "Enter");
+  await act(async () => within(detail()).getByRole("button", { name: "Gün panchangını aç" }).click());
+  expect(transit.at).toEqual({ Jd: calendarFixture().month.days[8].panchang_jd_ut });
+  expect(kinds).toEqual(["calendar", "muhurta_day"]);
+  // Takvim yeniden istenmez.
+  expect(api.monthCalendar).toHaveBeenCalledTimes(1);
+});
+
+test("Page Down and Page Up change the month and keep the focus on the same day of the new month", async () => {
+  renderPanel();
+  const november = await grid();
+  cell(november, "2026-11-25").focus();
+  await act(async () => {
+    key(cell(november, "2026-11-25"), "PageDown");
+  });
+  expect(requests().at(-1)?.month).toEqual({ year: 2026, month: 12 });
+  const december = await grid(DECEMBER);
+  expect(focused()).toBe("2026-12-25");
+  expect(cell(december, "2026-12-25")).toHaveAttribute("tabindex", "0");
+  // Aralık 31 gündür; Kasım'da 31 yoktur: odak ayın son gününe gider.
+  cell(december, "2026-12-31").focus();
+  await act(async () => {
+    key(cell(december, "2026-12-31"), "PageUp");
+  });
+  expect(requests().at(-1)?.month).toEqual({ year: 2026, month: 11 });
+  await grid();
+  expect(focused()).toBe("2026-11-30");
+  // Ay seçimi düğmeleri odağı ızgaraya taşımaz.
+  const next = screen.getByRole("button", { name: "Sonraki ay" });
+  next.focus();
+  await act(async () => next.click());
+  await grid(DECEMBER);
+  expect(document.activeElement).toBe(next);
+});
+
+test("at the end of the range Page Down asks nothing; if the new month fails the focus goes to the panel, not the body", async () => {
+  vi.mocked(api.monthCalendar).mockImplementationOnce(async () => ok({ ...calendarFixture().month, next: null }));
+  renderPanel();
+  const table = await grid();
+  cell(table, "2026-11-10").focus();
+  await act(async () => {
+    key(cell(table, "2026-11-10"), "PageDown");
+  });
+  expect(api.monthCalendar).toHaveBeenCalledTimes(1);
+  expect(focused()).toBe("2026-11-10");
+  // Önceki ay hatayla biter: ızgara kalkar, odak panelin çerçevesine taşınır.
+  vi.mocked(api.monthCalendar).mockResolvedValueOnce({ status: "error", error: { code: "calc", params: { detail: "x" } } });
+  await act(async () => {
+    key(cell(table, "2026-11-10"), "PageUp");
+  });
+  expect(await screen.findByRole("alert")).toHaveTextContent("Hesap tamamlanamadı (x).");
+  expect(screen.queryByRole("grid")).toBeNull();
+  expect(document.activeElement).toBe(screen.getByRole("region", { name: "Transit takvimi" }));
+});
+
 test("without a current place the panel asks for one and nothing is computed; the chart's place can be used", async () => {
   const { persistence } = renderPanel({}, { prefs: FX.prefs.prefs });
   expect(screen.getByRole("note")).toHaveTextContent("Bulunulan yer seçilmedi.");
````

- [ ] **Step 2: Testlerin düştüğünü gör**

Run: `pnpm --dir app exec vitest run ui/calendar/CalendarPanel.test.tsx 2>&1 | grep -E "Test Files|Tests " | head -3`
Expected: `Test Files  1 failed (1)`, `Tests  16 failed | 2 passed (18)`: ızgara henüz `grid` rolünde değildir (var olan on test `findByRole("grid")` ile düşer) ve altı yeni testin aradığı sekme durağı, ayrıntı ve tuşlar yoktur. Çıktının başında `Failed Tests 16` satırı da görünür.

- [ ] **Step 3: Izgaranın klavyesi, gün ayrıntısı ve panel bağlantısı**

**Dosyanın yeni tam içeriği:** `app/ui/calendar/CalendarGrid.tsx`

````tsx
// Ay ızgarası (Plan 29): hafta günleri sütun, haftalar satırdır. Hücre takvim günüdür: ayın günü,
// gün doğumundaki tithi ve Ay nakshatrası ve o güne düşen olayların ilk birkaçı. Değerler
// `month_calendar`'dan gelir; burada hesap yoktur (C4). Kısaltmaların tam adı ekran okuyucuya
// ayrıca verilir; işaretler yazıdır (renk tek başına anlam taşımaz, C9).
//
// Izgara tek bir sekme durağıdır (gezici `tabindex`): ok tuşları günler arasında gezer, Home ve
// End haftanın, Ctrl+Home ve Ctrl+End ayın ucuna gider, Page Up ve Page Down ayı değiştirir,
// Enter ya da boşluk günü seçer. Gezinme yerleşimdir, takvim hesabı değil: komşu gün, Rust'ın
// verdiği günlerin satır ve sütunundan bulunur.
import { useId, useLayoutEffect, useRef, useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";
import type { CalendarDayView, CalendarView } from "../bindings";
import { currentLanguage } from "../i18n";
import { formatJdClock } from "../panels/format";
import { nakshatraName, tithiInPaksha, tithiName, tithiShort } from "../panels/labels";
import { MAIN_ID } from "../shell/focus";
import { CELL_EVENTS, eventLong, eventShort, eventWhere, WEEK_START, weekdayOrder, weekRows } from "./model";

/** Gün doğumu olmayan günün işareti (dipnotla açıklanır). */
export const NO_SUNRISE_MARK = "*";

export interface CalendarGridProps {
  view: CalendarView;
  /** Seçili günün tarihi (`YYYY-MM-DD`); bu ayda değilse hiçbir hücre seçili değildir. */
  selected: string | null;
  onSelect(date: string): void;
  /** Page Up (−1) ya da Page Down (+1): ay değişir ve odak yeni ayda `day` gününe taşınır. */
  onPage(step: -1 | 1, day: number): void;
  /** Ay değiştikten sonra odağın taşınacağı gün; o gün yoksa ayın daha küçük en yakın günü. */
  focusDay: number | null;
  onFocused(): void;
}

export function CalendarGrid({ view, selected, onSelect, onPage, focusDay, onFocused }: CalendarGridProps) {
  const { t } = useTranslation();
  const captionId = useId();
  const hintId = useId();
  const wrap = useRef<HTMLDivElement>(null);
  const start = WEEK_START[currentLanguage()];
  const order = weekdayOrder(start);
  const rows = weekRows(view.days, start);
  const month = t(`calendar.month.${view.month.month}`);
  // Gezici sekme durağı: son odaklanan gün; yoksa seçili gün, bugün ya da ayın ilk günü.
  const [cursor, setCursor] = useState<string | null>(null);
  const has = (date: string | null) => date !== null && view.days.some((d) => d.date === date);
  const todayDate = view.days.find((d) => d.day === view.today)?.date ?? null;
  const stop = has(cursor) ? cursor : has(selected) ? selected : (todayDate ?? view.days[0]?.date ?? null);

  const focus = (day: CalendarDayView | null | undefined) => {
    if (!day) return;
    setCursor(day.date);
    wrap.current?.querySelector<HTMLElement>(`td[data-date="${day.date}"]`)?.focus();
  };
  // Ay değişti (Page Up ya da Page Down): odak aynı güne, o gün yoksa ayın son gününe taşınır.
  const monthKey = `${view.month.year}-${view.month.month}`;
  useLayoutEffect(() => {
    if (focusDay === null) return;
    const target = [...view.days].reverse().find((d) => d.day <= focusDay) ?? view.days[0];
    focus(target);
    onFocused();
    // Yalnızca ay ya da istenen gün değişince.
  }, [monthKey, focusDay]);
  // Izgara odaktayken DOM'dan çıkarsa (hata) odak gövdeye düşmez: panelin çerçevesine taşınır.
  useLayoutEffect(() => {
    const el = wrap.current;
    return () => {
      if (el === null || !el.contains(document.activeElement)) return;
      const target = el.closest<HTMLElement>(".panel") ?? document.getElementById(MAIN_ID);
      target?.focus({ preventScroll: true });
    };
  }, []);

  const onKeyDown = (e: KeyboardEvent<HTMLTableCellElement>, day: CalendarDayView) => {
    const r = rows.findIndex((row) => row.includes(day));
    const c = rows[r].indexOf(day);
    const i = view.days.indexOf(day);
    const inRow = rows[r].filter((d) => d !== null);
    let target: CalendarDayView | null | undefined;
    switch (e.key) {
      case "ArrowLeft":
        target = view.days[i - 1];
        break;
      case "ArrowRight":
        target = view.days[i + 1];
        break;
      case "ArrowUp":
        target = rows[r - 1]?.[c];
        break;
      case "ArrowDown":
        target = rows[r + 1]?.[c];
        break;
      case "Home":
        target = e.ctrlKey || e.metaKey ? view.days[0] : inRow[0];
        break;
      case "End":
        target = e.ctrlKey || e.metaKey ? view.days.at(-1) : inRow.at(-1);
        break;
      case "PageUp":
      case "PageDown":
        e.preventDefault();
        onPage(e.key === "PageUp" ? -1 : 1, day.day);
        return;
      case "Enter":
      case " ":
        e.preventDefault();
        onSelect(day.date);
        return;
      default:
        return;
    }
    // Ok tuşu sayfayı kaydırmaz; ayın ucunda odak yerinde durur.
    e.preventDefault();
    focus(target);
  };

  return (
    <>
      <div ref={wrap} className="bala-scroll calendar-scroll">
        <table className="calendar-grid" role="grid" aria-labelledby={captionId} aria-describedby={hintId}>
          <caption id={captionId}>{t("calendar.caption", { month, year: view.month.year })}</caption>
          <thead>
            <tr>
              {order.map((v) => (
                <th key={v} scope="col" abbr={t(`panchang.weekday.${v}`)}>
                  <span aria-hidden="true">{t(`calendar.weekdayShort.${v}`)}</span>
                  <span className="visually-hidden">{t(`panchang.weekday.${v}`)}</span>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map((row, r) => (
              <tr key={row.find((d) => d !== null)?.date ?? r}>
                {row.map((day, c) =>
                  day === null ? (
                    <td key={`empty-${c}`} className="calendar-empty" />
                  ) : (
                    <DayCell
                      key={day.date}
                      day={day}
                      today={view.today === day.day}
                      selected={selected === day.date}
                      tabStop={stop === day.date}
                      onPick={() => {
                        setCursor(day.date);
                        onSelect(day.date);
                      }}
                      onFocus={() => setCursor(day.date)}
                      onKeyDown={(e) => onKeyDown(e, day)}
                    />
                  ),
                )}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <p id={hintId} className="chart-toolbar-hint">
        {t("calendar.gridHint")}
      </p>
    </>
  );
}

interface DayCellProps {
  day: CalendarDayView;
  today: boolean;
  selected: boolean;
  tabStop: boolean;
  onPick(): void;
  onFocus(): void;
  onKeyDown(e: KeyboardEvent<HTMLTableCellElement>): void;
}

function DayCell({ day, today, selected, tabStop, onPick, onFocus, onKeyDown }: DayCellProps) {
  const { t } = useTranslation();
  const { facts } = day;
  const shown = day.events.slice(0, CELL_EVENTS);
  const more = day.events.length - shown.length;
  const classes = ["calendar-day", today ? "is-today" : null, selected ? "is-selected" : null].filter((c) => c !== null).join(" ");
  return (
    <td className={classes} data-date={day.date} tabIndex={tabStop ? 0 : -1} aria-selected={selected} onClick={onPick} onFocus={onFocus} onKeyDown={onKeyDown}>
      <div className="calendar-day-head">
        <span className="calendar-day-number">{day.day}</span>
        {today && <span className="calendar-today">{t("calendar.today")}</span>}
      </div>
      <div className="calendar-tithi">
        <span aria-hidden="true">
          {tithiShort(facts.tithi)} {tithiName(facts.tithi)}
        </span>
        <span className="visually-hidden">{t("calendar.tithi", { paksha: facts.paksha, number: tithiInPaksha(facts.tithi), name: tithiName(facts.tithi) })}</span>
        {facts.sunrise_jd_ut === null && (
          <>
            <span aria-hidden="true" title={t("calendar.noSunrise")}>
              {NO_SUNRISE_MARK}
            </span>
            <span className="visually-hidden"> {t("calendar.noSunrise")}</span>
          </>
        )}
      </div>
      <div className="calendar-nakshatra">{nakshatraName(facts.nakshatra)}</div>
      {day.events.length > 0 && (
        <ul className="calendar-events">
          {shown.map((row, n) => {
            const clock = formatJdClock(row.event.jd_ut, row.offset_secs, null);
            const where = eventWhere(t, row.event);
            return (
              <li key={`${n}:${row.event.jd_ut}`} data-event={row.event.kind}>
                <span className="calendar-event-time">{clock}</span>{" "}
                <span aria-hidden="true">{eventShort(t, row.event)}</span>
                <span className="visually-hidden">{where === "" ? eventLong(t, row.event) : `${eventLong(t, row.event)} · ${where}`}</span>
              </li>
            );
          })}
          {more > 0 && <li className="calendar-more">{t("calendar.more", { n: more })}</li>}
        </ul>
      )}
    </td>
  );
}
````

**Yeni dosya:** `app/ui/calendar/DayDetail.tsx`

````tsx
// Gün ayrıntısı (Plan 29): seçili takvim gününün gün doğumu olguları ve o güne düşen bütün
// olaylar, saatleriyle. Değerler `month_calendar`'dan gelir; burada hesap yoktur (C4). "Gün
// panchangını aç" transit tarihini Rust'ın verdiği ana götürür (o gün doğumuyla başlayan günün
// içinde) ve Gün panchangı paneli düzende yoksa ekler; olayın saati transit tarihini o ana
// götürür. Olay bir olgudur: hüküm ve açıklama metni yoktur.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { CalendarDayView, CalendarView } from "../bindings";
import { formatJdClock } from "../panels/format";
import { nakshatraName, tithiInPaksha, tithiName } from "../panels/labels";
import { useTransitControl } from "../transit/store";
import { useLayoutStore } from "../view/store";
import { eventLong, eventWhere } from "./model";

export function DayDetail({ view, day }: { view: CalendarView; day: CalendarDayView | null }) {
  const { t } = useTranslation();
  const transit = useTransitControl();
  const { active, togglePanel } = useLayoutStore();
  const titleId = useId();
  const captionId = useId();
  if (day === null) return <p className="panel-message">{t("calendar.pickDay")}</p>;
  const { facts } = day;
  const title = t("calendar.dayTitle", { day: day.day, month: t(`calendar.month.${view.month.month}`), year: view.month.year, weekday: t(`panchang.weekday.${day.weekday}`) });
  const tithi = t("calendar.tithi", { paksha: facts.paksha, number: tithiInPaksha(facts.tithi), name: tithiName(facts.tithi) });
  const openDay = () => {
    const at = day.panchang_jd_ut;
    if (at === null) return;
    transit.setAt({ Jd: at });
    if (!active.panels.some((p) => p.kind === "muhurta_day")) togglePanel("muhurta_day");
  };
  return (
    <section className="calendar-detail" aria-labelledby={titleId} data-detail={day.date}>
      <p id={titleId} className="calendar-detail-title" aria-live="polite">
        {title}
      </p>
      <p className="panel-message">
        {t(facts.sunrise_jd_ut === null ? "calendar.fallbackFacts" : "calendar.sunriseFacts", { time: formatJdClock(facts.jd_ut, day.offset_secs, null), tithi, nakshatra: nakshatraName(facts.nakshatra) })}
      </p>
      <div className="module-toolbar">
        <button type="button" title={t("calendar.openDayHint")} onClick={openDay}>
          {t("calendar.openDay")}
        </button>
      </div>
      {day.events.length === 0 ? (
        <p className="panel-message">{t("calendar.noEvents")}</p>
      ) : (
        <div className="bala-scroll" role="group" tabIndex={0} aria-labelledby={captionId}>
          <table className="module-table">
            <caption id={captionId}>{t("calendar.events", { date: title })}</caption>
            <thead>
              <tr>
                <th scope="col">{t("calendar.col.when")}</th>
                <th scope="col">{t("calendar.col.event")}</th>
                <th scope="col">{t("calendar.col.where")}</th>
              </tr>
            </thead>
            <tbody>
              {day.events.map((row, n) => {
                const jd = row.event.jd_ut;
                const when = formatJdClock(jd, row.offset_secs, null);
                return (
                  <tr key={`${n}:${jd}`} data-event={row.event.kind}>
                    <th scope="row">
                      {jd === null ? (
                        when
                      ) : (
                        <button type="button" className="link-button" aria-label={t("calendar.goTo", { when })} onClick={() => transit.setAt({ Jd: jd })}>
                          {when}
                        </button>
                      )}
                    </th>
                    <td>{eventLong(t, row.event)}</td>
                    <td>{eventWhere(t, row.event)}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}
````

````diff
diff --git a/app/ui/calendar/CalendarPanel.tsx b/app/ui/calendar/CalendarPanel.tsx
index 86d35be..79d92af 100644
--- a/app/ui/calendar/CalendarPanel.tsx
+++ b/app/ui/calendar/CalendarPanel.tsx
@@ -3,6 +3,7 @@
 // günler, olgular ve olaylar `month_calendar`'dan gelir; burada hesap yoktur (C4): "bugün",
 // önceki ve sonraki ay da Rust'tan okunur. Yer panelin ayarıdır ve düzenle birlikte saklanır
 // (varsayılan: bulunulan yer); gösterilen ay saklanmaz, panel her açılışta bugünün ayını gösterir.
+// Izgaranın altında seçili günün ayrıntısı durur: seçim yoksa bugünün (bu aydaysa).
 // Takvim olgudur: "uygun gün" hükmü, puan ve açıklama metni yoktur.
 import { useId, useState, type FormEvent } from "react";
 import { useTranslation } from "react-i18next";
@@ -16,7 +17,8 @@ import { PanelError, PanelSkeleton, usePanelLoadPhase } from "../panels/states";
 import type { PanelProps } from "../panels/types";
 import { useSession } from "../session/store";
 import "./calendar.css";
-import { CalendarGrid, NO_SUNRISE_MARK } from "./CalendarGrid";
+import { CalendarGrid, NO_SUNRISE_MARK, type CalendarGridProps } from "./CalendarGrid";
+import { DayDetail } from "./DayDetail";
 import { MAX_YEAR, MIN_YEAR, parseFilter } from "./model";
 import { useCalendar } from "./useCalendar";
 
@@ -36,6 +38,9 @@ function Calendar({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
   // `null`: yerin bugünkü ayı ("bugün" Rust'ta okunur).
   const [month, setMonth] = useState<YearMonth | null>(null);
   const [epoch, setEpoch] = useState(0);
+  // Seçili günün tarihi; ızgaradan Page Up ya da Page Down ile ay değişince odağın gideceği gün.
+  const [selected, setSelected] = useState<string | null>(null);
+  const [pending, setPending] = useState<{ month: YearMonth; day: number } | null>(null);
   const filter = parseFilter(panel.settings);
   // Doğum verisi yalnızca doğum boylamına gelişler seçiliyse gönderilir.
   const natal = filter.kinds.includes("TransitToNatal");
@@ -65,7 +70,19 @@ function Calendar({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
       ) : view === null ? (
         phase === "skeleton" && <PanelSkeleton />
       ) : (
-        <Month view={view} />
+        <Month
+          view={view}
+          selected={selected}
+          onSelect={setSelected}
+          onPage={(step, day) => {
+            const to = step < 0 ? view.prev : view.next;
+            if (to === null) return;
+            setPending({ month: to, day });
+            goTo(to);
+          }}
+          focusDay={pending !== null && pending.month.year === view.month.year && pending.month.month === view.month.month ? pending.day : null}
+          onFocused={() => setPending(null)}
+        />
       )}
     </div>
   );
@@ -142,9 +159,11 @@ function YearField({ id, shown, onMonth }: { id: string; shown: YearMonth | null
   );
 }
 
-function Month({ view }: { view: CalendarView }) {
+function Month({ view, selected, ...grid }: CalendarGridProps) {
   const { t } = useTranslation();
   const noSunrise = view.days.some((d) => d.facts.sunrise_jd_ut === null);
+  // Ayrıntı: seçili gün bu aydaysa o, değilse bugün (bu aydaysa).
+  const shown = view.days.find((d) => d.date === selected) ?? view.days.find((d) => d.day === view.today) ?? null;
   return (
     <>
       {view.warnings.includes("LowPrecision") && (
@@ -152,12 +171,13 @@ function Month({ view }: { view: CalendarView }) {
           <strong>{t("badge.LowPrecision")}</strong>: {t("badgeHint.LowPrecision")}
         </p>
       )}
-      <CalendarGrid view={view} />
+      <CalendarGrid view={view} selected={shown?.date ?? null} {...grid} />
       {noSunrise && (
         <p className="panel-notice" role="note">
           {t("calendar.noSunriseNote", { mark: NO_SUNRISE_MARK })}
         </p>
       )}
+      <DayDetail view={view} day={shown} />
       <p className="panel-message">{t("calendar.method", { method: view.method })}</p>
     </>
   );
diff --git a/app/ui/calendar/calendar.css b/app/ui/calendar/calendar.css
index fe93449..e759622 100644
--- a/app/ui/calendar/calendar.css
+++ b/app/ui/calendar/calendar.css
@@ -50,6 +50,24 @@
 .calendar-today {
   color: var(--ink-strong);
 }
+.calendar-day.is-selected {
+  background: var(--select);
+}
+.calendar-day:focus-visible {
+  outline: 2px solid var(--ink);
+  outline-offset: -2px;
+}
+.calendar-detail {
+  display: flex;
+  flex-direction: column;
+  gap: 4px;
+  padding-top: 4px;
+  border-top: 1px solid var(--line);
+}
+.calendar-detail-title {
+  margin: 0;
+  font-weight: var(--fw-medium);
+}
 .calendar-tithi,
 .calendar-nakshatra,
 .calendar-events li {
````

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index d8bd953..9fe4a71 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -624,6 +624,17 @@ export const en: Messages = {
       TransitToNatal: "{{a}} → natal {{b}}",
       return: "{{graha}} return",
     },
+    gridHint: "Arrow keys move between days; Enter selects the day; Page Up and Page Down change the month.",
+    pickDay: "Select a day to see its details.",
+    dayTitle: "{{weekday}}, {{month}} {{day}}, {{year}}",
+    sunriseFacts: "Sunrise {{time}} · {{tithi}} · {{nakshatra}}",
+    fallbackFacts: "No sunrise · at 06:00 local mean time ({{time}}) {{tithi}} · {{nakshatra}}",
+    openDay: "Open the day panchang",
+    openDayHint: "Moves the transit date to this day; adds the Day panchang panel if the layout has none.",
+    noEvents: "No event of the chosen kinds falls on this day.",
+    events: "{{date}}: events of the day",
+    col: { when: "Time", event: "Event", where: "Sign · nakshatra" },
+    goTo: "{{when}}: move the transit date to this moment",
     field: { place: "place", month: "month (years 1–2999)", kinds: "event kinds", range: "date range" },
     method: "Method: {{method}}. A cell is a calendar day (midnight to midnight); tithi and nakshatra are the values at that day’s sunrise. Times are in the chosen place’s time.",
   },
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 6a5cc62..8b8e1d9 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -624,6 +624,17 @@ export const fr: Messages = {
       TransitToNatal: "{{a}} → {{b}} natal",
       return: "Retour de {{graha}}",
     },
+    gridHint: "Les flèches passent d’un jour à l’autre ; Entrée sélectionne le jour ; Page précédente et Page suivante changent de mois.",
+    pickDay: "Sélectionnez un jour pour en voir le détail.",
+    dayTitle: "{{weekday}} {{day}} {{month}} {{year}}",
+    sunriseFacts: "Lever du Soleil {{time}} · {{tithi}} · {{nakshatra}}",
+    fallbackFacts: "Pas de lever du Soleil · à 06:00, heure locale moyenne ({{time}}) {{tithi}} · {{nakshatra}}",
+    openDay: "Ouvrir le panchang du jour",
+    openDayHint: "Place la date de transit sur ce jour ; ajoute le panneau Panchang du jour s’il manque à la disposition.",
+    noEvents: "Aucun événement des types choisis ne tombe ce jour-là.",
+    events: "{{date}} : événements du jour",
+    col: { when: "Heure", event: "Événement", where: "Signe · nakshatra" },
+    goTo: "{{when}} : placer la date de transit sur cet instant",
     field: { place: "lieu", month: "mois (années 1–2999)", kinds: "types d’événements", range: "intervalle de dates" },
     method: "Méthode : {{method}}. Une case est un jour civil (de minuit à minuit) ; le tithi et le nakshatra sont les valeurs au lever du Soleil de ce jour. Les heures sont celles du lieu choisi.",
   },
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 3582e0c..031d046 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -623,6 +623,17 @@ export const tr = {
       TransitToNatal: "{{a}} → doğum {{b}}",
       return: "{{graha}} dönüşü",
     },
+    gridHint: "Ok tuşları günler arasında gezer; Enter günü seçer; Page Up ve Page Down ayı değiştirir.",
+    pickDay: "Ayrıntısını görmek için bir gün seçin.",
+    dayTitle: "{{day}} {{month}} {{year}} {{weekday}}",
+    sunriseFacts: "Gün doğumu {{time}} · {{tithi}} · {{nakshatra}}",
+    fallbackFacts: "Gün doğumu yok · 06:00 yerel ortalama saatte ({{time}}) {{tithi}} · {{nakshatra}}",
+    openDay: "Gün panchangını aç",
+    openDayHint: "Transit tarihini bu güne götürür; Gün panchangı paneli düzende yoksa ekler.",
+    noEvents: "Seçili türlerde bu güne düşen olay yok.",
+    events: "{{date}}: günün olayları",
+    col: { when: "Saat", event: "Olay", where: "Burç · nakshatra" },
+    goTo: "{{when}}: transit tarihini bu ana götür",
     field: { place: "yer", month: "ay (yıl 1–2999)", kinds: "olay türleri", range: "tarih aralığı" },
     method: "Yöntem: {{method}}. Hücre takvim günüdür (gece yarısından gece yarısına); tithi ve nakshatra o günün gün doğumundaki değerdir. Saatler seçilen yerin saatiyledir.",
   },
````

- [ ] **Step 4: Testler ve kapılar**

```bash
pnpm --dir app exec vitest run ui/calendar ui/i18n 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
```
Expected: ilk satırlar `4 passed (4)`; Vitest `101 passed (101)`, `1003 passed (1003)`; `✓ built`.

- [ ] **Step 5: Commit**

```bash
git add app/ui/calendar app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts
git diff --cached --stat | tail -1
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): calendar keyboard navigation, the day detail and the jump to the day panchang" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: `8 files changed, 495 insertions(+), 35 deletions(-)`; tarama çıktısız; `git status --short` boş.

---

### Task 6: Arayüz: olay süzgeci, doğum boylamına gelişler ve Ay'ın olayları

Yol haritası §5 Plan 29 görev (6); D7–D9, D21. Süzgeç panelin ayarıdır; doğum verisi yalnızca doğum türü seçiliyken komuta gider (bu davranış Task 4'ten beri isteğin içindedir; bu görev denetimleri, notu ve "kesildi" bildirimini ekler).

**Files:**
- Create: `app/ui/calendar/EventFilter.tsx`
- Modify: `app/ui/calendar/CalendarPanel.tsx`, `app/ui/calendar/CalendarPanel.test.tsx`, `app/ui/calendar/model.ts`, `app/ui/calendar/model.test.ts`, `app/ui/calendar/calendar.css`, `app/ui/i18n/tr.ts`, `app/ui/i18n/en.ts`, `app/ui/i18n/fr.ts`
- Test: `CalendarPanel.test.tsx` (18 → 22); `model.test.ts` (bir beklenti)

**Interfaces:**
- Consumes: Task 4–5'in dosyaları; `search/CheckSet`, `modules/muhurta` (`toggleMember`), `session/birth` (`timeUnknown`), `view/store` (`setPanelSettings`).
- Produces: `EventFilter({ panel, filter, natalApprox })`; `model.ts`'te `MAX_EVENTS = 300`; i18n `calendar.filter.*`, `calendar.natalApprox`, `calendar.truncated`. Panel ayarları: `kinds: SkyEventKind[]`, `moon: boolean`.

- [ ] **Step 1: Testler: süzgeç, ayarın kaydı, türsüz takvim, saat bilinmeyen doğum, kesilen liste**

````diff
diff --git a/app/ui/calendar/CalendarPanel.test.tsx b/app/ui/calendar/CalendarPanel.test.tsx
index 9f42766..dea346c 100644
--- a/app/ui/calendar/CalendarPanel.test.tsx
+++ b/app/ui/calendar/CalendarPanel.test.tsx
@@ -12,7 +12,7 @@ import { tr } from "../i18n/tr";
 import { MODULE_PANELS, type Panel } from "../layout/model";
 import { PANEL_REGISTRY } from "../panels/registry";
 import { calendarFixture } from "../test/calendarFixture";
-import { fixture } from "../test/fixture";
+import { fixture, unknownTimeOpened } from "../test/fixture";
 import { renderWithStores } from "../test/render";
 import { useTransitControl, type Transit } from "../transit/store";
 import { useLayoutStore } from "../view/store";
@@ -412,6 +412,80 @@ test("at the end of the range Page Down asks nothing; if the new month fails the
   expect(document.activeElement).toBe(screen.getByRole("region", { name: "Transit takvimi" }));
 });
 
+test("the filter lists the calendar kinds; a choice is the panel's setting and asks for the month again", async () => {
+  const { persistence } = renderPanel();
+  await grid();
+  const saved = () => persistence.saved.at(-1)?.viewState.panel_layout!.layouts.find((l) => l.id === "user:t")!.panels[0].settings;
+  const filter = screen.getByRole("group", { name: "Olay türleri" });
+  const boxes = within(filter).getAllByRole("checkbox") as HTMLInputElement[];
+  expect(boxes.map((b) => b.closest("label")?.textContent)).toEqual(["Burç girişi", "Nakshatra girişi", "R başlangıcı", "R bitişi", "Yeni ay", "Dolunay", "Doğum boylamına geliş"]);
+  // Varsayılan: girişler, dönüşler, yeni ay ve dolunay; doğum boylamına gelişler ve Ay kapalı.
+  expect(boxes.map((b) => b.checked)).toEqual([true, true, true, true, true, true, false]);
+  expect(screen.getByText("Olay türleri: 6 / 7 seçili")).toBeInTheDocument();
+  const moon = screen.getByRole("checkbox", { name: /^Ay'ın olayları da/ }) as HTMLInputElement;
+  expect(moon.checked).toBe(false);
+
+  // Doğum boylamına gelişler: tür isteğe girer ve açık haritanın girdisi ancak şimdi gönderilir.
+  await act(async () => fireEvent.click(within(filter).getByRole("checkbox", { name: "Doğum boylamına geliş" })));
+  expect(requests()).toHaveLength(2);
+  expect(requests()[1].filter).toEqual({ kinds: CAL.ids.kinds, moon: false });
+  expect(requests()[1].natal).toEqual(CAL.request_full.natal);
+  expect(screen.getByText("Olay türleri: 7 / 7 seçili")).toBeInTheDocument();
+  // Ay'ın olayları: ayrı bir seçimdir.
+  await act(async () => fireEvent.click(moon));
+  expect(requests()[2]).toEqual(CAL.request_full);
+  const table = await grid();
+  await act(async () => {});
+  expect(events(cell(table, "2026-11-24")).at(-1)).toBe("+2 olay");
+  expect(saved()).toEqual({ kinds: CAL.ids.kinds, moon: true });
+  // Gün ayrıntısı kesilmez: günün beş olayı da listelenir.
+  key(cell(table, "2026-11-24"), "Enter");
+  expect(rowsOf(within(detail()).getByRole("table"))).toEqual([
+    ["02:02", "Ay: nakshatra girişi", "Krittika"],
+    ["07:24", "Ay: burç girişi", "Boğ"],
+    ["11:09", "Güneş, doğum Venüs boylamında", "Akr"],
+    ["20:23", "Dolunay", "Boğ · Krittika"],
+    ["23:25", "Ay: nakshatra girişi", "Rohini"],
+  ]);
+});
+
+test("with no kind chosen the grid still shows the facts of every day", async () => {
+  vi.mocked(api.monthCalendar).mockImplementation(async (request) => {
+    const view = calendarFixture().month;
+    return ok(request.filter.kinds.length === 0 ? { ...view, days: view.days.map((d) => ({ ...d, events: [] })) } : view);
+  });
+  renderPanel({ kinds: [], moon: false });
+  const table = await grid();
+  expect(requests()[0].filter).toEqual({ kinds: [], moon: false });
+  expect(requests()[0].natal).toBeNull();
+  expect(screen.getByText("Olay türleri: 0 / 7 seçili")).toBeInTheDocument();
+  expect(table.querySelectorAll("td[data-date]")).toHaveLength(30);
+  expect(table.querySelectorAll("li")).toHaveLength(0);
+  expect(lines(cell(table, "2026-11-25"))).toEqual(["25bugün", "K1 PratipadaKrishna 1 · Pratipada", "Rohini"]);
+  // Bir tür açılınca seçim Rust'ın sırasıyla kaydedilir.
+  await act(async () => fireEvent.click(screen.getByRole("checkbox", { name: "Dolunay" })));
+  await act(async () => fireEvent.click(screen.getByRole("checkbox", { name: "Burç girişi" })));
+  expect(requests().at(-1)?.filter.kinds).toEqual(["SignIngress", "FullMoon"]);
+});
+
+test("with an unknown birth time the natal kind is marked approximate and the input says so", async () => {
+  renderPanel({ kinds: ["TransitToNatal"] }, { opened: unknownTimeOpened() });
+  await grid();
+  expect(requests()[0].natal).toMatchObject({ time_known: false, jd_ut: FX.opened.chart.jd_ut });
+  expect(screen.getByText("Doğum saati bilinmiyor: doğum boylamları öğlen 12:00 içindir; doğum Ay'ına gelişler yaklaşıktır.")).toHaveAttribute("role", "note");
+  // Tür seçili değilken not da doğum verisi de yoktur.
+  await act(async () => fireEvent.click(screen.getByRole("checkbox", { name: "Doğum boylamına geliş" })));
+  expect(requests().at(-1)?.natal).toBeNull();
+  expect(screen.queryByText(/^Doğum saati bilinmiyor/)).toBeNull();
+});
+
+test("a cut event list is said above the grid", async () => {
+  vi.mocked(api.monthCalendar).mockImplementation(async () => ok({ ...calendarFixture().month, truncated: true }));
+  renderPanel();
+  await grid();
+  expect(screen.getByText("Olay listesi 300 olayda kesildi: sonraki olaylar gösterilmiyor. Tür seçimini daraltın.")).toHaveAttribute("role", "note");
+});
+
 test("without a current place the panel asks for one and nothing is computed; the chart's place can be used", async () => {
   const { persistence } = renderPanel({}, { prefs: FX.prefs.prefs });
   expect(screen.getByRole("note")).toHaveTextContent("Bulunulan yer seçilmedi.");
diff --git a/app/ui/calendar/model.test.ts b/app/ui/calendar/model.test.ts
index 1b0a4a3..db94658 100644
--- a/app/ui/calendar/model.test.ts
+++ b/app/ui/calendar/model.test.ts
@@ -2,7 +2,7 @@ import { expect, test } from "vitest";
 import type { CalendarDayView, SkyEvent } from "../bindings";
 import { i18n, LANGUAGES } from "../i18n";
 import { calendarFixture } from "../test/calendarFixture";
-import { CALENDAR_KINDS, CELL_EVENTS, DEFAULT_CALENDAR_KINDS, eventLong, eventShort, eventWhere, MAX_YEAR, MIN_YEAR, parseFilter, WEEK_START, weekdayOrder, weekRows } from "./model";
+import { CALENDAR_KINDS, CELL_EVENTS, DEFAULT_CALENDAR_KINDS, eventLong, eventShort, eventWhere, MAX_EVENTS, MAX_YEAR, MIN_YEAR, parseFilter, WEEK_START, weekdayOrder, weekRows } from "./model";
 
 const t = i18n.getFixedT("tr");
 const days = (row: (CalendarDayView | null)[]) => row.map((d) => d?.day ?? null);
@@ -12,6 +12,7 @@ test("the kind lists and the year range are Rust's", () => {
   expect([...CALENDAR_KINDS]).toEqual(ids.kinds);
   expect([...DEFAULT_CALENDAR_KINDS]).toEqual(ids.default_kinds);
   expect([MIN_YEAR, MAX_YEAR]).toEqual([ids.min_year, ids.max_year]);
+  expect(MAX_EVENTS).toBe(ids.max_events);
   expect(CELL_EVENTS).toBe(3);
 });
 
````

- [ ] **Step 2: Testlerin düştüğünü gör**

Run: `pnpm --dir app exec vitest run ui/calendar 2>&1 | grep -E "Test Files|Tests " | head -3`
Expected: `Test Files  2 failed | 1 passed (3)`, `Tests  5 failed | 26 passed (31)`: dört yeni panel testi (süzgeç grubu, özet metni, not ve bildirim bulunamaz) ve `model.test.ts`'in `MAX_EVENTS` beklentisi düşer. Çıktının başında `Failed Tests 5` satırı da görünür.

- [ ] **Step 3: Süzgeç bileşeni, panel bağlantısı ve metinler**

**Yeni dosya:** `app/ui/calendar/EventFilter.tsx`

````tsx
// Takvimin olay süzgeci (Plan 29): olay türleri ve Ay'ın kendi olayları. Seçim panelin ayarıdır
// ve düzenle birlikte saklanır; burada hesap yoktur (C4): seçim `month_calendar`'a gider ve
// olayları Rust arar. Doğum boylamına gelişler açık haritanın doğum verisini ister; veri yalnızca
// o tür seçiliyken gönderilir.
import { useId } from "react";
import { useTranslation } from "react-i18next";
import type { CalendarFilter } from "../bindings";
import type { Panel } from "../layout/model";
import { toggleMember } from "../modules/muhurta";
import { CheckSet } from "../search/CheckSet";
import "../search/search.css";
import { useLayoutStore } from "../view/store";
import { CALENDAR_KINDS } from "./model";

export function EventFilter({ panel, filter, natalApprox }: { panel: Panel; filter: CalendarFilter; natalApprox: boolean }) {
  const { t } = useTranslation();
  const { setPanelSettings } = useLayoutStore();
  const id = useId();
  return (
    <div className="calendar-filter">
      <CheckSet
        legend={t("calendar.filter.kinds")}
        fold={t("calendar.filter.summary", { n: filter.kinds.length, of: CALENDAR_KINDS.length })}
        options={CALENDAR_KINDS.map((k) => ({ value: k, label: t(`eventSearch.kind.${k}`) }))}
        selected={filter.kinds}
        onToggle={(k) => setPanelSettings(panel.id, { kinds: toggleMember(filter.kinds, k, CALENDAR_KINDS) })}
      />
      <label htmlFor={`${id}-moon`}>
        <input id={`${id}-moon`} type="checkbox" checked={filter.moon} onChange={() => setPanelSettings(panel.id, { moon: !filter.moon })} />
        {t("calendar.filter.moon")}
      </label>
      {natalApprox && (
        <p className="panel-notice" role="note">
          {t("calendar.natalApprox")}
        </p>
      )}
    </div>
  );
}
````

````diff
diff --git a/app/ui/calendar/CalendarPanel.tsx b/app/ui/calendar/CalendarPanel.tsx
index 79d92af..c0efb31 100644
--- a/app/ui/calendar/CalendarPanel.tsx
+++ b/app/ui/calendar/CalendarPanel.tsx
@@ -3,7 +3,8 @@
 // günler, olgular ve olaylar `month_calendar`'dan gelir; burada hesap yoktur (C4): "bugün",
 // önceki ve sonraki ay da Rust'tan okunur. Yer panelin ayarıdır ve düzenle birlikte saklanır
 // (varsayılan: bulunulan yer); gösterilen ay saklanmaz, panel her açılışta bugünün ayını gösterir.
-// Izgaranın altında seçili günün ayrıntısı durur: seçim yoksa bugünün (bu aydaysa).
+// Izgaranın altında seçili günün ayrıntısı durur: seçim yoksa bugünün (bu aydaysa). Olay
+// süzgeci (türler ve Ay'ın olayları) de panelin ayarıdır.
 // Takvim olgudur: "uygun gün" hükmü, puan ve açıklama metni yoktur.
 import { useId, useState, type FormEvent } from "react";
 import { useTranslation } from "react-i18next";
@@ -15,11 +16,13 @@ import { PlaceControls, PlaceMissing, usePanelPlace } from "../panels/MuhurtaPla
 import "../panels/panels.css";
 import { PanelError, PanelSkeleton, usePanelLoadPhase } from "../panels/states";
 import type { PanelProps } from "../panels/types";
+import { timeUnknown } from "../session/birth";
 import { useSession } from "../session/store";
 import "./calendar.css";
 import { CalendarGrid, NO_SUNRISE_MARK, type CalendarGridProps } from "./CalendarGrid";
 import { DayDetail } from "./DayDetail";
-import { MAX_YEAR, MIN_YEAR, parseFilter } from "./model";
+import { EventFilter } from "./EventFilter";
+import { MAX_EVENTS, MAX_YEAR, MIN_YEAR, parseFilter } from "./model";
 import { useCalendar } from "./useCalendar";
 
 const MONTHS = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12] as const;
@@ -63,6 +66,7 @@ function Calendar({ panel, opened }: { panel: Panel; opened: OpenedClient }) {
         <PlaceControls panel={panel} opened={opened} at={place} />
       </div>
       <MonthNav view={view} requested={month} onMonth={goTo} />
+      <EventFilter panel={panel} filter={filter} natalApprox={natal && timeUnknown(opened)} />
       {place.place === null ? (
         <PlaceMissing choice={place.choice} />
       ) : shownError ? (
@@ -171,6 +175,11 @@ function Month({ view, selected, ...grid }: CalendarGridProps) {
           <strong>{t("badge.LowPrecision")}</strong>: {t("badgeHint.LowPrecision")}
         </p>
       )}
+      {view.truncated && (
+        <p className="panel-notice" role="note">
+          {t("calendar.truncated", { max: MAX_EVENTS })}
+        </p>
+      )}
       <CalendarGrid view={view} selected={shown?.date ?? null} {...grid} />
       {noSunrise && (
         <p className="panel-notice" role="note">
diff --git a/app/ui/calendar/calendar.css b/app/ui/calendar/calendar.css
index e759622..900a993 100644
--- a/app/ui/calendar/calendar.css
+++ b/app/ui/calendar/calendar.css
@@ -9,6 +9,13 @@
   outline: 2px solid var(--danger);
   outline-offset: -1px;
 }
+.calendar-filter {
+  display: flex;
+  flex-direction: column;
+  align-items: flex-start;
+  gap: 4px;
+  color: var(--text-2);
+}
 .calendar-grid {
   width: 100%;
   min-width: 560px;
diff --git a/app/ui/calendar/model.ts b/app/ui/calendar/model.ts
index 77113c6..2aa9dae 100644
--- a/app/ui/calendar/model.ts
+++ b/app/ui/calendar/model.ts
@@ -16,6 +16,8 @@ export const DEFAULT_CALENDAR_KINDS = ["SignIngress", "NakshatraIngress", "Stati
 /** Rust `CALENDAR_YEARS`. */
 export const MIN_YEAR = 1;
 export const MAX_YEAR = 2999;
+/** Rust `CALENDAR_MAX_EVENTS`: bir ayın taşıdığı en çok olay. */
+export const MAX_EVENTS = 300;
 /** Bir hücrede yazılan en çok olay; fazlası sayılır ve gün ayrıntısında listelenir. */
 export const CELL_EVENTS = 3;
 
````

````diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
index 9fe4a71..b048eeb 100644
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -635,6 +635,9 @@ export const en: Messages = {
     events: "{{date}}: events of the day",
     col: { when: "Time", event: "Event", where: "Sign · nakshatra" },
     goTo: "{{when}}: move the transit date to this moment",
+    filter: { kinds: "Event kinds", summary: "Event kinds: {{n}} of {{of}} chosen", moon: "The Moon’s events too (sign and nakshatra ingresses, arrivals at natal longitudes)" },
+    natalApprox: "The birth time is unknown: natal longitudes are for 12:00 noon; arrivals at the natal Moon are approximate.",
+    truncated: "The event list was cut at {{max}} events: later events are not shown. Choose fewer kinds.",
     field: { place: "place", month: "month (years 1–2999)", kinds: "event kinds", range: "date range" },
     method: "Method: {{method}}. A cell is a calendar day (midnight to midnight); tithi and nakshatra are the values at that day’s sunrise. Times are in the chosen place’s time.",
   },
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
index 8b8e1d9..3539a24 100644
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -635,6 +635,9 @@ export const fr: Messages = {
     events: "{{date}} : événements du jour",
     col: { when: "Heure", event: "Événement", where: "Signe · nakshatra" },
     goTo: "{{when}} : placer la date de transit sur cet instant",
+    filter: { kinds: "Types d’événements", summary: "Types d’événements : {{n}} sur {{of}} choisis", moon: "Aussi les événements de la Lune (entrées en signe et en nakshatra, arrivées sur les longitudes natales)" },
+    natalApprox: "L’heure de naissance est inconnue : les longitudes natales valent pour 12:00 ; les arrivées sur la Lune natale sont approximatives.",
+    truncated: "La liste a été coupée à {{max}} événements : les suivants ne sont pas affichés. Choisissez moins de types.",
     field: { place: "lieu", month: "mois (années 1–2999)", kinds: "types d’événements", range: "intervalle de dates" },
     method: "Méthode : {{method}}. Une case est un jour civil (de minuit à minuit) ; le tithi et le nakshatra sont les valeurs au lever du Soleil de ce jour. Les heures sont celles du lieu choisi.",
   },
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
index 031d046..4988cd3 100644
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -634,6 +634,9 @@ export const tr = {
     events: "{{date}}: günün olayları",
     col: { when: "Saat", event: "Olay", where: "Burç · nakshatra" },
     goTo: "{{when}}: transit tarihini bu ana götür",
+    filter: { kinds: "Olay türleri", summary: "Olay türleri: {{n}} / {{of}} seçili", moon: "Ay'ın olayları da (burç ve nakshatra girişleri, doğum boylamına gelişleri)" },
+    natalApprox: "Doğum saati bilinmiyor: doğum boylamları öğlen 12:00 içindir; doğum Ay'ına gelişler yaklaşıktır.",
+    truncated: "Olay listesi {{max}} olayda kesildi: sonraki olaylar gösterilmiyor. Tür seçimini daraltın.",
     field: { place: "yer", month: "ay (yıl 1–2999)", kinds: "olay türleri", range: "tarih aralığı" },
     method: "Yöntem: {{method}}. Hücre takvim günüdür (gece yarısından gece yarısına); tithi ve nakshatra o günün gün doğumundaki değerdir. Saatler seçilen yerin saatiyledir.",
   },
````

- [ ] **Step 4: Testler ve kapılar**

```bash
pnpm --dir app exec vitest run ui/calendar ui/i18n 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
```
Expected: ilk satırlar `4 passed (4)`; Vitest `101 passed (101)`, `1007 passed (1007)`; `✓ built`.

- [ ] **Step 5: Commit**

```bash
git add app/ui/calendar app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts
git diff --cached --stat | tail -1
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): calendar event filter with natal arrivals and the Moon's own events" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: `9 files changed, 145 insertions(+), 4 deletions(-)`; tarama çıktısız; `git status --short` boş.

---

### Task 7: Uçtan uca ve görsel akışların kodu; `month_calendar` taklidi

D22, D23. Bu görev üç dosya yazar ve **hiçbirini koşmaz**: akışları kontrolör Task 8'in "[kontrolör]" adımlarında koşar. Kod, bu plan yazılırken deponun bir kopyasında özel bir portta koşulmuş ve geçmiştir (üç akış, üç temel). Yürüten ajanın kapısı tip denetimidir (`pnpm --dir app build`, `tsc --noEmit -p e2e` dahil).

**Files:**
- Create: `app/e2e/calendar.desk.spec.ts`, `app/e2e/calendar.visual.spec.ts`
- Modify: `app/e2e/ipc.ts` (paylaşılan dosya; on satır)
- Test: yürüten ajan için yalnızca tip denetimi; kontrolör için e2e 33 → 36, görsel 15 → 18

**Interfaces:**
- Consumes: fikstürün `calendar` alanı (`month`, `month_full`, `month_next`, `ids`, `request`); `e2e/ipc.ts` (`mockIpc`, `ipcCalls`, `FIXTURE`).
- Produces: `month_calendar` taklidi (ay verilmemişse ya da Kasım 2026 ise `month`, süzgeç Ay'ı ya da doğum türünü içeriyorsa `month_full`; Aralık 2026 ise `month_next`; başka ay `invalid_calendar_input` hatası).

- [ ] **Step 1: Taklit ve iki akış dosyası**

````diff
diff --git a/app/e2e/ipc.ts b/app/e2e/ipc.ts
index 751f48f..a8c4417 100644
--- a/app/e2e/ipc.ts
+++ b/app/e2e/ipc.ts
@@ -177,6 +177,16 @@ export async function mockIpc(page: Page, { session, unknownTime = false, partne
           const cursor = args.cursor as { cursor: { chunk: number } } | null;
           return fixture.muhurta.pages[cursor === null ? 0 : cursor.cursor.chunk];
         },
+        // Transit takvimi (Plan 29): ay yeniden hesaplanmaz, fikstürün ayları döner. Ay verilmemişse
+        // ya da Kasım 2026 ise bugünün ayı (süzgeç Ay'ı ya da doğum boylamına gelişleri içeriyorsa
+        // bütün olaylarıyla), Aralık 2026 ise sonraki ay; başka bir ay Rust'taki gibi alan adlı hatadır.
+        month_calendar: (args) => {
+          const request = args.request as { month: { year: number; month: number } | null; filter: { kinds: string[]; moon: boolean } };
+          const asked = request.month;
+          if (asked !== null && asked.year === 2026 && asked.month === 12) return fixture.calendar.month_next;
+          if (asked !== null && !(asked.year === 2026 && asked.month === 11)) throw { code: "invalid_calendar_input", params: { field: "month" } };
+          return request.filter.moon || request.filter.kinds.includes("TransitToNatal") ? fixture.calendar.month_full : fixture.calendar.month;
+        },
         load_muhurta_filters: () => muhurtaFilters,
         save_muhurta_filters: (args) => {
           muhurtaFilters = args.filters as unknown[];
````

**Yeni dosya:** `app/e2e/calendar.desk.spec.ts`

````ts
// Transit takvimi akışları (yol haritası Plan 29). Ayrı dosyadır: `desk.spec.ts` paralel planlarla
// paylaşılır; "e2e" projesi adı `desk.spec.ts` ile biten her dosyayı koşar.
import { expect, test, type Page } from "@playwright/test";
import { FIXTURE, ipcCalls, mockIpc } from "./ipc";

type CalendarCall = { request: { month: { year: number; month: number } | null; place: unknown; filter: { kinds: string[]; moon: boolean }; natal: { jd_ut: number; time_known: boolean } | null } };
type Saved = { viewState: { panel_layout: { active_layout_id: string; layouts: { id: string; panels: { kind: string; settings: Record<string, unknown> }[] }[] } } };

const NOVEMBER = "Kasım 2026 takvimi: gün doğumundaki tithi ve Ay nakshatrası, günün olayları";
const DECEMBER = "Aralık 2026 takvimi: gün doğumundaki tithi ve Ay nakshatrası, günün olayları";

async function open(page: Page, options: Parameters<typeof mockIpc>[1] = { session: true }) {
  await mockIpc(page, options);
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
}

async function toggle(page: Page) {
  await page.keyboard.press("Meta+k");
  const palette = page.getByRole("dialog", { name: "Komut paleti" });
  await palette.getByRole("combobox").fill("takvim");
  await palette.getByRole("option", { name: /Transit takvimi panelini aç\/kapat/ }).click();
  await expect(page.getByRole("dialog")).toHaveCount(0);
}

const calls = async (page: Page) => (await ipcCalls(page, "month_calendar")) as CalendarCall[];
const savedPanels = async (page: Page) => {
  const layout = ((await ipcCalls(page, "save_view_state")).at(-1) as Saved | undefined)?.viewState.panel_layout;
  return layout?.layouts.find((l) => l.id === layout.active_layout_id)?.panels;
};

test("transit calendar: a palette panel; the place, the month grid, days by keyboard, the day detail and the day panchang", async ({ page }) => {
  await open(page);
  // Takvim panel açılmadan istenmez (ileri modüller spec'i K1).
  expect(await calls(page)).toHaveLength(0);
  await toggle(page);
  const panel = page.getByRole("region", { name: "Transit takvimi" });
  await expect(panel).toBeFocused();
  await expect(panel).toBeInViewport();
  // Bulunulan yer seçilmemiş: panel yer ister, ay hesaplanmaz.
  await expect(panel.getByRole("note")).toContainText("Bulunulan yer seçilmedi.");
  expect(await calls(page)).toHaveLength(0);

  // Bulunulan yer penceresi: açık haritanın doğum yeri. Klavyeyle açılır (WebKit tıklanan
  // düğmeye odak vermez): pencere kapanınca odak düğmeye döner.
  await panel.getByRole("button", { name: "Bulunulan yeri seç" }).focus();
  await page.keyboard.press("Enter");
  const dialog = page.getByRole("dialog", { name: "Bulunulan yer" });
  await dialog.getByRole("button", { name: /^Açık haritanın doğum yerini kullan/ }).click();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toHaveCount(0);

  // Ay ızgarası: bir istek; ay verilmez ("bugün" Rust'ta okunur), yerin adı ve doğum verisi gitmez.
  const grid = panel.getByRole("grid", { name: NOVEMBER });
  await expect(grid.locator("td[data-date]")).toHaveCount(30);
  const first = await calls(page);
  expect(first).toHaveLength(1);
  expect(first[0].request).toMatchObject({ month: null, place: FIXTURE.muhurta.place, filter: FIXTURE.calendar.request.filter, natal: null });
  await expect(grid.getByRole("columnheader")).toHaveText(["PztPazartesi", "SalSalı", "ÇarÇarşamba", "PerPerşembe", "CumCuma", "CmtCumartesi", "PazPazar"]);
  await expect(grid.locator("tbody tr")).toHaveCount(6);
  // Bugünün hücresi: gün doğumundaki tithi ve Ay nakshatrası, günün olayı saatiyle.
  const today = grid.locator('td[data-date="2026-11-25"]');
  await expect(today).toHaveClass(/is-today/);
  await expect(today.locator(".calendar-day-head")).toHaveText("25bugün");
  await expect(today.locator(".calendar-tithi")).toContainText("K1 Pratipada");
  await expect(today.locator(".calendar-nakshatra")).toHaveText("Rohini");
  await expect(today.locator("li")).toHaveText(["13:41 Me → VishakhaMerkür: nakshatra girişi · Vishakha"]);
  await expect(grid.locator('td[data-date="2026-11-09"] li')).toContainText(["12:32 Yeni ay"]);
  await expect(grid.locator('td[data-date="2026-11-16"] li')).toContainText(["19:43 Gü → Akr"]);
  await expect(panel.getByText(/^Yöntem: calendar\.month_v1\./)).toBeVisible();

  // Ayrıntı seçim yokken bugünündür.
  const detail = panel.locator("section.calendar-detail");
  await expect(detail).toHaveAttribute("data-detail", "2026-11-25");
  await expect(detail.getByText("Gün doğumu 06:55 · Krishna 1 · Pratipada · Rohini")).toBeVisible();

  // Klavye: ızgara tek sekme durağıdır; oklar günler arasında gezer, Enter günü seçer.
  await today.focus();
  await page.keyboard.press("ArrowLeft");
  await expect(grid.locator('td[data-date="2026-11-24"]')).toBeFocused();
  await page.keyboard.press("ArrowUp");
  await expect(grid.locator('td[data-date="2026-11-17"]')).toBeFocused();
  await page.keyboard.press("ArrowLeft");
  await page.keyboard.press("Enter");
  await expect(detail).toHaveAttribute("data-detail", "2026-11-16");
  await expect(detail.getByText("16 Kasım 2026 Pazartesi", { exact: true })).toBeVisible();
  await expect(detail.getByRole("table").locator("tbody tr").first().locator("th, td")).toHaveText(["19:43", "Güneş: burç girişi", "Akr"]);
  await expect(grid.locator('td[aria-selected="true"]')).toHaveAttribute("data-date", "2026-11-16");
  await expect(grid.locator('td[tabindex="0"]')).toHaveCount(1);

  // Page Down: sonraki ay (Rust'ın verdiği ay istenir) ve odak aynı günde durur.
  await page.keyboard.press("PageDown");
  const december = panel.getByRole("grid", { name: DECEMBER });
  await expect(december.locator("td[data-date]")).toHaveCount(31);
  await expect(december.locator('td[data-date="2026-12-16"]')).toBeFocused();
  expect((await calls(page)).at(-1)?.request.month).toEqual({ year: 2026, month: 12 });
  await expect(december.locator("td.is-today")).toHaveCount(0);
  await expect(panel.getByRole("combobox", { name: "Ay" })).toHaveValue("12");
  // Başka bir ayda seçim yoktur: ayrıntı gün ister.
  await expect(panel.getByText("Ayrıntısını görmek için bir gün seçin.")).toBeVisible();
  await page.keyboard.press("PageUp");
  await expect(grid.locator('td[data-date="2026-11-16"]')).toBeFocused();

  // Olayın saati transit tarihini o ana götürür; "Gün panchangını aç" paneli ekler.
  await detail.getByRole("button", { name: "19:43: transit tarihini bu ana götür" }).click();
  await detail.getByRole("button", { name: "Gün panchangını aç" }).click();
  const day = page.getByRole("region", { name: "Gün panchangı" });
  await expect(day).toBeVisible();
  const dayCalls = (await ipcCalls(page, "day_panchang")) as { at: unknown }[];
  expect(dayCalls.at(-1)?.at).toEqual({ Jd: FIXTURE.calendar.month.days[15].panchang_jd_ut });
  await expect.poll(async () => (await savedPanels(page))?.map((p) => p.kind).filter((k) => k === "calendar" || k === "muhurta_day")).toEqual(["calendar", "muhurta_day"]);

  // Aynı komut paneli kaldırır ve odak gövdeye düşmez.
  await toggle(page);
  await expect(panel).toHaveCount(0);
  expect(await page.evaluate(() => document.activeElement !== null && document.activeElement !== document.body)).toBe(true);
});

test("transit calendar filter: natal arrivals and the Moon's events are asked only when chosen and are saved with the layout", async ({ page }) => {
  await open(page);
  await toggle(page);
  const panel = page.getByRole("region", { name: "Transit takvimi" });
  // Yer: haritanın doğum yeri (panelin ayarı).
  await panel.getByRole("combobox", { name: "Yer" }).selectOption("chart");
  const grid = panel.getByRole("grid", { name: NOVEMBER });
  await expect(grid.locator('td[data-date="2026-11-24"] li')).toHaveCount(1);
  expect((await calls(page))[0].request.natal).toBeNull();

  // Süzgeç kapalı bir bölümdedir; özet seçili tür sayısını söyler.
  await panel.locator("summary", { hasText: "Olay türleri: 6 / 7 seçili" }).click();
  await panel.getByRole("checkbox", { name: "Doğum boylamına geliş", exact: true }).check();
  await expect(panel.locator("summary", { hasText: "Olay türleri: 7 / 7 seçili" })).toBeVisible();
  await panel.getByRole("checkbox", { name: /^Ay'ın olayları da/ }).check();
  // Doğum verisi ancak tür seçilince gider.
  await expect.poll(async () => (await calls(page)).at(-1)?.request.filter).toEqual({ kinds: FIXTURE.calendar.ids.kinds, moon: true });
  expect((await calls(page)).at(-1)?.request.natal).toMatchObject({ jd_ut: 2451545, time_known: true });
  // Hücre en çok üç olay yazar ve gerisini sayar; ayrıntı hepsini listeler.
  const full = grid.locator('td[data-date="2026-11-24"]');
  await expect(full.locator("li")).toHaveCount(4);
  await expect(full.locator("li.calendar-more")).toHaveText("+2 olay");
  await full.click();
  const detail = panel.locator("section.calendar-detail");
  await expect(detail.getByRole("table").locator("tbody tr")).toHaveCount(5);
  await expect(detail.getByRole("table").locator('tbody tr[data-event="TransitToNatal"] td').first()).toHaveText("Güneş, doğum Venüs boylamında");
  await expect.poll(async () => (await savedPanels(page))?.find((p) => p.kind === "calendar")?.settings).toEqual({ place: "chart", kinds: FIXTURE.calendar.ids.kinds, moon: true });

  // İngilizcede hafta Pazar başlar: aynı ay beş satırdır.
  await page.evaluate(() => localStorage.setItem("jyotish-desk.language.v1", "en"));
  await page.reload();
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  await page.keyboard.press("Meta+k");
  const palette = page.getByRole("dialog", { name: "Command palette" });
  await palette.getByRole("combobox").fill("calendar");
  await palette.getByRole("option", { name: /Show or hide the Transit calendar panel/ }).click();
  const english = page.getByRole("region", { name: "Transit calendar" });
  await english.getByRole("combobox", { name: "Place" }).selectOption("chart");
  const englishGrid = english.getByRole("grid", { name: /^Calendar for November 2026/ });
  await expect(englishGrid.locator("tbody tr")).toHaveCount(5);
  await expect(englishGrid.getByRole("columnheader").first()).toHaveText("SunSunday");
});

test("transit calendar with an unknown birth time: natal arrivals are marked approximate", async ({ page }) => {
  await open(page, { session: true, unknownTime: true });
  await toggle(page);
  const panel = page.getByRole("region", { name: "Transit takvimi" });
  await panel.getByRole("combobox", { name: "Yer" }).selectOption("chart");
  await expect(panel.getByRole("grid", { name: NOVEMBER })).toBeVisible();
  await expect(panel.getByText(/^Doğum saati bilinmiyor/)).toHaveCount(0);
  await panel.locator("summary", { hasText: /^Olay türleri/ }).click();
  await panel.getByRole("checkbox", { name: "Doğum boylamına geliş", exact: true }).check();
  await expect(panel.getByText("Doğum saati bilinmiyor: doğum boylamları öğlen 12:00 içindir; doğum Ay'ına gelişler yaklaşıktır.")).toBeVisible();
  await expect.poll(async () => (await calls(page)).at(-1)?.request.natal?.time_known).toBe(false);
});
````

**Yeni dosya:** `app/e2e/calendar.visual.spec.ts`

````ts
// Transit takviminin görsel temelleri (yol haritası Plan 29; yalnızca yerel: `pnpm e2e:visual`).
// Ayrı dosyadır: `visual.spec.ts` paralel planlarla paylaşılır; "visual" projesi adı
// `visual.spec.ts` ile biten her dosyayı koşar. Fikstürün ayı (Kasım 2026, Delhi), açık ve koyu
// tema: ay ızgarası (bugün işaretli) ve bütün olaylarla dolu ızgara.
import { expect, test, type Page } from "@playwright/test";
import { mockIpc } from "./ipc";

async function openCalendar(page: Page, theme: "light" | "dark") {
  await mockIpc(page, { session: true });
  await page.addInitScript((t) => localStorage.setItem("jyotish-desk.theme.v1", t), theme);
  await page.goto("/");
  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
  // Kenar çubuğu kapalı: temel görüntüler pano genişliğine bağlı değil.
  await page.keyboard.press("Meta+Backslash");
  await expect(page.getByRole("complementary", { name: "Danışanlar" })).toHaveCount(0);
  await page.keyboard.press("Meta+k");
  const palette = page.getByRole("dialog", { name: "Komut paleti" });
  await palette.getByRole("combobox").fill("takvim");
  await palette.getByRole("option", { name: /Transit takvimi panelini aç\/kapat/ }).click();
  const panel = page.getByRole("region", { name: "Transit takvimi" });
  await panel.getByRole("combobox", { name: "Yer" }).selectOption("chart");
  const grid = panel.locator("table.calendar-grid");
  await expect(grid.locator("td[data-date]")).toHaveCount(30);
  return { panel, grid };
}

for (const theme of ["light", "dark"] as const) {
  test(`calendar ${theme}`, async ({ page }) => {
    const { grid } = await openCalendar(page, theme);
    await grid.scrollIntoViewIfNeeded();
    await expect(grid).toHaveScreenshot(`calendar-${theme}.png`);
  });
}

test("calendar full light", async ({ page }) => {
  const { panel, grid } = await openCalendar(page, "light");
  await panel.locator("summary", { hasText: /^Olay türleri/ }).click();
  await panel.getByRole("checkbox", { name: "Doğum boylamına geliş", exact: true }).check();
  await panel.getByRole("checkbox", { name: /^Ay'ın olayları da/ }).check();
  await expect(grid.locator("li.calendar-more").first()).toBeVisible();
  // Seçili gün: dolunay günü.
  await grid.locator('td[data-date="2026-11-24"]').click();
  await expect(grid.locator('td[aria-selected="true"]')).toHaveAttribute("data-date", "2026-11-24");
  await grid.scrollIntoViewIfNeeded();
  await expect(grid).toHaveScreenshot("calendar-full-light.png");
});
````

- [ ] **Step 2: Tip denetimi ve kapılar**

```bash
pnpm --dir app build 2>&1 | grep -E "error TS|✓ built"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
ls app/e2e/calendar.visual.spec.ts-snapshots 2>/dev/null
```
Expected: yalnızca `✓ built` (TS hatası yok); Vitest `101 passed (101)`, `1007 passed (1007)`; `ls` çıktısız (temel PNG'leri kontrolör üretir; yürüten ajan `pnpm --dir app e2e` ve `e2e:visual` koşmaz).

- [ ] **Step 3: Commit**

```bash
git add app/e2e/ipc.ts app/e2e/calendar.desk.spec.ts app/e2e/calendar.visual.spec.ts
git diff --cached --stat | tail -1
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "test(e2e): transit calendar flows and visual specs; month_calendar mock" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: `3 files changed, 229 insertions(+)`; tarama çıktısız; `git status --short` boş.

---

### Task 8: Kapılar, REVIEW.md ve elle denetim kapısı

Yol haritası §5 Plan 29 görev (8) ve "Sahibin denetimi"; Y3. Bu görev iki belge dosyası üretir (REVIEW kaydı ve el örnekleri kılavuzunun paragrafı), bütün kapıları koşar ve el örneklerini bağımsız olarak yeniden türetir. Kod değişmez; bulunan bir hata kendi düzeltme commit'iyle çözülür ve bu görev baştan koşulur. Aşağıda `BASE`, Task 1 Step 1'de `git log -1 --format=%h` ile okunan commit'tir (bu plan yazılırken `9727c55`).

**Files:**
- Create: `tests/golden/modules/calendar/REVIEW.md`
- Modify: `tests/golden/modules/README.md` (paylaşılan dosya; bir paragraf)

**Interfaces:**
- Consumes: Task 1–7'nin tamamı.
- Produces: okuma kurallarının ve sahibin kararını bekleyen dokuz noktanın kaydı; yeşil kapılar; bağımsız yeniden türetmenin sonucu; kural ve gizlilik taramaları; kontrolöre devredilen **[kontrolör]** adımları ve sahibine devredilen **[insan]** maddeleri (ilerleme notlarına yazılır).

- [ ] **Step 1: REVIEW.md ve kılavuz paragrafı**

**Yeni dosya:** `tests/golden/modules/calendar/REVIEW.md`

````markdown
# Transit takvimi: okuma kuralları, sahibin kararını bekleyen noktalar ve gözden geçirme kaydı

Kaynak: yol haritası (`docs/superpowers/specs/2026-11-27-eksikler-yol-haritasi.md`) §4.11 ve §5
"Plan 29", Plan 29'un kararları (D1–D22) ve ileri modüller tasarım belgesinin K1–K5 kuralları.
Bu modül bellekten aktarılmış klasik tablo taşımaz (Y4'ün "doğrulanmamış tablo" notu gerekmez)
ve yeni astronomi içermez: olayları olay aramasının çekirdeği bulur, tithi ve nakshatra v1'in
panchang fonksiyonlarıyla okunur. Taşıdığı şey gün tanımı ve varsayılan seçimlerdir; bunlar
aşağıda sahibin kararını bekler. Bu dizinde JSON örneği yoktur ve hiçbir test bu klasörü
yüklemez: el örnekleri Rust birim testleridir ve türetmeleri test yorumlarında yazılıdır
(`crates/jyotish-core/src/calendar/grid.rs`, `crates/jyotish-core/src/calendar/events.rs`).
Beklenen değerler kuraldan elle türetilmiştir; motor çıktısı kopyalanmaz (K9). Hiçbir hücre
hüküm değildir: takvim olguları sayar, "uygun gün" demez.

## Okuma kuralları

1. **Hücre takvim günüdür.** Gün, seçilen yerin duvar saatiyle gece yarısından sonraki gece
   yarısınadır (`[başlangıç, bitiş)`). Yaz saati geçişinde gün 23 ya da 25 saat sürer. Saat
   diliminin takvimden atladığı gün (örnek: Samoa, 30 Aralık 2011) ızgarada yoktur.
2. **Günün olguları gün doğumundadır.** Hücrenin tithisi ve Ay nakshatrası, o takvim gününün
   içindeki gün doğumunda geçerli olan değerdir: o gün başlayan panchang gününün ilk değeri.
   Gün içinde değişen tithi ya da nakshatra hücrede yazılmaz; gün panchangı paneli bütün
   değişimleri saatleriyle verir. Bir tithi iki gün doğumunda birden geçerliyse iki hücrede de
   yazılır; hiçbir gün doğumuna rastlamayan tithi hiçbir hücrede yazılmaz.
3. **Tithi ve nakshatra.** Tithi, Ay − Güneş uzanımının 12°'lik dilimidir (1–30; 1–15 Shukla,
   16–30 Krishna). Nakshatra, Ay'ın sidereal boylamının 13°20′'lık dilimidir (1–27). İkisi de
   her zodyak kipinde sidereal boylamdan okunur. Dilim sınırı sonraki dilime aittir.
4. **Gün doğumu.** Gün doğumu hesap ayarındaki tanımla (disk kenarı ya da merkezi, kırılma)
   ve seçilen yerin koordinatıyla bulunur. Günün içinde iki doğuş varsa ilki alınır; tam gece
   yarısındaki doğuş başlayan günündür.
5. **Gün doğumu olmayan gün.** O takvim gününde Güneş doğmuyorsa (kutup gecesi ya da gündüzü)
   olgular 06:00 yerel ortalama saatte okunur: günün ortasının düştüğü yerel ortalama tarihin
   06:00'sı. Hücre `*` işareti ve "gün doğumu yok" sözü taşır. Kural gün panchangı panelinin
   kuralıdır: o panel aynı günü gösterir.
6. **Olayın günü.** Olay, anını içeren takvim günündedir; tam gece yarısındaki olay başlayan
   günündür. Günün olayları zaman sırasındadır; saatler olayın kendi anındaki ofsetle yazılır.
7. **Olay türleri.** Takvim yedi türü gösterebilir: burç girişi, nakshatra girişi, R
   başlangıcı, R bitişi, yeni ay, dolunay ve doğum boylamına geliş. Kavuşum ve doğum Ay'ından
   ev değişimi takvimde yoktur (olay araması panelindedir). Burç rasi çerçevesindedir (zodyak
   kipini izler); nakshatra siderealdir.
8. **Ay ayrı bir seçimdir.** Ay kapalıyken aranan grahalar Ay dışındaki sekiz grahadır; Ay
   açılınca Ay'ın burç ve nakshatra girişleri ile doğum boylamlarına gelişleri de aranır. Yeni
   ay ve dolunay bu seçime bağlı değildir.
9. **Doğum boylamına geliş** açık haritayı ister: geçiş grahasının, dokuz doğum grahasından
   birinin sidereal boylamına gelişidir (kendi boylamına dönüşü dahil). Tür seçili değilse
   doğum verisi komuta gönderilmez. Doğum saati bilinmiyorsa doğum boylamları öğlen içindir ve
   panel bunu söyler.
10. **Sınırlar.** Bir ay en çok 300 olay taşır; fazlası kesilir ve panel bunu söyler. Bir hücre
    en çok üç olay yazar ve gerisini sayar; gün ayrıntısı günün bütün olaylarını listeler.
11. **Dilimler.** Ay en çok yedi günlük dilimlerle hesaplanır; dilim sınırı olay kaybettirmez
    ve çoğaltmaz (sınırdaki değişim içinde bulunduğu dilime aittir).
12. **Yıl aralığı** 1–2999'dur. Paketlenen ephemeris dosyalarının dışındaki aylar (1800'den
    önce, 2399'dan sonra) düşük hassasiyetle hesaplanır ve panel uyarıyı taşır.
13. **Hafta başlangıcı** arayüz diline bağlıdır: Türkçe ve Fransızcada Pazartesi, İngilizcede
    Pazar. Günün sütununu Rust'ın verdiği hafta günü belirler.
14. **"Bugün"** seçilen yerin saat dilimindeki bugündür ve Rust'ta okunur.

## El örnekleri (Rust birim testleri)

Sahte gökyüzü (`crates/jyotish-core/src/fake.rs`): sidereal boylamlar J2000'de (JD 2451545,0)
Güneş 346° (0,98°/gün), Ay 71° (13,2°/gün), Mars 176° (0,6), Merkür 356° (1,2), Jüpiter 226°
(−0,1), Venüs 16° (1,1), Satürn 276° (0,05); gün doğumu her gün JD'nin 0,67 kesrindedir. Test
günleri 2000-01-01 00:00 UT'de (JD 2451544,5) başlayan ardışık UT günleridir; gün sırası 0'dan
başlar.

| Test | Sınadığı kural |
|---|---|
| `grid.rs › the_facts_of_a_day_are_read_at_the_sunrise_inside_it` (üç gün: tithi 7, 8, 9; nakshatra 5, 6, 7) | 2, 3, 4 |
| `grid.rs › the_waning_half_starts_after_the_fifteenth_tithi` | 3 |
| `grid.rs › a_sunrise_belongs_to_the_day_that_contains_it_and_midnight_opens_the_day` | 1, 4 |
| `grid.rs › a_day_without_a_sunrise_is_read_at_six_local_mean_time` (boylam 30° D: 04:00 UT) | 5 |
| `grid.rs › spans_are_one_to_thirty_one_gapless_days_of_plausible_length` | 1 |
| `grid.rs › a_month_is_cut_into_slices_of_at_most_seven_days` | 11 |
| `events.rs › the_filter_takes_calendar_kinds_only_and_the_moon_is_a_separate_choice` | 7, 8, 9 |
| `events.rs › an_event_belongs_to_the_day_that_contains_its_moment` | 6, 10 |
| `events.rs › sign_ingresses_and_lunations_land_on_their_days` (yedi olay: gün 3, 7, 8, 13, 14, 23, 28) | 6, 7 |
| `events.rs › the_moon_adds_its_own_ingresses_only_when_asked` (Ay: 13 burç, 31 nakshatra girişi) | 8 |
| `events.rs › the_sliced_month_finds_exactly_the_events_of_one_search` (ikinci yazım) | 9, 11 |
| `events.rs › the_event_limit_cuts_the_list_and_says_so` | 10 |
| `events.rs › bad_input_is_named_by_field_and_low_precision_is_reported` | 12 |
| `app/src-tauri/tests/calendar.rs` (gerçek motor: Delhi Kasım 2026 gün panchangı ve olay aramasıyla çapraz denetim; Berlin yaz saati günleri; Tromsø kutup gecesi ve gece yarısı güneşi; Samoa'nın atlanan günü; yıl aralığının uçları) | 1–12, 14 |
| `app/ui/calendar/model.test.ts`, `CalendarPanel.test.tsx` | 10, 13 |

## Sahibin kararını bekleyen noktalar

Aşağıdaki her satır tek seçimle uygulanmıştır (Y5); ikinci seçim ayar değildir. Sahibi kararını
alttaki tabloya yazar. Değişiklik ayrı bir commit'tir; gün tanımı değişirse yöntem kimliği
`calendar.month_v2` olur.

| # | Nokta | Uygulanan seçim | Öteki seçim |
|---|---|---|---|
| 1 | Hücrenin gösterdiği gün | Takvim günü (gece yarısından gece yarısına); olgular o günün gün doğumunda | Panchang günü (gün doğumundan gün doğumuna): olaylar da o güne yazılır, gece yarısından sonraki olay önceki günün hücresine düşer |
| 2 | Hücredeki tithi | Gün doğumundaki tithi (tek değer) | Günün bütün tithileri bitiş saatleriyle; gün içinde biten tithiyi de yazan düzen |
| 3 | Gün doğumu olmayan gün | 06:00 yerel ortalama saat, `*` işaretiyle | Hücreyi boş bırakmak; yerel öğleyi almak |
| 4 | Varsayılan olay türleri | Burç ve nakshatra girişleri, R başlangıcı ve bitişi, yeni ay, dolunay; Ay'ın kendi olayları ve doğum boylamına gelişler kapalı | Yalnızca burç girişleri ve Ay evreleri; ya da Ay'ın burç girişleri açık |
| 5 | Takvimde olmayan türler | Kavuşum ve doğum Ay'ından ev değişimi yok | İkisini de süzgece eklemek |
| 6 | Varsayılan yer | Bulunulan yer (gün panchangı gibi) | Açık haritanın doğum yeri (olay araması gibi) |
| 7 | Hafta başlangıcı | Türkçe ve Fransızcada Pazartesi, İngilizcede Pazar | Her dilde Pazartesi; ya da ayrı bir tercih |
| 8 | Hücredeki olay sayısı | En çok üç, gerisi sayılır | Hepsini yazan, satırı uzatan düzen |
| 9 | Panelin Türkçe adı | "Transit takvimi" (arayüzdeki "Transit tablosu" ve "transit tarihi" ile aynı sözcük) | "Geçiş takvimi" (yol haritasındaki ad) |

| Karar veren | Tarih | Sonuç |
|---|---|---|
| — | — | Bekliyor: sahibine devredildi (Plan 29 Task 8) |

## Denetim maddeleri

**[otomatik]** maddelerin karşılığı testlerde vardır; yürüten ajan testin adını göstererek
"geçti" sayar. **[insan]** maddeler yalnızca gerçek uygulamaya ya da basılı bir takvime bakan
biri tarafından doğrulanabilir ve sahibine devredilir (Y3: sonraki planı engellemez).

- [x] **[otomatik]** Hücrenin tithisi ve nakshatrası, o gün doğumuyla başlayan panchang gününün ilk değeridir (otuz gün, gerçek motor). (`tests/calendar.rs`; `grid.rs › the_facts_of_a_day_…`)
- [x] **[otomatik]** Takvimin olayları aynı ay için olay aramasının bulduklarıdır; dilimleme olay kaybettirmez ve çoğaltmaz. (`tests/calendar.rs`; `events.rs › the_sliced_month_…`)
- [x] **[otomatik]** 2026 Kasım'ında yeni ay 9'unda, dolunay 24'ündedir; Güneş ayda bir kez burç değiştirir. (`tests/calendar.rs`)
- [x] **[otomatik]** Yaz saati günleri 23 ve 25 saattir; kutup gecesinde ve gece yarısı güneşinde olgular 06:00 yerel ortalama saatte okunur ve gün panchangıyla aynı günü gösterir; takvimden atlanan gün ızgarada yoktur. (`tests/calendar.rs`)
- [x] **[otomatik]** Girdi hataları yalnızca alan adını taşır; doğum verisi yalnızca doğum türü seçiliyken gönderilir; bir ayın ortanca süresi bütçenin içindedir. (`tests/calendar.rs`; `calendar_commands.rs` testleri; `CalendarPanel.test.tsx`)
- [x] **[otomatik]** Panel: ay ızgarası, bugünün işareti, hücrede en çok üç olay, ay seçimi (önceki, sonraki, ay, yıl, "Bu ay"), yer seçimi, gün doğumu olmayan gün, düşük hassasiyet, hata ve yeniden deneme. (`CalendarPanel.test.tsx`)
- [x] **[otomatik]** Klavye: ızgara tek sekme durağıdır; oklar, Home, End, Page Up ve Page Down; Enter ve boşluk günü seçer; odak gövdeye düşmez. (`CalendarPanel.test.tsx`)
- [x] **[otomatik]** Gün ayrıntısı günün bütün olaylarını listeler; olayın saati transit tarihini değiştirir; "Gün panchangını aç" paneli ekler. (`CalendarPanel.test.tsx`)
- [x] **[otomatik]** Süzgeç panelin ayarıdır; hafta İngilizcede Pazar, Türkçe ve Fransızcada Pazartesi başlar; üç dilde ay ve gün adları vardır. (`CalendarPanel.test.tsx`, `model.test.ts`, `i18n.test.ts`)
- [x] **[otomatik]** Eski isteğin geç gelen yanıtı gösterilmez; yeni ay yüklenirken eski ızgara durur. (`useCalendar.test.tsx`)
- [ ] **[kontrolör]** Uçtan uca akışlar ve görsel temeller. (`app/e2e/calendar.desk.spec.ts`: üç akış; `app/e2e/calendar.visual.spec.ts`: üç temel)
- [ ] **[insan]** Gün tanımı (1. ve 2. satırlar): bir ayın hücreleri, sahibin kullandığı basılı bir panchang takvimiyle yan yana konur; gün doğumundaki tithi ve nakshatra uyuşur. Farklı bir gün tanımı isteniyorsa karar tablosuna yazılır.
- [ ] **[insan]** Varsayılan olay türleri (4. ve 5. satırlar) bir danışan görüşmesinde yeterlidir; hücreler ne boş ne de aşırı doludur.
- [ ] **[insan]** Izgara ekranda okunur: açık ve koyu temada, üç dilde; dar panelde yatay kayar; bugün ve seçili gün ayırt edilir (renk dışında sözle ve çerçeveyle); üç olaylı hücre taşmaz.
- [ ] **[insan]** Klavyeyle bir ay boyunca gezilir; Page Down ile ay değişince odak aynı günde durur; VoiceOver hücreyi gün, tithi, nakshatra ve olaylarıyla, seçili günü "seçili" diye okur.
- [ ] **[insan]** "Gün panchangını aç" doğru günü açar (gün panchangı panelinin tarih bağı "Transit tarihi" iken); gece yarısından sonra "Bu ay" bugünü yeni güne taşır.
- [ ] **[insan]** Kutup dairesinin ötesinde bir yerde kış ayı: `*` işaretli günler ve dipnot okunur.
- [ ] **[insan]** Bütün denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, doğum anı, yer, koordinat ya da takvim içeriği görünmez.

## Gözden geçirme kaydı

Gözden geçiren (motoru yazan dışında biri) `grid.rs` ve `events.rs` testlerindeki el örneklerini
yalnızca yukarıdaki sahte gökyüzünden ve okuma kurallarından başlayarak, motoru çalıştırmadan
yeniden hesaplar: üç günün gün doğumundaki tithisi ve nakshatrası, gün doğumu olmayan günün anı,
bir aydaki yedi olayın günü ve Ay'ın giriş sayıları. Sonucunu testteki beklenen değerlerle
karşılaştırır ve aşağıya adını, tarihini ve bulduğu farkları yazar.

| Gözden geçiren | Tarih | Sonuç |
|---|---|---|
| — | — | Bekliyor: Plan 29 Task 8 Step 4 |
````

````diff
diff --git a/tests/golden/modules/README.md b/tests/golden/modules/README.md
index ebf72ad..3bf7506 100644
--- a/tests/golden/modules/README.md
+++ b/tests/golden/modules/README.md
@@ -122,4 +122,11 @@ Gerçek motorla çapraz denetim `app/src-tauri/tests/natal_extras.rs` ve
 sahibin onayını bekleyen noktalar, denetim maddeleri ve gözden geçirme kaydı
 `natal_extras/REVIEW.md` ve `bhava/REVIEW.md`'dedir.
 
+Transit takvimi (yol haritası §4.11, Plan 29) de bu dizinde JSON örneği taşımaz: `calendar/`
+klasöründe yalnızca `REVIEW.md` vardır ve hiçbir test bu klasörü yüklemez. El örnekleri modülün
+birim testleridir (`crates/jyotish-core/src/calendar/` içindeki `mod tests`, sahte gökyüzüyle);
+gerçek motorla çapraz denetim (gün panchangı ve olay aramasıyla) `app/src-tauri/tests/calendar.rs`
+dosyasındadır. Okuma kuralları (gün tanımı, gün doğumu olmayan gün, olayın günü), sahibin kararını
+bekleyen noktalar, denetim maddeleri ve gözden geçirme kaydı `calendar/REVIEW.md`'dedir.
+
 Sonraki alt projeler kendi klasörünü ve `expected` yapısını ekler.
````

- [ ] **Step 2: Rust kapıları**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
BASE=9727c55
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -1
cargo test --workspace 2>&1 | grep -E "^test result|Running"
cargo test --workspace 2>&1 | grep -E "^test result" | grep -vc "ok\."
cargo test -p jyotish-desk --test calendar -- --nocapture 2>&1 | grep -E "month_calendar median"
cargo deny check 2>&1 | tail -1
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p jyotish-core -p atlas -p store -p jyotish-desk --all-targets 2>&1 | tail -1
git diff $BASE..HEAD --stat -- deny.toml Cargo.toml Cargo.lock app/package.json app/pnpm-lock.yaml app/vite.config.ts app/playwright.config.ts app/src-tauri/capabilities app/src-tauri/tauri.conf.json app/src-tauri/Cargo.toml crates/jyotish-core/Cargo.toml crates/store crates/atlas crates/ephemeris crates/jyotish-core/src/search crates/jyotish-core/src/timeline.rs crates/jyotish-core/src/panchang.rs crates/jyotish-core/src/muhurta crates/jyotish-core/src/error.rs crates/jyotish-core/src/chart.rs crates/jyotish-core/tests app/src-tauri/src/commands.rs app/src-tauri/src/search_commands.rs app/src-tauri/src/muhurta_commands.rs app/src-tauri/src/module_commands.rs app/src-tauri/tests/modules.rs
git diff $BASE..HEAD --stat -- crates/jyotish-core/src/lib.rs app/src-tauri/src/lib.rs | tail -1
ls crates/jyotish-core/proptest-regressions 2>/dev/null
```
Expected: fmt çıktısız; clippy `Finished`; testler: `jyotish-core` lib `479 passed`, `avastha_golden` `5 passed`, golden `8 passed`, `modules_golden` `17 passed`; `atlas` `build` 6, `moment_fixed_lmt` 11, `moment_iana` 13, `search` 8, `shipped` 3; `store` lib `122 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `61 passed`, `avastha` `1 passed`, `bhava` `1 passed`, `bindings` `1 passed`, `calendar` `1 passed`, `compute_chart` `1 passed`, `modules` `1 passed`, `natal_extras` `1 passed`, `open_client` `1 passed`, `ui_fixture` `2 passed`; sayaç `0`; ölçüm satırı bütçenin içinde (D10; aşan değer testi zaten düşürür); deny `advisories ok, bans ok, licenses ok, sources ok`; MSRV `Finished` (1.93 araç zinciri kurulu değilse bu satır atlanır ve notlara yazılır); ilk `git diff --stat` çıktısız (bağımlılıklar, yapılandırmalar, veritabanı, olay araması, zaman çizelgesi, panchang, muhurta, hata türleri, ortak komut ve test dosyaları değişmedi); iki `lib.rs` birlikte `2 files changed, 3 insertions(+)`; `ls` çıktısız. `BASE` Task 1 Step 1'de başka bir commit okunduysa o yazılır ve taban sayaçları o ölçümdendir.

- [ ] **Step 3: Arayüz kapıları**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | grep "✓ built"
node --test "scripts/release/*.test.mjs" 2>&1 | grep -E "ℹ (pass|fail)"
git status --short
```
Expected: `Test Files  101 passed (101)`, `Tests  1007 passed (1007)`; `✓ built`; betik testleri `pass 14`, `fail 0`; `git status --short` yalnızca Step 1'in iki dosyasını gösterir. Uçtan uca ve görsel testler burada koşulmaz (Step 8).

- [ ] **Step 4: Bağımsız yeniden türetme**

Yol haritası §8: "el örneğini yazanın kuralı yanlış anlaması motoru da yanlış doğrular; ayrı gözden geçirmede yeniden türetme zorunludur". Yürüten ajan örnekleri yazan oturum değildir ve bu denetimi yapabilir. Kural: **yalnızca** `tests/golden/modules/calendar/REVIEW.md`'nin "Okuma kuralları" bölümü ile "El örnekleri" bölümündeki sahte gökyüzü okunur; Rust kaynağının uygulama kısmına bakılmaz, motor çalıştırılmaz. Beklenen değerler karşılaştırma anında `grid.rs` ve `events.rs`'in test bloklarından okunur.

1. Depo **dışında** (ör. `/tmp` altında; `git add` edilmez) atılacak bir betik ya da elle hesap: sahte gökyüzünün boylamları `boylam(t) = başlangıç + hız × (JD − 2451545,0)` ile yürütülür.
2. Hesaplananlar: (a) gün 0, 1 ve 2'nin gün doğumundaki (JD kesri 0,67) uzanımı, tithisi ve Ay nakshatrası; (b) boylam 30° D için gün doğumu olmayan günün okuma anı (UT saati); (c) 31 günlük ayda Ay dışındaki grahaların burç girişlerinin, dolunayın ve yeni ayın gün sırası; (d) aynı ayda Ay'ın burç ve nakshatra girişlerinin sayısı; (e) Güneş'in 0° boylamına geldiği gün.
3. Sonuç testlerdeki beklenen değerlerle karşılaştırılır: (a) tithi 7, 8, 9 ve nakshatra 5, 6, 7; (b) 04:00 UT; (c) gün 3 (Merkür), 7 (Mars), 8 (dolunay), 13 (Venüs), 14 (Güneş), 23 (yeni ay), 28 (Merkür); (d) 13 ve 31; (e) gün 14.
4. Hepsi uyuşuyorsa REVIEW.md'nin "Gözden geçirme kaydı" tablosundaki son satır (`| — | — | Bekliyor: Plan 29 Task 8 Step 4 |`) şu satırla değiştirilir (tarih koşunun günüdür):

   ````markdown
   | Plan 29 Task 8'i yürüten ajan (örnekleri yazan oturum dışında) | <tarih> | Fark yok: üç günün tithisi ve nakshatrası, gün doğumu olmayan günün anı, yedi olayın günü, Ay'ın giriş sayıları ve Güneş'in dönüş günü yalnızca sahte gökyüzünden ve bu dosyanın okuma kurallarından, motor çalıştırılmadan yeniden hesaplandı. Sınır: yürüten ajan aynı planın kodunu da yazdı; bu kayıt aritmetiği ve kuralların uygulanışını doğrular, gün tanımını ve varsayılan seçimleri (karar tablosu) değil. |
   ````

5. Bir fark varsa **hiçbir beklenen değer ve hiçbir kural değiştirilmez**: fark (test, gün, iki değer ve kuralın hangi cümlesinden çıktığı) ilerleme notlarına yazılır, tablodaki satır `Fark bulundu: <kısa özet>; sahibine devredildi` diye doldurulur ve görev kapıların sonucuyla birlikte sahibine bildirilir.

```bash
grep -c "Bekliyor: Plan 29 Task 8 Step 4" tests/golden/modules/calendar/REVIEW.md
grep -inE 'k(a|ā|aa)la|\bcala\b' tests/golden/modules/calendar/REVIEW.md tests/golden/modules/README.md
git status --short
```
Expected: sayaç `0` (satır dolduruldu); tarama çıktısız; `git status --short` yalnızca iki belge dosyasını gösterir (betik depoda değildir).

- [ ] **Step 5: Commit**

```bash
git add tests/golden/modules/calendar/REVIEW.md tests/golden/modules/README.md
git diff --cached --stat | tail -1
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "docs(golden): transit calendar reading rules, points awaiting the owner's decision and the review record" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Expected: `2 files changed, 147 insertions(+)`; tarama çıktısız; `git status --short` boş.

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
grep -rnE '#\[serde\(default' crates/jyotish-core/src/calendar app/src-tauri/src/calendar_commands.rs
grep -rnE 'XMLHttpRequest|WebSocket|\bfetch\(' app/ui/calendar --include='*.ts' --include='*.tsx' | grep -v '\.test\.'
grep -rnE 'localStorage|sessionStorage|new Date\(|Date\.now|Intl\.' app/ui/calendar --include='*.ts' --include='*.tsx' | grep -v '\.test\.'
grep -cE 'calendar_commands::month_calendar,' app/src-tauri/src/lib.rs
grep -cE '"invalid_calendar_input"' app/src-tauri/src/calendar_commands.rs
grep -rn 'calendar' crates/store/src | head -1
git diff $BASE..HEAD --stat -- app/ui/chart app/ui/print app/ui/shortcuts.ts app/e2e/visual.spec.ts-snapshots app/e2e/desk.spec.ts app/e2e/visual.spec.ts app/ui/test/fixture.ts app/ui/panels/panels.test.tsx app/ui/panels/panels.css app/ui/panels/labels.ts app/ui/i18n/i18n.test.ts
git ls-files | grep -c '^\.superpowers/'
git tag --points-at HEAD
```
Expected (`BASE` Task 1 Step 1'de okunan commit'tir; `9727c55` değilse o yazılır): `trailers:` satırında ilk iki sayı eşit (sekiz ve sekiz; inceleme düzeltmesi commit'i varsa ikisi birlikte büyür) ve `other: 0` (başka bir imza satırı yok); `grep -ciE` satırı `0`; komut sayacı `1`; hata kodu sayacı `2` (üretim satırı ve birim testi); `.superpowers` sayacı `0`. Bunların dışındaki her komut çıktısızdır (çıktısız `grep` 1 ile çıkar; beklenen budur): yasaklı ad yok (üretilen fikstürde de), `app/`, `crates/` ve `tests/` dışında dosya yok, arayüzde `console` ve Rust kaynağında log çağrısı yok (girdi ve takvim loglanmaz: C1; `tests/calendar.rs`'teki ölçüm satırı test dosyasıdır ve bu taramanın dışındadır), yasak çağrı yok, yeni stilde hex ya da `rgb(` yok (C9), IPC tiplerinde `serde(default)` yok, panelde ağ, tarayıcı deposu, saat ve `Intl` çağrısı yok (C4, D14), veritabanı katmanında takvim yok (Y6), harita çizimi, baskı, kısayollar, var olan görsel temeller ve bilerek dokunulmayan ortak dosyalar değişmedi, etiket yok.

- [ ] **Step 7: Elle denetim kapısı**

Yürüten ajan gerçek uygulamayı **çalıştırmaz** (Global Constraints). Her madde ilerleme notlarına "geçti", "devredildi" ya da bulgu olarak yazılır; bulgu varsa düzeltme commit'i ve bu görev baştan. **[otomatik]** maddelerin karşılığı testlerde vardır ve yürüten ajan bunları testin adını göstererek "geçti" sayar; **[kontrolör]** maddeler Step 8'de kontrolörce koşulur; **[insan]** maddeler yalnızca gerçek uygulamaya ya da basılı bir takvime bakan bir insan tarafından doğrulanabilir ve sahibine devredilir (bu devir sonraki planı engellemez: Y3; hiçbir şey yayımlanmaz).

- [ ] **[otomatik]** Hücrenin tithisi ve Ay nakshatrası o takvim gününün içindeki gün doğumunda okunur; üç günün el örneği ve Krishna yarısı. (`grid.rs › the_facts_of_a_day_…`, `› the_waning_half_…`)
- [ ] **[otomatik]** Gün sınırları: gece yarısındaki doğuş başlayan günündür, iki doğuşlu günde ilki alınır; 23 ve 25 saatlik günler geçerlidir; günler boşluksuz ardışıktır. (`grid.rs › a_sunrise_belongs_…`, `› spans_are_…`)
- [ ] **[otomatik]** Gün doğumu olmayan gün 06:00 yerel ortalama saatte okunur ve an günün içindedir. (`grid.rs › a_day_without_a_sunrise_…`)
- [ ] **[otomatik]** Süzgeç yalnızca yedi takvim türünü alır; Ay ayrı bir seçimdir; açık harita yoksa doğum türü sessizce düşer. (`events.rs › the_filter_takes_…`, `› the_moon_adds_…`)
- [ ] **[otomatik]** Olay, anını içeren gündedir; yedi olayın günü elle türetilmiştir; sınır listeyi keser ve söyler. (`events.rs › an_event_belongs_…`, `› sign_ingresses_and_lunations_…`, `› the_event_limit_…`)
- [ ] **[otomatik]** Dilimlenmiş ay, tek aramanın bulduğu olayları tam olarak bulur (ikinci yazım). (`events.rs › the_sliced_month_…`; `tests/calendar.rs` bölüm 3)
- [ ] **[otomatik]** Gerçek motorla: otuz günün tithisi ve nakshatrası gün panchangının ilk değeridir; 2026 Kasım'ında yeni ay 9'unda, dolunay 24'ündedir; Güneş ayda bir kez burç değiştirir. (`tests/calendar.rs` bölüm 1–2)
- [ ] **[otomatik]** Yaz saati günleri 23 ve 25 saattir; Tromsø'da kutup gecesi ve gece yarısı güneşi; Samoa'nın atlanan günü; yıl aralığının uçları ve düşük hassasiyet; girdi hataları yalnızca alan adını taşır; bütçe. (`tests/calendar.rs` bölüm 4–9; `calendar_commands.rs` testleri)
- [ ] **[otomatik]** Panel: bir istek (ay verilmez, yerin adı ve doğum verisi gitmez); otuz hücre, altı satır, bugün işareti; hücrede tithi, nakshatra ve en çok üç olay; kısaltmaların tam adı ekran okuyucuya. (`CalendarPanel.test.tsx › asks for this month…`, `› a cell lists at most three…`)
- [ ] **[otomatik]** Ay seçimi Rust'ın verdiği aylarla yürür; ay ve yıl alanları; "Bu ay" yeniden okur; aralığın ucunda düğme bir şey istemez; yeni ay yüklenirken eski ızgara durur; eski yanıt düşer. (`CalendarPanel.test.tsx` ay seçimi testleri; `useCalendar.test.tsx`)
- [ ] **[otomatik]** Yer yokken panel yer ister ve hesap yapılmaz; gün doğumu olmayan günler işaretlidir ve açıklanır; düşük hassasiyet söylenir; hata yeniden denenir ve odak gövdeye düşmez. (`CalendarPanel.test.tsx`)
- [ ] **[otomatik]** Klavye: tek sekme durağı; oklar, Home, End ve Ctrl biçimleri; Page Up ve Page Down odağı aynı günde tutar; ızgara kalkarsa odak panele taşınır. (`CalendarPanel.test.tsx › the grid is one tab stop…`, `› Page Down and Page Up…`, `› at the end of the range…`)
- [ ] **[otomatik]** Gün ayrıntısı: seçim yokken bugün; Enter, boşluk ve tıklama seçer; olayın saati transit tarihini değiştirir; "Gün panchangını aç" paneli ekler ve var olanı kaldırmaz. (`CalendarPanel.test.tsx › the detail shows today…`, `› an event's time moves…`)
- [ ] **[otomatik]** Süzgeç panelin ayarıdır; doğum verisi yalnızca tür seçilince gider; türsüz takvim olguları gösterir; saat bilinmeyen doğum notu; kesilen liste bildirimi. (`CalendarPanel.test.tsx` süzgeç testleri)
- [ ] **[otomatik]** Hafta İngilizcede Pazar, Türkçe ve Fransızcada Pazartesi başlar; ay ve gün adları üç dilde vardır; Fransızca metinlerde düz kesme işareti yoktur; üç dilin anahtar kümeleri ve yer tutucuları eşittir. (`CalendarPanel.test.tsx › the week starts…`, `› the panel is in the palette catalog…`; `model.test.ts`; `i18n.test.ts`)
- [ ] **[otomatik]** Bağımlılıklar, veritabanı, olay araması, panchang ve var olan görsel temeller değişmedi. (Step 2 ve Step 6'nın `git diff --stat` satırları)
- [ ] **[otomatik]** El örnekleri motor çalıştırılmadan yeniden türetildi ve uyuştu. (Step 4; REVIEW.md "Gözden geçirme kaydı")
- [ ] **[kontrolör]** Uçtan uca akışlar (üç akış) ve bütün e2e paketi geçer; üç görsel temel üretilir, gözle denetlenir ve commit edilir. (Step 8)
- [ ] **[insan]** **Gün tanımı (asıl karar).** `tests/golden/modules/calendar/REVIEW.md`'nin karar tablosundaki dokuz nokta gözden geçirilir; özellikle 1. ve 2. satır: bir ayın hücreleri sahibin kullandığı basılı bir panchang takvimiyle yan yana konur. Sonuç tabloya yazılır.
- [ ] **[insan]** Gerçek uygulamada, **yalnızca deneme danışanlarıyla**: palette "Transit takvimi panelini aç/kapat" paneli açar; bulunulan yer seçilince bu ay görünür; bugün işaretlidir; önceki ve sonraki ay, ay listesi, yıl alanı ve "Bu ay" çalışır.
- [ ] **[insan]** Bir günün hücresi gün panchangı paneliyle tutarlıdır: "Gün panchangını aç" o günü açar ve panelin ilk tithisi ile nakshatrası hücredekilerdir; olayın saati transit tablosunu o ana götürür.
- [ ] **[insan]** Varsayılan olay türleri bir görüşmede yeterlidir; "Doğum boylamına geliş" ve "Ay'ın olayları" açılınca hücreler okunur durur ("+N olay" sayacı; gün ayrıntısı hepsini listeler).
- [ ] **[insan]** Izgara açık ve koyu temada, Türkçe, İngilizce ve Fransızca dilde okunur; dar panelde yatay kayar; bugün ve seçili gün renk dışında da ayırt edilir.
- [ ] **[insan]** Klavye ve VoiceOver: ızgaraya sekmeyle bir kez girilir; oklar günler arasında gezer; Page Down ile ay değişince odak aynı günde durur; hücre gün, tithi, nakshatra ve olaylarıyla, seçili gün "seçili" diye okunur; gün ayrıntısının başlığı seçimde duyurulur.
- [ ] **[insan]** Kutup dairesinin ötesinde bir yerde (özel yer) kış ayı: `*` işaretli günler ve dipnot okunur; yaz saati geçişi olan bir yerde geçiş günü hücresi doğru olayları taşır.
- [ ] **[insan]** Doğum saati bilinmeyen bir deneme haritasında doğum türü seçilince not görünür.
- [ ] **[insan]** Bütün denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, doğum anı, yer, koordinat ya da takvim içeriği görünmez.

Kapı geçtiyse ilerleme notlarına `Plan 29 Task 8 elle denetim: otomatik maddeler geçti, kontrolör adımları ve insan maddeleri devredildi (<tarih>)` yazılır. Bu adım commit üretmez.

- [ ] **Step 8: [kontrolör] Uçtan uca akışlar ve görsel temeller**

**Yürüten ajan bu adımı koşmaz**; kontrolör, yürüten ajanın dalında koşar. Sunucu özel bir portta açılır (1421 başka bir çalışma ağacının sunucusu olabilir): depodaki yapılandırma değişmez, geçici bir kopyası kullanılır ve silinir.

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
lsof -i :14329
sed 's/1421/14329/g; s/reuseExistingServer: !process.env.CI/reuseExistingServer: false/' app/playwright.config.ts > app/playwright.p29.config.ts
(cd app && pnpm exec playwright test -c playwright.p29.config.ts --project=e2e calendar 2>&1 | tail -1)
(cd app && pnpm exec playwright test -c playwright.p29.config.ts --project=e2e 2>&1 | tail -1)
(cd app && pnpm exec playwright test -c playwright.p29.config.ts --project=visual calendar --update-snapshots 2>&1 | tail -1)
(cd app && pnpm exec playwright test -c playwright.p29.config.ts --project=visual 2>&1 | tail -1)
rm -f app/playwright.p29.config.ts
rm -rf app/test-results
ls app/e2e/calendar.visual.spec.ts-snapshots
git status --short
```
Expected: `lsof` çıktısız (çıktı varsa başka bir port seçilir); takvim akışları `3 passed`; bütün e2e `36 passed`; temel üretimi `3 passed`; bütün görsel paket `18 passed` (var olan on beş temel değişmedi, üç yenisi az önce üretilenle aynı); `ls` üç dosya: `calendar-dark-visual-darwin.png`, `calendar-full-light-visual-darwin.png`, `calendar-light-visual-darwin.png`; `git status --short` yalnızca bu klasörü gösterir. Kontrolör üç PNG'ye bakar: ızgara yedi sütun ve altı satırdır, 1 Kasım son sütundadır, 25 Kasım çerçevelidir ve "bugün" yazar, hücrelerde tithi, nakshatra ve saatli olaylar okunur; dolu ızgarada 24 Kasım seçilidir ve "+2 olay" taşır. Sonra:

```bash
git add app/e2e/calendar.visual.spec.ts-snapshots
git commit -m "test(visual): transit calendar baselines" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
git status --short
```
Bir akış düşerse kontrolör düzeltmeyi yürüten ajana geri verir (ayrı düzeltme commit'i); temeller akışlar geçtikten sonra üretilir.

---

## Başarı ölçütleri

- Seçilen yerin bir ayı tek komutla, saf bir çekirdekten ve açık haritanın hesap ayarıyla hesaplanır; sonuç saklanmaz ve `Chart`'a eklenmez; yeni astronomi ve yeni bağımlılık yoktur.
- Her hücre bir takvim günüdür; tithisi ve Ay nakshatrası o günün gün doğumundaki değerdir ve gün panchangıyla gerçek motorda otuz gün boyunca uyuşur.
- Takvimin olayları aynı ay için olay aramasının bulduklarıdır; dilimleme olay kaybettirmez ve çoğaltmaz; ayın ortanca süresi bütçenin içindedir.
- Yaz saati günleri, kutup dairesi, takvimden atlanan gün ve yıl aralığının uçları testlidir; girdi hataları yalnızca alan adı taşır.
- Izgara klavyeyle ve ekran okuyucuyla kullanılır; odak hiçbir durumda gövdeye düşmez; gün ayrıntısı günün bütün olaylarını listeler ve gün panchangına geçer.
- Doğum verisi yalnızca doğum türü seçiliyken komuta gider; süzgeç panelin ayarıdır.
- Plan 26'nın hiçbir adı kullanılmaz; paralel planlarla paylaşılan her dosyadaki değişiklik ekleme türündedir ve "Paylaşılan dosyalar" tablosundadır.
- Sayaçlar: `jyotish-core` lib 479, `jyotish-desk` lib 61 ve `calendar` 1, Vitest 101 dosya / 1007 test; kontrolörde e2e 36 ve görsel 18; öteki bütün sayaçlar tabandaki gibidir.

## Bu planın kapsamadıkları

- Panchang gününü hücre yapan ikinci gün tanımı ve bunun ayarı (Y5: tek seçenekli okuma ayar olmaz); gün içinde biten tithilerin, nitya yoganın ve karananın hücrede yazılması (gün panchangı panelindedir).
- Nakshatra lordu, Sankranti işareti ve giriş anının panchangı (Plan 26 birleştikten sonra ayrı iş: D2); tutulmalar (Plan 30; `CALENDAR_KINDS`'e eklenir).
- Kavuşumlar ve doğum Ay'ından ev değişimi (olay araması panelindedir); graha bazında süzgeç (takvimde yalnızca Ay ayrı seçilir).
- Hafta ve yıl görünümü; birden çok ayın yan yana gösterimi; gösterilen ayın saklanması.
- Olayın harita olarak açılması (Plan 30); takvimin dışa aktarımı (CSV, ICS) ve baskı düzeni (K10).
- Durdurma kaydı ve ilerleme göstergesi (D10: ölçülen süre gerektirmez).
- Panelin hazır düzenlere eklenmesi; "Gün panchangını aç" ile açılan panelin yer ve tarih bağı ayarının takvimle eşlenmesi (panel kendi ayarını kullanır).
- Tithi, nakshatra ya da olay için "uygun", "uygunsuz" işareti, puan ve açıklama metni; muhurta süzgeçlerinin takvimde gösterimi.
- Yorum metni, AI paneli; Windows ve Android.

## Spec'e geri bildirim (yol haritası farkları ve boşlukları)

Kararlar bölümünde işaretlenen maddeler; yol haritası bir sonraki düzenlemede bunlara göre güncellenmelidir.

1. **(D2) Plan 26 bağımlılığı.** §5 tablosu ve §6 Plan 29'u Plan 26'ya bağlar; takvim Plan 26'nın hiçbir adını kullanmaz. Tablodaki "Bağımlı olduğu" hücresi "Plan 20, 22" olmalı; nakshatra lordu ve Sankranti işareti "Plan 26'dan sonra eklenebilir" diye ayrı satıra yazılmalıdır.
2. **(D10) Dilimleme ve durdurma.** §8 takvim için dilimleme ve eski yanıtın düşürülmesini ister; ölçülen süre (ay başına 12–17 ms) ve seçilen biçim (yedi günlük dilimler, durdurma kaydı yok, eski yanıt arayüzde düşer; bütçe 150 ms) §4.11'e yazılmalıdır.
3. **(D25) Görev taslağı.** §5 Plan 29'un (7) maddesi ("i18n, boş ve hata durumları") panelle aynı görevdedir; (7) uçtan uca ve görsel akışların kodudur ve yürüten ajan bunları koşmaz.
4. **(D3) Gün tanımı.** §4.11 kararı açık bırakır: hücre takvim günüdür, olgular o günün içindeki gün doğumunda okunur, olay anını içeren gündedir. Gerekçesiyle yazılmalıdır.
5. **(D4, D5) Uç durumlar.** Yaz saati günleri (23 ve 25 saat), takvimden atlanan gün ve gün doğumu olmayan günün kuralı (06:00 yerel ortalama saat, `*` işareti) §4.11'e eklenmelidir.
6. **(D6, D7) Türler.** Takvimin yedi türü, varsayılan altı tür ve Ay'ın ayrı seçim olduğu (ölçümle: 13 ve 71 olay) yazılmalıdır.
7. **(D9) Sınırlar.** Ay başına 300 olay, hücre başına üç satır.
8. **(D12, D13) Yıl aralığı ve hata kodu.** 1–2999; `invalid_calendar_input`.
9. **(D15) Panelin adı.** Arayüzde "Transit takvimi"; yol haritasındaki "geçiş takvimi" belge dilidir. Ek A'ya not düşülebilir.
10. **Y4'ün kapsamı.** Tablo taşımayan modüllerin (Plan 25, 26, 29) REVIEW kaydının "sahibin kararını bekleyen noktalar" olduğu ve "doğrulanmamış tablo" notu taşımadığı §2'de tek cümleyle yazılmalıdır.

## Self-Review

1. **Kapsam:** yol haritası §4.11'in ve §5 Plan 29'un her maddesi "Kapsam" tablosunda bir göreve bağlıdır. Taslaktan bilerek ayrılan yerler Kararlar'da işaretlidir (D2, D10, D25) ve geri bildirim listesindedir. Sekiz görev vardır; son görev kapılar, REVIEW.md ve **[otomatik]**/**[kontrolör]**/**[insan]** maddeleridir.
2. **Yer tutucu taraması:** planda "TBD", "TODO", "sonra doldurulur", "uygun hata işleme ekle" ya da kodu olmayan kod adımı yoktur; her kod adımı tam dosya ya da uygulanabilir `diff` taşır. `<tarih>` ve `<kısa özet>` yürütme anında yazılan değerlerdir (Task 8 Step 4 ve Step 7); `BASE` Task 1 Step 1'de okunur.
3. **Tip ve ad tutarlılığı:** `DaySpan`, `DayFacts` Task 1'de; `CalendarFilter`, `CalendarSpec`, `CalendarDay`, `CalendarSlice` Task 2'de; `YearMonth`, `CalendarRequest`, `CalendarDayView`, `CalendarView` Task 3'te tanımlanır ve sonraki görevlerde aynı adlarla kullanılır. Komut adı Rust'ta `month_calendar`, arayüzde `monthCalendar`'dır; panel türü `calendar`, i18n bloğu `calendar`, fikstür alanı `calendar`, hata kodu `invalid_calendar_input`'tur. Arayüzün sabitleri (`CALENDAR_KINDS`, `DEFAULT_CALENDAR_KINDS`, `MIN_YEAR`, `MAX_YEAR`, `MAX_EVENTS`) fikstürdeki Rust değerleriyle testte karşılaştırılır.
4. **Dersler:** arayüz tarih hesabı yapmaz, ay adımı ve "bugün" Rust'tadır (C4); doğum verisi yalnızca gerektiğinde gider; hata yalnızca alan adı taşır; el örneği motorla uyuşmazsa kural yeniden okunur; odak gövdeye düşmez (hata ve ay değişimi testleri); bütçe `assert!` ile doğrulanır (D10); paylaşılan dosyalara yalnızca ekleme yapılır ve liste tamdır; üretilen dosyalar yalnızca iki komutla güncellenir (C10); gerçek uygulama çalıştırılmaz; yürüten ajan e2e koşmaz ve kontrolör özel portta koşar.
5. **Koşulmuş kod:** bu plandaki her kod bloğu, `9727c55`'in bir kopyasında sekiz ayrı commit olarak uygulanmış ağaçtan alınmıştır; sonra bu belgenin blokları ikinci bir temiz kopyaya betikle sırayla uygulanmış ve her görevin sonunda ağacın ilk kopyadaki commit ile birebir aynı olduğu doğrulanmıştır; üretilen iki dosyanın güncelliği o kopyada `bindings` ve `ui_fixture` testleriyle, komutun kendisi `tests/calendar.rs` ile doğrulanmıştır. İlk kopyada: Task 1–2'nin commit'lerinde `cargo test -p jyotish-core` ve clippy; Task 3'ün commit'inde `cargo test --workspace` (başarısız sonuç satırı 0), clippy, Vitest (98 / 976) ve `pnpm build`; Task 4–6'nın commit'lerinde Vitest (101 / 997, 1003, 1007) ve `pnpm build`; son ağaçta `cargo fmt --all --check`, clippy, `cargo test --workspace` (0), Vitest (101 / 1007), `pnpm build`, `cargo deny check`, 1.93 ile `cargo check` ve yayın betiği testleri (14) koşulmuş ve geçmiştir. Uçtan uca (36) ve görsel (18; üç yeni temel o kopyada üretilerek) paketler özel bir portta (14329) koşulmuş ve geçmiştir. "Testlerin düştüğünü gör" adımlarından arayüz adımları (Task 3 Step 6, Task 4 Step 2, Task 5 Step 2, Task 6 Step 2) ikinci kopyada koşulmuş ve beklenen sayılar oradan alınmıştır; Rust kırmızı adımlarının beklentisi derleme hatasının türüdür. Bu plan yazılırken bir kez clippy düştü (Task 1'de kullanılmayan `check_place`: `day_facts` yer denetimini yapar hâle getirildi) ve REVIEW metninde bir kez yasaklı dizi çıktı (D26'daki sözcük); ikisi de bu belgedeki kodda düzeltilmiştir.
6. **Yasaklı ad:** bu belge ve bütün kod blokları `grep -inE 'k(a|ā|aa)la|\bcala\b'` ile taranmıştır; çıktı boştur.
