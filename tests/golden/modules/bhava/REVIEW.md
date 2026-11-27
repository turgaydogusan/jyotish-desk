# Bhava tablosu ve Sudarshana chakra: okuma kuralları, onay bekleyen noktalar ve gözden geçirme kaydı

Kaynak: yol haritası (`docs/superpowers/specs/2026-11-27-eksikler-yol-haritasi.md`) §4.5 ve
§4.6, Plan 25'in kararları (D1–D20) ve ileri modüller tasarım belgesinin K1–K5 kuralları. Bu
modül bellekten aktarılmış klasik tablo taşımaz (Y4'ün "doğrulanmamış tablo" notu gerekmez);
taşıdığı şey okuma seçimleridir ve aşağıda sahibin onayını bekler. Bu dizinde JSON örneği
yoktur: el örnekleri Rust birim testleridir ve türetmeleri test yorumlarında yazılıdır
(`crates/jyotish-core/src/bhava/table.rs`, `crates/jyotish-core/src/bhava/sudarshana.rs`).
Beklenen değerler kuraldan elle türetilmiştir; motor çıktısı kopyalanmaz (K9). Hiçbir satır
hüküm değildir: tablo ve çizim olguları sayar.

## Okuma kuralları

1. **Çerçeve.** Bhava noktalarının boylamı, burcu ve burç içi derecesi rasi çerçevesindedir
   (zodyak kipini izler, K5). Nakshatra ve pada sidereal boylamdan okunur; tropikal-rasi kipinde
   boylamdan ayanamsa düşülür.
2. **Nokta.** Burç lordu v1'in burç lordu tablosundandır (Akrep: Mars, Kova: Satürn). Nakshatra
   lordu Vimshottari sırasıdır: Ashwini'den başlayarak Ketu, Venüs, Güneş, Ay, Mars, Rahu,
   Jüpiter, Satürn, Merkür; sıra üç kez yinelenir. İkinci yazım: bir boylamın nakshatra lordu,
   o boylamdaki Ay'ın Vimshottari doğum lordudur (test iki yazımı 27 nakshatrada karşılaştırır).
3. **Tam burç.** Bhava burcun kendisidir: başlangıç 0°, bitiş 30°. Orta, lagnanın derecesinin
   o burçtaki karşılığıdır (lagna + 30° × (n − 1)); Bhava bala da aynı noktayı kullanır. Hiçbir
   graha burç evinden başka bir bhavada olamaz.
4. **Sripati.** Ephemeris'in döndürdüğü on iki nokta bhava başlangıcıdır (sandhi). Orta,
   Porphyry noktasıdır: MC'den lagnaya uzanan yay ve lagnadan IC'ye uzanan yay üçe bölünür;
   öteki altı nokta bunların karşısıdır. 1. bhavanın ortası lagna, 4. bhavanınki IC, 7.
   bhavanınki batı ufku, 10. bhavanınki MC'dir. İkinci yazım: her sandhi, iki komşu ortanın tam
   ortasıdır (gerçek motorla beş golden anında, iki zodyak kipinde 10⁻⁶° içinde sınanır).
5. **Placidus.** Nokta bhavanın başlangıcıdır (KP okuması). Bhava ortası tanımlanmaz: orta
   sütunu çizgidir ve "ortadan uzaklık" verilmez. Kutup dairesinde Placidus hesaplanamazsa
   başlangıçlar Porphyry noktalarıdır; tablo ve chalit görünümü bunu sözle söyler.
6. **Bitiş ve yay.** Bir bhavanın bitişi sonraki bhavanın başlangıcıdır; yay ikisinin farkıdır
   (mod 360°). On iki yayın toplamı 360°'dir.
7. **Grahanın bhavası.** Boylamdan geriye en yakın başlangıç, içinde bulunulan bhavanın
   başlangıcıdır; başlangıç noktası o bhavaya dahildir (graha tablosunun "ev" sütunuyla aynı
   kural). "Başlangıçtan" değeri [0°, yay) aralığındadır. "Ortadan" değeri işaretlidir: eksi,
   graha ortadan öncedir; artı, sonradır.
8. **Burç evi ve işaret.** Burç evi, grahanın burcunun lagna burcundan sayısıdır (K4). Bhava
   burç evinden farklıysa graha `*` ile işaretlenir.
9. **Chalit görünümü.** Hücreler yerinde durur: n. hücre n. rasi evinin hücresidir ve burç
   numarası o evin burcudur. Her graha, bhavasının numarasını taşıyan hücreye çizilir; burcu ve
   derecesi değişmez (erişilebilir ad grahanın kendi burcunu söyler). İki burca yayılan bhava
   tek hücredir; kesin sınırlar Bhava tablosu panelindedir. Görünüm yalnızca D1'de, doğum saati
   biliniyorsa ve ev sistemi tam burç değilse çizilir; transit halkası, eş halkası ve arudha
   padalar bu görünümde çizilmez.
10. **Sayı prashnası.** Lagna sayıdan gelir ve evler ayar ne olursa olsun tam burçtur.
11. **Doğum saati bilinmiyorsa** bhava tablosu yoktur (Rust `None` döner); chalit görünümü
    çizilmez.
12. **Sudarshana chakra.** Üç halka içten dışa lagna, Ay ve Güneş içindir. Her halkada n. ev, o
    halkanın başlangıç burcundan sayılan n. burçtur (burç evi, K4) ve grahalar rasi
    çerçevesindeki burçlarından okunur. Aynı numaralı ev üç halkada aynı açıdadır; 1. ev
    soldadır ve evler saat yönünün tersine ilerler. Doğum saati bilinmiyorsa lagna halkası
    çizilmez ve Ay'ın burcu yaklaşık (≈) işaretlenir. Çizimde yalnızca graha kısaltması yazılır
    (retro, yanma ve vargottama işaretleri yazılmaz). Sudarshana dashası hesaplanmaz.

## El örnekleri (Rust birim testleri)

| Test | Sınadığı kural |
|---|---|
| `table.rs › nakshatra_lords_repeat_every_nine_and_agree_with_the_dasha_seed` | 2 (ikinci yazım) |
| `table.rs › a_point_reads_sign_nakshatra_and_both_lords` | 1, 2 |
| `table.rs › porphyry_points_trisect_the_quadrants_and_sandhis_bisect_the_points` | 3, 4 |
| `table.rs › sripati_table_by_hand` (lagna 101°, MC 341°; dokuz graha) | 4, 6, 7, 8 |
| `table.rs › whole_sign_table_never_shifts_and_placidus_has_no_middle` | 3, 5 |
| `table.rs › collector_follows_the_chart_and_hides_the_table_when_time_is_unknown` | 5, 11 |
| `table.rs › tropical_rasi_mode_and_number_prashna_follow_the_chart` | 1, 10 |
| `sudarshana.rs › three_rings_count_from_the_lagna_the_moon_and_the_sun` | 12 |
| `sudarshana.rs › without_a_birth_time_the_lagna_ring_is_empty_and_the_result_is_approximate` | 12 |
| `app/src-tauri/tests/bhava.rs` (gerçek motor; beş golden anı × üç ev sistemi × iki zodyak kipi) | 3–8, 10–12 |

## Sahibin onayını bekleyen okuma seçimleri ve bilinen varyant noktaları

Aşağıdaki her satır tek okumayla uygulanmıştır (Y5); ikinci okuma ayar değildir. Sahibi basılı
kaynağıyla karşılaştırır ve sonucu alttaki tabloya yazar. Yürüten ajan bu seçimleri başka
yazılımla ya da çevrimiçi kaynakla karşılaştırmaz.

| # | Nokta | Uygulanan okuma | Bilinen öteki okuma |
|---|---|---|---|
| 1 | Sripati'de bhava ortası | Porphyry noktası (1. bhavanın ortası lagna) | Ortayı başlangıç ile bitişin yay ortası sayan okuma (köşe bhavalarında, yani 1., 4., 7. ve 10. bhavada Porphyry noktasından ayrışır) |
| 2 | Sripati'de bhava sınırı | Ephemeris'in döndürdüğü sandhi bhavanın başlangıcıdır | Porphyry noktasını bhava başlangıcı sayan (sandhisiz) okuma |
| 3 | Tam burçta bhava ortası | Lagnanın derecesi (her burçta aynı derece) | Burcun 15°'sini orta sayan okuma |
| 4 | Placidus'ta bhava ortası | Tanımlanmaz (çizgi) | Başlangıç ile bitişin yay ortasını yazan uygulamalar |
| 5 | Chalit çiziminde iki burca yayılan bhava | Tek hücre: n. bhava n. rasi evinin hücresinde; yer değiştiren graha `*` taşır | Hücreye bhava ortasının burcunu yazan çizim; sandhi derecelerini hücre kenarına yazan çizim |
| 6 | Sudarshana'da halka sırası | İçten dışa lagna, Ay, Güneş | İçten dışa Güneş, Ay, lagna |
| 7 | Sudarshana'da yön | 1. ev solda, saat yönünün tersine | 1. ev üstte; saat yönünde |
| 8 | KP alt lordu (nakshatra alt lordu) | Hesaplanmaz (kapsam dışı) | Bhava başlangıçları ve grahalar için alt lord sütunu |

| Denetleyen | Tarih | Kaynak (genel ad) | Sonuç |
|---|---|---|---|
| — | — | — | Bekliyor: sahibine devredildi (Plan 25 Task 8) |

## Denetim maddeleri

**[otomatik]** maddelerin karşılığı testlerde vardır; yürüten ajan testin adını göstererek
"geçti" sayar. **[insan]** maddeler yalnızca gerçek uygulamaya bakan biri tarafından
doğrulanabilir ve sahibine devredilir (Y3: sonraki planı engellemez).

- [ ] **[otomatik]** Sripati sandhileri Porphyry ortalarının ikişer ikişer ortasıdır; 1. bhavanın ortası lagnadır. (`tests/bhava.rs`; `table.rs › porphyry_points_…`)
- [ ] **[otomatik]** Grahanın bhavası graha tablosundaki evdir; tam burçta hiçbir graha yer değiştirmez; Placidus'ta orta yoktur. (`tests/bhava.rs`; `table.rs › whole_sign_table_…`)
- [ ] **[otomatik]** Doğum saati bilinmiyorsa tablo yoktur, chakra lagna halkası olmadan ve yaklaşık döner; sayı prashnasında evler tam burçtur; geçersiz konum saat bilinmese de bildirilir. (`tests/bhava.rs`; `table.rs › collector_…`; `sudarshana.rs › collector_reads_…`)
- [ ] **[otomatik]** Bhava tablosu paneli üç ev sistemini, kutup dairesindeki düşmeyi, nokta ayarını, `*` işaretini (sözle de) ve "saat bilinmiyor" durumunu gösterir. (`BhavaTablePanel.test.tsx`, on bir test)
- [ ] **[otomatik]** Chalit görünümü grahaları haritanın verdiği bhavaya çizer, yeni istek yapmaz, tam burçta kutuyu sunmaz, D1 dışında ve saat bilinmezken nedenini söyler, öteki katmanları çizmez. (`ChartPanel.test.tsx` chalit testleri; `adapter.test.ts`; `ChartSvg.test.tsx`)
- [ ] **[otomatik]** Sudarshana çizimi 36 hücredir, her graha üç kez yazılır, çizimde sekme durağı yoktur; tablo aynı veriyi taşır ve ortak vurguyu sürer. (`SudarshanaSvg.test.tsx`, `SudarshanaPanel.test.tsx`, `sudarshana.test.ts`; e2e)
- [ ] **[otomatik]** Görsel temeller: chalit (üç biçim) ve Sudarshana (açık, koyu). (`pnpm --dir app e2e:visual`: 15)
- [ ] **[insan]** Sripati ayarlı bir deneme haritasında Bhava tablosunun başlangıç ve orta değerleri, sahibin basılı kaynağındaki bir örnekle (ya da elle Porphyry bölmesiyle) uyuşur; yukarıdaki 1–4. satırlar onaylanır ya da düzeltme istenir.
- [ ] **[insan]** Chalit görünümü ekranda okunur: iki burca yayılan bhavanın tek hücrede gösterimi, `*` işareti ve açıklama satırı yeterlidir (5. satır); Kuzey, Güney ve daire biçimlerinde yer değiştiren graha aynı bhavadadır.
- [ ] **[insan]** Sudarshana çizimi açık ve koyu temada, üç dilde okunur; dar panelde tablo çizimin altında kayar; halka sırası ve yön onaylanır (6. ve 7. satırlar); beş grahalık bir yığılmada etiketler okunur.
- [ ] **[insan]** VoiceOver: Sudarshana çizimi tek bir resim olarak adıyla okunur; tabloda burç ve graha düğmeleri tam adlarıyla okunur; chalit görünümünde yer değiştiren graha "bhavası farklı" diye okunur.
- [ ] **[insan]** KP alt lordunun istenip istenmediği sahibin kararına yazılır (8. satır).

## Gözden geçirme kaydı

Gözden geçiren (motoru yazan dışında biri) `table.rs` ve `sudarshana.rs` testlerindeki el
örneklerini yalnızca girdilerinden (lagna, MC ve graha boylamları) başlayarak, motoru
çalıştırmadan, yukarıdaki okuma kurallarından yeniden hesaplar; sonucunu testteki beklenen
değerlerle karşılaştırır ve aşağıya adını, tarihini ve bulduğu farkları yazar.

| Gözden geçiren | Tarih | Sonuç |
|---|---|---|
| — | — | Bekliyor (Plan 25 Task 8) |
