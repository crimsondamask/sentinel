use log::info;
use std::time::Duration;

use crate::device_link::Tag;
use anyhow::Result;

use crate::GlobalState;

pub enum TaskType {
    DeviceLink,
    Logging,
    Eval,
}

pub enum TaskMessage {
    //LinkPollResult,
    LinkConfig,
    DeviceWrite(Tag),
}

pub struct Task {
    pub id: usize,
    pub task_type: TaskType,
    pub state: GlobalState,
}

impl Task {
    pub fn new(task_type: TaskType, state: GlobalState, id: usize) -> Self {
        Self {
            id,
            task_type,
            state,
        }
    }
}

pub async fn handle_link_task(task: Task) {
    loop {
        tokio::time::sleep(Duration::from_millis(1000)).await;
        info!("Poll from task: {}", task.id);
    }
}
pub async fn handle_logging_task(task: Task) {
    loop {
        unimplemented!()
    }
}
pub async fn handle_eval_task(task: Task) {
    loop {
        unimplemented!()
    }
}

pub fn spawn(task: Task) -> Result<()> {
    match task.task_type {
        TaskType::DeviceLink => {
            tokio::spawn(handle_link_task(task));
        }
        TaskType::Logging => {
            tokio::spawn(handle_logging_task(task));
        }
        TaskType::Eval => {
            tokio::spawn(handle_eval_task(task));
        }
    }
    Ok(())
}
