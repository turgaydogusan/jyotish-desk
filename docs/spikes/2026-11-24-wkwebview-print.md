# Spike: WKWebView print → PDF

**Durum:** Kod hazır; cihaz testi bekliyor (Plan 02 sonunda elle).
**Karar kuralı:** Aşağıdaki maddelerin hepsi geçerse M7 print düzenleri HTML/CSS ile yapılır.
1, 2, 3 ya da 5 numaralı maddeden biri kalırsa yedek plan devreye girer: Rust tarafında Typst ile PDF.

## Nasıl test edilir

1. `cd app && pnpm tauri dev`
2. "Print spike" → "Print…" → print penceresinde "PDF" → "PDF olarak kaydet".
3. PDF'i Önizleme'de aç.

## Kontrol listesi

| # | Kontrol | Sonuç |
|---|---|---|
| 1 | `@page { size: A4 }` uygulanıyor (Önizleme > Araçlar > Denetçi: 210 × 297 mm) | |
| 2 | 15 mm kenar boşluğu uygulanıyor | |
| 3 | `break-after: page` ile tam 2 sayfa; tablo ikinci sayfada | |
| 4 | Harita SVG vektör kalıyor (yakınlaştırınca bulanıklaşmıyor) | |
| 5 | Metin seçilebilir; Türkçe karakterler (ğüşıöç İ) doğru | |
| 6 | Devanagari (राशि) doğru biçimleniyor | |
| 7 | Düğmeler (`.no-print`) PDF'te yok | |
| 8 | Koyu modda bile arka plan beyaz, metin siyah | |
| 9 | Print penceresinin üst/alt bilgi (URL, tarih) seçeneği kapatılabiliyor | |
| 10 | Letter boyutu seçilince düzen bozulmuyor | |

## Bulgular

(Cihaz testinden sonra doldurulacak.)
