

bitflags::bitflags! {
    /// Flags which shaders use to determine properties of a material
    #[derive(Default, Debug)]
    pub struct MaterialFlags : u32 {
        const ALBEDO_ACTIVE =       0b0000_0000_0000_0001;
        const ALBEDO_BLEND =        0b0000_0000_0000_0010;
        const ALBEDO_VERTEX_SRGB =  0b0000_0000_0000_0100;
        const BICOMPONENT_NORMAL =  0b0000_0000_0000_1000;
        const SWIZZLED_NORMAL =     0b0000_0000_0001_0000;
        const YDOWN_NORMAL =        0b0000_0000_0010_0000;
        const AOMR_COMBINED =       0b0000_0000_0100_0000;
        const AOMR_SWIZZLED_SPLIT = 0b0000_0000_1000_0000;
        const AOMR_SPLIT =          0b0000_0001_0000_0000;
        const AOMR_BW_SPLIT =       0b0000_0010_0000_0000;
        const CC_GLTF_COMBINED =    0b0000_0100_0000_0000;
        const CC_GLTF_SPLIT =       0b0000_1000_0000_0000;
        const CC_BW_SPLIT =         0b0001_0000_0000_0000;
        const UNLIT =               0b0010_0000_0000_0000;
        const NEAREST =             0b0100_0000_0000_0000;
    }
}