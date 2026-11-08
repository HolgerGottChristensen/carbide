use carbide::draw::Scalar;

#[derive(Clone, Copy, Debug)]
pub enum PercentOrValue {
    Percent(Scalar),
    Value(Scalar),
    None
}