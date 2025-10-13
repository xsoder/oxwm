use crate::Config;
use crate::bar::Bar;
use crate::keyboard::{self, Arg, KeyAction, handlers};
use crate::layout::GapConfig;
use crate::layout::Layout;
use crate::layout::tiling::TilingLayout;
use anyhow::Result;
use std::collections::HashSet;
use std::path::PathBuf;
use x11rb::cursor::Handle as CursorHandle;

use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

pub type TagMask = u32;
pub fn tag_mask(tag: usize) -> TagMask {
    1 << tag
}

pub struct WindowManager {
    config: Config,
    connection: RustConnection,
    screen_number: usize,
    root: Window,
    screen: Screen,
    windows: Vec<Window>,
    focused_window: Option<Window>,
    layout: Box<dyn Layout>,
    window_tags: std::collections::HashMap<Window, TagMask>,
    selected_tags: TagMask,
    gaps_enabled: bool,
    fullscreen_window: Option<Window>,
    floating_windows: HashSet<Window>,
    bar: Bar,
}

impl WindowManager {
    pub fn new(config: Config) -> Result<Self> {
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
                            | EventMask::BUTTON_PRESS,
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

        let bar = Bar::new(&connection, &screen, screen_number, &config)?;

        let selected_tags = Self::get_saved_selected_tags(&connection, root, config.tags.len())?;
        let gaps_enabled = config.gaps_enabled;

        let mut window_manager = Self {
            config,
            connection,
            screen_number,
            root,
            screen,
            windows: Vec::new(),
            focused_window: None,
            layout: Box::new(TilingLayout),
            window_tags: std::collections::HashMap::new(),
            selected_tags,
            gaps_enabled,
            fullscreen_window: None,
            floating_windows: HashSet::new(),
            bar,
        };

        window_manager.scan_existing_windows()?;
        window_manager.update_bar()?;

        Ok(window_manager)
    }

    fn get_saved_selected_tags(
        connection: &RustConnection,
        root: Window,
        tag_count: usize,
    ) -> Result<TagMask> {
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

    fn scan_existing_windows(&mut self) -> Result<()> {
        let tree = self.connection.query_tree(self.root)?.reply()?;
        let net_client_info = self
            .connection
            .intern_atom(false, b"_NET_CLIENT_INFO")?
            .reply()?
            .atom;

        let wm_state_atom = self
            .connection
            .intern_atom(false, b"WM_STATE")?
            .reply()?
            .atom;

        for &window in &tree.children {
            if window == self.bar.window() {
                continue;
            }

            let attrs = match self.connection.get_window_attributes(window)?.reply() {
                Ok(attrs) => attrs,
                Err(_) => continue,
            };

            if attrs.override_redirect {
                continue;
            }

            if attrs.map_state == MapState::VIEWABLE {
                let tag = self.get_saved_tag(window, net_client_info)?;
                self.windows.push(window);
                self.window_tags.insert(window, tag);
                continue;
            }

            if attrs.map_state == MapState::UNMAPPED {
                let has_wm_state = match self
                    .connection
                    .get_property(false, window, wm_state_atom, AtomEnum::ANY, 0, 2)?
                    .reply()
                {
                    Ok(prop) => !prop.value.is_empty(),
                    Err(_) => false,
                };

                if !has_wm_state {
                    continue;
                }

                let has_wm_class = match self
                    .connection
                    .get_property(false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 1024)?
                    .reply()
                {
                    Ok(prop) => !prop.value.is_empty(),
                    Err(_) => false,
                };

                if has_wm_class {
                    let tag = self.get_saved_tag(window, net_client_info)?;
                    self.windows.push(window);
                    self.window_tags.insert(window, tag);
                }
            }
        }

        if let Some(&first) = self.windows.first() {
            self.set_focus(Some(first))?;
        }

        self.apply_layout()?;
        Ok(())
    }

    fn get_saved_tag(&self, window: Window, net_client_info: Atom) -> Result<TagMask> {
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

        Ok(self.selected_tags)
    }

    fn save_client_tag(&self, window: Window, tag: TagMask) -> Result<()> {
        let net_client_info = self
            .connection
            .intern_atom(false, b"_NET_CLIENT_INFO")?
            .reply()?
            .atom;

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

    fn set_wm_state(&self, window: Window, state: u32) -> Result<()> {
        let wm_state_atom = self
            .connection
            .intern_atom(false, b"WM_STATE")?
            .reply()?
            .atom;

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

    fn handle_restart(&self) -> Result<bool> {
        Ok(true)
    }

    pub fn run(&mut self) -> Result<bool> {
        println!("oxwm started on display {}", self.screen_number);

        keyboard::setup_keybinds(&self.connection, self.root, &self.config.keybindings)?;
        self.update_bar()?;

        loop {
            self.bar.update_blocks()?;

            if let Ok(Some(event)) = self.connection.poll_for_event() {
                if let Some(should_restart) = self.handle_event(event)? {
                    return Ok(should_restart);
                }
            }

            if self.bar.needs_redraw() {
                self.update_bar()?;
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    fn toggle_fullscreen(&mut self) -> Result<()> {
        if let Some(focused) = self.focused_window {
            if self.fullscreen_window == Some(focused) {
                self.fullscreen_window = None;

                self.connection.map_window(self.bar.window())?;

                self.apply_layout()?;
                self.update_focus_visuals()?;
            } else {
                self.fullscreen_window = Some(focused);

                self.connection.unmap_window(self.bar.window())?;

                let screen_width = self.screen.width_in_pixels as u32;
                let screen_height = self.screen.height_in_pixels as u32;

                self.connection.configure_window(
                    focused,
                    &ConfigureWindowAux::new()
                        .x(0)
                        .y(0)
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

    fn update_bar(&mut self) -> Result<()> {
        let mut occupied_tags: TagMask = 0;
        for &tags in self.window_tags.values() {
            occupied_tags |= tags;
        }

        self.bar.invalidate();
        self.bar
            .draw(&self.connection, self.selected_tags, occupied_tags)?;
        Ok(())
    }

    fn handle_key_action(&mut self, action: KeyAction, arg: &Arg) -> Result<()> {
        match action {
            KeyAction::Spawn => handlers::handle_spawn_action(action, arg)?,
            KeyAction::KillClient => {
                if let Some(focused) = self.focused_window {
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
            KeyAction::None => {}
        }
        Ok(())
    }

    fn is_window_visible(&self, window: Window) -> bool {
        if let Some(&tags) = self.window_tags.get(&window) {
            (tags & self.selected_tags) != 0
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

    fn update_window_visibility(&self) -> Result<()> {
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

    pub fn view_tag(&mut self, tag_index: usize) -> Result<()> {
        if tag_index >= self.config.tags.len() {
            return Ok(());
        }

        if self.fullscreen_window.is_some() {
            self.fullscreen_window = None;
            self.connection.map_window(self.bar.window())?;
        }

        self.selected_tags = tag_mask(tag_index);

        self.save_selected_tags()?;

        self.update_window_visibility()?;
        self.apply_layout()?;
        self.update_bar()?;

        let visible = self.visible_windows();
        self.set_focus(visible.first().copied())?;

        Ok(())
    }

    fn save_selected_tags(&self) -> Result<()> {
        let net_current_desktop = self
            .connection
            .intern_atom(false, b"_NET_CURRENT_DESKTOP")?
            .reply()?
            .atom;

        let desktop = self.selected_tags.trailing_zeros();

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

    pub fn move_to_tag(&mut self, tag_index: usize) -> Result<()> {
        if tag_index >= self.config.tags.len() {
            return Ok(());
        }

        if let Some(focused) = self.focused_window {
            let mask = tag_mask(tag_index);
            self.window_tags.insert(focused, mask);

            let _ = self.save_client_tag(focused, mask);

            self.update_window_visibility()?;
            self.apply_layout()?;
            self.update_bar()?;
        }

        Ok(())
    }

    pub fn cycle_focus(&mut self, direction: i32) -> Result<()> {
        let visible = self.visible_windows();

        if visible.is_empty() {
            return Ok(());
        }

        let next_window = if let Some(current) = self.focused_window {
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

        self.set_focus(Some(next_window))?;
        Ok(())
    }

    pub fn set_focus(&mut self, window: Option<Window>) -> Result<()> {
        self.focused_window = window;

        if let Some(win) = window {
            self.connection
                .set_input_focus(InputFocus::POINTER_ROOT, win, x11rb::CURRENT_TIME)?;
            self.connection.flush()?;
        }

        self.update_focus_visuals()?;
        Ok(())
    }

    fn update_focus_visuals(&self) -> Result<()> {
        for &window in &self.windows {
            let (border_color, border_width) = if self.focused_window == Some(window) {
                (self.config.border_focused, self.config.border_width)
            } else {
                (self.config.border_unfocused, self.config.border_width)
            };

            self.connection.configure_window(
                window,
                &ConfigureWindowAux::new().border_width(border_width),
            )?;

            self.connection.change_window_attributes(
                window,
                &ChangeWindowAttributesAux::new().border_pixel(border_color),
            )?;
        }
        self.connection.flush()?;
        Ok(())
    }

    fn move_mouse(&mut self, window: Window) -> Result<()> {
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

    fn resize_mouse(&mut self, window: Window) -> Result<()> {
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

    fn handle_event(&mut self, event: Event) -> Result<Option<bool>> {
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

                self.windows.push(event.window);
                self.window_tags.insert(event.window, self.selected_tags);
                self.set_wm_state(event.window, 1)?;
                let _ = self.save_client_tag(event.window, self.selected_tags);

                self.apply_layout()?;
                self.update_bar()?;
                self.set_focus(Some(event.window))?;
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
                    self.set_focus(Some(event.event))?;
                }
            }
            Event::KeyPress(event) => {
                let (action, arg) = keyboard::handle_key_press(event, &self.config.keybindings)?;
                match action {
                    KeyAction::Quit => return Ok(Some(false)),
                    KeyAction::Restart => return Ok(Some(self.handle_restart()?)),
                    _ => self.handle_key_action(action, &arg)?,
                }
            }
            Event::ButtonPress(event) => {
                if event.event == self.bar.window() {
                    if let Some(tag_index) = self.bar.handle_click(event.event_x) {
                        self.view_tag(tag_index)?;
                    }
                } else if event.child != x11rb::NONE {
                    self.set_focus(Some(event.child))?;

                    if event.detail == ButtonIndex::M1.into() {
                        self.move_mouse(event.child)?;
                    } else if event.detail == ButtonIndex::M3.into() {
                        self.resize_mouse(event.child)?;
                    }
                }
            }
            Event::Expose(event) => {
                if event.window == self.bar.window() {
                    self.bar.invalidate();
                    self.update_bar()?;
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn apply_layout(&self) -> Result<()> {
        if self.fullscreen_window.is_some() {
            return Ok(());
        }
        let screen_width = self.screen.width_in_pixels as u32;
        let screen_height = self.screen.height_in_pixels as u32;
        let border_width = self.config.border_width;

        let bar_height = self.bar.height() as u32;
        let usable_height = screen_height.saturating_sub(bar_height);

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

        let visible: Vec<Window> = self
            .visible_windows()
            .into_iter()
            .filter(|w| !self.floating_windows.contains(w))
            .collect();

        let geometries = self
            .layout
            .arrange(&visible, screen_width, usable_height, &gaps);

        for (window, geometry) in visible.iter().zip(geometries.iter()) {
            let adjusted_width = geometry.width.saturating_sub(2 * border_width);
            let adjusted_height = geometry.height.saturating_sub(2 * border_width);

            let adjusted_y = geometry.y_coordinate + bar_height as i32;

            self.connection.configure_window(
                *window,
                &ConfigureWindowAux::new()
                    .x(geometry.x_coordinate)
                    .y(adjusted_y)
                    .width(adjusted_width)
                    .height(adjusted_height),
            )?;
        }
        self.connection.flush()?;
        Ok(())
    }

    fn remove_window(&mut self, window: Window) -> Result<()> {
        let initial_count = self.windows.len();
        self.windows.retain(|&w| w != window);
        self.window_tags.remove(&window);
        self.floating_windows.remove(&window);

        if self.fullscreen_window == Some(window) {
            self.fullscreen_window = None;
            self.connection.map_window(self.bar.window())?;
        }

        if self.windows.len() < initial_count {
            if self.focused_window == Some(window) {
                let new_focus = if self.windows.is_empty() {
                    None
                } else {
                    Some(self.windows[self.windows.len() - 1])
                };
                self.set_focus(new_focus)?;
            }

            self.apply_layout()?;
            self.update_bar()?;
        }
        Ok(())
    }
}
