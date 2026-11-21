#![no_main]
#![no_std]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4xx_hal::{pac, prelude::*, rcc::Config};

boot_layout::slot_header!();

#[entry]
fn main() -> ! {
    if let (Some(dp), Some(_cp)) = (
        pac::Peripherals::take(),
        cortex_m::peripheral::Peripherals::take(),
    ) {
        let mut rcc = dp.RCC.freeze(Config::hsi().sysclk(48.MHz()));
        let gpioa = dp.GPIOA.split(&mut rcc);
        let gpioc = dp.GPIOC.split(&mut rcc);
        let mut led = gpioa.pa5.into_push_pull_output();
        let button = gpioc.pc13.into_floating_input();

        loop {
            if button.is_low() {
                led.set_high();
            } else {
                led.set_low();
            }
        }
    }

    loop {}
}
