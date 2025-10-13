use anyhow::Result;
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
            println!("✓ Config created at ~/.config/oxwm/config.rs");
            println!("  Edit and reload with Mod+Shift+R");
            return Ok(());
        }
        Some("--recompile") => {
            recompile_config()?;
            return Ok(());
        }
        _ => {}
    }

    let user_binary = get_user_binary_path();
    let config_path = get_config_path().join("config.rs");

    if config_path.exists() && user_binary.exists() {
        if !should_recompile(&config_path, &user_binary)? {
            use std::os::unix::process::CommandExt;
            let err = std::process::Command::new(&user_binary)
                .args(&args[1..])
                .exec();
            eprintln!("Failed to exec user binary: {}", err);
            std::process::exit(1);
        }
    }

    if !config_path.exists() {
        eprintln!("No config found, creating default at ~/.config/oxwm/config.rs");
        init_config()?;
        eprintln!("✓ Edit ~/.config/oxwm/config.rs and run 'oxwm --recompile'");
    }

    let config = oxwm::Config::default();
    run_wm_with_config(config, &args)?;

    Ok(())
}

fn run_wm_with_config(config: oxwm::Config, args: &[String]) -> Result<()> {
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

fn should_recompile(config: &PathBuf, binary: &PathBuf) -> Result<bool> {
    let config_dir = get_config_path();
    let binary_time = std::fs::metadata(binary)?.modified()?;

    let watch_files = ["config.rs", "Cargo.toml"];

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

fn init_config() -> Result<()> {
    let config_dir = get_config_path();
    std::fs::create_dir_all(&config_dir)?;

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
            std::process::Command::new("nix")
                .args(&["build", ".#", "--no-link"])
                .current_dir(&config_dir)
                .output()?
        }
        BuildMethod::NixBuild => {
            println!("Using nix-shell...");
            std::process::Command::new("nix-shell")
                .args(&["--run", "cargo build --release"])
                .current_dir(&config_dir)
                .output()?
        }
        BuildMethod::Cargo => std::process::Command::new("cargo")
            .args(&["build", "--release"])
            .current_dir(&config_dir)
            .output()?,
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("\n❌ Compilation failed:\n{}", stderr);
        anyhow::bail!("Failed to compile configuration");
    }

    let source = config_dir.join("target/release/oxwm-user");
    let dest = get_user_binary_path();

    std::fs::create_dir_all(dest.parent().unwrap())?;
    std::fs::copy(&source, &dest)?;

    println!("✓ Compiled successfully to {}", dest.display());
    println!("  Restart oxwm to use new config");

    Ok(())
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

fn is_nixos() -> bool {
    std::path::Path::new("/etc/NIXOS").exists()
        || std::path::Path::new("/run/current-system/nixos-version").exists()
        || std::env::var("NIX_PATH").is_ok()
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .expect("Could not find config directory")
        .join("oxwm")
}

fn get_user_binary_path() -> PathBuf {
    get_config_path().join("oxwm-user")
}

fn print_help() {
    println!("OXWM - A dynamic window manager written in Rust\n");
    println!("USAGE:");
    println!("    oxwm [OPTIONS]\n");
    println!("OPTIONS:");
    println!("    --init         Create default config in ~/.config/oxwm");
    println!("    --recompile    Recompile user configuration");
    println!("    --version      Print version information");
    println!("    --help         Print this help message\n");
    println!("CONFIG:");
    println!("    First run: Creates config at ~/.config/oxwm/config.rs");
    println!("    Edit your config and run 'oxwm --recompile'");
    println!("    Use Mod+Shift+R to hot-reload after recompiling\n");
}
