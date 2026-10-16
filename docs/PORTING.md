# Porting guide: ref/ (C++) → rust/ (Rust)

The C++ in `ref/` is the spec. Port behaviour faithfully, **except** for the defects listed in
`docs/BUGS.md` for your files: fix those. Keep the gameplay quirks listed at the end of BUGS.md
("may be intentional") as they are. Files in `ref/` are Latin-1 with CRLF; read them with
`LC_ALL=C` tools (`LC_ALL=C /usr/bin/grep`, `LC_ALL=C sed -n`), or with the Read tool.

Read first: `rust/src/engine.rs` (object model and Game API), `rust/src/objects/anim.rs`,
`rust/src/objects/mod.rs` (spawn table), `ref/include/object.h`, `ref/include/object2.h`,
`ref/include/engine.h`, `ref/include/ai.h`.

## Files and names

- `ref/objects/foo.cc` → `rust/src/objects/foo.rs` (the stub there is replaced completely).
- Class names drop `_o` and become CamelCase: `soldier_o` → `Soldier`, `Boss_soldier_o` →
  `BossSoldier`, `HighSpeed_Bullet_o` → `HighSpeedBullet`, `bodyparts_o`/`shells_o` → `Bodyparts`,
  `explosion_o` → `Explosion`, `gnistor_o` → `Gnistor`, `blood_o` → `Blood`, `player_o` → `Player`.
- Constructors are `pub fn new(...) -> Self` taking **every** C++ constructor parameter in order
  (C++ default arguments become explicit arguments; callers pass the default value). Keep the
  exact signature already in the stub. `char *` → `&str`, `Cardtype` → `i32` (`CARD_*`).
- Methods already declared in a stub (elevators, tank wheel, spy, player) keep their signatures:
  other files call them.

## Object model

```rust
pub struct Soldier { pub base: Base, anim: Anim, /* subclass fields */ }
impl Obj for Soldier {
    crate::obj_boilerplate!();
    fn update(&mut self, g: &mut Game) -> Upd { ... }
    fn collision(&mut self, other: &mut dyn Obj, g: &mut Game) { ... }
}
```

- `Base` has the C++ `Object` fields with the same names (`x, y, z, layer, speed_x, ...,
  counter..counter5, fcounter.., grav_counter, score, friends, enemies, me, energy, strength,
  images, current_image, ...`). `Base::new(x, y, z)` / `Base::with_mode(x, y, z, mode)` mirror the
  Object constructor (sets `layer = z / TILE_TOP_HEIGHT`).
- `images` is `Vec<Img>` (`Rc<Bitmap>`). Load with `icache::get("dead/blod1.pcx")` (path relative
  to gfx/, as in C++) or `icache::seq("dead/blod", 1, 3, ".pcx")`. `images[i]->w` →
  `self.base.images[i].w`. `num_images` → `images.len()`. GetImage → `self.base.image()`.
- Boss subclasses: wrap or flag, e.g. `pub struct BossSoldier { inner: Soldier }` delegating
  `base()`, or one struct with a `boss: bool`. Calls to the parent's `update()`/`Collision()` must
  keep the same order.
- Function-level `static` locals become per-instance fields (they leaked across instances/levels).

## Update return values

| C++                                           | Rust                    |
|-----------------------------------------------|-------------------------|
| `return 0;`                                   | `Upd::Keep`             |
| `return -1;` / `return DELETE_ME;`            | `Upd::Delete`           |
| `ENGINE.create_dekoration(this); return REMOVE_ME;` | `Upd::ToDekoration` |
| `ENGINE.create_alwaysupdate(this); return REMOVE_ME;` | `Upd::ToAlwaysUpdate` |
| `return REMOVE_ME;` alone                     | `Upd::Delete` (it leaked) |

The engine awards `base.score` points when any object in objects/alwaysupdate is removed.

## Engine services (`g: &mut Game`)

| C++                                   | Rust                                           |
|---------------------------------------|------------------------------------------------|
| `ENGINE.check_floor(x, y, z)`         | `g.check_floor(x, y, z)` → `bool`              |
| `ENGINE.check_wall(x, y, z)`          | `g.check_wall(x, y, z)` → `bool`               |
| `ENGINE.create_object(new X(...))`    | `g.create_object(X::new(...))`                 |
| `ENGINE.create_effect(...)`           | `g.create_effect(...)`                         |
| `ENGINE.create_alwaysupdate(...)`     | `g.create_alwaysupdate(...)`                   |
| `ENGINE.create_dekoration(...)`       | `g.create_dekoration(...)`                     |
| `ENGINE.PushMessage(s)`               | `g.push_message(s)`                            |
| `ENGINE.EnableEarthquake()`           | `g.enable_earthquake()`                        |
| `ENGINE.ClearLevel()` / `PlayExtraLevel()` | `g.clear_level()` / `g.play_extra_level()` |
| `ENGINE.ClearTile(x,y,z)`             | `g.clear_tile(x, y, z)`                        |
| `ENGINE.SetMaxUpdateX(n)`             | `g.set_max_update_x(n)`                        |
| `ENGINE.score.AddScore/InitBossHealth/SetBossHealth` | `g.score.add_score/init_boss_health/set_boss_health` |
| `ENGINE.map.SetPal(PAL_WATER)` / `GetPal()` | `g.map.set_pal(crate::map::PAL_WATER)` / `g.map.pal()` |
| `cheat_codes_active & CHEAT_X`        | `g.cheat_codes_active & CHEAT_X`               |
| `SOUND.PlaySFX("x.wav")`              | `crate::sound::play_sfx("x.wav")`              |
| `SOUND.PlaySFX_Critical(...)`         | `crate::sound::play_sfx_critical(...)`         |
| `random()`                            | `crate::rng::random()` (non-negative i32)      |
| `fade_in(ENGINE.map.GetPal(), 2)`     | `crate::engine::fade_in_map(g, 2)`             |
| `rest(ms)`, `key[KEY_X]`              | `crate::sys::rest(ms)`, `crate::sys::key(crate::keys::KEY_X)` |

## The player

- `PLAYER->GetX()` etc. → `g.pbase().x`, `.y`, `.z`, `.layer`, `.energy`, `.width`, ...
- `((player_o *)PLAYER)->AddAmmo(t, n)` → `g.player().add_ammo(t, n)`; likewise `have_weapon`,
  `enable_weapon`, `add_life`, `give_card`, `have_card`, `lives`, `ammo`, `kill_player(g)`.
- While the player is updating itself, `g.player` is empty: the player uses `self`.
- In `collision(other, g)`, when `other` is the player, `g.player` is empty too. Use
  `crate::engine::player_of(other, g)` (returns the player whether it is `other` or in `g`), or
  `other.downcast_mut::<Player>()`. Test `other.base().me == FRIEND_PLAYER` like the C++ does.

## Other objects

- `o->GetWho()` → `other.base().me`; `GetStrength()` → `.strength`; `GetFriends()` → `.friends`;
  `GetEnemies()` → `.enemies`; `GetCollisionX()` → `.collision_x()`; `o->SetMode(m)` →
  `other.base_mut().mode = m`.
- `Object::Collision(o)` → `default_collision(&mut self.base, other.base())`.
- Pointers between objects (`Object *elevator`, `Spy_o *spy`, `TankWheel_o *wheel1`) become
  `Option<ObjId>` (`base.id`). Reach them with `g.find::<T>(id)` → `Option<&mut T>`. It returns
  `None` for an object that is currently taken out (the one updating, or the two colliding).
  When you create a linked object, keep its id: `let w = TankWheel::new(..); let id = w.base.id;
  g.create_object(w);`.
- `g.ids_where(List::Innerlayer, |b| b.me == FRIEND_ELEVATOR)` replaces array scans.

## C semantics

- `int` → `i32`. Integer `/` and `%` truncate toward zero in both languages. `abs()` →
  `.abs()`. `(int)floatval` → `as i32`. `>>` on signed values is arithmetic in both.
- Where C uses an int as a boolean, keep `i32` if the value is used numerically, otherwise `bool`.
- Bounds: index with checks or clamps where the C++ could go out of range (see BUGS.md).

## Rules

- Complete ports only: no `todo!()`, `unimplemented!()`, TODO comments, or stubs left in your files.
- Comments only where logic is non-obvious; keep any meaningful original comment (translate
  Swedish to English).
- Do not edit files outside your list. If you need something from engine.rs or another file
  that is missing or wrong, work around it locally if trivial, and report it.
- `cd rust && cargo check` must show **no errors in your files** (stubs elsewhere are fine).
- When done, `git add` your files and commit in your worktree.
