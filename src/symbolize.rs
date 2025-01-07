use std::path::Path;

#[cfg(feature = "symbolize")]
pub use wholesym::{LookupAddress, SymbolManager, SymbolManagerConfig, SymbolMap};

/// Builder for [`SymbolMap`].
#[cfg(feature = "symbolize")]
pub struct SymbolMapBuilder<'a> {
    binary_path: Option<&'a Path>,
}
#[cfg(feature = "symbolize")]
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