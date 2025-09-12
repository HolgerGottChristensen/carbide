
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum WgpuMsaa {
    X1,
    X4,
}

impl WgpuMsaa {
    pub fn to_samples(&self) -> u32 {
        match self {
            WgpuMsaa::X1 => 1,
            WgpuMsaa::X4 => 4,
        }
    }
}