use boot_layout::{HEADER_SIZE, SLOT_COUNT, SLOTS, SlotError, SlotHeader, validate};

type Slots = [Result<SlotHeader, SlotError>; SLOT_COUNT];

/// Scan the application slots and log debug info
pub fn scan_slots() -> Slots {
    core::array::from_fn(|index| {
        let slot = SLOTS[index];
        let r = validate(slot);
        match &r {
            Ok(header) => {
                let (major, minor, patch) = header.version_parts();
                let size = header.image_size + HEADER_SIZE;
                let permille = (size * 1000) / slot.region.length;
                debug!(
                    "slot {}: valid '{=str}' v{}.{}.{} ({} B of {} B ({}.{} %))",
                    index,
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
            Err(err) => debug!("slot {}: {}", index, err),
        }
        r
    })
}

/// Returns the index of the defualt slot to boot into
/// TODO: to be replaced by persistent storage rather than first valid slot
pub fn default_slot(slots: &Slots) -> Option<usize> {
    slots.iter().position(|slot| slot.is_ok())
}
