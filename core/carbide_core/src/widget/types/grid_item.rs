

#[derive(Debug, Clone, Copy)]
pub enum GridItem {
    Fixed(f64),
    Adaptive(f64),
    Flexible,
    MinMax {
        minimum: f64,
        maximum: f64,
    }
}