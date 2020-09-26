use layout::bounds::Bounds;

trait Layout {
    fn layout(&self, bounds: Bounds) -> Bounds;
}