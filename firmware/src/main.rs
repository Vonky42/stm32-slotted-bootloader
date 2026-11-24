#![no_std]
#![no_main]

#[macro_use]
mod fmt;
mod board;
mod boot;
mod indicator;
mod input;
mod menu;
mod slots;
mod time;

use crate::{
    board::Board,
    boot::jump_to_slot,
    indicator::led::LedIndicator,
    input::single_button::SingleButton,
    menu::BootMenu,
    slots::{default_slot, scan_slots},
    time::{Duration, Instant},
};
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

    let now = Instant::now();

    // Menu Indicator
    let mut led = LedIndicator::new(board.ld2, now);

    // Input
    let mut input = SingleButton::new(board.button, now);

    // Start the bootloader menu
    BootMenu::run(slots, &mut input, &mut led);
}
