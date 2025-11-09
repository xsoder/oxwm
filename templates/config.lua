-- OXWM Configuration File (Lua)
-- Migrated from config.ron
-- Edit this file and reload with Mod+Shift+R (no compilation needed!)

local terminal = "st"
local modkey = "Mod4"
local secondary_modkey = "Mod1"

-- Color palette
local colors = {
    fg = "#bbbbbb",
    red = "#f7768e",
    bg = "#1a1b26",
    cyan = "#0db9d7",
    green = "#9ece6a",
    lavender = "#a9b1d6",
    light_blue = "#7aa2f7",
    grey = "#bbbbbb",
    blue = "#6dade3",
    purple = "#ad8ee6",
}

-- Main configuration table
return {
    -- Appearance
    border_width = 2,
    border_focused = colors.blue,
    border_unfocused = colors.grey,
    font = "monospace:style=Bold:size=10",

    -- Window gaps
    gaps_enabled = true,
    gap_inner_horizontal = 5,
    gap_inner_vertical = 5,
    gap_outer_horizontal = 5,
    gap_outer_vertical = 5,

    -- Basics
    modkey = "Mod4",
    terminal = "st",

    -- Workspace tags
    tags = { "1", "2", "3", "4", "5", "6", "7", "8", "9" },

    -- Layout symbol overrides
    layout_symbols = {
        { name = "tiling", symbol = "[T]" },
        { name = "normie", symbol = "[F]" },
    },

    -- Keybindings
    keybindings = {
        { modifiers = { "Mod4" }, key = "Return", action = "Spawn", arg = "st" },
        { modifiers = { "Mod4" }, key = "D", action = "Spawn", arg = { "sh", "-c", "dmenu_run -l 10" } },
        { modifiers = { "Mod4" }, key = "S", action = "Spawn", arg = { "sh", "-c", "maim -s | xclip -selection clipboard -t image/png" } },
        { modifiers = { "Mod4" }, key = "Q", action = "KillClient" },
        { modifiers = { "Mod4", "Shift" }, key = "Slash", action = "ShowKeybindOverlay" },
        { modifiers = { "Mod4", "Shift" }, key = "F", action = "ToggleFullScreen" },
        { modifiers = { "Mod4", "Shift" }, key = "Space", action = "ToggleFloating" },
        { modifiers = { "Mod4" }, key = "F", action = "ChangeLayout", arg = "normie" },
        { modifiers = { "Mod4" }, key = "C", action = "ChangeLayout", arg = "tiling" },
        { modifiers = { "Mod1" }, key = "N", action = "CycleLayout" },
        { modifiers = { "Mod4" }, key = "A", action = "ToggleGaps" },
        { modifiers = { "Mod4", "Shift" }, key = "Q", action = "Quit" },
        { modifiers = { "Mod4", "Shift" }, key = "R", action = "Restart" },
        { modifiers = { "Mod4" }, key = "H", action = "FocusDirection", arg = 2 },
        { modifiers = { "Mod4" }, key = "J", action = "FocusDirection", arg = 1 },
        { modifiers = { "Mod4" }, key = "K", action = "FocusDirection", arg = 0 },
        { modifiers = { "Mod4" }, key = "L", action = "FocusDirection", arg = 3 },
        { modifiers = { "Mod4" }, key = "Comma", action = "FocusMonitor", arg = -1 },
        { modifiers = { "Mod4" }, key = "Period", action = "FocusMonitor", arg = 1 },
        { modifiers = { "Mod4" }, key = "1", action = "ViewTag", arg = 0 },
        { modifiers = { "Mod4" }, key = "2", action = "ViewTag", arg = 1 },
        { modifiers = { "Mod4" }, key = "3", action = "ViewTag", arg = 2 },
        { modifiers = { "Mod4" }, key = "4", action = "ViewTag", arg = 3 },
        { modifiers = { "Mod4" }, key = "5", action = "ViewTag", arg = 4 },
        { modifiers = { "Mod4" }, key = "6", action = "ViewTag", arg = 5 },
        { modifiers = { "Mod4" }, key = "7", action = "ViewTag", arg = 6 },
        { modifiers = { "Mod4" }, key = "8", action = "ViewTag", arg = 7 },
        { modifiers = { "Mod4" }, key = "9", action = "ViewTag", arg = 8 },
        { modifiers = { "Mod4", "Shift" }, key = "1", action = "MoveToTag", arg = 0 },
        { modifiers = { "Mod4", "Shift" }, key = "2", action = "MoveToTag", arg = 1 },
        { modifiers = { "Mod4", "Shift" }, key = "3", action = "MoveToTag", arg = 2 },
        { modifiers = { "Mod4", "Shift" }, key = "4", action = "MoveToTag", arg = 3 },
        { modifiers = { "Mod4", "Shift" }, key = "5", action = "MoveToTag", arg = 4 },
        { modifiers = { "Mod4", "Shift" }, key = "6", action = "MoveToTag", arg = 5 },
        { modifiers = { "Mod4", "Shift" }, key = "7", action = "MoveToTag", arg = 6 },
        { modifiers = { "Mod4", "Shift" }, key = "8", action = "MoveToTag", arg = 7 },
        { modifiers = { "Mod4", "Shift" }, key = "9", action = "MoveToTag", arg = 8 },
        { modifiers = { "Mod4", "Shift" }, key = "H", action = "SwapDirection", arg = 2 },
        { modifiers = { "Mod4", "Shift" }, key = "J", action = "SwapDirection", arg = 1 },
        { modifiers = { "Mod4", "Shift" }, key = "K", action = "SwapDirection", arg = 0 },
        { modifiers = { "Mod4", "Shift" }, key = "L", action = "SwapDirection", arg = 3 },
        {
            keys = {
                { modifiers = { "Mod4" }, key = "Space" },
                { modifiers = {  }, key = "T" },
            },
            action = "Spawn",
            arg = "st"
        },
    },

    -- Status bar blocks
    status_blocks = {
        {
            format = "Ram: {used}/{total} GB",
            command = "Ram",
            interval_secs = 5,
            color = colors.light_blue,
            underline = true
        },
        {
            format = " │  ",
            command = "Static",
            interval_secs = 999999999,
            color = colors.lavender,
            underline = false
        },
        {
            format = "Kernel: {}",
            command = "Shell",
            command_arg = "uname -r",
            interval_secs = 999999999,
            color = colors.red,
            underline = true
        },
        {
            format = " │  ",
            command = "Static",
            interval_secs = 999999999,
            color = colors.lavender,
            underline = false
        },
        {
            format = "{}",
            command = "DateTime",
            command_arg = "%a, %b %d - %-I:%M %P",
            interval_secs = 1,
            color = colors.cyan,
            underline = true
        },
    },

    -- Color schemes for bar
    scheme_normal = {
        foreground = colors.fg,
        background = colors.bg,
        underline = "#444444"
    },
    scheme_occupied = {
        foreground = colors.cyan,
        background = colors.bg,
        underline = colors.cyan
    },
    scheme_selected = {
        foreground = colors.cyan,
        background = colors.bg,
        underline = colors.purple
    },

    -- Autostart commands
    autostart = {  },
}
