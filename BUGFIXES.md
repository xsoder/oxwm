# Bug Fixes Summary - bugfixes-floating-fullscreen branch

## Branch Info
- **Location**: `~/repos/oxwm-worktrees/bugfixes-floating-fullscreen`
- **Based on**: `lua-config-test` branch
- **Status**: Ready for testing on bare metal

## Bugs Fixed

### ✅ 1. Kill Client Killing All Firefox Instances
**Problem**: Pressing `Mod+Q` killed ALL Firefox windows, not just the focused one.

**Solution**: Implemented proper EWMH window closing protocol (src/window_manager.rs:1489-1547)
- First tries to send `WM_DELETE_WINDOW` message (graceful close)
- Only forcefully kills window if it doesn't support WM_DELETE_WINDOW
- Matches dwm behavior

**Test**: Open 2-3 Firefox windows, focus one, press `Mod+Q` → should only close that window

---

### ✅ 2. Fullscreen Not Working in Floating Mode
**Problem**: Fullscreen only hid the bar, didn't actually fullscreen windows

**Solution**: Implemented proper per-window EWMH fullscreen (src/window_manager.rs:1549-1610)
- Added `_NET_WM_STATE` and `_NET_WM_STATE_FULLSCREEN` atoms
- Saves window geometry before fullscreening
- Removes borders and makes window cover entire monitor
- Restores geometry when exiting fullscreen
- Added smart context-aware behavior:
  - In normie (floating) layout → fullscreens the focused window
  - With explicitly floating window → fullscreens that window
  - In tiling layout → hides bar (monitor-level fullscreen)

**Test**:
1. Switch to normie layout (`Mod+F`)
2. Press `Mod+Shift+F` → window should fullscreen properly
3. In tiling mode, press `Mod+Shift+F` → should hide bar

---

### ✅ 3. YouTube Fullscreen (F key) Not Working
**Problem**: Applications couldn't request fullscreen via EWMH protocol

**Solution**: Implemented ClientMessage event handler (src/window_manager.rs:1909-1930)
- Listens for `_NET_WM_STATE` change requests from clients
- Handles add/remove/toggle fullscreen actions
- Added `PROPERTY_CHANGE` to window event mask

**Test**:
1. Open Firefox, go to YouTube
2. Press `F` on a video → should properly fullscreen
3. Press `F` or `Esc` to exit → should return to previous size

---

### ✅ 4. Modal Windows Not Auto-Floating
**Problem**: Dialogs like Discord loading screens required manual floating

**Solution**: Implemented transient window detection (src/window_manager.rs:1612-1628, 1900-1903)
- Checks `WM_TRANSIENT_FOR` hint during MapRequest
- Automatically floats transient windows (dialogs, modals, popups)
- Matches dwm behavior

**Test**: Open Discord → loading modal should automatically float

---

### ⏸️ 5. Firefox Cursor Bug (NOT YET FIXED)
**Problem**: Cursor gets "stuck" in one state (e.g., pointer/hand) when hovering over Firefox. The cursor won't update to show different states (arrow, text cursor, etc.) even though clicks/hovers still work functionally. **This is intermittent and hard to reproduce.**

**Current Status**: Need more info to diagnose
- Only happens in Firefox (so far)
- Random/semi-reproducible
- Not tested if it happens in Xephyr or only bare metal

**Debug Steps** (when it happens next):
1. Does it happen more in floating or tiling mode?
2. Does switching tags fix it?
3. Does `Mod+Shift+R` (restart WM) fix it?
4. Does it only happen with certain Firefox actions (opening menus, hovering specific elements)?
5. Does moving cursor out of Firefox and back in fix it?

**Possible Causes**:
- Race condition during cursor changes
- Focus-related timing issue
- Firefox-specific cursor handling
- Event propagation issue

**Investigation Notes**:
- DWM sets cursor with `CWCursor` flag on root window (dwm.c:1705)
- OXWM already does this correctly
- Event masks look correct (ENTER_WINDOW, STRUCTURE_NOTIFY, PROPERTY_CHANGE)
- Likely needs more investigation with reproducible steps

---

## Files Modified

### Core Changes
- `src/window_manager.rs`:
  - Added atoms: `wm_protocols`, `wm_delete_window`, `net_wm_state`, `net_wm_state_fullscreen`
  - Added `fullscreen_windows: HashSet<Window>` to track fullscreen state
  - Added `kill_client()` - graceful window closing
  - Added `send_event()` - send WM protocol messages
  - Added `set_window_fullscreen()` - per-window EWMH fullscreen
  - Added `is_transient_window()` - detect modal/dialog windows
  - Modified `ToggleFullScreen` handler - smart context-aware behavior
  - Added `ClientMessage` event handler - handle fullscreen requests
  - Modified `MapRequest` handler - auto-float transient windows
  - Modified `remove_window()` - clean up fullscreen state

### Config Support
- `src/config/lua.rs`:
  - Added `"ToggleWindowFullscreen"` to string_to_key_action()

- `src/keyboard/handlers.rs`:
  - Added `ToggleWindowFullscreen` enum variant

- `src/overlay/keybind.rs`:
  - Added description for `ToggleWindowFullscreen`

### Test Config
- `resources/test-config.lua`:
  - Uses smart `ToggleFullScreen` (`Mod+Shift+F`)

---

## How to Test on Bare Metal

```bash
cd ~/repos/oxwm-worktrees/bugfixes-floating-fullscreen
cargo build --release
cp target/release/oxwm ~/.local/bin/oxwm  # or wherever you install it
```

Then reload WM with `Mod+Shift+R`

---

## Next Steps

1. **Test all fixes on bare metal** (except cursor bug)
2. **Gather more info on cursor bug** when it happens
3. **If all tests pass**: Merge into `lua-config-test` branch
4. **If issues found**: Iterate on fixes

---

## Notes

- `ToggleWindowFullscreen` action exists but is unused in config (smart `ToggleFullScreen` covers it)
- May want to add `strum` crate to auto-generate enum string parsing (see conversation notes)
- Consider dropping `ron` as dependency eventually
