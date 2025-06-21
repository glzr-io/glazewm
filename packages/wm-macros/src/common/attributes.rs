//! Utilities for working with [syn::Attribute]

pub mod prelude {
  pub use super::FindAttributes;
}

#[allow(dead_code)]
/// Trait for filtering lists of [syn::Attribute] by name and type.
pub trait FindAttributes {
  /// Find attributes by name. E.g. `#[name]`, `#[name(...)]` or `#[name =
  /// <value>]`
  fn find_attrs(
    &self,
    name: &str,
  ) -> impl Iterator<Item = &syn::Attribute>;
  /// Find list attributes by name. E.g. `#[name(...)]`
  fn find_list_attrs(
    &self,
    name: &str,
  ) -> impl Iterator<Item = &syn::MetaList> {
    self
      .find_attrs(name)
      .filter_map(|attr| attr.meta.require_list().ok())
  }
  /// Find name-value attributes by name. E.g. `#[name = <value>]`
  fn find_name_attrs(
    &self,
    name: &str,
  ) -> impl Iterator<Item = &syn::MetaNameValue> {
    self
      .find_attrs(name)
      .filter_map(|attr| attr.meta.require_name_value().ok())
  }
  /// Find path attributes by name. E.g. `#[name]` or `#[name::path]`
  fn find_path_attrs(
    &self,
    name: &str,
  ) -> impl Iterator<Item = &syn::Path> {
    self
      .find_attrs(name)
      .filter_map(|attr| attr.meta.require_path_only().ok())
  }
}

impl FindAttributes for Vec<syn::Attribute> {
  fn find_attrs(
    &self,
    name: &str,
  ) -> impl Iterator<Item = &syn::Attribute> {
    self.iter().filter(move |attr| {
      attr.path().get_ident().is_some_and(|ident| ident == name)
    })
  }
}

impl FindAttributes for &[syn::Attribute] {
  fn find_attrs(
    &self,
    name: &str,
  ) -> impl Iterator<Item = &syn::Attribute> {
    self.iter().filter(move |attr| {
      attr.path().get_ident().is_some_and(|ident| ident == name)
    })
  }
}
