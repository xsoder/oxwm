use x11rb::protocol::xproto::Window;

pub type TagMask = u32;

#[derive(Debug, Clone)]
pub struct Client {
    pub name: String,
    pub min_aspect: f32,
    pub max_aspect: f32,
    pub x_position: i16,
    pub y_position: i16,
    pub width: u16,
    pub height: u16,
    pub old_x_position: i16,
    pub old_y_position: i16,
    pub old_width: u16,
    pub old_height: u16,
    pub base_width: i32,
    pub base_height: i32,
    pub increment_width: i32,
    pub increment_height: i32,
    pub max_width: i32,
    pub max_height: i32,
    pub min_width: i32,
    pub min_height: i32,
    pub hints_valid: bool,
    pub border_width: u16,
    pub old_border_width: u16,
    pub tags: TagMask,
    pub is_fixed: bool,
    pub is_floating: bool,
    pub is_urgent: bool,
    pub never_focus: bool,
    pub old_state: bool,
    pub is_fullscreen: bool,
    pub next: Option<Window>,
    pub stack_next: Option<Window>,
    pub monitor_index: usize,
    pub window: Window,
}

impl Client {
    pub fn new(window: Window, monitor_index: usize, tags: TagMask) -> Self {
        Self {
            name: String::new(),
            min_aspect: 0.0,
            max_aspect: 0.0,
            x_position: 0,
            y_position: 0,
            width: 0,
            height: 0,
            old_x_position: 0,
            old_y_position: 0,
            old_width: 0,
            old_height: 0,
            base_width: 0,
            base_height: 0,
            increment_width: 0,
            increment_height: 0,
            max_width: 0,
            max_height: 0,
            min_width: 0,
            min_height: 0,
            hints_valid: false,
            border_width: 0,
            old_border_width: 0,
            tags,
            is_fixed: false,
            is_floating: false,
            is_urgent: false,
            never_focus: false,
            old_state: false,
            is_fullscreen: false,
            next: None,
            stack_next: None,
            monitor_index,
            window,
        }
    }

    pub fn width_with_border(&self) -> u16 {
        self.width.saturating_add(2 * self.border_width)
    }

    pub fn height_with_border(&self) -> u16 {
        self.height.saturating_add(2 * self.border_width)
    }
}
