#!/bin/sh
# Baskı önizlemesini A4 ve Letter kâğıda PDF olarak yazdırır ve sonucu denetler (Plan 12 D2).
# Yalnızca macOS; Xcode komut satırı araçları (swiftc) gerekir. Kullanım: scripts/print-probe.sh
set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
port=1423
server=
trap '[ -z "$server" ] || kill "$server" 2>/dev/null || true; rm -rf "$work"' EXIT

xcrun swiftc -O "$root/scripts/print-probe/probe.swift" -o "$work/probe"
(cd "$root/app" && exec pnpm exec vite --port "$port" --strictPort >"$work/vite.log" 2>&1) &
server=$!
tries=0
until curl -sf "http://localhost:$port/" >/dev/null; do
  tries=$((tries + 1))
  [ "$tries" -lt 100 ] || { echo "print-probe: dev server did not start" >&2; exit 1; }
  sleep 0.2
done
for paper in a4 letter; do
  for time in exact unknown; do
    "$work/probe" "http://localhost:$port/" "$root/app/ui/test/fixtures/delhi.json" "$work/$paper-$time.pdf" "$paper" "$time"
  done
done
