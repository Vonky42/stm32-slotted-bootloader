fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var_os("CARGO_FEATURE_DEFMT").is_some() {
        println!("cargo:rustc-link-arg=-Tdefmt.x");
    }
    boot_layout::emit_application_memory_x(0);
}
