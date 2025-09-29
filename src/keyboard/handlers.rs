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
