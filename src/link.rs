use crate::device_link::*;
use anyhow::{Result, anyhow};

#[derive(Clone, Debug, PartialEq)]
pub enum Link {
    Device(DeviceLink),
    Eval,
    InputDb,
    MbServer,
}
#[derive(Clone, Debug, PartialEq)]
pub enum LinkStatus {
    Connected,
    Disconnected,
    Error,
}
