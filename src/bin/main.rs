use anyhow::{Context, Result};
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("--version") => {
            println!("oxwm {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some("--help") => {
            print_help();
            return Ok(());
        }
        Some("--init") => {
            init_config()?;
            return Ok(());
        }
        _ => {}
    }

    let config = load_config()?;

    let mut wm = oxwm::window_manager::WindowManager::new(config)?;
    let should_restart = wm.run()?;

    drop(wm);

    if should_restart {
        use std::os::unix::process::CommandExt;
        let err = std::process::Command::new(&args[0]).args(&args[1..]).exec();
        eprintln!("Failed to restart: {}", err);
    }

    Ok(())
}

fn load_config() -> Result<oxwm::Config> {
    let config_path = get_config_path().join("config.ron");

    if !config_path.exists() {
        println!("No config found at {:?}", config_path);
        println!("Creating default config...");
        init_config()?;
    }

    let config_str =
        std::fs::read_to_string(&config_path).with_context(|| "Failed to read config file")?;

    oxwm::config::parse_config(&config_str).with_context(|| "Failed to parse config")
}

fn init_config() -> Result<()> {
    let config_dir = get_config_path();
    std::fs::create_dir_all(&config_dir)?;

    let config_template = include_str!("../../templates/config.ron");
    let config_path = config_dir.join("config.ron");

    std::fs::write(&config_path, config_template)?;

    println!("✓ Config created at {:?}", config_path);
    println!("  Edit the file and reload with Mod+Shift+R");
    println!("  No compilation needed - changes take effect immediately!");

    Ok(())
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .expect("Could not find config directory")
        .join("oxwm")
}

fn print_help() {
    println!("OXWM - A dynamic window manager written in Rust\n");
    println!("USAGE:");
    println!("    oxwm [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --init         Create default config in ~/.config/oxwm/config.ron");
    println!("    --version      Print version information");
    println!("    --help         Print this help message\n");
    println!("CONFIG:");
    println!("    Location: ~/.config/oxwm/config.ron");
    println!("    Edit the config file and use Mod+Shift+R to reload");
    println!("    No compilation needed - instant hot-reload!\n");
    println!("FIRST RUN:");
    println!("    Run 'oxwm --init' to create a config file");
    println!("    Or just start oxwm and it will create one automatically\n");
}
