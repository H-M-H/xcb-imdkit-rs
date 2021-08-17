extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let use_system_lib = env::var("CARGO_FEATURE_USE_SYSTEM_LIB").is_ok();
    let mut include_paths: Vec<String> = vec![];
    let mut link_paths: Vec<String> = vec![];

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
        include_paths = xcb_imdkit
            .include_paths
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        link_paths = xcb_imdkit
            .link_paths
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
    } else {
        println!("cargo:rustc-link-lib=static=xcb-imdkit");
        if !std::process::Command::new("sh")
            .arg("build.sh")
            .current_dir("deps")
            .status()
            .expect("Failed to execut deps/build.sh")
            .success()
        {
            panic!("Failed to build xcb-imdkit C library.");
        }
        include_paths.push("deps/dist/include".into());
        link_paths.push("deps/dist/lib".into())
    }

    for path in link_paths {
        println!("cargo:rustc-link-search={}", path);
    }

    println!("cargo:rerun-if-changed=logging.c");
    cc::Build::new().file("logging.c").compile("logging");

    let white_list =
        "(xcb|XCB)_(xim|XIM|im|xic)_.*|xcb_compound_text.*|xcb_utf8_to_compound_text|free";

    let bindings = bindgen::Builder::default()
        .clang_args(include_paths.iter().map(|p| format!("-I{}", p)))
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
