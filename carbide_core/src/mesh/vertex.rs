#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub rgba: [f32; 4],
    pub mode: u32,
}

impl Vertex {
    pub fn new_from_2d(x: f32, y: f32, color: [f32; 4], tex_coords: [f32; 2], mode: u32) -> Vertex {
        Vertex {
            position: [x, y, 0.0],
            tex_coords,
            rgba: color,
            mode,
        }
    }
}
