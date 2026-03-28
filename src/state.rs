use crate::{Link, TagWriteData};
use crossbeam_channel::Sender;
use rhai::AST;
use std::sync::Arc;
use tokio::sync::Mutex;
pub type StateDb = Arc<Mutex<Vec<Link>>>;
pub type EvalAstList = Arc<Mutex<Vec<Option<AST>>>>;

// Global state. Must be clone-able because it will be shared
// with multiple tasks.
#[derive(Clone, Debug)]
pub struct GlobalState {
    pub state_db: StateDb,
}

impl GlobalState {
    pub fn new(links: Vec<Link>) -> Self {
        Self {
            state_db: Arc::new(Mutex::new(links)),
        }
    }
}
