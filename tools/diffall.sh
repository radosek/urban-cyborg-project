#!/bin/sh
# Diff C++ vs Rust on every campaign map with several random input scripts.
#   tools/diffall.sh [seeds-per-map] [frames]
ROOT=$(cd "$(dirname "$0")/.." && pwd)
N=${1:-3}; FRAMES=${2:-3000}
OUT=$ROOT/build/diffall; mkdir -p "$OUT"
pass=0; fail=0
for m in 11 12 13 21 22 23 31 32 33 41 42 43 51 52 53 61 62 63; do
	for s in $(seq 1 "$N"); do
		r="$OUT/replay_${m}_$s.txt"
		if [ $((s % 2)) -eq 1 ]; then
			python3 -I "$ROOT/tools/genreplay.py" "$m$s" "$FRAMES" LINUSISGOD NVTRULES TALKINGTODEATH > "$r"
		else
			python3 -I "$ROOT/tools/genreplay.py" "$m$s" "$FRAMES" > "$r"
		fi
		res=$(DIFF_OUT="$OUT/m${m}_$s" "$ROOT/tools/difftest.sh" "gamemaps/level$m.map" "$r" "$FRAMES" "$m$s" 2>&1 | head -1)
		echo "level$m seed$s: $res"
		case "$res" in IDENTICAL*) pass=$((pass + 1));; *) fail=$((fail + 1));; esac
	done
done
echo "pass=$pass fail=$fail"
