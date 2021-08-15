#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]
#![allow(improper_ctypes)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

impl Default for xcb_xim_im_callback {
    fn default() -> Self {
        Self {
            set_event_mask: None,
            forward_event: None,
            commit_string: None,
            geometry: None,
            preedit_start: None,
            preedit_draw: None,
            preedit_caret: None,
            preedit_done: None,
            status_start: None,
            status_draw_text: None,
            status_draw_bitmap: None,
            status_done: None,
            sync: None,
            disconnected: None,
        }
    }
}

