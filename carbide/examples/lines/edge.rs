use crate::Line;

#[derive(Debug, Clone)]
pub struct Edge {
    pub id: usize,
    pub from: usize,
    pub to: usize,
    pub pos_line: Line,
    pub neg_line: Line,
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
            }
        }
    }

    pub fn flip_lines(&mut self) {
        let temp = self.pos_line;
        self.pos_line = self.neg_line;
        self.neg_line = temp;
    }
}