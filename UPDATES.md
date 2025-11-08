# OXWM Recent Updates

## Lua Configuration Support (Latest)

OXWM now supports **Lua-based configuration** as an alternative to the compiled Rust configuration. This is a major update that makes the window manager much more user-friendly and easier to customize.

### Key Features

#### 1. **Dynamic Configuration - No Compilation Required**
- Edit your config file and reload with `Mod+Shift+R` - changes apply instantly
- No need to recompile the entire window manager for config changes
- Configuration file located at `~/.config/oxwm/config.lua`

#### 2. **Full Feature Parity**
The Lua configuration supports all features previously available in Rust config:
- Window appearance (borders, gaps, fonts)
- Keybindings (single keys and keychords)
- Layout management and custom symbols
- Status bar configuration
- Workspace tags
- Autostart commands
- Color schemes

#### 3. **LSP Support & Autocomplete**
Config files include comprehensive type annotations for Lua language servers:
- Full autocomplete for all configuration options
- Type checking to catch errors before runtime
- Hover documentation for all fields
- IntelliSense support in modern editors (VS Code, Neovim, etc.)

### Configuration Structure

```lua
---@type Config
config = {
    -- Appearance
    border_width = 2,
    border_focused = 0x6dade3,
    border_unfocused = 0xbbbbbb,
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
    tags = {"1", "2", "3", "4", "5", "6", "7", "8", "9"},

    -- Keybindings (see below for details)
    keybindings = { ... },

    -- Status bar blocks (see below for details)
    status_blocks = { ... },

    -- Color schemes
    scheme_normal = { ... },
    scheme_occupied = { ... },
    scheme_selected = { ... },

    -- Autostart commands
    autostart = {
        "picom -b",
        "nitrogen --restore &",
    },
}
```

### Keybindings

#### Single Key Bindings
```lua
{
    modifiers = {"Mod", "Shift"},
    key = "Return",
    action = "Spawn",
    arg = "alacritty"
}
```

#### Keychords (Multi-key Sequences)
```lua
{
    keys = {
        {modifiers = {"Mod4"}, key = "Space"},
        {modifiers = {}, key = "T"}
    },
    action = "Spawn",
    arg = "st"
}
```

Press Escape to cancel any in-progress keychord.

#### Available Modifiers
- `"Mod"` - Replaced with your configured modkey
- `"Mod1"` - Alt key
- `"Mod4"` - Super/Windows key
- `"Shift"` - Shift key
- `"Control"` - Control key
- `"Mod2"`, `"Mod3"`, `"Mod5"` - Additional modifiers

#### Available Actions
- `"Spawn"` - Launch a program (arg: string or string[])
- `"KillClient"` - Close focused window
- `"FocusStack"` - Focus next/prev in stack (arg: 1 or -1)
- `"FocusDirection"` - Focus by direction (arg: 0=up, 1=down, 2=left, 3=right)
- `"SwapDirection"` - Swap window by direction (arg: 0=up, 1=down, 2=left, 3=right)
- `"Quit"` - Exit window manager
- `"Restart"` - Restart and reload config
- `"Recompile"` - Recompile and restart (for Rust config)
- `"ViewTag"` - Switch to tag (arg: tag index)
- `"MoveToTag"` - Move window to tag (arg: tag index)
- `"ToggleGaps"` - Toggle gaps on/off
- `"ToggleFullScreen"` - Toggle fullscreen mode
- `"ToggleFloating"` - Toggle floating mode
- `"ChangeLayout"` - Switch to specific layout (arg: layout name)
- `"CycleLayout"` - Cycle through layouts
- `"FocusMonitor"` - Focus monitor (arg: 1 or -1)
- `"SmartMoveWin"` - Smart window movement
- `"ExchangeClient"` - Exchange window positions

### Status Bar Configuration

Status blocks support various command types for displaying system information:

```lua
status_blocks = {
    -- RAM usage
    {
        format = "Ram: {used}/{total} GB",
        command = "Ram",
        interval_secs = 5,
        color = 0x7aa2f7,
        underline = true
    },

    -- Date/Time with custom format
    {
        format = "{}",
        command = "DateTime",
        command_arg = "%a, %b %d - %-I:%M %P",
        interval_secs = 1,
        color = 0x0db9d7,
        underline = true
    },

    -- Custom shell command
    {
        format = "Kernel: {}",
        command = "Shell",
        command_arg = "uname -r",
        interval_secs = 999999999,
        color = 0xf7768e,
        underline = true
    },

    -- Static text (separator)
    {
        format = " │  ",
        command = "Static",
        interval_secs = 999999999,
        color = 0xa9b1d6,
        underline = false
    },

    -- Battery status
    {
        format = "{}",
        command = "Battery",
        interval_secs = 30,
        color = 0x9ece6a,
        underline = true,
        battery_formats = {
            charging = "⚡ {}%",
            discharging = "🔋 {}%",
            full = "✓ {}%"
        }
    },
}
```

#### Available Status Block Commands
- `"Ram"` - Shows RAM usage (no command_arg needed)
- `"DateTime"` - Shows date/time with strftime format in command_arg
- `"Shell"` - Runs shell command from command_arg
- `"Static"` - Static text (format field determines what's shown)
- `"Battery"` - Shows battery status (requires battery_formats table)

### Layout Symbols

Customize how layouts appear in the status bar:

```lua
layout_symbols = {
    {name = "tiling", symbol = "[T]"},
    {name = "normie", symbol = "[F]"},
}
```

### Color Schemes

Define color schemes for tag indicators in the status bar:

```lua
scheme_normal = {
    foreground = 0xbbbbbb,
    background = 0x1a1b26,
    underline = 0x444444
},
scheme_occupied = {
    foreground = 0x0db9d7,
    background = 0x1a1b26,
    underline = 0x0db9d7
},
scheme_selected = {
    foreground = 0x0db9d7,
    background = 0x1a1b26,
    underline = 0xad8ee6
}
```

### Autostart Commands

Commands to run when the window manager starts:

```lua
autostart = {
    "picom -b",
    "nitrogen --restore &",
    "dunst &",
}
```

### Advanced Usage

You can use Lua's full programming capabilities for dynamic configuration:

```lua
-- Generate keybindings programmatically
for i = 1, 9 do
    table.insert(config.keybindings, {
        modifiers = {"Mod"},
        key = tostring(i),
        action = "ViewTag",
        arg = i - 1
    })
end

-- Create helper functions
local function create_spawn_binding(mods, key, cmd)
    return {modifiers = mods, key = key, action = "Spawn", arg = cmd}
end

table.insert(config.keybindings,
    create_spawn_binding({"Mod"}, "Return", terminal))
```

### Migration from Rust Config

For existing users with Rust-based configurations:
1. Your old Rust config still works - no breaking changes
2. To migrate to Lua, run OXWM and it will generate `~/.config/oxwm/config.lua`
3. Customize the generated file to match your preferences
4. Reload with `Mod+Shift+R` to apply changes

### Technical Implementation

- Configuration parsing via `mlua` (Lua 5.4)
- Full deserialization into Rust types with `serde`
- Hot-reloading on `Restart` action
- Comprehensive error reporting for config issues
- Type-safe validation of all configuration values

### Files

- Template: `templates/config.lua` - Default configuration template
- User config: `~/.config/oxwm/config.lua` - User's configuration file
- Test config: `resources/test-config.lua` - Configuration for Xephyr testing

---

## Documentation Suggestions

### New Pages to Create

1. **"Lua Configuration Guide"** - Complete guide to Lua config
   - Getting started
   - Configuration structure
   - Examples for common use cases
   - Tips and best practices

2. **"Keybindings Reference"** - Detailed keybinding documentation
   - How to define single key bindings
   - How to create keychords
   - List of all available actions with examples
   - Modifier key reference

3. **"Status Bar Configuration"** - Status bar setup guide
   - Available command types
   - Format string syntax
   - Custom shell commands
   - Battery configuration
   - Creating custom blocks

### Pages to Update

1. **Installation/Quickstart**
   - Mention Lua configuration as the recommended approach
   - Add note that no recompilation needed for config changes
   - Update first-run instructions

2. **Configuration Page** (if exists)
   - Add prominent notice about Lua support
   - Deprecation notice for Rust config (if planned)
   - Link to new Lua configuration guide

3. **Keybindings Page** (if exists)
   - Update with Lua syntax
   - Add keychord examples
   - Update modifier key documentation

### Highlights for Documentation

**Key Selling Points:**
- "Edit and reload instantly - no compilation required"
- "LSP-powered autocomplete for config editing"
- "Full Lua programming support for dynamic configs"
- "Keychord support for complex key combinations"

**Common Questions to Address:**
- How do I reload my config? (`Mod+Shift+R`)
- Where is my config file? (`~/.config/oxwm/config.lua`)
- Can I still use the Rust config? (Yes, for now)
- How do I see config errors? (Check terminal output when starting OXWM)

### Example Snippets for Docs

#### Quick Config Example
```lua
-- Minimal working config
config = {
    modkey = "Mod4",
    terminal = "alacritty",
    tags = {"web", "code", "term"},
    border_width = 2,
    border_focused = 0x89b4fa,
    border_unfocused = 0x45475a,
    keybindings = {
        {modifiers = {"Mod"}, key = "Return", action = "Spawn", arg = "alacritty"},
        {modifiers = {"Mod"}, key = "Q", action = "KillClient"},
        {modifiers = {"Mod", "Shift"}, key = "Q", action = "Quit"},
    }
}
```

#### Advanced Keychord Example
```lua
-- Open application menu with Mod+Space, then choose app
keybindings = {
    -- Mod+Space, then B -> browser
    {
        keys = {
            {modifiers = {"Mod4"}, key = "Space"},
            {modifiers = {}, key = "B"}
        },
        action = "Spawn",
        arg = "firefox"
    },
    -- Mod+Space, then T -> terminal
    {
        keys = {
            {modifiers = {"Mod4"}, key = "Space"},
            {modifiers = {}, key = "T"}
        },
        action = "Spawn",
        arg = "alacritty"
    },
}
```

---

## Other Recent Updates

### Autostart Support
- Added `autostart` field to configuration
- Commands run when window manager starts
- Useful for launching compositors, wallpaper setters, notification daemons, etc.

### Fullscreen Fixes
- Fixed issue where fullscreen windows weren't properly applying geometries
- Border handling corrected for fullscreen mode

### Layout Updates
- Improved layout switching behavior
- Better handling of layout symbols in status bar

---

**Last Updated:** 2025-11-07
