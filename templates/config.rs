use oxwm::ColorScheme;
use oxwm::prelude::*;

// ========================================
// APPEARANCE
// ========================================
pub const BORDER_WIDTH: u32 = 2;
pub const BORDER_FOCUSED: u32 = 0x6dade3;
pub const BORDER_UNFOCUSED: u32 = 0xbbbbbb;
pub const FONT: &str = "monospace:size=12";

// ========================================
// GAPS (Vanity Gaps)
// ========================================
pub const GAPS_ENABLED: bool = true;
pub const GAP_INNER_HORIZONTAL: u32 = 6;
pub const GAP_INNER_VERTICAL: u32 = 6;
pub const GAP_OUTER_HORIZONTAL: u32 = 6;
pub const GAP_OUTER_VERTICAL: u32 = 6;

// ========================================
// DEFAULTS
// ========================================
pub const TERMINAL: &str = "st";
pub const MODKEY: KeyButMask = KeyButMask::MOD4;

// ========================================
// TAGS (Workspaces)
// ========================================
pub const TAG_COUNT: usize = 9;
pub const TAGS: [&str; TAG_COUNT] = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];

// ========================================
// BAR COLORS
// ========================================
const GRAY_DARK: u32 = 0x1a1b26;
const GRAY_MID: u32 = 0x444444;
const GRAY_LIGHT: u32 = 0xbbbbbb;
const CYAN: u32 = 0x0db9d7;
const MAGENTA: u32 = 0xad8ee6;

pub const SCHEME_NORMAL: ColorScheme = ColorScheme {
    foreground: GRAY_LIGHT,
    background: GRAY_DARK,
    underline: GRAY_MID,
};

pub const SCHEME_OCCUPIED: ColorScheme = ColorScheme {
    foreground: CYAN,
    background: GRAY_DARK,
    underline: CYAN,
};

pub const SCHEME_SELECTED: ColorScheme = ColorScheme {
    foreground: CYAN,
    background: GRAY_DARK,
    underline: MAGENTA,
};

// ========================================
// KEYBINDINGS
// ========================================
const SHIFT: KeyButMask = KeyButMask::SHIFT;

#[rustfmt::skip]
pub const KEYBINDINGS: &[Key] = &[
    // Launch applications
    Key::new(&[MODKEY],        keycodes::RETURN, KeyAction::Spawn,      Arg::Str(TERMINAL)),
    Key::new(&[MODKEY],        keycodes::D,      KeyAction::Spawn,      Arg::Array(&["sh", "-c", "dmenu_run"])),
    
    // Window management
    Key::new(&[MODKEY],        keycodes::Q,      KeyAction::KillClient, Arg::None),
    Key::new(&[MODKEY, SHIFT], keycodes::F,      KeyAction::ToggleFullScreen, Arg::None),
    Key::new(&[MODKEY],        keycodes::A,      KeyAction::ToggleGaps, Arg::None),
    
    // WM controls
    Key::new(&[MODKEY, SHIFT], keycodes::Q,      KeyAction::Quit,       Arg::None),
    Key::new(&[MODKEY, SHIFT], keycodes::R,      KeyAction::Restart,    Arg::None),
    
    // Focus
    Key::new(&[MODKEY],        keycodes::J,      KeyAction::FocusStack, Arg::Int(-1)),
    Key::new(&[MODKEY],        keycodes::K,      KeyAction::FocusStack, Arg::Int(1)),
    
    // View tags (workspaces)
    Key::new(&[MODKEY], keycodes::KEY_1, KeyAction::ViewTag, Arg::Int(0)),
    Key::new(&[MODKEY], keycodes::KEY_2, KeyAction::ViewTag, Arg::Int(1)),
    Key::new(&[MODKEY], keycodes::KEY_3, KeyAction::ViewTag, Arg::Int(2)),
    Key::new(&[MODKEY], keycodes::KEY_4, KeyAction::ViewTag, Arg::Int(3)),
    Key::new(&[MODKEY], keycodes::KEY_5, KeyAction::ViewTag, Arg::Int(4)),
    Key::new(&[MODKEY], keycodes::KEY_6, KeyAction::ViewTag, Arg::Int(5)),
    Key::new(&[MODKEY], keycodes::KEY_7, KeyAction::ViewTag, Arg::Int(6)),
    Key::new(&[MODKEY], keycodes::KEY_8, KeyAction::ViewTag, Arg::Int(7)),
    Key::new(&[MODKEY], keycodes::KEY_9, KeyAction::ViewTag, Arg::Int(8)),
    
    // Move windows to tags
    Key::new(&[MODKEY, SHIFT], keycodes::KEY_1, KeyAction::MoveToTag, Arg::Int(0)),
    Key::new(&[MODKEY, SHIFT], keycodes::KEY_2, KeyAction::MoveToTag, Arg::Int(1)),
    Key::new(&[MODKEY, SHIFT], keycodes::KEY_3, KeyAction::MoveToTag, Arg::Int(2)),
    Key::new(&[MODKEY, SHIFT], keycodes::KEY_4, KeyAction::MoveToTag, Arg::Int(3)),
    Key::new(&[MODKEY, SHIFT], keycodes::KEY_5, KeyAction::MoveToTag, Arg::Int(4)),
    Key::new(&[MODKEY, SHIFT], keycodes::KEY_6, KeyAction::MoveToTag, Arg::Int(5)),
    Key::new(&[MODKEY, SHIFT], keycodes::KEY_7, KeyAction::MoveToTag, Arg::Int(6)),
    Key::new(&[MODKEY, SHIFT], keycodes::KEY_8, KeyAction::MoveToTag, Arg::Int(7)),
    Key::new(&[MODKEY, SHIFT], keycodes::KEY_9, KeyAction::MoveToTag, Arg::Int(8)),
];

// ========================================
// STATUS BAR BLOCKS
// ========================================
pub const STATUS_BLOCKS: &[BlockConfig] = &[BlockConfig {
    format: "{}",
    command: BlockCommand::DateTime("%a, %b %d - %-I:%M %P"),
    interval_secs: 1,
    color: CYAN,
    underline: true,
}];

// ========================================
// BUILD CONFIG FROM CONSTANTS
// ========================================
pub fn build_config() -> oxwm::Config {
    oxwm::Config {
        border_width: BORDER_WIDTH,
        border_focused: BORDER_FOCUSED,
        border_unfocused: BORDER_UNFOCUSED,
        font: FONT.to_string(),
        gaps_enabled: GAPS_ENABLED,
        gap_inner_horizontal: GAP_INNER_HORIZONTAL,
        gap_inner_vertical: GAP_INNER_VERTICAL,
        gap_outer_horizontal: GAP_OUTER_HORIZONTAL,
        gap_outer_vertical: GAP_OUTER_VERTICAL,
        terminal: TERMINAL.to_string(),
        modkey: MODKEY,
        tags: TAGS.iter().map(|s| s.to_string()).collect(),
        keybindings: KEYBINDINGS.to_vec(),
        status_blocks: STATUS_BLOCKS.to_vec(),
        scheme_normal: SCHEME_NORMAL,
        scheme_occupied: SCHEME_OCCUPIED,
        scheme_selected: SCHEME_SELECTED,
    }
}
