use syn::parse::ParseStream;

use crate::{
  Os,
  common::{
    error_handling::ErrorContext,
    lookahead::{LookaheadPeekThenAdvance, PeekThenAdvance},
  },
};

/// Custom keywords used when parsing the enum attributes.
// Custom keywords can be parsed and peeked, which is better than forking
// the stream to parse an ident and then check if it matches a string.
mod kw {
  syn::custom_keyword!(win_prefix);
  syn::custom_keyword!(macos_prefix);
  syn::custom_keyword!(None);
}

/// Holds the attributes for an enum that contains platform-specific
/// prefixes
pub struct EnumAttr {
  pub win_prefix: Option<syn::Path>,
  pub macos_prefix: Option<syn::Path>,
}

// Parse the `#[key(...)]` attribute on the enum to extract the prefixes.
impl syn::parse::Parse for EnumAttr {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let mut win_prefix = None;
    let mut macos_prefix = None;

    // Could use input.parse_terminated here, but this allows to skip
    // parsing a prefix if we already have it.

    while !input.is_empty() {
      // Lookaheads can be used to peek at the next token without consuming
      // it, `lookahead.error()` returns an error with a formated string
      // containing all `peek`s that failed.
      let lookahead = input.lookahead1();

      // If we havn't seen a `win_prefix` yet, check if the next token
      // is `win_prefix`, and if it is, parse it.
      let os = if win_prefix.is_none()
        && lookahead
          .peek_then_advance::<kw::win_prefix, _>(kw::win_prefix, input)
          .is_some()
      {
        Os::Windows
        // Same with `macos_prefix`
      } else if macos_prefix.is_none()
        && lookahead
          .peek_then_advance::<kw::macos_prefix, _>(
            kw::macos_prefix,
            input,
          )
          .is_some()
      {
        Os::MacOS
      } else {
        return Err(lookahead.error());
      };

      // Consume the `=` token after the `win_prefix` or `macos_prefix`,
      // erroring if its not present.
      _ = input.parse::<syn::Token![=]>()?;

      // Parse the prefix for the platforms key codes, which is a path or
      // None.
      let prefix = if input
        .peek_then_advance::<kw::None, _>(kw::None)
        .is_some()
      {
        None
      } else {
        Some(input.parse::<syn::Path>().add_context("Expected a prefix for the platforms key codes, or `None` for no prefix.")?)
      };

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
