macro_rules! info {
    ($($arg:tt)*) => {{
        #[cfg(feature = "defmt")]
        ::defmt::info!($($arg)*);
    }};
}

macro_rules! debug {
    ($($arg:tt)*) => {{
        #[cfg(feature = "defmt")]
        ::defmt::debug!($($arg)*);
    }};
}
