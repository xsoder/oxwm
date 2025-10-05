use anyhow::Result;
mod bar;
mod config;
mod keyboard;
mod layout;
mod window_manager;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut window_manager = window_manager::WindowManager::new()?;
    let should_restart = window_manager.run()?;

    drop(window_manager);

    if should_restart {
        use std::os::unix::process::CommandExt;
        let err = std::process::Command::new(&args[0]).args(&args[1..]).exec();
        eprintln!("Failed to restart: {}", err);
    }

    Ok(())
}
