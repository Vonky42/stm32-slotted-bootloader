use crate::layout::{NAME_LENGTH, SLOT_MAGIC};

#[repr(C)]
pub struct SlotHeader {
    pub magic: u32,
    pub slot_number: u8,
    pub name: [u8; NAME_LENGTH],
    pub version: u32,
    pub image_size: u32,
    pub image_crc: u32,
    pub header_crc: u32,
}

impl SlotHeader {
    /// Provides the unpatched SlotHeader filled with compile-time values. `image_size`, `image_crc`, and `header_crc` are not set yet
    pub const fn unpatched(slot_number: u8, name: &str, version: u32) -> Self {
        Self {
            magic: SLOT_MAGIC,
            slot_number,
            name: name_bytes(name),
            version,
            image_size: 0,
            image_crc: 0,
            header_crc: 0,
        }
    }
}

/// Returns the bytes of the name string `s`. Truncated at 63 bytes
pub const fn name_bytes(s: &str) -> [u8; NAME_LENGTH] {
    let b = s.as_bytes();
    let mut out = [0u8; NAME_LENGTH];
    let mut i = 0;
    while i < b.len() && i < NAME_LENGTH - 1 {
        out[i] = b[i];
        i += 1;
    }
    out
}

/// Parse a u8 from a string
pub const fn parse_u8(s: &str) -> u8 {
    match u8::from_str_radix(s, 10) {
        Ok(v) => v,
        Err(_) => panic!("not a u8"),
    }
}

/// Gives the pack version of the current application as a u32: (Major << 16 | Minor << 8 | Patch)
pub const fn pack_version(major: &str, minor: &str, patch: &str) -> u32 {
    (parse_u8(major) as u32) << 16 | (parse_u8(minor) as u32) << 8 | parse_u8(patch) as u32
}

#[macro_export]
/// Places the boot slots header struct static in the `.slot_header` linker section
macro_rules! slot_header {
    () => {
        #[used]
        #[unsafe(link_section = ".slot_header")]
        static SLOT_HEADER: $crate::SlotHeader = $crate::SlotHeader::unpatched(
            $crate::parse_u8(env!("BOOT_SLOT")),
            env!("CARGO_PKG_NAME"),
            $crate::pack_version(
                env!("CARGO_PKG_VERSION_MAJOR"),
                env!("CARGO_PKG_VERSION_MINOR"),
                env!("CARGO_PKG_VERSION_PATCH"),
            ),
        );
    };
}
