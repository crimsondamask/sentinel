use crate::LinkStatus;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::Duration};

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
}

pub enum DeviceLinkContext {
    ModbusContext(tokio_modbus::prelude::client::Context),
    S7Context,
    EipContext,
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
                    anyhow::bail!("Link context not comatible with tag address.")
                }
            },
            _ => {}
        }
        Ok(())
    }
    pub async fn write(&mut self, value: TagValue, ctx: &mut DeviceLinkContext) -> Result<()> {
        unimplemented!()
    }
}

impl DeviceLink {
    pub fn new(name: String, tk: String, id: usize, protocol: Protocol, n_tags: usize) -> Self {
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
            status: LinkStatus::Disconnected,
            error_message: String::from("Disconnected."),
            tags: tag_list,
            tag_count,
        }
    }

    pub async fn connect(&mut self) -> Result<DeviceLinkContext> {
        match &self.protocol {
            Protocol::ModbusTcp(config) => {
                let socket_address: SocketAddr =
                    format!("{}:{}", config.ip, config.port).parse()?;
                let ctx = tcp::connect(socket_address).await?;

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

    pub async fn poll(&mut self, ctx: &mut DeviceLinkContext) -> Result<()> {
        for tag in self.tags.iter_mut() {
            if tag.enabled {
                match tag.read(ctx).await {
                    Ok(_) => {}
                    Err(e) => {
                        tag.status = TagStatus::Error(format!("{}", e));
                    }
                }
            } else {
                anyhow::bail!("The Tag is not enabled.")
            }
        }
        Ok(())
    }

    pub async fn write_tags(
        &self,
        tags_to_write: Vec<Tag>,
        ctx: &mut DeviceLinkContext,
    ) -> Result<()> {
        unimplemented!()
    }
    pub fn reconfigure(&mut self, link_update: DeviceLink, ctx: &mut DeviceLinkContext) {
        // TODO
        // Need to do more checks.
        // Should the link be disconnected?
        *self = link_update;
    }
}
