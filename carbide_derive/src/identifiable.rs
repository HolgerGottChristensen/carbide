use proc_macro2::{Ident, Span};
use syn::{Attribute, Field, Fields, Meta, Type};

pub fn impl_identifiable(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let struct_ident = &ast.ident;
    let generics = &ast.generics;
    let wheres = &ast.generics.where_clause;

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

    let fields_with_id_attributes: Vec<(Ident, Type)> = fields.map_or(vec![], |n| n.iter().filter_map(|field| {
        let contains_state = has_id_attribute(&field);

        if contains_state {
            Some((field.ident.clone().unwrap(), field.ty.clone()))
        } else {
            None
        }
    }).collect::<Vec<_>>());

    let id_named_field: Vec<(Ident, Type)> = fields.map_or(vec![], |n| n.iter().filter_map(|field| {
        if let Some(ident) = &field.ident {
            if ident == &Ident::new("id", Span::call_site()) {
                Some((ident.clone(), field.ty.clone()))
            } else {
                None
            }
        } else {
            None
        }
    }).collect::<Vec<_>>());

    let (id_ident, ty) = fields_with_id_attributes.first().unwrap_or(id_named_field.first().unwrap());

    quote! {
        #[automatically_derived]
        impl #generics carbide::identifiable::Identifiable<#ty> for #struct_ident #generics #wheres {
            fn id(&self) -> #ty {
                self.#id_ident
            }
        }
    }
}

fn has_id_attribute(field: &&Field) -> bool {
    let mut contains_state = false;

    for attr in &field.attrs {
        match Attribute::parse_meta(&attr) {
            Ok(n) => {
                match n {
                    // Path token
                    Meta::Path(path) => {
                        if crate::widget::is_attribute_path(path, "id") {
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
    contains_state
}

