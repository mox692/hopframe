/// Basic usage.
use hopframe::{read_aslr_offset, LookupAddress, SymbolMapBuilder, UnwindBuilderX86_64};

#[tokio::main]
async fn main() {
    let symbol_map = SymbolMapBuilder::new().build().await;
    let mut unwinder = UnwindBuilderX86_64::new().build();

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
