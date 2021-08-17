use std::env;

const XCB_IMDKIT_SRC: &[&str] = &[
    "parser.c",
    "ximproto.c",
    // "imdkit.c", // currently unused as this crate only implements the client
    "protocolhandler.c",
    "message.c",
    "common.c",
    "imclient.c",
    "clientprotocolhandler.c",
    "encoding.c",
    "xlibi18n/lcCT.c",
    "xlibi18n/lcUTF8.c",
    "xlibi18n/lcCharSet.c",
];

fn main() {
    let use_system_lib = env::var("CARGO_FEATURE_USE_SYSTEM_LIB").is_ok();

    println!("cargo:rerun-if-changed=deps/build.sh");
    println!("cargo:rerun-if-changed=xcb-imdkit.h");

    println!("cargo:rustc-link-lib=xcb");
    println!("cargo:rustc-link-lib=xcb-util");

    if use_system_lib {
        println!("cargo:rustc-link-lib=xcb-imdkit");
        let xcb_imdkit = match pkg_config::Config::new()
            .atleast_version("1.0.3")
            .probe("xcb-imdkit")
        {
            Ok(l) => l,
            Err(err) => {
                println!(
                    "cargo:warning=Could find NO suitable version of xcb-imdkit: {}",
                    err
                );
                std::process::exit(1);
            }
        };
        for path in xcb_imdkit.link_paths {
            println!("cargo:rustc-link-search={}", path.to_string_lossy());
        }
    } else {
        if !std::path::Path::new("deps/xcb-imdkit/.git").exists() {
            if !std::process::Command::new("git")
                .args(&["submodule", "update", "--init"])
                .status()
                .expect("Failed to invoke git to init submodule.")
                .success()
            {
                panic!("Initializing xcb-imdkit submodule failed!");
            }
        }

        let mut xcb_imdkit_build = cc::Build::new();
        xcb_imdkit_build.warnings(false);
        xcb_imdkit_build.includes(&[
            "deps/xcb-imdkit/uthash",
            "deps/xcb-imdkit/src",
            "deps/xcb-imdkit-generated-headers",
        ]);
        for p in XCB_IMDKIT_SRC {
            xcb_imdkit_build.file(format!("deps/xcb-imdkit/src/{}", p));
        }
        xcb_imdkit_build.compile("xcb-imdkit");
    }

    println!("cargo:rerun-if-changed=logging.c");
    cc::Build::new().file("logging.c").compile("logging");
}
