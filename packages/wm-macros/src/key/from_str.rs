use proc_macro2::TokenStream;
use quote::quote;

use super::{Key, attrs::variant::VariantAttr};

/// Converts a `KeyStrArm` into a match arm for the `from_str`
/// implementation.
fn to_match_arm(key: &Key) -> TokenStream {
  let ident = &key.ident;
  match &key.attrs {
    VariantAttr::Wildcard => {
      // If the key is a wildcard, we match it to the `Custom` variant.
      quote! { _ => {
          #[cfg(target_os = "windows")]
          {
          // Check if the key exists on the current keyboard layout.
          let mut encoding = key.encode_utf16();
          let utf16_key = encoding.next()?;

          if encoding.next().is_some() {
            return None; // Only single-character keys are supported.
          }

          let layout = unsafe { ::windows::Win32::UI::Input::KeyboardAndMouse::GetKeyboardLayout(0) };
          let vk_code = unsafe { ::windows::Win32::UI::Input::KeyboardAndMouse::VkKeyScanExW(utf16_key, layout) };

          if vk_code == -1 {
            return None;
          }

          // The low-order byte contains the virtual-key code and the high-
          // order byte contains the shift state.
          let [high_order, low_order] = vk_code.to_be_bytes();

          // Key is valid if it doesn't require shift or alt to be pressed.
          match high_order {
            0 => Some(Key::Custom(u16::from(low_order))),
            _ => None,
          }
          }
          #[cfg(target_os = "macos")]
          {
            // TODO: Verify if a key is on a macOS keyboard layout.
            None
          }

          #[cfg(target_os = "linux")]
          {
            // TODO: Check if the key exists on the current keyboard layout for linux.
            None
          }
        }
      }
    }
    super::VariantAttr::Key(key_attrs) => {
      // For regular keys, we match the string values to the key variant.
      let str_values = &key_attrs.strings;

      // Output the match arms.
      quote! {#str_values => Some(Self::#ident)}
    }
  }
}

/// Creates a `from_str` implementation for the `Key` enum using the list
/// of keys.
pub fn make_from_str_impl(keys: &[Key]) -> TokenStream {
  let arms = keys.iter().map(to_match_arm).filter(|arm| !arm.is_empty());

  quote! {
    pub fn from_str(key: &str) -> Option<Self> {
      // Unpack the match arms (lines) made above, using `,` as the separator.
      match key {
        #(#arms),*
      }
    }
  }
}
