#![cfg_attr(not(feature = "build"), no_std)]

mod header;
mod layout;
#[cfg(feature = "build")]
mod memory_x;

pub use header::*;
pub use layout::*;
#[cfg(feature = "build")]
pub use memory_x::emit_memory_x;
