#![cfg(all(
    feature = "symbolize",
    any(target_os = "linux", target_os = "windows", target_os = "macos")
))]

mod common;

use hopframe::aslr::read_aslr_offset;
use hopframe::symbolize::{LookupAddress, SymbolMapBuilder};
use hopframe::unwinder::UnwindBuilder;
use std::collections::HashSet;

#[tokio::test]
async fn test_function_symbols_in_backtrace() {
    verify_function_symbols_in_backtrace().await;
}

async fn verify_function_symbols_in_backtrace() {
    #[inline(never)]
    fn test_level_3_with_unwinder() -> Vec<u64> {
        let mut unwinder = UnwindBuilder::new().build();
        let mut iter = unwinder.unwind();
        let mut addresses = Vec::new();

        while let Some(frame) = iter.next() {
            addresses.push(frame.address_for_lookup());
        }

        addresses
    }

    #[inline(never)]
    fn test_level_2_with_unwinder() -> Vec<u64> {
        test_level_3_with_unwinder()
    }

    #[inline(never)]
    fn test_level_1_with_unwinder() -> Vec<u64> {
        test_level_2_with_unwinder()
    }

    let addresses = test_level_1_with_unwinder();
    let symbol_map = SymbolMapBuilder::new().build().await;
    let aslr_offset = read_aslr_offset().unwrap();

    let expected_functions = vec![
        "test_level_1_with_unwinder",
        "test_level_2_with_unwinder",
        "test_level_3_with_unwinder",
    ];

    let mut found_functions = HashSet::new();
    let mut total_addresses = 0;
    let mut resolved_symbols = 0;

    for addr in &addresses {
        total_addresses += 1;
        let symbol = symbol_map
            .lookup(LookupAddress::Relative((addr - aslr_offset) as u32))
            .await;

        if let Some(sym) = symbol {
            resolved_symbols += 1;
            let name = sym.symbol.name;

            for expected in &expected_functions {
                if name.contains(expected) {
                    found_functions.insert(expected.to_string());
                }
            }
        }
    }

    assert!(total_addresses > 0, "Should have captured some addresses");

    assert!(
        resolved_symbols > 0,
        "Should have resolved at least some symbols. Make sure debug symbols are available."
    );

    assert!(
        found_functions.len() >= 3,
        "Should find all three test functions. Found: {:?}",
        found_functions
    );
}

#[tokio::test]
async fn test_recursive_function_symbols() {
    verify_recursive_function_symbols().await;
}

async fn verify_recursive_function_symbols() {
    let depth = 10;

    #[inline(never)]
    fn recursive_with_unwinder(current: u32, max_depth: u32) -> Vec<u64> {
        if current >= max_depth {
            let mut unwinder = UnwindBuilder::new().build();
            let mut iter = unwinder.unwind();
            let mut addresses = Vec::new();

            while let Some(frame) = iter.next() {
                addresses.push(frame.address_for_lookup());
            }

            addresses
        } else {
            recursive_with_unwinder(current + 1, max_depth)
        }
    }

    let addresses = recursive_with_unwinder(0, depth);
    let symbol_map = SymbolMapBuilder::new().build().await;
    let aslr_offset = read_aslr_offset().unwrap();

    let mut recursive_count = 0;

    for addr in &addresses {
        let symbol = symbol_map
            .lookup(LookupAddress::Relative((addr - aslr_offset) as u32))
            .await;

        if let Some(sym) = symbol {
            if sym.symbol.name.contains("recursive_with_unwinder") {
                recursive_count += 1;
            }
        }
    }

    assert!(
        recursive_count >= depth as usize / 2,
        "Should find at least {} instances of recursive_with_unwinder. Found: {}",
        depth / 2,
        recursive_count
    );
}

#[tokio::test]
async fn test_symbol_resolution_consistency() {
    #[inline(never)]
    fn unique_test_function_a() -> Vec<u64> {
        let mut unwinder = UnwindBuilder::new().build();
        let mut iter = unwinder.unwind();
        let mut addresses = Vec::new();

        while let Some(frame) = iter.next() {
            addresses.push(frame.address_for_lookup());
        }

        addresses
    }

    #[inline(never)]
    fn unique_test_function_b() -> Vec<u64> {
        unique_test_function_a()
    }

    let addresses = unique_test_function_b();
    let symbol_map = SymbolMapBuilder::new().build().await;
    let aslr_offset = read_aslr_offset().unwrap();

    let expected_unique_functions = vec!["unique_test_function_a", "unique_test_function_b"];

    let mut found_unique = HashSet::new();

    for addr in &addresses {
        let symbol = symbol_map
            .lookup(LookupAddress::Relative((addr - aslr_offset) as u32))
            .await;

        if let Some(sym) = symbol {
            let name = sym.symbol.name;
            for expected in &expected_unique_functions {
                if name.contains(expected) {
                    found_unique.insert(expected.to_string());
                }
            }
        }
    }

    assert_eq!(
        found_unique.len(),
        2,
        "Should find both unique test functions. Found: {:?}",
        found_unique
    );
}

#[tokio::test]
async fn test_inline_prevention_verification() {
    #[inline(never)]
    fn must_not_be_inlined() -> Vec<u64> {
        common::test_function_level_1()
    }

    let addresses = must_not_be_inlined();
    let symbol_map = SymbolMapBuilder::new().build().await;
    let aslr_offset = read_aslr_offset().unwrap();

    let mut found_self = false;

    for addr in &addresses {
        let symbol = symbol_map
            .lookup(LookupAddress::Relative((addr - aslr_offset) as u32))
            .await;

        if let Some(sym) = symbol {
            if sym.symbol.name.contains("must_not_be_inlined") {
                found_self = true;
                break;
            }
        }
    }

    assert!(
        found_self,
        "The function 'must_not_be_inlined' should appear in the symbols"
    );

    assert!(!addresses.is_empty(), "Should have captured addresses");
}

#[tokio::test]
async fn test_common_test_functions_symbolization() {
    let addresses = common::test_function_level_1();
    let symbol_map = SymbolMapBuilder::new().build().await;
    let aslr_offset = read_aslr_offset().unwrap();

    let expected_functions = vec![
        "test_function_level_1",
        "test_function_level_2",
        "test_function_level_3",
    ];

    let mut found_functions = HashSet::new();

    for addr in &addresses {
        let symbol = symbol_map
            .lookup(LookupAddress::Relative((addr - aslr_offset) as u32))
            .await;

        if let Some(sym) = symbol {
            let name = sym.symbol.name;
            for expected in &expected_functions {
                if name.contains(expected) {
                    found_functions.insert(expected.to_string());
                }
            }
        }
    }

    assert!(
        found_functions.len() >= 3,
        "Should find all three test functions from common module. Found: {:?}",
        found_functions
    );
}
