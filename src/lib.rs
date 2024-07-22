#![warn(
    // missing_debug_implementations,
    // missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]

use framehop::{
    x86_64::{CacheX86_64, UnwindRegsX86_64, UnwinderX86_64},
    FrameAddress, Unwinder,
};
use std::arch::asm;
pub use wholesym::{LookupAddress, SymbolManager, SymbolManagerConfig, SymbolMap};

/// load libraries, configure cache or unwinder, etc.
pub struct UnwindBuilderX86_64 {}

impl UnwindBuilderX86_64 {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn build(self) -> StackUnwinderX86_64 {
        StackUnwinderX86_64 {
            cache: CacheX86_64::<_>::new(),
            unwinder: UnwinderX86_64::new(),
            closure: Box::new(|addr: u64| {
                // Unaligned address
                assert!(addr % 8 == 0);
                unsafe { Ok(*(addr as *const u64)) }
            }),
        }
    }
}
impl Default for UnwindBuilderX86_64 {
    fn default() -> Self {
        Self {}
    }
}

pub struct StackUnwinderX86_64 {
    cache: CacheX86_64,
    unwinder: UnwinderX86_64<Vec<u8>>,
    closure: Box<dyn FnMut(u64) -> Result<u64, ()>>,
}

impl StackUnwinderX86_64 {
    pub fn unwind<'a>(&'a mut self) -> UnwindIterator<'a> {
        #[allow(unused)]
        let (rip, regs) = {
            let mut rip = 0;
            let mut rsp = 0;
            let mut rbp = 0;
            unsafe {
                asm!("lea {}, [rip]", out(reg) rip);
                asm!("mov {}, rsp", out(reg) rsp);
                asm!("mov {}, rbp", out(reg) rbp);
            }
            (rip, UnwindRegsX86_64::new(rip, rsp, rbp))
        };

        let iter = self
            .unwinder
            .iter_frames(rip, regs, &mut self.cache, &mut self.closure);

        UnwindIterator::new(iter)
    }
}

pub struct UnwindIterator<'a> {
    inner: framehop::UnwindIterator<
        'a,
        'a,
        'a,
        UnwinderX86_64<Vec<u8>>,
        Box<dyn FnMut(u64) -> Result<u64, ()>>,
    >,
}

impl<'a> UnwindIterator<'a> {
    fn new(
        inner: framehop::UnwindIterator<
            'a,
            'a,
            'a,
            UnwinderX86_64<Vec<u8>>,
            Box<dyn FnMut(u64) -> Result<u64, ()>>,
        >,
    ) -> Self {
        Self { inner }
    }
}

impl<'a> Iterator for UnwindIterator<'a> {
    type Item = FrameAddress;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().ok().flatten()
    }
}

/// Builder for [`SymbolMap`].
pub struct SymbolMapBuilder {}
impl SymbolMapBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn build(self) -> SymbolMap {
        let config = SymbolManagerConfig::default();
        let symbol_manager = SymbolManager::with_config(config);

        let path = std::env::current_exe().unwrap();

        let symbol_map: SymbolMap = symbol_manager
            .load_symbol_map_for_binary_at_path(&path, None)
            .await
            .unwrap();

        symbol_map
    }
}

#[cfg(target_os = "linux")]
pub fn read_aslr_offset() -> Result<u64, std::io::Error> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let path = "/proc/self/maps";
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut start_address: u64 = 0;

    // Maps file looks like this.
    // We want to get `55f229ade000` in the example below.
    //
    // 55f229ade000-55f229bcc000 r--p 00000000 103:05 405068 /home/user/work/...
    // 55f229bcc000-55f22a6ec000 r-xp 000ee000 103:05 405068 /home/user/work/...
    // 55f22a6ec000-55f22aa46000 r--p 00c0e000 103:05 405068 /home/user/work/...
    if let Some(Ok(line)) = reader.lines().next() {
        if let Some(hex_str) = line.split('-').next() {
            let address = u64::from_str_radix(hex_str, 16).expect("Failed to convert hex to u64");
            start_address = address;
        }
    }

    Ok(start_address)
}
