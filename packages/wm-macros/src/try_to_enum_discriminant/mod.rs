use quote::{ToTokens, quote};

use crate::prelude::*;

pub fn try_to_enum_discriminant(
  input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(input as syn::DeriveInput);

  let name = &input.ident;

  let enum_data = match input.data {
    syn::Data::Enum(data) => data,
    _ => panic!("`TryToDiscriminant` can only be derived for enums"),
  };

  let variants = match enum_data
    .variants
    .iter()
    .map(syn_varaint_to_variant)
    .collect::<syn::Result<Vec<Variant>>>()
  {
    Ok(variants) => variants,
    Err(err) => return err.to_compile_error().into(),
  };

  let repr = input
    .attrs
    .find_list_attrs("repr")
    .next()
    .map(|attr| {
      attr
        .parse_args::<syn::Type>()
        .map(|t| t.to_token_stream())
        .expect("Failed to parse repr attribute")
    })
    .unwrap_or_else(|| quote! {isize});

  let variants = variants.iter().map(|variant| {
    let var = &variant.name;
    let discriminant = &variant.discriminant;
    quote! { #discriminant => Ok(#name::#var) }
  });

  quote! {
    impl TryFrom<#repr> for #name {
      type Error = &'static str;

      fn try_from(value: #repr) -> Result<Self, Self::Error> {
        match value {
          #(#variants),*,
          _ => Err("Value does not match any enum variant")
        }
      }
    }
  }
  .into()
}

struct Variant {
  pub name: syn::Ident,
  pub discriminant: syn::LitInt,
}

fn syn_varaint_to_variant(variant: &syn::Variant) -> syn::Result<Variant> {
  let name = variant.ident.clone();
  let discriminant = match &variant.discriminant {
    Some((_, expr)) => match expr {
      syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(lit_int),
        ..
      }) => lit_int.clone(),
      _ => {
        return Err(ToError::error(
          &expr,
          "Expected a literal integer as a discriminant",
        ));
      }
    },
    None => {
      return Err(ToError::error(
        &variant,
        "Enum variant must have a discriminant",
      ));
    }
  };

  Ok(Variant { name, discriminant })
}
