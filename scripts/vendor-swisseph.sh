#!/usr/bin/env bash
# Swiss Ephemeris kaynağını ve veri dosyalarını sabit bir sürümden getirir.
set -euo pipefail
SWE_TAG="v2.10.03"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$(mktemp -d)/swisseph"
DEST="$ROOT/crates/ephemeris/vendor/swisseph"
EPHE="$ROOT/data/ephe"

git clone --depth 1 --branch "$SWE_TAG" https://github.com/aloistr/swisseph.git "$SRC"

mkdir -p "$DEST" "$EPHE"
cp "$SRC"/*.c "$SRC"/*.h "$DEST"/
# Kütüphanede kullanılmayan örnek ve araç programları; swetest.c fixture üretimi için kalır.
rm -f "$DEST"/swemini.c "$DEST"/swevents.c "$DEST"/swephgen4.c "$DEST"/sweasp.c "$DEST"/swemptab.c.orig
cp "$SRC"/LICENSE "$DEST"/LICENSE 2>/dev/null || true

# Veri dosyaları v2.10.03 tag'inde yok (astro.com/ftp/.../ephe da 404); resmi repo master'ından alınır.
for f in sepl_18.se1 semo_18.se1 sefstars.txt; do
  curl -fsSL -o "$EPHE/$f" "https://raw.githubusercontent.com/aloistr/swisseph/master/ephe/$f"
done
echo "$SWE_TAG" > "$DEST"/VERSION
ls -la "$DEST" "$EPHE"
