"""Generates a random input script for tools/difftest.sh.

usage: genreplay.py <seed> <frames> [cheat ...]  > replay.txt
Cheats are typed first (one letter per frame, released between letters), e.g. LINUSISGOD NVTRULES.
"""
import random, sys

KEYS = {c: k for c, k in zip("QWERTYUIOP", range(16, 26))}
KEYS.update({c: k for c, k in zip("ASDFGHJKL", range(30, 39))})
KEYS.update({c: k for c, k in zip("ZXCVBNM", range(44, 51))})
RIGHT, LEFT, UP, DOWN, SPACE, CTRL = 77, 75, 72, 80, 57, 29
NUM = list(range(2, 9))  # weapons 1..7

seed, frames = int(sys.argv[1]), int(sys.argv[2])
rnd = random.Random(seed)
out = ["0"]
f = 150  # let the level fade in
for cheat in sys.argv[3:]:
    for ch in cheat:
        out.append(f"{f} {KEYS[ch]}")
        out.append(f"{f + 1}")
        f += 2
    f += 5
while f < frames:
    held = []
    move = rnd.random()
    if move < 0.55:
        held.append(RIGHT)
    elif move < 0.7:
        held.append(LEFT)
    if rnd.random() < 0.15:
        held.append(rnd.choice([UP, DOWN]))
    if rnd.random() < 0.5:
        held.append(CTRL)
    if rnd.random() < 0.12:
        held.append(SPACE)
    if rnd.random() < 0.05:
        held.append(rnd.choice(NUM))
    out.append(f"{f} " + " ".join(map(str, held)))
    f += rnd.randint(3, 60)
print("\n".join(out))
