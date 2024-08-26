use syn::Ident;

use super::*;

pub(super) fn codegen(input: &Input, struct_names: &StructNames) -> TokenStream {
    let remove = remove_impl(input, struct_names, parse_quote!(remove));
    let swap_remove = remove_impl(input, struct_names, parse_quote!(swap_remove));
    let pop = pop(input, struct_names);


    quote! {
        #remove
        #swap_remove
        #pop
    }
}

fn pop(input: &Input, StructNames { original, .. }: &StructNames) -> TokenStream {
    let fields = input.map_fields_with_delimiters(
        |field| {
            let name = field.ident.as_ref().unwrap();

            quote! {
                #name: self.#name.pop()?
            }
        },
        |_field, index| {
            quote! {
                self.#index.pop()?
            }
        },
    );


    quote! {
        #[inline]
        pub fn pop(&mut self) -> Option<#original> {
            Some(#original #fields)
        }
    }
}

fn remove_impl(
    input: &Input,
    StructNames { original, .. }: &StructNames,
    remove_fn: Ident,
) -> TokenStream {
    let fields = input.map_fields_with_delimiters(
        |field| {
            let name = field.ident.as_ref().unwrap();

            quote! {
                #name: self.#name.#remove_fn(index)
            }
        },
        |_field, index| {
            quote! {
                self.#index.#remove_fn(index)
            }
        },
    );
    quote! {
        pub fn #remove_fn(&mut self, index: usize) -> #original {
            #original #fields
        }
    }
}
