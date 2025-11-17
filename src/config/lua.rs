use crate::errors::ConfigError;
use mlua::Lua;

use super::lua_api;

pub fn parse_lua_config(
    input: &str,
    config_dir: Option<&std::path::Path>,
) -> Result<crate::Config, ConfigError> {
    let lua = Lua::new();

    if let Some(dir) = config_dir {
        if let Some(dir_str) = dir.to_str() {
            let setup_code = format!("package.path = '{}/?.lua;' .. package.path", dir_str);
            lua.load(&setup_code)
                .exec()
                .map_err(|e| ConfigError::LuaError(format!("Failed to set package.path: {}", e)))?;
        }
    }

    let builder = lua_api::register_api(&lua)?;

    lua.load(input)
        .exec()
        .map_err(|e| ConfigError::LuaError(format!("{}", e)))?;

    let builder_data = builder.borrow().clone();

    return Ok(crate::Config {
        border_width: builder_data.border_width,
        border_focused: builder_data.border_focused,
        border_unfocused: builder_data.border_unfocused,
        font: builder_data.font,
        gaps_enabled: builder_data.gaps_enabled,
        gap_inner_horizontal: builder_data.gap_inner_horizontal,
        gap_inner_vertical: builder_data.gap_inner_vertical,
        gap_outer_horizontal: builder_data.gap_outer_horizontal,
        gap_outer_vertical: builder_data.gap_outer_vertical,
        terminal: builder_data.terminal,
        modkey: builder_data.modkey,
        tags: builder_data.tags,
        layout_symbols: builder_data.layout_symbols,
        keybindings: builder_data.keybindings,
        status_blocks: builder_data.status_blocks,
        scheme_normal: builder_data.scheme_normal,
        scheme_occupied: builder_data.scheme_occupied,
        scheme_selected: builder_data.scheme_selected,
        autostart: builder_data.autostart,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_lua_config() {
        let config_str = r#"
oxwm.border.set_width(2)
oxwm.border.set_focused_color(0x6dade3)
oxwm.border.set_unfocused_color(0xbbbbbb)
oxwm.bar.set_font("monospace:style=Bold:size=10")

oxwm.gaps.set_enabled(true)
oxwm.gaps.set_inner(5, 5)
oxwm.gaps.set_outer(5, 5)

oxwm.set_modkey("Mod4")
oxwm.set_terminal("st")
oxwm.set_tags({"1", "2", "3"})

oxwm.key.bind({"Mod4"}, "Return", oxwm.spawn("st"))
oxwm.key.bind({"Mod4"}, "Q", oxwm.client.kill())

oxwm.bar.add_block("{}", "DateTime", "%H:%M", 1, 0xffffff, true)

oxwm.bar.set_scheme_normal(0xffffff, 0x000000, 0x444444)
oxwm.bar.set_scheme_occupied(0xffffff, 0x000000, 0x444444)
oxwm.bar.set_scheme_selected(0xffffff, 0x000000, 0x444444)
"#;

        let config = parse_lua_config(config_str, None).expect("Failed to parse config");

        assert_eq!(config.border_width, 2);
        assert_eq!(config.border_focused, 0x6dade3);
        assert_eq!(config.terminal, "st");
        assert_eq!(config.tags.len(), 3);
        assert_eq!(config.keybindings.len(), 2);
        assert_eq!(config.status_blocks.len(), 1);
        assert!(config.gaps_enabled);
    }
}
