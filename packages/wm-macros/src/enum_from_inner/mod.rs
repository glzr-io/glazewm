use crate::prelude::*;

pub fn enum_from_inner(
  input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(input as syn::DeriveInput);

  let name = &input.ident;

  let enum_data = match input.data.require_enum() {
    Ok(data) => data,
    Err(err) => return err.to_compile_error().into(),
  };

  let variants = enum_data.variants.iter().map(|variant| {
    let ident = &variant.ident;
    let inner_type = match &variant.fields {
      syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
        &fields.unnamed[0].ty
      }
      syn::Fields::Unit => {
        return quote::quote! {};
      }
      _ => {
        return ToError::error(
          &variant,
          "Enum variants must have exactly one unnamed field",
        )
        .to_compile_error();
      }
    };

    quote::quote! {
      impl From<#inner_type> for #name {
        fn from(value: #inner_type) -> Self {
          #name::#ident(value)
        }
      }
    }
  });

  quote::quote! {
    #(#variants)*
  }
  .into()
}
