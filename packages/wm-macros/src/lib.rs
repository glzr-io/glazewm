mod common;
mod key;
use proc_macro::TokenStream;

enum Os {
  Windows,
  MacOS,
}

enum Either<L, R> {
  Left(L),
  Right(R),
}

// Proc macro functions *must* live in lib.rs, not in a submodule.
// Calls an equivilant function in the `key` module to get around it.
//                 (Macro Name    , attributes(attribute, names))
#[proc_macro_derive(KeyConversions, attributes(key))]
pub fn key_conversions(input: TokenStream) -> TokenStream {
  key::key_conversions(input)
}
