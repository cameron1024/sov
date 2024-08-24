use proc_macro2::TokenStream;
use quote::quote;

use crate::parse::Input;

use super::StructNames;
pub(super) fn generate_structs(input: &Input, names: &StructNames) -> TokenStream {
    let shared = generate(input, names, false);
    let mutable = generate(input, names, true);

    quote! {
        #shared
        #mutable
    }
}

fn generate(input: &Input, names: &StructNames, mutable: bool) -> TokenStream {
    let name = match mutable {
        true => &names.mut_ref,
        false => &names.shared_ref,
    };

    let maybe_mut = match mutable {
        true => quote! { mut },
        false => quote! {},
    };

    let lifetime = &names.lifetime;

    let fields = input.map_fields_with_delimiters(
        |field| {
            let vis = &field.vis;
            let name = field.ident.as_ref().unwrap();
            let ty = &field.ty;

            quote! { #vis #name: & #lifetime #maybe_mut #ty }
        },
        |field, _index| {
            let vis = &field.vis;
            let ty = &field.ty;

            quote! { #vis & #lifetime #maybe_mut #ty }
        },
    );

    quote! {
        pub struct #name <#lifetime> #fields
    }
}
