// Enable proc macro diagnostics to allow emitting warnings and errors in
// line
#![feature(proc_macro_diagnostic)]

mod common;
mod enum_from_inner;
mod key;
mod subenum;
use proc_macro::TokenStream;

mod prelude {
  pub use crate::common::{
    attributes::prelude::*, derive::prelude::*,
    error_handling::prelude::*, peekable::prelude::*,
  };
}

enum Os {
  Windows,
  MacOS,
}

// Proc macro functions *must* live in lib.rs, not in a submodule.
// Calls an equivilant function in the `key` module to get around it.
//                 (Macro Name    , attributes(attribute, names))
#[proc_macro_derive(KeyConversions, attributes(key))]
pub fn key_conversions(input: TokenStream) -> TokenStream {
  key::key_conversions(input)
}

#[proc_macro_derive(SubEnum, attributes(subenum))]
pub fn sub_enum(input: TokenStream) -> TokenStream {
  subenum::sub_enum(input)
}

#[proc_macro_derive(EnumFromInner)]
pub fn enum_from_inner(input: TokenStream) -> TokenStream {
  enum_from_inner::enum_from_inner(input)
}
