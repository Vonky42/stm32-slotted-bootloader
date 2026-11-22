use core::ops::Sub;

pub const SYSCLK_HZ: u32 = 84_000_000;
const CYCLES_PER_MS: u32 = SYSCLK_HZ / 1000;

/// An instant in CPU cycles
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Instant(u32);

/// A duration in CPU cycles
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duration(u32);

impl Instant {
    pub fn now() -> Self {
        Self(cortex_m::peripheral::DWT::cycle_count())
    }
}

// Instant - Instant = Duration
impl Sub for Instant {
    type Output = Duration;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Duration(self.0.wrapping_sub(rhs.0))
    }
}

impl Duration {
    pub const fn from_millis(ms: u32) -> Self {
        assert!(
            ms <= u32::MAX / CYCLES_PER_MS,
            "Duration too long to fit u32"
        );
        Self(ms * CYCLES_PER_MS)
    }

    pub const fn as_millis(&self) -> u32 {
        self.0 / CYCLES_PER_MS
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Duration {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "{=u32}ms", self.as_millis())
    }
}
