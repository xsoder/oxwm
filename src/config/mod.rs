use crate::bar::{BlockCommand, BlockConfig};
use crate::errors::ConfigError;
use crate::keyboard::handlers::Key;
use crate::keyboard::keycodes;
use crate::keyboard::{Arg, KeyAction};
use serde::Deserialize;
use x11rb::protocol::xproto::{KeyButMask, Keycode};

pub fn parse_config(input: &str) -> Result<crate::Config, ConfigError> {
    let config_data: ConfigData = ron::from_str(input)?;
    config_data_to_config(config_data)
}

#[derive(Debug, Deserialize)]
struct ConfigData {
    border_width: u32,
    border_focused: u32,
    border_unfocused: u32,
    font: String,

    gaps_enabled: bool,
    gap_inner_horizontal: u32,
    gap_inner_vertical: u32,
    gap_outer_horizontal: u32,
    gap_outer_vertical: u32,

    terminal: String,
    modkey: String,

    tags: Vec<String>,
    keybindings: Vec<KeybindingData>,
    status_blocks: Vec<StatusBlockData>,

    scheme_normal: ColorSchemeData,
    scheme_occupied: ColorSchemeData,
    scheme_selected: ColorSchemeData,
}

#[derive(Debug, Deserialize)]
struct KeybindingData {
    modifiers: Vec<String>,
    key: String,
    action: String,
    #[serde(default)]
    arg: ArgData,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ArgData {
    None,
    String(String),
    Int(i32),
    Array(Vec<String>),
}

impl Default for ArgData {
    fn default() -> Self {
        ArgData::None
    }
}

#[derive(Debug, Deserialize)]
struct StatusBlockData {
    format: String,
    command: String,
    #[serde(default)]
    command_arg: Option<String>,
    #[serde(default)]
    battery_formats: Option<BatteryFormats>,
    interval_secs: u64,
    color: u32,
    underline: bool,
}

#[derive(Debug, Deserialize)]
struct BatteryFormats {
    charging: String,
    discharging: String,
    full: String,
}

#[derive(Debug, Deserialize)]
struct ColorSchemeData {
    foreground: u32,
    background: u32,
    underline: u32,
}

fn config_data_to_config(data: ConfigData) -> Result<crate::Config, ConfigError> {
    let modkey = parse_modkey(&data.modkey)?;

    let mut keybindings = Vec::new();
    for kb_data in data.keybindings {
        let modifiers = kb_data
            .modifiers
            .iter()
            .map(|s| parse_modkey(s))
            .collect::<Result<Vec<_>, _>>()?;

        let key = string_to_keycode(&kb_data.key)?;
        let action = parse_key_action(&kb_data.action)?;
        let arg = arg_data_to_arg(kb_data.arg)?;

        keybindings.push(Key::new(modifiers, key, action, arg));
    }

    let mut status_blocks = Vec::new();
    for block_data in data.status_blocks {
        let command = match block_data.command.as_str() {
            "DateTime" => {
                let fmt = block_data
                    .command_arg
                    .ok_or_else(|| ConfigError::MissingCommandArg {
                        command: "DateTime".to_string(),
                        field: "command_arg".to_string(),
                    })?;
                BlockCommand::DateTime(fmt)
            }
            "Shell" => {
                let cmd = block_data
                    .command_arg
                    .ok_or_else(|| ConfigError::MissingCommandArg {
                        command: "Shell".to_string(),
                        field: "command_arg".to_string(),
                    })?;
                BlockCommand::Shell(cmd)
            }
            "Ram" => BlockCommand::Ram,
            "Static" => {
                let text = block_data.command_arg.unwrap_or_default();
                BlockCommand::Static(text)
            }
            "Battery" => {
                let formats =
                    block_data
                        .battery_formats
                        .ok_or_else(|| ConfigError::MissingCommandArg {
                            command: "Battery".to_string(),
                            field: "battery_formats".to_string(),
                        })?;
                BlockCommand::Battery {
                    format_charging: formats.charging,
                    format_discharging: formats.discharging,
                    format_full: formats.full,
                }
            }
            _ => return Err(ConfigError::UnknownBlockCommand(block_data.command)),
        };

        status_blocks.push(BlockConfig {
            format: block_data.format,
            command,
            interval_secs: block_data.interval_secs,
            color: block_data.color,
            underline: block_data.underline,
        });
    }

    Ok(crate::Config {
        border_width: data.border_width,
        border_focused: data.border_focused,
        border_unfocused: data.border_unfocused,
        font: data.font,
        gaps_enabled: data.gaps_enabled,
        gap_inner_horizontal: data.gap_inner_horizontal,
        gap_inner_vertical: data.gap_inner_vertical,
        gap_outer_horizontal: data.gap_outer_horizontal,
        gap_outer_vertical: data.gap_outer_vertical,
        terminal: data.terminal,
        modkey,
        tags: data.tags,
        keybindings,
        status_blocks,
        scheme_normal: crate::ColorScheme {
            foreground: data.scheme_normal.foreground,
            background: data.scheme_normal.background,
            underline: data.scheme_normal.underline,
        },
        scheme_occupied: crate::ColorScheme {
            foreground: data.scheme_occupied.foreground,
            background: data.scheme_occupied.background,
            underline: data.scheme_occupied.underline,
        },
        scheme_selected: crate::ColorScheme {
            foreground: data.scheme_selected.foreground,
            background: data.scheme_selected.background,
            underline: data.scheme_selected.underline,
        },
    })
}

fn parse_modkey(s: &str) -> Result<KeyButMask, ConfigError> {
    match s {
        "Mod1" => Ok(KeyButMask::MOD1),
        "Mod2" => Ok(KeyButMask::MOD2),
        "Mod3" => Ok(KeyButMask::MOD3),
        "Mod4" => Ok(KeyButMask::MOD4),
        "Mod5" => Ok(KeyButMask::MOD5),
        "Shift" => Ok(KeyButMask::SHIFT),
        "Control" => Ok(KeyButMask::CONTROL),
        _ => Err(ConfigError::InvalidModkey(s.to_string())),
    }
}

fn string_to_keycode(s: &str) -> Result<Keycode, ConfigError> {
    match s.to_lowercase().as_str() {
        "return" => Ok(keycodes::RETURN),
        "q" => Ok(keycodes::Q),
        "escape" => Ok(keycodes::ESCAPE),
        "space" => Ok(keycodes::SPACE),
        "tab" => Ok(keycodes::TAB),
        "backspace" => Ok(keycodes::BACKSPACE),
        "delete" => Ok(keycodes::DELETE),

        "f1" => Ok(keycodes::F1),
        "f2" => Ok(keycodes::F2),
        "f3" => Ok(keycodes::F3),
        "f4" => Ok(keycodes::F4),
        "f5" => Ok(keycodes::F5),
        "f6" => Ok(keycodes::F6),
        "f7" => Ok(keycodes::F7),
        "f8" => Ok(keycodes::F8),
        "f9" => Ok(keycodes::F9),
        "f10" => Ok(keycodes::F10),
        "f11" => Ok(keycodes::F11),
        "f12" => Ok(keycodes::F12),

        "a" => Ok(keycodes::A),
        "b" => Ok(keycodes::B),
        "c" => Ok(keycodes::C),
        "d" => Ok(keycodes::D),
        "e" => Ok(keycodes::E),
        "f" => Ok(keycodes::F),
        "g" => Ok(keycodes::G),
        "h" => Ok(keycodes::H),
        "i" => Ok(keycodes::I),
        "j" => Ok(keycodes::J),
        "k" => Ok(keycodes::K),
        "l" => Ok(keycodes::L),
        "m" => Ok(keycodes::M),
        "n" => Ok(keycodes::N),
        "o" => Ok(keycodes::O),
        "p" => Ok(keycodes::P),
        "r" => Ok(keycodes::R),
        "s" => Ok(keycodes::S),
        "t" => Ok(keycodes::T),
        "u" => Ok(keycodes::U),
        "v" => Ok(keycodes::V),
        "w" => Ok(keycodes::W),
        "x" => Ok(keycodes::X),
        "y" => Ok(keycodes::Y),
        "z" => Ok(keycodes::Z),

        "0" => Ok(keycodes::KEY_0),
        "1" => Ok(keycodes::KEY_1),
        "2" => Ok(keycodes::KEY_2),
        "3" => Ok(keycodes::KEY_3),
        "4" => Ok(keycodes::KEY_4),
        "5" => Ok(keycodes::KEY_5),
        "6" => Ok(keycodes::KEY_6),
        "7" => Ok(keycodes::KEY_7),
        "8" => Ok(keycodes::KEY_8),
        "9" => Ok(keycodes::KEY_9),

        "left" => Ok(keycodes::LEFT),
        "right" => Ok(keycodes::RIGHT),
        "up" => Ok(keycodes::UP),
        "down" => Ok(keycodes::DOWN),
        "home" => Ok(keycodes::HOME),
        "end" => Ok(keycodes::END),
        "pageup" => Ok(keycodes::PAGE_UP),
        "pagedown" => Ok(keycodes::PAGE_DOWN),
        "insert" => Ok(keycodes::INSERT),

        "minus" | "-" => Ok(keycodes::MINUS),
        "equal" | "=" => Ok(keycodes::EQUAL),
        "bracketleft" | "[" => Ok(keycodes::LEFT_BRACKET),
        "bracketright" | "]" => Ok(keycodes::RIGHT_BRACKET),
        "semicolon" | ";" => Ok(keycodes::SEMICOLON),
        "apostrophe" | "'" => Ok(keycodes::APOSTROPHE),
        "grave" | "`" => Ok(keycodes::GRAVE),
        "backslash" | "\\" => Ok(keycodes::BACKSLASH),
        "comma" | "," => Ok(keycodes::COMMA),
        "period" | "." => Ok(keycodes::PERIOD),
        "slash" | "/" => Ok(keycodes::SLASH),

        _ => Err(ConfigError::UnknownKey(s.to_string())),
    }
}

fn parse_key_action(s: &str) -> Result<crate::keyboard::KeyAction, ConfigError> {
    match s {
        "Spawn" => Ok(KeyAction::Spawn),
        "KillClient" => Ok(KeyAction::KillClient),
        "FocusStack" => Ok(KeyAction::FocusStack),
        "Quit" => Ok(KeyAction::Quit),
        "Restart" => Ok(KeyAction::Restart),
        "ViewTag" => Ok(KeyAction::ViewTag),
        "MoveToTag" => Ok(KeyAction::MoveToTag),
        "ToggleGaps" => Ok(KeyAction::ToggleGaps),
        "ToggleFullScreen" => Ok(KeyAction::ToggleFullScreen),
        "ToggleFloating" => Ok(KeyAction::ToggleFloating),
        _ => Err(ConfigError::UnknownAction(s.to_string())),
    }
}

fn arg_data_to_arg(data: ArgData) -> Result<Arg, ConfigError> {
    match data {
        ArgData::None => Ok(Arg::None),
        ArgData::String(s) => Ok(Arg::Str(s)),
        ArgData::Int(n) => Ok(Arg::Int(n)),
        ArgData::Array(arr) => Ok(Arg::Array(arr)),
    }
}
