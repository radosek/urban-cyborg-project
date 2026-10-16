mod config;
mod ctrls;
mod dat;
mod engine;
mod flags;
mod font;
mod gfx;
mod icache;
mod keys;
mod map;
mod objects;
mod otypes;
mod rng;
mod score;
mod sound;
mod sys;
mod text;
mod tiles;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut level: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "-h" => {
                println!("Urban v1.2 (Rust)\nOptions:\n-f<mapname>   Play a specific map\n-x            Disable sound effects\n-s            Disable music\n-b            Disable backgrounds\n-i            Don't show intro\n-h            Show this");
                return;
            }
            "-s" => flags::set_sound(false),
            "-x" => flags::set_soundfx(false),
            "-b" => flags::set_background(false),
            "-i" => flags::set_intro(false),
            _ if a.starts_with("-f") => {
                level = Some(if a.len() > 2 { a[2..].to_string() } else { i += 1; args.get(i).cloned().unwrap_or_default() });
            }
            _ => {}
        }
        i += 1;
    }
    config::load();
    sys::init();
    icache::init();
    sound::set_music_volume(config::get().music_vol);
    let mut g = engine::Game::new();
    if let Some(l) = level {
        flags::set_spec_level(true);
        sys::set_palette(&gfx::BLACK_PALETTE);
        g.play_level(&l, None, config::get().ctrl_type);
        return;
    }
    eprintln!("menu not ported yet; use -f gamemaps/level11.map");
}
