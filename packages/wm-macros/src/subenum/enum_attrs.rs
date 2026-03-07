use crate::{common::branch::Ordered, prelude::*};

mod kw {
  crate::common::custom_keyword!(defaults);
}

/// Collects all `#[subenum(...)]` attributes from the given iterator of
/// attributes and returns a list of `Subenum` instances.
pub fn collect_sub_enums<'a>(
  attrs: impl Iterator<Item = &'a syn::MetaList>,
) -> syn::Result<Vec<Subenum>> {
  attrs
    .map(|attr| attr.parse_args())
    .collect::<syn::Result<Vec<_>>>()
}

/// Each subenum can either be a declaration with a name and attribute
/// block, or a defaults block to append to every subenum.
pub enum Subenum {
  Defaults(proc_macro2::TokenStream),
  Declaration(SubenumDeclaration),
}

impl syn::parse::Parse for Subenum {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    type Defaults = Ordered<(kw::defaults, AttrBlock), syn::Token![,]>;
    if input.tpeek::<Defaults>() {
      let Ordered {
        items: (_, AttrBlock { attrs }),
        ..
      } = input.parse::<Defaults>()?;
      return Ok(Self::Defaults(attrs));
    }

    let declaration: SubenumDeclaration = input.parse()?;
    Ok(Self::Declaration(declaration))
  }
}

/// Parser for `<name>, {...}` where name is the name of the subenum, and
/// the block contents are passed through as is.
pub struct SubenumDeclaration {
  pub name: syn::Ident,
  pub attrs: proc_macro2::TokenStream,
}

impl syn::parse::Parse for SubenumDeclaration {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    type Declaration = Ordered<(syn::Ident, AttrBlock), syn::Token![,]>;

    let Ordered {
      items: (name, AttrBlock { attrs }),
      ..
    } = input.parse::<Declaration>()?;
    Ok(Self { name, attrs })
  }
}

/// Block of arbitray tokens that are contained within { ... }
struct AttrBlock {
  pub attrs: proc_macro2::TokenStream,
}

impl syn::parse::Parse for AttrBlock {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let content;
    let _ = syn::braced!(content in input);
    let attrs: proc_macro2::TokenStream = content.parse()?;
    Ok(Self { attrs })
  }
}
