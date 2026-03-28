mod toolbar;
mod widget_viewer;
mod widget_outline;

pub use toolbar::*;
pub use widget_viewer::*;
pub use widget_outline::*;

use carbide::render::Style;
use carbide::draw::ImageId;
use carbide::state::ReadState;
use carbide::widget::{AnyWidget, Image};

trait StudioWidget: AnyWidget {
    //fn new() -> Box<dyn StudioWidget>;
}

/*impl<Id: ReadState<T=ImageId>, C: ReadState<T=Style>> StudioWidget for Image<Id, C> {
    fn new() -> Box<dyn StudioWidget> {
        Box::new(Image::new("images/landscape.png"))
    }
}*/