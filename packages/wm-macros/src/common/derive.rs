//! Utlities for working with [syn::DeriveInput] and other related types.

use syn::DataEnum;

pub mod prelude {
  pub use super::RequireEnum;
}

pub trait RequireEnum {
  /// Return the enum data, or an error if the data is not an enum.
  fn require_enum(&self) -> syn::Result<&DataEnum>;
}

impl RequireEnum for syn::Data {
  fn require_enum(&self) -> syn::Result<&DataEnum> {
    match self {
      syn::Data::Enum(data) => Ok(data),
      _ => Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "This macro can only be used on enums",
      )),
    }
  }
}
