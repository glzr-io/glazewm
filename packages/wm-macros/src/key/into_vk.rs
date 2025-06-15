use proc_macro2::TokenStream;
use quote::quote;

use super::{
  Key,
  attrs::{enums::EnumAttr, variant::VariantVkValue},
};

/// Wrapper type to implement the `to_*_tokens` methods for the `into_vk`
/// impl.
struct KeyIntoVkArm<'a> {
  ident: syn::Ident,
  attrs: super::VariantAttrs,
  enum_attrs: &'a EnumAttr,
}

impl<'a> KeyIntoVkArm<'a> {
  pub fn new(key: &Key, enum_attrs: &'a EnumAttr) -> Self {
    Self {
      ident: key.ident.clone(),
      attrs: key.attrs.clone(),
      enum_attrs,
    }
  }
}

impl<'a> KeyIntoVkArm<'a> {
  /// Creates the match arms for the `into_vk` impl for the
  /// windows platform.
  pub fn to_win_tokens(&self, tokens: &mut TokenStream) {
    let ident = &self.ident;
    match &self.attrs {
      super::VariantAttrs::Wildcard => {
        // If the key is a wildcard, we match it to the `Custom` variant.
        tokens.extend(quote! { Self::Custom(vk) => vk,});
      }
      super::VariantAttrs::Key(key_attrs) => {
        let value = &key_attrs.win_key;
        let prefix = &self.enum_attrs.win_prefix;
        // Output the match arms.
        match value {
          VariantVkValue::Virt(value) => {
            // For virtual keys, we return the value directly.
            tokens.extend(quote! {Self::#ident => #prefix::#value.0,});
          }
          VariantVkValue::Key(value) => {
            // For regular keys, we return the value from the enum.
            tokens.extend(quote! {Self::#ident => #prefix::#value.0,});
          }
          _ => {}
        }
      }
    }
  }

  /// Creates the match arms for the `into_vk` impl for the
  /// macOS platform.
  pub fn to_macos_tokens(&self, tokens: &mut TokenStream) {
    let ident = &self.ident;
    match &self.attrs {
      super::VariantAttrs::Wildcard => {
        // If the key is a wildcard, we match it to the `Custom` variant.
        tokens.extend(quote! { Self::Custom(vk) => vk,});
      }
      super::VariantAttrs::Key(key_attrs) => {
        let value = &key_attrs.macos_key;
        let prefix = &self.enum_attrs.macos_prefix;
        // Output the match arms.
        match value {
          VariantVkValue::Virt(value) => {
            // For virtual keys, we return the value directly.
            tokens.extend(quote! {Self::#ident => #prefix::#value.0,});
          }
          VariantVkValue::Key(value) => {
            // For regular keys, we return the value from the enum.
            tokens.extend(quote! {Self::#ident => #prefix::#value.0,});
          }
          _ => {}
        }
      }
    }
  }
}

/// Creates a `into_vk` implementation for the `Key` enum using the given
/// keys.
pub fn make_into_vk_impl(
  keys: &[Key],
  enum_attrs: &EnumAttr,
) -> TokenStream {
  let lines = keys
    .iter()
    .map(|key| KeyIntoVkArm::new(key, enum_attrs))
    .collect::<Vec<_>>();

  let win_lines = lines.iter().map(|key| {
    let mut tokens = TokenStream::new();
    key.to_win_tokens(&mut tokens);
    tokens
  });

  let macos_lines = lines.iter().map(|key| {
    let mut tokens = TokenStream::new();
    key.to_macos_tokens(&mut tokens);
    tokens
  });

  quote! {
    #[cfg(target_os = "windows")]
    pub fn into_vk(self) -> u16 {
      match self {
        #(#win_lines)*
        _ => { unreachable!("Key not found in Windows VK mapping"); }
      }
    }

    #[cfg(target_os = "macos")]
    pub fn into_vk(self) -> u16 {
      match self {
        #(#macos_lines)*
        _ => { unreachable!("Key not found in macOS VK mapping"); }
      }
    }
  }
}
