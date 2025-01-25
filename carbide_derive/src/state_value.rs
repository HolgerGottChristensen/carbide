use syn::punctuated::Punctuated;
use crate::utils::get_crate_name;

// The implementation for `Value`.
pub fn impl_state_value(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let struct_ident = &ast.ident;
    let mut generics = ast.generics.clone();
    let wheres = ast.generics.where_clause.clone().map(|a| a.predicates).unwrap_or(Punctuated::default());



    let crate_name = get_crate_name();

    // Additional where: T: StateContract + Add<U>

    quote! {
        impl #generics #crate_name::state::StateSync for #struct_ident #generics #wheres {
            fn sync(&mut self, _env: &mut #crate_name::environment::EnvironmentStack) -> bool {
                true
            }
        }
        impl #generics #crate_name::state::AnyReadState for #struct_ident #generics #wheres {
            type T = #struct_ident #generics;
            fn value_dyn(&self) -> #crate_name::state::ValueRef<#struct_ident #generics> {
                #crate_name::state::ValueRef::Borrow(self)
            }
        }
        impl #generics #crate_name::state::AnyState for #struct_ident #generics #wheres {
            fn value_dyn_mut(&mut self) -> #crate_name::state::ValueRefMut<#struct_ident #generics> {
                #crate_name::state::ValueRefMut::Borrow(Some(self))
            }

            fn set_value_dyn(&mut self, value: #struct_ident #generics) {
                *self = value;
            }
        }
    }
}