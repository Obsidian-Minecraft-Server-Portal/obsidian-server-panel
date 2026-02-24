#[cfg(feature = "logging")]
pub use log::{debug, error, info, trace, warn};

#[cfg(not(feature = "logging"))]
pub use crate::{debug, error, info, trace, warn};

#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {};
}

#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {};
}

#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {};
}

#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {};
}

#[cfg(not(feature = "logging"))]
#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {};
}