#!/bin/sh
# Start game, clear level 1 via cheat, save as "AB" in slot 1, ESC, load slot 1, ESC, quit.
# Compares screens at every key read and every gameplay frame.
ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT=$ROOT/build/savetest; rm -rf "$OUT"; mkdir -p "$OUT/cpp_home" "$OUT/rust_home"
export SDL_VIDEODRIVER=offscreen SDL_AUDIODRIVER=dummy URBAN_HEADLESS=1 URBAN_DATA="$ROOT/ref/data" URBAN_SEED=7
export URBAN_MENU_KEYS="$ROOT/tools/menus/saveload.txt" URBAN_REPLAY="$ROOT/tools/replays/saveload_play.txt" URBAN_FRAMES=1600
for b in cpp rust; do
	bin="$ROOT/build/urban"; [ $b = rust ] && bin="$ROOT/rust/target/release/urban"
	URBAN_USER_DIR="$OUT/${b}_home" URBAN_TRACE_OUT="$OUT/$b.trace" URBAN_MENU_TRACE="$OUT/$b.menu" "$bin" -i -s -x >/dev/null 2>"$OUT/$b.log" || true
done
wc -l "$OUT/cpp.trace" "$OUT/rust.trace"
paste -d' ' "$OUT/cpp.trace" "$OUT/rust.trace" | awk '$2 != $5 { print "DIVERGED at", $1; f=1; exit } END { if (!f) print "IDENTICAL" }'
wc -l "$OUT/cpp.menu" "$OUT/rust.menu"
paste -d' ' "$OUT/cpp.menu" "$OUT/rust.menu" | awk '$2 != $4 { print "MENU DIVERGED at key read", $1; f=1; exit } END { if (!f) print "MENU IDENTICAL" }'
ls "$OUT"/*_home
