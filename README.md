# hopframe
A simple, easy wrapper for [framehop](https://github.com/mstange/framehop) stack unwinding library.  

While framehop gives you powerful low-level control, this crate provides higher-level interface on top of framehop. This would be good option for cases where you just want to perform stack unwinding without writing pretty low-level code, such as assembly.

# Example
Here is a basic usage:
```rust
#[tokio::main]
async fn main() {
    use hopframe::aslr::read_aslr_offset;
    use hopframe::symbolize::{LookupAddress, SymbolMapBuilder};
    use hopframe::unwinder::UnwindBuilder;

    let symbol_map = SymbolMapBuilder::new().build().await;
    let mut unwinder = UnwindBuilder::new().build();

    // Unwinding.
    let mut iter = unwinder.unwind();

    // To simbolize propery, we get aslr offset.
    let aslr_offset = read_aslr_offset().unwrap();
    while let Some(frame) = iter.next() {
        // Get symbol for each frame.
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

You need to use `RUSTFLAGS="-C force-frame-pointers=yes"` and some features to run the program above.

```shell
$ RUSTFLAGS="-C force-frame-pointers=yes" cargo run --example basic --features "symbolize,aslr"


frame: InstructionPointer(4300423432) symbol: Some("hopframe::unwinder::aarch64::StackUnwinderAarch64::unwind")
frame: ReturnAddress(4298871812) symbol: Some("basic::main::{{closure}}")
frame: ReturnAddress(4299910828) symbol: Some("tokio::runtime::park::CachedParkThread::block_on::{{closure}}")
frame: ReturnAddress(4299909648) symbol: Some("tokio::runtime::park::CachedParkThread::block_on")
frame: ReturnAddress(4299609608) symbol: Some("tokio::runtime::context::blocking::BlockingRegionGuard::block_on")
frame: ReturnAddress(4298762828) symbol: Some("tokio::runtime::scheduler::multi_thread::MultiThread::block_on::{{closure}}")
frame: ReturnAddress(4299965468) symbol: Some("tokio::runtime::context::runtime::enter_runtime")
frame: ReturnAddress(4298762668) symbol: Some("tokio::runtime::scheduler::multi_thread::MultiThread::block_on")
frame: ReturnAddress(4299041024) symbol: Some("tokio::runtime::runtime::Runtime::block_on_inner")
frame: ReturnAddress(4299041576) symbol: Some("tokio::runtime::runtime::Runtime::block_on")
frame: ReturnAddress(4300177668) symbol: Some("basic::main")
frame: ReturnAddress(4298772212) symbol: Some("core::ops::function::FnOnce::call_once")
frame: ReturnAddress(4299668660) symbol: Some("std::sys::backtrace::__rust_begin_short_backtrace")
frame: ReturnAddress(4299228452) symbol: Some("std::rt::lang_start::{{closure}}")
frame: ReturnAddress(4308728420) symbol: Some("std::rt::lang_start_internal")
frame: ReturnAddress(4299228412) symbol: Some("std::rt::lang_start")
frame: ReturnAddress(4300177812) symbol: Some("main")
frame: ReturnAddress(6675786648) symbol: None
```

See also `examples` directory.

# Platform Support

| OS      | aarch64 | x86_64 |
| ------- | ------- | ------ |
| linux   | ❌       | ✅      |
| windows | ❌       | ❌      |
| macos   | ✅       | ✅      |
