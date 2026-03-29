// use carbide::layout::{Layout, LayoutContext};
// use carbide::render::{Render, RenderContext};
// use carbide::state::{AnyState, IndexState, LocalState, ReadState};
// use carbide::widget::{AnyWidget, Rectangle};
// use carbide_core::draw::{Dimension, Position};
// use carbide_core::state::{
//     State, StateContract,
// };
// use carbide_core::widget::{
//     CommonWidget, Widget,
//     WidgetExt, WidgetId,
// };
// use carbide_core::widget::EmptyDelegate;
// use carbide_core::CommonWidgetImpl;
// use std::collections::HashMap;
// use std::fmt::{Debug, Formatter};
// use std::hash::{DefaultHasher, Hash, Hasher};
// use std::marker::PhantomData;
// use std::ops::Range;
// use carbide::event::{MouseEvent, MouseEventContext, MouseEventHandler};
//
// pub trait ListDelegate<Item, ModelContent>: Clone + 'static
// where
//     Item: StateContract,
//     ModelContent: RandomAccessCollection<Item>,
// {
//     fn call(&self, item: &Item, index: <ModelContent as RandomAccessCollection<Item>>::Idx) -> Box<dyn AnyWidget>;
// }
//
// impl<K, Item: StateContract, ModelContent: RandomAccessCollection<Item>> ListDelegate<Item, ModelContent> for K where K: Fn(&Item, <ModelContent as RandomAccessCollection<Item>>::Idx) -> Box<dyn AnyWidget> + Clone + 'static {
//     fn call(&self, item: &Item, index: <ModelContent as RandomAccessCollection<Item>>::Idx) -> Box<dyn AnyWidget> {
//         self(item, index)
//     }
// }
//
// impl<Item: StateContract, ModelContent: RandomAccessCollection<Item>> ListDelegate<Item, ModelContent> for EmptyDelegate {
//     fn call(&self, item: &Item, index: <ModelContent as RandomAccessCollection<Item>>::Idx) -> Box<dyn AnyWidget> {
//         Rectangle::new().boxed()
//     }
// }
//
// pub trait RandomAccessCollection<T>: StateContract + 'static where T: StateContract + 'static {
//     type Idx: StateContract + 'static;
//     type Indices: IntoIterator<Item=Self::Idx> + StateContract + 'static;
//
//     fn index(&self, index: Self::Idx) -> &T;
//     fn index_mut(&mut self, index: Self::Idx) -> &mut T;
//
//     fn start_index(&self) -> Self::Idx;
//     fn indices(&self) -> Self::Indices;
//     fn end_index(&self) -> Self::Idx;
//
//     fn next_index(&self, idx: Self::Idx) -> Self::Idx;
// }
//
// impl<T: StateContract> RandomAccessCollection<T> for Vec<T> {
//     type Idx = usize;
//     type Indices = Range<usize>;
//
//     fn index(&self, index: Self::Idx) -> &T {
//         &self[index]
//     }
//
//     fn index_mut(&mut self, index: Self::Idx) -> &mut T {
//         &mut self[index]
//     }
//
//     fn start_index(&self) -> Self::Idx {
//         0
//     }
//
//     fn indices(&self) -> Self::Indices {
//         0..self.len()
//     }
//
//     fn end_index(&self) -> Self::Idx {
//         self.len()
//     }
//
//     fn next_index(&self, idx: Self::Idx) -> Self::Idx {
//         idx + 1
//     }
// }
//
// pub trait Bla {
//     type Item;
//     type St: State<T=Self::Item>;
//     type Iter: Iterator<Item=Self::St>;
//
//     fn bla(&self) -> Self::Iter;
// }
//
// impl<T: StateContract> Bla for Vec<T> {
//     type Item = T;
//     type St = IndexState<Vec<T>, T, usize, Vec<T>, usize>;
//     type Iter = Test<Self::Item, Vec<T>>;
//
//     fn bla(&self) -> Self::Iter {
//         Test {
//             state: self.clone(),
//             current: 0,
//         }
//     }
// }
//
// impl<T: StateContract> Bla for LocalState<Vec<T>> {
//     type Item = T;
//     type St = IndexState<Vec<T>, T, usize, LocalState<Vec<T>>, usize>;
//     type Iter = Test<Self::Item, LocalState<Vec<T>>>;
//
//     fn bla(&self) -> Self::Iter {
//         Test {
//             state: self.clone(),
//             current: 0,
//         }
//     }
// }
//
// pub struct Test<T, S> where S: State<T=Vec<T>> {
//     state: S,
//     current: usize
// }
//
// impl<T: StateContract, S: State<T=Vec<T>>> Iterator for Test<T, S> {
//     type Item = IndexState<Vec<T>, T, usize, S, usize>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.current < self.state.value().len() {
//             self.current += 1;
//             Some(IndexState::new(self.state.clone(), self.current - 1))
//         } else {
//             None
//         }
//     }
// }
//
// #[derive(Widget)]
// #[carbide_exclude(Layout, Render, MouseEvent)]
// pub struct ListNew<Item, ModelContent, Model, Delegate>
// where
//     // The item we iterate
//     Item: StateContract + Hash,
//     // The type of the state stored in the model
//     ModelContent: RandomAccessCollection<Item>,
//     // The model that can be iterated, which is also a state
//     Model: State<T=ModelContent>,
//     // The delegate that can turn the items into widgets
//     Delegate: ListDelegate<Item, ModelContent>
// {
//     #[id] id: WidgetId,
//     position: Position,
//     dimension: Dimension,
//
//     model: Model,
//     delegate: Delegate,
//     spacing: f64,
//     scroll: f64,
//
//     widgets: HashMap<u64, Box<dyn AnyWidget>>,
//
//     phantom_item: PhantomData<Item>,
//     phantom_model_content: PhantomData<ModelContent>,
// }
//
//
// impl ListNew<(), Vec<()>, Vec<()>, EmptyDelegate> {
//     pub fn new<
//         // The item we iterate
//         Item: StateContract + Hash,
//         // The type of the state stored in the model
//         ModelContent: RandomAccessCollection<Item>,
//         // The model that can be iterated, which is also a state
//         Model: State<T=ModelContent>,
//         // The delegate that can turn the items into widgets
//         Delegate: ListDelegate<Item, ModelContent>
//     >(model: Model, delegate: Delegate) -> ListNew<Item, ModelContent, Model, Delegate> {
//         let spacing = 10.0;
//
//         ListNew {
//             id: WidgetId::new(),
//             position: Default::default(),
//             dimension: Default::default(),
//             model,
//             delegate,
//             spacing,
//             scroll: 0.0,
//             widgets: Default::default(),
//             phantom_item: Default::default(),
//             phantom_model_content: Default::default(),
//         }
//     }
// }
//
// impl<
//     // The item we iterate
//     Item: StateContract + Hash,
//     // The type of the state stored in the model
//     ModelContent: RandomAccessCollection<Item>,
//     // The model that can be iterated, which is also a state
//     Model: State<T=ModelContent>,
//     // The delegate that can turn the items into widgets
//     Delegate: ListDelegate<Item, ModelContent>
// > MouseEventHandler for ListNew<Item, ModelContent, Model, Delegate> {
//     fn handle_mouse_event(&mut self, event: &MouseEvent, ctx: &mut MouseEventContext) {
//         match event {
//             MouseEvent::Scroll { x, y, mouse_position, modifiers } => {
//                 self.scroll += y;
//             }
//             _ => {}
//         }
//     }
// }
//
// impl<
//     // The item we iterate
//     Item: StateContract + Hash,
//     // The type of the state stored in the model
//     ModelContent: RandomAccessCollection<Item>,
//     // The model that can be iterated, which is also a state
//     Model: State<T=ModelContent>,
//     // The delegate that can turn the items into widgets
//     Delegate: ListDelegate<Item, ModelContent>
// > Layout for ListNew<Item, ModelContent, Model, Delegate> {
//     fn calculate_size(&mut self, requested_size: Dimension, ctx: &mut LayoutContext) -> Dimension {
//         let model = self.model.value();
//
//         let indices = model.indices();
//
//         for index in indices {
//             let item = model.index(index.clone());
//             let mut hasher = DefaultHasher::new();
//             item.hash(&mut hasher);
//             let hash = hasher.finish();
//
//             if !self.widgets.contains_key(&hash) {
//                 self.widgets.insert(hash, self.delegate.call(item, index));
//             }
//
//             let widget = self.widgets.get_mut(&hash).unwrap();
//
//             widget.calculate_size(requested_size, ctx);
//         }
//
//         self.dimension = requested_size;
//
//         requested_size
//     }
//
//     fn position_children(&mut self, ctx: &mut LayoutContext) {
//         let mut current_y = self.position.y - self.scroll;
//         let model = self.model.value();
//
//         let indices = model.indices();
//
//         for index in indices {
//             let item = model.index(index.clone());
//             let mut hasher = DefaultHasher::new();
//             item.hash(&mut hasher);
//             let hash = hasher.finish();
//
//             let widget = self.widgets.get_mut(&hash).unwrap();
//
//             widget.set_position(Position::new(self.position.x, current_y));
//             widget.position_children(ctx);
//             current_y += widget.height() + self.spacing;
//         }
//     }
// }
//
// impl<
//     // The item we iterate
//     Item: StateContract + Hash,
//     // The type of the state stored in the model
//     ModelContent: RandomAccessCollection<Item>,
//     // The model that can be iterated, which is also a state
//     Model: State<T=ModelContent>,
//     // The delegate that can turn the items into widgets
//     Delegate: ListDelegate<Item, ModelContent>
// > Render for ListNew<Item, ModelContent, Model, Delegate> {
//     fn render(&mut self, ctx: &mut RenderContext) {
//         let model = self.model.value();
//
//         let indices = model.indices();
//
//         for index in indices {
//             let item = model.index(index.clone());
//             let mut hasher = DefaultHasher::new();
//             item.hash(&mut hasher);
//             let hash = hasher.finish();
//
//             let widget = self.widgets.get_mut(&hash).unwrap();
//
//             widget.render(ctx);
//         }
//     }
// }
//
// impl<
//     // The item we iterate
//     Item: StateContract + Hash,
//     // The type of the state stored in the model
//     ModelContent: RandomAccessCollection<Item>,
//     // The model that can be iterated, which is also a state
//     Model: State<T=ModelContent>,
//     // The delegate that can turn the items into widgets
//     Delegate: ListDelegate<Item, ModelContent>
// > CommonWidget for ListNew<Item, ModelContent, Model, Delegate> {
//     CommonWidgetImpl!(self, child: (), position: self.position, dimension: self.dimension);
// }
//
// impl<
//     // The item we iterate
//     Item: StateContract + Hash,
//     // The type of the state stored in the model
//     ModelContent: RandomAccessCollection<Item>,
//     // The model that can be iterated, which is also a state
//     Model: State<T=ModelContent>,
//     // The delegate that can turn the items into widgets
//     Delegate: ListDelegate<Item, ModelContent>
// > Clone for ListNew<Item, ModelContent, Model, Delegate> {
//     fn clone(&self) -> Self {
//         ListNew {
//             id: WidgetId::new(),
//             position: self.position.clone(),
//             dimension: self.dimension.clone(),
//             model: self.model.clone(),
//             delegate: self.delegate.clone(),
//             spacing: self.spacing.clone(),
//             scroll: self.scroll.clone(),
//             widgets: self.widgets.clone(),
//             phantom_item: Default::default(),
//             phantom_model_content: Default::default(),
//         }
//     }
// }
//
// impl<
//     // The item we iterate
//     Item: StateContract + Hash,
//     // The type of the state stored in the model
//     ModelContent: RandomAccessCollection<Item>,
//     // The model that can be iterated, which is also a state
//     Model: State<T=ModelContent>,
//     // The delegate that can turn the items into widgets
//     Delegate: ListDelegate<Item, ModelContent>
// > Debug for ListNew<Item, ModelContent, Model, Delegate> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("ListNew").finish()
//     }
// }