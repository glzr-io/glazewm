use quote::quote;
use syn::{
  Token,
  parse::{ParseStream, discouraged::Speculative},
  spanned::Spanned,
};

use crate::spanned_string::SpannedString;

/// An enum variant attribute. Is either a ``#[key("string" | "list", win =
/// <key code>, macos = <key code>)]` or the wildcard `#[key(..)]` variant
/// used on the Custom() variant.
#[derive(Debug, Clone)]
pub enum VariantAttrs {
  Key(VariantKeyAttrs),
  Wildcard,
}

/// This struct holds the attributes for a key variant, including the
/// string list and each platform's key code.
/// Attributes are parsed from the `#[key("string" | "list", win = <key
/// code>, macos = <key code>)]`
#[derive(Debug, Clone)]
pub struct VariantKeyAttrs {
  pub str_values: VariantStringList,
  pub win_key: VariantVkValue,
  pub macos_key: VariantVkValue,
}

/// A platform's key code value for a variant.
#[derive(Debug, Clone)]
pub enum VariantVkValue {
  /// Should not be included at all
  None,
  /// Should only be included in the `into_vk` impl, as it is not a real
  /// key
  Virt(syn::Path),
  /// Normal Key
  Key(syn::Path),
}

impl VariantVkValue {
  pub fn span(&self) -> proc_macro2::Span {
    match self {
      VariantVkValue::None => proc_macro2::Span::call_site(),
      VariantVkValue::Virt(path) => path.span(),
      VariantVkValue::Key(path) => path.span(),
    }
  }
}

/// The string list for a variant from the #[key()] attribute, which
/// contains one or more strings that were split by the `|` character.
#[derive(Debug, Clone, Default)]
pub struct VariantStringList {
  pub strings: Vec<SpannedString>,
}

impl syn::parse::Parse for VariantAttrs {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    // Check if the input is a wildcard
    if input.peek(Token![..]) {
      input.parse::<Token![..]>()?;
      return Ok(VariantAttrs::Wildcard);
    }

    let attrs = input.parse::<VariantKeyAttrs>()?;

    Ok(VariantAttrs::Key(attrs))
  }
}

impl syn::parse::Parse for VariantKeyAttrs {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let str_values = input.parse::<VariantStringList>()?;
    // Expect a comma after the string values
    _ = input.parse::<Token![,]>()?;
    let key_codes = input.parse::<parsing::VariantKeyCodes>()?;
    let win_key = key_codes.win_key;
    let macos_key = key_codes.macos_key;

    Ok(VariantKeyAttrs {
      str_values,
      win_key,
      macos_key,
    })
  }
}

impl syn::parse::Parse for VariantVkValue {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    // Check for an identifier that is either `None` or `Virt`
    if input.peek(syn::Ident) {
      // Fork so we can advance to check the ident without consuming it
      let ident_fork = input.fork();
      let ident = ident_fork.parse::<syn::Ident>()?;
      if ident == "None" {
        input.advance_to(&ident_fork);
        return Ok(VariantVkValue::None);
      }
      if ident == "Virt" {
        // If the identifier is `Virt`, we expect a path to follow within
        // parentheses
        let content;
        _ = syn::parenthesized!(content in ident_fork);
        let path = content.parse::<syn::Path>()?;

        input.advance_to(&ident_fork);
        return Ok(VariantVkValue::Virt(path));
      }
    }

    let path = input.parse::<syn::Path>()?;
    Ok(VariantVkValue::Key(path))
  }
}

impl syn::parse::Parse for VariantStringList {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    // Parse the first string, which is required
    let mut strings =
      vec![input.parse::<SpannedString>().map_err(|err| {
        syn::Error::new(
          err.span(),
          "Expected a string value, or a string list seperated by `|`. Example: `\"enter\" | \"return\"`",
        )
      })?];

    // Iterate while the next token is the seperator
    while input.peek(Token![|]) {
      _ = input.parse::<Token![|]>()?;
      let next_string = input.parse::<SpannedString>()?;
      strings.push(next_string);
    }

    let mut validated_strings = Vec::new();

    // Validate and create any variants for each string
    for string in strings {
      if string.value.is_empty() {
        return Err(syn::Error::new_spanned(
          string,
          "String value cannot be empty",
        ));
      }

      if string.value.contains('+') {
        return Err(syn::Error::new_spanned(
          string,
          "String value cannot contain '+'",
        ));
      }
      if string.value.ends_with(' ') {
        return Err(syn::Error::new_spanned(
          string,
          "String value should not end with a space",
        ));
      }

      // Creates string variants such as ["num lock", "numlock",
      // "num_lock", "num-lock", "numLock"] from just "num lock"
      // "a" will just return ["a"] since it has no spaces.
      validated_strings.extend(parsing::get_string_variants(string));
    }

    Ok(VariantStringList {
      strings: validated_strings,
    })
  }
}

/// To format the string list back into an `|` seperated list.
impl quote::ToTokens for VariantStringList {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    let strings = &self.strings;

    // Output the string separated by `|` eg. "a" | "b"
    tokens.extend(quote! { #(#strings)|* });
  }
}

/// Holds structs and methods that are only used by the structs above for
/// parsing
mod parsing {
  use super::*;

  /// Contains the key codes for each platform.
  #[derive(Debug, Clone)]
  pub struct VariantKeyCodes {
    pub win_key: VariantVkValue,
    pub macos_key: VariantVkValue,
  }

  impl syn::parse::Parse for VariantKeyCodes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
      let mut win_key = None;
      let mut macos_key = None;

      let mut is_first_loop = true;

      while !input.is_empty() {
        if !is_first_loop {
          _ = input.parse::<Token![,]>()?;
        } else {
          is_first_loop = false;
        }

        let key_code = input.parse::<AnyKeyCode>()?;

        match key_code {
          AnyKeyCode::Win(win) => {
            if win_key.is_some() {
              return Err(syn::Error::new(
                win.vk_value.span(),
                "Duplicate `win` key code",
              ));
            }
            win_key = Some(win.vk_value);
          }
          AnyKeyCode::Macos(macos) => {
            if macos_key.is_some() {
              return Err(syn::Error::new(
                macos.vk_value.span(),
                "Duplicate `macos` key code",
              ));
            }
            macos_key = Some(macos.vk_value);
          }
        }
      }

      let win_key = win_key.ok_or_else(|| {
        syn::Error::new(
          input.span(),
          "Missing `win` key code. Example: `win = <key code>`",
        )
      })?;

      let macos_key = macos_key.ok_or_else(|| {
        syn::Error::new(
          input.span(),
          "Missing `macos` key code. Example: `macos = <key code>`",
        )
      })?;

      Ok(VariantKeyCodes { win_key, macos_key })
    }
  }

  /// Can be either a Windows or macOS key code. This is used to
  /// parse the `win = <key code>` or `macos = <key code>`
  #[derive(Debug, Clone)]
  enum AnyKeyCode {
    Win(WinKeyCode),
    Macos(MacosKeyCode),
  }

  impl syn::parse::Parse for AnyKeyCode {
    fn parse(input: ParseStream) -> syn::Result<Self> {
      let fork = input.fork();

      let ident_fork = fork.fork();

      let ident = ident_fork.parse::<syn::Ident>().map_err(|err| syn::Error::new(
          err.span(),
          "Expected either `win` or `macos` key specifiers. Eg. `win = <key code>` or `macos = <key code>`",
        ))?;

      if ident == "win" {
        let win_key = fork.parse::<WinKeyCode>()?;
        input.advance_to(&fork);
        Ok(AnyKeyCode::Win(win_key))
      } else if ident == "macos" {
        let macos_key = fork.parse::<MacosKeyCode>()?;
        input.advance_to(&fork);
        Ok(AnyKeyCode::Macos(macos_key))
      } else {
        Err(syn::Error::new(
          input.span(),
          "Expected either `win` or `macos` key specifiers. Eg. `win = <key code>` or `macos = <key code>`",
        ))
      }
    }
  }

  /// Windows key code, which is parsed from the `win = <key code>`
  /// Wrapper to implement the `syn::parse::Parse` trait for
  #[derive(Debug, Clone)]
  struct WinKeyCode {
    pub vk_value: VariantVkValue,
  }

  impl syn::parse::Parse for WinKeyCode {
    fn parse(input: ParseStream) -> syn::Result<Self> {
      let ident = input.parse::<syn::Ident>()?;
      if ident != "win" {
        return Err(syn::Error::new_spanned(
          &ident,
          "Expected `win` identifier for Windows key code",
        ));
      }

      _ = input.parse::<Token![=]>()?;

      let vk_value = input.parse::<VariantVkValue>().map_err(|err| {
        syn::Error::new(
          err.span(),
          "Expected a valid Windows key code. Example: `win = <key code>`",
        )
      })?;

      Ok(WinKeyCode { vk_value })
    }
  }

  /// MacOS key code, which is parsed from the `macos = <key code>`
  /// Wrapper to implement the `syn::parse::Parse` trait for
  #[derive(Debug, Clone)]
  struct MacosKeyCode {
    pub vk_value: VariantVkValue,
  }

  impl syn::parse::Parse for MacosKeyCode {
    fn parse(input: ParseStream) -> syn::Result<Self> {
      let ident = input.parse::<syn::Ident>()?;
      if ident != "macos" {
        return Err(syn::Error::new_spanned(
          &ident,
          "Expected `macos` identifier for macOS key code",
        ));
      }

      _ = input.parse::<Token![=]>()?;

      let vk_value = input.parse::<VariantVkValue>().map_err(|err| {
        syn::Error::new(
          err.span(),
          "Expected a valid macOS key code. Example: `macos = <key code>`",
        )
      })?;

      Ok(MacosKeyCode { vk_value })
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
}
