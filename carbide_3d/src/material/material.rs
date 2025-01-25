use carbide::environment::{EnvironmentStack};
use carbide::state::StateSync;
use carbide::widget::WidgetSync;
use crate::material::pbr_material::PbrMaterial;

#[derive(Debug, Clone)]
pub enum Material {
    PBR(PbrMaterial)
}

impl StateSync for Material {
    fn sync(&mut self, env: &mut EnvironmentStack) -> bool {
        match self {
            Material::PBR(p) => p.sync(env)
        }
    }
}