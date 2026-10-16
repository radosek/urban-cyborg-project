URBAN – The Cyborg Project **2.0.1**: the 1999 game, ported to Rust for Windows, Linux and macOS.

**Game data not included.** Download the original 1999 release (`URBAN-The-Cyborg-Project_DOS_EN.zip`) and copy `urban.dat`, `levels.dat`, `intro.dat` and the `snd` folder into a folder named `data` next to the program (next to `URBAN.app` on macOS), or into the per-user folder the game shows on first start:
- macOS: `~/Library/Application Support/urban/data`
- Windows: `%APPDATA%\urban\data`
- Linux: `~/.local/share/urban/data`

### Downloads
- **macOS:** `urban-macos-universal.zip` – URBAN.app for Apple Silicon and Intel.
- **Windows:** `urban-windows-x86_64.zip`
- **Linux:** `urban-linux-x86_64.zip`

### Faithful port
- Complete rewrite in Rust on SDL2; single executable, no install.
- Verified frame for frame against the original C++ game: all 18 levels, intro, ending, credits, menus, save slots.
- 30+ bugs from the original fixed, including a crash when starting a second game ([docs/FIXES.md](docs/FIXES.md)).

### New in 2.0.1
- macOS universal URBAN.app (Apple Silicon + Intel) with icon.
- The game explains where to put the game data if it is missing; data can also live in the per-user folder.

### New in 2.0
- Pause menu (ESC): resume, save, load, options, restart level, quit.
- Quick save **F5** / quick load **F9** anywhere; autosave at level start and checkpoints; **CONTINUE** on the main menu.
- Graphics options (CONFIG → GRAPHICS): pixel filters Scale3x / hq4x / smooth (**F6**), CRT look (**F7**), 16:9 widescreen (**F8**), fullscreen (Alt/Cmd+Enter).
- Gamepad support (Xbox, PlayStation, Switch Pro …).

### Notes
- The apps are not signed by Apple or Microsoft. On first start macOS asks once: right-click URBAN.app → Open (or System Settings → Privacy & Security → Open Anyway). Windows SmartScreen: More info → Run anyway.
- Based on URBAN 1.2/1.5.3 by Thomas Nyberg and Jonas Borgström (GPL v2). The port was written with an AI coding assistant (Claude).
