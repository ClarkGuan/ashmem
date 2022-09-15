use cc::Build;
use std::env;

fn main() {
    if let Ok(ref os) = env::var("CARGO_CFG_TARGET_OS") {
        if os != "android" {
            return;
        }
    }

    println!("cargo:rerun-if-changed=csrc/ashmem.c");
    Build::new()
        .file("csrc/ashmem.c")
        .opt_level(2)
        .compile("ashmem");
    println!(
        "cargo:cargo:rustc-link-search=native={}",
        env::var("OUT_DIR").unwrap()
    );
    println!("cargo:rustc-link-lib=ashmem");
}
