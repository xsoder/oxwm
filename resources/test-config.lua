---@meta
---OXWM Test Configuration File (Lua)
---Using the new functional API
---Edit this file and reload with Mod+Alt+R

---Load type definitions for LSP (lua-language-server)
---Option 1: Copy templates/oxwm.lua to the same directory as your config
---Option 2: Add to your LSP settings (e.g., .luarc.json):
---  {
---    "workspace.library": [
---      "/path/to/oxwm/templates"
---    ]
---  }
---Option 3: Symlink templates/oxwm.lua to your config directory
---@module 'oxwm'


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

local modkey = "Mod1"

oxwm.set_terminal("st")
oxwm.set_modkey(modkey)
oxwm.set_tags({ "1", "2", "3", "4", "5", "6", "7", "8", "9" })

oxwm.set_layout_symbol("tiling", "[T]")
oxwm.set_layout_symbol("normie", "[F]")

oxwm.border.set_width(2)
oxwm.border.set_focused_color(colors.blue)
oxwm.border.set_unfocused_color(colors.grey)

oxwm.gaps.set_enabled(true)
oxwm.gaps.set_inner(5, 5)
oxwm.gaps.set_outer(5, 5)

oxwm.bar.set_font("JetBrainsMono Nerd Font:style=Bold:size=12")

oxwm.bar.set_scheme_normal(colors.fg, colors.bg, 0x444444)
oxwm.bar.set_scheme_occupied(colors.cyan, colors.bg, colors.cyan)
oxwm.bar.set_scheme_selected(colors.cyan, colors.bg, colors.purple)

oxwm.key.chord({
    { { modkey }, "Space" },
    { {},         "T" }
}, oxwm.spawn("st"))

oxwm.key.bind({ modkey }, "Return", oxwm.spawn("st"))
oxwm.key.bind({ modkey }, "D", oxwm.spawn({ "sh", "-c", "dmenu_run -l 10" }))
oxwm.key.bind({ modkey }, "S", oxwm.spawn({ "sh", "-c", "maim -s | xclip -selection clipboard -t image/png" }))
oxwm.key.bind({ modkey }, "Q", oxwm.client.kill())

oxwm.key.bind({ modkey, "Shift" }, "Slash", oxwm.show_keybinds())

oxwm.key.bind({ modkey, "Shift" }, "F", oxwm.client.toggle_fullscreen())
oxwm.key.bind({ modkey, "Shift" }, "Space", oxwm.client.toggle_floating())

oxwm.key.bind({ modkey }, "F", oxwm.layout.set("normie"))
oxwm.key.bind({ modkey }, "C", oxwm.layout.set("tiling"))
oxwm.key.bind({ modkey }, "N", oxwm.layout.cycle())

oxwm.key.bind({ modkey }, "A", oxwm.toggle_gaps())

oxwm.key.bind({ modkey, "Shift" }, "Q", oxwm.quit())
oxwm.key.bind({ modkey, "Shift" }, "R", oxwm.restart())

oxwm.key.bind({ modkey }, "H", oxwm.client.focus_direction("left"))
oxwm.key.bind({ modkey }, "J", oxwm.client.focus_direction("down"))
oxwm.key.bind({ modkey }, "K", oxwm.client.focus_direction("up"))
oxwm.key.bind({ modkey }, "L", oxwm.client.focus_direction("right"))

oxwm.key.bind({ modkey, "Shift" }, "H", oxwm.client.swap_direction("left"))
oxwm.key.bind({ modkey, "Shift" }, "J", oxwm.client.swap_direction("down"))
oxwm.key.bind({ modkey, "Shift" }, "K", oxwm.client.swap_direction("up"))
oxwm.key.bind({ modkey, "Shift" }, "L", oxwm.client.swap_direction("right"))

oxwm.key.bind({ modkey }, "1", oxwm.tag.view(0))
oxwm.key.bind({ modkey }, "2", oxwm.tag.view(1))
oxwm.key.bind({ modkey }, "3", oxwm.tag.view(2))
oxwm.key.bind({ modkey }, "4", oxwm.tag.view(3))
oxwm.key.bind({ modkey }, "5", oxwm.tag.view(4))
oxwm.key.bind({ modkey }, "6", oxwm.tag.view(5))
oxwm.key.bind({ modkey }, "7", oxwm.tag.view(6))
oxwm.key.bind({ modkey }, "8", oxwm.tag.view(7))
oxwm.key.bind({ modkey }, "9", oxwm.tag.view(8))

oxwm.key.bind({ modkey, "Shift" }, "1", oxwm.tag.move_to(0))
oxwm.key.bind({ modkey, "Shift" }, "2", oxwm.tag.move_to(1))
oxwm.key.bind({ modkey, "Shift" }, "3", oxwm.tag.move_to(2))
oxwm.key.bind({ modkey, "Shift" }, "4", oxwm.tag.move_to(3))
oxwm.key.bind({ modkey, "Shift" }, "5", oxwm.tag.move_to(4))
oxwm.key.bind({ modkey, "Shift" }, "6", oxwm.tag.move_to(5))
oxwm.key.bind({ modkey, "Shift" }, "7", oxwm.tag.move_to(6))
oxwm.key.bind({ modkey, "Shift" }, "8", oxwm.tag.move_to(7))
oxwm.key.bind({ modkey, "Shift" }, "9", oxwm.tag.move_to(8))

oxwm.bar.set_blocks({
    oxwm.bar.block.battery({
        format = "Bat: {}%",
        charging = "⚡ Bat: {}%",
        discharging = "🔋 Bat: {}%",
        full = "✓ Bat: {}%",
        interval = 30,
        color = colors.green,
        underline = true,
    }),
    -- oxwm.bar.block.battery({
    --     charging = "󰂄 Bat: {}%",
    --     discharging = "󰁹 Bat: {}%",
    --     full = "󰁹 Bat: {}%",
    --     format = "",
    --     interval = 30,
    --     color = colors.green,
    --     underline = true
    -- }),
    oxwm.bar.block.static({
        text = " │  ",
        format = "",
        interval = 999999999,
        color = colors.lavender,
        underline = false
    }),
    oxwm.bar.block.ram({
        format = "󰍛 {used}/{total} GB",
        interval = 5,
        color = colors.light_blue,
        underline = true
    }),
    oxwm.bar.block.static({
        text = " │  ",
        format = "",
        interval = 999999999,
        color = colors.lavender,
        underline = false
    }),
    oxwm.bar.block.shell({
        command = "uname -r",
        format = " {}",
        interval = 999999999,
        color = colors.red,
        underline = true
    }),
    oxwm.bar.block.static({
        text = " │  ",
        format = "",
        interval = 999999999,
        color = colors.lavender,
        underline = false
    }),
    oxwm.bar.block.datetime({
        format = "󰸘 {}",
        interval = 1,
        color = colors.cyan,
        underline = true,
        date_format = "%a, %b %d - %-I:%M %P"
    })
})
