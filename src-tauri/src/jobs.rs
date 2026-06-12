use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobsFile {
    pub version: String,
    pub jobs: Vec<Job>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub title: String,
    #[serde(default)] pub company: String,
    #[serde(default)] pub location: String,
    #[serde(default)] pub level: String,
    #[serde(default)] pub track: String,
    #[serde(default)] pub jd: String,
    #[serde(default)] pub hard_requirements: Vec<String>,
    #[serde(default)] pub relevant_topic_ids: Vec<String>,
    #[serde(default)] pub cherry_picked_cards: Vec<String>,
    #[serde(default)] pub my_anchors: Vec<String>,
    #[serde(default)] pub notes: String,
}

pub fn load_from_file(path: &Path) -> Result<JobsFile> {
    let s = std::fs::read_to_string(path)
        .with_context(|| format!("read jobs file at {}", path.display()))?;
    let file: JobsFile = serde_yaml::from_str(&s).context("parse jobs yaml")?;
    Ok(file)
}
