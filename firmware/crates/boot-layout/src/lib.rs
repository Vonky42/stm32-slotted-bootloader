#![no_std]

const K: u32 = 1024;
// May be specific to the STM32F411RE
const RAM_ORIGIN: u32 = 0x20000000;
const RAM_LENGTH: u32 = 128;

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
pub const HEADER_SIZE: u32 = 0x200;
pub struct Slot { pub base: u32, pub length: u32 }
pub const FLASH_SLOTS: [Slot; 4] = [
    Slot { base: 0x0801_0000, length: 64  * K },    // Slot 0
    Slot { base: 0x0802_0000, length: 128 * K },    // Slot 1
    Slot { base: 0x0804_0000, length: 128 * K },    // Slot 2
    Slot { base: 0x0806_0000, length: 128 * K },    // Slot 3
];

// RAM memory layout
pub const RAM: Slot = Slot { base: RAM_ORIGIN, length: RAM_LENGTH * K };
