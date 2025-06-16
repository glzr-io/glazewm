use syn::parse::ParseStream;

use crate::common::{
  branch::Combined, error_handling::ErrorContext,
  lookahead::PeekThenAdvance, named_parameter::NamedParameter,
};

/// Custom keywords used when parsing the enum attributes.
// Custom keywords can be parsed and peeked, which is better than forking
// the stream to parse an ident and then check if it matches a string.
mod kw {
  use crate::common::custom_keyword;

  custom_keyword!(win_prefix);
  custom_keyword!(macos_prefix);
  custom_keyword!(None);
}

/// Holds the attributes for an enum that contains platform-specific
/// prefixes
pub struct EnumAttr {
  pub win_prefix: PlatformPrefix,
  pub macos_prefix: PlatformPrefix,
}

pub enum PlatformPrefix {
  None,
  Some(syn::Path),
}

impl syn::parse::Parse for PlatformPrefix {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    if input.peek_then_advance::<kw::None>().is_some() {
      Ok(PlatformPrefix::None)
    } else {
      Ok(PlatformPrefix::Some(input.parse::<syn::Path>()
        .add_context("Expected a prefix for the platforms key codes, or `None` for no prefix.")?))
    }
  }
}

impl quote::ToTokens for PlatformPrefix {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    match self {
      PlatformPrefix::None => tokens.extend(quote::quote! {}),
      PlatformPrefix::Some(prefix) => {
        tokens.extend(quote::quote! { #prefix:: })
      }
    }
  }
}

// Parse the `#[key(...)]` attribute on the enum to extract the prefixes.
impl syn::parse::Parse for EnumAttr {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    type WinPrefixParam = NamedParameter<kw::win_prefix, PlatformPrefix>;
    type MacOSPrefixParam =
      NamedParameter<kw::macos_prefix, PlatformPrefix>;

    let (win_prefix, macos_prefix) = input.parse_all_unordered::<(WinPrefixParam, MacOSPrefixParam), syn::Token![,]>()
      .map(|(win_prefix, macos_prefix)| {
        (win_prefix.param, macos_prefix.param)
      })?;

    Ok(EnumAttr {
      win_prefix,
      macos_prefix,
    })
  }
}
