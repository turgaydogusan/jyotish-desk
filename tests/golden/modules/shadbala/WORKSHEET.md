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
