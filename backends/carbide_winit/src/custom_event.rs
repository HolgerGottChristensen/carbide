

#[derive(Debug)]
pub enum CustomEvent {
    Core(carbide_core::event::CoreEvent),
    Accessibility(accesskit_winit::Event)
}