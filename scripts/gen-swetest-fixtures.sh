#!/usr/bin/env bash
# Vendored kaynaktan swetest derler ve ephemeris golden fixture'ını üretir.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
V="$ROOT/crates/ephemeris/vendor/swisseph"
OUT="$ROOT/crates/ephemeris/tests/fixtures/swetest.csv"
BIN="$ROOT/target/swetest"
mkdir -p "$ROOT/target" "$(dirname "$OUT")"

cc -O2 -w -o "$BIN" "$V/swetest.c" \
  "$V/swedate.c" "$V/swehouse.c" "$V/swejpl.c" "$V/swemmoon.c" "$V/swemplan.c" \
  "$V/sweph.c" "$V/swephlib.c" "$V/swecl.c" "$V/swehel.c" -lm

DATES=(
  "1.1.1850 6:30"
  "15.6.1923 0:00"
  "29.10.1923 12:00"
  "17.3.1987 23:59"
  "1.1.2000 12:00"
  "21.12.2050 18:15"
)

echo "date,time,body,mode,lon" > "$OUT"
for dt in "${DATES[@]}"; do
  d="${dt% *}"; t="${dt#* }"
  for mode in trop lahiri; do
    sid=""; [ "$mode" = lahiri ] && sid="-sid1"
    raw="$("$BIN" -b"$d" -ut"$t" -p0123456mt -fPl -head -eswe -edir"$ROOT/data/ephe" $sid)"
    if echo "$raw" | grep -qi -e warning -e error; then
      echo "swetest reported a problem for $dt $mode:" >&2; echo "$raw" >&2; exit 1
    fi
    echo "$raw" | while IFS= read -r line; do
      [ -z "$line" ] && continue
      lon="$(echo "$line" | awk '{print $NF}')"
      name="$(echo "$line" | awk '{$NF=""; sub(/[ \t]+$/, ""); print}')"
      echo "$d,$t,$name,$mode,$lon" >> "$OUT"
    done
  done
done
wc -l "$OUT"
