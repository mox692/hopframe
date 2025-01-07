#![warn(
    // missing_debug_implementations,
    // missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]

#[cfg(feature = "symbolize")]
pub mod symbolize;

pub mod unwinder;