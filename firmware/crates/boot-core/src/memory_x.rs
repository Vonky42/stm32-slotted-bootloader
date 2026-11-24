use crate::layout::{Image, RAM, Region, SLOTS};
use std::fmt::Write as _;

const HEADER_SECTION: &str = "
SECTIONS
{
  .slot_header :
  {
    KEEP(*(.slot_header))
  } > HEADER
} INSERT BEFORE .vector_table;
";

fn region_line(out: &mut String, name: &str, r: Region) {
    writeln!(
        out,
        "  {name} : ORIGIN = {:#x}, LENGTH = {:#x}",
        r.base, r.length
    )
    .unwrap();
}

/// Emits the memory.x linker file
#[cfg(feature = "build")]
pub fn emit_memory_x(image: Image) {
    use std::{env, fs, path::PathBuf};
    let mut memory_x = String::from("MEMORY\n{\n");

    match image {
        Image::Bootloader(region) => {
            region_line(&mut memory_x, "FLASH", region);
            region_line(&mut memory_x, "RAM", RAM);
            memory_x.push_str("}\n");
        }
        Image::Application(slot) => {
            region_line(&mut memory_x, "HEADER", slot.header_region());
            region_line(&mut memory_x, "FLASH", slot.application_region());
            region_line(&mut memory_x, "RAM", RAM);
            memory_x.push_str("}\n");
            memory_x.push_str(HEADER_SECTION);
        }
    }

    // Write the memory.x file
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("memory.x"), memory_x).unwrap();
    println!("cargo:rustc-link-search={}", out_dir.display());
}

pub fn emit_application_memory_x_from_env() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var_os("CARGO_FEATURE_DEFMT").is_some() {
        println!("cargo:rustc-link-arg=-Tdefmt.x");
    }

    let slot: usize = match std::env::var("BOOT_SLOT") {
        Ok(s) => s.parse().expect("BOOT_SLOT must be a slot number"),
        Err(_) => {
            println!(
                "cargo:warning=BOOT_SLOT not set, building for slot 0 (flash with `cargo xtask flash app`)"
            );
            0
        }
    };

    emit_memory_x(Image::Application(SLOTS[slot]));
    println!("cargo:rustc-env=BOOT_SLOT={slot}");
}
