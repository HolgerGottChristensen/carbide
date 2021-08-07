use std::collections::HashSet;

use proc_macro2;
use proc_macro2::{Ident, TokenStream};
use syn;
use syn::{Attribute, DeriveInput, Fields, GenericParam, Meta, NestedMeta, Path, Type, WherePredicate};

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
        }
        Fields::Unnamed(_) => {
            panic!("Unnamed field structs not supported for derive macro Widget")
        }
        Fields::Unit => {
            panic!("Widget can only be implemented on named field structs and not unit structs")
        }
    };

    let state_idents_iter = named.named
        .iter()
        .filter_map(|field| {
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


    /*let (global_state, global_state_use) = if let Some(_) = struct_attributes.get("global_state") {
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
        let generic = quote! { GS: carbide_core::state::global_state::GlobalStateContract };
        let generic_use = quote! {GS};

        (generic, generic_use)
    };*/


    let capture_state = if let Some(_) = struct_attributes.get("state_sync.capture_state") {
        quote! {#struct_ident::capture_state(self, env);}
    } else {
        quote! {}
    };

    let release_state = if let Some(_) = struct_attributes.get("state_sync.release_state") {
        quote! {#struct_ident::release_state(self, env);}
    } else {
        quote! {}
    };

    let handle_mouse_event = if let Some(_) = struct_attributes.get("event.handle_mouse_event") {
        quote! {#struct_ident::handle_mouse_event(self, event, consumed, env);}
    } else {
        quote! {}
    };

    let handle_keyboard_event = if let Some(_) = struct_attributes.get("event.handle_keyboard_event") {
        quote! {#struct_ident::handle_keyboard_event(self, event, env);}
    } else {
        quote! {}
    };

    let handle_other_event = if let Some(_) = struct_attributes.get("event.handle_other_event") {
        quote! {#struct_ident::handle_other_event(self, event, env);}
    } else {
        quote! {}
    };

    let process_mouse_event = if let Some(_) = struct_attributes.get("event.process_mouse_event") {
        quote! {#struct_ident::process_mouse_event(self, event, consumed, env);}
    } else {
        quote! {self.process_mouse_event_default(event, consumed, env);}
    };

    let process_keyboard_event = if let Some(_) = struct_attributes.get("event.process_keyboard_event") {
        quote! {#struct_ident::process_keyboard_event(self, event, env);}
    } else {
        quote! {self.process_keyboard_event_default(event, env);}
    };

    let process_other_event = if let Some(_) = struct_attributes.get("event.process_other_event") {
        quote! {#struct_ident::process_other_event(self, event, env);}
    } else {
        quote! {self.process_other_event_default(event, env);}
    };

    let get_focus = if let Some(_) = struct_attributes.get("focusable") {
        quote! {self.focus.get_latest_value().clone()}
    } else {
        quote! {carbide_core::focus::Focus::Unfocused}
    };

    let set_focus_and_request = if let Some(_) = struct_attributes.get("focusable") {
        quote! {
            if focus == carbide_core::focus::Focus::FocusReleased {
                env.request_focus(Refocus::FocusRequest)
            } else if focus == carbide_core::focus::Focus::FocusRequested {
                env.request_focus(Refocus::FocusRequest)
            }
            *self.focus.get_latest_value_mut() = focus;
        }
    } else {
        quote! {}
    };

    let set_focus = if let Some(_) = struct_attributes.get("focusable") {
        quote! {
            *self.focus.get_latest_value_mut() = focus;
        }
    } else {
        quote! {}
    };

    let focus_retrieved = if let Some(_) = struct_attributes.get("focusable.focus_retrieved") {
        quote! {#struct_ident::focus_retrieved(self, event, focus_request, env);}
    } else {
        quote! {}
    };

    let focus_dismissed = if let Some(_) = struct_attributes.get("focusable.focus_dismissed") {
        quote! {#struct_ident::focus_dismissed(self, event, focus_request, env);}
    } else {
        quote! {}
    };

    let override_default_tab_focus_behavior = if let Some(_) = struct_attributes.get("focusable.custom_tab_behavior") {
        quote! {}
    } else {
        quote! {
            if self.get_focus() == carbide_core::focus::Focus::Focused {
                match event {
                    carbide_core::event::KeyboardEvent::Press(key, modifier) => {
                        if key == &carbide_core::input::Key::Tab {
                            if modifier == &carbide_core::input::ModifierKey::SHIFT {

                                self.set_focus(carbide_core::focus::Focus::FocusReleased);
                                env.request_focus(carbide_core::focus::Refocus::FocusPrevious);

                            } else if modifier == &carbide_core::input::ModifierKey::NO_MODIFIER {

                                self.set_focus(carbide_core::focus::Focus::FocusReleased);
                                env.request_focus(carbide_core::focus::Refocus::FocusNext);

                            }
                        }
                    }
                    _ => {}
                }

            }
        }
    };

    let block_focus_request = if let Some(_) = struct_attributes.get("focusable.block_focus") {
        quote! {
             let mut any_focus = false;

            if self.get_flag().contains(Flags::FOCUSABLE) {
                let focus = self.get_focus();
                if focus == Focus::FocusRequested {
                    self.set_focus(Focus::Focused);
                    self.focus_retrieved(event, focus_request, env);
                    any_focus = true;
                } else if focus != Focus::Unfocused {
                    self.set_focus(Focus::Unfocused);
                    self.focus_dismissed(event, focus_request, env);
                }
            }

            // Do not send the focus request to its children

            any_focus
        }
    } else {
        quote! {self.process_focus_request_default(event, focus_request, env)}
    };

    let block_focus_next = if let Some(_) = struct_attributes.get("focusable.block_focus") {
        quote! {
            if self.get_flag().contains(Flags::FOCUSABLE) {
                if focus_up_for_grab {
                    self.set_focus(Focus::Focused);
                    self.focus_retrieved(event, focus_request, env);
                    false
                } else if self.get_focus() == Focus::FocusReleased {
                    self.set_focus(Focus::Unfocused);
                    self.focus_dismissed(event, focus_request, env);
                    true
                } else {
                    false
                }
            } else {
                focus_up_for_grab
            }

            // Do not send the request to its children
        }
    } else {
        quote! {self.process_focus_next_default(event, focus_request, focus_up_for_grab, env)}
    };

    let block_focus_previous = if let Some(_) = struct_attributes.get("focusable.block_focus") {
        quote! {
            if self.get_flag().contains(Flags::FOCUSABLE) {
                if focus_up_for_grab {
                    self.set_focus(Focus::Focused);
                    self.focus_retrieved(event, focus_request, env);
                    false
                } else if self.get_focus() == Focus::FocusReleased {
                    self.set_focus(Focus::Unfocused);
                    self.focus_dismissed(event, focus_request, env);
                    true
                } else {
                    false
                }
            } else {
                focus_up_for_grab
            }

            // Do not send the request to its children
        }
    } else {
        quote! {self.process_focus_previous_default(event, focus_request, focus_up_for_grab, env)}
    };

    let default_tab_focus_behavior = if let Some(_) = struct_attributes.get("focusable") {
        quote! {#override_default_tab_focus_behavior}
    } else {
        quote! {}
    };

    let process_get_primitives = if let Some(_) = struct_attributes.get("render.process_get_primitives") {
        quote! {#struct_ident::process_get_primitives(self, primitives, env);}
    } else {
        quote! {self.process_get_primitives_default(primitives, env);}
    };

    let wheres = filtered_where_clause(&ast);

    quote! {

        #[automatically_derived]
        impl<#(#generics_without_gs ,)*> carbide_core::focus::Focusable for #struct_ident #generics_with_gs #wheres {
            fn focus_retrieved(&mut self, event: &carbide_core::event::WidgetEvent, focus_request: &carbide_core::focus::Refocus, env: &mut carbide_core::environment::Environment) {
                #focus_retrieved
            }

            fn focus_dismissed(&mut self, event: &carbide_core::event::WidgetEvent, focus_request: &carbide_core::focus::Refocus, env: &mut carbide_core::environment::Environment) {
                #focus_dismissed
            }

            fn get_focus(&self) -> carbide_core::focus::Focus {
                #get_focus
            }

            fn set_focus_and_request(&mut self, focus: carbide_core::focus::Focus, env: &mut carbide_core::environment::Environment) {
                #set_focus_and_request
            }

            fn set_focus(&mut self, focus: carbide_core::focus::Focus) {
                #set_focus
            }

            fn process_focus_request(&mut self, event: &carbide_core::event::WidgetEvent, focus_request: &carbide_core::focus::Refocus, env: &mut carbide_core::environment::Environment) -> bool {
                #block_focus_request
            }

            fn process_focus_next(&mut self, event: &carbide_core::event::WidgetEvent, focus_request: &carbide_core::focus::Refocus, focus_up_for_grab: bool, env: &mut carbide_core::environment::Environment) -> bool {
                #block_focus_next
            }

            fn process_focus_previous(&mut self, event: &carbide_core::event::WidgetEvent, focus_request: &carbide_core::focus::Refocus, focus_up_for_grab: bool, env: &mut carbide_core::environment::Environment) -> bool {
                #block_focus_previous
            }

        }

        #[automatically_derived]
        impl<#(#generics_without_gs ,)*> carbide_core::event::Event for #struct_ident #generics_with_gs #wheres {
            fn handle_mouse_event(&mut self, event: &carbide_core::event::MouseEvent, consumed: &bool, env: &mut carbide_core::environment::Environment) {
                #handle_mouse_event
            }

            fn handle_keyboard_event(&mut self, event: &carbide_core::event::KeyboardEvent, env: &mut carbide_core::environment::Environment) {
                #default_tab_focus_behavior
                #handle_keyboard_event
            }

            fn handle_other_event(&mut self, event: &carbide_core::event::WidgetEvent, env: &mut carbide_core::environment::Environment) {
                #handle_other_event
            }

            fn process_mouse_event(&mut self, event: &carbide_core::event::MouseEvent, consumed: &bool, env: &mut carbide_core::environment::Environment) {
                #process_mouse_event
            }

            fn process_keyboard_event(&mut self, event: &carbide_core::event::KeyboardEvent, env: &mut carbide_core::environment::Environment) {
                #process_keyboard_event
            }

            fn process_other_event(&mut self, event: &carbide_core::event::WidgetEvent, env: &mut carbide_core::environment::Environment) {
                #process_other_event
            }
        }

        #[automatically_derived]
        impl<#(#generics_without_gs ,)*> carbide_core::widget::render::RenderProcessor for #struct_ident #generics_with_gs #wheres {
            fn process_get_primitives(&mut self, primitives: &mut std::vec::Vec<carbide_core::render::primitive::Primitive>, env: &mut carbide_core::environment::Environment) {
                #process_get_primitives
            }
        }


        #[automatically_derived]
        impl<#(#generics_without_gs ,)*> carbide_core::state::StateSync for #struct_ident #generics_with_gs #wheres {
            fn capture_state(&mut self, env: &mut carbide_core::environment::Environment) {
                #(self.#state_idents.capture_state(env);)*

                #capture_state
            }

            fn release_state(&mut self, env: &mut carbide_core::environment::Environment) {
                #(self.#state_idents.release_state(env);)*

                #release_state
            }
        }

        #[automatically_derived]
        impl<#(#generics_without_gs ,)*> carbide_core::widget::primitive::widget::Widget for #struct_ident #generics_with_gs #wheres {}

        // When this is implemented in a macro you lose IntelliJ autocomplete
        //#[automatically_derived]
        //impl<#(#generics_without_gs ,)* #global_state> carbide_core::widget::primitive::widget::WidgetExt<#global_state_use> for #struct_ident #generics_with_gs #wheres {}
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

    string.remove(string.len() - 1);
    string
}


fn filtered_where_clause(ast: &&DeriveInput) -> TokenStream {
    if ast.generics.where_clause.is_none() {
        return quote! {};
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