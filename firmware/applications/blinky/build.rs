fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    boot_layout::emit_application_memory_x(0);
}
