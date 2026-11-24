#!/usr/bin/env bash
# GeoNames dökümlerini indirir ve data/atlas/atlas.sqlite dosyasını yeniden üretir.
# GeoNames dökümleri günlük güncellenir. Tekrarlanabilirlik için indirme tarihi ve
# dosyaların sha256 özetleri data/atlas/SOURCE.md'ye yazılır.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
BASE="https://download.geonames.org/export/dump"
FILES=(cities1000.zip admin1CodesASCII.txt countryInfo.txt)

for f in "${FILES[@]}"; do
  curl -fsSL --retry 3 -o "$TMP/$f" "$BASE/$f"
done
unzip -q "$TMP/cities1000.zip" -d "$TMP"

sha() {
  if command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | cut -d' ' -f1
  else sha256sum "$1" | cut -d' ' -f1; fi
}

DATE="$(date -u +%Y-%m-%d)"
mkdir -p "$ROOT/data/atlas"
cargo run -q --release -p atlas --bin build-atlas -- \
  "$TMP/cities1000.txt" "$TMP/admin1CodesASCII.txt" "$TMP/countryInfo.txt" \
  "$DATE" "$ROOT/data/atlas/atlas.sqlite"

{
  echo "# Atlas kaynağı"
  echo
  echo "- Kaynak: GeoNames ($BASE), lisans CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/)"
  echo "- İndirme tarihi (UTC): $DATE"
  echo "- Üretim: \`scripts/fetch-geonames.sh\`"
  echo
  echo "| Dosya | sha256 |"
  echo "|---|---|"
  for f in "${FILES[@]}"; do echo "| $f | $(sha "$TMP/$f") |"; done
} > "$ROOT/data/atlas/SOURCE.md"

ls -lh "$ROOT/data/atlas/atlas.sqlite"
