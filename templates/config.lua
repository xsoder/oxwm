---@meta
-------------------------------------------------------------------------------
-- OXWM Configuration File
-------------------------------------------------------------------------------
-- This is the default configuration for OXWM, a dynamic window manager.
-- Edit this file and reload with Mod+Shift+R (no compilation needed!)
--
-- For more information about configuring OXWM, see the documentation.
-- The Lua Language Server provides autocomplete and type checking.
-------------------------------------------------------------------------------

---Load type definitions for LSP
---@module 'oxwm'

-------------------------------------------------------------------------------
-- Variables
-------------------------------------------------------------------------------
-- Define your variables here for easy customization throughout the config.
-- This makes it simple to change keybindings, colors, and settings in one place.

-- Modifier key: "Mod4" is the Super/Windows key, "Mod1" is Alt
local modkey = "Mod4"

-- Terminal emulator command
local terminal = "st"

-- Color palette - customize these to match your theme
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

-- Workspace tags - can be numbers, names, or icons (requires a Nerd Font)
local tags = { "1", "2", "3", "4", "5", "6", "7", "8", "9" }

-- Font for the status bar (use "fc-list" to see available fonts)
local bar_font = "monospace:style=Bold:size=10"

-------------------------------------------------------------------------------
-- Basic Settings
-------------------------------------------------------------------------------
oxwm.set_terminal(terminal)
oxwm.set_modkey(modkey)
oxwm.set_tags(tags)

-------------------------------------------------------------------------------
-- Layouts
-------------------------------------------------------------------------------
-- Set custom symbols for layouts (displayed in the status bar)
-- Available layouts: "tiling" (master-stack), "normie" (floating)
oxwm.set_layout_symbol("tiling", "[T]")
oxwm.set_layout_symbol("normie", "[F]")

-------------------------------------------------------------------------------
-- Appearance
-------------------------------------------------------------------------------
-- Border configuration
oxwm.border.set_width(2)                        -- Width in pixels
oxwm.border.set_focused_color(colors.blue)      -- Color of focused window border
oxwm.border.set_unfocused_color(colors.grey)    -- Color of unfocused window borders

-- Gap configuration (space between windows and screen edges)
oxwm.gaps.set_enabled(true)                     -- Enable or disable gaps
oxwm.gaps.set_inner(5, 5)                       -- Inner gaps (horizontal, vertical) in pixels
oxwm.gaps.set_outer(5, 5)                       -- Outer gaps (horizontal, vertical) in pixels

-------------------------------------------------------------------------------
-- Status Bar Configuration
-------------------------------------------------------------------------------
-- Font configuration
oxwm.bar.set_font(bar_font)

-- Bar color schemes (for workspace tag display)
-- Parameters: foreground, background, border
oxwm.bar.set_scheme_normal(colors.fg, colors.bg, "#444444")        -- Unoccupied tags
oxwm.bar.set_scheme_occupied(colors.cyan, colors.bg, colors.cyan)  -- Occupied tags
oxwm.bar.set_scheme_selected(colors.cyan, colors.bg, colors.purple) -- Currently selected tag

-------------------------------------------------------------------------------
-- Keybindings
-------------------------------------------------------------------------------
-- Keybindings are defined using oxwm.key.bind(modifiers, key, action)
-- Modifiers: {"Mod4"}, {"Mod1"}, {"Shift"}, {"Control"}, or combinations like {"Mod4", "Shift"}
-- Keys: Use uppercase for letters (e.g., "Return", "H", "J", "K", "L")
-- Actions: Functions that return actions (e.g., oxwm.spawn(), oxwm.client.kill())
--
-- A list of available keysyms can be found in the X11 keysym definitions.
-- Common keys: Return, Space, Tab, Escape, Backspace, Delete, Left, Right, Up, Down

-- Basic window management
oxwm.key.bind({ modkey }, "Return", oxwm.spawn(terminal))                          -- Spawn terminal
oxwm.key.bind({ modkey }, "D", oxwm.spawn({ "sh", "-c", "dmenu_run -l 10" }))     -- Application launcher
oxwm.key.bind({ modkey }, "S", oxwm.spawn({ "sh", "-c", "maim -s | xclip -selection clipboard -t image/png" }))  -- Screenshot selection
oxwm.key.bind({ modkey }, "Q", oxwm.client.kill())                                 -- Close focused window

-- Keybind overlay - Shows important keybindings on screen
oxwm.key.bind({ modkey, "Shift" }, "Slash", oxwm.show_keybinds())

-- Window state toggles
oxwm.key.bind({ modkey, "Shift" }, "F", oxwm.client.toggle_fullscreen())           -- Toggle fullscreen
oxwm.key.bind({ modkey, "Shift" }, "Space", oxwm.client.toggle_floating())         -- Toggle floating mode

-- Layout management
oxwm.key.bind({ modkey }, "F", oxwm.layout.set("normie"))                          -- Set floating layout
oxwm.key.bind({ modkey }, "C", oxwm.layout.set("tiling"))                          -- Set tiling layout
oxwm.key.bind({ "Mod1" }, "N", oxwm.layout.cycle())                                -- Cycle through layouts

-- Gaps toggle
oxwm.key.bind({ modkey }, "A", oxwm.toggle_gaps())                                 -- Toggle gaps on/off

-- Window manager controls
oxwm.key.bind({ modkey, "Shift" }, "Q", oxwm.quit())                               -- Quit OXWM
oxwm.key.bind({ modkey, "Shift" }, "R", oxwm.restart())                            -- Restart OXWM (reloads config)

-- Focus movement (vim keys)
oxwm.key.bind({ modkey }, "H", oxwm.client.focus_direction("left"))                -- Focus window to the left
oxwm.key.bind({ modkey }, "J", oxwm.client.focus_direction("down"))                -- Focus window below
oxwm.key.bind({ modkey }, "K", oxwm.client.focus_direction("up"))                  -- Focus window above
oxwm.key.bind({ modkey }, "L", oxwm.client.focus_direction("right"))               -- Focus window to the right

-- Multi-monitor support
oxwm.key.bind({ modkey }, "Comma", oxwm.focus_monitor(-1))                         -- Focus previous monitor
oxwm.key.bind({ modkey }, "Period", oxwm.focus_monitor(1))                         -- Focus next monitor

-- Workspace (tag) navigation
-- Switch to workspace N (tags are 0-indexed, so tag "1" is index 0)
oxwm.key.bind({ modkey }, "1", oxwm.tag.view(0))
oxwm.key.bind({ modkey }, "2", oxwm.tag.view(1))
oxwm.key.bind({ modkey }, "3", oxwm.tag.view(2))
oxwm.key.bind({ modkey }, "4", oxwm.tag.view(3))
oxwm.key.bind({ modkey }, "5", oxwm.tag.view(4))
oxwm.key.bind({ modkey }, "6", oxwm.tag.view(5))
oxwm.key.bind({ modkey }, "7", oxwm.tag.view(6))
oxwm.key.bind({ modkey }, "8", oxwm.tag.view(7))
oxwm.key.bind({ modkey }, "9", oxwm.tag.view(8))

-- Move focused window to workspace N
oxwm.key.bind({ modkey, "Shift" }, "1", oxwm.tag.move_to(0))
oxwm.key.bind({ modkey, "Shift" }, "2", oxwm.tag.move_to(1))
oxwm.key.bind({ modkey, "Shift" }, "3", oxwm.tag.move_to(2))
oxwm.key.bind({ modkey, "Shift" }, "4", oxwm.tag.move_to(3))
oxwm.key.bind({ modkey, "Shift" }, "5", oxwm.tag.move_to(4))
oxwm.key.bind({ modkey, "Shift" }, "6", oxwm.tag.move_to(5))
oxwm.key.bind({ modkey, "Shift" }, "7", oxwm.tag.move_to(6))
oxwm.key.bind({ modkey, "Shift" }, "8", oxwm.tag.move_to(7))
oxwm.key.bind({ modkey, "Shift" }, "9", oxwm.tag.move_to(8))

-- Swap windows in direction (vim keys with Shift)
oxwm.key.bind({ modkey, "Shift" }, "H", oxwm.client.swap_direction("left"))        -- Swap with window to the left
oxwm.key.bind({ modkey, "Shift" }, "J", oxwm.client.swap_direction("down"))        -- Swap with window below
oxwm.key.bind({ modkey, "Shift" }, "K", oxwm.client.swap_direction("up"))          -- Swap with window above
oxwm.key.bind({ modkey, "Shift" }, "L", oxwm.client.swap_direction("right"))       -- Swap with window to the right

-------------------------------------------------------------------------------
-- Advanced: Keychords
-------------------------------------------------------------------------------
-- Keychords allow you to bind multiple-key sequences (like Emacs or Vim)
-- Format: {{modifiers}, key1}, {{modifiers}, key2}, ...
-- Example: Press Mod4+Space, then release and press T to spawn a terminal
oxwm.key.chord({
    { { modkey }, "Space" },
    { {},         "T" }
}, oxwm.spawn(terminal))

-------------------------------------------------------------------------------
-- Status Bar Blocks
-------------------------------------------------------------------------------
-- Add informational blocks to the status bar
-- Format: oxwm.bar.add_block(format, type, data, update_interval, color, separator)
--   format: Display format with {} placeholders
--   type: Block type ("Ram", "DateTime", "Shell", "Static", "Battery")
--   data: Type-specific data (command for Shell, format for DateTime, etc.)
--   update_interval: Seconds between updates (large number for static content)
--   color: Text color (from color palette)
--   separator: Whether to add space after this block

oxwm.bar.add_block("Ram: {used}/{total} GB", "Ram", nil, 5, colors.light_blue, true)
oxwm.bar.add_block(" │  ", "Static", " │  ", 999999999, colors.lavender, false)
oxwm.bar.add_block("Kernel: {}", "Shell", "uname -r", 999999999, colors.red, true)
oxwm.bar.add_block(" │  ", "Static", " │  ", 999999999, colors.lavender, false)
oxwm.bar.add_block("{}", "DateTime", "%a, %b %d - %-I:%M %P", 1, colors.cyan, true)

-- Uncomment to add battery status (useful for laptops)
-- oxwm.bar.add_block("Bat: {}%", "Battery", {
--     charging = "⚡ Bat: {}%",
--     discharging = "🔋 Bat: {}%",
--     full = "✓ Bat: {}%"
-- }, 30, colors.green, true)

-------------------------------------------------------------------------------
-- Autostart
-------------------------------------------------------------------------------
-- Commands to run once when OXWM starts
-- Uncomment and modify these examples, or add your own

-- oxwm.autostart("picom")                                  -- Compositor for transparency and effects
-- oxwm.autostart("feh --bg-scale ~/wallpaper.jpg")        -- Set wallpaper
-- oxwm.autostart("dunst")                                  -- Notification daemon
-- oxwm.autostart("nm-applet")                              -- Network manager applet
