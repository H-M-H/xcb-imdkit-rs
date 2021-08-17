# xcb-imdkit-rs

This library is a wrapper around [xcb-imdkit](https://github.com/fcitx/xcb-imdkit), providing an IME
client.

[xcb-imdkit](https://github.com/fcitx/xcb-imdkit) provides a partial implementation of the [X11
Input Method Protocol](https://www.x.org/releases/current/doc/libX11/XIM/xim.html) using
[XCB](https://xcb.freedesktop.org/). This wrapper library provides the most essential functionality
of said library as simply as possible.

To get started quickly, consult the examples folder.

## Dependencies

This crate depends on `xcb` and `xcb-util`. `xcb-imdkit` is built from source, which requires git,
cmake and a C compiler, and statically linked by default. If you prefer to use the system version of
`xcb-imdkit` (make sure you have at least version 1.0.3 installed), you can specify `use-system-lib`
as feature flag.

## Using xcb-imdkit-rs

```toml
[dependencies]
xcb-imdkit = "0.1"
# xcb-imdkit = { version = "0.1", features = ["use-system-lib"] }
```

## License

Just as the original library this is licensed under the LGPLv2.1, see LICENSE for the full text.
