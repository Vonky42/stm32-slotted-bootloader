#![allow(unused_macros)]

#[cfg(feature = "defmt")]
macro_rules! info {
    ($($t:tt)*) => { ::defmt::info!($($t)*) };
}

#[cfg(not(feature = "defmt"))]
macro_rules! info {
    ($s:literal $(, $x:expr)* $(,)?) => {{
        if false {
            let _ = ($( & $x ),*);
        }
    }};
}

#[cfg(feature = "defmt")]
macro_rules! debug {
    ($($t:tt)*) => { ::defmt::debug!($($t)*) };
}

#[cfg(not(feature = "defmt"))]
macro_rules! debug {
    ($s:literal $(, $x:expr)* $(,)?) => {{
        if false {
            let _ = ($( & $x ),*);
        }
    }};
}

#[cfg(feature = "defmt")]
macro_rules! error {
    ($($t:tt)*) => { ::defmt::error!($($t)*) };
}

#[cfg(not(feature = "defmt"))]
macro_rules! error {
    ($s:literal $(, $x:expr)* $(,)?) => {{
        if false {
            let _ = ($( & $x ),*);
        }
    }};
}

#[cfg(feature = "defmt")]
macro_rules! my_assert_eq {
    ($($t:tt)*) => { ::defmt::assert_eq!($($t)*) };
}

#[cfg(not(feature = "defmt"))]
macro_rules! my_assert_eq {
    ($($t:tt)*) => { ::core::assert_eq!($($t)*) };
}
