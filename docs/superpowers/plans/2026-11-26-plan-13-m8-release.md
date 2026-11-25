# Plan 13 — M8: imza, notarizasyon, güncelleme ve kapalı beta hazırlığı

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** M8'in depoda yapılabilen bölümünü kapatmak: `v<sürüm>` etiketi evrensel (arm64 + x86_64) macOS paketini üretir; sırlar varsa paket Developer ID ile imzalanır, notarize edilir ve imzalı güncelleme paketiyle birlikte taslak bir GitHub yayınına yüklenir, sırlar yoksa aynı iş imzasız bir deneme derlemesi yapar. Uygulama, yalnızca kullanıcının onayıyla güncelleme denetler ve imzası doğrulanan paketi kurar. Sertifika, notarizasyon anahtarı, güncelleme anahtarı ve GitHub sırları gibi yalnızca bir insanın yapabileceği adımlar "İNSAN ADIMI" olarak ayrılır ve sahibine bir rehberle devredilir.

**Architecture:**
- **Güncelleme Rust'tadır.** `tauri-plugin-updater` yalnızca `update_commands.rs` içinden (`UpdaterExt`) kullanılır; arayüz üç komut çağırır: `app_info`, `check_for_update`, `install_update(version)`. Eklentinin kendi IPC komutları yetenek dosyasında izinli değildir; CSP ve yetenek dosyası değişmez (D4, C2′).
- **Onay iki adımdır.** Denetim: Hakkında penceresindeki "Şimdi denetle" düğmesi ya da varsayılanı kapalı "Açılışta güncellemeleri denetle" tercihi (`localStorage`). Kurulum: bulunan sürüm için ayrı düğme; komut o sürümü ister, yayın değiştiyse kurmaz (D1, D2).
- **İmza zorunludur.** Paket, `tauri.conf.json`'a gömülü minisign açık anahtarıyla doğrulanır ve imzanın taşıdığı sürüm bildirilen sürümle aynı olmalıdır (`requireSignedVersion`). Açık anahtar boşken derleme çalışır ama güncelleme "yapılandırılmamış"tır (D5, D6).
- **Yayın hattı betiklerdedir.** `scripts/release/` (sürüm denetimi, derleme, paket denetimi, paketleme, `latest.json`) hem yerelde hem `release.yml` içinde aynı biçimde çalışır; iş yalnızca sırları ortam değişkenine bağlar ve adımları sıralar. İmzasız yol sır istemez ve bu plan yazılırken uçtan uca koşulmuştur (D13).
- **Testler:** Rust birim testleri (yapılandırma, hata eşlemesi, izinler, kilit), Vitest (tercih, açılış denetimi, Hakkında penceresi, kabuk), Playwright e2e (güncelleme akışı, IPC taklidi), Node'un kendi test koşucusu (yayın betikleri ve iş dosyasının güvenlik kuralları), `verify-bundle.sh` (üretilen paket).

**Tech Stack:** Rust 1.93+ (stable), Tauri 2.12.7, **`tauri-plugin-updater` 2.14.1** (yeni; D3), React 19, TypeScript 7, Vitest 5, Playwright 1.65.1, Node 24+ (`node --test`), POSIX `sh`, Xcode komut satırı araçları (`codesign`, `notarytool`, `stapler`, `spctl`, `lipo`), GitHub Actions (`macos-15`). **Yeni npm bağımlılığı yok.**

**Spec:** `docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md` §2 (dağıtım: notarize edilmiş DMG, App Store dışı; otomatik güncelleme kullanıcı onaylı), §8 (CI: etiket atılınca imzalı ve notarize edilmiş DMG), §9 (GeoNames atfı Hakkında ekranında), §10 (imza, en düşük macOS, güncelleme, isim kuralları, AGPL kaynak bağlantısı), §11 (M8), §12 (şifresiz veri: kullanıcı belgelerinde yazılır; tzdb bayatlaması: güncelleyici hafifletir).

**Ön koşul:** Taban commit `70e2324` (Plan 12 ve son inceleme düzeltmeleri `main`'de). Plan 12 Task 8 adım 4'ün elle denetim kapısı ilerleme notlarında "geçti" olarak kayıtlı olmalıdır; değilse Task 1 başlamaz. `main` bu commit'ten ilerlediyse Task 1 adım 1 tabanları yeniden ölçer ve sayaç zinciri o ölçümden yürütülür (farklar aynıdır).

## Kararlar

Muhafazakâr seçenek seçildi; her karar bir satır gerekçeyle. D3, D5, D10, D11 ve D13 bu plan yazılırken deponun bir kopyasında ölçülmüştür (macOS 27.2, Rust 1.99, Tauri CLI 2.12.6).

- **D1 — Güncelleme denetimi varsayılan olarak kapalıdır; kullanıcı ya Hakkında penceresinde "Şimdi denetle"ye basar ya da aynı penceredeki "Açılışta güncellemeleri denetle" tercihini açar. İlk açılışta soru sorulmaz. Kurulum ayrı bir onaydır ve sürüme bağlıdır.** Gerekçe: spec §10 "Kontrol kullanıcı onayıyla yapılır"; kapalı varsayılan, kullanıcı hiçbir şey yapmadıkça uygulamanın ağa hiç bağlanmamasını sağlar. İlk açılış sorusu bir pencere daha ve yanlışlıkla verilen onay demektir; beta katılımcılarına tercihi açmaları rehberde önerilir (Task 8).
- **D2 — Tercih `localStorage`'da `jyotish-desk.updates.v1` anahtarında durur ("auto" | "off"); okunamayan, tanınmayan ya da yazılamayan değer kapalı sayılır.** Gerekçe: tema, dil ve kenar çubuğu gibi bir arayüz tercihidir; `AppPrefs` şeması ve göç değişmez (C3). Onay kaydı yitirilirse sonuç "denetleme"dir, tersi değil.
- **D3 — Yeni bağımlılık: `tauri-plugin-updater = "=2.14.1"`, `default-features = false`, özellikler `native-tls` ve `zip`.** Gerekçe: spec §10 güncelleyiciyi adıyla ister; imza doğrulama, indirme ve yerine koyma elle yazılmaz. Lisans Apache-2.0 OR MIT (AGPL ile uyumlu). Ölçüm: varsayılan `rustls-tls` özelliği `ring`i ve CDLA-Permissive-2.0 lisanslı `webpki-root-certs`i getirir ve `cargo deny check` `licenses FAILED` verir; `native-tls` (macOS'ta Security.framework) ile `deny.toml` değişmeden `advisories ok, bans ok, licenses ok, sources ok` çıkar. `Cargo.lock`'a 25 paket eklenir (`minisign-verify`, `tar`, `native-tls`, `security-framework`, `zip` ve geçişli olanlar). `tauri-plugin-process` eklenmez (D7); npm tarafına `@tauri-apps/plugin-updater` eklenmez (D4).
- **D4 — Güncelleyici yalnızca Rust komutlarıyla sürülür; yetenek dosyası `["core:event:default"]` olarak durur, CSP değişmez; komutlar adres ya da dosya yolu almaz.** Gerekçe: C2′'nin aynısı: web görünümü ele geçirilse bile başka bir adresten paket indirtemez ya da imza denetimini atlatamaz. Eklenti IPC komutlarını kaydeder ama izin verilmediği için çağrılamazlar; bir Rust testi yetenek dosyasını ve CSP'yi kilitler.
- **D5 — Tek ve sabit adres: `https://github.com/turgaydogusan/jyotish-desk/releases/latest/download/latest.json`; adreste sürüm, hedef ya da başka bir değişken yoktur. `requireSignedVersion: true`; `dangerous*` anahtarları ve `allowDowngrades` yoktur. Ön sürüm (pre-release) yayını kullanılmaz; iş yayını taslak oluşturur, sahibi yayımlar.** Gerekçe: spec §10 (GitHub Releases, imzalı); isteğe sürüm konmaması GitHub'ın kurulu sürümleri ayırt etmesini önler (gizlilik: tek GET, telemetri yok). `latest.json` imzalı değildir; imzanın sürümü bağlaması, eski ve geçerli imzalı bir paketin yeni sürüm diye sunulmasını engeller (ölçüm: `tauri signer sign --app-version 0.9.0` imzanın güvenilir yorumuna `version:0.9.0` yazar; `tauri build` bunu kendiliğinden yapar). Depo herkese açıktır (ölçüm: `gh repo view`), `releases/latest` ön sürümleri ve taslakları göstermez.
- **D6 — Açık anahtar `tauri.conf.json` içinde `plugins.updater.pubkey` değeridir ve bu planda boştur; anahtar çiftini sahibi üretir (İNSAN ADIMI 4). Boşken `app_info.updaterConfigured` `false`'tur, denetim `update_unconfigured` döner ve Hakkında penceresi bunu yazar. `bundle.createUpdaterArtifacts` yapılandırmada yoktur; yalnızca imzalı derleme `--config` ile açar.** Gerekçe: özel anahtarı ve parolasını ajanlar üretemez ve göremez; açık anahtarın standart yeri burasıdır (Tauri CLI özel anahtarın bu açık anahtarla eşleştiğini derlemede denetler). Güncelleme paketi üretimi özel anahtar ister; varsayılanı kapalı tutmak her geliştirme ve CI derlemesini sırsız çalışır bırakır.
- **D7 — Uygulama kurulumdan sonra kendini yeniden başlatmaz; "yeni sürüm, uygulamayı kapatıp yeniden açınca çalışır" yazar.** Gerekçe: macOS'ta güncelleyici `.app` paketini yerinde değiştirir, çalışan süreç eski sürümle sürer. Zorla yeniden başlatma bekleyen otomatik kayıtları (C5, C17) riske atar ve `tauri-plugin-process` ister.
- **D8 — Güncelleme hataları kararlı kodlardır: `update_unconfigured`, `update_check_failed{reason}`, `update_install_failed{reason}`, `update_changed`, `update_busy`; `reason` beş değerden biridir (`network`, `signature`, `manifest`, `install`, `other`) ve arayüz dilinde gösterilir. Eklentinin hata metni taşınmaz.** Gerekçe: metin adres ve dosya yolu içerebilir (C1); kullanıcıya gereken ayrım "ağ mı, imza mı, yayın mı, yerine koyma mı"dır.
- **D9 — Yeni "Hakkında" penceresi: sürüm, lisans (AGPL-3.0-or-later), kaynak kod adresi, Swiss Ephemeris ve GeoNames atıfları, güncelleme bölümü. Adres bağlantı değil, seçilebilir metindir; üst çubukta düğmesi ve palette komutu vardır, kısayolu yoktur.** Gerekçe: spec §9 ve §10 atfı ve kaynak bağlantısını Hakkında ekranında ister ve ikili dağıtımdan önce gereklidir (AGPL, CC BY). Tarayıcı açmak `opener`/`shell` eklentisi ve yetenek izni ister; metin yeterlidir.
- **D10 — Beta hattı `0.9.x`'tir; sürüm tektir (çalışma alanı sürümü: `Cargo.toml`, `app/package.json`, `tauri.conf.json`, `Cargo.lock`) ve `MAJOR.MINOR.PATCH` biçimindedir (`-beta.N` eki yok). Bu plan sürümü `0.9.0` yapar.** Gerekçe: güncelleyici semver karşılaştırır; ek taşımayan sürümler `CFBundleShortVersionString` ile de aynıdır. Ölçüm: çalışma alanı sürümü `jyotish-core`'un `ENGINE_VERSION`'ıdır ve kapakta basılır; sürüm değişince `delhi.json` fikstürü yeniden üretilir ve iki baskı testi güncellenir (Task 5). Görsel temeller değişmedi (kapak dahil, eşik içinde).
- **D11 — Tek evrensel paket; `latest.json` içinde `darwin-aarch64` ve `darwin-x86_64` anahtarlarının ikisi de aynı `.app.tar.gz`'yi gösterir; yayın dosyalarının adları boşluksuzdur (`Jyotish-Desk_<sürüm>_universal.*`).** Gerekçe: spec §10 "Universal binary". Ölçüm: `tauri build --target universal-apple-darwin` bu depoda çalışır (`lipo -archs`: `x86_64 arm64`, 63 MB `.app`, 23 MB güncelleme paketi). GitHub yüklenen dosya adındaki boşluğu noktaya çevirir; adı betikte sabitlemek `latest.json` adresini kesinleştirir. İmza dosya içeriğini imzalar; ad değişikliği imzayı bozmaz.
- **D12 — İmza: Developer ID Application, hardened runtime (Tauri varsayılanı), yetki (entitlements) dosyası yok. Notarizasyon App Store Connect API anahtarıyladır (Apple kimliği parolası ve uygulamaya özel parola kullanılmaz). `.dmg` ayrıca notarize edilir ve damgalanır.** Gerekçe: spec §10. Uygulama JIT, ağ sunucusu ya da korumalı kaynak kullanmaz; en az yetki. API anahtarı dar yetkilidir ve tek başına iptal edilebilir. Tauri `.app`'i notarize edip damgalar; spec "notarize edilmiş DMG" dediği için disk görüntüsü de gönderilir ve `verify-bundle.sh signed` ikisini de Gatekeeper'a sorar.
- **D13 — Yayın ayrı bir iş dosyasıdır (`release.yml`): yalnızca `v*` etiketi ya da elle başlatma; sırlar `release` ortamında; sekiz sırdan biri eksikse imzasız deneme derlemesi yapılır, denetlenir ve yayın oluşturulmaz. Eylemler commit SHA'sıyla sabitlenir, önbellek kullanılmaz, `tauri-action` kullanılmaz. Mevcut `ci.yml` işleri yalnızca bir adım kazanır (yayın betiklerinin testi).** Gerekçe: sırlara dokunan iş, çekme isteklerinden ve paylaşılan önbellekten yalıtılır; sır olmadan da çalışan yol, ajanların ve sahibin hattı sertifika gelmeden denemesini sağlar. Üçüncü taraf bir eylemin sırları görmesi yerine Tauri CLI doğrudan çağrılır. Bir Node testi bu kuralları iş dosyasının metninde denetler.
- **D14 — Veri klasörü 0700, veritabanı ve SQLite yan dosyaları 0600 yapılır (Plan 10 kapsam dışı 3; yalnızca unix, Windows'ta işlem yok).** Gerekçe: danışan verisi şifresizdir (spec §12); çok kullanıcılı bir Mac'te varsayılan 0755/0644 başka kullanıcılara okuma verir. Kullanıcının seçtiği yere yazılan dışa aktarma dosyaları kapsam dışıdır.
- **D15 — Zehirlenmiş store kilidi bir sonraki çağrıda toparlanır: `Store::recover` açık işlem varsa geri alır, sonra zehir kaldırılır; atlas kilidi salt okunur olduğu için doğrudan sürdürülür (Plan 10 kapsam dışı 4).** Gerekçe: bugün tek bir panik uygulamayı kapanana dek `store_unavailable` durumunda bırakır; beta kullanıcısı için bu veri kaybı gibi görünür. Store işlemleri `rusqlite::Transaction` ile açılır ve panikte geri alınır; `recover` bunu ayrıca garanti eder.
- **D16 — Telemetri, çökme raporu ve tanı dışa aktarma eklenmez.** Gerekçe: spec bunları istemez; "veri bu Mac'te durur" sözü beta rehberinin temelidir. Geri bildirim elle yazılır (şablon: Task 8).
- **D17 — Açılıştaki denetim yeni sürüm bulursa bildirim bandına tek bir bilgi satırı düşer; bulamazsa ya da başarısız olursa hiçbir şey gösterilmez ve loglanmaz.** Gerekçe: çevrimdışı açılış sessiz olmalıdır; kurulum kararı pencerede verilir.

## Global Constraints

- Lisans: AGPL-3.0-or-later. Tek yeni bağımlılık D3'tür; `deny.toml` değişmez ve `cargo deny check` Task 2 ve 9'da `advisories ok, bans ok, licenses ok, sources ok` verir. `app/package.json` yalnızca sürüm satırında değişir; `pnpm-lock.yaml` değişmez.
- **Yasaklı ad kuralı (v1 spec §10 "İsim kuralları", ilk madde):** o maddedeki dört ad ve türevleri hiçbir isimde, yorumda, metinde, dosya adında ya da commit mesajında kullanılmaz. Kural harf dizisi düzeyindedir: Türkçe sözcüklerin içinde de geçemez (yerine ör. "diğer", "yarıda kesilen", "durur" yazılır; "sertifika" sözcüğünün çoğulu da eşleşir, tekil kullanılır). Her commit adımı eklenen satırları, dosya adlarını ve commit mesajını `grep -inE 'k(a|ā|aa)la|\bcala\b'` ile tarar; çıktı boş olmalıdır (C13). Bu belgede de eşleşme yoktur.
- **Sırlar (C18, bu planın kuralı):** Yürütücü ajanlar Apple sertifikası, Apple kimliği parolası, uygulamaya özel parola, notarizasyon API anahtarı, güncelleme özel anahtarı ya da parolası **üretmez, istemez, girmez ve yazdırmaz**; `gh secret`, `security`, `notarytool store-credentials` ve `tauri signer generate` komutlarını çalıştırmaz; `git tag` ve `git push` yapmaz. Bunlar "İNSAN ADIMI"dır (aşağıdaki tablo; ayrıntı `docs/release/RELEASING.md`, Task 8). Bir adım bir sır isterse ajan durur ve sahibine bildirir. Depoya hiçbir anahtar, `.p12`, `.p8` ya da parola girmez; `plugins.updater.pubkey` açık anahtardır ve yalnızca sahibi tarafından doldurulur.
- Kapılar: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `pnpm --dir app test`, `pnpm --dir app build` her görev sonunda yeşil; `pnpm --dir app e2e` Task 4 ve 9'da; `node --test "scripts/release/*.test.mjs"` Task 5'ten itibaren. `tsc` `strict`.
- **Toolchain:** `export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"` (oturum başına bir kez). Node 24+, pnpm 11.8.0, `cargo-deny`. Task 6 için `rustup target add x86_64-apple-darwin` kurulu olmalıdır (CI'ın `macos-x86_64` işi için zaten gerekir).
- **Çalışan `pnpm tauri dev` durdurulmaz** (port 1420); e2e 1421'i kullanır. `pkill -f` kullanılmaz. Üretilen `.app` **çalıştırılmaz**: gerçek veri klasörünü (`~/Library/Application Support/io.github.turgaydogusan.jyotishdesk/`) açar; paket yalnızca `verify-bundle.sh` ile denetlenir. Çalıştırarak denetim elle denetim kapısındadır (Task 9).
- Yalnızca macOS hedeflenir: Windows ve Android imzası ya da paketi yoktur; `ci.yml`'deki `core (ubuntu-latest, windows-latest)`, `deny`, `msrv` ve `macos-x86_64` işleri değişmez. Yorum metni (astrolojik yorum) eklenmez.
- **Tabanlar (`70e2324`):** Vitest `Test Files  57 passed (57)`, `Tests  500 passed (500)`; `jyotish-core` lib 110, golden 8; `store` lib 80 (+1 `#[ignore]`); `jyotish-desk` lib 34, `bindings` 1, `compute_chart` 1, `open_client` 1, `ui_fixture` 2; e2e 11; görsel 10 (yerel, darwin); yayın betiği testi yok.
- **Sayaç zinciri:** Task 1: `store` lib 81, `jyotish-desk` lib 38. Task 2: `jyotish-desk` lib 44, Vitest 57/501. Task 3: Vitest 60/517. Task 4: e2e 12. Task 5: yayın betiği testleri 4. Task 7: yayın betiği testleri 9. Diğer sayılar değişmez.
- **Taşınan kurallar (Plan 09 C1–C14, Plan 10 C2′, C15–C17), her görevin gereksinimidir:**
  - **C1** Gizlilik: danışan adları, not gövdeleri, doğum verisi, dosya yolları ve `CommandError.params` loglanmaz; `no-console.test.ts` durur ve izin listesi büyümez. Güncelleme isteği yalnızca D5'teki adrese gider ve hiçbir kullanıcı ya da danışan verisi taşımaz; eklentinin hata metni arayüze taşınmaz (D8). Yayın notlarına ve betik çıktılarına kişisel veri yazılmaz.
  - **C2′** Hiçbir IPC komutu arayüzden dosya yolu ya da adres almaz. `install_update`'in tek dizgi bağımsız değişkeni kullanıcının onayladığı sürümdür; `api.test.ts`'in dizgi bağımsız değişken listesi bunu kaydeder. Yetenek dosyası `updater:`, `dialog:`, `fs:` izni vermez.
  - **C3** Göç V1 donmuştur; bu plan şemaya, `AppPrefs`'e ve `ViewState`'e dokunmaz (D2).
  - **C4** Arayüz astrolojik hesap yapmaz. **C5/C17** Otomatik kayıtlar ve boşaltılmaları değişmez; uygulama kendini yeniden başlatmaz (D7). **C6** Düzen ayrıştırıcıları değişmez.
  - **C7** Kısayollar yalnızca `shortcuts.ts`'tedir; bu plan kısayol eklemez. **C8** Her pencere `shell/Dialog`'u kullanır (Hakkında dahil).
  - **C9** Renk yalnızca token'lardan gelir (`ui/about` dahil; hex yok) ve tek başına anlam taşımaz.
  - **C10** Üretilen dosyalar elle düzenlenmez: `bindings.ts` Task 2'de, `delhi.json` Task 5'te komutla yeniden üretilir.
  - **C11** Görsel PNG temelleri yereldir; bu planda değişmeleri beklenmez (Task 5 ve 9 doğrular).
  - **C12** Yeni `CommandError.code`'lar (D8) üç dilde çevrilidir ve `i18n.test.ts`'in `BACKEND_CODES` listesindedir.
  - **C13** Yasaklı ad taraması her commit'te; depo dışındaki ilerleme notları ve `.superpowers/` hiçbir zaman `git add` edilmez.
  - **C14** Plan 07'nin gün doğumu toleransı değişmez. **C15** `alert`, `confirm`, `prompt`, `window.print()` ve `window.open()` çağrılmaz. **C16** ⌘ kısayolları değişmez.
- **Erişilebilirlik:** WCAG AA; Hakkında penceresinde her işlem klavyeyle yapılır; odak `body`'ye düşmez (kurulum bitince "Kapat" düğmesine taşınır); süren işlemde düğmeler yerel `disabled` değil `aria-disabled`'dır ve durum satırına `aria-describedby` ile bağlıdır; durum `role="status"`, hata `role="alert"`tir.
- Commit mesajları İngilizcedir ve deponun `Co-Authored-By:` satırıyla biter (yürütücünün oturum yönergesindeki satır). Her görev tek commit'tir (gerekirse inceleme düzeltmesi ayrı commit). `git push`, `git tag`, yıkıcı git komutu ve `.superpowers/` commit'i yoktur.
- Kod blokları `70e2324` ağacına göre yazılmış ve bu plan yazılırken deponun bir kopyasında koşulmuştur (Self-Review madde 5). Yeni dosyalar tam içerikle verilir. Değişen dosyalar `diff` bloğuyla verilir: blok bir dosyaya konup depo kökünde `git apply <dosya>` ile ya da elle uygulanır; bağlam tutmazsa önceki görevin adımı denetlenir, elle uydurma yapılmaz. `Cargo.lock`, `app/ui/bindings.ts` ve `app/ui/test/fixtures/delhi.json` diff olarak verilmez; adımlardaki komutlar üretir.

## İnsan adımları (özet)

Ajanlar bu adımları yapmaz; hepsi `docs/release/RELEASING.md` içinde ayrıntılıdır (Task 8) ve Task 9 sahibine devreder. Otomatik görevlerin hiçbiri bu adımların tamamlanmasını beklemez.

| # | İNSAN ADIMI | Ürettiği | Olmadan ne olur |
|---|---|---|---|
| 1 | Apple Developer Program üyeliği | Team ID | İmzalı yayın yapılamaz; her şey imzasız yolda çalışır |
| 2 | Developer ID Application sertifikası; `.p12` dışa aktarma | `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY` | aynı |
| 3 | App Store Connect API anahtarı (notarizasyon) | `APPLE_API_KEY_ID`, `APPLE_API_ISSUER`, `APPLE_API_KEY_P8` | aynı |
| 4 | Güncelleme anahtar çifti ve parolası; açık anahtarın `tauri.conf.json`'a konması | `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, `plugins.updater.pubkey` | Hakkında "Bu derlemede güncelleme denetimi yok" der |
| 5 | GitHub `release` ortamı (etiket kuralı `v*`) ve sekiz sır | Sırların işe verilmesi | `release` işi imzasız deneme derler |
| 6 | Ad ve marka taraması (spec §10) | Kayıt satırı | Yayımlama ertelenir |
| 7 | `v0.9.0` etiketi, taslağın temiz bir Mac'te denenmesi, yayımlama | İlk yayın | Beta başlamaz |
| 8 | Güncelleme provası (0.9.0 → 0.9.1; bozulmuş imzanın reddi) | Güncelleyicinin gerçek kanıtı | Güncelleme yalnızca taklitle sınanmış olur |
| 9 | 3–5 astrologun seçilmesi, daveti ve geri bildirimin toplanması | Beta | — |

## Dosya Yapısı (bu plan)

```
crates/store/src/db.rs                          Store::recover (açık işlemi geri alır)
app/src-tauri/
  Cargo.toml                                    tauri-plugin-updater (D3)
  tauri.conf.json                               plugins.updater; sürüm 0.9.0
  src/lib.rs                                    eklenti, InstallGuard, üç komut, veri klasörü izinleri
  src/data_dir.rs                               0700 / 0600 (yeni)
  src/update_commands.rs                        app_info, check_for_update, install_update (yeni)
  src/client_commands.rs                        lock(): zehirli kilidi toparlar
  src/atlas_commands.rs                         zehirli atlas kilidi sürdürülür
app/ui/
  bindings.ts                                   üretilir (AppInfo, UpdateInfo, üç komut)
  api.ts, api.test.ts                           üç sarmalayıcı
  update/{pref.ts, startup.ts} + testleri       tercih (varsayılan kapalı), açılış denetimi (yeni)
  about/{AboutDialog.tsx, about.css} + testi    Hakkında penceresi (yeni)
  shell/{Shell.tsx, TopBar.tsx}                 Hakkında düğmesi, açılış denetimi
  palette/{items.ts, Palette.tsx}               "about" komutu
  i18n/{tr,en,fr}.ts, errors.ts, i18n.test.ts   metinler, hata kodları, nedenler
  App.test.tsx                                  onaysız denetim yok; Hakkında; palet
  print/{model,PrintDocumentView}.test.ts(x)    sürüm 0.9.0
  test/fixtures/delhi.json                      üretilir (engine_version 0.9.0)
app/e2e/{ipc.ts, desk.spec.ts}                  güncelleme taklidi ve akışı
scripts/release/
  check-version.mjs (+test)                     sürüm tutarlılığı, etiket, yayın notu
  latest-json.mjs (+test)                       latest.json üreticisi
  build.sh                                      unsigned | signed
  verify-bundle.sh                              paket denetimi (unsigned | signed)
  package.sh                                    yayın dosyaları, latest.json, SHA256SUMS
  workflow.test.mjs                             release.yml güvenlik kuralları
.github/workflows/release.yml                   etiket → paket → (sır varsa) imza, notarizasyon, taslak yayın
.github/workflows/ci.yml                        + yayın betiği testleri adımı
docs/release/{RELEASING.md, release-notes-template.md, notes/v0.9.0.md}
docs/beta/{beta-guide.md, feedback-template.md}
README.md, docs/superpowers/specs/…-v1-design.md  yayın bölümü; §10 güncelleme ayrıntısı
Cargo.toml, Cargo.lock, app/package.json        sürüm 0.9.0
```

### Kapsam

| Madde | Yer |
|---|---|
| Spec §8/§10: etiket → imzalı, notarize edilmiş evrensel DMG | Task 6, 7; İNSAN ADIMI 1–3, 5, 7 |
| Spec §10: Tauri updater, imzalı, GitHub Releases, kullanıcı onayı | Task 2, 3, 4 (D1–D8); İNSAN ADIMI 4, 8 |
| Gizlilik: telemetri yok, tek adres | D5, D16; Task 2 (yapılandırma testi), Task 3 (onaysız istek yok testi) |
| Spec §9/§10: Hakkında ekranında GeoNames atfı ve AGPL kaynak adresi | Task 3 (D9) |
| Spec §10: en düşük macOS 13, evrensel ikili | Task 6 (`verify-bundle.sh`) |
| Spec §12: şifresiz veri kullanıcı belgelerinde yazılır | Task 8 (beta rehberi, yayın notu) |
| Plan 10 kapsam dışı 3 (0700/0600) ve 4 (zehirli kilit) | Task 1 (D14, D15) |
| Kapalı beta hazırlığı: sürüm, yayın notu ve şablonu, beta rehberi, geri bildirim başlıkları | Task 5, 8 |
| Sırların insan adımlarına ayrılması; sırsız çalışan otomasyon | C18, D6, D13; Task 6, 7, 8, 9 |
| Son doğrulama ve elle denetim | Task 9 |

---

### Task 1: Veri klasörü izinleri ve zehirlenmiş kilitten kurtulma

Plan 10 kapsam dışı 3 ve 4 (Plan 08 ertelenen madde 10 ve madde 11'in ikinci yarısı). Davranış değişiklikleri: veri klasörü 0700, veritabanı ve `-wal`/`-shm` dosyaları 0600 olur (var olan kurulumlarda da açılışta daraltılır); kilidi tutan bir iş panik ettikten sonraki ilk store çağrısı kilidi toparlar ve çalışır.

**Files:**
- Create: `app/src-tauri/src/data_dir.rs`
- Modify: `crates/store/src/db.rs`, `app/src-tauri/src/client_commands.rs`, `app/src-tauri/src/atlas_commands.rs`, `app/src-tauri/src/lib.rs`
- Test: aynı dosyalardaki `mod tests` blokları

**Interfaces:**
- Consumes: `store::Store`, `client_commands::{StoreState, DB_FILE_NAME, lock}`, `CommandError`.
- Produces: `Store::recover(&mut self) -> Result<(), StoreError>`; `data_dir::prepare(dir: &Path) -> io::Result<()>`, `data_dir::restrict_db_files(db: &Path) -> io::Result<()>`; `lock()` imzası değişmez (zehirli kilitte artık `Ok`).

- [ ] **Step 1: Tabanları ölç**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
git log -1 --format=%h
git status --short
cargo test -p store -p jyotish-desk 2>&1 | grep -E "^test result"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
```
Expected: `git status --short` boş; `store` lib `80 passed; 0 failed; 1 ignored`, `jyotish-desk` lib `34 passed`; Vitest `57 passed (57)`, `500 passed (500)`. Commit `70e2324` değilse ölçülen sayılar ilerleme notlarına taban olarak yazılır ve zincir oradan yürütülür.

- [ ] **Step 2: Başarısız testleri yaz**

Aşağıdaki diff'in yalnızca `mod tests` içindeki eklemelerini uygula (`db.rs`'te `recover_rolls_back_…`, `client_commands.rs`'te `a_poisoned_store_lock_is_recovered_…`):

```diff
diff --git a/app/src-tauri/src/atlas_commands.rs b/app/src-tauri/src/atlas_commands.rs
--- a/app/src-tauri/src/atlas_commands.rs
+++ b/app/src-tauri/src/atlas_commands.rs
@@ -130,7 +130,8 @@ pub fn search_places_blocking(
     }
     let places = atlas
         .lock()
-        .map_err(|_| CommandError::new("atlas_unavailable", &[("reason", "io".to_owned())]))?
+        // Atlas salt okunurdur: zehirli kilit veriyi bozmuş olamaz, arama sürer.
+        .unwrap_or_else(std::sync::PoisonError::into_inner)
         .search(&query, limit)?;
     Ok(places
         .into_iter()
diff --git a/app/src-tauri/src/client_commands.rs b/app/src-tauri/src/client_commands.rs
--- a/app/src-tauri/src/client_commands.rs
+++ b/app/src-tauri/src/client_commands.rs
@@ -125,10 +125,22 @@ fn clip(text: &str) -> String {
     }
 }
 
+/// Store kilidi. Kilidi tutan bir iş panik ettiyse (`blocking` bunu `task_failed` olarak
+/// bildirir) kilit zehirlenir; sonraki çağrı bağlantıyı toparlar (`Store::recover`) ve zehri
+/// kaldırır, böylece tek bir panik uygulamayı kapanana dek kullanılmaz bırakmaz (Plan 08
+/// ertelenen madde 11). Toparlama başarısızsa kilit zehirli durur ve `store_unavailable` döner.
 pub(crate) fn lock(store: &Mutex<Store>) -> Result<std::sync::MutexGuard<'_, Store>, CommandError> {
-    store
-        .lock()
-        .map_err(|_| CommandError::new("store_unavailable", &[]))
+    match store.lock() {
+        Ok(guard) => Ok(guard),
+        Err(poisoned) => {
+            let mut guard = poisoned.into_inner();
+            guard
+                .recover()
+                .map_err(|_| CommandError::new("store_unavailable", &[]))?;
+            store.clear_poison();
+            Ok(guard)
+        }
+    }
 }
 
 /// `open_client` gövdesi: kayıt (store kilidi kısa tutulur), sonra motor iş parçacığında
@@ -362,6 +374,21 @@ mod tests {
         assert!(e.params.is_empty());
     }
 
+    #[test]
+    fn a_poisoned_store_lock_is_recovered_on_the_next_call() {
+        let store: StoreState = Arc::new(Mutex::new(Store::open_in_memory().unwrap()));
+        let held = Arc::clone(&store);
+        let _ = std::thread::spawn(move || {
+            let _guard = held.lock().unwrap();
+            panic!("job failed while holding the store");
+        })
+        .join();
+        assert!(store.is_poisoned());
+        let prefs = lock(&store).unwrap().app_prefs().unwrap();
+        assert_eq!(prefs, store::AppPrefs::default());
+        assert!(!store.is_poisoned());
+    }
+
     #[test]
     fn store_error_codes_are_stable() {
         let e = CommandError::from(StoreError::NotFound {
diff --git a/app/src-tauri/src/lib.rs b/app/src-tauri/src/lib.rs
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -1,6 +1,7 @@
 pub mod atlas_commands;
 pub mod client_commands;
 pub mod commands;
+pub mod data_dir;
 pub mod store_commands;
 pub mod transit_commands;
 
@@ -74,8 +75,11 @@ pub fn run() {
             // Danışan veritabanı: ~/Library/Application Support/<identifier>/. Açılamazsa
             // (ör. daha yeni bir sürümün veritabanı) setup hatası, motorla aynı yol.
             let data_dir = app.path().app_data_dir()?;
-            std::fs::create_dir_all(&data_dir)?;
-            let db = store::Store::open(&data_dir.join(client_commands::DB_FILE_NAME))?;
+            // Klasör 0700, veritabanı dosyaları 0600 (Plan 08 ertelenen madde 10).
+            data_dir::prepare(&data_dir)?;
+            let db_path = data_dir.join(client_commands::DB_FILE_NAME);
+            let db = store::Store::open(&db_path)?;
+            data_dir::restrict_db_files(&db_path)?;
             let state: client_commands::StoreState = Arc::new(Mutex::new(db));
             app.manage(state);
             // Atlas ölümcül değildir: açılamazsa yer araması `atlas_unavailable` döner, elle
diff --git a/crates/store/src/db.rs b/crates/store/src/db.rs
--- a/crates/store/src/db.rs
+++ b/crates/store/src/db.rs
@@ -72,6 +72,17 @@ impl Store {
         format_ts(Timestamp::from_millisecond(ms).unwrap_or(Timestamp::MAX))
     }
 
+    /// Yarıda kesilmiş bir işin ardından bağlantıyı kullanılır duruma getirir: açık bir işlem
+    /// varsa geri alınır. `Mutex<Store>` bir panikle zehirlendiğinde çağrılır (Plan 08 ertelenen
+    /// madde 11). Bu crate'in işlemleri `rusqlite::Transaction` ile açılır ve panikte kendiliğinden
+    /// geri alınır; buradaki denetim, bağlantının gerçekten işlem dışında olduğunu garanti eder.
+    pub fn recover(&mut self) -> Result<(), StoreError> {
+        if !self.conn.is_autocommit() {
+            self.conn.execute_batch("ROLLBACK")?;
+        }
+        Ok(())
+    }
+
     /// Veritabanının şema sürümü (`PRAGMA user_version`).
     pub fn schema_version(&self) -> Result<usize, StoreError> {
         let v: i64 = self
@@ -159,6 +170,19 @@ mod tests {
         assert_eq!(s.now(), "2025-11-24T16:00:00.002Z");
     }
 
+    #[test]
+    fn recover_rolls_back_an_open_transaction_and_is_a_no_op_otherwise() {
+        let mut s = store();
+        s.recover().unwrap();
+        s.conn
+            .execute_batch("BEGIN; INSERT INTO app_state (key, value_json) VALUES ('half', '1');")
+            .unwrap();
+        assert!(!s.conn.is_autocommit());
+        s.recover().unwrap();
+        assert!(s.conn.is_autocommit());
+        assert_eq!(s.app_state_get("half").unwrap(), None);
+    }
+
     #[test]
     fn fresh_database_has_schema_and_foreign_keys() {
         let s = store();
```

- [ ] **Step 3: Testlerin başarısız olduğunu doğrula**

Run: `cargo test -p store --lib recover 2>&1 | tail -5; cargo test -p jyotish-desk --lib poisoned 2>&1 | tail -8`
Expected: ilki derlenmez (`no method named \`recover\``); ikincisi `a_poisoned_store_lock_is_recovered_on_the_next_call` için FAILED (`lock` `store_unavailable` döner, `unwrap` panikler).

- [ ] **Step 4: Uygula**

Step 2'deki diff'in diğer bölümlerini uygula ve `app/src-tauri/src/data_dir.rs`'i oluştur:

```rust
//! Veri klasörünün ve veritabanı dosyalarının izinleri (Plan 08 ertelenen madde 10).
//! Danışan verisi şifresiz SQLite'ta durur (spec §12); klasör yalnızca sahibine açıktır (0700),
//! dosyalar yalnızca sahibince okunur ve yazılır (0600). Kullanıcının seçtiği yere yazılan dışa
//! aktarma dosyaları bu kuralın dışındadır.

use std::io;
use std::path::{Path, PathBuf};

/// Klasörü oluşturur (yoksa) ve izinlerini 0700 yapar; var olan klasör de daraltılır.
pub fn prepare(dir: &Path) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    set_mode(dir, 0o700)
}

/// Veritabanı dosyası ve SQLite'ın yan dosyaları (`-wal`, `-shm`); olmayan dosya atlanır.
pub fn restrict_db_files(db: &Path) -> io::Result<()> {
    for path in db_files(db) {
        if path.exists() {
            set_mode(&path, 0o600)?;
        }
    }
    Ok(())
}

fn db_files(db: &Path) -> [PathBuf; 3] {
    let with = |suffix: &str| {
        let mut name = db.as_os_str().to_owned();
        name.push(suffix);
        PathBuf::from(name)
    };
    [db.to_path_buf(), with("-wal"), with("-shm")]
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> io::Result<()> {
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn mode(path: &Path) -> u32 {
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn a_new_and_an_existing_data_directory_are_owner_only() {
        let tmp = tempfile::tempdir().unwrap();
        let fresh = tmp.path().join("a/b");
        prepare(&fresh).unwrap();
        assert_eq!(mode(&fresh), 0o700);

        let open = tmp.path().join("open");
        std::fs::create_dir(&open).unwrap();
        std::fs::set_permissions(&open, std::fs::Permissions::from_mode(0o755)).unwrap();
        prepare(&open).unwrap();
        assert_eq!(mode(&open), 0o700);
    }

    #[test]
    fn the_database_and_its_side_files_are_owner_read_write_and_missing_ones_are_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        let db = tmp.path().join("clients.sqlite");
        let wal = tmp.path().join("clients.sqlite-wal");
        for f in [&db, &wal] {
            std::fs::write(f, b"x").unwrap();
            std::fs::set_permissions(f, std::fs::Permissions::from_mode(0o644)).unwrap();
        }
        restrict_db_files(&db).unwrap();
        assert_eq!((mode(&db), mode(&wal)), (0o600, 0o600));
        assert!(!tmp.path().join("clients.sqlite-shm").exists());
    }

    #[test]
    fn a_real_store_keeps_working_after_its_files_are_restricted() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("data");
        prepare(&dir).unwrap();
        let db = dir.join("clients.sqlite");
        let store = store::Store::open(&db).unwrap();
        restrict_db_files(&db).unwrap();
        assert_eq!(mode(&db), 0o600);
        store.set_app_prefs(&store::AppPrefs::default()).unwrap();
        assert_eq!(store.app_prefs().unwrap(), store::AppPrefs::default());
    }
}
```

- [ ] **Step 5: Kapılar**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p store -p jyotish-desk 2>&1 | grep -E "^test result"
```
Expected: fmt ve clippy çıktısız; `store` lib `81 passed; 0 failed; 1 ignored`; `jyotish-desk` lib `38 passed` (üç `data_dir` testi ve kilit testi); diğer satırlar tabandaki gibi.

- [ ] **Step 6: Commit**

```bash
git add crates/store/src/db.rs app/src-tauri/src/data_dir.rs app/src-tauri/src/client_commands.rs app/src-tauri/src/atlas_commands.rs app/src-tauri/src/lib.rs
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "fix(app): owner-only data directory and database files; a poisoned store lock recovers on the next call"
```
Expected: iki `grep` çıktısız (çıkış kodu 1 beklenir); commit mesajı da aynı desenle eşleşmez.

---

### Task 2: Güncelleyici: bağımlılık, yapılandırma, komutlar, bağlamalar ve hata metinleri

**Files:**
- Create: `app/src-tauri/src/update_commands.rs`
- Modify: `app/src-tauri/Cargo.toml`, `Cargo.lock` (cargo üretir), `app/src-tauri/tauri.conf.json`, `app/src-tauri/src/lib.rs`, `app/ui/bindings.ts` (üretilir), `app/ui/api.ts`, `app/ui/api.test.ts`, `app/ui/i18n/errors.ts`, `app/ui/i18n/{tr,en,fr}.ts`, `app/ui/i18n/i18n.test.ts`
- Test: `update_commands.rs` içindeki `mod tests`; `api.test.ts`; `i18n.test.ts`

**Interfaces:**
- Consumes: `CommandError::new` (`commands.rs`); `tauri_plugin_updater::{UpdaterExt, Update, Error}`.
- Produces (Rust): `UPDATE_ENDPOINT: &str`, `MAX_NOTES_CHARS: usize = 4000`, `struct AppInfo { version: String, updater_configured: bool }` (IPC'de `updaterConfigured`), `struct UpdateInfo { version: String, notes: Option<String>, published: Option<String> }`, `struct InstallGuard`, `updater_configured(&serde_json::Value) -> bool`, `clip_notes(&str) -> String`, `update_reason(&Error) -> &'static str`; komutlar `app_info() -> Result<AppInfo, CommandError>`, `check_for_update() -> Result<Option<UpdateInfo>, CommandError>`, `install_update(version: String) -> Result<(), CommandError>`.
- Produces (TS): `api.appInfo()`, `api.checkForUpdate()`, `api.installUpdate(version: string)`; `AppInfo`, `UpdateInfo` tipleri (`bindings.ts`); hata kodları `update_unconfigured`, `update_check_failed`, `update_install_failed`, `update_changed`, `update_busy`; `updateReason.{network,signature,manifest,install,other}` metinleri.

- [ ] **Step 1: Bağımlılığı ve yapılandırmayı ekle; lisans kapısını koş**

```diff
diff --git a/app/src-tauri/Cargo.toml b/app/src-tauri/Cargo.toml
--- a/app/src-tauri/Cargo.toml
+++ b/app/src-tauri/Cargo.toml
@@ -28,6 +28,12 @@ specta-typescript = "0.0.15"
 # Dosya seçme/kaydetme pencereleri Rust tarafında açılır (`DialogExt`); arayüze yol seçtiren
 # bir komut ve yetenek (capability) yoktur. Apache-2.0 OR MIT.
 tauri-plugin-dialog = "=2.8.4"
+# Güncelleme (spec §10; Plan 13 D3): imzalı paketi indirir, minisign imzasını doğrular ve kurar.
+# Yalnızca Rust'tan kullanılır (`UpdaterExt`); eklentinin IPC komutları yetenek dosyasında izinli
+# değildir. TLS işletim sisteminindir (`native-tls`: macOS'ta Security.framework); varsayılan
+# `rustls-tls` özelliği `ring` ve CDLA lisanslı kök sertifika paketini getirir. `zip`: macOS
+# paketi `.app.tar.gz`'dir. Apache-2.0 OR MIT.
+tauri-plugin-updater = { version = "=2.14.1", default-features = false, features = ["native-tls", "zip"] }
 
 [dev-dependencies]
 tempfile = "3"
diff --git a/app/src-tauri/tauri.conf.json b/app/src-tauri/tauri.conf.json
--- a/app/src-tauri/tauri.conf.json
+++ b/app/src-tauri/tauri.conf.json
@@ -19,5 +19,12 @@
     "icon": ["icons/32x32.png", "icons/128x128.png", "icons/128x128@2x.png", "icons/icon.icns", "icons/icon.png"],
     "resources": { "../../data/ephe/": "ephe/", "../../data/atlas/atlas.sqlite": "atlas/atlas.sqlite" },
     "macOS": { "minimumSystemVersion": "13.0" }
+  },
+  "plugins": {
+    "updater": {
+      "endpoints": ["https://github.com/turgaydogusan/jyotish-desk/releases/latest/download/latest.json"],
+      "pubkey": "",
+      "requireSignedVersion": true
+    }
   }
 }
```

```bash
cargo metadata --format-version 1 > /dev/null
cargo deny check 2>&1 | tail -1
cargo tree -p jyotish-desk -i ring 2>&1 | head -2
```
Expected: `advisories ok, bans ok, licenses ok, sources ok`; `cargo tree` `ring` paketini bulamaz (`error: package ID specification \`ring\` did not match any packages`). `licenses FAILED` çıkarsa özellik listesi yanlıştır (`rustls-tls` açık durur); `deny.toml`'a lisans eklenmez (D3).

- [ ] **Step 2: Başarısız testleri yaz**

`app/src-tauri/src/update_commands.rs`'i aşağıdaki tam içeriğin yalnızca başlık yorumu, `use` satırları ve `#[cfg(test)] mod tests` bloğuyla oluştur; `lib.rs`'e `pub mod update_commands;` satırını ekle (Step 4'teki diff'in ilk parçası).

Run: `cargo test -p jyotish-desk --lib update_commands 2>&1 | grep -E "^error" | sort -u | head`
Expected: derlenmez (`cannot find function \`updater_configured\``, `\`clip_notes\``, `\`update_reason\``, `\`update_error\``, `cannot find value \`UPDATE_ENDPOINT\``).

- [ ] **Step 3: `update_commands.rs`'in tamamı**

```rust
//! Güncelleme (spec §10): Tauri güncelleyicisi, imzalı, GitHub Releases üzerinden. Denetim yalnızca
//! kullanıcı isteyince yapılır: bu modülde kendiliğinden çalışan hiçbir şey yoktur; arayüz
//! `check_for_update`'i kullanıcının açık isteğiyle (düğme ya da açtığı tercih) çağırır.
//!
//! Ağa giden tek istek güncelleme adresine yapılan GET'tir (`UPDATE_ENDPOINT`) ve kurulumda
//! paketin indirilmesidir; istekte sürüm, kimlik ya da danışan verisi yoktur. İndirilen paket
//! gömülü açık anahtarla doğrulanmadan kurulmaz (eklenti bunu atlamaya izin vermez) ve imzanın
//! taşıdığı sürüm, bildirilen sürümle aynı olmalıdır (`requireSignedVersion`).
//!
//! Eklentinin kendi IPC komutları (`plugin:updater|…`) yetenek dosyasında izinli değildir;
//! arayüz yalnızca buradaki üç komutu çağırır ve hiçbiri adres ya da dosya yolu almaz (C2′).

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, State};
use tauri_plugin_updater::{Error as UpdaterError, UpdaterExt};

use crate::commands::CommandError;

/// Güncelleme bildiriminin adresi; `tauri.conf.json` ile aynıdır (test denetler).
pub const UPDATE_ENDPOINT: &str =
    "https://github.com/turgaydogusan/jyotish-desk/releases/latest/download/latest.json";
/// Bildirim isteğinin ve paket indirmenin zaman sınırları.
const CHECK_TIMEOUT: Duration = Duration::from_secs(30);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(600);
/// Arayüze taşınan yayın notunun en büyük uzunluğu (karakter).
pub const MAX_NOTES_CHARS: usize = 4000;

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    /// Açık anahtar gömülü mü; değilse (anahtar üretilmeden yapılan derleme) denetim kapalıdır.
    pub updater_configured: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct UpdateInfo {
    pub version: String,
    pub notes: Option<String>,
    /// Yayın günü, `YYYY-MM-DD`.
    pub published: Option<String>,
}

/// Aynı anda tek kurulum (managed state).
#[derive(Default)]
pub struct InstallGuard(AtomicBool);

/// `plugins.updater.pubkey` boş değilse güncelleme yapılandırılmıştır.
pub fn updater_configured(plugins: &serde_json::Value) -> bool {
    plugins
        .pointer("/updater/pubkey")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|k| !k.trim().is_empty())
}

fn configured(app: &AppHandle) -> bool {
    serde_json::to_value(&app.config().plugins).is_ok_and(|v| updater_configured(&v))
}

pub fn clip_notes(notes: &str) -> String {
    match notes.char_indices().nth(MAX_NOTES_CHARS) {
        Some((i, _)) => format!("{}…", &notes[..i]),
        None => notes.to_owned(),
    }
}

/// Hatanın kararlı nedeni; arayüz bunu kendi dilinde gösterir. Eklentinin iletisi taşınmaz
/// (adres ve dosya yolu içerebilir).
pub fn update_reason(e: &UpdaterError) -> &'static str {
    match e {
        UpdaterError::Reqwest(_) | UpdaterError::Network(_) | UpdaterError::Http(_) => "network",
        UpdaterError::Minisign(_)
        | UpdaterError::Base64(_)
        | UpdaterError::SignatureUtf8(_)
        | UpdaterError::MissingSignedVersion
        | UpdaterError::SignedVersionMismatch { .. } => "signature",
        UpdaterError::ReleaseNotFound
        | UpdaterError::Serialization(_)
        | UpdaterError::Semver(_)
        | UpdaterError::UrlParse(_)
        | UpdaterError::TargetNotFound(_)
        | UpdaterError::TargetsNotFound(_) => "manifest",
        UpdaterError::Io(_)
        | UpdaterError::PreviousAppNotRestored(_)
        | UpdaterError::TempDirNotOnSameMountPoint
        | UpdaterError::TempDirNotFound
        | UpdaterError::BinaryNotFoundInArchive
        | UpdaterError::FailedToDetermineExtractPath
        | UpdaterError::AuthenticationFailed => "install",
        _ => "other",
    }
}

fn update_error(code: &str, e: &UpdaterError) -> CommandError {
    CommandError::new(code, &[("reason", update_reason(e).to_owned())])
}

async fn find_update(
    app: &AppHandle,
    code: &str,
) -> Result<Option<tauri_plugin_updater::Update>, CommandError> {
    if !configured(app) {
        return Err(CommandError::new("update_unconfigured", &[]));
    }
    let updater = app
        .updater_builder()
        .timeout(CHECK_TIMEOUT)
        .build()
        .map_err(|e| update_error(code, &e))?;
    updater.check().await.map_err(|e| update_error(code, &e))
}

#[tauri::command]
#[specta::specta]
pub async fn app_info(app: AppHandle) -> Result<AppInfo, CommandError> {
    Ok(AppInfo {
        version: app.package_info().version.to_string(),
        updater_configured: configured(&app),
    })
}

/// Yeni sürüm var mı. `None`: kurulu sürüm günceldir.
#[tauri::command]
#[specta::specta]
pub async fn check_for_update(app: AppHandle) -> Result<Option<UpdateInfo>, CommandError> {
    let update = find_update(&app, "update_check_failed").await?;
    Ok(update.map(|u| UpdateInfo {
        version: u.version.clone(),
        notes: u.body.as_deref().map(clip_notes),
        published: u.date.map(|d| d.date().to_string()),
    }))
}

/// Kullanıcının onayladığı sürümü indirir, imzasını doğrular ve kurar. Yayın bu arada
/// değiştiyse (`version` artık son sürüm değilse) hiçbir şey kurulmaz: onay o sürüme verilmiştir.
/// Yeni sürüm uygulama kapatılıp açılınca çalışır; uygulama kendini yeniden başlatmaz.
#[tauri::command]
#[specta::specta]
pub async fn install_update(
    app: AppHandle,
    guard: State<'_, InstallGuard>,
    version: String,
) -> Result<(), CommandError> {
    if guard.0.swap(true, Ordering::SeqCst) {
        return Err(CommandError::new("update_busy", &[]));
    }
    let result = install(&app, &version).await;
    guard.0.store(false, Ordering::SeqCst);
    result
}

async fn install(app: &AppHandle, version: &str) -> Result<(), CommandError> {
    let code = "update_install_failed";
    let mut update = find_update(app, code)
        .await?
        .filter(|u| u.version == version)
        .ok_or_else(|| CommandError::new("update_changed", &[]))?;
    update.timeout = Some(DOWNLOAD_TIMEOUT);
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| update_error(code, &e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn conf() -> serde_json::Value {
        serde_json::from_str(include_str!("../tauri.conf.json")).unwrap()
    }

    #[test]
    fn the_updater_is_configured_only_with_a_public_key() {
        assert!(!updater_configured(&json!({})));
        assert!(!updater_configured(&json!({ "updater": { "pubkey": "" } })));
        assert!(!updater_configured(
            &json!({ "updater": { "pubkey": "  " } })
        ));
        assert!(!updater_configured(&json!({ "updater": { "pubkey": 7 } })));
        assert!(updater_configured(
            &json!({ "updater": { "pubkey": "dW50cnVzdGVk" } })
        ));
    }

    #[test]
    fn the_shipped_configuration_checks_one_https_endpoint_and_requires_signed_versions() {
        let updater = &conf()["plugins"]["updater"];
        assert_eq!(updater["endpoints"], json!([UPDATE_ENDPOINT]));
        assert!(UPDATE_ENDPOINT.starts_with("https://github.com/"));
        // Adreste sürüm, hedef ya da başka bir değişken yoktur: istek kimseyi ayırt etmez.
        assert!(!UPDATE_ENDPOINT.contains("{{") && !UPDATE_ENDPOINT.contains('?'));
        assert_eq!(updater["requireSignedVersion"], json!(true));
        let keys: Vec<&str> = updater
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["endpoints", "pubkey", "requireSignedVersion"]);
        // Anahtar ya henüz yoktur (boş) ya da base64'tür (minisign açık anahtar dosyası).
        let key = updater["pubkey"].as_str().unwrap();
        assert!(
            key.is_empty()
                || (key.len() >= 100
                    && key
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"+/=".contains(&b))),
            "plugins.updater.pubkey is neither empty nor base64"
        );
    }

    #[test]
    fn the_webview_gets_no_updater_or_network_permission() {
        let cap: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/default.json")).unwrap();
        assert_eq!(cap["permissions"], json!(["core:event:default"]));
        let csp = conf()["app"]["security"]["csp"]
            .as_str()
            .unwrap()
            .to_owned();
        assert!(csp.contains("connect-src ipc: http://ipc.localhost;"));
        assert!(!csp.contains("https:"));
    }

    #[test]
    fn updater_artifacts_are_off_by_default_so_a_build_needs_no_signing_key() {
        assert!(conf()["bundle"].get("createUpdaterArtifacts").is_none());
    }

    #[test]
    fn long_release_notes_are_clipped_on_a_character_boundary() {
        assert_eq!(clip_notes("kısa"), "kısa");
        let long = "ş".repeat(MAX_NOTES_CHARS + 5);
        let clipped = clip_notes(&long);
        assert_eq!(clipped.chars().count(), MAX_NOTES_CHARS + 1);
        assert!(clipped.ends_with('…'));
    }

    #[test]
    fn updater_errors_map_to_stable_reasons_without_their_text() {
        assert_eq!(update_reason(&UpdaterError::ReleaseNotFound), "manifest");
        assert_eq!(
            update_reason(&UpdaterError::MissingSignedVersion),
            "signature"
        );
        assert_eq!(update_reason(&UpdaterError::Network("x".into())), "network");
        assert_eq!(
            update_reason(&UpdaterError::Io(std::io::Error::other("/Users/a/b"))),
            "install"
        );
        assert_eq!(update_reason(&UpdaterError::EmptyEndpoints), "other");
        let e = update_error(
            "update_check_failed",
            &UpdaterError::Io(std::io::Error::other("/Users/a/b")),
        );
        assert_eq!(e.code, "update_check_failed");
        assert_eq!(e.params.len(), 1);
        assert_eq!(e.params["reason"], "install");
    }
}
```

- [ ] **Step 4: Eklentiyi ve komutları kaydet**

```diff
diff --git a/app/src-tauri/src/lib.rs b/app/src-tauri/src/lib.rs
--- a/app/src-tauri/src/lib.rs
+++ b/app/src-tauri/src/lib.rs
@@ -4,6 +4,7 @@ pub mod commands;
 pub mod data_dir;
 pub mod store_commands;
 pub mod transit_commands;
+pub mod update_commands;
 
 use std::sync::{Arc, Mutex};
 
@@ -57,6 +58,9 @@ pub fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
             store_commands::import_clients_json,
             atlas_commands::search_places,
             atlas_commands::preview_moment,
+            update_commands::app_info,
+            update_commands::check_for_update,
+            update_commands::install_update,
         ])
 }
 
@@ -64,6 +68,9 @@ pub fn run() {
     let builder = specta_builder();
     tauri::Builder::default()
         .plugin(tauri_plugin_dialog::init())
+        // Güncelleyici yalnızca `update_commands` üzerinden çalışır; açılışta denetim yapmaz.
+        .plugin(tauri_plugin_updater::Builder::new().build())
+        .manage(update_commands::InstallGuard::default())
         .invoke_handler(builder.invoke_handler())
         .setup(move |app| {
             builder.mount_events(app);
```

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p jyotish-desk --lib 2>&1 | grep -E "^test result"
```
Expected: fmt ve clippy çıktısız; `44 passed` (altı yeni test).

- [ ] **Step 5: Bağlamaları üret; arayüz sarmalayıcıları ve hata metinleri için başarısız testleri yaz**

```bash
UPDATE_BINDINGS=1 cargo test -p jyotish-desk --test bindings 2>&1 | grep "test result"
git diff --stat -- app/ui/bindings.ts
grep -n "appInfo\|checkForUpdate\|installUpdate" app/ui/bindings.ts | cut -c1-120
```
Expected: `1 passed`; `bindings.ts` 22 satır eklenmiş; üç komut satırı `typedError<AppInfo, CommandError>`, `typedError<UpdateInfo | null, CommandError>` ve `installUpdate: (version: string, …) => typedError<null, CommandError>` içerir.

Test dosyalarını değiştir:

```diff
diff --git a/app/ui/api.test.ts b/app/ui/api.test.ts
--- a/app/ui/api.test.ts
+++ b/app/ui/api.test.ts
@@ -36,6 +36,9 @@ vi.mock("./bindings", () => ({
     updateClient: vi.fn(),
     transitsAt: vi.fn(),
     keepStoredMoment: vi.fn(),
+    appInfo: vi.fn(),
+    checkForUpdate: vi.fn(),
+    installUpdate: vi.fn(),
   },
 }));
 
@@ -112,6 +115,9 @@ test("every wrapper forwards its arguments to the command of the same name", asy
     ["updateClient", [2, "A", null]],
     ["transitsAt", [{ kind: "now" }, { ayanamsa: "Lahiri" }, [], { birth_jd_ut: 1 }]],
     ["keepStoredMoment", [3]],
+    ["appInfo", []],
+    ["checkForUpdate", []],
+    ["installUpdate", ["0.9.1"]],
   ];
   for (const [name, args] of calls) {
     const command = vi.mocked(commands[name] as (...a: unknown[]) => Promise<unknown>);
@@ -145,6 +151,6 @@ test("every string argument of a command is a known, non-path one (found by type
     }
   }
   expect(found.sort()).toEqual(
-    ["addNote.body", "listClientsByTag.tag", "searchClients.query", "searchPlaces.query", "setChartTags.tags", "updateClient.name", "updateNote.body"].sort(),
+    ["addNote.body", "installUpdate.version", "listClientsByTag.tag", "searchClients.query", "searchPlaces.query", "setChartTags.tags", "updateClient.name", "updateNote.body"].sort(),
   );
 });
diff --git a/app/ui/i18n/i18n.test.ts b/app/ui/i18n/i18n.test.ts
--- a/app/ui/i18n/i18n.test.ts
+++ b/app/ui/i18n/i18n.test.ts
@@ -78,6 +78,7 @@ const BACKEND_CODES = [
   "atlas_unavailable", "atlas_sqlite", "atlas_io", "atlas_schema",
   "file_io", "dialog_path", "import_too_large", "import_format", "import_version", "import_record",
   "prefs_too_new",
+  "update_unconfigured", "update_check_failed", "update_install_failed", "update_changed", "update_busy",
 ] as const;
 
 test("every backend error code reachable from the UI has a message", () => {
@@ -94,6 +95,14 @@ test("atlas reasons are localized in every language", () => {
   for (const lang of [tr, en, fr]) expect(Object.keys(lang.atlasReason).sort()).toEqual(["io", "missing", "schema"]);
 });
 
+test("update failure reasons are localized in every language and an unknown reason falls back", () => {
+  for (const lang of [tr, en, fr]) expect(Object.keys(lang.updateReason).sort()).toEqual(["install", "manifest", "network", "other", "signature"]);
+  const t = i18n.getFixedT("tr");
+  expect(errorMessage(t, i18n, { code: "update_check_failed", params: { reason: "network" } })).toBe("Güncelleme denetlenemedi (ağa ulaşılamadı).");
+  expect(errorMessage(t, i18n, { code: "update_install_failed", params: { reason: "signature" } })).toBe("Güncelleme kurulamadı (imza doğrulanamadı). Kurulu sürüm değişmedi.");
+  expect(errorMessage(t, i18n, { code: "update_check_failed", params: { reason: "/Users/a" } })).toBe("Güncelleme denetlenemedi (beklenmeyen bir sorun).");
+});
+
 test("file error details are localized in every language", () => {
   const codes = ["already_exists", "invalid_data", "invalid_input", "not_found", "other", "permission_denied", "storage_full"];
   for (const lang of [tr, en, fr]) expect(Object.keys(lang.errors.fileIoDetail).sort()).toEqual(codes);
```

Run: `pnpm --dir app exec vitest run ui/api.test.ts ui/i18n/i18n.test.ts 2>&1 | grep -E "✓|×|FAIL|Tests " | head`
Expected: FAIL: `api.test.ts` sarmalayıcı tablosu ve dizgi bağımsız değişken listesi tutmaz (`appInfo` `api`'de yok); `i18n.test.ts`'te `update_unconfigured` hata metni ve `updateReason` yok.

- [ ] **Step 6: Sarmalayıcıları ve metinleri ekle**

```diff
diff --git a/app/ui/api.ts b/app/ui/api.ts
--- a/app/ui/api.ts
+++ b/app/ui/api.ts
@@ -77,6 +77,11 @@ export const api = {
   dashaChildren: (seed: DashaSeed, path: Graha[]) => wrap(() => commands.dashaChildren(seed, path)),
   searchPlaces: (query: string, limit: number) => wrap(() => commands.searchPlaces(query, limit)),
   previewMoment: (birth: BirthData) => wrap(() => commands.previewMoment(birth)),
+  appInfo: () => wrap(() => commands.appInfo()),
+  // Güncelleme: denetim yalnızca kullanıcının isteğiyle çağrılır (Plan 13 D1); `version`
+  // kullanıcının kurmayı onayladığı sürümdür, adres ya da yol değildir (C2′).
+  checkForUpdate: () => wrap(() => commands.checkForUpdate()),
+  installUpdate: (version: string) => wrap(() => commands.installUpdate(version)),
 };
 
 export type Api = typeof api;
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -101,6 +101,11 @@ export const en: Messages = {
     store_json: "Stored data could not be read ({{detail}}).",
     store_too_new: "The database is newer than this app (version {{found}}, supported {{supported}}). Update the app.",
     prefs_too_new: "Preferences were saved by a newer version of the app (version {{found}}, supported {{supported}}). The change was not saved; update the app.",
+    update_unconfigured: "This build has no update check.",
+    update_check_failed: "Could not check for updates ({{reason}}).",
+    update_install_failed: "The update could not be installed ({{reason}}). The installed version is unchanged.",
+    update_changed: "The release changed in the meantime; check again.",
+    update_busy: "An installation is already running.",
     store_unavailable: "The client database is unavailable.",
     not_found: "Record not found ({{what}} {{id}}).",
     invalid_input: "Invalid value: {{field}}.",
@@ -185,6 +190,7 @@ export const en: Messages = {
     SunriseUnavailable: "No sunrise or sunset at the birth place.",
   },
   atlasReason: { missing: "the atlas file is missing", schema: "the atlas file cannot be read", io: "the atlas file cannot be accessed" },
+  updateReason: { network: "the network could not be reached", signature: "the signature could not be verified", manifest: "the release information could not be read", install: "the package could not be put in place", other: "an unexpected problem" },
   notice: {
     tzdbFailed: "The time zone decision could not be saved: {{message}}",
     settingsSaveFailed: "Settings could not be saved: {{message}}",
diff --git a/app/ui/i18n/errors.ts b/app/ui/i18n/errors.ts
--- a/app/ui/i18n/errors.ts
+++ b/app/ui/i18n/errors.ts
@@ -22,5 +22,9 @@ export function errorMessage(t: TFunction, i18n: I18n, error: CommandError): str
     const detail = String(error.params.detail ?? "other");
     params.detail = i18n.exists(`errors.fileIoDetail.${detail}`) ? t(`errors.fileIoDetail.${detail}`) : detail;
   }
+  if (error.code === "update_check_failed" || error.code === "update_install_failed") {
+    const reason = String(error.params.reason ?? "other");
+    params.reason = t(i18n.exists(`updateReason.${reason}`) ? `updateReason.${reason}` : "updateReason.other");
+  }
   return t(key, params);
 }
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -101,6 +101,11 @@ export const fr: Messages = {
     store_json: "Les données enregistrées sont illisibles ({{detail}}).",
     store_too_new: "La base est plus récente que l’application (version {{found}}, prise en charge {{supported}}). Mettez l’application à jour.",
     prefs_too_new: "Les préférences ont été enregistrées par une version plus récente de l’application (version {{found}}, prise en charge {{supported}}). La modification n’a pas été enregistrée ; mettez l’application à jour.",
+    update_unconfigured: "Cette version ne vérifie pas les mises à jour.",
+    update_check_failed: "Impossible de vérifier les mises à jour ({{reason}}).",
+    update_install_failed: "La mise à jour n’a pas pu être installée ({{reason}}). La version installée est inchangée.",
+    update_changed: "La publication a changé entre-temps ; vérifiez à nouveau.",
+    update_busy: "Une installation est déjà en cours.",
     store_unavailable: "La base des consultants est indisponible.",
     not_found: "Enregistrement introuvable ({{what}} {{id}}).",
     invalid_input: "Valeur invalide : {{field}}.",
@@ -185,6 +190,7 @@ export const fr: Messages = {
     SunriseUnavailable: "Pas de lever ni de coucher du Soleil au lieu de naissance.",
   },
   atlasReason: { missing: "fichier d’atlas absent", schema: "fichier d’atlas illisible", io: "fichier d’atlas inaccessible" },
+  updateReason: { network: "réseau inaccessible", signature: "signature non vérifiée", manifest: "informations de publication illisibles", install: "paquet impossible à mettre en place", other: "problème inattendu" },
   notice: {
     tzdbFailed: "La décision de fuseau n’a pas pu être enregistrée : {{message}}",
     settingsSaveFailed: "Les réglages n’ont pas pu être enregistrés : {{message}}",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -100,6 +100,11 @@ export const tr = {
     store_json: "Saklanan veri okunamadı ({{detail}}).",
     store_too_new: "Veritabanı bu sürümden yeni (sürüm {{found}}, desteklenen {{supported}}). Uygulamayı güncelleyin.",
     prefs_too_new: "Tercihler bu sürümden yeni bir uygulamayla kaydedilmiş (sürüm {{found}}, desteklenen {{supported}}). Değişiklik kaydedilmedi; uygulamayı güncelleyin.",
+    update_unconfigured: "Bu derlemede güncelleme denetimi yok.",
+    update_check_failed: "Güncelleme denetlenemedi ({{reason}}).",
+    update_install_failed: "Güncelleme kurulamadı ({{reason}}). Kurulu sürüm değişmedi.",
+    update_changed: "Yayın bu arada değişti; yeniden denetleyin.",
+    update_busy: "Bir kurulum zaten sürüyor.",
     store_unavailable: "Danışan veritabanı kullanılamıyor.",
     not_found: "Kayıt bulunamadı ({{what}} {{id}}).",
     invalid_input: "Geçersiz değer: {{field}}.",
@@ -184,6 +189,7 @@ export const tr = {
     SunriseUnavailable: "Doğum yerinde gün doğumu ya da batımı yok.",
   },
   atlasReason: { missing: "atlas dosyası yok", schema: "atlas dosyası okunamıyor", io: "atlas dosyasına erişilemiyor" },
+  updateReason: { network: "ağa ulaşılamadı", signature: "imza doğrulanamadı", manifest: "yayın bilgisi okunamadı", install: "paket yerine konamadı", other: "beklenmeyen bir sorun" },
   notice: {
     tzdbFailed: "Saat dilimi kararı kaydedilemedi: {{message}}",
     settingsSaveFailed: "Ayarlar kaydedilemedi: {{message}}",
```

- [ ] **Step 7: Kapılar**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | tail -1
cargo test --workspace 2>&1 | grep -E "^test result" | grep -v " 0 failed"
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p store -p jyotish-desk --all-targets 2>&1 | tail -1
```
Expected: `57 passed (57)`, `501 passed (501)`; `✓ built`; üçüncü komut çıktısız (başarısız test yok); MSRV denetimi `Finished` (1.93 araç zinciri kurulu değilse `rustup toolchain install 1.93 --profile minimal`).

- [ ] **Step 8: Commit**

```bash
git add Cargo.lock app/src-tauri/Cargo.toml app/src-tauri/tauri.conf.json app/src-tauri/src/lib.rs app/src-tauri/src/update_commands.rs app/ui/bindings.ts app/ui/api.ts app/ui/api.test.ts app/ui/i18n/errors.ts app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts app/ui/i18n/i18n.test.ts
git diff --cached -U0 -- . ':!Cargo.lock' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(app): signed updater behind three commands; update error codes and reasons"
```
Expected: iki `grep` çıktısız. (`Cargo.lock` paket adları taramanın dışındadır; adlar crates.io'dan gelir.)

---

### Task 3: Hakkında penceresi, güncelleme tercihi ve açılış denetimi

**Files:**
- Create: `app/ui/update/pref.ts`, `app/ui/update/startup.ts`, `app/ui/about/AboutDialog.tsx`, `app/ui/about/about.css`
- Modify: `app/ui/shell/Shell.tsx`, `app/ui/shell/TopBar.tsx`, `app/ui/palette/items.ts`, `app/ui/palette/Palette.tsx`, `app/ui/i18n/{tr,en,fr}.ts`
- Test: `app/ui/update/pref.test.ts`, `app/ui/update/startup.test.ts`, `app/ui/about/AboutDialog.test.tsx`, `app/ui/App.test.tsx`

**Interfaces:**
- Consumes: `api.appInfo`, `api.checkForUpdate`, `api.installUpdate` ve `AppInfo`, `UpdateInfo` (Task 2); `Dialog` (`shell/Dialog.tsx`); `errorMessage` (`i18n/errors.ts`); `Notice`, `useNotices` (`shell/notices.tsx`).
- Produces: `UPDATE_STORAGE_KEY = "jyotish-desk.updates.v1"`, `loadAutoCheck(storage?): boolean`, `saveAutoCheck(on, storage?): boolean`; `startupUpdateCheck(push, storage?): Promise<void>`; `AboutDialog({ onClose })`, `SOURCE_URL`; `TopBarProps.onAbout(): void`; `PaletteCommand` üyesi `"about"`; i18n: `about.*`, `update.*`, `notice.updateAvailable`, `palette.command.about`.

- [ ] **Step 1: Başarısız testleri yaz**

`app/ui/update/pref.test.ts`:

```ts
import { loadAutoCheck, saveAutoCheck, UPDATE_STORAGE_KEY } from "./pref";

test("the update check is off until the user turns it on", () => {
  expect(loadAutoCheck()).toBe(false);
  expect(saveAutoCheck(true)).toBe(true);
  expect(localStorage.getItem(UPDATE_STORAGE_KEY)).toBe("auto");
  expect(loadAutoCheck()).toBe(true);
  expect(saveAutoCheck(false)).toBe(true);
  expect(localStorage.getItem(UPDATE_STORAGE_KEY)).toBe("off");
  expect(loadAutoCheck()).toBe(false);
});

test("an unknown stored value and a storage that throws both mean off", () => {
  for (const v of ["true", "1", "AUTO", ""]) {
    localStorage.setItem(UPDATE_STORAGE_KEY, v);
    expect(loadAutoCheck(), v).toBe(false);
  }
  const broken = {
    getItem: () => {
      throw new Error("denied");
    },
    setItem: () => {
      throw new Error("full");
    },
  } as unknown as Storage;
  expect(loadAutoCheck(broken)).toBe(false);
  expect(saveAutoCheck(true, broken)).toBe(false);
});
```

`app/ui/update/startup.test.ts`:

```ts
import { expect, test, vi } from "vitest";

vi.mock("../api", () => ({ api: { checkForUpdate: vi.fn() } }));

import { api } from "../api";
import { saveAutoCheck } from "./pref";
import { startupUpdateCheck } from "./startup";

afterEach(() => vi.mocked(api.checkForUpdate).mockReset());

test("without consent nothing is asked of the network", async () => {
  const push = vi.fn();
  await startupUpdateCheck(push);
  saveAutoCheck(false);
  await startupUpdateCheck(push);
  expect(api.checkForUpdate).not.toHaveBeenCalled();
  expect(push).not.toHaveBeenCalled();
});

test("with consent a newer version becomes one info notice", async () => {
  saveAutoCheck(true);
  vi.mocked(api.checkForUpdate).mockResolvedValue({ status: "ok", data: { version: "0.9.1", notes: null, published: null } });
  const push = vi.fn();
  await startupUpdateCheck(push);
  expect(api.checkForUpdate).toHaveBeenCalledTimes(1);
  expect(push).toHaveBeenCalledWith({ id: "update", type: "info", messageKey: "notice.updateAvailable", params: { version: "0.9.1" } });
});

test("an up-to-date app and a failed check are both silent", async () => {
  saveAutoCheck(true);
  const push = vi.fn();
  vi.mocked(api.checkForUpdate).mockResolvedValueOnce({ status: "ok", data: null });
  await startupUpdateCheck(push);
  vi.mocked(api.checkForUpdate).mockResolvedValueOnce({ status: "error", error: { code: "update_check_failed", params: { reason: "network" } } });
  await startupUpdateCheck(push);
  expect(push).not.toHaveBeenCalled();
});
```

`app/ui/about/AboutDialog.test.tsx`:

```tsx
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";

vi.mock("../api", () => ({
  api: {
    appInfo: vi.fn(),
    checkForUpdate: vi.fn(),
    installUpdate: vi.fn(),
  },
}));

import { api } from "../api";
import { UPDATE_STORAGE_KEY } from "../update/pref";
import { AboutDialog, SOURCE_URL } from "./AboutDialog";

const info = (updaterConfigured: boolean) => ({ status: "ok" as const, data: { version: "0.9.0", updaterConfigured } });
const found = { version: "0.9.1", notes: "- Düzeltme: baskı\n- Yeni: etiket", published: "2026-12-01" };

afterEach(() => {
  for (const f of Object.values(api)) vi.mocked(f).mockReset();
});

async function open(updaterConfigured = true) {
  vi.mocked(api.appInfo).mockResolvedValue(info(updaterConfigured));
  render(<AboutDialog onClose={() => {}} />);
  const dialog = screen.getByRole("dialog", { name: "Jyotish Desk hakkında" });
  await within(dialog).findByText("Sürüm 0.9.0");
  return within(dialog);
}

test("shows the version, the licence, the source address and the data credits", async () => {
  const about = await open();
  expect(about.getByText(/AGPL-3\.0-or-later/)).toBeInTheDocument();
  expect(about.getByText(SOURCE_URL)).toBeInTheDocument();
  expect(SOURCE_URL).toBe("https://github.com/turgaydogusan/jyotish-desk");
  expect(about.getByText(/Swiss Ephemeris/)).toBeInTheDocument();
  expect(about.getByText(/GeoNames.*CC BY 4\.0/)).toBeInTheDocument();
  // Pencere tarayıcı açmaz: adres bağlantı değil, seçilebilir metindir.
  expect(about.queryByRole("link")).toBeNull();
});

test("opening the window asks nothing of the network and the startup check is off", async () => {
  const about = await open();
  expect(api.checkForUpdate).not.toHaveBeenCalled();
  const auto = about.getByRole("checkbox", { name: "Açılışta güncellemeleri denetle" });
  expect(auto).not.toBeChecked();
  expect(auto).toHaveAccessibleDescription(/danışan verisi, kimlik ya da kullanım bilgisi yoktur/);
  fireEvent.click(auto);
  expect(auto).toBeChecked();
  expect(localStorage.getItem(UPDATE_STORAGE_KEY)).toBe("auto");
  expect(api.checkForUpdate).not.toHaveBeenCalled();
  fireEvent.click(auto);
  expect(localStorage.getItem(UPDATE_STORAGE_KEY)).toBe("off");
});

test("a preference that cannot be stored stays off and says so", async () => {
  const about = await open();
  const setItem = vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
    throw new Error("full");
  });
  const auto = about.getByRole("checkbox", { name: "Açılışta güncellemeleri denetle" });
  fireEvent.click(auto);
  setItem.mockRestore();
  expect(auto).not.toBeChecked();
  expect(about.getByRole("alert")).toHaveTextContent("Tercih saklanamadı; açılışta denetim yapılmayacak.");
});

test("a build without a signing key explains itself and offers no check", async () => {
  const about = await open(false);
  expect(about.getByText("Bu derlemede güncelleme denetimi yok (imza anahtarı gömülmemiş).")).toBeInTheDocument();
  expect(about.queryByRole("button", { name: "Şimdi denetle" })).toBeNull();
  expect(about.queryByRole("checkbox")).toBeNull();
});

test("check now reports an up-to-date app", async () => {
  vi.mocked(api.checkForUpdate).mockResolvedValue({ status: "ok", data: null });
  const about = await open();
  fireEvent.click(about.getByRole("button", { name: "Şimdi denetle" }));
  expect(await about.findByText("Kurulu sürüm güncel.")).toHaveAttribute("role", "status");
  expect(about.queryByRole("button", { name: /indir ve kur/ })).toBeNull();
});

test("a found version shows its notes and is installed only after a second, version-bound consent", async () => {
  vi.mocked(api.checkForUpdate).mockResolvedValue({ status: "ok", data: found });
  let finish!: (r: { status: "ok"; data: null }) => void;
  vi.mocked(api.installUpdate).mockReturnValue(new Promise((resolve) => (finish = resolve)));
  const about = await open();
  const check = about.getByRole("button", { name: "Şimdi denetle" });
  fireEvent.click(check);
  expect(await about.findByText("Yeni sürüm: 0.9.1 (2026-12-01).")).toBeInTheDocument();
  expect(about.getByLabelText("0.9.1 sürüm notları")).toHaveTextContent("- Düzeltme: baskı - Yeni: etiket");
  expect(api.installUpdate).not.toHaveBeenCalled();

  const install = about.getByRole("button", { name: "0.9.1 sürümünü indir ve kur" });
  fireEvent.click(install);
  expect(api.installUpdate).toHaveBeenCalledWith("0.9.1");
  expect(await about.findByText("0.9.1 indiriliyor ve imzası doğrulanıyor…")).toBeInTheDocument();
  // Sürerken düğmeler kilitlidir (yerel `disabled` değil) ve ikinci tıklama yeni istek göndermez.
  expect(install).toHaveAttribute("aria-disabled", "true");
  expect(check).toHaveAttribute("aria-disabled", "true");
  fireEvent.click(install);
  fireEvent.click(check);
  expect(api.installUpdate).toHaveBeenCalledTimes(1);
  expect(api.checkForUpdate).toHaveBeenCalledTimes(1);

  finish({ status: "ok", data: null });
  expect(await about.findByText("0.9.1 kuruldu. Yeni sürüm, uygulamayı kapatıp yeniden açınca çalışır.")).toBeInTheDocument();
  expect(about.queryByRole("button", { name: /indir ve kur/ })).toBeNull();
  expect(document.activeElement).toBe(about.getByRole("button", { name: "Kapat" }));
});

test("a failed check and a failed install are shown in the UI language and can be retried", async () => {
  vi.mocked(api.checkForUpdate).mockResolvedValueOnce({ status: "error", error: { code: "update_check_failed", params: { reason: "network" } } });
  const about = await open();
  fireEvent.click(about.getByRole("button", { name: "Şimdi denetle" }));
  expect(await about.findByRole("alert")).toHaveTextContent("Güncelleme denetlenemedi (ağa ulaşılamadı).");

  vi.mocked(api.checkForUpdate).mockResolvedValueOnce({ status: "ok", data: { ...found, published: null } });
  vi.mocked(api.installUpdate).mockResolvedValueOnce({ status: "error", error: { code: "update_install_failed", params: { reason: "signature" } } });
  fireEvent.click(about.getByRole("button", { name: "Şimdi denetle" }));
  expect(await about.findByText("Yeni sürüm: 0.9.1.")).toBeInTheDocument();
  expect(about.queryByRole("alert")).toBeNull();
  fireEvent.click(about.getByRole("button", { name: "0.9.1 sürümünü indir ve kur" }));
  expect(await about.findByRole("alert")).toHaveTextContent("Güncelleme kurulamadı (imza doğrulanamadı). Kurulu sürüm değişmedi.");
  // Kur düğmesi durur: kullanıcı yeniden deneyebilir.
  await waitFor(() => expect(about.getByRole("button", { name: "0.9.1 sürümünü indir ve kur" })).not.toHaveAttribute("aria-disabled"));
});
```

`app/ui/App.test.tsx`:

```diff
diff --git a/app/ui/App.test.tsx b/app/ui/App.test.tsx
--- a/app/ui/App.test.tsx
+++ b/app/ui/App.test.tsx
@@ -19,6 +19,9 @@ vi.mock("./api", () => ({
     deleteClient: vi.fn(),
     listNotes: vi.fn(async () => ({ status: "ok", data: [] })),
     setChartTags: vi.fn(async (_chartId: number, tags: string[]) => ({ status: "ok", data: tags })),
+    appInfo: vi.fn(async () => ({ status: "ok", data: { version: "0.9.0", updaterConfigured: true } })),
+    checkForUpdate: vi.fn(async () => ({ status: "ok", data: { version: "0.9.1", notes: null, published: null } })),
+    installUpdate: vi.fn(),
   },
 }));
 
@@ -27,6 +30,7 @@ import { App } from "./App";
 import type { Boot } from "./boot";
 import { fixture, unknownTimeOpened } from "./test/fixture";
 import { THEME_STORAGE_KEY } from "./theme/theme";
+import { UPDATE_STORAGE_KEY } from "./update/pref";
 import { memoryPersistence } from "./view/persistence";
 
 /** `store: true` gerçek store kalıcılığını (taklit `api`) kullanır; varsayılan bellek içi. */
@@ -443,3 +447,41 @@ test("the sidebar's closed state is remembered at the next start", () => {
   fireEvent.keyDown(window, { key: "\\", code: "Backslash", metaKey: true });
   expect(screen.getByRole("complementary", { name: "Danışanlar" })).toBeInTheDocument();
 });
+
+test("the app never checks for updates at startup unless the user turned that on", async () => {
+  renderApp();
+  await screen.findByRole("heading", { level: 1 });
+  expect(api.checkForUpdate).not.toHaveBeenCalled();
+  expect(api.appInfo).not.toHaveBeenCalled();
+});
+
+test("with the startup check turned on a new version is announced once in the notice bar", async () => {
+  localStorage.setItem(UPDATE_STORAGE_KEY, "auto");
+  vi.mocked(api.checkForUpdate).mockClear();
+  renderApp();
+  expect(await screen.findByText("Yeni sürüm var: 0.9.1. Kurmak için Hakkında penceresini açın.")).toBeInTheDocument();
+  expect(api.checkForUpdate).toHaveBeenCalledTimes(1);
+  expect(api.installUpdate).not.toHaveBeenCalled();
+});
+
+test("About opens from the top bar as a modal and returns focus to its button", async () => {
+  renderApp();
+  const opener = screen.getByRole("button", { name: "Hakkında" });
+  opener.focus();
+  fireEvent.click(opener);
+  const dialog = screen.getByRole("dialog", { name: "Jyotish Desk hakkında" });
+  expect(await within(dialog).findByText("Sürüm 0.9.0")).toBeInTheDocument();
+  expect(document.activeElement).toBe(within(dialog).getByRole("button", { name: "Şimdi denetle" }));
+  fireEvent.keyDown(document, { key: "Escape" });
+  expect(screen.queryByRole("dialog")).toBeNull();
+  expect(document.activeElement).toBe(opener);
+});
+
+test("the palette opens About too", async () => {
+  renderApp();
+  fireEvent.keyDown(window, { key: "k", metaKey: true });
+  const palette = screen.getByRole("dialog", { name: "Komut paleti" });
+  fireEvent.change(within(palette).getByRole("combobox"), { target: { value: "hakkında" } });
+  fireEvent.click(await within(palette).findByRole("option", { name: /Hakkında ve güncellemeler/ }));
+  expect(await screen.findByRole("dialog", { name: "Jyotish Desk hakkında" })).toBeInTheDocument();
+});
```

- [ ] **Step 2: Testlerin başarısız olduğunu doğrula**

Run: `pnpm --dir app exec vitest run ui/update ui/about ui/App.test.tsx 2>&1 | grep -E "FAIL|Error:|Tests " | head`
Expected: FAIL: üç yeni dosya içe aktarımda düşer (`./pref`, `./startup`, `./AboutDialog` yok); `App.test.tsx`'te `./update/pref` çözülemez.

- [ ] **Step 3: Tercih ve açılış denetimi**

`app/ui/update/pref.ts`:

```ts
// Güncelleme denetimi tercihi (spec §10: denetim kullanıcı onayıyla yapılır; Plan 13 D1).
// Varsayılan kapalıdır ve yalnızca kullanıcı açınca "auto" yazılır. Okunamayan ya da tanınmayan
// değer kapalı sayılır: onay kaydı yitirilirse uygulama ağa bağlanmaz. Danışan verisi içermez.
export const UPDATE_STORAGE_KEY = "jyotish-desk.updates.v1";

export function loadAutoCheck(storage: Storage = window.localStorage): boolean {
  try {
    return storage.getItem(UPDATE_STORAGE_KEY) === "auto";
  } catch {
    return false;
  }
}

/** Tercihi yazar; yazılamadıysa `false` döner (çağıran tercihi açık saymaz). Hata loglanmaz (C1). */
export function saveAutoCheck(on: boolean, storage: Storage = window.localStorage): boolean {
  try {
    storage.setItem(UPDATE_STORAGE_KEY, on ? "auto" : "off");
    return true;
  } catch {
    return false;
  }
}
```

`app/ui/update/startup.ts`:

```ts
// Açılıştaki güncelleme denetimi: yalnızca kullanıcı tercihi açtıysa (`pref.ts`) çalışır.
// Yeni sürüm varsa bildirim bandına bir bilgi satırı düşer; kurulum Hakkında penceresinden
// kullanıcının ayrı onayıyla yapılır. Denetim başarısızsa (çevrimdışı, yapılandırılmamış
// derleme) sessizce geçilir: açılış engellenmez, hata gösterilmez ve loglanmaz (C1).
import { api } from "../api";
import type { Notice } from "../shell/notices";
import { loadAutoCheck } from "./pref";

export async function startupUpdateCheck(push: (notice: Notice) => void, storage: Storage = window.localStorage): Promise<void> {
  if (!loadAutoCheck(storage)) return;
  const r = await api.checkForUpdate();
  if (r.status === "ok" && r.data !== null) {
    push({ id: "update", type: "info", messageKey: "notice.updateAvailable", params: { version: r.data.version } });
  }
}
```

- [ ] **Step 4: Hakkında penceresi**

`app/ui/about/about.css`:

```css
.about {
  display: grid;
  gap: 10px;
  min-width: 420px;
  max-width: 520px;
}

.about p {
  margin: 0;
}

.about-version {
  font-weight: var(--fw-medium);
}

.about-source {
  user-select: all;
  overflow-wrap: anywhere;
}

.about-hint,
.about-status {
  color: var(--text-2);
  font-size: 12px;
}

.about-updates {
  display: grid;
  gap: 8px;
  border: 1px solid var(--line);
  padding: 8px 10px;
}

.about-notes {
  max-height: 10rem;
  overflow: auto;
  white-space: pre-wrap;
  border-left: 2px solid var(--ink);
  padding-left: 8px;
}

.about button[aria-disabled="true"] {
  opacity: 0.6;
  cursor: default;
}
```

`app/ui/about/AboutDialog.tsx`:

```tsx
// Hakkında penceresi: sürüm, lisans ve kaynak kod adresi (AGPL, spec §10), veri kaynaklarının
// atfı (GeoNames CC BY, spec §9) ve güncelleme (spec §10: imzalı, kullanıcı onayıyla).
// Güncelleme iki ayrı onaydır: denetim (düğme ya da açılış tercihi, varsayılan kapalı) ve
// kurulum (bulunan sürüm için düğme). Kaynak adresi seçilebilir metindir; pencere tarayıcı açmaz.
import { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "../api";
import type { AppInfo, CommandError, UpdateInfo } from "../bindings";
import { errorMessage } from "../i18n/errors";
import { Dialog } from "../shell/Dialog";
import { loadAutoCheck, saveAutoCheck } from "../update/pref";
import "./about.css";

export const SOURCE_URL = "https://github.com/turgaydogusan/jyotish-desk";

type Phase =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "current" }
  | { kind: "available"; update: UpdateInfo }
  | { kind: "installing"; update: UpdateInfo }
  | { kind: "installed"; version: string };

export function AboutDialog({ onClose }: { onClose(): void }) {
  const { t, i18n } = useTranslation();
  const id = useId();
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [auto, setAuto] = useState(() => loadAutoCheck());
  const [prefLost, setPrefLost] = useState(false);
  const [phase, setPhase] = useState<Phase>({ kind: "idle" });
  const [error, setError] = useState<CommandError | null>(null);
  const mounted = useRef(true);
  const closeRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    mounted.current = true;
    void api.appInfo().then((r) => {
      if (mounted.current && r.status === "ok") setInfo(r.data);
    });
    return () => {
      mounted.current = false;
    };
  }, []);

  const busy = phase.kind === "checking" || phase.kind === "installing";

  const changeAuto = (next: boolean) => {
    const stored = saveAutoCheck(next);
    // Yazılamayan onay açık sayılmaz.
    setAuto(stored && next);
    setPrefLost(!stored);
  };

  const check = async () => {
    if (busy) return;
    setError(null);
    setPhase({ kind: "checking" });
    const r = await api.checkForUpdate();
    if (!mounted.current) return;
    if (r.status === "error") {
      setError(r.error);
      setPhase({ kind: "idle" });
    } else setPhase(r.data === null ? { kind: "current" } : { kind: "available", update: r.data });
  };

  const install = async (update: UpdateInfo) => {
    if (busy) return;
    setError(null);
    setPhase({ kind: "installing", update });
    const r = await api.installUpdate(update.version);
    if (!mounted.current) return;
    if (r.status === "error") {
      setError(r.error);
      setPhase({ kind: "available", update });
    } else {
      setPhase({ kind: "installed", version: update.version });
      // Kur düğmesi kalkar; odak pencerede bir düğmede durur.
      closeRef.current?.focus();
    }
  };

  const update = phase.kind === "available" || phase.kind === "installing" ? phase.update : null;
  const status =
    phase.kind === "checking"
      ? t("update.checking")
      : phase.kind === "current"
        ? t("update.current")
        : phase.kind === "installing"
          ? t("update.installing", { version: phase.update.version })
          : phase.kind === "installed"
            ? t("update.installed", { version: phase.version })
            : phase.kind === "available"
              ? phase.update.published
                ? t("update.availableOn", { version: phase.update.version, date: phase.update.published })
                : t("update.available", { version: phase.update.version })
              : "";

  return (
    <Dialog title={t("about.title")} onClose={onClose}>
      <div className="about">
        <p className="about-version">{info ? t("about.version", { version: info.version }) : t("about.versionUnknown")}</p>
        <p>{t("about.license")}</p>
        <p>
          {t("about.source")} <span className="about-source">{SOURCE_URL}</span>
        </p>
        <p className="about-hint">{t("about.swisseph")}</p>
        <p className="about-hint">{t("about.geonames")}</p>
        <fieldset className="about-updates">
          <legend>{t("update.title")}</legend>
          {info !== null && !info.updaterConfigured ? (
            <p className="about-hint">{t("update.unconfigured")}</p>
          ) : (
            <>
              <label>
                <input type="checkbox" checked={auto} aria-describedby={`${id}-privacy`} onChange={(e) => changeAuto(e.target.checked)} /> {t("update.auto")}
              </label>
              <p id={`${id}-privacy`} className="about-hint">
                {t("update.privacy")}
              </p>
              {prefLost && (
                <p className="field-error" role="alert">
                  {t("update.prefLost")}
                </p>
              )}
              <div className="dialog-actions">
                <button type="button" data-autofocus aria-disabled={busy || undefined} aria-describedby={busy ? `${id}-status` : undefined} onClick={() => void check()}>
                  {t("update.check")}
                </button>
                {update && (
                  <button type="button" aria-disabled={busy || undefined} aria-describedby={`${id}-status`} onClick={() => void install(update)}>
                    {t("update.install", { version: update.version })}
                  </button>
                )}
              </div>
              <p id={`${id}-status`} role="status" className="about-status">
                {status}
              </p>
              {update?.notes && (
                <p className="about-notes" aria-label={t("update.notes", { version: update.version })}>
                  {update.notes}
                </p>
              )}
              {error && (
                <p className="field-error" role="alert">
                  {errorMessage(t, i18n, error)}
                </p>
              )}
            </>
          )}
        </fieldset>
      </div>
      <div className="dialog-actions">
        <button ref={closeRef} type="button" onClick={onClose}>
          {t("about.close")}
        </button>
      </div>
    </Dialog>
  );
}
```

- [ ] **Step 5: Metinler, kabuk, üst çubuk ve palet**

```diff
diff --git a/app/ui/i18n/en.ts b/app/ui/i18n/en.ts
--- a/app/ui/i18n/en.ts
+++ b/app/ui/i18n/en.ts
@@ -210,6 +210,7 @@ export const en: Messages = {
     noteSaveLost: "A note could not be saved while its panel was closing; the last changes were not written: {{message}}",
     exportFailed: "Export failed: {{message}}",
     importFailed: "Import failed; nothing was added: {{message}}",
+    updateAvailable: "A new version is available: {{version}}. Open the About window to install it.",
   },
   settingsBadge: { label: "Settings differ", hint: "This chart's settings differ from the default: {{fields}}" },
   tzdb: { accept: "Accept the new time", keep: "Keep the stored time" },
@@ -391,6 +392,7 @@ export const en: Messages = {
       toggleNotes: "Show or hide the notes panel",
       sidebar: "Show or hide the client sidebar",
       help: "Keyboard shortcuts",
+      about: "About and updates",
     },
   },
   notes: {
@@ -477,6 +479,33 @@ export const en: Messages = {
     credit: "{{app}} {{engine}} · Swiss Ephemeris {{swe}}",
     dasha: { lord: "Antardasha", start: "Start", end: "End" },
   },
+  about: {
+    open: "About",
+    title: "About Jyotish Desk",
+    version: "Version {{version}}",
+    versionUnknown: "The version could not be read.",
+    license: "Free software: GNU Affero General Public License, version 3 or later (AGPL-3.0-or-later).",
+    source: "Source code:",
+    swisseph: "Calculations use the Swiss Ephemeris (© Astrodienst AG, AGPL-3.0).",
+    geonames: "Place data comes from GeoNames (geonames.org, CC BY 4.0); its fields were trimmed and built into a searchable database.",
+    close: "Close",
+  },
+  update: {
+    title: "Updates",
+    auto: "Check for updates at startup",
+    privacy: "A check sends a single request to github.com; it carries no client data, identity or usage information. While this option is off and you do not check yourself, the app does not connect to the network.",
+    prefLost: "The preference could not be stored; no check will run at startup.",
+    check: "Check now",
+    checking: "Checking…",
+    current: "The installed version is up to date.",
+    available: "New version: {{version}}.",
+    availableOn: "New version: {{version}} ({{date}}).",
+    install: "Download and install {{version}}",
+    installing: "Downloading {{version}} and verifying its signature…",
+    installed: "{{version}} is installed. The new version runs after you quit and reopen the app.",
+    notes: "Release notes for {{version}}",
+    unconfigured: "This build has no update check (no signing key is embedded).",
+  },
   transit: {
     date: "Transit moment",
     dateLabel: "Date",
diff --git a/app/ui/i18n/fr.ts b/app/ui/i18n/fr.ts
--- a/app/ui/i18n/fr.ts
+++ b/app/ui/i18n/fr.ts
@@ -210,6 +210,7 @@ export const fr: Messages = {
     noteSaveLost: "Une note n’a pas pu être enregistrée à la fermeture du panneau ; les dernières modifications n’ont pas été écrites : {{message}}",
     exportFailed: "Échec de l’export : {{message}}",
     importFailed: "Échec de l’import ; rien n’a été ajouté : {{message}}",
+    updateAvailable: "Une nouvelle version est disponible : {{version}}. Ouvrez la fenêtre À propos pour l’installer.",
   },
   settingsBadge: { label: "Réglages différents", hint: "Les réglages de cette carte diffèrent du défaut : {{fields}}" },
   tzdb: { accept: "Accepter la nouvelle heure", keep: "Garder l’heure enregistrée" },
@@ -391,6 +392,7 @@ export const fr: Messages = {
       toggleNotes: "Afficher ou masquer le panneau de notes",
       sidebar: "Afficher ou masquer la barre des clients",
       help: "Raccourcis clavier",
+      about: "À propos et mises à jour",
     },
   },
   notes: {
@@ -477,6 +479,33 @@ export const fr: Messages = {
     credit: "{{app}} {{engine}} · Swiss Ephemeris {{swe}}",
     dasha: { lord: "Antardasha", start: "Début", end: "Fin" },
   },
+  about: {
+    open: "À propos",
+    title: "À propos de Jyotish Desk",
+    version: "Version {{version}}",
+    versionUnknown: "Version illisible.",
+    license: "Logiciel libre : licence publique générale GNU Affero, version 3 ou ultérieure (AGPL-3.0-or-later).",
+    source: "Code source :",
+    swisseph: "Les calculs utilisent Swiss Ephemeris (© Astrodienst AG, AGPL-3.0).",
+    geonames: "Les données de lieux proviennent de GeoNames (geonames.org, CC BY 4.0) ; leurs champs ont été réduits et intégrés à une base de données consultable.",
+    close: "Fermer",
+  },
+  update: {
+    title: "Mises à jour",
+    auto: "Vérifier les mises à jour au démarrage",
+    privacy: "Une vérification envoie une seule requête à github.com ; elle ne contient ni donnée de client, ni identité, ni information d’utilisation. Tant que cette option est désactivée et que vous ne vérifiez pas vous-même, l’application ne se connecte pas au réseau.",
+    prefLost: "La préférence n’a pas pu être enregistrée ; aucune vérification au démarrage.",
+    check: "Vérifier maintenant",
+    checking: "Vérification…",
+    current: "La version installée est à jour.",
+    available: "Nouvelle version : {{version}}.",
+    availableOn: "Nouvelle version : {{version}} ({{date}}).",
+    install: "Télécharger et installer {{version}}",
+    installing: "Téléchargement de {{version}} et vérification de sa signature…",
+    installed: "{{version}} est installée. La nouvelle version s’exécute après avoir quitté et rouvert l’application.",
+    notes: "Notes de version {{version}}",
+    unconfigured: "Cette version ne vérifie pas les mises à jour (aucune clé de signature intégrée).",
+  },
   transit: {
     date: "Moment du transit",
     dateLabel: "Date",
diff --git a/app/ui/i18n/tr.ts b/app/ui/i18n/tr.ts
--- a/app/ui/i18n/tr.ts
+++ b/app/ui/i18n/tr.ts
@@ -209,6 +209,7 @@ export const tr = {
     noteSaveLost: "Panel kapanırken bir not kaydedilemedi; son değişiklikler yazılmadı: {{message}}",
     exportFailed: "Dışa aktarılamadı: {{message}}",
     importFailed: "İçe aktarılamadı; hiçbir kayıt eklenmedi: {{message}}",
+    updateAvailable: "Yeni sürüm var: {{version}}. Kurmak için Hakkında penceresini açın.",
   },
   settingsBadge: { label: "Ayar farkı", hint: "Bu haritanın ayarı varsayılandan farklı: {{fields}}" },
   tzdb: { accept: "Yeni zamanı kabul et", keep: "Saklanan zamanı koru" },
@@ -390,6 +391,7 @@ export const tr = {
       toggleNotes: "Notlar panelini aç/kapat",
       sidebar: "Danışan kenar çubuğunu aç/kapat",
       help: "Klavye kısayolları",
+      about: "Hakkında ve güncellemeler",
     },
   },
   notes: {
@@ -476,6 +478,33 @@ export const tr = {
     credit: "{{app}} {{engine}} · Swiss Ephemeris {{swe}}",
     dasha: { lord: "Antardasa", start: "Başlangıç", end: "Bitiş" },
   },
+  about: {
+    open: "Hakkında",
+    title: "Jyotish Desk hakkında",
+    version: "Sürüm {{version}}",
+    versionUnknown: "Sürüm okunamadı.",
+    license: "Özgür yazılım: GNU Affero Genel Kamu Lisansı, sürüm 3 ya da sonrası (AGPL-3.0-or-later).",
+    source: "Kaynak kod:",
+    swisseph: "Hesaplar Swiss Ephemeris ile yapılır (© Astrodienst AG, AGPL-3.0).",
+    geonames: "Yer verisi GeoNames'ten alınmıştır (geonames.org, CC BY 4.0); alanları kırpılmış ve aranabilir bir veritabanına dönüştürülmüştür.",
+    close: "Kapat",
+  },
+  update: {
+    title: "Güncelleme",
+    auto: "Açılışta güncellemeleri denetle",
+    privacy: "Denetim github.com'a tek bir istek gönderir; istekte danışan verisi, kimlik ya da kullanım bilgisi yoktur. Bu seçenek kapalıyken ve siz denetlemedikçe uygulama ağa bağlanmaz.",
+    prefLost: "Tercih saklanamadı; açılışta denetim yapılmayacak.",
+    check: "Şimdi denetle",
+    checking: "Denetleniyor…",
+    current: "Kurulu sürüm güncel.",
+    available: "Yeni sürüm: {{version}}.",
+    availableOn: "Yeni sürüm: {{version}} ({{date}}).",
+    install: "{{version}} sürümünü indir ve kur",
+    installing: "{{version}} indiriliyor ve imzası doğrulanıyor…",
+    installed: "{{version}} kuruldu. Yeni sürüm, uygulamayı kapatıp yeniden açınca çalışır.",
+    notes: "{{version}} sürüm notları",
+    unconfigured: "Bu derlemede güncelleme denetimi yok (imza anahtarı gömülmemiş).",
+  },
   transit: {
     date: "Transit anı",
     dateLabel: "Tarih",
diff --git a/app/ui/palette/Palette.tsx b/app/ui/palette/Palette.tsx
--- a/app/ui/palette/Palette.tsx
+++ b/app/ui/palette/Palette.tsx
@@ -100,6 +100,7 @@ export function Palette({ onClose, onRun }: PaletteProps) {
       { command: "sidebar", label: t("palette.command.sidebar") },
       { command: "settings", label: t("palette.command.settings") },
       { command: "help", label: t("palette.command.help") },
+      { command: "about", label: t("palette.command.about") },
     ];
     const clients = clientItems(hits, (h) => (h.matched === "Note" && h.snippet ? t("clients.matchedNote", { snippet: h.snippet }) : h.matched === "Tag" ? t("clients.matchedTag") : null));
     const local = localItems(
diff --git a/app/ui/palette/items.ts b/app/ui/palette/items.ts
--- a/app/ui/palette/items.ts
+++ b/app/ui/palette/items.ts
@@ -4,7 +4,7 @@
 import type { ChartRecord, SearchHit, Varga } from "../bindings";
 import type { Layout } from "../layout/model";
 
-export type PaletteCommand = "newChart" | "addChart" | "editChart" | "settings" | "print" | "toggleNotes" | "sidebar" | "help";
+export type PaletteCommand = "newChart" | "addChart" | "editChart" | "settings" | "print" | "toggleNotes" | "sidebar" | "help" | "about";
 
 export type PaletteItem =
   | { kind: "client"; key: string; label: string; detail: string | null; clientId: number }
diff --git a/app/ui/shell/Shell.tsx b/app/ui/shell/Shell.tsx
--- a/app/ui/shell/Shell.tsx
+++ b/app/ui/shell/Shell.tsx
@@ -1,4 +1,6 @@
+import { AboutDialog } from "../about/AboutDialog";
 import { SettingsDialog } from "../settings/SettingsDialog";
+import { startupUpdateCheck } from "../update/startup";
 import { useEffect, useLayoutEffect, useRef, useState } from "react";
 import { api } from "../api";
 import type { CommandError } from "../bindings";
@@ -29,6 +31,7 @@ export function Shell() {
   const [printError, setPrintError] = useState<CommandError | null>(null);
   const [help, setHelp] = useState(false);
   const [settings, setSettings] = useState(false);
+  const [about, setAbout] = useState(false);
   const [form, setForm] = useState<"new" | "add" | "edit" | null>(null);
   const [sidebar, setSidebar] = useState(() => loadSidebarOpen());
   // Açık/kapalı durumu bir sonraki açılışta da geçerlidir (Plan 12 D11).
@@ -58,6 +61,14 @@ export function Shell() {
     if (r.status === "error") setPrintError(r.error);
   };
   const { push } = useNotices();
+  // Açılışta güncelleme denetimi yalnızca kullanıcı tercihi açtıysa yapılır (Plan 13 D1) ve
+  // bir kez: StrictMode'un ikinci effect koşusu yeni istek göndermez.
+  const updateChecked = useRef(false);
+  useEffect(() => {
+    if (updateChecked.current) return;
+    updateChecked.current = true;
+    void startupUpdateCheck(push);
+  }, [push]);
   const measureRef = useRef<HTMLDivElement>(null);
   const width = useElementWidth(measureRef);
 
@@ -143,6 +154,7 @@ export function Shell() {
           return toggleNotes();
         }
         if (item.command === "sidebar") return printView ? undefined : toggleSidebar();
+        if (item.command === "about") return setAbout(true);
         return setHelp(true);
     }
   };
@@ -161,6 +173,7 @@ export function Shell() {
         onToggleSidebar={toggleSidebar}
         onPalette={() => setPalette(true)}
         onSettings={() => setSettings(true)}
+        onAbout={() => setAbout(true)}
       />
       <NoticeBar />
       <div className="shell-body">
@@ -172,6 +185,7 @@ export function Shell() {
       {help && <HelpDialog onClose={() => setHelp(false)} />}
       {palette && <Palette onClose={() => setPalette(false)} onRun={runPalette} />}
       {settings && <SettingsDialog onClose={() => setSettings(false)} />}
+      {about && <AboutDialog onClose={() => setAbout(false)} />}
       {form && <BirthForm mode={form} onClose={() => setForm(null)} />}
     </div>
   );
diff --git a/app/ui/shell/TopBar.tsx b/app/ui/shell/TopBar.tsx
--- a/app/ui/shell/TopBar.tsx
+++ b/app/ui/shell/TopBar.tsx
@@ -19,10 +19,11 @@ export interface TopBarProps {
   onToggleSidebar(): void;
   onPalette(): void;
   onSettings(): void;
+  onAbout(): void;
 }
 
 /** Üst çubuk (spec §4): solda danışan ve harita (uyarı rozetleriyle), ortada düzen seçici, sağda tema, dil ve baskı. */
-export function TopBar({ themePref, onThemeChange, printView, onTogglePrint, onHelp, onNewChart, onEditChart, sidebarOpen, onToggleSidebar, onPalette, onSettings }: TopBarProps) {
+export function TopBar({ themePref, onThemeChange, printView, onTogglePrint, onHelp, onNewChart, onEditChart, sidebarOpen, onToggleSidebar, onPalette, onSettings, onAbout }: TopBarProps) {
   const { t, i18n } = useTranslation();
   const language = isLanguage(i18n.language) ? i18n.language : DEFAULT_LANGUAGE;
   return (
@@ -74,6 +75,9 @@ export function TopBar({ themePref, onThemeChange, printView, onTogglePrint, onH
         <button type="button" aria-keyshortcuts="?" onClick={onHelp}>
           {t("shortcuts.open")}
         </button>
+        <button type="button" onClick={onAbout}>
+          {t("about.open")}
+        </button>
         {onTogglePrint && (
           <button type="button" aria-keyshortcuts={printView ? undefined : "Meta+P"} onClick={onTogglePrint}>
             {printView ? t("print.back") : t("print.open")}
```

- [ ] **Step 6: Kapılar**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | tail -1
grep -rnE '#[0-9a-fA-F]{3,8}\b' app/ui/about app/ui/update
grep -rnE 'console\.|window\.open|<a ' app/ui/about app/ui/update --include='*.ts' --include='*.tsx' | grep -v '\.test\.'
```
Expected: `60 passed (60)`, `517 passed (517)`; `✓ built`; iki `grep` çıktısız (hex yok; `console`, `window.open` ve bağlantı yok).

- [ ] **Step 7: Commit**

```bash
git add app/ui/update app/ui/about app/ui/App.test.tsx app/ui/shell/Shell.tsx app/ui/shell/TopBar.tsx app/ui/palette/items.ts app/ui/palette/Palette.tsx app/ui/i18n/tr.ts app/ui/i18n/en.ts app/ui/i18n/fr.ts
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "feat(ui): About window with licence, credits and consent-based updates; startup check is off by default"
```
Expected: iki `grep` çıktısız.

---

### Task 4: Uçtan uca: güncelleme akışı

**Files:**
- Modify: `app/e2e/ipc.ts`, `app/e2e/desk.spec.ts`

**Interfaces:**
- Consumes: Task 3'ün metinleri ("Hakkında", "Jyotish Desk hakkında", "Şimdi denetle", "Açılışta güncellemeleri denetle", "Yeni sürüm: 0.9.1 (2026-12-01).", "0.9.1 sürümünü indir ve kur", "Yeni sürüm var: 0.9.1."); `ipcCalls(page, cmd)`.
- Produces: IPC taklidinde `app_info`, `check_for_update`, `install_update` yanıtları (sahte yayın 0.9.1; ağ yok).

- [ ] **Step 1: Başarısız testi yaz**

`app/e2e/desk.spec.ts` sonuna:

```diff
diff --git a/app/e2e/desk.spec.ts b/app/e2e/desk.spec.ts
--- a/app/e2e/desk.spec.ts
+++ b/app/e2e/desk.spec.ts
@@ -236,3 +236,30 @@ test("print sheets are paper-white in the dark theme and, at their longest, each
     expect(await sheet.evaluate((el) => el.scrollWidth <= el.clientWidth)).toBe(true);
   }
 });
+
+test("updates: nothing is checked until asked; About checks, shows the notes and installs the consented version", async ({ page }) => {
+  await mockIpc(page, { session: true });
+  await page.goto("/");
+  await expect(page.getByRole("heading", { level: 1, name: "Örnek Danışan" })).toBeVisible();
+  await page.getByRole("button", { name: "Hakkında" }).click();
+  const about = page.getByRole("dialog", { name: "Jyotish Desk hakkında" });
+  await expect(about.getByText("Sürüm 0.9.0")).toBeVisible();
+  await expect(about.getByText("https://github.com/turgaydogusan/jyotish-desk")).toBeVisible();
+  await expect(about.getByRole("checkbox", { name: "Açılışta güncellemeleri denetle" })).not.toBeChecked();
+  expect(await ipcCalls(page, "check_for_update")).toHaveLength(0);
+
+  await about.getByRole("button", { name: "Şimdi denetle" }).click();
+  await expect(about.getByRole("status")).toHaveText("Yeni sürüm: 0.9.1 (2026-12-01).");
+  await expect(about.getByLabel("0.9.1 sürüm notları")).toHaveText("- Baskı düzeltmesi");
+  expect(await ipcCalls(page, "install_update")).toHaveLength(0);
+  await about.getByRole("button", { name: "0.9.1 sürümünü indir ve kur" }).click();
+  await expect(about.getByRole("status")).toContainText("0.9.1 kuruldu.");
+  expect(await ipcCalls(page, "install_update")).toEqual([{ version: "0.9.1" }]);
+  await expect(about.getByRole("button", { name: "Kapat" })).toBeFocused();
+
+  // Tercih açılınca bir sonraki açılış denetler ve bildirim bandında haber verir.
+  await about.getByRole("checkbox", { name: "Açılışta güncellemeleri denetle" }).check();
+  await page.reload();
+  await expect(page.getByRole("region", { name: "Bildirimler" })).toContainText("Yeni sürüm var: 0.9.1.");
+  expect(await ipcCalls(page, "check_for_update")).toHaveLength(1);
+});
```

- [ ] **Step 2: Testin başarısız olduğunu doğrula**

Run: `pnpm --dir app e2e 2>&1 | tail -8`
Expected: `1 failed`, `11 passed`: yeni test "Sürüm 0.9.0" metnini bulamaz (taklitte `app_info` yok: `no mock for app_info`).

- [ ] **Step 3: IPC taklidine güncelleme komutlarını ekle**

```diff
diff --git a/app/e2e/ipc.ts b/app/e2e/ipc.ts
--- a/app/e2e/ipc.ts
+++ b/app/e2e/ipc.ts
@@ -94,6 +94,13 @@ export async function mockIpc(page: Page, { session, unknownTime = false }: IpcO
           JSON.stringify(args.path) === JSON.stringify(fixture.dasha_children.path) ? fixture.dasha_children.periods : [],
         transits_at: () => fixture.transit,
         print_window: () => null,
+        // Güncelleme: sahte yayın; ağ yoktur. Kurulum yalnızca denetimin bildirdiği sürümü kabul eder.
+        app_info: () => ({ version: "0.9.0", updaterConfigured: true }),
+        check_for_update: () => ({ version: "0.9.1", notes: "- Baskı düzeltmesi", published: "2026-12-01" }),
+        install_update: (args) => {
+          if (args.version !== "0.9.1") throw { code: "update_changed", params: {} };
+          return null;
+        },
       };
       Object.assign(window, {
         __TAURI_INTERNALS__: {
```

- [ ] **Step 4: Kapılar**

```bash
pnpm --dir app build 2>&1 | tail -1
pnpm --dir app e2e 2>&1 | tail -1
pnpm --dir app e2e:visual 2>&1 | tail -1
rm -rf app/test-results
```
Expected: `✓ built`; `12 passed`; görsel `10 passed` (üst çubuktaki yeni düğme temelleri değiştirmez; değiştirirse durulur ve bulgu ilerleme notlarına yazılır, temel yeniden üretilmez).

- [ ] **Step 5: Commit**

```bash
git add app/e2e/ipc.ts app/e2e/desk.spec.ts
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "test(e2e): update flow asks nothing until consent, then checks, shows notes and installs the consented version"
```
Expected: `grep` çıktısız.

---

### Task 5: Sürüm 0.9.0, sürüm denetimi, `latest.json` üreticisi ve yayın notu

**Files:**
- Create: `scripts/release/check-version.mjs`, `scripts/release/latest-json.mjs`, `docs/release/notes/v0.9.0.md`
- Modify: `Cargo.toml`, `Cargo.lock` (cargo üretir), `app/package.json`, `app/src-tauri/tauri.conf.json`, `app/ui/test/fixtures/delhi.json` (üretilir), `app/ui/print/model.test.ts`, `app/ui/print/PrintDocumentView.test.tsx`, `.github/workflows/ci.yml`
- Test: `scripts/release/check-version.test.mjs`, `scripts/release/latest-json.test.mjs`

**Interfaces:**
- Consumes: yok (betikler bağımlılıksızdır; Node'un `node:test` ve `node:assert` modülleri).
- Produces: `check-version.mjs`: `ROOT`, `readVersions(root?) -> Record<dosya, string | null>`, `versionProblems(versions, { tag?, hasNotes? }) -> string[]`; komut satırı `node scripts/release/check-version.mjs [etiket]` sürümü yazdırır (sorun varsa çıkış 1). `latest-json.mjs`: `RELEASE_BASE`, `PLATFORMS`, `buildLatestJson({ version, notes, signature, asset, pubDate })`; komut satırı `node scripts/release/latest-json.mjs <sürüm> <notlar.md> <paket.sig> <paket adı> <çıktı>`.

- [ ] **Step 1: Başarısız testleri yaz**

`scripts/release/check-version.test.mjs`:

```js
import assert from "node:assert/strict";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { readVersions, ROOT, versionProblems } from "./check-version.mjs";

const same = (v) => ({ "Cargo.toml": v, "Cargo.lock": v, "app/package.json": v, "app/src-tauri/tauri.conf.json": v });

test("the repository carries one version in all four places and has its release notes", () => {
  const versions = readVersions();
  const version = versions["Cargo.toml"];
  assert.deepEqual(versionProblems(versions, { hasNotes: existsSync(join(ROOT, `docs/release/notes/v${version}.md`)) }), []);
});

test("a differing file, a malformed version, a wrong tag and missing notes are each reported", () => {
  assert.deepEqual(versionProblems(same("0.9.0"), { tag: "v0.9.0" }), []);
  assert.match(versionProblems({ ...same("0.9.0"), "app/package.json": "0.8.0" })[0], /app\/package\.json: 0\.8\.0 differs/);
  assert.match(versionProblems({ ...same("0.9.0"), "Cargo.lock": null })[0], /Cargo\.lock: version not found/);
  assert.match(versionProblems(same("0.9.0-beta.1"))[0], /not MAJOR\.MINOR\.PATCH/);
  assert.match(versionProblems(same("0.9.0"), { tag: "v0.9.1" })[0], /tag v0\.9\.1 does not match/);
  assert.match(versionProblems(same("0.9.0"), { tag: "0.9.0" })[0], /expected v0\.9\.0/);
  assert.match(versionProblems(same("0.9.0"), { hasNotes: false })[0], /notes\/v0\.9\.0\.md is missing/);
});
```

`scripts/release/latest-json.test.mjs`:

```js
import assert from "node:assert/strict";
import { test } from "node:test";
import { buildLatestJson, PLATFORMS } from "./latest-json.mjs";

const input = {
  version: "0.9.1",
  notes: "## 0.9.1\n\n- Düzeltme\n",
  signature: "dW50cnVzdGVkIGNvbW1lbnQ=\n",
  asset: "Jyotish-Desk_0.9.1_universal.app.tar.gz",
  pubDate: "2026-12-01T10:00:00+03:00",
};

test("both architectures point at the one universal bundle of that tag", () => {
  const json = buildLatestJson(input);
  assert.deepEqual(Object.keys(json), ["version", "notes", "pub_date", "platforms"]);
  assert.equal(json.version, "0.9.1");
  assert.equal(json.notes, "## 0.9.1\n\n- Düzeltme");
  assert.equal(json.pub_date, "2026-12-01T07:00:00.000Z");
  assert.deepEqual(Object.keys(json.platforms), PLATFORMS);
  for (const p of PLATFORMS) {
    assert.deepEqual(json.platforms[p], {
      signature: "dW50cnVzdGVkIGNvbW1lbnQ=",
      url: "https://github.com/turgaydogusan/jyotish-desk/releases/download/v0.9.1/Jyotish-Desk_0.9.1_universal.app.tar.gz",
    });
  }
});

test("a malformed version, asset name, signature or date is refused", () => {
  assert.throws(() => buildLatestJson({ ...input, version: "v0.9.1" }), /version/);
  assert.throws(() => buildLatestJson({ ...input, version: "0.9" }), /version/);
  assert.throws(() => buildLatestJson({ ...input, asset: "Jyotish Desk.app.tar.gz" }), /asset/);
  assert.throws(() => buildLatestJson({ ...input, asset: "Jyotish-Desk.dmg" }), /asset/);
  assert.throws(() => buildLatestJson({ ...input, signature: " \n" }), /signature/);
  assert.throws(() => buildLatestJson({ ...input, signature: "a\nb" }), /signature/);
  assert.throws(() => buildLatestJson({ ...input, pubDate: "yesterday" }), /pub_date/);
});
```

Run: `node --test "scripts/release/*.test.mjs" 2>&1 | grep -E "ℹ (tests|pass|fail)|ERR_MODULE"`
Expected: FAIL (`ERR_MODULE_NOT_FOUND`: `check-version.mjs` ve `latest-json.mjs` yok).

- [ ] **Step 2: Betikleri yaz**

`scripts/release/check-version.mjs`:

```js
// Sürüm tek değerdir ama üç dosyada yazar: çalışma alanı (`Cargo.toml`), arayüz paketi
// (`app/package.json`) ve Tauri yapılandırması (`app/src-tauri/tauri.conf.json`). Üçü de aynı
// olmalı, `Cargo.lock` çalışma alanı paketlerini o sürümle taşımalı ve sürümün yayın notu
// bulunmalıdır. Etiket verilirse (`v0.9.0`) o da aynı sürümü göstermelidir.
//
// Kullanım: node scripts/release/check-version.mjs [etiket]
import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

export const ROOT = join(dirname(fileURLToPath(import.meta.url)), "../..");
const SEMVER = /^\d+\.\d+\.\d+$/;

export function readVersions(root = ROOT) {
  const read = (p) => readFileSync(join(root, p), "utf8");
  const workspace = /\[workspace\.package\][^[]*?\nversion = "([^"]+)"/.exec(read("Cargo.toml"))?.[1] ?? null;
  const lock = /\[\[package\]\]\nname = "jyotish-desk"\nversion = "([^"]+)"/.exec(read("Cargo.lock"))?.[1] ?? null;
  return {
    "Cargo.toml": workspace,
    "Cargo.lock": lock,
    "app/package.json": JSON.parse(read("app/package.json")).version ?? null,
    "app/src-tauri/tauri.conf.json": JSON.parse(read("app/src-tauri/tauri.conf.json")).version ?? null,
  };
}

/** Sorunların listesi; boşsa sürüm tutarlıdır. */
export function versionProblems(versions, { tag = null, hasNotes = true } = {}) {
  const problems = [];
  const entries = Object.entries(versions);
  const version = entries[0][1];
  for (const [file, v] of entries) {
    if (v === null) problems.push(`${file}: version not found`);
    else if (!SEMVER.test(v)) problems.push(`${file}: ${v} is not MAJOR.MINOR.PATCH`);
    else if (v !== version) problems.push(`${file}: ${v} differs from ${entries[0][0]} (${version})`);
  }
  if (tag !== null && tag !== `v${version}`) problems.push(`tag ${tag} does not match version ${version} (expected v${version})`);
  if (!hasNotes) problems.push(`docs/release/notes/v${version}.md is missing`);
  return problems;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  const tag = process.argv[2] ?? null;
  const versions = readVersions();
  const version = versions["Cargo.toml"];
  const problems = versionProblems(versions, { tag, hasNotes: existsSync(join(ROOT, `docs/release/notes/v${version}.md`)) });
  if (problems.length > 0) {
    for (const p of problems) console.error(`version check: ${p}`);
    process.exit(1);
  }
  console.log(version);
}
```

`scripts/release/latest-json.mjs`:

```js
// Güncelleme bildirimi (`latest.json`): Tauri güncelleyicisinin GitHub Releases'ten okuduğu
// dosya. Evrensel (arm64 + x86_64) paket tek dosyadır; iki mimari anahtarı da onu gösterir.
// İmza, `tauri build`in ürettiği `.sig` dosyasının içeriğidir; burada üretilmez ve doğrulanmaz
// (doğrulamayı uygulama, gömülü açık anahtarla yapar).
//
// Kullanım: node scripts/release/latest-json.mjs <sürüm> <notlar.md> <paket.sig> <paket adı> <çıktı>
import { readFileSync, writeFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

export const RELEASE_BASE = "https://github.com/turgaydogusan/jyotish-desk/releases/download";
export const PLATFORMS = ["darwin-aarch64", "darwin-x86_64"];
const SEMVER = /^\d+\.\d+\.\d+$/;
const ASSET = /^[A-Za-z0-9._-]+\.app\.tar\.gz$/;

export function buildLatestJson({ version, notes, signature, asset, pubDate }) {
  if (!SEMVER.test(version)) throw new Error(`version is not MAJOR.MINOR.PATCH: ${version}`);
  if (!ASSET.test(asset)) throw new Error(`asset name must be a space-free .app.tar.gz: ${asset}`);
  const sig = signature.trim();
  if (sig === "" || /\s/.test(sig)) throw new Error("signature must be the single-line content of the .sig file");
  if (Number.isNaN(Date.parse(pubDate))) throw new Error(`pub_date is not a date: ${pubDate}`);
  const entry = { signature: sig, url: `${RELEASE_BASE}/v${version}/${asset}` };
  return {
    version,
    notes: notes.trim(),
    pub_date: new Date(pubDate).toISOString(),
    platforms: Object.fromEntries(PLATFORMS.map((p) => [p, entry])),
  };
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  const [version, notesPath, sigPath, asset, out] = process.argv.slice(2);
  if (!out) {
    console.error("usage: latest-json.mjs <version> <notes.md> <bundle.sig> <asset name> <out>");
    process.exit(2);
  }
  const json = buildLatestJson({
    version,
    notes: readFileSync(notesPath, "utf8"),
    signature: readFileSync(sigPath, "utf8"),
    asset,
    pubDate: new Date().toISOString(),
  });
  writeFileSync(out, `${JSON.stringify(json, null, 2)}\n`);
  console.log(`latest.json: ${version}, ${PLATFORMS.join(" + ")}`);
}
```

Run: `node --test "scripts/release/*.test.mjs" 2>&1 | grep -E "^✖|ℹ (pass|fail)|missing"`
Expected: `pass 3`, `fail 1`: "the repository carries one version…" testi `docs/release/notes/v0.1.0.md is missing` ile düşer (sürüm henüz 0.1.0 ve notu yok).

- [ ] **Step 3: Sürümü 0.9.0 yap ve yayın notunu yaz**

```diff
diff --git a/Cargo.toml b/Cargo.toml
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -9,7 +9,7 @@ members = [
 ]
 
 [workspace.package]
-version = "0.1.0"
+version = "0.9.0"
 edition = "2021"
 license = "AGPL-3.0-or-later"
 rust-version = "1.93"
diff --git a/app/package.json b/app/package.json
--- a/app/package.json
+++ b/app/package.json
@@ -1,7 +1,7 @@
 {
   "name": "jyotish-desk-app",
   "private": true,
-  "version": "0.1.0",
+  "version": "0.9.0",
   "license": "AGPL-3.0-or-later",
   "type": "module",
   "packageManager": "pnpm@11.8.0",
diff --git a/app/src-tauri/tauri.conf.json b/app/src-tauri/tauri.conf.json
--- a/app/src-tauri/tauri.conf.json
+++ b/app/src-tauri/tauri.conf.json
@@ -1,7 +1,7 @@
 {
   "$schema": "https://schema.tauri.app/config/2",
   "productName": "Jyotish Desk",
-  "version": "0.1.0",
+  "version": "0.9.0",
   "identifier": "io.github.turgaydogusan.jyotishdesk",
   "build": {
     "beforeDevCommand": "pnpm dev",
diff --git a/app/ui/print/PrintDocumentView.test.tsx b/app/ui/print/PrintDocumentView.test.tsx
--- a/app/ui/print/PrintDocumentView.test.tsx
+++ b/app/ui/print/PrintDocumentView.test.tsx
@@ -36,7 +36,7 @@ test("cover lists birth data and settings; the warnings section appears only whe
   const cover = sheet("Örnek Danışan");
   expect(within(cover).getByText("New Delhi, Delhi, India")).toBeInTheDocument();
   expect(within(cover).getByText("Lahiri 23°51′")).toBeInTheDocument();
-  expect(within(cover).getByText("Jyotish Desk 0.1.0 · Swiss Ephemeris 2.10.03")).toBeInTheDocument();
+  expect(within(cover).getByText("Jyotish Desk 0.9.0 · Swiss Ephemeris 2.10.03")).toBeInTheDocument();
   expect(within(cover).queryByRole("heading", { name: "Uyarılar" })).toBeNull();
 });
 
diff --git a/app/ui/print/model.test.ts b/app/ui/print/model.test.ts
--- a/app/ui/print/model.test.ts
+++ b/app/ui/print/model.test.ts
@@ -29,7 +29,7 @@ test("cover: stored birth data, the settings the chart was computed with and the
     ["Dasa yılı", "365.25 gün"],
   ]);
   expect(cover.warnings).toEqual([]);
-  expect(cover.credit).toBe("Jyotish Desk 0.1.0 · Swiss Ephemeris 2.10.03");
+  expect(cover.credit).toBe("Jyotish Desk 0.9.0 · Swiss Ephemeris 2.10.03");
 });
 
 test("cover warnings carry the name and the explanation as text", () => {
```

`docs/release/notes/v0.9.0.md`:

```markdown
# Jyotish Desk 0.9.0

İlk kapalı beta sürümü (macOS 13 ve sonrası, Apple silicon ve Intel).

## Bu sürümde

- Doğum girişi, yer atlası ve saat dilimi çözümü; üç biçimde harita (Kuzey, Güney, daire).
- 16 varga, Vimshottari dasa, panchang, ashtakavarga ve transit.
- Danışan veritabanı: arama, etiketler, notlar, oturumun geri yüklenmesi, JSON ve CSV dışa aktarma, JSON içe aktarma.
- Baskı önizleme ve PDF (kapak, D1 ve D9, dasa, graha tablosu).
- Hakkında penceresi ve imzalı güncelleme. Güncelleme denetimi varsayılan olarak kapalıdır.

## Bilinen sınırlar

- Danışan verisi bu Mac'te, şifrelenmemiş bir veritabanında durur; diskinizde FileVault'u açık tutun.
- Yorum metni yoktur; uygulama hesap ve gösterim aracıdır.
- Jagannatha Hora (`.jhd`) içe aktarma bu sürümde kapalıdır.

## Güncelleme

Hakkında ▸ "Şimdi denetle". Yeni sürüm kurulduktan sonra uygulamayı kapatıp yeniden açın.
```

- [ ] **Step 4: Kilit dosyasını ve fikstürü yeniden üret**

Çalışma alanı sürümü hesap motorunun sürümüdür (`ENGINE_VERSION`) ve arayüz fikstüründe yazar.

```bash
cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep -E "stale|test result"
UPDATE_FIXTURES=1 cargo test -p jyotish-desk --test ui_fixture 2>&1 | grep "test result"
git diff --stat -- Cargo.lock app/ui/test/fixtures/delhi.json
grep -o '"engine_version":"[^"]*"' app/ui/test/fixtures/delhi.json | sort -u
```
Expected: ilk komut `delhi.json is stale` ve `FAILED. 1 passed; 1 failed`; ikincisi `2 passed`; `Cargo.lock`'ta beş satır (beş çalışma alanı paketinin sürümü), `delhi.json`'da tek satır değişmiş (`6 insertions(+), 6 deletions(-)`); `"engine_version":"0.9.0"`.

- [ ] **Step 5: CI'a yayın betiği testlerini ekle**

```diff
diff --git a/.github/workflows/ci.yml b/.github/workflows/ci.yml
--- a/.github/workflows/ci.yml
+++ b/.github/workflows/ci.yml
@@ -19,6 +19,8 @@ jobs:
           cache: pnpm
           cache-dependency-path: app/pnpm-lock.yaml
       - run: pnpm --dir app install --frozen-lockfile
+      # Yayın betikleri (sürüm tutarlılığı, latest.json); bağımlılıksız, Node'un kendi test koşucusu.
+      - run: node --test "scripts/release/*.test.mjs"
       - run: pnpm --dir app test
       - run: pnpm --dir app build
       # Uçtan uca arayüz testleri (WebKit, IPC taklidi). Görsel karşılaştırma yalnızca yerelde.
```

- [ ] **Step 6: Kapılar**

```bash
node --test "scripts/release/*.test.mjs" 2>&1 | grep -E "ℹ (pass|fail)"
node scripts/release/check-version.mjs
node scripts/release/check-version.mjs v0.9.1; echo "exit $?"
cargo test --workspace 2>&1 | grep -E "^test result" | grep -v " 0 failed"
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | tail -1
pnpm --dir app e2e:visual 2>&1 | tail -1
rm -rf app/test-results
```
Expected: `pass 4`, `fail 0`; `0.9.0`; `version check: tag v0.9.1 does not match version 0.9.0 (expected v0.9.0)` ve `exit 1`; cargo satırı çıktısız; Vitest `60 passed (60)`, `517 passed (517)`; `✓ built`; görsel `10 passed` (kapaktaki sürüm metni eşik içinde durur; kapak temeli düşerse yalnızca o temel `pnpm --dir app exec playwright test --project=visual -g "print sheet cover" --update-snapshots` ile yeniden üretilir ve commit'e eklenir).

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock app/package.json app/src-tauri/tauri.conf.json app/ui/test/fixtures/delhi.json app/ui/print/model.test.ts app/ui/print/PrintDocumentView.test.tsx .github/workflows/ci.yml scripts/release/check-version.mjs scripts/release/check-version.test.mjs scripts/release/latest-json.mjs scripts/release/latest-json.test.mjs docs/release/notes/v0.9.0.md
git diff --cached -U0 -- . ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "chore(release): version 0.9.0, version check, latest.json builder and first release notes"
```
Expected: iki `grep` çıktısız.

---

### Task 6: Derleme, paket denetimi ve paketleme betikleri (imzasız yol)

**Files:**
- Create: `scripts/release/build.sh`, `scripts/release/verify-bundle.sh`, `scripts/release/package.sh`

**Interfaces:**
- Consumes: `node scripts/release/check-version.mjs` (sürümü yazdırır), `node scripts/release/latest-json.mjs …` (Task 5); `pnpm --dir app tauri build`.
- Produces: `sh scripts/release/build.sh unsigned|signed` (çıktı: `target/universal-apple-darwin/release/bundle/`); `sh scripts/release/verify-bundle.sh unsigned|signed` (her denetim `ok: …`, başarısızlıkta `verify-bundle: FAIL: …` ve çıkış 1); `sh scripts/release/package.sh <klasör>` (`Jyotish-Desk_<sürüm>_universal.dmg`, `.app.tar.gz`, `.app.tar.gz.sig`, `latest.json`, `SHA256SUMS`).

Bu görevin testi betiklerin kendisidir: imzasız derleme gerçek paketi üretir ve `verify-bundle.sh unsigned` onu denetler. İmzalı kip ve `package.sh` sır ister; ajan bunları **çalıştırmaz** (C18), yalnızca sözdizimini ve sırsız reddi denetler. Bu plan yazılırken `package.sh`, atılan bir deneme anahtarıyla imzalanmış paket ve sahte bir `.dmg` ile koşulmuş, beş dosyayı ve geçerli `latest.json`'ı üretmiştir; `--no-sign` ile güncelleme imzasının da atlandığı görülmüştür (imzalı kip bu bayrağı kullanmaz).

- [ ] **Step 1: `verify-bundle.sh`'i yaz ve paket yokken başarısız olduğunu gör**

`scripts/release/verify-bundle.sh`:

```sh
#!/bin/sh
# Üretilen paketi denetler. Her satır bir denetimdir; ilk başarısızlıkta durur.
#
#   sh scripts/release/verify-bundle.sh unsigned|signed
#
# unsigned: evrensel ikili, sürüm, kimlik, en düşük macOS, paket verisi.
# signed:   ayrıca Developer ID imzası, hardened runtime, notarizasyon damgası (.app ve .dmg),
#           Gatekeeper kabulü ve güncelleme paketiyle imza dosyası.
set -eu

MODE="${1:-unsigned}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BUNDLE="$ROOT/target/universal-apple-darwin/release/bundle"
APP="$BUNDLE/macos/Jyotish Desk.app"
VERSION="$(node "$ROOT/scripts/release/check-version.mjs")"
PLIST="$APP/Contents/Info.plist"

fail() {
  echo "verify-bundle: FAIL: $1" >&2
  exit 1
}
ok() {
  echo "ok: $1"
}
plist() {
  /usr/libexec/PlistBuddy -c "Print :$1" "$PLIST"
}

[ -d "$APP" ] || fail "no app bundle at target/universal-apple-darwin/release/bundle/macos"

BIN="$APP/Contents/MacOS/$(plist CFBundleExecutable)"
ARCHS="$(lipo -archs "$BIN")"
case "$ARCHS" in
  *x86_64*arm64* | *arm64*x86_64*) ok "universal binary ($ARCHS)" ;;
  *) fail "binary is not universal: $ARCHS" ;;
esac

[ "$(plist CFBundleShortVersionString)" = "$VERSION" ] || fail "bundle version is not $VERSION"
ok "version $VERSION"
[ "$(plist CFBundleIdentifier)" = "io.github.turgaydogusan.jyotishdesk" ] || fail "unexpected bundle identifier"
ok "identifier"
[ "$(plist LSMinimumSystemVersion)" = "13.0" ] || fail "minimum macOS is not 13.0"
ok "minimum macOS 13.0"

for f in ephe/sepl_18.se1 ephe/semo_18.se1 ephe/sefstars.txt atlas/atlas.sqlite; do
  [ -s "$APP/Contents/Resources/$f" ] || fail "missing resource $f"
done
ok "ephemeris and atlas data"

if [ "$MODE" = "unsigned" ]; then
  echo "verify-bundle: unsigned bundle is well formed (not signed, not notarized, no updater package)"
  exit 0
fi
[ "$MODE" = "signed" ] || fail "usage: verify-bundle.sh unsigned|signed"

codesign --verify --deep --strict "$APP" || fail "codesign verification"
ok "code signature"
SIGN_INFO="$(codesign -dvv "$APP" 2>&1)"
echo "$SIGN_INFO" | grep -q '^Authority=Developer ID Application: ' || fail "not signed with a Developer ID Application certificate"
ok "Developer ID Application"
echo "$SIGN_INFO" | grep -q 'flags=.*runtime' || fail "hardened runtime is off"
ok "hardened runtime"
xcrun stapler validate -q "$APP" || fail "no notarization ticket stapled to the app"
ok "app notarized and stapled"
spctl --assess --type execute "$APP" || fail "Gatekeeper rejects the app"
ok "Gatekeeper accepts the app"

DMG="$BUNDLE/dmg/Jyotish Desk_${VERSION}_universal.dmg"
[ -s "$DMG" ] || fail "no dmg"
xcrun stapler validate -q "$DMG" || fail "no notarization ticket stapled to the dmg"
spctl --assess --type open --context context:primary-signature "$DMG" || fail "Gatekeeper rejects the dmg"
ok "dmg notarized, stapled and accepted"

[ -s "$APP.tar.gz" ] || fail "no updater package (.app.tar.gz)"
[ -s "$APP.tar.gz.sig" ] || fail "no updater signature (.sig)"
ok "updater package and signature"
echo "verify-bundle: signed bundle is ready to publish"
```

Run: `sh -n scripts/release/verify-bundle.sh && sh scripts/release/verify-bundle.sh unsigned; echo "exit $?"`
Expected: `verify-bundle: FAIL: no app bundle at target/universal-apple-darwin/release/bundle/macos`, `exit 1`.

- [ ] **Step 2: `build.sh` ve `package.sh`**

`scripts/release/build.sh`:

```sh
#!/bin/sh
# Evrensel (arm64 + x86_64) macOS paketi üretir (spec §10).
#
#   sh scripts/release/build.sh unsigned   imzasız deneme: yalnızca .app; sır gerekmez
#   sh scripts/release/build.sh signed     imzalı ve notarize edilmiş .app + .dmg + güncelleme paketi
#
# "signed" kipinde Tauri CLI'nin okuduğu ortam değişkenleri çağıran tarafından verilir
# (docs/release/RELEASING.md): APPLE_CERTIFICATE, APPLE_CERTIFICATE_PASSWORD,
# APPLE_SIGNING_IDENTITY, APPLE_API_KEY, APPLE_API_ISSUER, APPLE_API_KEY_PATH,
# TAURI_SIGNING_PRIVATE_KEY, TAURI_SIGNING_PRIVATE_KEY_PASSWORD. Bu betik değerlerini yazdırmaz;
# yalnızca var olup olmadıklarına bakar.
set -eu

MODE="${1:-unsigned}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TARGET=universal-apple-darwin

case "$MODE" in
  unsigned)
    # --no-sign: kimlik aranmaz. Güncelleme paketi üretilmez (imza anahtarı gerekmez).
    pnpm --dir "$ROOT/app" tauri build --ci --target "$TARGET" --bundles app --no-sign
    ;;
  signed)
    for name in APPLE_CERTIFICATE APPLE_CERTIFICATE_PASSWORD APPLE_SIGNING_IDENTITY \
      APPLE_API_KEY APPLE_API_ISSUER APPLE_API_KEY_PATH \
      TAURI_SIGNING_PRIVATE_KEY TAURI_SIGNING_PRIVATE_KEY_PASSWORD; do
      eval "value=\${$name:-}"
      if [ -z "$value" ]; then
        echo "build.sh: $name is not set; a signed build needs all eight values" >&2
        exit 2
      fi
    done
    if ! grep -q '"pubkey": "[A-Za-z0-9+/=]\{100,\}"' "$ROOT/app/src-tauri/tauri.conf.json"; then
      echo "build.sh: plugins.updater.pubkey is empty; a signed build without it could never be updated" >&2
      exit 2
    fi
    # Güncelleme paketi (.app.tar.gz + .sig) yalnızca imzalı derlemede üretilir.
    pnpm --dir "$ROOT/app" tauri build --ci --target "$TARGET" --bundles app,dmg \
      --config '{"bundle":{"createUpdaterArtifacts":true}}'
    ;;
  *)
    echo "usage: build.sh unsigned|signed" >&2
    exit 2
    ;;
esac

echo "bundle: $ROOT/target/$TARGET/release/bundle"
```

`scripts/release/package.sh`:

```sh
#!/bin/sh
# İmzalı derlemenin yayın dosyalarını boşluksuz adlarla bir klasöre toplar ve `latest.json`
# üretir. GitHub, yüklenen dosya adlarındaki boşlukları noktaya çevirir; adlar burada sabitlenir
# ki `latest.json`'daki adres yüklenen dosyayla aynı olsun. İmza dosyanın içeriğini imzalar,
# ad değişikliği imzayı bozmaz.
#
#   sh scripts/release/package.sh <çıktı klasörü>
set -eu

OUT="${1:?usage: package.sh <out dir>}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BUNDLE="$ROOT/target/universal-apple-darwin/release/bundle"
VERSION="$(node "$ROOT/scripts/release/check-version.mjs")"
NAME="Jyotish-Desk_${VERSION}_universal"

mkdir -p "$OUT"
cp "$BUNDLE/dmg/Jyotish Desk_${VERSION}_universal.dmg" "$OUT/$NAME.dmg"
cp "$BUNDLE/macos/Jyotish Desk.app.tar.gz" "$OUT/$NAME.app.tar.gz"
cp "$BUNDLE/macos/Jyotish Desk.app.tar.gz.sig" "$OUT/$NAME.app.tar.gz.sig"
node "$ROOT/scripts/release/latest-json.mjs" "$VERSION" "$ROOT/docs/release/notes/v$VERSION.md" \
  "$OUT/$NAME.app.tar.gz.sig" "$NAME.app.tar.gz" "$OUT/latest.json"
(cd "$OUT" && shasum -a 256 "$NAME.dmg" "$NAME.app.tar.gz" > SHA256SUMS)
ls -1 "$OUT"
```

```bash
sh -n scripts/release/build.sh && sh -n scripts/release/package.sh && echo syntax-ok
sh scripts/release/build.sh signed; echo "exit $?"
sh scripts/release/build.sh nonsense; echo "exit $?"
```
Expected: `syntax-ok`; `build.sh: APPLE_CERTIFICATE is not set; a signed build needs all eight values` ve `exit 2`; `usage: build.sh unsigned|signed` ve `exit 2`. Ortamda `APPLE_*` ya da `TAURI_SIGNING_*` değişkeni tanımlıysa ajan durur ve sahibine bildirir (bu değerlerle derleme yapılmaz).

- [ ] **Step 3: İmzasız evrensel paketi derle ve denetle**

```bash
rustup target list --installed | grep -c "apple-darwin"
sh scripts/release/build.sh unsigned 2>&1 | tail -6
sh scripts/release/verify-bundle.sh unsigned; echo "exit $?"
```
Expected: `2` (iki Apple hedefi kurulu; değilse `rustup target add x86_64-apple-darwin`); derleme `Warn Skipping signing due to --no-sign flag.` ve `Finished 1 bundle at:` ile biter (ilk kez yaklaşık on dakika; iki mimari sıfırdan derlenir); denetim:

```
ok: universal binary (x86_64 arm64)
ok: version 0.9.0
ok: identifier
ok: minimum macOS 13.0
ok: ephemeris and atlas data
verify-bundle: unsigned bundle is well formed (not signed, not notarized, no updater package)
exit 0
```

Yerelde `stripping debug info with \`rust-objcopy\` failed` uyarısı görülebilir (araç zincirinin `libLLVM` yolu); derlemeyi durdurmaz ve CI'da görülmez. Üretilen `.app` çalıştırılmaz (Global Constraints).

- [ ] **Step 4: İmzalı kipin imzasız pakette durduğunu gör**

Run: `sh scripts/release/verify-bundle.sh signed; echo "exit $?"`
Expected: beş `ok` satırından sonra `…Jyotish Desk.app: code object is not signed at all`, `verify-bundle: FAIL: codesign verification` ve `exit 1`. İmzalı kipin başarılı koşusu İNSAN ADIMI 7'dedir.

- [ ] **Step 5: Commit**

```bash
git status --short
git add scripts/release/build.sh scripts/release/verify-bundle.sh scripts/release/package.sh
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "build(release): universal macOS build, bundle verification and release packaging scripts"
```
Expected: `git status --short` yalnızca üç yeni betiği gösterir (`target/` yok sayılır); `grep` çıktısız.

---

### Task 7: Yayın işi (`release.yml`)

**Files:**
- Create: `.github/workflows/release.yml`
- Test: `scripts/release/workflow.test.mjs`

**Interfaces:**
- Consumes: `scripts/release/{check-version.mjs, build.sh, verify-bundle.sh, package.sh}` (Task 5, 6); `ROOT` (`check-version.mjs`).
- Produces: `release` işi. Sır adları (İNSAN ADIMI 5 bunları ekler): `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_API_KEY_ID`, `APPLE_API_ISSUER`, `APPLE_API_KEY_P8`, `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. İş çıktıları: `unsigned-dry-run` ya da `signed-release`; etikette ve sırlarla taslak GitHub yayını.

Eylem SHA'ları bu plan yazılırken `gh api repos/<eylem>/commits/<etiket> --jq .sha` ile okunmuştur (`actions/checkout@v4`, `pnpm/action-setup@v4`, `actions/setup-node@v4`, `actions/upload-artifact@v4`, `dtolnay/rust-toolchain@stable`). Yürütücü aynı komutla doğrulayabilir; SHA'yı yenilemek ayrı bir iştir, bu görevde değiştirilmez.

- [ ] **Step 1: Başarısız testi yaz**

`scripts/release/workflow.test.mjs`:

```js
// Yayın işinin (release.yml) güvenlik kuralları metin düzeyinde denetlenir: sırlara dokunan iş
// yalnızca etiketle ya da elle başlar, eylemler commit SHA'sıyla sabittir, önbellek yoktur,
// hiçbir adım bir sırrı yazdırmaz ve yayın taslak olarak oluşturulur.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT } from "./check-version.mjs";

const yml = readFileSync(join(ROOT, ".github/workflows/release.yml"), "utf8");
const code = yml
  .split("\n")
  .filter((l) => !l.trim().startsWith("#"))
  .join("\n");

const SECRETS = [
  "APPLE_CERTIFICATE",
  "APPLE_CERTIFICATE_PASSWORD",
  "APPLE_SIGNING_IDENTITY",
  "APPLE_API_KEY_ID",
  "APPLE_API_ISSUER",
  "APPLE_API_KEY_P8",
  "TAURI_SIGNING_PRIVATE_KEY",
  "TAURI_SIGNING_PRIVATE_KEY_PASSWORD",
];

test("the release job starts only from a version tag or by hand", () => {
  assert.match(code, /^on:\n  push:\n    tags: \["v\*"\]\n  workflow_dispatch:\n/m);
  assert.doesNotMatch(code, /pull_request|schedule:|branches:/);
});

test("every action is pinned to a commit and nothing is cached", () => {
  const uses = [...code.matchAll(/uses: (\S+)/g)].map((m) => m[1]);
  assert.ok(uses.length >= 5);
  for (const u of uses) assert.match(u, /^[\w-]+\/[\w-]+@[0-9a-f]{40}$/, u);
  assert.doesNotMatch(code, /rust-cache|actions\/cache|cache:/);
});

test("the eight secrets are the documented ones and are never echoed", () => {
  const used = [...new Set([...code.matchAll(/secrets\.(\w+)/g)].map((m) => m[1]))].sort();
  assert.deepEqual(used, [...SECRETS].sort());
  // Bir sır yalnızca `env:` eşlemesinde geçer; `run:` satırlarında `${{ secrets… }}` yoktur.
  for (const line of code.split("\n")) {
    if (line.includes("secrets.")) assert.match(line, /^\s+\w+: \$\{\{ secrets\.\w+ \}\}$/, line);
  }
  assert.doesNotMatch(code, /set -x|echo "?\$(S[1-8]|APPLE_|TAURI_SIGNING)/);
});

test("the default token is read-only; only the release job may write, and the release is a draft", () => {
  assert.match(code, /^permissions:\n  contents: read\n/m);
  assert.equal([...code.matchAll(/contents: write/g)].length, 1);
  assert.match(code, /gh release create "\$GITHUB_REF_NAME" --draft --verify-tag/);
  assert.match(code, /if: steps\.mode\.outputs\.signed == 'true' && github\.ref_type == 'tag'/);
  assert.match(code, /persist-credentials: false/);
});

test("without the secrets the build is an unsigned dry run that is still verified", () => {
  assert.match(code, /sh scripts\/release\/build\.sh unsigned\n\s+sh scripts\/release\/verify-bundle\.sh unsigned/);
  assert.match(code, /sh scripts\/release\/build\.sh signed/);
  assert.match(code, /sh scripts\/release\/verify-bundle\.sh signed/);
  assert.match(code, /if: always\(\)\n\s+run: rm -f "\$RUNNER_TEMP\/notary-key\.p8"/);
});
```

Run: `node --test "scripts/release/*.test.mjs" 2>&1 | grep -E "ℹ (pass|fail)|ENOENT" | head -4`
Expected: FAIL (`ENOENT: … .github/workflows/release.yml`); `pass 4`, `fail 1` (dosya okunamadığı için test dosyası tek başarısızlık sayılır).

- [ ] **Step 2: İş dosyasını yaz**

`.github/workflows/release.yml`:

```yaml
name: release

# Etiket (`v0.9.0`) atılınca evrensel macOS paketi üretilir (spec §8, §10).
#  - Sekiz sırrın hepsi varsa: Developer ID ile imzalı, notarize edilmiş .app + .dmg, imzalı
#    güncelleme paketi ve `latest.json`; hepsi TASLAK bir GitHub yayınına yüklenir. Yayını sahibi
#    elle yayımlar; güncelleyici taslağı görmez.
#  - Sırlardan biri bile yoksa: imzasız deneme derlemesi yapılır, paket denetlenir ve iş çıktısı
#    olarak saklanır; yayın oluşturulmaz.
# Elle çalıştırma (workflow_dispatch) hiçbir zaman yayın oluşturmaz.
# Sırlar `release` ortamındadır (docs/release/RELEASING.md); hiçbir adım değerlerini yazdırmaz.
# Eylemler commit SHA'sıyla sabitlenir ve bu işte önbellek kullanılmaz (scripts/release/workflow.test.mjs).

on:
  push:
    tags: ["v*"]
  workflow_dispatch:

permissions:
  contents: read

concurrency:
  group: release-${{ github.ref }}
  cancel-in-progress: false

jobs:
  macos:
    runs-on: macos-15
    timeout-minutes: 120
    environment: release
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4
        with:
          fetch-depth: 0
          persist-credentials: false
      - uses: pnpm/action-setup@b906affcce14559ad1aafd4ab0e942779e9f58b1 # v4
        with:
          package_json_file: app/package.json
      - uses: actions/setup-node@49933ea5288caeca8642d1e84afbd3f7d6820020 # v4
        with:
          node-version: 24
      - uses: dtolnay/rust-toolchain@39008827e6b4319a349e9a42dfc90579dd25f1ca # stable
        with:
          toolchain: stable
          targets: aarch64-apple-darwin,x86_64-apple-darwin

      - name: Version, tag and branch
        id: version
        run: |
          if [ "$GITHUB_REF_TYPE" = "tag" ]; then
            node scripts/release/check-version.mjs "$GITHUB_REF_NAME"
            # Etiket ana daldaki bir commit'i göstermelidir.
            git merge-base --is-ancestor "$GITHUB_SHA" origin/main
          fi
          echo "version=$(node scripts/release/check-version.mjs)" >> "$GITHUB_OUTPUT"

      - name: Signing secrets present?
        id: mode
        env:
          S1: ${{ secrets.APPLE_CERTIFICATE }}
          S2: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          S3: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          S4: ${{ secrets.APPLE_API_KEY_ID }}
          S5: ${{ secrets.APPLE_API_ISSUER }}
          S6: ${{ secrets.APPLE_API_KEY_P8 }}
          S7: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          S8: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        run: |
          if [ -n "$S1" ] && [ -n "$S2" ] && [ -n "$S3" ] && [ -n "$S4" ] && [ -n "$S5" ] && [ -n "$S6" ] && [ -n "$S7" ] && [ -n "$S8" ]; then
            echo "signed=true" >> "$GITHUB_OUTPUT"
            echo "Signing secrets are present: signed build."
          else
            echo "signed=false" >> "$GITHUB_OUTPUT"
            echo "::warning::Signing secrets are missing: unsigned dry-run build, no release is created."
          fi

      - run: pnpm --dir app install --frozen-lockfile
      - run: node --test "scripts/release/*.test.mjs"

      - name: Build (unsigned dry run)
        if: steps.mode.outputs.signed != 'true'
        run: |
          sh scripts/release/build.sh unsigned
          sh scripts/release/verify-bundle.sh unsigned
          ditto -c -k --keepParent "target/universal-apple-darwin/release/bundle/macos/Jyotish Desk.app" "$RUNNER_TEMP/Jyotish-Desk_${{ steps.version.outputs.version }}_UNSIGNED.zip"
      - if: steps.mode.outputs.signed != 'true'
        uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4
        with:
          name: unsigned-dry-run
          path: ${{ runner.temp }}/Jyotish-Desk_${{ steps.version.outputs.version }}_UNSIGNED.zip
          retention-days: 7

      - name: Build, sign and notarize
        if: steps.mode.outputs.signed == 'true'
        env:
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          APPLE_API_KEY: ${{ secrets.APPLE_API_KEY_ID }}
          APPLE_API_ISSUER: ${{ secrets.APPLE_API_ISSUER }}
          APPLE_API_KEY_P8: ${{ secrets.APPLE_API_KEY_P8 }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        run: |
          umask 077
          export APPLE_API_KEY_PATH="$RUNNER_TEMP/notary-key.p8"
          printf '%s\n' "$APPLE_API_KEY_P8" > "$APPLE_API_KEY_PATH"
          sh scripts/release/build.sh signed
          # Tauri .app'i notarize eder ve damgalar; .dmg ayrıca gönderilir ve damgalanır.
          DMG="target/universal-apple-darwin/release/bundle/dmg/Jyotish Desk_${{ steps.version.outputs.version }}_universal.dmg"
          xcrun notarytool submit "$DMG" --key "$APPLE_API_KEY_PATH" --key-id "$APPLE_API_KEY" --issuer "$APPLE_API_ISSUER" --wait
          xcrun stapler staple "$DMG"
      - name: Remove the notary key file
        if: always()
        run: rm -f "$RUNNER_TEMP/notary-key.p8"

      - name: Verify and package
        if: steps.mode.outputs.signed == 'true'
        run: |
          sh scripts/release/verify-bundle.sh signed
          sh scripts/release/package.sh "$RUNNER_TEMP/release"
      - if: steps.mode.outputs.signed == 'true'
        uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4
        with:
          name: signed-release
          path: ${{ runner.temp }}/release/
          retention-days: 14

      - name: Draft GitHub release
        if: steps.mode.outputs.signed == 'true' && github.ref_type == 'tag'
        env:
          GH_TOKEN: ${{ github.token }}
          VERSION: ${{ steps.version.outputs.version }}
        run: |
          gh release create "$GITHUB_REF_NAME" --draft --verify-tag \
            --title "Jyotish Desk $VERSION" \
            --notes-file "docs/release/notes/v$VERSION.md" \
            "$RUNNER_TEMP"/release/*
```

- [ ] **Step 3: Kapılar**

```bash
node --test "scripts/release/*.test.mjs" 2>&1 | grep -E "ℹ (pass|fail)"
ruby -ryaml -e 'y = YAML.load_file(".github/workflows/release.yml"); puts y["jobs"]["macos"]["steps"].size'
```
Expected: `pass 9`, `fail 0`; `15` (dosya geçerli YAML'dir, on beş adım). Her sırrın yalnızca bir `env:` eşlemesinde geçtiğini ve hiçbir adımın sır yazdırmadığını `workflow.test.mjs` denetler. `actionlint` kuruluysa `actionlint .github/workflows/release.yml` çıktısızdır (kurulu değilse atlanır; bağımlılık eklenmez).

İş bu görevde **çalıştırılmaz**: ajan etiket atmaz, `git push` ve `gh workflow run` yapmaz (C18). İlk koşu İNSAN ADIMI 7'dir; sırlar eklenmeden atılan bir etiket ya da elle başlatma imzasız yolu dener.

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/release.yml scripts/release/workflow.test.mjs
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "ci(release): tag builds the universal bundle; signs, notarizes and drafts a release when the secrets exist, dry-runs unsigned otherwise"
```
Expected: `grep` çıktısız.

---

### Task 8: Belgeler: yayın rehberi, yayın notu şablonu, beta rehberi; README ve spec

**Files:**
- Create: `docs/release/RELEASING.md`, `docs/release/release-notes-template.md`, `docs/beta/beta-guide.md`, `docs/beta/feedback-template.md`
- Modify: `README.md`, `docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md`

**Interfaces:**
- Consumes: Task 5–7'nin betik adları, sır adları ve dosya adları; Task 3'ün arayüz metinleri ("Hakkında", "Şimdi denetle", "Açılışta güncellemeleri denetle", "Kurulu sürüm güncel.").
- Produces: sahibin izleyeceği İNSAN ADIMI 1–9 (`RELEASING.md`); beta katılımcısına verilecek rehber ve geri bildirim başlıkları.

Belgelerde yer tutucu değer yoktur: geri bildirim adresi davet iletisindedir (İNSAN ADIMI 9), marka taraması kaydı İNSAN ADIMI 6 tamamlanınca sahibi tarafından yazılır.

- [ ] **Step 1: Yayın rehberi**

`docs/release/RELEASING.md`:

````markdown
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
````

- [ ] **Step 2: Yayın notu şablonu**

`docs/release/release-notes-template.md`:

```markdown
# Jyotish Desk X.Y.Z

Bu dosya `docs/release/notes/vX.Y.Z.md` olarak kopyalanır ve doldurulur. İçeriği hem GitHub
yayınının metni hem de uygulamanın Hakkında penceresinde gösterilen güncelleme notudur: düz metin
olarak okunur, kısa tutulur (uygulama ilk 4000 karakteri gösterir). Danışan adı ya da verisi,
hata raporundan kopyalanmış kişisel bilgi yazılmaz.

## Yenilikler

- Kullanıcının göreceği değişiklik, bir cümle.

## Düzeltmeler

- Ne bozuktu, şimdi nasıl davranıyor.

## Bilinen sınırlar

- Varsa; yoksa bu başlık silinir.

## Güncelleme

Hakkında ▸ "Şimdi denetle". Yeni sürüm kurulduktan sonra uygulamayı kapatıp yeniden açın.
```

- [ ] **Step 3: Beta rehberi ve geri bildirim başlıkları**

`docs/beta/beta-guide.md`:

```markdown
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
```

`docs/beta/feedback-template.md`:

```markdown
# Geri bildirim başlıkları

Her konu için ayrı bir ileti yazın. Gerçek danışan adı, doğum verisi ya da not eklemeyin.

- **Jyotish Desk sürümü:** (Hakkında penceresinin ilk satırı)
- **macOS sürümü ve Mac türü:** (ör. macOS 15.2, Apple silicon)
- **Tür:** hesap farkı / hata / kullanım zorluğu / eksik özellik
- **Ne yaptınız:** adım adım
- **Ne bekliyordunuz:**
- **Ne oldu:**
- **Hesap farkıysa:** deneme doğum verisi (tarih, saat, yer), hesap ayarları (ayanamsa, düğüm,
  ev sistemi, dasa yılı), karşılaştırdığınız kaynak ve o kaynaktaki değer
- **Ne sıklıkla oluyor:** her zaman / bazen / bir kez
- **Ekran görüntüsü:** (isteğe bağlı; yalnızca deneme kaydıyla)
```

- [ ] **Step 4: README ve spec**

```diff
diff --git a/README.md b/README.md
--- a/README.md
+++ b/README.md
@@ -33 +33,12 @@ Rust komut imzaları değişince TS bağlamalarını yeniden üretin:
 `app/ui/bindings.ts` elle düzenlenmez.
+
+## Yayın ve beta
+
+- Yayın akışı ve yalnızca sahibin yapabileceği adımlar (sertifika, notarizasyon anahtarı,
+  güncelleme anahtarı, GitHub sırları): `docs/release/RELEASING.md`.
+- İmzasız deneme paketi (sır gerekmez):
+
+      sh scripts/release/build.sh unsigned
+      sh scripts/release/verify-bundle.sh unsigned
+
+- Kapalı beta katılımcıları için: `docs/beta/beta-guide.md`.
diff --git a/docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md b/docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md
--- a/docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md
+++ b/docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md
@@ -215,2 +215,5 @@ FTS5: clients.name, client_notes.body
 - **Güncelleme:** Tauri updater, imzalı, GitHub Releases üzerinden. Kontrol kullanıcı onayıyla yapılır.
+  - Denetim varsayılan olarak kapalıdır: kullanıcı Hakkında penceresinde "Şimdi denetle"ye basar ya da açılışta denetimi açar (Plan 13 D1). Kurulum, bulunan sürüm için ayrı bir onaydır.
+  - Ağa giden tek istek `releases/latest/download/latest.json` adresine yapılır; telemetri yoktur. İmzası gömülü açık anahtarla doğrulanmayan ve imzadaki sürümü bildirilen sürümle aynı olmayan paket kurulmaz.
+  - Yeni sürüm uygulama kapatılıp açılınca çalışır; uygulama kendini yeniden başlatmaz.
 - **İsim kuralları:**
```

- [ ] **Step 5: Belgelerin kodla tutarlılığını denetle**

```bash
for s in APPLE_CERTIFICATE APPLE_CERTIFICATE_PASSWORD APPLE_SIGNING_IDENTITY APPLE_API_KEY_ID APPLE_API_ISSUER APPLE_API_KEY_P8 TAURI_SIGNING_PRIVATE_KEY TAURI_SIGNING_PRIVATE_KEY_PASSWORD; do
  echo "$s $(grep -c "secrets\.$s }}" .github/workflows/release.yml) $(grep -c "\`$s\`\|gh secret set $s " docs/release/RELEASING.md)"
done
grep -c "^## İNSAN ADIMI [1-9] " docs/release/RELEASING.md
for t in "Şimdi denetle" "Açılışta güncellemeleri denetle" "Kurulu sürüm güncel." "Hakkında"; do grep -q "$t" app/ui/i18n/tr.ts && grep -rq "$t" docs/release docs/beta && echo "ok: $t"; done
grep -n "io.github.turgaydogusan.jyotishdesk" docs/beta/beta-guide.md app/src-tauri/tauri.conf.json | cut -c1-90
```
Expected: sekiz satırın her birinde iki sayı da 1 ya da daha büyük; `9`; dört `ok:` satırı; paket kimliği iki dosyada da aynı.

- [ ] **Step 6: Commit**

```bash
git add docs/release/RELEASING.md docs/release/release-notes-template.md docs/beta/beta-guide.md docs/beta/feedback-template.md README.md docs/superpowers/specs/2026-11-24-jyotish-desk-v1-design.md
git diff --cached -U0 | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff --cached --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git commit -m "docs(release): releasing guide with the owner-only steps, release notes template, closed beta guide and feedback headings"
```
Expected: iki `grep` çıktısız.

---

### Task 9: Son doğrulama, elle denetim kapısı ve insan adımlarının devri

Kod değişmez; bulunan bir hata kendi düzeltme commit'iyle kapanır ve bu görev baştan koşulur.

**Files:**
- Yok (yalnızca doğrulama). İlerleme notları depo dışıdır; `git add` edilmez.

**Interfaces:**
- Consumes: Task 1–8'in tamamı.
- Produces: yeşil kapılar, elle denetim sonucu ve sahibine devredilen insan adımları listesi (ilerleme notlarına yazılır).

- [ ] **Step 1: Rust kapıları**

```bash
export PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace 2>&1 | grep -E "^test result|Running"
cargo deny check 2>&1 | tail -1
PATH="$HOME/.rustup/toolchains/1.93-aarch64-apple-darwin/bin:$PATH" cargo check -p store -p jyotish-desk --all-targets 2>&1 | tail -1
git diff 70e2324..HEAD --stat -- deny.toml app/pnpm-lock.yaml app/src-tauri/capabilities crates/store/src/schema.rs
```
Expected: fmt ve clippy çıktısız; testler: `jyotish-core` lib `110 passed`, golden `8 passed`, `store` lib `81 passed; 0 failed; 1 ignored`, `jyotish-desk` lib `44 passed`, `bindings` `1 passed`, `compute_chart` `1 passed`, `open_client` `1 passed`, `ui_fixture` `2 passed`; deny `advisories ok, bans ok, licenses ok, sources ok`; MSRV `Finished`; son `git diff --stat` çıktısız (lisans listesi, npm kilidi, yetenek dosyası ve şema değişmedi).

- [ ] **Step 2: Arayüz ve betik kapıları**

```bash
pnpm --dir app test 2>&1 | grep -E "Test Files|Tests "
pnpm --dir app build 2>&1 | tail -1
pnpm --dir app e2e 2>&1 | tail -1
pnpm --dir app e2e:visual 2>&1 | tail -1
rm -rf app/test-results
node --test "scripts/release/*.test.mjs" 2>&1 | grep -E "ℹ (pass|fail)"
sh scripts/release/build.sh unsigned 2>&1 | tail -3
sh scripts/release/verify-bundle.sh unsigned
git status --short
```
Expected: `Test Files  60 passed (60)`, `Tests  517 passed (517)`; `✓ built`; e2e `12 passed`; görsel `10 passed`; betik testleri `pass 9`, `fail 0`; derleme `Finished 1 bundle at:`; denetim beş `ok` satırı ve `unsigned bundle is well formed`; `git status --short` boş.

- [ ] **Step 3: Kural taramaları (`70e2324`'ten bu yana)**

```bash
git diff 70e2324..HEAD -U0 -- . ':!Cargo.lock' ':!app/ui/test/fixtures/delhi.json' | grep '^+' | grep -v '^+++' | grep -inE 'k(a|ā|aa)la|\bcala\b'
git log 70e2324..HEAD --format=%B | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff 70e2324..HEAD --name-only | grep -inE 'k(a|ā|aa)la|\bcala\b'
git diff 70e2324..HEAD --name-only | grep -vE '^(app|crates|scripts|docs|\.github)/|^(Cargo\.(toml|lock)|README\.md)$'
echo "trailers: $(git log 70e2324..HEAD --format=%B | grep -c '^Co-Authored-By: ') commits: $(git rev-list --count 70e2324..HEAD)"
git diff 70e2324..HEAD -U0 -- app/ui | grep '^+' | grep -E 'console\.(log|info|warn|error|debug)'
git diff 70e2324..HEAD -U0 -- app/ui | grep '^+' | grep -E '\b(alert|confirm|prompt)\(|window\.(print|open)\('
grep -n '"updater:\|"dialog:\|"fs:\|"process:\|"shell:\|"opener:' app/src-tauri/capabilities/*.json
grep -rn 'tauri_plugin_fs\|tauri_plugin_process\|tauri_plugin_shell' app/src-tauri/src
grep -c '"pubkey": ""' app/src-tauri/tauri.conf.json
git diff 70e2324..HEAD --name-only | grep -iE '\.(p12|p8|pem|key|cer|pub)$'
git diff 70e2324..HEAD -U0 | grep '^+' | grep -iE 'BEGIN [A-Z ]*PRIVATE KEY|untrusted comment|APPLE_PASSWORD'
git tag --points-at HEAD
```
Expected: `trailers:` satırındaki iki sayı eşit; `grep -c '"pubkey": ""'` `1` yazar (açık anahtar bu planda boş durur; sahibi doldurduysa `0` yazar ve bu beklenen durumdur); bunların dışındaki her komut çıktısızdır (çıktısız `grep` 1 ile çıkar; beklenen budur): yasaklı ad yok, beklenmeyen yerde dosya yok, `console` ve yasak çağrı yok, yeni yetenek izni ve eklenti yok, depoda anahtar dosyası ya da anahtar metni yok, ajan etiket atmamış.

- [ ] **Step 4: Elle denetim kapısı (`pnpm --dir app tauri dev`)**

Gerçek uygulamada, deneme danışanlarıyla (gerçek danışan verisi kullanılmaz). Her madde ilerleme notlarına "geçti" ya da bulgu olarak yazılır; bulgu varsa düzeltme commit'i ve bu görev baştan. Bu denetim sır gerektirmez; açık anahtar boşken ("yapılandırılmamış" derleme) yapılır.

- [ ] Üst çubukta "Hakkında" düğmesi vardır; pencere "Sürüm 0.9.0", lisans satırı, kaynak kod adresi, Swiss Ephemeris ve GeoNames atıflarını gösterir. Adres fareyle seçilip kopyalanır; tıklamak tarayıcı açmaz.
- [ ] Güncelleme bölümünde yalnızca "Bu derlemede güncelleme denetimi yok (imza anahtarı gömülmemiş)." yazar; onay kutusu ve "Şimdi denetle" düğmesi yoktur.
- [ ] `⌘K` → "Hakkında ve güncellemeler" aynı pencereyi açar; Esc kapatır ve odak açan öğeye döner; pencere açıkken `⌘P`, `⌘N` ve `⌘1` hiçbir şey yapmaz.
- [ ] Uygulama açılırken ve Hakkında penceresi açılıp kapatılırken ağ isteği yapılmaz: Little Snitch, LuLu ya da `nettop -p $(pgrep -x jyotish-desk)` uygulama sürecinde bağlantı göstermez (web görünümünün `localhost:1420` geliştirme sunucusu bağlantısı ayrı süreçtedir ve yalnızca `tauri dev`'e özgüdür).
- [ ] Koyu temada ve İngilizce ile Fransızca dilde pencere okunur; metinler taşmaz. VoiceOver pencereyi başlığıyla, güncelleme bölümünü "Güncelleme" grubu olarak duyurur.
- [ ] Veri klasörü ve dosyaları daralmıştır: `stat -f "%Sp %N" ~/Library/Application\ Support/io.github.turgaydogusan.jyotishdesk ~/Library/Application\ Support/io.github.turgaydogusan.jyotishdesk/jyotish-desk.sqlite*` klasör için `drwx------`, dosyalar için `-rw-------` yazar. Danışan açma, not yazma ve JSON dışa aktarma eskisi gibi çalışır.
- [ ] Baskı önizlemesinin kapağı "Jyotish Desk 0.9.0 · Swiss Ephemeris …" basar.
- [ ] Bütün denetim boyunca terminalde (Rust ve Vite çıktısı) danışan adı, not gövdesi, doğum verisi ya da dosya yolu görünmez.

Kapı geçtiyse ilerleme notlarına `Plan 13 Task 9 elle denetim: geçti (<tarih>)` yazılır. Bu adım commit üretmez.

- [ ] **Step 5: İnsan adımlarını sahibine devret**

İlerleme notlarına ve son rapora şu liste yazılır (durum sütunu sahibince doldurulur). Ajan bu adımların hiçbirini başlatmaz, hiçbirinin sonucunu beklemez ve sahibinden sır istemez.

```
Plan 13 otomatik bölümü tamam. Yayın için sahibin yapacakları (docs/release/RELEASING.md):
[ ] İNSAN ADIMI 1  Apple Developer Program üyeliği
[ ] İNSAN ADIMI 2  Developer ID Application sertifikası ve .p12
[ ] İNSAN ADIMI 3  App Store Connect API anahtarı (notarizasyon)
[ ] İNSAN ADIMI 4  Güncelleme anahtar çifti; açık anahtarın tauri.conf.json'a konması ve commit'i
[ ] İNSAN ADIMI 5  GitHub "release" ortamı (etiket kuralı v*) ve sekiz sır
[ ] İNSAN ADIMI 6  Ad ve marka taraması; sonucu RELEASING.md'ye yazma
[ ] İNSAN ADIMI 7  v0.9.0 etiketi; taslağın temiz bir Mac'te denenmesi; yayımlama
[ ] İNSAN ADIMI 8  Güncelleme provası (0.9.0 → 0.9.1) ve bozulmuş imzanın reddi
[ ] İNSAN ADIMI 9  3–5 astrologun daveti; geri bildirimin toplanması
İmzalı yolun ilk gerçek koşusu İNSAN ADIMI 7'dir; orada çıkan bulgular (notarizasyon, Gatekeeper,
.dmg imzası) ayrı düzeltme commit'leriyle bu plana eklenir.
```

---

## Başarı ölçütleri

- Spec M8'in depodaki bölümü tamamdır: etiket evrensel paketi üretir; sırlar varken imzalı, notarize edilmiş `.dmg`, imzalı güncelleme paketi ve `latest.json` taslak yayına yüklenir; sırlar yokken aynı iş imzasız deneme derlemesi yapar ve denetler.
- Uygulama kullanıcı onayı olmadan ağa bağlanmaz; denetim tek ve sabit bir adrese gider; imzası ve imzadaki sürümü doğrulanmayan paket kurulmaz; kurulum sürüme bağlı ayrı bir onaydır. Yetenek dosyası ve CSP değişmemiştir.
- Hakkında penceresi sürümü, AGPL kaynak adresini ve veri atıflarını gösterir (spec §9, §10).
- Veri klasörü 0700, veritabanı dosyaları 0600'dür; zehirlenmiş store kilidi kendiliğinden toparlanır (Plan 10 kapsam dışı 3 ve 4).
- Sürüm 0.9.0'dır ve dört yerde aynıdır; ilk yayın notu, not şablonu, yayın rehberi, beta rehberi ve geri bildirim başlıkları depodadır.
- Depoda sır, anahtar dosyası ya da parola yoktur; insan adımları dokuz madde olarak sahibine devredilmiştir.
- Kapılar: Vitest 60/517, e2e 12, görsel 10, yayın betiği testleri 9; `store` lib 81, `jyotish-desk` lib 44, diğer Rust sayıları tabandaki gibi; `cargo deny` temiz; imzasız paket denetimi beş `ok`; elle denetim kapısı geçmiştir.

## Bu planın kapsamadıkları

- **İnsan işi (yukarıdaki dokuz adım):** üyelik, sertifika, notarizasyon anahtarı, güncelleme anahtarı, GitHub sırları, marka taraması, etiket ve yayımlama, güncelleme provası, beta katılımcılarının seçimi ve geri bildirimin toplanması. İmzalı yolun ve gerçek güncellemenin kanıtı bu adımlardadır; bu plan onları taklitle ve imzasız yolla hazırlar.
- Windows ve Android imzası, paketi ve güncellemesi; Plan 10 kapsam dışı 3'ün Windows/Linux yarısı (dosya izinleri yalnızca unix'te daraltılır).
- Yorum metinleri ve AI yorum paneli.
- Telemetri, çökme raporu, tanı dışa aktarma, uygulama içinden geri bildirim gönderme (D16).
- Kendiliğinden yeniden başlatma, arka planda sessiz güncelleme, indirme ilerleme çubuğu, güncelleme kanalları (kararlı/beta), sürüm atlama ve geri dönme (D1, D7).
- Güncelleme anahtarının döndürülmesi için uygulama içi düzenek (anahtar değişirse yeni `.dmg` dağıtılır; rehberde yazar).
- Mac App Store dağıtımı, sandbox, yetki (entitlements) dosyası (D12).
- SQLCipher ile veritabanı şifreleme (spec §12: v1 sonrası).
- Eylem SHA'larının kendiliğinden yenilenmesi (Dependabot) ve `ci.yml`'deki eylemlerin SHA'ya sabitlenmesi: ayrı bir bakım işi.
- Plan 10 kapsam dışı 2 (JHD), 5 (göç 2), 8 (M6-5), 14, 15 ve 16'nın boyutlandırma yarısı: beta geri bildirimi isterse.
- Uygulama simgesinin ve `.dmg` arka planının tasarımı; web sitesi ve indirme sayfası.

---

## Self-Review

**1. Spec kapsamı.**

| Gereksinim | Görev |
|---|---|
| v1 spec §2: notarize edilmiş DMG, App Store dışı; otomatik güncelleme (kullanıcı onaylı) | Task 2, 3, 6, 7; İNSAN ADIMI 1–5, 7 |
| §8: etiket atılınca imzalı ve notarize edilmiş DMG | Task 7 (`release.yml`), Task 6 (betikler) |
| §9: GeoNames CC BY atfı Hakkında ekranında | Task 3 |
| §10 İmza: Developer ID, hardened runtime, `notarytool` + staple, evrensel ikili | Task 6 (`build.sh`, `verify-bundle.sh signed`), Task 7; D11, D12 |
| §10 En düşük macOS 13 | Task 6 (`verify-bundle.sh`) |
| §10 Güncelleme: Tauri updater, imzalı, GitHub Releases, kullanıcı onayı | Task 2 (komutlar, yapılandırma testi), Task 3 (onay), Task 4 (akış), Task 5 (`latest.json`) |
| §10 İsim kuralları ve marka taraması | C13 taramaları; İNSAN ADIMI 6 |
| §10 AGPL: Hakkında ekranında kaynak kod bağlantısı | Task 3 (D9: seçilebilir adres) |
| §11 M8: imza, notarizasyon, updater; 3–5 astrologla beta | Task 1–8; İNSAN ADIMI 7–9 |
| §12: şifresiz veri kullanıcı belgelerinde yazılır | Task 5 (yayın notu), Task 8 (beta rehberi) |
| Plan 10 kapsam dışı 3 ve 4 | Task 1 |
| Sırlar yalnızca insanda; otomasyon sırsız çalışır | C18; Task 6 Step 2–4, Task 7 Step 3, Task 9 Step 3 ve 5 |
| Son doğrulama ve elle denetim | Task 9 |

Boşluk: imzalı yolun ve gerçek güncellemenin uçtan uca kanıtı bu planın otomatik bölümünde yoktur ve olamaz; İNSAN ADIMI 7 ve 8 olarak adlandırılmış ve devredilmiştir.

**2. Yer tutucu taraması.** "TODO", "TBD", "daha sonra", "uygun hata işleme" yok. Task 9'daki `<tarih>` denetimin yapıldığı günün yazılacağı yerdir; `RELEASING.md`'deki `<sürüm>`, `<kimlik>` ve `AuthKey_XXXXXXXXXX.p8` sahibinin kendi değerleridir ve bilerek değer taşımaz (sır ya da kimliktir). Yayın notu şablonundaki `X.Y.Z` şablonun kendisidir. Her kod adımı tam içerik ya da `70e2324` ağacına karşı üretilmiş diff taşır.

**3. Tip tutarlılığı.**
- `AppInfo { version, updaterConfigured }` ve `UpdateInfo { version, notes, published }` Task 2'de Rust'ta tanımlanır, `bindings.ts`'e üretilir; Task 3 (`AboutDialog`, `startup.ts`), Task 3 testleri ve Task 4 IPC taklidi aynı alan adlarını kullanır.
- `api.appInfo()`, `api.checkForUpdate()`, `api.installUpdate(version)` Task 2'de tanımlanır; Task 3 aynı adlarla çağırır; `api.test.ts` tablosu üçünü de içerir ve dizgi bağımsız değişken listesine yalnızca `installUpdate.version` eklenir.
- Hata kodları (`update_unconfigured`, `update_check_failed`, `update_install_failed`, `update_changed`, `update_busy`) Rust'ta, `i18n.test.ts`'in `BACKEND_CODES` listesinde ve üç dil dosyasında aynıdır; `reason` değerleri (`network`, `signature`, `manifest`, `install`, `other`) `update_reason`, `updateReason.*` ve testlerde aynıdır.
- `UPDATE_STORAGE_KEY = "jyotish-desk.updates.v1"` ve değerleri (`"auto"`, `"off"`) `pref.ts`, testleri, `App.test.tsx` ve e2e'de (onay kutusu üzerinden) aynıdır.
- `UPDATE_ENDPOINT` (Rust) ile `tauri.conf.json`'daki adres bir Rust testiyle eşitlenir; `RELEASE_BASE` (`latest-json.mjs`) aynı deponun `releases/download` yoludur; `SOURCE_URL` (`AboutDialog.tsx`) depo adresidir.
- Dosya adları: `Jyotish-Desk_<sürüm>_universal.{dmg,app.tar.gz,app.tar.gz.sig}` `package.sh`, `latest-json.test.mjs` ve `RELEASING.md`'de aynıdır; Tauri'nin ürettiği `Jyotish Desk.app`, `Jyotish Desk.app.tar.gz(.sig)` ve `Jyotish Desk_<sürüm>_universal.dmg` adları `verify-bundle.sh`, `package.sh` ve `release.yml`'de aynıdır.
- Sekiz sır adı `release.yml`, `workflow.test.mjs` ve `RELEASING.md`'de aynıdır; işin Tauri'ye verdiği ortam değişkeni adları (`APPLE_API_KEY`, `APPLE_API_KEY_PATH`, …) `build.sh`'in denetlediği adlardır (ölçüm: Tauri CLI 2.12.6 ikilisindeki adlar).
- i18n: `about.open` "Hakkında", `about.title` "Jyotish Desk hakkında", `update.check` "Şimdi denetle", `update.auto` "Açılışta güncellemeleri denetle", `update.current` "Kurulu sürüm güncel.", `palette.command.about` "Hakkında ve güncellemeler" — Vitest, e2e, elle denetim kapısı ve belgeler aynı metinleri kullanır.

**4. Kural denetimi.** C1 (eklenti hata metni taşınmaz; açılış denetimi sessiz; betikler sır yazdırmaz), C2′ (komutlar adres ya da yol almaz; yetenek dosyası değişmez), C3 (şema ve `AppPrefs` değişmez), C8 (Hakkında `Dialog` kullanır), C9 (`about.css` yalnızca token), C10 (üretilen dosyalar komutla), C12 (beş yeni kod çevrili), C13 (her commit'te tarama), C15 (`window.open` yok), C18 (sır gerektiren her komut İNSAN ADIMI'dır) görevlerin adımlarında ve Task 9 Step 3'te denetlenir.

**5. Koşulan doğrulama.** Bu plandaki kod, deponun `70e2324` üzerindeki bir kopyasında sekiz commit olarak uygulanmış ve ölçülmüştür: her commit'te Vitest sayıları (500 → 500 → 501 → 517 → …), son durumda `cargo fmt`, `clippy -D warnings`, `cargo test --workspace` (`store` 81, `jyotish-desk` 44), `cargo deny check` (dört `ok`), 1.93 ile `cargo check`, `pnpm build`, e2e 12, görsel 10, yayın betiği testleri 9. İmzasız evrensel derleme yapılmış ve `verify-bundle.sh unsigned` beş `ok` vermiştir. Atılan bir deneme anahtarıyla `tauri signer sign --app-version 0.9.0` çıktısının sürümü bağladığı ve `package.sh`'in beş dosyayı ürettiği görülmüştür. **Koşulmayanlar:** `release.yml` (GitHub'da), imzalama, notarizasyon, `verify-bundle.sh signed`'ın başarılı yolu ve gerçek bir güncellemenin indirilip kurulması; bunlar sır ister ve İNSAN ADIMI 7–8'dedir. Tauri'nin `.dmg`'yi Developer ID ile imzaladığı varsayılmıştır; imzalamıyorsa `verify-bundle.sh signed` bunu `Gatekeeper rejects the dmg` ile gösterir ve düzeltme (işe bir `codesign` adımı) İNSAN ADIMI 7'nin bulgusu olarak eklenir.
