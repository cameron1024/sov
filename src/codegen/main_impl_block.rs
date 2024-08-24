use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_quote, spanned::Spanned, Field, Visibility};

use crate::{
    parse::Input,
    util::{self, pluralize},
};

use super::structs::StructNames;

pub(super) fn generate_impl_block(input: &Input, struct_names: &StructNames) -> TokenStream {
    let name = &struct_names.vec;
    let new = gen_new(input);
    let with_capacity = gen_with_capacity(input);
    let get = get(input, struct_names);
    let push = push(input, struct_names);
    let field_accessors = field_accessors(input);
    let len_is_empty = len_is_empty(input);

    quote! {
        impl #name {
            #new
            #with_capacity
            #get
            #push
            #field_accessors
            #len_is_empty
        }
    }
}

fn gen_new(input: &Input) -> TokenStream {
    let fields = input.map_fields_with_delimiters(
        |field| {
            let name = &field.ident.as_ref().unwrap();
            quote! { #name: ::std::vec::Vec::new()}
        },
        |_, _| quote! {::std::vec::Vec::new()},
    );

    quote! {
        #[inline]
        pub fn new() -> Self {
            Self #fields
        }
    }
}

fn gen_with_capacity(input: &Input) -> TokenStream {
    let fields = input.map_fields_with_delimiters(
        |field| {
            let name = &field.ident.as_ref().unwrap();
            quote! { #name: ::std::vec::Vec::with_capacity(capacity)}
        },
        |_, _| quote! {::std::vec::Vec::with_capacity(capacity)},
    );

    quote! {
        #[inline]
        pub fn with_capacity(capacity: ::core::primitive::usize) -> Self {
            Self #fields
        }
    }
}

fn get(input: &Input, struct_names: &StructNames) -> TokenStream {
    fn get_impl(input: &Input, struct_names: &StructNames, mutable: bool) -> TokenStream {
        let name = match mutable {
            true => &struct_names.mut_ref,
            false => &struct_names.shared_ref,
        };

        let fn_name = match mutable {
            true => quote!(get_mut),
            false => quote!(get),
        };

        let maybe_mut = match mutable {
            true => quote!(mut),
            false => quote!(),
        };

        let fields = input.map_fields_with_delimiters(
            |field| {
                let name = &field.ident.as_ref().unwrap();
                quote! { #name: & #maybe_mut self.#name[index]}
            },
            |_field, field_index| {
                quote! { & #maybe_mut self.#field_index[index]}
            },
        );
        quote! {
            #[inline]
            pub fn #fn_name(& #maybe_mut self, index: ::core::primitive::usize) -> #name {
                #name #fields
            }
        }
    }

    let shared = get_impl(input, struct_names, false);
    let mutable = get_impl(input, struct_names, true);

    quote! {
        #shared
        #mutable
    }
}

fn push(input: &Input, struct_names: &StructNames) -> TokenStream {
    let original = &struct_names.original;

    // destructuring seems to break spans, so we get dead code warnings here
    // using `let` instead doesn't cause this issue
    let assignments = input.map_fields(
        |field| {
            let name = field.ident.as_ref().unwrap();
            quote! { let #name = &value.#name; }
        },
        |field, index| {
            let ident = util::nth_field(index);
            quote_spanned! { field.span() => let #ident = value.#index; }
        },
    );

    let push_fields = input.map_fields(
        |field| {
            let name = field.ident.as_ref().unwrap();
            quote! {
                self.#name.push(value.#name);
            }
        },
        |_field, index| {
            quote! {
                self.#index.push(value.#index);
            }
        },
    );

    quote! {
        #[inline]
        pub fn push(&mut self, value: #original) {
            #assignments
            #push_fields
        }
    }
}

fn field_accessors(input: &Input) -> TokenStream {
    /// This is defined in the hidden module, so we need to make private fields `pub(super)`, so
    /// they are visible in the parent module (which is the module that the original struct is
    /// defined in)
    fn map_vis(vis: &Visibility) -> Visibility {
        match vis {
            Visibility::Inherited => parse_quote! { pub(super) },
            other => other.clone(),
        }
    }

    input.map_fields(
        |field| {
            let vis = map_vis(&field.vis);
            let ty = &field.ty;
            let field_name = field.ident.as_ref().unwrap();
            let fn_name = pluralize(field_name);
            let fn_name_mut = format_ident!("{fn_name}_mut");

            quote! {
                #vis fn #fn_name(&self) -> &[#ty] {
                    &self.#field_name
                }

                #vis fn #fn_name_mut(&mut self) -> &mut [#ty] {
                    &mut self.#field_name
                }
            }
        },
        |_field, _index| quote! {},
    )
}

fn len_is_empty(input: &Input) -> TokenStream {
    let Some(first_field) = input.fields.iter().next() else {
        return TokenStream::new();
    };

    let first_field_token = match &first_field.ident {
        Some(ident) => ident.clone(),
        None => parse_quote!(0),
    };

    quote! {
        #[inline]
        pub fn len(&self) -> usize {
            self.#first_field_token.len()
        }

        #[inline]
        pub fn is_empty(&self) -> bool {
            self.len() == 0
        }
    }
}
