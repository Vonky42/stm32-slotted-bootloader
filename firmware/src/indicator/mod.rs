use crate::time::Instant;
use boot_layout::{SlotError, SlotHeader};

pub trait Indicator {
    fn show_selection(&mut self, slot: u8, info: &Result<SlotHeader, SlotError>, now: Instant);
    fn show_boot_pending(&mut self, now: Instant);
    fn _show_updating(&mut self, now: Instant); // once UART is implemented
    fn show_error(&mut self, now: Instant);
    fn tick(&mut self, now: Instant);
}

pub mod led;
