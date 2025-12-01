use crate::Config;
use crate::bar::Bar;
use crate::client::{Client, TagMask};
use crate::errors::WmError;
use crate::keyboard::{self, Arg, KeyAction, handlers};
use crate::layout::GapConfig;
use crate::layout::tiling::TilingLayout;
use crate::layout::{Layout, LayoutBox, LayoutType, layout_from_str, next_layout};
use crate::monitor::{Monitor, detect_monitors};
use crate::overlay::{ErrorOverlay, KeybindOverlay, Overlay};
use std::collections::{HashMap, HashSet};
use std::process::Command;
use x11rb::cursor::Handle as CursorHandle;

use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

const DEFAULT_FLOAT_WIDTH_RATIO: f32 = 0.5;
const DEFAULT_FLOAT_HEIGHT_RATIO: f32 = 0.5;

#[derive(Debug, Clone, Copy)]
pub struct CachedGeometry {
    pub x_position: i16,
    pub y_position: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
}

pub fn tag_mask(tag: usize) -> TagMask {
    1 << tag
}

struct AtomCache {
    net_current_desktop: Atom,
    net_client_info: Atom,
    wm_state: Atom,
    wm_protocols: Atom,
    wm_delete_window: Atom,
    net_wm_state: Atom,
    net_wm_state_fullscreen: Atom,
    net_wm_window_type: Atom,
    net_wm_window_type_dialog: Atom,
    wm_name: Atom,
    net_wm_name: Atom,
    wm_normal_hints: Atom,
    wm_hints: Atom,
    wm_transient_for: Atom,
}

impl AtomCache {
    fn new(connection: &RustConnection) -> WmResult<Self> {
        let net_current_desktop = connection
            .intern_atom(false, b"_NET_CURRENT_DESKTOP")?
            .reply()?
            .atom;

        let net_client_info = connection
            .intern_atom(false, b"_NET_CLIENT_INFO")?
            .reply()?
            .atom;

        let wm_state = connection.intern_atom(false, b"WM_STATE")?.reply()?.atom;

        let wm_protocols = connection
            .intern_atom(false, b"WM_PROTOCOLS")?
            .reply()?
            .atom;

        let wm_delete_window = connection
            .intern_atom(false, b"WM_DELETE_WINDOW")?
            .reply()?
            .atom;

        let net_wm_state = connection
            .intern_atom(false, b"_NET_WM_STATE")?
            .reply()?
            .atom;

        let net_wm_state_fullscreen = connection
            .intern_atom(false, b"_NET_WM_STATE_FULLSCREEN")?
            .reply()?
            .atom;

        let net_wm_window_type = connection
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE")?
            .reply()?
            .atom;

        let net_wm_window_type_dialog = connection
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE_DIALOG")?
            .reply()?
            .atom;

        let wm_name = AtomEnum::WM_NAME.into();
        let net_wm_name = connection.intern_atom(false, b"_NET_WM_NAME")?.reply()?.atom;
        let wm_normal_hints = AtomEnum::WM_NORMAL_HINTS.into();
        let wm_hints = AtomEnum::WM_HINTS.into();
        let wm_transient_for = AtomEnum::WM_TRANSIENT_FOR.into();

        Ok(Self {
            net_current_desktop,
            net_client_info,
            wm_state,
            wm_protocols,
            wm_delete_window,
            net_wm_state,
            net_wm_state_fullscreen,
            net_wm_window_type,
            net_wm_window_type_dialog,
            wm_name,
            net_wm_name,
            wm_normal_hints,
            wm_hints,
            wm_transient_for,
        })
    }
}

pub struct WindowManager {
    config: Config,
    connection: RustConnection,
    screen_number: usize,
    root: Window,
    screen: Screen,
    windows: Vec<Window>,
    clients: HashMap<Window, Client>,
    layout: LayoutBox,
    window_geometry_cache: HashMap<Window, CachedGeometry>,
    gaps_enabled: bool,
    floating_windows: HashSet<Window>,
    fullscreen_windows: HashSet<Window>,
    floating_geometry_before_fullscreen: HashMap<Window, (i16, i16, u16, u16, u16)>,
    bars: Vec<Bar>,
    tab_bars: Vec<crate::tab_bar::TabBar>,
    show_bar: bool,
    last_layout: Option<&'static str>,
    monitors: Vec<Monitor>,
    selected_monitor: usize,
    atoms: AtomCache,
    previous_focused: Option<Window>,
    display: *mut x11::xlib::Display,
    font: crate::bar::font::Font,
    keychord_state: keyboard::handlers::KeychordState,
    error_message: Option<String>,
    overlay: ErrorOverlay,
    keybind_overlay: KeybindOverlay,
}

type WmResult<T> = Result<T, WmError>;

impl WindowManager {
    pub fn new(config: Config) -> WmResult<Self> {
        let (connection, screen_number) = x11rb::connect(None)?;
        let root = connection.setup().roots[screen_number].root;
        let screen = connection.setup().roots[screen_number].clone();

        let normal_cursor = CursorHandle::new(
            &connection,
            screen_number,
            &x11rb::resource_manager::new_from_default(&connection)?,
        )?
        .reply()?
        .load_cursor(&connection, "left_ptr")?;

        connection
            .change_window_attributes(
                root,
                &ChangeWindowAttributesAux::new()
                    .cursor(normal_cursor)
                    .event_mask(
                        EventMask::SUBSTRUCTURE_REDIRECT
                            | EventMask::SUBSTRUCTURE_NOTIFY
                            | EventMask::PROPERTY_CHANGE
                            | EventMask::KEY_PRESS
                            | EventMask::BUTTON_PRESS
                            | EventMask::POINTER_MOTION,
                    ),
            )?
            .check()?;

        let ignore_modifiers = [
            0,
            u16::from(ModMask::LOCK),
            u16::from(ModMask::M2),
            u16::from(ModMask::LOCK | ModMask::M2),
        ];

        for &ignore_mask in &ignore_modifiers {
            let grab_mask = u16::from(config.modkey) | ignore_mask;

            connection.grab_button(
                false,
                root,
                EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE,
                GrabMode::SYNC,
                GrabMode::ASYNC,
                x11rb::NONE,
                x11rb::NONE,
                ButtonIndex::M1,
                grab_mask.into(),
            )?;

            connection.grab_button(
                false,
                root,
                EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE,
                GrabMode::SYNC,
                GrabMode::ASYNC,
                x11rb::NONE,
                x11rb::NONE,
                ButtonIndex::M3,
                grab_mask.into(),
            )?;
        }

        let monitors = detect_monitors(&connection, &screen, root)?;

        let display = unsafe { x11::xlib::XOpenDisplay(std::ptr::null()) };
        if display.is_null() {
            return Err(WmError::X11(crate::errors::X11Error::DisplayOpenFailed));
        }

        let font = crate::bar::font::Font::new(display, screen_number as i32, &config.font)?;

        let mut bars = Vec::new();
        for monitor in monitors.iter() {
            let bar = Bar::new(
                &connection,
                &screen,
                screen_number,
                &config,
                display,
                &font,
                monitor.screen_x as i16,
                monitor.screen_y as i16,
                monitor.screen_width as u16,
            )?;
            bars.push(bar);
        }

        let bar_height = font.height() as f32 * 1.4;
        let mut tab_bars = Vec::new();
        for monitor in monitors.iter() {
            let tab_bar = crate::tab_bar::TabBar::new(
                &connection,
                &screen,
                screen_number,
                display,
                &font,
                (monitor.screen_x + config.gap_outer_horizontal as i32) as i16,
                (monitor.screen_y as f32 + bar_height + config.gap_outer_vertical as f32) as i16,
                monitor.screen_width.saturating_sub(2 * config.gap_outer_horizontal as i32) as u16,
                config.scheme_occupied,
                config.scheme_selected,
            )?;
            tab_bars.push(tab_bar);
        }

        let gaps_enabled = config.gaps_enabled;

        let atoms = AtomCache::new(&connection)?;

        let overlay = ErrorOverlay::new(
            &connection,
            &screen,
            screen_number,
            display,
            &font,
            screen.width_in_pixels,
        )?;

        let keybind_overlay =
            KeybindOverlay::new(&connection, &screen, screen_number, display, config.modkey)?;

        let mut window_manager = Self {
            config,
            connection,
            screen_number,
            root,
            screen,
            windows: Vec::new(),
            clients: HashMap::new(),
            layout: Box::new(TilingLayout),
            window_geometry_cache: HashMap::new(),
            gaps_enabled,
            floating_windows: HashSet::new(),
            fullscreen_windows: HashSet::new(),
            floating_geometry_before_fullscreen: HashMap::new(),
            bars,
            tab_bars,
            show_bar: true,
            last_layout: None,
            monitors,
            selected_monitor: 0,
            atoms,
            previous_focused: None,
            display,
            font,
            keychord_state: keyboard::handlers::KeychordState::Idle,
            error_message: None,
            overlay,
            keybind_overlay,
        };

        for tab_bar in &window_manager.tab_bars {
            tab_bar.hide(&window_manager.connection)?;
        }

        window_manager.scan_existing_windows()?;
        window_manager.update_bar()?;
        window_manager.run_autostart_commands()?;

        Ok(window_manager)
    }

    pub fn show_migration_overlay(&mut self) {
        let message = "Your config.lua uses legacy syntax or has errors.\n\n\
                       You are now running with default configuration.\n\n\
                       Press Mod+Shift+/ to see default keybinds\n\
                       Press Mod+Shift+R to reload after fixing your config";

        let screen_width = self.screen.width_in_pixels;
        let screen_height = self.screen.height_in_pixels;

        if let Err(e) = self.overlay.show_error(
            &self.connection,
            &self.font,
            message,
            screen_width,
            screen_height,
        ) {
            eprintln!("Failed to show migration overlay: {:?}", e);
        }
    }

    fn try_reload_config(&mut self) -> Result<(), String> {
        let config_dir = if let Some(xdg_config) = std::env::var_os("XDG_CONFIG_HOME") {
            std::path::PathBuf::from(xdg_config).join("oxwm")
        } else if let Some(home) = std::env::var_os("HOME") {
            std::path::PathBuf::from(home).join(".config").join("oxwm")
        } else {
            return Err("Could not find config directory".to_string());
        };

        let lua_path = config_dir.join("config.lua");

        if !lua_path.exists() {
            return Err("No config.lua file found".to_string());
        }

        let config_str = std::fs::read_to_string(&lua_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;

        let new_config = crate::config::parse_lua_config(&config_str, Some(&config_dir))
            .map_err(|e| format!("{}", e))?;

        self.config = new_config;
        self.error_message = None;

        for bar in &mut self.bars {
            bar.update_from_config(&self.config);
        }

        Ok(())
    }

    fn scan_existing_windows(&mut self) -> WmResult<()> {
        let tree = self.connection.query_tree(self.root)?.reply()?;
        let net_client_info = self.atoms.net_client_info;
        let wm_state_atom = self.atoms.wm_state;

        for &window in &tree.children {
            if self.bars.iter().any(|bar| bar.window() == window) {
                continue;
            }

            let Ok(attrs) = self.connection.get_window_attributes(window)?.reply() else {
                continue;
            };

            if attrs.override_redirect {
                continue;
            }

            if attrs.map_state == MapState::VIEWABLE {
                let _tag = self.get_saved_tag(window, net_client_info)?;
                self.windows.push(window);
                continue;
            }

            if attrs.map_state == MapState::UNMAPPED {
                let has_wm_state = self
                    .connection
                    .get_property(false, window, wm_state_atom, AtomEnum::ANY, 0, 2)?
                    .reply()
                    .is_ok_and(|prop| !prop.value.is_empty());

                if !has_wm_state {
                    continue;
                }

                let has_wm_class = self
                    .connection
                    .get_property(false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 1024)?
                    .reply()
                    .is_ok_and(|prop| !prop.value.is_empty());

                if has_wm_class {
                    let _tag = self.get_saved_tag(window, net_client_info)?;
                    self.connection.map_window(window)?;
                    self.windows.push(window);
                }
            }
        }

        if let Some(&first) = self.windows.first() {
            self.focus(Some(first))?;
        }

        self.apply_layout()?;
        Ok(())
    }

    fn get_saved_tag(&self, window: Window, net_client_info: Atom) -> WmResult<TagMask> {
        match self
            .connection
            .get_property(false, window, net_client_info, AtomEnum::CARDINAL, 0, 2)?
            .reply()
        {
            Ok(prop) if prop.value.len() >= 4 => {
                let tags = u32::from_ne_bytes([
                    prop.value[0],
                    prop.value[1],
                    prop.value[2],
                    prop.value[3],
                ]);

                if tags != 0 && tags < (1 << self.config.tags.len()) {
                    return Ok(tags);
                }
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("No _NET_CLIENT_INFO property ({})", e);
            }
        }

        Ok(self
            .monitors
            .get(self.selected_monitor)
            .map(|m| m.tagset[m.selected_tags_index])
            .unwrap_or(tag_mask(0)))
    }

    fn save_client_tag(&self, window: Window, tag: TagMask) -> WmResult<()> {
        let net_client_info = self.atoms.net_client_info;

        let bytes = tag.to_ne_bytes().to_vec();

        self.connection.change_property(
            PropMode::REPLACE,
            window,
            net_client_info,
            AtomEnum::CARDINAL,
            32,
            1,
            &bytes,
        )?;

        self.connection.flush()?;
        Ok(())
    }

    fn set_wm_state(&self, window: Window, state: u32) -> WmResult<()> {
        let wm_state_atom = self.atoms.wm_state;

        let data = [state, 0u32];
        let bytes: Vec<u8> = data.iter().flat_map(|&v| v.to_ne_bytes()).collect();

        self.connection.change_property(
            PropMode::REPLACE,
            window,
            wm_state_atom,
            wm_state_atom,
            32,
            2,
            &bytes,
        )?;

        self.connection.flush()?;
        Ok(())
    }

    pub fn run(&mut self) -> WmResult<bool> {
        println!("oxwm started on display {}", self.screen_number);

        keyboard::setup_keybinds(&self.connection, self.root, &self.config.keybindings)?;
        self.update_bar()?;

        let mut last_bar_update = std::time::Instant::now();
        const BAR_UPDATE_INTERVAL_MS: u64 = 100;

        loop {
            match self.connection.poll_for_event_with_sequence()? {
                Some((event, _sequence)) => {
                    if let Some(should_restart) = self.handle_event(event)? {
                        return Ok(should_restart);
                    }
                }
                None => {
                    if last_bar_update.elapsed().as_millis() >= BAR_UPDATE_INTERVAL_MS as u128 {
                        if let Some(bar) = self.bars.get_mut(self.selected_monitor) {
                            bar.update_blocks();
                        }
                        if self.bars.iter().any(|bar| bar.needs_redraw()) {
                            self.update_bar()?;
                        }
                        last_bar_update = std::time::Instant::now();
                    }

                    self.connection.flush()?;
                    std::thread::sleep(std::time::Duration::from_millis(16));
                }
            }
        }
    }

    fn toggle_floating(&mut self) -> WmResult<()> {
        let focused = self
            .monitors
            .get(self.selected_monitor)
            .and_then(|m| m.selected_client);

        if focused.is_none() {
            return Ok(());
        }
        let focused = focused.unwrap();

        if let Some(client) = self.clients.get(&focused) {
            if client.is_fullscreen {
                return Ok(());
            }
        }

        let (is_fixed, x, y, w, h) = if let Some(client) = self.clients.get(&focused) {
            (client.is_fixed, client.x_position as i32, client.y_position as i32, client.width as u32, client.height as u32)
        } else {
            return Ok(());
        };

        let was_floating = self.floating_windows.contains(&focused);

        if was_floating {
            self.floating_windows.remove(&focused);
            if let Some(client) = self.clients.get_mut(&focused) {
                client.is_floating = false;
            }
        } else {
            self.floating_windows.insert(focused);
            if let Some(client) = self.clients.get_mut(&focused) {
                client.is_floating = is_fixed || !client.is_floating;
            }

            self.connection.configure_window(
                focused,
                &ConfigureWindowAux::new()
                    .x(x)
                    .y(y)
                    .width(w)
                    .height(h)
                    .stack_mode(StackMode::ABOVE),
            )?;
        }

        self.apply_layout()?;
        Ok(())
    }

    fn set_master_factor(&mut self, delta: f32) -> WmResult<()> {
        if let Some(monitor) = self.monitors.get_mut(self.selected_monitor) {
            let new_mfact = (monitor.master_factor + delta).max(0.05).min(0.95);
            monitor.master_factor = new_mfact;
            self.apply_layout()?;
        }
        Ok(())
    }

    fn inc_num_master(&mut self, delta: i32) -> WmResult<()> {
        if let Some(monitor) = self.monitors.get_mut(self.selected_monitor) {
            let new_nmaster = (monitor.num_master + delta).max(0);
            monitor.num_master = new_nmaster;
            self.apply_layout()?;
        }
        Ok(())
    }

    fn exchange_client(&mut self, direction: i32) -> WmResult<()> {
        let focused = match self
            .monitors
            .get(self.selected_monitor)
            .and_then(|m| m.selected_client)
        {
            Some(win) => win,
            None => return Ok(()),
        };

        if self.floating_windows.contains(&focused) {
            return Ok(());
        }

        let visible = self.visible_windows();
        if visible.len() < 2 {
            return Ok(());
        }

        let current_idx = match visible.iter().position(|&w| w == focused) {
            Some(idx) => idx,
            None => return Ok(()),
        };

        let target_idx = match direction {
            0 | 2 => {
                // UP or LEFT - previous in stack
                if current_idx == 0 {
                    visible.len() - 1
                } else {
                    current_idx - 1
                }
            }
            1 | 3 => {
                // DOWN or RIGHT - next in stack
                (current_idx + 1) % visible.len()
            }
            _ => return Ok(()),
        };

        let target = visible[target_idx];

        let focused_pos = self.windows.iter().position(|&w| w == focused);
        let target_pos = self.windows.iter().position(|&w| w == target);

        if let (Some(f_pos), Some(t_pos)) = (focused_pos, target_pos) {
            self.windows.swap(f_pos, t_pos);

            self.apply_layout()?;

            self.focus(Some(focused))?;

            if let Ok(geometry) = self.connection.get_geometry(focused)?.reply() {
                self.connection.warp_pointer(
                    x11rb::NONE,
                    focused,
                    0,
                    0,
                    0,
                    0,
                    geometry.width as i16 / 2,
                    geometry.height as i16 / 2,
                )?;
            }
        }

        Ok(())
    }


    fn get_layout_symbol(&self) -> String {
        let layout_name = self.layout.name();
        self.config
            .layout_symbols
            .iter()
            .find(|l| l.name == layout_name)
            .map(|l| l.symbol.clone())
            .unwrap_or_else(|| self.layout.symbol().to_string())
    }

    fn get_keychord_indicator(&self) -> Option<String> {
        match &self.keychord_state {
            keyboard::handlers::KeychordState::Idle => None,
            keyboard::handlers::KeychordState::InProgress {
                candidates,
                keys_pressed,
            } => {
                if candidates.is_empty() {
                    return None;
                }

                let binding = &self.config.keybindings[candidates[0]];
                let mut indicator = String::new();

                for (i, key_press) in binding.keys.iter().take(*keys_pressed).enumerate() {
                    if i > 0 {
                        indicator.push(' ');
                    }

                    for modifier in &key_press.modifiers {
                        indicator.push_str(Self::format_modifier(*modifier));
                        indicator.push('+');
                    }

                    indicator.push_str(&keyboard::keysyms::format_keysym(key_press.keysym));
                }

                indicator.push('-');
                Some(indicator)
            }
        }
    }

    fn format_modifier(modifier: KeyButMask) -> &'static str {
        match modifier {
            KeyButMask::MOD1 => "Alt",
            KeyButMask::MOD4 => "Super",
            KeyButMask::SHIFT => "Shift",
            KeyButMask::CONTROL => "Ctrl",
            _ => "Mod",
        }
    }


    fn update_bar(&mut self) -> WmResult<()> {
        let layout_symbol = self.get_layout_symbol();
        let keychord_indicator = self.get_keychord_indicator();

        for (monitor_index, monitor) in self.monitors.iter().enumerate() {
            if let Some(bar) = self.bars.get_mut(monitor_index) {
                let mut occupied_tags: TagMask = 0;
                for client in self.clients.values() {
                    if client.monitor_index == monitor_index {
                        occupied_tags |= client.tags;
                    }
                }

                let draw_blocks = monitor_index == self.selected_monitor;
                bar.invalidate();
                bar.draw(
                    &self.connection,
                    &self.font,
                    self.display,
                    monitor.tagset[monitor.selected_tags_index],
                    occupied_tags,
                    draw_blocks,
                    &layout_symbol,
                    keychord_indicator.as_deref(),
                )?;
            }
        }
        Ok(())
    }

    fn update_tab_bars(&mut self) -> WmResult<()> {
        for (monitor_index, monitor) in self.monitors.iter().enumerate() {
            if let Some(tab_bar) = self.tab_bars.get_mut(monitor_index) {
                let visible_windows: Vec<Window> = self
                    .windows
                    .iter()
                    .filter(|&&window| {
                        if let Some(client) = self.clients.get(&window) {
                            if client.monitor_index != monitor_index
                                || self.floating_windows.contains(&window)
                                || self.fullscreen_windows.contains(&window)
                            {
                                return false;
                            }
                            (client.tags & monitor.tagset[monitor.selected_tags_index]) != 0
                        } else {
                            false
                        }
                    })
                    .copied()
                    .collect();

                let focused_window = monitor.selected_client;

                tab_bar.draw(
                    &self.connection,
                    &self.font,
                    &visible_windows,
                    focused_window,
                )?;
            }
        }
        Ok(())
    }

    fn handle_key_action(&mut self, action: KeyAction, arg: &Arg) -> WmResult<()> {
        match action {
            KeyAction::Spawn => handlers::handle_spawn_action(action, arg, self.selected_monitor)?,
            KeyAction::SpawnTerminal => {
                use std::process::Command;
                let terminal = &self.config.terminal;
                if let Err(error) = Command::new(terminal).spawn() {
                    eprintln!("Failed to spawn terminal {}: {:?}", terminal, error);
                }
            }
            KeyAction::KillClient => {
                if let Some(focused) = self
                    .monitors
                    .get(self.selected_monitor)
                    .and_then(|m| m.selected_client)
                {
                    self.kill_client(focused)?;
                }
            }
            KeyAction::ToggleFullScreen => {
                self.fullscreen()?;
                self.restack()?;
            }
            KeyAction::ChangeLayout => {
                if let Arg::Str(layout_name) = arg {
                    match layout_from_str(layout_name) {
                        Ok(layout) => {
                            self.layout = layout;
                            if layout_name != "normie" && layout_name != "floating" {
                                self.floating_windows.clear();
                            }
                            self.apply_layout()?;
                            self.update_bar()?;
                            self.restack()?;
                        }
                        Err(e) => eprintln!("Failed to change layout: {}", e),
                    }
                }
            }
            KeyAction::CycleLayout => {
                let current_name = self.layout.name();
                let next_name = next_layout(current_name);
                match layout_from_str(next_name) {
                    Ok(layout) => {
                        self.layout = layout;
                        if next_name != "normie" && next_name != "floating" {
                            self.floating_windows.clear();
                        }
                        self.apply_layout()?;
                        self.update_bar()?;
                        self.restack()?;
                    }
                    Err(e) => eprintln!("Failed to cycle layout: {}", e),
                }
            }
            KeyAction::ToggleFloating => {
                self.toggle_floating()?;
                self.restack()?;
            }

            KeyAction::ExchangeClient => {
                if let Arg::Int(direction) = arg {
                    self.exchange_client(*direction)?;
                }
            }

            KeyAction::FocusStack => {
                if let Arg::Int(direction) = arg {
                    self.focusstack(*direction)?;
                }
            }
            KeyAction::FocusDirection => {
                if let Arg::Int(direction) = arg {
                    self.focus_direction(*direction)?;
                }
            }
            KeyAction::SwapDirection => {
                if let Arg::Int(direction) = arg {
                    self.swap_direction(*direction)?;
                }
            }
            KeyAction::Quit | KeyAction::Restart => {
                // Handled in handle_event
            }
            KeyAction::Recompile => {
                match std::process::Command::new("oxwm")
                    .arg("--recompile")
                    .spawn()
                {
                    Ok(_) => eprintln!("Recompiling in background"),
                    Err(e) => eprintln!("Failed to spawn recompile: {}", e),
                }
            }
            KeyAction::ViewTag => {
                if let Arg::Int(tag_index) = arg {
                    self.view_tag(*tag_index as usize)?;
                }
            }
            KeyAction::ToggleView => {
                if let Arg::Int(tag_index) = arg {
                    self.toggleview(*tag_index as usize)?;
                }
            }
            KeyAction::MoveToTag => {
                if let Arg::Int(tag_index) = arg {
                    self.move_to_tag(*tag_index as usize)?;
                }
            }
            KeyAction::ToggleTag => {
                if let Arg::Int(tag_index) = arg {
                    self.toggletag(*tag_index as usize)?;
                }
            }
            KeyAction::ToggleGaps => {
                self.gaps_enabled = !self.gaps_enabled;
                self.apply_layout()?;
                self.restack()?;
            }
            KeyAction::FocusMonitor => {
                if let Arg::Int(direction) = arg {
                    self.focus_monitor(*direction)?;
                }
            }
            KeyAction::TagMonitor => {
                if let Arg::Int(direction) = arg {
                    self.tag_monitor(*direction)?;
                }
            }
            KeyAction::ShowKeybindOverlay => {
                let monitor = &self.monitors[self.selected_monitor];
                self.keybind_overlay.toggle(
                    &self.connection,
                    &self.font,
                    &self.config.keybindings,
                    monitor.screen_width as u16,
                    monitor.screen_height as u16,
                )?;
            }
            KeyAction::SetMasterFactor => {
                if let Arg::Int(delta) = arg {
                    self.set_master_factor(*delta as f32 / 100.0)?;
                }
            }
            KeyAction::IncNumMaster => {
                if let Arg::Int(delta) = arg {
                    self.inc_num_master(*delta)?;
                }
            }
            KeyAction::None => {}
        }
        Ok(())
    }


    fn is_window_visible(&self, window: Window) -> bool {
        if let Some(client) = self.clients.get(&window) {
            let monitor = self.monitors.get(client.monitor_index);
            let selected_tags = monitor.map(|m| m.tagset[m.selected_tags_index]).unwrap_or(0);
            (client.tags & selected_tags) != 0
        } else {
            false
        }
    }

    fn visible_windows(&self) -> Vec<Window> {
        let mut result = Vec::new();
        for monitor in &self.monitors {
            let mut current = monitor.clients_head;
            while let Some(window) = current {
                if let Some(client) = self.clients.get(&window) {
                    let visible_tags = client.tags & monitor.tagset[monitor.selected_tags_index];
                    if visible_tags != 0 {
                        result.push(window);
                    }
                    current = client.next;
                } else {
                    break;
                }
            }
        }
        result
    }

    fn visible_windows_on_monitor(&self, monitor_index: usize) -> Vec<Window> {
        let mut result = Vec::new();
        if let Some(monitor) = self.monitors.get(monitor_index) {
            let mut current = monitor.clients_head;
            while let Some(window) = current {
                if let Some(client) = self.clients.get(&window) {
                    let visible_tags = client.tags & monitor.tagset[monitor.selected_tags_index];
                    if visible_tags != 0 {
                        result.push(window);
                    }
                    current = client.next;
                } else {
                    break;
                }
            }
        }
        result
    }

    fn get_monitor_at_point(&self, x: i32, y: i32) -> Option<usize> {
        self.monitors
            .iter()
            .position(|mon| mon.contains_point(x, y))
    }

    fn rect_to_monitor(&self, x: i32, y: i32, w: i32, h: i32) -> usize {
        let mut best_monitor = self.selected_monitor;
        let mut max_area = 0;

        for (idx, monitor) in self.monitors.iter().enumerate() {
            let intersect_width = 0.max((x + w).min(monitor.window_area_x + monitor.window_area_width) - x.max(monitor.window_area_x));
            let intersect_height = 0.max((y + h).min(monitor.window_area_y + monitor.window_area_height) - y.max(monitor.window_area_y));
            let area = intersect_width * intersect_height;

            if area > max_area {
                max_area = area;
                best_monitor = idx;
            }
        }

        best_monitor
    }

    fn dir_to_monitor(&self, direction: i32) -> Option<usize> {
        if self.monitors.len() <= 1 {
            return None;
        }

        if direction > 0 {
            if self.selected_monitor + 1 < self.monitors.len() {
                Some(self.selected_monitor + 1)
            } else {
                Some(0)
            }
        } else {
            if self.selected_monitor == 0 {
                Some(self.monitors.len() - 1)
            } else {
                Some(self.selected_monitor - 1)
            }
        }
    }

    // Dwm's g-loaded approach to handling the spam alternating crash.
    fn update_window_visibility(&self) -> WmResult<()> {
        for &window in &self.windows {
            if !self.is_window_visible(window) {
                if let Ok(geom) = self.connection.get_geometry(window)?.reply() {
                    self.connection.configure_window(
                        window,
                        &ConfigureWindowAux::new()
                            .x(-(geom.width as i32 * 2))
                            .y(geom.y as i32),
                    )?;
                }
            }
        }
        self.connection.flush()?;
        Ok(())
    }

    fn showhide(&mut self, window: Option<Window>) -> WmResult<()> {
        let Some(window) = window else {
            return Ok(());
        };

        let Some(client) = self.clients.get(&window).cloned() else {
            return Ok(());
        };

        let monitor = match self.monitors.get(client.monitor_index) {
            Some(m) => m,
            None => return Ok(()),
        };

        let is_visible = (client.tags & monitor.tagset[monitor.selected_tags_index]) != 0;

        if is_visible {
            self.connection.configure_window(
                window,
                &ConfigureWindowAux::new()
                    .x(client.x_position as i32)
                    .y(client.y_position as i32),
            )?;

            let is_floating = client.is_floating;
            let is_fullscreen = client.is_fullscreen;
            let has_no_layout = self.layout.name() == LayoutType::Normie.as_str();

            if (has_no_layout || is_floating) && !is_fullscreen {
                self.connection.configure_window(
                    window,
                    &ConfigureWindowAux::new()
                        .x(client.x_position as i32)
                        .y(client.y_position as i32)
                        .width(client.width as u32)
                        .height(client.height as u32),
                )?;
            }

            self.showhide(client.stack_next)?;
        } else {
            self.showhide(client.stack_next)?;

            let width = client.width_with_border() as i32;
            self.connection.configure_window(
                window,
                &ConfigureWindowAux::new()
                    .x(width * -2)
                    .y(client.y_position as i32),
            )?;
        }

        Ok(())
    }

    pub fn view_tag(&mut self, tag_index: usize) -> WmResult<()> {
        if tag_index >= self.config.tags.len() {
            return Ok(());
        }

        let monitor = match self.monitors.get_mut(self.selected_monitor) {
            Some(m) => m,
            None => return Ok(()),
        };

        let new_tagset = tag_mask(tag_index);

        if new_tagset == monitor.tagset[monitor.selected_tags_index] {
            return Ok(());
        }

        monitor.selected_tags_index ^= 1;
        monitor.tagset[monitor.selected_tags_index] = new_tagset;

        self.save_selected_tags()?;
        self.focus(None)?;
        self.apply_layout()?;  
        self.update_bar()?;

        Ok(())
    }

    pub fn toggleview(&mut self, tag_index: usize) -> WmResult<()> {
        if tag_index >= self.config.tags.len() {
            return Ok(());
        }

        let monitor = match self.monitors.get_mut(self.selected_monitor) {
            Some(m) => m,
            None => return Ok(()),
        };

        let mask = tag_mask(tag_index);
        let new_tagset = monitor.tagset[monitor.selected_tags_index] ^ mask;

        if new_tagset == 0 {
            return Ok(());
        }

        monitor.tagset[monitor.selected_tags_index] = new_tagset;

        self.save_selected_tags()?;
        self.focus(None)?;
        self.apply_layout()?;
        self.update_bar()?;

        Ok(())
    }

    fn save_selected_tags(&self) -> WmResult<()> {
        let net_current_desktop = self.atoms.net_current_desktop;

        let selected_tags = self
            .monitors
            .get(self.selected_monitor)
            .map(|m| m.tagset[m.selected_tags_index])
            .unwrap_or(tag_mask(0));
        let desktop = selected_tags.trailing_zeros();

        let bytes = (desktop as u32).to_ne_bytes();
        self.connection.change_property(
            PropMode::REPLACE,
            self.root,
            net_current_desktop,
            AtomEnum::CARDINAL,
            32,
            1,
            &bytes,
        )?;

        self.connection.flush()?;
        Ok(())
    }

    pub fn move_to_tag(&mut self, tag_index: usize) -> WmResult<()> {
        if tag_index >= self.config.tags.len() {
            return Ok(());
        }

        let focused = match self
            .monitors
            .get(self.selected_monitor)
            .and_then(|m| m.selected_client)
        {
            Some(win) => win,
            None => return Ok(()),
        };

        let mask = tag_mask(tag_index);

        if let Some(client) = self.clients.get_mut(&focused) {
            client.tags = mask;
        }

        if let Err(error) = self.save_client_tag(focused, mask) {
            eprintln!("Failed to save client tag: {:?}", error);
        }

        self.focus(None)?;
        self.apply_layout()?;
        self.update_bar()?;

        Ok(())
    }

    pub fn toggletag(&mut self, tag_index: usize) -> WmResult<()> {
        if tag_index >= self.config.tags.len() {
            return Ok(());
        }

        let focused = match self
            .monitors
            .get(self.selected_monitor)
            .and_then(|m| m.selected_client)
        {
            Some(win) => win,
            None => return Ok(()),
        };

        let mask = tag_mask(tag_index);
        let current_tags = self.clients.get(&focused).map(|c| c.tags).unwrap_or(0);
        let new_tags = current_tags ^ mask;

        if new_tags == 0 {
            return Ok(());
        }

        if let Some(client) = self.clients.get_mut(&focused) {
            client.tags = new_tags;
        }

        if let Err(error) = self.save_client_tag(focused, new_tags) {
            eprintln!("Failed to save client tag: {:?}", error);
        }

        self.focus(None)?;
        self.apply_layout()?;
        self.update_bar()?;

        Ok(())
    }

    pub fn cycle_focus(&mut self, direction: i32) -> WmResult<()> {
        let visible = self.visible_windows();

        if visible.is_empty() {
            return Ok(());
        }

        let current = self
            .monitors
            .get(self.selected_monitor)
            .and_then(|m| m.selected_client);

        let next_window = if let Some(current) = current {
            if let Some(current_index) = visible.iter().position(|&w| w == current) {
                let next_index = if direction > 0 {
                    (current_index + 1) % visible.len()
                } else {
                    (current_index + visible.len() - 1) % visible.len()
                };
                visible[next_index]
            } else {
                visible[0]
            }
        } else {
            visible[0]
        };

        let is_tabbed = self.layout.name() == "tabbed";
        if is_tabbed {
            self.connection.configure_window(
                next_window,
                &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
            )?;
        }

        self.focus(Some(next_window))?;

        if is_tabbed {
            self.update_tab_bars()?;
        }

        Ok(())
    }

    fn find_directional_window_candidate(&mut self, focused_window: Window, direction: i32) -> Option<Window> {
        let visible_windows = self.visible_windows();
        if visible_windows.len() < 2 {
            return None;
        }

        let focused_geometry = self.get_or_query_geometry(focused_window).ok()?;
        let focused_center_x = focused_geometry.x_position + (focused_geometry.width as i16 / 2);
        let focused_center_y = focused_geometry.y_position + (focused_geometry.height as i16 / 2);

        let mut candidates = Vec::new();

        for &window in &visible_windows {
            if window == focused_window {
                continue;
            }

            let geometry = match self.get_or_query_geometry(window) {
                Ok(geometry) => geometry,
                Err(_) => continue,
            };

            let center_x = geometry.x_position + (geometry.width as i16 / 2);
            let center_y = geometry.y_position + (geometry.height as i16 / 2);

            let is_valid_direction = match direction {
                0 => center_y < focused_center_y,
                1 => center_y > focused_center_y,
                2 => center_x < focused_center_x,
                3 => center_x > focused_center_x,
                _ => false,
            };

            if is_valid_direction {
                let delta_x = (center_x - focused_center_x) as i32;
                let delta_y = (center_y - focused_center_y) as i32;
                let distance_squared = delta_x * delta_x + delta_y * delta_y;
                candidates.push((window, distance_squared));
            }
        }

        candidates.iter().min_by_key(|&(_window, distance)| distance).map(|&(window, _distance)| window)
    }

    pub fn focus_direction(&mut self, direction: i32) -> WmResult<()> {
        let focused_window = match self
            .monitors
            .get(self.selected_monitor)
            .and_then(|monitor| monitor.selected_client)
        {
            Some(window) => window,
            None => return Ok(()),
        };

        if let Some(target_window) = self.find_directional_window_candidate(focused_window, direction) {
            self.focus(Some(target_window))?;
        }

        Ok(())
    }

    pub fn swap_direction(&mut self, direction: i32) -> WmResult<()> {
        let focused_window = match self
            .monitors
            .get(self.selected_monitor)
            .and_then(|monitor| monitor.selected_client)
        {
            Some(window) => window,
            None => return Ok(()),
        };

        if let Some(target_window) = self.find_directional_window_candidate(focused_window, direction) {
            let focused_position = self.windows.iter().position(|&window| window == focused_window);
            let target_position = self.windows.iter().position(|&window| window == target_window);

            if let (Some(focused_index), Some(target_index)) = (focused_position, target_position) {
                self.windows.swap(focused_index, target_index);
                self.apply_layout()?;
                self.focus(Some(focused_window))?;

                if let Ok(geometry) = self.get_or_query_geometry(focused_window) {
                    self.connection.warp_pointer(
                        x11rb::NONE,
                        focused_window,
                        0,
                        0,
                        0,
                        0,
                        geometry.width as i16 / 2,
                        geometry.height as i16 / 2,
                    )?;
                }
            }
        }

        Ok(())
    }

    fn grab_next_keys(&self, candidates: &[usize], keys_pressed: usize) -> WmResult<()> {
        use std::collections::HashMap;
        use x11rb::protocol::xproto::Keycode;

        let setup = self.connection.setup();
        let min_keycode = setup.min_keycode;
        let max_keycode = setup.max_keycode;

        let keyboard_mapping = self
            .connection
            .get_keyboard_mapping(min_keycode, max_keycode - min_keycode + 1)?
            .reply()?;

        let mut keysym_to_keycode: HashMap<keyboard::keysyms::Keysym, Vec<Keycode>> =
            HashMap::new();
        let keysyms_per_keycode = keyboard_mapping.keysyms_per_keycode;

        for keycode in min_keycode..=max_keycode {
            let index = (keycode - min_keycode) as usize * keysyms_per_keycode as usize;
            for i in 0..keysyms_per_keycode as usize {
                if let Some(&keysym) = keyboard_mapping.keysyms.get(index + i) {
                    if keysym != 0 {
                        keysym_to_keycode
                            .entry(keysym)
                            .or_insert_with(Vec::new)
                            .push(keycode);
                    }
                }
            }
        }

        let mut grabbed_keys: HashSet<(u16, Keycode)> = HashSet::new();

        let ignore_modifiers = [
            0,
            u16::from(ModMask::LOCK),
            u16::from(ModMask::M2),
            u16::from(ModMask::LOCK | ModMask::M2),
        ];

        for &candidate_index in candidates {
            let binding = &self.config.keybindings[candidate_index];
            if keys_pressed < binding.keys.len() {
                let next_key = &binding.keys[keys_pressed];
                let modifier_mask = keyboard::handlers::modifiers_to_mask(&next_key.modifiers);

                if let Some(keycodes) = keysym_to_keycode.get(&next_key.keysym) {
                    if let Some(&keycode) = keycodes.first() {
                        for &ignore_mask in &ignore_modifiers {
                            let grab_mask = modifier_mask | ignore_mask;
                            let key_tuple = (grab_mask, keycode);

                            if grabbed_keys.insert(key_tuple) {
                                self.connection.grab_key(
                                    false,
                                    self.root,
                                    grab_mask.into(),
                                    keycode,
                                    GrabMode::ASYNC,
                                    GrabMode::ASYNC,
                                )?;
                            }
                        }
                    }
                }
            }
        }

        if let Some(keycodes) = keysym_to_keycode.get(&keyboard::keysyms::XK_ESCAPE) {
            if let Some(&keycode) = keycodes.first() {
                for &ignore_mask in &ignore_modifiers {
                    self.connection.grab_key(
                        false,
                        self.root,
                        ModMask::from(ignore_mask),
                        keycode,
                        GrabMode::ASYNC,
                        GrabMode::ASYNC,
                    )?;
                }
            }
        }

        self.connection.flush()?;
        Ok(())
    }

    fn ungrab_chord_keys(&self) -> WmResult<()> {
        self.connection
            .ungrab_key(x11rb::protocol::xproto::Grab::ANY, self.root, ModMask::ANY)?;
        keyboard::setup_keybinds(&self.connection, self.root, &self.config.keybindings)?;
        self.connection.flush()?;
        Ok(())
    }

    fn kill_client(&self, window: Window) -> WmResult<()> {
        if self.send_event(window, self.atoms.wm_delete_window)? {
            self.connection.flush()?;
        } else {
            eprintln!("Window {} doesn't support WM_DELETE_WINDOW, killing forcefully", window);
            self.connection.kill_client(window)?;
            self.connection.flush()?;
        }
        Ok(())
    }

    fn send_event(&self, window: Window, protocol: Atom) -> WmResult<bool> {
        let protocols_reply = self.connection.get_property(
            false,
            window,
            self.atoms.wm_protocols,
            AtomEnum::ATOM,
            0,
            100,
        )?.reply();

        let protocols_reply = match protocols_reply {
            Ok(reply) => reply,
            Err(_) => return Ok(false),
        };

        let protocols: Vec<Atom> = protocols_reply
            .value
            .chunks_exact(4)
            .map(|chunk| u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        if !protocols.contains(&protocol) {
            return Ok(false);
        }

        let event = x11rb::protocol::xproto::ClientMessageEvent {
            response_type: x11rb::protocol::xproto::CLIENT_MESSAGE_EVENT,
            format: 32,
            sequence: 0,
            window,
            type_: self.atoms.wm_protocols,
            data: x11rb::protocol::xproto::ClientMessageData::from([protocol, x11rb::CURRENT_TIME, 0, 0, 0]),
        };

        self.connection.send_event(
            false,
            window,
            EventMask::NO_EVENT,
            event,
        )?;
        self.connection.flush()?;
        Ok(true)
    }

    fn set_urgent(&mut self, window: Window, urgent: bool) -> WmResult<()> {
        if let Some(client) = self.clients.get_mut(&window) {
            client.is_urgent = urgent;
        }

        let hints_reply = self.connection.get_property(
            false,
            window,
            AtomEnum::WM_HINTS,
            AtomEnum::WM_HINTS,
            0,
            9,
        )?.reply();

        if let Ok(hints) = hints_reply {
            if hints.value.len() >= 4 {
                let mut flags = u32::from_ne_bytes([
                    hints.value[0],
                    hints.value[1],
                    hints.value[2],
                    hints.value[3],
                ]);

                if urgent {
                    flags |= 256;
                } else {
                    flags &= !256;
                }

                let mut new_hints = hints.value.clone();
                new_hints[0..4].copy_from_slice(&flags.to_ne_bytes());

                self.connection.change_property(
                    PropMode::REPLACE,
                    window,
                    AtomEnum::WM_HINTS,
                    AtomEnum::WM_HINTS,
                    32,
                    new_hints.len() as u32 / 4,
                    &new_hints,
                )?;
            }
        }

        Ok(())
    }

    fn get_state(&self, window: Window) -> WmResult<i32> {
        let reply = self.connection.get_property(
            false,
            window,
            self.atoms.wm_state,
            self.atoms.wm_state,
            0,
            2,
        )?.reply();

        match reply {
            Ok(prop) if !prop.value.is_empty() && prop.value.len() >= 4 => {
                let state = u32::from_ne_bytes([
                    prop.value[0],
                    prop.value[1],
                    prop.value[2],
                    prop.value[3],
                ]);
                Ok(state as i32)
            }
            _ => Ok(-1),
        }
    }

    fn get_atom_prop(&self, window: Window, property: Atom) -> WmResult<Option<Atom>> {
        let reply = self.connection.get_property(
            false,
            window,
            property,
            AtomEnum::ATOM,
            0,
            1,
        )?.reply();

        match reply {
            Ok(prop) if !prop.value.is_empty() && prop.value.len() >= 4 => {
                let atom = u32::from_ne_bytes([
                    prop.value[0],
                    prop.value[1],
                    prop.value[2],
                    prop.value[3],
                ]);
                Ok(Some(atom))
            }
            _ => Ok(None),
        }
    }

    fn get_text_prop(&self, window: Window, atom: Atom) -> WmResult<Option<String>> {
        let reply = self.connection.get_property(
            false,
            window,
            atom,
            AtomEnum::ANY,
            0,
            1024,
        )?.reply();

        match reply {
            Ok(prop) if !prop.value.is_empty() => {
                let text = String::from_utf8_lossy(&prop.value).to_string();
                Ok(Some(text.trim_end_matches('\0').to_string()))
            }
            _ => Ok(None),
        }
    }

    fn fullscreen(&mut self) -> WmResult<()> {
        if self.show_bar {
            let windows: Vec<Window> = self.windows.iter()
                .filter(|&&w| self.is_window_visible(w))
                .copied()
                .collect();

            for window in &windows {
                if let Ok(geom) = self.connection.get_geometry(*window)?.reply() {
                        self.floating_geometry_before_fullscreen.insert(
                            *window,
                            (geom.x, geom.y, geom.width, geom.height, geom.border_width as u16),
                        );
                    }
            }

            self.last_layout = Some(self.layout.name());
            if let Ok(layout) = layout_from_str("monocle") {
                self.layout = layout;
            }
            self.toggle_bar()?;
            self.apply_layout()?;

            let border_width = self.config.border_width;
            let floating_windows: Vec<Window> = windows.iter()
                .filter(|&&w| self.floating_windows.contains(&w))
                .copied()
                .collect();

            for window in floating_windows {
                let monitor_idx = self.clients.get(&window)
                    .map(|c| c.monitor_index)
                    .unwrap_or(self.selected_monitor);
                let monitor = &self.monitors[monitor_idx];

                let (outer_gap_h, outer_gap_v) = if self.gaps_enabled {
                    (
                        self.config.gap_outer_horizontal,
                        self.config.gap_outer_vertical,
                    )
                } else {
                    (0, 0)
                };

                let window_x = monitor.screen_x + outer_gap_h as i32;
                let window_y = monitor.screen_y + outer_gap_v as i32;
                let window_width = monitor.screen_width.saturating_sub(2 * outer_gap_h as i32).saturating_sub(2 * border_width as i32);
                let window_height = monitor.screen_height.saturating_sub(2 * outer_gap_v as i32).saturating_sub(2 * border_width as i32);

                self.connection.configure_window(
                    window,
                    &x11rb::protocol::xproto::ConfigureWindowAux::new()
                        .x(window_x)
                        .y(window_y)
                        .width(window_width as u32)
                        .height(window_height as u32),
                )?;
            }
            self.connection.flush()?;
        } else {
            if let Some(last) = self.last_layout {
                if let Ok(layout) = layout_from_str(last) {
                    self.layout = layout;
                }
            }

            let windows_to_restore: Vec<Window> = self.floating_geometry_before_fullscreen
                .keys()
                .copied()
                .collect();

            for window in windows_to_restore {
                if let Some(&(x, y, width, height, border_width)) = self.floating_geometry_before_fullscreen.get(&window) {
                    self.connection.configure_window(
                        window,
                        &x11rb::protocol::xproto::ConfigureWindowAux::new()
                            .x(x as i32)
                            .y(y as i32)
                            .width(width as u32)
                            .height(height as u32)
                            .border_width(border_width as u32),
                    )?;

                    self.update_geometry_cache(window, CachedGeometry {
                        x_position: x,
                        y_position: y,
                        width,
                        height,
                        border_width,
                    });

                    self.floating_geometry_before_fullscreen.remove(&window);
                }
            }
            self.connection.flush()?;

            self.toggle_bar()?;

            if self.layout.name() != "normie" {
                self.apply_layout()?;
            } else {
                if let Some(bar) = self.bars.get(self.selected_monitor) {
                    self.connection.configure_window(
                        bar.window(),
                        &x11rb::protocol::xproto::ConfigureWindowAux::new()
                            .stack_mode(x11rb::protocol::xproto::StackMode::ABOVE),
                    )?;
                    self.connection.flush()?;
                }
            }
        }
        Ok(())
    }

    fn set_window_fullscreen(&mut self, window: Window, fullscreen: bool) -> WmResult<()> {
        let monitor_idx = self.clients.get(&window)
            .map(|c| c.monitor_index)
            .unwrap_or(self.selected_monitor);
        let monitor = &self.monitors[monitor_idx];

        if fullscreen && !self.fullscreen_windows.contains(&window) {
            let bytes = self.atoms.net_wm_state_fullscreen.to_ne_bytes().to_vec();
            self.connection.change_property(
                PropMode::REPLACE,
                window,
                self.atoms.net_wm_state,
                AtomEnum::ATOM,
                32,
                1,
                &bytes,
            )?;

            if let Some(client) = self.clients.get_mut(&window) {
                client.is_fullscreen = true;
                client.old_state = client.is_floating;
                client.old_border_width = client.border_width;
                client.border_width = 0;
                client.is_floating = true;
            }

            self.fullscreen_windows.insert(window);

            self.connection.configure_window(
                window,
                &x11rb::protocol::xproto::ConfigureWindowAux::new()
                    .border_width(0)
                    .x(monitor.screen_x)
                    .y(monitor.screen_y)
                    .width(monitor.screen_width as u32)
                    .height(monitor.screen_height as u32)
                    .stack_mode(x11rb::protocol::xproto::StackMode::ABOVE),
            )?;

            self.connection.flush()?;
        } else if !fullscreen && self.fullscreen_windows.contains(&window) {
            self.connection.change_property(
                PropMode::REPLACE,
                window,
                self.atoms.net_wm_state,
                AtomEnum::ATOM,
                32,
                0,
                &[],
            )?;

            self.fullscreen_windows.remove(&window);

            if let Some(client) = self.clients.get_mut(&window) {
                client.is_fullscreen = false;
                client.is_floating = client.old_state;
                client.border_width = client.old_border_width;

                let x = client.old_x_position;
                let y = client.old_y_position;
                let w = client.old_width;
                let h = client.old_height;
                let bw = client.border_width;

                self.connection.configure_window(
                    window,
                    &x11rb::protocol::xproto::ConfigureWindowAux::new()
                        .x(x as i32)
                        .y(y as i32)
                        .width(w as u32)
                        .height(h as u32)
                        .border_width(bw as u32),
                )?;
            }

            self.apply_layout()?;
        }

        Ok(())
    }

    fn toggle_bar(&mut self) -> WmResult<()> {
        self.show_bar = !self.show_bar;
        if let Some(bar) = self.bars.get(self.selected_monitor) {
            if self.show_bar {
                self.connection.map_window(bar.window())?;
            } else {
                self.connection.unmap_window(bar.window())?;
            }
            self.connection.flush()?;
        }
        self.apply_layout()?;
        Ok(())
    }

    fn get_transient_parent(&self, window: Window) -> Option<Window> {
        self.connection
            .get_property(
                false,
                window,
                AtomEnum::WM_TRANSIENT_FOR,
                AtomEnum::WINDOW,
                0,
                1,
            )
            .ok()
            .and_then(|cookie| cookie.reply().ok())
            .filter(|reply| !reply.value.is_empty())
            .and_then(|reply| {
                if reply.value.len() >= 4 {
                    let parent_window = u32::from_ne_bytes([
                        reply.value[0],
                        reply.value[1],
                        reply.value[2],
                        reply.value[3],
                    ]);
                    Some(parent_window)
                } else {
                    None
                }
            })
    }

    fn is_transient_window(&self, window: Window) -> bool {
        self.get_transient_parent(window).is_some()
    }

    fn is_dialog_window(&self, window: Window) -> bool {
        let window_type_property = self.connection
            .get_property(
                false,
                window,
                self.atoms.net_wm_window_type,
                AtomEnum::ATOM,
                0,
                32,
            )
            .ok()
            .and_then(|cookie| cookie.reply().ok());

        if let Some(reply) = window_type_property {
            let atoms: Vec<Atom> = reply
                .value
                .chunks_exact(4)
                .map(|chunk| u32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect();

            atoms.contains(&self.atoms.net_wm_window_type_dialog)
        } else {
            false
        }
    }

    fn get_window_class(&self, window: Window) -> Option<String> {
        self.connection
            .get_property(false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 1024)
            .ok()
            .and_then(|cookie| cookie.reply().ok())
            .and_then(|reply| {
                if reply.value.is_empty() {
                    None
                } else {
                    std::str::from_utf8(&reply.value).ok().map(|s| {
                        s.split('\0').nth(1).unwrap_or(s.split('\0').next().unwrap_or("")).to_string()
                    })
                }
            })
    }

    fn get_window_class_instance(&self, window: Window) -> (String, String) {
        let reply = self.connection
            .get_property(false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 1024)
            .ok()
            .and_then(|cookie| cookie.reply().ok());

        if let Some(reply) = reply {
            if !reply.value.is_empty() {
                if let Ok(text) = std::str::from_utf8(&reply.value) {
                    let parts: Vec<&str> = text.split('\0').collect();
                    let instance = parts.get(0).unwrap_or(&"").to_string();
                    let class = parts.get(1).unwrap_or(&"").to_string();
                    return (instance, class);
                }
            }
        }

        (String::new(), String::new())
    }

    fn apply_rules(&mut self, window: Window) -> WmResult<()> {
        let (instance, class) = self.get_window_class_instance(window);
        let title = self.clients.get(&window).map(|c| c.name.clone()).unwrap_or_default();

        let mut rule_tags: Option<u32> = None;
        let mut rule_floating: Option<bool> = None;
        let mut rule_monitor: Option<usize> = None;

        for rule in &self.config.window_rules {
            if rule.matches(&class, &instance, &title) {
                if rule.tags.is_some() {
                    rule_tags = rule.tags;
                }
                if rule.is_floating.is_some() {
                    rule_floating = rule.is_floating;
                }
                if rule.monitor.is_some() {
                    rule_monitor = rule.monitor;
                }
            }
        }

        if let Some(client) = self.clients.get_mut(&window) {
            if let Some(is_floating) = rule_floating {
                client.is_floating = is_floating;
                if is_floating {
                    self.floating_windows.insert(window);
                } else {
                    self.floating_windows.remove(&window);
                }
            }

            if let Some(monitor_index) = rule_monitor {
                if monitor_index < self.monitors.len() {
                    client.monitor_index = monitor_index;
                }
            }

            let tags = rule_tags.unwrap_or_else(|| {
                self.monitors
                    .get(client.monitor_index)
                    .map(|m| m.tagset[m.selected_tags_index])
                    .unwrap_or(tag_mask(0))
            });

            client.tags = tags;
        }

        Ok(())
    }

    fn manage_window(&mut self, window: Window) -> WmResult<()> {
        let geometry = self.connection.get_geometry(window)?.reply()?;
        let mut window_x = geometry.x as i32;
        let mut window_y = geometry.y as i32;
        let window_width = geometry.width as u32;
        let window_height = geometry.height as u32;

        let transient_parent = self.get_transient_parent(window);
        let (window_tags, monitor_index) = if let Some(parent) = transient_parent {
            if let Some(parent_client) = self.clients.get(&parent) {
                (parent_client.tags, parent_client.monitor_index)
            } else {
                let tags = self.monitors
                    .get(self.selected_monitor)
                    .map(|m| m.tagset[m.selected_tags_index])
                    .unwrap_or(tag_mask(0));
                (tags, self.selected_monitor)
            }
        } else {
            let selected_tags = self.monitors
                .get(self.selected_monitor)
                .map(|m| m.tagset[m.selected_tags_index])
                .unwrap_or(tag_mask(0));
            (selected_tags, self.selected_monitor)
        };

        let monitor = self.monitors[monitor_index].clone();
        let border_width = self.config.border_width;

        let mut client = Client::new(window, monitor_index, window_tags);
        client.x_position = window_x as i16;
        client.y_position = window_y as i16;
        client.width = window_width as u16;
        client.height = window_height as u16;
        client.old_x_position = window_x as i16;
        client.old_y_position = window_y as i16;
        client.old_width = window_width as u16;
        client.old_height = window_height as u16;
        client.border_width = border_width as u16;
        client.old_border_width = border_width as u16;

        if window_x + (window_width as i32) + (2 * border_width as i32) > monitor.screen_x + monitor.screen_width as i32 {
            window_x = monitor.screen_x + monitor.screen_width as i32 - (window_width as i32) - (2 * border_width as i32);
        }
        if window_y + (window_height as i32) + (2 * border_width as i32) > monitor.screen_y + monitor.screen_height as i32 {
            window_y = monitor.screen_y + monitor.screen_height as i32 - (window_height as i32) - (2 * border_width as i32);
        }
        window_x = window_x.max(monitor.screen_x);
        window_y = window_y.max(monitor.screen_y);

        let is_transient = transient_parent.is_some();
        let is_dialog = self.is_dialog_window(window);

        let class_name = self.get_window_class(window).unwrap_or_default();
        eprintln!("MapRequest 0x{:x}: class='{}' size={}x{} pos=({},{}) transient={} dialog={}",
            window, class_name, window_width, window_height, window_x, window_y, is_transient, is_dialog);

        let off_screen_x = window_x + (2 * self.screen.width_in_pixels as i32);

        self.connection.configure_window(
            window,
            &ConfigureWindowAux::new()
                .x(off_screen_x)
                .y(window_y)
                .width(window_width)
                .height(window_height)
                .border_width(border_width)
        )?;

        self.connection.change_window_attributes(
            window,
            &ChangeWindowAttributesAux::new().event_mask(
                EventMask::ENTER_WINDOW | EventMask::STRUCTURE_NOTIFY | EventMask::PROPERTY_CHANGE
            ),
        )?;

        client.is_floating = is_transient || is_dialog;

        self.clients.insert(window, client);
        self.update_size_hints(window)?;
        self.update_window_title(window)?;
        self.apply_rules(window)?;

        let updated_monitor_index = self.clients.get(&window).map(|c| c.monitor_index).unwrap_or(monitor_index);
        let updated_monitor = self.monitors.get(updated_monitor_index).cloned().unwrap_or(monitor.clone());
        let is_rule_floating = self.clients.get(&window).map(|c| c.is_floating).unwrap_or(false);

        self.attach_aside(window, updated_monitor_index);
        self.attach_stack(window, updated_monitor_index);

        self.windows.push(window);

        if is_transient || is_dialog {
            self.floating_windows.insert(window);

            let (center_x, center_y) = if let Some(parent) = transient_parent {
                if let Ok(parent_geom) = self.connection.get_geometry(parent)?.reply() {
                    let parent_center_x = parent_geom.x as i32 + (parent_geom.width as i32 / 2);
                    let parent_center_y = parent_geom.y as i32 + (parent_geom.height as i32 / 2);
                    (parent_center_x, parent_center_y)
                } else {
                    let monitor_center_x = monitor.screen_x + (monitor.screen_width as i32 / 2);
                    let monitor_center_y = monitor.screen_y + (monitor.screen_height as i32 / 2);
                    (monitor_center_x, monitor_center_y)
                }
            } else {
                let monitor_center_x = monitor.screen_x + (monitor.screen_width as i32 / 2);
                let monitor_center_y = monitor.screen_y + (monitor.screen_height as i32 / 2);
                (monitor_center_x, monitor_center_y)
            };

            let positioned_x = center_x - (window_width as i32 / 2);
            let positioned_y = center_y - (window_height as i32 / 2);

            let clamped_x = positioned_x
                .max(monitor.screen_x)
                .min(monitor.screen_x + monitor.screen_width as i32 - window_width as i32);
            let clamped_y = positioned_y
                .max(monitor.screen_y)
                .min(monitor.screen_y + monitor.screen_height as i32 - window_height as i32);

            self.update_geometry_cache(window, CachedGeometry {
                x_position: clamped_x as i16,
                y_position: clamped_y as i16,
                width: window_width as u16,
                height: window_height as u16,
                border_width: border_width as u16,
            });

            self.connection.configure_window(
                window,
                &ConfigureWindowAux::new()
                    .x(clamped_x)
                    .y(clamped_y)
                    .width(window_width)
                    .height(window_height)
                    .border_width(border_width)
                    .stack_mode(StackMode::ABOVE),
            )?;
        } else if is_rule_floating && !is_transient && !is_dialog {
            let mut adjusted_x = window_x;
            let mut adjusted_y = window_y;

            if adjusted_x + (window_width as i32) + (2 * border_width as i32) > updated_monitor.screen_x + updated_monitor.screen_width as i32 {
                adjusted_x = updated_monitor.screen_x + updated_monitor.screen_width as i32 - (window_width as i32) - (2 * border_width as i32);
            }
            if adjusted_y + (window_height as i32) + (2 * border_width as i32) > updated_monitor.screen_y + updated_monitor.screen_height as i32 {
                adjusted_y = updated_monitor.screen_y + updated_monitor.screen_height as i32 - (window_height as i32) - (2 * border_width as i32);
            }
            adjusted_x = adjusted_x.max(updated_monitor.screen_x);
            adjusted_y = adjusted_y.max(updated_monitor.screen_y);

            if let Some(client) = self.clients.get_mut(&window) {
                client.x_position = adjusted_x as i16;
                client.y_position = adjusted_y as i16;
                client.width = window_width as u16;
                client.height = window_height as u16;
            }

            self.update_geometry_cache(window, CachedGeometry {
                x_position: adjusted_x as i16,
                y_position: adjusted_y as i16,
                width: window_width as u16,
                height: window_height as u16,
                border_width: border_width as u16,
            });

            self.connection.configure_window(
                window,
                &ConfigureWindowAux::new()
                    .x(adjusted_x)
                    .y(adjusted_y)
                    .width(window_width)
                    .height(window_height)
                    .border_width(border_width)
                    .stack_mode(StackMode::ABOVE),
            )?;
        }

        let is_normie_layout = self.layout.name() == "normie";
        if is_normie_layout && !is_transient && !is_dialog && !is_rule_floating {
            if let Ok(pointer) = self.connection.query_pointer(self.root)?.reply() {
                let cursor_monitor = self.get_monitor_at_point(pointer.root_x as i32, pointer.root_y as i32)
                    .and_then(|idx| self.monitors.get(idx))
                    .unwrap_or(&monitor);

                let float_width = (cursor_monitor.screen_width as f32 * 0.6) as u32;
                let float_height = (cursor_monitor.screen_height as f32 * 0.6) as u32;
                let spawn_x = pointer.root_x as i32 - (float_width as i32 / 2);
                let spawn_y = pointer.root_y as i32 - (float_height as i32 / 2);

                let clamped_x = spawn_x
                    .max(cursor_monitor.screen_x)
                    .min(cursor_monitor.screen_x + cursor_monitor.screen_width as i32 - float_width as i32);
                let clamped_y = spawn_y
                    .max(cursor_monitor.screen_y)
                    .min(cursor_monitor.screen_y + cursor_monitor.screen_height as i32 - float_height as i32);

                self.connection.configure_window(
                    window,
                    &ConfigureWindowAux::new()
                        .x(clamped_x)
                        .y(clamped_y)
                        .width(float_width)
                        .height(float_height)
                        .border_width(border_width)
                        .stack_mode(StackMode::ABOVE),
                )?;

                if let Some(client) = self.clients.get_mut(&window) {
                    client.is_floating = true;
                }
                self.floating_windows.insert(window);
            }
        }

        self.set_wm_state(window, 1)?;
        if let Err(error) = self.save_client_tag(window, window_tags) {
            eprintln!("Failed to save client tag for new window: {:?}", error);
        }

        self.apply_layout()?;
        self.connection.map_window(window)?;
        self.update_bar()?;
        self.focus(Some(window))?;

        if self.layout.name() == "tabbed" {
            self.update_tab_bars()?;
        }

        Ok(())
    }

    pub fn set_focus(&mut self, window: Window) -> WmResult<()> {
        let old_focused = self.previous_focused;

        if let Some(monitor) = self.monitors.get_mut(self.selected_monitor) {
            monitor.selected_client = Some(window);
        }

        self.connection
            .set_input_focus(InputFocus::POINTER_ROOT, window, x11rb::CURRENT_TIME)?;
        self.connection.flush()?;

        self.update_focus_visuals(old_focused, window)?;
        self.previous_focused = Some(window);

        if self.layout.name() == "tabbed" {
            self.update_tab_bars()?;
        }

        Ok(())
    }

    fn unfocus(&self, window: Window) -> WmResult<()> {
        if !self.windows.contains(&window) {
            return Ok(());
        }

        self.connection.change_window_attributes(
            window,
            &ChangeWindowAttributesAux::new().border_pixel(self.config.border_unfocused),
        )?;

        self.connection.grab_button(
            false,
            window,
            EventMask::BUTTON_PRESS.into(),
            GrabMode::SYNC,
            GrabMode::SYNC,
            x11rb::NONE,
            x11rb::NONE,
            ButtonIndex::ANY,
            ModMask::ANY,
        )?;

        Ok(())
    }

    fn focus(&mut self, window: Option<Window>) -> WmResult<()> {
        let monitor = self.monitors.get_mut(self.selected_monitor).unwrap();
        let old_selected = monitor.selected_client;

        if let Some(old_win) = old_selected {
            if old_selected != window {
                self.unfocus(old_win)?;
            }
        }

        if let Some(win) = window {
            if !self.windows.contains(&win) {
                return Ok(());
            }

            let monitor_idx = self.clients.get(&win)
                .map(|c| c.monitor_index)
                .unwrap_or(self.selected_monitor);
            if monitor_idx != self.selected_monitor {
                self.selected_monitor = monitor_idx;
            }

            self.detach_stack(win);
            self.attach_stack(win, monitor_idx);

            self.connection.change_window_attributes(
                win,
                &ChangeWindowAttributesAux::new().border_pixel(self.config.border_focused),
            )?;

            self.connection.ungrab_button(ButtonIndex::ANY, win, ModMask::ANY)?;

            self.connection.set_input_focus(
                InputFocus::POINTER_ROOT,
                win,
                x11rb::CURRENT_TIME,
            )?;

            if let Some(monitor) = self.monitors.get_mut(self.selected_monitor) {
                monitor.selected_client = Some(win);
            }

            self.previous_focused = Some(win);
        } else {
            self.connection.set_input_focus(
                InputFocus::POINTER_ROOT,
                self.root,
                x11rb::CURRENT_TIME,
            )?;

            if let Some(monitor) = self.monitors.get_mut(self.selected_monitor) {
                monitor.selected_client = None;
            }
        }

        self.restack()?;
        self.connection.flush()?;

        Ok(())
    }

    fn restack(&mut self) -> WmResult<()> {
        let monitor = match self.monitors.get(self.selected_monitor) {
            Some(m) => m,
            None => return Ok(()),
        };

        let mut windows_to_restack: Vec<Window> = Vec::new();

        if let Some(selected) = monitor.selected_client {
            if self.floating_windows.contains(&selected) {
                windows_to_restack.push(selected);
            }
        }

        let mut current = monitor.stack_head;
        while let Some(win) = current {
            if self.windows.contains(&win) && self.floating_windows.contains(&win) {
                if Some(win) != monitor.selected_client {
                    windows_to_restack.push(win);
                }
            }
            current = self.clients.get(&win).and_then(|c| c.stack_next);
        }

        current = monitor.stack_head;
        while let Some(win) = current {
            if self.windows.contains(&win) && !self.floating_windows.contains(&win) {
                windows_to_restack.push(win);
            }
            current = self.clients.get(&win).and_then(|c| c.stack_next);
        }

        for (i, &win) in windows_to_restack.iter().enumerate() {
            if i == 0 {
                self.connection.configure_window(
                    win,
                    &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
                )?;
            } else {
                self.connection.configure_window(
                    win,
                    &ConfigureWindowAux::new()
                        .sibling(windows_to_restack[i - 1])
                        .stack_mode(StackMode::BELOW),
                )?;
            }
        }

        Ok(())
    }

    fn focusstack(&mut self, direction: i32) -> WmResult<()> {
        let monitor = match self.monitors.get(self.selected_monitor) {
            Some(m) => m,
            None => return Ok(()),
        };

        let selected = match monitor.selected_client {
            Some(win) => win,
            None => return Ok(()),
        };

        let selected_tags = monitor.tagset[monitor.selected_tags_index];

        let mut stack_windows: Vec<Window> = Vec::new();
        let mut current = monitor.clients_head;
        while let Some(win) = current {
            if let Some(client) = self.clients.get(&win) {
                if client.tags & selected_tags != 0 && !client.is_floating {
                    stack_windows.push(win);
                }
                current = client.next;
            } else {
                break;
            }
        }

        if stack_windows.is_empty() {
            return Ok(());
        }

        let current_idx = stack_windows.iter().position(|&w| w == selected);

        let next_window = if let Some(idx) = current_idx {
            if direction > 0 {
                if idx + 1 < stack_windows.len() {
                    stack_windows[idx + 1]
                } else {
                    stack_windows[0]
                }
            } else {
                if idx > 0 {
                    stack_windows[idx - 1]
                } else {
                    stack_windows[stack_windows.len() - 1]
                }
            }
        } else {
            return Ok(());
        };

        self.focus(Some(next_window))?;

        Ok(())
    }

    pub fn focus_monitor(&mut self, direction: i32) -> WmResult<()> {
        if self.monitors.len() <= 1 {
            return Ok(());
        }

        let target_monitor = match self.dir_to_monitor(direction) {
            Some(idx) if idx != self.selected_monitor => idx,
            _ => return Ok(()),
        };

        let old_selected = self.monitors
            .get(self.selected_monitor)
            .and_then(|m| m.selected_client);

        if let Some(win) = old_selected {
            self.unfocus(win)?;
        }

        self.selected_monitor = target_monitor;
        self.focus(None)?;

        Ok(())
    }

    pub fn tag_monitor(&mut self, direction: i32) -> WmResult<()> {
        if self.monitors.len() <= 1 {
            return Ok(());
        }

        let selected_window = self.monitors
            .get(self.selected_monitor)
            .and_then(|m| m.selected_client);

        let window = match selected_window {
            Some(win) => win,
            None => return Ok(()),
        };

        let target_monitor = match self.dir_to_monitor(direction) {
            Some(idx) => idx,
            None => return Ok(()),
        };

        self.send_to_monitor(window, target_monitor)?;

        Ok(())
    }

    fn update_focus_visuals(
        &self,
        old_focused: Option<Window>,
        new_focused: Window,
    ) -> WmResult<()> {
        if let Some(old_win) = old_focused {
            if old_win != new_focused {
                self.connection.configure_window(
                    old_win,
                    &ConfigureWindowAux::new().border_width(self.config.border_width),
                )?;

                self.connection.change_window_attributes(
                    old_win,
                    &ChangeWindowAttributesAux::new().border_pixel(self.config.border_unfocused),
                )?;
            }
        }

        self.connection.configure_window(
            new_focused,
            &ConfigureWindowAux::new().border_width(self.config.border_width),
        )?;

        self.connection.change_window_attributes(
            new_focused,
            &ChangeWindowAttributesAux::new().border_pixel(self.config.border_focused),
        )?;

        self.connection.flush()?;
        Ok(())
    }

    fn move_mouse(&mut self, window: Window) -> WmResult<()> {
        self.floating_windows.insert(window);

        let geometry = self.connection.get_geometry(window)?.reply()?;

        self.connection
            .grab_pointer(
                false,
                self.root,
                (EventMask::POINTER_MOTION | EventMask::BUTTON_RELEASE).into(),
                GrabMode::ASYNC,
                GrabMode::ASYNC,
                x11rb::NONE,
                x11rb::NONE,
                x11rb::CURRENT_TIME,
            )?
            .reply()?;

        let pointer = self.connection.query_pointer(self.root)?.reply()?;
        let (start_x, start_y) = (pointer.root_x, pointer.root_y);

        self.connection.configure_window(
            window,
            &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
        )?;

        loop {
            let event = self.connection.wait_for_event()?;
            match event {
                Event::MotionNotify(e) => {
                    let new_x = geometry.x + (e.root_x - start_x);
                    let new_y = geometry.y + (e.root_y - start_y);
                    self.connection.configure_window(
                        window,
                        &ConfigureWindowAux::new().x(new_x as i32).y(new_y as i32),
                    )?;
                    self.connection.flush()?;
                }
                Event::ButtonRelease(_) => break,
                _ => {}
            }
        }

        self.connection
            .ungrab_pointer(x11rb::CURRENT_TIME)?
            .check()?;
        self.connection
            .allow_events(Allow::REPLAY_POINTER, x11rb::CURRENT_TIME)?
            .check()?;

        Ok(())
    }

    fn resize_mouse(&mut self, window: Window) -> WmResult<()> {
        self.floating_windows.insert(window);

        let geometry = self.connection.get_geometry(window)?.reply()?;

        self.connection.warp_pointer(
            x11rb::NONE,
            window,
            0,
            0,
            0,
            0,
            geometry.width as i16,
            geometry.height as i16,
        )?;

        self.connection
            .grab_pointer(
                false,
                self.root,
                (EventMask::POINTER_MOTION | EventMask::BUTTON_RELEASE).into(),
                GrabMode::ASYNC,
                GrabMode::ASYNC,
                x11rb::NONE,
                x11rb::NONE,
                x11rb::CURRENT_TIME,
            )?
            .reply()?;

        self.connection.configure_window(
            window,
            &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
        )?;

        loop {
            let event = self.connection.wait_for_event()?;
            match event {
                Event::MotionNotify(e) => {
                    let new_width = (e.root_x - geometry.x).max(1) as u32;
                    let new_height = (e.root_y - geometry.y).max(1) as u32;

                    self.connection.configure_window(
                        window,
                        &ConfigureWindowAux::new()
                            .width(new_width)
                            .height(new_height),
                    )?;
                    self.connection.flush()?;
                }
                Event::ButtonRelease(_) => break,
                _ => {}
            }
        }

        self.connection
            .ungrab_pointer(x11rb::CURRENT_TIME)?
            .check()?;
        self.connection
            .allow_events(Allow::REPLAY_POINTER, x11rb::CURRENT_TIME)?
            .check()?;

        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> WmResult<Option<bool>> {
        match event {
            Event::KeyPress(ref key_event) if key_event.event == self.overlay.window() => {
                if self.overlay.is_visible() {
                    if let Err(error) = self.overlay.hide(&self.connection) {
                        eprintln!("Failed to hide overlay: {:?}", error);
                    }
                }
                return Ok(None);
            }
            Event::ButtonPress(ref button_event) if button_event.event == self.overlay.window() => {
                if self.overlay.is_visible() {
                    if let Err(error) = self.overlay.hide(&self.connection) {
                        eprintln!("Failed to hide overlay: {:?}", error);
                    }
                }
                return Ok(None);
            }
            Event::Expose(ref expose_event) if expose_event.window == self.overlay.window() => {
                if self.overlay.is_visible() {
                    if let Err(error) = self.overlay.draw(&self.connection, &self.font) {
                        eprintln!("Failed to draw overlay: {:?}", error);
                    }
                }
                return Ok(None);
            }
            Event::KeyPress(ref e) if e.event == self.keybind_overlay.window() => {
                if self.keybind_overlay.is_visible()
                    && !self.keybind_overlay.should_suppress_input()
                {
                    use crate::keyboard::keysyms;
                    let keyboard_mapping = self
                        .connection
                        .get_keyboard_mapping(
                            self.connection.setup().min_keycode,
                            self.connection.setup().max_keycode
                                - self.connection.setup().min_keycode
                                + 1,
                        )?
                        .reply()?;

                    let min_keycode = self.connection.setup().min_keycode;
                    let keysyms_per_keycode = keyboard_mapping.keysyms_per_keycode;
                    let index = (e.detail - min_keycode) as usize * keysyms_per_keycode as usize;

                    if let Some(&keysym) = keyboard_mapping.keysyms.get(index) {
                        if keysym == keysyms::XK_ESCAPE || keysym == keysyms::XK_Q {
                            if let Err(error) = self.keybind_overlay.hide(&self.connection) {
                                eprintln!("Failed to hide keybind overlay: {:?}", error);
                            }
                        }
                    }
                }
                return Ok(None);
            }
            Event::ButtonPress(ref e) if e.event == self.keybind_overlay.window() => {
                return Ok(None);
            }
            Event::Expose(ref expose_event) if expose_event.window == self.keybind_overlay.window() => {
                if self.keybind_overlay.is_visible() {
                    if let Err(error) = self.keybind_overlay.draw(&self.connection, &self.font) {
                        eprintln!("Failed to draw keybind overlay: {:?}", error);
                    }
                }
                return Ok(None);
            }
            Event::MapRequest(event) => {
                let attrs = match self.connection.get_window_attributes(event.window)?.reply() {
                    Ok(attrs) => attrs,
                    Err(_) => return Ok(None),
                };

                if attrs.override_redirect {
                    return Ok(None);
                }

                if !self.windows.contains(&event.window) {
                    self.manage_window(event.window)?;
                }
            }
            Event::UnmapNotify(event) => {
                if self.windows.contains(&event.window) && self.is_window_visible(event.window) {
                    self.remove_window(event.window)?;
                }
            }
            Event::DestroyNotify(event) => {
                if self.windows.contains(&event.window) {
                    self.remove_window(event.window)?;
                }
            }
            Event::PropertyNotify(event) => {
                if !self.windows.contains(&event.window) {
                    return Ok(None);
                }

                if event.atom == self.atoms.wm_name || event.atom == self.atoms.net_wm_name {
                    let _ = self.update_window_title(event.window);
                    if self.layout.name() == "tabbed" {
                        self.update_tab_bars()?;
                    }
                } else if event.atom == self.atoms.wm_normal_hints {
                    let _ = self.update_size_hints(event.window);
                }
            }
            Event::EnterNotify(event) => {
                if event.mode != x11rb::protocol::xproto::NotifyMode::NORMAL {
                    return Ok(None);
                }
                if self.windows.contains(&event.event) {
                    if let Some(client) = self.clients.get(&event.event) {
                        if client.monitor_index != self.selected_monitor {
                            self.selected_monitor = client.monitor_index;
                            self.update_bar()?;
                        }
                    }
                    self.focus(Some(event.event))?;
                }
            }
            Event::MotionNotify(event) => {
                if event.event != self.root {
                    return Ok(None);
                }

                if let Some(monitor_index) =
                    self.get_monitor_at_point(event.root_x as i32, event.root_y as i32)
                {
                    if monitor_index != self.selected_monitor {
                        self.selected_monitor = monitor_index;
                        self.update_bar()?;

                        let visible = self.visible_windows_on_monitor(monitor_index);
                        if let Some(&win) = visible.first() {
                            self.focus(Some(win))?;
                        }
                    }
                }
            }
            Event::KeyPress(event) => {
                let result = keyboard::handle_key_press(
                    event,
                    &self.config.keybindings,
                    &self.keychord_state,
                    &self.connection,
                )?;

                match result {
                    keyboard::handlers::KeychordResult::Completed(action, arg) => {
                        self.keychord_state = keyboard::handlers::KeychordState::Idle;
                        self.ungrab_chord_keys()?;
                        self.update_bar()?;

                        match action {
                            KeyAction::Quit => return Ok(Some(false)),
                            KeyAction::Restart => match self.try_reload_config() {
                                Ok(()) => {
                                    self.gaps_enabled = self.config.gaps_enabled;
                                    self.error_message = None;
                                    if let Err(error) = self.overlay.hide(&self.connection) {
                                        eprintln!("Failed to hide overlay after config reload: {:?}", error);
                                    }
                                    self.apply_layout()?;
                                    self.update_bar()?;
                                }
                                Err(err) => {
                                    eprintln!("Config reload error: {}", err);
                                    self.error_message = Some(err.clone());
                                    let screen_width = self.screen.width_in_pixels;
                                    let screen_height = self.screen.height_in_pixels;
                                    match self.overlay.show_error(
                                        &self.connection,
                                        &self.font,
                                        &err,
                                        screen_width,
                                        screen_height,
                                    ) {
                                        Ok(()) => eprintln!("Error modal displayed"),
                                        Err(e) => eprintln!("Failed to show error modal: {:?}", e),
                                    }
                                }
                            },
                            _ => self.handle_key_action(action, &arg)?,
                        }
                    }
                    keyboard::handlers::KeychordResult::InProgress(candidates) => {
                        let keys_pressed = match &self.keychord_state {
                            keyboard::handlers::KeychordState::Idle => 1,
                            keyboard::handlers::KeychordState::InProgress {
                                keys_pressed, ..
                            } => keys_pressed + 1,
                        };

                        self.keychord_state = keyboard::handlers::KeychordState::InProgress {
                            candidates: candidates.clone(),
                            keys_pressed,
                        };

                        self.grab_next_keys(&candidates, keys_pressed)?;
                        self.update_bar()?;
                    }
                    keyboard::handlers::KeychordResult::Cancelled
                    | keyboard::handlers::KeychordResult::None => {
                        self.keychord_state = keyboard::handlers::KeychordState::Idle;
                        self.ungrab_chord_keys()?;
                        self.update_bar()?;
                    }
                }
            }
            Event::ButtonPress(event) => {
                let is_bar_click = self
                    .bars
                    .iter()
                    .enumerate()
                    .find(|(_, bar)| bar.window() == event.event);

                if let Some((monitor_index, bar)) = is_bar_click {
                    if let Some(tag_index) = bar.handle_click(event.event_x) {
                        if monitor_index != self.selected_monitor {
                            self.selected_monitor = monitor_index;
                        }
                        self.view_tag(tag_index)?;
                    }
                } else {
                    let is_tab_bar_click = self
                        .tab_bars
                        .iter()
                        .enumerate()
                        .find(|(_, tab_bar)| tab_bar.window() == event.event);

                    if let Some((monitor_index, tab_bar)) = is_tab_bar_click {
                        if monitor_index != self.selected_monitor {
                            self.selected_monitor = monitor_index;
                        }

                        let visible_windows: Vec<Window> = self
                            .windows
                            .iter()
                            .filter(|&&window| {
                                if let Some(client) = self.clients.get(&window) {
                                    if client.monitor_index != monitor_index
                                        || self.floating_windows.contains(&window)
                                        || self.fullscreen_windows.contains(&window)
                                    {
                                        return false;
                                    }
                                    let monitor_tags = self.monitors.get(monitor_index).map(|m| m.tagset[m.selected_tags_index]).unwrap_or(0);
                                    (client.tags & monitor_tags) != 0
                                } else {
                                    false
                                }
                            })
                            .copied()
                            .collect();

                        if let Some(clicked_window) = tab_bar.get_clicked_window(&visible_windows, event.event_x) {
                            self.connection.configure_window(
                                clicked_window,
                                &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
                            )?;
                            self.focus(Some(clicked_window))?;
                            self.update_tab_bars()?;
                        }
                    } else if event.child != x11rb::NONE {
                        self.focus(Some(event.child))?;

                        if event.detail == ButtonIndex::M1.into() {
                            self.move_mouse(event.child)?;
                        } else if event.detail == ButtonIndex::M3.into() {
                            self.resize_mouse(event.child)?;
                        }
                    }
                }
            }
            Event::Expose(event) => {
                for bar in &mut self.bars {
                    if event.window == bar.window() {
                        bar.invalidate();
                        self.update_bar()?;
                        break;
                    }
                }
                for _tab_bar in &self.tab_bars {
                    if event.window == _tab_bar.window() {
                        self.update_tab_bars()?;
                        break;
                    }
                }
            }
            Event::ConfigureRequest(event) => {
                if self.windows.contains(&event.window) {
                    let monitor_index = self.clients.get(&event.window)
                        .map(|c| c.monitor_index)
                        .unwrap_or(self.selected_monitor);
                    let monitor = &self.monitors[monitor_index];
                    let is_floating = self.floating_windows.contains(&event.window);
                    let is_tiling_layout = self.layout.name() != "normie";

                    if is_floating || !is_tiling_layout {
                        let cached_geom = self.window_geometry_cache.get(&event.window);
                        let border_width = self.config.border_width as u16;

                        let mut config = ConfigureWindowAux::new();
                        let value_mask = event.value_mask;

                        if value_mask.contains(ConfigWindow::BORDER_WIDTH) {
                            config = config.border_width(event.border_width as u32);
                        }

                        if value_mask.contains(ConfigWindow::X) {
                            let mut x = event.x as i32;
                            x = x.max(monitor.screen_x);
                            if x + event.width as i32 + 2 * border_width as i32 > monitor.screen_x + monitor.screen_width as i32 {
                                x = monitor.screen_x + monitor.screen_width as i32 - event.width as i32 - 2 * border_width as i32;
                            }
                            config = config.x(x);
                        }

                        if value_mask.contains(ConfigWindow::Y) {
                            let mut y = event.y as i32;
                            y = y.max(monitor.screen_y);
                            if y + event.height as i32 + 2 * border_width as i32 > monitor.screen_y + monitor.screen_height as i32 {
                                y = monitor.screen_y + monitor.screen_height as i32 - event.height as i32 - 2 * border_width as i32;
                            }
                            config = config.y(y);
                        }

                        if value_mask.contains(ConfigWindow::WIDTH) {
                            config = config.width(event.width as u32);
                        }

                        if value_mask.contains(ConfigWindow::HEIGHT) {
                            config = config.height(event.height as u32);
                        }

                        if value_mask.contains(ConfigWindow::SIBLING) {
                            config = config.sibling(event.sibling);
                        }

                        if value_mask.contains(ConfigWindow::STACK_MODE) {
                            config = config.stack_mode(event.stack_mode);
                        }

                        self.connection.configure_window(event.window, &config)?;

                        let final_x = if value_mask.contains(ConfigWindow::X) {
                            let mut x = event.x as i32;
                            x = x.max(monitor.screen_x);
                            if x + event.width as i32 + 2 * border_width as i32 > monitor.screen_x + monitor.screen_width as i32 {
                                x = monitor.screen_x + monitor.screen_width as i32 - event.width as i32 - 2 * border_width as i32;
                            }
                            x as i16
                        } else {
                            cached_geom.map(|g| g.x_position).unwrap_or(0)
                        };

                        let final_y = if value_mask.contains(ConfigWindow::Y) {
                            let mut y = event.y as i32;
                            y = y.max(monitor.screen_y);
                            if y + event.height as i32 + 2 * border_width as i32 > monitor.screen_y + monitor.screen_height as i32 {
                                y = monitor.screen_y + monitor.screen_height as i32 - event.height as i32 - 2 * border_width as i32;
                            }
                            y as i16
                        } else {
                            cached_geom.map(|g| g.y_position).unwrap_or(0)
                        };

                        let final_width = if value_mask.contains(ConfigWindow::WIDTH) { event.width } else { cached_geom.map(|g| g.width).unwrap_or(1) };
                        let final_height = if value_mask.contains(ConfigWindow::HEIGHT) { event.height } else { cached_geom.map(|g| g.height).unwrap_or(1) };

                        self.update_geometry_cache(event.window, CachedGeometry {
                            x_position: final_x,
                            y_position: final_y,
                            width: final_width,
                            height: final_height,
                            border_width: if value_mask.contains(ConfigWindow::BORDER_WIDTH) { event.border_width } else { border_width },
                        });

                        if is_floating {
                            let new_monitor = self.rect_to_monitor(final_x as i32, final_y as i32, final_width as i32, final_height as i32);

                            if new_monitor != monitor_index {
                                self.send_to_monitor(event.window, new_monitor)?;
                            }
                        }
                    } else {
                        self.send_configure_notify(event.window)?;
                    }
                } else {
                    let mut config = ConfigureWindowAux::new()
                        .x(event.x as i32)
                        .y(event.y as i32)
                        .width(event.width as u32)
                        .height(event.height as u32)
                        .border_width(event.border_width as u32);

                    if event.value_mask.contains(ConfigWindow::SIBLING) {
                        config = config.sibling(event.sibling);
                    }

                    if event.value_mask.contains(ConfigWindow::STACK_MODE) {
                        config = config.stack_mode(event.stack_mode);
                    }

                    self.connection.configure_window(event.window, &config)?;
                }
            }
            Event::ClientMessage(event) => {
                if event.type_ == self.atoms.net_wm_state {
                    if let Some(data) = event.data.as_data32().get(1) {
                        if *data == self.atoms.net_wm_state_fullscreen {
                            let action = event.data.as_data32()[0];
                            let fullscreen = match action {
                                1 => true,
                                0 => false,
                                2 => !self.fullscreen_windows.contains(&event.window),
                                _ => return Ok(None),
                            };
                            self.set_window_fullscreen(event.window, fullscreen)?;
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn apply_layout(&mut self) -> WmResult<()> {
        if self.layout.name() == LayoutType::Normie.as_str() {
            return Ok(());
        }

        let monitor_count = self.monitors.len();
        for monitor_index in 0..monitor_count {
            let monitor = &self.monitors[monitor_index];
            let border_width = self.config.border_width;

            let gaps = if self.gaps_enabled {
                GapConfig {
                    inner_horizontal: self.config.gap_inner_horizontal,
                    inner_vertical: self.config.gap_inner_vertical,
                    outer_horizontal: self.config.gap_outer_horizontal,
                    outer_vertical: self.config.gap_outer_vertical,
                }
            } else {
                GapConfig {
                    inner_horizontal: 0,
                    inner_vertical: 0,
                    outer_horizontal: 0,
                    outer_vertical: 0,
                }
            };

            let monitor_x = monitor.screen_x;
            let monitor_y = monitor.screen_y;
            let monitor_width = monitor.screen_width;
            let monitor_height = monitor.screen_height;

            let mut visible: Vec<Window> = Vec::new();
            let mut current = self.next_tiled(monitor.clients_head, monitor);
            while let Some(window) = current {
                visible.push(window);
                if let Some(client) = self.clients.get(&window) {
                    current = self.next_tiled(client.next, monitor);
                } else {
                    break;
                }
            }

            let bar_height = if self.show_bar {
                self.bars
                    .get(monitor_index)
                    .map(|bar| bar.height() as u32)
                    .unwrap_or(0)
            } else {
                0
            };
            let usable_height = monitor_height.saturating_sub(bar_height as i32);
            let master_factor = monitor.master_factor;
            let num_master = monitor.num_master;
            let smartgaps_enabled = self.config.smartgaps_enabled;

            let geometries = self.layout.arrange(
                &visible,
                monitor_width as u32,
                usable_height as u32,
                &gaps,
                master_factor,
                num_master,
                smartgaps_enabled,
            );

            for (window, geometry) in visible.iter().zip(geometries.iter()) {
                let mut adjusted_width = geometry.width.saturating_sub(2 * border_width);
                let mut adjusted_height = geometry.height.saturating_sub(2 * border_width);

                if let Some(client) = self.clients.get(window) {
                    if !client.is_floating {
                        let (hint_width, hint_height) = self.apply_size_hints(
                            client,
                            adjusted_width as i32,
                            adjusted_height as i32,
                        );
                        adjusted_width = hint_width as u32;
                        adjusted_height = hint_height as u32;
                    }
                }

                let adjusted_x = geometry.x_coordinate + monitor_x;
                let adjusted_y = geometry.y_coordinate + monitor_y + bar_height as i32;

                if let Some(client) = self.clients.get_mut(window) {
                    client.x_position = adjusted_x as i16;
                    client.y_position = adjusted_y as i16;
                    client.width = adjusted_width as u16;
                    client.height = adjusted_height as u16;
                }

                self.connection.configure_window(
                    *window,
                    &ConfigureWindowAux::new()
                        .x(adjusted_x)
                        .y(adjusted_y)
                        .width(adjusted_width)
                        .height(adjusted_height)
                        .border_width(border_width),
                )?;

                self.update_geometry_cache(*window, CachedGeometry {
                    x_position: adjusted_x as i16,
                    y_position: adjusted_y as i16,
                    width: adjusted_width as u16,
                    height: adjusted_height as u16,
                    border_width: border_width as u16,
                });
            }
        }

        for monitor_index in 0..self.monitors.len() {
            let stack_head = self.monitors[monitor_index].stack_head;
            self.showhide(stack_head)?;
        }

        self.connection.flush()?;

        let is_tabbed = self.layout.name() == LayoutType::Tabbed.as_str();

        if is_tabbed {
            let outer_horizontal = if self.gaps_enabled {
                self.config.gap_outer_horizontal
            } else {
                0
            };
            let outer_vertical = if self.gaps_enabled {
                self.config.gap_outer_vertical
            } else {
                0
            };

            for monitor_index in 0..self.tab_bars.len() {
                if let Some(monitor) = self.monitors.get(monitor_index) {
                    let bar_height = if self.show_bar {
                        self.bars
                            .get(monitor_index)
                            .map(|bar| bar.height() as f32)
                            .unwrap_or(0.0)
                    } else {
                        0.0
                    };

                    let tab_bar_x = (monitor.screen_x + outer_horizontal as i32) as i16;
                    let tab_bar_y = (monitor.screen_y as f32 + bar_height + outer_vertical as f32) as i16;
                    let tab_bar_width = monitor.screen_width.saturating_sub(2 * outer_horizontal as i32) as u16;

                    if let Err(e) = self.tab_bars[monitor_index].reposition(
                        &self.connection,
                        tab_bar_x,
                        tab_bar_y,
                        tab_bar_width,
                    ) {
                        eprintln!("Failed to reposition tab bar: {:?}", e);
                    }
                }
            }
        }

        for monitor_index in 0..self.tab_bars.len() {
            let has_visible_windows = self
                .windows
                .iter()
                .any(|&window| {
                    if let Some(client) = self.clients.get(&window) {
                        if client.monitor_index != monitor_index
                            || self.floating_windows.contains(&window)
                            || self.fullscreen_windows.contains(&window)
                        {
                            return false;
                        }
                        if let Some(monitor) = self.monitors.get(monitor_index) {
                            return (client.tags & monitor.tagset[monitor.selected_tags_index]) != 0;
                        }
                    }
                    false
                });

            if is_tabbed && has_visible_windows {
                if let Err(e) = self.tab_bars[monitor_index].show(&self.connection) {
                    eprintln!("Failed to show tab bar: {:?}", e);
                }
            } else {
                if let Err(e) = self.tab_bars[monitor_index].hide(&self.connection) {
                    eprintln!("Failed to hide tab bar: {:?}", e);
                }
            }
        }

        if is_tabbed {
            self.update_tab_bars()?;
        }

        Ok(())
    }

    pub fn change_layout<L: Layout + 'static>(&mut self, new_layout: L) -> WmResult<()> {
        self.layout = Box::new(new_layout);
        self.apply_layout()?;
        Ok(())
    }

    fn update_geometry_cache(&mut self, window: Window, geometry: CachedGeometry) {
        self.window_geometry_cache.insert(window, geometry);
    }

    fn get_cached_geometry(&self, window: Window) -> Option<CachedGeometry> {
        self.window_geometry_cache.get(&window).copied()
    }

    fn get_or_query_geometry(&mut self, window: Window) -> WmResult<CachedGeometry> {
        if let Some(cached) = self.get_cached_geometry(window) {
            return Ok(cached);
        }

        let geometry = self.connection.get_geometry(window)?.reply()?;
        let cached = CachedGeometry {
            x_position: geometry.x,
            y_position: geometry.y,
            width: geometry.width,
            height: geometry.height,
            border_width: geometry.border_width as u16,
        };
        self.update_geometry_cache(window, cached);
        Ok(cached)
    }

    fn send_configure_notify(&mut self, window: Window) -> WmResult<()> {
        let geometry = self.get_or_query_geometry(window)?;

        let event = x11rb::protocol::xproto::ConfigureNotifyEvent {
            response_type: x11rb::protocol::xproto::CONFIGURE_NOTIFY_EVENT,
            sequence: 0,
            event: window,
            window,
            above_sibling: x11rb::NONE,
            x: geometry.x_position,
            y: geometry.y_position,
            width: geometry.width,
            height: geometry.height,
            border_width: geometry.border_width,
            override_redirect: false,
        };

        self.connection.send_event(
            false,
            window,
            x11rb::protocol::xproto::EventMask::STRUCTURE_NOTIFY,
            event,
        )?;

        Ok(())
    }

    fn update_size_hints(&mut self, window: Window) -> WmResult<()> {
        let size_hints = self.connection
            .get_property(
                false,
                window,
                x11rb::protocol::xproto::AtomEnum::WM_NORMAL_HINTS,
                x11rb::protocol::xproto::AtomEnum::WM_SIZE_HINTS,
                0,
                18,
            )?
            .reply()?;

        if size_hints.value.is_empty() {
            if let Some(client) = self.clients.get_mut(&window) {
                client.hints_valid = false;
            }
            return Ok(());
        }

        if size_hints.value.len() < 18 * 4 {
            if let Some(client) = self.clients.get_mut(&window) {
                client.hints_valid = false;
            }
            return Ok(());
        }

        let read_u32 = |offset: usize| -> u32 {
            let bytes = &size_hints.value[offset * 4..(offset + 1) * 4];
            u32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
        };

        let flags = read_u32(0);

        const P_SIZE: u32 = 1 << 3;
        const P_MIN_SIZE: u32 = 1 << 4;
        const P_MAX_SIZE: u32 = 1 << 5;
        const P_RESIZE_INC: u32 = 1 << 6;
        const P_ASPECT: u32 = 1 << 7;
        const P_BASE_SIZE: u32 = 1 << 8;

        if let Some(client) = self.clients.get_mut(&window) {
            if flags & P_BASE_SIZE != 0 {
                client.base_width = read_u32(8) as i32;
                client.base_height = read_u32(9) as i32;
            } else if flags & P_MIN_SIZE != 0 {
                client.base_width = read_u32(5) as i32;
                client.base_height = read_u32(6) as i32;
            } else {
                client.base_width = 0;
                client.base_height = 0;
            }

            if flags & P_RESIZE_INC != 0 {
                client.increment_width = read_u32(10) as i32;
                client.increment_height = read_u32(11) as i32;
            } else {
                client.increment_width = 0;
                client.increment_height = 0;
            }

            if flags & P_MAX_SIZE != 0 {
                client.max_width = read_u32(7) as i32;
                client.max_height = read_u32(8) as i32;
            } else {
                client.max_width = 0;
                client.max_height = 0;
            }

            if flags & P_MIN_SIZE != 0 {
                client.min_width = read_u32(5) as i32;
                client.min_height = read_u32(6) as i32;
            } else if flags & P_SIZE != 0 {
                client.min_width = read_u32(3) as i32;
                client.min_height = read_u32(4) as i32;
            } else {
                client.min_width = 0;
                client.min_height = 0;
            }

            if flags & P_ASPECT != 0 {
                client.min_aspect = (read_u32(12) as f32) / (read_u32(13) as f32).max(1.0);
                client.max_aspect = (read_u32(14) as f32) / (read_u32(15) as f32).max(1.0);
            } else {
                client.min_aspect = 0.0;
                client.max_aspect = 0.0;
            }

            client.is_fixed = client.max_width > 0
                && client.max_height > 0
                && client.max_width == client.min_width
                && client.max_height == client.min_height;

            client.hints_valid = true;
        }
        Ok(())
    }

    fn update_window_title(&mut self, window: Window) -> WmResult<()> {
        let name = self.connection
            .get_property(
                false,
                window,
                x11rb::protocol::xproto::AtomEnum::WM_NAME,
                x11rb::protocol::xproto::AtomEnum::STRING,
                0,
                256,
            )?
            .reply()?;

        if !name.value.is_empty() {
            if let Ok(title) = String::from_utf8(name.value.clone()) {
                if let Some(client) = self.clients.get_mut(&window) {
                    client.name = title;
                }
            }
        }

        Ok(())
    }

    fn apply_size_hints(&self, client: &Client, mut width: i32, mut height: i32) -> (i32, i32) {
        if !client.hints_valid {
            return (width.max(1), height.max(1));
        }

        if client.min_width > 0 {
            width = width.max(client.min_width);
        }
        if client.min_height > 0 {
            height = height.max(client.min_height);
        }

        if client.max_width > 0 {
            width = width.min(client.max_width);
        }
        if client.max_height > 0 {
            height = height.min(client.max_height);
        }

        if client.increment_width > 0 {
            width -= client.base_width;
            width -= width % client.increment_width;
            width += client.base_width;
        }
        if client.increment_height > 0 {
            height -= client.base_height;
            height -= height % client.increment_height;
            height += client.base_height;
        }

        if client.min_aspect > 0.0 || client.max_aspect > 0.0 {
            let actual_aspect = width as f32 / height as f32;

            if client.max_aspect > 0.0 && actual_aspect > client.max_aspect {
                width = (height as f32 * client.max_aspect) as i32;
            } else if client.min_aspect > 0.0 && actual_aspect < client.min_aspect {
                height = (width as f32 / client.min_aspect) as i32;
            }
        }

        (width.max(1), height.max(1))
    }

    fn next_tiled(&self, start: Option<Window>, monitor: &Monitor) -> Option<Window> {
        let mut current = start;
        while let Some(window) = current {
            if let Some(client) = self.clients.get(&window) {
                let visible_tags = client.tags & monitor.tagset[monitor.selected_tags_index];
                if visible_tags != 0 && !client.is_floating {
                    return Some(window);
                }
                current = client.next;
            } else {
                break;
            }
        }
        None
    }

    fn count_tiled(&self, monitor: &Monitor) -> usize {
        let mut count = 0;
        let mut current = monitor.clients_head;
        while let Some(window) = current {
            if let Some(client) = self.clients.get(&window) {
                let visible_tags = client.tags & monitor.tagset[monitor.selected_tags_index];
                if visible_tags != 0 && !client.is_floating {
                    count += 1;
                }
                current = client.next;
            } else {
                break;
            }
        }
        count
    }

    fn attach(&mut self, window: Window, monitor_index: usize) {
        if let Some(monitor) = self.monitors.get_mut(monitor_index) {
            if let Some(client) = self.clients.get_mut(&window) {
                client.next = monitor.clients_head;
                monitor.clients_head = Some(window);
            }
        }
    }

    fn attach_aside(&mut self, window: Window, monitor_index: usize) {
        let monitor = match self.monitors.get(monitor_index) {
            Some(m) => m,
            None => return,
        };

        if monitor.clients_head.is_none() {
            self.attach(window, monitor_index);
            return;
        }

        let num_master = monitor.num_master.max(1) as usize;
        let mut current = monitor.clients_head;
        let mut position = 0;

        while position < num_master - 1 {
            if let Some(current_window) = current {
                if let Some(current_client) = self.clients.get(&current_window) {
                    current = current_client.next;
                    position += 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if let Some(insert_after) = current {
            if let Some(after_client) = self.clients.get(&insert_after) {
                let old_next = after_client.next;
                if let Some(new_client) = self.clients.get_mut(&window) {
                    new_client.next = old_next;
                }
                if let Some(after_client_mut) = self.clients.get_mut(&insert_after) {
                    after_client_mut.next = Some(window);
                }
            }
        } else {
            self.attach(window, monitor_index);
        }
    }

    fn detach(&mut self, window: Window) {
        let monitor_index = self.clients.get(&window).map(|c| c.monitor_index);
        if let Some(monitor_index) = monitor_index {
            if let Some(monitor) = self.monitors.get_mut(monitor_index) {
                if monitor.clients_head == Some(window) {
                    if let Some(client) = self.clients.get(&window) {
                        monitor.clients_head = client.next;
                    }
                } else {
                    let mut current = monitor.clients_head;
                    while let Some(current_window) = current {
                        if let Some(current_client) = self.clients.get(&current_window) {
                            if current_client.next == Some(window) {
                                let new_next = self.clients.get(&window).and_then(|c| c.next);
                                if let Some(current_client_mut) = self.clients.get_mut(&current_window) {
                                    current_client_mut.next = new_next;
                                }
                                break;
                            }
                            current = current_client.next;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    }

    fn attach_stack(&mut self, window: Window, monitor_index: usize) {
        if let Some(monitor) = self.monitors.get_mut(monitor_index) {
            if let Some(client) = self.clients.get_mut(&window) {
                client.stack_next = monitor.stack_head;
                monitor.stack_head = Some(window);
            }
        }
    }

    fn detach_stack(&mut self, window: Window) {
        let monitor_index = self.clients.get(&window).map(|c| c.monitor_index);
        if let Some(monitor_index) = monitor_index {
            if let Some(monitor) = self.monitors.get_mut(monitor_index) {
                if monitor.stack_head == Some(window) {
                    if let Some(client) = self.clients.get(&window) {
                        monitor.stack_head = client.stack_next;
                    }
                    let should_update_selected = monitor.selected_client == Some(window);
                    let mut new_selected: Option<Window> = None;
                    if should_update_selected {
                        let mut stack_current = monitor.stack_head;
                        while let Some(stack_window) = stack_current {
                            if let Some(stack_client) = self.clients.get(&stack_window) {
                                if self.is_window_visible(stack_window) {
                                    new_selected = Some(stack_window);
                                    break;
                                }
                                stack_current = stack_client.stack_next;
                            } else {
                                break;
                            }
                        }
                    }
                    if should_update_selected {
                        if let Some(monitor) = self.monitors.get_mut(monitor_index) {
                            monitor.selected_client = new_selected;
                        }
                    }
                } else {
                    let mut current = monitor.stack_head;
                    while let Some(current_window) = current {
                        if let Some(current_client) = self.clients.get(&current_window) {
                            if current_client.stack_next == Some(window) {
                                let new_stack_next = self.clients.get(&window).and_then(|c| c.stack_next);
                                if let Some(current_client_mut) = self.clients.get_mut(&current_window) {
                                    current_client_mut.stack_next = new_stack_next;
                                }
                                break;
                            }
                            current = current_client.stack_next;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    }

    fn send_to_monitor(&mut self, window: Window, target_monitor: usize) -> WmResult<()> {
        if target_monitor >= self.monitors.len() {
            return Ok(());
        }

        let current_monitor = self.clients.get(&window).map(|c| c.monitor_index);
        if current_monitor == Some(target_monitor) {
            return Ok(());
        }

        self.unfocus(window)?;

        self.detach(window);
        self.detach_stack(window);

        if let Some(client) = self.clients.get_mut(&window) {
            client.monitor_index = target_monitor;
            let new_tags = self.monitors
                .get(target_monitor)
                .map(|m| m.tagset[m.selected_tags_index])
                .unwrap_or(1);
            client.tags = new_tags;
        }

        self.attach_aside(window, target_monitor);
        self.attach_stack(window, target_monitor);

        self.focus(None)?;
        self.apply_layout()?;

        Ok(())
    }

    fn remove_window(&mut self, window: Window) -> WmResult<()> {
        let initial_count = self.windows.len();

        let focused = self
            .monitors
            .get(self.selected_monitor)
            .and_then(|m| m.selected_client);

        if self.clients.contains_key(&window) {
            self.detach(window);
            self.detach_stack(window);
            self.clients.remove(&window);
        }

        self.windows.retain(|&w| w != window);
        self.window_geometry_cache.remove(&window);
        self.floating_windows.remove(&window);

        if self.windows.len() < initial_count {
            if focused == Some(window) {
                let visible = self.visible_windows_on_monitor(self.selected_monitor);
                if let Some(&new_win) = visible.last() {
                    self.focus(Some(new_win))?;
                } else if let Some(monitor) = self.monitors.get_mut(self.selected_monitor) {
                    monitor.selected_client = None;
                }
            }

            self.apply_layout()?;
            self.update_bar()?;
        }
        Ok(())
    }

    fn run_autostart_commands(&self) -> Result<(), WmError> {
        for command in &self.config.autostart {
            Command::new("sh")
                .arg("-c")
                .arg(command)
                .spawn()
                .map_err(|e| WmError::Autostart(command.clone(), e))?;
            eprintln!("[autostart] Spawned: {}", command);
        }
        Ok(())
    }
}
