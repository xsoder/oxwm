use crate::Config;
use crate::bar::Bar;
use crate::errors::WmError;
use crate::keyboard::{self, Arg, KeyAction, handlers};
use crate::layout::GapConfig;
use crate::layout::Layout;
use crate::layout::tiling::TilingLayout;
use crate::monitor::{Monitor, detect_monitors};
use std::collections::HashSet;
use x11rb::cursor::Handle as CursorHandle;

use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

pub type TagMask = u32;
pub fn tag_mask(tag: usize) -> TagMask {
    1 << tag
}

struct AtomCache {
    net_current_desktop: Atom,
    net_client_info: Atom,
    wm_state: Atom,
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

        let wm_state = connection
            .intern_atom(false, b"WM_STATE")?
            .reply()?
            .atom;

        Ok(Self {
            net_current_desktop,
            net_client_info,
            wm_state,
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
    layout: Box<dyn Layout>,
    window_tags: std::collections::HashMap<Window, TagMask>,
    window_monitor: std::collections::HashMap<Window, usize>,
    gaps_enabled: bool,
    fullscreen_window: Option<Window>,
    floating_windows: HashSet<Window>,
    bars: Vec<Bar>,
    monitors: Vec<Monitor>,
    selected_monitor: usize,
    atoms: AtomCache,
    previous_focused: Option<Window>,
    display: *mut x11::xlib::Display,
    font: crate::bar::font::Font,
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

        connection.grab_button(
            false,
            root,
            EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE,
            GrabMode::SYNC,
            GrabMode::ASYNC,
            x11rb::NONE,
            x11rb::NONE,
            ButtonIndex::M1,
            u16::from(config.modkey).into(),
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
            u16::from(config.modkey).into(),
        )?;

        let mut monitors = detect_monitors(&connection, &screen, root)?;

        let selected_tags = Self::get_saved_selected_tags(&connection, root, config.tags.len())?;
        if !monitors.is_empty() {
            monitors[0].selected_tags = selected_tags;
        }

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
                monitor.x as i16,
                monitor.y as i16,
                monitor.width as u16,
            )?;
            bars.push(bar);
        }

        let gaps_enabled = config.gaps_enabled;

        let atoms = AtomCache::new(&connection)?;

        let mut window_manager = Self {
            config,
            connection,
            screen_number,
            root,
            screen,
            windows: Vec::new(),
            layout: Box::new(TilingLayout),
            window_tags: std::collections::HashMap::new(),
            window_monitor: std::collections::HashMap::new(),
            gaps_enabled,
            fullscreen_window: None,
            floating_windows: HashSet::new(),
            bars,
            monitors,
            selected_monitor: 0,
            atoms,
            previous_focused: None,
            display,
            font,
        };

        window_manager.scan_existing_windows()?;
        window_manager.update_bar()?;

        Ok(window_manager)
    }

    fn get_saved_selected_tags(
        connection: &RustConnection,
        root: Window,
        tag_count: usize,
    ) -> WmResult<TagMask> {
        let net_current_desktop = connection
            .intern_atom(false, b"_NET_CURRENT_DESKTOP")?
            .reply()?
            .atom;

        match connection
            .get_property(false, root, net_current_desktop, AtomEnum::CARDINAL, 0, 1)?
            .reply()
        {
            Ok(prop) if prop.value.len() >= 4 => {
                let desktop = u32::from_ne_bytes([
                    prop.value[0],
                    prop.value[1],
                    prop.value[2],
                    prop.value[3],
                ]);
                if desktop < tag_count as u32 {
                    let mask = tag_mask(desktop as usize);
                    return Ok(mask);
                }
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("No _NET_CURRENT_DESKTOP property ({})", e);
            }
        }
        Ok(tag_mask(0))
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
                let tag = self.get_saved_tag(window, net_client_info)?;
                self.windows.push(window);
                self.window_tags.insert(window, tag);
                self.window_monitor.insert(window, self.selected_monitor);
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
                    let tag = self.get_saved_tag(window, net_client_info)?;
                    self.windows.push(window);
                    self.window_tags.insert(window, tag);
                    self.window_monitor.insert(window, self.selected_monitor);
                }
            }
        }

        if let Some(&first) = self.windows.first() {
            self.set_focus(first)?;
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

        Ok(self.monitors.get(self.selected_monitor).map(|m| m.selected_tags).unwrap_or(tag_mask(0)))
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

        loop {
            while let Some(event) = self.connection.poll_for_event()? {
                if let Some(should_restart) = self.handle_event(event)? {
                    return Ok(should_restart);
                }
            }

            if let Some(bar) = self.bars.get_mut(self.selected_monitor) {
                bar.update_blocks();
            }

            if self.bars.iter().any(|bar| bar.needs_redraw()) {
                self.update_bar()?;
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    fn toggle_floating(&mut self) -> WmResult<()> {
        if let Some(focused) = self.monitors.get(self.selected_monitor).and_then(|m| m.focused_window) {
            if self.floating_windows.contains(&focused) {
                self.floating_windows.remove(&focused);
                self.apply_layout()?;
            } else {
                self.floating_windows.insert(focused);
                self.connection.configure_window(
                    focused,
                    &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
                )?;
                self.apply_layout()?;
                self.connection.flush()?;
            }
        }
        Ok(())
    }

    fn toggle_fullscreen(&mut self) -> WmResult<()> {
        if let Some(focused) = self.monitors.get(self.selected_monitor).and_then(|m| m.focused_window) {
            if self.fullscreen_window == Some(focused) {
                self.fullscreen_window = None;

                for bar in &self.bars {
                    self.connection.map_window(bar.window())?;
                }

                self.apply_layout()?;
            } else {
                self.fullscreen_window = Some(focused);

                if let Some(bar) = self.bars.get(self.selected_monitor) {
                    self.connection.unmap_window(bar.window())?;
                }

                let monitor = &self.monitors[self.selected_monitor];
                let screen_width = monitor.width;
                let screen_height = monitor.height;

                self.connection.configure_window(
                    focused,
                    &ConfigureWindowAux::new()
                        .x(monitor.x)
                        .y(monitor.y)
                        .width(screen_width)
                        .height(screen_height)
                        .border_width(0)
                        .stack_mode(StackMode::ABOVE),
                )?;

                self.connection.flush()?;
            }
        }
        Ok(())
    }

    fn update_bar(&mut self) -> WmResult<()> {
        for (monitor_index, monitor) in self.monitors.iter().enumerate() {
            if let Some(bar) = self.bars.get_mut(monitor_index) {
                let mut occupied_tags: TagMask = 0;
                for (&window, &tags) in &self.window_tags {
                    if self.window_monitor.get(&window).copied().unwrap_or(0) == monitor_index {
                        occupied_tags |= tags;
                    }
                }

                let draw_blocks = monitor_index == self.selected_monitor;
                bar.invalidate();
                bar.draw(&self.connection, &self.font, self.display, monitor.selected_tags, occupied_tags, draw_blocks)?;
            }
        }
        Ok(())
    }

    fn handle_key_action(&mut self, action: KeyAction, arg: &Arg) -> WmResult<()> {
        match action {
            KeyAction::Spawn => handlers::handle_spawn_action(action, arg)?,
            KeyAction::KillClient => {
                if let Some(focused) = self.monitors.get(self.selected_monitor).and_then(|m| m.focused_window) {
                    match self.connection.kill_client(focused) {
                        Ok(_) => {
                            self.connection.flush()?;
                        }
                        Err(e) => {
                            eprintln!("Failed to kill window {}: {}", focused, e);
                        }
                    }
                }
            }
            KeyAction::ToggleFullScreen => {
                self.toggle_fullscreen()?;
            }
            KeyAction::ToggleFloating => {
                self.toggle_floating()?;
            }

            KeyAction::FocusStack => {
                if let Arg::Int(direction) = arg {
                    self.cycle_focus(*direction)?;
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
            KeyAction::MoveToTag => {
                if let Arg::Int(tag_index) = arg {
                    self.move_to_tag(*tag_index as usize)?;
                }
            }
            KeyAction::ToggleGaps => {
                self.gaps_enabled = !self.gaps_enabled;
                self.apply_layout()?;
            }
            KeyAction::FocusMonitor => {
                if let Arg::Int(direction) = arg {
                    self.focus_monitor(*direction)?;
                }
            }
            KeyAction::None => {}
        }
        Ok(())
    }

    fn focus_monitor(&mut self, direction: i32) -> WmResult<()> {
        if self.monitors.is_empty() {
            return Ok(());
        }

        let new_monitor = if direction > 0 {
            (self.selected_monitor + 1) % self.monitors.len()
        } else {
            (self.selected_monitor + self.monitors.len() - 1) % self.monitors.len()
        };

        if new_monitor == self.selected_monitor {
            return Ok(());
        }

        self.selected_monitor = new_monitor;

        self.update_bar()?;

        let visible = self.visible_windows_on_monitor(new_monitor);
        if let Some(&win) = visible.first() {
            self.set_focus(win)?;
        }

        Ok(())
    }

    fn is_window_visible(&self, window: Window) -> bool {
        let window_mon = self.window_monitor.get(&window).copied().unwrap_or(0);

        if let Some(&tags) = self.window_tags.get(&window) {
            let monitor = self.monitors.get(window_mon);
            let selected_tags = monitor.map(|m| m.selected_tags).unwrap_or(0);
            (tags & selected_tags) != 0
        } else {
            false
        }
    }

    fn visible_windows(&self) -> Vec<Window> {
        self.windows
            .iter()
            .filter(|&&w| self.is_window_visible(w))
            .copied()
            .collect()
    }

    fn visible_windows_on_monitor(&self, monitor_index: usize) -> Vec<Window> {
        self.windows
            .iter()
            .filter(|&&w| {
                let window_mon = self.window_monitor.get(&w).copied().unwrap_or(0);
                if window_mon != monitor_index {
                    return false;
                }
                if let Some(&tags) = self.window_tags.get(&w) {
                    let monitor = self.monitors.get(monitor_index);
                    let selected_tags = monitor.map(|m| m.selected_tags).unwrap_or(0);
                    (tags & selected_tags) != 0
                } else {
                    false
                }
            })
            .copied()
            .collect()
    }

    fn get_monitor_at_point(&self, x: i32, y: i32) -> Option<usize> {
        self.monitors
            .iter()
            .position(|mon| mon.contains_point(x, y))
    }

    fn update_window_visibility(&self) -> WmResult<()> {
        for &window in &self.windows {
            if self.is_window_visible(window) {
                self.connection.map_window(window)?;
            } else {
                self.connection.unmap_window(window)?;
            }
        }
        self.connection.flush()?;
        Ok(())
    }

    pub fn view_tag(&mut self, tag_index: usize) -> WmResult<()> {
        if tag_index >= self.config.tags.len() {
            return Ok(());
        }

        if self.fullscreen_window.is_some() {
            self.fullscreen_window = None;
            for bar in &self.bars {
                self.connection.map_window(bar.window())?;
            }
        }

        if let Some(monitor) = self.monitors.get_mut(self.selected_monitor) {
            monitor.selected_tags = tag_mask(tag_index);
        }

        self.save_selected_tags()?;

        self.update_window_visibility()?;
        self.apply_layout()?;
        self.update_bar()?;

        let visible = self.visible_windows_on_monitor(self.selected_monitor);
        if let Some(&win) = visible.first() {
            self.set_focus(win)?;
        }

        Ok(())
    }

    fn save_selected_tags(&self) -> WmResult<()> {
        let net_current_desktop = self.atoms.net_current_desktop;

        let selected_tags = self.monitors.get(self.selected_monitor).map(|m| m.selected_tags).unwrap_or(tag_mask(0));
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

        if let Some(focused) = self.monitors.get(self.selected_monitor).and_then(|m| m.focused_window) {
            let mask = tag_mask(tag_index);
            self.window_tags.insert(focused, mask);

            let _ = self.save_client_tag(focused, mask);

            self.update_window_visibility()?;
            self.apply_layout()?;
            self.update_bar()?;
        }

        Ok(())
    }

    pub fn cycle_focus(&mut self, direction: i32) -> WmResult<()> {
        let visible = self.visible_windows();

        if visible.is_empty() {
            return Ok(());
        }

        let current = self.monitors.get(self.selected_monitor).and_then(|m| m.focused_window);

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

        self.set_focus(next_window)?;
        Ok(())
    }

    pub fn set_focus(&mut self, window: Window) -> WmResult<()> {
        let old_focused = self.previous_focused;

        if let Some(monitor) = self.monitors.get_mut(self.selected_monitor) {
            monitor.focused_window = Some(window);
        }

        self.connection
            .set_input_focus(InputFocus::POINTER_ROOT, window, x11rb::CURRENT_TIME)?;
        self.connection.flush()?;

        self.update_focus_visuals(old_focused, window)?;
        self.previous_focused = Some(window);
        Ok(())
    }

    fn update_focus_visuals(&self, old_focused: Option<Window>, new_focused: Window) -> WmResult<()> {
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
            Event::MapRequest(event) => {
                let attrs = match self.connection.get_window_attributes(event.window)?.reply() {
                    Ok(attrs) => attrs,
                    Err(_) => return Ok(None),
                };

                if attrs.override_redirect {
                    return Ok(None);
                }

                if self.windows.contains(&event.window) {
                    return Ok(None);
                }

                self.connection.map_window(event.window)?;
                self.connection.change_window_attributes(
                    event.window,
                    &ChangeWindowAttributesAux::new().event_mask(EventMask::ENTER_WINDOW),
                )?;

                let selected_tags = self.monitors.get(self.selected_monitor).map(|m| m.selected_tags).unwrap_or(tag_mask(0));

                self.windows.push(event.window);
                self.window_tags.insert(event.window, selected_tags);
                self.window_monitor.insert(event.window, self.selected_monitor);
                self.set_wm_state(event.window, 1)?;
                let _ = self.save_client_tag(event.window, selected_tags);

                self.apply_layout()?;
                self.update_bar()?;
                self.set_focus(event.window)?;
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
            Event::EnterNotify(event) => {
                if event.mode != x11rb::protocol::xproto::NotifyMode::NORMAL {
                    return Ok(None);
                }
                if self.windows.contains(&event.event) {
                    self.set_focus(event.event)?;
                }
            }
            Event::MotionNotify(event) => {
                if event.event != self.root {
                    return Ok(None);
                }

                if let Some(monitor_index) = self.get_monitor_at_point(event.root_x as i32, event.root_y as i32) {
                    if monitor_index != self.selected_monitor {
                        self.selected_monitor = monitor_index;
                        self.update_bar()?;

                        let visible = self.visible_windows_on_monitor(monitor_index);
                        if let Some(&win) = visible.first() {
                            self.set_focus(win)?;
                        }
                    }
                }
            }
            Event::KeyPress(event) => {
                let (action, arg) = keyboard::handle_key_press(event, &self.config.keybindings);
                match action {
                    KeyAction::Quit => return Ok(Some(false)),
                    KeyAction::Restart => return Ok(Some(true)),
                    _ => self.handle_key_action(action, &arg)?,
                }
            }
            Event::ButtonPress(event) => {
                let is_bar_click = self.bars.iter()
                    .enumerate()
                    .find(|(_, bar)| bar.window() == event.event);

                if let Some((monitor_index, bar)) = is_bar_click {
                    if let Some(tag_index) = bar.handle_click(event.event_x) {
                        if monitor_index != self.selected_monitor {
                            self.selected_monitor = monitor_index;
                        }
                        self.view_tag(tag_index)?;
                    }
                } else if event.child != x11rb::NONE {
                    self.set_focus(event.child)?;

                    if event.detail == ButtonIndex::M1.into() {
                        self.move_mouse(event.child)?;
                    } else if event.detail == ButtonIndex::M3.into() {
                        self.resize_mouse(event.child)?;
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
            }
            _ => {}
        }
        Ok(None)
    }

    fn apply_layout(&self) -> WmResult<()> {
        if self.fullscreen_window.is_some() {
            return Ok(());
        }

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

        for (monitor_index, monitor) in self.monitors.iter().enumerate() {
            let visible: Vec<Window> = self
                .windows
                .iter()
                .filter(|&&w| {
                    let window_mon = self.window_monitor.get(&w).copied().unwrap_or(0);
                    if window_mon != monitor_index {
                        return false;
                    }
                    if self.floating_windows.contains(&w) {
                        return false;
                    }
                    if let Some(&tags) = self.window_tags.get(&w) {
                        (tags & monitor.selected_tags) != 0
                    } else {
                        false
                    }
                })
                .copied()
                .collect();

            let bar_height = self.bars.get(monitor_index).map(|b| b.height() as u32).unwrap_or(0);
            let usable_height = monitor.height.saturating_sub(bar_height);

            let geometries = self
                .layout
                .arrange(&visible, monitor.width, usable_height, &gaps);

            for (window, geometry) in visible.iter().zip(geometries.iter()) {
                let adjusted_width = geometry.width.saturating_sub(2 * border_width);
                let adjusted_height = geometry.height.saturating_sub(2 * border_width);

                let adjusted_x = geometry.x_coordinate + monitor.x;
                let adjusted_y = geometry.y_coordinate + monitor.y + bar_height as i32;

                self.connection.configure_window(
                    *window,
                    &ConfigureWindowAux::new()
                        .x(adjusted_x)
                        .y(adjusted_y)
                        .width(adjusted_width)
                        .height(adjusted_height),
                )?;
            }
        }

        self.connection.flush()?;
        Ok(())
    }

    fn remove_window(&mut self, window: Window) -> WmResult<()> {
        let initial_count = self.windows.len();
        self.windows.retain(|&w| w != window);
        self.window_tags.remove(&window);
        self.window_monitor.remove(&window);
        self.floating_windows.remove(&window);

        if self.fullscreen_window == Some(window) {
            self.fullscreen_window = None;
            for bar in &self.bars {
                self.connection.map_window(bar.window())?;
            }
        }

        if self.windows.len() < initial_count {
            let focused = self.monitors.get(self.selected_monitor).and_then(|m| m.focused_window);
            if focused == Some(window) {
                let visible = self.visible_windows_on_monitor(self.selected_monitor);
                if let Some(&new_win) = visible.last() {
                    self.set_focus(new_win)?;
                } else if let Some(monitor) = self.monitors.get_mut(self.selected_monitor) {
                    monitor.focused_window = None;
                }
            }

            self.apply_layout()?;
            self.update_bar()?;
        }
        Ok(())
    }
}
