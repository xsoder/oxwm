use std::collections::HashMap;
use std::io::{ErrorKind, Result};
use std::process::Command;

use serde::Deserialize;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

use crate::errors::X11Error;
use crate::keyboard::keysyms::{self, Keysym};

#[derive(Debug, Copy, Clone, Deserialize)]
pub enum KeyAction {
    Spawn,
    KillClient,
    FocusStack,
    FocusDirection,
    SwapDirection,
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
    pub(crate) keysym: Keysym,
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

    pub fn single_key(
        modifiers: Vec<KeyButMask>,
        keysym: Keysym,
        func: KeyAction,
        arg: Arg,
    ) -> Self {
        Self {
            keys: vec![KeyPress { modifiers, keysym }],
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

fn build_keysym_maps(
    connection: &impl Connection,
) -> std::result::Result<(HashMap<Keysym, Vec<Keycode>>, HashMap<Keycode, Keysym>), X11Error> {
    let setup = connection.setup();
    let min_keycode = setup.min_keycode;
    let max_keycode = setup.max_keycode;

    let keyboard_mapping = connection
        .get_keyboard_mapping(min_keycode, max_keycode - min_keycode + 1)?
        .reply()?;

    let mut keysym_to_keycode: HashMap<Keysym, Vec<Keycode>> = HashMap::new();
    let mut keycode_to_keysym: HashMap<Keycode, Keysym> = HashMap::new();
    let keysyms_per_keycode = keyboard_mapping.keysyms_per_keycode;

    for keycode in min_keycode..=max_keycode {
        let index = (keycode - min_keycode) as usize * keysyms_per_keycode as usize;

        for i in 0..keysyms_per_keycode as usize {
            if let Some(&keysym) = keyboard_mapping.keysyms.get(index + i) {
                if keysym != 0 {
                    keysym_to_keycode
                        .entry(keysym)
                        .or_insert_with(Vec::new)
                        .push(keycode);
                    keycode_to_keysym.entry(keycode).or_insert(keysym);
                }
            }
        }
    }

    Ok((keysym_to_keycode, keycode_to_keysym))
}

pub fn setup_keybinds(
    connection: &impl Connection,
    root: Window,
    keybindings: &[KeyBinding],
) -> std::result::Result<(), X11Error> {
    use std::collections::HashSet;

    let (keysym_to_keycode, _) = build_keysym_maps(connection)?;
    let mut grabbed_keys: HashSet<(u16, Keycode)> = HashSet::new();

    for keybinding in keybindings {
        if keybinding.keys.is_empty() {
            continue;
        }

        let first_key = &keybinding.keys[0];
        let modifier_mask = modifiers_to_mask(&first_key.modifiers);

        if let Some(keycodes) = keysym_to_keycode.get(&first_key.keysym) {
            if let Some(&keycode) = keycodes.first() {
                let key_tuple = (modifier_mask, keycode);

                if grabbed_keys.insert(key_tuple) {
                    connection.grab_key(
                        false,
                        root,
                        modifier_mask.into(),
                        keycode,
                        GrabMode::ASYNC,
                        GrabMode::ASYNC,
                    )?;
                }
            }
        }
    }

    Ok(())
}

pub fn handle_key_press(
    event: KeyPressEvent,
    keybindings: &[KeyBinding],
    keychord_state: &KeychordState,
    connection: &impl Connection,
) -> std::result::Result<KeychordResult, X11Error> {
    let (_, keycode_to_keysym) = build_keysym_maps(connection)?;
    let event_keysym = keycode_to_keysym.get(&event.detail).copied().unwrap_or(0);

    if event_keysym == keysyms::XK_ESCAPE {
        return Ok(match keychord_state {
            KeychordState::InProgress { .. } => KeychordResult::Cancelled,
            KeychordState::Idle => KeychordResult::None,
        });
    }

    Ok(match keychord_state {
        KeychordState::Idle => handle_first_key(event, event_keysym, keybindings),
        KeychordState::InProgress {
            candidates,
            keys_pressed,
        } => handle_next_key(event, event_keysym, keybindings, candidates, *keys_pressed),
    })
}

fn handle_first_key(
    event: KeyPressEvent,
    event_keysym: Keysym,
    keybindings: &[KeyBinding],
) -> KeychordResult {
    let mut candidates = Vec::new();

    for (keybinding_index, keybinding) in keybindings.iter().enumerate() {
        if keybinding.keys.is_empty() {
            continue;
        }

        let first_key = &keybinding.keys[0];
        let modifier_mask = modifiers_to_mask(&first_key.modifiers);

        if event_keysym == first_key.keysym && event.state == modifier_mask.into() {
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
    event_keysym: Keysym,
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

        if event_keysym == next_key.keysym && modifiers_match {
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

pub fn handle_spawn_action(action: KeyAction, arg: &Arg) -> Result<()> {
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
