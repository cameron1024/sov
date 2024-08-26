use super::*;

pub(super) fn codegen(input: &Input) -> TokenStream {
    let new = gen_new(input);
    let with_capacity = gen_with_capacity(input);

    quote! {
        #new
        #with_capacity
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
