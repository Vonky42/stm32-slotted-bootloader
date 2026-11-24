#![no_std]
#![no_main]

#[macro_use]
mod fmt;
mod board;

use crate::board::Board;
use boot_backend::{default_slot, jump_to_slot, scan_slots};
use boot_core::Duration;
use boot_frontend::{Action, BootMenu, LedIndicator, SingleButton};
use cortex_m_rt::entry;

// Debug logging
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};
#[cfg(not(feature = "defmt"))]
#[inline(never)]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// How long the bootmenu allows inactivity before booting into the default slot / selected slot (TODO)
pub const MENU_TIMEOUT: Option<Duration> = Some(Duration::from_millis(30_000)); // 30 seconds

#[entry]
fn main() -> ! {
    // Initialized peripherals
    let mut board = Board::init();

    // Scan the flash slots
    let slots = scan_slots();

    // Holding the user button and the reset button enters the boot menu
    if !board.button_pressed() {
        if let Some(slot) = default_slot(&slots) {
            jump_to_slot(slot);
        }
    }

    let now = boot_backend::clock::now();

    // Menu Indicator
    let mut led = LedIndicator::new(board.ld2, now);

    // Input
    let mut input = SingleButton::new(board.button, now);

    // Start the bootloader menu
    let Action::Boot(slot_to_boot) = BootMenu::run(
        slots,
        &mut input,
        &mut led,
        MENU_TIMEOUT,
        boot_backend::clock::now,
    );
    jump_to_slot(slot_to_boot);
}
