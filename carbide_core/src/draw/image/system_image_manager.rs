use carbide::environment::EnvironmentKey;

#[derive(Copy, Clone, Debug)]
pub struct SystemImageManager;

impl EnvironmentKey for SystemImageManager {
    type Value = fn(&str)->Option<&'static [u8]>;
}