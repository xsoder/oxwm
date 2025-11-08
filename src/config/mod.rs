mod lua;

use crate::bar::{BlockCommand, BlockConfig};
use crate::errors::ConfigError;
use crate::keyboard::handlers::{KeyBinding, KeyPress};
use crate::keyboard::keysyms;
use crate::keyboard::{Arg, KeyAction};
use crate::keyboard::keysyms::Keysym;
use serde::Deserialize;
use std::collections::HashMap;
use x11rb::protocol::xproto::KeyButMask;

pub use lua::parse_lua_config;

#[derive(Debug, Deserialize)]
pub enum ModKey {
    Mod,
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
            ModKey::Mod => panic!("ModKey::Mod should be replaced during config parsing"),
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
    AudioRaiseVolume,
    AudioLowerVolume,
    AudioMute,
    MonBrightnessUp,
    MonBrightnessDown,
}

impl KeyData {
    fn to_keysym(&self) -> Keysym {
        match self {
            KeyData::Return => keysyms::XK_RETURN,
            KeyData::Q => keysyms::XK_Q,
            KeyData::Escape => keysyms::XK_ESCAPE,
            KeyData::Space => keysyms::XK_SPACE,
            KeyData::Tab => keysyms::XK_TAB,
            KeyData::Backspace => keysyms::XK_BACKSPACE,
            KeyData::Delete => keysyms::XK_DELETE,
            KeyData::F1 => keysyms::XK_F1,
            KeyData::F2 => keysyms::XK_F2,
            KeyData::F3 => keysyms::XK_F3,
            KeyData::F4 => keysyms::XK_F4,
            KeyData::F5 => keysyms::XK_F5,
            KeyData::F6 => keysyms::XK_F6,
            KeyData::F7 => keysyms::XK_F7,
            KeyData::F8 => keysyms::XK_F8,
            KeyData::F9 => keysyms::XK_F9,
            KeyData::F10 => keysyms::XK_F10,
            KeyData::F11 => keysyms::XK_F11,
            KeyData::F12 => keysyms::XK_F12,
            KeyData::A => keysyms::XK_A,
            KeyData::B => keysyms::XK_B,
            KeyData::C => keysyms::XK_C,
            KeyData::D => keysyms::XK_D,
            KeyData::E => keysyms::XK_E,
            KeyData::F => keysyms::XK_F,
            KeyData::G => keysyms::XK_G,
            KeyData::H => keysyms::XK_H,
            KeyData::I => keysyms::XK_I,
            KeyData::J => keysyms::XK_J,
            KeyData::K => keysyms::XK_K,
            KeyData::L => keysyms::XK_L,
            KeyData::M => keysyms::XK_M,
            KeyData::N => keysyms::XK_N,
            KeyData::O => keysyms::XK_O,
            KeyData::P => keysyms::XK_P,
            KeyData::R => keysyms::XK_R,
            KeyData::S => keysyms::XK_S,
            KeyData::T => keysyms::XK_T,
            KeyData::U => keysyms::XK_U,
            KeyData::V => keysyms::XK_V,
            KeyData::W => keysyms::XK_W,
            KeyData::X => keysyms::XK_X,
            KeyData::Y => keysyms::XK_Y,
            KeyData::Z => keysyms::XK_Z,
            KeyData::Key0 => keysyms::XK_0,
            KeyData::Key1 => keysyms::XK_1,
            KeyData::Key2 => keysyms::XK_2,
            KeyData::Key3 => keysyms::XK_3,
            KeyData::Key4 => keysyms::XK_4,
            KeyData::Key5 => keysyms::XK_5,
            KeyData::Key6 => keysyms::XK_6,
            KeyData::Key7 => keysyms::XK_7,
            KeyData::Key8 => keysyms::XK_8,
            KeyData::Key9 => keysyms::XK_9,
            KeyData::Left => keysyms::XK_LEFT,
            KeyData::Right => keysyms::XK_RIGHT,
            KeyData::Up => keysyms::XK_UP,
            KeyData::Down => keysyms::XK_DOWN,
            KeyData::Home => keysyms::XK_HOME,
            KeyData::End => keysyms::XK_END,
            KeyData::PageUp => keysyms::XK_PAGE_UP,
            KeyData::PageDown => keysyms::XK_PAGE_DOWN,
            KeyData::Insert => keysyms::XK_INSERT,
            KeyData::Minus => keysyms::XK_MINUS,
            KeyData::Equal => keysyms::XK_EQUAL,
            KeyData::BracketLeft => keysyms::XK_LEFT_BRACKET,
            KeyData::BracketRight => keysyms::XK_RIGHT_BRACKET,
            KeyData::Semicolon => keysyms::XK_SEMICOLON,
            KeyData::Apostrophe => keysyms::XK_APOSTROPHE,
            KeyData::Grave => keysyms::XK_GRAVE,
            KeyData::Backslash => keysyms::XK_BACKSLASH,
            KeyData::Comma => keysyms::XK_COMMA,
            KeyData::Period => keysyms::XK_PERIOD,
            KeyData::Slash => keysyms::XK_SLASH,
            KeyData::AudioRaiseVolume => keysyms::XF86_AUDIO_RAISE_VOLUME,
            KeyData::AudioLowerVolume => keysyms::XF86_AUDIO_LOWER_VOLUME,
            KeyData::AudioMute => keysyms::XF86_AUDIO_MUTE,
            KeyData::MonBrightnessUp => keysyms::XF86_MON_BRIGHTNESS_UP,
            KeyData::MonBrightnessDown => keysyms::XF86_MON_BRIGHTNESS_DOWN,
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

    #[serde(default)]
    autostart: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct KeybindingData {
    #[serde(default)]
    keys: Option<Vec<KeyPressData>>,
    #[serde(default)]
    modifiers: Option<Vec<ModKey>>,
    #[serde(default)]
    key: Option<KeyData>,
    action: KeyAction,
    #[serde(default)]
    arg: ArgData,
}

#[derive(Debug, Deserialize)]
struct KeyPressData {
    modifiers: Vec<ModKey>,
    key: KeyData,
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
        let keys = if let Some(keys_data) = kb_data.keys {
            keys_data
                .into_iter()
                .map(|kp| {
                    let modifiers = kp
                        .modifiers
                        .iter()
                        .map(|m| match m {
                            ModKey::Mod => modkey,
                            _ => m.to_keybut_mask(),
                        })
                        .collect();

                    KeyPress {
                        modifiers,
                        keysym: kp.key.to_keysym(),
                    }
                })
                .collect()
        } else if let (Some(modifiers), Some(key)) = (kb_data.modifiers, kb_data.key) {
            vec![KeyPress {
                modifiers: modifiers
                    .iter()
                    .map(|m| match m {
                        ModKey::Mod => modkey,
                        _ => m.to_keybut_mask(),
                    })
                    .collect(),
                keysym: key.to_keysym(),
            }]
        } else {
            return Err(ConfigError::ValidationError(
                "Keybinding must have either 'keys' or 'modifiers'+'key'".to_string(),
            ));
        };

        let action = kb_data.action;
        let arg = arg_data_to_arg(kb_data.arg)?;

        keybindings.push(KeyBinding::new(keys, action, arg));
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
        autostart: data.autostart,
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
