use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Taxonomy {
    pub version: String,
    pub locked_levels: u8,
    pub groups: Vec<Group>,
    pub topics: Vec<Topic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub topics: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub id: serde_yaml::Value,
    pub name: String,
    #[serde(default)] pub short: Option<String>,
    #[serde(default)] pub children: Vec<SubTopic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTopic {
    pub id: serde_yaml::Value,
    pub name: String,
    #[serde(default)] pub hint: Option<String>,
}

pub fn load_from_file(path: &Path) -> Result<Taxonomy> {
    let s = std::fs::read_to_string(path)
        .with_context(|| format!("read taxonomy at {}", path.display()))?;
    let tax: Taxonomy = serde_yaml::from_str(&s).context("parse taxonomy yaml")?;
    Ok(tax)
}
