#![warn(
    // missing_debug_implementations,
    // missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]

#[cfg(all(
    feature = "symbolize",
    any(target_os = "linux", target_os = "windows", target_os = "macos")
))]
pub mod symbolize;

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub mod unwinder;
