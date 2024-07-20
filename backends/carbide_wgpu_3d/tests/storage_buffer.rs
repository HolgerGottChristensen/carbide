use encase::{ArrayLength, ShaderType, StorageBuffer};
use carbide_core::render::matrix::Vector2;

#[derive(ShaderType)]
struct Positions {
    length: ArrayLength,
    #[size(runtime)]
    positions: Vec<Vector2<f32>>
}

#[test]
fn test() {
    let mut positions = Positions {
        length: ArrayLength,
        positions: Vec::from([
            Vector2 { x: 4.5, y: 3.4 },
            Vector2 { x: 1.5, y: 7.4 },
            Vector2 { x: 4.3, y: 1.9 },
        ])
    };

    let mut byte_buffer: Vec<u8> = Vec::new();

    let mut buffer = StorageBuffer::new(&mut byte_buffer);
    buffer.write(&positions).unwrap();

    println!("{:?}", byte_buffer);
}