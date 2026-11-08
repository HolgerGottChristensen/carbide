
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum WgpuMsaa {
    // Only the headless renderer in `testing` renders without multisampling.
    #[cfg_attr(not(feature = "testing"), allow(dead_code))]
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