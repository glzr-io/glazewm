mod key;
use proc_macro::TokenStream;

#[proc_macro_derive(KeyConversions, attributes(key))]
pub fn key_conversions(input: TokenStream) -> TokenStream {
  key::key_conversions(input)
}
