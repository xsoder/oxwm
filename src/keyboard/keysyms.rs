#![allow(dead_code)]

pub type Keysym = u32;
pub const XK_ESCAPE: Keysym = 0xff1b;
pub const XK_RETURN: Keysym = 0xff0d;
pub const XK_SPACE: Keysym = 0x0020;
pub const XK_TAB: Keysym = 0xff09;
pub const XK_BACKSPACE: Keysym = 0xff08;
pub const XK_DELETE: Keysym = 0xffff;
pub const XK_F1: Keysym = 0xffbe;
pub const XK_F2: Keysym = 0xffbf;
pub const XK_F3: Keysym = 0xffc0;
pub const XK_F4: Keysym = 0xffc1;
pub const XK_F5: Keysym = 0xffc2;
pub const XK_F6: Keysym = 0xffc3;
pub const XK_F7: Keysym = 0xffc4;
pub const XK_F8: Keysym = 0xffc5;
pub const XK_F9: Keysym = 0xffc6;
pub const XK_F10: Keysym = 0xffc7;
pub const XK_F11: Keysym = 0xffc8;
pub const XK_F12: Keysym = 0xffc9;
pub const XK_A: Keysym = 0x0061;
pub const XK_B: Keysym = 0x0062;
pub const XK_C: Keysym = 0x0063;
pub const XK_D: Keysym = 0x0064;
pub const XK_E: Keysym = 0x0065;
pub const XK_F: Keysym = 0x0066;
pub const XK_G: Keysym = 0x0067;
pub const XK_H: Keysym = 0x0068;
pub const XK_I: Keysym = 0x0069;
pub const XK_J: Keysym = 0x006a;
pub const XK_K: Keysym = 0x006b;
pub const XK_L: Keysym = 0x006c;
pub const XK_M: Keysym = 0x006d;
pub const XK_N: Keysym = 0x006e;
pub const XK_O: Keysym = 0x006f;
pub const XK_P: Keysym = 0x0070;
pub const XK_Q: Keysym = 0x0071;
pub const XK_R: Keysym = 0x0072;
pub const XK_S: Keysym = 0x0073;
pub const XK_T: Keysym = 0x0074;
pub const XK_U: Keysym = 0x0075;
pub const XK_V: Keysym = 0x0076;
pub const XK_W: Keysym = 0x0077;
pub const XK_X: Keysym = 0x0078;
pub const XK_Y: Keysym = 0x0079;
pub const XK_Z: Keysym = 0x007a;
pub const XK_0: Keysym = 0x0030;
pub const XK_1: Keysym = 0x0031;
pub const XK_2: Keysym = 0x0032;
pub const XK_3: Keysym = 0x0033;
pub const XK_4: Keysym = 0x0034;
pub const XK_5: Keysym = 0x0035;
pub const XK_6: Keysym = 0x0036;
pub const XK_7: Keysym = 0x0037;
pub const XK_8: Keysym = 0x0038;
pub const XK_9: Keysym = 0x0039;
pub const XK_LEFT: Keysym = 0xff51;
pub const XK_UP: Keysym = 0xff52;
pub const XK_RIGHT: Keysym = 0xff53;
pub const XK_DOWN: Keysym = 0xff54;
pub const XK_HOME: Keysym = 0xff50;
pub const XK_END: Keysym = 0xff57;
pub const XK_PAGE_UP: Keysym = 0xff55;
pub const XK_PAGE_DOWN: Keysym = 0xff56;
pub const XK_INSERT: Keysym = 0xff63;
pub const XK_MINUS: Keysym = 0x002d;
pub const XK_EQUAL: Keysym = 0x003d;
pub const XK_LEFT_BRACKET: Keysym = 0x005b;
pub const XK_RIGHT_BRACKET: Keysym = 0x005d;
pub const XK_SEMICOLON: Keysym = 0x003b;
pub const XK_APOSTROPHE: Keysym = 0x0027;
pub const XK_GRAVE: Keysym = 0x0060;
pub const XK_BACKSLASH: Keysym = 0x005c;
pub const XK_COMMA: Keysym = 0x002c;
pub const XK_PERIOD: Keysym = 0x002e;
pub const XK_SLASH: Keysym = 0x002f;
pub const XK_PRINT: Keysym = 0xff61;

pub const XF86_AUDIO_RAISE_VOLUME: Keysym = 0x1008ff13;
pub const XF86_AUDIO_LOWER_VOLUME: Keysym = 0x1008ff11;
pub const XF86_AUDIO_MUTE: Keysym = 0x1008ff12;
pub const XF86_MON_BRIGHTNESS_UP: Keysym = 0x1008ff02;
pub const XF86_MON_BRIGHTNESS_DOWN: Keysym = 0x1008ff03;

pub fn format_keysym(keysym: Keysym) -> String {
    match keysym {
        XK_RETURN => "Return".to_string(),
        XK_ESCAPE => "Esc".to_string(),
        XK_SPACE => "Space".to_string(),
        XK_TAB => "Tab".to_string(),
        XK_BACKSPACE => "Backspace".to_string(),
        XK_DELETE => "Del".to_string(),
        XK_LEFT => "Left".to_string(),
        XK_RIGHT => "Right".to_string(),
        XK_UP => "Up".to_string(),
        XK_DOWN => "Down".to_string(),
        XK_HOME => "Home".to_string(),
        XK_END => "End".to_string(),
        XK_PAGE_UP => "PgUp".to_string(),
        XK_PAGE_DOWN => "PgDn".to_string(),
        XK_INSERT => "Ins".to_string(),
        XK_F1 => "F1".to_string(),
        XK_F2 => "F2".to_string(),
        XK_F3 => "F3".to_string(),
        XK_F4 => "F4".to_string(),
        XK_F5 => "F5".to_string(),
        XK_F6 => "F6".to_string(),
        XK_F7 => "F7".to_string(),
        XK_F8 => "F8".to_string(),
        XK_F9 => "F9".to_string(),
        XK_F10 => "F10".to_string(),
        XK_F11 => "F11".to_string(),
        XK_F12 => "F12".to_string(),
        XK_SLASH => "/".to_string(),
        XK_COMMA => ",".to_string(),
        XK_PERIOD => ".".to_string(),
        XK_MINUS => "-".to_string(),
        XK_EQUAL => "=".to_string(),
        XK_GRAVE => "`".to_string(),
        XK_LEFT_BRACKET => "[".to_string(),
        XK_RIGHT_BRACKET => "]".to_string(),
        XK_SEMICOLON => ";".to_string(),
        XK_APOSTROPHE => "'".to_string(),
        XK_BACKSLASH => "\\".to_string(),
        XK_PRINT => "Print".to_string(),
        XF86_AUDIO_RAISE_VOLUME => "Vol+".to_string(),
        XF86_AUDIO_LOWER_VOLUME => "Vol-".to_string(),
        XF86_AUDIO_MUTE => "Mute".to_string(),
        XF86_MON_BRIGHTNESS_UP => "Bright+".to_string(),
        XF86_MON_BRIGHTNESS_DOWN => "Bright-".to_string(),
        XK_A..=XK_Z => {
            let ch = (keysym - XK_A + b'A' as u32) as u8 as char;
            ch.to_string()
        }
        XK_0..=XK_9 => {
            let ch = (keysym - XK_0 + b'0' as u32) as u8 as char;
            ch.to_string()
        }
        _ => format!("0x{:x}", keysym),
    }
}
