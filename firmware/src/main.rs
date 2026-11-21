#![no_std]
#![no_main]

#[macro_use]
mod fmt;
mod validate;

use boot_layout::{HEADER_SIZE, Image, SLOT_COUNT, SLOTS, SlotError, SlotHeader, validate};
use cortex_m::asm::{self, bootload};
use cortex_m_rt::entry;
use stm32f4xx_hal as _;

// Debug logging
#[cfg(not(feature = "defmt"))]
use panic_halt as _;

#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

/// Performs a bootload jump to the provided `slot`
fn jump_to_slot(image: Image) -> ! {
    match image {
        Image::Bootloader(_) => {
            panic!("cannot jump to the bootloader")
        }
        Image::Application(slot) => {
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
                let application = slot.application_region();
                cp.SCB.vtor.write(application.base);
                asm::dsb();
                asm::isb();
                cortex_m::interrupt::enable();
                bootload(application.base as *const u32);
            }
        }
    }
}

type Scan = [Result<SlotHeader, SlotError>; SLOT_COUNT];

/// Scan the application slots and log debug info
fn scan_slots() -> Scan {
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

fn default_slot(scan: &Scan) -> Option<usize> {
    scan.iter().position(|slot_scan| slot_scan.is_ok())
}

#[entry]
fn main() -> ! {
    info!("Hello, poard!");
    cortex_m::asm::delay(8_000_000); // for some reason this is still needed
    let scan = scan_slots();
    match default_slot(&scan) {
        Some(index) => jump_to_slot(Image::Application(SLOTS[index])),
        None => {
            error!("no valid slot!");
        }
    }

    loop {} // do nothing
}
