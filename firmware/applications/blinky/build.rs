use boot_layout::{Image, SLOTS, emit_memory_x};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var_os("CARGO_FEATURE_DEFMT").is_some() {
        println!("cargo:rustc-link-arg=-Tdefmt.x");
    }
    println!("cargo:rustc-env=BOOT_SLOT=0"); // hardcoded for now
    emit_memory_x(Image::Application(SLOTS[0])); // hardcoded for now
}
