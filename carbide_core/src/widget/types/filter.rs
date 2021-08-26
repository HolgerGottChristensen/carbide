pub struct Filter {
    pub filter: Vec<FilterValue>,
}

pub struct FilterValue {
    pub offset_x: u32,
    pub offset_y: u32,
    pub weight: f32,
}