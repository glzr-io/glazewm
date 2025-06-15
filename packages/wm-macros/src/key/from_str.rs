use proc_macro2::TokenStream;
use quote::quote;

use super::{Key, attrs::variant::VariantAttrs};

/// Wrapper type to implement the `to_tokens` method for the `from_str`
/// impl.
struct KeyStrArm {
  ident: syn::Ident,
  attrs: VariantAttrs,
}

impl From<Key> for KeyStrArm {
  fn from(key: Key) -> Self {
    KeyStrArm {
      ident: key.ident,
      attrs: key.attrs,
    }
  }
}

/// Converts a `KeyStrArm` into a match arm for the `from_str`
/// implementation.
impl quote::ToTokens for KeyStrArm {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let ident = &self.ident;
    match &self.attrs {
      VariantAttrs::Wildcard => {
        // If the key is a wildcard, we match it to the `Custom` variant.
        tokens.extend(quote! { _ => {
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
        }});
      }
      super::VariantAttrs::Key(key_attrs) => {
        // For regular keys, we match the string values to the key variant.
        let str_values = &key_attrs.str_values;

        // Output the match arms.
        tokens.extend(quote! {#str_values => Some(Self::#ident)});
      }
    }
  }
}

/// Creates a `from_str` implementation for the `Key` enum using the list
/// of keys.
pub fn make_from_str_impl(keys: &[Key]) -> TokenStream {
  let lines = keys.iter().map(|key| KeyStrArm::from(key.clone()));

  quote! {
    pub fn from_str(key: &str) -> Option<Self> {
      // Unpack the match arms (lines) made above, using `,` as the separator.
      match key {
        #(#lines),*
      }
    }
  }
}
