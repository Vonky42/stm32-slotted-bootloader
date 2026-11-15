#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;
use stm32f4xx_hal::pac;

#[entry]
fn main() -> ! {
    loop {}
}