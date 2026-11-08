use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;

pub(crate) fn get_crate_name() -> TokenStream {
    let name = match crate_name("carbide") {
        Ok(FoundCrate::Name(name)) => name,
        Ok(FoundCrate::Itself) => "carbide".to_string(),
        Err(e1) => match crate_name("carbide_core") {
            Ok(FoundCrate::Name(name)) => name,
            Ok(FoundCrate::Itself) => "carbide_core".to_string(),
            Err(e2) => {
                panic!("{} \n {}", e1, e2);
            }
        }
    };
    let name = Ident::new(&name, Span::call_site());
    quote!(#name)
}