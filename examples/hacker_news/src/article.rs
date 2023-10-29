use carbide::widget::WidgetId;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Article {
    #[serde(skip_deserializing)]
    pub carbide_id: WidgetId,
    pub title: String,
    pub url: Option<String>,
    pub by: String,
    pub time: u64,
    pub descendants: Option<u64>,
    pub score: u64,
}
