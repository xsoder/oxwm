---@meta
-------------------------------------------------------------------------------
-- OXWM Configuration File
-------------------------------------------------------------------------------
-- This is the default configuration for OXWM, a dynamic window manager.
-- Edit this file and reload with Mod+Shift+R (no compilation needed)
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

-- Terminal emulator command (defualts to alacritty)
local terminal = "alacritty"

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
-- local tags = { "", "󰊯", "", "", "󰙯", "󱇤", "", "󱘶", "󰧮" } -- Example of nerd font icon tags

-- Font for the status bar (use "fc-list" to see available fonts)
local bar_font = "monospace:style=Bold:size=10"

-- Define your blocks
-- Similar to widgets in qtile, or dwmblocks
local blocks = {
    oxwm.bar.block.ram({
        format = "Ram: {used}/{total} GB",
        interval = 5,
        color = colors.light_blue,
        underline = true,
    }),
    oxwm.bar.block.static({
        format = "{}",
        text = " │  ",
        interval = 999999999,
        color = colors.lavender,
        underline = false,
    }),
    oxwm.bar.block.shell({
        format = "Kernel: {}",
        command = "uname -r",
        interval = 999999999,
        color = colors.red,
        underline = true,
    }),
    oxwm.bar.block.static({
        format = "{}",
        text = " │  ",
        interval = 999999999,
        color = colors.lavender,
        underline = false,
    }),
    oxwm.bar.block.datetime({
        format = "{}",
        date_format = "%a, %b %d - %-I:%M %P",
        interval = 1,
        color = colors.cyan,
        underline = true,
    }),
    -- Uncomment to add battery status (useful for laptops)
    -- oxwm.bar.block.battery({
    --     format = "Bat: {}%",
    --     charging = "⚡ Bat: {}%",
    --     discharging = "- Bat: {}%",
    --     full = "✓ Bat: {}%",
    --     interval = 30,
    --     color = colors.green,
    --     underline = true,
    -- }),
};

-------------------------------------------------------------------------------
-- Basic Settings
-------------------------------------------------------------------------------
oxwm.set_terminal(terminal)
oxwm.set_modkey(modkey) -- This is for Mod + mouse binds, such as drag/resize
oxwm.set_tags(tags)

-------------------------------------------------------------------------------
-- Layouts
-------------------------------------------------------------------------------
-- Set custom symbols for layouts (displayed in the status bar)
-- Available layouts: "tiling", "normie" (floating), "grid", "monocle", "tabbed"
oxwm.set_layout_symbol("tiling", "[T]")
oxwm.set_layout_symbol("normie", "[F]")
oxwm.set_layout_symbol("tabbed", "[=]")

-------------------------------------------------------------------------------
-- Appearance
-------------------------------------------------------------------------------
-- Border configuration
oxwm.border.set_width(2)                     -- Width in pixels
oxwm.border.set_focused_color(colors.blue)   -- Color of focused window border
oxwm.border.set_unfocused_color(colors.grey) -- Color of unfocused window borders

-- Gap configuration (space between windows and screen edges)
oxwm.gaps.set_enabled(true) -- Enable or disable gaps
oxwm.gaps.set_inner(5, 5)   -- Inner gaps (horizontal, vertical) in pixels
oxwm.gaps.set_outer(5, 5)   -- Outer gaps (horizontal, vertical) in pixels

-------------------------------------------------------------------------------
-- Window Rules
-------------------------------------------------------------------------------
-- Rules allow you to automatically configure windows based on their properties
-- You can match windows by class, instance, title, or role
-- Available properties: floating, tag, fullscreen, etc.
--
-- Common use cases:
-- - Force floating for certain applications (dialogs, utilities)
-- - Send specific applications to specific workspaces
-- - Configure window behavior based on title or class

-- Examples (uncomment to use):
-- oxwm.rule.add({ class = "firefox", title = "Library", floating = true })  -- Make Firefox Library window floating
-- oxwm.rule.add({ instance = "gimp", tag = 5 })                             -- Send GIMP to workspace 6 (0-indexed)
-- oxwm.rule.add({ instance = "mpv", floating = true })                      -- Make mpv always float

-- To find window properties, use xprop and click on the window
-- WM_CLASS(STRING) shows both instance and class (instance, class)

-------------------------------------------------------------------------------
-- Status Bar Configuration
-------------------------------------------------------------------------------
-- Font configuration
oxwm.bar.set_font(bar_font)

-- Set your blocks here (defined above)
oxwm.bar.set_blocks(blocks)

-- Bar color schemes (for workspace tag display)
-- Parameters: foreground, background, border
oxwm.bar.set_scheme_normal(colors.fg, colors.bg, "#444444")         -- Unoccupied tags
oxwm.bar.set_scheme_occupied(colors.cyan, colors.bg, colors.cyan)   -- Occupied tags
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
oxwm.key.bind({ modkey }, "Return", oxwm.spawn_terminal())                                                      -- Spawn terminal
oxwm.key.bind({ modkey }, "D", oxwm.spawn({ "sh", "-c", "dmenu_run -l 10" }))                                   -- Application launcher
oxwm.key.bind({ modkey }, "S", oxwm.spawn({ "sh", "-c", "maim -s | xclip -selection clipboard -t image/png" })) -- Screenshot selection
oxwm.key.bind({ modkey }, "Q", oxwm.client.kill())                                                              -- Close focused window

-- Keybind overlay - Shows important keybindings on screen
oxwm.key.bind({ modkey, "Shift" }, "Slash", oxwm.show_keybinds())

-- Window state toggles
oxwm.key.bind({ modkey, "Shift" }, "F", oxwm.client.toggle_fullscreen())   -- Toggle fullscreen
oxwm.key.bind({ modkey, "Shift" }, "Space", oxwm.client.toggle_floating()) -- Toggle floating mode

-- Layout management
oxwm.key.bind({ modkey }, "F", oxwm.layout.set("normie")) -- Set floating layout
oxwm.key.bind({ modkey }, "C", oxwm.layout.set("tiling")) -- Set tiling layout
oxwm.key.bind({ modkey }, "N", oxwm.layout.cycle())       -- Cycle through layouts

-- Master area controls (tiling layout)
oxwm.key.bind({ modkey }, "H", oxwm.set_master_factor(-5))   -- Decrease master area width
oxwm.key.bind({ modkey }, "L", oxwm.set_master_factor(5))    -- Increase master area width
oxwm.key.bind({ modkey }, "I", oxwm.inc_num_master(1))       -- Increment number of master windows
oxwm.key.bind({ modkey }, "P", oxwm.inc_num_master(-1))      -- Decrement number of master windows

-- Gaps toggle
oxwm.key.bind({ modkey }, "A", oxwm.toggle_gaps()) -- Toggle gaps on/off

-- Window manager controls
oxwm.key.bind({ modkey, "Shift" }, "Q", oxwm.quit())    -- Quit OXWM
oxwm.key.bind({ modkey, "Shift" }, "R", oxwm.restart()) -- Restart OXWM (reloads config)

-- Focus movement (stack cycling)
oxwm.key.bind({ modkey }, "J", oxwm.client.focus_stack(1))  -- Focus next window in stack
oxwm.key.bind({ modkey }, "K", oxwm.client.focus_stack(-1)) -- Focus previous window in stack

-- Multi-monitor support
oxwm.key.bind({ modkey }, "Comma", oxwm.monitor.focus(-1))       -- Focus previous monitor
oxwm.key.bind({ modkey }, "Period", oxwm.monitor.focus(1))       -- Focus next monitor
oxwm.key.bind({ modkey, "Shift" }, "Comma", oxwm.monitor.tag(-1))  -- Send window to previous monitor
oxwm.key.bind({ modkey, "Shift" }, "Period", oxwm.monitor.tag(1))  -- Send window to next monitor

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

-------------------------------------------------------------------------------
-- Advanced: Keychords
-------------------------------------------------------------------------------
-- Keychords allow you to bind multiple-key sequences (like Emacs or Vim)
-- Format: {{modifiers}, key1}, {{modifiers}, key2}, ...
-- Example: Press Mod4+Space, then release and press T to spawn a terminal
oxwm.key.chord({
    { { modkey }, "Space" },
    { {},         "T" }
}, oxwm.spawn_terminal())

-------------------------------------------------------------------------------
-- Autostart
-------------------------------------------------------------------------------
-- Commands to run once when OXWM starts
-- Uncomment and modify these examples, or add your own

-- oxwm.autostart("picom")                                  -- Compositor for transparency and effects
-- oxwm.autostart("feh --bg-scale ~/wallpaper.jpg")        -- Set wallpaper
-- oxwm.autostart("dunst")                                  -- Notification daemon
-- oxwm.autostart("nm-applet")                              -- Network manager applet
