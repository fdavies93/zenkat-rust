use crate::Tree;
use std::num::NonZeroUsize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub app_config: AppConfig,
    pub trees: Arc<Mutex<Vec<Tree>>>,
}

#[derive(Clone)]
pub struct AppConfig {
    pub follow_symlinks: bool,
    pub doc_parser: String,
    pub processes: NonZeroUsize,
}
