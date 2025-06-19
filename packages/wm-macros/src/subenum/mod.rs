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

  let sub_enum_to_main_impls = sub_enums
    .iter()
    .map(|sub_enum| from_sub_to_main_impl(&input.ident, sub_enum));

  let main_to_sub_impls = sub_enums
    .iter()
    .map(|sub| try_from_main_to_sub_impl(&input.ident, sub));

  // TODO: TryFrom sub enums that share variants.

  quote::quote! {
    #(#sub_enums)*

    #(#sub_enum_to_main_impls)*

    #(#main_to_sub_impls)*
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

impl quote::ToTokens for SubEnum {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    let name = &self.name;
    let derives = &self.derives;
    let delegates = &self.delegates;
    let docs = &self.docs;

    let variants = &self.variants;

    tokens.extend(quote::quote! {
      #(#[doc = #docs])*
      #[derive(#(#derives),*)]
      #(#[delegate(#delegates)])*
      pub enum #name {
        #(#variants),*
      }
    });
  }
}

impl quote::ToTokens for Variant {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    let variant_name = &self.name;
    let contained = &self.contained;
    tokens.extend(quote::quote! {
      #variant_name(#contained)
    });
  }
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

fn from_sub_to_main_impl(
  name: &syn::Ident,
  sub_enum: &SubEnum,
) -> proc_macro2::TokenStream {
  let sub_name = &sub_enum.name;

  let variants = sub_enum.variants.iter().map(|v| {
    let var_name = &v.name;
    quote::quote! {
      #sub_name::#var_name(v) => #name::#var_name(v)
    }
  });

  quote::quote! {
    impl From<#sub_name> for #name {
      fn from(value: #sub_name) -> Self {
        match value {
          #(#variants),*
        }
      }
    }
  }
}

fn try_from_main_to_sub_impl(
  name: &syn::Ident,
  sub_enum: &SubEnum,
) -> proc_macro2::TokenStream {
  let sub_name = &sub_enum.name;

  let variants = sub_enum.variants.iter().map(|v| {
    let var_name = &v.name;
    quote::quote! {
      #name::#var_name(v) => Ok(#sub_name::#var_name(v))
    }
  });

  let error =
    format!("Cannot convert sub enum `{sub_name}` to main enum `{name}`.");

  quote::quote! {
    impl TryFrom<#name> for #sub_name {
      type Error = &'static str;

      fn try_from(value: #name) -> Result<Self, Self::Error> {
        match value {
          #(#variants),*,
          _ => Err(#error),
        }
      }
    }
  }
}
