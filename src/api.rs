use crate::state::GlobalState;
use crate::{DeviceLink, link::Link};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use log::info;
use serde::Deserialize;

// Used as a link ID for the lookup.
#[derive(Deserialize)]
pub struct LinkIdQuery {
    pub link_id: u32,
}

#[derive(Deserialize)]
pub struct TagIdQuery {
    pub link_id: u32,
    pub tag_id: u32,
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

// Reconfig the Device Link.
// This is a post request.
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
