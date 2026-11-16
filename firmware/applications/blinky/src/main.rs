#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4xx_hal::pac;

#[entry]
fn main() -> ! {
    loop {}
}
