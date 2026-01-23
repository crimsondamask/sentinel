use chrono::NaiveDateTime;
use log::info;
use std::time::Duration;

use crate::{DeviceLink, Link, ModbusTcpConfig, Protocol, device_link::Tag};
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
        let mut default_link: DeviceLink = DeviceLink::new(
            "new_link".to_string(),
            "PLC".to_string(),
            0,
            Protocol::ModbusTcp(ModbusTcpConfig::new("127.0.0.1".to_string(), 5502)),
            1000,
        );

        // We make sure that we only lock the Mutex to update the default link
        // and release the lock.
        {
            let locked_state = task.state.state_db.lock().await;
            match &locked_state[task.id] {
                Link::Device(config) => {
                    default_link = config.clone();
                }
                _ => {}
            }
        }
        match default_link.connect().await {
            Ok(mut link_context) => {
                info!(
                    "Connection successful from task: {}. Device: {}",
                    task.id, default_link.name
                );
                /*
                Handle the connected link context
                inside a loop
                */
                loop {
                    default_link.poll(&mut link_context).await;
                }
            }
            Err(e) => {
                info!("Failed to connect: {e}. Task: {}", task.id);
            }
        }
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
