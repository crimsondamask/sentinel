use axum::extract::rejection::JsonRejection;
use std::usize::MAX;
use tracing::instrument;

use crate::state::GlobalState;
use crate::{DeviceLink, link::Link};
use crate::{MAX_NUM_LINKS, Tag, TagValue};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use log::info;
use serde::{Deserialize, Serialize};

// Used as a link ID for the lookup.
#[derive(Serialize, Deserialize)]
pub struct LinkIdQuery {
    pub link_id: u32,
}

#[derive(Deserialize, Debug)]
pub struct TagIdQuery {
    pub link_id: u32,
    pub tag_id: u32,
}

#[derive(Deserialize, Debug)]
pub struct TagReconfigData {
    pub tag_info: TagIdQuery,
    pub tag_data: Tag,
}

#[derive(Deserialize)]
pub struct TagWriteData {
    pub tag_info: TagIdQuery,
    pub tag_value: TagValue,
}

pub async fn get_links_config(
    State(state): State<GlobalState>,
) -> Result<impl IntoResponse, StatusCode> {
    let locked_state = state.state_db.lock().await;
    Ok(Json(locked_state.clone()))
}

pub async fn reconfig_links(
    State(state): State<GlobalState>,
    Json(mut links): Json<Vec<Link>>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut locked_state = state.state_db.lock().await;

    if links.len() > MAX_NUM_LINKS {
        return Err(StatusCode::NOT_FOUND);
    } else {
        for link in links.iter_mut() {
            match link {
                Link::Device(link) => {
                    link.status = crate::LinkStatus::NeedsToReconnect;
                }
                Link::Eval(link) => {
                    link.status = crate::LinkStatus::PendingTagReconfig;
                }
                _ => {}
            }
        }

        *locked_state = links;
        return Ok(StatusCode::OK);
    }
}
// Return the whole config and data of the link device
// specified by the link_id
pub async fn get_device_link_config(
    State(state): State<GlobalState>,
    Json(link_id): Json<LinkIdQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let locked_state = state.state_db.lock().await;

    for link in locked_state.iter() {
        match link {
            Link::Device(link) => {
                if link.id as u32 == link_id.link_id {
                    info!("Found device");
                    return Ok(Json(Link::Device(link.clone())));
                }
            }
            _ => {
                continue;
            }
        }
    }
    Err(StatusCode::NOT_FOUND)
}

// Return the whole config and data of the link device
// specified by the link_id
pub async fn get_tag_config(
    State(state): State<GlobalState>,
    Json(tag_id): Json<TagIdQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let locked_state = state.state_db.lock().await;

    for link in locked_state.iter() {
        match link {
            Link::Device(link) => {
                if link.id as u32 == tag_id.link_id {
                    for tag in link.tags.iter() {
                        if tag.id as u32 == tag_id.tag_id {
                            info!("Found tag");
                            return Ok(Json(tag.clone()));
                        }
                    }
                }
            }
            _ => {
                // TODO check for other types of tags.
                continue;
            }
        }
    }
    Err(StatusCode::NOT_FOUND)
}

/*
Reconfig the Device Link.
This is a post request.
*/
pub async fn reconfig_device_link(
    State(state): State<GlobalState>,
    Json(config): Json<DeviceLink>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut locked_state = state.state_db.lock().await;

    for (_, link) in locked_state.iter_mut().enumerate() {
        match link {
            Link::Device(link) => {
                if link.id == config.id {
                    info!("Reconfigured device: {}.", link.id);
                    link.reconfigure(config);
                    //locked_state[i] = Link::Device(config);
                    return Ok(StatusCode::OK);
                }
            }
            _ => {
                continue;
            }
        }
    }
    Err(StatusCode::NOT_FOUND)
}

/*
 * Reconfig a specifig tag given the tag info in the post request.
 * This is more granular than reconfig_device_ling.
 */
pub async fn reconfig_device_tag(
    State(state): State<GlobalState>,
    //Json(config): Json<TagReconfigData>,
    payload: Result<Json<TagReconfigData>, JsonRejection>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut locked_state = state.state_db.lock().await;

    match payload {
        Ok(config) => {
            for link in locked_state.iter_mut() {
                match link {
                    Link::Device(link) => {
                        if link.id as u32 == config.tag_info.link_id {
                            for tag in link.tags.iter_mut() {
                                if tag.id as u32 == config.tag_info.tag_id {
                                    info!("Found tag to reconfigure.");
                                    *tag = config.tag_data.clone();
                                    link.status = crate::LinkStatus::PendingTagReconfig;
                                    return Ok(Json(tag.clone()));
                                }
                            }
                        }
                    }
                    _ => {
                        // TODO check for other types of tags.
                        continue;
                    }
                }
            }
        }
        Err(e) => {
            info!("{}", e);
            return Err(StatusCode::NOT_FOUND);
        }
    }
    info!("Could not find tag to reconfigure.");
    Err(StatusCode::NOT_FOUND)
}

pub async fn write_link_tag(
    State(state): State<GlobalState>,
    Json(data): Json<TagWriteData>,
) -> Result<StatusCode, StatusCode> {
    let mut locked_state = state.state_db.lock().await;

    for link in locked_state.iter_mut() {
        match link {
            Link::Device(link) => {
                if link.id as u32 == data.tag_info.link_id {
                    for tag in link.tags.iter_mut() {
                        if tag.id as u32 == data.tag_info.tag_id {
                            info!("Found tag to write. Value: {:?}", &data.tag_value);
                            tag.pending_write = Some(data.tag_value);

                            return Ok(StatusCode::OK);
                        }
                    }
                }
            }
            Link::Inputs(link) => {
                if link.id as u32 == data.tag_info.link_id {
                    for tag in link.tags.iter_mut() {
                        if tag.id as u32 == data.tag_info.tag_id {
                            info!("Found tag to write. Value: {:?}", &data.tag_value);
                            tag.value = data.tag_value;
                            return Ok(StatusCode::OK);
                        }
                    }
                }
            }
            _ => {
                // TODO check for other types of tags.
                continue;
            }
        }
    }
    info!("Could not find tag to write.");
    Err(StatusCode::NOT_FOUND)
}
