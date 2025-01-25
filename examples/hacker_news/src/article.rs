use serde::Deserialize;
use carbide::controls::Identifiable;

#[derive(Deserialize, Clone, Debug)]
pub struct Article {
    pub id: u64,
    pub title: String,
    pub url: Option<String>,
    pub by: String,
    pub time: u64,
    pub descendants: Option<u64>,
    pub score: u64,
}

impl Identifiable<u64> for Article {
    fn identifier(&self) -> u64 {
        self.id
    }
}