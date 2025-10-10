mod config;

use anyhow::Result;

fn main() -> Result<()> {
    let cfg = config::build_config();

    let args: Vec<String> = std::env::args().collect();
    let mut wm = oxwm::window_manager::WindowManager::new(cfg)?;
    let should_restart = wm.run()?;

    drop(wm);

    if should_restart {
        use std::os::unix::process::CommandExt;
        let err = std::process::Command::new(&args[0]).args(&args[1..]).exec();
        eprintln!("Failed to restart: {}", err);
    }

    Ok(())
}
