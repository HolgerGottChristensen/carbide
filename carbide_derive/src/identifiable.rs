use proc_macro2::{Ident, Span, TokenStream};
use syn::{Attribute, DataStruct, Field, Fields, Generics, Meta, Type, WhereClause};

// First we should detect whether an id attribute is provided. If none are provided, we return self as the identity
pub fn impl_identifiable(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let outer_ident = &ast.ident;
    let generics = &ast.generics;
    let wheres = &ast.generics.where_clause;

    // Ensure we are deriving for a struct.
    match ast.data {
        syn::Data::Struct(ref body) => impl_identifiable_struct(outer_ident, generics, wheres, &body),
        syn::Data::Enum(_) => impl_identifiable_enum(outer_ident, generics, wheres),
        _ => panic!("Id can only be derived on structs"),
    }
}

fn impl_identifiable_enum(enum_ident: &Ident, generics: &Generics, wheres: &Option<WhereClause>) -> TokenStream {
    quote! {
        #[automatically_derived]
        impl #generics carbide::identifiable::Identifiable for #enum_ident #generics #wheres {
            type Id = #enum_ident #generics;
            fn id(&self) -> Self::Id {
                self.clone()
            }
        }
    }
}

fn impl_identifiable_struct(struct_ident: &Ident, generics: &Generics, wheres: &Option<WhereClause>, body: &&DataStruct) -> TokenStream {
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

    match &fields_with_id_attributes[..] {
        [] => {
            quote! {
                #[automatically_derived]
                impl #generics carbide::identifiable::Identifiable for #struct_ident #generics #wheres {
                    type Id = #struct_ident #generics;
                    fn id(&self) -> Self::Id {
                        self.clone()
                    }
                }
            }
        }
        [(id_ident, ty)] => {
            quote! {
                #[automatically_derived]
                impl #generics carbide::identifiable::Identifiable for #struct_ident #generics #wheres {
                    type Id = #ty;
                    fn id(&self) -> Self::Id {
                        self.#id_ident
                    }
                }
            }
        }
        _ => {
            panic!("Multiple id fields not supported yet")
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

