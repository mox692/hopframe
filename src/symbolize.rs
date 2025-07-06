use std::path::Path;

pub use wholesym::{LookupAddress, SymbolManager, SymbolManagerConfig, SymbolMap};

/// Builder for [`SymbolMap`].
pub struct SymbolMapBuilder<'a> {
    binary_path: Option<&'a Path>,
}
impl<'a> SymbolMapBuilder<'a> {
    pub fn new() -> Self {
        Self { binary_path: None }
    }

    pub fn with_binary_path(mut self, binary_path: &'a Path) -> Self {
        self.binary_path = Some(binary_path);
        self
    }

    pub async fn build(self) -> SymbolMap {
        let config = SymbolManagerConfig::default();
        let symbol_manager = SymbolManager::with_config(config);
        if self.binary_path.is_some() {
            symbol_manager
                .load_symbol_map_for_binary_at_path(&self.binary_path.unwrap(), None)
                .await
                .unwrap()
        } else {
            let path = std::env::current_exe().unwrap();
            let path = path.as_path();
            symbol_manager
                .load_symbol_map_for_binary_at_path(path, None)
                .await
                .unwrap()
        }
    }
}

#[cfg(all(
    feature = "symbolize",
    any(target_os = "linux", target_os = "windows", target_os = "macos")
))]
pub fn read_aslr_offset() -> u64 {
    imp::_read_aslr_offset()
}

#[cfg(target_os = "linux")]
mod imp {
    pub(super) fn _read_aslr_offset() -> u64 {
        use procfs::process::{MMapPath, Process};

        let process = Process::myself().unwrap();
        let exe = process.exe().unwrap();
        let maps = &process.maps().unwrap();
        let mut addresses: Vec<u64> = maps
            .iter()
            .filter_map(|map| {
                let MMapPath::Path(bin_path) = &map.pathname else {
                    return None;
                };
                if bin_path != &exe {
                    return None;
                }

                return Some(map.address.0);
            })
            .collect();

        addresses.sort();
        if let Some(addr) = addresses.get(0) {
            Ok(*addr)
        } else {
            panic!("no memory map error.")
        }
    }
}

#[cfg(target_os = "macos")]
mod imp {
    extern "C" {
        fn _dyld_get_image_vmaddr_slide(image_index: u32) -> isize;
    }

    pub(super) fn _read_aslr_offset() -> u64 {
        // image_index = 0 is your main executable
        unsafe { _dyld_get_image_vmaddr_slide(0) as u64 }
    }
}

#[cfg(target_os = "windows")]
mod imp {
    pub(super) fn _read_aslr_offset() -> u64 {
        use winapi::um::winnt::{IMAGE_DOS_HEADER, IMAGE_NT_HEADERS64};
        let base = GetModuleHandleW(null_mut()) as usize;
        // DOS header is at base
        let dos = &*(base as *const IMAGE_DOS_HEADER);
        // NT headers live at base + e_lfanew
        let nth = &*((base + dos.e_lfanew as usize) as *const IMAGE_NT_HEADERS64);
        let preferred = nth.OptionalHeader.ImageBase as usize;
        // slide = actual – preferred
        (base - preferred) as u64
    }
}
