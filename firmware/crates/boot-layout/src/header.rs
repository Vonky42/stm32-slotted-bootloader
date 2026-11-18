use crate::layout::Slot;

pub const ERASED: u32 = 0xFFFF_FFFF;
pub const SLOT_MAGIC: u32 = 0x099D_7DD5;
pub const NAME_LENGTH: usize = 63;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
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

    /// View the header at the start of `slot` in flash memory
    pub unsafe fn at(slot: &Slot) -> &'static SlotHeader {
        unsafe { &*(slot.header_region().base as *const SlotHeader) }
    }

    /// Application name as found in the header
    pub fn name(&self) -> &str {
        let len = self
            .name
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(NAME_LENGTH);
        core::str::from_utf8(&self.name[..len]).unwrap_or("<not utf8>")
    }

    /// Version parts (major, minor, patch) as found in the header
    pub fn version_parts(&self) -> (u8, u8, u8) {
        (
            ((self.version >> 16) as u8),
            ((self.version >> 8) as u8),
            (self.version as u8),
        )
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self as *const SlotHeader as *const u8, size_of::<Self>())
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
