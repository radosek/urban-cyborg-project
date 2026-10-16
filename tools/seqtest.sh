#!/bin/sh
# Plays the intro (no -i) in both builds, passing only the warning screen, and compares the
# screen at every rest() until the main menu.   tools/seqtest.sh
ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT=$ROOT/build/seqtest; rm -rf "$OUT"; mkdir -p "$OUT/cpp_home" "$OUT/rust_home"
export SDL_VIDEODRIVER=offscreen SDL_AUDIODRIVER=dummy URBAN_HEADLESS=1 URBAN_DATA="$ROOT/ref/data" URBAN_SEED=1
export URBAN_KEYPRESSES=${KEYPRESSES:-1}
[ -n "$RUN" ] && export URBAN_RUN=$RUN
for b in cpp rust; do
	bin="$ROOT/build/urban"; [ $b = rust ] && bin="$ROOT/rust/target/release/urban"
	URBAN_USER_DIR="$OUT/${b}_home" URBAN_SEQ_TRACE="$OUT/$b.trace" "$bin" -s -x >/dev/null 2>"$OUT/$b.log" || true
done
wc -l "$OUT/cpp.trace" "$OUT/rust.trace"
paste -d' ' "$OUT/cpp.trace" "$OUT/rust.trace" | awk '$2 != $4 { print "DIVERGED at rest()", $1; f=1; exit } END { if (!f) print "IDENTICAL" }'
