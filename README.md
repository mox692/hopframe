## hopframe
A simple, easy wrapper for [framehop](https://github.com/mstange/framehop).
Currently available for linux only.

## Usage
Here is a basic usage:
```rust
use hopframe::{read_aslr_offset, LookupAddress, SymbolMapBuilder, UnwindBuilderX86_64};

#[tokio::main]
async fn main() {
    let symbol_map = SymbolMapBuilder::new().build().await;
    let mut unwinder = UnwindBuilderX86_64::new().build();
    let mut iter = unwinder.unwind();
    let aslr_offset = read_aslr_offset().unwrap();

    while let Some(frame) = iter.next() {
        let symbol = symbol_map
            .lookup(LookupAddress::Relative(
                (frame.address_for_lookup() - aslr_offset) as u32,
            ))
            .await;
        println!(
            "frame: {:?} symbol: {:?}",
            &frame,
            &symbol.map(|s| s.symbol.name)
        );
    }
}
```

You need to use `RUSTFLAGS="-C force-frame-pointers=yes"` flag.

```shell
$ RUSTFLAGS="-C force-frame-pointers=yes" cargo run --example basic


frame: InstructionPointer(94006149978623) symbol: Some("hopframe::StackUnwinderX86_64::unwind")
frame: ReturnAddress(94006148593726) symbol: Some("basic::main::{{closure}}")
frame: ReturnAddress(94006147962242) symbol: Some("tokio::runtime::park::CachedParkThread::block_on::{{closure}}")
frame: ReturnAddress(94006147960790) symbol: Some("tokio::runtime::park::CachedParkThread::block_on")
frame: ReturnAddress(94006149394329) symbol: Some("tokio::runtime::context::blocking::BlockingRegionGuard::block_on")
frame: ReturnAddress(94006148068576) symbol: Some("tokio::runtime::scheduler::multi_thread::MultiThread::block_on::{{closure}}")
frame: ReturnAddress(94006149464552) symbol: Some("tokio::runtime::context::runtime::enter_runtime")
frame: ReturnAddress(94006148068491) symbol: Some("tokio::runtime::scheduler::multi_thread::MultiThread::block_on")
frame: ReturnAddress(94006148098756) symbol: Some("tokio::runtime::runtime::Runtime::block_on")
frame: ReturnAddress(94006148264083) symbol: Some("basic::main")
frame: ReturnAddress(94006149083790) symbol: Some("core::ops::function::FnOnce::call_once")
frame: ReturnAddress(94006147746145) symbol: Some("std::sys_common::backtrace::__rust_begin_short_backtrace")
frame: ReturnAddress(94006149464100) symbol: Some("std::rt::lang_start::{{closure}}")
frame: ReturnAddress(94006159644801) symbol: Some("std::rt::lang_start_internal")
```
