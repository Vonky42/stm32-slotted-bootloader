use stm32f4xx_hal::{
    gpio::{GpioExt, Input, Output, PA5, PC13, PushPull},
    pac,
    prelude::_fugit_RateExtU32,
    rcc::{Config, RccExt},
};

use boot_core::SYSCLK_HZ;

pub struct Board {
    pub ld2: PA5<Output<PushPull>>, // active-high
    pub button: PC13<Input>,        // active-low
}

impl Board {
    pub fn init() -> Self {
        let dp = pac::Peripherals::take().unwrap();
        let mut cp = cortex_m::Peripherals::take().unwrap();

        // Clocks
        let mut rcc = dp.RCC.freeze(Config::hsi().sysclk(SYSCLK_HZ.Hz()));
        my_assert_eq!(rcc.clocks.sysclk().raw(), SYSCLK_HZ);

        // Time
        cp.DCB.enable_trace();
        cp.DWT.enable_cycle_counter();

        // GPIO
        let gpioa = dp.GPIOA.split(&mut rcc);
        let gpioc = dp.GPIOC.split(&mut rcc);

        Self {
            ld2: gpioa.pa5.into_push_pull_output(),
            button: gpioc.pc13.into_input(),
        }
    }

    pub fn button_pressed(&mut self) -> bool {
        self.button.is_low()
    }
}
