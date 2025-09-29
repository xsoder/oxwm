use crate::keyboard::keycodes;
use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

#[derive(Debug)]
pub enum KeyAction {
    SpawnTerminal,
    CloseWindow,
    CycleWindow,
    Quit,
    None,
}

pub fn setup_keybinds(connection: &impl Connection, root: Window) -> Result<()> {
    connection.grab_key(
        false,
        root,
        ModMask::M1.into(),
        keycodes::RETURN,
        GrabMode::ASYNC,
        GrabMode::ASYNC,
    )?;

    connection.grab_key(
        false,
        root,
        (ModMask::M1 | ModMask::SHIFT).into(),
        keycodes::Q,
        GrabMode::ASYNC,
        GrabMode::ASYNC,
    )?;

    connection.grab_key(
        false,
        root,
        ModMask::M1.into(),
        keycodes::Q,
        GrabMode::ASYNC,
        GrabMode::ASYNC,
    )?;

    connection.grab_key(
        false,
        root,
        ModMask::M1.into(),
        keycodes::J,
        GrabMode::ASYNC,
        GrabMode::ASYNC,
    )?;

    Ok(())
}

pub fn handle_key_press(event: KeyPressEvent) -> Result<KeyAction> {
    println!("KeyPress: detail={}, state={:?}", event.detail, event.state);
    let action = match (event.detail, event.state) {
        (keycodes::RETURN, state) if state.contains(KeyButMask::MOD1) => KeyAction::SpawnTerminal,
        (keycodes::Q, state) if state.contains(KeyButMask::MOD1 | KeyButMask::SHIFT) => {
            KeyAction::CloseWindow
        }
        (keycodes::Q, state) if state.contains(KeyButMask::MOD1) => KeyAction::Quit,
        (keycodes::J, state) if state.contains(KeyButMask::MOD1) => KeyAction::CycleWindow,
        _ => KeyAction::None,
    };
    Ok(action)
}

// pub struct Key {
//     pub(crate) modifiers: &'static [KeyButMask], // List of modifiers
//     pub(crate) key: Keycode,
//     pub(crate) func: KeyAction,
//     pub(crate) arg: Arg,
// }
//
// impl Key {
//     pub const fn new(
//         modifiers: &'static [KeyButMask],
//         key: Keycode,
//         func: KeyAction,
//         arg: Arg,
//     ) -> Self {
//         Self {
//             modifiers,
//             key,
//             func,
//             arg,
//         }
//     }
// }
//
// const KEYBINDINGS: &[Key] = &[
//     Key::new(
//         &[KeyButMask::MOD1],
//         keycodes::RETURN,
//         KeyAction::Spawn,
//         Arg::Str("xclock"),
//     ),
//     Key::new(
//         &[KeyButMask::MOD1, KeyButMask::SHIFT],
//         keycodes::Q,
//         KeyAction::KillClient,
//         Arg::None,
//     ),
//     Key::new(&[KeyButMask::MOD1], keycodes::Q, KeyAction::Quit, Arg::None),
//     Key::new(
//         &[KeyButMask::MOD1],
//         keycodes::J,
//         KeyAction::FocusStack,
//         Arg::Int(1),
//     ),
//     Key::new(
//         &[KeyButMask::MOD1],
//         keycodes::K,
//         KeyAction::FocusStack,
//         Arg::Int(-1),
//     ),
// ];
//
