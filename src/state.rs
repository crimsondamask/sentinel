use crate::Link;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type StateDb = Arc<Mutex<Vec<Link>>>;
