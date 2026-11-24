#![cfg_attr(not(test), no_std)]

#[macro_use]
mod fmt;

pub mod indicator;
pub mod input;
pub mod menu;

pub use indicator::{Indicator, led::LedIndicator};
pub use input::{MenuInput, single_button::SingleButton};
pub use menu::{Action, BootMenu, MenuCommand};

use boot_core::Instant;
