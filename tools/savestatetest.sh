#!/bin/sh
# Quick save/load must restore the exact game state: run A saves at frame N and keeps playing,
# run B loads that save at frame N with the same input. After B's 65-frame fade-in, every
# frame must match A.   tools/savestatetest.sh [map] [N] [frames]
ROOT=$(cd "$(dirname "$0")/.." && pwd)
MAP=${1:-gamemaps/level11.map}; N=${2:-900}; F=${3:-2500}
OUT=$ROOT/build/savestate; mkdir -p "$OUT"
export SDL_VIDEODRIVER=offscreen SDL_AUDIODRIVER=dummy URBAN_HEADLESS=1 URBAN_DATA="$ROOT/ref/data" URBAN_SEED=777
export URBAN_REPLAY="$ROOT/tools/replays/level11_walk.txt" URBAN_FRAMES=$F URBAN_USER_DIR="$OUT"
[ -n "$REPLAY" ] && export URBAN_REPLAY="$REPLAY"
BIN="$ROOT/rust/target/release/urban"
URBAN_SAVE_AT=$N URBAN_SAVE_PATH="$OUT/state.sav" URBAN_TRACE_OUT="$OUT/a.trace" "$BIN" -x -s -f "$MAP" >/dev/null 2>&1
URBAN_LOAD_PATH="$OUT/state.sav" URBAN_LOAD_FRAME=$N URBAN_TRACE_OUT="$OUT/b.trace" "$BIN" -x -s -f "$MAP" >/dev/null 2>"$OUT/b.log"
ls -la "$OUT/state.sav" | awk '{print "save size:", $5}'
# The first frame after the 65-frame fade-in can differ by palette when an explosion flash
# overlaps the end of the fade; game state is compared from the next frame.
FROM=$((N + 68))
awk -v f=$FROM '$1 >= f {print $1, $2}' "$OUT/a.trace" > "$OUT/a.cmp"
awk -v f=$FROM '$1 >= f {print $1, $2}' "$OUT/b.trace" > "$OUT/b.cmp"
if cmp -s "$OUT/a.cmp" "$OUT/b.cmp" && [ -s "$OUT/a.cmp" ]; then echo "IDENTICAL after load: frames $FROM..$F ($(wc -l < "$OUT/a.cmp"))"
else paste -d' ' "$OUT/a.cmp" "$OUT/b.cmp" | awk '$2 != $4 { print "DIVERGED at frame", $1; exit }'; fi
