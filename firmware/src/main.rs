#![no_std]
#![no_main]

#[macro_use]
mod fmt;
mod boot;
mod button;
mod indicator;
mod menu;
mod slots;
mod time;

use crate::{
    boot::jump_to_slot,
    button::Debouncer,
    indicator::{Indicator, led::LedIndicator},
    menu::{Action, BootMenu},
    slots::scan_slots,
    time::{Duration, Instant, SYSCLK_HZ},
};
use boot_layout::{Image, SLOTS};
use cortex_m_rt::entry;
use stm32f4xx_hal::{pac, prelude::*, rcc::Config};

// Debug logging
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};
#[cfg(not(feature = "defmt"))]
#[inline(never)]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[entry]
fn main() -> ! {
    // Setup
    let mut cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    let mut rcc = dp.RCC.freeze(Config::hsi().sysclk(SYSCLK_HZ.Hz()));
    cp.DCB.enable_trace();
    cp.DWT.enable_cycle_counter();

    my_assert_eq!(rcc.clocks.sysclk().raw(), SYSCLK_HZ);

    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpioc = dp.GPIOC.split(&mut rcc);
    let button = gpioc.pc13.into_floating_input();

    let now = Instant::now();
    let mut led = LedIndicator::new(gpioa.pa5.into_push_pull_output(), now);
    let mut debouncer = Debouncer::new(button.is_low(), now);
    let scan = scan_slots();
    let mut menu = BootMenu::new(
        scan,
        &mut led,
        Instant::now(),
        Some(Duration::from_millis(5_000)),
    );

    loop {
        let now = Instant::now();
        let action = debouncer
            .poll(button.is_low(), now)
            .and_then(|event| menu.handle(event, &mut led, now))
            .or_else(|| menu.tick(now));
        if let Some(Action::Boot(slot)) = action {
            jump_to_slot(Image::Application(SLOTS[slot]));
        }
        led.tick(now);
    }
}
