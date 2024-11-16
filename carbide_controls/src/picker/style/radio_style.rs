use std::any::type_name;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::{BuildHasherDefault, Hasher};
use indexmap::IndexMap;
use carbide::CommonWidgetImpl;
use carbide::draw::{Dimension, Position};
use carbide::environment::EnvironmentStack;
use carbide::flags::WidgetFlag;
use carbide::focus::Focus;
use carbide::state::{AnyReadState, AnyState, ReadState, State, StateContract};
use carbide::widget::{AnyWidget, BuildWidgetIdHasher, CommonWidget, Delegate, ForEach, Rectangle, Widget, WidgetExt, WidgetId, WidgetSync};
use crate::picker::style::{PickerStyle, SelectableWidgetSequence};

#[derive(Debug, Clone)]
pub struct RadioStyle;

impl PickerStyle for RadioStyle {
    fn create(&self, focus: Box<dyn AnyState<T=Focus>>, enabled: Box<dyn AnyReadState<T=bool>>, label: Box<dyn AnyReadState<T=String>>, model: Box<dyn SelectableWidgetSequence>) -> Box<dyn AnyWidget> {
        dbg!(&model);

        RadioForEach {
            id: Default::default(),
            position: Default::default(),
            dimension: Default::default(),
            sequence: model,
            children: IndexMap::default(),
        }.boxed()
    }

    fn test<T>(self, t: T) -> T where Self: Sized {
        println!("Radio style called with: {}", type_name::<T>());
        t
    }
}


#[derive(Clone, Widget)]
#[carbide_exclude(StateSync)]
pub struct RadioForEach {
    id: WidgetId,
    position: Position,
    dimension: Dimension,
    sequence: Box<dyn SelectableWidgetSequence>,
    children: IndexMap<WidgetId, Box<dyn AnyWidget>, BuildWidgetIdHasher>,
}

impl WidgetSync for RadioForEach {
    fn sync(&mut self, env: &mut EnvironmentStack) {
        if self.sequence.has_changed(&mut self.children.keys()) {
            println!("HasChanged!");

            self.children.clear();

            self.sequence.update(&mut |widget, selected| {
                self.children.insert(widget.id(), widget.padding(20.0).background(Rectangle::new()).boxed());
            })
        }
    }
}

impl CommonWidget for RadioForEach {
    CommonWidgetImpl!(self, id: self.id, position: self.position, dimension: self.dimension, flag: WidgetFlag::PROXY);

    fn foreach_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn AnyWidget)) {
        for (_, child) in self.children.iter() {
            if child.is_ignore() {
                continue;
            }

            if child.is_proxy() {
                child.foreach_child(f);
                continue;
            }

            f(child);
        }
    }

    fn foreach_child_mut<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        for (_, child) in self.children.iter_mut() {
            if child.is_ignore() {
                continue;
            }

            if child.is_proxy() {
                child.foreach_child_mut(f);
                continue;
            }

            f(child);
        }
    }

    fn foreach_child_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        for (_, child) in self.children.iter_mut().rev() {
            if child.is_ignore() {
                continue;
            }

            if child.is_proxy() {
                child.foreach_child_rev(f);
                continue;
            }

            f(child);
        }
    }

    fn foreach_child_direct<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        for (_, child) in self.children.iter_mut() {
            f(child);
        }
    }

    fn foreach_child_direct_rev<'a>(&'a mut self, f: &mut dyn FnMut(&'a mut dyn AnyWidget)) {
        for (_, child) in self.children.iter_mut().rev() {
            f(child);
        }
    }
}

impl Debug for RadioForEach {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}