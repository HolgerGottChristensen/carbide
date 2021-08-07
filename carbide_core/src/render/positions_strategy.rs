use crate::draw::Dimensions;
use crate::draw::Point;

trait PositionStrategy {
    fn position(parent_position: Point, child_size: Dimensions);
}