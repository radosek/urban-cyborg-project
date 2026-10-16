#!/bin/sh
# Runs the C++ reference build and the Rust port on the same scripted input and random seed,
# then reports the first frame where the rendered output differs (and dumps both frames).
#   tools/difftest.sh <map> <replay.txt> [frames] [seed]
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
MAP=${1:-gamemaps/level11.map}
SCRIPT=$(cd "$(dirname "${2:-$ROOT/tools/replays/level11_walk.txt}")" && pwd)/$(basename "${2:-level11_walk.txt}")
FRAMES=${3:-1500}
SEED=${4:-12345}
OUT=${DIFF_OUT:-$ROOT/build/difftest}
mkdir -p "$OUT"
# Headless: no window or sound device during tests.
export SDL_VIDEODRIVER=offscreen SDL_AUDIODRIVER=dummy URBAN_HEADLESS=1
export URBAN_REPLAY="$SCRIPT" URBAN_SEED=$SEED URBAN_FRAMES=$FRAMES URBAN_DATA="$ROOT/ref/data"

run() { # name binary
	URBAN_TRACE_OUT="$OUT/$1.trace" "$2" -x -s -f "$MAP" >/dev/null 2>"$OUT/$1.log" || true
}
run cpp "$ROOT/build/urban"
run rust "$ROOT/rust/target/release/urban"

FIRST=$(paste -d' ' "$OUT/cpp.trace" "$OUT/rust.trace" | awk '$2 != $5 { print $1; exit }')
NC=$(wc -l < "$OUT/cpp.trace"); NR=$(wc -l < "$OUT/rust.trace")
if [ -z "$FIRST" ] && [ "$NC" -eq "$NR" ]; then
	echo "IDENTICAL: $NC frames"
	exit 0
fi
[ -z "$FIRST" ] && FIRST=$(( NC < NR ? NC : NR ))
echo "DIVERGED at frame $FIRST (cpp $NC frames, rust $NR frames)"
for b in cpp rust; do
	bin="$ROOT/build/urban"; [ $b = rust ] && bin="$ROOT/rust/target/release/urban"
	URBAN_DUMP=$FIRST URBAN_DUMP_DIR="$OUT" URBAN_FRAMES=$((FIRST + 1)) "$bin" -x -s -f "$MAP" >/dev/null 2>&1 || true
done
ls "$OUT"/*_$(printf %06d "$FIRST").bmp 2>/dev/null
exit 1
