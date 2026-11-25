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
BUNDLE="$ROOT/target/$TARGET/release/bundle"
CONF="$ROOT/app/src-tauri/tauri.conf.json"

# Önceki derlemenin paketleri silinir: bu derleme bir dosyayı üretmezse (ör. güncelleme paketi),
# verify-bundle.sh ve package.sh eski bir .dmg, .app.tar.gz ya da .sig dosyasını kabul edemesin.
clean_bundle() {
  rm -rf "$BUNDLE"
}

case "$MODE" in
  unsigned)
    clean_bundle
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
    # Yol yazdırılmaz; yalnızca dosyanın var olduğuna bakılır (yoksa derleme ancak notarizasyon adımında, epey sonra durur).
    if [ ! -f "$APPLE_API_KEY_PATH" ]; then
      echo "build.sh: APPLE_API_KEY_PATH does not point to a file (it is the path of the .p8 key, not its contents)" >&2
      exit 2
    fi
    if ! node -e '
      const conf = JSON.parse(require("node:fs").readFileSync(process.argv[1], "utf8"));
      const key = conf.plugins?.updater?.pubkey;
      process.exit(typeof key === "string" && /^[A-Za-z0-9+\/=]{100,}$/.test(key) ? 0 : 1);
    ' "$CONF"; then
      echo "build.sh: plugins.updater.pubkey is empty; a signed build without it could never be updated" >&2
      exit 2
    fi
    clean_bundle
    # Güncelleme paketi (.app.tar.gz + .sig) yalnızca imzalı derlemede üretilir.
    pnpm --dir "$ROOT/app" tauri build --ci --target "$TARGET" --bundles app,dmg \
      --config '{"bundle":{"createUpdaterArtifacts":true}}'
    ;;
  *)
    echo "usage: build.sh unsigned|signed" >&2
    exit 2
    ;;
esac

echo "bundle: $BUNDLE"
