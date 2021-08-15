extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=dylib=xcb");
    println!("cargo:rustc-link-lib=dylib=xcb-util");
    println!("cargo:rustc-link-lib=dylib=xcb-imdkit");
    println!("cargo:rerun-if-changed=xcb-imdkit.h");
    let xcb_imdkit = pkg_config::probe_library("xcb-imdkit")
        .expect("Could not find xcb-imdkit using pkg_config, make sure it is properly installed.");

    println!("cargo:rerun-if-changed=logging.c");
    cc::Build::new().file("logging.c").compile("logging");

    let white_list = "(xcb|XCB)_(xim|XIM|im|xic)_.*|xcb_compound_text.*|xcb_utf8_to_compound_text|free";

    let bindings = bindgen::Builder::default()
        .clang_args(
            xcb_imdkit
                .include_paths
                .iter()
                .map(|p| format!("-I{}", p.to_string_lossy())),
        )
        .allowlist_function(white_list)
        .allowlist_var(white_list)
        .allowlist_type("_xcb_im_style_t")
        .size_t_is_usize(true)
        .impl_debug(true)
        .header("xcb-imdkit.h")
        .generate()
        .expect("Failed to generate bindings.");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
