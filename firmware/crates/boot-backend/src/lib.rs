#![no_std]

#[macro_use]
mod fmt;

pub mod boot;
pub mod clock;
pub mod slots;

pub use boot::jump_to_slot;
pub use slots::{default_slot, scan_slots};
