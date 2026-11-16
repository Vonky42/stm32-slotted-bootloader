#![cfg_attr(not(feature = "build"), no_std)]

const K: u32 = 1024;
pub const HEADER_SIZE: u32 = 0x200;
// May be specific to the STM32F411RE
const FLASH_ORIGIN: u32 = 0x0800_0000;
const BOOTLOADER_LENGTH: u32 = 32 * K; // Sectors 0 and 1
const RAM_ORIGIN: u32 = 0x20000000;
const RAM_LENGTH: u32 = 128 * K;

// Flash memory layout split into slots
/* The STM32F411RE 512KB Flash memory is split into
   erase-sectors as follows:
-----------------------------------------------------
Sector 0 | 0x0800 0000 - 0x0800 3FFF | 16 Kbytes
Sector 1 | 0x0800 4000 - 0x0800 7FFF | 16 Kbytes
Sector 2 | 0x0800 8000 - 0x0800 BFFF | 16 Kbytes
Sector 3 | 0x0800 C000 - 0x0800 FFFF | 16 Kbytes
Sector 4 | 0x0801 0000 - 0x0801 FFFF | 64 Kbytes
Sector 5 | 0x0802 0000 - 0x0803 FFFF | 128 Kbytes
Sector 6 | 0x0804 0000 - 0x0805 FFFF | 128 Kbytes
Sector 7 | 0x0806 0000 - 0x0807 FFFF | 128 Kbytes
----------------------------------------------------- */

#[derive(Copy, Clone)]
pub struct Slot {
    pub base: u32,
    pub length: u32,
}
impl Slot {
    pub const fn add_vtor_header(self) -> Slot {
        Slot {
            base: self.base + HEADER_SIZE,
            length: self.length - HEADER_SIZE,
        }
    }
}
pub const FLASH_SLOTS: [Slot; 4] = [
    Slot {
        base: 0x0801_0000,
        length: 64 * K,
    }, // Slot 0
    Slot {
        base: 0x0802_0000,
        length: 128 * K,
    }, // Slot 1
    Slot {
        base: 0x0804_0000,
        length: 128 * K,
    }, // Slot 2
    Slot {
        base: 0x0806_0000,
        length: 128 * K,
    }, // Slot 3
];

// RAM memory layout
pub const RAM: Slot = Slot {
    base: RAM_ORIGIN,
    length: RAM_LENGTH,
};

// Bootloader memory layout
pub const BOOTLOADER: Slot = Slot {
    base: FLASH_ORIGIN,
    length: BOOTLOADER_LENGTH,
}; // Sectors 0 and 1
#[cfg(feature = "build")]
pub fn emit_bootloader_memory_x() {
    emit_memory_x(BOOTLOADER);
}

// Application memory layout
#[cfg(feature = "build")]
pub fn emit_application_memory_x(slot_number: usize) {
    let slot: Slot = FLASH_SLOTS[slot_number].add_vtor_header();
    emit_memory_x(slot);
}

#[cfg(feature = "build")]
fn emit_memory_x(flash: Slot) {
    use std::{env, fs, path::PathBuf};

    // File content
    let memory_x = format!(
        "MEMORY{{FLASH : ORIGIN = {:#010x}, LENGTH = {:#x} RAM : ORIGIN = {:#010x}, LENGTH = {:#x}}}",
        flash.base, flash.length, RAM.base, RAM.length,
    );

    // Write the memory.x file
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("memory.x"), memory_x).unwrap();
    println!("cargo:rustc-link-search={}", out_dir.display());
}
