use proc_macro2::TokenStream;
use quote::quote;

use super::{
  Key,
  attrs::{enums::EnumAttr, variant::VkValue},
};
use crate::Os;

/// Creates the match arms for the `into_vk` impl for the given key and Os.
fn to_match_arm(key: &Key, enum_attrs: &EnumAttr, os: Os) -> TokenStream {
  let ident = &key.ident;
  match &key.attrs {
    super::VariantAttr::Wildcard => {
      // If the key is a wildcard, we match it to the `Custom` variant.
      quote! { Self::Custom(vk) => vk }
    }
    super::VariantAttr::Key(key_attrs) => {
      let (value, prefix) = match os {
        Os::Windows => (&key_attrs.key_codes.win, &enum_attrs.win_enum),
        Os::MacOS => (&key_attrs.key_codes.macos, &enum_attrs.macos_enum),
        Os::Linux => (&key_attrs.key_codes.linux, &enum_attrs.linux_enum),
      };

      // Output the match arms.
      match value {
        VkValue::Key(value) => {
          quote! { Self::#ident => #prefix::#value as u16}
        }
        VkValue::Virt(value) => {
          quote! { Self::#ident => #prefix::#value as u16 }
        }
        _ => quote! {},
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
    pub fn into_vk(self) -> u16 {
      // The comma is inside the brackes so that a trailing comma is generated for the last arm.
      match self {
        #(#win_arms,)*
        _ => { unreachable!("Key not found in Windows VK mapping"); }
      }
    }

    #[cfg(target_os = "macos")]
    pub fn into_vk(self) -> u16 {
      match self {
        #(#mac_arms,)*
        _ => { unreachable!("Key not found in macOS VK mapping"); }
      }
    }

    #[cfg(target_os = "linux")]
    pub fn into_vk(self) -> u16 {
      match self {
        #(#linux_arms,)*
        _ => { unreachable!("Key not found in linux VK mapping"); }
      }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    pub fn into_vk(self) -> u16 {
      compile_error!("`into_vk` is not supported on this OS at this time.");
      return 0; // This line is unreachable, but needed to satisfy the function signature.
    }
  }
}
