use proc_macro2::TokenStream;
use quote::quote;

use super::Key;

pub fn make_into_vk_impl(keys: &[Key]) -> TokenStream {
  let lines = keys.iter().map(|key| {
    let ident = &key.ident;
    // The `Custom` variant is handled manually in the wildcard.
    if ident == "Custom" {
      return quote! {Self::Custom(vk) => vk};
    }

    let vk_value = &key.vk_value;

    // Return the match arm tokens for this key.
    quote! {
      Self::#ident => ::windows::Win32::UI::Input::KeyboardAndMouse::#vk_value.0
    }
  });

  quote! {
    pub fn into_vk(self) -> u16 {
      match self {
        // Unpack the match arms (lines) made above, using `,` as the separator.
        #(#lines),*
      }
    }
  }
}
