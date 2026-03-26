use crate::{InputsLink, device_link::*};
use serde::{Deserialize, Serialize};

pub const MAX_NUM_LINKS: usize = 5;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
//#[serde(tag = "link_type")]
pub enum Link {
    Device(DeviceLink),
    Eval,
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

// TODO
// define the link interfaces here.
impl Link {}
