#![no_std]
#![no_main]

#[macro_use]
mod fmt;

use boot_layout::Slot;
use cortex_m::asm::{self, bootload};
use cortex_m_rt::entry;
use stm32f4xx_hal as _;

// Debug logging
#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

/// Performs a bootload jump to the provided `slot`
fn jump_to_slot(slot: Slot) -> ! {
    unsafe {
        // Disable interrupts and clear pending interrupts
        debug!("Bootloader: disabling interrupts");
        cortex_m::interrupt::disable();
        let mut cp = cortex_m::Peripherals::steal();
        cp.SYST.disable_counter();
        cp.SYST.disable_interrupt();
        cp.SCB.icsr.write(1 << 25);

        debug!("Bootloader: clearing NVIC registers");
        for icer in cp.NVIC.icer.iter() {
            icer.write(0xFFFF_FFFF);
        }
        for icpr in cp.NVIC.icpr.iter() {
            icpr.write(0xFFFF_FFFF);
        }

        // Update Vector Table Offset Register (VTOR), re-enable interrupts, and perform jump
        debug!("Bootloader: Updating VTOR, enabling interrupts, and performing boot jump");
        let application = slot.add_vtor_header();
        cp.SCB.vtor.write(application.base);
        asm::dsb();
        asm::isb();
        cortex_m::interrupt::enable();
        bootload(application.base as *const u32);
    }
}

#[entry]
fn main() -> ! {
    info!("Hello, poard!");
    cortex_m::asm::delay(8_000_000);
    jump_to_slot(boot_layout::FLASH_SLOTS[0]);
}
