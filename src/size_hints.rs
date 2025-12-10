#![allow(dead_code)]

pub mod flags {
    pub const P_MIN_SIZE: u32 = 1 << 4;
    pub const P_MAX_SIZE: u32 = 1 << 5;
    pub const P_RESIZE_INC: u32 = 1 << 6;
    pub const P_ASPECT: u32 = 1 << 7;
    pub const P_BASE_SIZE: u32 = 1 << 8;
}

pub mod offset {
    pub const FLAGS: usize = 0;
    pub const MIN_WIDTH: usize = 5;
    pub const MIN_HEIGHT: usize = 6;
    pub const MAX_WIDTH: usize = 7;
    pub const MAX_HEIGHT: usize = 8;
    pub const WIDTH_INC: usize = 9;
    pub const HEIGHT_INC: usize = 10;
    pub const MIN_ASPECT_X: usize = 11;
    pub const MIN_ASPECT_Y: usize = 12;
    pub const MAX_ASPECT_X: usize = 13;
    pub const MAX_ASPECT_Y: usize = 14;
    pub const BASE_WIDTH: usize = 15;
    pub const BASE_HEIGHT: usize = 16;
}
