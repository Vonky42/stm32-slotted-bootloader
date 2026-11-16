#![no_std]
#![no_main]

#[macro_use]
mod fmt;

use cortex_m_rt::entry;
use stm32f4xx_hal::pac;

// Debug logging
#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

#[entry]
fn main() -> ! {
    info!("Hello, poard!");
    loop {}
}
