use carbide::render::matrix::Vector3;
use carbide::state::ReadState;

pub struct Transform<P, R, S> where P: ReadState<T=Vector3<f32>>, R: ReadState<T=Vector3<f32>>, S: ReadState<T=Vector3<f32>>{
    position: P,
    rotation: R,
    scale: S,
}