use stm32f4xx_hal::hal::digital::InputPin;

use crate::input::MenuCommand;
use crate::input::debounce::LONG_PRESS;
use crate::input::{
    MenuInput,
    debounce::{ButtonEvent, Debouncer},
};
use crate::time::Instant;

pub struct SingleButton<P> {
    pin: P,
    debouncer: Debouncer,
    press_seen: bool,
    long_press_seen: bool,
}

impl<P: InputPin> SingleButton<P> {
    pub fn new(mut pin: P, now: Instant) -> Self {
        let pressed = pin.is_low().unwrap_or(false);
        Self {
            pin,
            debouncer: Debouncer::new(pressed, now),
            press_seen: false,
            long_press_seen: false,
        }
    }
}

impl<P: InputPin> MenuInput for SingleButton<P> {
    fn poll(&mut self, now: crate::time::Instant) -> Option<MenuCommand> {
        let pressed = self.pin.is_low().unwrap_or(false);

        match self.debouncer.poll(pressed, now)? {
            ButtonEvent::Press => {
                self.press_seen = true;
                self.long_press_seen = false;
                None
            }
            ButtonEvent::LongPressReached => {
                self.long_press_seen = true;
                Some(MenuCommand::Arm)
            }
            ButtonEvent::Release { held } => {
                if !self.press_seen {
                    return None;
                }

                if self.long_press_seen {
                    return Some(MenuCommand::Boot);
                }

                if held < LONG_PRESS {
                    return Some(MenuCommand::Next);
                }

                None
            }
        }
    }
}
