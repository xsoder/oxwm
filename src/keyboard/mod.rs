// Re-export the public interface
pub mod handlers;
pub mod keycodes;

// Re-export commonly used items for convenience
pub use handlers::{handle_key_press, setup_keybinds};
pub use keycodes::*;
