use crate::time::{Duration, Instant};

const DEBOUNCE_MS: Duration = Duration::from_millis(20); // 20 ms
pub const LONG_PRESS: Duration = Duration::from_millis(1_000); // 1 second

/// Button debouncer used for the bootloader menu
pub struct Debouncer {
    pressed: bool,
    last_change: Instant,
    stable: bool,
    stable_at: Instant,
    long_reported: bool,
}

impl Debouncer {
    pub fn new(pressed: bool, now: Instant) -> Self {
        Self {
            pressed,
            last_change: now,
            stable: false,
            stable_at: now,
            long_reported: false,
        }
    }

    /// Polls the debouncer and returns ButtonEvent when applicable
    pub fn poll(&mut self, pressed: bool, now: Instant) -> Option<ButtonEvent> {
        if pressed != self.pressed {
            self.pressed = pressed;
            self.last_change = now;
            return None;
        }

        if self.pressed != self.stable && now - self.last_change >= DEBOUNCE_MS {
            self.stable = self.pressed;
            return Some(if self.stable {
                self.stable_at = now;
                self.long_reported = false;
                ButtonEvent::Press
            } else {
                ButtonEvent::Release {
                    held: now - self.stable_at,
                }
            });
        }

        if self.stable && !self.long_reported && now - self.stable_at >= LONG_PRESS {
            self.long_reported = true;
            return Some(ButtonEvent::LongPressReached);
        }

        None
    }
}

/// The event the `Debouncer` returns
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ButtonEvent {
    /// The button has been pressed (debounced)
    Press,
    /// Button has been released at was held for `held` CPU cycles
    Release { held: Duration },
    /// The button was pressed long enough to cross the long press threshold
    LongPressReached,
}
