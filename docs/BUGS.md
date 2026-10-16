# Bugs in the original C++ (fix in the Rust port)

Paths relative to `ref/`. Source: audit of URBAN 1.2.

I found about 45 real or highly plausible defects in the C++ source; no files were edited. The audit was split across four parallel agents, each told to confirm findings against surrounding code and callers. I merged their reports without re-checking every line myself, and nothing was compiled or run. Paths are under `ref/`.

**PCX claim: confirmed, and it is worse than reported.** `datfile/datfile.cc:116-131` decodes exactly `width` bytes per row and ignores `bytes_per_line`. The padding byte on odd-width images becomes the first pixel of the next row, so images skew. A run-length run that crosses the end of a row also writes past it, and on the last row that is a heap overrun. `color_planes` is read but never used. Correct behaviour: decode `bytes_per_line * planes` bytes per row into a temporary buffer, keep the first `width`, and clamp runs.

## Bugs, most severe first

### Memory safety and crashes

1. **engine/engine.cc:906-909** – `play_level` deletes every `alwaysupdate[i]` but never resets `num_alwaysupdate`.
   - Trigger: an airstrike object (map airstriker or the RETURNOFMRA cheat) is alive at level end.
   - Consequence: next level, display (1142/1145) and collision (1509) use freed objects.
   - Fix: reset the count to 0.
2. **engine.cc:848-851** – `NewGame` loops over `num_alwaysupdate` but deletes `objects[i]`, a copy-paste error.
   - Consequence: double delete of objects already freed at 844, and the always-update objects leak.
   - Fix: delete `alwaysupdate[i]`.
3. **engine.cc:1325 and `create_objects` (310-718)** – no bound check against `MAX_OBJECTS` (1000) or `MAX_INNERLAYER` (100).
   - Trigger: heavy projectile spam or a dense map overflows into the next arrays.
   - Fix: use bounded or growable collections.
4. **map/map.cc:660ff** – row parsing does `j++` with no check against `x_size`, row count comes from the file, and `num_tiles++` is unbounded against `MAX_TILES`.
   - Consequence: a long or malformed row overflows the heap.
   - Fix: validate dimensions.
5. **map.cc:621, 636, 899, 923** – on EOF, `GetLine` returns NULL straight into `strcmp`/`strtok`.
   - Maps inside the dat file are read in binary, so a blank CRLF line is `"\r\n"` and isn't skipped as blank.
   - Consequence: inside `<OPTIONS>` it crashes; in a section it is taken as a map row.
   - Fix: strip `\r\n` and handle EOF.
6. **map.cc:901-902** – `options[i++]` is written before the `MAX_OPTIONS` check, so the 51st option writes out of bounds. A full array also has no NULL terminator, which `SaveMap` and `GetOption` rely on.
7. **engine.cc:1371/1547** – `check_floor`/`check_wall` call `IS_FAKE(GetBG())` on empty tile 0, which reads `TileInfo[-1]` on every gravity check over air.
   - `GetTileInfo` has no bounds check, and entries between `num_tiles` and 300 are uninitialised `malloc` memory.
   - Fix: test for 0 first and bound-check.
8. **map.cc:939, 952, 964, 981, 996, 1013, 1091** – `GetBackGround`, `GetEffect`, `GetDekor` and `GetTile(x,y,z)` use `x > x_size` instead of `>=`.
   - Consequence: off-by-one reads at the right and bottom edges.
9. **map.cc:503-531, 571/577** – memory from `new[]` is released with `free`, memory from `strdup` with `delete[]`, and `sections[]`/`objects[]` aren't NULLed after free.
   - Trigger: a map missing a `<SECTION_n>` uses freed memory, then double-frees it on the next load.
10. **objects/player.cc:188-508** – `elev_station` is never initialised.
    - Trigger: fire while facing up on the first frame calls `StartElevator()` on a garbage pointer.
    - Fix: set it to NULL.
11. **player.cc:196-210** – `ctrl` stays uninitialised for an unknown or unsupported `ctrl_type`.
    - Consequence: crash on the first frame, and on `delete ctrl` later.
    - Fix: fall back to keyboard.
12. **objects/tank.cc:440, 461** – `static int cmc` indexes `comments[5]` and is never reset.
    - Trigger: a second boss fight in the same process (retry or new game) reads past the array and passes a garbage string to `PlaySFX_Critical`.
    - The statics `aim`, `aif` and `cluster_counter` (231-233) also carry over, so a retried boss starts its attack script part-way through.
    - Fix: make them per-instance members.
13. **engine/game.cc:279** – inverted test, `if ((fs = fopen(..., "wb")) == NULL) fwrite(..., fs)`.
    - Consequence: crash when the save can't be created, empty file when it can.
    - The `fread` at 282 is unchecked, and the saved `level` indexes `maps[level-1]` (294) without validation.
    - Fix: use `!= NULL`, check the read size, clamp `level` to 1..18.
14. **game.cc:252** – `char name[80]` is uninitialised. Saving to an empty slot runs `strlen` on garbage.
    - SPACE allows position 9 without writing a terminator (229-233), and `textbmp` leaks on ENTER (212-216).
15. **game.cc:314** – `extra_maps` has 7 entries but is indexed by `level-1` up to 17.
    - Latent: `door.cc` calls `ClearLevel`, not `PlayExtraLevel`.
16. **highscor/highscor.cc:103** – `strcpy(Name, getenv("USER"))` into `Name[64]` overflows on a long USER.
    - Unchecked `fread` of hs.dat and savegame.dat (highscor.cc:202, game.cc:282, meny.cc:212) can leave names unterminated, which then overflow `buffer[80]` through `strcat` (game.cc:118, meny.cc:220).
17. **gfx/flc.cc:321-322 (SS2 chunk)** – the word fill loops `count*2` times, storing a `short` at every byte offset, so the pattern is wrong and it writes one byte past the run.
    - Unsupported chunks aren't skipped by `frame_h.size`, so the stream desyncs.
    - Frames larger than 320x240 overflow the buffer, the palette index isn't capped at 256, and the file is never closed.
    - Low priority: intro and outtro use Allegro's own FLI player.
18. **objects/bdyparts.cc:90-91** – returns `REMOVE_ME`, but the engine only deletes on `DELETE_ME`. Every body part spawned inside a wall leaks.

### Gameplay logic

19. **player.cc:1089** – `Collision` kills on `energy < 0`, while landing checks `<= 0`. A hit that leaves exactly 0 energy doesn't kill.
20. **player.cc:741** – the electric weapon's wall test uses `x` (the player) instead of `tmp` (the beam segment). Either all 5 segments spawn, through walls, or none do.
21. **rocket.cc:99-102, missile.cc:100-102** – `counter += speed_x` goes negative for leftward shots, so the 600-unit range limit only works to the right. Use `abs(speed_x)`.
    - Also `direction` is inverted (rocket.cc:76, missile.cc:76).
22. **floorgun.cc:105 / spy.cc** – `Spy_o` never initialises `victim`, so a floor gun can fire on its own from the start.
23. **powerup.cc:186-198 with player.cc:1065** – a medipak picked up while the player is immortal (respawn or touch immunity) is consumed but doesn't heal.
24. **crycheck.cc:418-427** – fireball, beam, ice and explosion hits kill `Boss_Crycheck` (energy 30) outright, ignoring its energy. Only bullets and plasma respect its health.
25. **drgreen.cc:256-259** – a plasma hit spawns gibs every frame of contact but never sets `STATE_DEAD`. Lines 271-276 are dead code after two returns, so bullets never draw blood.
26. **door.cc:142-163** – `msg` is uninitialised for a `none` or unrecognised card, and the garbage is pushed as a message every frame.
27. **cluster.cc:109-110** – a leftward burst spawns at `x + width + 5`, copied from the right case. Should be `x - 5`.
28. **grenade.cc:83-86, cluster.cc:84-87** – they move in z without clamping z or updating `layer`, so they draw and collide on the wrong layer. `clugren.cc` and `airgren.cc` do it correctly.
29. **crycheck.cc:441/443** – blood direction differs: right uses `-2 + random()%4`, left uses `2 + random()%4`. Right should be `-(2 + random()%4)`.
30. **water.cc:93-99** – anything falling into water (soldier, grenade) switches the global palette to water. Only do it for the player.
31. **rambo.cc / dolph.cc jump logic**:
    - Mid-air, `speed_x = ±5` runs with no wall check, so they jump through walls.
    - The re-roll on `!counter` replaces the jump state mid-air, so landing is almost never reached.
    - `counter2` is shared with the firing logic and can skip the jump delay.
32. **soldier3.cc:249-293** – the fire state is a commented-out stub, so this soldier freezes in "fire" and never shoots.
33. **frosen.cc:105** – frozen bodies only shatter on an exact match with `ENEMY_HS_BULLET` or `ENEMY_GRENADE`. Explosions, fireballs and plasma pass through.

### Sprite frame indexing (wrong frame shown)

34. **seagal.cc ~347** – the firing frame `r` goes 0..5, but there are only 4 shot frames per side. Use `next_frame(3, ...)` and stop at `r == 3`.
35. **soldier2.cc:322, soldier4.cc:329** – the throw base `RIGHT ? 23 : 21` should be `22 : 20`.
36. **dolph.cc jump** – uses Rambo's `RIGHT ? 29 : 27`. Should be `31 : 29`.
37. **arnold.cc:373** – the alarm frame `RIGHT ? 24 : 20` should use the right-facing base 25. The guard copied from Rambo is also wrong for Arnold's layout.
38. **science.cc:226, einstein.cc:178** – precedence bug: in `dir == RIGHT ? 34 : 37 + frame` the animation frame only applies to the left branch. Parenthesise the conditional.
39. **drgreen.cc:94/99** – both death loops load `doc/docv/`. The first should load `doc/doch/`.
40. **science.cc:253, einstein, drgreen.cc:177, burning.cc:146** – idle `RIGHT ? 5 : 0` is mirrored (index 0 is the right-facing set everywhere else).

### Input, config and UI

41. **ctrls/keyb.cc:48-49** – next and previous weapon keys are swapped.
42. **ctrls/gamepad.cc:104** – a second `install_joystick` is called with a controller enum instead of an Allegro joystick type; this one rests on Allegro 3.x numbering recalled from memory, not checked against its source.
    - `key_right` is left out of the NULL initialisation (103).
    - Prev/next weapon bindings are commented out (113-114), so the gamepad can't switch weapons even though the config screen asks for those buttons.
43. **include/game.h `_SET_BACKGROUND_ON/OFF`** – these use `FLAG_SOUNDFX`, so `-b` turns off sound effects instead of backgrounds.
44. **config/config.cc:315-381** – sound-effect volume and graphics quality are never saved.
    - ESC part-way through configuring keyboard or gamepad leaves a half-changed mapping (and `ctrl_type`), which still gets saved.
45. **game.cc:149-169** – after finishing level 18 it prints an uninitialised `message` (its `sprintf` is commented out).
46. **score.cc:123/136** – the face index goes out of range for energy below -30 or above 300, and a `BossMax` of 0 divides by zero.
47. **engine.cc:1365 vs 1381** – collision computes the layer as `z/15`, tile edits as `z/16`, so near layer boundaries they hit different layers.

### Leaks and uninitialised state

48. Leaks:
    - `~Object` never frees the `images` array, and `num_images` counts only the images that loaded, so the free loop can skip or misindex (object.cc:50-55).
    - `PushMessage` overwrites the message bitmap without freeing it (engine.cc:1638).
    - `BGTiles`/`BGFilename` leak every level (map.cc:592), and so does the `strdup` in `GetOption` (map.cc:1139).
    - `StopMusic` drops the current module (sound.cc:148).
    - Skipping credits leaks its buffers and font.
    - `~Engine` never deletes `innerlayer`.
49. The `Engine` constructor leaves `lightening`, `do_shake`, `num_shells`, `control_type` and `level_name` uninitialised.
50. Smaller:
    - The datfile `entries[2000]` isn't checked against `hdr.num_entries` (datfile.cc:171).
    - The message pop loop at engine.cc:954 clears only half the queue (harmless while it holds at most one).
    - `user_dir()` is called but defined nowhere, so a non-DOS build won't link.
    - `HOLE_RIGHT` in `include/ai.h` uses `FOOT_LEFT`, but nothing uses that macro.

## Quirks that may be intentional (your call)

- Your own rocket and air-grenade explosions can hurt the player. A rocket fired left spawns near the player's own hitbox.
- Beams always drift right, because direction comes from a speed of 0.
- Every level starts on the highest weapon you own (weapons are re-enabled in ascending order and auto-switch).
- A weapon pickup you already own, and the "access denied" door message, repeat every frame while you stand on them.
- Fireballs and iceballs pass through walls. Mines only trigger on the player.
- Rambo's bullets can kill scientists.
- A second hit during a scientist's collapse gibs him, even from a bullet.
- Shattering a frozen corpse gives another 200 points.
- The tank is immune to fire, beam, ice and plasma, and explosions do half damage to it.
- Rambo, Seagal and Dolph flinch whenever `energy % 4 == 0`.
- Soldier2/4 turn their back on the player after every throw. Soldier4 fires cluster bombs across layers beyond 150px.
- Seagal changes layer instead of shooting when you're on another layer.
- Objects outside roughly ±500px of the camera are neither updated nor collided.
- The TIME option kills and respawns the player rather than ending the game.
- The name entry filter `'A'..'z'` lets through `[\]^_` and the backtick, which the font then drops.
- Out-of-range tile numbers silently become tile 0 (the source marks this as a hack).
- The intro and outtro force music volume to 64, ignoring the config.
- The image cache never frees anything; `FreeImage` only decrements a counter. That avoids double-free bugs but means nothing is ever released.
- Gamepad weapon keys set to 0 fall back to INSERT/DEL.