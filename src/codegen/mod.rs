use proc_macro2::TokenStream;
use quote::quote;

use crate::parse::Input;

mod main_impl_block;
mod structs;

pub fn codegen(input: Input) -> TokenStream {
    let vis = &input.vis;
    let (vec, ref_and_ref_mut, struct_names) = structs::codegen_structs(&input);

    let impl_block = main_impl_block::generate_impl_block(&input, &struct_names);
    let mod_name = &struct_names.module;

    let vec_name = &struct_names.vec;

    quote! {
        #vis use #mod_name::#vec_name;

        #ref_and_ref_mut

        #[doc(hidden)]
        mod #mod_name {
            use super::*;

            #vec
            #impl_block
        }
    }
}
