use quote::format_ident;
use syn::Ident;

pub fn nth_field(n: usize) -> Ident {
    format_ident!("field{n}")
}

/// Best effort attempt to pluralize an English word
pub fn pluralize(ident: &Ident) -> Ident {
    match ident.to_string().chars().last().unwrap() {
        's' => format_ident!("{ident}es"),
        _ => format_ident!("{ident}s"),
    }
}
