use Point;
use position::Dimensions;

trait PositionStrategy {
    fn position(parent_position: Point, child_size: Dimensions);
}