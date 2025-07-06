// //! Basic usage.
// #![cfg(target_os = "linux")]

// #[cfg(feature = "symbolize")]
// #[tokio::main]
// async fn main() {
//     use hopframe::symbolize::{LookupAddress, SymbolMapBuilder};
//     use hopframe::unwinder::UnwindBuilderX86_64;

//     let symbol_map = SymbolMapBuilder::new().build().await;
//     let mut unwinder = UnwindBuilderX86_64::new().build();

//     // Unwinding.
//     let mut iter = unwinder.unwind();

//     // To simbolize propery, we get aslr offset.
//     let aslr_offset = read_aslr_offset().unwrap();
//     while let Some(frame) = iter.next() {
//         // Get symbol for each frame.
//         let symbol = symbol_map
//             .lookup(LookupAddress::Relative(
//                 (frame.address_for_lookup() - aslr_offset) as u32,
//             ))
//             .await;
//         println!(
//             "frame: {:?} symbol: {:?}",
//             &frame,
//             &symbol.map(|s| s.symbol.name)
//         );
//     }
// }

// pub fn read_aslr_offset() -> procfs::ProcResult<u64> {
//     use procfs::process::{MMapPath, Process};

//     let process = Process::myself()?;
//     let exe = process.exe()?;
//     let maps = &process.maps()?;
//     let mut addresses: Vec<u64> = maps
//         .iter()
//         .filter_map(|map| {
//             let MMapPath::Path(bin_path) = &map.pathname else {
//                 return None;
//             };
//             if bin_path != &exe {
//                 return None;
//             }

//             return Some(map.address.0);
//         })
//         .collect();

//     addresses.sort();
//     if let Some(addr) = addresses.get(0) {
//         Ok(*addr)
//     } else {
//         panic!("no memory map error.")
//     }
// }

// #[cfg(not(feature = "symbolize"))]
// #[tokio::main]
// async fn main() {
//     compile_error!("`symbolize` feature is required to execute this examples.");
// }
