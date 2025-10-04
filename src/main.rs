use anyhow::Result;
mod bar;
mod config;
mod keyboard;
mod layout;
mod window_manager;

fn main() -> Result<()> {
    loop {
        let mut window_manager = window_manager::WindowManager::new()?;
        let should_restart = window_manager.run()?;

        if !should_restart {
            break;
        }

        use std::os::unix::process::CommandExt;
        let err = std::process::Command::new(config::WM_BINARY).exec();
        eprintln!("Failed to restart: {}", err);
        break;
    }

    Ok(())
}
