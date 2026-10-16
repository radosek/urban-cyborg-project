# URBAN – The Cyborg Project (native port)

Original 1999 DOS game by Thomas Nyberg & Jonas Borgström, GPLv2 (`ref/copying`).
`ref/` is the original source with minimal portability fixes; `port/` replaces Allegro 3 / GGI with SDL2.

## Build (macOS / Linux)

    brew install sdl2 libmikmod cmake pkgconf      # or apt: libsdl2-dev libmikmod-dev cmake pkg-config
    cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
    cmake --build build -j
    ./build/urban

Options: `-i` skip intro, `-s` no music, `-x` no sfx, `-f gamemaps/level11.map` play one map.
Alt/Cmd+Enter toggles fullscreen (`URBAN_FULLSCREEN=1` starts fullscreen). Gamepads via SDL GameController.

Saves, config and highscores live in SDL's per-user pref dir (`~/Library/Application Support/urban/` on macOS).

Debug env: `URBAN_DATA=<dir>` data path, `URBAN_SHOTS=<dir>` dump a frame per second, `URBAN_AUTOPLAY=1` scripted input.
