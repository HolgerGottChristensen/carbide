use serde::Deserialize;
use carbide::Id;

#[derive(Id, Clone, Debug, Deserialize)]
pub struct Article {
    #[id] pub id: u64,
    pub title: String,
    pub url: Option<String>,
    pub by: String,
    pub time: u64,
    pub descendants: Option<u64>,
    pub score: u64,
}
