use proc_macro2::TokenStream;
use quote::quote;

use super::Key;

/// Creates a `from_vk` implementation for the `Key` enum using the given
/// keys.
pub fn make_from_vk_impl(keys: &[Key]) -> TokenStream {
  let lines = keys.iter().map(|key| {
    let ident = &key.ident;
    // The `Custom` variant is handled manually in the wildcard.
    if ident == "Custom" {
      return quote! {};
    }

    let vk_value = &key.vk_value;

    // Create the match arm tokens for this key.
    quote! {
      ::windows::Win32::UI::Input::KeyboardAndMouse::#vk_value => Self::#ident
    }
  });

  quote! {
    pub fn from_vk(vk: u16) -> Self {
      match ::windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(vk) {
        // Unpack the match arms (lines) made above, using `,` as the separator.
        #(#lines),*
        // Handle the `Custom` variant separately
        _ => {
          Self::Custom(vk)
        }
      }
    }
  }
}
