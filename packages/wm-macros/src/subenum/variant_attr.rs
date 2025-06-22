use syn::punctuated::Punctuated;

use crate::prelude::*;

/// Holds the parsed data for a single subenum variant.
pub struct SubenumVariant {
  pub name: syn::Ident,
  pub contained: syn::Type,
  pub enums: Vec<syn::Ident>,
}

/// Parse a single subenum variant from a [syn::Variant].
pub fn parse_variant(
  variant: &syn::Variant,
) -> syn::Result<SubenumVariant> {
  let name = variant.ident.clone();
  let mut contained_iter = variant.fields.iter();
  let contained = contained_iter
    .next()
    .map(|field| field.ty.clone())
    .ok_or_else(|| {
      variant.error("Subenum variants must have a contained type")
    })?;

  contained_iter.next().is_some().then_error(
    variant.error("Subenum variants must have exactly one contained type"),
  )?;

  let enums = if let Some(enums) = variant
    .attrs
    .find_list_attrs(crate::subenum::SUBENUM_ATTR_NAME)
    .map(|attr| {
      attr.parse_args_with(
        Punctuated::<syn::Ident, syn::Token![,]>::parse_terminated,
      )
    })
    .reduce(|acc, el| {
      acc.and_then(|mut acc| {
        el.map(|el| {
          acc.extend(el);
          acc
        })
      })
    }) {
    enums.map(|list| list.iter().cloned().collect::<Vec<_>>())?
  } else {
    vec![]
  };

  Ok(SubenumVariant {
    name,
    enums,
    contained,
  })
}
