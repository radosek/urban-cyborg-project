#!/bin/sh
# Binary-searches the first frame whose object state differs between the builds.
#   tools/firstdiff.sh <map> <replay> <seed> <lo> <hi>
ROOT=$(cd "$(dirname "$0")/.." && pwd)
MAP=$1; R=$2; SEED=$3; LO=$4; HI=$5
same() { "$ROOT/tools/diagnose.sh" "$MAP" "$R" "$SEED" "$1" | awk '/^--- state diff/{s=1;next} /^---/{s=0} s{n++} END{exit n>0}'; }
while [ $((HI - LO)) -gt 1 ]; do
	MID=$(((LO + HI) / 2))
	if same $MID; then LO=$MID; else HI=$MID; fi
done
echo "first differing state at frame $HI"
"$ROOT/tools/diagnose.sh" "$MAP" "$R" "$SEED" "$HI"
