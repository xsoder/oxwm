use anyhow::Result;
mod layout;
mod window_manager;

fn main() -> Result<()> {
    let mut window_manager = window_manager::WindowManager::new()?;
    return window_manager.run();
}
