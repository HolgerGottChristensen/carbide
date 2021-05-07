use syn;
use syn::{Path, DeriveInput, WherePredicate, Meta, NestedMeta, Attribute, GenericParam, Generics, Type};
use proc_macro2::{TokenStream, Ident, Span};
use std::collections::HashSet;

// An iterator yielding all carbide attributes in the given attributes.
pub struct CarbideAttrs<I> {
    attrs: I,
}

pub fn carbide_attrs<'a, I>(attrs: I) -> CarbideAttrs<I::IntoIter>
    where I: IntoIterator<Item=&'a syn::Attribute>,
{
    CarbideAttrs { attrs: attrs.into_iter() }
}

impl<'a, I> Iterator for CarbideAttrs<I>
    where I: Iterator<Item=&'a syn::Attribute>,
{
    type Item = Vec<syn::NestedMeta>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(attr) = self.attrs.next() {
            if let Ok(_meta) = attr.parse_meta() {
                if let &syn::Meta::List(ref _metalist) = &_meta{
                    if _metalist.path.is_ident("carbide") {
                        let j = _metalist.nested.clone().into_pairs().map(|pair|pair.into_value()).collect::<Vec<syn::NestedMeta>>();
                        return Some(j);
                    }
                }
            }
        }
        None
    }
}

pub fn is_attribute_path(path: Path, value: &str) -> bool {
    let is_state = path.segments.len() == 1 &&
        match path.segments.first() {
            None => false,
            Some(segment) => {
                segment.ident.to_string() == value
            }
        };

    is_state
}

pub(crate) fn extract_global_state(struct_attributes: &HashSet<String>) -> (TokenStream, TokenStream) {
    let (global_state, global_state_use) = if let Some(_) = struct_attributes.get("global_state") {
        let idents: Vec<Ident> = struct_attributes.iter().filter_map(|st| {
            if st.starts_with("global_state.") {
                let str_ident: Vec<&str> = st.split(".").collect();
                Some(Ident::new(str_ident[1], Span::call_site()))
            } else {
                None
            }
        }).collect();

        let ident = idents.first().unwrap();

        let generic = quote! {};
        let generic_use = quote! { #ident };

        (generic, generic_use)
    } else {
        let generic = quote! { GS: carbide_core::state::global_state::GlobalState };
        let generic_use = quote! {GS};

        (generic, generic_use)
    };
    (global_state, global_state_use)
}

pub fn extract_generics(ast: &&DeriveInput) -> (Generics, Vec<GenericParam>) {
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
    (ast.generics.clone(), generics_without_gs.clone())
}

pub fn parse_attributes(attr: &Vec<Attribute>) -> HashSet<String> {
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

pub fn stringify_meta(meta: Meta) -> Vec<String> {
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

pub fn path_to_string(path: Path) -> String {
    let mut string = path.segments.iter().fold(String::from(""), |mut state, new| {
        state.push_str(new.ident.to_string().as_str());
        state.push_str(".");
        state
    });

    string.remove(string.len()-1);
    string
}


pub fn filtered_where_clause(ast: &&DeriveInput) -> TokenStream {
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
                    _ => Some(WherePredicate::Type(t.clone()))
                }
            }
            b => Some(b.clone())
        }
    });
    quote! { where #(#filtered),*}
}

