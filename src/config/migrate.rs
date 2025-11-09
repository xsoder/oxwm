use crate::errors::ConfigError;
use std::collections::HashMap;

pub fn ron_to_lua(ron_content: &str) -> Result<String, ConfigError> {
    let mut lua_output = String::new();
    let defines = extract_defines(ron_content);

    lua_output.push_str("-- OXWM Configuration File (Lua)\n");
    lua_output.push_str("-- Migrated from config.ron\n");
    lua_output.push_str("-- Edit this file and reload with Mod+Shift+R (no compilation needed!)\n\n");

    let terminal = resolve_value(&defines.get("$terminal").cloned().unwrap_or_else(|| "\"st\"".to_string()), &defines);
    let modkey = resolve_value(&defines.get("$modkey").cloned().unwrap_or_else(|| "Mod4".to_string()), &defines);
    let secondary_modkey = defines.get("$secondary_modkey").map(|v| resolve_value(v, &defines));

    lua_output.push_str(&format!("local terminal = {}\n", terminal));
    lua_output.push_str(&format!("local modkey = \"{}\"\n", modkey.trim_matches('"')));
    if let Some(sec_mod) = secondary_modkey {
        lua_output.push_str(&format!("local secondary_modkey = \"{}\"\n", sec_mod.trim_matches('"')));
    }
    lua_output.push_str("\n");

    lua_output.push_str("-- Color palette\n");
    lua_output.push_str("local colors = {\n");
    for (key, value) in &defines {
        if key.starts_with("$color_") {
            let color_name = &key[7..];
            let color_value = if value.starts_with("0x") {
                format!("\"#{}\"", &value[2..])
            } else {
                value.clone()
            };
            lua_output.push_str(&format!("    {} = {},\n", color_name, color_value));
        }
    }
    lua_output.push_str("}\n\n");

    lua_output.push_str("-- Main configuration table\n");
    lua_output.push_str("return {\n");

    if let Some(config_start) = ron_content.find('(') {
        let config_content = &ron_content[config_start + 1..];

        lua_output.push_str("    -- Appearance\n");
        if let Some(val) = extract_field(config_content, "border_width") {
            lua_output.push_str(&format!("    border_width = {},\n", val));
        }
        if let Some(val) = extract_field(config_content, "border_focused") {
            lua_output.push_str(&format!("    border_focused = {},\n", resolve_color_value(&val, &defines)));
        }
        if let Some(val) = extract_field(config_content, "border_unfocused") {
            lua_output.push_str(&format!("    border_unfocused = {},\n", resolve_color_value(&val, &defines)));
        }
        if let Some(val) = extract_field(config_content, "font") {
            lua_output.push_str(&format!("    font = {},\n", val));
        }

        lua_output.push_str("\n    -- Window gaps\n");
        if let Some(val) = extract_field(config_content, "gaps_enabled") {
            lua_output.push_str(&format!("    gaps_enabled = {},\n", val));
        }
        if let Some(val) = extract_field(config_content, "gap_inner_horizontal") {
            lua_output.push_str(&format!("    gap_inner_horizontal = {},\n", val));
        }
        if let Some(val) = extract_field(config_content, "gap_inner_vertical") {
            lua_output.push_str(&format!("    gap_inner_vertical = {},\n", val));
        }
        if let Some(val) = extract_field(config_content, "gap_outer_horizontal") {
            lua_output.push_str(&format!("    gap_outer_horizontal = {},\n", val));
        }
        if let Some(val) = extract_field(config_content, "gap_outer_vertical") {
            lua_output.push_str(&format!("    gap_outer_vertical = {},\n", val));
        }

        lua_output.push_str("\n    -- Basics\n");
        if let Some(val) = extract_field(config_content, "modkey") {
            let resolved = resolve_value(&val, &defines).trim_matches('"').to_string();
            if resolved == "modkey" {
                lua_output.push_str("    modkey = modkey,\n");
            } else {
                lua_output.push_str(&format!("    modkey = \"{}\",\n", resolved));
            }
        }
        if let Some(val) = extract_field(config_content, "terminal") {
            let resolved = resolve_value(&val, &defines);
            if resolved == "terminal" {
                lua_output.push_str("    terminal = terminal,\n");
            } else {
                lua_output.push_str(&format!("    terminal = {},\n", resolved));
            }
        }

        lua_output.push_str("\n    -- Workspace tags\n");
        if let Some(val) = extract_field(config_content, "tags") {
            lua_output.push_str(&format!("    tags = {},\n", convert_array_to_lua(&val)));
        }

        lua_output.push_str("\n    -- Layout symbol overrides\n");
        if let Some(val) = extract_field(config_content, "layout_symbols") {
            lua_output.push_str("    layout_symbols = ");
            lua_output.push_str(&convert_layout_symbols(&val));
            lua_output.push_str(",\n");
        }

        lua_output.push_str("\n    -- Keybindings\n");
        if let Some(val) = extract_field(config_content, "keybindings") {
            lua_output.push_str("    keybindings = ");
            lua_output.push_str(&convert_keybindings(&val, &defines));
            lua_output.push_str(",\n");
        }

        lua_output.push_str("\n    -- Status bar blocks\n");
        if let Some(val) = extract_field(config_content, "status_blocks") {
            lua_output.push_str("    status_blocks = ");
            lua_output.push_str(&convert_status_blocks(&val, &defines));
            lua_output.push_str(",\n");
        }

        lua_output.push_str("\n    -- Color schemes for bar\n");
        if let Some(val) = extract_field(config_content, "scheme_normal") {
            lua_output.push_str("    scheme_normal = ");
            lua_output.push_str(&convert_color_scheme(&val, &defines));
            lua_output.push_str(",\n");
        }
        if let Some(val) = extract_field(config_content, "scheme_occupied") {
            lua_output.push_str("    scheme_occupied = ");
            lua_output.push_str(&convert_color_scheme(&val, &defines));
            lua_output.push_str(",\n");
        }
        if let Some(val) = extract_field(config_content, "scheme_selected") {
            lua_output.push_str("    scheme_selected = ");
            lua_output.push_str(&convert_color_scheme(&val, &defines));
            lua_output.push_str(",\n");
        }

        lua_output.push_str("\n    -- Autostart commands\n");
        if let Some(val) = extract_field(config_content, "autostart") {
            let converted = convert_array_to_lua(&val);
            lua_output.push_str("    autostart = ");
            lua_output.push_str(&converted);
            lua_output.push_str(",\n");
        } else {
            lua_output.push_str("    autostart = {},\n");
        }
    }

    lua_output.push_str("}\n");

    Ok(lua_output)
}

fn extract_defines(content: &str) -> HashMap<String, String> {
    let mut defines = HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#DEFINE") {
            if let Some(rest) = trimmed.strip_prefix("#DEFINE") {
                if let Some(eq_pos) = rest.find('=') {
                    let var_name = rest[..eq_pos].trim().to_string();
                    let value = rest[eq_pos + 1..].trim().trim_end_matches(',').to_string();
                    defines.insert(var_name, value);
                }
            }
        }
    }
    defines
}

fn resolve_value(value: &str, defines: &HashMap<String, String>) -> String {
    if let Some(resolved) = defines.get(value) {
        resolved.clone()
    } else {
        value.to_string()
    }
}

fn resolve_color_value(value: &str, defines: &HashMap<String, String>) -> String {
    let resolved = resolve_value(value, defines);
    if resolved.starts_with("$color_") {
        format!("colors.{}", &resolved[7..])
    } else if value.starts_with("$color_") {
        format!("colors.{}", &value[7..])
    } else if resolved.starts_with("0x") {
        format!("\"#{}\"", &resolved[2..])
    } else {
        resolved
    }
}

fn extract_field(content: &str, field_name: &str) -> Option<String> {
    let pattern = format!("{}:", field_name);
    let cleaned_content = remove_comments(content);

    if let Some(start) = cleaned_content.find(&pattern) {
        let after_colon = &cleaned_content[start + pattern.len()..];
        let value_start = after_colon.trim_start();

        if value_start.starts_with('[') {
            extract_bracketed(value_start, '[', ']')
        } else if value_start.starts_with('(') {
            extract_bracketed(value_start, '(', ')')
        } else if value_start.starts_with('"') {
            if let Some(end) = value_start[1..].find('"') {
                Some(value_start[..end + 2].to_string())
            } else {
                None
            }
        } else {
            let end = value_start
                .find(|c: char| c == ',' || c == '\n' || c == ')')
                .unwrap_or(value_start.len());
            Some(value_start[..end].trim().to_string())
        }
    } else {
        None
    }
}

fn extract_bracketed(s: &str, open: char, close: char) -> Option<String> {
    if !s.starts_with(open) {
        return None;
    }
    let mut depth = 0;
    let mut end = 0;
    for (i, c) in s.char_indices() {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                end = i + 1;
                break;
            }
        }
    }
    if end > 0 {
        Some(s[..end].to_string())
    } else {
        None
    }
}

fn convert_array_to_lua(ron_array: &str) -> String {
    let inner = ron_array.trim_start_matches('[').trim_end_matches(']');
    let items: Vec<&str> = inner.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    format!("{{ {} }}", items.join(", "))
}

fn convert_layout_symbols(ron_array: &str) -> String {
    let mut result = String::from("{\n");
    let inner = ron_array.trim_start_matches('[').trim_end_matches(']');

    let items = extract_all_bracketed(inner, '(', ')');
    for item in items {
        let item_inner = item.trim_start_matches('(').trim_end_matches(')');
        if let (Some(name), Some(symbol)) = (extract_quoted_value(item_inner, "name"), extract_quoted_value(item_inner, "symbol")) {
            result.push_str(&format!("        {{ name = \"{}\", symbol = \"{}\" }},\n", name, symbol));
        }
    }

    result.push_str("    }");
    result
}

fn convert_keybindings(ron_array: &str, defines: &HashMap<String, String>) -> String {
    let mut result = String::from("{\n");
    let inner = ron_array.trim_start_matches('[').trim_end_matches(']');

    let items = extract_all_bracketed(inner, '(', ')');
    for item in items {
        let binding = convert_keybinding(&item, defines);
        result.push_str(&binding);
        result.push_str(",\n");
    }

    result.push_str("    }");
    result
}

fn convert_keybinding(ron_binding: &str, defines: &HashMap<String, String>) -> String {
    let inner = ron_binding.trim_start_matches('(').trim_end_matches(')');

    if inner.contains("keys:") {
        convert_keychord(inner, defines)
    } else {
        convert_single_key(inner, defines)
    }
}

fn convert_keychord(inner: &str, defines: &HashMap<String, String>) -> String {
    let mut result = String::from("        {\n            keys = {\n");

    if let Some(keys_str) = extract_field(inner, "keys") {
        let keys = extract_all_bracketed(&keys_str, '(', ')');
        for key in keys {
            let key_inner = key.trim_start_matches('(').trim_end_matches(')');
            let mods = extract_modifiers(key_inner, defines);
            let key_name = extract_key(key_inner);
            result.push_str(&format!("                {{ modifiers = {{ {} }}, key = \"{}\" }},\n", mods, key_name));
        }
    }

    result.push_str("            },\n");

    if let Some(action) = extract_identifier(inner, "action") {
        result.push_str(&format!("            action = \"{}\",\n", action));
    }

    if let Some(arg) = extract_arg(inner, defines) {
        result.push_str(&format!("            arg = {}\n", arg));
    }

    result.push_str("        }");
    result
}

fn convert_single_key(inner: &str, defines: &HashMap<String, String>) -> String {
    let mods = extract_modifiers(inner, defines);
    let key = extract_key(inner);
    let action = extract_identifier(inner, "action").unwrap_or_default();

    let mut result = format!("        {{ modifiers = {{ {} }}, key = \"{}\", action = \"{}\"", mods, key, action);

    if let Some(arg) = extract_arg(inner, defines) {
        result.push_str(&format!(", arg = {}", arg));
    }

    result.push_str(" }");
    result
}

fn extract_modifiers(content: &str, defines: &HashMap<String, String>) -> String {
    if let Some(mods_str) = extract_field(content, "modifiers") {
        let inner = mods_str.trim_start_matches('[').trim_end_matches(']').trim();
        if inner.is_empty() {
            return String::new();
        }
        let mods: Vec<String> = inner
            .split(',')
            .map(|s| {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    let resolved = resolve_value(trimmed, defines);
                    format!("\"{}\"", resolved)
                } else {
                    String::new()
                }
            })
            .filter(|s| !s.is_empty())
            .collect();
        mods.join(", ")
    } else {
        String::new()
    }
}

fn extract_key(content: &str) -> String {
    if let Some(key_str) = extract_identifier(content, "key") {
        if key_str.starts_with("Key") && key_str.len() == 4 {
            if let Some(digit) = key_str.chars().nth(3) {
                if digit.is_ascii_digit() {
                    return digit.to_string();
                }
            }
        }
        key_str
    } else {
        String::from("Return")
    }
}

fn extract_identifier(content: &str, field_name: &str) -> Option<String> {
    let pattern = format!("{}:", field_name);
    if let Some(start) = content.find(&pattern) {
        let after_colon = &content[start + pattern.len()..];
        let value_start = after_colon.trim_start();
        let end = value_start
            .find(|c: char| c == ',' || c == ')' || c == '\n')
            .unwrap_or(value_start.len());
        Some(value_start[..end].trim().to_string())
    } else {
        None
    }
}

fn extract_arg(content: &str, defines: &HashMap<String, String>) -> Option<String> {
    if let Some(arg_str) = extract_field(content, "arg") {
        let resolved = resolve_value(&arg_str, defines);
        if resolved.starts_with('[') {
            Some(convert_array_to_lua(&resolved))
        } else if resolved.starts_with('"') || resolved.parse::<i32>().is_ok() || resolved.starts_with("0x") {
            Some(resolved)
        } else {
            Some(format!("\"{}\"", resolved))
        }
    } else {
        None
    }
}

fn convert_status_blocks(ron_array: &str, defines: &HashMap<String, String>) -> String {
    let mut result = String::from("{\n");
    let inner = ron_array.trim_start_matches('[').trim_end_matches(']');

    let items = extract_all_bracketed(inner, '(', ')');
    for item in items {
        let block = convert_status_block(&item, defines);
        if !block.trim().ends_with("{\n        }") {
            result.push_str(&block);
            result.push_str(",\n");
        }
    }

    result.push_str("    }");
    result
}

fn convert_status_block(ron_block: &str, defines: &HashMap<String, String>) -> String {
    let mut result = String::from("        {\n");
    let inner = ron_block.trim_start_matches('(').trim_end_matches(')');

    if let Some(format) = extract_field(inner, "format") {
        result.push_str(&format!("            format = {},\n", format));
    }
    if let Some(command) = extract_field(inner, "command") {
        result.push_str(&format!("            command = {},\n", command));
    }
    if let Some(command_arg) = extract_field(inner, "command_arg") {
        result.push_str(&format!("            command_arg = {},\n", command_arg));
    }
    if inner.contains("battery_formats:") {
        if let Some(battery_str) = extract_field(inner, "battery_formats") {
            result.push_str("            battery_formats = {\n");
            let battery_inner = battery_str.trim_start_matches('(').trim_end_matches(')');
            if let Some(charging) = extract_quoted_value(battery_inner, "charging") {
                result.push_str(&format!("                charging = \"{}\",\n", charging));
            }
            if let Some(discharging) = extract_quoted_value(battery_inner, "discharging") {
                result.push_str(&format!("                discharging = \"{}\",\n", discharging));
            }
            if let Some(full) = extract_quoted_value(battery_inner, "full") {
                result.push_str(&format!("                full = \"{}\"\n", full));
            }
            result.push_str("            },\n");
        }
    }
    if let Some(interval) = extract_field(inner, "interval_secs") {
        let interval_val = if interval.len() > 10 {
            "999999999".to_string()
        } else {
            interval
        };
        result.push_str(&format!("            interval_secs = {},\n", interval_val));
    }
    if let Some(color) = extract_field(inner, "color") {
        let resolved = resolve_color_value(&color, defines);
        result.push_str(&format!("            color = {},\n", resolved));
    }
    if let Some(underline) = extract_field(inner, "underline") {
        result.push_str(&format!("            underline = {}\n", underline));
    }

    result.push_str("        }");
    result
}

fn convert_color_scheme(ron_scheme: &str, defines: &HashMap<String, String>) -> String {
    let mut result = String::from("{\n");
    let inner = ron_scheme.trim_start_matches('(').trim_end_matches(')');

    if let Some(fg) = extract_field(inner, "foreground") {
        let resolved = resolve_color_value(&fg, defines);
        result.push_str(&format!("        foreground = {},\n", resolved));
    }
    if let Some(bg) = extract_field(inner, "background") {
        let resolved = resolve_color_value(&bg, defines);
        result.push_str(&format!("        background = {},\n", resolved));
    }
    if let Some(ul) = extract_field(inner, "underline") {
        let resolved = resolve_color_value(&ul, defines);
        result.push_str(&format!("        underline = {}\n", resolved));
    }

    result.push_str("    }");
    result
}

fn extract_all_bracketed(s: &str, open: char, close: char) -> Vec<String> {
    let mut results = Vec::new();
    let mut depth = 0;
    let mut start = None;

    let cleaned = remove_comments(s);

    for (i, c) in cleaned.char_indices() {
        if c == open {
            if depth == 0 {
                start = Some(i);
            }
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                if let Some(start_idx) = start {
                    results.push(cleaned[start_idx..=i].to_string());
                    start = None;
                }
            }
        }
    }

    results
}

fn remove_comments(s: &str) -> String {
    let mut result = String::new();
    for line in s.lines() {
        let mut in_string = false;
        let mut comment_start = None;

        for (i, c) in line.char_indices() {
            if c == '"' && (i == 0 || line.chars().nth(i - 1) != Some('\\')) {
                in_string = !in_string;
            }
            if !in_string && i + 1 < line.len() && &line[i..i + 2] == "//" {
                comment_start = Some(i);
                break;
            }
        }

        if let Some(pos) = comment_start {
            result.push_str(&line[..pos]);
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }
    result
}

fn extract_quoted_value(content: &str, field_name: &str) -> Option<String> {
    let pattern = format!("{}:", field_name);
    if let Some(start) = content.find(&pattern) {
        let after_colon = &content[start + pattern.len()..];
        let trimmed = after_colon.trim_start();
        if trimmed.starts_with('"') {
            if let Some(end) = trimmed[1..].find('"') {
                return Some(trimmed[1..end + 1].to_string());
            }
        }
    }
    None
}
