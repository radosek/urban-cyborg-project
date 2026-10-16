#!/bin/sh
# Drives the menus of both builds with the same key script and compares the screen
# every time the game reads a key.   tools/menutest.sh [keys.txt]
ROOT=$(cd "$(dirname "$0")/.." && pwd)
KEYS=$(cd "$(dirname "${1:-$ROOT/tools/menus/tour.txt}")" && pwd)/$(basename "${1:-tour.txt}")
OUT=$ROOT/build/menutest; rm -rf "$OUT"; mkdir -p "$OUT/cpp_home" "$OUT/rust_home"
export SDL_VIDEODRIVER=offscreen SDL_AUDIODRIVER=dummy URBAN_HEADLESS=1 URBAN_DATA="$ROOT/ref/data"
export URBAN_MENU_KEYS="$KEYS" URBAN_SEED=1
run() { URBAN_USER_DIR="$OUT/${1}_home" URBAN_TRACE_OUT="$OUT/$1.trace" "$2" -i -s -x >/dev/null 2>"$OUT/$1.log" || true; }
run cpp "$ROOT/build/urban"
run rust "$ROOT/rust/target/release/urban"
FIRST=$(paste -d' ' "$OUT/cpp.trace" "$OUT/rust.trace" | awk '$2 != $4 { print $1; exit }')
NC=$(wc -l < "$OUT/cpp.trace"); NR=$(wc -l < "$OUT/rust.trace")
if [ -z "$FIRST" ] && [ "$NC" -eq "$NR" ]; then echo "IDENTICAL: $NC key reads"; exit 0; fi
[ -z "$FIRST" ] && FIRST=$(( NC < NR ? NC : NR ))
echo "DIVERGED at key read $FIRST (cpp $NC reads, rust $NR reads)"
for b in cpp rust; do
	bin="$ROOT/build/urban"; [ $b = rust ] && bin="$ROOT/rust/target/release/urban"
	rm -rf "$OUT/${b}_home"; mkdir -p "$OUT/${b}_home"
	URBAN_USER_DIR="$OUT/${b}_home" URBAN_DUMP=$FIRST URBAN_DUMP_DIR="$OUT" "$bin" -i -s -x >/dev/null 2>&1 || true
done
exit 1
