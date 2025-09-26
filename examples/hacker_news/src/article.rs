use serde::Deserialize;
use carbide::Identifiable;

#[derive(Identifiable, Clone, Debug, Deserialize)]
pub struct Article {
    pub id: u64,
    pub title: String,
    pub url: Option<String>,
    pub by: String,
    pub time: u64,
    pub descendants: Option<u64>,
    pub score: u64,
}