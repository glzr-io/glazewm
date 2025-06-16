use syn::parse::ParseStream;

use crate::{
  Either, Os,
  common::{
    branch::Alt, error_handling::ErrorContext, lookahead::PeekThenAdvance,
    named_parameter::ParseNamedParameter,
  },
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
    let mut win_prefix = None;
    let mut macos_prefix = None;

    while !input.is_empty() {
      let (os, prefix) = input.parse_named_parameter_with(
        |input| {
          input
            .alt_if::<kw::win_prefix, kw::macos_prefix>(
              win_prefix.is_none(),
              macos_prefix.is_none(),
            )
            .map(|kw| match kw {
              Either::Left(_) => Os::Windows,
              Either::Right(_) => Os::MacOS,
            })
        },
        PlatformPrefix::parse,
      )?;

      match os {
        Os::Windows => {
          win_prefix = Some(prefix);
        }
        Os::MacOS => {
          macos_prefix = Some(prefix);
        }
      }

      if !input.is_empty() {
        // If there are more tokens, consume the `,` token.
        _ = input.parse::<syn::Token![,]>()?;
      }
    }

    // Ensure that both prefixes are present
    let win_prefix =
      win_prefix.ok_or(input.error("Missing `win_prefix` attribute"))?;

    let macos_prefix = macos_prefix
      .ok_or(input.error("Missing `macos_prefix` attribute"))?;

    Ok(EnumAttr {
      win_prefix,
      macos_prefix,
    })
  }
}
