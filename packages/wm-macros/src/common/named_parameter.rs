//! Types for parsing `name = value` patterns.

/// Type to represent a `name = value` pair, where `name` can be peeked and
/// both `name` and `value` can be parsed.
///
/// # Example
/// Parse a name-value pair of [syn::Ident] and [syn::LitStr].
/// ```
/// type NamedParam = NamedParameter<syn::Ident, syn::LitStr>;
///
/// fn example(stream: syn::parse::ParseStream) -> syn::Result<NamedParam> {
///   stream.parse::<NamedParam>()
/// }
///
/// fn main() {
///   # use quote::quote;
///   let tokens = quote! { some_name = "some_value" }.into();
///
///   assert!(example(tokens).is_ok());
/// }
/// ```
// Will be used in future.
#[allow(dead_code)]
pub struct NamedParameter<Name, Param>
where
  Name: syn::parse::Parse + crate::common::peekable::Peekable,
  Param: syn::parse::Parse,
{
  pub name: Name,
  pub param: Param,
}

impl<Name, Param> syn::parse::Parse for NamedParameter<Name, Param>
where
  Name: syn::parse::Parse + crate::common::peekable::Peekable,
  Param: syn::parse::Parse,
{
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let name = input.parse()?;
    input.parse::<syn::Token![=]>()?;
    let param = input.parse()?;
    Ok(Self { name, param })
  }
}

impl<Name, Param> crate::common::peekable::Peekable
  for NamedParameter<Name, Param>
where
  Name: syn::parse::Parse + crate::common::peekable::Peekable,
  Param: syn::parse::Parse,
{
  fn peek<S>(stream: S) -> bool
  where
    S: crate::common::peekable::PeekableStream,
  {
    Name::peek(stream)
  }

  fn display() -> &'static str {
    Name::display()
  }
}
