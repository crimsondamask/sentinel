use crate::Link;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type StateDb = Arc<Mutex<Vec<Link>>>;

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
