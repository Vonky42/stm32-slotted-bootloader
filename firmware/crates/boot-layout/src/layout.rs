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

// Constants
const HEADER_SIZE: u32 = 0x200;
const K: u32 = 1024;
// May be specific to the STM32F411RE
const FLASH_ORIGIN: u32 = 0x0800_0000;
const BOOTLOADER_LENGTH: u32 = 32 * K; // Sectors 0 and 1
const RAM_ORIGIN: u32 = 0x20000000;
const RAM_LENGTH: u32 = 128 * K;

#[derive(Clone, Copy, Debug)]
pub struct Region {
    pub base: u32,
    pub length: u32,
}

impl Region {
    pub const fn new(base: u32, length: u32) -> Self {
        Self { base, length }
    }

    pub const fn split_at(self, at: u32) -> (Region, Region) {
        assert!(at <= self.length);
        (
            Region::new(self.base, at),
            Region::new(self.base + at, self.length - at),
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Slot {
    pub index: u8,
    pub region: Region,
}

impl Slot {
    pub const fn new(index: u8, region: Region) -> Self {
        Self { index, region }
    }

    pub const fn header_region(self) -> Region {
        self.region.split_at(HEADER_SIZE).0
    }
    pub const fn application_region(self) -> Region {
        self.region.split_at(HEADER_SIZE).1
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Image {
    Bootloader(Region),
    Application(Slot),
}

/// Indexed application `Slot` flash memory Regions
pub const SLOTS: [Slot; 4] = [
    Slot::new(0, Region::new(0x0801_0000, 64 * K)),
    Slot::new(1, Region::new(0x0802_0000, 128 * K)),
    Slot::new(2, Region::new(0x0804_0000, 128 * K)),
    Slot::new(3, Region::new(0x0806_0000, 128 * K)),
];

/// The bootloader flash memory Region
pub const BOOTLOADER: Region = Region::new(FLASH_ORIGIN, BOOTLOADER_LENGTH);

/// The RAM memory Region
pub const RAM: Region = Region::new(RAM_ORIGIN, RAM_LENGTH);
