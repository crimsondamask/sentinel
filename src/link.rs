use crate::{InputsLink, device_link::*, eval_link::*};
use serde::{Deserialize, Serialize};

pub const MAX_NUM_LINKS: usize = 5;

#[derive(Clone, Debug, Serialize, PartialEq, Deserialize)]
//#[serde(tag = "link_type")]
pub enum Link {
    Device(DeviceLink),
    Eval(EvalLink),
    Inputs(InputsLink),
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
