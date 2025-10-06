use crate::config::KEYBINDINGS;
use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

#[derive(Debug, Copy, Clone)]
pub enum KeyAction {
    Spawn,
    KillClient,
    FocusStack,
    Quit,
    Restart,
    ViewTag,
    MoveToTag,
    None,
}

#[derive(Debug)]
pub enum Arg {
    None,
    Int(i32),
    Str(&'static str),
    Array(&'static [&'static str]),
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

fn modifiers_to_mask(modifiers: &[KeyButMask]) -> u16 {
    modifiers
        .iter()
        .fold(0u16, |acc, &modifier| acc | u16::from(modifier))
}

pub fn setup_keybinds(connection: &impl Connection, root: Window) -> Result<()> {
    for keybinding in KEYBINDINGS {
        let modifier_mask = modifiers_to_mask(keybinding.modifiers);

        println!("Grabbing key {} with mod {}", keybinding.key, modifier_mask);

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
    for keybinding in KEYBINDINGS {
        let modifier_mask = modifiers_to_mask(keybinding.modifiers);

        if event.detail == keybinding.key && event.state == modifier_mask.into() {
            return Ok((keybinding.func, &keybinding.arg));
        }
    }

    Ok((KeyAction::None, &Arg::None))
}
