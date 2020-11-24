use uuid::Uuid;
use widget::primitive::CWidget;
use ::{Point, Scalar};
use position::Dimensions;

pub trait CommonWidget {
    fn get_id(&self) -> Uuid;

    fn get_children(&self) -> &Vec<CWidget>;

    fn get_position(&self) -> Point;
    fn get_x(&self) -> Scalar;
    fn set_x(&mut self, x: Scalar);

    fn get_y(&self) -> Scalar;
    fn set_y(&mut self, y: Scalar);

    fn get_size(&self) -> Dimensions;
    fn get_width(&self) -> Scalar;
    fn get_height(&self) -> Scalar;
    fn calc_width(&self, pref_width: Scalar) -> Scalar {
        self.get_width()
    }
    fn calc_height(&self, pref_height: Scalar) -> Scalar {
        self.get_height()
    }
}