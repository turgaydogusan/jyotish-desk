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
