use syn::spanned::Spanned;

use super::{PathList, kw};
use crate::{
  common::{
    branch::{Optional, Ordered, Unordered},
    lookahead::PeekThenAdvance,
    parenthesized::Parenthesized,
  },
  prelude::*,
};

pub fn collect_sub_enums<'a>(
  attrs: impl Iterator<Item = &'a syn::MetaList>,
) -> syn::Result<Vec<Subenum>> {
  attrs
    .map(|attr| attr.parse_args())
    .collect::<syn::Result<Vec<_>>>()
}

pub enum Subenum {
  Defaults(proc_macro2::TokenStream),
  Declaration(SubenumDeclaration),
}

impl syn::parse::Parse for Subenum {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    type Defaults = Ordered<(kw::defaults, AttrBlock), syn::Token![,]>;
    if let Some(def) = input.peek_then_advance::<Defaults>() {
      let Ordered {
        items: (_, AttrBlock { attrs }),
        ..
      } = def?;
      return Ok(Self::Defaults(attrs));
    }

    let declaration: SubenumDeclaration = input.parse()?;
    Ok(Self::Declaration(declaration))
  }
}

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
