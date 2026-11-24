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
# Etiketteki, kütüphanede kullanılmayan bağımsız örnek/araç programları (build.rs derlemez); swetest.c fixture üretimi için kalır.
rm -f "$DEST"/swemini.c "$DEST"/swevents.c "$DEST"/swephgen4.c "$DEST"/sweasp.c
cp "$SRC"/LICENSE "$DEST"/LICENSE

# Veri dosyaları v2.10.03 tag'inde yok (astro.com/ftp/.../ephe da 404); resmi repodan sabit commit SHA ile alınır.
EPHE_SHA="e308765d10b9463ab0ed53b102ad454820ee0b43"
EPHE_FILES=(sepl_18.se1 semo_18.se1 sefstars.txt)
EPHE_HASHES=(
  9f057b316b1890a7f9340d0bf92db96edbab4939dbcc54762b62f6a70a059eb0
  1ca07bd67c24374d77226180c20a4f9996cba013697894810518e7eb582ca4f7
  18b0dcafbe5b7240773daba2c038a325f5b3fc4163f61e0a7f4e92abd4f517c6
)
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
for i in "${!EPHE_FILES[@]}"; do
  f="${EPHE_FILES[$i]}"
  curl -fsSL -o "$TMP/$f" "https://raw.githubusercontent.com/aloistr/swisseph/$EPHE_SHA/ephe/$f"
  actual="$(shasum -a 256 "$TMP/$f" | awk '{print $1}')"
  if [ "$actual" != "${EPHE_HASHES[$i]}" ]; then
    echo "HATA: $f sha256 uyuşmuyor (beklenen ${EPHE_HASHES[$i]}, gelen $actual)" >&2
    exit 1
  fi
done
for f in "${EPHE_FILES[@]}"; do
  mv "$TMP/$f" "$EPHE/$f"
done
echo "$SWE_TAG" > "$DEST"/VERSION
ls -la "$DEST" "$EPHE"
