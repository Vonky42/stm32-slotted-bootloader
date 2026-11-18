use crate::crc::{HEADER_CRC_PAYLOAD, header_crc, image_crc};
use crate::header::SlotHeader;
use core::mem::offset_of;

pub fn patch(header: &mut [u8], application: &[u8]) {
    let mut offset = offset_of!(SlotHeader, image_size);
    // Patch application image size
    header[offset..offset + 4].copy_from_slice(&(application.len() as u32).to_le_bytes());
    // Patch application image CRC
    offset = offset_of!(SlotHeader, image_crc);
    header[offset..offset + 4].copy_from_slice(&(image_crc(application)).to_le_bytes());
    let header_crc = header_crc(header);
    header[HEADER_CRC_PAYLOAD..HEADER_CRC_PAYLOAD + 4].copy_from_slice(&(header_crc).to_le_bytes());
}
