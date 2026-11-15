fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    boot_layout::emit_bootloader_memory_x();
}