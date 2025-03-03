use crate::draw::path::path_instruction::PathInstruction;

#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    pub instructions: Vec<PathInstruction>
}