use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum HNItem {
    Story {
        author: String,
        children: Vec<HNItem>,
        created_at: DateTime<Utc>,
        id: u64,
        title: String,
        url: String,
    },
    Job {
        author: String,
        children: Vec<HNItem>,
        created_at: DateTime<Utc>,
        id: u64,
        title: String,
        url: String,
    },
    Comment {
        author: String,
        children: Vec<HNItem>,
        created_at: DateTime<Utc>,
        id: u64,
        text: String,
    }
}