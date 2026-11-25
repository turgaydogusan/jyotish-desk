# Yayın rehberi (macOS)

Bu belge deponun sahibi içindir. Bir sürümün etiketten kullanıcının Mac'ine nasıl ulaştığını ve
yalnızca bir insanın yapabileceği adımları anlatır. Otomatik bölüm `.github/workflows/release.yml`
ve `scripts/release/` altındadır.

**Kural:** aşağıdaki sırların hiçbiri depoya, bir issue'ya, bir sohbet penceresine ya da bir
yapay zekâ aracına yazılmaz. Sırlar yalnızca sizin terminalinizde, parola yöneticinizde ve GitHub'ın
sır deposunda durur. Kod yazan ajanlar bu adımları yapamaz ve yapmamalıdır.

Anahtar dosyaları (`.p12`, `.p8`, `.cer`, güncelleme anahtarı) **deponun dışında** durur; bu
rehberdeki örnekler `~/.jyotish-signing/` (izin `700`) ve `~/.tauri/` klasörlerini kullanır.
`.gitignore` bu uzantıları yine de dışlar, ama bu yalnızca ikinci bir emniyettir.

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
   "Dışa Aktar…" ▸ `.p12` ▸ güçlü, yalnızca bu iş için üretilmiş bir parola verin. Dosyayı
   deponun dışına kaydedin:

   ```bash
   mkdir -p ~/.jyotish-signing && chmod 700 ~/.jyotish-signing
   # dışa aktarırken hedef: ~/.jyotish-signing/DeveloperID.p12
   chmod 600 ~/.jyotish-signing/DeveloperID.p12
   ```
5. Sertifika beş yıl geçerlidir; bitiş tarihini takviminize yazın. `.p12` dosyasını sırları
   GitHub'a ekledikten sonra silin ya da şifreli bir yedekte tutun.

## İNSAN ADIMI 3 — Notarizasyon için App Store Connect API anahtarı

1. <https://appstoreconnect.apple.com/access/integrations/api> ▸ "Team Keys" ▸ "+" ▸ ad:
   `jyotish-desk-notary`, erişim: **Developer**.
2. `AuthKey_<kimlik>.p8` dosyasını indirin (yalnızca bir kez indirilebilir) ve
   `~/.jyotish-signing/` klasörüne taşıyın (`chmod 600`); İndirilenler'de ya da depoda bırakmayın.
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
   Önerilir: aynı sayfada "Required reviewers" ▸ kendinizi ekleyin. O zaman sırlara dokunan her iş
   (etiketle ya da elle başlasın) sizin onayınızı bekler; yanlışlıkla ya da ele geçirilmiş bir
   hesapla atılan etiket onaysız imza alamaz.
3. Sekiz sırrı ortam sırrı olarak ekleyin. Değerler komut satırı bağımsız değişkeni olarak değil,
   dosyadan ya da istemden okunur:

   ```bash
   base64 -i ~/.jyotish-signing/DeveloperID.p12 | gh secret set APPLE_CERTIFICATE --env release
   gh secret set APPLE_CERTIFICATE_PASSWORD --env release          # değeri istem sorar
   gh secret set APPLE_SIGNING_IDENTITY --env release
   gh secret set APPLE_API_KEY_ID --env release
   gh secret set APPLE_API_ISSUER --env release
   gh secret set APPLE_API_KEY_P8 --env release < ~/.jyotish-signing/AuthKey_XXXXXXXXXX.p8
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

1. Sürüm ve notlar hazır olmalı; **önce ana dal gönderilir**, CI yeşile döner, etiket ondan sonra
   atılır. İş, etiketin `origin/main` üzerindeki bir commit'i gösterdiğini denetler; ana dal
   gönderilmeden atılan etiket işi durdurur.

   ```bash
   node scripts/release/check-version.mjs      # ör. 0.9.0
   git push origin main                        # Actions ▸ ci yeşil olana dek bekleyin
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
5. Hakkında penceresinde sürümü görün ve "Şimdi denetle"ye basın. Taslak henüz yayımlanmadığı
   için güncelleyici yalnızca bir önceki yayını görür:
   - **İlk yayında** (yayımlanmış hiçbir sürüm yokken) `latest.json` bulunmaz; beklenen ileti
     "Güncelleme denetlenemedi (yayın bilgisi okunamadı)." olur. Bu aşamada hata değildir.
   - **Sonraki yayınlarda** beklenen ileti "Kurulu sürüm güncel."dir.
   - "Bu derlemede güncelleme denetimi yok" görürseniz açık anahtar gömülmemiştir (İNSAN ADIMI 4);
     yayımlamayın.

   Yayımladıktan sonra aynı düğme her iki durumda da "Kurulu sürüm güncel." demelidir.
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
- İmzalı (isteğe bağlı, kendi Mac'inizde): `build.sh signed` sekiz ortam değişkeni ister. Adlar
  Tauri CLI'nin okuduğu adlardır ve **ikisi GitHub sır adlarından farklıdır**:

  | Ortam değişkeni (`build.sh signed`) | GitHub sırrı | Değer |
  |---|---|---|
  | `APPLE_CERTIFICATE` | aynı ad | `.p12` dosyasının base64'ü |
  | `APPLE_CERTIFICATE_PASSWORD` | aynı ad | `.p12` parolası |
  | `APPLE_SIGNING_IDENTITY` | aynı ad | `Developer ID Application: Ad Soyad (TAKIMKODU)` |
  | `APPLE_API_KEY` | `APPLE_API_KEY_ID` | **Key ID** (10 karakter); anahtarın kendisi değil |
  | `APPLE_API_ISSUER` | aynı ad | Issuer ID (UUID) |
  | `APPLE_API_KEY_PATH` | yok (iş, `APPLE_API_KEY_P8` içeriğini geçici bir dosyaya yazar) | `.p8` dosyasının **tam yolu**; dosya deponun dışında durur |
  | `TAURI_SIGNING_PRIVATE_KEY` | aynı ad | güncelleme özel anahtarı dosyasının içeriği |
  | `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | aynı ad | o anahtarın parolası |

  Yeni bir terminal penceresinde (iş bitince pencereyi kapatın; parolalar komut satırına ve kabuk
  geçmişine yazılmaz, `read -rs` sorar):

  ```bash
  export APPLE_CERTIFICATE="$(base64 -i ~/.jyotish-signing/DeveloperID.p12)"
  read -rs APPLE_CERTIFICATE_PASSWORD && export APPLE_CERTIFICATE_PASSWORD
  export APPLE_SIGNING_IDENTITY="Developer ID Application: Ad Soyad (TAKIMKODU)"
  export APPLE_API_KEY="XXXXXXXXXX"                                   # Key ID
  export APPLE_API_ISSUER="00000000-0000-0000-0000-000000000000"      # Issuer ID
  export APPLE_API_KEY_PATH="$HOME/.jyotish-signing/AuthKey_XXXXXXXXXX.p8"
  export TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/jyotish-desk-updater.key)"
  read -rs TAURI_SIGNING_PRIVATE_KEY_PASSWORD && export TAURI_SIGNING_PRIVATE_KEY_PASSWORD

  sh scripts/release/build.sh signed
  # Tauri .app'i notarize eder ve damgalar; .dmg ayrıca gönderilir (işteki adımın aynısı).
  DMG="target/universal-apple-darwin/release/bundle/dmg/Jyotish Desk_$(node scripts/release/check-version.mjs)_universal.dmg"
  xcrun notarytool submit "$DMG" --key "$APPLE_API_KEY_PATH" --key-id "$APPLE_API_KEY" --issuer "$APPLE_API_ISSUER" --wait
  xcrun stapler staple "$DMG"
  sh scripts/release/verify-bundle.sh signed
  ```

  `build.sh` bir değer eksikse, `APPLE_API_KEY_PATH` bir dosyayı göstermiyorsa ya da açık anahtar
  gömülü değilse derlemeye başlamadan durur; değerleri yazdırmaz. Her derleme önceki paket
  klasörünü (`target/universal-apple-darwin/release/bundle`) siler.

## Sorun giderme

- **İş imzasız derledi:** `release` ortamında sekiz sırdan biri eksik ya da iş bir etiketten
  başlamadı (ortam kuralı sırları vermez).
- **İş imzalı derledi ama taslak yok:** iş elle başlatılmıştır (Actions ▸ "Run workflow"). Elle
  çalıştırma bir etiket üzerinde olsa bile yayın oluşturmaz; taslak yalnızca etiket gönderilince
  oluşur.
- **"Version, tag and branch" adımı durdu:** etiket sürümle aynı değil, yayın notu dosyası yok ya
  da etiketlenen commit `origin/main` üzerinde değil (önce ana dalı gönderin).
- **Notarizasyon reddedildi:** iş günlüğündeki gönderim kimliğiyle
  `xcrun notarytool log <kimlik> --key … --key-id … --issuer …` ayrıntıyı verir.
- **"A public key has been found, but no private key":** `TAURI_SIGNING_PRIVATE_KEY` boş.
- **Sertifika ya da API anahtarı yenilendi:** ilgili sırları güncelleyin; güncelleme anahtarı
  bundan etkilenmez.

## Marka taraması kaydı

(İNSAN ADIMI 6 tamamlanınca buraya tarih ve sonuç yazılır.)
