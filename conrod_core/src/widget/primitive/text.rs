//! The primitive widget used for displaying text.

use {Color, Colorable, FontSize, Ui, OldWidget};
use position::{Dimension, Scalar, Dimensions, Align};
use ::{std, Rect};
use ::{text, Point};
use utils;
use widget;
use widget::render::Render;
use render::primitive::Primitive;
use graph::Container;
use widget::{Id, Rectangle};
use ::render::text::Text as RenderText;
use render::primitive_kind::PrimitiveKind;
use render::util::new_primitive;
use daggy::petgraph::graph::node_index;
use widget::common_widget::CommonWidget;
use uuid::Uuid;
use widget::primitive::Widget;
use text::Justify;
use widget::envelope_editor::EnvelopePoint;
use widget::layout::Layout;
use text::font::Map;
use layout::basic_layouter::BasicLayouter;
use event::event::Event;
use event_handler::{WidgetEvent, MouseEvent, KeyboardEvent};
use widget::primitive::widget::WidgetExt;
use color::WHITE;


/// Displays some given text centered within a rectangular area.
///
/// By default, the rectangular dimensions are fit to the area occuppied by the text.
///
/// If some horizontal dimension is given, the text will automatically wrap to the width and align
/// in accordance with the produced **Alignment**.
#[derive(Debug, WidgetCommon_)]
pub struct Text {
    /// Data necessary and common for all widget builder render.
    #[conrod(common_builder)]
    pub common: widget::CommonBuilder,
    /// The text to be drawn by the **Text**.
    pub text: String,
    /// Unique styling for the **Text**.
    pub style: Style,
    position: Point,
    dimension: Dimensions,
    wrap_mode: Wrap,
    color: Color,

    pub children: Vec<Box<dyn Widget>>,
}

impl Event for Text {
    fn handle_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        unimplemented!()
    }

    fn handle_keyboard_event(&mut self, event: &KeyboardEvent) {
        match event {
            KeyboardEvent::Text(s, _) => {
                if self.text.len() < 10 {
                    self.text = s.clone();
                }

            }
            _ => ()
        }
    }

    fn handle_other_event(&mut self, event: &WidgetEvent) {
        unimplemented!()
    }

    fn process_mouse_event(&mut self, event: &MouseEvent, consumed: &bool) {
        self.process_mouse_event_default(event, consumed);
    }

    fn process_keyboard_event(&mut self, event: &KeyboardEvent) {
        self.process_keyboard_event_default(event);
    }
}

impl Layout for Text {

    fn flexibility(&self) -> u32 {
        2
    }

    fn calculate_size(&mut self, proposed_size: Dimensions, fonts: &Map) -> Dimensions {
        let pref_width = self.default_x(fonts);
        let pref_height = self.default_y(fonts);

        // Todo calculate size of children here

        match (pref_width, pref_height) {
            (width, height) if width > proposed_size[0] && height > proposed_size[1] => {
                self.dimension = proposed_size;
            }
            (width, height) if width > proposed_size[0] && height <= proposed_size[1] => {
                self.dimension = [proposed_size[0], height];
            }
            (width, height) if width <= proposed_size[0] && height > proposed_size[1] => {
                self.dimension = [width, proposed_size[1]];
            }
            (_, _) => {
                self.dimension = [pref_width, pref_height];
            }
        }

        self.dimension

    }

    fn position_children(&mut self) {
        let positioning = BasicLayouter::Center.position();
        let position = self.position;
        let dimension = self.dimension;

        for child in &mut self.children {
            positioning(position, dimension, child);
            child.position_children();
        }
    }
}

impl Render for Text {
    fn render(self, id: Id, clip: Rect, container: &Container) -> Option<Primitive> {
        unimplemented!()
    }

    fn layout(&mut self, proposed_size: Dimensions, fonts: &text::font::Map, positioner: &dyn Fn(&mut dyn CommonWidget, Dimensions)) {
        let pref_width = self.default_x(fonts);
        let pref_height = self.default_y(fonts);

        match (pref_width, pref_height) {
            (width, height) if width > proposed_size[0] && height > proposed_size[1] => {
                positioner(self, proposed_size);
                self.dimension = proposed_size;
            }
            (width, height) if width > proposed_size[0] && height <= proposed_size[1] => {
                positioner(self, [proposed_size[0], height]);
                self.dimension = [proposed_size[0], height];
            }
            (width, height) if width <= proposed_size[0] && height > proposed_size[1] => {
                positioner(self, [width, proposed_size[1]]);
                self.dimension = [width, proposed_size[1]];
            }
            (_, _) => {
                positioner(self, [pref_width, pref_height]);
                self.dimension = [pref_width, pref_height];
            }
        }
    }

    fn get_primitives(&self, proposed_dimensions: Dimensions, fonts: &text::font::Map) -> Vec<Primitive> {
        let font_id = match fonts.ids().next() {
            Some(id) => id,
            None => return vec![],
        };
        let font = match fonts.get(font_id) {
            Some(font) => font,
            None => return vec![],
        };

        let rect = Rect::new(self.position, self.dimension);

        let new_line_infos = match self.wrap_mode {
            Wrap::None =>
                text::line::infos(&self.text, font, 14),
            Wrap::Character =>
                text::line::infos(&self.text, font, 14).wrap_by_character(rect.w()),
            Wrap::Whitespace =>
                text::line::infos(&self.text, font, 14).wrap_by_whitespace(rect.w()),
        };

        let text = RenderText {
            positioned_glyphs: Vec::new(),
            window_dim: self.dimension,
            text: self.text.clone(),
            line_infos: new_line_infos.collect(),
            font: font.clone(),
            font_size: 14,
            rect,
            justify: Justify::Left,
            y_align: Align::End,
            line_spacing: 1.0,
        };

        let kind = PrimitiveKind::Text {
            color: self.color,
            text,
            font_id,
        };

        let mut prims: Vec<Primitive> = vec![new_primitive(node_index(0), kind, Rect::new(self.position, self.dimension), Rect::new(self.position, self.dimension))];
        prims.extend(Rectangle::rect_outline(Rect::new(self.position, self.dimension), 0.5));
        let children: Vec<Primitive> = self.get_children().iter().flat_map(|f| f.get_primitives(proposed_dimensions, fonts)).collect();
        prims.extend(children);

        return prims;
    }
}

impl WidgetExt for Text {}

impl CommonWidget for Text {
    fn get_id(&self) -> Uuid {
        unimplemented!()
    }

    fn get_children(&self) -> &Vec<Box<dyn Widget>> {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut Vec<Box<dyn Widget>> {
        &mut self.children
    }

    fn get_position(&self) -> Dimensions {
        unimplemented!()
    }

    fn get_x(&self) -> f64 {
        self.position[0]
    }

    fn set_x(&mut self, x: f64) {
        self.position = Point::new(x, self.position.get_y());
    }

    fn get_y(&self) -> Scalar {
        self.position[1]
    }

    fn set_y(&mut self, y: f64) {
        self.position = Point::new(self.position.get_x(), y);
    }

    fn get_size(&self) -> Dimensions {
        unimplemented!()
    }

    fn get_width(&self) -> f64 {
        self.dimension[0]
    }

    fn get_height(&self) -> f64 {
        self.dimension[1]
    }
}

/// The styling for a **Text**'s graphics.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle_)]
pub struct Style {
    /// The font size for the **Text**.
    #[conrod(default = "theme.font_size_medium")]
    pub font_size: Option<FontSize>,
    /// The color of the **Text**.
    #[conrod(default = "theme.label_color")]
    pub color: Option<Color>,
    /// Whether or not the text should wrap around the width.
    #[conrod(default = "Some(Wrap::Whitespace)")]
    pub maybe_wrap: Option<Option<Wrap>>,
    /// The spacing between consecutive lines.
    #[conrod(default = "1.0")]
    pub line_spacing: Option<Scalar>,
    /// Alignment of the text along the *x* axis.
    #[conrod(default = "text::Justify::Left")]
    pub justify: Option<text::Justify>,
    /// The id of the font to use for rendering and layout.
    #[conrod(default = "theme.font_id")]
    pub font_id: Option<Option<text::font::Id>>,
    // /// The line styling for the text.
    // #[conrod(default = "None")]
    // pub line: Option<Option<Line>>,
}

/// The way in which text should wrap around the width.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Wrap {
    /// Wrap at the first character that exceeds the width.
    Character,
    /// Wrap at the first word that exceeds the width.
    Whitespace,
    /// No wrapping
    None,
}

// /// Line styling for the **Text**.
// pub enum Line {
//     /// Underline the text.
//     Under,
//     /// Overline the text.
//     Over,
//     /// Strikethrough the text.
//     Through,
// }

/// The state to be stored between updates for the **Text**.
#[derive(Clone, Debug, PartialEq)]
pub struct State {
    /// An owned version of the string.
    pub string: String,
    /// The indices and width for each line of text within the `string`.
    pub line_infos: Vec<text::line::Info>,
}


impl Text {
    pub fn initialize(text: String, children: Vec<Box<dyn Widget>>) -> Box<Self> {
        Box::new(Text {
            common: widget::CommonBuilder::default(),
            text,
            style: Style::default(),
            position: [0.0,0.0],
            dimension: [100.0,100.0],
            wrap_mode: Wrap::Whitespace,
            color: WHITE,
            children
        })
    }

    /// Build a new **Text** widget.
    pub fn new(text: String, position: Point, dimension: Dimensions, children: Vec<Box<dyn Widget>>) -> Box<Self> {
        Box::new(Text {
            common: widget::CommonBuilder::default(),
            text,
            style: Style::default(),
            position,
            dimension,
            wrap_mode: Wrap::Whitespace,
            color: WHITE,
            children
        })
    }

    /// If no specific width was given, we'll use the width of the widest line as a default.
    ///
    /// The `Font` used by the `Text` is retrieved in order to determine the width of each line. If
    /// the font used by the `Text` cannot be found, a dimension of `Absolute(0.0)` is returned.
    fn default_x(&self, fonts: &text::font::Map) -> Scalar {
        let font = fonts.ids().next()
            .and_then(|id| fonts.get(id));
        let font = match font {
            Some(font) => font,
            None => return 0.0,
        };

        let font_size = 14;
        let mut max_width = 0.0;
        for line in self.text.lines() {
            let width = text::line::width(line, font, font_size);
            max_width = utils::partial_max(max_width, width);
        }
        max_width
    }

    /// If no specific height was given, we'll use the total height of the text as a default.
    ///
    /// The `Font` used by the `Text` is retrieved in order to determine the width of each line. If
    /// the font used by the `Text` cannot be found, a dimension of `Absolute(0.0)` is returned.
    fn default_y(&self, fonts: &text::font::Map) -> Scalar {
        use position::Sizeable;

        let font = fonts.ids().next()
            .and_then(|id| fonts.get(id));

        let font = match font {
            Some(font) => font,
            None => return 0.0,
        };

        let text = &self.text;
        let font_size = 14;
        let wrap = Wrap::Whitespace;
        let num_lines = match wrap {
            Wrap::Character =>
                text::line::infos(text, font, font_size)
                    .wrap_by_character(self.dimension[0])
                    .count(),
            Wrap::Whitespace =>
                text::line::infos(text, font, font_size)
                    .wrap_by_whitespace(self.dimension[0])
                    .count(),
            _ => {
                text.lines().count()
            }
        };
        let line_spacing =  1.0;
        let height = text::height(std::cmp::max(num_lines, 1), font_size, line_spacing);
        height
    }


    /// Specify that the **Text** should not wrap lines around the width.
    pub fn no_line_wrap(mut self) -> Self {
        self.style.maybe_wrap = Some(None);
        self
    }

    /// Line wrap the **Text** at the beginning of the first word that exceeds the width.
    pub fn wrap_by_word(mut self) -> Self {
        self.style.maybe_wrap = Some(Some(Wrap::Whitespace));
        self
    }

    /// Line wrap the **Text** at the beginning of the first character that exceeds the width.
    pub fn wrap_by_character(mut self) -> Self {
        self.style.maybe_wrap = Some(Some(Wrap::Character));
        self
    }

    /// A method for specifying the `Font` used for displaying the `Text`.
    pub fn font_id(mut self, font_id: text::font::Id) -> Self {
        self.style.font_id =  Some(Some(font_id));
        self
    }

    /// Build the **Text** with the given **Style**.
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Align the text to the left of its bounding **Rect**'s *x* axis range.
    pub fn left_justify(self) -> Self {
        self.justify(text::Justify::Left)
    }

    /// Align the text to the middle of its bounding **Rect**'s *x* axis range.
    pub fn center_justify(self) -> Self {
        self.justify(text::Justify::Center)
    }

    /// Align the text to the right of its bounding **Rect**'s *x* axis range.
    pub fn right_justify(self) -> Self {
        self.justify(text::Justify::Right)
    }

    builder_methods!{
        pub font_size { style.font_size = Some(FontSize) }
        pub justify { style.justify = Some(text::Justify) }
        pub line_spacing { style.line_spacing = Some(Scalar) }
    }

}


impl OldWidget for Text {
    type State = State;
    type Style = Style;
    type Event = ();

    fn init_state(&self, _: widget::id::Generator) -> Self::State {
        State {
            string: String::new(),
            line_infos: Vec::new(),
        }
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }
    /// If no specific width was given, we'll use the width of the widest line as a default.
        ///
        /// The `Font` used by the `Text` is retrieved in order to determine the width of each line. If
        /// the font used by the `Text` cannot be found, a dimension of `Absolute(0.0)` is returned.
    fn default_x_dimension(&self, ui: &Ui) -> Dimension {
        let font = match self.style.font_id(&ui.theme)
            .or(ui.fonts.ids().next())
            .and_then(|id| ui.fonts.get(id))
        {
            Some(font) => font,
            None => return Dimension::Absolute(0.0),
        };

        let font_size = self.style.font_size(&ui.theme);
        let mut max_width = 0.0;
        for line in self.text.lines() {
            let width = text::line::width(line, font, font_size);
            max_width = utils::partial_max(max_width, width);
        }
        Dimension::Absolute(max_width)
    }

    /// If no specific height was given, we'll use the total height of the text as a default.
    ///
    /// The `Font` used by the `Text` is retrieved in order to determine the width of each line. If
    /// the font used by the `Text` cannot be found, a dimension of `Absolute(0.0)` is returned.
    fn default_y_dimension(&self, ui: &Ui) -> Dimension {
        use position::Sizeable;

        let font = match self.style.font_id(&ui.theme)
            .or(ui.fonts.ids().next())
            .and_then(|id| ui.fonts.get(id))
        {
            Some(font) => font,
            None => return Dimension::Absolute(0.0),
        };

        let text = &self.text;
        let font_size = self.style.font_size(&ui.theme);
        let num_lines = 1 as usize; /*match self.style.maybe_wrap(&ui.theme) {
            None => text.lines().count(),
            Some(wrap) => match self.get_w(ui) {
                None => text.lines().count(),
                Some(max_w) => match wrap {
                    Wrap::Character =>
                        text::line::infos(text, font, font_size)
                            .wrap_by_character(max_w)
                            .count(),
                    Wrap::Whitespace =>
                        text::line::infos(text, font, font_size)
                            .wrap_by_whitespace(max_w)
                            .count(),
                },
            },
        };*/
        let line_spacing = self.style.line_spacing(&ui.theme);
        let height = text::height(std::cmp::max(num_lines, 1), font_size, line_spacing);
        Dimension::Absolute(height)
    }
    /// Update the state of the Text.
    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { rect, state, style, ui, .. } = args;
        let Text { text, .. } = self;

        let maybe_wrap = style.maybe_wrap(ui.theme());
        let font_size = style.font_size(ui.theme());

        let font = match style.font_id(&ui.theme)
            .or(ui.fonts.ids().next())
            .and_then(|id| ui.fonts.get(id))
        {
            Some(font) => font,
            None => return,
        };

        // Produces an iterator yielding info for each line within the `text`.
        let new_line_infos = || text::line::infos(&text, font, font_size);

        // If the string is different, we must update both the string and the line breaks.
        if &state.string[..] != text {
            state.update(|state| {
                state.string = text.to_owned();
                state.line_infos = new_line_infos().collect();
            });

        // Otherwise, we'll check to see if we have to update the line breaks.
        } else {
            use utils::write_if_different;
            use std::borrow::Cow;

            // Compare the line_infos and only collect the new ones if they are different.
            let maybe_new_line_infos = {
                let line_infos = &state.line_infos[..];
                match write_if_different(line_infos, new_line_infos()) {
                    Cow::Owned(new) => Some(new),
                    _ => None,
                }
            };

            if let Some(new_line_infos) = maybe_new_line_infos {
                state.update(|state| state.line_infos = new_line_infos);
            }
        }
    }

}

impl Colorable for Text {
    fn color(mut self, color: Color) -> Self {
        self.style.color = Some(color);
        self
    }
}
