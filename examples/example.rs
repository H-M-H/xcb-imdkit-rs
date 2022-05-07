use std::sync::Arc;
use xcb::x::{Cw, EventMask, Window};
use xcb::Event;
use xcb_imdkit::{ImeClient, InputStyle};

fn create_window(connection: Arc<xcb::Connection>, screen: &xcb::x::Screen) -> Window {
    let wid = connection.generate_id();
    let mask = EventMask::KEY_PRESS
        | EventMask::KEY_RELEASE
        | EventMask::FOCUS_CHANGE
        | EventMask::VISIBILITY_CHANGE
        | EventMask::STRUCTURE_NOTIFY;
    connection.send_request(&xcb::x::CreateWindow {
        depth: xcb::x::COPY_FROM_PARENT as u8,
        wid,
        parent: screen.root(),
        x: 0,
        y: 0,
        width: 400,
        height: 400,
        border_width: 10,
        class: xcb::x::WindowClass::InputOutput,
        visual: screen.root_visual(),
        value_list: &[Cw::BackPixel(screen.white_pixel()), Cw::EventMask(mask)],
    });
    connection.send_request(&xcb::x::MapWindow { window: wid });
    connection.flush().unwrap();
    wid
}

fn main() {
    let (connection, screen_default_nbr) = xcb::Connection::connect(None).unwrap();
    let connection = Arc::new(connection);
    let screen = connection
        .get_setup()
        .roots()
        .nth(screen_default_nbr as usize)
        .unwrap();

    ImeClient::set_logger(|msg| println!("Log: {}", msg));
    let mut ime = ImeClient::new(
        connection.clone(),
        screen_default_nbr,
        InputStyle::PREEDIT_CALLBACKS,
        None,
    );
    ime.set_commit_string_cb(|win, input| println!("Win {:?}, got: {}", win, input));
    ime.set_forward_event_cb(|win, e| {
        eprintln!("win={:?} {:?}", win, e);
    });
    ime.set_preedit_draw_cb(|win, info| {
        dbg!(win, info);
    });

    let mut wins = vec![];
    for _ in 0..3 {
        wins.push(create_window(connection.clone(), &screen));
    }

    let mut focus_win = wins[0];
    let mut n = 0;
    loop {
        let event = dbg!(connection.wait_for_event().unwrap());
        match &event {
            Event::X(xcb::x::Event::FocusIn(event)) => {
                focus_win = event.event();
                ime.update_pos(focus_win, 0, 0);
            }
            Event::X(xcb::x::Event::ConfigureNotify(_)) => {
                ime.update_pos(focus_win, 0, 0);
            }
            _ => {}
        }

        println!(">>>>{}>>>>", n);
        ime.process_event(&event);
        println!("<<<<{}<<<<", n);
        n += 1;
    }
}
