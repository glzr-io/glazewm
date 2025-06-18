use syn::DataEnum;

pub mod prelude {
  pub use super::RequireEnum;
}

pub trait RequireEnum {
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
