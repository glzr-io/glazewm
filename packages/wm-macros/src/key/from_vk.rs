use proc_macro2::TokenStream;
use quote::quote;

use super::Key;

pub fn make_from_vk_impl(keys: &[Key]) -> TokenStream {
  let lines = keys.iter().map(|key| {
    let ident = &key.ident;
    if ident == "Custom" {
      return quote! {};
    }

    let vk_value = &key.vk_value;

    quote! {
      ::windows::Win32::UI::Input::KeyboardAndMouse::#vk_value => Self::#ident
    }
  });

  quote! {
    pub fn from_vk(vk: u16) -> Self {
      match ::windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(vk) {
        #(#lines),*
        _ => {
          Self::Custom(vk)
        }
      }
    }
  }
}
