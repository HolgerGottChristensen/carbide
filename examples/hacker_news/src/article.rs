use carbide::widget::WidgetId;
use serde::Deserialize;
use carbide::controls::Identifiable;

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

impl Identifiable<WidgetId> for Article {
    fn identifier(&self) -> WidgetId {
        self.carbide_id
    }
}