#![no_std]
#![no_main]

use core::cell::RefCell;
use cortex_m_rt::entry;
use critical_section::Mutex;
use panic_halt as _;
use stm32f4xx_hal::{
    gpio::{Output, PA5},
    pac,
    pac::interrupt,
    prelude::*,
    rcc::Config,
    timer::{CounterHz, Event, Flag},
};
static LED: Mutex<RefCell<Option<PA5<Output>>>> = Mutex::new(RefCell::new(None));
static TIMER: Mutex<RefCell<Option<CounterHz<pac::TIM2>>>> = Mutex::new(RefCell::new(None));

const TOGGLE_HZ: u32 = 4;

#[interrupt]
fn TIM2() {
    critical_section::with(|cs| {
        if let Some(timer) = TIMER.borrow_ref_mut(cs).as_mut() {
            timer.clear_flags(Flag::Update);
        }
        if let Some(led) = LED.borrow_ref_mut(cs).as_mut() {
            led.toggle();
        }
    });
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.freeze(Config::hsi().sysclk(84.MHz()));

    let gpioa = dp.GPIOA.split(&mut rcc);
    let led = gpioa.pa5.into_push_pull_output();

    let mut timer = dp.TIM2.counter_hz(&mut rcc);
    timer.start(TOGGLE_HZ.Hz()).unwrap();
    timer.listen(Event::Update);

    critical_section::with(|cs| {
        LED.borrow(cs).replace(Some(led));
        TIMER.borrow(cs).replace(Some(timer));
    });

    unsafe { cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM2) };

    loop {}
}
