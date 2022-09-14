use cc::Build;

fn main() {
    println!("cargo:rerun-if-changed=csrc/ashmem.c");
    Build::new()
        .file("csrc/ashmem.c")
        .opt_level(2)
        .compile("ashmem");
    println!(
        "cargo:cargo:rustc-link-search=native={}",
        std::env::var("OUT_DIR").unwrap()
    );
    println!("cargo:rustc-link-lib=ashmem");
}
