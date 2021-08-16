#[macro_use]
extern crate lazy_static;

use std::borrow::Cow;
use std::os::raw::{c_char, c_void};
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use clib::*;

mod clib;

type LogFn = dyn for<'a> FnMut(Cow<'a, str>) + Send;

lazy_static! {
    static ref LOGGER: Mutex<Option<Box<LogFn>>> = Mutex::default();
}

extern "C" {
    fn xcb_log_wrapper(msg: *const c_char, ...);
}

#[no_mangle]
fn rust_log(msg: *const c_char) {
    let msg = unsafe { std::ffi::CStr::from_ptr(msg) }.to_string_lossy();
    if let Some(logger) = LOGGER.lock().unwrap().as_mut() {
        logger(msg);
    }
}

extern "C" fn create_ic_callback(im: *mut xcb_xim_t, new_ic: xcb_xic_t, user_data: *mut c_void) {
    let ic = unsafe { &mut *(user_data as *mut Ic) };
    ic.ic = new_ic;
    unsafe {
        xcb_xim_set_ic_focus(im, new_ic);
    }
}

extern "C" fn open_callback(im: *mut xcb_xim_t, user_data: *mut c_void) {
    let ic = unsafe { &mut *(user_data as *mut Ic) };
    let input_style = _xcb_im_style_t_XCB_IM_PreeditPosition
        | _xcb_im_style_t_XCB_IM_StatusArea
        | _xcb_im_style_t_XCB_IM_PreeditCallbacks;
    let spot = xcb_point_t { x: 0, y: 0 };
    let w = &mut ic.win as *mut _;
    unsafe {
        let nested = xcb_xim_create_nested_list(
            im,
            XCB_XIM_XNSpotLocation,
            &spot,
            std::ptr::null_mut::<c_void>(),
        );
        xcb_xim_create_ic(
            im,
            Some(create_ic_callback),
            user_data,
            XCB_XIM_XNInputStyle,
            &input_style,
            XCB_XIM_XNClientWindow,
            w,
            XCB_XIM_XNFocusWindow,
            w,
            XCB_XIM_XNPreeditAttributes,
            &nested,
            std::ptr::null_mut::<c_void>(),
        );
        free(nested.data as _);
    }
}

unsafe fn xim_encoding_to_utf8(
    im: *mut xcb_xim_t,
    xim_str: *const c_char,
    length: usize,
) -> String {
    let mut buf: Vec<u8> = vec![];
    if xcb_xim_get_encoding(im) == _xcb_xim_encoding_t_XCB_XIM_UTF8_STRING {
        buf.extend(std::slice::from_raw_parts(
            xim_str as *const u8,
            length as usize,
        ));
    } else if xcb_xim_get_encoding(im) == _xcb_xim_encoding_t_XCB_XIM_COMPOUND_TEXT {
        let mut new_length = 0usize;
        let utf8 = xcb_compound_text_to_utf8(xim_str, length as usize, &mut new_length);
        if !utf8.is_null() {
            buf.extend(std::slice::from_raw_parts(utf8 as _, new_length));
            free(utf8 as _);
        }
    }
    String::from_utf8_lossy(&buf).into()
}

extern "C" fn commit_string_callback(
    im: *mut xcb_xim_t,
    _ic: xcb_xic_t,
    _flag: u32,
    input: *mut c_char,
    length: u32,
    _keysym: *mut u32,
    _n_keysym: usize,
    user_data: *mut c_void,
) {
    let input = unsafe { xim_encoding_to_utf8(im, input, length as usize) };
    let ime = unsafe { &mut *(user_data as *mut Ime) };
    let win = ime.ic.as_ref().unwrap().win;
    ime.callbacks.commit_string.as_mut().map(|f| f(win, &input));
}

extern "C" fn forward_event_callback(
    _im: *mut xcb_xim_t,
    _ic: xcb_xic_t,
    event: *mut xcb_key_press_event_t,
    user_data: *mut c_void,
) {
    let ptr = event as *const xcb::ffi::xcb_key_press_event_t;
    let event = xcb::KeyPressEvent { ptr: ptr as _ };
    let ime = unsafe { &mut *(user_data as *mut Ime) };
    ime.callbacks.forward_event.as_mut().map(|f| f(&event));
    std::mem::forget(event);
}

extern "C" fn preedit_start_callback(_im: *mut xcb_xim_t, _ic: xcb_xic_t, user_data: *mut c_void) {
    let ime = unsafe { &mut *(user_data as *mut Ime) };
    let win = ime.ic.as_ref().unwrap().win;
    ime.callbacks.preedit_start.as_mut().map(|f| f(win));
}

extern "C" fn preedit_draw_callback(
    im: *mut xcb_xim_t,
    _ic: xcb_xic_t,
    frame: *mut xcb_im_preedit_draw_fr_t,
    user_data: *mut c_void,
) {
    let frame = unsafe { &*frame };
    let preedit_info = PreeditInfo { inner: frame, im };
    let ime = unsafe { &mut *(user_data as *mut Ime) };
    let win = ime.ic.as_ref().unwrap().win;
    ime.callbacks
        .preedit_draw
        .as_mut()
        .map(|f| f(win, preedit_info));
}

extern "C" fn preedit_done_callback(_im: *mut xcb_xim_t, _ic: xcb_xic_t, user_data: *mut c_void) {
    let ime = unsafe { &mut *(user_data as *mut Ime) };
    let win = ime.ic.as_ref().unwrap().win;
    ime.callbacks.preedit_done.as_mut().map(|f| f(win));
}

type StringCB = dyn for<'a> FnMut(u32, &'a str);
type KeyPressCB = dyn for<'a> FnMut(&'a xcb::KeyPressEvent);
type PreeditDrawCB = dyn for<'a> FnMut(u32, PreeditInfo<'a>);
type NotifyCB = dyn FnMut(u32);

#[derive(Default)]
struct Callbacks {
    commit_string: Option<Box<StringCB>>,
    forward_event: Option<Box<KeyPressCB>>,
    preedit_start: Option<Box<NotifyCB>>,
    preedit_draw: Option<Box<PreeditDrawCB>>,
    preedit_done: Option<Box<NotifyCB>>,
}

#[derive(Debug, Clone)]
struct Ic {
    win: u32,
    ic: xcb_xic_t,
}

pub struct PreeditInfo<'a> {
    im: *mut xcb_xim_t,
    inner: &'a xcb_im_preedit_draw_fr_t,
}

impl<'a> PreeditInfo<'a> {
    pub fn test(&self) {
        self.inner.chg_first;
    }
    pub fn status(&self) -> u32 {
        self.inner.status
    }
    pub fn caret(&self) -> u32 {
        self.inner.caret
    }
    pub fn chg_first(&self) -> u32 {
        self.inner.chg_first
    }
    pub fn chg_length(&self) -> u32 {
        self.inner.chg_length
    }
    pub fn text(&self) -> String {
        unsafe {
            xim_encoding_to_utf8(
                self.im,
                self.inner.preedit_string as _,
                self.inner.length_of_preedit_string as usize,
            )
        }
    }
}

impl<'a> std::fmt::Debug for PreeditInfo<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreeditInfo")
            .field("status", &self.status())
            .field("caret", &self.caret())
            .field("chg_first", &self.chg_first())
            .field("chg_length", &self.chg_length())
            .field("text", &self.text());
        Ok(())
    }
}

pub struct Ime {
    conn: Option<Arc<xcb::Connection>>,
    im: *mut xcb_xim_t,
    ic: Option<Ic>,
    callbacks: Callbacks,
}

impl Ime {
    pub fn set_logger<F>(f: F)
    where
        F: for<'a> FnMut(Cow<'a, str>) + Send + 'static,
    {
        LOGGER.lock().unwrap().replace(Box::new(f));
    }

    pub fn new(
        conn: Arc<xcb::Connection>,
        screen_id: i32,
        im_name: Option<String>,
    ) -> Pin<Box<Self>> {
        let mut res = unsafe { Self::unsafe_new(&conn, screen_id, im_name) };
        res.conn = Some(conn);
        res
    }

    pub unsafe fn unsafe_new(
        conn: &xcb::Connection,
        screen_id: i32,
        im_name: Option<String>,
    ) -> Pin<Box<Self>> {
        xcb_compound_text_init();
        let im = xcb_xim_create(
            conn.get_raw_conn() as _,
            screen_id,
            im_name.map_or(std::ptr::null(), |name| name.as_ptr() as _),
        );
        let mut res = Box::pin(Self {
            conn: None,
            im,
            ic: None,
            callbacks: Callbacks::default(),
        });
        let callbacks = xcb_xim_im_callback {
            commit_string: Some(commit_string_callback),
            forward_event: Some(forward_event_callback),
            preedit_start: Some(preedit_start_callback),
            preedit_draw: Some(preedit_draw_callback),
            preedit_done: Some(preedit_done_callback),
            ..Default::default()
        };
        let data: *mut Self = res.as_mut().get_mut();
        xcb_xim_set_im_callback(im, &callbacks, data as _);
        xcb_xim_set_log_handler(im, Some(xcb_log_wrapper));
        xcb_xim_set_use_compound_text(im, true);
        xcb_xim_set_use_utf8_string(im, true);
        res
    }

    fn try_open_ic(&mut self, win: u32) {
        if self.ic.is_some() {
            return;
        }
        let ic = self.ic.insert(Ic { win, ic: 0 });
        let data: *mut Ic = ic;
        if !unsafe { xcb_xim_open(self.im, Some(open_callback), true, data as _) } {
            self.ic.take();
            return;
        }
    }

    pub fn process_event(&mut self, event: &xcb::GenericEvent) -> bool {
        if !unsafe { xcb_xim_filter_event(self.im, event.ptr as _) } {
            let mask = event.response_type() & !0x80;
            if (mask == xcb::ffi::XCB_KEY_PRESS) || (mask == xcb::ffi::XCB_KEY_RELEASE) {
                let win = if mask == xcb::ffi::XCB_KEY_PRESS {
                    unsafe { &*(event.ptr as *const xcb::ffi::xcb_key_press_event_t) }.event
                } else {
                    unsafe { &*(event.ptr as *const xcb::ffi::xcb_key_release_event_t) }.event
                };
                if let Some(ic) = self.ic.as_mut() {
                    if ic.ic == 0 {
                        return false;
                    }
                    unsafe {
                        xcb_xim_forward_event(self.im, ic.ic, event.ptr as _);
                    }
                    return true;
                } else {
                    self.try_open_ic(win);
                }
            }
        }
        false
    }

    pub fn update_pos(&mut self, win: u32, x: i16, y: i16) -> bool {
        match &mut self.ic {
            Some(ic) if ic.ic != 0 => {
                let spot = xcb_point_t { x, y };
                let nested = unsafe {
                    xcb_xim_create_nested_list(
                        self.im,
                        XCB_XIM_XNSpotLocation,
                        &spot,
                        std::ptr::null_mut::<c_void>(),
                    )
                };
                if win != ic.win {
                    ic.win = win;
                    let w = &mut ic.win as *mut _;
                    unsafe {
                        xcb_xim_set_ic_values(
                            self.im,
                            ic.ic,
                            None,
                            std::ptr::null_mut::<c_void>(),
                            XCB_XIM_XNClientWindow,
                            w,
                            XCB_XIM_XNFocusWindow,
                            w,
                            XCB_XIM_XNPreeditAttributes,
                            &nested,
                            std::ptr::null_mut::<c_void>(),
                        );
                    }
                } else {
                    unsafe {
                        xcb_xim_set_ic_values(
                            self.im,
                            ic.ic,
                            None,
                            std::ptr::null_mut::<c_void>(),
                            XCB_XIM_XNPreeditAttributes,
                            &nested,
                            std::ptr::null_mut::<c_void>(),
                        );
                    }
                }
                unsafe { free(nested.data as _) };
                true
            }
            _ => false,
        }
    }

    pub fn set_commit_string_cb<F>(&mut self, f: F)
    where
        F: for<'a> FnMut(u32, &'a str) + 'static,
    {
        self.callbacks.commit_string = Some(Box::new(f));
    }

    pub fn set_forward_event_cb<F>(&mut self, f: F)
    where
        F: for<'a> FnMut(&'a xcb::KeyPressEvent) + 'static,
    {
        self.callbacks.forward_event = Some(Box::new(f));
    }

    pub fn set_preedit_start_cb<F>(&mut self, f: F)
    where
        F: FnMut(u32) + 'static,
    {
        self.callbacks.preedit_start = Some(Box::new(f));
    }

    pub fn set_preedit_draw_cb<F>(&mut self, f: F)
    where
        F: for<'a> FnMut(u32, PreeditInfo<'a>) + 'static,
    {
        self.callbacks.preedit_draw = Some(Box::new(f));
    }

    pub fn set_preedit_done_cb<F>(&mut self, f: F)
    where
        F: FnMut(u32) + 'static,
    {
        self.callbacks.preedit_done = Some(Box::new(f));
    }
}

impl Drop for Ime {
    fn drop(&mut self) {
        unsafe {
            xcb_xim_close(self.im);
            xcb_xim_destroy(self.im);
        }
    }
}
