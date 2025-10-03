use crate::keyboard::handlers::Key;
use crate::keyboard::{Arg, KeyAction, keycodes};
use x11rb::protocol::xproto::KeyButMask;

// ========================================
// APPEARANCE
// ========================================
pub const BORDER_WIDTH: u32 = 1;
pub const BORDER_FOCUSED: u32 = 0x6dade3;
pub const BORDER_UNFOCUSED: u32 = 0xbbbbbb;
pub const FONT: &str = "JetBrainsMono Nerd Font:style=Bold:size=12";

// ========================================
// DEFAULTS
// ========================================
pub const TERMINAL: &str = "alacritty";
pub const MODKEY: KeyButMask = KeyButMask::MOD1;

// ========================================
// BAR COLORS
// ========================================

// Base colors
const GRAY_DARK: u32 = 0x222222;
const GRAY_MID: u32 = 0x444444;
const GRAY_LIGHT: u32 = 0xbbbbbb;
// const GRAY_LIGHTEST: u32 = 0xeeeeee;
const CYAN: u32 = 0x6dade3;
const MAGENTA: u32 = 0xad8ee6;

pub struct ColorScheme {
    pub foreground: u32,
    pub background: u32,
    pub border: u32,
}

pub const SCHEME_NORMAL: ColorScheme = ColorScheme {
    foreground: GRAY_LIGHT,
    background: GRAY_DARK,
    border: GRAY_MID,
};

pub const SCHEME_OCCUPIED: ColorScheme = ColorScheme {
    foreground: CYAN,
    background: GRAY_DARK,
    border: CYAN,
};

pub const SCHEME_SELECTED: ColorScheme = ColorScheme {
    foreground: CYAN,
    background: GRAY_DARK,
    border: MAGENTA,
};

// ========================================
// Commands
// ========================================
const SCREENSHOT_CMD: &[&str] = &[
    "sh",
    "-c",
    "maim ~/screenshots/screenshot_$(date +%Y%m%d_%H%M%S).png",
];

const DMENU_CMD: &[&str] = &["sh", "-c", "dmenu_run -l 10"];

// ========================================
// TAGS
// ========================================
pub const TAG_COUNT: usize = 9;
pub const TAGS: [&str; TAG_COUNT] = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];

// ========================================
// KEYBINDINGS
// ========================================
#[rustfmt::skip]
pub const KEYBINDINGS: &[Key] = &[
    Key::new(&[MODKEY],        keycodes::RETURN, KeyAction::Spawn,      Arg::Str(TERMINAL)),

    Key::new(&[MODKEY],        keycodes::S,      KeyAction::Spawn,      Arg::Array(SCREENSHOT_CMD)),
    Key::new(&[MODKEY],        keycodes::D,      KeyAction::Spawn,      Arg::Array(DMENU_CMD)),
    Key::new(&[MODKEY],        keycodes::Q,      KeyAction::KillClient, Arg::None),
    Key::new(&[MODKEY, SHIFT], keycodes::Q,      KeyAction::Quit,       Arg::None),
    Key::new(&[MODKEY],        keycodes::J,      KeyAction::FocusStack, Arg::Int(-1)),
    Key::new(&[MODKEY],        keycodes::K,      KeyAction::FocusStack, Arg::Int(1)),
    
    Key::new(&[MODKEY], keycodes::KEY_1, KeyAction::ViewTag, Arg::Int(0)),
    Key::new(&[MODKEY], keycodes::KEY_2, KeyAction::ViewTag, Arg::Int(1)),
    Key::new(&[MODKEY], keycodes::KEY_3, KeyAction::ViewTag, Arg::Int(2)),
    Key::new(&[MODKEY], keycodes::KEY_4, KeyAction::ViewTag, Arg::Int(3)),
    Key::new(&[MODKEY], keycodes::KEY_5, KeyAction::ViewTag, Arg::Int(4)),
    Key::new(&[MODKEY], keycodes::KEY_6, KeyAction::ViewTag, Arg::Int(5)),
    Key::new(&[MODKEY], keycodes::KEY_7, KeyAction::ViewTag, Arg::Int(6)),
    Key::new(&[MODKEY], keycodes::KEY_8, KeyAction::ViewTag, Arg::Int(7)),
    Key::new(&[MODKEY], keycodes::KEY_9, KeyAction::ViewTag, Arg::Int(8)),
    
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

const SHIFT: KeyButMask = KeyButMask::SHIFT;
