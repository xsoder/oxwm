use crate::bar::{BlockCommand, BlockConfig};
use crate::errors::ConfigError;
use crate::keyboard::handlers::{KeyBinding, KeyPress};
use crate::keyboard::keysyms::{self, Keysym};
use crate::keyboard::{Arg, KeyAction};
use crate::{ColorScheme, LayoutSymbolOverride};
use mlua::{Lua, Table, Value};
use x11rb::protocol::xproto::KeyButMask;

pub fn parse_lua_config(input: &str) -> Result<crate::Config, ConfigError> {
    let lua = Lua::new();

    let config: Table = lua.load(input)
        .eval()
        .map_err(|e| ConfigError::LuaError(format!("Failed to execute Lua config: {}", e)))?;
    let border_width: u32 = get_table_field(&config, "border_width")?;
    let border_focused: u32 = parse_color(&config, "border_focused")?;
    let border_unfocused: u32 = parse_color(&config, "border_unfocused")?;
    let font: String = get_table_field(&config, "font")?;

    let gaps_enabled: bool = get_table_field(&config, "gaps_enabled")?;
    let gap_inner_horizontal: u32 = get_table_field(&config, "gap_inner_horizontal")?;
    let gap_inner_vertical: u32 = get_table_field(&config, "gap_inner_vertical")?;
    let gap_outer_horizontal: u32 = get_table_field(&config, "gap_outer_horizontal")?;
    let gap_outer_vertical: u32 = get_table_field(&config, "gap_outer_vertical")?;

    let terminal: String = get_table_field(&config, "terminal")?;
    let modkey = parse_modkey(&config)?;

    let tags = parse_tags(&config)?;
    let layout_symbols = parse_layout_symbols(&config)?;
    let keybindings = parse_keybindings(&config, modkey)?;
    let status_blocks = parse_status_blocks(&config)?;

    let scheme_normal = parse_color_scheme(&config, "scheme_normal")?;
    let scheme_occupied = parse_color_scheme(&config, "scheme_occupied")?;
    let scheme_selected = parse_color_scheme(&config, "scheme_selected")?;

    let autostart = parse_autostart(&config)?;

    Ok(crate::Config {
        border_width,
        border_focused,
        border_unfocused,
        font,
        gaps_enabled,
        gap_inner_horizontal,
        gap_inner_vertical,
        gap_outer_horizontal,
        gap_outer_vertical,
        terminal,
        modkey,
        tags,
        layout_symbols,
        keybindings,
        status_blocks,
        scheme_normal,
        scheme_occupied,
        scheme_selected,
        autostart,
    })
}

fn get_table_field<T>(table: &Table, field: &str) -> Result<T, ConfigError>
where
    T: mlua::FromLua,
{
    table
        .get::<T>(field)
        .map_err(|e| ConfigError::LuaError(format!("Failed to get field '{}': {}", field, e)))
}

fn parse_color(table: &Table, field: &str) -> Result<u32, ConfigError> {
    let value: Value = table
        .get(field)
        .map_err(|e| ConfigError::LuaError(format!("Failed to get color field '{}': {}", field, e)))?;

    match value {
        Value::String(s) => {
            let s = s.to_str().map_err(|e| {
                ConfigError::LuaError(format!("Invalid UTF-8 in color string: {}", e))
            })?;
            parse_color_string(&s)
        }
        Value::Integer(i) => Ok(i as u32),
        Value::Number(n) => Ok(n as u32),
        _ => Err(ConfigError::LuaError(format!(
            "Color field '{}' must be a string or number",
            field
        ))),
    }
}

fn parse_color_string(s: &str) -> Result<u32, ConfigError> {
    let s = s.trim();
    if s.starts_with('#') {
        u32::from_str_radix(&s[1..], 16)
            .map_err(|e| ConfigError::LuaError(format!("Invalid hex color '{}': {}", s, e)))
    } else if s.starts_with("0x") {
        u32::from_str_radix(&s[2..], 16)
            .map_err(|e| ConfigError::LuaError(format!("Invalid hex color '{}': {}", s, e)))
    } else {
        s.parse::<u32>()
            .map_err(|e| ConfigError::LuaError(format!("Invalid color '{}': {}", s, e)))
    }
}

fn parse_modkey(config: &Table) -> Result<KeyButMask, ConfigError> {
    let modkey_str: String = get_table_field(config, "modkey")?;
    parse_modkey_string(&modkey_str)
}

fn parse_modkey_string(s: &str) -> Result<KeyButMask, ConfigError> {
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

fn parse_tags(config: &Table) -> Result<Vec<String>, ConfigError> {
    let tags_table: Table = get_table_field(config, "tags")?;
    let mut tags = Vec::new();

    for i in 1..=tags_table.len().map_err(|e| {
        ConfigError::LuaError(format!("Failed to get tags length: {}", e))
    })? {
        let tag: String = tags_table.get(i).map_err(|e| {
            ConfigError::LuaError(format!("Failed to get tag at index {}: {}", i, e))
        })?;
        tags.push(tag);
    }

    Ok(tags)
}

fn parse_layout_symbols(config: &Table) -> Result<Vec<LayoutSymbolOverride>, ConfigError> {
    let layout_symbols_result: Result<Table, _> = config.get("layout_symbols");

    match layout_symbols_result {
        Ok(layout_symbols_table) => {
            let mut layout_symbols = Vec::new();

            for i in 1..=layout_symbols_table.len().map_err(|e| {
                ConfigError::LuaError(format!("Failed to get layout_symbols length: {}", e))
            })? {
                let entry: Table = layout_symbols_table.get(i).map_err(|e| {
                    ConfigError::LuaError(format!("Failed to get layout_symbol at index {}: {}", i, e))
                })?;

                let name: String = get_table_field(&entry, "name")?;
                let symbol: String = get_table_field(&entry, "symbol")?;

                layout_symbols.push(LayoutSymbolOverride { name, symbol });
            }

            Ok(layout_symbols)
        }
        Err(_) => Ok(Vec::new()), // layout_symbols is optional
    }
}

fn parse_keybindings(
    config: &Table,
    modkey: KeyButMask,
) -> Result<Vec<KeyBinding>, ConfigError> {
    let keybindings_table: Table = get_table_field(config, "keybindings")?;
    let mut keybindings = Vec::new();

    for i in 1..=keybindings_table.len().map_err(|e| {
        ConfigError::LuaError(format!("Failed to get keybindings length: {}", e))
    })? {
        let kb_table: Table = keybindings_table.get(i).map_err(|e| {
            ConfigError::LuaError(format!("Failed to get keybinding at index {}: {}", i, e))
        })?;

        let keys = parse_keypress_list(&kb_table, modkey)?;
        let action = parse_key_action(&kb_table)?;
        let arg = parse_arg(&kb_table)?;

        keybindings.push(KeyBinding::new(keys, action, arg));
    }

    Ok(keybindings)
}

fn parse_keypress_list(
    kb_table: &Table,
    modkey: KeyButMask,
) -> Result<Vec<KeyPress>, ConfigError> {
    // Check if 'keys' field exists (for keychords)
    let keys_result: Result<Table, _> = kb_table.get("keys");

    if let Ok(keys_table) = keys_result {
        // Parse keychord
        let mut keys = Vec::new();
        for i in 1..=keys_table.len().map_err(|e| {
            ConfigError::LuaError(format!("Failed to get keys length: {}", e))
        })? {
            let key_entry: Table = keys_table.get(i).map_err(|e| {
                ConfigError::LuaError(format!("Failed to get key at index {}: {}", i, e))
            })?;

            let modifiers = parse_modifiers(&key_entry, "modifiers", modkey)?;
            let keysym = parse_keysym(&key_entry, "key")?;

            keys.push(KeyPress { modifiers, keysym });
        }
        Ok(keys)
    } else {
        // Parse single key (old format)
        let modifiers = parse_modifiers(kb_table, "modifiers", modkey)?;
        let keysym = parse_keysym(kb_table, "key")?;

        Ok(vec![KeyPress { modifiers, keysym }])
    }
}

fn parse_modifiers(
    table: &Table,
    field: &str,
    modkey: KeyButMask,
) -> Result<Vec<KeyButMask>, ConfigError> {
    let mods_table: Table = get_table_field(table, field)?;
    let mut modifiers = Vec::new();

    for i in 1..=mods_table.len().map_err(|e| {
        ConfigError::LuaError(format!("Failed to get modifiers length: {}", e))
    })? {
        let mod_str: String = mods_table.get(i).map_err(|e| {
            ConfigError::LuaError(format!("Failed to get modifier at index {}: {}", i, e))
        })?;

        let modifier = if mod_str == "Mod" {
            modkey
        } else {
            parse_modkey_string(&mod_str)?
        };

        modifiers.push(modifier);
    }

    Ok(modifiers)
}

fn parse_keysym(table: &Table, field: &str) -> Result<Keysym, ConfigError> {
    let key_str: String = get_table_field(table, field)?;
    string_to_keysym(&key_str)
}

fn string_to_keysym(s: &str) -> Result<Keysym, ConfigError> {
    let keysym = match s {
        "Return" => keysyms::XK_RETURN,
        "Q" => keysyms::XK_Q,
        "Escape" => keysyms::XK_ESCAPE,
        "Space" => keysyms::XK_SPACE,
        "Tab" => keysyms::XK_TAB,
        "Backspace" => keysyms::XK_BACKSPACE,
        "Delete" => keysyms::XK_DELETE,
        "F1" => keysyms::XK_F1,
        "F2" => keysyms::XK_F2,
        "F3" => keysyms::XK_F3,
        "F4" => keysyms::XK_F4,
        "F5" => keysyms::XK_F5,
        "F6" => keysyms::XK_F6,
        "F7" => keysyms::XK_F7,
        "F8" => keysyms::XK_F8,
        "F9" => keysyms::XK_F9,
        "F10" => keysyms::XK_F10,
        "F11" => keysyms::XK_F11,
        "F12" => keysyms::XK_F12,
        "A" => keysyms::XK_A,
        "B" => keysyms::XK_B,
        "C" => keysyms::XK_C,
        "D" => keysyms::XK_D,
        "E" => keysyms::XK_E,
        "F" => keysyms::XK_F,
        "G" => keysyms::XK_G,
        "H" => keysyms::XK_H,
        "I" => keysyms::XK_I,
        "J" => keysyms::XK_J,
        "K" => keysyms::XK_K,
        "L" => keysyms::XK_L,
        "M" => keysyms::XK_M,
        "N" => keysyms::XK_N,
        "O" => keysyms::XK_O,
        "P" => keysyms::XK_P,
        "R" => keysyms::XK_R,
        "S" => keysyms::XK_S,
        "T" => keysyms::XK_T,
        "U" => keysyms::XK_U,
        "V" => keysyms::XK_V,
        "W" => keysyms::XK_W,
        "X" => keysyms::XK_X,
        "Y" => keysyms::XK_Y,
        "Z" => keysyms::XK_Z,
        "0" => keysyms::XK_0,
        "1" => keysyms::XK_1,
        "2" => keysyms::XK_2,
        "3" => keysyms::XK_3,
        "4" => keysyms::XK_4,
        "5" => keysyms::XK_5,
        "6" => keysyms::XK_6,
        "7" => keysyms::XK_7,
        "8" => keysyms::XK_8,
        "9" => keysyms::XK_9,
        "Left" => keysyms::XK_LEFT,
        "Right" => keysyms::XK_RIGHT,
        "Up" => keysyms::XK_UP,
        "Down" => keysyms::XK_DOWN,
        "Home" => keysyms::XK_HOME,
        "End" => keysyms::XK_END,
        "PageUp" => keysyms::XK_PAGE_UP,
        "PageDown" => keysyms::XK_PAGE_DOWN,
        "Insert" => keysyms::XK_INSERT,
        "Minus" => keysyms::XK_MINUS,
        "Equal" => keysyms::XK_EQUAL,
        "BracketLeft" => keysyms::XK_LEFT_BRACKET,
        "BracketRight" => keysyms::XK_RIGHT_BRACKET,
        "Semicolon" => keysyms::XK_SEMICOLON,
        "Apostrophe" => keysyms::XK_APOSTROPHE,
        "Grave" => keysyms::XK_GRAVE,
        "Backslash" => keysyms::XK_BACKSLASH,
        "Comma" => keysyms::XK_COMMA,
        "Period" => keysyms::XK_PERIOD,
        "Slash" => keysyms::XK_SLASH,
        "AudioRaiseVolume" => keysyms::XF86_AUDIO_RAISE_VOLUME,
        "AudioLowerVolume" => keysyms::XF86_AUDIO_LOWER_VOLUME,
        "AudioMute" => keysyms::XF86_AUDIO_MUTE,
        "MonBrightnessUp" => keysyms::XF86_MON_BRIGHTNESS_UP,
        "MonBrightnessDown" => keysyms::XF86_MON_BRIGHTNESS_DOWN,
        _ => return Err(ConfigError::UnknownKey(s.to_string())),
    };

    Ok(keysym)
}

fn parse_key_action(kb_table: &Table) -> Result<KeyAction, ConfigError> {
    let action_str: String = get_table_field(kb_table, "action")?;
    string_to_key_action(&action_str)
}

fn string_to_key_action(s: &str) -> Result<KeyAction, ConfigError> {
    let action = match s {
        "Spawn" => KeyAction::Spawn,
        "KillClient" => KeyAction::KillClient,
        "FocusStack" => KeyAction::FocusStack,
        "FocusDirection" => KeyAction::FocusDirection,
        "SwapDirection" => KeyAction::SwapDirection,
        "Quit" => KeyAction::Quit,
        "Restart" => KeyAction::Restart,
        "Recompile" => KeyAction::Recompile,
        "ViewTag" => KeyAction::ViewTag,
        "ToggleGaps" => KeyAction::ToggleGaps,
        "ToggleFullScreen" => KeyAction::ToggleFullScreen,
        "ToggleFloating" => KeyAction::ToggleFloating,
        "ChangeLayout" => KeyAction::ChangeLayout,
        "CycleLayout" => KeyAction::CycleLayout,
        "MoveToTag" => KeyAction::MoveToTag,
        "FocusMonitor" => KeyAction::FocusMonitor,
        "SmartMoveWin" => KeyAction::SmartMoveWin,
        "ExchangeClient" => KeyAction::ExchangeClient,
        "None" => KeyAction::None,
        _ => return Err(ConfigError::UnknownAction(s.to_string())),
    };

    Ok(action)
}

fn parse_arg(kb_table: &Table) -> Result<Arg, ConfigError> {
    let arg_result: Result<Value, _> = kb_table.get("arg");

    match arg_result {
        Ok(Value::Nil) | Err(_) => Ok(Arg::None),
        Ok(Value::String(s)) => {
            let s = s
                .to_str()
                .map_err(|e| ConfigError::LuaError(format!("Invalid UTF-8 in arg: {}", e)))?;
            Ok(Arg::Str(s.to_string()))
        }
        Ok(Value::Integer(i)) => Ok(Arg::Int(i as i32)),
        Ok(Value::Number(n)) => Ok(Arg::Int(n as i32)),
        Ok(Value::Table(t)) => {
            let mut arr = Vec::new();
            for i in 1..=t
                .len()
                .map_err(|e| ConfigError::LuaError(format!("Failed to get arg array length: {}", e)))?
            {
                let item: String = t.get(i).map_err(|e| {
                    ConfigError::LuaError(format!("Failed to get arg array item at index {}: {}", i, e))
                })?;
                arr.push(item);
            }
            Ok(Arg::Array(arr))
        }
        Ok(_) => Err(ConfigError::LuaError(
            "Arg must be nil, string, number, or array".to_string(),
        )),
    }
}

fn parse_status_blocks(config: &Table) -> Result<Vec<BlockConfig>, ConfigError> {
    let blocks_table: Table = get_table_field(config, "status_blocks")?;
    let mut blocks = Vec::new();

    for i in 1..=blocks_table.len().map_err(|e| {
        ConfigError::LuaError(format!("Failed to get status_blocks length: {}", e))
    })? {
        let block_table: Table = blocks_table.get(i).map_err(|e| {
            ConfigError::LuaError(format!("Failed to get status_block at index {}: {}", i, e))
        })?;

        let format: String = get_table_field(&block_table, "format")?;
        let command_str: String = get_table_field(&block_table, "command")?;

        // Parse interval_secs - handle both integer and number types
        let interval_secs: u64 = {
            let value: Value = block_table.get("interval_secs").map_err(|e| {
                ConfigError::LuaError(format!("Failed to get interval_secs: {}", e))
            })?;
            match value {
                Value::Integer(i) => i as u64,
                Value::Number(n) => n as u64,
                _ => return Err(ConfigError::LuaError("interval_secs must be a number".to_string())),
            }
        };

        let color: u32 = parse_color(&block_table, "color")?;
        let underline: bool = get_table_field(&block_table, "underline")?;

        let command = match command_str.as_str() {
            "DateTime" => {
                let fmt: String = get_table_field(&block_table, "command_arg")?;
                BlockCommand::DateTime(fmt)
            }
            "Shell" => {
                let cmd: String = get_table_field(&block_table, "command_arg")?;
                BlockCommand::Shell(cmd)
            }
            "Ram" => BlockCommand::Ram,
            "Static" => {
                let text_result: Result<String, _> = block_table.get("command_arg");
                let text = text_result.unwrap_or_default();
                BlockCommand::Static(text)
            }
            "Battery" => {
                let formats_table: Table = get_table_field(&block_table, "battery_formats")?;
                let format_charging: String = get_table_field(&formats_table, "charging")?;
                let format_discharging: String = get_table_field(&formats_table, "discharging")?;
                let format_full: String = get_table_field(&formats_table, "full")?;

                BlockCommand::Battery {
                    format_charging,
                    format_discharging,
                    format_full,
                }
            }
            _ => return Err(ConfigError::UnknownBlockCommand(command_str)),
        };

        blocks.push(BlockConfig {
            format,
            command,
            interval_secs,
            color,
            underline,
        });
    }

    Ok(blocks)
}

fn parse_color_scheme(config: &Table, field: &str) -> Result<ColorScheme, ConfigError> {
    let scheme_table: Table = get_table_field(config, field)?;

    let foreground = parse_color(&scheme_table, "foreground")?;
    let background = parse_color(&scheme_table, "background")?;
    let underline = parse_color(&scheme_table, "underline")?;

    Ok(ColorScheme {
        foreground,
        background,
        underline,
    })
}

fn parse_autostart(config: &Table) -> Result<Vec<String>, ConfigError> {
    let autostart_result: Result<Table, _> = config.get("autostart");

    match autostart_result {
        Ok(autostart_table) => {
            let mut autostart = Vec::new();

            for i in 1..=autostart_table.len().map_err(|e| {
                ConfigError::LuaError(format!("Failed to get autostart length: {}", e))
            })? {
                let cmd: String = autostart_table.get(i).map_err(|e| {
                    ConfigError::LuaError(format!("Failed to get autostart command at index {}: {}", i, e))
                })?;
                autostart.push(cmd);
            }

            Ok(autostart)
        }
        Err(_) => Ok(Vec::new()), // autostart is optional
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_lua_config() {
        let config_str = r#"
return {
    border_width = 2,
    border_focused = 0x6dade3,
    border_unfocused = 0xbbbbbb,
    font = "monospace:style=Bold:size=10",

    gaps_enabled = true,
    gap_inner_horizontal = 5,
    gap_inner_vertical = 5,
    gap_outer_horizontal = 5,
    gap_outer_vertical = 5,

    modkey = "Mod4",
    terminal = "st",

    tags = {"1", "2", "3"},

    keybindings = {
        {modifiers = {"Mod4"}, key = "Return", action = "Spawn", arg = "st"},
        {modifiers = {"Mod4"}, key = "Q", action = "KillClient"},
    },

    status_blocks = {
        {format = "{}", command = "DateTime", command_arg = "%H:%M", interval_secs = 1, color = 0xffffff, underline = true},
    },

    scheme_normal = {foreground = 0xffffff, background = 0x000000, underline = 0x444444},
    scheme_occupied = {foreground = 0xffffff, background = 0x000000, underline = 0x444444},
    scheme_selected = {foreground = 0xffffff, background = 0x000000, underline = 0x444444},

    autostart = {},
}
"#;

        let config = parse_lua_config(config_str).expect("Failed to parse config");

        assert_eq!(config.border_width, 2);
        assert_eq!(config.border_focused, 0x6dade3);
        assert_eq!(config.terminal, "st");
        assert_eq!(config.tags.len(), 3);
        assert_eq!(config.keybindings.len(), 2);
        assert_eq!(config.status_blocks.len(), 1);
        assert!(config.gaps_enabled);
    }
}
