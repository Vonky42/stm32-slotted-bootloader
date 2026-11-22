#![no_std]
#![no_main]

#[macro_use]
mod fmt;
mod button;
mod time;

use crate::{
    button::Debouncer,
    time::{Instant, SYSCLK_HZ},
};
use boot_layout::{HEADER_SIZE, Image, SLOT_COUNT, SLOTS, SlotError, SlotHeader, validate};
use cortex_m::asm::{self, bootload};
use cortex_m_rt::entry;
use stm32f4xx_hal::{pac, prelude::*, rcc::Config};

// Debug logging
#[cfg(feature = "no_defmt")]
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
    // cortex_m::asm::delay(84_000_000); // for some reason this is still needed

    let mut cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.freeze(Config::hsi().sysclk(SYSCLK_HZ.Hz()));

    cp.DCB.enable_trace();
    cp.DWT.enable_cycle_counter();

    my_assert_eq!(rcc.clocks.sysclk().raw(), SYSCLK_HZ);
    info!("sysclk = {} Hz", rcc.clocks.sysclk().raw());

    let gpioc = dp.GPIOC.split(&mut rcc);
    let button = gpioc.pc13.into_floating_input();

    let mut debounced = Debouncer::new(button.is_low(), Instant::now());
    loop {
        if let Some(button_event) = debounced.poll(button.is_low(), Instant::now()) {
            info!("{}", button_event);
        }
    }

    #[allow(unreachable_code)] // stop crying pls
    {
        let scan = scan_slots();
        match default_slot(&scan) {
            Some(index) => jump_to_slot(Image::Application(SLOTS[index])),
            None => {
                error!("no valid slot!");
            }
        }

        loop {} // do nothing
    }
}
