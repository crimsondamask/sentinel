use crate::{Input, InputsLink, LoggerLink, device_link::*, eval_link::*};
use serde::{Deserialize, Serialize};

pub const MAX_NUM_LINKS: usize = 5;

// Helper type to use for the logger tag list.
#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
pub enum AbstractTag {
    DeviceTag(Tag),
    InputTag(Input),
    EvalTag(Eval),
}
#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
pub enum Link {
    Device(DeviceLink),
    Eval(EvalLink),
    Inputs(InputsLink),
    Logger(LoggerLink),
    MbServer,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LinkStatus {
    Normal,
    NeedsToReconnect,
    PendingTagReconfig,
    Error(String),
}

impl Default for LinkStatus {
    fn default() -> Self {
        Self::Normal
    }
}

// TODO
// define the link interfaces here.
impl Link {}
