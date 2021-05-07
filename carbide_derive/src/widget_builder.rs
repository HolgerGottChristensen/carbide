use proc_macro2::{Ident, Span};
use syn::{Fields, Attribute, Meta, FnArg, PatType, PatIdent, Pat};
use utils::*;
use syn::punctuated::Punctuated;
use std::collections::HashSet;

pub fn impl_widget_builder(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let builder_ident = Ident::new(&format!("{}Builder", ast.ident.to_string()), Span::call_site());
    let ident = &ast.ident;
    let struct_attributes: HashSet<String> = parse_attributes(&ast.attrs);

    // Ensure we are deriving for a struct.
    let body = match ast.data {
        syn::Data::Struct(ref body) => body,
        _ => panic!("Widget can only be derived on a struct"),
    };

    let body_fields = &body.fields;

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

    let idents_vec = named.named.iter().map(|field| {
        field.ident.clone().unwrap()
    }).collect::<Vec<_>>();

    let fn_idents_vec = named.named.iter().map(|field| {
        Ident::new(&format!("__{}", field.ident.clone().unwrap().to_string()), Span::call_site())
    }).collect::<Vec<_>>();

    let ty_vec = named.named.iter().map(|field| {
        field.ty.clone()
    }).collect::<Vec<_>>();

    let required_idents_iter = named.named
        .iter()
        .filter_map(|field|{
            let mut contains_state = false;

            for attr in &field.attrs {
                match Attribute::parse_meta(&attr) {
                    Ok(n) => {
                        match n {
                            // Path token
                            Meta::Path(path) => {
                                if is_attribute_path(path, "required") {
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
                Some((field.ident.clone().unwrap(), field.ty.clone()))
            } else {
                None
            }
        });

    let mut fn_new_args = Punctuated::<FnArg, syn::token::Comma>::new();

    fn_new_args.extend(required_idents_iter.clone().map(|(ident, ty)| {
        let pat = PatIdent{
            attrs: vec![],
            by_ref: None,
            mutability: None,
            ident,
            subpat: None
        };

        FnArg::Typed(PatType {
            attrs: vec![],
            pat: Box::new(Pat::Ident(pat)),
            colon_token: syn::token::Colon(Span::call_site()),
            ty: Box::new(ty)
        })
    }));

    let required_idents = required_idents_iter.map(|(ident, _)| {
        ident
    }).collect::<Vec<_>>();

    let builder_generics = &ast.generics;

    let wheres = filtered_where_clause(&ast);
    let (global_state, _global_state_use) = extract_global_state(&struct_attributes);
    let (_generics_with_gs, generics_without_gs) = extract_generics(&ast);

    quote!(

        impl<#(#generics_without_gs ,)* #global_state> #ident #wheres {
            pub fn __builder(#fn_new_args) -> #builder_ident {
                #builder_ident {
                    #(#required_idents)*
                    ..Default::default()
                }
            }
        }

        #[derive(Default, Debug, Clone)]
        struct #builder_ident #builder_generics #wheres {
            #(#idents_vec: #ty_vec,)*
        }

        impl<#(#generics_without_gs ,)* #global_state> #builder_ident #wheres {
            #(
                pub fn #fn_idents_vec<I: Into<#ty_vec>> (mut self, #idents_vec: I) -> #builder_ident {
                    self.#idents_vec = #idents_vec;
                    self
                }
            )*

            pub fn __build(self) -> #ident {
                #ident {
                    #(
                        #idents_vec: self . #idents_vec ,
                    )*
                }
            }
        }
    )

}