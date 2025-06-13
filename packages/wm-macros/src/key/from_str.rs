use proc_macro2::TokenStream;
use quote::quote;

use super::Key;

/// Creates a `from_str` implementation for the `Key` enum using the list
/// of keys.
pub fn make_from_str_impl(keys: &[Key]) -> TokenStream {
  let lines = keys.iter().map(|key| {
    let ident = &key.ident;
    if key.str_values.is_empty() {
      return quote! {};
    }

    let str_values = &key.str_values;

    // Create a `|` separated list of string literals for the match arm.
    let str_values = quote! {
      #(#str_values)|*
    };

    // Return the match arm tokens for this key.
    quote! {
      #str_values => Some(Self::#ident)
    }
  });

  quote! {
    pub fn from_str(key: &str) -> Option<Self> {
      // Unpack the match arms (lines) made above, using `,` as the separator.
      match key {
        #(#lines),*
        _ => {
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
      }
    }
  }
}
