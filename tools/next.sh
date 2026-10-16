#!/bin/sh
# Diff one map/replay/seed; on divergence, diagnose the frame before it.
#   tools/next.sh <level-number e.g. 42> <seed-index>
ROOT=$(cd "$(dirname "$0")/.." && pwd)
M=$1; S=$2; R="$ROOT/build/diffall/replay_${M}_$S.txt"
res=$(DIFF_OUT="$ROOT/build/next" "$ROOT/tools/difftest.sh" "gamemaps/level$M.map" "$R" 3000 "$M$S" | head -1)
echo "$res"
F=$(echo "$res" | sed -n 's/DIVERGED at frame \([0-9]*\).*/\1/p')
[ -n "$F" ] && "$ROOT/tools/diagnose.sh" "gamemaps/level$M.map" "$R" "$M$S" $((F - 1))
