use crate::QueryParser;
use crate::TreeStore;
use std::num::NonZeroUsize;
use std::sync::RwLock;

pub struct AppState {
    pub parser: QueryParser,
    pub store: RwLock<TreeStore>,
    pub app_config: AppConfig,
}

pub struct AppConfig {
    pub follow_symlinks: bool,
    pub doc_parser: String,
    pub processes: NonZeroUsize,
}
