use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::*;
use x11rb::rust_connection::RustConnection;

fn main() -> Result<()> {
    let (connection, screen_number) = x11rb::connect(None)?;
    let root = connection.setup().roots[screen_number].root;
    let screen = &connection.setup().roots[screen_number];

    connection
        .change_window_attributes(
            root,
            &ChangeWindowAttributesAux::new().event_mask(
                EventMask::SUBSTRUCTURE_REDIRECT
                    | EventMask::SUBSTRUCTURE_NOTIFY
                    | EventMask::PROPERTY_CHANGE,
            ),
        )?
        .check()?;

    println!("oxwm started on display {}", screen_number);

    let mut window_count = 0;
    loop {
        let event = connection.wait_for_event()?;
        println!("event: {:?}", event);
        match event {
            Event::MapRequest(event) => {
                connection.map_window(event.window)?;
                let x_coordinate = if window_count == 0 {
                    0
                } else {
                    (screen.width_in_pixels / 2) as i32
                };
                connection.configure_window(
                    event.window,
                    &ConfigureWindowAux::new()
                        .x(x_coordinate)
                        .y(0)
                        .border_width(1)
                        .width((screen.width_in_pixels / 2) as u32)
                        .height(screen.height_in_pixels as u32),
                )?;
                window_count += 1;
                connection.set_input_focus(
                    InputFocus::POINTER_ROOT,
                    event.window,
                    x11rb::CURRENT_TIME,
                )?;
                connection.flush()?;
            }
            _ => {}
        }
    }
}
