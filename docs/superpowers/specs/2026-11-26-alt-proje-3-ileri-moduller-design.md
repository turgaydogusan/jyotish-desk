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
- **K2 — Ortak girdi `ChartInput { jd_ut, lat, lon, settings, time_known }`.** `compute_chart` ile aynı alanlar, artı `time_known`. Doğum saati bilinmiyorsa lagnaya bağlı her çıktı `None` döner; arayüzün gizlemesine güvenilmez. Gerekçe: aynı veriyi AI paneli ve baskı da okuyacak; gizleme tek yerde olmalı. İstisna: verisini v1'den beri `Chart`'tan okuyan paneller (3.1'in Panchang ve Ashtakavarga panelleri). K1 `Chart`'ı dondurduğu için bu ikisinde "saat bilinmiyor" davranışı v1 panellerindeki gibi arayüzde uygulanır (§4).
- **K3 — Saf çekirdek, ince toplayıcı.** Her modülün hesap fonksiyonu ephemeris görmeyen düz bir girdi alır (`Positions` ve `GrahaPoint`; tam tanım §4'tedir ve esas olan odur. `lagna_lon` sidereal boylamdır; `lagna_sign` ve `rasi_sign` rasi çerçevesindedir (K5). `compute_positions(eph, &ChartInput)` ya da `From<&Chart>` ile üretilir; el örnekleri için ephemeris görmeyen `Positions::synthetic` vardır). Zamana bağlı modüller (Shadbala, Muhurta) gereken astronomik değerleri ayrı bir toplayıcı fonksiyonla alır ve saf fonksiyona verir. Gerekçe: el hesabı fixture'ları ephemeris olmadan yazılabilir (§13).
- **K4 — Evler rasi evidir.** Aksi yazmadıkça "ev" lagna burcundan sayılan bütün burç evidir; `CalcSettings.house_system` yalnızca v1'deki bhava tablosunu etkiler. Gerekçe: yoga, arudha, argala ve kuta kuralları klasik kaynaklarda burç sayımıyla tanımlıdır; ev sistemi ayarıyla sessizce değişen yoga listesi hata kaynağıdır.
- **K5 — Rasi çerçevesi `settings.zodiac`'ı izler.** Burca dayalı her kural haritanın rasi çerçevesindeki burçları kullanır (v1'deki dignity ve varga ile aynı); nakshatraya dayalı kurallar sidereal boylamı kullanır. Burç içi derece de rasi çerçevesindedir (`GrahaPoint.deg_in_sign`; chara karaka sırası bunu kullanır).
- **K6 — Doğal iyicil/kötücül sınıfı tek fonksiyondur.** `nature(graha, positions) -> Benefic | Malefic`: Jüpiter ve Venüs iyicil; Güneş, Mars, Satürn, Rahu, Ketu kötücül; Ay büyürken (Güneş'ten uzanım `[0°, 180°)`; tam 180° küçülen yarıdadır, v1 `paksha` sınırıyla aynı) iyicil, küçülürken kötücül; Merkür aynı burçta kötücül varsa ve iyicil yoksa kötücül, değilse iyicil (BPHS, graha nitelikleri bölümü). Modüller kendi listesini tutmaz.
- **K7 — Ayarlar: `CalcSettings` şema sürümü 2.** `advanced: AdvancedSettings` alt yapısı eklenir. Alanlar ve göç 3.1'de tanımlanır; hesapta kullanımları 3.2 ve 3.3'tedir; ayar penceresindeki iki denetim ("İleri modül ayarları" grubu: karaka şeması seçicisi ve eş lord kutusu) 3.2'dedir, çünkü `jaimini_co_lords` arudha hesabında da kullanılır; 3.3 yeni denetim eklemez. Tolerans saklanan kaydın okunmasındadır (`store`): eksik `advanced` anahtarları varsayılanla dolar, bilinmeyen anahtarlar yok sayılır ve kayıtta durur. IPC tipinde alanlar zorunludur (`serde(default)` yoktur); tanınmayan bir seçenek değeri kaydı okunamaz kılar ve ayar genel varsayılana döner (uyarıyla). Sürüm 1 → 2 göçü varsayılanları yazar. Yalnızca en az iki uygulanmış seçeneği olan kararlar ayar olur: `karaka_scheme {Eight, Seven}` ve `jaimini_co_lords: bool`. Tek seçenekli yöntemler ayar değildir; her modül sonucu `method` kimliği taşır (ör. `"chara_dasha.padakrama_v1"`), baskı ve AI paneli hangi yöntemin kullanıldığını buradan söyler. Fark rozeti `advanced` alanlarını da kapsar.
- **K8 — Hata ve uyarı.** Yeni hatalar `CoreError` → `{code, params}` yolunu izler; parametreler danışan verisi taşımaz. Ölümcül olmayan durumlar modül sonucunun `warnings` alanındadır.
- **K9 — Başka yazılım çıktısı fixture olmaz.** Bu modüllerde referans değerler klasik kuraldan elle türetilir (§13). Gerekçe: clean-room; ayrıca bu teknikler yazılımdan yazılıma varyant farkı taşır, başka programla aynı sonucu vermek doğruluk ölçütü değildir.
- **K10 — Baskı düzeni eklenmez.** Dört baskı düzeni (UI spec §7) değişmez. Modül baskıları ayrı bir işin konusudur.
- **K11 — Panel dili.** Klasik terimler üç dilde aynı transliterasyonla yazılır, yanında kısa çeviri etiketi olabilir ("Uccha bala · yücelme gücü"). Etiket bir addır, yorum değildir.
- **K12 — Modül panelleri paletten eklenir ve çıkarılır.** Her modül paneli için komut paletinde "… panelini aç/kapat" komutu vardır (yalnızca harita açıkken). Panel etkin düzenin altına, sola, kendi varsayılan boyutuyla eklenir ve odak ona geçer; aynı komut paneli çıkarır, odak gövdeye düşmez. Hazır düzenlere modül paneli eklenmez; kısayol atanmaz. Yeni panel türü getiren her alt proje panelini `MODULE_PANELS` listesine ekler (3.3 panel türü getirmez: var olan iki dasa paneline `system` ayarını ekler). Gerekçe: v1'de panel ekleme yüzeyi yoktur (yalnızca notlar için `⌘⇧N`); genel bir panel kataloğu beta geri bildirimine bırakılmıştır.

## 3. Alt projeler, sıra ve bağımlılıklar

| Sıra | Alt proje | Plan | Bağımlı olduğu | Ürettiği |
|---|---|---|---|---|
| 1 | 3.1 Ortak temel: bakışlar, ev lordları, modül IPC'si, panchang, ashtakavarga ve bakış panelleri | 14 | v1 | `drishti`, `lordship`, `ChartInput`, üç panel |
| 2 | 3.2 Jaimini çekirdeği: chara karaka, rashi drishti, arudha pada, karakamsha, argala | 15 | 3.1 | `jaimini` modülü, üç yeni panel; `chart`, `drishti` ve `graha_table` panellerine birer ek; ayar penceresinde iki denetim |
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
| Ashtakavarga | `ashtakavarga` | görünüm (SAV / tek graha BAV), biçim (tablo / harita içi) | 12 burç × 8 satır tablo: yedi graha BAV'ı ve SAV satırı. Sütun toplamı SAV satırıdır (Rust'tan gelir); satır toplamı gösterilen hücrelerin toplamıdır (gösterim). "Tek graha BAV" görünümü o grahanın tek satırıdır (katkı dökümü değil). Doğum saati bilinmiyorsa tablo gösterilmez (bindular lagnaya da bağlıdır). |
| Bakışlar | `drishti` | tür (graha / rashi — rashi 3.2 ile gelir), değer (tam bakış işareti / virupa) | Bakan × bakılan matrisi. Tam bakış `●`, kısmi değer sayı olarak; renk yoktur. |

Üç panel de ortak seçimle bağlıdır (UI spec §3.4): bir graha ya da burç vurgulanınca ilgili satır ve sütun `--select` zemin ve yazı ağırlığı alır; "harita içi" kutu ayrıca çerçeveyle seçilir (renk tek başına anlam taşımaz). Bu üç panel retro göstermez; "retro her yerde düz `R`'dir ve Rahu ile Ketu'da gösterilmez" kuralı v1 panellerinde olduğu gibi sürer.

### Test

- Sphuta drishti: tablo sınır noktalarında süreklilik; tam noktalarda beklenen değerler (60° → 15, 90° → 45, 120° → 30, 180° → 60, 210° → 45, 240° → 30, 270° → 15); özel bakışlarla 60. Property: değer her zaman [0, 60].
- Burç bakışı: her graha için hedef burç kümesi el tablosuyla birebir; 7. bakış simetrik.
- Göç: sürüm 1 ayar JSON'u sürüm 2'ye varsayılanlarla taşınır; bilinmeyen `advanced` anahtarı açılışı engellemez.
- Paneller: boş, dolu ve "saat bilinmiyor" durumları (UI spec §8); "yükleniyor" ve "hata" durumları yalnızca komut çağıran Bakışlar panelindedir (Panchang ve Ashtakavarga `Chart`'tan okur). "Yeniden dene" hata kutusunu değiştirirken odak panelde durur.

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
- Derece rasi çerçevesindeki burç içi derecedir (K5). `effective_deg` sıralamada kullanılan değerdir: Rahu için `30 − deg_in_sign`, yani `(0, 30]` aralığındadır (Rahu burcun 0°'sindeyse 30); diğerleri için burç içi derecenin kendisi.
- Eşitlik: sıradaki iki komşunun etkin derece farkı 1″ ya da daha azsa (sınır dahil) iki kayıt da `tie: true` taşır; sıra tam `f64` değeriyle, o da eşitse Güneş…Rahu doğal sırasıyla belirlenir. Karaka birleşmesi kuralları uygulanmaz.
- Karar: varsayılan `Eight`, çünkü BPHS'nin karaka bölümü iki şemayı da verir ve sekizli şema karaka kaybetmez; yedili şemayı kullanan ekoller harita başına ayarla seçer.

**Rashi drishti.** Yürük (chara) burç, kendisine bitişik olan dışında üç sabit burca bakar; sabit (sthira) burç, bitişiği dışında üç yürük burca bakar; çift doğalı (dwiswabhava) burç diğer üç çift doğalıya bakar. "Bitişik": yürük burç için kendinden sonraki sabit burç, sabit burç için kendinden önceki yürük burç. Örnek: Koç → Aslan, Akrep, Kova; Boğa → Yengeç, Terazi, Oğlak; İkizler → Başak, Yay, Balık. Bir burçtaki grahalar o burcun baktığı burçlara ve oradaki grahalara bakar. `graha_rashi_drishti` 27 kayıttır: `Graha::ALL` sırasında dokuz graha (düğümler dahil), her biri için üç hedef burç ve o burçtaki grahalar.

**Sayım yönü.** Jaimini sayımında burçlar iki gruptur: düz sayan Koç, Boğa, İkizler, Terazi, Akrep, Yay; ters sayan Yengeç, Aslan, Başak, Oğlak, Kova, Balık. `counts_forward(sign)` grubu, `directed_count(from, to)` burcun kendi yönündeki kapsayıcı sayımı (1–12) verir. Gruplar burada tanımlanır, çünkü eş lord seçiminin 5. ölçütü onlara muhtaçtır; chara dasha (§6) aynı iki fonksiyonu kullanır.

**Eş lordlar.** `jaimini_co_lords = true` (varsayılan) iken Akrep'in lordları Mars ile Ketu, Kova'nın lordları Satürn ile Rahu'dur ve güçlü olan lord sayılır. `stronger_co_lord(sign, positions)` sırayla şu ölçütleri uygular, ilk ayrım kazanır:

1. Yalnızca biri burcun kendisindeyse, burcun dışındaki lord seçilir. İkisi de burcun içindeyse ya da ikisi de dışındaysa bu ölçüt ayırmaz.
2. Bulunduğu burçta daha çok graha ile birlikte olan. Sayıma yalnızca öteki eş lord girmez; öteki düğüm girer (ör. Akrep için Mars ile Ketu karşılaştırılırken Rahu sayılır).
3. Düğümlerin yücelme ve düşüş burcu yoktur; ölçüt yalnızca graha lordun (Mars ya da Satürn) durumuna bakar: yücelme burcundaysa o, düşüş burcundaysa düğüm seçilir; ikisi de değilse ölçüt ayırmaz.
4. Bulunduğu burcun türü: çift doğalı > sabit > yürük.
5. Söz konusu burçtan lorda, burcun sayım yönünde (yukarıdaki "Sayım yönü") daha uzak olan.
6. Mars (Akrep) ya da Satürn (Kova).

`jaimini_co_lords = false` iken yalnızca Mars ve Satürn kullanılır. Bu fonksiyon arudha ve chara dasha tarafından ortak kullanılır.

**Arudha pada** (BPHS, pada bölümü). `h` evinin burcu `S`, lordu `T` burcunda olsun; `n`, `S`'den `T`'ye kapsayıcı sayım (1–12). Hesaplanan pada `T`'den `n`. burçtur. İstisna: hesaplanan burç `S`'nin kendisi ya da `S`'den 7. burç ise pada, hesaplanan burçtan 10. burçtur (yani sırasıyla `S`'den 10. ve `S`'den 4. burç). On iki pada verilir: A1 (Arudha Lagna, AL) … A12 (Upapada, UL). `ArudhaPada.lord` evin hesapta kullanılan lordudur (Akrep ve Kova evlerinde seçilen eş lord); padanın düştüğü burcun lordu değildir. Graha arudhaları kapsam dışıdır.

**Karakamsha.** Atmakaraka'nın navamsa (D9) burcu. Sonuç iki görünüm taşır: karakamsha burcu rasi haritasına taşınmış (her grahanın D1 burcunun karakamshadan ev sayısı) ve swamsha (her grahanın D9 burcunun karakamshadan ev sayısı). Karakamsha lagnaya bağlı değildir; doğum saati bilinmiyorsa "yaklaşık" işaretlenir (AK değişebilir). İşaret arayüzdedir (v1 panellerindeki gibi): uyarı satırı, Ay'ın derecesinde ve karakamsha burcunda "≈"; graha tablosunun karaka sütununda Ay'ın rolünde "≈". Raporda yaklaşıklık alanı yoktur; AI paketleyicisi yaklaşıklığı girdideki `time_known`'dan okur.

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
    pub graha_rashi_drishti: Vec<SignAspect>,      // 27 kayıt: Graha::ALL sırasında, graha başına üç hedef burç ve oradaki grahalar
    pub arudhas: Option<[ArudhaPada; 12]>,         // saat bilinmiyorsa None
    pub argala: [ArgalaRow; 12],                   // lagna burcundan sıralı; saat bilinmiyorsa Koç'tan
    pub warnings: Vec<ChartWarning>,
}
```

Argala lagnaya bağlı değildir (referans burçtur), bu yüzden doğum saati bilinmese de döner; yalnızca satır sırası değişir. Komut: `compute_jaimini(input: ChartInput) -> JaiminiReport`. `JaiminiReport` AI paneli için sözleşmedir; alan adları değişirse AI paketleyicisi de değişir. `warnings` yalnızca temel haritanın `LowPrecision` uyarısını taşır; ev sistemi ve gün doğumu uyarıları bu modülü ilgilendirmez ve taşınmaz.

### Arayüz

| Panel | `kind` | Ayarlar | İçerik |
|---|---|---|---|
| Jaimini karaka tablosu | `jaimini_karakas` | — | Rol, graha, burç içi derece (Rahu için hem gerçek hem etkin derece), eşitlik işareti `=`; altında karakamsha burcu ve grahaların karakamshadan ev sayıları (D1 / D9). |
| Arudha padalar | `arudha` | — | A1…A12: burç, lord, istisna uygulanmışsa `*` ve dipnot. AL ve UL etiketlidir. |
| Argala | `argala` | referans (lagna / seçili burç) | Dört çift için iki graha listesi ve durum metni ("geçerli", "çekişmeli", "engelli", "yok"). |

- Argala panelinin `reference` ayarı `lagna` (varsayılan) ya da `selected`'dır. "Seçili burç" ortak seçimde sabitlenen öğenin burcudur: burç kendisi; ev, lagna burcundan sayılan burç (lagna yoksa haritanın çizdiği gibi Koç'tan); graha, haritadaki burcu. Panel on iki burç düğmesiyle de seçtirir. Doğum saati bilinmiyorsa lagna referansı tablo yerine bir satır açıklama gösterir; seçili burç referansı çalışır.
- `chart` paneline `arudha_marks` ayarı eklenir (varsayılan kapalı): padalar burçlarına "AL", "A2"… "UL" olarak yazılır. Yalnızca D1'de ve doğum saati biliniyorken yazılır; diğer durumlarda kutu kullanılamaz. Kullanılamayan kutu yerel `disabled` değildir: `aria-disabled` taşır, odaklanır, etkinleştirmek ayarı değiştirmez ve nedeni görünür bir satır olarak `aria-describedby` ile bağlıdır (transit halkası kutusu da böyledir). İşaretler etkileşimsizdir ve ekran okuyucu için ayrı bir metin listesi olarak okunur; baskıya girmez (K10).
- `drishti` paneline `rashi` türü eklenir (`kind` ayarı; dokuz graha × dokuz graha matrisi, virupa değeri yoktur; yalnızca seçili türün komutu çağrılır).
- `graha_table` paneline isteğe bağlı "karaka" sütunu eklenir. Sütun `settings.columns` listesinde değil, ayrı bir `settings.karaka` ayarındadır: baskı tablosu o listeyi kullanır ve sütun baskıya girmez (K10).
- Her tüketici (üç panel, harita işaretleri, rashi türü, karaka sütunu) raporu kendi isteğiyle alır; paylaşılan önbellek yoktur (K1; komut 1 ms'nin altındadır).
- Durumlar yalnızca metinle ayrılır; renk anlam taşımaz.

### Test

- **El örnekleri** (`tests/golden/modules/jaimini/`, kaynak `hand`): en az 8 sentetik konum seti. Her biri karaka sırasını, on iki padayı, karakamshayı ve en az iki referans için argala listelerini kural adımlarıyla birlikte verir. Zorunlu durumlar: Rahu'nun AK olduğu, 1″ içinde eşitlik, lordun kendi evinde, 4., 7. ve 10. evde olduğu arudha istisnaları, eş lord seçiminin altı adımından her birinin belirleyici olduğu durumlar, AK'nın vargottama olduğu durum.
- **Property:** karaka sırası seçilen şemadaki grahaların bir permütasyonudur; rashi drishti simetriktir ve her burç tam üç burca bakar, kendi türüne bakmaz (çift doğalılar hariç); hiçbir pada evin kendi burcuna ya da 7. burcuna düşmez; argala çiftlerinde aynı graha iki tarafta görünmez.
- **Uçtan uca:** mevcut beş `swetest` fixture anında `compute_jaimini` hata vermez ve property'leri sağlar.

### Gizlilik, performans, kapsam dışı

- Gizlilik: §14. Yeni kalıcı veri yok; 3.1'de tanımlanan iki ayar alanı (`karaka_scheme`, `jaimini_co_lords`) kullanılır.
- Performans: ≤ 10 ms.
- Kapsam dışı: graha arudhaları, karaka birleşmesi, Ketu için ters argala, güçle hükmedilen argala, Jaimini rasi güç sıralamasının tamamı, sthira karaka şeması, pada lagnaları dışındaki özel lagnalar (hora, ghati, bhava lagna).

---

## 6. Alt proje 3.3 — Chara dasha

### Kapsam ve yöntem

Burç temelli Jaimini dashası, padakrama düzeni (20. yüzyıl Jaimini yorum geleneğinde en yaygın uygulanan biçim). `method = "chara_dasha.padakrama_v1"`.

- **Burç grupları.** §5 "Sayım yönü"ndeki iki gruptur (`counts_forward`, `directed_count`; 3.2'de tanımlıdır). Düz sayan: Koç, Boğa, İkizler, Terazi, Akrep, Yay. Ters sayan: Yengeç, Aslan, Başak, Oğlak, Kova, Balık.
- **Dasha sırası.** İlk dasha lagna burcudur. Lagnadan 9. burç düz sayan gruptaysa dashalar zodyak sırasıyla, ters sayan gruptaysa ters sırayla ilerler. On iki burç birer kez gelir.
- **Dasha süresi.** Dasha burcu düz sayan gruptaysa burçtan lorduna ileri, değilse geri doğru kapsayıcı sayılır; yıl = sayım − 1. Lord kendi burcundaysa 12 yıl. Akrep ve Kova için lord `jaimini_lord(sign, positions, co_lords)` ile seçilir: `jaimini_co_lords` açıkken `stronger_co_lord`'un seçtiği lord, kapalıyken Mars ve Satürn (K7; `stronger_co_lord` tek başına ayara bakmaz). İki eş lord da burcun içindeyse seçilen lord burcun içindedir ve süre 12 yıldır; bu ayrı bir kural değil, seçimin sonucudur. Yücelme ve düşüş için ±1 yıl düzeltmesi uygulanmaz.
- **İkinci çevrim.** On iki dasha bittikten sonra aynı sırayla ikinci çevrim gelir; her burcun süresi `12 − ilk çevrimdeki yıl`dır, 0 yıllık burçlar atlanır. İkinci çevrimin bir dashası, başlangıcı doğumdan sonraki 120. yıldan önceyse (birikimli tamsayı yıl `< 120`) bütün süresiyle üretilir; tam 120. yılda başlayacak dasha ve sonrakiler üretilmez. İlk çevrimin on iki dashası her zaman üretilir (toplamı 120 yıla ulaşamaz).
- **Alt dönemler.** Her dönem on iki eşit parçaya bölünür. Sıra harita düzeyindeki yönle aynıdır; ilk alt dönem dasha burcundan sonraki burç, sonuncusu dasha burcunun kendisidir. Aynı kural 3. seviyeye de uygulanır. En çok 3 seviye. Sınırlar `ebeveynin başlangıcı + süre × k / 12`'dir; son alt dönemin bitişi ebeveynin bitişine tam eşittir.
- **Başlangıç.** Doğumda ilk dasha tam süresiyle başlar (bakiye hesabı yoktur). Yıl uzunluğu `settings.dasha_year`'dır; dasha sınırları `doğum anı + birikimli tamsayı yıl × dasha_year` ile hesaplanır (boşluk ve örtüşme olmaz).
- Doğum saati bilinmiyorsa chara dasha hesaplanmaz (lagnaya bağlı).

### Veri modeli ve IPC

```rust
pub struct SignDashaPeriod { pub signs: Vec<Sign>, pub level: u8, pub cycle: u8,
                             pub start_jd_ut: f64, pub end_jd_ut: f64, pub has_children: bool }
pub struct CharaDasha { pub forward: bool, pub years: [u8; 12],        // lagnadan dasha sırasıyla
                        pub periods: Vec<SignDashaPeriod>,              // 2 seviye
                        pub method: String }
pub struct SignDashaPath { pub cycle: u8, pub signs: Vec<Sign> }       // bir anın dönem yolu; açık dalın yolu
```

`periods` iki çevrimin 1. ve 2. seviyesini ön-sırada taşır (her dashanın ardından on iki alt dönemi; `cycle` çevrimi ayırır); 3. seviye istenerek alınır. `years` yalnızca ilk çevrimin yıllarıdır. `CharaDasha` `warnings` alanı taşımaz: temel haritanın `LowPrecision` uyarısı harita başlığından okunur (K8'in bu modüldeki karşılığı).

Komutlar:

- `compute_chara_dasha(input: ChartInput) -> Option<CharaDasha>`.
- `chara_dasha_children(input: ChartInput, path: Vec<Sign>, cycle: u8) -> Vec<SignDashaPeriod>`: boş `path` o çevrimin dashalarını, bir burç on iki alt dönemi, iki burç on iki 3. seviye dönemi verir; `cycle` 1 ya da 2'dir.
- `chara_dasha_at(input: ChartInput, jd_ut: f64) -> Option<SignDashaPath>`: anı içeren dönemlerin yolu (çevrim ve üç burç); an üretilen dönemlerin dışındaysa `None`. Şeridin "güncel dönem"i buradan gelir: arayüz tarih karşılaştırmaz.

Geçersiz çevrim, üç burçluk yol ve o çevrimde üretilmemiş dönem `invalid_dasha_input` (`cycle` | `path`), sonlu olmayan an `invalid_dasha_input` (`jd_ut`) hatasıdır; yeni hata kodu yoktur. Doğum saati bilinmiyorsa `compute_chara_dasha` ve `chara_dasha_at` `None`, `chara_dasha_children` boş liste döner (K2); ayar, an ve konum denetimi saat bilinmese de yapılır.

### Arayüz

- `dasha_tree` ve `dasha_strip` panellerinin `system` ayarı `vimshottari | chara` olur. Chara seçiliyken satırlar graha yerine burç kısaltması taşır; ikinci çevrim satırları "2." önekiyle ayrılır.
- Ağaç başlığında yön ("ileri" / "geri") ve on iki burcun yıl dizisi küçük bir satırda gösterilir; ağacın altında yöntem kimliği satırı durur. 3. seviye yalnızca açık yol için istenir ve son seviyedir. Doğum saati bilinmiyorsa ağaç ve şerit yerine tek satır açıklama çıkar; sistem seçicisi yükleme, hata ve "saat bilinmiyor" durumlarında da görünür.
- Şerit: seviye adları Vimshottari şeridiyle aynıdır (maha / antar / pratyantar = 1. / 2. / 3. seviye). "Maha" iki çevrimin dashalarını tek şeritte gösterir; "antar" anın dashasının on iki alt dönemini, "pratyantar" anın alt döneminin 3. seviyesini (istenerek yüklenir) gösterir. Güncel dönem `chara_dasha_at`'in yoludur ve "transit anında" metniyle işaretlenir. An üretilen dönemlerin dışındaysa şerit yerine bir satır ileti çıkar.
- Dönem seçimi transit tarihini v1'deki gibi sürer: yalnızca şeritteki seçim sürer; ağaçta tıklama dalı açar ya da kapatır (v1 ağacı da böyledir; UI spec §3.4 ile fark ayrı bir iştir).
- Şeritte bir dönem seçilince yeni anın yolu (ve 3. seviyesi) istenirken son çizilen şerit yerinde durur (`aria-busy`); iskelet yalnızca gösterilecek eski şerit yokken çizilir. Seçilen düğme odağını yitirmez; odaktaki dönem şeritten çıkarsa odak güncel döneme, şerit yoksa panel çerçevesine geçer (Plan 16 final incelemesinin kararı).
- Açık dal sistem başına ayrı tutulur: Vimshottari yolu `ViewState.dasha_expanded_path`'te durur, chara yolu kardeş alan `chara_expanded_path: { cycle, signs }`'tedir (çevrim 1 ya da 2, en çok 3 burç). Şema sürümü değişmez; ayrıştırma toleranslıdır; bu alanı taşımayan eski kayıt Vimshottari yolu sayılır ve chara yolu boş okunur.

### Test

- **El örnekleri:** en az 6 sentetik harita; her biri yönü, on iki yılı ve ilk iki dashanın alt dönem sırasını kural adımlarıyla verir. Zorunlu durumlar: ileri ve geri yön, lordu kendi burcunda (12 yıl), lordu 2. ve 12. burçta (1 yıl ve 11 yıl iki grup için), Akrep ve Kova'da eş lord seçimi, ikinci çevrimde atlanan burç.
- **Property:** her süre 1–12 yıl; ilk çevrim on iki burcun permütasyonudur ve lagnadan başlar; dönemler boşluksuz ardışıktır; alt dönemlerin toplamı ebeveyne eşittir; son alt dönem ebeveyn burcudur; iki çevrimde aynı burcun yılları toplamı 12'dir.
- **Çapraz denetim:** yılları üreten fonksiyonun yanında, aynı kuralı tablo arayarak (12 burç × 12 lord konumu, elle yazılmış 144 hücre) veren ikinci bir test yardımcısı; ikisi her girdide eşit olmalıdır.
- **El örneği biçimi:** `expected` `forward`, `order`, `years`, `second_cycle` (üretilen dashalar: burç ve yıl) ve `sub_periods` (ilk iki dasha) taşır; doğum saati bilinmeyen örnekte `expected` `null`'dır. Doğum anı ve yıl uzunluğu girdide yoktur: test sabit bir an ve 365,25 günlük yıl kullanır. "Lordu 2. ve 12. burçta" zodyak sırasıyla sonraki ve önceki burçtur: düz sayan burçta 1 ve 11 yıl, ters sayan burçta 11 ve 1 yıl (dört durum).

### Gizlilik, performans, kapsam dışı

- Gizlilik: §14. Kalıcı veri: panel ayarı (`system`) ve haritanın görünüm kaydındaki chara yolu; dasha sonucu saklanmaz.
- Performans: ≤ 10 ms; alt dönemler ≤ 5 ms; `chara_dasha_at` ≤ 10 ms.
- Kapsam dışı: diğer chara dasha varyantları, ±1 yıl düzeltmesi, sthira/shoola/narayana ve diğer burç dashaları, Yogini ve koşullu nakshatra dashaları.

---

## 7. Alt proje 3.4 — Shadbala ve Bhava bala

Kaynak: BPHS'nin graha gücü ve bhava gücü bölümleri; hesap adımlarında 20. yüzyılın yaygın hesap geleneği. Yalnızca yedi graha. Birim virupa; 60 virupa = 1 rupa. `method = "shadbala.bphs_v1"`.

### Yöntem — altı güç

**Çerçeveler (K5).** Burca ve burç içi dereceye dayalı kalemler rasi çerçevesindedir (uccha, saptavargaja, ojayugma, kendradi, drekkana, bhavanın burç türü). Açı farkına dayalı kalemler sidereal boylamlarla hesaplanır ve çerçeveden bağımsızdır (dig, drik, bhava drishti, paksha, graha savaşı). Ayana ve cheshta tropikal boylam kullanır. Grahanın rasi çerçevesindeki boylamı `burç başı + deg_in_sign`'dır. `Positions` lagnanın rasi çerçevesindeki derecesini taşımaz; bhava ortasının derecesi `lagna_lon + (Güneş'in rasi boylamı − Güneş'in sidereal boylamı)` ile bulunur (sidereal kipte fark 0'dır, tropikal-rasi kipinde ayanamsadır).

**1. Sthana bala (konum gücü)** — beş kalemin toplamı:

- *Uccha bala:* grahanın düşüş noktasından açısal uzaklığı (0°–180°) / 3.
- *Saptavargaja bala:* D1, D2, D3, D7, D9, D12, D30 vargalarının her birinde, grahanın bulunduğu burcun lorduyla ilişkisine göre: moolatrikona 45 (yalnızca D1), kendi burcu 30, çok dost 20, dost 15, nötr 10, düşman 4, çok düşman 2. İlişki v1'deki bileşik ilişkidir (doğal + D1'den geçici). Karar: BPHS'nin değerleri kullanılır; 22,5 / 7,5 / 3,75 / 1,875 dizisini veren hesap geleneği uygulanmaz. Kuralların sırası: (1) moolatrikona 45 yalnızca D1'de ve v1'in moolatrikona derece aralığında (`dignity::moolatrikona`); (2) varga burcu grahanın kendi burcuysa 30 (yedi vargada da; D2'de Güneş için Aslan, Ay için Yengeç); (3) değilse varga burcunun lorduyla bileşik ilişki. Geçici ilişki her vargada D1 burçlarından alınır (grahanın D1 burcu ile lordun D1 burcu; aynı burç düşmandır). Yücelme ve düşüş burcu ayrıca sayılmaz (uccha bala onu ölçer).
- *Ojayugma bala:* Ay ve Venüs çift burçta, diğerleri tek burçta 15; aynı kural navamsa burcu için ayrıca 15.
- *Kendradi bala:* kendra (1, 4, 7, 10) 60; panaphara (2, 5, 8, 11) 30; apoklima (3, 6, 9, 12) 15.
- *Drekkana bala:* eril grahalar (Güneş, Mars, Jüpiter) burcun ilk 10°'sinde, nötrler (Merkür, Satürn) ikinci 10°'de, dişiller (Ay, Venüs) son 10°'de 15.

**2. Dig bala (yön gücü):** güçsüz noktadan açısal uzaklık (0°–180°) / 3. Güçsüz nokta: Güneş ve Mars için IC, Ay ve Venüs için MC, Merkür ve Jüpiter için alçalan (lagna + 180°), Satürn için lagna. Karar: lagna ve MC astronomik açılardır (`Ephemeris::houses`); ev sistemi ayarından bağımsızdır.

**3. Zaman gücü** (`temporal`; klasik adı yasak diziyi içerdiği için bu adla anılır, Ek A) — kalemleri:

- *Gündüz–gece gücü* (`day_night`, klasik natonnata): `m` = doğum anının doğum gününün görünür öğlesine uzaklığı, saat; 12 saatten büyükse `24 − m` alınır (ardışık görünür öğleler 24 saat arayla sayılır; gerçek fark 30 saniyenin, yani 0,04 virupanın altındadır). Görünür öğle, o günün gün doğumu ile batımının orta noktasıdır; "gün" doğumdan önceki son gün doğumunda başlar, bu yüzden şafaktan önce doğanlarda uzaklık 12 saati aşar. Güneş, Jüpiter, Venüs: `60 × (1 − m/12)`. Ay, Mars, Satürn: `60 × m/12`. Merkür her zaman 60.
- *Paksha bala:* `e` = Ay'ın Güneş'ten uzanımı, 0°–180°'ye indirgenmiş. Ay: `2 × e/3`. Diğerleri K6 sınıfına göre: iyicil `e/3`, kötücül `(180 − e)/3`.
- *Tribhaga bala:* gündüz (doğuş → batış) üç eşit dilim: Merkür, Güneş, Satürn. Gece (batış → sonraki doğuş) üç eşit dilim: Ay, Venüs, Mars. İçinde bulunulan dilimin lordu 60 alır. Jüpiter her zaman 60 alır. Dilimler yarı açık aralıklardır (`[başlangıç, bitiş)`): tam batış anı gecedir, tam sınırdaki an sonraki dilimdedir; saat dilimlerinde de aynı kural geçerlidir.
- *Yıl, ay, gün ve saat lordu:* yıl lordu 15, ay lordu 30, gün lordu 45, saat lordu 60.
  - Gün lordu: vara lordu (vara gün doğumunda değişir, v1).
  - Saat lordu (hora): gün doğumundan sonraki gün doğumuna 24 eşit dilim; ilk dilim gün lordunundur, sonrakiler Satürn → Jüpiter → Mars → Güneş → Venüs → Merkür → Ay döngüsünde ilerler. Gün 24 saat değilse dilimler de eşit uzar ya da kısalır.
  - Yıl lordu: Güneş'in doğumdan önceki son sidereal Koç girişinin gerçekleştiği varanın lordu. Ay lordu: Güneş'in bulunduğu sidereal burca girişinin gerçekleştiği varanın lordu.
  - Giriş her zodyak kipinde sidereal çerçevededir (ayarın ayanamsasıyla). Giriş anının varası doğum yerinde, v1'in vara kuralıyla ölçülür (o andan önceki son gün doğumunun yerel hafta günü; gün doğumu yoksa gün 06:00 yerel ortalama saatte başlar). Giriş anı, doğum anındaki hızdan ilk tahmin ve sekiz sabit Newton adımıyla bulunur (`sun_entry_before`).
  - Karar: yıl ve ay lordu güneş girişinden bulunur, çünkü ephemeris ile doğrulanabilir ve tek anlamlıdır; 360 günlük yıllarla ahargana sayımı uygulanmaz. Bu, modülün en çok tartışılabilir varyant seçimidir (§17).
- *Ayana bala:* `δ = asin(sin 23°27′ × sin λ)`, `λ` tropikal boylam. Güneş, Mars, Jüpiter, Venüs: `(24 + δ) / 48 × 60`. Ay, Satürn: `(24 − δ) / 48 × 60`. Merkür: `(24 + |δ|) / 48 × 60`. Güneş'inki iki katıdır.
- *Graha savaşı:* Mars, Merkür, Jüpiter, Venüs, Satürn'den ikisi 1° içindeyse `war: Some { with, winner }` işaretlenir; kazanan, ekliptik enlemi daha kuzeyde olandır. Güç aktarımı uygulanmaz (kapsam dışı). Ayrıntı: fark sidereal boylamların açısal uzaklığıdır ve 1° sınırı dahildir; savaşın iki tarafı da işaretlenir. Üç graha 1° içindeyse her grahanın karşısındaki, boylamca en yakın olandır (eşitlikte Mars, Merkür, Jüpiter, Venüs, Satürn sırasında önce gelen). Enlemler eşitse aynı sırada önce gelen kazanır.

**4. Cheshta bala (hareket gücü):**

- Güneş: ayana bala değeri (iki katı alınmamış hâli). Ay: paksha bala değeri (iki katı alınmamış hâli).
- Mars, Jüpiter, Satürn: `ort = (ortalama boylam + gerçek boylam) / 2` (dairesel orta nokta: kısa yayın ortası; tam karşıt iki boylamda yay −180° sayılır); `K = (ortalama Güneş − ort) mod 360`, 180°'den büyükse `360 − K`; güç `K / 3`.
- Merkür, Venüs: aynı formül, "ortalama boylam" yerine ortalama Güneş, "ortalama Güneş" yerine grahanın güneş merkezli ortalama boylamı.
- Ortalama boylamlar J2000 çağlı doğrusal ortalama öğelerden hesaplanır (`L0 + L1 × T`, tropikal, tarihin ekinoksu); gerçek boylam tropikal jeosantrik boylamdır. Katsayılar yayımlanmış tablodandır (J. Meeus, *Astronomical Algorithms*, 2. baskı, Tablo 31.A; yalnızca sabit ve doğrusal terim; `T` UT'den, J2000'den Jülyen yüzyılı): Güneş 280,466457 + 36000,7698278 T (Dünya'nın güneş merkezli ortalama boylamı + 180°); Merkür 252,250906 + 149474,0722491 T; Venüs 181,979801 + 58519,2130302 T; Mars 355,433000 + 19141,6964471 T; Jüpiter 34,351519 + 3036,3027748 T; Satürn 50,077444 + 1223,5110686 T. Katsayılar ölçüme uydurulmaz.
- Doğrulama gerçek motorla, 1900–2100 arasında, gezegen başına sınırla yapılır (gerçek jeosantrik boylam − ortalama boylam farkının ortalaması): Güneş ve Mars ±0,05°; Jüpiter ±0,1°, Satürn ±0,3° (ikisinin ~900 yıllık karşılıklı tedirginliği iki yüzyılda ortalanmaz; ölçüm +0,055° ve −0,212°). `Ephemeris` güneş merkezli konum vermediği için Merkür ve Venüs'ün ortalama boylamı alt kavuşum anlarında Güneş'in karşısıyla karşılaştırılır: Venüs ±0,05°, Merkür ±0,5°.

**5. Naisargika bala (doğal güç):** sabit, 60'ın yedide birinin katları (`60 × k / 7`; Güneş 7 … Satürn 1). İki ondalıkla: Güneş 60,00; Ay 51,43; Venüs 42,86; Jüpiter 34,29; Merkür 25,71; Mars 17,14; Satürn 8,57. Hesapta yuvarlanmamış değer kullanılır.

**6. Drik bala (bakış gücü):** grahaya diğer altı grahadan gelen sphuta drishti değerleri: `(Σ iyicil − Σ kötücül) / 4`, artı Merkür'ün ve Jüpiter'in bakış değerlerinin tamamı. Negatif olabilir. Karar (harfiyen okuma): diğer altı grahanın her biri K6 sınıfına göre `±bakış / 4` verir; ardından Merkür'ün ve Jüpiter'in bakış değeri bir kez daha, tamamıyla ve artı işaretle eklenir (Merkür kötücül sayılsa da). Yani iyicil Merkür ve Jüpiter bakışın 1,25 katını, kötücül sayılan Merkür 0,75 katını verir. Öteki okuma (tamamı çeyreğin *yerine* geçer: 1,00 kat) uygulanmaz; bu bir varyant seçimidir (§17). Okuma tek fonksiyondadır (`drik_bala_at`); bhava drishti aynı fonksiyonu kullanır.

**Toplam ve eşik.** Toplam altı gücün toplamıdır. Klasik gerekli en az değerler (virupa): Güneş 390, Ay 360, Mars 300, Merkür 420, Jüpiter 390, Venüs 330, Satürn 300. Sonuç `total / required` oranını ve yedi graha arasındaki sırayı verir. Sıra toplam virupaya göredir (1 en güçlü); eşit toplamda Güneş, Ay, Mars, Merkür, Jüpiter, Venüs, Satürn sırasında önce gelen öndedir. Oran ayrı sütundur ve sırayı belirlemez. `rupas = total / 60`.

### Yöntem — Bhava bala

Bhava ortası `lagna_lon + 30° × (h − 1)` alınır (karar: eşit bhava; ev sistemi ayarından bağımsız ve tek anlamlı). Bhavanın burcu lagna burcundan sayılan rasi evinin burcudur, derecesi lagnanın rasi çerçevesindeki derecesidir ("Çerçeveler"); lord o burcun v1 lordudur (Akrep: Mars, Kova: Satürn).

- *Bhavadhipati bala:* ev lordunun Shadbala toplamı.
- *Bhava dig bala:* bhava ortasının düştüğü burcun türüne göre güçsüz noktadan açısal uzaklık / 3. İnsan burçları (İkizler, Başak, Terazi, Kova, Yay'ın ilk yarısı): güçsüz nokta 7. bhava ortası. Dört ayaklılar (Koç, Boğa, Aslan, Yay'ın ikinci yarısı, Oğlak'ın ilk yarısı): 4. bhava ortası. Su burçları (Yengeç, Balık, Oğlak'ın ikinci yarısı): 10. bhava ortası. Akrep: 1. bhava ortası. Yay ve Oğlak 15°'de bölünür (ilk yarı `[0°, 15°)`). Bhava ortaları 30° arayla durduğu için değer 0, 10, …, 60'tır.
- *Bhava drishti bala:* drik bala formülü, bakılan nokta bhava ortası; yedi grahanın hepsi sayılır.
- Toplam üçünün toplamıdır; on iki bhava toplama göre sıralanır (eşitlikte küçük numaralı bhava öndedir).

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
pub struct GrahaWar { pub with: Graha, pub winner: Graha }
pub struct TimeLords { pub year: Graha, pub month: Graha, pub weekday: Graha, pub hora: Graha,
                       pub tribhaga: Graha }
pub struct BhavaBala { pub house: u8, pub lord: Graha, pub lord_bala: f64, pub dig: f64,
                       pub drishti: f64, pub total: f64, pub rank: u8 }
pub struct ShadbalaReport { pub grahas: [GrahaShadbala; 7], pub time_lords: TimeLords,
                            pub bhavas: [BhavaBala; 12], pub method: String,
                            pub warnings: Vec<ChartWarning> }
```

Saf fonksiyon `shadbala(positions, astro: &ShadbalaAstro) -> Option<ShadbalaReport>`; `ShadbalaAstro` toplayıcıdan (`compute_shadbala_astro`) gelir, el örnekleri onu elle yazar. Tip IPC'ye çıkmaz ve bilinmeyen alanı reddeder:

```rust
pub struct ShadbalaAstro {
    pub jd_ut: f64,               // doğum anı
    pub sunrise_jd_ut: f64,       // doğumdan önceki son gün doğumu (doğum anı dahil)
    pub sunset_jd_ut: f64,        // o gün doğumundan sonraki ilk gün batımı
    pub next_sunrise_jd_ut: f64,  // sonraki gün doğumu
    pub mc_lon: f64,              // sidereal
    pub tropical_lons: [f64; 7],
    pub mean: MeanLongitudes,     // sun, mercury, venus, mars, jupiter, saturn
    pub weekday: Vara, pub year_entry_weekday: Vara, pub month_entry_weekday: Vara,
}
```

Gündüz üçte birleri doğuş → batış, gece üçte birleri batış → sonraki doğuş, saat dilimleri doğuş → sonraki doğuş arasındadır; önceki günün batışı hiçbir kalemde kullanılmaz. Saf fonksiyon lagna yoksa ya da girdi tutarsızsa `None` döner. Tutarlılık (`ShadbalaAstro::is_consistent`): değerler sonlu; doğuş ≤ doğum < sonraki doğuş; doğuş < batış < sonraki doğuş; iki doğuş arası en çok 1,5 gün (`MAX_SUNRISE_GAP_DAYS`; daha uzun bir aralık "sonraki" doğuşun ertesi günün doğuşu olmadığını gösterir ve dilimler tanımsızdır).

Komut: `compute_shadbala(input: ChartInput) -> Option<ShadbalaReport>`. `None` döner: doğum saati bilinmiyorsa (K2); doğum gününün doğuşu, batışı ya da sonraki doğuşu yoksa (kutup günü ya da gecesi); harita sayı prashnasıysa (`input.lagna_number` dolu: lagna sayıdan gelir, dig balanın istediği meridyen tanımsızdır; §11; sayının kendisi yine denetlenir). Ayar, an ve konum denetimi (`compute_chart`) her durumda yapılır ve hataları aynı kodlarla döner; yeni hata kodu yoktur. `None` uyarı taşıyamaz: "ilgili uyarı" temel haritanın `SunriseUnavailable` uyarısıdır (`Chart.meta.warnings`; harita başlığındaki rozet) ve arayüz nedeni kendi bildiğinden okur (doğum saati bilinmiyorsa onu, biliniyorsa gün doğumunun yokluğunu söyler). Dolu raporun `warnings` alanı temel haritanın yalnızca `LowPrecision` uyarısını taşır. Rahu ve Ketu için kalem fonksiyonları 0 döner; rapor onları içermez.

### Arayüz

| Panel | `kind` | Ayarlar | İçerik |
|---|---|---|---|
| Shadbala | `shadbala` | ayrıntı (özet / kalemler), birim (virupa / rupa) | Satır: yedi graha. Özet sütunları: altı güç, toplam, gerekli, oran, sıra. "Kalemler" görünümünde sthana ve zaman gücünün alt kalemleri açılır. Altında zaman lordları satırı. |
| Bhava bala | `bhava_bala` | birim | On iki bhava: lord, üç kalem, toplam, sıra. |

- Oran 1'in altındaysa hücrede `<1` metin işareti ve ekran okuyucu için "gerekli değerin altında" sözü vardır; altında bir açıklama satırı çıkar; renk kullanılmaz. `ratio < 1` karşılaştırması gösterim eşiğidir (oran Rust'tan gelir). Yatay çubuk bu sürümde çizilmez; çizilirse sayı her zaman yanındadır.
- Graha savaşı satırda `⚔` yerine düz metin "savaş: Ve" ile gösterilir; ekran okuyucuya tam adlar ve kazanan okunur.
- Sayılar `tabular-nums`. Virupa bir ondalık; rupa ve oran iki ondalık (bir ondalık 0,96'yı `1.0 <1` gösterirdi ve 0–10 arasındaki rupa değerlerini ayırt etmezdi); eksi işareti tipografiktir (−). 0,995 ile 1 arasındaki oran "0.99" yazılır, "1.00" değil (işaretle çelişmesin). Kalemlerin rupa gösterimi `virupa / 60`'tır (birim çevirisi, gösterim); toplamın rupası Rust'ın `rupas` alanıdır.
- Shadbala paneli: özet tablosu her zaman görünür; "Kalemler" görünümü altına iki tablo ekler (sthana balanın beş kalemi, zaman gücünün sekiz kalemi; 24 sütunluk tek tablo dar panelde okunmaz). Altında zaman lordları listesi (beş lord, ortak vurgu düğmesi), birim açıklaması ve "Yöntem: shadbala.bphs_v1" satırı durur. Bhava bala paneli aynı raporu kendi isteğiyle okur (rapor oturumda paylaşılmaz).
- Rapor yoksa (`None`) tablo yerine tek satır açıklama çıkar: doğum saati bilinmiyor ya da gün doğumu ya da batımı yok. Araç çubuğu yükleme, hata ve "rapor yok" durumlarında da görünür.
- Tablolar dar panelde yatay kayar; kaydırma kabı klavye odağı alır ve adını tablonun başlığından okur (içinde odaklanabilir denetim olmayan tablolar da klavyeyle kaydırılır).
- Klasik terimler üç dilde aynı yazılır (K11); "zaman gücü" çevrilir (Ek A).

### Test

- **Çalışma kâğıdı** (`tests/golden/modules/shadbala/WORKSHEET.md` ve eşlik eden JSON, kaynak `hand`): en az 3 sentetik girdi için her kalemin aritmetiği adım adım yazılır; kâğıdı motoru yazan dışında bir gözden geçiren doğrular ve kâğıdın başındaki tabloya adını, tarihi ve sonucu yazar. Tolerans 0,01 virupa. JSON girdisi §13 şemasına iki alan ekler: `astro` (`ShadbalaAstro`; zamanlar gün kesridir) ve isteğe bağlı `lats` (ekliptik enlemler); `expected` yedi grahanın 20 kalemini, `rank` ve `war` alanlarını, `time_lords`'u ve on iki bhavayı taşır; doğum saati bilinmeyen örnekte `expected` `null`'dır. Ayrı bir test örneklerin kapsaması gereken durumları sabitler (gündüz, gece ve şafak öncesi doğum; büyüyen ve küçülen Ay; enlemle ve eşit enlemle biten savaş; moolatrikona; negatif drik; Akrep ve Yay lagna; oranın iki yanı).
- **Kalem düzeyinde el örnekleri:** her kalem için en az 4 durum (uç değerler ve bir ara değer). Örnek: yücelme noktasında uccha 60, düşüş noktasında 0; öğlede Güneş'in gündüz–gece gücü 60, gece yarısında 0; dolunayda Ay'ın paksha bala değeri 120.
- **Property:** uccha(λ) + uccha(λ + 180°) = 60; karşıt noktalarda dig bala toplamı 60; gündüz ve gece güçlü grahaların `day_night` toplamı 60; iyicil ve kötücül paksha değerleri toplamı 60; tribhaga'da Jüpiter dışında tam bir graha 60 alır; yıl, ay, gün, saat lordu değerleri {0, 15}, {0, 30}, {0, 45}, {0, 60} kümelerindedir; naisargika sabittir; her kalem kendi aralığındadır; sıra 1–7 permütasyonudur.
- **Astronomik çapraz denetim:** dış gezegen karşı konumdayken cheshta ≥ 55, kavuşumdayken ≤ 5 (ephemeris'ten bulunan gerçek karşı konum ve kavuşum anlarında); iç gezegenlerde alt kavuşumda ≥ 50, üst kavuşumda ≤ 10 (Merkür'ün yörünge dış merkezliği ortalama boylamı gerçek boylamdan 23°'ye kadar ayırır; ölçüm 52,3 ve 8,3); yıl girişi anında Güneş'in sidereal boylamı 0° ± 1″; giriş varası v1 panchangıyla aynıdır.
- **Uçtan uca:** beş `swetest` fixture anında hata yoktur; dördü rapor verir ve property'ler sağlanır, biri (Tromsø 2050-12-21, kutup gecesi) `None` döner ve haritası `SunriseUnavailable` uyarısını taşır. Kutup dairesinin hemen altında ve üstünde, iki gündönümünde gerçek motorla: 65,5° ve 66,5° kuzeyde rapor vardır ve ardışık doğuşlar bir gün arayladır; 67°'de rapor yoktur.

### Gizlilik, performans, kapsam dışı

- Gizlilik: §14. Kalıcı veri yok.
- Performans: ≤ 40 ms (yıl ve ay girişi aramaları dahil).
- Kapsam dışı: graha savaşında güç aktarımı, ishta ve kashta phala, vimshopaka bala, ahargana temelli yıl/ay lordu, sekiz hareket durumuna dayalı cheshta, bhava bala'nın yerleşim ve gündüz/gece ek düzeltmeleri, avasthalar, Rahu ve Ketu gücü.

---

## 8. Alt proje 3.5 — Yogalar

### Kapsam ve yöntem

Veriyle tanımlı kural kataloğu; her kural saf bir yüklemdir. Sonuç bir olgu listesidir: hangi yoga, hangi grahalar, hangi evler. Güç derecelendirmesi ve hüküm yoktur. Yalnızca D1 değerlendirilir. `YOGA_CATALOG_VERSION = 1`.

Ortak tanımlar: "birlikte" = aynı burç; "karşılıklı bakış" = iki grahanın birbirine burç temelli tam bakışı (7. burç her çiftte karşılıklıdır; özel bakışlardan yalnızca Mars'ın 4. ile Satürn'ün 10. bakışı birbirini karşılar); "değişim" (parivartana) = iki farklı grahanın birbirinin kendi burcunda olması; "ilişkili" = bu üçünden biri. Ay ve Güneş yogalarında sayılan grahalar Mars, Merkür, Jüpiter, Venüs, Satürn'dür (Güneş, Ay ve düğümler sayılmaz). Düğümler bakış atmaz, burç lordu değildir ve hiçbir kayıtta yer almaz. Lordlar lagna burcundan sayılan rasi evlerinin lordlarıdır (K4; Akrep: Mars, Kova: Satürn).

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
| Değişim | Maha / üçüncü ev lordu / Dainya | Değişim; sınıfı değişen iki ev (iki grahanın bulunduğu evler) belirler: biri 6., 8. ya da 12. evse Dainya; değilse biri 3. evse "3. ev lordu değişimi"; değilse Maha |
| Viparita raja | Harsha, Sarala, Vimala | Sırasıyla 6., 8., 12. lord 6., 8. ya da 12. evde |
| Neecha bhanga | Neecha bhanga | Graha düşüş burcunda ve şunlardan en az biri: düşüş burcunun lordu lagnadan ya da Ay'dan kendrada; o burçta yücelen graha lagnadan ya da Ay'dan kendrada; graha kendi dispozitörünün bakışını alıyor. Düşüş burcunda yücelen graha yoksa (Akrep) ikinci koşul uygulanmaz. Ay kendisinden sayılmaz: lord ya da yücelen graha Ay ise yalnızca lagna referansı geçerlidir. Sağlanan koşullar listelenir |
| Nabhasa – sayı | Vallaki, Dama, Pasha, Kedara, Shula, Yuga, Gola | Yedi graha sırasıyla 7, 6, 5, 4, 3, 2, 1 burca dağılmış |
| Nabhasa – ashraya | Rajju, Musala, Nala | Yedi grahanın hepsi yürük / sabit / çift doğalı burçlarda |
| Dosha | Kuja dosha | 3.1'deki `kuja_dosha`: Mars lagnadan, Ay'dan ya da Venüs'ten 1, 2, 4, 7, 8 ya da 12. evde; tek kayıt, sağlayan referanslar alt koşuldur (`from_lagna`, `from_moon`, `from_venus`) |

Kararlar:

- Mahapurusha yalnızca lagnadan sayılır (Ay'dan sayan varyant uygulanmaz).
- Sunapha, Anapha ve Durudhara birbirini dışlar; Vesi, Vasi ve Ubhayachari de öyle.
- Bir çift birden çok ilişki taşıyorsa tek kayıt, ilişki listesiyle döner.
- İptal kuralları yalnızca tabloda yazılı olanlardır (Kemadruma, Shakata, Neecha bhanga). Kuja dosha iptalleri uygulanmaz.
- Doğum saati bilinmiyorsa koşulunda lagna zorunlu olan 17 kural değerlendirilmez ve `skipped_time_unknown`'da sayılır: beş mahapurusha, Kemadruma ve Shakata (iptal kuralları lagna ister; iptali denetlenemeyen yoga bildirilmez), üç raja, dhana, üç değişim, üç viparita. Diğer 24 kural lagna seçenekleri olmadan değerlendirilir (Amala ve Vasumati yalnızca Ay'dan; Neecha bhanga'nın kendra koşulları yalnızca Ay'dan; Kuja dosha Ay'dan ve Venüs'ten); her kayıt `approximate = true` taşır ve `houses` boştur. `evaluated + skipped_time_unknown` her zaman 41'dir.
- Katalog 41 satırdır ve tablo sırasındadır; kimlikler `"<grup>.<ad>"` biçimindedir (önekler: `mahapurusha`, `lunar`, `solar`, `raja`, `dhana`, `exchange`, `viparita`, `neecha_bhanga`, `nabhasa_count`, `nabhasa_ashraya`, `dosha`). Kayıtlar katalog sırasıyla, bir kuralın birden çok eşleşmesi graha sırasıyla döner.
- Mahapurusha burç düzeyindedir: kendi burcu `own_sign`, yücelme burcu `exaltation` koşulunu verir, derece ayrımı yoktur (Merkür Başak'ta ikisini de taşır).
- Kemadruma'da "boş" ve "yalnız" sayılan beş grahaya göre okunur (Güneş ve düğümler Ay'ın yanında durabilir); "Ay'dan kendralar" 1., 4., 7. ve 10. evlerdir. Gajakesari'de Ay'la aynı burç kendradır. Adhi'de üç graha aynı evde olabilir. Amala'da "o evde kötücül yok", evdeki her grahanın (düğümler dahil) K6'ya göre iyicil olmasıdır.
- Raja: lagna lordu hem kendra hem trikona lordudur; kendra–trikona kaydı çift başınadır. Dharma-Karmadhipati, 9. ve 10. lord farklı grahalarsa ve ilişkiliyse ek bir kayıttır (aynı çift `raja.kendra_trikona` olarak da döner). Yogakaraka yalnızca lagna burcuna bağlıdır: Boğa ve Terazi'de Satürn, Yengeç ve Aslan'da Mars, Oğlak ve Kova'da Venüs.
- Dhana: çiftin iki grahası da {1, 2, 5, 9, 11} evlerinden en az birinin lordudur, en az biri 2. ya da 11. evin lordudur ve ilişkilidirler; çift başına tek kayıt.
- Viparita raja harfiyen uygulanır (lordun kendi evi dahil). Neecha bhanga'da düşüş burcun tamamıdır; düşüşteki her graha ayrı kayıttır; hiçbir koşul sağlanmıyorsa kayıt yoktur. Sayı ve ashraya yogaları birbirini dışlamaz.
- `grahas` yogayı oluşturan grahalardır (`Graha::ALL` sırasında): Ay ve Güneş yogalarında ışık da listelenir; Neecha bhanga'da yalnızca düşüşteki graha; nabhasa yogalarında yedi graha; Kuja dosha'da Mars. `houses` lagnadan sayılır (artan, yinelemesiz): raja ve dhana kayıtlarında çiftin lordu olduğu ilgili evler; değişimde değişen iki ev; viparita'da lordu olunan ve bulunulan ev; diğerlerinde listelenen grahaların bulunduğu evler.
- `relations` yalnızca çift kurallarında (kendra–trikona, Dharma-Karmadhipati, dhana, değişim) ve iki "birlikte" yogasında (Chandra-Mangala, Budha-Aditya) doludur; değişim kayıtları yalnızca `Exchange` taşır.
- `conditions` sekiz kimlikten oluşur (`YOGA_CONDITIONS`): `own_sign`, `exaltation` (mahapurusha); `from_lagna`, `from_moon`, `from_venus` (Amala, Vasumati, Kuja dosha: tek kayıt, sağlayan referanslar); `sign_lord_in_kendra`, `exalted_graha_in_kendra`, `dispositor_aspect` (Neecha bhanga).
- K7 ve K8 istisnası: yoga raporu `method` ve `warnings` alanı taşımaz. Yöntem kimliği `catalog_version`'dır (panel "Katalog sürümü: 1" satırını gösterir); temel haritanın `LowPrecision` uyarısı rapora taşınmaz (harita başlığındaki rozette görünür). `compute_yogas` her girdide rapor döner (`Option` değildir) ve yeni hata kodu eklemez.
- Tüm grahaların düğüm ekseninin tek yanında toplanmasına dayanan ve klasik kaynaklarda yer almayan modern yoga kataloğa alınmaz.

### Veri modeli ve IPC

```rust
pub enum YogaGroup { Mahapurusha, Lunar, Solar, Raja, Dhana, Exchange, ViparitaRaja,
                     NeechaBhanga, NabhasaCount, NabhasaAshraya, Dosha }
pub enum YogaRelation { Conjunction, MutualAspect, Exchange }
pub struct YogaHit { pub id: String, pub group: YogaGroup, pub grahas: Vec<Graha>,
                     pub houses: Vec<u8>, pub relations: Vec<YogaRelation>,
                     pub conditions: Vec<String>,          // sağlanan alt koşul kimlikleri
                     pub approximate: bool }
pub struct YogaReport { pub catalog_version: u32, pub hits: Vec<YogaHit>,
                        pub evaluated: u32, pub skipped_time_unknown: u32 }
```

`id` kararlı bir kimliktir (`"mahapurusha.hamsa"`); görünen ad i18n anahtarıdır (`yogas.name.<grup>.<ad>`, `yogas.group.<YogaGroup>`, `yogas.condition.<kimlik>`, `yogas.relation.<YogaRelation>`). İlişki tipinin adı `YogaRelation`'dır: v1'in `dignity::Relation` tipi (doğal dostluk) aynı crate kökünden ve aynı TS bağlama dosyasından dışa aktarılır. Komut: `compute_yogas(input: ChartInput) -> YogaReport`.

Katalog (kimlik ve grup) ve koşul kimlikleri arayüze IPC ile verilmez: `ui_fixture.rs` ikisini test fikstürüne yazar (`yoga_catalog`, `yoga_conditions`) ve Vitest üç dildeki adları bu listeden denetler. 3.9'un arama yüzeyi kataloğu çalışma zamanında isterse bir `yoga_catalog` komutu o alt projede tanımlanır; çekirdekte `YOGA_CATALOG`, `yoga_rule(id)` ve `YogaRule::matches` bunun için dışa açıktır.

### Arayüz

`yogas` paneli (7 × 14; paletten eklenir, K12). Ayar: grup süzgeci (`group`: `"all"` ya da bir `YogaGroup`; tanınmayan değer "tümü"dür); süzgeç yeniden hesap istemez ve yükleme ile hata durumlarında da görünür. Gruplara ayrılmış liste: grup başına bir tablo (başlığı grup adı), yalnızca kaydı olan gruplar, raporun sırasıyla; sütunlar Yoga, Grahalar, Evler, İlişki ("birlikte", "karşılıklı bakış", "değişim"), Koşullar; boş hücre "—". Graha kısaltmayla yazılır ve tam adıyla okunur; ev numarası "N. ev" diye okunur. Yaklaşık kayıtlar "≈" işaretini ve ekran okuyucu için "yaklaşık" sözünü taşır. Tablolar klavye odağı alan tek bir kaydırma kabındadır ("Oluşan yogalar"). Altta: doğum saati bilinmiyorsa "lagnaya bağlı N kural değerlendirilmedi" açıklaması; "Değerlendirilen kural: N. Oluşan kayıt: M."; yalnızca D1'in değerlendirildiğini ve evlerin rasi evi olduğunu söyleyen satır; "Katalog sürümü: 1". Arayüzün tanımadığı bir grubun kaydı da listelenir; hiçbir kayıt sessizce atılmaz.

Ortak vurgu: ortak seçim deposu tek hedef taşır (UI spec §3.4), bu yüzden satırın tamamı birden vurgulanmaz. Satırdaki her graha ve her ev ayrı bir vurgu düğmesidir; ters yönde, vurgudaki grahayı ya da evi (burç vurgusunda: o burcun lagnadan sayılan evini; doğum saati bilinmiyorsa burç hiçbir satırı işaretlemez) içeren bütün yoga satırları işaretlenir. Bu eşleme gösterimdir (C4).

Boş durum: süzülen grupta kayıt yoksa "Bu grupta oluşan yoga yok: <grup>". Raporun tamamı boş olamaz (sayı yogalarından tam biri her haritada oluşur); "Katalogdaki yogalardan hiçbiri oluşmuyor" metni ve değerlendirilen kural sayısı yalnızca savunma amaçlı çizilir. Açıklama metni yoktur.

### Test

- **Kural başına el örneği:** her yoga için en az bir oluşan ve bir oluşmayan sentetik konum; iptal kuralı olanlarda ayrıca iptal edilen durum. Fixture kuralın hangi koşulla sağlandığını yazar. Biçim: on beş örnek raporun tamamını yazar (`tests/golden/modules/yogas/`; ikisi doğum saati bilinmeyen); `expected` = `hits` (bütün kayıtlar, katalog sırasıyla), `absent` (örneğin "oluşmayan durum" olarak gösterdiği kurallar), `cancelled`, `evaluated`, `skipped_time_unknown`. "İptal edilen durum": temel koşulu sağlandığı hâlde kendradaki grahayla oluşmayan Kemadruma; Ay'ı lagnadan kendrada olan Shakata; düşüşte olup hiçbir koşulu sağlanmayan graha (kayıt yok). Test, iptal örneğinin temel koşulunu girdiden ayrıca denetler ve 41 kuralın, sekiz koşulun ve üç ilişki türünün örneklerde geçtiğini sabitler. Okuma kuralları ve bağımsız yeniden türetmenin kaydı `yogas/REVIEW.md`'dedir. Tablo olarak yazılabilen kurallar testte elle girilmiş yedi tabloyla durur (karşılıklı bakış, mahapurusha burçları, düşüş burcu, sayı yogası, yogakaraka, değişim sınıfı, lagna isteyen kurallar).
- **Property:** sayı yogalarından tam biri oluşur; kendrada olmayan graha mahapurusha veremez; Durudhara varsa Sunapha ve Anapha ayrı kayıt olarak dönmez; Kemadruma ile Sunapha/Anapha/Durudhara aynı anda oluşmaz; aynı `id` ve aynı graha kümesi iki kez dönmez; her `id` katalogdadır ve üç dilde adı vardır (ad denetimi fikstürdeki katalogdan, Vitest'te).
- **Uçtan uca:** beş `swetest` fixture anında hata yok; beşinde de rapor döner (yogalar gün doğumuna bağlı değildir). v1 haritasıyla çapraz denetim: rapor `yogas(&Positions::from(&chart))` ile aynıdır; beş mahapurusha iki yönlü denetlenir (yoga varsa ve ancak varsa graha v1'de yücelmiş, moolatrikonada ya da kendi burcunda ve rasi kendrasındadır); Neecha bhanga kaydındaki graha `Debilitated`'dir; beş anda en az bir mahapurusha ve bir Neecha bhanga vardır; ev sistemi ayarı raporu değiştirmez (K4).

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
| Vashya | 2 | Beş grup: dört ayaklı (Koç, Boğa, Yay'ın ikinci yarısı, Oğlak'ın ilk yarısı), insan (İkizler, Başak, Terazi, Yay'ın ilk yarısı, Kova), su (Yengeç, Balık, Oğlak'ın ikinci yarısı), yabanıl (Aslan), böcek (Akrep). İlk yarı `[0°, 15°)` aralığıdır; tam 15° ikinci yarıdadır. Puan 5 × 5 tablodan: {0, ½, 1, 2} |
| Tara | 3 | Gelin nakshatrasından damadınkine sayım mod 9; 3, 5, 7 uygunsuz. Ters yön için aynısı. Her uygun yön 1½. Sayım başlangıç nakshatrasını da sayar (aynı nakshatra 1); tara numarası sayımın dokuzlu döngüdeki yeridir (10 → 1, 18 → 9). İki yön birden uygunsuz olamaz (3 ↔ 8, 5 ↔ 6, 7 ↔ 4): Tara en az 1½, toplam en az 2½'dir |
| Yoni | 4 | Nakshatranın 14 hayvan simgesinden biri; puan 14 × 14 tablodan: aynı 4, düşman çift 0, ara değerler 1–3 |
| Graha maitri | 5 | Ay burçlarının lordları arasındaki doğal ilişki: aynı lord ya da iki yönlü dost 5; dost + nötr 4; iki yönlü nötr 3; dost + düşman 1; nötr + düşman ½; iki yönlü düşman 0 |
| Gana | 6 | Deva, Manushya, Rakshasa; puan 3 × 3 tablodan |
| Bhakoot | 7 | Ay burçları arası karşılıklı sayım 2/12, 5/9 ya da 6/8 ise 0, değilse 7 (sayım başlangıç burcunu da sayar: aynı burç 1) |
| Nadi | 8 | Adi, Madhya, Antya; farklıysa 8, aynıysa 0 |

Sabit eşlemeler:

- **Nadi:** nakshatra sırası (Ashwini = 0) mod 6 üzerinden `[Adi, Madhya, Antya, Antya, Madhya, Adi]`.
- **Gana:** Deva — Ashwini, Mrigashira, Punarvasu, Pushya, Hasta, Swati, Anuradha, Shravana, Revati. Manushya — Bharani, Rohini, Ardra, Purva Phalguni, Uttara Phalguni, Purva Ashadha, Uttara Ashadha, Purva Bhadrapada, Uttara Bhadrapada. Rakshasa — öteki dokuz.
- **Yoni:** at — Ashwini, Shatabhisha; fil — Bharani, Revati; koyun — Krittika, Pushya; yılan — Rohini, Mrigashira; köpek — Ardra, Mula; kedi — Punarvasu, Ashlesha; sıçan — Magha, Purva Phalguni; inek — Uttara Phalguni, Uttara Bhadrapada; manda — Hasta, Swati; kaplan — Chitra, Vishakha; geyik — Anuradha, Jyeshtha; maymun — Purva Ashadha, Shravana; firavun faresi — Uttara Ashadha; aslan — Dhanishta, Purva Bhadrapada. Düşman çiftler: at–manda, fil–aslan, koyun–maymun, yılan–firavun faresi, köpek–geyik, kedi–sıçan, inek–kaplan.

**Puan tabloları** (vashya 5 × 5, yoni 14 × 14, gana 3 × 3) `kuta_tables.rs` içinde sabit veridir ve aşağıda hücre hücre yazılıdır (aynı tablolar `tests/golden/modules/kuta/REVIEW.md` içindedir). Satır damat rolünün, sütun gelin rolünün sınıfıdır (`[damat][gelin]`). Tablolar yaygın basılı biçimden (damat sütunda, gelin satırda verilen düzen) özgün olarak aktarılmıştır; yürüten ajanın elinde basılı kaynak yoktur ve başka yazılımın çıktısı ya da çevrimiçi bir tablo kaynak olamaz (K9). **İki bağımsız basılı klasik kaynakla hücre hücre karşılaştırma sahibin onay maddesidir (§17)** ve `REVIEW.md`'deki kayıt tablosuna yazılır; o zamana dek uyum puanları beta astrologlarına "doğrulanmamış tablo" notuyla sunulur. Bilinen ayrışmalar ve seçim (dosyanın başında notludur): vashya bakışımsızdır (yabanıl sınıf gelin rolündeyse diğer dört sınıfla 0; damat rolündeyse dört ayaklıyla ½, suyla 1; bazı derlemeler tabloyu bakışımlı verir); gana'da damat Rakshasa – gelin Deva 1, damat Deva – gelin Rakshasa 0'dır (bazı derlemelerde tersi) ve damat Manushya – gelin Deva 5'tir; yoni bakışımlıdır, ara değerlerde (1, 2, 3) derlemeler arasında tek tük hücre farkı görülür. Bir varyant ancak ikinci seçenek uygulanınca ayar olur (K7); yöntem kimliği `kuta.ashtakuta_v1`'dir.

**Vashya** (sıra: dört ayaklı, insan, su, yabanıl, böcek)

| damat \ gelin | dört ayaklı | insan | su | yabanıl | böcek |
|---|---|---|---|---|---|
| dört ayaklı | 2 | 1 | 1 | 0 | 1 |
| insan | 1 | 2 | ½ | 0 | 1 |
| su | 1 | ½ | 2 | 0 | 1 |
| yabanıl | ½ | 0 | 1 | 2 | 0 |
| böcek | 1 | 1 | 1 | 0 | 2 |

**Gana**

| damat \ gelin | Deva | Manushya | Rakshasa |
|---|---|---|---|
| Deva | 6 | 6 | 0 |
| Manushya | 5 | 6 | 0 |
| Rakshasa | 1 | 0 | 6 |

**Yoni** (bakışımlı; sıra: at, fil, koyun, yılan, köpek, kedi, sıçan, inek, manda, kaplan, geyik,
maymun, firavun faresi, aslan)

| | at | fil | koy | yıl | köp | ked | sıç | ine | man | kap | gey | may | fir | asl |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| at | 4 | 2 | 2 | 3 | 2 | 2 | 2 | 1 | 0 | 1 | 3 | 3 | 2 | 1 |
| fil | 2 | 4 | 3 | 3 | 2 | 2 | 2 | 2 | 3 | 1 | 2 | 3 | 2 | 0 |
| koyun | 2 | 3 | 4 | 2 | 1 | 2 | 1 | 3 | 3 | 1 | 2 | 0 | 3 | 1 |
| yılan | 3 | 3 | 2 | 4 | 2 | 1 | 1 | 1 | 1 | 2 | 2 | 2 | 0 | 2 |
| köpek | 2 | 2 | 1 | 2 | 4 | 2 | 1 | 2 | 2 | 1 | 0 | 2 | 1 | 1 |
| kedi | 2 | 2 | 2 | 1 | 2 | 4 | 0 | 2 | 2 | 1 | 3 | 3 | 2 | 1 |
| sıçan | 2 | 2 | 1 | 1 | 1 | 0 | 4 | 2 | 2 | 2 | 2 | 2 | 1 | 2 |
| inek | 1 | 2 | 3 | 1 | 2 | 2 | 2 | 4 | 3 | 0 | 3 | 2 | 2 | 1 |
| manda | 0 | 3 | 3 | 1 | 2 | 2 | 2 | 3 | 4 | 1 | 2 | 2 | 2 | 1 |
| kaplan | 1 | 1 | 1 | 2 | 1 | 1 | 2 | 0 | 1 | 4 | 1 | 1 | 2 | 1 |
| geyik | 3 | 2 | 2 | 2 | 0 | 3 | 2 | 3 | 2 | 1 | 4 | 2 | 2 | 1 |
| maymun | 3 | 3 | 0 | 2 | 2 | 3 | 2 | 2 | 2 | 1 | 2 | 4 | 3 | 2 |
| firavun faresi | 2 | 2 | 3 | 0 | 1 | 2 | 1 | 2 | 2 | 2 | 2 | 3 | 4 | 2 |
| aslan | 1 | 0 | 1 | 2 | 1 | 1 | 2 | 1 | 1 | 1 | 1 | 2 | 2 | 4 |

Tabloların sağlaması gereken değişmezler: köşegen en yüksek puandır; yoni tablosu simetriktir, yedi düşman çift 0'dır, öteki hücreler {1, 2, 3}'tedir; gana tablosunda Manushya–Rakshasa iki yönde de 0'dır; vashya'da yabanıl dışındaki dört sınıf arasında tablo bakışımlıdır, yalnızca insan–su çifti ½, diğer beş farklı çift 1'dir. Testler vashya ve yoni tablolarını puan sınıfına göre ikinci bir yazımla (yoni: 15 çift 3, 26 çift 1, diğerleri 2), gana tablosunu dokuz hücresiyle sabitler.

**Ek olgular** (puana girmez):

- Bhakoot 0 iken iki Ay burcunun lordu aynıysa ya da iki yönlü dostsa `bhakoot_exception: true`. Nadi 0 iken iki Ay aynı nakshatranın farklı padalarındaysa ya da aynı burcun farklı nakshatralarındaysa `nadi_exception: true`. Aynı nakshatranın aynı padasında nadi istisnası yoktur. Puan değişmez; istisna ayrı gösterilir, iptal ya da telafi önerilmez.
- Her iki harita için Kuja dosha olgusu: 3.1'deki ortak `kuja_dosha` fonksiyonu (3.5 de aynısını kullanır).

**Okuma kararları.**

- Sınıf kimlikleri (`kuta_class_ids`): varna `brahmin`, `kshatriya`, `vaishya`, `shudra`; vashya `quadruped`, `human`, `water`, `wild`, `insect`; yoni on dört hayvanın İngilizce adı (`horse` … `lion`); gana `deva`, `manushya`, `rakshasa`; nadi `adi`, `madhya`, `antya`. Sınıfı tablo olmayan üç kutada "sınıf" şudur: Tara'da tara numarası (`1` … `9`; damat rolününki kendi nakshatrasının gelininkinden sayısıdır, gelin rolününki tersi), graha maitride Ay burcunun lordu (`Sun` … `Saturn`), Bhakoot'ta Ay burcu (`Aries` … `Pisces`). İkili alanların sırası her yerde `[damat, gelin]`'dir.
- Her harita kendi hesap ayarıyla hesaplanır (`chart_input` haritanın kendi ayarını döner): iki harita ayrı ayanamsa ya da zodyak kipinde olabilir; panel farkı bir satırla söyler. Karşılıklı yerleşimin evleri diğer tarafın lagna burcundan ve Ay burcundan sayılan bütün burç evleridir (K4); ev sistemi ayarı raporu etkilemez.
- Doğum saati bilinmeyen haritada (K2) lagnaya bağlı değerler Rust'ta boştur: o haritanın `kuja_dosha.from_lagna` değeri ve o haritanın lagnasından sayılan dokuz yerleşim evi. Ay'dan sayılan on sekiz değer ve diğer yönün lagna değerleri vardır. Rapor `approximate`'tir; Ay'ın o yerel günde (öğle ∓ 12 saat, hesap aralığına kırpılır) nakshatra ya da burç değiştirip değiştirmediği `moon_day_change`'dedir. Doğum saati bilinmeyen haritaların çoğunda Ay o gün nakshatra değiştirir; rapor öğle konumunu ve değişim olgusunu verir, günün iki ucu için ayrı rapor üretmez.

**Karşılıklı yerleşim.** A'nın dokuz grahasının B'nin lagnasından ve Ay'ından ev sayıları ve tersi. Karar: görev tanımındaki "composite" bu karşılıklı yerleşim tablosu ve iki haritalı görünüm olarak yorumlanır; orta nokta bileşik haritası Batı tekniğidir ve klasik Jyotish'te yeri yoktur, uygulanmaz.

**Eş haritanın seçimi.** Eş, danışan veritabanındaki bir haritadır. Seçim açık haritanın `ViewState.compat_partner_uuid` alanında saklanır (harita başına; düzenler kullanıcıya ait olduğu için panel ayarında tutulmaz). Roller `ViewState.compat_roles` alanındadır (`CompatRoles { OpenIsGroom, OpenIsBride }`; eş yokken `OpenIsGroom`). İlk atama: yalnızca açık haritanın danışanı kadın ve eşin danışanı erkekse açık harita gelin rolünü alır; diğer her durumda (cinsiyet boş, aynı ya da "diğer") açık harita "1. kişi (damat rolü)" olur; "rolleri değiştir" düğmesi vardır. `VIEW_STATE_SCHEMA_VERSION` değişmez: iki alanı taşımayan eski kayıt eşsiz okunur. Bağ tek yönlüdür ve haritanındır (A'nın eşi B ise B'nin kaydı değişmez); bir haritanın tek eşi vardır; bağ mevcut görünüm kaydıyla (debounce, `save_view_state`) yazılır.

### Veri modeli ve IPC

```rust
pub enum Kuta { Varna, Vashya, Tara, Yoni, GrahaMaitri, Gana, Bhakoot, Nadi }   // tablo sırası
pub enum KutaRole { Groom, Bride }
pub struct KutaScore { pub kuta: Kuta, pub score: f64, pub max: f64,
                       pub groom_class: String, pub bride_class: String }   // sınıf kimlikleri
pub struct OverlayRow { pub owner: KutaRole, pub graha: Graha,
                        pub sign: Sign, pub deg_in_sign: f64, pub retrograde: bool,   // eş halkasının verisi
                        pub house_from_lagna: Option<u8>,   // diğer tarafın lagnası yoksa None
                        pub house_from_moon: u8 }
pub struct MoonDayChange { pub nakshatra: bool, pub sign: bool }
pub struct KutaReport { pub scores: [KutaScore; 8], pub total: f64,          // 2½..=36
                        pub bhakoot_exception: bool, pub nadi_exception: bool,
                        pub kuja_dosha: [KujaDosha; 2],                      // [damat, gelin]
                        pub overlay: [OverlayRow; 18],                       // önce damat rolünün dokuz grahası
                        pub approximate: bool,
                        pub moon_day_change: [MoonDayChange; 2],             // [damat, gelin]
                        pub method: String }                                 // "kuta.ashtakuta_v1"
pub struct ChartBrief { pub client_name: String, pub client_gender: Option<Gender>,
                        pub chart_label: String, pub chart_kind: ChartKind }
```

`overlay` `Option` değildir: bir haritanın doğum saati bilinmiyorsa yalnızca o haritanın lagnasından sayılan dokuz değer yoktur (`house_from_lagna: None`). K8 istisnası (§8'deki yoga raporuyla aynı): rapor `warnings` alanı taşımaz; ölümcül olmayan durumlar tipli alanlardır (`approximate`, `moon_day_change`) ve temel haritanın `LowPrecision` uyarısı rapora taşınmaz.

Komutlar:

- `compute_compat(groom: ChartInput, bride: ChartInput) -> KutaReport`. Hatalar `compute_chart`'ın kodlarına ve parametrelerine eşlenir, hangi haritadan gelirse gelsin; parametre yalnızca reddedilen girdinin kendi alanıdır, diğer haritanın verisi hataya girmez. Yeni hata kodu yoktur. Rapor veritabanına yazılmaz, oturumda saklanmaz.
- `chart_input(chart_uuid) -> Option<ChartInput>`: yalnızca çözülmüş an, konum, haritanın kendi ayarı ve `time_known`; ad, not, yer adı ve saat dilimi dönmez. Harita yoksa `None` (silinmiş eş). Okuma hiçbir şey yazmaz (ayar göçü yazılmaz).
- `chart_brief(chart_uuid) -> Option<ChartBrief>`: panel başlığındaki ad ve ilk rol ataması için danışanın adı ve cinsiyeti, haritanın etiketi ve türü; doğum verisi dönmez. `chart_input` bilerek ad taşımadığı için ayrı komuttur.
- `chart_uuid` bir kayıt kimliğidir (dosya yolu ya da adres değildir, C2′) ve Rust'ta UUID olarak denetlenir; geçersiz yazım `invalid_input` (`field: chart_uuid`) verir.

### Arayüz

`compat` paneli (7 × 16; paletten eklenir, K12). Başlıkta iki kişinin adı, haritası ve rolü (damat rolü önce), eş seçici (palet tarzı danışan/harita araması) ve "rolleri değiştir".

- Sekiz satırlı tablo: kuta, iki tarafın sınıfı (sütun başlığı rol ve ad), puan / en çok. Altında toplam "n / 36". Yarım puan "1½" diye yazılır. Bhakoot sınıfı burç kısaltmasıyla gösterilir. Puanlar renklendirilmez; uyum yüzdesi, hüküm, eşik ve puanı anlatan metin yoktur.
- İstisna ve Kuja dosha olguları tablo altında düz metin satırlarıdır (adlı bir liste).
- Karşılıklı yerleşim: iki sütunlu 9 satırlık tablo; hücre "lagnadan n · Ay'dan n". Bilinmeyen ev çizgiyle çizilir ve ekran okuyucuya "bilinmiyor" diye okunur. Graha adları ortak vurgu düğmesidir.
- Altta yöntem satırı ("Yöntem: kuta.ashtakuta_v1"). İki haritanın hesap ayarı farklıysa bunu söyleyen bir satır.
- Eş seçici (`shell/Dialog`, WAI-ARIA combobox + listbox): danışan araması (boş aramada son açılanlar); danışanın açık harita dışında tek haritası varsa doğrudan eş olur, birden çoksa ikinci adımda harita seçilir; açık haritanın kendisi sunulmaz.
- Eş seçilmemişse boş durum tek eylem sunar: "Eş harita seç". Eş seçilmeden hiçbir uyum komutu çağrılmaz.
- Bağ varken araç çubuğu ("Eşi değiştir", "Eşi çıkar"; eş okunmuşsa "Rolleri değiştir") her durumda görünür: eş okunurken, okunamazken ve yeniden denenirken de bağ değiştirilebilir ya da çözülebilir. Eş yeniden okunurken son okunan eş başlıkta durur (panel `aria-busy`); okuma hata verirse durmaz (yeniden denemede artık bağlı olmayan bir eşin adı gösterilmez). Rapor yalnızca okunmuş eşle istenir. Eşi çıkarınca odak "Eş harita seç"e, boş durumdan eş seçince "Eşi değiştir"e gider (eş okunamasa da); odak gövdeye düşmez.
- Eş harita silinmişse (`chart_input` ve `chart_brief` boş döner) bağ görünüm modelinden çıkarılır ve panel boş duruma döner; okuma hatası silinmiş eş sayılmaz (bağ durur, "Yeniden dene").
- `chart` paneline `partner_ring` ayarı eklenir: transit halkasının çizimini kullanarak eşin grahalarını dış halkada gösterir (transit halkasıyla aynı anda açık olamaz; kayıtta ikisi de açıksa transit halkası çizilir). Konumlar uyum raporunun `overlay` satırlarındandır (burç, derece, retro). Düzen kullanıcınındır, eş haritanındır: kutu yalnızca açık haritanın bir eşi varken gösterilir; eşi olmayan haritada ayar açık olsa da hiçbir şey istenmez ve çizilmez. Yalnızca D1'de çizilir; başka vargada kutu odaklanır ve nedenini söyler. Eşin grahaları ekran okuyucu için sözcükle listelenir.
- İki haritadan birinin doğum saati bilinmiyorsa "≈ yaklaşık" rozeti (kişinin adında ve raporda); o kişinin Ay'ı o gün nakshatra ya da burç değiştiriyorsa (`moon_day_change`) kişinin adıyla uyarı satırı (kenar çizgisi ve metin; renk tek başına anlam taşımaz).

### Test

- **El örnekleri:** her kuta için tüm sınıf çiftlerini kapsayan tablo testleri (varna 16 sınıf çifti, maitri 49 lord çifti, bhakoot 144 burç çifti, nadi 9 sınıf çifti; tara için 729 nakshatra çiftinin tamamı, formülün ikinci bağımsız yazımıyla). Ayrıca raporun tamamını yazan sekiz tam çift örneği, toplamı adım adım yazılmış (`tests/golden/modules/kuta/`; §13 şemasına `input.bride` eklenir: üst düzey damat rolünün, `bride` gelin rolünün haritasıdır; `expected` = sekiz puan ve sınıf çifti, toplam, iki istisna, iki Kuja dosha olgusu, on sekiz yerleşim satırı, `approximate`). Bir kapsam testi örneklerin şunları içerdiğini denetler: her kutanın en çok ve en düşük puanı, yarım puanlar, toplamın iki ucu (36 ve 2½), iki istisna (var; puan 0 iken yok; nadi istisnasının iki yolu), aynı Ay, bölünen iki burcun dört yarısı, doğum saati bilinmeyen haritanın iki rolü ve Kuja dosha'nın üç referansı. Okuma kuralları, üç tablo, basılı kaynak karşılaştırma kaydı ve bağımsız yeniden türetme kaydı `REVIEW.md`'dedir.
- **Property:** toplam [0, 36] (gerçek alt sınır 2½; Tara en az 1½) ve kalem toplamına eşit; nadi, bhakoot, yoni ve maitri rol değişiminde değişmez; aynı Ay (aynı nakshatra, aynı pada) için nadi 0, yoni 4, bhakoot 7, maitri 5; tablolar yukarıdaki değişmezleri sağlar.
- **Store:** `chart_input` ad ve not döndürmez; silinmiş eş haritada panel boş duruma döner; bağ yalnızca var olan başka bir haritaya saklanır, silmede temizlenir ve geç gelen kayıtla geri gelmez; dışa ve içe aktarım kuralları (aşağıda).
- **Komut katmanı:** beş golden anının 25 çiftinde v1 haritalarıyla çapraz denetim; ev sistemi ayarından bağımsızlık; hata kodları ve parametreleri (yalnızca reddedilen girdinin alanı); bütçe.

### Gizlilik, performans, kapsam dışı

- Gizlilik: §14. Eş bağı (`compat_partner_uuid`) iki danışan arasında bir ilişki bilgisidir: JSON dışa aktarımda yalnızca iki harita da aynı dosyadaysa yazılır; bir harita ya da danışan silinince onu gösteren alanlar aynı işlemde temizlenir; loglanmaz ve hata parametresine girmez. Kayıt ve okuma yalnızca var olan başka bir haritayı gösteren bağı tutar (kendisini ya da silinmiş bir haritayı gösteren bağ sessizce bırakılır): arayüzün gecikmeli görünüm kaydı silinmiş bir eşi geri yazamaz. İçe aktarım yalnızca eşi aynı dosyada ya da veritabanında olan bağı alır; kendisini gösteren ya da UUID olmayan değer alınmaz. Veritabanındaki bir haritaya bağ, yalnızca o haritanın 122 bitlik rastgele UUID'sini bilen (yani bu veritabanının bir dışa aktarımını görmüş) elle düzenlenmiş bir dosyayla kurulabilir; bağ veritabanının dışına hiçbir veri çıkarmaz ve kullanıcı onu panelden çözer. "Yalnızca dosyadaki eş" kuralı bunu önlemez (aynı dosyaya o UUID'yi taşıyan bir kayıt yazılabilir), bu yüzden kural sıkılaştırılmamıştır. `KutaReport` saklanmaz. Elle denetimde yalnızca deneme danışanları kullanılır.
- Performans: ≤ 20 ms (iki temel harita dahil).
- Kapsam dışı: Güney Hint dashakuta sistemi (Rajju, Vedha, Mahendra, Stree Deergha…), dosha iptalleri, dasa uyumu, orta nokta bileşik haritası, uyum yüzdesi ya da sözel hüküm.

---

## 10. Alt proje 3.7 — Muhurta

### Kapsam ve yöntem

**Zaman çizelgesi çekirdeği** (`timeline`): bir zaman aralığı ve yer için aralık dizileri üretir. Her dizi boşluksuz ardışık `[start_jd, end_jd)` aralıklarıdır ve verilen aralığa kırpılır (ilk aralık aralığın başında başlar, sonuncusu sonunda biter). `muhurta` modülü bunun üstünde durur; 3.9'un olay araması yalnızca `timeline`'ı kullanır.

- **Yöntem.** Bütün diziler tek bir yürüyücüden gelir: ayrık değer sabit adımlarla örneklenir; iki örnek arasında değer değişmişse değişim anı ikiye bölmeyle bulunur (çözünürlük 0,05 sn); bir adımın içindeki birden çok değişim sırayla bulunur. Sınır, yeni değerin geçerli olduğu ilk andır. Değer v1 panchangının fonksiyonlarıyla aynı sorgulardan hesaplanır; böylece aralığın içinde `compute_panchang` ile aynı değer yapıdan gelir. v1'in bitiş araması tekdüze artan açı ister ve retro girişlerde ve lagnada kullanılamaz; ayrık değerin yürüyüşü tekdüzelik istemez. Adımlar: Ay'a bağlı diziler 0,5 gün, graha dizileri 1 gün, lagna 30 dakika, gün doğumu olmayan yerde vara 6 saat. **Sınır:** bir adımın içinde değişip aynı değere dönen kısa gidiş (dönüş anının tam burç sınırına düşmesi; kutup dairesinin içinde lagnanın anlık sıçraması) görülmez.
- Tithi, karana, nakshatra (pada ile), nitya yoga, vara, Ay burcu, Güneş burcu. Tithi dizisi karana dizisinden türetilir (ardışık iki karana bir tithidir): iki dizi aynı sınır anlarını paylaşır.
- Vara `compute_panchang`'ın kuralıdır: gün doğumunda değişir; gün doğumu yoksa gün 06:00 yerel ortalama saatte başlar. Olağan enlemde dizi doğuş anlarından kurulur; doğuşlar aralığı örtmüyorsa v1'in fonksiyonu yürünür. İki yol her anda aynı değeri verir.
- Lagna burcu (yükselen burcun değiştiği anlar).
- Graha dizileri (olay ayrı bir tip değildir: bir giriş ya da dönüş, iki aralığın sınırıdır): rasi çerçevesinde burç ve sidereal nakshatra dokuz graha için; hareket yönü (`Motion { Direct, Retrograde }`) yalnızca Mars, Merkür, Jüpiter, Venüs ve Satürn için (Güneş ve Ay geri gitmez; düğümlerin yön değişimi olay sayılmaz); yanma (v1 kuralı, haritanın orbları).
- Çerçeveler (K5): burçlar ve chandra bala rasi çerçevesindedir; tithi, karana, nakshatra, pada, nitya yoga, panchaka ve tara bala her kipte sidereal boylamdandır.

**Gün panchangı.** Gün, verilen anı içeren gündür (gün doğumundan sonraki gün doğumuna); gün doğumundan önceki an bir önceki güne aittir (vara ile aynı kural). Bir takvim günü ve saat dilimi hangi gün doğumunun kastedildiğini tek başına söylemez; girdi bu yüzden bir andır.

- Beş uzvun o gün içindeki tüm değerleri ve değişim saatleri, sınıflarıyla. Günün dışına taşan uç günün başı ya da sonudur; arayüz bunu işaretler ("günün başından önce başlar", "günün sonundan sonra sürer").
- Gündüzün sekizde bir dilimleri: gerçek gün doğumu → batımı sekiz eşit parça. Varaya göre dilim sırası (Pazar…Cumartesi): Rahu dilimi 8, 2, 7, 5, 6, 4, 3; Yamaganda 5, 4, 3, 2, 1, 7, 6; Gulika 7, 6, 5, 4, 3, 2, 1.
- Abhijit: gündüzün on beş eşit muhurtasının sekizincisi. Çarşamba günü `abhijit_weekday_exception: true`'dur; dilim yine verilir.
- Hora tablosu: gün doğumundan sonraki gün doğumuna 24 eşit dilim; ilk lord vara lordudur, sonrakiler 3.4'teki kuralla.
- **Gün doğumu olmayan gün** (kutup gecesi ya da gündüzü): gün 06:00 yerel ortalama saatte başlar ve 24 saat sürer; gün doğumu alanları ve dilimler boştur, `warnings` `SunriseUnavailable` taşır; uzuvlar, işaretler ve kişiye bağlı etkenler yine verilir.

**Sınıflar ve işaretler** (olgu olarak; uygun/uygunsuz hükmü süzgeci kuran astrologundur):

- Tithi grubu: Nanda (1, 6, 11), Bhadra (2, 7, 12), Jaya (3, 8, 13), Rikta (4, 9, 14), Purna (5, 10, 15); her iki pakshada (Purnima ve Amavasya Purna'dır).
- Nakshatra doğası: sabit (Rohini, üç Uttara), yürük (Punarvasu, Swati, Shravana, Dhanishta, Shatabhisha), sert (Bharani, Magha, üç Purva), karışık (Krittika, Vishakha), çevik (Ashwini, Pushya, Hasta), yumuşak (Mrigashira, Chitra, Anuradha, Revati), keskin (Ardra, Ashlesha, Jyeshtha, Mula).
- İşaretler tek bir enum'dur ve ikili kalemler ayrı değerlerdir (süzgeç birini dışlayıp diğerini bırakabilir): `MuhurtaFlag { Rikta, Vishti, Vyatipata, Vaidhriti, RahuSegment, Yamaganda, Gulika, Panchaka, JupiterCombust, VenusCombust }`. Rikta tithi grubundan, Vishti karananın adından, panchaka Ay'ın sidereal 300°–360° aralığından, yanma v1 kuralından okunur. Gün panchangının `flags` listesi yedisini aralık olarak verir (üç gündüz dilimi `segments` içindedir); aramanın `exclude` listesi onunu da alır.
- Bu tablolar ve kümeler muhurta derlemelerinin yaygın biçimidir ve bellekten aktarılmıştır; basılı kaynakla karşılaştırma sahibin işidir (§17; `tests/golden/modules/muhurta/REVIEW.md`).

**Kişiye bağlı etkenler** (açık haritanın Ay'ına göre; gün içinde değişebildikleri için aralık dizisidir):

- Tara bala: doğum nakshatrasından günün nakshatrasına sayım (başlangıç da sayılır) mod 9 → Janma, Sampat, Vipat, Kshema, Pratyak, Sadhana, Naidhana, Mitra, Parama Mitra; "elverişli" kümesi {2, 4, 6, 8, 9}. Janma bu kümede değildir.
- Chandra bala: geçiş Ay'ının rasi burcunun doğum Ay burcundan sayısı; "elverişli" kümesi {1, 3, 6, 7, 10, 11}.
- "Elverişli" bu iki kümenin adıdır, hüküm değildir.
- **Doğum saati bilinmiyorsa** (K2): muhurta doğum lagnasını kullanmaz ve `None` dönen alan yoktur. Doğum Ay'ı öğle konumundadır; arayüz kişiye bağlı tabloyu ve iki kişi koşulunu "≈" ve açıklamasıyla işaretler.

**Pencere araması.** Kullanıcının kurduğu süzgece uyan zaman pencerelerini bulur.

- Koşullar (`MuhurtaCriteria`, saklanabilir; yer, tarih ve kişi taşımaz): izinli varalar, izinli tithiler (1..=30, paksha ayrımıyla), izinli nakshatralar, izinli lagna burçları, "yalnızca gündüz", en kısa süre (dakika), dışlanan işaretler, "tara bala elverişli" ve "chandra bala elverişli". Boş küme kısıt değildir. Arama girdisi (`MuhurtaFilter`) koşullara aralığı (en çok 366 gün), enlem ve boylamı ve doğum Ay'ını ekler.
- Sonuç: aralık kesişimlerinden çıkan pencereler; her birinin özeti (`WindowSnapshot`: beş uzuv ve lagna burcu) pencerenin başlangıç anının noktasal hesabıdır.
- **Durumsuz, imleçli sayfalama.** Bir çağrı 31 günlük tek bir dilimi hesaplar ve sonraki dilimin imlecini döner; aynı süzgeç ve imleç her zaman aynı sayfayı verir. Rust'ta arama durumu, `search_id` ve iptal komutu yoktur: durdurmak, sonraki dilimi istememektir; eski aramanın geç gelen sayfasını arayüz düşürür. Dilimler takvim ayı değil 31 gündür.
- Dilim sınırını aşan pencere bölünmez: başı imleçte taşınır ve pencere kapandığı dilimde tam uzunluğuyla verilir; "en kısa süre" birleşmiş pencereye uygulanır. 1 saniyeden kısa pencere hiç verilmez (aynı ana düşen iki ayrı dizinin sınırları çözünürlük kadar ayrışır ve kıymık pencere üretirdi). Dilim uzunluğu sonucu değiştirmez; bu, sınırlarda 1 saniye içinde geçerlidir.
- **Gün doğumu ya da batımı olmayan günlerde:** "yalnızca gündüz" o günlerde pencere vermez; üç gündüz dilimi dışlaması uygulanmaz; iki durumda da sayfa `SunriseUnavailable` uyarısı taşır. Vara koşulu yukarıdaki kuralla çalışır.
- Karar: hazır etkinlik şablonu (evlilik, yolculuk, eve giriş…) yoktur. Şablonlar klasik muhurta derlemelerinin kural kümeleridir, hüküm içerir ve ayrı bir derleme işidir; v1'de astrolog kendi süzgecini kurar ve adlandırıp saklar.

**Bulunulan yer.** `AppPrefs.home_place: Option<HomePlace>` eklenir; `HomePlace { place: Place, tz: TzChoice }` (o yerin saatleri gösterileceği için saat dilimi de saklanır). `APP_PREFS_SCHEMA_VERSION` değişmez: alanı taşımayan eski kayıt yersiz okunur. Okuma toleranslıdır (geçersiz yer "seçilmemiş"tir); yazım koordinat aralığını, ad uzunluğunu ve saat dilimini denetler ve `invalid_input` alan `home_place` döner. Tercih kaydı bütün kaydı değiştirir: arayüz her yazımda okuduğu kaydı değiştirip yazar. Gün panchangı, muhurta araması ve Prashna bunu varsayılan yer olarak kullanır. Seçilmemişse panel yer ister ve hiçbir komut çağırmaz; açık haritanın doğum yeri önerilir ama sessizce kullanılmaz. Yer atlastan aranır ya da açık haritanın doğum yerinden alınır; ayrı bir elle koordinat formu yoktur (elle girilmiş koordinat, o koordinatla kurulmuş bir haritanın doğum yeri olarak alınır). Konum servisi kullanılmaz.

### Veri modeli ve IPC

```rust
pub struct Interval<T> { pub value: T, pub start_jd_ut: f64, pub end_jd_ut: f64 }
pub struct DaySegments { pub eighths: Vec<Interval<u8>>,                   // sekiz dilim, değer 1..=8
                         pub rahu: Interval<u8>, pub yamaganda: Interval<u8>, pub gulika: Interval<u8>,
                         pub abhijit: Interval<u8>,                        // değer 8: muhurtanın sırası
                         pub abhijit_weekday_exception: bool, pub horas: Vec<Interval<Graha>> }
pub struct DayTithi { pub no: u8, pub paksha: Paksha, pub group: TithiGroup }
pub struct DayNakshatra { pub no: u8, pub pada: u8, pub nature: NakshatraNature }
pub struct DayKarana { pub no: u8, pub name: KaranaName }
pub struct TaraBala { pub number: u8, pub favourable: bool }
pub struct ChandraBala { pub house: u8, pub favourable: bool }
pub struct PersonalFactors { pub tara: Vec<Interval<TaraBala>>, pub chandra: Vec<Interval<ChandraBala>> }
pub struct DayPanchang { pub day_start_jd_ut: f64, pub day_end_jd_ut: f64,
                         pub sunrise_jd_ut: Option<f64>, pub sunset_jd_ut: Option<f64>,
                         pub next_sunrise_jd_ut: Option<f64>,              // gün doğumu yoksa üçü de None
                         pub vara: Vara, pub tithis: Vec<Interval<DayTithi>>,
                         pub nakshatras: Vec<Interval<DayNakshatra>>,
                         pub yogas: Vec<Interval<u8>>, pub karanas: Vec<Interval<DayKarana>>,
                         pub segments: Option<DaySegments>, pub flags: Vec<Interval<MuhurtaFlag>>,
                         pub personal: Option<PersonalFactors>, pub warnings: Vec<ChartWarning>,
                         pub method: String }                              // "muhurta.v1" (K7)
pub struct MuhurtaCriteria { pub weekdays: Vec<Vara>, pub tithis: Vec<u8>, pub nakshatras: Vec<u8>,
                             pub lagna_signs: Vec<Sign>, pub daytime_only: bool, pub min_minutes: u32,
                             pub exclude: Vec<MuhurtaFlag>, pub require_tara: bool,
                             pub require_chandra: bool }                   // boş küme = kısıt yok
pub struct MuhurtaFilter { pub from_jd_ut: f64, pub to_jd_ut: f64, pub lat: f64, pub lon: f64,
                           pub criteria: MuhurtaCriteria, pub natal: Option<NatalMoon> }
pub struct MuhurtaCursor { pub chunk: u32, pub open_start_jd_ut: Option<f64> }
pub struct MuhurtaWindow { pub start_jd_ut: f64, pub end_jd_ut: f64, pub snapshot: WindowSnapshot }
pub struct MuhurtaSearchPage { pub windows: Vec<MuhurtaWindow>, pub scanned: u32, pub chunks: u32,
                               pub next: Option<MuhurtaCursor>, pub warnings: Vec<ChartWarning>,
                               pub method: String }
```

Komutlar:

- `day_panchang(at: TransitAt, place: MuhurtaPlace, settings, natal: Option<NatalMoon>) -> DayPanchangView { day, offset_secs }`. `at` transit anıdır ("şimdi" Rust'ta okunur). `MuhurtaPlace { lat, lon, tz }` ad taşımaz.
- `muhurta_search(query: MuhurtaQuery, settings, cursor: Option<MuhurtaPageCursor>) -> MuhurtaSearchView`. `MuhurtaQuery { from_date, to_date, place, criteria, natal }`: tarihler yerin takvim günleridir (`YYYY-MM-DD`, ikisi de dahil); boş başlangıç bugün, boş bitiş başlangıçtan 30 gün sonrasıdır. Günün başı yerin 00:00'ıdır (yaz saati boşluğuna düşerse öğleden 12 saat öncesi). Aralık ilk çağrıda çözülür ve imleçte taşınır: gece yarısını aşan bir arama aralığını değiştirmez.
- `load_muhurta_filters()`, `save_muhurta_filters(filters)`: kayıtlı süzgeçler `app_state`'in `muhurta_filters` anahtarında `{"schema_version": 1, "filters": [{"name", "criteria"}]}` olarak durur. Okuma toleranslıdır; kayıt listenin tamamını yazar ve kanonik listeyi döner. En çok 32 süzgeç; ad kırpılır, boş olamaz, 200 karakteri aşamaz ve büyük/küçük harf ayırmadan tektir (karşılaştırma dilden bağımsızdır; arayüz aynı karşılaştırmayı kullanır). Kişi, yer ve tarih içermez.
- **Saatler seçilen yerin saatiyle gösterilir.** Ofset Rust'ta çözülür (`atlas::offset_at`) ve sonuçla gelir: gün panchangında günün başındaki ofset, aramada pencerenin iki ucundaki ofset. Gün içinde yaz saati değişirse gün panchangının saatleri o gün bir saat kayık gösterilir (yılda iki gün).
- **Hatalar yalnızca alan adı taşır:** `invalid_muhurta_input { field }`; alanlar `range`, `place`, `natal`, `graha`, `tithis`, `nakshatras`, `min_minutes`, `cursor` (çekirdek) ve `at`, `from_date`, `to_date` (komut). Muhurta yolları konumu, saat dilimini ve anı kendileri denetler: v1'in `invalid_location { lat, lon }`, `invalid_julian_day { jd }`, `unknown_time_zone { tz }` ve `nonexistent_local_time { local, suggestion }` hataları bu yollarda üretilmez. Yer her şeyden önce denetlenir: geçersiz bir yer, tarihler ve an ne olursa olsun `place` alanıyla döner. Motorun kendi `calc { detail }` hatası v1'deki gibi geçer (ayrıntı Swiss Ephemeris'in iletisidir; yer ve kişi taşımaz).

### Arayüz

| Panel | `kind` | Ayarlar | İçerik |
|---|---|---|---|
| Gün panchangı | `muhurta_day` | `place` (bulunulan yer / harita yeri / özel), `custom` (özel yer), `link` (transit tarihi / bugün) | Başlık (tarih, vara); gün doğumu, batımı ve sonraki gün doğumu; gün şeridi (sekiz dilimli sıralı liste, işaretli üçü ve altında Abhijit); beş uzvun tablosu (değer ve sınıf, başlangıç, bitiş); işaretler; tara ve chandra bala tablosu; hora tablosu; yöntem kimliği. |
| Muhurta araması | `muhurta_search` | `place`, `custom`, `criteria` (etkin süzgeç) | Kayıtlı süzgeç satırı; yer; başlangıç ve bitiş tarihi (oturumluk); onay kutusu kümeleri; "Ara"; sonuç tablosu: başlangıç, bitiş, süre, uzuvlar ve lagna. Satırın başlangıcı transit tarihini o ana götürür. |

- İki panel de modül panelidir (K12): yalnızca harita açıkken sunulur ve doğum Ay'ı her zaman açık haritanınkidir; haritasız gün panchangı yoktur.
- Gün şeridinde dilimler desen ve yazıyla ayrılır ("Rahu", "Yama", "Guli", "Abhi"); renk tek başına anlam taşımaz. Abhijit çubuğu yalnızca çizimdir; saatleri metin olarak da yazılıdır. Saat "SS:DD"dir; günün başladığı takvim gününden sonraki saat "(+1)" taşır.
- Gün yeniden okunurken son okunan gün ve odak yerinde durur (`aria-busy`); hatadan sonraki okuma boş başlar (hatadan önceki gün gösterilmez).
- Arama 100 ms'yi aşarsa satır içi gösterge, 1 s'yi aşarsa ilerleme (taranan bölüm / toplam) ve "Durdur". Sonuç tablosu arama boyunca DOM'da durur. İlk sayfadan önce durdurulan arama bunu söyler; hata kendi satırındadır. En çok 500 satır çizilir; fazlası sayılır.
- **Liste, başlatıldığı forma aittir:** koşullar, tarihler ya da yer arama sürerken değişirse arama durdurulur; değişmiş formun altındaki liste "önceki aramanın sonucu" diye işaretlenir.
- **Kayıtlı süzgeçler:** kayıt listenin tamamını yazdığı için saklanan liste okunmadan (okunurken ya da okunamadıysa) kaydetmek ve silmek kullanılamaz; okunamayan liste yeniden okunabilir. Ad alanında Enter süzgeci kaydeder, aramayı başlatmaz.
- Kullanılamayan düğme yerel `disabled` değildir: odaklanır ve nedeni görünür bir satırdır (yer seçilmedi; ad yok; 32 süzgeç sınırı; seçili süzgeç yok; liste okunmadı). Odaktaki denetimi kaldıran her eylem odağı bilerek taşır ("Durdur" → "Ara"; "Süzgeci sil" ve "Süzgeçleri yeniden oku" → seçici; "Yeri kaldır" → "doğum yerini kullan").
- Kutup günü ya da gecesinde gündüz dilimleri ve hora tablosu çizilmez; panel uyarı rozetiyle uzuvları, işaretleri ve kişiye bağlı etkenleri gösterir.

### Test

- **Çapraz denetim (asıl güvence):** `timeline` aralıklarının içinden örneklenen anlarda `compute_panchang` aynı tithi, nakshatra, yoga, karana ve varayı verir; aralık sınırı, `compute_panchang`'ın bitiş zamanıyla ≤ 1 sn içinde örtüşür (sahte ve gerçek ephemeris'le). v1 panchangı golden fixture'larla zaten doğrulanmıştır.
- **El örnekleri:** yedi vara için birer sentetik gün (`tests/golden/modules/muhurta/`, kaynak `hand`): dilim saatleri, Abhijit, hora düzeni, tithi grubu, nakshatra doğası, tara ve chandra bala. Dilim sıraları formül ve elle yazılmış tablo olarak iki ayrı yazımdır. Örnekler motoru yazan dışında biri tarafından kuraldan yeniden türetilir (kayıt `REVIEW.md`'de).
- **Property:** aralıklar boşluksuz ve örtüşmesiz; sekiz dilim gündüzü tam böler; Rahu, Yamaganda ve Gulika her varada üç farklı dilimdir; 24 hora günü tam böler ve ilk hora lordu vara lordudur; arama sonucu her pencerenin her anında süzgeci sağlar, pencere araları ve son pencereden sonrası sağlamaz (kısa dilimlerle yürünen aramada da), pencereler en kısa süreden kısa değildir ve dilim uzunluğu sonucu değiştirmez.
- **Astronomik:** bulunan burç girişi anında boylam sınırın ± 1″ içindedir; dönüş anında hız işaret değiştirir.

### Gizlilik, performans, kapsam dışı

- Gizlilik: §14. `home_place` kişisel veridir: loglanmaz, hata parametresine girmez, dışa aktarıma yazılmaz. Yer, sorgu, doğum Ay'ı ve kayıtlı süzgeç adları loglanmaz; komutlara yerin adı gönderilmez; kayıtlı süzgeçler de dışa aktarılmaz ve içe aktarma tercihlere dokunmaz. Sonuçlar saklanmaz. Ağ isteği yok.
- Performans: `day_panchang` ≤ 50 ms. Arama: lagna süzgeci olmadan 366 gün ≤ 2 s, lagna süzgeciyle ≤ 5 s; ilk sayfa ≤ 500 ms; her sayfa ≤ 300 ms. Arama motor iş parçacığını dilim aralarında bırakır; araya giren harita hesabı en çok bir dilim bekler. Ölçüm (hata ayıklama derlemesi): gün ~0,6 ms; 366 gün ~0,2 sn, lagna koşuluyla ~0,4 sn; en yavaş dilim ~40 ms.
- Kapsam dışı: etkinlik şablonları, vara–nakshatra birleşim yogaları, otuz muhurtanın adları, choghadiya ve benzeri bölgesel gün bölümlemeleri, tithi–vara–nakshatra dosha tablolarının tamamı, tutulmalar, takvim görünümünde aylık panchang, bildirim/hatırlatma; haritasız gün panchangı; elle koordinat formu; günün dışına taşan uzuvların gerçek uçları; 500'den fazla pencerenin listelenmesi; sonuçların ve kayıtlı süzgeçlerin dışa aktarılması; ikinci okumalar (gündüz ve gece ayrı bölünen horalar, sabit 06:00–18:00 günü, öğlenin iki yanında sabit Abhijit, Janma tarasının elverişli sayılması, shukla pakshada genişleyen chandra bala kümesi, 293°20′'de başlayan panchaka): K7'ye göre ancak ikinci seçenek uygulanınca ayar olur.

---

## 11. Alt proje 3.8 — Prashna

### Kapsam ve yöntem

Soru anı için harita kurma ve o haritanın olgularını gösterme. Hesap Parashari motorunun aynısıdır; ayrı bir Prashna ekolünün (Tajika yogaları, Kerala ashtamangala) kuralları uygulanmaz.

- **Zaman prashnası:** an = şimdi (ya da girilen an), yer = `home_place`. `compute_chart` ile aynı.
- **Sayı prashnası (1–108):** grahalar anın grahalarıdır; lagna sayıdan gelir: `lagna_lon = (n − 1) × 3°20′ + 1°40′` (sidereal; n. navamsanın ortası). Evler bu lagnadan rasi evidir. Sayı bir lagna verir, meridyen vermez: Shadbala ve Bhava bala (§7) sayı prashnasında hesaplanmaz (`compute_shadbala` `None` döner; iki panel nedenini söyler). Çekirdeğe `compute_chart_with_lagna(eph, jd_ut, lat, lon, settings, lagna_sidereal_lon)` eklenir.
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

1. **El örnekleri** (`tests/golden/modules/<modül>/*.json`, test dosyası `crates/jyotish-core/tests/modules_golden.rs`). Girdi sentetik `Positions`'tır (ephemeris gerekmez); beklenen değer klasik kuraldan elle türetilir ve fixture'ın `derivation` alanında adım adım yazılır. Şema: `{ id, source: "hand", rule_ref, derivation, input, expected }`. Sonucu ayara bağlı modüllerde `input` bir `advanced` alanı taşır (ör. Jaimini örneklerinde karaka şeması ve eş lord kuralı). `rule_ref` genel bir göndermedir (ör. "BPHS, pada bölümü"). Bilinmeyen alan reddedilir; modül başına en az sayı test içinde sabitlenir (sessiz silinmeye karşı). `source` olarak yalnızca `hand` kabul edilir.
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
| `compute_drishti`, `compute_jaimini`, `compute_chara_dasha`, `chara_dasha_at`, `compute_yogas` | ≤ 10 ms |
| `chara_dasha_children` | ≤ 5 ms |
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

- **Varyant seçimleri tartışılabilir.** Shadbala'da yıl/ay lordunun güneş girişinden bulunması (giriş sidereal çerçevede, varası doğum yerinde ölçülür), saptavargaja değerleri, drik balada Merkür ve Jüpiter'in bakışının çeyreğin *üstüne* tam eklenmesi (1,25 kat; öteki okuma 1,00 kat — §7 madde 6; çalışma kâğıdı ile motor aynı okumayı paylaşır, yani kâğıt aritmetiği doğrular, bu seçimi değil), cheshtanın ortalama öğelerle hesabı, sıranın toplama göre verilmesi, özel bakışların ekleme-ve-kesme biçimi; chara dashada padakrama düzeni ve alt dönem sırası; chara karakada varsayılan sekizli şema; yogalarda mahapurushanın yalnızca lagnadan sayılması, Kemadruma'nın iptalleri, Neecha bhanga'nın üç koşulu ve Ay'ın kendisinden sayılmaması, değişim sınıfının değişen evlerle belirlenmesi, Dharma-Karmadhipati'nin ek kayıt olması (ikinci bir okuma gelirse katalog sürümü artar). Hepsi `method` kimliğiyle görünürdür ve tek fonksiyonda toplanmıştır; beta astrologlarından gelen geri bildirimle ikinci bir yöntem eklenebilir (o zaman K7'ye göre ayar olur).
- **Kuta puan tabloları basılı kaynaklarla karşılaştırılmadı.** Üç tablo §9'da hücre hücre yazılıdır ve yaygın basılı biçimden aktarılmıştır; değişmezleri ve ikinci yazımları testtedir, el örnekleri bağımsız olarak yeniden türetilmiştir. Bu, aritmetiği ve okumayı doğrular, hücreleri değil: iki bağımsız basılı klasik kaynakla hücre hücre karşılaştırma sahibin işidir (`tests/golden/modules/kuta/REVIEW.md`'deki kayıt). Bakılacak ayrışma noktaları: vashya'da insan–böcek hücreleri ve yabanıl sınıfın insan, böcek ve suyla hücreleri (bakışımlı varyant); gana'da Deva/Manushya 6 ve 5'in yeri ile damat Rakshasa – gelin Deva (1 ya da 0) ve tersi; yoni'de 1, 2, 3 değerli tek tük hücreler. Karşılaştırma yapılana dek puanlar "doğrulanmamış tablo" notuyla sunulur. Öteki okuma seçimleri (taranın iki yönlü sayımı, vashya yarılarının 15° sınırı, istisnaların puanı değiştirmemesi, her haritanın kendi ayarıyla hesaplanması) §9'da yazılıdır ve ikinci bir okuma uygulanana dek ayar değildir (K7).
- **Muhurta tabloları basılı kaynaklarla karşılaştırılmadı.** Üç işaretli dilimin vara sıraları, tithi grupları, nakshatra doğaları ve iki elverişli küme §10'da yazılıdır ve muhurta derlemelerinin yaygın biçiminden bellekten aktarılmıştır; ikinci yazımları ve değişmezleri testtedir, yedi el örneği bağımsız olarak yeniden türetilmiştir. Bu, kuralların uygulanışını ve aritmetiği doğrular, tabloları ve okuma seçimlerini değil: basılı bir panchang ya da muhurta derlemesiyle karşılaştırma sahibin işidir (`tests/golden/modules/muhurta/REVIEW.md`'deki sekiz satır ve kayıt tablosu). Bakılacak ayrışma noktaları, önem sırasıyla: **horaların uzunluğu** (burada gün doğumundan sonraki gün doğumuna 24 eşit dilim; yaygın öteki okuma gündüzün ve gecenin ayrı ayrı on ikiye bölünmesidir ve gündüz ile gece eşit değilken her horanın saatini değiştirir); dilimlerin gerçek gün doğumu ve batımına göre orantılı olması (öteki okuma sabit 06:00–18:00 günü ve 90 dakikalık dilimler); Gulika'nın dilimin tamamı olması (başını, ortasını ya da sonunu alan ve gece Gulika'sını da veren gelenekler); Abhijit'in gündüzün on beşte biri olması (öğlenin iki yanında sabit 24'er dakika; Çarşamba günü hiç verilmemesi); Janma tarasının elverişli sayılmaması; shukla pakshada genişleyen chandra bala kümesi; panchakanın 300°'de başlaması (öteki okuma 293°20′); Vishti'nin yalnızca karanadan okunması. Karşılaştırma yapılana dek gün dilimleri ve sınıflar "doğrulanmamış tablo" notuyla sunulur. İkinci bir okuma uygulanana dek hiçbiri ayar değildir (K7); yöntem kimliği `muhurta.v1`'dir.
- **Zaman çizelgesinin göremediği kısa gidişler.** Bir örnekleme adımının içinde değişip aynı değere dönen değer (dönüş anının tam burç sınırına düşmesi; kutup dairesinin içinde lagnanın anlık sıçraması) dizide görünmez (§10 "Yöntem"). 3.9'un olay araması aynı çekirdeği kullanacağı için bu sınır orada da geçerlidir.
- **Ortalama boylam katsayıları** yayımlanmış tablodan alındı ve ephemeris ile gezegen başına sınırlarla doğrulandı (§7). Jüpiter ve Satürn'de tek bir ±0,05° sınırı tutmaz; iç gezegenlerin güneş merkezli ortalama boylamı yalnızca alt kavuşumlarda dolaylı ölçülür.
- **`Positions` lagnanın rasi çerçevesindeki derecesini taşımaz.** Shadbala onu Güneş'in iki çerçevedeki farkından türetir (§7 "Çerçeveler"). Başka bir modül de gerek duyarsa §4'e `lagna_deg_in_sign` eklenmesi değerlendirilir.
- **Tek ephemeris iş parçacığı.** Uzun aramalar harita açmayı bekletebilir; dilimleme ve 300 ms sınırı bunu karşılar. Yetmezse ikinci bir `SwissEph` örneği ayrı bir işin konusudur (SE global durumu nedeniyle süreç içinde güvenli değildir; ayrı süreç gerekir).
- **Harita araması büyük veritabanında yavaşlayabilir.** Ölçüm ilk görevdir; oturumluk önbellek yedeği tanımlıdır (§12).
- **El örneği yazanın kuralı yanlış anlaması** motoru da yanlış doğrular. Karşılığı: ayrı gözden geçirmede yeniden türetme (§13) ve değişmezler.
- **Sahibin kararına bırakılanlar** (engelleyici değil; varsayılanlarla ilerlenir): Shadbala yıl/ay lordu yöntemi, drik balanın okunuşu (1,25 kat) ve chara karaka varsayılanı beta öncesinde bir astrologla teyit edilmeli; üç kuta puan tablosu iki basılı kaynakla karşılaştırılmalı (§9); muhurta tabloları ve özellikle hora uzunluğunun okunuşu basılı bir panchangla karşılaştırılmalı (§10); Muhurta etkinlik şablonlarının (§10) sonraki bir sürüme alınıp alınmayacağı.

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
