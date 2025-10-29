use std::collections::HashSet;
use std::io;
use std::io::ErrorKind;
use std::process::Command;

use serde::Deserialize;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use crate::errors::X11Error;
use crate::keyboard::keycodes;

#[derive(Debug, Copy, Clone, Deserialize)]
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
    ChangeLayout,
    CycleLayout,
    MoveToTag,
    FocusMonitor,
    SmartMoveWin,
    ExchangeClient,
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

#[derive(Clone, Debug)]
pub struct KeyPress {
    pub(crate) modifiers: Vec<KeyButMask>,
    pub(crate) key: Keycode,
}

#[derive(Clone)]
pub struct KeyBinding {
    pub(crate) keys: Vec<KeyPress>,
    pub(crate) func: KeyAction,
    pub(crate) arg: Arg,
}

impl KeyBinding {
    pub fn new(keys: Vec<KeyPress>, func: KeyAction, arg: Arg) -> Self {
        Self { keys, func, arg }
    }

    pub fn single_key(modifiers: Vec<KeyButMask>, key: Keycode, func: KeyAction, arg: Arg) -> Self {
        Self {
            keys: vec![KeyPress { modifiers, key }],
            func,
            arg,
        }
    }
}

pub type Key = KeyBinding;

#[derive(Debug, Clone)]
pub enum KeychordState {
    Idle,
    InProgress {
        candidates: Vec<usize>,
        keys_pressed: usize,
    },
}

pub enum KeychordResult {
    Completed(KeyAction, Arg),
    InProgress(Vec<usize>),
    None,
    Cancelled,
}

pub fn modifiers_to_mask(modifiers: &[KeyButMask]) -> u16 {
    modifiers
        .iter()
        .fold(0u16, |acc, &modifier| acc | u16::from(modifier))
}

pub fn setup_keybinds(
    connection: &impl Connection,
    root: Window,
    keybindings: &[KeyBinding],
) -> Result<(), X11Error> {
    let mut grabbed_keys: HashSet<(u16, Keycode)> = HashSet::new();

    for keybinding in keybindings {
        if keybinding.keys.is_empty() {
            continue;
        }

        let first_key = &keybinding.keys[0];
        let modifier_mask = modifiers_to_mask(&first_key.modifiers);
        let key_tuple = (modifier_mask, first_key.key);

        if grabbed_keys.insert(key_tuple) {
            connection.grab_key(
                false,
                root,
                modifier_mask.into(),
                first_key.key,
                GrabMode::ASYNC,
                GrabMode::ASYNC,
            )?;
        }
    }

    connection.grab_key(
        false,
        root,
        ModMask::from(0u16),
        keycodes::ESCAPE,
        GrabMode::ASYNC,
        GrabMode::ASYNC,
    )?;

    Ok(())
}

pub fn handle_key_press(
    event: KeyPressEvent,
    keybindings: &[KeyBinding],
    keychord_state: &KeychordState,
) -> KeychordResult {
    if event.detail == keycodes::ESCAPE {
        return match keychord_state {
            KeychordState::InProgress { .. } => KeychordResult::Cancelled,
            KeychordState::Idle => KeychordResult::None,
        };
    }

    match keychord_state {
        KeychordState::Idle => handle_first_key(event, keybindings),
        KeychordState::InProgress {
            candidates,
            keys_pressed,
        } => handle_next_key(event, keybindings, candidates, *keys_pressed),
    }
}

fn handle_first_key(event: KeyPressEvent, keybindings: &[KeyBinding]) -> KeychordResult {
    let mut candidates = Vec::new();

    for (keybinding_index, keybinding) in keybindings.iter().enumerate() {
        if keybinding.keys.is_empty() {
            continue;
        }

        let first_key = &keybinding.keys[0];
        let modifier_mask = modifiers_to_mask(&first_key.modifiers);

        if event.detail == first_key.key && event.state == modifier_mask.into() {
            if keybinding.keys.len() == 1 {
                return KeychordResult::Completed(keybinding.func, keybinding.arg.clone());
            } else {
                candidates.push(keybinding_index);
            }
        }
    }

    if candidates.is_empty() {
        KeychordResult::None
    } else {
        KeychordResult::InProgress(candidates)
    }
}

fn handle_next_key(
    event: KeyPressEvent,
    keybindings: &[KeyBinding],
    candidates: &[usize],
    keys_pressed: usize,
) -> KeychordResult {
    let mut new_candidates = Vec::new();

    for &candidate_index in candidates {
        let keybinding = &keybindings[candidate_index];

        if keys_pressed >= keybinding.keys.len() {
            continue;
        }

        let next_key = &keybinding.keys[keys_pressed];
        let required_mask = modifiers_to_mask(&next_key.modifiers);
        let event_state: u16 = event.state.into();

        let modifiers_match = if next_key.modifiers.is_empty() {
            true
        } else {
            (event_state & required_mask) == required_mask
        };

        if event.detail == next_key.key && modifiers_match {
            if keys_pressed + 1 == keybinding.keys.len() {
                return KeychordResult::Completed(keybinding.func, keybinding.arg.clone());
            } else {
                new_candidates.push(candidate_index);
            }
        }
    }

    if new_candidates.is_empty() {
        KeychordResult::Cancelled
    } else {
        KeychordResult::InProgress(new_candidates)
    }
}

pub fn handle_spawn_action(action: KeyAction, arg: &Arg) -> io::Result<()> {
    if let KeyAction::Spawn = action {
        match arg {
            Arg::Str(command) => match Command::new(command.as_str()).spawn() {
                Err(error) if error.kind() == ErrorKind::NotFound => {
                    eprintln!(
                        "KeyAction::Spawn failed: could not spawn \"{}\", command not found",
                        command
                    );
                }
                Err(error) => Err(error)?,
                _ => (),
            },
            Arg::Array(command) => {
                let Some((cmd, args)) = command.split_first() else {
                    return Ok(());
                };

                let args_str: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                match Command::new(cmd.as_str()).args(&args_str).spawn() {
                    Err(error) if error.kind() == ErrorKind::NotFound => {
                        eprintln!(
                            "KeyAction::Spawn failed: could not spawn \"{}\", command not found",
                            cmd
                        );
                    }
                    Err(error) => Err(error)?,
                    _ => (),
                }
            }
            _ => {}
        }
    }

    Ok(())
}
