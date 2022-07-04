use carbide_core::draw::Position;
use std::fmt::{Display, Formatter, Write};

#[derive(Debug, Clone)]
pub enum EditingMode {
    Selection {
        selected: SelectedState,
        hovered: SelectedState,
    },
    Editing,
    CreateWallP1 {
        mouse_position: Position,
        state: CreateWallState,
    },
    CreateWallP2 {
        mouse_position: Position,
        first_node_id: usize,
        state: CreateWallState,
    },
}

impl Display for EditingMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EditingMode::Selection { .. } => f.write_str("Selection"),
            EditingMode::Editing => f.write_str("Editing"),
            EditingMode::CreateWallP1 { .. } => f.write_str("Create Wall Initial"),
            EditingMode::CreateWallP2 { .. } => f.write_str("Create Wall Subsequent"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SelectedState {
    None,
    Node(usize),
    Edge(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum CreateWallState {
    Invalid,
    ExistingNode,
    SplitEdge,
    Floating,
}
