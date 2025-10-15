use std::io;
use std::process::Command;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use crate::errors::X11Error;

#[derive(Debug, Copy, Clone)]
pub enum KeyAction {
    Spawn,
    KillClient,
    FocusStack,
    Quit,
    Restart,
    Recompile,
    ViewTag,
    ToggleGaps,
    ToggleFullScreen,
    ToggleFloating,
    MoveToTag,
    None,
}

#[derive(Debug, Clone)]
pub enum Arg {
    None,
    Int(i32),
    Str(String),
    Array(Vec<String>),
}

impl Arg {
    pub const fn none() -> Self {
        Arg::None
    }
}

#[derive(Clone)]
pub struct Key {
    pub(crate) modifiers: Vec<KeyButMask>,
    pub(crate) key: Keycode,
    pub(crate) func: KeyAction,
    pub(crate) arg: Arg,
}

impl Key {
    pub fn new(modifiers: Vec<KeyButMask>, key: Keycode, func: KeyAction, arg: Arg) -> Self {
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

pub fn setup_keybinds(
    connection: &impl Connection,
    root: Window,
    keybindings: &[Key],
) -> Result<(), X11Error> {
    for keybinding in keybindings {
        let modifier_mask = modifiers_to_mask(&keybinding.modifiers);

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

pub fn handle_key_press(event: KeyPressEvent, keybindings: &[Key]) -> (KeyAction, Arg) {
    for keybinding in keybindings {
        let modifier_mask = modifiers_to_mask(&keybinding.modifiers);

        if event.detail == keybinding.key && event.state == modifier_mask.into() {
            return (keybinding.func, keybinding.arg.clone());
        }
    }

    (KeyAction::None, Arg::None)
}

pub fn handle_spawn_action(action: KeyAction, arg: &Arg) -> io::Result<()> {
    use io::ErrorKind;
    if let KeyAction::Spawn = action {
        match arg {
            Arg::Str(command) => match Command::new(command.as_str()).spawn() {
                Err(err) if err.kind() == ErrorKind::NotFound => {
                    eprintln!(
                        "KeyAction::Spawn failed: could not spawn \"{}\", command not found",
                        command
                    );
                }
                Err(err) => Err(err)?,
                _ => (),
            },
            Arg::Array(command) => {
                let Some((cmd, args)) = command.split_first() else {
                    return Ok(());
                };

                let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                match Command::new(cmd.as_str()).args(&args_str).spawn() {
                    Err(err) if err.kind() == ErrorKind::NotFound => {
                        eprintln!(
                            "KeyAction::Spawn failed: could not spawn \"{}\", command not found",
                            cmd
                        );
                    }
                    Err(err) => Err(err)?,
                    _ => (),
                }
            }
            _ => {}
        }
    }

    Ok(())
}
