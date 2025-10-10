pub mod bar;
pub mod keyboard;
pub mod layout;
pub mod window_manager;

pub mod prelude {
    pub use crate::ColorScheme;
    pub use crate::bar::{BlockCommand, BlockConfig};
    pub use crate::keyboard::{Arg, KeyAction, handlers::Key, keycodes};
    pub use x11rb::protocol::xproto::KeyButMask;
}

#[derive(Clone)]
pub struct Config {
    // Appearance
    pub border_width: u32,
    pub border_focused: u32,
    pub border_unfocused: u32,
    pub font: String,

    // Gaps
    pub gaps_enabled: bool,
    pub gap_inner_horizontal: u32,
    pub gap_inner_vertical: u32,
    pub gap_outer_horizontal: u32,
    pub gap_outer_vertical: u32,

    // Basics
    pub terminal: String,
    pub modkey: x11rb::protocol::xproto::KeyButMask,

    // Tags
    pub tags: Vec<String>,

    // Keybindings
    pub keybindings: Vec<crate::keyboard::handlers::Key>,

    // Status bar
    pub status_blocks: Vec<crate::bar::BlockConfig>,

    // Bar color schemes
    pub scheme_normal: ColorScheme,
    pub scheme_occupied: ColorScheme,
    pub scheme_selected: ColorScheme,
}

#[derive(Clone, Copy)]
pub struct ColorScheme {
    pub foreground: u32,
    pub background: u32,
    pub underline: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            border_width: 2,
            border_focused: 0x6dade3,
            border_unfocused: 0xbbbbbb,
            font: "monospace:size=12".to_string(),
            gaps_enabled: false,
            gap_inner_horizontal: 0,
            gap_inner_vertical: 0,
            gap_outer_horizontal: 0,
            gap_outer_vertical: 0,
            terminal: "xterm".to_string(),
            modkey: x11rb::protocol::xproto::KeyButMask::MOD4,
            tags: vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"]
                .into_iter()
                .map(String::from)
                .collect(),
            keybindings: Vec::new(),
            status_blocks: Vec::new(),
            scheme_normal: ColorScheme {
                foreground: 0xbbbbbb,
                background: 0x1a1b26,
                underline: 0x444444,
            },
            scheme_occupied: ColorScheme {
                foreground: 0x0db9d7,
                background: 0x1a1b26,
                underline: 0x0db9d7,
            },
            scheme_selected: ColorScheme {
                foreground: 0x0db9d7,
                background: 0x1a1b26,
                underline: 0xad8ee6,
            },
        }
    }
}
