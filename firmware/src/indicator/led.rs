use crate::indicator::Indicator;
use crate::time::{Duration, Instant};
use boot_layout::{SlotError, SlotHeader};
use stm32f4xx_hal::hal::digital::OutputPin;

const ON: Duration = Duration::from_millis(150);
const OFF: Duration = Duration::from_millis(250);
const PAUSE: Duration = Duration::from_millis(1000);
const FLASH: Duration = Duration::from_millis(50);

enum Pattern {
    Blinks(u8),
    Solid,
    FastFlash,
}

pub struct LedIndicator<P> {
    pin: P,
    pattern: Pattern,
    cycle_start: Instant,
}

impl<P: OutputPin> LedIndicator<P> {
    pub fn new(mut pin: P, now: Instant) -> Self {
        let _ = pin.set_high();
        Self {
            pin,
            pattern: Pattern::Solid,
            cycle_start: now,
        }
    }

    fn start(&mut self, pattern: Pattern, now: Instant) {
        self.pattern = pattern;
        self.cycle_start = now;
        self.tick(now);
    }

    fn set(&mut self, on: bool) {
        let _ = if on {
            self.pin.set_high()
        } else {
            self.pin.set_low()
        };
    }

    fn elapsed_in_cycles(&mut self, now: Instant, cycle: Duration) -> Duration {
        let mut elapsed = now - self.cycle_start;
        if elapsed > cycle {
            self.cycle_start = self.cycle_start + cycle;
            elapsed = now - self.cycle_start;
        }
        elapsed
    }
}

impl<P: OutputPin> Indicator for LedIndicator<P> {
    fn show_selection(&mut self, slot: u8, _info: &Result<SlotHeader, SlotError>, now: Instant) {
        self.start(Pattern::Blinks(slot + 1), now);
    }

    fn show_boot_pending(&mut self, now: Instant) {
        self.start(Pattern::Solid, now)
    }

    // once UART is implemented
    fn _show_updating(&mut self, now: Instant) {
        self.start(Pattern::Solid, now)
    }

    fn show_error(&mut self, now: Instant) {
        self.start(Pattern::FastFlash, now);
    }

    fn tick(&mut self, now: Instant) {
        let on = match self.pattern {
            Pattern::Solid => true,
            Pattern::FastFlash => {
                let elapsed = self.elapsed_in_cycles(now, FLASH * 2);
                elapsed < FLASH
            }
            Pattern::Blinks(n) => {
                let blink = ON + OFF;
                let elapsed = self.elapsed_in_cycles(now, blink * n as u32 + PAUSE);
                if elapsed >= blink * n as u32 {
                    false
                } else {
                    elapsed % blink < ON
                }
            }
        };
        self.set(on);
    }
}
