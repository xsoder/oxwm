use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("--init") => return init_config(),
        Some("--recompile") => return recompile_config(),
        Some("--version") => {
            println!("oxwm {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        Some("--help") => {
            print_help();
            return Ok(());
        }
        _ => {}
    }

    let cache_binary = get_cache_binary_path();

    if cache_binary.exists() {
        let config_path = get_config_path().join("config.rs");
        if should_recompile(&config_path, &cache_binary)? {
            println!("Config changed, recompiling...");
            recompile_config()?;
        }

        use std::os::unix::process::CommandExt;
        let err = Command::new(&cache_binary).args(&args[1..]).exec();
        anyhow::bail!("Failed to exec user binary: {}", err);
    } else {
        eprintln!("No configuration found.");
        eprintln!("Run: oxwm --init");
        eprintln!("Then add 'exec oxwm' to your ~/.xinitrc");
        std::process::exit(1);
    }
}

fn init_config() -> Result<()> {
    let config_dir = get_config_path();
    std::fs::create_dir_all(&config_dir)?;

    if config_dir.join("config.rs").exists() {
        eprintln!("Config already exists at ~/.config/oxwm/config.rs");
        eprintln!("Remove it first if you want to reinitialize.");
        return Ok(());
    }

    let config_template = include_str!("../../templates/config.rs");
    std::fs::write(config_dir.join("config.rs"), config_template)?;

    let main_template = include_str!("../../templates/main.rs");
    std::fs::write(config_dir.join("main.rs"), main_template)?;

    let cargo_toml = r#"[package]
name = "oxwm-user"
version = "0.1.0"
edition = "2024"

[dependencies]
oxwm = { git = "https://github.com/tonybanters/oxwm" }
anyhow = "1"

[[bin]]
name = "oxwm-user"
path = "main.rs"
"#;

    std::fs::write(config_dir.join("Cargo.toml"), cargo_toml)?;

    std::fs::write(config_dir.join(".gitignore"), "target/\nCargo.lock\n")?;

    println!("✓ Created ~/.config/oxwm/config.rs");
    println!("✓ Created ~/.config/oxwm/main.rs");
    println!("✓ Edit ~/.config/oxwm/config.rs to customize your setup");

    println!("\nCompiling initial configuration...");
    recompile_config()?;

    println!("\n✓ Setup complete!");
    println!("  Add 'exec oxwm' to your ~/.xinitrc");
    println!("  Reload config anytime with Mod+Shift+R");

    Ok(())
}

fn recompile_config() -> Result<()> {
    let config_dir = get_config_path();

    if !config_dir.join("config.rs").exists() {
        anyhow::bail!("No config found. Run: oxwm --init");
    }

    println!("Compiling oxwm configuration...");

    let output = Command::new("cargo")
        .args(&["build", "--release"])
        .current_dir(&config_dir)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("\n❌ Compilation failed:\n{}", stderr);

        let _ = Command::new("notify-send")
            .args(&[
                "-u",
                "critical",
                "OXWM Compile Error",
                "Check terminal for details",
            ])
            .spawn();

        anyhow::bail!("Failed to compile configuration");
    }

    let source = config_dir.join("target/release/oxwm-user");
    let dest = get_cache_binary_path();

    std::fs::create_dir_all(dest.parent().unwrap())?;
    std::fs::copy(&source, &dest)?;

    println!("✓ Compiled successfully");

    let _ = Command::new("notify-send")
        .args(&[
            "OXWM",
            "Configuration recompiled! Hit Mod+Shift+R to restart.",
        ])
        .spawn();

    Ok(())
}

fn should_recompile(config: &PathBuf, binary: &PathBuf) -> Result<bool> {
    if !config.exists() {
        return Ok(false);
    }

    let config_dir = get_config_path();
    let binary_time = std::fs::metadata(binary)?.modified()?;

    let watch_files = ["config.rs", "main.rs", "Cargo.toml"];

    for filename in &watch_files {
        let path = config_dir.join(filename);
        if !path.exists() {
            continue;
        }

        let file_time = std::fs::metadata(&path)?.modified()?;
        if file_time > binary_time {
            return Ok(true);
        }
    }

    Ok(false)
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .expect("Could not find config directory")
        .join("oxwm")
}

fn get_cache_binary_path() -> PathBuf {
    dirs::cache_dir()
        .expect("Could not find cache directory")
        .join("oxwm/oxwm-binary")
}

fn print_help() {
    println!("OXWM - A dynamic window manager written in Rust\n");
    println!("USAGE:");
    println!("    oxwm [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --init         Initialize user configuration in ~/.config/oxwm");
    println!("    --recompile    Recompile user configuration");
    println!("    --version      Print version information");
    println!("    --help         Print this help message\n");
    println!("SETUP:");
    println!("    1. Run 'oxwm --init' to create your config");
    println!("    2. Edit ~/.config/oxwm/config.rs");
    println!("    3. Add 'exec oxwm' to your ~/.xinitrc");
    println!("    4. Start X with 'startx'\n");
    println!("CONFIGURATION:");
    println!("    Config location: ~/.config/oxwm/config.rs");
    println!("    Reload hotkey:   Mod+Shift+R (auto-recompiles if needed)");
}
