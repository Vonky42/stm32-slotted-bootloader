// Direct the linker to the memory.x file at firmware/ld/
fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-search={dir}/ld");
    println!("cargo:rerun-if-changed=ld/memory.x");
    println!("cargo:rerun-if-changed=build.rs");
}