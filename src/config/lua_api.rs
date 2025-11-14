use mlua::{Lua, Table, Value};
use std::cell::RefCell;
use std::rc::Rc;

use crate::bar::BlockConfig;
use crate::errors::ConfigError;
use crate::keyboard::handlers::{Arg, KeyAction, KeyBinding, KeyPress};
use crate::keyboard::keysyms::{self, Keysym};
use crate::ColorScheme;
use x11rb::protocol::xproto::KeyButMask;

#[derive(Clone)]
pub struct ConfigBuilder {
    pub border_width: u32,
    pub border_focused: u32,
    pub border_unfocused: u32,
    pub font: String,
    pub gaps_enabled: bool,
    pub gap_inner_horizontal: u32,
    pub gap_inner_vertical: u32,
    pub gap_outer_horizontal: u32,
    pub gap_outer_vertical: u32,
    pub terminal: String,
    pub modkey: KeyButMask,
    pub tags: Vec<String>,
    pub layout_symbols: Vec<crate::LayoutSymbolOverride>,
    pub keybindings: Vec<KeyBinding>,
    pub status_blocks: Vec<BlockConfig>,
    pub scheme_normal: ColorScheme,
    pub scheme_occupied: ColorScheme,
    pub scheme_selected: ColorScheme,
    pub autostart: Vec<String>,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self {
            border_width: 2,
            border_focused: 0x6dade3,
            border_unfocused: 0xbbbbbb,
            font: "monospace:style=Bold:size=10".to_string(),
            gaps_enabled: true,
            gap_inner_horizontal: 5,
            gap_inner_vertical: 5,
            gap_outer_horizontal: 5,
            gap_outer_vertical: 5,
            terminal: "st".to_string(),
            modkey: KeyButMask::MOD4,
            tags: vec!["1".into(), "2".into(), "3".into()],
            layout_symbols: Vec::new(),
            keybindings: Vec::new(),
            status_blocks: Vec::new(),
            scheme_normal: ColorScheme {
                foreground: 0xffffff,
                background: 0x000000,
                underline: 0x444444,
            },
            scheme_occupied: ColorScheme {
                foreground: 0xffffff,
                background: 0x000000,
                underline: 0x444444,
            },
            scheme_selected: ColorScheme {
                foreground: 0xffffff,
                background: 0x000000,
                underline: 0x444444,
            },
            autostart: Vec::new(),
        }
    }
}

type SharedBuilder = Rc<RefCell<ConfigBuilder>>;

pub fn register_api(lua: &Lua) -> Result<SharedBuilder, ConfigError> {
    let builder = Rc::new(RefCell::new(ConfigBuilder::default()));

    let oxwm_table = lua.create_table()
        .map_err(|e| ConfigError::LuaError(format!("Failed to create oxwm table: {}", e)))?;

    register_spawn(&lua, &oxwm_table, builder.clone())?;
    register_key_module(&lua, &oxwm_table, builder.clone())?;
    register_gaps_module(&lua, &oxwm_table, builder.clone())?;
    register_border_module(&lua, &oxwm_table, builder.clone())?;
    register_client_module(&lua, &oxwm_table)?;
    register_layout_module(&lua, &oxwm_table)?;
    register_tag_module(&lua, &oxwm_table)?;
    register_bar_module(&lua, &oxwm_table, builder.clone())?;
    register_misc(&lua, &oxwm_table, builder.clone())?;

    lua.globals().set("oxwm", oxwm_table)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set oxwm global: {}", e)))?;

    Ok(builder)
}

fn register_spawn(lua: &Lua, parent: &Table, _builder: SharedBuilder) -> Result<(), ConfigError> {
    let spawn = lua.create_function(|lua, cmd: Value| {
        create_action_table(lua, "Spawn", cmd)
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create spawn: {}", e)))?;
    parent.set("spawn", spawn)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set spawn: {}", e)))?;
    Ok(())
}

fn register_key_module(lua: &Lua, parent: &Table, builder: SharedBuilder) -> Result<(), ConfigError> {
    let key_table = lua.create_table()
        .map_err(|e| ConfigError::LuaError(format!("Failed to create key table: {}", e)))?;

    let builder_clone = builder.clone();
    let bind = lua.create_function(move |lua, (mods, key, action): (Value, String, Value)| {
        let modifiers = parse_modifiers_value(lua, mods)?;
        let keysym = parse_keysym(&key)?;
        let (key_action, arg) = parse_action_value(lua, action)?;

        let binding = KeyBinding::single_key(modifiers, keysym, key_action, arg);
        builder_clone.borrow_mut().keybindings.push(binding);

        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create bind: {}", e)))?;

    let builder_clone = builder.clone();
    let chord = lua.create_function(move |lua, (keys, action): (Table, Value)| {
        let mut key_presses = Vec::new();

        for i in 1..=keys.len()? {
            let key_spec: Table = keys.get(i)?;
            let mods: Value = key_spec.get(1)?;
            let key: String = key_spec.get(2)?;

            let modifiers = parse_modifiers_value(lua, mods)?;
            let keysym = parse_keysym(&key)?;

            key_presses.push(KeyPress { modifiers, keysym });
        }

        let (key_action, arg) = parse_action_value(lua, action)?;
        let binding = KeyBinding::new(key_presses, key_action, arg);
        builder_clone.borrow_mut().keybindings.push(binding);

        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create chord: {}", e)))?;

    key_table.set("bind", bind)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set bind: {}", e)))?;
    key_table.set("chord", chord)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set chord: {}", e)))?;
    parent.set("key", key_table)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set key: {}", e)))?;
    Ok(())
}

fn register_gaps_module(lua: &Lua, parent: &Table, builder: SharedBuilder) -> Result<(), ConfigError> {
    let gaps_table = lua.create_table()
        .map_err(|e| ConfigError::LuaError(format!("Failed to create gaps table: {}", e)))?;

    let builder_clone = builder.clone();
    let set_enabled = lua.create_function(move |_, enabled: bool| {
        builder_clone.borrow_mut().gaps_enabled = enabled;
        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create set_enabled: {}", e)))?;

    let builder_clone = builder.clone();
    let enable = lua.create_function(move |_, ()| {
        builder_clone.borrow_mut().gaps_enabled = true;
        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create enable: {}", e)))?;

    let builder_clone = builder.clone();
    let disable = lua.create_function(move |_, ()| {
        builder_clone.borrow_mut().gaps_enabled = false;
        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create disable: {}", e)))?;

    let builder_clone = builder.clone();
    let set_inner = lua.create_function(move |_, (h, v): (u32, u32)| {
        let mut b = builder_clone.borrow_mut();
        b.gap_inner_horizontal = h;
        b.gap_inner_vertical = v;
        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create set_inner: {}", e)))?;

    let builder_clone = builder.clone();
    let set_outer = lua.create_function(move |_, (h, v): (u32, u32)| {
        let mut b = builder_clone.borrow_mut();
        b.gap_outer_horizontal = h;
        b.gap_outer_vertical = v;
        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create set_outer: {}", e)))?;

    gaps_table.set("set_enabled", set_enabled)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set set_enabled: {}", e)))?;
    gaps_table.set("enable", enable)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set enable: {}", e)))?;
    gaps_table.set("disable", disable)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set disable: {}", e)))?;
    gaps_table.set("set_inner", set_inner)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set set_inner: {}", e)))?;
    gaps_table.set("set_outer", set_outer)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set set_outer: {}", e)))?;
    parent.set("gaps", gaps_table)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set gaps: {}", e)))?;
    Ok(())
}

fn register_border_module(lua: &Lua, parent: &Table, builder: SharedBuilder) -> Result<(), ConfigError> {
    let border_table = lua.create_table()
        .map_err(|e| ConfigError::LuaError(format!("Failed to create border table: {}", e)))?;

    let builder_clone = builder.clone();
    let set_width = lua.create_function(move |_, width: u32| {
        builder_clone.borrow_mut().border_width = width;
        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create set_width: {}", e)))?;

    let builder_clone = builder.clone();
    let set_focused_color = lua.create_function(move |_, color: Value| {
        let color_u32 = parse_color_value(color)?;
        builder_clone.borrow_mut().border_focused = color_u32;
        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create set_focused_color: {}", e)))?;

    let builder_clone = builder.clone();
    let set_unfocused_color = lua.create_function(move |_, color: Value| {
        let color_u32 = parse_color_value(color)?;
        builder_clone.borrow_mut().border_unfocused = color_u32;
        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create set_unfocused_color: {}", e)))?;

    border_table.set("set_width", set_width)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set set_width: {}", e)))?;
    border_table.set("set_focused_color", set_focused_color)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set set_focused_color: {}", e)))?;
    border_table.set("set_unfocused_color", set_unfocused_color)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set set_unfocused_color: {}", e)))?;
    parent.set("border", border_table)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set border: {}", e)))?;
    Ok(())
}

fn register_client_module(lua: &Lua, parent: &Table) -> Result<(), ConfigError> {
    let client_table = lua.create_table()
        .map_err(|e| ConfigError::LuaError(format!("Failed to create client table: {}", e)))?;

    let kill = lua.create_function(|lua, ()| {
        create_action_table(lua, "KillClient", Value::Nil)
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create kill: {}", e)))?;

    let toggle_fullscreen = lua.create_function(|lua, ()| {
        create_action_table(lua, "ToggleFullScreen", Value::Nil)
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create toggle_fullscreen: {}", e)))?;

    let toggle_floating = lua.create_function(|lua, ()| {
        create_action_table(lua, "ToggleFloating", Value::Nil)
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create toggle_floating: {}", e)))?;

    let focus_stack = lua.create_function(|lua, dir: i32| {
        create_action_table(lua, "FocusStack", Value::Integer(dir as i64))
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create focus_stack: {}", e)))?;

    let focus_direction = lua.create_function(|lua, dir: String| {
        let dir_int = direction_string_to_int(&dir)?;
        create_action_table(lua, "FocusDirection", Value::Integer(dir_int))
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create focus_direction: {}", e)))?;

    let swap_direction = lua.create_function(|lua, dir: String| {
        let dir_int = direction_string_to_int(&dir)?;
        create_action_table(lua, "SwapDirection", Value::Integer(dir_int))
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create swap_direction: {}", e)))?;

    let smart_move = lua.create_function(|lua, dir: String| {
        let dir_int = direction_string_to_int(&dir)?;
        create_action_table(lua, "SmartMoveWin", Value::Integer(dir_int))
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create smart_move: {}", e)))?;

    let exchange = lua.create_function(|lua, ()| {
        create_action_table(lua, "ExchangeClient", Value::Nil)
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create exchange: {}", e)))?;

    client_table.set("kill", kill)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set kill: {}", e)))?;
    client_table.set("toggle_fullscreen", toggle_fullscreen)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set toggle_fullscreen: {}", e)))?;
    client_table.set("toggle_floating", toggle_floating)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set toggle_floating: {}", e)))?;
    client_table.set("focus_stack", focus_stack)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set focus_stack: {}", e)))?;
    client_table.set("focus_direction", focus_direction)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set focus_direction: {}", e)))?;
    client_table.set("swap_direction", swap_direction)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set swap_direction: {}", e)))?;
    client_table.set("smart_move", smart_move)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set smart_move: {}", e)))?;
    client_table.set("exchange", exchange)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set exchange: {}", e)))?;

    parent.set("client", client_table)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set client: {}", e)))?;
    Ok(())
}

fn register_layout_module(lua: &Lua, parent: &Table) -> Result<(), ConfigError> {
    let layout_table = lua.create_table()
        .map_err(|e| ConfigError::LuaError(format!("Failed to create layout table: {}", e)))?;

    let cycle = lua.create_function(|lua, ()| {
        create_action_table(lua, "CycleLayout", Value::Nil)
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create cycle: {}", e)))?;

    let set = lua.create_function(|lua, name: String| {
        create_action_table(lua, "ChangeLayout", Value::String(lua.create_string(&name)?))
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create set: {}", e)))?;

    layout_table.set("cycle", cycle)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set cycle: {}", e)))?;
    layout_table.set("set", set)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set set: {}", e)))?;
    parent.set("layout", layout_table)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set layout: {}", e)))?;
    Ok(())
}

fn register_tag_module(lua: &Lua, parent: &Table) -> Result<(), ConfigError> {
    let tag_table = lua.create_table()
        .map_err(|e| ConfigError::LuaError(format!("Failed to create tag table: {}", e)))?;

    let view = lua.create_function(|lua, idx: i32| {
        create_action_table(lua, "ViewTag", Value::Integer(idx as i64))
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create view: {}", e)))?;

    let move_to = lua.create_function(|lua, idx: i32| {
        create_action_table(lua, "MoveToTag", Value::Integer(idx as i64))
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create move_to: {}", e)))?;

    tag_table.set("view", view)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set view: {}", e)))?;
    tag_table.set("move_to", move_to)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set move_to: {}", e)))?;
    parent.set("tag", tag_table)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set tag: {}", e)))?;
    Ok(())
}

fn register_bar_module(lua: &Lua, parent: &Table, builder: SharedBuilder) -> Result<(), ConfigError> {
    let bar_table = lua.create_table()
        .map_err(|e| ConfigError::LuaError(format!("Failed to create bar table: {}", e)))?;

    let builder_clone = builder.clone();
    let set_font = lua.create_function(move |_, font: String| {
        builder_clone.borrow_mut().font = font;
        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create set_font: {}", e)))?;

    let builder_clone = builder.clone();
    let add_block = lua.create_function(move |_, (format, command, arg, interval, color, underline): (String, String, Option<Value>, u64, Value, bool)| {
        use crate::bar::BlockCommand;

        let cmd = match command.as_str() {
            "DateTime" => {
                let fmt = arg.and_then(|v| {
                    if let Value::String(s) = v {
                        s.to_str().ok().map(|s| s.to_string())
                    } else {
                        None
                    }
                }).ok_or_else(|| mlua::Error::RuntimeError("oxwm.bar.add_block: DateTime command requires a format string as the third argument. example: oxwm.bar.add_block(\"\", \"DateTime\", \"%H:%M\", 60, 0xffffff, false)".into()))?;
                BlockCommand::DateTime(fmt)
            }
            "Shell" => {
                let cmd_str = arg.and_then(|v| {
                    if let Value::String(s) = v {
                        s.to_str().ok().map(|s| s.to_string())
                    } else {
                        None
                    }
                }).ok_or_else(|| mlua::Error::RuntimeError("oxwm.bar.add_block: Shell command requires a shell command string as the third argument. example: oxwm.bar.add_block(\"\", \"Shell\", \"date +%H:%M\", 60, 0xffffff, false)".into()))?;
                BlockCommand::Shell(cmd_str)
            }
            "Ram" => BlockCommand::Ram,
            "Static" => {
                let text = arg.and_then(|v| {
                    if let Value::String(s) = v {
                        s.to_str().ok().map(|s| s.to_string())
                    } else {
                        None
                    }
                }).unwrap_or_default();
                BlockCommand::Static(text)
            }
            "Battery" => {
                let formats = arg.and_then(|v| {
                    if let Value::Table(t) = v {
                        Some(t)
                    } else {
                        None
                    }
                }).ok_or_else(|| mlua::Error::RuntimeError("oxwm.bar.add_block: Battery command requires a formats table as the third argument. example: {charging=\"CHR {percentage}%\", discharging=\"BAT {percentage}%\", full=\"FULL\"}".into()))?;

                let charging: String = formats.get("charging")?;
                let discharging: String = formats.get("discharging")?;
                let full: String = formats.get("full")?;

                BlockCommand::Battery {
                    format_charging: charging,
                    format_discharging: discharging,
                    format_full: full,
                }
            }
            _ => return Err(mlua::Error::RuntimeError(format!("oxwm.bar.add_block: unknown block command '{}'. valid commands: DateTime, Shell, Ram, Static, Battery", command))),
        };

        let color_u32 = parse_color_value(color)?;

        let block = crate::bar::BlockConfig {
            format,
            command: cmd,
            interval_secs: interval,
            color: color_u32,
            underline,
        };

        builder_clone.borrow_mut().status_blocks.push(block);
        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create add_block: {}", e)))?;

    let builder_clone = builder.clone();
    let set_scheme_normal = lua.create_function(move |_, (fg, bg, ul): (Value, Value, Value)| {
        let foreground = parse_color_value(fg)?;
        let background = parse_color_value(bg)?;
        let underline = parse_color_value(ul)?;

        builder_clone.borrow_mut().scheme_normal = ColorScheme {
            foreground,
            background,
            underline,
        };
        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create set_scheme_normal: {}", e)))?;

    let builder_clone = builder.clone();
    let set_scheme_occupied = lua.create_function(move |_, (fg, bg, ul): (Value, Value, Value)| {
        let foreground = parse_color_value(fg)?;
        let background = parse_color_value(bg)?;
        let underline = parse_color_value(ul)?;

        builder_clone.borrow_mut().scheme_occupied = ColorScheme {
            foreground,
            background,
            underline,
        };
        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create set_scheme_occupied: {}", e)))?;

    let builder_clone = builder.clone();
    let set_scheme_selected = lua.create_function(move |_, (fg, bg, ul): (Value, Value, Value)| {
        let foreground = parse_color_value(fg)?;
        let background = parse_color_value(bg)?;
        let underline = parse_color_value(ul)?;

        builder_clone.borrow_mut().scheme_selected = ColorScheme {
            foreground,
            background,
            underline,
        };
        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create set_scheme_selected: {}", e)))?;

    bar_table.set("set_font", set_font)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set set_font: {}", e)))?;
    bar_table.set("add_block", add_block)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set add_block: {}", e)))?;
    bar_table.set("set_scheme_normal", set_scheme_normal)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set set_scheme_normal: {}", e)))?;
    bar_table.set("set_scheme_occupied", set_scheme_occupied)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set set_scheme_occupied: {}", e)))?;
    bar_table.set("set_scheme_selected", set_scheme_selected)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set set_scheme_selected: {}", e)))?;
    parent.set("bar", bar_table)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set bar: {}", e)))?;
    Ok(())
}

fn register_misc(lua: &Lua, parent: &Table, builder: SharedBuilder) -> Result<(), ConfigError> {
    let builder_clone = builder.clone();
    let set_terminal = lua.create_function(move |_, term: String| {
        builder_clone.borrow_mut().terminal = term;
        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create set_terminal: {}", e)))?;

    let builder_clone = builder.clone();
    let set_modkey = lua.create_function(move |_, modkey_str: String| {
        let modkey = parse_modkey_string(&modkey_str)
            .map_err(|e| mlua::Error::RuntimeError(format!("{}", e)))?;
        builder_clone.borrow_mut().modkey = modkey;
        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create set_modkey: {}", e)))?;

    let builder_clone = builder.clone();
    let set_tags = lua.create_function(move |_, tags: Vec<String>| {
        builder_clone.borrow_mut().tags = tags;
        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create set_tags: {}", e)))?;

    let quit = lua.create_function(|lua, ()| {
        create_action_table(lua, "Quit", Value::Nil)
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create quit: {}", e)))?;

    let restart = lua.create_function(|lua, ()| {
        create_action_table(lua, "Restart", Value::Nil)
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create restart: {}", e)))?;

    let recompile = lua.create_function(|lua, ()| {
        create_action_table(lua, "Recompile", Value::Nil)
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create recompile: {}", e)))?;

    let toggle_gaps = lua.create_function(|lua, ()| {
        create_action_table(lua, "ToggleGaps", Value::Nil)
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create toggle_gaps: {}", e)))?;

    let show_keybinds = lua.create_function(|lua, ()| {
        create_action_table(lua, "ShowKeybindOverlay", Value::Nil)
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create show_keybinds: {}", e)))?;

    let focus_monitor = lua.create_function(|lua, idx: i32| {
        create_action_table(lua, "FocusMonitor", Value::Integer(idx as i64))
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create focus_monitor: {}", e)))?;

    let builder_clone = builder.clone();
    let set_layout_symbol = lua.create_function(move |_, (name, symbol): (String, String)| {
        builder_clone.borrow_mut().layout_symbols.push(crate::LayoutSymbolOverride {
            name,
            symbol,
        });
        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create set_layout_symbol: {}", e)))?;

    let builder_clone = builder.clone();
    let autostart = lua.create_function(move |_, cmd: String| {
        builder_clone.borrow_mut().autostart.push(cmd);
        Ok(())
    }).map_err(|e| ConfigError::LuaError(format!("Failed to create autostart: {}", e)))?;

    parent.set("set_terminal", set_terminal)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set set_terminal: {}", e)))?;
    parent.set("set_modkey", set_modkey)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set set_modkey: {}", e)))?;
    parent.set("set_tags", set_tags)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set set_tags: {}", e)))?;
    parent.set("set_layout_symbol", set_layout_symbol)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set set_layout_symbol: {}", e)))?;
    parent.set("autostart", autostart)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set autostart: {}", e)))?;
    parent.set("quit", quit)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set quit: {}", e)))?;
    parent.set("restart", restart)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set restart: {}", e)))?;
    parent.set("recompile", recompile)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set recompile: {}", e)))?;
    parent.set("toggle_gaps", toggle_gaps)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set toggle_gaps: {}", e)))?;
    parent.set("show_keybinds", show_keybinds)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set show_keybinds: {}", e)))?;
    parent.set("focus_monitor", focus_monitor)
        .map_err(|e| ConfigError::LuaError(format!("Failed to set focus_monitor: {}", e)))?;
    Ok(())
}

fn parse_modifiers_value(_lua: &Lua, value: Value) -> mlua::Result<Vec<KeyButMask>> {
    match value {
        Value::Table(t) => {
            let mut mods = Vec::new();
            for i in 1..=t.len()? {
                let mod_str: String = t.get(i)?;
                let mask = parse_modkey_string(&mod_str)
                    .map_err(|e| mlua::Error::RuntimeError(format!("oxwm.key.bind: invalid modifier - {}", e)))?;
                mods.push(mask);
            }
            Ok(mods)
        }
        Value::String(s) => {
            let s_str = s.to_str()?;
            let mask = parse_modkey_string(&s_str)
                .map_err(|e| mlua::Error::RuntimeError(format!("oxwm.key.bind: invalid modifier - {}", e)))?;
            Ok(vec![mask])
        }
        _ => Err(mlua::Error::RuntimeError(
            "oxwm.key.bind: first argument must be a table of modifiers like {\"Mod4\"} or {\"Mod4\", \"Shift\"}".into(),
        )),
    }
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
        _ => Err(ConfigError::InvalidModkey(format!("'{}' is not a valid modifier. Use one of: Mod1, Mod4, Shift, Control", s))),
    }
}

fn parse_keysym(key: &str) -> mlua::Result<Keysym> {
    keysyms::keysym_from_str(key)
        .ok_or_else(|| mlua::Error::RuntimeError(format!("unknown key '{}'. valid keys include: Return, Space, A-Z, 0-9, F1-F12, Left, Right, Up, Down, etc. check oxwm.lua type definitions for the complete list", key)))
}

fn parse_action_value(_lua: &Lua, value: Value) -> mlua::Result<(KeyAction, Arg)> {
    match value {
        Value::Function(_) => {
            Err(mlua::Error::RuntimeError(
                "action must be a function call, not a function reference. did you forget ()? example: oxwm.spawn('st') not oxwm.spawn".into()
            ))
        }
        Value::Table(t) => {
            if let Ok(action_name) = t.get::<String>("__action") {
                let action = string_to_action(&action_name)?;
                let arg = if let Ok(arg_val) = t.get::<Value>("__arg") {
                    value_to_arg(arg_val)?
                } else {
                    Arg::None
                };
                return Ok((action, arg));
            }

            Err(mlua::Error::RuntimeError(
                "action must be a table returned by oxwm functions like oxwm.spawn(), oxwm.client.kill(), oxwm.quit(), etc.".into(),
            ))
        }
        _ => Err(mlua::Error::RuntimeError(
            "action must be a table returned by oxwm functions like oxwm.spawn(), oxwm.client.kill(), oxwm.quit(), etc.".into(),
        )),
    }
}

fn string_to_action(s: &str) -> mlua::Result<KeyAction> {
    match s {
        "Spawn" => Ok(KeyAction::Spawn),
        "KillClient" => Ok(KeyAction::KillClient),
        "FocusStack" => Ok(KeyAction::FocusStack),
        "FocusDirection" => Ok(KeyAction::FocusDirection),
        "SwapDirection" => Ok(KeyAction::SwapDirection),
        "Quit" => Ok(KeyAction::Quit),
        "Restart" => Ok(KeyAction::Restart),
        "Recompile" => Ok(KeyAction::Recompile),
        "ViewTag" => Ok(KeyAction::ViewTag),
        "ToggleGaps" => Ok(KeyAction::ToggleGaps),
        "ToggleFullScreen" => Ok(KeyAction::ToggleFullScreen),
        "ToggleFloating" => Ok(KeyAction::ToggleFloating),
        "ChangeLayout" => Ok(KeyAction::ChangeLayout),
        "CycleLayout" => Ok(KeyAction::CycleLayout),
        "MoveToTag" => Ok(KeyAction::MoveToTag),
        "FocusMonitor" => Ok(KeyAction::FocusMonitor),
        "SmartMoveWin" => Ok(KeyAction::SmartMoveWin),
        "ExchangeClient" => Ok(KeyAction::ExchangeClient),
        "ShowKeybindOverlay" => Ok(KeyAction::ShowKeybindOverlay),
        _ => Err(mlua::Error::RuntimeError(format!("unknown action '{}'. this is an internal error, please report it", s))),
    }
}

fn value_to_arg(value: Value) -> mlua::Result<Arg> {
    match value {
        Value::Nil => Ok(Arg::None),
        Value::String(s) => Ok(Arg::Str(s.to_str()?.to_string())),
        Value::Integer(i) => Ok(Arg::Int(i as i32)),
        Value::Number(n) => Ok(Arg::Int(n as i32)),
        Value::Table(t) => {
            let mut arr = Vec::new();
            for i in 1..=t.len()? {
                let item: String = t.get(i)?;
                arr.push(item);
            }
            Ok(Arg::Array(arr))
        }
        _ => Ok(Arg::None),
    }
}

fn create_action_table(lua: &Lua, action_name: &str, arg: Value) -> mlua::Result<Table> {
    let table = lua.create_table()?;
    table.set("__action", action_name)?;
    table.set("__arg", arg)?;
    Ok(table)
}

fn direction_string_to_int(dir: &str) -> mlua::Result<i64> {
    match dir {
        "up" => Ok(0),
        "down" => Ok(1),
        "left" => Ok(2),
        "right" => Ok(3),
        _ => Err(mlua::Error::RuntimeError(
            format!("invalid direction '{}'. must be one of: up, down, left, right", dir)
        )),
    }
}

fn parse_color_value(value: Value) -> mlua::Result<u32> {
    match value {
        Value::Integer(i) => Ok(i as u32),
        Value::Number(n) => Ok(n as u32),
        Value::String(s) => {
            let s = s.to_str()?;
            if s.starts_with('#') {
                u32::from_str_radix(&s[1..], 16)
                    .map_err(|e| mlua::Error::RuntimeError(format!("invalid hex color '{}': {}. use format like #ff0000 or 0xff0000", s, e)))
            } else if s.starts_with("0x") {
                u32::from_str_radix(&s[2..], 16)
                    .map_err(|e| mlua::Error::RuntimeError(format!("invalid hex color '{}': {}. use format like 0xff0000 or #ff0000", s, e)))
            } else {
                s.parse::<u32>()
                    .map_err(|e| mlua::Error::RuntimeError(format!("invalid color '{}': {}. use hex format like 0xff0000 or #ff0000", s, e)))
            }
        }
        _ => Err(mlua::Error::RuntimeError(
            "color must be a number (0xff0000) or string ('#ff0000' or '0xff0000')".into(),
        )),
    }
}
