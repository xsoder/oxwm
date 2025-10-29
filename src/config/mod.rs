use crate::bar::{BlockCommand, BlockConfig};
use crate::errors::ConfigError;
use crate::keyboard::handlers::Key;
use crate::keyboard::keycodes;
use crate::keyboard::{Arg, KeyAction};
use serde::Deserialize;
use std::collections::HashMap;
use x11rb::protocol::xproto::{KeyButMask, Keycode};

#[derive(Debug, Deserialize)]
pub enum ModKey {
    Mod1,
    Mod2,
    Mod3,
    Mod4,
    Mod5,
    Shift,
    Control,
}

impl ModKey {
    fn to_keybut_mask(&self) -> KeyButMask {
        match self {
            ModKey::Mod1 => KeyButMask::MOD1,
            ModKey::Mod2 => KeyButMask::MOD2,
            ModKey::Mod3 => KeyButMask::MOD3,
            ModKey::Mod4 => KeyButMask::MOD4,
            ModKey::Mod5 => KeyButMask::MOD5,
            ModKey::Shift => KeyButMask::SHIFT,
            ModKey::Control => KeyButMask::CONTROL,
        }
    }
}

#[rustfmt::skip]
#[derive(Debug, Deserialize)]
pub enum KeyData {
    Return,
    Q,
    Escape,
    Space,
    Tab,
    Backspace,
    Delete,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, R, S, T, U, V, W, X, Y, Z,
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    Minus,
    Equal,
    BracketLeft,
    BracketRight,
    Semicolon,
    Apostrophe,
    Grave,
    Backslash,
    Comma,
    Period,
    Slash,
}

impl KeyData {
    fn to_keycode(&self) -> Keycode {
        match self {
            KeyData::Return => keycodes::RETURN,
            KeyData::Q => keycodes::Q,
            KeyData::Escape => keycodes::ESCAPE,
            KeyData::Space => keycodes::SPACE,
            KeyData::Tab => keycodes::TAB,
            KeyData::Backspace => keycodes::BACKSPACE,
            KeyData::Delete => keycodes::DELETE,
            KeyData::F1 => keycodes::F1,
            KeyData::F2 => keycodes::F2,
            KeyData::F3 => keycodes::F3,
            KeyData::F4 => keycodes::F4,
            KeyData::F5 => keycodes::F5,
            KeyData::F6 => keycodes::F6,
            KeyData::F7 => keycodes::F7,
            KeyData::F8 => keycodes::F8,
            KeyData::F9 => keycodes::F9,
            KeyData::F10 => keycodes::F10,
            KeyData::F11 => keycodes::F11,
            KeyData::F12 => keycodes::F12,
            KeyData::A => keycodes::A,
            KeyData::B => keycodes::B,
            KeyData::C => keycodes::C,
            KeyData::D => keycodes::D,
            KeyData::E => keycodes::E,
            KeyData::F => keycodes::F,
            KeyData::G => keycodes::G,
            KeyData::H => keycodes::H,
            KeyData::I => keycodes::I,
            KeyData::J => keycodes::J,
            KeyData::K => keycodes::K,
            KeyData::L => keycodes::L,
            KeyData::M => keycodes::M,
            KeyData::N => keycodes::N,
            KeyData::O => keycodes::O,
            KeyData::P => keycodes::P,
            KeyData::R => keycodes::R,
            KeyData::S => keycodes::S,
            KeyData::T => keycodes::T,
            KeyData::U => keycodes::U,
            KeyData::V => keycodes::V,
            KeyData::W => keycodes::W,
            KeyData::X => keycodes::X,
            KeyData::Y => keycodes::Y,
            KeyData::Z => keycodes::Z,
            KeyData::Key0 => keycodes::KEY_0,
            KeyData::Key1 => keycodes::KEY_1,
            KeyData::Key2 => keycodes::KEY_2,
            KeyData::Key3 => keycodes::KEY_3,
            KeyData::Key4 => keycodes::KEY_4,
            KeyData::Key5 => keycodes::KEY_5,
            KeyData::Key6 => keycodes::KEY_6,
            KeyData::Key7 => keycodes::KEY_7,
            KeyData::Key8 => keycodes::KEY_8,
            KeyData::Key9 => keycodes::KEY_9,
            KeyData::Left => keycodes::LEFT,
            KeyData::Right => keycodes::RIGHT,
            KeyData::Up => keycodes::UP,
            KeyData::Down => keycodes::DOWN,
            KeyData::Home => keycodes::HOME,
            KeyData::End => keycodes::END,
            KeyData::PageUp => keycodes::PAGE_UP,
            KeyData::PageDown => keycodes::PAGE_DOWN,
            KeyData::Insert => keycodes::INSERT,
            KeyData::Minus => keycodes::MINUS,
            KeyData::Equal => keycodes::EQUAL,
            KeyData::BracketLeft => keycodes::LEFT_BRACKET,
            KeyData::BracketRight => keycodes::RIGHT_BRACKET,
            KeyData::Semicolon => keycodes::SEMICOLON,
            KeyData::Apostrophe => keycodes::APOSTROPHE,
            KeyData::Grave => keycodes::GRAVE,
            KeyData::Backslash => keycodes::BACKSLASH,
            KeyData::Comma => keycodes::COMMA,
            KeyData::Period => keycodes::PERIOD,
            KeyData::Slash => keycodes::SLASH,
        }
    }
}

fn preprocess_variables(input: &str) -> Result<String, ConfigError> {
    let mut variables: HashMap<String, String> = HashMap::new();
    let mut result = String::new();

    for line in input.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("#DEFINE") {
            let rest = trimmed.strip_prefix("#DEFINE").unwrap().trim();

            if let Some(eq_pos) = rest.find('=') {
                let var_name = rest[..eq_pos].trim();
                let value = rest[eq_pos + 1..].trim().trim_end_matches(',');

                if !var_name.starts_with('$') {
                    return Err(ConfigError::InvalidVariableName(var_name.to_string()));
                }

                variables.insert(var_name.to_string(), value.to_string());
            } else {
                return Err(ConfigError::InvalidDefine(trimmed.to_string()));
            }

            result.push('\n');
        } else {
            let mut processed_line = line.to_string();
            for (var_name, value) in &variables {
                processed_line = processed_line.replace(var_name, value);
            }
            result.push_str(&processed_line);
            result.push('\n');
        }
    }

    for line in result.lines() {
        if let Some(var_start) = line.find('$') {
            let rest = &line[var_start..];
            let var_end = rest[1..]
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(rest.len() - 1)
                + 1;
            let undefined_var = &rest[..var_end];
            return Err(ConfigError::UndefinedVariable(undefined_var.to_string()));
        }
    }
    Ok(result)
}

pub fn parse_config(input: &str) -> Result<crate::Config, ConfigError> {
    let preprocessed = preprocess_variables(input)?;
    let config_data: ConfigData = ron::from_str(&preprocessed)?;
    config_data_to_config(config_data)
}

#[derive(Debug, Deserialize)]
struct LayoutSymbolOverrideData {
    name: String,
    symbol: String,
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
    modkey: ModKey,

    tags: Vec<String>,
    #[serde(default)]
    layout_symbols: Vec<LayoutSymbolOverrideData>,
    keybindings: Vec<KeybindingData>,
    status_blocks: Vec<StatusBlockData>,

    scheme_normal: ColorSchemeData,
    scheme_occupied: ColorSchemeData,
    scheme_selected: ColorSchemeData,
}

#[derive(Debug, Deserialize)]
struct KeybindingData {
    modifiers: Vec<ModKey>,
    key: KeyData,
    action: KeyAction,
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
    let modkey = data.modkey.to_keybut_mask();

    let mut keybindings = Vec::new();
    for kb_data in data.keybindings {
        let modifiers = kb_data
            .modifiers
            .iter()
            .map(|m| m.to_keybut_mask())
            .collect();

        let key = kb_data.key.to_keycode();
        let action = kb_data.action;
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

    let layout_symbols = data
        .layout_symbols
        .into_iter()
        .map(|l| crate::LayoutSymbolOverride {
            name: l.name,
            symbol: l.symbol,
        })
        .collect();

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
        layout_symbols,
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

fn arg_data_to_arg(data: ArgData) -> Result<Arg, ConfigError> {
    match data {
        ArgData::None => Ok(Arg::None),
        ArgData::String(s) => Ok(Arg::Str(s)),
        ArgData::Int(n) => Ok(Arg::Int(n)),
        ArgData::Array(arr) => Ok(Arg::Array(arr)),
    }
}
