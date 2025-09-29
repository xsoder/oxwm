use crate::keyboard::keycodes;
use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

#[derive(Debug, Copy, Clone)]
pub enum KeyAction {
    Spawn,
    KillClient,
    FocusStack,
    Quit,
    None,
}

#[derive(Debug)]
pub enum Arg {
    Str(&'static str),
    Int(i32),
    None,
}

pub struct Key {
    pub(crate) modifiers: &'static [KeyButMask],
    pub(crate) key: Keycode,
    pub(crate) func: KeyAction,
    pub(crate) arg: Arg,
}

impl Key {
    pub const fn new(
        modifiers: &'static [KeyButMask],
        key: Keycode,
        func: KeyAction,
        arg: Arg,
    ) -> Self {
        Self {
            modifiers,
            key,
            func,
            arg,
        }
    }
}

const KEYBINDINGS: &[Key] = &[
    Key::new(
        &[KeyButMask::MOD1],
        keycodes::RETURN,
        KeyAction::Spawn,
        Arg::Str("xclock"),
    ),
    Key::new(
        &[KeyButMask::MOD1, KeyButMask::SHIFT],
        keycodes::Q,
        KeyAction::KillClient,
        Arg::None,
    ),
    Key::new(&[KeyButMask::MOD1], keycodes::Q, KeyAction::Quit, Arg::None),
    Key::new(
        &[KeyButMask::MOD1],
        keycodes::J,
        KeyAction::FocusStack,
        Arg::Int(1),
    ),
    Key::new(
        &[KeyButMask::MOD1],
        keycodes::K,
        KeyAction::FocusStack,
        Arg::Int(-1),
    ),
];

fn modifiers_to_mask(modifiers: &[KeyButMask]) -> u16 {
    modifiers
        .iter()
        .fold(0u16, |acc, &modifier| acc | u16::from(modifier))
}

pub fn setup_keybinds(connection: &impl Connection, root: Window) -> Result<()> {
    for keybinding in KEYBINDINGS {
        let modifier_mask = modifiers_to_mask(keybinding.modifiers);

        connection.grab_key(
            false,
            root,
            modifier_mask.into(),
            keybinding.key,
            GrabMode::ASYNC,
            GrabMode::ASYNC,
        )?;
    }
    Ok(())
}

pub fn handle_key_press(event: KeyPressEvent) -> Result<(KeyAction, &'static Arg)> {
    println!("KeyPress: detail={}, state={:?}", event.detail, event.state);

    for keybinding in KEYBINDINGS {
        let modifier_mask = modifiers_to_mask(keybinding.modifiers);

        if event.detail == keybinding.key && event.state == modifier_mask.into() {
            return Ok((keybinding.func, &keybinding.arg));
        }
    }

    Ok((KeyAction::None, &Arg::None))
}
