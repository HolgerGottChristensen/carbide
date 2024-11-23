use std::collections::HashSet;

use proc_macro2::{Ident, Span, TokenStream};
use syn::{Generics, WhereClause};

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum DeriveType {
    // Events
    MouseEvent,
    KeyboardEvent,
    WindowEvent,
    OtherEvent,
    AccessibilityEvent,

    // StateSync
    StateSync,
    Id,

    // Render
    Render,

    // Focus
    Focusable,

    // Layout
    Layout,

    // Update
    Update,
    Initialize,
    Accessibility,
}

impl DeriveType {
    pub fn all() -> HashSet<DeriveType> {
        let mut set = HashSet::new();
        set.insert(DeriveType::MouseEvent);
        set.insert(DeriveType::KeyboardEvent);
        set.insert(DeriveType::WindowEvent);
        set.insert(DeriveType::OtherEvent);
        set.insert(DeriveType::AccessibilityEvent);
        set.insert(DeriveType::StateSync);
        set.insert(DeriveType::Render);
        set.insert(DeriveType::Focusable);
        set.insert(DeriveType::Layout);
        set.insert(DeriveType::Update);
        set.insert(DeriveType::Initialize);
        set.insert(DeriveType::Accessibility);
        set.insert(DeriveType::Id);
        set
    }

    pub fn from_str(string: &str) -> DeriveType {
        match string {
            "MouseEvent" => DeriveType::MouseEvent,
            "KeyboardEvent" => DeriveType::KeyboardEvent,
            "WindowEvent" => DeriveType::WindowEvent,
            "OtherEvent" => DeriveType::OtherEvent,
            "AccessibilityEvent" => DeriveType::AccessibilityEvent,
            "StateSync" => DeriveType::StateSync,
            "Render" => DeriveType::Render,
            "Focusable" => DeriveType::Focusable,
            "Layout" => DeriveType::Layout,
            "Update" => DeriveType::Update,
            "Initialize" => DeriveType::Initialize,
            "Accessibility" => DeriveType::Accessibility,
            _ => panic!("Could not match with any of the derive types."),
        }
    }

    pub fn to_token_stream(
        &self,
        ident: &Ident,
        generics: &Generics,
        wheres: &Option<WhereClause>,
        state_idents: &Vec<Ident>,
        id_idents: &Vec<Ident>,
    ) -> TokenStream {
        match self {
            DeriveType::MouseEvent => mouse_event_token_stream(ident, generics, wheres),
            DeriveType::KeyboardEvent => keyboard_event_token_stream(ident, generics, wheres),
            DeriveType::WindowEvent => window_event_token_stream(ident, generics, wheres),
            DeriveType::OtherEvent => other_event_token_stream(ident, generics, wheres),
            DeriveType::AccessibilityEvent => accessibility_event_token_stream(ident, generics, wheres),
            DeriveType::StateSync => state_sync_token_stream(ident, generics, wheres, state_idents),
            DeriveType::Id => id_token_stream(ident, generics, wheres, id_idents),
            DeriveType::Render => render_token_stream(ident, generics, wheres),
            DeriveType::Focusable => focusable_token_stream(ident, generics, wheres),
            DeriveType::Layout => layout_token_stream(ident, generics, wheres),
            DeriveType::Update => update_token_stream(ident, generics, wheres),
            DeriveType::Accessibility => accessibility_token_stream(ident, generics, wheres),
            DeriveType::Initialize => initialize_token_stream(ident, generics, wheres),
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

fn accessibility_event_token_stream(
    ident: &Ident,
    generics: &Generics,
    wheres: &Option<WhereClause>,
) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl #generics carbide::event::AccessibilityEventHandler for #ident #generics #wheres {}
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
        impl #generics carbide::widget::WidgetSync for #ident #generics #wheres {
            fn sync(&mut self, env: &mut carbide::environment::EnvironmentStack) {
                use carbide::state::StateSync;
                #(self.#state_idents.sync(env);)*
            }
        }
    }
}

fn id_token_stream(
    ident: &Ident,
    generics: &Generics,
    wheres: &Option<WhereClause>,
    id_idents: &Vec<Ident>,
) -> TokenStream {
    if let Some(id) = id_idents.first() {
        quote! {
            #[automatically_derived]
            impl #generics carbide::widget::Identifiable<carbide::widget::WidgetId> for #ident #generics #wheres {
                fn id(&self) -> carbide::widget::WidgetId {
                    self.#id
                }
            }
        }
    } else {
        quote! {}
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

fn update_token_stream(
    ident: &Ident,
    generics: &Generics,
    wheres: &Option<WhereClause>,
) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl #generics carbide::lifecycle::Update for #ident #generics #wheres {}
    }
}

fn initialize_token_stream(
    ident: &Ident,
    generics: &Generics,
    wheres: &Option<WhereClause>,
) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl #generics carbide::lifecycle::Initialize for #ident #generics #wheres {}
    }
}

fn accessibility_token_stream(
    ident: &Ident,
    generics: &Generics,
    wheres: &Option<WhereClause>,
) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl #generics carbide::accessibility::Accessibility for #ident #generics #wheres {}
    }
}