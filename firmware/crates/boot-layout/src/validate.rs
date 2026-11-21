use crate::{ERASED, RAM, SLOT_MAGIC, Slot, SlotHeader, header_crc, image_crc};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SlotError {
    Empty,
    BadMagic(u32),
    BadHeaderCrc { stored: u32, computed: u32 },
    WrongSlot { built_for: u8, found_in: u8 },
    BadSize(u32),
    BadImageCrc { stored: u32, computed: u32 },
    BadVectors { msp: u32, rv: u32 },
}

fn validate_header(slot: Slot, header: &SlotHeader) -> Result<SlotHeader, SlotError> {
    use SlotError::*;

    // Magic
    match header.magic {
        ERASED => return Err(Empty),
        SLOT_MAGIC => {}
        other => return Err(BadMagic(other)),
    };

    // Header CRC
    let computed: u32 = header_crc(header.as_bytes());
    if computed != header.header_crc {
        return Err(BadHeaderCrc {
            stored: header.header_crc,
            computed,
        });
    }

    // Valid header CRC: the remaining header fields can now be trusted

    // Built for this slot?
    if header.slot_number != slot.index {
        return Err(WrongSlot {
            built_for: header.slot_number,
            found_in: slot.index,
        });
    }

    Ok(*header)
}

fn validate_image(slot: Slot, header: &SlotHeader, image: &[u8]) -> Result<SlotHeader, SlotError> {
    use SlotError::*;
    let application = slot.application_region();

    // Image size
    if image.len() != header.image_size as usize {
        return Err(BadSize(header.image_size));
    }

    // Image CRC
    let computed: u32 = image_crc(image);
    if computed != header.image_crc {
        return Err(BadImageCrc {
            stored: header.image_crc,
            computed,
        });
    }

    // Vector table
    let msp: u32 = unsafe { (application.base as *const u32).read() };
    let rv: u32 = unsafe { (application.base as *const u32).add(1).read() };
    let msp_ok: bool = msp > RAM.base && msp <= RAM.base + RAM.length && msp % 8 == 0;
    let reset_target: u32 = rv & !1;
    let rv_ok: bool = rv & 0b1 > 0
        && reset_target >= application.base
        && reset_target < application.base + header.image_size;
    if !(msp_ok && rv_ok) {
        return Err(BadVectors { msp, rv });
    }

    Ok(*header)
}

pub fn validate(slot: Slot) -> Result<SlotHeader, SlotError> {
    let header = unsafe { *SlotHeader::at(&slot) };
    validate_header(slot, &header)?;
    let image = unsafe {
        core::slice::from_raw_parts(
            slot.application_region().base as *const u8,
            header.image_size as usize,
        )
    };
    validate_image(slot, &header, image)?;
    Ok(header)
}
