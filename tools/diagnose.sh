#!/bin/sh
# Explains a divergence: compares RNG call sites and object states of both builds.
#   tools/diagnose.sh <map> <replay> <seed> <frame>
ROOT=$(cd "$(dirname "$0")/.." && pwd)
MAP=$1; R=$(cd "$(dirname "$2")" && pwd)/$(basename "$2"); SEED=$3; F=$4
OUT=$ROOT/build/diagnose; mkdir -p "$OUT"
export SDL_VIDEODRIVER=offscreen SDL_AUDIODRIVER=dummy URBAN_HEADLESS=1 URBAN_DATA="$ROOT/ref/data"
export URBAN_REPLAY="$R" URBAN_SEED=$SEED URBAN_FRAMES=$((F + 1)) URBAN_TRACE_OUT=/dev/null
for b in cpp rust; do
	bin="$ROOT/build/urban"; [ $b = rust ] && bin="$ROOT/rust/target/release/urban"
	URBAN_RNG_TRACE=1 URBAN_STATE_FRAME=$F "$bin" -x -s -f "$MAP" 2> "$OUT/$b.err" >/dev/null
	grep '^RNG' "$OUT/$b.err" | c++filt | sed -E 's/\+[0-9]+$//; s/:[0-9]+$//' > "$OUT/$b.rng"
	grep '^S' "$OUT/$b.err" > "$OUT/$b.state"
done
echo "RNG calls: cpp $(wc -l < "$OUT/cpp.rng") rust $(wc -l < "$OUT/rust.rng")"
paste -d'|' "$OUT/cpp.rng" "$OUT/rust.rng" | awk -F'|' '{split($1,a," "); split($2,b," "); if (a[2]!=b[2]) {print "RNG count mismatch at line " NR; exit}}'
echo "--- state diff at frame $F (cpp < > rust)"
diff "$OUT/cpp.state" "$OUT/rust.state" | head -20
