use convert_case::Casing;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_quote, Ident, Lifetime};

use crate::parse::Input;

mod ref_and_ref_mut;
mod vec;

pub struct StructNames {
    pub original: Ident,
    pub vec: Ident,
    pub shared_ref: Ident,
    pub mut_ref: Ident,
    pub module: Ident,
    pub lifetime: Lifetime,
}

impl StructNames {
    fn from_struct_name(name: &Ident) -> Self {
        Self {
            original: name.clone(),
            vec: format_ident!("Vec{name}"),
            shared_ref: format_ident!("{name}Ref"),
            mut_ref: format_ident!("{name}RefMut"),
            module: format_ident!(
                "__hidden_vec_{}",
                name.to_string().to_case(convert_case::Case::Snake),
                span = name.span()
            ),
            lifetime: parse_quote!('a),
        }
    }
}

pub(super) fn codegen_structs(input: &Input) -> (TokenStream, TokenStream, StructNames) {
    let names = StructNames::from_struct_name(&input.name);

    let vec = vec::generate_struct(input, &names);
    let ref_and_ref_mut = ref_and_ref_mut::generate_structs(input, &names);


    (vec, ref_and_ref_mut, names)
}
