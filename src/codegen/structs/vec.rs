use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, Type};

use crate::parse::Input;

use super::StructNames;

pub(super) fn generate_struct(input: &Input, names: &StructNames) -> TokenStream {
    let name = &names.vec;

    let struct_fields =
        input.map_fields_with_delimiters(map_field, |field, _index| map_field(field));

    quote! {
        pub struct #name #struct_fields

    }
}

fn map_field(field: &Field) -> TokenStream {
    let field = Field {
        ty: map_type(&field.ty),
        ..field.clone()
    };

    quote!(#field)
}

fn map_type(ty: &Type) -> Type {
    syn::parse_quote!(::std::vec::Vec<#ty>)
}
