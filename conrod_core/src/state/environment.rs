use std::collections::HashMap;
use std::fmt::Debug;

use bitflags::_core::fmt::Formatter;
use serde::{Deserialize, Serialize};

use crate::{Color, from_ron};
use crate::{text, to_ron};
use crate::text::font::{Error, Id};

pub struct Environment {
    stack: Vec<EnvironmentVariable>,
    fonts: text::font::Map,
    pub(crate) local_state: HashMap<String, String>,
}

impl std::fmt::Debug for Environment {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Environment {

    pub fn new() -> Self {
        Environment {
            stack: vec![],
            fonts: text::font::Map::new(),
            local_state: HashMap::new()
        }
    }

    pub fn clear_local_state(&mut self) {
        if self.local_state.len() > 0 {
            println!("Some local state was left on the stack. This might result in unexpected behavior: {:?}", self.local_state);
        }
        self.local_state.clear()
    }

    pub fn update_local_state<'a, T: Serialize + Clone + Debug + Deserialize<'a>, U: Clone>(&'a self, local_state: &mut super::state::State<T, U>) {
        if let super::state::State::LocalState { id, value } = local_state {
            let local_value: &String = match self.local_state.get(id) {
                Some(n) => n,
                None => return,
            };
            *value = from_ron::<'a, T>(&local_value).unwrap();
        }
    }

    pub fn insert_local_state<T: Serialize + Clone + Debug, U: Clone>(&mut self, local_state: &super::state::State<T, U>) {
        if let super::state::State::LocalState { id, value } = local_state {
            self.local_state.insert(id.clone(), to_ron(value).unwrap());
        }
    }

    pub fn get_fonts_map(&self) -> &text::font::Map {
        &self.fonts
    }

    pub fn insert_font_from_file<P>(&mut self, path: P) -> Result<Id, Error>
        where P: AsRef<std::path::Path>,
    {
        self.fonts.insert_from_file(path)

    }

    pub fn get_font(&self, id: Id) -> &rusttype::Font<'static> {
        self.fonts.get(id).expect("No font was found with the id")
    }

    /// Adds the given `rusttype::Font` to the `Map` and returns a unique `Id` for it.
    pub fn insert_font(&mut self, font: rusttype::Font<'static>) -> Id {
        self.fonts.insert(font)
    }

    /*pub fn init_from_ron(ron: String) -> Self {
        from_ron(&ron).unwrap()
    }*/

    pub fn push_vec(&mut self, value: Vec<EnvironmentVariable>) {
        for v in value {
            self.push(v);
        }
    }

    pub fn push(&mut self, value: EnvironmentVariable) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }
}


pub enum EnvironmentVariable {
    String{key: String, value: String},
    U32{key: String, value: u32},
    F64{key: String, value: f64},
    Color{key: String, value: Color},
    I32{key: String, value: i32},
}