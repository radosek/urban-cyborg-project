#!/bin/sh
# Plays every animation in intro.dat in both builds (looped, N frames) and compares each frame.
#   tools/flitest.sh [frames]
ROOT=$(cd "$(dirname "$0")/.." && pwd)
N=${1:-200}
OUT=$ROOT/build/flitest; mkdir -p "$OUT"
export SDL_VIDEODRIVER=offscreen SDL_AUDIODRIVER=dummy URBAN_HEADLESS=1 URBAN_DATA="$ROOT/ref/data" URBAN_FRAMES=$N
pass=0; fail=0
for i in 2 3 4 5 6 7 8 9 10 11 12; do
	f="gfx/intro/$i.flc"
	for b in cpp rust; do
		bin="$ROOT/build/urban"; [ $b = rust ] && bin="$ROOT/rust/target/release/urban"
		URBAN_FLI_TEST=$f URBAN_TRACE_OUT="$OUT/${b}_$i.trace" "$bin" -s -x >/dev/null 2>"$OUT/${b}_$i.log" || true
	done
	first=$(paste -d' ' "$OUT/cpp_$i.trace" "$OUT/rust_$i.trace" | awk '$2 != $4 { print $1; exit }')
	nc=$(wc -l < "$OUT/cpp_$i.trace"); nr=$(wc -l < "$OUT/rust_$i.trace")
	if [ -z "$first" ] && [ "$nc" -eq "$nr" ] && [ "$nc" -gt 0 ]; then echo "$f: IDENTICAL ($nc frames)"; pass=$((pass+1))
	else echo "$f: DIVERGED at frame ${first:-?} (cpp $nc, rust $nr)"; fail=$((fail+1)); fi
done
echo "pass=$pass fail=$fail"
