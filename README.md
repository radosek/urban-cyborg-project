# URBAN – The Cyborg Project

A native, cross-platform port of **URBAN – The Cyborg Project** (1999), a side-scrolling DOS action
game by Thomas Nyberg and Jonas Borgström, released under the GNU GPL v2.

- `rust/` — the port: a complete rewrite in Rust on SDL2 (statically linked, single binary).
- `ref/` — the original C++ source (URBAN 1.2), lightly patched to build natively on SDL2 via
  `port/` (an Allegro 3.11 subset). It is the reference the Rust port is tested against.

The Rust port reproduces the original frame for frame (see *Verification*) and fixes the original
game's bugs listed in [docs/FIXES.md](docs/FIXES.md).

## Game data

This repository contains code only. The game data (graphics, levels, music) is not included.
Get the original 1999 release (it is widely mirrored as
`URBAN-The-Cyborg-Project_DOS_EN.zip`) and copy these from its `urban` folder:

    urban.dat  levels.dat  intro.dat  snd/

into a folder named `data` next to the `urban` executable (or set `URBAN_DATA=/path/to/them`).

## Download

Prebuilt binaries for Windows, Linux and macOS are attached to each
[release](https://github.com/radosek/urban-cyborg-project/releases), and to every CI run.

## Build from source

    cd rust
    cargo build --release        # needs CMake; SDL2 is built from source and linked statically

On Linux install the usual SDL build dependencies first (ALSA/PulseAudio, X11, Wayland headers;
see `.github/workflows/build.yml`).

## Play

    ./urban            # -i skip intro, -s no music, -x no sound effects, -b no backgrounds
    ./urban -f gamemaps/level11.map   # play one map

- Alt/Cmd+Enter toggles fullscreen (`URBAN_FULLSCREEN=1` starts fullscreen).
- Keyboard by default: arrows, Ctrl fire, Space jump, Shift prev/next weapon, 1–7 weapons, P pause.
- Gamepads (Xbox, PlayStation, Switch Pro, …) via SDL; choose GAMEPAD in CONFIG → controls.
- Config, saves and highscores live in the per-user app data folder.

## Verification

The C++ reference and the Rust port run with identical scripted input and random seed. Every frame
(or every key read / pause in menus and cutscenes) is fingerprinted and compared. All identical:

    tools/diffall.sh 4 6000   # 18 levels x 4 random playthroughs x 6000 frames
    tools/savetest.sh         # new game, clear level, save, load
    tools/campaigntest.sh     # all 18 levels, ending, credits, highscore, second game
    tools/menutest.sh         # main menu, highscores, credits, config
    tools/seqtest.sh          # intro; RUN=outtro / RUN=credits KEYPRESSES=999999 for the others
    tools/flitest.sh          # all 11 intro animations

These need the game data in `ref/data` and the C++ reference build:

    brew install sdl2 libmikmod cmake pkgconf    # or the Linux equivalents
    cmake -S . -B build -DCMAKE_BUILD_TYPE=Release && cmake --build build -j

`tools/next.sh`, `tools/diagnose.sh` and `tools/firstdiff.sh` explain a divergence (random-number
call sites, object states, collision pairs). Porting notes: [docs/PORTING.md](docs/PORTING.md).

## License

GPL v2 or later, as the original (`ref/copying`). Original game © 1999 Thomas Nyberg and Jonas
Borgström.
