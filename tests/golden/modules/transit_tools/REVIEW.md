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

## 3. Efemeris tablosu (yöntem `ephemeris_table.v1`)

### 3.1 Okuma kuralları

1. **Satır.** Seçilen yerin takvim ayının her günü için bir satır; satırın anı o günün başıdır:
   yerin duvar saatiyle 00:00. Yaz saati gece yarısında başlıyorsa (00:00 yoksa) satır günün
   ilk geçerli anındadır, yani geçiş anında (São Paulo 2017-10-15: 03:00 UT = 01:00 yerel);
   o gün önceki satırdan 24, sonraki satıra 23 saat uzaktadır. Saat geri alınırken 00:00 iki
   kez yaşanıyorsa önceki an alınır. Saat dilimi bir günü tümden atlıyorsa (Pacific/Apia
   2011-12-30, Pacific/Kiritimati 1994-12-31) o gün yerde yoktur: satırı yazılmaz, ay hata
   vermez ve 29–30 satır döner; işaretler atlanan günün üstünden ardışık satırları karşılaştırır.
   Ayı kapatan an, sonraki ayın ilk var olan gününün başıdır.
2. **Konum.** Dokuz grahanın boylamı transit hesabıyla aynıdır: burç ve burç içi derece rasi
   çerçevesindedir (haritanın zodyak kipini izler, K5); nakshatra ve pada sidereal boylamdandır.
   Ayanamsa ve düğüm türü açık haritanın ayarıdır. Retro, boylam hızının eksi olmasıdır; Rahu ve
   Ketu'da retro işareti yazılmaz.
3. **Giriş işareti.** Graha, satırın anı ile bir sonraki satırın anı arasında başka bir burca
   giriyorsa satır girilen burcu taşır. Ayın son gününün işareti, sonraki ayın ilk gününün
   başındaki konumdan okunur.
4. **Dönüş işareti.** Mars, Merkür, Jüpiter, Venüs ve Satürn için: graha aynı süre içinde retroya
   ya da düze dönüyorsa satır dönülen yönü taşır. Düğümlerin yönü işaretlenmez.
5. **İşaretlerin sınırı.** İşaret, ardışık iki satırın karşılaştırmasıdır: kesin an olay
   aramasındadır. Bir gün içinde burç sınırını geçip geri dönen durağan bir graha işaret almaz.
6. **Aralık.** Bir ay (28–31 satır); yıl 1–2999. Ay verilmezse yerin bugünkü ayıdır. Ephemeris
   dosyalarının dışındaki yıllarda (1800 öncesi, 2400 ve sonrası) tablo `LowPrecision` uyarısını
   taşır.
7. **Kapsam dışı.** CSV ve baskı (K10); enlem, hız ve deklinasyon sütunları; yıl görünümü;
   00:00 UT satırı.

### 3.2 Onay bekleyen noktalar (sahibi)

- [ ] Satır saatinin seçilen yerin yerel gece yarısı olması (öteki okuma: 00:00 UT).
- [ ] Varsayılan yerin bulunulan yer olması (öteki okuma: haritanın doğum yeri).
- [ ] Burç ve derecenin haritanın zodyak kipini izlemesi (tropikal-rasi kipinde tropikal).

## 4. Geçiş anının adımı ve oynatma

### 4.1 Okuma kuralları

1. **Birimler.** Saat, gün, hafta, ay, yıl. Varsayılan birim gündür. Her tıklama bir adımdır.
2. **Saat adımı mutlaktır:** tam 3 600 saniye; yaz saati geçişinde de böyledir.
3. **Gün, hafta, ay ve yıl adımı duvar saatini korur.** An, tarayıcının saat dilimindeki takvim
   tarihine ve saatine çevrilir; takvimde yürünür; aynı duvar saati yeniden çözülür. Yaz saati
   geçişinden geçen bir gün adımı 23 ya da 25 saat sürer.
4. **Ay sonu.** Varılan ayda o gün yoksa ayın son günü alınır: 31 Ocak + 1 ay = 28 Şubat (artık
   yılda 29 Şubat); 29 Şubat ± 1 yıl = 28 Şubat. Bu yüzden ay adımı her zaman geri alınamaz:
   31 Ocak → 28 Şubat → 28 Ocak.
5. **Yaz saati boşluğu.** Varılan duvar saati yoksa (ör. 02:30, saatlerin 02:00'den 03:00'e
   atladığı gün) boşluk kadar ileri alınır: 03:30. Örtüşmede (aynı duvar saati iki kez) önceki an
   alınır. İkisi de transit anının v1 kuralıdır.
6. **Aralık.** 1–2999 yılları; dışına çıkan adım `invalid_transit_input { field: "range" }` verir
   ve an değişmez. Saat adımı da bu aralıktadır: sınır Jülyen gün sınırı değil, yıl aralığıdır
   (574914c); sonuç 0001 öncesine ya da 2999 sonrasına düşecekse adım reddedilir.
7. **Çıkış anı.** Transit deposunun anıdır: an bir Jülyen günüyse o, "şimdi" ya da yazılmış bir
   duvar saatiyse gösterilen yanıtın anı. İlk adımdan sonra an "şimdi" olmaktan çıkar.
8. **Eşzamanlılık.** Bir adım isteği sürerken yeni adım başlatılmaz. İstek sürerken an başka bir
   yoldan değişirse, harita değişirse ya da panel kapanırsa geç gelen yanıt düşürülür.

### 4.2 Onay bekleyen noktalar (sahibi)

- [ ] Gün ve hafta adımının duvar saatini koruması (öteki okuma: tam 24 ve 168 saat).
- [ ] Duvar saatinin tarayıcının diliminde okunması (öteki okuma: haritanın ya da bulunulan
      yerin dilimi).
- [ ] Ay sonunda ayın son gününe düşme ve boşlukta ileri alma.

### 4.3 Oynatma

1. **Oynat** seçili birimle ileri adım atar; **Duraklat** durdurur. Geri oynatma yoktur.
2. **Hız** iki adım arasındaki bekleme süresidir: Yavaş 2 sn, Orta 1 sn (varsayılan), Hızlı 0,4 sn.
3. **Sıra.** Sonraki adım, önceki an yerleşince planlanır: adım isteği bitmiş ve o anın
   transitleri gelmiş olmalıdır. Motor meşgulse oynatma bekler; iş birikmez ve oynatma sürerken
   harita açmak beklemez.
4. **Durma.** Duraklat düğmesi; panelin kapanması; başka bir haritanın açılması; adım ya da
   transit hatası (aralığın sonu dahil); sistemin "hareketi azalt" ayarının açılması.
5. **Elle seçilen an** (tarih kutuları, Şimdi, başka bir panelin satırı, adım düğmeleri) oynatmayı
   durdurmaz: oynatma o andan sürer.
6. **Hareketi azalt.** Sistem ayarı açıkken oynatma düğmesi ve hız seçici çizilmez; yerinde
   nedeni yazan bir satır durur. Adım düğmeleri çalışır.
7. Birim, hız ve oynatma durumu oturumluktur: düzenle birlikte saklanmaz (transit anı da
   saklanmaz).

### 4.4 Onay bekleyen noktalar (sahibi)

- [ ] Üç hız ve süreleri (2 sn, 1 sn, 0,4 sn); varsayılan hız.
- [ ] Geri oynatmanın olmaması.
- [ ] Elle seçilen anın oynatmayı durdurmaması.
- [ ] Denetimlerin yalnızca Transit tablosu panelinde durması (harita panelinde ayrı düğme yok).

## 5. Elle denetim kaydı (Plan 26 Task 8)

Her maddenin başında, otomatik bir karşılığı olup olmadığı yazılıdır. **[otomatik]** maddelerin
karşılığı testlerdedir; yürüten ajan testin adını göstererek işaretler. **[insan]** maddeler
yalnızca gerçek uygulamaya (ya da isterse basılı bir efemerise) bakan bir insan tarafından
doğrulanır ve sahibine devredilir; devir sonraki planı engellemez (Y3). Gerçek uygulamada yalnızca
deneme danışanları kullanılır.

- [x] **[otomatik]** Nakshatra lordları tablosu Vimshottari sırasının üç kez yazılmış biçimidir;
  hesap fonksiyonuyla ve doğum dasasının lorduyla 27 nakshatrada aynıdır; arayüzün okuduğu sabit,
  fikstürdeki Rust lordlarıyla aynıdır. (`lords.rs` üç test; `nakshatraLord.test.ts` üç test;
  `tests/bindings.rs`)
- [x] **[otomatik]** Geçiş tablosu her satırda nakshatra lordunu yazar; graha tablosunda sütun
  isteğe bağlıdır, varsayılan olarak gizlidir, lagna satırını da doldurur ve doğum saati
  bilinmiyorsa Ay'ın lordu "≈" taşır; olay aramasında yalnızca nakshatra girişi satırı lordu
  yazar ve sütun sayısı değişmez; hazır seçim aramayı başlatmaz. (`TransitTablePanel.test.tsx ›
  the nakshatra lord column …`; `GrahaTableNakshatraLord.test.tsx` beş test;
  `EventSearchLords.test.tsx` dört test)
- [x] **[otomatik]** Sankranti: bir yılda on iki giriş, aritmetiğin verdiği anlarda ve burç
  sırasıyla; giriş anında Güneş burcun başındadır; iki el örneği tithiyi, varayı ve Ay'ın
  nakshatrasını verir; liste tropikal-rasi kipinde de sidereal girişlerdir; aralıklar boşluksuz
  birleşir. (`sankranti.rs` yedi test)
- [x] **[otomatik]** Sankranti komutu gerçek motorla: 2026 için on iki giriş, ilki Ocak ortasında
  Oğlak; anlar ephemeris'in sidereal Güneş boylamıyla ve olay aramasının Güneş burç girişleriyle
  aynıdır; tithi, vara ve nakshatra v1'in panchangıdır; yıl verilmezse yerin bugünkü yılıdır;
  kutup gecesindeki giriş `SunriseUnavailable` uyarısı taşır; hata yalnızca alan adını taşır; bir
  yıl 100 ms'nin altındadır. (`tests/sankranti.rs`)
- [x] **[otomatik]** Efemeris tablosu: satır, anının dokuz grahasıdır ve v1'in transit hesabıyla
  iki zodyak kipinde aynıdır; giriş işareti girilen burcu, dönüş işareti dönülen yönü taşır;
  düğümlerin yönü işaretlenmez; kapatan an satır olmaz; bozuk anlar alan adıyla reddedilir; satır
  günün ilk geçerli yerel anında başlar ve zonun atladığı gün satır vermez (9105210).
  (`ephemeris_table.rs` yedi test)
- [x] **[otomatik]** Efemeris komutu gerçek motorla: Kasım 2026 otuz satırdır ve her satır yerin
  gece yarısındadır; konumlar transit komutuyla, işaretler `timeline` dizilerinin sınırlarıyla üç
  ayda, iki zodyak kipinde ve gerçek düğümle aynıdır; ay verilmezse yerin bugünkü ayıdır; önceki
  ve sonraki ay yıl sınırını geçer ve aralığın ucunda durur; yaz saati gece yarısında başlayan
  yerde gün 23 saattir; dosyaların dışındaki yıl `LowPrecision` taşır; bir ay 100 ms'nin
  altındadır. (`tests/ephemeris_table.rs`; `transit_tools_commands/ephemeris.rs ›
  months_step_across_the_year_and_stop_at_the_range_ends`)
- [x] **[otomatik]** Adım aritmetiği: saat adımı tam 3 600 saniyedir ve sonucu yıl aralığında
  tutar (574914c); gün ve hafta adımı duvar saatini korur (yaz saati geçişinde 23 saat; bir hafta
  ve bir saat); ay ve yıl adımı ayın son gününe düşer; boşluğa düşen duvar saati ileri alınır,
  örtüşmede önceki an alınır; hata yalnızca alan adını taşır; aralığın ucunda adım reddedilir.
  (`transit_tools_commands/step.rs` beş test)
- [x] **[otomatik]** Sankranti paneli: yer seçilmeden istek gitmez; "bu yıl" Rust'ta okunur; on
  iki satır yerin saatiyle yazılır; yazılan yıl gönderilince istenir, geçersiz yıl gönderilmez;
  başka yıl okunurken tablo yerinde durur; satırdaki an transit tarihini taşır; uyarılar ve
  tropikal harita notu sözle yazılır; hata alan adını arayüz dilinde söyler.
  (`SankrantiPanel.test.tsx` on test; e2e `sankranti: …`)
- [x] **[otomatik]** Efemeris paneli: bir gün bir satırdır; sütun başlıkları ekran okuyucuya tam
  adı okur; retro düz "R"dir ve düğümlerde yazılmaz; giriş ve dönüş işaretleri simge ve sözdür ve
  Rust'ın işaretlediği hücrelerdedir; "Nakshatra" kutusu istek başlatmaz ve düzenle birlikte
  saklanır; önceki, sonraki ve "bu ay" Rust'ın döndürdüğü aylarla çalışır; aralığın ucundaki
  düğme odakta durur ve bir şey yapmaz. (`EphemerisPanel.test.tsx` on test; e2e `ephemeris: …`)
- [x] **[otomatik]** Adım düğmeleri: adım Rust'a gösterilen an, birim ve tarayıcının dilimiyle
  sorulur; sonraki adım, transitleri gelmeden de varılan andan çıkar; istek sürerken tıklama yok
  sayılır; an başka bir yoldan değişmişse ya da bileşen kalkmışsa geç yanıt düşer; hata alan
  adını söyler ve yeniden denenir. (`TransitStepper.test.tsx` on bir test; e2e `transit steps: …`)
- [x] **[otomatik]** Oynatma: aralık başına bir adım; önceki anın transitleri gelmeden adım
  planlanmaz; hız ve birim seçilir; adım ya da transit hatası, bileşenin kalkması, başka harita
  ve "hareketi azalt" oynatmayı durdurur; elle seçilen an yeni çıkış noktasıdır; "hareketi azalt"
  açıkken oynatma düğmesi yoktur ve adım düğmeleri çalışır. (`TransitPlayback.test.tsx` on bir
  test; e2e `transit steps: …` iki akış)
- [x] **[otomatik]** İki yeni panel paletten eklenir ve çıkarılır; liste ve tablo panel açılmadan
  istenmez; var olan on beş görsel temel değişmez. (`panels.test.tsx`, `model.test.ts`; üç e2e
  dosyası; `pnpm e2e:visual`)
- [x] **[otomatik]** Her metin üç dilde vardır; Fransızca metinlerde düz kesme işareti yoktur.
  (`i18n.test.ts`; beş panel ve bileşen testinin dil testleri)
- [ ] **[insan]** Sankranti listesinin tropikal haritada da sidereal verilmesi onaylanır (2.2);
  varanın gün doğumuna göre okunması onaylanır.
- [ ] **[insan]** Efemeris satırının saati (yerel gece yarısı ya da 00:00 UT) ve varsayılan yer
  için karar verilir (3.2). İstenirse basılı bir efemerisle birkaç satır karşılaştırılır
  (karşılaştırırken satır saati ve ayanamsa eşitlenir).
- [ ] **[insan]** Gerçek uygulamada oynatma: üç hız okunur mu, varsayılan hız uygun mu; adım
  birimleri yeterli mi (4.4). Harita panelindeki geçiş halkası ile tablo birlikte ilerler;
  oynatma sürerken başka bir danışan açmak beklemez.
- [ ] **[insan]** Gerçek uygulamada macOS "Hareketi azalt" (Erişilebilirlik › Ekran) açılınca
  oynatma düğmesi gider ve satır görünür; oynatma sürerken açılınca oynatma durur.
- [ ] **[insan]** Gerçek uygulamada Efemeris paneli: on iki sütunluk panelde tablo okunur; dar
  panelde yana kayar; "Nakshatra" açıkken satır yüksekliği kabul edilir; açık ve koyu temada, üç
  dilde okunur.
- [ ] **[insan]** Gerçek uygulamada geçiş tablosu: yeni "Nakshatra lordu" sütunuyla tablo
  varsayılan panel genişliğine sığar; graha tablosunda sütun "Sütunlar"dan açılır.
- [ ] **[insan]** VoiceOver: iki yeni tablo başlıklarıyla duyurulur; efemeris işaretleri sözle
  okunur; oynatma sürerken "Gösterilen an" duyurusu rahatsız edici değildir (ediciyse karar
  olarak yazılır).
- [ ] **[insan]** Denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, doğum verisi,
  koordinat, yer ya da dosya yolu görünmez.

### 5.1 Yürütülen el denetiminin kaydı

Gözden geçiren bir insan değildir: görev gözden geçirmeleri, yazan oturumdan ayrı model
oturumlarıdır. Bu kayıt insan denetiminin yerine geçmez ve yukarıdaki **[insan]** maddeleri ile
onay noktaları açık kalır. Aşağıdaki değerler `python3` ve `zoneinfo` ile kuraldan, motorun
çıktısına bakılmadan yeniden türetilmiştir; yürüten ajan gerçek uygulamayı çalıştırmamış ve
sonuçları başka bir yazılımla ya da çevrimiçi kaynakla karşılaştırmamıştır.

- **Ay aritmetiği (`MonthId`).** Aralık → Ocak (yıl artar), Ocak → Aralık (yıl azalır), 0001/01 ve
  2999/12 uçlarında durma ve artık yılda Şubat'ın 29 gün olması.
- **Yaz saati ve atlanan günler (efemeris satırları).** São Paulo (2017-10-15: gece yarısı yok,
  satır 01:00 yerel), Apia (2011-12-30 yok: satır yazılmaz), Kiritimati (1994-12-31 yok: satır
  yazılmaz) ve Havana (saat geri alınırken 00:00 iki kez: önceki an). İlk geçerli an kuralı ve
  atlanan gün kuralı (9105210) bu değerlerle uyuşmuştur.
- **Adım aritmetiği.** 28 durum: Berlin'de boşluk ve örtüşme, 31 Ocak + 1 ay, 29 Şubat + yıllar,
  New York, Lord Howe (yarım saatlik yaz saati), Kiritimati ve 500. yıl. Saat adımının yıl
  aralığında kalması (574914c) ayrıca incelenmiştir.
- **Kapsam dışı.** Sankranti anlarının ve efemeris konumlarının astronomik doğruluğu bu kayıtta
  yeniden türetilmemiştir; o denetim motorun çapraz testlerine ve sahibine kalmıştır.

## 6. Gözden geçirme kaydı

| Tarih | Gözden geçiren | Kapsam | Sonuç |
|---|---|---|---|
| | | El örneklerinin kuraldan yeniden türetilmesi (Sankranti'nin iki girişi, efemerisin satır ve işaret örnekleri, adım aritmetiğinin takvim örnekleri) | |
| | | Sahibinin okuma seçimleri onayı (1.1, 2.2, 3.2, 4.2, 4.4) | |
