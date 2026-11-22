macro_rules! info {
    ($($arg:tt)*) => {{
        #[cfg(feature = "defmt")]
        ::defmt::info!($($arg)*);

        #[cfg(feature = "no_defmt")]
        if false {
            let _ = ($($arg)*);
        }
    }};
}

macro_rules! debug {
    ($($arg:tt)*) => {{
        #[cfg(feature = "defmt")]
        ::defmt::debug!($($arg)*);

        #[cfg(feature = "no_defmt")]
        if false {
            let _ = ($($arg)*);
        }
    }};
}

macro_rules! error {
    ($($arg:tt)*) => {{
        #[cfg(feature = "defmt")]
        ::defmt::error!($($arg)*);

        #[cfg(feature = "no_defmt")]
        if false {
            let _ = ($($arg)*);
        }
    }};
}

macro_rules! my_assert_eq {
    ($($arg:tt)*) => {{
        #[cfg(feature = "defmt")]
        ::defmt::assert_eq!($($arg)*);
        #[cfg(feature = "no_defmt")]
        assert_eq!($($arg)*);
    }};
}
