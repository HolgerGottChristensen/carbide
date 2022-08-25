use crate::draw::{Dimension, Position};
use crate::event::{MouseButton, MouseEventHandler};
use crate::CommonWidgetImpl;
use carbide_core::event::MouseEvent;
use crate::environment::{Environment, EnvironmentColor};
use crate::flags::Flags;
use crate::layout::Layout;
use crate::state::{LocalState, Map1, TState};
use crate::widget::{Background, CommonWidget, EdgeInsets, ForEach, IfElse, Menu, MenuItem, MouseArea, Overlay, Rectangle, Text, VStack, Widget, WidgetExt, WidgetId, Wrap};

#[derive(Debug, Clone, Widget)]
#[carbide_exclude(MouseEvent, Layout)]
pub struct PopupMenu {
    id: WidgetId,
    child: Box<dyn Widget>,
    position: Position,
    dimension: Dimension,
    popup: Result<Overlay, Box<Background>>,
    menu: TState<Menu>,
}

impl PopupMenu {
    pub fn new(menu: TState<Menu>, top_level: bool) -> Box<Self> {
        let item = Text::new(
            Map1::read_map(menu.clone(), |a: &Menu| a.name().to_string()).ignore_writes(),
        )
        .wrap_mode(Wrap::None)
        .padding(EdgeInsets::single(5.0, 5.0, 7.0, 7.0))
        .background(Rectangle::new().fill(EnvironmentColor::Orange));

        let menu_items =
            Map1::read_map(menu.clone(), |menu: &Menu| menu.items().clone()).ignore_writes();

        let list = VStack::new(vec![ForEach::new(menu_items, Self::menu_item_delegate)])
            .spacing(1.0)
            .padding(1.0)
            .background(Rectangle::new().fill(EnvironmentColor::Blue));

        if top_level {
            Box::new(PopupMenu {
                id: WidgetId::new(),
                child: item,
                position: Default::default(),
                dimension: Default::default(),
                popup: Ok(Overlay::new(MouseArea::new(list).on_click_outside(
                    move |env: &mut Environment, _: _| {
                        env.add_overlay("controls_popup_layer", None);
                        env.request_animation_frame();
                    },
                ))),
                menu,
            })
        } else {
            let hovered = LocalState::new(false);

            Box::new(PopupMenu {
                id: WidgetId::new(),
                child: MouseArea::new(item).hovered(hovered),
                position: Default::default(),
                dimension: Default::default(),
                popup: Err(list),
                menu,
            })
        }
    }

    fn menu_item_delegate(item: TState<MenuItem>, index: TState<usize>) -> Box<dyn Widget> {
        let default_item = Map1::read_map(item.clone(), |item: &MenuItem| {
            matches!(item, MenuItem::Item { .. })
        })
        .ignore_writes();

        let separator_item = Map1::read_map(item.clone(), |item: &MenuItem| {
            matches!(item, MenuItem::Separator)
        })
        .ignore_writes();

        let name = Map1::read_map(item.clone(), |item: &MenuItem| match item {
            MenuItem::Item { name, .. } => name.clone(),
            MenuItem::Separator => String::new(),
            MenuItem::SubMenu { menu } => menu.name().to_string(),
        });

        let submenu = Map1::read_map(item, |item: &MenuItem| match item {
            MenuItem::SubMenu { menu } => menu.clone(),
            MenuItem::Item { .. } => Menu::new(""),
            MenuItem::Separator => Menu::new(""),
        });

        let separator_or_submenu = IfElse::new(separator_item)
            .when_true(
                Rectangle::new()
                    .fill(EnvironmentColor::Red)
                    .frame_fixed_height(1)
                    .custom_flags(Flags::USEMAXCROSSAXIS),
            )
            .when_false(PopupMenu::new(submenu.ignore_writes(), false));

        IfElse::new(default_item)
            .when_true(
                Text::new(name.ignore_writes())
                    .padding(5.0)
                    .background(Rectangle::new().fill(EnvironmentColor::Green)),
            )
            .when_false(separator_or_submenu)
    }
}

impl MouseEventHandler for PopupMenu {
    fn handle_mouse_event(&mut self, event: &MouseEvent, _consumed: &bool, env: &mut Environment) {
        match event {
            MouseEvent::NClick(MouseButton::Left, pos, _, _)
            | MouseEvent::Click(MouseButton::Left, pos, _) => {
                if self.is_inside(*pos) {
                    if let Ok(over) = &mut self.popup {
                        over.set_showing(true);
                        env.add_overlay("controls_popup_layer", Some(over.clone()));
                        env.request_animation_frame();
                    }
                }
            }
            _ => (),
        }
    }
}

impl Layout for PopupMenu {
    fn calculate_size(&mut self, requested_size: Dimension, env: &mut Environment) -> Dimension {
        let dim = self.child.calculate_size(requested_size, env);
        if let Ok(over) = &mut self.popup {
            over.calculate_size(Dimension::new(1000.0, 1000.0), env);
        }
        if let Err(b) = &mut self.popup {
            b.calculate_size(Dimension::new(1000.0, 1000.0), env);
        }
        self.set_dimension(dim);
        dim
    }

    fn position_children(&mut self) {
        let positioning = self.alignment().positioner();
        let position = self.position();
        let dimension = self.dimension();
        positioning(position, dimension, &mut *self.child);
        self.child.position_children();

        if let Ok(over) = &mut self.popup {
            let position = position + Dimension::new(0.0, dimension.height);

            over.set_position(position);
            over.position_children();
        }
        if let Err(b) = &mut self.popup {
            //let position = position + Dimension::new(dimension.width, 0.0);

            //b.set_position(position);
            b.position_children();
        }
    }
}

CommonWidgetImpl!(PopupMenu, self, id: self.id, child: self.child, position: self.position, dimension: self.dimension);

impl WidgetExt for PopupMenu {}
