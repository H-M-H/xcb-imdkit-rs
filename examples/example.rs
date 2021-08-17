use std::sync::Arc;
use xcb_imdkit::ImeClient;

fn create_window(connection: Arc<xcb::Connection>, screen: &xcb::Screen) -> u32 {
    let w = connection.generate_id();
    let mask = xcb::EVENT_MASK_KEY_PRESS
        | xcb::EVENT_MASK_KEY_RELEASE
        | xcb::EVENT_MASK_FOCUS_CHANGE
        | xcb::EVENT_MASK_VISIBILITY_CHANGE
        | xcb::EVENT_MASK_STRUCTURE_NOTIFY;
    let values = [
        (xcb::CW_BACK_PIXEL, screen.white_pixel()),
        (xcb::CW_EVENT_MASK, mask),
    ];
    xcb::create_window(
        &connection,
        xcb::COPY_FROM_PARENT as u8,
        w,
        screen.root(),
        0,
        0,
        400,
        400,
        10,
        xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
        screen.root_visual(),
        &values,
    );
    xcb::map_window(&connection, w);
    unsafe {
        xcb::ffi::xcb_flush(connection.get_raw_conn());
    }
    w
}

fn main() {
    let (connection, screen_default_nbr) = xcb::Connection::connect(None).unwrap();
    let connection = Arc::new(connection);
    let screen = connection
        .get_setup()
        .roots()
        .nth(screen_default_nbr as usize)
        .unwrap();

    ImeClient::set_logger(|msg| print!("Log: {}", msg));
    let mut ime = ImeClient::new(connection.clone(), screen_default_nbr, None);
    ime.set_commit_string_cb(|win, input| println!("Win {}, got: {}", win, input));
    ime.set_forward_event_cb(|win, e| {
        dbg!(
            win,
            e.response_type(),
            e.detail(),
            e.time(),
            e.root(),
            e.event(),
            e.child(),
            e.root_x(),
            e.root_y(),
            e.event_x(),
            e.event_y(),
            e.state(),
            e.same_screen(),
        );
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
        let event = connection.wait_for_event();
        if event.is_none() {
            break;
        }
        let event = event.unwrap();
        dbg!(event.response_type());

        let event_type = event.response_type() & !0x80;
        if xcb::FOCUS_IN == event_type {
            let event: &xcb::FocusInEvent = unsafe { xcb::cast_event(&event) };
            focus_win = event.event();
            ime.update_pos(focus_win, 0, 0);
        }

        if xcb::CONFIGURE_NOTIFY == event_type {
            ime.update_pos(focus_win, 0, 0);
        }

        println!(">>>>{}>>>>", n);
        ime.process_event(&event);
        println!("<<<<{}<<<<", n);
        n += 1;
    }
}
