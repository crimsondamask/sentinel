use log::info;
use std::time::Duration;
use tokio::time::{self};

use crate::{DeviceLink, EvalLink, Link, LinkStatus, ModbusTcpConfig, Protocol, device_link::Tag};
use anyhow::Result;

use crate::GlobalState;

pub enum TaskType {
    DeviceLink,
    Inputs,
    Logging,
    Eval,
    ConfigHash,
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
    let mut default_link: DeviceLink = DeviceLink::new(
        "new_link".to_string(),
        "PLC".to_string(),
        0,
        Protocol::ModbusTcp(ModbusTcpConfig::new("127.0.0.1".to_string(), 5502)),
        1000,
        500,
    );

    loop {
        info!("Starting the loop");
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

                default_link.status = LinkStatus::Normal;
                /*
                Handle the connected link context
                inside a loop.
                This traps the execution in an infinite loop until an error occurs.
                */
                let mut interval = time::interval(Duration::from_millis(500));
                interval.tick().await;

                // The polling loop:

                loop {
                    // Poll the device.
                    default_link.poll(&mut link_context).await;

                    // Lock the state and update the link with the polled values.
                    {
                        let locked_state = &mut task.state.state_db.lock().await[task.id];
                        match locked_state {
                            Link::Device(link) => match link.status {
                                LinkStatus::Normal => {
                                    *link = default_link.clone();
                                }
                                LinkStatus::PendingTagReconfig => {
                                    link.status = LinkStatus::Normal;
                                    default_link = link.clone();
                                    // If receiving a tag update, we don't wait.
                                    continue;
                                }
                                LinkStatus::NeedsToReconnect => {
                                    link.status = LinkStatus::Normal;
                                    default_link = link.clone();
                                    info!("Needs to reconnect.");
                                    break;
                                }
                                LinkStatus::Error(_) => {
                                    link.status = LinkStatus::Normal;
                                    default_link = link.clone();
                                    info!("Error! Needs to reconnect.");
                                    break;
                                }
                            },
                            _ => {}
                        }
                    }

                    // Wait
                    interval.tick().await;
                }
            }
            Err(e) => {
                info!("Failed to connect: {e}. Task: {}", task.id);
                let mut locked_state = task.state.state_db.lock().await;
                match &mut locked_state[task.id] {
                    Link::Device(link) => {
                        link.status = LinkStatus::Error(e.to_string());
                    }
                    _ => {}
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(2000)).await;
    }
}
pub async fn handle_logging_task(_task: Task) {
    loop {
        unimplemented!()
    }
}
pub async fn handle_inputs_task(_task: Task) {
    loop {}
}
pub async fn handle_hash_task(_task: Task) {
    loop {}
}
pub async fn handle_eval_task(task: Task) {
    // Store all the eval ASTs here and only update them if triggered by PendingTagReconfig.
    let mut default_link = EvalLink::new(task.id, "EVAL".to_owned(), 1000);
    let mut links_list = Vec::new();
    let mut interval = time::interval(Duration::from_millis(500));

    interval.tick().await;

    loop {
        // Lock the mutex and update.
        let now = std::time::Instant::now();
        interval.tick().await;
        {
            let mut locked_state = task.state.state_db.lock().await;
            match &mut locked_state[task.id] {
                Link::Eval(config) => {
                    config.status = LinkStatus::Normal;
                    default_link = config.clone();
                }
                _ => {}
            }

            links_list = locked_state.clone();
        }

        for eval in default_link.tags.iter_mut() {
            eval.evaluate(&links_list);
        }

        let _duration = now.elapsed();

        //info!("Evaluation Elapsed time: {}", duration.as_millis());
        {
            // Lock the mutex and update.
            let locked_link = &mut task.state.state_db.lock().await[task.id];
            match locked_link {
                Link::Eval(link) => match link.status {
                    LinkStatus::PendingTagReconfig => {
                        continue;
                    }
                    _ => {
                        *locked_link = Link::Eval(default_link.clone());
                    }
                },
                _ => {}
            }
        }
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
        TaskType::Inputs => {
            tokio::spawn(handle_inputs_task(task));
        }
        TaskType::ConfigHash => {
            tokio::spawn(handle_hash_task(task));
        }
    }
    Ok(())
}
