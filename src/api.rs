use crate::state::GlobalState;
use crate::{DeviceLink, link::Link};
use crate::{Eval, MAX_NUM_LINKS, ModbusSerialConfig, ModbusTcpConfig, Protocol, Tag, TagValue};
use axum::extract::rejection::JsonRejection;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use log::info;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufWriter;
use std::time::Duration;
use std::usize::MAX;
use tracing::instrument;

// Used as a link ID for the lookup.
#[derive(Serialize, Deserialize)]
pub struct LinkIdQuery {
    pub link_id: u32,
}

#[derive(Serialize, Deserialize)]
pub struct LinkProtocolReconfig {
    pub link_id: u32,
    pub protocol: String,
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

#[derive(Deserialize, Debug)]
pub struct EvalReconfigData {
    pub tag_info: TagIdQuery,
    pub tag_data: Eval,
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

        *locked_state = links.clone();

        let file = File::create("./CurrentConfig/current_config.json");
        if let Ok(file) = file {
            let mut writer = BufWriter::new(file);
            if let Ok(_) = serde_json::to_writer_pretty(&mut writer, &links) {
                info!("Success");
            } else {
                info!("error");
            }
        } else {
            info!("Could not create file");
        }
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

// Using a protocol details string to reconfigure the device.
pub async fn reconfig_device_protocol(
    State(state): State<GlobalState>,
    Json(config): Json<LinkProtocolReconfig>,
) -> Result<impl IntoResponse, StatusCode> {
    let fields: Vec<&str> = config.protocol.split(':').collect();

    if fields.len() == 5 {
        match fields[0] {
            "modbus" => {
                if fields[1] == "tcp" {
                    let ip = fields[2].to_string();
                    if let (Ok(port), Ok(slave)) =
                        (fields[3].parse::<usize>(), fields[4].parse::<u8>())
                    {
                        let tcp_config = ModbusTcpConfig { ip, port, slave };
                        let mut locked_state = state.state_db.lock().await;
                        for link in locked_state.iter_mut() {
                            match link {
                                Link::Device(link) => {
                                    if link.id == config.link_id as usize {
                                        link.protocol = Protocol::ModbusTcp(tcp_config);
                                        link.status = crate::LinkStatus::NeedsToReconnect;
                                        return Ok(StatusCode::OK);
                                    }
                                }
                                _ => {
                                    continue;
                                }
                            }
                        }
                    } else {
                        info!("Could not parse port or slave.");
                        return Err(StatusCode::NOT_FOUND);
                    }
                } else {
                    info!("Only TCP or RTU.");
                    return Err(StatusCode::NOT_FOUND);
                }
            }
            _ => {
                info!("Only Modbus.");
                return Err(StatusCode::NOT_FOUND);
            }
        }
    } else if fields.len() == 5 {
        match fields[0] {
            "modbus" => {
                if fields[1] == "rtu" {
                    let com = fields[2].to_string();
                    if let (Ok(baudrate), Ok(slave)) =
                        (fields[3].parse::<u32>(), fields[4].parse::<u8>())
                    {
                        let serial_config = ModbusSerialConfig {
                            com_port: com,
                            baudrate,
                            slave,
                            parity: crate::ParityType::None,
                            timeout: Duration::from_millis(2000),
                        };
                        let mut locked_state = state.state_db.lock().await;
                        for link in locked_state.iter_mut() {
                            match link {
                                Link::Device(link) => {
                                    if link.id == config.link_id as usize {
                                        link.protocol = Protocol::ModbusSerial(serial_config);
                                        link.status = crate::LinkStatus::NeedsToReconnect;
                                        return Ok(StatusCode::OK);
                                    }
                                }
                                _ => {
                                    continue;
                                }
                            }
                        }
                    } else {
                        info!("Could not parse baudrate or slave.");
                        return Err(StatusCode::NOT_FOUND);
                    }
                } else {
                    info!("Only TCP and RTU.");
                    return Err(StatusCode::NOT_FOUND);
                }
            }
            _ => {
                info!("Only Modbus.");
                return Err(StatusCode::NOT_FOUND);
            }
        }
    }
    info!("Wrong number of fields.");
    Err(StatusCode::NOT_FOUND)
}

pub async fn reconfig_eval(
    State(state): State<GlobalState>,
    //Json(config): Json<TagReconfigData>,
    payload: Result<Json<EvalReconfigData>, JsonRejection>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut locked_state = state.state_db.lock().await;

    match payload {
        Ok(config) => {
            for link in locked_state.iter_mut() {
                match link {
                    Link::Eval(link) => {
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
