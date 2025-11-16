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

    let (config, had_broken_config) = load_config(custom_config_path)?;

    let mut wm = oxwm::window_manager::WindowManager::new(config)?;

    if had_broken_config {
        wm.show_migration_overlay();
    }

    let should_restart = wm.run()?;

    drop(wm);

    if should_restart {
        use std::os::unix::process::CommandExt;
        let err = std::process::Command::new(&args[0]).args(&args[1..]).exec();
        eprintln!("Failed to restart: {}", err);
    }

    Ok(())
}

fn load_config(custom_path: Option<PathBuf>) -> Result<(oxwm::Config, bool)> {
    let config_path = if let Some(path) = custom_path {
        path
    } else {
        let config_dir = get_config_path();
        let lua_path = config_dir.join("config.lua");

        if !lua_path.exists() {
            let ron_path = config_dir.join("config.ron");
            let had_ron_config = ron_path.exists();

            println!("No config found at {:?}", config_dir);
            println!("Creating default Lua config...");
            init_config()?;

            if had_ron_config {
                println!("\n NOTICE: OXWM has migrated to Lua configuration.");
                println!("   Your old config.ron has been preserved, but is no longer used.");
                println!("   Your settings have been reset to defaults.");
                println!("   Please manually port your configuration to the new Lua format.");
                println!("   See the new config.lua template for examples.\n");
            }
        } else {
            // Always update LSP files to match the running version
            update_lsp_files()?;
        }

        lua_path
    };

    let config_str =
        std::fs::read_to_string(&config_path).with_context(|| "Failed to read config file")?;

    let config_dir = config_path.parent();

    match oxwm::config::parse_lua_config(&config_str, config_dir) {
        Ok(config) => Ok((config, false)),
        Err(_) => {
            let template = include_str!("../../templates/config.lua");
            let config = oxwm::config::parse_lua_config(template, None)
                .with_context(|| "Failed to parse default template config")?;
            Ok((config, true))
        }
    }
}

fn init_config() -> Result<()> {
    let config_dir = get_config_path();
    std::fs::create_dir_all(&config_dir)?;

    let config_template = include_str!("../../templates/config.lua");
    let config_path = config_dir.join("config.lua");
    std::fs::write(&config_path, config_template)?;

    // Update LSP files
    update_lsp_files()?;

    println!("✓ Config created at {:?}", config_path);
    println!("✓ LSP definitions installed at {:?}/lib/oxwm.lua", config_dir);
    println!("  Edit the file and reload with Mod+Shift+R");
    println!("  No compilation needed - changes take effect immediately!");

    Ok(())
}

fn update_lsp_files() -> Result<()> {
    let config_dir = get_config_path();

    // Create lib directory for LSP files
    let lib_dir = config_dir.join("lib");
    std::fs::create_dir_all(&lib_dir)?;

    // Always overwrite oxwm.lua with the version from this binary
    let oxwm_lua_template = include_str!("../../templates/oxwm.lua");
    let oxwm_lua_path = lib_dir.join("oxwm.lua");
    std::fs::write(&oxwm_lua_path, oxwm_lua_template)?;

    // Create/update .luarc.json for LSP configuration
    let luarc_content = r#"{
  "workspace.library": [
    "lib"
  ]
}
"#;
    let luarc_path = config_dir.join(".luarc.json");
    std::fs::write(&luarc_path, luarc_content)?;

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
    println!("    --init              Create default config in ~/.config/oxwm/config.lua");
    println!("    --config <PATH>     Use custom config file");
    println!("    --version           Print version information");
    println!("    --help              Print this help message\n");
    println!("CONFIG:");
    println!("    Location: ~/.config/oxwm/config.lua");
    println!("    Edit the config file and use Mod+Shift+R to reload");
    println!("    No compilation needed - instant hot-reload!");
    println!("    LSP support included with oxwm.lua type definitions\n");
    println!("FIRST RUN:");
    println!("    Run 'oxwm --init' to create a config file");
    println!("    Or just start oxwm and it will create one automatically\n");
}
