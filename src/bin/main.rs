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
        eprintln!("╔════════════════════════════════════════╗");
        eprintln!("║  OXWM: Running with default config    ║");
        eprintln!("╚════════════════════════════════════════╝");
        eprintln!();
        eprintln!("ℹ️  Run 'oxwm --init' to create a custom config");
        eprintln!();

        let config = oxwm::Config::default();

        let mut wm = oxwm::window_manager::WindowManager::new(config)?;
        let should_restart = wm.run()?;

        drop(wm);

        if should_restart {
            use std::os::unix::process::CommandExt;
            let err = Command::new(&args[0]).args(&args[1..]).exec();
            eprintln!("Failed to restart: {}", err);
        }

        Ok(())
    }
}

#[derive(Debug)]
enum BuildMethod {
    NixFlake,
    NixBuild,
    Cargo,
}

fn detect_build_method() -> BuildMethod {
    let config_dir = get_config_path();

    if config_dir.join("flake.nix").exists() {
        println!("Detected flake.nix, will use nix flake for recompilation");
        return BuildMethod::NixFlake;
    }

    if config_dir.join("default.nix").exists() || config_dir.join("shell.nix").exists() {
        println!("Detected nix files, will use nix-shell for recompilation");
        return BuildMethod::NixBuild;
    }

    println!("Will use cargo for recompilation");
    BuildMethod::Cargo
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

    if is_nixos() {
        let shell_nix = r#"{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    pkg-config
    xorg.libX11
    xorg.libXft
    xorg.libXrender
    freetype
    fontconfig
  ];
}
"#;
        std::fs::write(config_dir.join("shell.nix"), shell_nix)?;
        println!("✓ Created shell.nix for NixOS");
    }

    println!("✓ Created ~/.config/oxwm/config.rs");
    println!("✓ Created ~/.config/oxwm/main.rs");
    println!("✓ Edit ~/.config/oxwm/config.rs to customize your setup");

    println!("\nCompiling initial configuration...");
    recompile_config()?;

    println!("\n✓ Setup complete!");
    println!("  Your custom config is now active");
    println!("  Reload config anytime with Mod+Shift+R");

    Ok(())
}

fn recompile_config() -> Result<()> {
    let config_dir = get_config_path();

    if !config_dir.join("config.rs").exists() {
        anyhow::bail!("No config found. Run: oxwm --init");
    }

    println!("Compiling oxwm configuration...");

    let build_method = detect_build_method();

    let output = match build_method {
        BuildMethod::NixFlake => {
            println!("Using nix flake build...");
            Command::new("nix")
                .args(&["build", ".#", "--no-link"])
                .current_dir(&config_dir)
                .output()?
        }
        BuildMethod::NixBuild => {
            println!("Using nix-shell...");
            Command::new("nix-shell")
                .args(&["--run", "cargo build --release"])
                .current_dir(&config_dir)
                .output()?
        }
        BuildMethod::Cargo => Command::new("cargo")
            .args(&["build", "--release"])
            .current_dir(&config_dir)
            .output()?,
    };

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

    match std::fs::copy(&source, &dest) {
        Ok(_) => {}
        Err(e) if e.raw_os_error() == Some(26) => {
            let temp_dest = dest.with_extension("new");
            std::fs::copy(&source, &temp_dest)?;
            std::fs::rename(&temp_dest, &dest)?;
        }
        Err(e) => return Err(e.into()),
    }

    println!("✓ Compiled successfully");

    let _ = Command::new("notify-send")
        .args(&[
            "OXWM",
            "Configuration recompiled! Hit Mod+Shift+R to restart.",
        ])
        .spawn();

    Ok(())
}

fn is_nixos() -> bool {
    std::path::Path::new("/etc/NIXOS").exists()
        || std::path::Path::new("/run/current-system/nixos-version").exists()
        || std::env::var("NIX_PATH").is_ok()
}

fn should_recompile(config: &PathBuf, binary: &PathBuf) -> Result<bool> {
    if !config.exists() {
        return Ok(false);
    }

    let config_dir = get_config_path();
    let binary_time = std::fs::metadata(binary)?.modified()?;

    let watch_files = [
        "config.rs",
        "main.rs",
        "Cargo.toml",
        "flake.nix",
        "shell.nix",
    ];

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
    println!("CONFIGURATION:");
    println!("    Without --init: Runs with built-in defaults");
    println!("    With --init:    Uses custom config from ~/.config/oxwm/config.rs");
    println!("    Reload hotkey:  Mod+Shift+R (auto-recompiles if needed)\n");
    println!("SETUP:");
    println!("    1. Add 'exec oxwm' to your ~/.xinitrc (works immediately with defaults)");
    println!("    2. Optionally run 'oxwm --init' to create custom config");
    println!("    3. Edit ~/.config/oxwm/config.rs to customize");
    println!("    4. Restart with Mod+Shift+R\n");
    println!("ADVANCED:");
    println!("    Create flake.nix or shell.nix in ~/.config/oxwm to use nix builds");
}
