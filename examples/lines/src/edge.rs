use crate::line::Line;

#[derive(Debug, Clone)]
pub struct Edge {
    pub id: usize,
    pub from: usize,
    pub to: usize,
    pub pos_line: Line,
    pub neg_line: Line,
    pub offset: f64,
    pub width: f64,
}

impl Edge {
    pub fn new() -> Edge {
        Edge {
            id: 0,
            from: 0,
            to: 0,
            pos_line: Line {
                start: Default::default(),
                end: Default::default(),
            },
            neg_line: Line {
                start: Default::default(),
                end: Default::default(),
            },
            offset: 0.5,
            width: 20.0,
        }
    }

    pub fn flip_lines(&mut self) {
        let temp = self.pos_line;
        self.pos_line = self.neg_line;
        self.neg_line = temp;
    }

    pub fn offset(mut self, offset: f64) -> Self {
        self.offset = offset;
        self
    }

    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }
}
