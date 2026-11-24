use crate::header::SlotHeader;
use core::mem::offset_of;
use crc::{CRC_32_ISO_HDLC, Crc};

const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
pub const HEADER_CRC_PAYLOAD: usize = offset_of!(SlotHeader, header_crc);

pub fn crc32(bytes: &[u8]) -> u32 {
    CRC32.checksum(bytes)
}

pub fn header_crc(bytes: &[u8]) -> u32 {
    crc32(&bytes[..HEADER_CRC_PAYLOAD])
}

pub fn image_crc(image: &[u8]) -> u32 {
    crc32(image)
}
