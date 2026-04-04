use serde::{Deserialize, Serialize};

use crate::TagValue;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub id: usize,
    pub tk: String,
    pub name: String,
    pub unit: String,
    pub description: String,
    pub enabled: bool,
    //#[serde(skip_deserializing)]
    pub value: TagValue,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputsLink {
    pub id: usize,
    pub tk: String,
    pub name: String,
    pub enabled: bool,
    pub tags: Vec<Input>,
    pub tags_count: usize,
}

impl Input {
    pub fn new(id: usize, tk: String, name: String) -> Self {
        Self {
            id,
            tk,
            name,
            unit: String::from("-"),
            description: String::from("Input tag."),
            enabled: true,
            value: TagValue::Real(0.0),
        }
    }
}

impl InputsLink {
    pub fn new(id: usize, tk: String, name: String, tags_count: usize) -> Self {
        let mut tags = Vec::with_capacity(tags_count);

        for i in 0..tags_count {
            let tag = Input::new(i, format!("{tk}:{i}"), format!("TAG_{i}"));
            tags.push(tag);
        }

        Self {
            id,
            tk,
            name,
            enabled: true,
            tags,
            tags_count,
        }
    }
}
