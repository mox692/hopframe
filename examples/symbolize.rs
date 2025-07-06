//! Basic usage.

#[cfg(all(
    feature = "symbolize",
    any(target_os = "linux", target_os = "windows", target_os = "macos")
))]
#[tokio::main]
async fn main() {
    use hopframe::symbolize::{read_aslr_offset, LookupAddress, SymbolMapBuilder};
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

#[cfg(not(all(
    feature = "symbolize",
    any(target_os = "linux", target_os = "windows", target_os = "macos")
)))]
fn main() {
    compile_error!("Symbolization is not enabled, or the target OS is not supported.");
}
