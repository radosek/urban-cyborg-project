mod config;
mod config_ui;
mod credits;
mod ctrls;
mod dat;
mod engine;
mod flags;
mod flc;
mod game;
mod highscore;
mod font;
mod gfx;
mod icache;
mod intro;
mod keys;
mod map;
mod menu;
mod objects;
mod otypes;
mod outtro;
mod rng;
mod saves;
mod score;
mod sound;
mod sys;
mod text;
mod video;
mod tiles;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut level: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "-h" => {
                println!("Urban v2.0.1 (Rust)\nOptions:\n-f<mapname>   Play a specific map\n-x            Disable sound effects\n-s            Disable music\n-b            Disable backgrounds\n-i            Don't show intro\n-h            Show this");
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
    if let Ok(name) = std::env::var("URBAN_FLI_TEST") {
        fli_test(&name);
    }
    // Test hook (tools/seqtest.sh): run one sequence directly.
    if let Ok(what) = std::env::var("URBAN_RUN") {
        match what.as_str() {
            "outtro" => outtro::run_outtro(),
            "credits" => credits::show_credits(),
            _ => {}
        }
        std::process::exit(0);
    }
    sound::set_music_volume(config::get().music_vol);
    let mut g = engine::Game::new();
    if let Some(l) = level {
        flags::set_spec_level(true);
        sys::set_palette(&gfx::BLACK_PALETTE);
        g.play_level(&l, None, config::get().ctrl_type);
        return;
    }
    menu::run(&mut g);
}

/// Test hook (tools/flitest.sh): play one animation from intro.dat and fingerprint every frame.
fn fli_test(name: &str) -> ! {
    use std::io::Write;
    sys::set_palette(&gfx::BLACK_PALETTE);
    let dat = dat::Datfile::open(&sys::data_dir().join("intro.dat")).expect("intro.dat");
    let data = dat.get(name).unwrap_or_else(|| {
        eprintln!("no {}", name);
        std::process::exit(1)
    });
    let mut out = std::fs::File::create(std::env::var("URBAN_TRACE_OUT").expect("URBAN_TRACE_OUT")).expect("trace");
    let max: usize = std::env::var("URBAN_FRAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(200);
    let mut n = 0;
    flc::play_memory_fli(data, true, || {
        let _ = writeln!(out, "{} {:016x}", n, sys::current_screen_hash());
        if std::env::var("URBAN_DUMP").ok().and_then(|v| v.parse::<usize>().ok()) == Some(n) {
            if let Some(d) = std::env::var_os("URBAN_DUMP_DIR") {
                sys::dump_screen(&std::path::PathBuf::from(d).join(format!("rust_{:06}.bmp", n)));
            }
        }
        n += 1;
        n >= max
    });
    std::process::exit(0)
}
