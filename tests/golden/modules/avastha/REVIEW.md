# Avastha el örnekleri: okuma kuralları, tablolar, onay bekleyen noktalar ve gözden geçirme kaydı

Kaynak: eksikler yol haritası §4.4 ve Plan 24'ün kararları; klasik dayanak BPHS'nin avasthalar
bölümüdür (genel gönderme; alıntı yapılmaz). Bu dizinde iki dizi durur:

- `lajjitadi/`: sentetik konumlar (dokuz grahanın ve varsa lagnanın sidereal boylamı). Her dosya
  yedi grahanın Baladi, Jagradadi ve Lajjitadi durumlarını ve o örneğin "oluşmayan durum" olarak
  gösterdiği çiftleri (`absent`) yazar.
- `shayanadi/`: aynı konumlar ve `minutes_since_sunrise` (gün doğumundan doğuma geçen dakika;
  `null` o gün gün doğumu olmayan yerdir). Her dosya formülün ortak terimlerini (`basis`) ve yedi
  grahanın nakshatrasını, navamsa dilimini, toplamını ve durumunu yazar; Shayanadi hesaplanmayan
  haritada `expected` `null`'dır.

Ephemeris yoktur; rasi çerçevesi sidereal sayılır. Değerler kuraldan türetilmiştir ve
`derivation` alanında adım adım yazılıdır; motor çıktısı buraya kopyalanmaz (K9). Bir değer
motorla uyuşmazsa önce kural yeniden okunur. Testler: `crates/jyotish-core/tests/avastha_golden.rs`.

## Okuma kuralları

Ortak tanımlar:

- **Kapsam.** Yedi graha (Güneş … Satürn). Rahu ve Ketu'ya avastha atanmaz; Lajjita'da eşlikçi
  olarak sayılırlar.
- **Burç ve ev** rasi çerçevesindedir; ev, lagna burcundan sayılan burç evidir ve ev sistemi
  ayarından bağımsızdır. **Nakshatra** her zodyak kipinde sidereal boylamdandır.
- **Birliktelik** aynı burçta bulunmaktır. **Bakış** burçtan burca tam graha bakışıdır: her graha
  bulunduğu burçtan 7. burca; ayrıca Mars 4. ve 8., Jüpiter 5. ve 9., Satürn 3. ve 10. burca bakar.
  Rahu ve Ketu bakmaz. Aynı burçtaki graha bakmış sayılmaz.
- **Dost** ve **düşman** doğal (naisargika) ilişkidir ve durumu sorulan grahanın gözünden okunur
  (aşağıdaki tablo). Düğümler kimsenin dostu ya da düşmanı değildir. Bir burcun lordu grahanın
  kendisiyse o burç ne dost ne düşman burcudur.
- **İyicil** ve **kötücül** uygulamanın ortak doğal sınıfıdır: Jüpiter ve Venüs iyicil; Güneş,
  Mars, Satürn, Rahu ve Ketu kötücül; Ay büyürken (uzanım 0°–180°) iyicil, küçülürken kötücül;
  Merkür aynı burçta kötücül varken iyicil yoksa kötücül, değilse iyicil. Düğümler bakmadığı için
  sınıfları bu dizide yalnızca Merkür'ün sınıfına etki eder: Merkür aynı burçta yalnız Rahu ya da
  Ketu ile ise kötücüldür.

1. **Baladi.** Burç altışar derecelik beş dilime bölünür. Tek burçlarda (Koç, İkizler, Aslan,
   Terazi, Yay, Kova) dilimler sırayla Bala, Kumara, Yuva, Vriddha, Mrita'dır; çift burçlarda sıra
   terstir (ilk dilim Mrita, son dilim Bala). Dilim sınırı sonraki dilime aittir (6°00′ ikinci
   dilimdir).
2. **Jagradadi.** Graha yücelme burcunda ya da kendi burcundaysa Jagrat; düşüş burcundaysa
   Sushupti; öteki burçlarda burç lordu dostu ya da nötrüyse Swapna, düşmanıysa Sushupti. Sıra
   budur: yücelme ve kendi burcu önce, düşüş sonra, ilişki en son okunur.
3. **Lajjita.** Graha lagnadan 5. evde **ve** Güneş, Mars, Satürn, Rahu ya da Ketu'dan en az biriyle
   birlikte. Lagna yoksa (doğum saati bilinmiyor) değerlendirilmez.
4. **Garvita.** Graha yücelme burcunda (burcun tamamı) **ya da** moolatrikona aralığında.
5. **Kshudhita.** Graha düşmanının burcunda, **ya da** bir düşmanıyla birlikte, **ya da** bir
   düşmanının bakışı altında, **ya da** Satürn ile birlikte (Satürn o grahanın düşmanı olmasa da).
6. **Trushita.** Graha su burcunda (Yengeç, Akrep, Balık) **ve** bir düşmanının bakışı altında
   **ve** ona bakan hiçbir iyicil yok.
7. **Mudita.** Graha dostunun burcunda, **ya da** bir dostuyla birlikte, **ya da** bir dostunun
   bakışı altında, **ya da** Jüpiter ile birlikte.
8. **Kshobhita.** Graha Güneş ile birlikte **ve** bir kötücülün ya da bir düşmanının bakışı
   altında. Güneş'in kendisi Kshobhita olmaz.
9. **Küme.** Lajjitadi'de bir graha aynı anda birden çok durumda olabilir ya da hiçbirinde
   olmayabilir. Liste yukarıdaki kural sırasındadır. Her kayıt, kuralı sağlayan öteki grahaları
   (birlikte olan ya da bakan) graha sırasıyla taşır; kural yalnızca burçtan sağlanıyorsa liste
   boştur.
10. **Shayanadi.** `toplam = N × G × A + J + Gh + L`; durum, toplamın 12'ye bölümünden artanın
    gösterdiği durumdur ve artan 0 ise on ikinci durumdur.
    - `N`: grahanın nakshatrası, Ashwini'den 1–27.
    - `G`: grahanın sayısı (aşağıdaki tablo).
    - `A`: grahanın bulunduğu navamsa diliminin burç içindeki sırası, 1–9 (dilim 3°20′; sınır
      sonraki dilime aittir).
    - `J`: doğum nakshatrası (Ay'ın nakshatrası), 1–27.
    - `Gh`: doğumdan önceki son gün doğumundan doğuma **tamamlanmış** ghati sayısı; bir ghati 24
      dakikadır (gün doğumundan geçen gün kesri × 60, aşağı yuvarlanır).
    - `L`: lagna burcunun sırası, Koç 1 … Balık 12.
    Lagna yoksa (doğum saati bilinmiyor) ya da o gün gün doğumu yoksa Shayanadi hesaplanmaz.

## Tablolar

Motorun kullandığı tablolar (`crates/jyotish-core/src/avastha/` ve v1'in `dignity.rs` dosyası).

**Doğal ilişki** (satırdaki grahanın sütundakine ilişkisi)

| Graha | Dost | Nötr | Düşman |
|---|---|---|---|
| Güneş | Ay, Mars, Jüpiter | Merkür | Venüs, Satürn |
| Ay | Güneş, Merkür | Mars, Jüpiter, Venüs, Satürn | — |
| Mars | Güneş, Ay, Jüpiter | Venüs, Satürn | Merkür |
| Merkür | Güneş, Venüs | Mars, Jüpiter, Satürn | Ay |
| Jüpiter | Güneş, Ay, Mars | Satürn | Merkür, Venüs |
| Venüs | Merkür, Satürn | Mars, Jüpiter | Güneş, Ay |
| Satürn | Merkür, Venüs | Jüpiter | Güneş, Ay, Mars |

**Burçlar** (Jagradadi ve Garvita)

| Graha | Yücelme burcu | Düşüş burcu | Kendi burçları | Moolatrikona aralığı |
|---|---|---|---|---|
| Güneş | Koç | Terazi | Aslan | Aslan 0°–20° |
| Ay | Boğa | Akrep | Yengeç | Boğa 3°–30° |
| Mars | Oğlak | Yengeç | Koç, Akrep | Koç 0°–12° |
| Merkür | Başak | Balık | İkizler, Başak | Başak 15°–20° |
| Jüpiter | Yengeç | Oğlak | Yay, Balık | Yay 0°–10° |
| Venüs | Balık | Başak | Boğa, Terazi | Terazi 0°–15° |
| Satürn | Terazi | Koç | Oğlak, Kova | Kova 0°–20° |

Aralığın başı içeride, sonu dışarıdadır.

**Shayanadi: graha sayıları ve durum sırası**

| Graha | Güneş | Ay | Mars | Merkür | Jüpiter | Venüs | Satürn |
|---|---|---|---|---|---|---|---|
| `G` | 1 | 2 | 3 | 4 | 5 | 6 | 7 |

| Artan | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 0 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Durum | Shayana | Upaveshana | Netrapani | Prakashana | Gamana | Agamana | Sabha | Agama | Bhojana | Nritya Lipsa | Kautuka | Nidra |

## Sahibin onayını bekleyen kurallar ve bilinen varyant noktaları

Yukarıdaki kurallar ve tablolar bu depoya bellekten aktarılmıştır; basılı bir kaynakla
karşılaştırılmamıştır (yol haritası Y4). Aşağıdaki noktalarda çeviriler ve gelenekler ayrışır.
Karşılaştırma bir insanın işidir: elinde basılı bir BPHS çevirisi ya da avasthaları anlatan bir
derleme bulunan kişi her satırı denetler, kaynağı genel adıyla (alıntı yapmadan) ve sonucu
aşağıdaki tabloya yazar. Fark bulunursa kural ayrı bir commit'le düzeltilir, `AVASTHA_METHOD`
kimliği değişir ve bu dizindeki örnekler kuraldan yeniden türetilir. Onaya dek Avastha paneli
"doğrulanmamış tablo" notunu taşır (`avastha.unverified`).

| No | Nokta | Bu depodaki okuma | Bilinen diğer okuma |
|---|---|---|---|
| 1 | Jagradadi'nin eklenmesi | Eklendi (yol haritası sahibin kararına bırakmıştı; maliyeti Baladi kadardır) | Hiç gösterilmemesi |
| 2 | Jagradadi'de dostluk | Doğal ilişki; moolatrikona ayrı basamak değildir (kendi burcu sayılır) | Bileşik (panchadha) ilişki; moolatrikonanın Jagrat'a ayrıca yazılması |
| 3 | Baladi'nin kapsamı | Yedi graha; düğümlere atanmaz | Dokuz grahaya ve lagnaya da basamak veren tablolar |
| 4 | Lajjita'nın evi ve eşlikçileri | Lagnadan 5. burç evi; Güneş, Mars, Satürn, Rahu, Ketu'dan biri | Yalnızca Rahu ya da Ketu ile; evin Ay'dan da sayılması; bhava (ev sistemi) evi |
| 5 | Garvita | Yücelme burcunun tamamı ya da moolatrikona aralığı | Moolatrikona burcunun tamamı; yalnızca derin yücelme derecesine dek |
| 6 | Düşmanlığın türü (Kshudhita, Trushita, Kshobhita) | Doğal ilişki | Bileşik (doğal + geçici) ilişki |
| 7 | Kshudhita'nın kolları | Dört kol: düşman burcu, düşmanla birliktelik, düşman bakışı, Satürn ile birliktelik | Satürn kolunu saymayan üç kollu okuma |
| 8 | Bakışın türü | Burçtan burca tam graha bakışı | Dereceden bakış (sphuta drishti, bir eşikle); rashi drishti |
| 9 | Trushita'nın su burcu | Yengeç, Akrep, Balık | Lagnadan 4. ev ("su evi") okuması; düşman yerine kötücül bakışı arayan okuma |
| 10 | Mudita | Dört kolun herhangi biri ("ya da") | Kolların birlikte aranması ("ve"); Jüpiter kolu yerine iyicil bakışı |
| 11 | Kshobhita | Güneş ile aynı burçta ve kötücül ya da düşman bakışı altında | Yanıklık orbuyla birliktelik; kötücülle birlikteliğin de sayılması |
| 12 | Birden çok durum | Küme: bütün tutan kurallar, kural sırasında | Öncelik sırasıyla tek durum |
| 13 | Shayanadi: graha sayıları | Güneş 1 … Satürn 7; düğümlere durum atanmaz | Rahu 8 ve Ketu 9 ile dokuz graha |
| 14 | Shayanadi: navamsa terimi | Burç içindeki dilim sırası, 1–9 | Navamsa burcunun Koç'tan sırası, 1–12 |
| 15 | Shayanadi: ghati | Tamamlanmış ghati; bir ghati 24 dakika | Süren ghati (tamamlanan + 1); gün doğumundan sonraki gün doğumuna altmış eşit dilim |
| 16 | Shayanadi: artan 0 | On ikinci durum (Nidra) | — (okuma ortaktır; kayıt için yazıldı) |
| 17 | Shayanadi: on iki adın sırası | Yukarıdaki tablo (bellekten aktarıldı) | Belirli bir varyant bilinmiyor; sıra basılı kaynakla hücre hücre denetlenmelidir |
| 18 | Shayanadi: lagna terimi ve çerçeve | Lagnanın rasi burcunun sırası; tropikal-rasi kipinde navamsa dilimi ve lagna rasi çerçevesinden, nakshatra sidereal | Her terimin sidereal çerçeveden okunması |
| 19 | Kapsam dışı bırakılanlar | Shayanadi'nin üç alt durumu (cheshta) ve Deeptadi yoktur | — |

| Denetleyen | Tarih | Kaynak (genel ad) | Sonuç |
|---|---|---|---|
| — | — | — | Bekliyor: sahibine devredildi (Plan 24 Task 8) |

## Gözden geçirme kaydı

Gözden geçiren (örnekleri yazan dışında biri) her örnekte yalnızca `input`'tan başlar, motoru
çalıştırmadan ve Rust kaynağını okumadan, yukarıdaki okuma kurallarından ve tablolardan yedi
grahanın Baladi, Jagradadi ve Lajjitadi durumlarını (`lajjitadi/`) ya da ghatiyi, ortak terimleri,
toplamı ve durumu (`shayanadi/`) yeniden hesaplar; sonucunu `expected` ile karşılaştırır ve aşağıya
adını, tarihini ve bulduğu farkları yazar.

| Gözden geçiren | Tarih | Sonuç |
|---|---|---|
| Plan 24'ü yazan oturum (örneklerin yazarı) | 2026-11-27 | Örnekler, motorun kaynağından ayrı yazılmış bir türetmeyle kural metninden üretildi ve motorla karşılaştırıldı: `lajjitadi/` yedi örnek, 49 graha satırı; `shayanadi/` dört hesaplı örnek, 28 graha satırı; fark yok. Sınır: türetmeyi ve motoru aynı yazar yazdı; kuralın okunuşundaki ortak bir yanlış ikisinde de aynıdır. Bu kayıt aritmetiği ve kuralların uygulanışını doğrular, okuma seçimlerini (yukarıdaki onay tablosu) değil. |
| Plan 24'ün son gözden geçirmesi (Claude; örnekleri ve motoru yazan oturumların dışında) | 2026-11-27 | 13/13 uyuştu: 49 + 28 graha satırı ve iki hesapsız örnek, fark yok. Depo dışında yazılan bir python3 betiği kuralları ve tabloları bu dosyadan aldı, her JSON'un `input`'undan başladı ve motoru çalıştırmadı; betik Rust kaynağı okunmadan yazıldı. Aynı betik birim testlerinin beklentilerini de yeniden hesapladı (Baladi sınırları, Jagradadi, altı Lajjitadi kuralı, Shayanadi terimleri ve toplamları: 143 beklenti, fark yok). Bulgu: "Okuma kuralları"ndaki iyicil ve kötücül maddesi düğümleri saymıyordu, motorun ortak sınıfı ise Rahu ve Ketu'yu kötücül sayar; fark yalnızca Merkür'ün sınıfına etki eder, on üç örnekte iki okuma da aynı sonucu verir. Madde motorun sınıfına göre düzeltildi ve durum bir birim testiyle sabitlendi (`lajjitadi.rs › mercury_alone_with_a_node_…`). Sınırlar: betik yazılmadan önce dosya biçimini öğrenmek için dört örneğin `expected` ve `derivation` alanları görüldü; gözden geçiren, planı ve örnekleri yazan oturumla aynı model ailesindendir ve kuralın okunuşundaki ortak bir yanlış burada da aynı olur. Yürüten ajanın bu satıra yazdığı önceki kayıt bağımsız bir türetme değildi (yalnızca `cargo test` koşulmuştu); bu satır onun yerine yazıldı. Bu kayıt kuralların uygulanışını ve aritmetiği doğrular, okuma seçimlerini (onay tablosu) değil. |
