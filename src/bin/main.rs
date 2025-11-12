use anyhow::{Context, Result};
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut custom_config_path: Option<PathBuf> = None;

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
        Some("--migrate") => {
            let path = args.get(2).map(PathBuf::from);
            migrate_config(path)?;
            return Ok(());
        }
        Some("--config") => {
            if let Some(path) = args.get(2) {
                custom_config_path = Some(PathBuf::from(path));
            } else {
                eprintln!("Error: --config requires a path argument");
                std::process::exit(1);
            }
        }
        _ => {}
    }

    let config = load_config(custom_config_path)?;

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

fn load_config(custom_path: Option<PathBuf>) -> Result<oxwm::Config> {
    let config_path = if let Some(path) = custom_path {
        path
    } else {
        let config_dir = get_config_path();
        let lua_path = config_dir.join("config.lua");
        let ron_path = config_dir.join("config.ron");

        if lua_path.exists() {
            lua_path
        } else if ron_path.exists() {
            ron_path
        } else {
            println!("No config found at {:?}", config_dir);
            println!("Creating default Lua config...");
            init_config()?;
            config_dir.join("config.lua")
        }
    };

    let config_str =
        std::fs::read_to_string(&config_path).with_context(|| "Failed to read config file")?;

    let is_lua = config_path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s == "lua")
        .unwrap_or(false);

    if is_lua {
        let config_dir = config_path.parent();
        oxwm::config::parse_lua_config(&config_str, config_dir)
            .with_context(|| "Failed to parse Lua config")
    } else {
        oxwm::config::parse_config(&config_str).with_context(|| "Failed to parse RON config")
    }
}

fn init_config() -> Result<()> {
    let config_dir = get_config_path();
    std::fs::create_dir_all(&config_dir)?;

    let config_template = include_str!("../../templates/config.lua");
    let config_path = config_dir.join("config.lua");

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

fn migrate_config(custom_path: Option<PathBuf>) -> Result<()> {
    let ron_path = if let Some(path) = custom_path {
        path
    } else {
        get_config_path().join("config.ron")
    };

    if !ron_path.exists() {
        eprintln!("Error: RON config file not found at {:?}", ron_path);
        eprintln!("Please specify a path: oxwm --migrate <path/to/config.ron>");
        std::process::exit(1);
    }

    println!("Migrating RON config to Lua...");
    println!("  Reading: {:?}", ron_path);

    let ron_content = std::fs::read_to_string(&ron_path)
        .with_context(|| format!("Failed to read RON config from {:?}", ron_path))?;

    let lua_content = oxwm::config::migrate::ron_to_lua(&ron_content)
        .with_context(|| "Failed to migrate RON config to Lua")?;

    let lua_path = ron_path.with_extension("lua");

    std::fs::write(&lua_path, lua_content)
        .with_context(|| format!("Failed to write Lua config to {:?}", lua_path))?;

    println!("✓ Migration complete!");
    println!("  Output: {:?}", lua_path);
    println!("\nYour old config.ron is still intact.");
    println!(
        "Review the new config.lua and then you can delete config.ron if everything looks good."
    );

    Ok(())
}

fn print_help() {
    println!("OXWM - A dynamic window manager written in Rust\n");
    println!("USAGE:");
    println!("    oxwm [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --init              Create default config in ~/.config/oxwm/config.lua");
    println!(
        "    --migrate [PATH]    Convert RON config to Lua (default: ~/.config/oxwm/config.ron)"
    );
    println!("    --config <PATH>     Use custom config file (.lua or .ron)");
    println!("    --version           Print version information");
    println!("    --help              Print this help message\n");
    println!("CONFIG:");
    println!("    Location: ~/.config/oxwm/config.lua (or config.ron for legacy)");
    println!("    Edit the config file and use Mod+Shift+R to reload");
    println!("    No compilation needed - instant hot-reload!\n");
    println!("MIGRATION:");
    println!("    To migrate from RON to Lua: oxwm --migrate");
    println!("    Or specify a custom path: oxwm --migrate /path/to/config.ron\n");
    println!("FIRST RUN:");
    println!("    Run 'oxwm --init' to create a config file");
    println!("    Or just start oxwm and it will create one automatically\n");
}
