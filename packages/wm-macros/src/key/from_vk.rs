use proc_macro2::TokenStream;
use quote::quote;

use super::{
  Key,
  attrs::{enums::EnumAttr, variant::VkValue},
};
use crate::Os;

/// Creates the match arms for the `from_vk` impl for the given key and Os.
fn to_match_arm(key: &Key, enum_attrs: &EnumAttr, os: Os) -> TokenStream {
  let ident = &key.ident;
  match &key.attrs {
    super::VariantAttr::Key(key_attrs) => {
      let (value, prefix) = match os {
        Os::Windows => (&key_attrs.key_codes.win, &enum_attrs.win_enum),
        Os::MacOS => (&key_attrs.key_codes.macos, &enum_attrs.macos_enum),
        Os::Linux => (&key_attrs.key_codes.linux, &enum_attrs.linux_enum),
      };

      // Output the match arms.
      if let VkValue::Key(value) = value {
        quote! {#prefix::#value => Self::#ident}
      } else {
        quote! {}
      }
    }
    _ => {
      quote! {}
    }
  }
}

/// Creates a `from_vk` implementation for the `Key` enum using the given
/// keys.
pub fn make_from_vk_impl(
  keys: &[Key],
  enum_attrs: &EnumAttr,
) -> TokenStream {
  let win_enum = &enum_attrs.win_enum;
  let macos_prefix = &enum_attrs.macos_enum;
  let linux_prefix = &enum_attrs.linux_enum;

  let win_arms = keys
    .iter()
    .map(|key| to_match_arm(key, enum_attrs, Os::Windows))
    .filter(|arm| !arm.is_empty());

  let mac_arms = keys
    .iter()
    .map(|key| to_match_arm(key, enum_attrs, Os::MacOS))
    .filter(|arm| !arm.is_empty());

  let linux_arms = keys
    .iter()
    .map(|key| to_match_arm(key, enum_attrs, Os::Linux))
    .filter(|arm| !arm.is_empty());

  quote! {
    #[cfg(target_os = "windows")]
    pub fn from_vk(vk: u16) -> Self {
      let res = #win_enum::try_from(vk);
      if let Ok(key) = res {
        match key {
          #(#win_arms),*
        }
      } else {
        Self::Custom(vk)
      }
    }

    #[cfg(target_os = "macos")]
    pub fn from_vk(vk: u16) -> Self {
      let res = #macos_prefix::try_from(vk);
      if let Ok(key) = res {
        match key {
          #(#mac_arms),*
        }
      } else {
        Self::Custom(vk)
      }
    }

    #[cfg(target_os = "linux")]
    pub fn from_vk(vk: u16) -> Self {
      let res = #linux_prefix::try_from(vk);
      if let Ok(key) = res {
        match key {
          #(#linux_arms),*
        }
      } else {
        Self::Custom(vk)
      }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    pub fn from_vk(_vk: u16) -> Self {
      compile_error!("`from_vk` is not supported on this OS at this time.");
      Self::Custom(0) // Return a default value to satisfy the function signature.
    }
  }
}
