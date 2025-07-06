//! Basic usage, without symbolication.

#[tokio::main]
async fn main() {
    use hopframe::unwinder::UnwindBuilder;

    let mut unwinder = UnwindBuilder::new().build();

    // Unwinding.
    let mut iter = unwinder.unwind();

    while let Some(frame) = iter.next() {
        println!("The raw address (AVMA): {:?}", frame.address());
    }
}
