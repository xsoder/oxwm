-- OXWM Test Configuration File (Lua)
-- This config uses Mod1 (Alt) as the modkey for testing in Xephyr

---@class LayoutSymbol
---@field name string The internal layout name (e.g., "tiling", "normie")
---@field symbol string The display symbol for the layout (e.g., "[T]", "[F]")

---@class KeyBinding
---@field modifiers string[] List of modifiers: "Mod", "Mod1"-"Mod5", "Shift", "Control"
---@field key string The key name (e.g., "Return", "Q", "1", "Space")
---@field action string Action to perform (see below for list)
---@field arg? string|number|string[] Optional argument for the action

---@class KeyChord
---@field keys {modifiers: string[], key: string}[] Sequence of keys to press
---@field action string Action to perform when chord completes
---@field arg? string|number|string[] Optional argument for the action

---@class StatusBlock
---@field format string Display format with {} placeholders
---@field command string Command type: "Ram", "DateTime", "Shell", "Static", "Battery"
---@field command_arg? string Argument for command (shell command, date format, etc.)
---@field interval_secs number Update interval in seconds
---@field color number Color as hex number (e.g., 0xff0000 for red)
---@field underline boolean Whether to show underline
---@field battery_formats? {charging: string, discharging: string, full: string} Battery format strings

---@class ColorScheme
---@field foreground number Foreground color (hex)
---@field background number Background color (hex)
---@field underline number Underline color (hex)

---@class Config
---@field border_width number Width of window borders in pixels
---@field border_focused number Color for focused window border (hex)
---@field border_unfocused number Color for unfocused window border (hex)
---@field font string Font specification (e.g., "monospace:style=Bold:size=10")
---@field gaps_enabled boolean Whether gaps are enabled
---@field gap_inner_horizontal number Inner horizontal gap size
---@field gap_inner_vertical number Inner vertical gap size
---@field gap_outer_horizontal number Outer horizontal gap size
---@field gap_outer_vertical number Outer vertical gap size
---@field modkey string Main modifier key (e.g., "Mod4" for Super)
---@field terminal string Terminal emulator command
---@field tags string[] List of workspace tag names
---@field layout_symbols LayoutSymbol[] Custom layout symbols
---@field keybindings (KeyBinding|KeyChord)[] List of keybindings
---@field status_blocks StatusBlock[] Status bar configuration blocks
---@field scheme_normal ColorScheme Color scheme for normal tags
---@field scheme_occupied ColorScheme Color scheme for occupied tags
---@field scheme_selected ColorScheme Color scheme for selected tag
---@field autostart string[] Commands to run on startup

-- Available Actions:
--   "Spawn" - Launch a program (arg: string or string[])
--   "KillClient" - Close focused window
--   "FocusStack" - Focus next/prev in stack (arg: 1 or -1)
--   "FocusDirection" - Focus by direction (arg: 0=up, 1=down, 2=left, 3=right)
--   "SwapDirection" - Swap window by direction (arg: 0=up, 1=down, 2=left, 3=right)
--   "Quit" - Exit window manager
--   "Restart" - Restart and reload config
--   "Recompile" - Recompile and restart
--   "ViewTag" - Switch to tag (arg: tag index)
--   "MoveToTag" - Move window to tag (arg: tag index)
--   "ToggleGaps" - Toggle gaps on/off
--   "ToggleFullScreen" - Toggle fullscreen mode
--   "ToggleFloating" - Toggle floating mode
--   "ChangeLayout" - Switch to specific layout (arg: layout name)
--   "CycleLayout" - Cycle through layouts
--   "FocusMonitor" - Focus monitor (arg: 1 or -1)
--   "SmartMoveWin" - Smart window movement
--   "ExchangeClient" - Exchange window positions

-- Available Modifiers:
--   "Mod"     - Replaced with configured modkey
--   "Mod1"    - Alt key
--   "Mod4"    - Super/Windows key
--   "Shift"   - Shift key
--   "Control" - Control key
--   "Mod2", "Mod3", "Mod5" - Additional modifiers

-- Define variables for easy customization
local terminal = "st"
local modkey = "Mod1"  -- Alt key for Xephyr testing
local secondary_modkey = "Control"

-- Color palette (Tokyo Night theme)
local colors = {
    blue = 0x6dade3,
    grey = 0xbbbbbb,
    green = 0x9ece6a,
    red = 0xf7768e,
    cyan = 0x0db9d7,
    purple = 0xad8ee6,
    lavender = 0xa9b1d6,
    bg = 0x1a1b26,
    fg = 0xbbbbbb,
    light_blue = 0x7aa2f7,
}

-- Main configuration table
---@type Config
config = {
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
    modkey = modkey,
    terminal = terminal,

    -- Workspace tags
    tags = {"1", "2", "3", "4", "5", "6", "7", "8", "9"},

    -- Layout symbol overrides
    layout_symbols = {
        {name = "tiling", symbol = "[T]"},
        {name = "normie", symbol = "[F]"},
    },

    -- Keybindings
    -- Using Mod1 (Alt) for Xephyr testing so it doesn't conflict with host WM
    keybindings = {
        -- Basic window management
        {modifiers = {"Mod"}, key = "Return", action = "Spawn", arg = terminal},
        {modifiers = {"Mod"}, key = "D", action = "Spawn", arg = {"sh", "-c", "dmenu_run -l 10"}},
        {modifiers = {"Mod"}, key = "S", action = "Spawn", arg = {"sh", "-c", "maim -s | xclip -selection clipboard -t image/png"}},
        {modifiers = {"Mod"}, key = "Q", action = "KillClient"},
        {modifiers = {"Mod", "Shift"}, key = "F", action = "ToggleFullScreen"},
        {modifiers = {"Mod", "Shift"}, key = "Space", action = "ToggleFloating"},

        -- Layout management
        {modifiers = {"Mod"}, key = "F", action = "ChangeLayout", arg = "normie"},
        {modifiers = {"Mod"}, key = "C", action = "ChangeLayout", arg = "tiling"},
        {modifiers = {secondary_modkey}, key = "N", action = "CycleLayout"},
        {modifiers = {"Mod"}, key = "A", action = "ToggleGaps"},

        -- WM control
        {modifiers = {"Mod", "Shift"}, key = "Q", action = "Quit"},
        {modifiers = {"Mod", "Shift"}, key = "R", action = "Restart"},

        -- Focus movement (vim-style hjkl)
        {modifiers = {"Mod"}, key = "H", action = "FocusDirection", arg = 2}, -- left
        {modifiers = {"Mod"}, key = "J", action = "FocusDirection", arg = 1}, -- down
        {modifiers = {"Mod"}, key = "K", action = "FocusDirection", arg = 0}, -- up
        {modifiers = {"Mod"}, key = "L", action = "FocusDirection", arg = 3}, -- right

        -- Window swapping (vim-style hjkl)
        {modifiers = {"Mod", "Shift"}, key = "H", action = "SwapDirection", arg = 2}, -- left
        {modifiers = {"Mod", "Shift"}, key = "J", action = "SwapDirection", arg = 1}, -- down
        {modifiers = {"Mod", "Shift"}, key = "K", action = "SwapDirection", arg = 0}, -- up
        {modifiers = {"Mod", "Shift"}, key = "L", action = "SwapDirection", arg = 3}, -- right

        -- Monitor focus
        {modifiers = {"Mod"}, key = "Comma", action = "FocusMonitor", arg = -1},
        {modifiers = {"Mod"}, key = "Period", action = "FocusMonitor", arg = 1},

        -- View tags
        {modifiers = {"Mod"}, key = "1", action = "ViewTag", arg = 0},
        {modifiers = {"Mod"}, key = "2", action = "ViewTag", arg = 1},
        {modifiers = {"Mod"}, key = "3", action = "ViewTag", arg = 2},
        {modifiers = {"Mod"}, key = "4", action = "ViewTag", arg = 3},
        {modifiers = {"Mod"}, key = "5", action = "ViewTag", arg = 4},
        {modifiers = {"Mod"}, key = "6", action = "ViewTag", arg = 5},
        {modifiers = {"Mod"}, key = "7", action = "ViewTag", arg = 6},
        {modifiers = {"Mod"}, key = "8", action = "ViewTag", arg = 7},
        {modifiers = {"Mod"}, key = "9", action = "ViewTag", arg = 8},

        -- Move window to tag
        {modifiers = {"Mod", "Shift"}, key = "1", action = "MoveToTag", arg = 0},
        {modifiers = {"Mod", "Shift"}, key = "2", action = "MoveToTag", arg = 1},
        {modifiers = {"Mod", "Shift"}, key = "3", action = "MoveToTag", arg = 2},
        {modifiers = {"Mod", "Shift"}, key = "4", action = "MoveToTag", arg = 3},
        {modifiers = {"Mod", "Shift"}, key = "5", action = "MoveToTag", arg = 4},
        {modifiers = {"Mod", "Shift"}, key = "6", action = "MoveToTag", arg = 5},
        {modifiers = {"Mod", "Shift"}, key = "7", action = "MoveToTag", arg = 6},
        {modifiers = {"Mod", "Shift"}, key = "8", action = "MoveToTag", arg = 7},
        {modifiers = {"Mod", "Shift"}, key = "9", action = "MoveToTag", arg = 8},

        -- Example keychord: Alt+Space, then T to spawn terminal
        {
            keys = {
                {modifiers = {"Mod1"}, key = "Space"},
                {modifiers = {}, key = "T"}
            },
            action = "Spawn",
            arg = terminal
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
            interval_secs = 999999999, -- Very large number for static blocks
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
        underline = 0x444444
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
    autostart = {},
}
