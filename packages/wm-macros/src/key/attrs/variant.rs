use quote::quote;
use syn::{Token, parse::ParseStream};

use crate::common::{
  branch::Unordered,
  error_handling::{ErrorContext, ThenError as _, ToError as _},
  lookahead::{LookaheadPeekThenAdvance, PeekThenAdvance as _},
  named_parameter::NamedParameter,
  spanned_string::SpannedString,
};

/// Custom keywords used when parsing the variant attributes.
// Custom keywords can be parsed and peeked, which is better than
// Forking the stream to parse an ident and then check if it matches a
// string.
mod kw {
  use crate::common::custom_keyword;

  custom_keyword!(win);
  custom_keyword!(macos);
  custom_keyword!(Virt);
  custom_keyword!(None);
}

/// An enum variant attribute. Is either a KeyAttr (`#[key("string" |
/// "list", win = <key code>, macos = <key code>)]`) or the wildcard
/// (`#[key(..)]`) variant used on the Custom() variant.
#[derive(Debug, Clone)]
pub enum VariantAttr {
  Key(KeyAttr),
  Wildcard,
}

impl syn::parse::Parse for VariantAttr {
  /// Parses the variant attribute, which can be either a wildcard pattert
  /// or a key.
  /// Expected format:
  /// `..` for the wildcard variant, or
  /// `"string" | "list", win = <key code>, macos = <key code>` for the key
  fn parse(input: ParseStream) -> syn::Result<Self> {
    // Check if the input is a wildcard
    if input.peek_then_advance::<Token![..]>().is_some() {
      Ok(VariantAttr::Wildcard)
    } else {
      // Else, try to parse a KeyAttr.
      input.parse().map(VariantAttr::Key)
    }
  }
}

/// This struct holds the attributes for a key variant, including the
/// string list and each platform's key code.
/// Attributes are parsed from the `#[key("string" | "list", win = <key
/// code>, macos = <key code>)]`
#[derive(Debug, Clone)]
pub struct KeyAttr {
  pub strings: VariantStringList,
  pub key_codes: PlatformKeyCodes,
}

impl syn::parse::Parse for KeyAttr {
  /// Parses the key parameters from a key attribute.
  /// Expected format:
  /// `"string" | "list", win = <key code>, macos = <key code>`
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let strings = input.parse()?;
    _ = input.parse::<Token![,]>()?;
    let key_codes = input.parse()?;
    Ok(KeyAttr { strings, key_codes })
  }
}

/// Contains the key codes for each platform.
/// Parsed from the `win = <key code>, macos = <key code>` part of the
/// variant `#[key(...)]` attribute.
#[derive(Debug, Clone)]
pub struct PlatformKeyCodes {
  pub win: VkValue,
  pub macos: VkValue,
}

impl syn::parse::Parse for PlatformKeyCodes {
  /// Parses platform key codes from the variant attribute,
  /// Expected format: `win = <key code>, macos = <key code>` (or vice
  /// versa).
  fn parse(input: ParseStream) -> syn::Result<Self> {
    type WinParam = NamedParameter<kw::win, VkValue>;
    type MacOSParam = NamedParameter<kw::macos, VkValue>;

    let (win, macos) = input
      .parse::<Unordered<(WinParam, MacOSParam), Token![,]>>()
      .map(|Unordered((win, macos), _)| (win.param, macos.param))?;

    Ok(PlatformKeyCodes { win, macos })
  }
}

/// A platform's key code value for a variant. Can be one of:
/// None (not included), Virt (key alias, not a real key), or a
/// standard key
#[derive(Debug, Clone)]
pub enum VkValue {
  /// Should not be included at all. Shows that the keybind does not exist
  /// on this platform.
  None,
  /// A key alias that does not correspond to a real key, but is a valid
  /// keybind. Such as the `Win` keybind does not have a real key code,
  /// but is a valid keybind for both LWin and RWin.
  /// A `from_vk`match arm will not be generated for this type of
  /// VkValue.
  Virt(syn::Path),
  /// Normal Key
  Key(syn::Path),
}

impl syn::parse::Parse for VkValue {
  /// Parses a VKValue, which can be `None`, `Virt(<ident / path>)` or
  /// `<ident / path>`
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let lookahead = input.lookahead1();
    if lookahead.peek_then_advance::<kw::None>(input).is_some() {
      Ok(VkValue::None)
    } else if lookahead.peek_then_advance::<kw::Virt>(input).is_some() {
      let content;
      _ = syn::parenthesized!(content in input);
      let path = content
        .parse()
        .add_context("Expected the virtual key code, eg. `Virt(VK_A)`")?;

      Ok(VkValue::Virt(path))
    } else {
      let path = input.parse()?;
      Ok(VkValue::Key(path))
    }
  }
}

/// The string list for a variant from the #[key(...)] attribute, which
/// contains one or more strings that were split by the `|` character.
#[derive(Debug, Clone, Default)]
pub struct VariantStringList {
  pub strings: Vec<SpannedString>,
}

impl syn::parse::Parse for VariantStringList {
  /// Parses a string list from the input, which can be a single string or
  /// a list of strings separated by `|`, eg. `"string"` or `"string" |
  /// "list"`.
  fn parse(input: ParseStream) -> syn::Result<Self> {
    // Parse the first string, which is required
    let mut strings =
      vec![input.parse().add_context("Expected a string value, or a string list seperated by `|`. Example: `\"enter\" | \"return\"`")?];

    // Iterate while the next token is the seperator, advancing over the
    // separator to parse the next string.
    while input.peek_then_advance::<Token![|]>().is_some() {
      strings.push(input.parse::<SpannedString>()?);
    }

    let mut validated_strings = Vec::new();

    // Validate and create any variants for each string
    for string in strings {
      let value = &string.value;
      value
        .is_empty()
        .then_error(string.error("String value cannot be empty"))?;

      value
        .contains('+')
        .then_error(string.error("String value cannot contain '+'"))?;

      value.ends_with(' ').then_error(
        string.error("String value should not end with a space"),
      )?;

      // Creates string variants such as ["num lock", "numlock",
      // "num_lock", "num-lock", "numLock"] from just "num lock"
      // "a" will just return ["a"] since it has no spaces.
      validated_strings.extend(get_string_variants(string));
    }

    Ok(VariantStringList {
      strings: validated_strings,
    })
  }
}

impl quote::ToTokens for VariantStringList {
  /// Formats the string list back into an `|` seperated list.
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    let strings = &self.strings;

    // Output the string separated by `|` eg. "a" | "b"
    tokens.extend(quote! { #(#strings)|* });
  }
}

/// Get string variants from a `SpannedString`.
/// "a" will just return ["a"] since it has no spaces.
/// "num lock" will return: ["num lock", "numlock", "num_lock",
/// "num-lock", "numLock"]. Camel case is only included if it is
/// different from the no spaces version (to prevent some strings such
/// as `numlock 0` from returning duplicates).
pub fn get_string_variants(string: SpannedString) -> Vec<SpannedString> {
  if string.value.contains(' ') {
    let no_spaces =
      SpannedString::new(string.value.replace(' ', ""), string.span);

    let underscored =
      SpannedString::new(string.value.replace(' ', "_"), string.span);

    let dashed =
      SpannedString::new(string.value.replace(' ', "-"), string.span);

    let camel_cased = SpannedString::new(
      string.value.split(' ').fold(String::new(), |acc, el| {
        if acc.is_empty() {
          return el.to_string();
        }
        let mut chars = el.chars();
        let first_char = chars.next().unwrap();
        let rest = chars.as_str();

        let first_char = first_char.to_uppercase().to_string();

        let mut new_el = acc.to_string();
        new_el.push_str(&first_char);
        new_el.push_str(rest);

        new_el
      }),
      string.span,
    );

    let base_string =
      SpannedString::new(string.value.clone(), string.span);

    if camel_cased.value != no_spaces.value {
      vec![base_string, no_spaces, underscored, dashed, camel_cased]
    } else {
      vec![base_string, no_spaces, underscored, dashed]
    }
  } else {
    vec![string]
  }
}
