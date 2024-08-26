use super::*;

pub(super) fn codegen(input: &Input, struct_names: &StructNames) -> TokenStream {
    let push = push(input, struct_names);
    let insert = insert(input, struct_names);

    quote! {
        #push
        #insert
    }
}

fn insert(input: &Input, struct_names: &StructNames) -> TokenStream {
    let original = &struct_names.original;
    let assignments = assignments(input);
    let insert_fields = input.map_fields(
        |field| {
            let name = field.ident.as_ref().unwrap();
            quote! {
                self.#name.insert(index, value.#name);
            }
        },
        |_field, index| {
            quote! {
                self.#index.insert(index, value.#index);
            }
        },
    );

    quote! {
        #[inline]
        pub fn insert(&mut self, index: usize, value: #original) {
            #assignments
            #insert_fields
        }
    }
}

fn push(input: &Input, struct_names: &StructNames) -> TokenStream {
    let original = &struct_names.original;

    // destructuring seems to break spans, so we get dead code warnings here
    // using `let` instead doesn't cause this issue

    let assignments = assignments(input);
    let push_fields = input.map_fields(
        |field| {
            let name = field.ident.as_ref().unwrap();
            quote! {
                self.#name.push(value.#name);
            }
        },
        |_field, index| {
            quote! {
                self.#index.push(value.#index);
            }
        },
    );

    quote! {
        #[inline]
        pub fn push(&mut self, value: #original) {
            #assignments
            #push_fields
        }
    }
}

fn assignments(input: &Input) -> TokenStream {
    // destructuring seems to break spans, so we get dead code warnings here
    // using `let` instead doesn't cause this issue
    input.map_fields(
        |field| {
            let name = field.ident.as_ref().unwrap();
            quote! { let #name = &value.#name; }
        },
        |field, index| {
            let ident = util::nth_field(index);
            quote_spanned! { field.span() => let #ident = value.#index; }
        },
    )
}
