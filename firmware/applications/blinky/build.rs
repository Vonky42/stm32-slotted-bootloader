use boot_layout::{FLASH_SLOTS, HEADER_SIZE, RAM};
use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let slot = &FLASH_SLOTS[0]; // TODO: hardcoded for now

    // File content
    let memory_x = format!(
        "MEMORY\n{{\n  FLASH : ORIGIN = {:#010x}, LENGTH = {:#x}\n  RAM   : ORIGIN = {:#010x}, LENGTH = {:#x}\n}}\n",
        slot.base + HEADER_SIZE,
        slot.length - HEADER_SIZE,
        RAM.base,
        RAM.length,
    );

    // Write the memory.x file
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("memory.x"), memory_x).unwrap();
    println!("cargo:rustc-link-search={}", out_dir.display());
}
