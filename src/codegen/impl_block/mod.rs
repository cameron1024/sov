use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_quote, spanned::Spanned, Visibility};

use crate::{
    parse::Input,
    util::{self, pluralize},
};

use super::structs::StructNames;

mod constructors;
mod remove;
mod insert;
mod get;

pub(super) fn generate_impl_block(input: &Input, struct_names: &StructNames) -> TokenStream {
    let name = &struct_names.vec;

    let constructors = constructors::codegen(input);
    let insert = insert::codegen(input, struct_names);
    let remove = remove::codegen(input, struct_names);
    let get = get::codegen(input, struct_names);

    let len_is_empty = len_is_empty(input);

    quote! {
        impl #name {
            #constructors
            #get
            #insert
            #len_is_empty
            #remove
        }
    }
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
