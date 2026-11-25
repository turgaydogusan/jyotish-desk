# Jyotish Desk kapalı beta rehberi

Denemeye katıldığınız için teşekkürler. Bu belge kurulum, gizlilik, güncelleme ve geri bildirim
için bilmeniz gerekenleri anlatır.

## Gereksinim

macOS 13 (Ventura) ya da sonrası; Apple silicon ya da Intel Mac.

## Kurulum

1. Davet iletisindeki `.dmg` dosyasını indirin ve açın.
2. Jyotish Desk'i Uygulamalar klasörüne sürükleyin ve oradan açın.
3. macOS "internetten indirildi, açmak istiyor musunuz?" diye sorar; "Aç" deyin. Başka bir uyarı
   ("geliştiricisi doğrulanamıyor" gibi) görürseniz açmayın ve bize bildirin.

## Verileriniz

- Danışanlarınız, haritalar ve notlar yalnızca bu Mac'te durur:
  `~/Library/Application Support/io.github.turgaydogusan.jyotishdesk/`.
- Uygulama hiçbir kullanım verisi, hata raporu ya da danışan bilgisi göndermez. Güncelleme
  denetimi dışında ağa bağlanmaz.
- Veritabanı şifrelenmemiştir. Mac'inizde **FileVault**'un açık olduğundan emin olun
  (Sistem Ayarları ▸ Gizlilik ve Güvenlik ▸ FileVault).
- Yedek: kenar çubuğu ▸ Veri ▸ "JSON olarak dışa aktar" (notlarla birlikte). Beta süresince her
  güncellemeden önce bir yedek alın ve gerçek danışan kayıtlarınızın tek kopyasını bu uygulamada
  tutmayın.

## Güncelleme

Uygulama kendiliğinden güncelleme aramaz. İki yol vardır:

- **Elle:** üst çubuk ▸ "Hakkında" ▸ "Şimdi denetle". Yeni sürüm varsa notları okuyup
  "indir ve kur"a basın; kurulum bitince uygulamayı kapatıp yeniden açın.
- **Açılışta:** aynı pencerede "Açılışta güncellemeleri denetle"yi işaretleyin. Beta boyunca bunu
  açmanızı öneririz. Denetim github.com'a tek bir istek gönderir; istekte sizinle ya da
  danışanlarınızla ilgili hiçbir bilgi yoktur.

İndirilen her paket, uygulamaya gömülü anahtarla imza denetiminden geçmeden kurulmaz.

## Neye bakmanızı istiyoruz

1. **Doğruluk:** iyi bildiğiniz birkaç haritayı girin; graha konumlarını, lagnayı, vargaları,
   Vimshottari dasa tarihlerini ve panchangı kullandığınız kaynakla karşılaştırın. Farkı
   bildirirken hesap ayarlarınızı (ayanamsa, düğüm, ev sistemi) da yazın.
2. **Doğum girişi:** yer araması, eski tarihler, yaz saati geçişleri, saati bilinmeyen doğum.
3. **Günlük akış:** danışan açma (`⌘K`), notlar, etiketler, düzenler, transit.
4. **Baskı:** `⌘P` ile önizleme ve PDF.
5. **Eksikler:** çalışırken aradığınız ama bulamadığınız şeyler.

## Geri bildirim

Davet iletisindeki adrese yazın; `feedback-template.md` başlıklarını kullanmanız işimizi
kolaylaştırır. Lütfen:

- **Gerçek danışan adı, doğum verisi ya da not göndermeyin.** Sorunu ünlü bir kişinin herkesçe
  bilinen doğum verisiyle ya da uydurma bir kayıtla yeniden üretin.
- Ekran görüntüsü gönderecekseniz kenar çubuğunu kapatın (`⌘\`) ve açık haritanın deneme kaydı
  olduğundan emin olun.
- Sürümü yazın (Hakkında penceresinin ilk satırı) ve macOS sürümünüzü ekleyin.

## Uygulamayı kaldırma

Uygulamayı Uygulamalar klasöründen çöpe atın. Verilerinizi de silmek isterseniz yukarıdaki veri
klasörünü silin (önce yedek alın).
