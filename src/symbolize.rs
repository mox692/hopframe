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
    use std::{
        fs::File,
        io::{BufRead, BufReader},
        path::PathBuf,
    };

    pub(super) fn _read_aslr_offset() -> Result<u64, Error> {
        // Resolve our real path once to avoid repeated allocations.
        let exe: PathBuf = std::fs::read_link("/proc/self/exe")
            .map_err(|e| Error::ExecutablePathError(format!("readlink failed: {e}")))?;

        let file = File::open("/proc/self/maps")
            .map_err(|e| Error::MemoryMapError(format!("open maps: {e}")))?;
        let reader = BufReader::new(file);

        let mut addrs: Vec<u64> = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| Error::MemoryMapError(format!("read maps: {e}")))?;
            // Example line:
            // 55b63ea4c000-55b63ea6e000 r-xp 00000000 fd:01 123456 /usr/bin/myapp
            let mut parts = line.split_whitespace();
            let range = parts
                .next()
                .ok_or_else(|| Error::MemoryMapError("malformed maps line".into()))?;
            let pathname = parts.nth(4); // skip perms, offset, dev, inode

            // Only interested in the executable’s own mapping.
            if pathname.map(|p| PathBuf::from(p) == exe).unwrap_or(false) {
                if let Some(start_hex) = range.split('-').next() {
                    let addr = u64::from_str_radix(start_hex, 16)
                        .map_err(|_| Error::MemoryMapError("invalid addr".into()))?;
                    addrs.push(addr);
                }
            }
        }

        addrs.sort_unstable();
        addrs.first().copied().ok_or(Error::NoMemoryMapping)
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
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

    pub(super) fn _read_aslr_offset() -> Result<u64, Error> {
        use windows_sys::Win32::System::SystemServices::{IMAGE_DOS_HEADER, IMAGE_NT_HEADERS64};

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
