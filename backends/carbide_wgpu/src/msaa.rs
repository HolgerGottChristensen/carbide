
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Msaa {
    X1,
    X4,
}

impl Msaa {
    pub fn to_samples(&self) -> u32 {
        match self {
            Msaa::X1 => 1,
            Msaa::X4 => 4,
        }
    }
}