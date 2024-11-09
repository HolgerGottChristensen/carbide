use crate::environment::type_map::TypeMap;

pub struct EnvironmentNew<'a, 'b: 'a> {
    pub stack: &'a mut TypeMap<'b>
}