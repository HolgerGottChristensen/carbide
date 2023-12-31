use std::collections::HashSet;

use proc_macro2::{Ident, TokenStream};
use syn::{Generics, WhereClause};

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum DeriveType {
    // Events
    MouseEvent,
    KeyboardEvent,
    WindowEvent,
    OtherEvent,

    // StateSync
    StateSync,

    // Render
    Render,

    // Focus
    Focusable,

    // Layout
    Layout,
}

impl DeriveType {
    pub fn all() -> HashSet<DeriveType> {
        let mut set = HashSet::new();
        set.insert(DeriveType::MouseEvent);
        set.insert(DeriveType::KeyboardEvent);
        set.insert(DeriveType::WindowEvent);
        set.insert(DeriveType::OtherEvent);
        set.insert(DeriveType::StateSync);
        set.insert(DeriveType::Render);
        set.insert(DeriveType::Focusable);
        set.insert(DeriveType::Layout);
        set
    }

    pub fn from_str(string: &str) -> DeriveType {
        match string {
            "MouseEvent" => DeriveType::MouseEvent,
            "KeyboardEvent" => DeriveType::KeyboardEvent,
            "WindowEvent" => DeriveType::WindowEvent,
            "OtherEvent" => DeriveType::OtherEvent,
            "StateSync" => DeriveType::StateSync,
            "Render" => DeriveType::Render,
            "Focusable" => DeriveType::Focusable,
            "Layout" => DeriveType::Layout,
            _ => panic!("Could not match with any of the derive types."),
        }
    }

    pub fn to_token_stream(
        &self,
        ident: &Ident,
        generics: &Generics,
        wheres: &Option<WhereClause>,
        state_idents: &Vec<Ident>,
    ) -> TokenStream {
        match self {
            DeriveType::MouseEvent => mouse_event_token_stream(ident, generics, wheres),
            DeriveType::KeyboardEvent => keyboard_event_token_stream(ident, generics, wheres),
            DeriveType::WindowEvent => window_event_token_stream(ident, generics, wheres),
            DeriveType::OtherEvent => other_event_token_stream(ident, generics, wheres),
            DeriveType::StateSync => state_sync_token_stream(ident, generics, wheres, state_idents),
            DeriveType::Render => render_token_stream(ident, generics, wheres),
            DeriveType::Focusable => focusable_token_stream(ident, generics, wheres),
            DeriveType::Layout => layout_token_stream(ident, generics, wheres),
        }
    }
}

fn mouse_event_token_stream(
    ident: &Ident,
    generics: &Generics,
    wheres: &Option<WhereClause>,
) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl #generics carbide::event::MouseEventHandler for #ident #generics #wheres {}
    }
}

fn keyboard_event_token_stream(
    ident: &Ident,
    generics: &Generics,
    wheres: &Option<WhereClause>,
) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl #generics carbide::event::KeyboardEventHandler for #ident #generics #wheres {}
    }
}

fn window_event_token_stream(
    ident: &Ident,
    generics: &Generics,
    wheres: &Option<WhereClause>,
) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl #generics carbide::event::WindowEventHandler for #ident #generics #wheres {}
    }
}

fn other_event_token_stream(
    ident: &Ident,
    generics: &Generics,
    wheres: &Option<WhereClause>,
) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl #generics carbide::event::OtherEventHandler for #ident #generics #wheres {}
    }
}

fn state_sync_token_stream(
    ident: &Ident,
    generics: &Generics,
    wheres: &Option<WhereClause>,
    state_idents: &Vec<Ident>,
) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl #generics carbide::state::StateSync for #ident #generics #wheres {
            fn capture_state(&mut self, env: &mut carbide::environment::Environment) {
                use carbide::state::NewStateSync;
                #(self.#state_idents.sync(env);)*
            }

            fn release_state(&mut self, env: &mut carbide::environment::Environment) {
                use carbide::state::NewStateSync;
                #(self.#state_idents.sync(env);)*
            }
        }
    }
}

fn render_token_stream(
    ident: &Ident,
    generics: &Generics,
    wheres: &Option<WhereClause>,
) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl #generics carbide::render::Render for #ident #generics #wheres {}
    }
}

fn focusable_token_stream(
    ident: &Ident,
    generics: &Generics,
    wheres: &Option<WhereClause>,
) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl #generics carbide::focus::Focusable for #ident #generics #wheres {}
    }
}

fn layout_token_stream(
    ident: &Ident,
    generics: &Generics,
    wheres: &Option<WhereClause>,
) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl #generics carbide::layout::Layout for #ident #generics #wheres {}
    }
}
