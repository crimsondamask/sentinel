use crate::{Input, Link, LinkStatus, Tag, TagStatus, TagValue};
use anyhow::Result;
use rhai::{AST, Engine, EvalAltResult, Scope};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EvalInputVarType {
    DeviceTag(Tag),
    InputTag(Input),
    EvalTag(Eval),
}
// Wrapper for the var type.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvalInputVar {
    pub name: String,
    pub link_id: usize,
    pub tag_id: usize,
    pub value: TagValue,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Eval {
    pub id: usize,
    pub tk: String,
    pub name: String,
    pub enabled: bool,
    // List of variables included in the formula.
    // The eval function will use this list to expand
    // the formula before evaluating.
    pub vars: Vec<EvalInputVar>,
    // Formula that might include variables.
    pub formula: String,
    pub value: TagValue,
    #[serde(skip_deserializing)]
    pub status: TagStatus,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvalLink {
    pub id: usize,
    pub tk: String,
    pub name: String,
    pub enabled: bool,
    pub tags: Vec<Eval>,
    pub tag_count: usize,
    #[serde(skip)]
    pub status: LinkStatus,
}

impl Eval {
    pub fn new(id: usize, tk: String, name: String) -> Self {
        let mut vars = Vec::new();
        for i in 0..5 {
            let var = EvalInputVar {
                name: format!("x{i}"),
                link_id: 1,
                tag_id: i,
                value: TagValue::Real(0.0),
            };

            vars.push(var);
        }
        Self {
            id,
            tk,
            name,
            enabled: true,
            vars,
            formula: String::from("5.0 + 5.0"),
            value: TagValue::Real(0.0),
            status: TagStatus::Normal,
        }
    }
    pub fn evaluate(&mut self, links: &Vec<Link>) {
        if self.enabled {
            // Expand the variables using the links list.
            for var in self.vars.iter_mut() {
                for link in links.iter() {
                    match link {
                        Link::Device(link) => {
                            if link.id == var.link_id {
                                for tag in &link.tags {
                                    if tag.id == var.tag_id {
                                        if !tag.enabled {
                                            self.status = TagStatus::Error(
                                                "Variable in the formula is not enabled."
                                                    .to_string(),
                                            );
                                            return;
                                        }
                                        var.value = tag.value.clone();
                                        break;
                                    }
                                }
                            }
                        }
                        Link::Inputs(link) => {
                            if link.id == var.link_id {
                                for tag in &link.tags {
                                    if tag.id == var.tag_id {
                                        if !tag.enabled {
                                            self.status = TagStatus::Error(
                                                "Variable in the formula is not enabled."
                                                    .to_string(),
                                            );
                                            return;
                                        }
                                        var.value = tag.value.clone();
                                        break;
                                    }
                                }
                            }
                        }
                        Link::Eval(link) => {
                            if link.id == var.link_id {
                                for tag in &link.tags {
                                    if tag.id == var.tag_id {
                                        if !tag.enabled {
                                            self.status = TagStatus::Error(
                                                "Variable in the formula is not enabled."
                                                    .to_string(),
                                            );
                                            return;
                                        }
                                        var.value = tag.value.clone();
                                        break;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            // Create the eval engine and push variables to its scope.
            let engine = Engine::new();
            let mut scope = Scope::new();

            for var in self.vars.iter_mut() {
                match var.value {
                    TagValue::Real(v) => {
                        scope.push(&var.name, v as f32);
                    }
                    TagValue::Int(v) => {
                        scope.push(&var.name, v as i64);
                    }
                    TagValue::Bit(v) => {
                        scope.push(&var.name, v);
                    }
                    TagValue::Dint(v) => {
                        scope.push(&var.name, v as i64);
                    }
                }
            }

            // Evaluate the formula.
            match self.value {
                TagValue::Real(_) => {
                    let res = engine.eval_with_scope::<f32>(&mut scope, &self.formula);
                    match res {
                        Ok(res) => {
                            self.value = TagValue::Real(res as f32);
                            self.status = TagStatus::Normal;
                        }
                        Err(e) => self.status = TagStatus::Error(e.to_string()),
                    }
                }
                TagValue::Int(_) => {
                    let res = engine.eval_with_scope::<i64>(&mut scope, &self.formula);
                    match res {
                        Ok(res) => {
                            self.status = TagStatus::Normal;
                            self.value = TagValue::Int(res as u16);
                        }
                        Err(e) => self.status = TagStatus::Error(e.to_string()),
                    }
                }
                TagValue::Dint(_) => {
                    let res = engine.eval_with_scope::<i64>(&mut scope, &self.formula);
                    match res {
                        Ok(res) => {
                            self.value = TagValue::Dint(res as u32);
                            self.status = TagStatus::Normal;
                        }
                        Err(e) => self.status = TagStatus::Error(e.to_string()),
                    }
                }
                TagValue::Bit(_) => {
                    let res = engine.eval_with_scope::<bool>(&mut scope, &self.formula);
                    match res {
                        Ok(res) => {
                            self.value = TagValue::Bit(res);
                            self.status = TagStatus::Normal;
                        }
                        Err(e) => self.status = TagStatus::Error(e.to_string()),
                    }
                }
            }
        }
    }
}

impl EvalLink {
    pub fn new(id: usize, name: String, tag_count: usize) -> Self {
        let mut tags: Vec<Eval> = Vec::with_capacity(tag_count);

        for i in 0..tag_count {
            let tag_tk = format!("EVAL{}:{:03}", id, i);
            let tag_name = format!("EVAL{i}");
            let eval = Eval::new(i, tag_tk, tag_name);
            tags.push(eval);
        }

        Self {
            id,
            tk: format!("EVAL{}", id),
            name,
            enabled: true,
            tags,
            tag_count,
            status: LinkStatus::Normal,
        }
    }
}
