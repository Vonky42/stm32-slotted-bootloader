use crate::{
    button::{ButtonEvent, LONG_PRESS},
    menu::MenuState::Armed,
    time::{Duration, Instant},
};
use boot_layout::{SLOT_COUNT, SlotError, SlotHeader};

use crate::indicator::Indicator;

type Slots = [Result<SlotHeader, SlotError>; SLOT_COUNT];

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum MenuState {
    NoValidSlot,
    Selecting(usize),
    Armed(usize),
}

pub struct BootMenu {
    state: MenuState,
    slots: Slots,
    timeout: Option<Duration>,
    last_activity: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Action {
    Boot(usize),
}

impl BootMenu {
    pub fn new(
        slots: Slots,
        indicator: &mut impl Indicator,
        now: Instant,
        timeout: Option<Duration>,
    ) -> Self {
        let first_valid = slots.iter().position(|s| s.is_ok());
        let initial_state = first_valid.map_or(MenuState::NoValidSlot, |selected| {
            MenuState::Selecting(selected)
        });

        if initial_state == MenuState::NoValidSlot {
            debug!("No valid slot?!");
        }

        let menu = Self {
            state: initial_state,
            slots: slots,
            timeout,
            last_activity: now,
        };
        menu.show(indicator, now);
        menu
    }

    /// Forward the boot menu's state to the implemented indicator(s)
    pub fn show(&self, indicator: &mut impl Indicator, now: Instant) {
        match self.state {
            MenuState::Selecting(index) => {
                indicator.show_selection(index as u8, &self.slots[index], now)
            }
            MenuState::Armed(_) => indicator.show_boot_pending(now),
            MenuState::NoValidSlot => indicator.show_error(now),
        }
    }

    /// Change the bootloader's MenuState based on `event` ButtenEvent input
    pub fn handle(
        &mut self,
        event: ButtonEvent,
        indicator: &mut impl Indicator,
        now: Instant,
    ) -> Option<Action> {
        let action = match (self.state, event) {
            (MenuState::NoValidSlot, _) => None,
            (MenuState::Selecting(s), ButtonEvent::LongPressReached) => {
                self.state = Armed(s);
                debug!("boot pending on slot {}...", s);
                None
            }
            (MenuState::Armed(s), ButtonEvent::Release { held: _ }) => Some(Action::Boot(s)),
            (MenuState::Selecting(_), ButtonEvent::Release { held }) => {
                if held < LONG_PRESS {
                    self.select_next();
                    debug!("next valid slot: {}", &self.selected().unwrap());
                }
                None
            }
            _ => {
                return None;
            }
        };

        self.show(indicator, now);

        action
    }

    /// Handle time-based menu logic such as the `timeout` if configured
    pub fn tick(&mut self, now: Instant) -> Option<Action> {
        let timeout = self.timeout?;
        if now - self.last_activity < timeout {
            return None;
        }
        match self.state {
            MenuState::Selecting(s) => {
                debug!(
                    "boot menu timeout reached. booting selected slot {}",
                    &self.selected().unwrap()
                );
                Some(Action::Boot(s))
            }
            _ => None,
        }
    }

    /// Select the next valid slot
    fn select_next(&mut self) {
        let next_valid = self.next_valid();
        if let (MenuState::Selecting(s), Some(n)) = (&mut self.state, next_valid) {
            *s = n;
        };
        debug!("selected is now: {}", &self.selected().unwrap());
    }

    /// Returns the next valid slot index, or None
    fn next_valid(&self) -> Option<usize> {
        let current = self.selected()?;
        (1..=self.slots.len())
            .map(|i| (current + i) % self.slots.len())
            .find(|&s| self.slots[s].is_ok())
    }

    /// Returns the selected slot, unwarpping it from the current MenuState
    fn selected(&self) -> Option<usize> {
        match self.state {
            MenuState::Selecting(s) | MenuState::Armed(s) => Some(s),
            MenuState::NoValidSlot => None,
        }
    }
}
