use crate::tree_store::TreeStore;
use crate::QueryParser;
use std::num::NonZeroUsize;

pub struct AppState {
    pub parser: QueryParser,
    pub working_store: TreeStore,
    pub app_config: AppConfig,
}

pub struct AppConfig {
    pub follow_symlinks: bool,
    pub doc_parser: String,
    pub processes: NonZeroUsize,
}
