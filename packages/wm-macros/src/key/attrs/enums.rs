use syn::parse::ParseStream;

use crate::common::{branch::Unordered, named_parameter::NamedParameter};

/// Custom keywords used when parsing the enum attributes.
// Custom keywords can be parsed and peeked, which is better than forking
// the stream to parse an ident and then check if it matches a string.
mod kw {
  use crate::common::custom_keyword;

  custom_keyword!(win);
  custom_keyword!(macos);
  custom_keyword!(linux);
  custom_keyword!(None);
}

/// Holds the attributes for an enum that contains platform-specific
/// prefixes
pub struct EnumAttr {
  pub win_enum: syn::Type,
  pub macos_enum: syn::Type,
  pub linux_enum: syn::Type,
}

// Parse the `#[key(...)]` attribute on the enum to extract the prefixes.
impl syn::parse::Parse for EnumAttr {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    type WinPrefixParam = NamedParameter<kw::win, syn::Type>;
    type MacOSPrefixParam = NamedParameter<kw::macos, syn::Type>;
    type LinuxPrefixParam = NamedParameter<kw::linux, syn::Type>;

    let (win_enum, macos_enum, linux_enum) = input
      .parse::<Unordered<
        (WinPrefixParam, MacOSPrefixParam, LinuxPrefixParam),
        syn::Token![,],
      >>()
      .map(
        |Unordered {
           items: (win_enum, macos_enum, linux_enum),
           ..
         }| {
          (win_enum.param, macos_enum.param, linux_enum.param)
        },
      )?;

    Ok(EnumAttr {
      win_enum,
      macos_enum,
      linux_enum,
    })
  }
}
