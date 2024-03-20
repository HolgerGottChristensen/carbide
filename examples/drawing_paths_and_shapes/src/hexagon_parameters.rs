use carbide::draw::Position;

pub struct HexagonParameters {
    pub adjustment: f64,
    pub segments: Vec<Segment>,
}

impl HexagonParameters {
    pub fn new() -> HexagonParameters {
        let adjustment = 0.085;

        HexagonParameters {
            adjustment,
            segments: vec![
                Segment {
                    line:    Position::new(0.60, 0.05),
                    curve:   Position::new(0.40, 0.05),
                    control: Position::new(0.50, 0.00)
                },
                Segment {
                    line:    Position::new(0.05, 0.20 + adjustment),
                    curve:   Position::new(0.00, 0.30 + adjustment),
                    control: Position::new(0.00, 0.25 + adjustment)
                },
                Segment {
                    line:    Position::new(0.00, 0.70 - adjustment),
                    curve:   Position::new(0.05, 0.80 - adjustment),
                    control: Position::new(0.00, 0.75 - adjustment)
                },
                Segment {
                    line:    Position::new(0.40, 0.95),
                    curve:   Position::new(0.60, 0.95),
                    control: Position::new(0.50, 1.00)
                },
                Segment {
                    line:    Position::new(0.95, 0.80 - adjustment),
                    curve:   Position::new(1.00, 0.70 - adjustment),
                    control: Position::new(1.00, 0.75 - adjustment)
                },
                Segment {
                    line:    Position::new(1.00, 0.30 + adjustment),
                    curve:   Position::new(0.95, 0.20 + adjustment),
                    control: Position::new(1.00, 0.25 + adjustment)
                },
            ]
        }
    }
}

#[derive(Clone)]
pub struct Segment {
    pub(crate) line: Position,
    pub(crate) curve: Position,
    pub(crate) control: Position,
}

