use anyhow::Result;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::StateDb;

pub enum TaskType {
    DeviceLink,
    Logging,
    Eval,
}

pub enum TaskMessage {
    LinkPollResult,
    LinkConfig,
    LinkWrite,
}

pub struct Task {
    pub task_type: TaskType,
    pub state: StateDb,
    pub sender: Sender<TaskMessage>,
    pub receiver: Receiver<TaskMessage>,
}

impl Task {
    pub fn new(
        &mut self,
        task_type: TaskType,
        state: StateDb,
        sender: Sender<TaskMessage>,
        receiver: Receiver<TaskMessage>,
    ) -> Self {
        Self {
            task_type,
            state,
            sender,
            receiver,
        }
    }
}

pub async fn handle_link_task(task: Task) {
    loop {
        unimplemented!()
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
