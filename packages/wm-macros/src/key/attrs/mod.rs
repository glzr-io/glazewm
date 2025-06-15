/// Contains parsing logic for the `#[key(...)]` attribute on the enum
/// itself
pub mod enums;
/// Contains parsing logic for the `#[key(...)]` attribute on enum variants
pub mod variant;

/// Finds the `#[key(...)]` attribute in the list of attributes.
/// Ignores all other attributes as they may be for other macros
pub fn find_key_attr(attrs: &[syn::Attribute]) -> Option<&syn::MetaList> {
  attrs
    .iter()
    // This will filter out any attributes that are not lists
    .filter_map(|attr| attr.meta.require_list().ok())
    // Find our `key` attribute
    .find(|list| {
      if let Ok(ident) = list.path.require_ident() {
        *ident == "key"
      } else {
        false
      }
    })
}
