use super::*;

pub(super) fn codegen(input: &Input, struct_names: &StructNames) -> TokenStream {
    let get = get(input, struct_names);
    let field_accessors = field_accessors(input);

    quote! {
        #get
        #field_accessors
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
