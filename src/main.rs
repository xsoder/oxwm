use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

fn main() -> Result<()> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let root = conn.setup().roots[screen_num].root;

    // Ask to manage the root window (like dwm does)
    conn.change_window_attributes(
        root,
        &ChangeWindowAttributesAux::new().event_mask(
            EventMask::SUBSTRUCTURE_REDIRECT
                | EventMask::SUBSTRUCTURE_NOTIFY
                | EventMask::PROPERTY_CHANGE,
        ),
    )?.check()?; // will error if another WM is already running

    println!("oxwm started on display {}", screen_num);

    loop {
        let event = conn.wait_for_event()?;
        println!("event: {:?}", event);
    }
}

