use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput, Field, Fields, Ident, Visibility};

pub(crate) struct Input {
    pub vis: Visibility,
    pub name: Ident,
    pub fields: Fields,
}

impl Input {
    pub fn map_fields(
        &self,
        map_named: impl FnMut(&Field) -> TokenStream,
        mut map_unnamed: impl FnMut(&Field, usize) -> TokenStream,
    ) -> TokenStream {
        match &self.fields {
            Fields::Unit => quote! {;},
            Fields::Named(fields) => {
                let fields = fields.named.iter().map(map_named);
                quote!(#(#fields)*)
            }
            Fields::Unnamed(fields) => {
                let fields = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(index, field)| map_unnamed(field, index));
                quote!(#(#fields)*)
            }
        }
    }
    pub fn map_fields_with_delimiters_and_separator(
        &self,
        separator: TokenStream,
        map_named: impl FnMut(&Field) -> TokenStream,
        mut map_unnamed: impl FnMut(&Field, usize) -> TokenStream,
    ) -> TokenStream {
        match &self.fields {
            Fields::Unit => quote! {;},
            Fields::Named(fields) => {
                let fields = fields.named.iter().map(map_named);
                quote!({ #(#fields #separator)* })
            }
            Fields::Unnamed(fields) => {
                let fields = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(index, field)| map_unnamed(field, index));
                quote!(( #(#fields #separator)* ))
            }
        }
    }

    pub fn map_fields_with_delimiters(
        &self,
        map_named: impl FnMut(&Field) -> TokenStream,
        map_unnamed: impl FnMut(&Field, usize) -> TokenStream,
    ) -> TokenStream {
        self.map_fields_with_delimiters_and_separator(quote!(,), map_named, map_unnamed)
    }
}

impl Input {
    pub fn from_derive_input(input: DeriveInput) -> crate::Result<Self> {
        let struc = match input.data {
            Data::Struct(s) => s,
            Data::Enum(e) => bail!(e.enum_token => "only structs are supported"),
            Data::Union(u) => bail!(u.union_token => "only structs are supported"),
        };

        let fields = struc.fields;

        let input = Input {
            vis: input.vis,
            fields,
            name: input.ident,
        };

        Ok(input)
    }
}
