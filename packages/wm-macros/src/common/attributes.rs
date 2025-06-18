pub mod prelude {
  pub use super::FindAttributes;
}

#[allow(dead_code)]
pub trait FindAttributes {
  fn find_attrs(
    &self,
    name: &str,
  ) -> impl Iterator<Item = &syn::Attribute>;
  fn find_list_attrs(
    &self,
    name: &str,
  ) -> impl Iterator<Item = &syn::MetaList> {
    self
      .find_attrs(name)
      .filter_map(|attr| attr.meta.require_list().ok())
  }
  fn find_name_attrs(
    &self,
    name: &str,
  ) -> impl Iterator<Item = &syn::MetaNameValue> {
    self
      .find_attrs(name)
      .filter_map(|attr| attr.meta.require_name_value().ok())
  }

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
