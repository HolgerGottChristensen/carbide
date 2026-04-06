use std::fmt::{Display, Formatter};
use std::ops::{Add, Index, IndexMut, Not};
use std::sync::atomic::{AtomicU32, Ordering};
use carbide::Id;

#[derive(Debug, Clone, Eq, PartialEq, Id)]
pub struct Tile {
    #[id] pub id: u32,
    pub info: TileInfo
}

impl Tile {
    fn new() -> Tile {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        Tile {
            id: COUNTER.fetch_add(1, Ordering::Relaxed),
            info: TileInfo::Empty
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TileInfo {
    Empty,
    Filled(Player)
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Player {
    Black,
    White,
}

#[derive(Debug, Clone, Copy)]
pub struct Score {
    pub black: u32,
    pub white: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct BoardPosition {
    pub(crate) x: usize,
    pub(crate) y: usize,
}

impl Not for Player {
    type Output = Player;

    fn not(self) -> Self::Output {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }
}

impl Add for BoardPosition {
    type Output = BoardPosition;

    fn add(self, rhs: Self) -> Self::Output {
        BoardPosition {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub board: Vec<Vec<Tile>>,
    pub current_player: Player,
    radius: usize,
}

impl Index<BoardPosition> for GameState {
    type Output = Tile;

    fn index(&self, index: BoardPosition) -> &Self::Output {
        &self.board[index.y][index.x]
    }
}

impl IndexMut<BoardPosition> for GameState {
    fn index_mut(&mut self, index: BoardPosition) -> &mut Self::Output {
        &mut self.board[index.y][index.x]
    }
}

impl GameState {
    pub fn new(radius: usize) -> GameState {
        let mut board = vec![vec![Tile::new(); radius*2]; radius*2];

        board[radius-1][radius-1].info = TileInfo::Filled(Player::White);
        board[radius][radius].info = TileInfo::Filled(Player::White);
        board[radius-1][radius].info = TileInfo::Filled(Player::Black);
        board[radius][radius-1].info = TileInfo::Filled(Player::Black);

        GameState {
            board,
            current_player: Player::Black,
            radius,
        }
    }

    pub fn score(&self) -> Score {
        let mut black = 0;
        let mut white = 0;

        for row in &self.board {
            for item in row {
                match item.info {
                    TileInfo::Empty => {}
                    TileInfo::Filled(Player::Black) => black += 1,
                    TileInfo::Filled(Player::White) => white += 1,
                }
            }
        }

        Score {
            black,
            white,
        }
    }

    pub fn legal_placements(&mut self) -> Vec<BoardPosition> {
        let mut empty_positions = vec![];

        for (y, row) in self.board.iter().enumerate() {
            for (x, item) in row.iter().enumerate() {
                if matches!(item.info, TileInfo::Empty) {
                    empty_positions.push(BoardPosition { x, y });
                }
            }
        }

        let mut legal_positions = vec![];

        for empty_position in empty_positions {
            let mut has_captured = false;

            for x in -1..=1 {
                for y in -1..=1 {
                    has_captured = has_captured || self.capture(empty_position, x, y, true, true);
                }
            }

            if has_captured {
                legal_positions.push(empty_position);
            }
        }

        legal_positions
    }

    pub fn place(&mut self, pos: BoardPosition) -> bool {
        let current = self.current_player;
        if pos.x >= self.radius * 2 {
            return false;
        }
        if pos.y >= self.radius * 2 {
            return false;
        }

        if matches!(self[pos].info, TileInfo::Filled(_)) {
            return false;
        }

        let mut has_captured = false;

        for x in -1..=1 {
            for y in -1..=1 {
                has_captured = self.capture(pos, x, y, false, true) || has_captured;
            }
        }

        if !has_captured {
            return false;
        }

        self[pos].info = TileInfo::Filled(current);
        self.current_player = !self.current_player;

        true
    }

    /// Returns boolean indicating if something was captured.
    fn capture(&mut self, pos: BoardPosition, x: i32, y: i32, check: bool, top: bool) -> bool {
        let opponent = !self.current_player;

        let mut current = BoardPosition { x: (pos.x as i32 + x) as usize, y: (pos.y as i32 + y) as usize };

        if current.x >= self.radius * 2 || current.y >= self.radius * 2 {
            return false;
        }

        match self[current].info {
            TileInfo::Empty => {
                false
            }
            TileInfo::Filled(p) if p == opponent => {
                let should_capture = self.capture(current, x, y, check, false);

                if should_capture {
                    if !check {
                        self[current].info = TileInfo::Filled(!p);
                    }
                    true
                } else {
                    false
                }
            }
            TileInfo::Filled(_) => {
                !top
            }
        }
    }
}

impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        for row in &self.board {
            for item in row {
                match item.info {
                    TileInfo::Empty => s.push('E'),
                    TileInfo::Filled(Player::Black) => s.push('B'),
                    TileInfo::Filled(Player::White) => s.push('W'),
                }
                s.push(' ');
            }
            s.push('\n');
        }

        f.write_str(&s)
    }
}