use crate::device_link::*;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Link {
    Device(DeviceLink),
    Eval,
    InputDb,
    MbServer,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LinkStatus {
    Normal,
    NeedsToReconnect,
    Error(String),
}
