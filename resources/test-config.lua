---@meta
---OXWM Test Configuration File (Lua)
---Using the new functional API
---Edit this file and reload with Mod+Alt+R

---Load type definitions for LSP
---@module 'oxwm'


-- Set variables as needed
-- Colors
local colors = {
    lavender = 0xa9b1d6,
    light_blue = 0x7aa2f7,
    grey = 0xbbbbbb,
    purple = 0xad8ee6,
    cyan = 0x0db9d7,
    bg = 0x1a1b26,
    green = 0x9ece6a,
    red = 0xf7768e,
    fg = 0xbbbbbb,
    blue = 0x6dade3,
}

local modkey = "Mod1";

-- Basic settings
oxwm.set_terminal("st")
oxwm.set_modkey(modkey)
oxwm.set_tags({ "1", "2", "3", "4", "5", "6", "7", "8", "9" })

-- Layout symbol overrides
oxwm.set_layout_symbol("tiling", "[T]")
oxwm.set_layout_symbol("normie", "[F]")

-- Border configuration
oxwm.border.set_width(2)
oxwm.border.set_focused_color(colors.blue)
oxwm.border.set_unfocused_color(colors.grey)

-- Gap configuration
oxwm.gaps.set_enabled(true)
oxwm.gaps.set_inner(5, 5)
oxwm.gaps.set_outer(5, 5)

-- Bar configuration
oxwm.bar.set_font("JetBrainsMono Nerd Font:style=Bold:size=12")

-- Bar color schemes (for tag display)
oxwm.bar.set_scheme_normal(colors.fg, colors.bg, 0x444444)
oxwm.bar.set_scheme_occupied(colors.cyan, colors.bg, colors.cyan)
oxwm.bar.set_scheme_selected(colors.cyan, colors.bg, colors.purple)

-- Keybindings

-- Keychord: Mod1+Space then T to spawn terminal
oxwm.key.chord({
    { { modkey }, "Space" },
    { {},         "T" }
}, oxwm.spawn("st"))

-- Basic window management
oxwm.key.bind({ modkey }, "Return", oxwm.spawn("st"))
oxwm.key.bind({ modkey }, "D", oxwm.spawn({ "sh", "-c", "dmenu_run -l 10" }))
oxwm.key.bind({ modkey }, "S", oxwm.spawn({ "sh", "-c", "maim -s | xclip -selection clipboard -t image/png" }))
oxwm.key.bind({ modkey }, "Q", oxwm.client.kill())

-- Keybind overlay
oxwm.key.bind({ modkey, "Shift" }, "Slash", oxwm.show_keybinds())

-- Client actions
oxwm.key.bind({ modkey, "Shift" }, "F", oxwm.client.toggle_fullscreen())
oxwm.key.bind({ modkey, "Shift" }, "Space", oxwm.client.toggle_floating())

-- Layout management
oxwm.key.bind({ modkey }, "F", oxwm.layout.set("normie"))
oxwm.key.bind({ modkey }, "C", oxwm.layout.set("tiling"))
oxwm.key.bind({ modkey }, "N", oxwm.layout.cycle())

-- Gaps toggle
oxwm.key.bind({ modkey }, "A", oxwm.toggle_gaps())

-- WM controls
oxwm.key.bind({ modkey, "Shift" }, "Q", oxwm.quit())
oxwm.key.bind({ modkey, "Shift" }, "R", oxwm.restart())

-- Focus direction (vim keys)
oxwm.key.bind({ modkey }, "H", oxwm.client.focus_direction("left"))
oxwm.key.bind({ modkey }, "J", oxwm.client.focus_direction("down"))
oxwm.key.bind({ modkey }, "K", oxwm.client.focus_direction("up"))
oxwm.key.bind({ modkey }, "L", oxwm.client.focus_direction("right"))

-- Swap windows in direction
oxwm.key.bind({ modkey, "Shift" }, "H", oxwm.client.swap_direction("left"))
oxwm.key.bind({ modkey, "Shift" }, "J", oxwm.client.swap_direction("down"))
oxwm.key.bind({ modkey, "Shift" }, "K", oxwm.client.swap_direction("up"))
oxwm.key.bind({ modkey, "Shift" }, "L", oxwm.client.swap_direction("right"))

-- Tag viewing
oxwm.key.bind({ modkey }, "1", oxwm.tag.view(0))
oxwm.key.bind({ modkey }, "2", oxwm.tag.view(1))
oxwm.key.bind({ modkey }, "3", oxwm.tag.view(2))
oxwm.key.bind({ modkey }, "4", oxwm.tag.view(3))
oxwm.key.bind({ modkey }, "5", oxwm.tag.view(4))
oxwm.key.bind({ modkey }, "6", oxwm.tag.view(5))
oxwm.key.bind({ modkey }, "7", oxwm.tag.view(6))
oxwm.key.bind({ modkey }, "8", oxwm.tag.view(7))
oxwm.key.bind({ modkey }, "9", oxwm.tag.view(8))

-- Move window to tag
oxwm.key.bind({ modkey, "Shift" }, "1", oxwm.tag.move_to(0))
oxwm.key.bind({ modkey, "Shift" }, "2", oxwm.tag.move_to(1))
oxwm.key.bind({ modkey, "Shift" }, "3", oxwm.tag.move_to(2))
oxwm.key.bind({ modkey, "Shift" }, "4", oxwm.tag.move_to(3))
oxwm.key.bind({ modkey, "Shift" }, "5", oxwm.tag.move_to(4))
oxwm.key.bind({ modkey, "Shift" }, "6", oxwm.tag.move_to(5))
oxwm.key.bind({ modkey, "Shift" }, "7", oxwm.tag.move_to(6))
oxwm.key.bind({ modkey, "Shift" }, "8", oxwm.tag.move_to(7))
oxwm.key.bind({ modkey, "Shift" }, "9", oxwm.tag.move_to(8))

-- Status bar blocks
oxwm.bar.add_block("", "Battery", {
    charging = "󰂄 Bat: {}%",
    discharging = "󰁹 Bat:{}%",
    full = "󰁹 Bat: {}%"
}, 30, colors.green, true)

oxwm.bar.add_block(" │  ", "Static", "", 999999999, colors.lavender, false)
oxwm.bar.add_block("󰍛 {used}/{total} GB", "Ram", nil, 5, colors.light_blue, true)
oxwm.bar.add_block(" │  ", "Static", "", 999999999, colors.lavender, false)
oxwm.bar.add_block(" {}", "Shell", "uname -r", 999999999, colors.red, true)
oxwm.bar.add_block(" │  ", "Static", "", 999999999, colors.lavender, false)
oxwm.bar.add_block("󰸘 {}", "DateTime", "%a, %b %d - %-I:%M %P", 1, colors.cyan, true)

-- Autostart commands (runs once at startup)
-- oxwm.autostart("picom")
-- oxwm.autostart("feh --bg-scale ~/wallpaper.jpg")
