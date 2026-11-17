use boot_layout::{BOOTLOADER, Image, emit_memory_x};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var_os("CARGO_FEATURE_DEFMT").is_some() {
        println!("cargo:rustc-link-arg=-Tdefmt.x");
    }
    emit_memory_x(Image::Bootloader(BOOTLOADER));
}
