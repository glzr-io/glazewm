//! Macro to derive `From` and `TryFrom` implementations for enum variants

use quote::ToTokens as _;

use crate::prelude::*;

/// Macro to derive `From` and `TryFrom` implementations for enum variants
/// Derives `From<T>` for each variant with format `Enum::Variant(T)`, and
/// implements `TryFrom<Enum>` for each variant, returning an error if the
/// variant does not match.
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
      // Don't error on Unit variants, just do nothing.
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

    let error = format!(
      "Cannot convert this variant of enum `{}` to {}",
      name,
      // syn::Type doesn't print well, so convert it to a token stream
      // which works well enough.
      inner_type.to_token_stream()
    );

    quote::quote! {
      impl From<#inner_type> for #name {
        fn from(value: #inner_type) -> Self {
          #name::#ident(value)
        }
      }

      impl TryFrom<#name> for #inner_type {
        type Error = &'static str;

        fn try_from(value: #name) -> Result<Self, Self::Error> {
          match value {
            #name::#ident(inner) => Ok(inner),
            _ => Err(#error),
          }
        }
      }

      impl<'a> TryFrom<&'a #name> for &'a #inner_type {
        type Error = &'static str;

        fn try_from(value: &'a #name) -> Result<Self, Self::Error> {
          match value {
            #name::#ident(inner) => Ok(inner),
            _ => Err(#error),
          }
        }
      }
    }
  });

  quote::quote! {
    #(#variants)*
  }
  .into()
}
