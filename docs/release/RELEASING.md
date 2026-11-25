# Yayın rehberi (macOS)

Bu belge deponun sahibi içindir. Bir sürümün etiketten kullanıcının Mac'ine nasıl ulaştığını ve
yalnızca bir insanın yapabileceği adımları anlatır. Otomatik bölüm `.github/workflows/release.yml`
ve `scripts/release/` altındadır.

**Kural:** aşağıdaki sırların hiçbiri depoya, bir issue'ya, bir sohbet penceresine ya da bir
yapay zekâ aracına yazılmaz. Sırlar yalnızca sizin terminalinizde, parola yöneticinizde ve GitHub'ın
sır deposunda durur. Kod yazan ajanlar bu adımları yapamaz ve yapmamalıdır.

## Akış

1. Sürüm üç dosyada ve `Cargo.lock`'ta aynıdır; `docs/release/notes/v<sürüm>.md` yazılmıştır
   (`node scripts/release/check-version.mjs` sürümü yazdırır).
2. `v<sürüm>` etiketi GitHub'a gönderilir.
3. `release` işi evrensel (arm64 + x86_64) paketi derler.
   - Sekiz sır da varsa: Developer ID ile imzalar, Apple'a notarize ettirir, `.dmg`'yi ayrıca
     notarize eder ve damgalar, güncelleme paketini (`.app.tar.gz`) güncelleme anahtarıyla imzalar,
     `latest.json` üretir ve hepsini **taslak** bir yayına yükler.
   - Sırlardan biri eksikse: imzasız bir deneme derlemesi yapar ve iş çıktısı olarak saklar;
     yayın oluşturmaz.
4. Siz taslağı denetler ve yayımlarsınız. Güncelleyici yalnızca yayımlanmış son yayını görür
   (`releases/latest/download/latest.json`).

## Sırlar

| Ad (`release` ortamı) | İçerik |
|---|---|
| `APPLE_CERTIFICATE` | Developer ID Application sertifikası ve özel anahtarı, `.p12`, base64 |
| `APPLE_CERTIFICATE_PASSWORD` | `.p12` dışa aktarılırken verilen parola |
| `APPLE_SIGNING_IDENTITY` | `Developer ID Application: Ad Soyad (TAKIMKODU)` |
| `APPLE_API_KEY_ID` | App Store Connect API anahtarının kimliği (10 karakter) |
| `APPLE_API_ISSUER` | App Store Connect "Issuer ID" (UUID) |
| `APPLE_API_KEY_P8` | İndirilen `AuthKey_<kimlik>.p8` dosyasının içeriği |
| `TAURI_SIGNING_PRIVATE_KEY` | Güncelleme özel anahtarı dosyasının içeriği |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | O anahtarın parolası |

## İNSAN ADIMI 1 — Apple Developer Program

1. <https://developer.apple.com/programs/enroll/> adresinden bireysel ya da kurumsal üyelik alın
   (yıllık ücretlidir; onay birkaç gün sürebilir).
2. <https://developer.apple.com/account> ▸ "Membership details" altındaki **Team ID**'yi not edin.

## İNSAN ADIMI 2 — Developer ID Application sertifikası

1. Anahtar Zinciri Erişimi ▸ Sertifika Yardımcısı ▸ "Sertifika Yetkilisinden Sertifika İste…":
   e-posta ve ad girin, "Diske kaydedildi"yi seçin (`.certSigningRequest` dosyası oluşur).
2. <https://developer.apple.com/account/resources/certificates/add> ▸ **Developer ID Application**
   ▸ isteği yükleyin ▸ `.cer` dosyasını indirin ve çift tıklayarak anahtar zincirine ekleyin.
3. Kimliğin adını okuyun:

   ```bash
   security find-identity -v -p codesigning
   ```

   `Developer ID Application: … (TAKIMKODU)` satırındaki tırnak içindeki metin
   `APPLE_SIGNING_IDENTITY` değeridir.
4. Anahtar Zinciri Erişimi ▸ "My Certificates" kategorisi ▸ sertifikayı (özel anahtarıyla birlikte) seçin ▸
   "Dışa Aktar…" ▸ `.p12` ▸ güçlü, yalnızca bu iş için üretilmiş bir parola verin.
5. Sertifika beş yıl geçerlidir; bitiş tarihini takviminize yazın. `.p12` dosyasını sırları
   GitHub'a ekledikten sonra silin ya da şifreli bir yedekte tutun.

## İNSAN ADIMI 3 — Notarizasyon için App Store Connect API anahtarı

1. <https://appstoreconnect.apple.com/access/integrations/api> ▸ "Team Keys" ▸ "+" ▸ ad:
   `jyotish-desk-notary`, erişim: **Developer**.
2. `AuthKey_<kimlik>.p8` dosyasını indirin (yalnızca bir kez indirilebilir).
3. Aynı sayfadaki **Key ID** ve **Issuer ID** değerlerini not edin.

Apple kimliği parolası ve uygulamaya özel parola kullanılmaz; API anahtarı istenince iptal edilebilir.

## İNSAN ADIMI 4 — Güncelleme anahtar çifti

Güncelleme paketleri bu anahtarla imzalanır; uygulama gömülü açık anahtarla doğrulamadan hiçbir
paketi kurmaz. **Özel anahtar ya da parolası yitirilirse kurulu uygulamalar bir daha güncellenemez**
(kullanıcılar yeni `.dmg` indirmek zorunda olur). Anahtar çalınırsa aynı şey bilinçli olarak yapılır:
yeni anahtar, yeni `.dmg`.

1. Kendi terminalinizde (parolayı araç sorar; komut satırına yazmayın):

   ```bash
   mkdir -p ~/.tauri && chmod 700 ~/.tauri
   pnpm --dir app tauri signer generate -w ~/.tauri/jyotish-desk-updater.key
   chmod 600 ~/.tauri/jyotish-desk-updater.key
   ```

2. Parolayı parola yöneticinize kaydedin. Anahtar dosyasını çevrimdışı, şifreli bir yere yedekleyin.
3. Açık anahtarı (`~/.tauri/jyotish-desk-updater.key.pub` dosyasının tek satırı; sır değildir)
   `app/src-tauri/tauri.conf.json` içindeki `plugins.updater.pubkey` değerine yapıştırın ve
   denetleyin:

   ```bash
   cargo test -p jyotish-desk --lib update_commands
   git add app/src-tauri/tauri.conf.json
   git commit -m "chore(release): embed the updater public key"
   ```

   Bu commit'ten önce yapılan derlemelerde Hakkında penceresi "Bu derlemede güncelleme denetimi
   yok" der ve imzalı derleme betiği (`build.sh signed`) çalışmayı reddeder.

## İNSAN ADIMI 5 — GitHub ortamı ve sırlar

1. Depo ▸ Settings ▸ Environments ▸ "New environment" ▸ ad: `release`.
2. "Deployment branches and tags" ▸ "Selected branches and tags" ▸ kural: tür **Tag**, desen `v*`.
   Böylece sırlar yalnızca sürüm etiketlerinden başlayan işlere verilir.
3. Sekiz sırrı ortam sırrı olarak ekleyin. Değerler komut satırı bağımsız değişkeni olarak değil,
   dosyadan ya da istemden okunur:

   ```bash
   base64 -i DeveloperID.p12 | gh secret set APPLE_CERTIFICATE --env release
   gh secret set APPLE_CERTIFICATE_PASSWORD --env release          # değeri istem sorar
   gh secret set APPLE_SIGNING_IDENTITY --env release
   gh secret set APPLE_API_KEY_ID --env release
   gh secret set APPLE_API_ISSUER --env release
   gh secret set APPLE_API_KEY_P8 --env release < AuthKey_XXXXXXXXXX.p8
   gh secret set TAURI_SIGNING_PRIVATE_KEY --env release < ~/.tauri/jyotish-desk-updater.key
   gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD --env release
   gh secret list --env release
   ```

   Son komut yalnızca adları listeler; sekiz ad da görünmelidir.
4. Settings ▸ Actions ▸ General ▸ "Workflow permissions": "Read repository contents" seçili olsun
   (yayın işi yazma iznini kendisi ister).

## İNSAN ADIMI 6 — Ad ve marka taraması

Spec §10: çıkıştan önce marka taraması yapılır ve yasaklı adlar kullanılmaz. "Jyotish Desk" adını
TÜRKPATENT, EUIPO ve USPTO kayıtlarında (yazılım sınıfı, 9 ve 42) ve Mac yazılım dizinlerinde
arayın; sonucu ve tarihi bu belgenin sonuna bir satır olarak ekleyin. Ad değişirse `productName`,
paket kimliği ve bu rehberdeki dosya adları ilk yayından **önce** değişmelidir (kimlik değişince
veri klasörü de değişir).

## İNSAN ADIMI 7 — Etiket, denetim ve yayımlama

1. Ana dal yeşil olmalı; sürüm ve notlar hazır olmalı:

   ```bash
   node scripts/release/check-version.mjs      # ör. 0.9.0
   git tag -a v0.9.0 -m "Jyotish Desk 0.9.0"
   git push origin v0.9.0
   ```

2. Actions ▸ `release` işini izleyin. "Signing secrets are present: signed build." satırı görünmeli;
   `verify-bundle` her satırda `ok` yazmalı.
3. Releases ▸ taslak yayın: `Jyotish-Desk_<sürüm>_universal.dmg`, `.app.tar.gz`, `.app.tar.gz.sig`,
   `latest.json`, `SHA256SUMS` dosyalarının beşi de olmalı.
4. `.dmg`'yi **bu depoyu hiç derlememiş** bir Mac'e ya da yeni bir macOS kullanıcısına indirin
   (Safari ile; karantina bayrağı böyle konur). Açın, uygulamayı Uygulamalar'a sürükleyin, çift
   tıklayın. Gatekeeper yalnızca "internetten indirildi" sorusunu sormalı; "geliştiricisi
   doğrulanamıyor" uyarısı çıkarsa yayımlamayın.
5. Hakkında penceresinde sürümü ve "Şimdi denetle"nin "Kurulu sürüm güncel." dediğini görün
   (taslak yayımlanmadan önce bir önceki yayına göre).
6. Taslakta "Set as the latest release" işaretli, "Set as a pre-release" işaretsiz olmalı;
   "Publish release".

## İNSAN ADIMI 8 — Güncelleme provası (ilk betadan önce bir kez)

1. 0.9.0 yayımlanmış ve bir Mac'e `.dmg`'den kurulmuş olsun.
2. Sürümü 0.9.1 yapın (üç dosya, `Cargo.lock`, `docs/release/notes/v0.9.1.md`), etiketleyin,
   taslağı yayımlayın.
3. 0.9.0'da: Hakkında ▸ "Şimdi denetle" ▸ "Yeni sürüm: 0.9.1" ve notlar ▸ "0.9.1 sürümünü indir ve
   kur" ▸ "kuruldu" iletisi ▸ uygulamayı kapatıp açın ▸ Hakkında "Sürüm 0.9.1" demeli; danışanlar
   yerinde olmalı.
4. Olumsuz deneme: yayımlanmış `latest.json`'ı indirip `signature` alanının bir karakterini
   değiştirerek yeniden yükleyin; 0.9.0 "imza doğrulanamadı" demeli ve hiçbir şey kurmamalı. Sonra
   özgün dosyayı geri yükleyin.

## İNSAN ADIMI 9 — Kapalı beta (3–5 astrolog)

Katılımcıları siz seçer ve davet edersiniz. Davet iletisine `.dmg` adresini,
`docs/beta/beta-guide.md` belgesini ve geri bildirimin gönderileceği adresi koyun. Gelen geri
bildirimi `docs/beta/feedback-template.md` başlıklarıyla depoda issue olarak açarken danışan adı,
doğum verisi ve not içeriği yazmayın.

## Yerelde deneme

- İmzasız: `sh scripts/release/build.sh unsigned && sh scripts/release/verify-bundle.sh unsigned`
  (sır gerekmez; yaklaşık on dakika).
- İmzalı (isteğe bağlı, kendi Mac'inizde): sekiz ortam değişkenini kendi kabuğunuzda verip
  `sh scripts/release/build.sh signed`, ardından `.dmg` için işteki `notarytool` ve `stapler`
  komutları ve `sh scripts/release/verify-bundle.sh signed`.

## Sorun giderme

- **İş imzasız derledi:** `release` ortamında sekiz sırdan biri eksik ya da iş bir etiketten
  başlamadı (ortam kuralı sırları vermez).
- **Notarizasyon reddedildi:** iş günlüğündeki gönderim kimliğiyle
  `xcrun notarytool log <kimlik> --key … --key-id … --issuer …` ayrıntıyı verir.
- **"A public key has been found, but no private key":** `TAURI_SIGNING_PRIVATE_KEY` boş.
- **Sertifika ya da API anahtarı yenilendi:** ilgili sırları güncelleyin; güncelleme anahtarı
  bundan etkilenmez.

## Marka taraması kaydı

(İNSAN ADIMI 6 tamamlanınca buraya tarih ve sonuç yazılır.)
