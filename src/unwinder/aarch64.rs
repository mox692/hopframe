use framehop::{
    aarch64::{CacheAarch64, UnwindRegsAarch64, UnwinderAarch64},
    FrameAddress, Unwinder,
};
use std::arch::asm;

/// load libraries, configure cache or unwinder, etc.
#[derive(Default)]
pub struct UnwindBuilderAarch64 {}

impl UnwindBuilderAarch64 {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn build(self) -> StackUnwinderAarch64 {
        StackUnwinderAarch64 {
            cache: CacheAarch64::<_>::new(),
            unwinder: UnwinderAarch64::new(),
            closure: Box::new(|addr: u64| {
                // Unaligned address
                assert!(addr % 8 == 0);
                unsafe { Ok(*(addr as *const u64)) }
            }),
        }
    }
}

pub struct StackUnwinderAarch64 {
    cache: CacheAarch64,
    unwinder: UnwinderAarch64<Vec<u8>>,
    closure: Box<dyn FnMut(u64) -> Result<u64, ()>>,
}

impl StackUnwinderAarch64 {
    pub fn unwind(&mut self) -> UnwindIterator<'_> {
        #[allow(unused)]
        let (pc, regs) = {
            let mut pc = 0;
            let mut lr = 0;
            let mut sp = 0;
            let mut fp = 0;
            unsafe {
                // Get current PC (program counter)
                asm!("adr {}, .", out(reg) pc);
                // Get LR (link register - x30)
                asm!("mov {}, x30", out(reg) lr);
                // Get SP (stack pointer)
                asm!("mov {}, sp", out(reg) sp);
                // Get FP (frame pointer - x29)
                asm!("mov {}, x29", out(reg) fp);
            }
            (pc, UnwindRegsAarch64::new(lr, sp, fp))
        };

        let iter = self
            .unwinder
            .iter_frames(pc, regs, &mut self.cache, &mut self.closure);

        UnwindIterator::new(iter)
    }
}

pub type UnwindIteratorAarch64<'a> = framehop::UnwindIterator<
    'a,
    'a,
    'a,
    UnwinderAarch64<Vec<u8>>,
    Box<dyn FnMut(u64) -> Result<u64, ()>>,
>;

pub struct UnwindIterator<'a> {
    inner: UnwindIteratorAarch64<'a>,
}

impl<'a> UnwindIterator<'a> {
    fn new(inner: UnwindIteratorAarch64<'a>) -> Self {
        Self { inner }
    }
}

impl<'a> Iterator for UnwindIterator<'a> {
    type Item = FrameAddress;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().ok().flatten()
    }
}
