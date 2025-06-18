use enum_attrs::SubEnumDeclaration;

use crate::prelude::*;

const SUBENUM_ATTR_NAME: &str = "subenum";

mod enum_attrs;
mod variant_attr;

mod kw {
  crate::common::custom_keyword!(doc);
  crate::common::custom_keyword!(defaults);
  crate::common::custom_keyword!(derives);
  crate::common::custom_keyword!(delegates);
  crate::common::custom_keyword!(None);
}

/// List of identifiers that are separated by commas.
#[derive(Debug, Clone, Default)]
struct IdentList(pub Vec<syn::Ident>);

impl syn::parse::Parse for IdentList {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let list = input
      .parse_terminated(syn::Ident::parse, syn::Token![,])?
      .iter()
      .cloned()
      .collect::<Vec<_>>();
    Ok(IdentList(list))
  }
}

pub fn sub_enum(
  input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
  let input = syn::parse_macro_input!(input as syn::DeriveInput);

  let attrs = &input.attrs;

  let sub_enums = match enum_attrs::collect_sub_enums(
    attrs.iter().map(|attr| &attr.meta),
  ) {
    Ok(sub_enums) => sub_enums,
    Err(err) => return err.to_compile_error().into(),
  };

  let enum_data = match input.data.require_enum() {
    Ok(data) => data,
    Err(err) => return err.to_compile_error().into(),
  };

  let variants = match enum_data
    .variants
    .iter()
    .map(variant_attr::parse_variant)
    .collect::<syn::Result<Vec<_>>>()
  {
    Ok(variants) => variants,
    Err(err) => return err.to_compile_error().into(),
  };

  for variant in &variants {
    for enum_name in &variant.enums {
      if !sub_enums.iter().any(|sub_enum| sub_enum.name == *enum_name) {
        enum_name.emit_warning(
          "Variant references a subenum that is not defined.",
        );
      }
    }
  }

  let sub_enums = combine_variants(sub_enums, variants);

  let sub_enum_decls = sub_enums.iter().map(|sub_enum| {
    let name = &sub_enum.name;
    let derives = &sub_enum.derives;
    let delegates = &sub_enum.delegates;
    let docs = &sub_enum.docs;

    let variants = sub_enum.variants.iter().map(|variant| {
      let variant_name = &variant.name;
      let contained = &variant.contained;
      quote::quote! {
        #variant_name(#contained)
      }
    });

    quote::quote! {
      #(#[doc = #docs])*
      #[derive(#(#derives),*)]
      #(#[delegate(#delegates)])*
      pub enum #name {
        #(#variants),*
      }
    }
  });

  quote::quote! {
    #(#sub_enum_decls)*
  }
  .into()
}

struct SubEnum {
  pub name: syn::Ident,
  pub derives: Vec<syn::Ident>,
  pub delegates: Vec<syn::Ident>,
  pub variants: Vec<Variant>,
  pub docs: Vec<syn::LitStr>,
}

struct Variant {
  pub name: syn::Ident,
  pub contained: syn::Type,
}

/// Combine the sub enum declarations with its enum variants
fn combine_variants(
  sub_enums: Vec<SubEnumDeclaration>,
  variants: Vec<variant_attr::SubenumVariant>,
) -> Vec<SubEnum> {
  sub_enums
    .into_iter()
    .map(|sub_enum| {
      let mut combined_variants = Vec::new();
      for variant in &variants {
        if variant.enums.contains(&sub_enum.name) {
          combined_variants.push(Variant {
            name: variant.name.clone(),
            contained: variant.contained.clone(),
          });
        }
      }
      SubEnum {
        name: sub_enum.name,
        derives: sub_enum.derives,
        delegates: sub_enum.delegates,
        variants: combined_variants,
        docs: sub_enum.docs,
      }
    })
    .collect()
}
