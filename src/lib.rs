#![warn(
    // missing_debug_implementations,
    // missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]

// #[cfg(feature = "symbolize")]
// pub mod symbolize;

#[cfg(target_arch = "x86_64")]
pub mod unwinder;
