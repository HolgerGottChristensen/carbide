use carbide::environment::EnvironmentKey;
use crate::environment::Shared;

#[derive(Copy, Clone, Debug)]
pub struct SystemImageManager;

impl EnvironmentKey for SystemImageManager {
    type Value = fn(&str)->Option<&'static [u8]>;
    type Access = Shared;
}