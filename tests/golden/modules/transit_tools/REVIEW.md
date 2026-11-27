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
