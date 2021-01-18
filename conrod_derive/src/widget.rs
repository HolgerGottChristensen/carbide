use proc_macro2;
use syn;

use crate::utils;
use proc_macro2::{Ident, TokenStream};
use syn::{Type, Fields, Attribute, Meta, Error, Path, PathSegment, GenericParam, WherePredicate, PredicateType, DeriveInput, NestedMeta};
use std::collections::{HashMap, HashSet};

// The implementation for `Widget`.
pub fn impl_widget(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {

    let struct_ident = &ast.ident;

    let generics_with_gs = &ast.generics;

    let struct_attributes: HashSet<String> = parse_attributes(&ast.attrs);

    let generics_without_gs = &ast.generics.params.iter().filter_map(|generic| {
        match generic {
            GenericParam::Type(ty) => {
                if ty.ident.to_string() == "GS" {
                    None
                } else {
                    Some(GenericParam::Type(ty.clone()))
                }
            }
            a => Some(a.clone()),
        }
    }).collect::<Vec<GenericParam>>();

    /*if generics_with_gs.params.iter().count() -1 != generics_without_gs.len() {
        panic!("The struct need to have a generic with the name GS for global state.")
    }*/

    // Ensure we are deriving for a struct.
    let body = match ast.data {
        syn::Data::Struct(ref body) => body,
        _ => panic!("Widget can only be derived on a struct"),
    };

    let named = match &body.fields {
        Fields::Named(n) => {
            n
        },
        Fields::Unnamed(_) => {
            panic!("Unnamed field structs not supported for derive macro Widget")
        }
        Fields::Unit => {
            panic!("Widget can only be implemented on named field structs and not unit structs")
        }
    };

    let state_idents_iter = named.named
        .iter()
        .filter_map(|field|{
            let mut contains_state = false;

            for attr in &field.attrs {
                match Attribute::parse_meta(&attr) {
                    Ok(n) => {
                        match n {
                            // Path token
                            Meta::Path(path) => {
                                if is_attribute_path_state(path) {
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
        });

    let state_idents: Vec<Ident> = state_idents_iter.collect();


    let insert_local_state = if let Some(_) = struct_attributes.get("state_sync.insert_local_state") {
        quote! {#struct_ident::insert_local_state(self, env);}
    } else {
        quote! {}
    };

    let update_all_widget_state = if let Some(_) = struct_attributes.get("state_sync.update_all_widget_state") {
        quote! {#struct_ident::update_all_widget_state(self, env, global_state);}
    } else {
        quote! {}
    };

    let update_local_widget_state = if let Some(_) = struct_attributes.get("state_sync.update_local_widget_state") {
        quote! {#struct_ident::update_local_widget_state(self, env);}
    } else {
        quote! {}
    };

    let sync_state = if let Some(_) = struct_attributes.get("state_sync.sync_state") {
        quote! {#struct_ident::sync_state(self, env, global_state);}
    } else {
        quote! {self.default_sync_state(env, global_state)}
    };

    let handle_mouse_event = if let Some(_) = struct_attributes.get("event.handle_mouse_event") {
        quote! {#struct_ident::handle_mouse_event(self, event, consumed, global_state);}
    } else {
        quote! {}
    };

    let handle_keyboard_event = if let Some(_) = struct_attributes.get("event.handle_keyboard_event") {
        quote! {#struct_ident::handle_keyboard_event(self, event,  global_state);}
    } else {
        quote! {}
    };

    let handle_other_event = if let Some(_) = struct_attributes.get("event.handle_other_event") {
        quote! {#struct_ident::handle_other_event(self, event);}
    } else {
        quote! {}
    };

    let process_mouse_event = if let Some(_) = struct_attributes.get("event.process_mouse_event") {
        quote! {#struct_ident::process_mouse_event(self, event, consumed, env, global_state);}
    } else {
        quote! {self.process_mouse_event_default(event, consumed, env, global_state);}
    };

    let process_keyboard_event = if let Some(_) = struct_attributes.get("event.process_keyboard_event") {
        quote! {#struct_ident::process_keyboard_event(self, event, env, global_state);}
    } else {
        quote! {self.process_keyboard_event_default(event, env, global_state);}
    };

    let process_other_event = if let Some(_) = struct_attributes.get("event.process_other_event") {
        quote! {#struct_ident::process_other_event(self, event, env, global_state);}
    } else {
        quote! {self.process_other_event_default(event, env, global_state);}
    };

    let mut wheres = filtered_where_clause(&ast);

    quote! {

        impl<#(#generics_without_gs ,)* GS: conrod_core::state::global_state::GlobalState> conrod_core::event::event::Event<GS> for #struct_ident #generics_with_gs #wheres {
            fn handle_mouse_event(&mut self, event: &conrod_core::event_handler::MouseEvent, consumed: &bool, global_state: &mut GS) {
                #handle_mouse_event
            }

            fn handle_keyboard_event(&mut self, event: &conrod_core::event_handler::KeyboardEvent, global_state: &mut GS) {
                #handle_keyboard_event
            }

            fn handle_other_event(&mut self, event: &conrod_core::event_handler::WidgetEvent) {
                #handle_other_event
            }

            fn process_mouse_event(&mut self, event: &conrod_core::event_handler::MouseEvent, consumed: &bool, env: &mut conrod_core::state::environment::Environment<GS>, global_state: &mut GS) {
                #process_mouse_event
            }

            fn process_keyboard_event(&mut self, event: &conrod_core::event_handler::KeyboardEvent, env: &mut conrod_core::state::environment::Environment<GS>, global_state: &mut GS) {
                #process_keyboard_event
            }

            fn process_other_event(&mut self, event: &conrod_core::event_handler::WidgetEvent, env: &mut conrod_core::state::environment::Environment<GS>, global_state: &mut GS) {
                #process_other_event
            }
        }


        #[automatically_derived]
        impl<#(#generics_without_gs ,)* GS: conrod_core::state::global_state::GlobalState> conrod_core::state::state_sync::StateSync<GS> for #struct_ident #generics_with_gs #wheres {
            fn insert_local_state(&self, env: &mut conrod_core::state::environment::Environment<GS>) {
                #(env.insert_local_state(&self.#state_idents);)*

                #insert_local_state
            }

            fn update_all_widget_state(&mut self, env: &conrod_core::state::environment::Environment<GS>, global_state: &GS) {
                self.update_local_widget_state(env);
                #(self.#state_idents.get_value(global_state);)*

                #update_all_widget_state
            }

            fn update_local_widget_state(&mut self, env: &conrod_core::state::environment::Environment<GS>) {
                #(env.update_local_state(&mut self.#state_idents);)*

                #update_local_widget_state
            }

            fn sync_state(&mut self, env: &mut conrod_core::state::environment::Environment<GS>, global_state: &GS) {
                #sync_state
            }
        }

        #[automatically_derived]
        impl<#(#generics_without_gs ,)* GS: conrod_core::state::global_state::GlobalState> conrod_core::widget::primitive::widget::Widget<GS> for #struct_ident #generics_with_gs #wheres {}

        #[automatically_derived]
        impl<#(#generics_without_gs ,)* GS: conrod_core::state::global_state::GlobalState> conrod_core::widget::primitive::widget::WidgetExt<GS> for #struct_ident #generics_with_gs #wheres {}
    }
}

fn parse_attributes(attr: &Vec<Attribute>) -> HashSet<String> {
    let mut set = HashSet::new();

    attr.iter().filter_map(|attribute| {
        match Attribute::parse_meta(attribute) {
            Ok(meta) => {
                Some(stringify_meta(meta))
            }
            Err(_) => panic!("Cound not parse attribute as meta"),
        }
    }).for_each(|a| {
        for x in a {
            set.insert(x);
        }
    });
    set
}

fn stringify_meta(meta: Meta) -> Vec<String> {
    match meta {
        Meta::Path(path) => {
            vec![path_to_string(path)]
        }
        Meta::List(list) => {
            let own_path = path_to_string(list.path);

            let metas = list.nested.iter().filter_map(|nested_meta| {
                match nested_meta {
                    NestedMeta::Meta(m) => {
                        Some(stringify_meta(m.clone()))
                    }
                    NestedMeta::Lit(_) => None
                }
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
    let mut string = path.segments.iter().fold(String::from(""), |mut state, new| {
        state.push_str(new.ident.to_string().as_str());
        state.push_str(".");
        state
    });

    string.remove(string.len()-1);
    string
}


fn filtered_where_clause(ast: &&DeriveInput) -> TokenStream {
    if ast.generics.where_clause.is_none() {
        return quote!{};
    }


    let _wheres = &ast.generics.where_clause.clone().unwrap();

    let filtered = _wheres.predicates.iter().filter_map(|a| {
        match a {
            WherePredicate::Type(t) => {
                match &t.bounded_ty {
                    Type::Path(path) => {
                        if path.path.segments.len() == 1 && path.path.segments.first().unwrap().ident.to_string() == "GS" {
                            None
                        } else {
                            Some(WherePredicate::Type(t.clone()))
                        }
                    }
                    n => Some(WherePredicate::Type(t.clone()))
                }
            }
            b => Some(b.clone())
        }
    });
    quote! { where #(#filtered),*}
}

fn is_attribute_path_state(path: Path) -> bool {
    let is_state = path.segments.len() == 1 &&
        match path.segments.first() {
            None => false,
            Some(segment) => {
                segment.ident.to_string() == "state"
            }
        };

    is_state
}