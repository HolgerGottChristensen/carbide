
#[derive(Debug, Clone)]
pub enum EditingMode {
    Normal,
    CreateWallP1,
    CreateWallP2 {
        first_node_id: usize,
    }
}