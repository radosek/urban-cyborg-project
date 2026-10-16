# Bugs fixed from the original URBAN (1999)

These are defects in the original C++ game that the Rust port fixes. Numbers like `#25` refer to
the audit in [BUGS.md](BUGS.md). The C++ reference build under `ref/` carries the same fixes
wherever the replay tests reached them, so both builds stay comparable frame by frame.

## Verified by the C++ vs Rust replay tests

Each of these caused a divergence in `tools/difftest.sh`, was traced to its cause, checked
against the original sprites or code, and then applied to both builds.

| # | Where | Bug in the original | Fix |
|---|-------|---------------------|-----|
| 40 | Einstein, scientist, Dr. Green, burning body | Idle frame mirrored: standing still showed the left-facing sprite when facing right | Right-facing idle uses the right-facing set |
| 38 | Einstein, scientist | Operator precedence: `RIGHT ? 24 : 27 + frame` animated only the left-facing writing pose | Parenthesised |
| 25 | Dr. Green | Blood on bullet hits was unreachable (after two `return`s); plasma never set the dead state, so he gibbed every frame of contact | Blood sprays on hit; plasma kills once |
| 39 | Dr. Green | Both death animations loaded the same left-falling frames | Right-facing death uses `doc/doch/dead*` |
| 35 | Soldier 2 / 4 | Throw animation started at the wrong frame (ran off the end into frame 0, or showed a right-facing frame while facing left) | Throw frames 20–21 / 22–23 |
| 19 | Player | A hit that left exactly 0 health didn't kill | Dies at `<= 0` |
| 29 | Crycheck, scientist, Einstein, Dr. Green | Blood from right-facing hits flew the wrong way (speed `-2 + rand%4`, sometimes 0) | `-(2 + rand%4)` |
| 31 | Rambo, Arnold, Dolph | Jump logic: AI re-rolled its behaviour mid-air (rarely landed), shared `counter2` with firing, jumped through walls | Own jump/land flags, no re-roll while airborne, wall check while airborne |
| 36 | Dolph | Mid-jump frame used Rambo's indices | Dolph's own jump frames |
| 20 | Player, electric weapon | Wall test used the player's position, so all 5 beam segments spawned through walls or none did | Each segment checks its own position |
| 21 | Rocket, missile | Range counter went negative for leftward shots (they flew forever); facing direction inverted | `abs(speed)` range, correct direction |
| 27 | Cluster bomb | Leftward burst spawned on the right side (copy-paste of the right case) | Spawns at `x - 5` |
| 34 | Seagal | Firing animation ran to frame 5 with only 4 shot frames | 4 frames |
| 33 | Frozen body | Only bullets and grenades shattered it; explosions, fireballs and plasma passed through | All of them shatter it |
| 3 | Engine | Innerlayer (doors/elevators/wires) array of 100 overflowed: level 6-3 has 114 entries, writing past the array | No fixed cap (C++ reference: 400) |

## Fixed in the Rust port, not yet reached by the replay tests

From the audit and the porting work. These are in the Rust code; the random test runs haven't
triggered them yet, so they haven't been mirrored into the C++ reference.

**Crashes and memory safety (impossible in the Rust port by construction, listed for completeness)**
- #1, #2: level/new-game cleanup reused freed objects and double-deleted always-update objects (airstrikes).
- #4–#9: map loader overflows on long rows, too many tiles or options; EOF and CRLF crashes; off-by-one edge reads; `TileInfo[-1]` read on every gravity check over empty tiles; mixed `free`/`delete`.
- #10, #11: player's elevator pointer and controller left uninitialised (crash on first fire or unknown controller type → now keyboard).
- #12: tank boss attack state (`cmc`, `aim`, `aif`, `cluster_counter`) carried over to the next fight and indexed past its array.
- #13–#16: save games: inverted write check, unchecked reads, uninitialised slot name, unbounded level index, `USER` name overflow.
- #17: FLI player: SS2 chunk decoded wrongly and overran; unknown chunks desynced the stream (new decoder).
- #18: body parts spawned inside walls leaked.
- PCX loader ignored `bytes_per_line` padding: odd-width images skewed and could overrun.

**Gameplay**
- #22: floor guns could fire on their own (spy's `victim` uninitialised).
- #23: a medipak picked up while immortal was used up without healing.
- #24: fireball/beam/ice/explosion hits killed Boss Crycheck outright, ignoring its 30 energy.
- #26: doors with no/unknown card pushed a garbage message every frame.
- #28: grenades and cluster bombs moved in depth without clamping or updating their layer (drawn and colliding on the wrong layer).
- #30: anything falling into water switched the whole palette to underwater (only the player should).
- #32: Soldier 3's fire state was an empty stub; he froze instead of shooting (now fires a burst).
- #37: Arnold's alarm frame used the wrong base.

**Input, config, UI**
- #41: next/previous weapon keys were swapped.
- #42: gamepad button 8 couldn't be bound (missing `break`); gamepad prev/next weapon disabled.
- #43: `-b` (no backgrounds) turned off sound effects instead.
- #44: sound-effect volume and graphics quality were never saved; ESC mid-way through key setup kept a half-changed mapping.
- #45: finishing the last level printed an uninitialised message.
- #46: scoreboard face index out of range for unusual energy values; boss bar divided by zero.

## Deliberately kept (original behaviour, possibly intended)

Listed at the end of [BUGS.md](BUGS.md): your own rockets can hurt you, weapon pickups you already
own repeat their message, the tank's immunities, objects outside ±500 px aren't updated, etc.
