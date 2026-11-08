use crate::material::pbr_material::PbrMaterial;
use carbide::environment::Environment;
use carbide::state::StateSync;

#[derive(Debug, Clone)]
pub enum Material {
    PBR(PbrMaterial)
}

impl StateSync for Material {
    fn sync(&mut self, env: &mut Environment) -> bool {
        match self {
            Material::PBR(p) => p.sync(env)
        }
    }
}