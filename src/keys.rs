use crate::keyboard;
use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

pub fn setup_keybinds(connection: &impl Connection, root: Window) -> Result<()> {
    connection.grab_key(
        false,
        root,
        ModMask::M4.into(),
        keyboard::RETURN,
        GrabMode::ASYNC,
        GrabMode::ASYNC,
    )?;
    connection.grab_key(
        false,
        root,
        (ModMask::M1 | ModMask::SHIFT).into(),
        keyboard::Q,
        GrabMode::ASYNC,
        GrabMode::ASYNC,
    )?;

    connection.grab_key(
        false,
        root,
        ModMask::M4.into(),
        keyboard::Q,
        GrabMode::ASYNC,
        GrabMode::ASYNC,
    )?;

    Ok(())
}

pub fn handle_key_press(connection: &impl Connection, event: KeyPressEvent) -> Result<()> {
    println!("KeyPress: detail={}, state={:?}", event.detail, event.state);

    match (event.detail, event.state) {
        (keyboard::RETURN, state) if state.contains(KeyButMask::MOD1) => {
            println!("Spawning terminal");
            std::process::Command::new("xterm").spawn()?;
        }
        (keyboard::Q, state) if state.contains(KeyButMask::MOD1 | KeyButMask::SHIFT) => {
            println!("Closing focused window");
            let focus_reply = connection.get_input_focus()?.reply()?;
            if focus_reply.focus != x11rb::NONE && focus_reply.focus != event.root {
                connection.kill_client(focus_reply.focus)?;
                connection.flush()?;
            }
        }
        _ => {}
    }
    Ok(())
}
