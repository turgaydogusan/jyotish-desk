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

# --- Plan 07: panchang ve SAV (Rust motorundan bağımsız; swetest + awk) ---
# Lahiri sidereal Güneş ve Ay boylamı, tek satır: "güneş ay".
sun_moon() {
  "$BIN" -bj"$1" -ut -p01 -fl -head -eswe -edir"$EPHE" -sid1 |
    awk '{ v[NR] = $1 } END { printf "%s %s", v[1], v[2] }'
}
# Panchang açısı, [0, 360): tithi/karana Ay − Güneş, nakshatra Ay, yoga Güneş + Ay.
angle() {
  sun_moon "$1" | awk -v q="$2" '{
    if (q == "elong") a = $2 - $1; else if (q == "moon") a = $2; else a = $1 + $2;
    a -= 360 * int(a / 360); if (a < 0) a += 360; printf "%.9f", a }'
}
# Açının `jd`'den sonra `target` dereceye ulaştığı an: [jd, jd + 1.5] aralığında 24 adım
# ikiye bölme (≈ 0.008 sn). Hedefe işaretli uzaklık [-180, 180) aralığına sarılır.
ends_at() {
  local jd="$1" q="$2" target="$3" lo hi mid up
  lo="$jd"; hi="$(awk -v j="$jd" 'BEGIN { printf "%.9f", j + 1.5 }')"
  for _ in $(seq 1 24); do
    mid="$(awk -v a="$lo" -v b="$hi" 'BEGIN { printf "%.9f", (a + b) / 2 }')"
    up="$(awk -v a="$(angle "$mid" "$q")" -v t="$target" 'BEGIN {
      d = a - t + 180; d -= 360 * int(d / 360); if (d < 0) d += 360; print (d - 180 >= 0) }')"
    if [ "$up" = 1 ]; then hi="$mid"; else lo="$mid"; fi
  done
  awk -v a="$lo" -v b="$hi" 'BEGIN { printf "%.6f", (a + b) / 2 }'
}
# `swetest -rise -hindu` satırlarından "rise JD" / "set JD" (UT; Meeus tarih → JD).
rise_set_jds() {
  "$BIN" -rise -hindu -bj"$1" -ut -p0 -geopos"$2","$3",0 -n4 -head -eswe -edir"$EPHE" | awk '
    function jd(d, t,   a, b, D, M, Y, h, A, B) {
      split(d, a, "."); split(t, b, ":"); D = a[1]; M = a[2]; Y = a[3];
      h = b[1] + b[2] / 60.0 + b[3] / 3600.0;
      if (M <= 2) { Y -= 1; M += 12 }
      A = int(Y / 100); B = 2 - A + int(A / 4);
      return int(365.25 * (Y + 4716)) + int(30.6001 * (M + 1)) + D + B - 1524.5 + h / 24.0 }
    $1 == "rise" {
      if ($2 != "-") printf "rise %.6f\n", jd($2, $3);
      for (i = 3; i < NF; i++) if ($i == "set" && $(i + 1) != "-") printf "set %.6f\n", jd($(i + 1), $(i + 2)) }'
}
# Doğum anının panchangı (Plan 07 tanımları): gün doğumu doğumdan önceki 1.25 gündeki son
# doğuş, batış ondan sonraki ilk batış (1 gün içinde). Vara gün doğumunun yerel ortalama
# tarihinden; gün doğumu yoksa 06:00 yerel ortalama saatten.
panchang_json() {
  local jd="$1" lat="$2" lon="$3" ev rise set vara sm t n y k
  ev="$(rise_set_jds "$(awk -v j="$jd" 'BEGIN { printf "%.6f", j - 1.25 }')" "$lon" "$lat")"
  rise="$(echo "$ev" | awk -v j="$jd" '$1 == "rise" && $2 <= j { r = $2 } END { print (r == "" ? "null" : r) }')"
  set="null"
  if [ "$rise" != null ]; then
    set="$(echo "$ev" | awk -v r="$rise" '$1 == "set" && $2 > r && $2 - r < 1 { print $2; exit }')"
    [ -z "$set" ] && set=null
  fi
  vara="$(awk -v r="$rise" -v j="$jd" -v l="$lon" 'BEGIN {
    split("Sunday Monday Tuesday Wednesday Thursday Friday Saturday", W, " ");
    x = (r == "null") ? j - 0.25 : r; x += 0.5 + l / 360;
    print W[(int(x) + 1) % 7 + 1] }')"
  sm="$(sun_moon "$jd")"
  read -r t n y k <<< "$(echo "$sm" | awk '{
    e = $2 - $1; e -= 360 * int(e / 360); if (e < 0) e += 360;
    s = $1 + $2; s -= 360 * int(s / 360);
    printf "%d %d %d %d", int(e / 12) + 1, int($2 * 3 / 40) + 1, int(s * 3 / 40) + 1, int(e / 6) + 1 }')"
  printf '{\n'
  printf '      "sunrise_jd": %s,\n      "sunset_jd": %s,\n      "vara": "%s",\n' "$rise" "$set" "$vara"
  printf '      "tithi": %s,\n      "tithi_ends_jd": %s,\n' "$t" "$(ends_at "$jd" elong "$((t * 12))")"
  printf '      "nakshatra": %s,\n      "nakshatra_ends_jd": %s,\n' "$n" \
    "$(ends_at "$jd" moon "$(awk -v n="$n" 'BEGIN { printf "%.12f", n * 40 / 3 }')")"
  printf '      "yoga": %s,\n      "yoga_ends_jd": %s,\n' "$y" \
    "$(ends_at "$jd" sum "$(awk -v n="$y" 'BEGIN { printf "%.12f", n * 40 / 3 }')")"
  printf '      "karana": %s,\n      "karana_ends_jd": %s\n    }' "$k" "$(ends_at "$jd" elong "$((k * 6))")"
}
# Sarvashtakavarga, Koç'tan 12 burç. Girdi: "Ad boylam" satırları (Lagna ve grahalar).
# Tablolar BPHS böl. 66; Rust tablosundan (crates/jyotish-core/src/ashtakavarga.rs) ayrı yazılmıştır.
sav_json() {
  awk '
    BEGIN {
      split("Sun Moon Mars Mercury Jupiter Venus Saturn Lagna", R, " ");
      T["Sun"]     = "1,2,4,7,8,9,10,11|3,6,10,11|1,2,4,7,8,9,10,11|3,5,6,9,10,11,12|5,6,9,11|6,7,12|1,2,4,7,8,9,10,11|3,4,6,10,11,12";
      T["Moon"]    = "3,6,7,8,10,11|1,3,6,7,10,11|2,3,5,6,9,10,11|1,3,4,5,7,8,10,11|1,4,7,8,10,11,12|3,4,5,7,9,10,11|3,5,6,11|3,6,10,11";
      T["Mars"]    = "3,5,6,10,11|3,6,11|1,2,4,7,8,10,11|3,5,6,11|6,10,11,12|6,8,11,12|1,4,7,8,9,10,11|1,3,6,10,11";
      T["Mercury"] = "5,6,9,11,12|2,4,6,8,10,11|1,2,4,7,8,9,10,11|1,3,5,6,9,10,11,12|6,8,11,12|1,2,3,4,5,8,9,11|1,2,4,7,8,9,10,11|1,2,4,6,8,10,11";
      T["Jupiter"] = "1,2,3,4,7,8,9,10,11|2,5,7,9,11|1,2,4,7,8,10,11|1,2,4,5,6,9,10,11|1,2,3,4,7,8,10,11|2,5,6,9,10,11|3,5,6,12|1,2,4,5,6,7,9,10,11";
      T["Venus"]   = "8,11,12|1,2,3,4,5,8,9,11,12|3,5,6,9,11,12|3,5,6,9,11|5,8,9,10,11|1,2,3,4,5,8,9,10,11|3,4,5,8,9,10,11|1,2,3,4,5,8,9,11";
      T["Saturn"]  = "1,2,4,7,8,10,11|3,6,11|3,5,6,10,11,12|6,8,9,10,11,12|5,6,11,12|6,11,12|3,5,6,11|1,3,4,6,10,11";
    }
    { sign[$1] = int($2 / 30) % 12 }
    END {
      for (g in T) {
        n = split(T[g], refs, "|");
        for (r = 1; r <= n; r++) {
          m = split(refs[r], places, ",");
          for (p = 1; p <= m; p++) sav[(sign[R[r]] + places[p] - 1) % 12]++;
        }
      }
      for (s = 0; s < 12; s++) printf "%s%d", (s ? ", " : "[ "), sav[s];
      printf " ]" }'
}

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
  # Plan 06 türetilmiş beklentileri. Rust motorundan bağımsız awk uygulaması; girdi
  # yukarıdaki swetest boylamlarıdır.
  points="$(printf 'Lagna %s\n' "$asc"; echo "$grahas" | tr -d '":,' | awk '{print $1, $2}')"
  # D9: BPHS navamsa kuralının sürekli biçimi, Koç'tan floor(boylam / 3°20') mod 12.
  d9="$(echo "$points" | awk '
    BEGIN { split("Aries Taurus Gemini Cancer Leo Virgo Libra Scorpio Sagittarius Capricorn Aquarius Pisces", S, " ") }
    { line[NR] = sprintf("      \"%s\": \"%s\"", $1, S[int($2 * 3 / 10) % 12 + 1]) }
    END { for (i = 1; i <= NR; i++) printf "%s%s\n", line[i], (i < NR ? "," : "") }')"
  # İlk mahadasha: Ay'ın nakshatrası (13°20') lordu verir; başlangıç = jd − kesir × yıl × 365.25.
  dasha="$(echo "$points" | awk -v jd="$jd" '$1 == "Moon" {
    split("Ketu Venus Sun Moon Mars Rahu Jupiter Saturn Mercury", L, " ");
    split("7 20 6 10 7 18 16 19 17", Y, " ");
    x = $2 * 3 / 40; n = int(x); f = x - n; i = n % 9 + 1;
    printf "{ \"first_lord\": \"%s\", \"first_start_jd\": %.6f }", L[i], jd - f * Y[i] * 365.25 }')"
  pan="$(panchang_json "$jd" "$lat" "$lon")"
  sav="$(echo "$points" | sav_json)"
  cat > "$OUT_DIR/$id.json" <<JSON
{
  "id": "$id",
  "source": "swetest",
  "source_version": "$SE_VERSION",
  "notes": "$note; swetest -sid1 (Lahiri), mean node, UT $d $t; d9_signs ve dasha awk ile türetildi (Plan 06); panchang swetest -rise -hindu ve ikiye bölmeyle, sav awk ile türetildi (Plan 07)",
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
    },
    "d9_signs": {
$d9
    },
    "dasha": $dasha,
    "panchang": $pan,
    "sav": $sav
  }
}
JSON
  echo "wrote $OUT_DIR/$id.json"
done
