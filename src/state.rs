use crate::Link;
use std::sync::Arc;
use tokio::sync::Mutex;
pub type StateDb = Arc<Mutex<Vec<Link>>>;
pub type ConfigHash = Arc<Mutex<String>>;

// Global state. Must be clone-able because it will be shared
// with multiple tasks.
#[derive(Clone, Debug)]
pub struct GlobalState {
    pub state_db: StateDb,
    pub current_config_hash: ConfigHash,
}

impl GlobalState {
    pub fn new(links: Vec<Link>) -> Self {
        Self {
            state_db: Arc::new(Mutex::new(links)),
            current_config_hash: Arc::new(Mutex::new(String::new())),
        }
    }
}
