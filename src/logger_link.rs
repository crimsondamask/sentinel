use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;

use crate::AbstractTag;
use crate::Link;
use crate::LinkStatus;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InfluxDbInfo {
    pub url: String,
    pub token: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DataBase {
    InfluxDb(InfluxDbInfo),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LogTagInfo {
    pub link_id: usize,
    pub tag_id: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoggerLink {
    pub name: String,
    pub id: usize,
    pub database: DataBase,
    pub status: LinkStatus,
    pub tags: Vec<LogTagInfo>,
    pub log_delay_millis: usize,
}

impl LoggerLink {
    pub fn new(name: String, id: usize, num_tags: usize) -> Self {
        let mut tags: Vec<LogTagInfo> = Vec::new();
        for i in 0..num_tags {
            let log_tag = LogTagInfo {
                link_id: 0,
                tag_id: i,
            };

            tags.push(log_tag);
        }
        let influx_info = InfluxDbInfo {
            url: String::new(),
            token: String::new(),
        };

        Self {
            name,
            id,
            database: DataBase::InfluxDb(influx_info),
            status: LinkStatus::Normal,
            tags,
            log_delay_millis: 1000,
        }
    }
    pub fn log(&self, links: &Vec<Link>) {
        let mut tags_to_log: Vec<AbstractTag> = Vec::new();
        // Logged tags lookup.
        for tag in &self.tags {
            for link in links {
                match link {
                    Link::Device(link) => {
                        if tag.link_id == link.id {
                            for link_tag in &link.tags {
                                if link_tag.id == tag.tag_id {
                                    tags_to_log.push(AbstractTag::DeviceTag(link_tag.clone()));
                                }
                            }
                        }
                    }
                    Link::Eval(link) => {
                        if tag.link_id == link.id {
                            for link_tag in &link.tags {
                                if link_tag.id == tag.tag_id {
                                    tags_to_log.push(AbstractTag::EvalTag(link_tag.clone()));
                                }
                            }
                        }
                    }
                    Link::Inputs(link) => {
                        if tag.link_id == link.id {
                            for link_tag in &link.tags {
                                if link_tag.id == tag.tag_id {
                                    tags_to_log.push(AbstractTag::InputTag(link_tag.clone()));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        
    }
}
