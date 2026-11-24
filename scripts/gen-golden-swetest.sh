#!/usr/bin/env bash
# Vendored swetest ile golden harita fixture'ları üretir (kaynak: swetest).
# Lahiri sidereal, mean node, Whole Sign; lagna ev sisteminden bağımsızdır.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
V="$ROOT/crates/ephemeris/vendor/swisseph"
OUT_DIR="$ROOT/tests/golden"
BIN="$ROOT/target/swetest"
EPHE="$ROOT/data/ephe"
mkdir -p "$ROOT/target" "$OUT_DIR"

cc -O2 -w -o "$BIN" "$V/swetest.c" \
  "$V/swedate.c" "$V/swehouse.c" "$V/swejpl.c" "$V/swemmoon.c" "$V/swemplan.c" \
  "$V/sweph.c" "$V/swephlib.c" "$V/swecl.c" "$V/swehel.c" -lm
SE_VERSION="$(tr -d 'v\n' < "$V/VERSION")"

# id | gün.ay.yıl | UT saat:dakika | enlem | boylam | not
CHARTS=(
  "london-1850-01-01|1.1.1850|6:30|51.5074|-0.1278|19. yüzyıl, kuzey yarıküre, batı boylamı"
  "buenos-aires-1923-06-15|15.6.1923|0:00|-34.6037|-58.3816|güney yarıküre, UT gece yarısı"
  "istanbul-1987-03-17|17.3.1987|23:59|41.0082|28.9784|Türkiye, gün sonu"
  "new-delhi-2000-01-01|1.1.2000|12:00|28.6139|77.2090|J2000"
  "tromso-2050-12-21|21.12.2050|18:15|69.6492|18.9553|yüksek enlem (kutup dairesi içi)"
)

se() { "$BIN" "$@" -head -eswe -edir"$EPHE" -sid1; }
check() { if echo "$1" | grep -qi -e warning -e error; then echo "swetest problem: $1" >&2; exit 1; fi; }

for row in "${CHARTS[@]}"; do
  IFS='|' read -r id d t lat lon note <<< "$row"
  # Gregoryen tarih → JD (UT), Meeus algoritması; swe_julday ile aynı sonuç.
  jd="$(echo "$d $t" | awk '{
    split($1, a, "."); split($2, b, ":");
    D = a[1]; M = a[2]; Y = a[3]; h = b[1] + b[2] / 60.0;
    if (M <= 2) { Y -= 1; M += 12 }
    A = int(Y / 100); B = 2 - A + int(A / 4);
    printf "%.10f", int(365.25 * (Y + 4716)) + int(30.6001 * (M + 1)) + D + B - 1524.5 + h / 24.0 }')"
  ayan_raw="$(se -b"$d" -ut"$t" -pb -fPl)"; check "$ayan_raw"
  ayan="$(echo "$ayan_raw" | awk '{print $NF}')"
  # Whole Sign ile çağrılır: kutup dairesinde de hata vermez.
  asc_raw="$(se -b"$d" -ut"$t" -p -house"$lon","$lat",W -fPl)"; check "$asc_raw"
  asc="$(echo "$asc_raw" | awk '/^Ascendant/ {print $NF}')"
  pl_raw="$(se -b"$d" -ut"$t" -p0123456m -fPl)"; check "$pl_raw"
  # swetest satırları sırayla değil adıyla eşlenir: "mean Node" → Rahu.
  grahas="$(echo "$pl_raw" | awk '
    { lon = $NF; $NF = ""; sub(/[ \t]+$/, ""); name = $0;
      if (name == "mean Node") name = "Rahu";
      v[name] = lon }
    END {
      split("Sun Moon Mars Mercury Jupiter Venus Saturn Rahu", n, " ");
      for (i = 1; i <= 8; i++) {
        if (!(n[i] in v)) { print "missing " n[i] > "/dev/stderr"; exit 1 }
        printf "      \"%s\": %s,\n", n[i], v[n[i]] }
      k = v["Rahu"] + 180.0; if (k >= 360.0) k -= 360.0;
      printf "      \"Ketu\": %.7f\n", k }')"
  cat > "$OUT_DIR/$id.json" <<JSON
{
  "id": "$id",
  "source": "swetest",
  "source_version": "$SE_VERSION",
  "notes": "$note; swetest -sid1 (Lahiri), mean node, UT $d $t",
  "settings": {
    "schema_version": 1,
    "zodiac": "Sidereal",
    "ayanamsa": "Lahiri",
    "node": "Mean",
    "house_system": "WholeSign",
    "dasha_year": 365.25,
    "combustion_orbs": {
      "moon": 12.0,
      "mars": 17.0,
      "mercury": 14.0,
      "mercury_retro": 12.0,
      "jupiter": 11.0,
      "venus": 10.0,
      "venus_retro": 8.0,
      "saturn": 15.0
    },
    "sunrise_def": "DiskCenterNoRefraction"
  },
  "input": { "jd_ut": $jd, "lat": $lat, "lon": $lon },
  "expected": {
    "ayanamsa_deg": $ayan,
    "lagna_lon": $asc,
    "grahas": {
$grahas
    }
  }
}
JSON
  echo "wrote $OUT_DIR/$id.json"
done
