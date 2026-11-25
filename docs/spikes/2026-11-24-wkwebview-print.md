# Spike: WKWebView print → PDF

**Durum:** Kapandı (Plan 12, 2026-11-26). M7 baskı düzenleri HTML/CSS ile yapıldı; Typst yedeğine gerek görülmedi. Denetim artık `scripts/print-probe.sh` ile otomatiktir; aşağıdaki liste M0'daki elle denetimin tarihçesidir.
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

Ölçüm: macOS 27.2, WKWebView `printOperation(with:)`, `NSPrintInfo` kenarları 0 (wry 0.57.3 `print()` ile aynı çağrı), çıktı PDF. Sonda: `scripts/print-probe.sh`.

| # | Kontrol | Sonuç |
|---|---|---|
| 1 | `@page { size: A4 }` | Kâğıt boyutu yazdırma penceresinden gelir; A4 seçiliyken sayfa 595 × 842 pt. Düzenler Letter'a da sığar. |
| 2 | 15 mm kenar boşluğu | wry kâğıt kenarlarını 0 yapar; boşluk sayfanın iç dolgusudur (`.print-page { padding: 15mm }`). WebKit içeriği yaklaşık 1,067 kat büyütür: dolgu kâğıtta 16 mm olur. |
| 3 | `break-after: page` | Çalışır: her `.print-page` yeni kâğıtta başlar. Kabuk baskıda blok akışına döner (`shell.css`). |
| 4 | SVG vektör | Evet. |
| 5 | Metin seçilebilir, Türkçe karakterler | Adlı yazı tipiyle evet (Helvetica Neue). **Sistem fontu Türkçe `lang` altında metin katmanında noktalı "i"yi yitirir** ("Tarih" → "Tar!h"); kâğıtta sistem fontu kullanılmaz. |
| 6 | Devanagari | Kâğıtta Devanagari metin basılmıyor; denetlenmedi. |
| 7 | `.no-print` | Araç çubuğu ve üst çubuk PDF'te yok. |
| 8 | Koyu temada beyaz zemin | Evet (`.paper` ve `@media print` paleti). |
| 9 | Üst/alt bilgi | WKWebView yazdırma işi üst/alt bilgi eklemiyor. |
| 10 | Letter | Dört düzen dört sayfa; saati bilinmeyen haritada da. |

Ek bulgular:

- Kâğıdın CSS karşılığı: A4 744 × 1053 px, Letter 765 × 990 px. Düzenler bu bütçeyle tasarlanır; e2e testi aynı sınırı denetler.
- Yazdırma işinin adı, PDF'in başlık alanı ve "PDF olarak kaydet"in önerdiği dosya adı `document.title`'dan gelir; önizleme açıkken başlık "Danışan – Harita"dır.
