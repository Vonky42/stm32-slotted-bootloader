use boot_core::Instant;

/// The current cycle count as an `Instant`
pub fn now() -> Instant {
    Instant::from_cycles(cortex_m::peripheral::DWT::cycle_count())
}
