use crate::probe;
use anyhow::Result;
use boot_core::{HEADER_SIZE, SLOTS, Slot, SlotError, SlotHeader, validate_header, validate_image};
use core::mem::size_of;
use probe_rs::{Core, MemoryInterface};

enum ScanError {
    Slot(SlotError),
    Probe(probe_rs::Error),
}
impl From<SlotError> for ScanError {
    fn from(e: SlotError) -> Self {
        ScanError::Slot(e)
    }
}
impl From<probe_rs::Error> for ScanError {
    fn from(e: probe_rs::Error) -> Self {
        ScanError::Probe(e)
    }
}

/// Scan the bootloaders flash slots using probe-rs
pub fn scan_slots(under_reset: bool) -> Result<()> {
    let mut session = if under_reset {
        probe::attach_under_reset()?
    } else {
        probe::attach()?
    };

    let mut core = session.core(0)?;

    println!();
    for slot in SLOTS {
        match scan_slot(&mut core, slot) {
            Ok(header) => {
                let (major, minor, patch) = header.version_parts();
                let size = header.image_size + HEADER_SIZE;
                let permille = (size * 1000) / slot.region.length;
                println!(
                    "slot {}: valid '{}' v{}.{}.{} ({} B of {} B ({}.{} %))",
                    slot.index,
                    header.name(),
                    major,
                    minor,
                    patch,
                    size,
                    slot.region.length,
                    permille / 10,
                    permille % 10
                );
            }
            Err(ScanError::Slot(e)) => println!("slot {}: {e:?}", slot.index),
            Err(ScanError::Probe(e)) => return Err(e.into()),
        }
    }

    Ok(())
}

/// Scan a single flash sector `slot` using a probe-rs session core
fn scan_slot(core: &mut Core, slot: Slot) -> Result<SlotHeader, ScanError> {
    let mut header_bytes = [0u8; size_of::<SlotHeader>()];
    core.read(slot.region.base as u64, &mut header_bytes)?;

    let header = SlotHeader::from_bytes(&header_bytes);
    validate_header(slot, &header)?;

    let mut image = vec![0u8; header.image_size as usize];
    core.read(slot.application_region().base as u64, &mut image)?;
    validate_image(slot, &header, &image)?;

    Ok(header)
}
