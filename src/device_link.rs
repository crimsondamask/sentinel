use crate::LinkStatus;
use anyhow::{Result, anyhow};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::Duration};
use tracing::info;

use tokio_modbus::prelude::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ParityType {
    Even,
    Odd,
    None,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModbusTcpConfig {
    pub ip: String,
    pub port: usize,
}

impl ModbusTcpConfig {
    pub fn new(ip: String, port: usize) -> Self {
        Self { ip, port }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModbusSerialConfig {
    pub com_port: String,
    pub baudrate: u32,
    pub parity: ParityType,
    pub timeout: Duration,
}

impl ModbusSerialConfig {
    // TODO!
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct S7Config {
    pub ip: String,
    pub rack: usize,
    pub slot: usize,
}

// TODO
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EipConfig {}
// TODO
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OpcUaConfig {}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Protocol {
    ModbusTcp(ModbusTcpConfig),
    ModbusSerial(ModbusSerialConfig),
    S7(S7Config),
    Eip(EipConfig),
    OpcUa(OpcUaConfig),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ModbusRegister {
    Holding(u16),
    Input(u16),
    Coil(u16),
    Status(u16),
}

// TODO
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct S7Addr {
    db: usize,
    offset: usize,
    start_bit: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TagAddress {
    ModbusAddr(ModbusRegister),
    S7Addr(S7Addr),
    EipAddr,
    OpcUaAddr,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TagValue {
    Int(u16),
    Dint(u32),
    Real(f32),
    Bit(bool),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TagStatus {
    Normal,
    Error(String),
    Warn,
    Alarm,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub id: usize,
    pub tk: String,
    pub name: String,
    pub enabled: bool,
    pub address: TagAddress,
    pub value: TagValue,
    pub status: TagStatus,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeviceLink {
    pub id: usize,
    pub tk: String,
    pub name: String,
    pub enabled: bool,
    pub protocol: Protocol,
    pub status: LinkStatus,
    pub error_message: String,
    pub tags: Vec<Tag>,
    pub tag_count: usize,
    pub last_poll_time: NaiveDateTime,
    pub poll_wait_duration: u64,
}

pub enum DeviceLinkContext {
    ModbusContext(tokio_modbus::prelude::client::Context),
    S7Context,
    EipContext,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TagWriteInfo {
    pub link_id: usize,
    pub tag_id: usize,
    pub value: TagValue,
}
impl Tag {
    pub fn new(name: String, tk: String, id: usize, address: TagAddress) -> Self {
        Self {
            id,
            tk,
            name,
            address,
            enabled: false,
            value: TagValue::Real(0.0),
            status: TagStatus::Error(String::from("Initiated.")),
        }
    }
    pub async fn read(&mut self, ctx: &mut DeviceLinkContext) -> Result<()> {
        self.status = TagStatus::Normal;
        match ctx {
            DeviceLinkContext::ModbusContext(ctx) => match &self.address {
                TagAddress::ModbusAddr(addr) => match addr {
                    ModbusRegister::Holding(reg) => {
                        match self.value {
                            TagValue::Int(_) => {
                                let data = ctx.read_holding_registers(*reg, 1).await?;
                                self.value = TagValue::Int(data?[0]);
                            }
                            TagValue::Dint(_) => {
                                // TODO
                                todo!()
                            }
                            TagValue::Real(_) => {
                                let data = ctx.read_holding_registers(*reg, 2).await??;
                                let data_32 = ((data[0] as u32) << 16) | (data[1] as u32);
                                let data = f32::from_bits(data_32);
                                self.value = TagValue::Real(data);
                            }
                            TagValue::Bit(_) => {
                                // TODO
                                todo!()
                            }
                        }
                    }
                    ModbusRegister::Input(reg) => match self.value {
                        TagValue::Int(_) => {
                            let data = ctx.read_input_registers(*reg, 1).await?;
                            self.value = TagValue::Int(data?[0]);
                        }
                        TagValue::Dint(_) => {
                            // TODO
                            todo!()
                        }
                        TagValue::Real(_) => {
                            let data = ctx.read_input_registers(*reg, 2).await??;
                            let data_32 = ((data[0] as u32) << 16) | (data[1] as u32);
                            let data = f32::from_bits(data_32);
                            self.value = TagValue::Real(data);
                        }
                        TagValue::Bit(_) => {
                            anyhow::bail!("Value type is incompatible with register type.");
                        }
                    },
                    ModbusRegister::Coil(reg) => {
                        todo!()
                    }
                    ModbusRegister::Status(reg) => {
                        todo!()
                    }
                },
                _ => {
                    anyhow::bail!("Link context not compatible with tag address.")
                }
            },
            _ => {}
        }
        Ok(())
    }

    // TODO
    pub async fn write(&mut self, ctx: &mut DeviceLinkContext, value: TagValue) -> Result<()> {
        self.status = TagStatus::Normal;
        match ctx {
            DeviceLinkContext::ModbusContext(ctx) => match &self.address {
                TagAddress::ModbusAddr(addr) => match addr {
                    ModbusRegister::Holding(reg) => {
                        match self.value {
                            TagValue::Int(_) => match value {
                                TagValue::Int(v) => {
                                    let _data = ctx.write_single_register(*reg, v).await?;
                                }
                                _ => {
                                    anyhow::bail!("Value type is incompatible with Tag type.");
                                }
                            },
                            TagValue::Dint(_) => match value {
                                TagValue::Dint(v) => {
                                    let bytes_array = v.to_le_bytes();
                                    let bytes = bytes_array.split_at(1);
                                    let h_bytes = bytes.0.try_into()?;
                                    let l_bytes = bytes.1.try_into()?;
                                    let u16_high = u16::from_le_bytes(h_bytes);
                                    let u16_low = u16::from_le_bytes(l_bytes);
                                    let data_to_write = [u16_high, u16_low];
                                    let _data = ctx
                                        .write_multiple_registers(*reg, &data_to_write)
                                        .await??;
                                }
                                _ => {
                                    anyhow::bail!("Value type is incompatible with Tag type.");
                                }
                            },
                            TagValue::Real(_) => match value {
                                TagValue::Real(v) => {
                                    let bytes_array = v.to_le_bytes();
                                    let bytes = bytes_array.split_at(1);
                                    let h_bytes = bytes.0.try_into()?;
                                    let l_bytes = bytes.1.try_into()?;
                                    let u16_high = u16::from_le_bytes(h_bytes);
                                    let u16_low = u16::from_le_bytes(l_bytes);
                                    let data_to_write = [u16_high, u16_low];
                                    let _data = ctx
                                        .write_multiple_registers(*reg, &data_to_write)
                                        .await??;
                                }
                                _ => {
                                    anyhow::bail!("Value type is incompatible with Tag type.");
                                }
                            },
                            TagValue::Bit(_) => {
                                // TODO
                                todo!()
                            }
                        }
                    }
                    ModbusRegister::Input(reg) => {
                        match self.value {
                            TagValue::Int(_) => match value {
                                TagValue::Int(v) => {
                                    let _data = ctx.write_single_register(*reg, v).await?;
                                }
                                _ => {
                                    anyhow::bail!("Value type is incompatible with Tag type.");
                                }
                            },
                            TagValue::Dint(_) => match value {
                                TagValue::Dint(v) => {
                                    let bytes_array = v.to_le_bytes();
                                    let bytes = bytes_array.split_at(1);
                                    let h_bytes = bytes.0.try_into()?;
                                    let l_bytes = bytes.1.try_into()?;
                                    let u16_high = u16::from_le_bytes(h_bytes);
                                    let u16_low = u16::from_le_bytes(l_bytes);
                                    let data_to_write = [u16_high, u16_low];
                                    let _data = ctx
                                        .write_multiple_registers(*reg, &data_to_write)
                                        .await??;
                                }
                                _ => {
                                    anyhow::bail!("Value type is incompatible with Tag type.");
                                }
                            },
                            TagValue::Real(_) => match value {
                                TagValue::Real(v) => {
                                    let bytes_array = v.to_le_bytes();
                                    let bytes = bytes_array.split_at(1);
                                    let h_bytes = bytes.0.try_into()?;
                                    let l_bytes = bytes.1.try_into()?;
                                    let u16_high = u16::from_le_bytes(h_bytes);
                                    let u16_low = u16::from_le_bytes(l_bytes);
                                    let data_to_write = [u16_high, u16_low];
                                    let _data = ctx
                                        .write_multiple_registers(*reg, &data_to_write)
                                        .await??;
                                }
                                _ => {
                                    anyhow::bail!("Value type is incompatible with Tag type.");
                                }
                            },
                            TagValue::Bit(_) => {
                                // TODO
                                todo!()
                            }
                        }
                    }
                    ModbusRegister::Coil(reg) => {
                        todo!()
                    }
                    ModbusRegister::Status(reg) => {
                        todo!()
                    }
                },
                _ => {
                    anyhow::bail!("Link context not compatible with tag address.")
                }
            },
            _ => {}
        }
        Ok(())
    }
}

impl DeviceLink {
    pub fn new(
        name: String,
        tk: String,
        id: usize,
        protocol: Protocol,
        n_tags: usize,
        poll_wait_duration: u64,
    ) -> Self {
        let tag_count: usize = n_tags;
        let mut tag_list: Vec<Tag> = Vec::with_capacity(tag_count);

        let address: TagAddress = match protocol {
            Protocol::ModbusTcp(_) => TagAddress::ModbusAddr(ModbusRegister::Holding(0)),
            Protocol::ModbusSerial(_) => TagAddress::ModbusAddr(ModbusRegister::Holding(0)),
            Protocol::S7(_) => TagAddress::S7Addr(S7Addr {
                db: 1,
                offset: 0,
                start_bit: 0,
            }),
            // TODO
            Protocol::Eip(_) => todo!(),
            // TODO
            Protocol::OpcUa(_) => todo!(),
        };
        for i in 0..tag_count {
            let tag = Tag::new(
                format!("TAG{i}"),
                format!("LK{}:{:03}", id, i),
                i,
                address.clone(),
            );

            tag_list.push(tag);
        }
        Self {
            id,
            tk: format!("{}{}", tk, id),
            name,
            enabled: false,
            protocol,
            status: LinkStatus::Error("Disconnected".to_string()),
            error_message: String::from("Disconnected."),
            tags: tag_list,
            tag_count,
            last_poll_time: NaiveDateTime::default(),
            poll_wait_duration,
        }
    }

    pub async fn connect(&mut self) -> Result<DeviceLinkContext> {
        match &self.protocol {
            Protocol::ModbusTcp(config) => {
                let socket_address: SocketAddr =
                    format!("{}:{}", config.ip, config.port).parse()?;
                let ctx = tcp::connect(socket_address).await?;

                self.status = LinkStatus::Normal;
                Ok(DeviceLinkContext::ModbusContext(ctx))
            }
            Protocol::ModbusSerial(config) => {
                todo!()
            }
            Protocol::S7(config) => {
                todo!()
            }
            Protocol::Eip(config) => {
                todo!()
            }
            Protocol::OpcUa(config) => {
                todo!()
            }
        }
    }

    pub async fn poll(&mut self, ctx: &mut DeviceLinkContext) {
        // Reset the link status.
        self.status = LinkStatus::Normal;
        for tag in self.tags.iter_mut() {
            if tag.enabled {
                match tag.read(ctx).await {
                    Ok(_) => {}
                    Err(e) => {
                        tag.status = TagStatus::Error(format!("{}", e));
                        self.status = LinkStatus::Error(format!(
                            "Reading failed at Tag: {}. Error: {}",
                            tag.id, e
                        ));
                    }
                }
            } else {
                tag.status = TagStatus::Error(format!("The Tag is not enabled."));
            }
        }
        self.last_poll_time = chrono::Local::now().naive_local();
    }

    pub async fn write_tag(
        &mut self,
        tag_to_write: &TagWriteInfo,
        ctx: &mut DeviceLinkContext,
    ) -> Result<()> {
        for tag in self.tags.iter_mut() {
            if tag.id == tag_to_write.tag_id {
                tag.write(ctx, tag_to_write.value.clone()).await?;
            }
        }
        Ok(())
    }
    pub fn reconfigure(&mut self, link_update: DeviceLink) {
        // TODO
        // Need to do more checks.
        // Should the link be disconnected?
        *self = link_update;
    }
}
