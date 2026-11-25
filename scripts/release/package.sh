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
