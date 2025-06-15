use proc_macro2::TokenStream;
use quote::quote;

use super::{
  Key,
  attrs::{enums::EnumAttr, variant::VariantVkValue},
};

/// Wraps a key enum variant and its attributes, providing methods to
/// create match arms for the `from_vk` implementation for each platform
struct KeyFromVkArm<'a> {
  ident: syn::Ident,
  attrs: super::VariantAttrs,
  enum_attrs: &'a EnumAttr,
}

impl<'a> KeyFromVkArm<'a> {
  pub fn new(key: &Key, enum_attrs: &'a EnumAttr) -> Self {
    Self {
      ident: key.ident.clone(),
      attrs: key.attrs.clone(),
      enum_attrs,
    }
  }
}

impl<'a> KeyFromVkArm<'a> {
  /// Creates the match arms for the `from_vk` impl for the
  /// windows platform.
  pub fn to_win_tokens(&self, tokens: &mut TokenStream) {
    let ident = &self.ident;
    match &self.attrs {
      super::VariantAttrs::Wildcard => {
        // If the key is a wildcard, we match it to the `Custom` variant.
        tokens.extend(quote! { _ => Self::Custom(vk),});
      }
      super::VariantAttrs::Key(key_attrs) => {
        let value = &key_attrs.win_key;
        let prefix = &self.enum_attrs.win_prefix;

        // Output the match arms.
        if let VariantVkValue::Key(value) = value {
          tokens.extend(quote! {#prefix::#value => Self::#ident,});
        }
      }
    }
  }

  /// Creates the match arms for the `from_vk` impl for the
  /// macOS platform.
  pub fn to_mac_tokens(&self, tokens: &mut TokenStream) {
    let ident = &self.ident;
    match &self.attrs {
      super::VariantAttrs::Wildcard => {
        // If the key is a wildcard, we match it to the `Custom` variant.
        tokens.extend(quote! { _ => Self::Custom(vk),});
      }
      super::VariantAttrs::Key(key_attrs) => {
        let value = &key_attrs.macos_key;
        let prefix = &self.enum_attrs.macos_prefix;

        // Output the match arms.
        if let VariantVkValue::Key(value) = value {
          tokens.extend(quote! {#prefix::#value => Self::#ident,});
        }
      }
    }
  }
}

/// Creates a `from_vk` implementation for the `Key` enum using the given
/// keys.
pub fn make_from_vk_impl(
  keys: &[Key],
  enum_attrs: &EnumAttr,
) -> TokenStream {
  let keys = keys
    .iter()
    .map(|key| KeyFromVkArm::new(key, enum_attrs))
    .collect::<Vec<_>>();

  let win_lines = keys.iter().map(|key| {
    let mut tokens = TokenStream::new();
    key.to_win_tokens(&mut tokens);
    tokens
  });

  let macos_lines = keys.iter().map(|key| {
    let mut tokens = TokenStream::new();
    key.to_mac_tokens(&mut tokens);
    tokens
  });

  quote! {
    #[cfg(target_os = "windows")]
    pub fn from_vk(vk: u16) -> Self {
      match ::windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(vk) {
        #(#win_lines)*
      }
    }

    #[cfg(target_os = "macos")]
    pub fn from_vk(vk: u16) -> Self {
      match vk {
        #(#macos_lines)*
      }
    }
  }
}
