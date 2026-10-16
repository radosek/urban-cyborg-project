URBAN – The Cyborg Project **2.0.0**: the 1999 game, ported to Rust for Windows, Linux and macOS.

**Game data not included.** Download the original 1999 release (`URBAN-The-Cyborg-Project_DOS_EN.zip`) and copy `urban.dat`, `levels.dat`, `intro.dat` and the `snd` folder into a folder named `data` next to the program.

### Faithful port
- Complete rewrite in Rust on SDL2; single executable, no install.
- Verified frame for frame against the original C++ game: all 18 levels, intro, ending, credits, menus, save slots.
- 30+ bugs from the original fixed, including a crash when starting a second game ([docs/FIXES.md](docs/FIXES.md)).

### New
- Pause menu (ESC): resume, save, load, options, restart level, quit.
- Quick save **F5** / quick load **F9** anywhere; autosave at level start and checkpoints; **CONTINUE** on the main menu.
- Graphics options (CONFIG → GRAPHICS): pixel filters Scale3x / hq4x / smooth (**F6**), CRT look (**F7**), 16:9 widescreen (**F8**), fullscreen (Alt/Cmd+Enter).
- Gamepad support (Xbox, PlayStation, Switch Pro …).

### Notes
- macOS: the binary is not notarized. If macOS blocks it, right-click → Open, or run `xattr -d com.apple.quarantine urban`.
- Based on URBAN 1.2/1.5.3 by Thomas Nyberg and Jonas Borgström (GPL v2). The port was written with an AI coding assistant (Claude).
