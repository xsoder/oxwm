// Re-export the public interface
pub mod handlers;
pub mod keycodes;

// Re-export commonly used items for convenience
pub use handlers::{Arg, KeyAction, handle_key_press, setup_keybinds}; // Add KeyAction here
pub use keycodes::*;
