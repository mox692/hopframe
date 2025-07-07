use hopframe::unwinder::UnwindBuilder;

#[inline(never)]
pub fn test_function_level_3() -> Vec<u64> {
    let mut unwinder = UnwindBuilder::new().build();
    let mut iter = unwinder.unwind();
    let mut addresses = Vec::new();

    while let Some(frame) = iter.next() {
        addresses.push(frame.address_for_lookup());
    }

    addresses
}

#[inline(never)]
pub fn test_function_level_2() -> Vec<u64> {
    test_function_level_3()
}

#[inline(never)]
pub fn test_function_level_1() -> Vec<u64> {
    test_function_level_2()
}
