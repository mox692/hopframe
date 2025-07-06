// Architecture-specific modules
#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "aarch64")]
pub use aarch64::*;

// Architecture-agnostic types
#[cfg(target_arch = "x86_64")]
pub type UnwindBuilder = UnwindBuilderX86_64;
#[cfg(target_arch = "x86_64")]
pub type StackUnwinder = StackUnwinderX86_64;

#[cfg(target_arch = "aarch64")]
pub type UnwindBuilder = UnwindBuilderAarch64;
#[cfg(target_arch = "aarch64")]
pub type StackUnwinder = StackUnwinderAarch64;
