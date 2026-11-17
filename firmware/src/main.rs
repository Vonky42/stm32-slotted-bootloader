#![no_std]
#![no_main]

#[macro_use]
mod fmt;

use boot_layout::{ERASED, Image, SLOT_MAGIC, SLOTS, SlotHeader, header_crc, image_crc};
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

fn scan_slots() {
    for (index, slot) in SLOTS.iter().enumerate() {
        let base = slot.header_region().base;
        let header = unsafe { SlotHeader::at(slot) };

        match header.magic {
            ERASED => {
                debug!("slot {} @ {:#010x}: empty", index, base)
            }
            SLOT_MAGIC => {
                let (major, minor, patch) = header.version_parts();
                debug!(
                    "slot {} @ {:#010x}: '{=str}' v{}.{}.{} built for slot {}, \
                     size={} crc={:#010x} hcrc={:#010x}",
                    index,
                    base,
                    header.name(),
                    major,
                    minor,
                    patch,
                    header.slot_number,
                    header.image_size,
                    header.image_crc,
                    header.header_crc,
                );
                debug!("Computed header crc: {}", header_crc(header.as_bytes()));
                let application = slot.application_region();
                unsafe {
                    debug!(
                        "Computed image crc: {}",
                        image_crc(core::slice::from_raw_parts(
                            application.base as *const u8,
                            application.length as usize
                        ))
                    );
                }
            }
            other => debug!(
                "slot {} @ {:#010x}: unknown magic {:#010x}",
                index, base, other
            ),
        }
    }
}

#[entry]
fn main() -> ! {
    info!("Hello, poard!");
    cortex_m::asm::delay(8_000_000);
    scan_slots();
    cortex_m::asm::delay(8_000_000);
    jump_to_slot(Image::Application(SLOTS[0]));
}
