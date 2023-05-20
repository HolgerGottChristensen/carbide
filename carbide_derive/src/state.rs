use std::collections::HashSet;
use proc_macro2::Ident;
use syn::{Attribute, Fields, GenericParam, Meta, parse_quote, TypeParam, WhereClause};
use syn::punctuated::Punctuated;
use derive_type::DeriveType;

// The implementation for `State`.
pub fn impl_state(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let struct_ident = &ast.ident;
    let mut original_generics = ast.generics.clone();
    let mut generics = ast.generics.clone();
    let mut wheres = ast.generics.where_clause.clone().map(|a| a.predicates).unwrap_or(Punctuated::default());



    let output_generic: GenericParam = parse_quote!(__Other_T: StateContract);
    let output_state_generic: GenericParam = parse_quote!(__Other_T_State: State<__Other_T> + Clone + 'static);

    generics.params.push(output_generic);
    generics.params.push(output_state_generic);




    // Additional where: T: StateContract + Add<U>

    quote! {
        #[automatically_derived]
        impl #generics std::ops::Add<__Other_T_State> for #struct_ident #original_generics where #wheres <T as std::ops::Add<__Other_T>>::Output: StateContract, T: StateContract + std::ops::Add<__Other_T> {

            type Output = carbide_core::state::RMap2<T, __Other_T, <T as std::ops::Add<__Other_T>>::Output, #struct_ident #original_generics, __Other_T_State>;

            fn add(self, rhs: __Other_T_State) -> Self::Output  {
                carbide_core::state::Map2::read_map(self, rhs, |val1: &T, val2: &__Other_T| {
                    val1.clone() + val2.clone()
                })
            }
        }

        #[automatically_derived]
        impl #generics std::ops::Add<__Other_T_State> for &#struct_ident #original_generics where #wheres <T as std::ops::Add<__Other_T>>::Output: StateContract, T: StateContract + std::ops::Add<__Other_T> {

            type Output = carbide_core::state::RMap2<T, __Other_T, <T as std::ops::Add<__Other_T>>::Output, #struct_ident #original_generics, __Other_T_State>;

            fn add(self, rhs: __Other_T_State) -> Self::Output  {
                carbide_core::state::Map2::read_map(self.clone(), rhs, |val1: &T, val2: &__Other_T| {
                    val1.clone() + val2.clone()
                })
            }
        }

        #[automatically_derived]
        impl #generics std::ops::Add<&__Other_T_State> for #struct_ident #original_generics where #wheres <T as std::ops::Add<__Other_T>>::Output: StateContract, T: StateContract + std::ops::Add<__Other_T> {

            type Output = carbide_core::state::RMap2<T, __Other_T, <T as std::ops::Add<__Other_T>>::Output, #struct_ident #original_generics, __Other_T_State>;

            fn add(self, rhs: &__Other_T_State) -> Self::Output  {
                carbide_core::state::Map2::read_map(self, rhs.clone(), |val1: &T, val2: &__Other_T| {
                    val1.clone() + val2.clone()
                })
            }
        }

        #[automatically_derived]
        impl #generics std::ops::Add<&__Other_T_State> for &#struct_ident #original_generics where #wheres <T as std::ops::Add<__Other_T>>::Output: StateContract, T: StateContract + std::ops::Add<__Other_T> {

            type Output = carbide_core::state::RMap2<T, __Other_T, <T as std::ops::Add<__Other_T>>::Output, #struct_ident #original_generics, __Other_T_State>;

            fn add(self, rhs: &__Other_T_State) -> Self::Output  {
                carbide_core::state::Map2::read_map(self.clone(), rhs.clone(), |val1: &T, val2: &__Other_T| {
                    val1.clone() + val2.clone()
                })
            }
        }
    }
}