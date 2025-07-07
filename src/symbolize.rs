use std::path::Path;

pub use wholesym::{LookupAddress, SymbolManager, SymbolManagerConfig, SymbolMap};

/// Error type for symbolization operations
#[derive(Debug)]
pub enum Error {
    /// Failed to access process information
    ProcessAccessError(String),
    /// Failed to read executable path
    ExecutablePathError(String),
    /// Failed to read memory maps
    MemoryMapError(String),
    /// No memory mapping found for the executable
    NoMemoryMapping,
    /// Platform-specific error
    PlatformError(String),
}

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
pub fn read_aslr_offset() -> Result<u64, Error> {
    imp::_read_aslr_offset()
}

#[cfg(target_os = "linux")]
mod imp {
    use super::Error;

    pub(super) fn _read_aslr_offset() -> Result<u64, Error> {
        use procfs::process::{MMapPath, Process};

        let process = Process::myself()
            .map_err(|e| Error::ProcessAccessError(format!("Failed to access process: {}", e)))?;
        let exe = process.exe().map_err(|e| {
            Error::ExecutablePathError(format!("Failed to get executable path: {}", e))
        })?;
        let maps = process
            .maps()
            .map_err(|e| Error::MemoryMapError(format!("Failed to read memory maps: {}", e)))?;

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
        addresses.first().copied().ok_or(Error::NoMemoryMapping)
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use super::Error;

    extern "C" {
        fn _dyld_get_image_vmaddr_slide(image_index: u32) -> isize;
    }

    pub(super) fn _read_aslr_offset() -> Result<u64, Error> {
        // image_index = 0 is your main executable
        // Note: _dyld_get_image_vmaddr_slide returns 0 if the image doesn't exist,
        // but for index 0 (main executable) it should always exist
        let slide = unsafe { _dyld_get_image_vmaddr_slide(0) };
        Ok(slide as u64)
    }
}

#[cfg(target_os = "windows")]
mod imp {
    use super::Error;
    use std::ptr::null_mut;
    use winapi::um::libloaderapi::GetModuleHandleW;

    pub(super) fn _read_aslr_offset() -> Result<u64, Error> {
        use winapi::um::winnt::{IMAGE_DOS_HEADER, IMAGE_NT_HEADERS64};

        let base = unsafe { GetModuleHandleW(null_mut()) as usize };
        if base == 0 {
            return Err(Error::PlatformError(
                "Failed to get module handle".to_string(),
            ));
        }

        unsafe {
            // DOS header is at base
            let dos = &*(base as *const IMAGE_DOS_HEADER);
            // NT headers live at base + e_lfanew
            let nth = &*((base + dos.e_lfanew as usize) as *const IMAGE_NT_HEADERS64);
            let preferred = nth.OptionalHeader.ImageBase as usize;
            // slide = actual – preferred
            Ok((base - preferred) as u64)
        }
    }
}
