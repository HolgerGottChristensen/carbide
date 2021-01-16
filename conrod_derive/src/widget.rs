use proc_macro2;
use syn;

use crate::utils;
use proc_macro2::Ident;
use syn::{Type, Fields, Attribute, Meta, Error, Path, PathSegment, GenericParam, WherePredicate, PredicateType};

// The implementation for `Widget`.
pub fn impl_widget(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {

    let struct_ident = &ast.ident;

    let generics_with_gs = &ast.generics;


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

    if generics_with_gs.params.iter().count() == 0 || generics_with_gs.params.iter().count() -1 != generics_without_gs.len() {
        panic!("The struct need to have a generic with the name GS for global state.")
    }

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

    let mut wheres = quote! {};

    if ast.generics.where_clause.is_some() {
        let _wheres = &ast.generics.where_clause.clone().unwrap();

        let filtered = _wheres.predicates.iter().filter_map(|a| {
            match a {
                WherePredicate::Type(t) => {
                    match &t.bounded_ty {
                        Type::Path(path) => {
                            if path.path.segments.len() == 1 && path.path.segments.first().unwrap().ident.to_string() == "GS" {
                                None
                            } else {
                                Some (WherePredicate::Type(t.clone()))
                            }
                        }
                        n => Some (WherePredicate::Type(t.clone()))
                    }
                }
                b => Some(b.clone())
            }
        });

        wheres = quote! { where #(#filtered),*}
    }

    quote! {
        #[automatically_derived]
        //impl<GS> conrod_core::state::state_sync::NoLocalStateSync for #struct_ident<GS> {}

        impl<#(#generics_without_gs ,)* GS: conrod_core::state::global_state::GlobalState> conrod_core::state::state_sync::StateSync<GS> for #struct_ident #generics_with_gs #wheres {
            fn insert_local_state(&self, env: &mut Environment<GS>) {
                #(env.insert_local_state(&self.#state_idents);)*
            }

            fn update_all_widget_state(&mut self, env: &Environment<GS>, global_state: &GS) {
                self.update_local_widget_state(env);
                #(self.#state_idents.get_value(global_state);)*
            }

            fn update_local_widget_state(&mut self, env: &Environment<GS>) {
                #(env.update_local_state(&mut self.#state_idents);)*
            }

            fn sync_state(&mut self, env: &mut Environment<GS>, global_state: &GS) {
                self.default_sync_state(env, global_state)
            }
        }
        #[automatically_derived]
        impl<#(#generics_without_gs ,)* GS: conrod_core::state::global_state::GlobalState> Widget<GS> for #struct_ident #generics_with_gs #wheres {}

        #[automatically_derived]
        impl<#(#generics_without_gs ,)* GS: conrod_core::state::global_state::GlobalState> WidgetExt<GS> for #struct_ident #generics_with_gs #wheres {}
    }
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