# jyotish-desk — Alt proje 3: İleri modüller — Tasarım Belgesi

Tarih: 2026-11-26 · Durum: Tasarım kararları proje sahibinin devriyle verildi (Fable); sahibin gözden geçirmesine açık · İlgili belgeler: `2026-11-24-jyotish-desk-v1-design.md` (v1 spec), `2026-11-24-ui-design.md` (UI spec)

## 1. Amaç

v1 spec §2 tablosunun 3. satırı: Shadbala, yogalar, Jaimini, uyum, Muhurta, Prashna ve arama. Bu belge o satırı dokuz bağımsız alt projeye böler. Her alt proje tek bir uygulama planına karşılık gelir ve kendi başına çalışan, test edilebilir yazılım üretir.

Değişmeyen kurallar (v1 spec ve UI spec'ten taşınır):

- **Clean-room.** Klasik yöntemler özgün olarak uygulanır. Başka ürünlerin adı, arayüz düzeni, yorum metni ve çıktısı kopyalanmaz. Kaynak gösterimi geneldir (ör. "BPHS, graha gücü bölümü"); klasik metinden alıntı yapılmaz.
- **Yorum metni yok.** Modüller hesaplanmış olguları ve klasik etiketleri gösterir ("Gajakesari", "A7", "Vipat"). Bir yoganın, puanın ya da dönemin ne anlama geldiğini anlatan metin Alt proje 6'nın işidir.
- **Arayüz hesap yapmaz.** Her değer Rust'tan gelir. Sonradan gelecek AI yorum paneli de hesap yapmaz; Jaimini dahil her şeyi uygulama hesaplar ve bu belgede tanımlanan tipleri okur.
- **Platform:** yalnızca macOS.
- **Adlandırma yasağı** (v1 spec §10) klasik terimleri de kapsar; yasak harf dizisini içeren klasik adlar bu belgede ve kodda yeniden adlandırılır (Ek A).

## 2. Ortak kararlar

Her karar gerekçesiyle kayıtlıdır; alt projelerin hepsini bağlar.

- **K1 — Modüller `Chart`'a eklenmez, istek üzerine hesaplanır.** `Chart` ve `open_client` değişmez; her modülün kendi komutu vardır ve panel açıkken çağrılır. Gerekçe: `open_client` bütçesi (Rust ≤ 50 ms) korunur, kullanılmayan modül hesaplanmaz, modül sürümleri ayrı ilerler.
- **K2 — Ortak girdi `ChartInput { jd_ut, lat, lon, settings, time_known }`.** `compute_chart` ile aynı alanlar, artı `time_known`. Doğum saati bilinmiyorsa lagnaya bağlı her çıktı `None` döner; arayüzün gizlemesine güvenilmez. Gerekçe: aynı veriyi AI paneli ve baskı da okuyacak; gizleme tek yerde olmalı.
- **K3 — Saf çekirdek, ince toplayıcı.** Her modülün hesap fonksiyonu ephemeris görmeyen düz bir girdi alır (`Positions { lagna_lon: Option<f64>, grahas: [GrahaPoint; 9] }`, `GrahaPoint { graha, sidereal_lon, lat, speed }`; `From<&Chart>` ile üretilir). Zamana bağlı modüller (Shadbala, Muhurta) gereken astronomik değerleri ayrı bir toplayıcı fonksiyonla alır ve saf fonksiyona verir. Gerekçe: el hesabı fixture'ları ephemeris olmadan yazılabilir (§13).
- **K4 — Evler rasi evidir.** Aksi yazmadıkça "ev" lagna burcundan sayılan bütün burç evidir; `CalcSettings.house_system` yalnızca v1'deki bhava tablosunu etkiler. Gerekçe: yoga, arudha, argala ve kuta kuralları klasik kaynaklarda burç sayımıyla tanımlıdır; ev sistemi ayarıyla sessizce değişen yoga listesi hata kaynağıdır.
- **K5 — Rasi çerçevesi `settings.zodiac`'ı izler.** Burca dayalı her kural haritanın rasi çerçevesindeki burçları kullanır (v1'deki dignity ve varga ile aynı); nakshatraya dayalı kurallar sidereal boylamı kullanır.
- **K6 — Doğal iyicil/kötücül sınıfı tek fonksiyondur.** `nature(graha, positions) -> Benefic | Malefic`: Jüpiter ve Venüs iyicil; Güneş, Mars, Satürn, Rahu, Ketu kötücül; Ay büyürken (Güneş'ten uzanım 0°–180°) iyicil, küçülürken kötücül; Merkür aynı burçta kötücül varsa ve iyicil yoksa kötücül, değilse iyicil (BPHS, graha nitelikleri bölümü). Modüller kendi listesini tutmaz.
- **K7 — Ayarlar: `CalcSettings` şema sürümü 2.** `advanced: AdvancedSettings` alt yapısı eklenir; tüm alanları varsayılanlıdır ve bilinmeyen anahtarlar yok sayılır. Sürüm 1 → 2 göçü varsayılanları yazar. Yalnızca en az iki uygulanmış seçeneği olan kararlar ayar olur: `karaka_scheme {Eight, Seven}` ve `jaimini_co_lords: bool`. Tek seçenekli yöntemler ayar değildir; her modül sonucu `method` kimliği taşır (ör. `"chara_dasha.padakrama_v1"`), baskı ve AI paneli hangi yöntemin kullanıldığını buradan söyler. Fark rozeti `advanced` alanlarını da kapsar.
- **K8 — Hata ve uyarı.** Yeni hatalar `CoreError` → `{code, params}` yolunu izler; parametreler danışan verisi taşımaz. Ölümcül olmayan durumlar modül sonucunun `warnings` alanındadır.
- **K9 — Başka yazılım çıktısı fixture olmaz.** Bu modüllerde referans değerler klasik kuraldan elle türetilir (§13). Gerekçe: clean-room; ayrıca bu teknikler yazılımdan yazılıma varyant farkı taşır, başka programla aynı sonucu vermek doğruluk ölçütü değildir.
- **K10 — Baskı düzeni eklenmez.** Dört baskı düzeni (UI spec §7) değişmez. Modül baskıları ayrı bir işin konusudur.
- **K11 — Panel dili.** Klasik terimler üç dilde aynı transliterasyonla yazılır, yanında kısa çeviri etiketi olabilir ("Uccha bala · yücelme gücü"). Etiket bir addır, yorum değildir.

## 3. Alt projeler, sıra ve bağımlılıklar

| Sıra | Alt proje | Plan | Bağımlı olduğu | Ürettiği |
|---|---|---|---|---|
| 1 | 3.1 Ortak temel: bakışlar, ev lordları, modül IPC'si, panchang, ashtakavarga ve bakış panelleri | 14 | v1 | `drishti`, `lordship`, `ChartInput`, üç panel |
| 2 | 3.2 Jaimini çekirdeği: chara karaka, rashi drishti, arudha pada, karakamsha, argala | 15 | 3.1 | `jaimini` modülü, üç panel |
| 3 | 3.3 Chara dasha | 16 | 3.2 | burç dashası, dasa panellerinde sistem seçimi |
| 4 | 3.4 Shadbala ve Bhava bala | 17 | 3.1 | `shadbala` modülü, iki panel |
| 5 | 3.5 Yogalar | 18 | 3.1 | kural kataloğu, yoga paneli |
| 6 | 3.6 Uyum: Ashtakuta ve karşılıklı yerleşim | 19 | 3.1 | `kuta` modülü, uyum paneli |
| 7 | 3.7 Muhurta: gün panchangı, gün dilimleri, zaman çizelgesi, pencere araması | 20 | 3.1 | `timeline`, `muhurta`, iki panel |
| 8 | 3.8 Prashna | 21 | 3.1, 3.7 (`home_place`) | prashna haritası, olgu paneli |
| 9 | 3.9 Arama: harita araması ve olay araması | 22 | 3.1, 3.7 (`timeline`); 3.2 ve 3.5 varsa koşulları | sorgu motoru, arama yüzeyi, olay paneli |

Sıranın gerekçesi: 3.1 herkesin kullandığı bakış ve lord yardımcılarını verir. Jaimini (3.2, 3.3) öne alınır, çünkü AI paneli Jaimini verisini bekler ve bu veri saf burç aritmetiğidir (risk düşük). Shadbala en çok varyant ve astronomi içeren parçadır; temel oturduktan sonra gelir. Arama en sonda yapılır ki önceki modüllerin olguları sorgulanabilsin. 3.4, 3.5 ve 3.6 birbirinden bağımsızdır; sıraları değişebilir ya da paralel yürüyebilir.

---

## 4. Alt proje 3.1 — Ortak temel

### Kapsam

- Graha bakışları: burç temelli tam bakış ve derece temelli bakış değeri (sphuta drishti).
- Ev lordluğu yardımcıları ve ortak `kuja_dosha` olgusu (3.5 ve 3.6 kullanır).
- `ChartInput`, `Positions`, `CalcSettings` sürüm 2 göçü, modül komutlarının ortak kalıbı.
- Üç panel: `panchang` ve `ashtakavarga` (verisi v1'den beri `Chart` içinde; UI spec §3.3 "v1 sonrası eklenir") ve `drishti`.

### Yöntem

**Burç temelli graha bakışı** (BPHS, bakışlar bölümü): her graha bulunduğu burçtan 7. burca bakar. Ek tam bakışlar: Mars 4 ve 8, Jüpiter 5 ve 9, Satürn 3 ve 10. Rahu ve Ketu'ya graha bakışı atanmaz (karar: düğüm bakışı klasik kaynaklarda tutarlı değildir; rashi drishti 3.2'de düğümleri zaten kapsar).

**Sphuta drishti:** `A = (bakılan − bakan) mod 360`, sidereal boylamlarla. Temel değer virupa cinsinden (0–60), parçalı doğrusal:

| A aralığı | Değer |
|---|---|
| 0°–30° | 0 |
| 30°–60° | (A − 30) / 2 |
| 60°–90° | (A − 60) + 15 |
| 90°–120° | (120 − A) / 2 + 30 |
| 120°–150° | 150 − A |
| 150°–180° | (A − 150) × 2 |
| 180°–300° | (300 − A) / 2 |
| 300°–360° | 0 |

Özel bakış eklemesi: Mars için A ∈ [90°, 120°) ve [210°, 240°) aralığında +15; Jüpiter için [120°, 150°) ve [240°, 270°) aralığında +30; Satürn için [60°, 90°) ve [270°, 300°) aralığında +45. Sonuç 60'ta kesilir. Karar: ekleme-ve-kesme biçimi seçildi, çünkü özel bakışın başladığı tam noktalarda (90°, 210°, 120°, 240°, 60°, 270°) tam 60 verir ve hiçbir yerde tam bakışı aşmaz. Bakan graha olarak yalnızca yedi graha alınır; bakılan nokta herhangi bir boylam olabilir (graha ya da bhava ortası).

**Ev lordluğu:** `house_lord(lagna_sign, house) -> Graha`, `houses_owned(lagna_sign, graha) -> Vec<u8>`; burç lordu v1'deki `dignity::sign_lord` (Akrep: Mars, Kova: Satürn). Jaimini'nin eş lordları 3.2'dedir.

**Kuja dosha olgusu:** `kuja_dosha(positions) -> KujaDosha { from_lagna: Option<bool>, from_moon: bool, from_venus: bool }`; Mars ilgili referanstan 1, 2, 4, 7, 8 ya da 12. evdeyse `true`. İptal kuralı uygulanmaz.

### Veri modeli ve IPC

```rust
pub struct ChartInput { pub jd_ut: f64, pub lat: f64, pub lon: f64,
                        pub settings: CalcSettings, pub time_known: bool }
pub struct GrahaPoint { pub graha: Graha, pub sidereal_lon: f64, pub rasi_sign: Sign,
                        pub deg_in_sign: f64, pub lat: f64, pub speed: f64 }
pub struct Positions { pub lagna_lon: Option<f64>, pub lagna_sign: Option<Sign>,
                       pub grahas: [GrahaPoint; 9] }
pub fn graha_aspect_signs(graha: Graha, from: Sign) -> Vec<Sign>;
pub fn sphuta_drishti(aspecting: Graha, from_lon: f64, to_lon: f64) -> f64;
pub struct SignAspect { pub graha: Graha, pub target: Sign, pub occupants: Vec<Graha> }
pub struct DrishtiTable {            // komut sonucu
    pub sign_aspects: Vec<SignAspect>,        // (graha, hedef burç, hedefteki grahalar)
    pub sphuta: [[f64; 9]; 7],                // bakan 7 graha × bakılan 9 graha, virupa
    pub method: String,
}
```

Komut: `compute_drishti(input: ChartInput) -> DrishtiTable`. Sonraki alt projelerin komutları aynı kalıbı izler: tek `ChartInput`, motor iş parçacığında çalışır, `spawn_blocking`, hata `CommandError`.

### Arayüz

| Panel | `kind` | Ayarlar | İçerik |
|---|---|---|---|
| Panchang | `panchang` | — | Doğum anının tithi (paksha ile), vara, nakshatra ve pada, nitya yoga, karana; her birinin bitiş saati; gün doğumu ve batımı. Gün doğumu yoksa mevcut uyarı rozeti. |
| Ashtakavarga | `ashtakavarga` | görünüm (SAV / tek graha BAV), biçim (tablo / harita içi) | 12 burç × 8 satır tablo; satır ve sütun toplamları. |
| Bakışlar | `drishti` | tür (graha / rashi — rashi 3.2 ile gelir), değer (tam bakış işareti / virupa) | Bakan × bakılan matrisi. Tam bakış `●`, kısmi değer sayı olarak; renk yoktur. |

Üç panel de ortak seçimle bağlıdır (UI spec §3.4): bir graha ya da burç vurgulanınca ilgili satır ve sütun `--select` zemin alır. Retro her yerde düz `R`'dir ve Rahu ile Ketu'da gösterilmez.

### Test

- Sphuta drishti: tablo sınır noktalarında süreklilik; tam noktalarda beklenen değerler (60° → 15, 90° → 45, 120° → 30, 180° → 60, 210° → 45, 240° → 30, 270° → 15); özel bakışlarla 60. Property: değer her zaman [0, 60].
- Burç bakışı: her graha için hedef burç kümesi el tablosuyla birebir; 7. bakış simetrik.
- Göç: sürüm 1 ayar JSON'u sürüm 2'ye varsayılanlarla taşınır; bilinmeyen `advanced` anahtarı açılışı engellemez.
- Paneller: boş, yükleniyor, hata, dolu ve "saat bilinmiyor" durumları (UI spec §8).

### Gizlilik, performans, kapsam dışı

- Gizlilik: §14. Yeni kalıcı veri yok.
- Performans: `compute_drishti` Rust tarafında ≤ 10 ms (temel harita hesabı dahil).
- Kapsam dışı: düğüm bakışı varyantları, Tajika bakışları, bakış çizgilerinin harita üstüne çizimi.

---

## 5. Alt proje 3.2 — Jaimini çekirdeği

Kaynak: Jaimini Upadesha Sutraları ve BPHS'nin karaka, pada ve argala bölümleri. AI panelinin beklediği altı kalemin beşi buradadır; altıncısı (chara dasha) 3.3'tedir.

### Kapsam ve yöntem

**Chara karaka.** Grahalar burç içi derecelerine göre büyükten küçüğe sıralanır.

- `Eight` (varsayılan): Güneş…Satürn ve Rahu. Rahu'nun derecesi `30 − deg_in_sign` alınır. Sıra: Atma (AK), Amatya (AmK), Bhratri (BK), Matri (MK), Pitri (PiK), Putra (PK), Gnati (GK), Dara (DK).
- `Seven`: Rahu yok. Sıra: AK, AmK, BK, MK, PK, GK, DK.
- Ketu karaka olmaz.
- Eşitlik: iki grahanın derecesi 1″ içinde eşitse sonuç `tie: true` taşır; sıra tam `f64` değeriyle, o da eşitse Güneş…Rahu doğal sırasıyla belirlenir. Karaka birleşmesi kuralları uygulanmaz.
- Karar: varsayılan `Eight`, çünkü BPHS'nin karaka bölümü iki şemayı da verir ve sekizli şema karaka kaybetmez; yedili şemayı kullanan ekoller harita başına ayarla seçer.

**Rashi drishti.** Yürük (chara) burç, kendisine bitişik olan dışında üç sabit burca bakar; sabit (sthira) burç, bitişiği dışında üç yürük burca bakar; çift doğalı (dwiswabhava) burç diğer üç çift doğalıya bakar. "Bitişik": yürük burç için kendinden sonraki sabit burç, sabit burç için kendinden önceki yürük burç. Örnek: Koç → Aslan, Akrep, Kova; Boğa → Yengeç, Terazi, Oğlak; İkizler → Başak, Yay, Balık. Bir burçtaki grahalar o burcun baktığı burçlara ve oradaki grahalara bakar.

**Eş lordlar.** `jaimini_co_lords = true` (varsayılan) iken Akrep'in lordları Mars ile Ketu, Kova'nın lordları Satürn ile Rahu'dur ve güçlü olan lord sayılır. `stronger_co_lord(sign, positions)` sırayla şu ölçütleri uygular, ilk ayrım kazanır:

1. Yalnızca biri burcun kendisindeyse, burcun dışındaki lord seçilir.
2. Aynı burçta daha çok graha ile birlikte olan (diğer eş lord sayılmaz).
3. Yücelme burcunda olan; ikisi de değilse, düşüş burcunda olmayan. Düğümlere bu adım uygulanmaz.
4. Bulunduğu burcun türü: çift doğalı > sabit > yürük.
5. Söz konusu burçtan lorda, burcun sayım yönünde (3.3) daha uzak olan.
6. Mars (Akrep) ya da Satürn (Kova).

`jaimini_co_lords = false` iken yalnızca Mars ve Satürn kullanılır. Bu fonksiyon arudha ve chara dasha tarafından ortak kullanılır.

**Arudha pada** (BPHS, pada bölümü). `h` evinin burcu `S`, lordu `T` burcunda olsun; `n`, `S`'den `T`'ye kapsayıcı sayım (1–12). Hesaplanan pada `T`'den `n`. burçtur. İstisna: hesaplanan burç `S`'nin kendisi ya da `S`'den 7. burç ise pada, hesaplanan burçtan 10. burçtur (yani sırasıyla `S`'den 10. ve `S`'den 4. burç). On iki pada verilir: A1 (Arudha Lagna, AL) … A12 (Upapada, UL). Graha arudhaları kapsam dışıdır.

**Karakamsha.** Atmakaraka'nın navamsa (D9) burcu. Sonuç iki görünüm taşır: karakamsha burcu rasi haritasına taşınmış (her grahanın D1 burcunun karakamshadan ev sayısı) ve swamsha (her grahanın D9 burcunun karakamshadan ev sayısı). Karakamsha lagnaya bağlı değildir; doğum saati bilinmiyorsa "yaklaşık" işaretlenir (AK değişebilir).

**Argala.** Her burç referans alınarak hesaplanır (12 satır):

- Argala veren evler ve engelleyenleri (virodha): 2 ↔ 12, 4 ↔ 10, 11 ↔ 3; ikincil: 5 ↔ 9.
- Her çift için iki graha listesi verilir (dokuz graha da sayılır). Durum yalnızca sayıya bakar: argala tarafı çoksa `Stands`, eşitse `Contested`, azsa `Obstructed`, argala tarafı boşsa `None`.
- Referanstan 3. evde üç ya da daha çok doğal kötücül varsa `third_house_malefic_argala: true`.
- Karar: güç karşılaştırması yapılmaz; iki liste de gösterildiği için astrolog kendi hükmünü verir. Ketu için ters sayım uygulanmaz.

### Veri modeli ve IPC

```rust
pub enum KarakaRole { Atma, Amatya, Bhratri, Matri, Pitri, Putra, Gnati, Dara }
pub struct CharaKaraka { pub role: KarakaRole, pub graha: Graha, pub effective_deg: f64, pub tie: bool }
pub struct ArudhaPada { pub house: u8, pub sign: Sign, pub lord: Graha, pub exception: bool }
pub struct Karakamsha { pub atmakaraka: Graha, pub sign: Sign,
                        pub houses_in_d1: [u8; 9], pub houses_in_d9: [u8; 9] }
pub enum ArgalaStatus { None, Stands, Contested, Obstructed }
pub struct ArgalaPair { pub argala_house: u8, pub virodha_house: u8,
                        pub argala_grahas: Vec<Graha>, pub virodha_grahas: Vec<Graha>,
                        pub status: ArgalaStatus }
pub struct ArgalaRow { pub reference: Sign, pub pairs: [ArgalaPair; 4],
                       pub third_house_malefic_argala: bool }
pub struct JaiminiReport {
    pub scheme: KarakaScheme, pub co_lords: bool, pub method: String,
    pub karakas: Vec<CharaKaraka>,                 // 7 ya da 8
    pub karakamsha: Karakamsha,
    pub rashi_drishti: [[bool; 12]; 12],
    pub graha_rashi_drishti: Vec<SignAspect>,      // graha → baktığı burçlar ve oradaki grahalar
    pub arudhas: Option<[ArudhaPada; 12]>,         // saat bilinmiyorsa None
    pub argala: [ArgalaRow; 12],                   // lagna burcundan sıralı; saat bilinmiyorsa Koç'tan
    pub warnings: Vec<ChartWarning>,
}
```

Argala lagnaya bağlı değildir (referans burçtur), bu yüzden doğum saati bilinmese de döner; yalnızca satır sırası değişir. Komut: `compute_jaimini(input: ChartInput) -> JaiminiReport`. `JaiminiReport` AI paneli için sözleşmedir; alan adları değişirse AI paketleyicisi de değişir.

### Arayüz

| Panel | `kind` | Ayarlar | İçerik |
|---|---|---|---|
| Jaimini karaka tablosu | `jaimini_karakas` | — | Rol, graha, burç içi derece (Rahu için hem gerçek hem etkin derece), eşitlik işareti `=`; altında karakamsha burcu ve grahaların karakamshadan ev sayıları (D1 / D9). |
| Arudha padalar | `arudha` | — | A1…A12: burç, lord, istisna uygulanmışsa `*` ve dipnot. AL ve UL etiketlidir. |
| Argala | `argala` | referans (lagna / seçili burç) | Dört çift için iki graha listesi ve durum metni ("geçerli", "çekişmeli", "engelli", "yok"). |

- `chart` paneline `arudha_marks` ayarı eklenir (varsayılan kapalı): padalar burçlarına "AL", "A2"… "UL" olarak yazılır.
- `drishti` paneline `rashi` türü eklenir.
- `graha_table` paneline isteğe bağlı "karaka" sütunu eklenir.
- Durumlar yalnızca metinle ayrılır; renk anlam taşımaz.

### Test

- **El örnekleri** (`tests/golden/modules/jaimini/`, kaynak `hand`): en az 8 sentetik konum seti. Her biri karaka sırasını, on iki padayı, karakamshayı ve en az iki referans için argala listelerini kural adımlarıyla birlikte verir. Zorunlu durumlar: Rahu'nun AK olduğu, 1″ içinde eşitlik, lordun kendi evinde, 4., 7. ve 10. evde olduğu arudha istisnaları, eş lord seçiminin altı adımından her birinin belirleyici olduğu durumlar, AK'nın vargottama olduğu durum.
- **Property:** karaka sırası seçilen şemadaki grahaların bir permütasyonudur; rashi drishti simetriktir ve her burç tam üç burca bakar, kendi türüne bakmaz (çift doğalılar hariç); hiçbir pada evin kendi burcuna ya da 7. burcuna düşmez; argala çiftlerinde aynı graha iki tarafta görünmez.
- **Uçtan uca:** mevcut beş `swetest` fixture anında `compute_jaimini` hata vermez ve property'leri sağlar.

### Gizlilik, performans, kapsam dışı

- Gizlilik: §14. Yeni kalıcı veri: yalnızca iki ayar alanı.
- Performans: ≤ 10 ms.
- Kapsam dışı: graha arudhaları, karaka birleşmesi, Ketu için ters argala, güçle hükmedilen argala, Jaimini rasi güç sıralamasının tamamı, sthira karaka şeması, pada lagnaları dışındaki özel lagnalar (hora, ghati, bhava lagna).

---

## 6. Alt proje 3.3 — Chara dasha

### Kapsam ve yöntem

Burç temelli Jaimini dashası, padakrama düzeni (20. yüzyıl Jaimini yorum geleneğinde en yaygın uygulanan biçim). `method = "chara_dasha.padakrama_v1"`.

- **Burç grupları.** Düz sayan: Koç, Boğa, İkizler, Terazi, Akrep, Yay. Ters sayan: Yengeç, Aslan, Başak, Oğlak, Kova, Balık.
- **Dasha sırası.** İlk dasha lagna burcudur. Lagnadan 9. burç düz sayan gruptaysa dashalar zodyak sırasıyla, ters sayan gruptaysa ters sırayla ilerler. On iki burç birer kez gelir.
- **Dasha süresi.** Dasha burcu düz sayan gruptaysa burçtan lorduna ileri, değilse geri doğru kapsayıcı sayılır; yıl = sayım − 1. Lord kendi burcundaysa 12 yıl. Akrep ve Kova için lord `stronger_co_lord` ile seçilir; iki eş lord da burcun içindeyse 12 yıl. Yücelme ve düşüş için ±1 yıl düzeltmesi uygulanmaz.
- **İkinci çevrim.** On iki dasha bittikten sonra aynı sırayla ikinci çevrim gelir; her burcun süresi `12 − ilk çevrimdeki yıl`dır, 0 yıllık burçlar atlanır. Üretim doğumdan 120 yıl sonrasını geçen ilk dashada durur.
- **Alt dönemler.** Her dönem on iki eşit parçaya bölünür. Sıra harita düzeyindeki yönle aynıdır; ilk alt dönem dasha burcundan sonraki burç, sonuncusu dasha burcunun kendisidir. Aynı kural 3. seviyeye de uygulanır. En çok 3 seviye.
- **Başlangıç.** Doğumda ilk dasha tam süresiyle başlar (bakiye hesabı yoktur). Yıl uzunluğu `settings.dasha_year`'dır.
- Doğum saati bilinmiyorsa chara dasha hesaplanmaz (lagnaya bağlı).

### Veri modeli ve IPC

```rust
pub struct SignDashaPeriod { pub signs: Vec<Sign>, pub level: u8, pub cycle: u8,
                             pub start_jd_ut: f64, pub end_jd_ut: f64, pub has_children: bool }
pub struct CharaDasha { pub forward: bool, pub years: [u8; 12],        // lagnadan dasha sırasıyla
                        pub periods: Vec<SignDashaPeriod>,              // 2 seviye
                        pub method: String }
```

Komutlar: `compute_chara_dasha(input: ChartInput) -> Option<CharaDasha>`, `chara_dasha_children(input: ChartInput, path: Vec<Sign>, cycle: u8) -> Vec<SignDashaPeriod>`.

### Arayüz

- `dasha_tree` ve `dasha_strip` panellerinin `system` ayarı `vimshottari | chara` olur. Chara seçiliyken satırlar graha yerine burç kısaltması taşır; ikinci çevrim satırları "2." önekiyle ayrılır.
- Ağaç başlığında yön ("ileri" / "geri") ve on iki burcun yıl dizisi küçük bir satırda gösterilir.
- Dönem seçimi transit tarihini v1'deki gibi sürer (UI spec §3.4).
- `ViewState.dasha_expanded_path` sistem başına ayrı tutulur (toleranslı ayrıştırma; eski kayıt Vimshottari yolu sayılır).

### Test

- **El örnekleri:** en az 6 sentetik harita; her biri yönü, on iki yılı ve ilk iki dashanın alt dönem sırasını kural adımlarıyla verir. Zorunlu durumlar: ileri ve geri yön, lordu kendi burcunda (12 yıl), lordu 2. ve 12. burçta (1 yıl ve 11 yıl iki grup için), Akrep ve Kova'da eş lord seçimi, ikinci çevrimde atlanan burç.
- **Property:** her süre 1–12 yıl; ilk çevrim on iki burcun permütasyonudur ve lagnadan başlar; dönemler boşluksuz ardışıktır; alt dönemlerin toplamı ebeveyne eşittir; son alt dönem ebeveyn burcudur; iki çevrimde aynı burcun yılları toplamı 12'dir.
- **Çapraz denetim:** yılları üreten fonksiyonun yanında, aynı kuralı tablo arayarak (12 burç × 12 lord konumu, elle yazılmış 144 hücre) veren ikinci bir test yardımcısı; ikisi her girdide eşit olmalıdır.

### Gizlilik, performans, kapsam dışı

- Gizlilik: §14. Kalıcı veri: yalnızca panel ayarı.
- Performans: ≤ 10 ms; alt dönemler ≤ 5 ms.
- Kapsam dışı: diğer chara dasha varyantları, ±1 yıl düzeltmesi, sthira/shoola/narayana ve diğer burç dashaları, Yogini ve koşullu nakshatra dashaları.

---

## 7. Alt proje 3.4 — Shadbala ve Bhava bala

Kaynak: BPHS'nin graha gücü ve bhava gücü bölümleri; hesap adımlarında 20. yüzyılın yaygın hesap geleneği. Yalnızca yedi graha. Birim virupa; 60 virupa = 1 rupa. `method = "shadbala.bphs_v1"`.

### Yöntem — altı güç

**1. Sthana bala (konum gücü)** — beş kalemin toplamı:

- *Uccha bala:* grahanın düşüş noktasından açısal uzaklığı (0°–180°) / 3.
- *Saptavargaja bala:* D1, D2, D3, D7, D9, D12, D30 vargalarının her birinde, grahanın bulunduğu burcun lorduyla ilişkisine göre: moolatrikona 45 (yalnızca D1), kendi burcu 30, çok dost 20, dost 15, nötr 10, düşman 4, çok düşman 2. İlişki v1'deki bileşik ilişkidir (doğal + D1'den geçici). Karar: BPHS'nin değerleri kullanılır; 22,5 / 7,5 / 3,75 / 1,875 dizisini veren hesap geleneği uygulanmaz.
- *Ojayugma bala:* Ay ve Venüs çift burçta, diğerleri tek burçta 15; aynı kural navamsa burcu için ayrıca 15.
- *Kendradi bala:* kendra (1, 4, 7, 10) 60; panaphara (2, 5, 8, 11) 30; apoklima (3, 6, 9, 12) 15.
- *Drekkana bala:* eril grahalar (Güneş, Mars, Jüpiter) burcun ilk 10°'sinde, nötrler (Merkür, Satürn) ikinci 10°'de, dişiller (Ay, Venüs) son 10°'de 15.

**2. Dig bala (yön gücü):** güçsüz noktadan açısal uzaklık (0°–180°) / 3. Güçsüz nokta: Güneş ve Mars için IC, Ay ve Venüs için MC, Merkür ve Jüpiter için alçalan (lagna + 180°), Satürn için lagna. Karar: lagna ve MC astronomik açılardır (`Ephemeris::houses`); ev sistemi ayarından bağımsızdır.

**3. Zaman gücü** (`temporal`; klasik adı yasak diziyi içerdiği için bu adla anılır, Ek A) — kalemleri:

- *Gündüz–gece gücü* (`day_night`, klasik natonnata): `m` = doğum anının en yakın görünür öğleye uzaklığı, saat (0–12). Görünür öğle, o günün gün doğumu ile batımının orta noktasıdır. Güneş, Jüpiter, Venüs: `60 × (1 − m/12)`. Ay, Mars, Satürn: `60 × m/12`. Merkür her zaman 60.
- *Paksha bala:* `e` = Ay'ın Güneş'ten uzanımı, 0°–180°'ye indirgenmiş. Ay: `2 × e/3`. Diğerleri K6 sınıfına göre: iyicil `e/3`, kötücül `(180 − e)/3`.
- *Tribhaga bala:* gündüz (doğuş → batış) üç eşit dilim: Merkür, Güneş, Satürn. Gece (batış → sonraki doğuş) üç eşit dilim: Ay, Venüs, Mars. İçinde bulunulan dilimin lordu 60 alır. Jüpiter her zaman 60 alır.
- *Yıl, ay, gün ve saat lordu:* yıl lordu 15, ay lordu 30, gün lordu 45, saat lordu 60.
  - Gün lordu: vara lordu (vara gün doğumunda değişir, v1).
  - Saat lordu (hora): gün doğumundan sonraki gün doğumuna 24 eşit dilim; ilk dilim gün lordunundur, sonrakiler Satürn → Jüpiter → Mars → Güneş → Venüs → Merkür → Ay döngüsünde ilerler.
  - Yıl lordu: Güneş'in doğumdan önceki son sidereal Koç girişinin gerçekleştiği varanın lordu. Ay lordu: Güneş'in bulunduğu sidereal burca girişinin gerçekleştiği varanın lordu.
  - Karar: yıl ve ay lordu güneş girişinden bulunur, çünkü ephemeris ile doğrulanabilir ve tek anlamlıdır; 360 günlük yıllarla ahargana sayımı uygulanmaz. Bu, modülün en çok tartışılabilir varyant seçimidir (§17).
- *Ayana bala:* `δ = asin(sin 23°27′ × sin λ)`, `λ` tropikal boylam. Güneş, Mars, Jüpiter, Venüs: `(24 + δ) / 48 × 60`. Ay, Satürn: `(24 − δ) / 48 × 60`. Merkür: `(24 + |δ|) / 48 × 60`. Güneş'inki iki katıdır.
- *Graha savaşı:* Mars, Merkür, Jüpiter, Venüs, Satürn'den ikisi 1° içindeyse `war: Some { with, winner }` işaretlenir; kazanan, ekliptik enlemi daha kuzeyde olandır. Güç aktarımı uygulanmaz (kapsam dışı).

**4. Cheshta bala (hareket gücü):**

- Güneş: ayana bala değeri (iki katı alınmamış hâli). Ay: paksha bala değeri (iki katı alınmamış hâli).
- Mars, Jüpiter, Satürn: `ort = (ortalama boylam + gerçek boylam) / 2` (dairesel orta nokta); `K = (ortalama Güneş − ort) mod 360`, 180°'den büyükse `360 − K`; güç `K / 3`.
- Merkür, Venüs: aynı formül, "ortalama boylam" yerine ortalama Güneş, "ortalama Güneş" yerine grahanın güneş merkezli ortalama boylamı.
- Ortalama boylamlar J2000 çağlı doğrusal ortalama öğelerden hesaplanır (`L0 + L1 × T`, tropikal, tarihin ekinoksu); gerçek boylam tropikal jeosantrik boylamdır. Katsayılar plan aşamasında yayımlanmış astronomik tablodan alınır ve ephemeris'ten uzun dönem ortalamasıyla ±0,05° içinde doğrulanır.

**5. Naisargika bala (doğal güç):** sabit. Güneş 60,00; Ay 51,43; Venüs 42,86; Jüpiter 34,29; Merkür 25,71; Mars 17,14; Satürn 8,57.

**6. Drik bala (bakış gücü):** grahaya diğer altı grahadan gelen sphuta drishti değerleri: `(Σ iyicil − Σ kötücül) / 4`, artı Merkür'ün ve Jüpiter'in bakış değerlerinin tamamı. Negatif olabilir.

**Toplam ve eşik.** Toplam altı gücün toplamıdır. Klasik gerekli en az değerler (virupa): Güneş 390, Ay 360, Mars 300, Merkür 420, Jüpiter 390, Venüs 330, Satürn 300. Sonuç `total / required` oranını ve yedi graha arasındaki sırayı verir.

### Yöntem — Bhava bala

Bhava ortası `lagna_lon + 30° × (h − 1)` alınır (karar: eşit bhava; ev sistemi ayarından bağımsız ve tek anlamlı).

- *Bhavadhipati bala:* ev lordunun Shadbala toplamı.
- *Bhava dig bala:* bhava ortasının düştüğü burcun türüne göre güçsüz noktadan açısal uzaklık / 3. İnsan burçları (İkizler, Başak, Terazi, Kova, Yay'ın ilk yarısı): güçsüz nokta 7. bhava ortası. Dört ayaklılar (Koç, Boğa, Aslan, Yay'ın ikinci yarısı, Oğlak'ın ilk yarısı): 4. bhava ortası. Su burçları (Yengeç, Balık, Oğlak'ın ikinci yarısı): 10. bhava ortası. Akrep: 1. bhava ortası.
- *Bhava drishti bala:* drik bala formülü, bakılan nokta bhava ortası.
- Toplam üçünün toplamıdır; on iki bhava sıralanır.

### Veri modeli ve IPC

```rust
pub struct SthanaBala { pub uccha: f64, pub saptavargaja: f64, pub ojayugma: f64,
                        pub kendradi: f64, pub drekkana: f64, pub total: f64 }
pub struct TemporalBala { pub day_night: f64, pub paksha: f64, pub tribhaga: f64,
                          pub year_lord: f64, pub month_lord: f64, pub weekday_lord: f64,
                          pub hora_lord: f64, pub ayana: f64, pub total: f64 }
pub struct GrahaShadbala { pub graha: Graha, pub sthana: SthanaBala, pub dig: f64,
                           pub temporal: TemporalBala, pub cheshta: f64, pub naisargika: f64,
                           pub drik: f64, pub total: f64, pub rupas: f64,
                           pub required: f64, pub ratio: f64, pub rank: u8,
                           pub war: Option<GrahaWar> }
pub struct TimeLords { pub year: Graha, pub month: Graha, pub weekday: Graha, pub hora: Graha,
                       pub tribhaga: Graha }
pub struct BhavaBala { pub house: u8, pub lord: Graha, pub lord_bala: f64, pub dig: f64,
                       pub drishti: f64, pub total: f64, pub rank: u8 }
pub struct ShadbalaReport { pub grahas: [GrahaShadbala; 7], pub time_lords: TimeLords,
                            pub bhavas: [BhavaBala; 12], pub method: String,
                            pub warnings: Vec<ChartWarning> }
```

Saf fonksiyon `shadbala(positions, astro: &ShadbalaAstro)`; `ShadbalaAstro` toplayıcıdan gelir: önceki ve sonraki gün doğumu ile batımı, MC, tropikal boylamlar, ortalama boylamlar, yıl ve ay girişlerinin varası. Komut: `compute_shadbala(input: ChartInput) -> Option<ShadbalaReport>`; doğum saati bilinmiyorsa ya da gün doğumu yoksa (kutup günü) `None` ve ilgili uyarı.

### Arayüz

| Panel | `kind` | Ayarlar | İçerik |
|---|---|---|---|
| Shadbala | `shadbala` | ayrıntı (özet / kalemler), birim (virupa / rupa) | Satır: yedi graha. Özet sütunları: altı güç, toplam, gerekli, oran, sıra. "Kalemler" görünümünde sthana ve zaman gücünün alt kalemleri açılır. Altında zaman lordları satırı. |
| Bhava bala | `bhava_bala` | birim | On iki bhava: lord, üç kalem, toplam, sıra. |

- Oran 1'in altındaysa hücrede `<1` metin işareti vardır; renk kullanılmaz. Yatay çubuk gösterilirse sayı her zaman yanındadır.
- Graha savaşı satırda `⚔` yerine düz metin "savaş: Ve" ile gösterilir.
- Sayılar `tabular-nums`, bir ondalık.

### Test

- **Çalışma kâğıdı** (`tests/golden/modules/shadbala/WORKSHEET.md` ve eşlik eden JSON, kaynak `hand`): en az 3 sentetik girdi için her kalemin aritmetiği adım adım yazılır; kâğıdı motoru yazan dışında bir gözden geçiren doğrular. Tolerans 0,01 virupa.
- **Kalem düzeyinde el örnekleri:** her kalem için en az 4 durum (uç değerler ve bir ara değer). Örnek: yücelme noktasında uccha 60, düşüş noktasında 0; öğlede Güneş'in gündüz–gece gücü 60, gece yarısında 0; dolunayda Ay'ın paksha bala değeri 120.
- **Property:** uccha(λ) + uccha(λ + 180°) = 60; karşıt noktalarda dig bala toplamı 60; gündüz ve gece güçlü grahaların `day_night` toplamı 60; iyicil ve kötücül paksha değerleri toplamı 60; tribhaga'da Jüpiter dışında tam bir graha 60 alır; yıl, ay, gün, saat lordu değerleri {0, 15}, {0, 30}, {0, 45}, {0, 60} kümelerindedir; naisargika sabittir; her kalem kendi aralığındadır; sıra 1–7 permütasyonudur.
- **Astronomik çapraz denetim:** dış gezegen karşı konumdayken cheshta ≥ 55, kavuşumdayken ≤ 5 (ephemeris'ten bulunan gerçek karşı konum ve kavuşum anlarında); yıl girişi anında Güneş'in sidereal boylamı 0° ± 1″.
- **Uçtan uca:** beş `swetest` fixture anında hata yok ve property'ler sağlanır.

### Gizlilik, performans, kapsam dışı

- Gizlilik: §14. Kalıcı veri yok.
- Performans: ≤ 40 ms (yıl ve ay girişi aramaları dahil).
- Kapsam dışı: graha savaşında güç aktarımı, ishta ve kashta phala, vimshopaka bala, ahargana temelli yıl/ay lordu, sekiz hareket durumuna dayalı cheshta, bhava bala'nın yerleşim ve gündüz/gece ek düzeltmeleri, avasthalar, Rahu ve Ketu gücü.

---

## 8. Alt proje 3.5 — Yogalar

### Kapsam ve yöntem

Veriyle tanımlı kural kataloğu; her kural saf bir yüklemdir. Sonuç bir olgu listesidir: hangi yoga, hangi grahalar, hangi evler. Güç derecelendirmesi ve hüküm yoktur. Yalnızca D1 değerlendirilir. `YOGA_CATALOG_VERSION = 1`.

Ortak tanımlar: "birlikte" = aynı burç; "karşılıklı bakış" = iki grahanın birbirine burç temelli tam bakışı; "değişim" (parivartana) = iki grahanın birbirinin burcunda olması; "ilişkili" = bu üçünden biri. Ay ve Güneş yogalarında sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn'dür.

| Grup | Yoga | Koşul |
|---|---|---|
| Pancha Mahapurusha | Ruchaka, Bhadra, Hamsa, Malavya, Shasha | Sırasıyla Mars, Merkür, Jüpiter, Venüs, Satürn kendi ya da yücelme burcunda ve lagnadan kendrada |
| Ay | Sunapha / Anapha / Durudhara | Sayılan graha Ay'dan yalnızca 2. evde / yalnızca 12. evde / her ikisinde |
| Ay | Kemadruma | Ay'dan 2. ve 12. boş, Ay yalnız, ve Ay'dan ve lagnadan kendralarda sayılan graha yok |
| Ay | Gajakesari | Jüpiter Ay'dan kendrada |
| Ay | Chandra-Mangala | Ay ile Mars birlikte |
| Ay | Adhi | Merkür, Jüpiter, Venüs'ün üçü de Ay'dan 6., 7. ya da 8. evde |
| Ay | Shakata | Ay Jüpiter'den 6., 8. ya da 12. evde ve Ay lagnadan kendrada değil |
| Ay | Amala | Lagnadan ya da Ay'dan 10. evde en az bir doğal iyicil (K6) ve o evde kötücül yok |
| Ay | Vasumati | Merkür, Jüpiter ve Venüs'ün üçü de lagnadan upachaya evlerinde (3, 6, 10, 11) ya da üçü de Ay'dan upachaya evlerinde |
| Güneş | Vesi / Vasi / Ubhayachari | Sayılan graha Güneş'ten yalnızca 2. evde / yalnızca 12. evde / her ikisinde |
| Güneş | Budha-Aditya | Güneş ile Merkür birlikte |
| Raja | Kendra–trikona | Bir kendra lordu (1, 4, 7, 10) ile bir trikona lordu (1, 5, 9) farklı grahalardır ve ilişkilidir |
| Raja | Dharma-Karmadhipati | Yukarıdakinin 9. ve 10. lord özel durumu |
| Raja | Yogakaraka | Tek graha lagna dışında hem bir kendranın hem bir trikonanın lordu |
| Dhana | Dhana | {1, 2, 5, 9, 11} lordlarından, en az biri 2. ya da 11. lord olan iki farklı graha ilişkili |
| Değişim | Maha / üçüncü ev lordu / Dainya | Değişim; lordlar {1,2,4,5,7,9,10,11} içindeyse Maha; biri 3. lord ve diğeri o kümedeyse "3. ev lordu değişimi"; biri 6., 8. ya da 12. lordsa Dainya |
| Viparita raja | Harsha, Sarala, Vimala | Sırasıyla 6., 8., 12. lord 6., 8. ya da 12. evde |
| Neecha bhanga | Neecha bhanga | Graha düşüş burcunda ve şunlardan en az biri: düşüş burcunun lordu lagnadan ya da Ay'dan kendrada; o burçta yücelen graha lagnadan ya da Ay'dan kendrada; graha kendi dispozitörünün bakışını alıyor. Düşüş burcunda yücelen graha yoksa (Akrep) ikinci koşul uygulanmaz. Sağlanan koşullar listelenir |
| Nabhasa – sayı | Vallaki, Dama, Pasha, Kedara, Shula, Yuga, Gola | Yedi graha sırasıyla 7, 6, 5, 4, 3, 2, 1 burca dağılmış |
| Nabhasa – ashraya | Rajju, Musala, Nala | Yedi grahanın hepsi yürük / sabit / çift doğalı burçlarda |
| Dosha | Kuja dosha | 3.1'deki `kuja_dosha`: Mars lagnadan, Ay'dan ya da Venüs'ten 1, 2, 4, 7, 8 ya da 12. evde; üç referans ayrı alanlarda |

Kararlar:

- Mahapurusha yalnızca lagnadan sayılır (Ay'dan sayan varyant uygulanmaz).
- Sunapha, Anapha ve Durudhara birbirini dışlar; Vesi, Vasi ve Ubhayachari de öyle.
- Bir çift birden çok ilişki taşıyorsa tek kayıt, ilişki listesiyle döner.
- İptal kuralları yalnızca tabloda yazılı olanlardır (Kemadruma, Shakata, Neecha bhanga). Kuja dosha iptalleri uygulanmaz.
- Lagnaya bağlı yogalar doğum saati bilinmiyorsa değerlendirilmez; Ay ve Güneş yogaları "yaklaşık" işaretiyle döner.
- Tüm grahaların düğüm ekseninin tek yanında toplanmasına dayanan ve klasik kaynaklarda yer almayan modern yoga kataloğa alınmaz.

### Veri modeli ve IPC

```rust
pub enum YogaGroup { Mahapurusha, Lunar, Solar, Raja, Dhana, Exchange, ViparitaRaja,
                     NeechaBhanga, NabhasaCount, NabhasaAshraya, Dosha }
pub enum Relation { Conjunction, MutualAspect, Exchange }
pub struct YogaHit { pub id: String, pub group: YogaGroup, pub grahas: Vec<Graha>,
                     pub houses: Vec<u8>, pub relations: Vec<Relation>,
                     pub conditions: Vec<String>,          // sağlanan alt koşul kimlikleri
                     pub approximate: bool }
pub struct YogaReport { pub catalog_version: u32, pub hits: Vec<YogaHit>,
                        pub evaluated: u32, pub skipped_time_unknown: u32 }
```

`id` kararlı bir kimliktir (`"mahapurusha.hamsa"`); görünen ad i18n anahtarıdır. Komut: `compute_yogas(input: ChartInput) -> YogaReport`.

### Arayüz

`yogas` paneli. Ayar: grup süzgeci. Gruplara ayrılmış liste; her satır: yoga adı, oluşturan grahalar, evler, ilişki türü ("birlikte", "karşılıklı bakış", "değişim"), varsa sağlanan alt koşullar. Satıra gelince grahalar ve evler ortak seçimle tüm panellerde vurgulanır. Boş durum: "Katalogdaki yogalardan hiçbiri oluşmuyor" ve değerlendirilen kural sayısı. Açıklama metni yoktur.

### Test

- **Kural başına el örneği:** her yoga için en az bir oluşan ve bir oluşmayan sentetik konum; iptal kuralı olanlarda ayrıca iptal edilen durum. Fixture kuralın hangi koşulla sağlandığını yazar.
- **Property:** sayı yogalarından tam biri oluşur; kendrada olmayan graha mahapurusha veremez; Durudhara varsa Sunapha ve Anapha ayrı kayıt olarak dönmez; Kemadruma ile Sunapha/Anapha/Durudhara aynı anda oluşmaz; aynı `id` ve aynı graha kümesi iki kez dönmez; her `id` katalogdadır ve üç dilde adı vardır.
- **Uçtan uca:** beş `swetest` fixture anında hata yok.

### Gizlilik, performans, kapsam dışı

- Gizlilik: §14. Kalıcı veri: yalnızca panel ayarı.
- Performans: ≤ 10 ms.
- Kapsam dışı: akriti ve dala nabhasa yogaları, bölümsel haritalarda yoga, yoga gücü puanı, kullanıcı tanımlı kural, arishta ve ayur yogaları, Jaimini raja yogaları, dosha iptalleri.

---

## 9. Alt proje 3.6 — Uyum

### Kapsam ve yöntem

**Ashtakuta** (Kuzey Hint geleneğinin 36 puanlık sistemi; muhurta ve evlilik uyumu derlemelerindeki tablolar). Girdi iki haritanın Ay'ıdır: sidereal nakshatra ve rasi çerçevesindeki Ay burcu. Sistem yönlüdür; roller "damat rolü" ve "gelin rolü" olarak açıkça atanır.

| Kuta | En çok | Kural |
|---|---|---|
| Varna | 1 | Burç sınıfı: Brahmin (Yengeç, Akrep, Balık) > Kshatriya (Koç, Aslan, Yay) > Vaishya (Boğa, Başak, Oğlak) > Shudra (İkizler, Terazi, Kova). Damat rolünün sınıfı eşit ya da üstünse 1 |
| Vashya | 2 | Beş grup: dört ayaklı (Koç, Boğa, Yay'ın ikinci yarısı, Oğlak'ın ilk yarısı), insan (İkizler, Başak, Terazi, Yay'ın ilk yarısı, Kova), su (Yengeç, Balık, Oğlak'ın ikinci yarısı), yabanıl (Aslan), böcek (Akrep). Puan 5 × 5 tablodan: {0, ½, 1, 2} |
| Tara | 3 | Gelin nakshatrasından damadınkine sayım mod 9; 3, 5, 7 uygunsuz. Ters yön için aynısı. Her uygun yön 1½ |
| Yoni | 4 | Nakshatranın 14 hayvan simgesinden biri; puan 14 × 14 tablodan: aynı 4, düşman çift 0, ara değerler 1–3 |
| Graha maitri | 5 | Ay burçlarının lordları arasındaki doğal ilişki: aynı lord ya da iki yönlü dost 5; dost + nötr 4; iki yönlü nötr 3; dost + düşman 1; nötr + düşman ½; iki yönlü düşman 0 |
| Gana | 6 | Deva, Manushya, Rakshasa; puan 3 × 3 tablodan |
| Bhakoot | 7 | Ay burçları arası karşılıklı sayım 2/12, 5/9 ya da 6/8 ise 0, değilse 7 |
| Nadi | 8 | Adi, Madhya, Antya; farklıysa 8, aynıysa 0 |

Sabit eşlemeler:

- **Nadi:** nakshatra sırası (Ashwini = 0) mod 6 üzerinden `[Adi, Madhya, Antya, Antya, Madhya, Adi]`.
- **Gana:** Deva — Ashwini, Mrigashira, Punarvasu, Pushya, Hasta, Swati, Anuradha, Shravana, Revati. Manushya — Bharani, Rohini, Ardra, Purva Phalguni, Uttara Phalguni, Purva Ashadha, Uttara Ashadha, Purva Bhadrapada, Uttara Bhadrapada. Rakshasa — öteki dokuz.
- **Yoni:** at — Ashwini, Shatabhisha; fil — Bharani, Revati; koyun — Krittika, Pushya; yılan — Rohini, Mrigashira; köpek — Ardra, Mula; kedi — Punarvasu, Ashlesha; sıçan — Magha, Purva Phalguni; inek — Uttara Phalguni, Uttara Bhadrapada; manda — Hasta, Swati; kaplan — Chitra, Vishakha; geyik — Anuradha, Jyeshtha; maymun — Purva Ashadha, Shravana; firavun faresi — Uttara Ashadha; aslan — Dhanishta, Purva Bhadrapada. Düşman çiftler: at–manda, fil–aslan, koyun–maymun, yılan–firavun faresi, köpek–geyik, kedi–sıçan, inek–kaplan.

**Puan tabloları** (vashya 5 × 5, yoni 14 × 14, gana 3 × 3) `kuta_tables.rs` içinde sabit veridir. Plan'ın ilk görevi bu üç tabloyu iki bağımsız basılı klasik kaynaktan aktarır, kaynaklar ayrışırsa Kuzey Hint panchang geleneğinde yaygın olanı seçer ve farkı dosyada not eder. Tabloların sağlaması gereken değişmezler: köşegen en yüksek puandır; yoni tablosu simetriktir, yedi düşman çift 0'dır, öteki hücreler {1, 2, 3}'tedir; gana tablosunda Manushya–Rakshasa iki yönde de 0'dır.

**Ek olgular** (puana girmez):

- Bhakoot 0 iken iki Ay burcunun lordu aynıysa ya da iki yönlü dostsa `bhakoot_exception: true`. Nadi 0 iken iki Ay aynı nakshatranın farklı padalarındaysa ya da aynı burcun farklı nakshatralarındaysa `nadi_exception: true`. Puan değişmez; istisna ayrı gösterilir.
- Her iki harita için Kuja dosha olgusu: 3.1'deki ortak `kuja_dosha` fonksiyonu (3.5 de aynısını kullanır).

**Karşılıklı yerleşim.** A'nın dokuz grahasının B'nin lagnasından ve Ay'ından ev sayıları ve tersi. Karar: görev tanımındaki "composite" bu karşılıklı yerleşim tablosu ve iki haritalı görünüm olarak yorumlanır; orta nokta bileşik haritası Batı tekniğidir ve klasik Jyotish'te yeri yoktur, uygulanmaz.

**Eş haritanın seçimi.** Eş, danışan veritabanındaki bir haritadır. Seçim açık haritanın `ViewState.compat_partner_uuid` alanında saklanır (harita başına; düzenler kullanıcıya ait olduğu için panel ayarında tutulmaz). Roller `ViewState.compat_roles` alanındadır; ilk atama iki danışanın `gender` alanı farklı ve doluysa ondan yapılır, değilse açık harita "1. kişi (damat rolü)" olur; "rolleri değiştir" düğmesi vardır.

### Veri modeli ve IPC

```rust
pub struct KutaScore { pub kuta: Kuta, pub score: f64, pub max: f64,
                       pub groom_class: String, pub bride_class: String }   // sınıf kimlikleri
pub struct KutaReport { pub scores: [KutaScore; 8], pub total: f64,          // 0..=36
                        pub bhakoot_exception: bool, pub nadi_exception: bool,
                        pub kuja_dosha: [KujaDosha; 2],
                        pub overlay: Option<[OverlayRow; 18]>,
                        pub approximate: bool, pub method: String }
```

Komut: `compute_compat(groom: ChartInput, bride: ChartInput) -> KutaReport`. Arayüz eş haritanın girdisini `chart_input(chart_uuid)` adlı yeni store komutuyla alır (yalnızca çözülmüş an, konum, ayarlar, `time_known`; ad dönmez).

### Arayüz

`compat` paneli. Başlıkta iki kişinin adı ve rolü, eş seçici (palet tarzı danışan/harita araması) ve "rolleri değiştir".

- Sekiz satırlı tablo: kuta, iki tarafın sınıfı, puan / en çok. Altında toplam "n / 36".
- İstisna ve Kuja dosha olguları tablo altında düz metin satırlarıdır.
- Karşılıklı yerleşim: iki sütunlu 9 satırlık tablo.
- `chart` paneline `partner_ring` ayarı eklenir: transit halkasının çizimini kullanarak eşin grahalarını dış halkada gösterir (transit halkasıyla aynı anda açık olamaz).
- Eş seçilmemişse boş durum tek eylem sunar: "Eş harita seç".
- İki haritadan birinin doğum saati bilinmiyorsa amber "yaklaşık" rozeti; Ay o gün nakshatra değiştiriyorsa v1'deki uyarı.

### Test

- **El örnekleri:** her kuta için tüm sınıf çiftlerini kapsayan tablo testleri (varna 16 sınıf çifti, maitri 49 lord çifti, bhakoot 144 burç çifti, nadi 9 sınıf çifti; tara için 729 nakshatra çiftinin tamamı, formülün ikinci bağımsız yazımıyla). Ayrıca 6 tam çift örneği, toplamı adım adım yazılmış.
- **Property:** toplam [0, 36] ve kalem toplamına eşit; nadi, bhakoot, yoni ve maitri rol değişiminde değişmez; aynı Ay (aynı nakshatra, aynı pada) için nadi 0, yoni 4, bhakoot 7, maitri 5; tablolar yukarıdaki değişmezleri sağlar.
- **Store:** `chart_input` ad ve not döndürmez; silinmiş eş haritada panel boş duruma döner.

### Gizlilik, performans, kapsam dışı

- Gizlilik: §14. Eş bağı (`compat_partner_uuid`) iki danışan arasında bir ilişki bilgisidir: JSON dışa aktarımda yalnızca iki harita da aynı dosyadaysa yazılır; bir harita ya da danışan silinince onu gösteren alanlar temizlenir; loglanmaz.
- Performans: ≤ 20 ms (iki temel harita dahil).
- Kapsam dışı: Güney Hint dashakuta sistemi (Rajju, Vedha, Mahendra, Stree Deergha…), dosha iptalleri, dasa uyumu, orta nokta bileşik haritası, uyum yüzdesi ya da sözel hüküm.

---

## 10. Alt proje 3.7 — Muhurta

### Kapsam ve yöntem

**Zaman çizelgesi çekirdeği** (`timeline`): bir tarih aralığı ve yer için aralık dizileri üretir. Her dizi boşluksuz ardışık `[start_jd, end_jd)` aralıklarıdır; sınırlar v1'deki panchang bitiş aramasıyla aynı yöntemle (ikiye bölme) bulunur.

- Tithi, nakshatra (pada ile), nitya yoga, karana, vara (gün doğumu → gün doğumu), Ay burcu, Güneş burcu.
- Lagna burcu (yükselen burcun değiştiği anlar).
- Graha burç girişleri, nakshatra girişleri, retro ve düz dönüş anları (hızın sıfır geçişi) — 3.9'un olay araması da bunları kullanır.

**Gün panchangı.** Seçilen tarih ve yer için gün doğumundan sonraki gün doğumuna:

- Beş uzvun o gün içindeki tüm değerleri ve değişim saatleri.
- Gündüzün sekizde bir dilimleri: gün doğumu → batımı sekiz eşit parça. Varaya göre dilim sırası (Pazar…Cumartesi): Rahu dilimi 8, 2, 7, 5, 6, 4, 3; Yamaganda 5, 4, 3, 2, 1, 7, 6; Gulika 7, 6, 5, 4, 3, 2, 1.
- Abhijit: gündüzün on beş eşit muhurtasının sekizincisi. Çarşamba günü `abhijit_weekday_exception: true`.
- Hora tablosu: 24 saat lordu (3.4'teki kural).

**Sınıflar ve işaretler** (olgu olarak; uygun/uygunsuz hükmü süzgeci kuran astrologundur):

- Tithi grubu: Nanda (1, 6, 11), Bhadra (2, 7, 12), Jaya (3, 8, 13), Rikta (4, 9, 14), Purna (5, 10, 15); her iki pakshada.
- Nakshatra doğası: sabit (Rohini, üç Uttara), yürük (Punarvasu, Swati, Shravana, Dhanishta, Shatabhisha), sert (Bharani, Magha, üç Purva), karışık (Krittika, Vishakha), çevik (Ashwini, Pushya, Hasta), yumuşak (Mrigashira, Chitra, Anuradha, Revati), keskin (Ardra, Ashlesha, Jyeshtha, Mula).
- İşaretler: Vishti karana; Vyatipata ve Vaidhriti nitya yogaları; panchaka (Ay sidereal 300°–360°); Jüpiter ya da Venüs yanık (v1 yanma kuralı).

**Kişiye bağlı etkenler** (bir harita seçiliyse):

- Tara bala: doğum nakshatrasından günün nakshatrasına sayım mod 9 → Janma, Sampat, Vipat, Kshema, Pratyak, Sadhana, Naidhana, Mitra, Parama Mitra; "elverişli" kümesi {2, 4, 6, 8, 9}.
- Chandra bala: geçiş Ay'ının doğum Ay burcundan ev sayısı; "elverişli" kümesi {1, 3, 6, 7, 10, 11}.

**Pencere araması.** Kullanıcının kurduğu süzgece uyan zaman pencerelerini bulur.

- Süzgeç alanları: tarih aralığı (en çok 366 gün), yer, izinli varalar, izinli tithiler, izinli nakshatralar, izinli lagna burçları, "yalnızca gündüz", en kısa süre (dakika), dışlanan işaretler (Rikta, Vishti, Vyatipata/Vaidhriti, Rahu dilimi, Yamaganda, Gulika, panchaka, yanık Jüpiter/Venüs), kişi için "tara bala elverişli" ve "chandra bala elverişli".
- Sonuç: aralık kesişimlerinden çıkan pencereler, her biri o andaki beş uzuv ve lagna burcuyla.
- Karar: hazır etkinlik şablonu (evlilik, yolculuk, eve giriş…) yoktur. Şablonlar klasik muhurta derlemelerinin kural kümeleridir, hüküm içerir ve ayrı bir derleme işidir; v1'de astrolog kendi süzgecini kurar ve adlandırıp saklar.

**Bulunulan yer.** `AppPrefs.home_place: Option<Place>` eklenir (atlas'tan seçilir ya da elle girilir). Gün panchangı ve Prashna bunu varsayılan yer olarak kullanır; boşsa açık haritanın doğum yeri önerilir ve seçim istenir. Konum servisi kullanılmaz.

### Veri modeli ve IPC

```rust
pub struct Interval<T> { pub value: T, pub start_jd_ut: f64, pub end_jd_ut: f64 }
pub struct DaySegments { pub rahu: Interval<()>, pub yamaganda: Interval<()>,
                         pub gulika: Interval<()>, pub abhijit: Interval<()>,
                         pub abhijit_weekday_exception: bool, pub horas: Vec<Interval<Graha>> }
pub struct DayPanchang { pub sunrise_jd_ut: f64, pub sunset_jd_ut: f64, pub next_sunrise_jd_ut: f64,
                         pub vara: Vara, pub tithis: Vec<Interval<u8>>,
                         pub nakshatras: Vec<Interval<(u8, u8)>>,          // (nakshatra, pada)
                         pub yogas: Vec<Interval<u8>>, pub karanas: Vec<Interval<u8>>,
                         pub segments: DaySegments, pub flags: Vec<Interval<MuhurtaFlag>>,
                         pub personal: Option<PersonalFactors>, pub warnings: Vec<ChartWarning> }
pub struct MuhurtaFilter { pub from_jd_ut: f64, pub to_jd_ut: f64, pub place: Place,
                           pub weekdays: Vec<Vara>, pub tithis: Vec<u8>, pub nakshatras: Vec<u8>,
                           pub lagna_signs: Vec<Sign>, pub daytime_only: bool,
                           pub min_minutes: u32, pub exclude: Vec<MuhurtaFlag>,
                           pub natal: Option<NatalMoon>, pub require_tara: bool,
                           pub require_chandra: bool }   // boş küme = kısıt yok
pub struct MuhurtaWindow { pub start_jd_ut: f64, pub end_jd_ut: f64, pub snapshot: WindowSnapshot }
```

Komutlar: `day_panchang(date, place, tz, settings, natal: Option<NatalMoon>) -> DayPanchang`; `muhurta_search(filter, search_id) -> MuhurtaSearchPage` (ay ay sayfalanır; `search_id` yeni aramada değişir, eski arama sayfaları atılır); `muhurta_cancel(search_id)`. `NatalMoon { nakshatra, sign }` yalnızca iki sayı taşır. Kayıtlı süzgeçler `app_state` altında `muhurta_filters` anahtarında JSON olarak durur (kişi verisi içermez; kişi seçimi kaydedilmez).

### Arayüz

| Panel | `kind` | Ayarlar | İçerik |
|---|---|---|---|
| Gün panchangı | `muhurta_day` | yer (bulunulan yer / harita yeri / özel), tarih bağı (transit tarihi / bugün) | Yatay gün şeridi: gün doğumu–batımı, sekiz dilim ve işaretli üçü, Abhijit; altında beş uzvun değişim saatli listesi, işaretler, hora tablosu; harita açıksa tara ve chandra bala. |
| Muhurta araması | `muhurta_search` | etkin süzgeç | Süzgeç formu (görünür etiketli alanlar), "Ara" düğmesi, sonuç listesi: başlangıç, bitiş, süre, uzuvlar. Satıra tıklamak transit tarihini o ana götürür. |

- Gün şeridinde dilimler desen ve yazıyla ayrılır ("Rahu", "Yama", "Guli", "Abhi"); renk tek başına anlam taşımaz.
- Arama 100 ms'yi aşarsa satır içi gösterge, 1 s'yi aşarsa ilerleme (taranan ay / toplam) ve "Durdur".
- Kutup günü ya da gecesinde gündüz dilimleri hesaplanmaz; panel uyarı rozetiyle yalnızca uzuvları gösterir.

### Test

- **Çapraz denetim (asıl güvence):** `timeline` aralıklarının içinden örneklenen anlarda `compute_panchang` aynı tithi, nakshatra, yoga, karana ve varayı verir; aralık sınırı, `compute_panchang`'ın bitiş zamanıyla ≤ 1 sn içinde örtüşür. v1 panchangı golden fixture'larla zaten doğrulanmıştır.
- **El örnekleri:** yedi vara için dilim sıraları; gün doğumu 06:00 ve batımı 18:00 olan sentetik günde dilim saatleri (Pazartesi Rahu dilimi 07:30–09:00 gibi) ve Abhijit 11:36–12:24; tara ve chandra bala için 27 × 27 ve 12 × 12 tabloların ikinci bağımsız yazımı.
- **Property:** aralıklar boşluksuz ve örtüşmesiz; sekiz dilim gündüzü tam böler; Rahu, Yamaganda ve Gulika her varada üç farklı dilimdir; 24 hora günü tam böler ve ilk hora lordu vara lordudur; arama sonucu her pencerenin her anında süzgeci sağlar (pencere içinden rastgele örneklemeyle) ve pencereler en kısa süreden kısa değildir.
- **Astronomik:** bulunan burç girişi anında boylam sınırın ± 1″ içindedir; dönüş anında hız işaret değiştirir.

### Gizlilik, performans, kapsam dışı

- Gizlilik: §14. `home_place` kişisel veridir: loglanmaz, hata parametresine girmez, dışa aktarıma yazılmaz. Ağ isteği yok.
- Performans: `day_panchang` ≤ 50 ms. Arama: lagna süzgeci olmadan 366 gün ≤ 2 s, lagna süzgeciyle ≤ 5 s; ilk sayfa ≤ 500 ms. Arama motor iş parçacığını ay sınırlarında bırakır; araya giren harita hesabı en çok 300 ms bekler.
- Kapsam dışı: etkinlik şablonları, vara–nakshatra birleşim yogaları, otuz muhurtanın adları, choghadiya ve benzeri bölgesel gün bölümlemeleri, tithi–vara–nakshatra dosha tablolarının tamamı, tutulmalar, takvim görünümünde aylık panchang, bildirim/hatırlatma.

---

## 11. Alt proje 3.8 — Prashna

### Kapsam ve yöntem

Soru anı için harita kurma ve o haritanın olgularını gösterme. Hesap Parashari motorunun aynısıdır; ayrı bir Prashna ekolünün (Tajika yogaları, Kerala ashtamangala) kuralları uygulanmaz.

- **Zaman prashnası:** an = şimdi (ya da girilen an), yer = `home_place`. `compute_chart` ile aynı.
- **Sayı prashnası (1–108):** grahalar anın grahalarıdır; lagna sayıdan gelir: `lagna_lon = (n − 1) × 3°20′ + 1°40′` (sidereal; n. navamsanın ortası). Evler bu lagnadan rasi evidir. Çekirdeğe `compute_chart_with_lagna(eph, jd_ut, lat, lon, settings, lagna_sidereal_lon)` eklenir.
- **Odak evi:** astrolog sorunun evini seçer (1–12); modül konu çıkarmaz.

**Olgular** (`PrashnaFacts`):

- Lagna: burç, burç türü (yürük / sabit / çift doğalı), doğuş biçimi — başla doğan (İkizler, Aslan, Başak, Terazi, Akrep, Kova), sırtla doğan (Koç, Boğa, Yengeç, Yay, Oğlak), iki yönlü (Balık).
- Lagna lordu ve odak evinin lordu: burç, ev, dignity, retro, yanma; aralarındaki ilişki (birlikte, karşılıklı bakış, tek yönlü bakış, değişim, hiçbiri) ve birbirinden ev sayıları.
- Ay: burç, ev, nakshatra, tithi ve paksha, aldığı burç temelli bakışlar.
- Kendralardaki ve odak evindeki grahalar, K6 sınıfıyla.
- Anın panchangı (v1) ve 3.5 kuruluysa yoga listesi.

**Saklama.** Prashna haritası danışana bağlı bir `Event` haritasıdır; `charts.kind` değişmez. Yeni tablo:

```
prashna(chart_id PRIMARY KEY REFERENCES charts ON DELETE CASCADE,
        question TEXT NOT NULL DEFAULT '', number INTEGER NULL, focus_house INTEGER NULL,
        created_at)
```

Bir harita, bu tabloda satırı varsa prashnadır. Karar: `kind` CHECK kısıtını değiştirmek tablo yeniden kurmayı gerektirir; yan tablo hem bunu önler hem soru metnini ayrı tutar. Soru metni en çok 2 000 karakterdir.

### Veri modeli ve IPC

Komutlar: `create_prashna(client_id, at: PrashnaMoment, place, number: Option<u8>, question, focus_house) -> OpenedChart`; `update_prashna(chart_id, question, focus_house)`; `compute_prashna_facts(input: ChartInput, number: Option<u8>, focus_house: Option<u8>) -> PrashnaFacts`. `open_client` dönüşüne `prashna: Option<PrashnaMeta>` eklenir.

### Arayüz

- Komut paleti ve kısayol `Cmd+Shift+P`: "Yeni prashna". Sağdan kayan panel: an (varsayılan şimdi), yer (varsayılan bulunulan yer), isteğe bağlı sayı, soru metni, odak evi. "Prashna kur" haritayı oluşturur ve açar. Danışan seçili değilse "Prashna" adlı yerleşik danışan altında toplanır (ilk kullanımda oluşturulur).
- `prashna_facts` paneli: yukarıdaki olgular bölümler hâlinde; odak evi panel başlığından değiştirilir. Soru metni panelin üstünde, düzenlenebilir.
- Sayı prashnasında harita başlığında "sayı: n" rozeti; lagna derecesi "navamsa ortası" notuyla gösterilir.
- Yerleşik düzenlere "Prashna" düzeni eklenir: D1, `prashna_facts`, `panchang`, notlar.

### Test

- **El örnekleri:** sayı → lagna eşlemesi için 1, 9, 10, 54, 108 ve sınır dışı girdiler (0, 109 hata); on iki burcun doğuş biçimi; lord ilişkilerinin beş türü için sentetik konumlar.
- **Property:** sayı prashnasında grahaların boylamı aynı anın zaman prashnasıyla aynıdır, yalnızca lagna ve evler değişir; `compute_chart_with_lagna` gerçek lagna verildiğinde `compute_chart` ile aynı haritayı döner.
- **Store:** harita silinince `prashna` satırı silinir; soru metni dışa aktarımda varsayılan olarak yoktur; gidiş-dönüş (kur → kapat → aç) soru, sayı ve odak evini korur.

### Gizlilik, performans, kapsam dışı

- Gizlilik: §14. Soru metni not gövdesi gibi hassastır: loglanmaz, hata iletisine girmez, JSON dışa aktarımda yalnızca "notları dahil et" işaretliyse yazılır, FTS dizinine eklenmez (karar: `Cmd+K` araması soru metnini açığa vurmasın; gerekirse sonra eklenir).
- Performans: `create_prashna` ≤ 100 ms uçtan uca; `compute_prashna_facts` ≤ 15 ms.
- Kapsam dışı: Tajika bakışları ve yogaları, 1–249 sayı sistemi ve alt lord yöntemleri, ashtamangala, arudha ve chhatra rasi prashnası, nimitta/alamet girişi, kayıp eşya yönü gibi hüküm çıkarımları, evet/hayır sonucu.

---

## 12. Alt proje 3.9 — Arama

`Cmd+K` paletindeki ad, etiket ve not araması (v1) değişmez. Bu alt proje iki yeni arama ekler.

### 12.1 Harita araması (danışan veritabanında astrolojik koşul)

**Ne aranır.** Kayıtlı haritalar, hesaplanan olgular üzerinden. Sorgu, hepsi sağlanması gereken koşulların listesidir (`all_of`); her koşulun değer kümesi kendi içinde "ya da"dır ve her koşul `negate` alabilir. Serbest VE/VEYA ağacı yoktur.

| Koşul | Parametreler |
|---|---|
| `GrahaInSign` | graha, burçlar, varga (varsayılan D1) |
| `GrahaInHouse` | graha, evler |
| `GrahaInNakshatra` | graha, nakshatralar, isteğe bağlı padalar |
| `GrahaState` | graha, durumlar: retro, yanık, vargottama, dignity kümesi |
| `LagnaInSign` | burçlar, varga |
| `Conjunction` | iki ya da daha çok graha aynı burçta |
| `GrahaAspects` | bakan, bakılan (burç temelli tam bakış) |
| `HouseLordInHouse` | hangi evin lordu, hangi evlerde |
| `MoonPhase` | tithiler ya da paksha |
| `DashaAt` | tarih (varsayılan bugün), seviye, lord (Vimshottari) |
| `CharaKaraka` | rol, graha (3.2 kuruluysa) |
| `ArudhaInSign` | pada, burçlar (3.2 kuruluysa) |
| `YogaPresent` | yoga kimlikleri (3.5 kuruluysa) |

Kapsam süzgeci: etiketler, harita türü (`Natal` / `Event`), prashna haritalarını dışla (varsayılan açık).

**Nasıl değerlendirilir.**

- Her harita kendi kayıtlı `resolved_utc`, konum ve kendi `calc_settings`'iyle hesaplanır. Karar: arama, harita açıldığında görülenle aynı sonucu vermelidir; ortak bir ayara zorlamak sessiz fark üretir.
- Sonuç saklanmaz; olgular her aramada canlı hesaplanır (v1 spec §4 "hesaplanmış harita saklanmaz" kuralı korunur). Karar: kalıcı olgu dizini kurulmaz; bayatlama ve göç yükü getirir, bütçe canlı hesapla tutulur.
- Hafif yol: `search_facts(eph, jd, lat, lon, settings, needs)` yalnızca koşulların gerektirdiğini hesaplar (konumlar ve evler her zaman; varga, dasha yolu, karaka, yoga gerektiğinde). Panchang bitiş aramaları ve gün doğumu hesaplanmaz.
- Doğum saati bilinmeyen haritalarda lagnaya bağlı koşul "değerlendirilemedi" sayılır: harita eşleşmez ve sonuç başlığında "saati bilinmediği için değerlendirilemeyen: n" olarak sayılır.
- Hesaplanamayan haritalar (aralık dışı tarih gibi) aramayı durdurmaz; sayıları ayrıca gösterilir.

**Kayıtlı aramalar.** Yeni tablo `saved_searches(id, uuid, name, query_json, created_at, updated_at)`. Sorgu yalnızca koşulları taşır; sonuç listesi ve sayısı saklanmaz.

**Sonuçla yapılabilenler.** Haritayı aç; "sonuçları etiketle" (seçilen etiketi eşleşen haritalara ekler, onay penceresiyle; araştırma koleksiyonu kurmak içindir, v1 spec §5).

### 12.2 Olay araması (zaman içinde gök olayı)

**Ne aranır.** Bir tarih aralığında, 3.7'nin `timeline` çekirdeğiyle:

- Graha burç girişi ve nakshatra girişi.
- Retro ve düz dönüş anları.
- İki grahanın boylamca tam kavuşumu.
- Yeni ay ve dolunay.
- Açık harita varsa: geçiş grahasının doğum grahasının boylamına gelişi (dönüşler dahil) ve doğum Ay burcuna göre Satürn ile Jüpiter'in ev değiştirmeleri.

Aralık sınırı: Ay içeren olaylar için en çok 5 yıl, ötekiler için en çok 120 yıl. Ayanamsa ve zodyak açık haritanın ayarıdır; harita yoksa genel varsayılandır.

### Veri modeli ve IPC

```rust
pub struct ChartQuery { pub all_of: Vec<QueryCondition>, pub scope: QueryScope }
pub struct QueryCondition { pub cond: Condition, pub negate: bool }
pub struct ChartHit { pub client_id: i64, pub chart_id: i64, pub client_name: String,
                      pub chart_label: String, pub matched: Vec<MatchFact> }
pub struct ChartSearchPage { pub search_id: u64, pub hits: Vec<ChartHit>, pub scanned: u32,
                             pub total: u32, pub skipped_time_unknown: u32, pub failed: u32,
                             pub done: bool }
pub struct SkyEvent { pub kind: SkyEventKind, pub jd_ut: f64, pub grahas: Vec<Graha>,
                      pub sign: Option<Sign>, pub nakshatra: Option<u8> }
```

Komutlar: `chart_search(query, search_id, cursor) -> ChartSearchPage` (500 haritalık dilimler; arayüz `done` olana dek ister), `chart_search_cancel(search_id)`, `tag_charts(chart_ids, tag)`, `list_saved_searches`, `save_search`, `delete_saved_search`, `event_search(spec: EventSearchSpec, search_id) -> Vec<SkyEvent>`.

### Arayüz

- **Harita araması** pano paneli değildir (bir haritaya değil veritabanına aittir). Kabukta sağdan kayan tam yükseklikte bir yüzeydir; `Cmd+Shift+F` ya da palet komutu "Haritalarda ara" ile açılır, `Esc` ile kapanır ve odak geri döner.
  - Koşul satırları: tür seçici, parametre alanları (görünür etiketli), "değil" kutusu, satırı sil. "Koşul ekle" düğmesi. Kapsam satırı: etiketler, tür.
  - Kayıtlı aramalar açılır listesi; "Aramayı sakla".
  - Sonuç listesi: danışan adı, harita etiketi, eşleşen olguların kısa özeti ("Ma Koç 10. ev; Ju R"). `Enter` haritayı açar. Başlıkta "n eşleşme / m harita tarandı" ve atlanan sayıları.
  - Tarama sürerken ilerleme ve "Durdur"; kısmi sonuçlar geldikçe listelenir.
  - Boş durum: koşul yokken örnek bir koşul eklemeyi öneren tek eylem; eşleşme yokken "Koşulları gevşet" ipucu ve taranan harita sayısı.
- **Olay araması** bir pano panelidir: `event_search`. Ayarlar: olay türleri, grahalar, aralık. Liste: tarih-saat (haritanın saat diliminde), olay, burç ya da nakshatra. Satıra tıklamak transit tarihini o ana götürür.
- Retro her iki yerde düz `R`; Rahu ve Ketu için retro koşulu sunulmaz.

### Test

- **Çapraz denetim (asıl güvence):** hafif yol `search_facts` ile tam yol (`compute_chart` + modül fonksiyonları) aynı haritalarda her koşul için aynı sonucu verir; proptest rastgele sorgu ve rastgele an üretir.
- **El örnekleri:** her koşul türü için eşleşen ve eşleşmeyen sentetik harita; `negate`; saat bilinmeyen haritada lagna koşulu.
- **Store entegrasyonu:** 1 000 sentetik haritalı veritabanında sayfalama tüm haritaları tam bir kez tarar; arama sırasında silinen harita hata üretmez; iptal sonrası eski `search_id` sayfaları reddedilir.
- **Olay araması:** 3.7'nin astronomik testleri; ayrıca bulunan kavuşum anında iki boylamın farkı ≤ 1″; yeni ay ve dolunayda uzanım 0° ve 180° ± 1″.
- **Performans regresyonu:** CI'da 2 000 haritalık sentetik veritabanında tek koşullu arama eşiği.

### Gizlilik

- Sorgu ve sonuç yerelde durur; ağ isteği yoktur.
- Sonuçlar (adlar, eşleşme sayıları, harita kimlikleri) loglanmaz ve `CommandError.params`'a girmez. Sorgunun kendisi kişi verisi taşımaz ama o da loglanmaz.
- Kayıtlı aramalar yalnızca koşulları saklar; JSON dışa aktarıma dahil edilmez.
- "Sonuçları etiketle" toplu bir yazmadır: etiket adı ve etkilenecek harita sayısı onay penceresinde gösterilir.
- Olay araması açık haritanın doğum boylamlarını yalnızca bellekte kullanır.

### Performans

- Harita araması: konum ve ev koşullarıyla 10 000 harita ≤ 5 s; ilk sayfa (500 harita) ≤ 300 ms; yoga ya da dasha koşulu varsa 10 000 harita ≤ 10 s. Plan'ın ilk görevi tek harita için `search_facts` süresini ölçer; hedef ≤ 0,4 ms. Hedef tutmazsa oturumluk bellek önbelleği (`chart_uuid` + `updated_at` anahtarlı konum dizisi) eklenir; kalıcı dizin yine kurulmaz.
- Olay araması: 120 yıl, yavaş grahalar ≤ 2 s; Ay olayları 5 yıl ≤ 2 s.
- Her iki arama motor iş parçacığını dilim sınırlarında bırakır; araya giren harita hesabı en çok 300 ms bekler.

### Kapsam dışı

Serbest VE/VEYA ağacı, derece aralığı ve orb koşulları, bölümsel haritalarda ev koşulu, Shadbala eşikleriyle arama, iki haritalı (uyum) arama, sonuçların dosyaya dışa aktarımı, tutulmalar, istatistik ve dağılım grafikleri.

---

## 13. Ortak test stratejisi

Doğruluk, başka yazılımın çıktısına dayanmadan dört katmanla sağlanır:

1. **El örnekleri** (`tests/golden/modules/<modül>/*.json`, test dosyası `crates/jyotish-core/tests/modules_golden.rs`). Girdi sentetik `Positions`'tır (ephemeris gerekmez); beklenen değer klasik kuraldan elle türetilir ve fixture'ın `derivation` alanında adım adım yazılır. Şema: `{ id, source: "hand", rule_ref, derivation, input, expected }`. `rule_ref` genel bir göndermedir (ör. "BPHS, pada bölümü"). Bilinmeyen alan reddedilir; modül başına en az sayı test içinde sabitlenir (sessiz silinmeye karşı). `source` olarak yalnızca `hand` kabul edilir.
2. **Değişmezler** (proptest): her alt projenin "Property" maddeleri. Kuralın yapısından gelen ve herhangi bir girdide doğru olması gereken eşitliklerdir.
3. **Bağımsız ikinci yazım:** tablo olarak yazılabilen kurallar (chara dasha yılları, tara, nadi, dilim sıraları, rashi drishti) testte elle girilmiş tabloyla, motorda formülle durur; ikisi her girdide eşit olmalıdır.
4. **Çapraz denetim:** yeni kod, v1'de golden fixture'larla doğrulanmış kodla karşılaştırılır (`timeline` ↔ `compute_panchang`, `search_facts` ↔ `compute_chart`), ve astronomik olgularla (giriş anında boylam, dönüşte hız işareti, karşı konumda cheshta).

Ek kurallar:

- El örneğini yazan ile motoru yazan aynı görev içinde olabilir, ama `derivation` ayrı bir gözden geçirmede kuraldan yeniden hesaplanır (gözden geçirme döngüsünün zorunlu maddesi).
- Bir el örneği motorla uyuşmazsa önce kural metni yeniden okunur; beklenen değer motor çıktısına bakarak düzeltilmez.
- Arayüz: her panel için boş, yükleniyor, hata, dolu ve "saat bilinmiyor" durumları; ortak seçimin yayılması; klavye erişimi (UI spec §6, §8). i18n testi her yeni anahtarın üç dilde bulunduğunu doğrular.
- Komut katmanı Rust entegrasyon testleriyle kapsanır (v1 spec §8).

## 14. Gizlilik (C1) — ortak

- Danışan adları, doğum verisi, not ve soru metinleri, yer bilgisi, dosya yolları ve `CommandError.params` loglanmaz; `no-console.test.ts` durur ve izin listesi büyümez.
- Yeni hata kodlarının parametreleri yalnızca alan adı, sayı ya da kimlik taşır; boylam, tarih, ad taşımaz.
- Hiçbir modül ağ isteği yapmaz. Atlas ve ephemeris yereldir.
- Modül sonuçları veritabanına yazılmaz. Yeni kalıcı veriler ve duyarlılıkları: `advanced` ayarları (düşük), `compat_partner_uuid` ve roller (ilişki bilgisi; §9), `home_place` (kişisel; §10), `prashna.question` (hassas; §11), `saved_searches` ve `muhurta_filters` (kişi verisi içermez).
- Panik iletisi hata parametresine taşınmaz (`task_failed` boş parametreli; Plan 10 D6).

## 15. Performans bütçeleri (Rust tarafı, temel harita hesabı dahil)

| Komut | Bütçe |
|---|---|
| `compute_drishti`, `compute_jaimini`, `compute_chara_dasha`, `compute_yogas` | ≤ 10 ms |
| `compute_prashna_facts` | ≤ 15 ms |
| `compute_compat` | ≤ 20 ms |
| `compute_shadbala` | ≤ 40 ms |
| `day_panchang` | ≤ 50 ms |
| `muhurta_search` (366 gün) | ≤ 2 s; lagna süzgeciyle ≤ 5 s |
| `chart_search` (10 000 harita) | ≤ 5 s; yoga/dasha koşuluyla ≤ 10 s |
| `event_search` | ≤ 2 s |

- `open_client` bütçesi (≤ 50 ms) değişmez; hiçbir modül onun içinde çalışmaz.
- 100 ms altındaki komutlarda yükleniyor göstergesi yoktur; uzun aramalar ilerleme ve iptal sunar (UI spec §5).
- Bütçeler CI'da regresyon eşiği olarak ölçülür (v1'deki `open_client` ölçümüyle aynı düzenek).

## 16. Kapsam dışı (Alt proje 3'ün tümü için)

- Yorum metinleri ve her tür sözel hüküm (Alt proje 6).
- AI yorum paneli (ayrı spec; bu belgenin tiplerini okur).
- Modüllerin baskı düzenleri (K10).
- Windows ve Android.
- Varshaphala ve Tajika, KP alt lord yöntemleri, nadi teknikleri, Yogini ve diğer nakshatra dashaları, adı yasak diziyi içeren çark temelli dasha sistemi.
- Kullanıcı tanımlı kural ya da betik.
- Hesaplanmış sonuçların veritabanında saklanması.

## 17. Riskler ve açık noktalar

- **Varyant seçimleri tartışılabilir.** Shadbala'da yıl/ay lordunun güneş girişinden bulunması, saptavargaja değerleri, özel bakışların ekleme-ve-kesme biçimi; chara dashada padakrama düzeni ve alt dönem sırası; chara karakada varsayılan sekizli şema. Hepsi `method` kimliğiyle görünürdür ve tek fonksiyonda toplanmıştır; beta astrologlarından gelen geri bildirimle ikinci bir yöntem eklenebilir (o zaman K7'ye göre ayar olur).
- **Kuta puan tabloları kaynaktan aktarılacak.** Üç tablo bu belgede hücre hücre yazılı değildir; §9'daki değişmezler ve iki kaynak kuralı planın ilk görevidir. Kaynaklar ayrışırsa seçim dosyada gerekçelendirilir.
- **Ortalama boylam katsayıları** plan aşamasında yayımlanmış tablodan alınacak ve ephemeris ile doğrulanacak (§7).
- **Tek ephemeris iş parçacığı.** Uzun aramalar harita açmayı bekletebilir; dilimleme ve 300 ms sınırı bunu karşılar. Yetmezse ikinci bir `SwissEph` örneği ayrı bir işin konusudur (SE global durumu nedeniyle süreç içinde güvenli değildir; ayrı süreç gerekir).
- **Harita araması büyük veritabanında yavaşlayabilir.** Ölçüm ilk görevdir; oturumluk önbellek yedeği tanımlıdır (§12).
- **El örneği yazanın kuralı yanlış anlaması** motoru da yanlış doğrular. Karşılığı: ayrı gözden geçirmede yeniden türetme (§13) ve değişmezler.
- **Sahibin kararına bırakılanlar** (engelleyici değil; varsayılanlarla ilerlenir): Shadbala yıl/ay lordu yöntemi ve chara karaka varsayılanı beta öncesinde bir astrologla teyit edilmeli; Muhurta etkinlik şablonlarının (§10) sonraki bir sürüme alınıp alınmayacağı.

## Ek A — Adlandırma eşlemeleri

v1 spec §10'daki yasak, klasik terimlerin içinde geçen harf dizisini de kapsar. Aşağıdaki kavramlar belgede, kodda, i18n metinlerinde ve testlerde yalnızca bu adlarla anılır.

| Kavram | Kullanılan ad | Kod kimliği |
|---|---|---|
| Shadbala'nın zamana bağlı üçüncü gücü | Zaman gücü / temporal strength / force temporelle | `temporal`, `TemporalBala` |
| Zaman gücünün gündüz–gece kalemi | Gündüz–gece gücü | `day_night` |
| Gündüzün varaya göre Rahu'ya ait sekizde bir dilimi | Rahu dilimi | `rahu` (`DaySegments`) |
| Değişim yogasının 3. ev lordunu içeren sınıfı | 3. ev lordu değişimi | `exchange.third_lord` |
| Eş karakası | Dara karaka | `KarakaRole::Dara` |
| Yay dakikası ve saniyesinin Sanskritçe alt birim adları | ′ ve ″ işaretleri | — |
| Hareket gücünün sekiz hareket durumu şeması | Uygulanmaz (§7 kapsam dışı) | — |
| Düğüm eksenine dayalı modern yoga | Uygulanmaz (§8) | — |
| Yasak diziyi içeren çark temelli dasha | Uygulanmaz (§16) | — |

Her commit'ten önce depo taraması sürer (Plan 09 C13); bu belge için denetim: `grep -inE 'k(a|ā|aa)la|\bcala\b'` boş dönmelidir.
