# Plan 01'den sonraki planlar için notlar

- **`SwissEph` `!Send`**: süreç çapında kilidin `MutexGuard`'ını tutar. Tauri'de (Plan 02) özel bir iş parçacığında sahiplenin, komutları kanallarla (channel) iletin. Çift açma kilitlenmesine karşı `try_open` / `AlreadyOpen` hatası eklemeyi değerlendirin.
- **Trait büyümeli**: mock'lar yazılmadan önce `Ephemeris` trait'ine evler (`swe_houses_ex` + sidereal) ve gündoğumu/batışı (`swe_rise_trans` + `SE_BIT_HINDU_RISING`) eklenmeli (Plan 04/05).
- **Sidereal mod SE'de global durum**: her çağrıda yeniden ayarlanıyor. 50 ms bütçesi sıkışırsa hesapları mod bazında toplu (batch) yapın.
- **Golden test yalnızca FFI tesisatını kanıtlar** (swetest ile aynı kütüphane). Plan 04 bağımsız referanslara ihtiyaç duyar: JHora, JPL Horizons nokta kontrolleri.
- **Veri dosyaları eksik**: spec §9 gereği 1200–1799 için `sepl_12` / `semo_12` eklenmeli (şu an yalnızca `_18`).
- **sefstars.txt**: SE 2.10.03'te Spica (True Chitra) yerleşik; `sefstars.txt` gerekmeyebilir. Gereksizse kaldırmayı değerlendirin.
- **Ayanamsa hassasiyet bayrağı**: SE `swe_get_ayanamsa_ex_ut` dosya aralığı dışında (yıl 3000) bile MOSEPH döndürmüyor (TrueChitra → `SwissEph`). Harita hassasiyetini Güneş/Ay konumlarından alın, düğümlerden (Rahu/Ketu) değil; Ketu = Rahu + 180°.
- **Ayanamsa nütasyon dahil**: `Ayanamsa.deg` `SEFLG_NONUT` verilmediği için Δψ içerir (J2000'de özel mod 23.5 → ~23.49613).
- **`repository` alanı**: düzeltildi (turgaydogusan/jyotish-desk).
