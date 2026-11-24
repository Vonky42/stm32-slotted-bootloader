use crate::{menu::MenuCommand, time::Instant};

pub mod debounce;
pub mod single_button;

pub trait MenuInput {
    fn poll(&mut self, now: Instant) -> Option<MenuCommand>;
}
