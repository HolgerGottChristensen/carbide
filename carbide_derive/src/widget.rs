use std::collections::HashSet;

use proc_macro2;
use proc_macro2::Ident;
use syn;
use syn::{Attribute, Fields, Meta, NestedMeta, Path};

use crate::derive_type::DeriveType;

// The implementation for `Widget`.
pub fn impl_widget(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let struct_ident = &ast.ident;
    let generics = &ast.generics;
    let wheres = &ast.generics.where_clause;

    let struct_attributes: HashSet<DeriveType> = parse_attributes(&ast.attrs);

    // Ensure we are deriving for a struct.
    let body = match ast.data {
        syn::Data::Struct(ref body) => body,
        _ => panic!("Widget can only be derived on structs"),
    };

    let fields = match &body.fields {
        Fields::Named(n) => Some(&n.named),
        Fields::Unnamed(u) => Some(&u.unnamed),
        Fields::Unit => None,
    };

    let state_idents: Vec<Ident> = fields.map_or(vec![], |n| n.iter().filter_map(|field| {
        let mut contains_state = false;

        for attr in &field.attrs {
            match Attribute::parse_meta(&attr) {
                Ok(n) => {
                    match n {
                        // Path token
                        Meta::Path(path) => {
                            if is_attribute_path(path, "state") {
                                contains_state = true
                            }
                        }
                        // We do not have any list attributes for our macro yet
                        Meta::List(_) => {}
                        // We do not have any nameValue attributes for our macro yet
                        Meta::NameValue(_) => {}
                    }
                }
                Err(_) => {}
            }
        }

        if contains_state {
            field.ident.clone()
        } else {
            None
        }
    }).collect::<Vec<_>>());

    let mut id_idents: Vec<Ident> = fields.map_or(vec![], |n| n.iter().filter_map(|field| {
        let mut contains_state = false;

        for attr in &field.attrs {
            match Attribute::parse_meta(&attr) {
                Ok(n) => {
                    match n {
                        // Path token
                        Meta::Path(path) => {
                            if is_attribute_path(path, "id") {
                                contains_state = true
                            }
                        }
                        // We do not have any list attributes for our macro yet
                        Meta::List(_) => {}
                        // We do not have any nameValue attributes for our macro yet
                        Meta::NameValue(_) => {}
                    }
                }
                Err(_) => {}
            }
        }

        if contains_state {
            field.ident.clone()
        } else {
            None
        }
    }).collect::<Vec<_>>());


    let streams = struct_attributes
        .iter()
        .map(|x| x.to_token_stream(struct_ident, generics, &wheres, &state_idents, &id_idents))
        .collect::<Vec<_>>();

    quote! {
        #(#streams)*

        #[automatically_derived]
        impl #generics carbide::widget::AnyWidget for #struct_ident #generics #wheres {
            fn as_widget(&self) -> &dyn carbide::widget::AnyWidget {
                self
            }

            fn as_widget_mut(&mut self) -> &mut dyn carbide::widget::AnyWidget {
                self
            }
        }

        #[automatically_derived]
        impl #generics carbide::widget::WidgetExt for #struct_ident #generics #wheres {}
    }
}

fn parse_attributes(attr: &Vec<Attribute>) -> HashSet<DeriveType> {
    let mut string_set = HashSet::new();
    attr.iter()
        .filter_map(|attribute| match Attribute::parse_meta(attribute) {
            Ok(meta) => Some(stringify_meta(meta)),
            Err(_) => panic!("Could not parse attribute as meta"),
        })
        .for_each(|a| {
            for x in a {
                string_set.insert(x);
            }
        });

    let mut set = if string_set.contains("carbide_derive") {
        HashSet::new()
    } else {
        DeriveType::all()
    };

    for string in string_set {
        if string.contains(".")
            && (string.starts_with("carbide_derive") || string.starts_with("carbide_exclude"))
        {
            let last = string.split('.').last().unwrap();
            let derive_type = DeriveType::from_str(last);
            if string.starts_with("carbide_derive") {
                set.insert(derive_type);
            } else {
                set.remove(&derive_type);
            }
        }
    }
    set
}

fn stringify_meta(meta: Meta) -> Vec<String> {
    match meta {
        Meta::Path(path) => {
            vec![path_to_string(path)]
        }
        Meta::List(list) => {
            let own_path = path_to_string(list.path);

            let metas = list
                .nested
                .iter()
                .filter_map(|nested_meta| match nested_meta {
                    NestedMeta::Meta(m) => Some(stringify_meta(m.clone())),
                    NestedMeta::Lit(_) => None,
                });

            let mut resulting = vec![own_path.clone()];

            for i in metas {
                let joined = i.join(".");

                let mut new = own_path.clone();
                new.push_str(".");
                new.push_str(joined.as_str());
                resulting.push(new);
            }

            resulting
        }
        Meta::NameValue(_) => {
            vec![]
        }
    }
}

fn path_to_string(path: Path) -> String {
    let mut string = path
        .segments
        .iter()
        .fold(String::from(""), |mut state, new| {
            state.push_str(new.ident.to_string().as_str());
            state.push_str(".");
            state
        });

    string.remove(string.len() - 1);
    string
}

fn is_attribute_path(path: Path, equal_to: &str) -> bool {
    let is = path.segments.len() == 1
        && match path.segments.first() {
            None => false,
            Some(segment) => segment.ident.to_string() == equal_to,
        };

    is
}

