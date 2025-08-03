// use backtrace::Backtrace;
// use criterion::{black_box, criterion_group, criterion_main, Criterion};
// use hopframe::symbolize::{LookupAddress, SymbolMapBuilder, UnwindBuilderX86_64};
// use std::time::Instant;
// use tokio::runtime::Runtime;

// // unwind only
// fn unwind_backtrace(c: &mut Criterion) {
//     let rt = runtime();

//     c.bench_function("unwind_backtrace", |b| {
//         b.iter_custom(|iter| {
//             let start = Instant::now();
//             for _ in 0..iter {
//                 rt.block_on(async {
//                     black_box(backtrace::trace(|_frame| true));
//                 });
//             }
//             start.elapsed()
//         })
//     });
// }

// // unwind only
// fn unwind_framehop(c: &mut Criterion) {
//     let rt = runtime();

//     c.bench_function("unwind_framehop", |b| {
//         b.iter_custom(|iter| {
//             let mut unwinder = UnwindBuilderX86_64::new().build();

//             let start = Instant::now();
//             for _ in 0..iter {
//                 rt.block_on(async {
//                     let mut iter = unwinder.unwind();
//                     while let Some(frame) = iter.next() {
//                         black_box(frame);
//                     }
//                 });
//             }
//             start.elapsed()
//         });
//     });
// }

// // unwind + symbolize
// fn full_backtrace(c: &mut Criterion) {
//     let rt = runtime();

//     c.bench_function("full_backtrace", |b| {
//         b.iter_custom(|iter| {
//             let start = Instant::now();
//             for _ in 0..iter {
//                 rt.block_on(async {
//                     black_box(Backtrace::new());
//                 });
//             }
//             start.elapsed()
//         })
//     });
// }

// // unwind + symbolize(wholesym)
// fn full_framehop(c: &mut Criterion) {
//     let rt = runtime();

//     c.bench_function("full_framehop", |b| {
//         b.iter_custom(|iter| {
//             let mut unwinder = UnwindBuilderX86_64::new().build();
//             let aslr_offset = read_aslr_offset().unwrap();
//             let symbol_map = rt.block_on(async { SymbolMapBuilder::new().build().await });

//             let start = Instant::now();
//             for _ in 0..iter {
//                 rt.block_on(async {
//                     let mut iter = unwinder.unwind();
//                     while let Some(frame) = iter.next() {
//                         black_box(
//                             symbol_map
//                                 .lookup(LookupAddress::Relative(
//                                     (frame.address_for_lookup() - aslr_offset) as u32,
//                                 ))
//                                 .await,
//                         );
//                     }
//                 });
//             }
//             start.elapsed()
//         });
//     });
// }

// fn runtime() -> Runtime {
//     tokio::runtime::Builder::new_multi_thread()
//         .enable_time()
//         .build()
//         .unwrap()
// }

// fn read_aslr_offset() -> procfs::ProcResult<u64> {
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

// criterion_group!(
//     framehop_vs_backtrace,
//     full_backtrace,
//     full_framehop,
//     unwind_backtrace,
//     unwind_framehop
// );
// criterion_main!(framehop_vs_backtrace);

fn main() {}
