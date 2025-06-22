//! Type wrappers for parsing content that is within delimiters, such as
//! parenthesis or brackets.

/// Type wrapper for parsing content within parenthesis.
///
/// # Example
/// Parse a [syn::Ident] within parenthesis:
/// ```
/// type ParenthesizedIdent = wm_macros::Parenthesized<syn::Ident>;
/// fn example(stream: syn::parse::ParseStream) -> syn::Result<ParenthesizedIdent> {
///   stream.parse::<ParenthesizedIdent>()
/// }
///
/// fn main() {
///   # use quote::quote;
///   let tokens = quote! { (some_name) }.into();
///
///   assert!(example(tokens).is_ok());
/// }
/// ```
pub struct Parenthesized<T>(pub T)
where
  T: syn::parse::Parse;

impl<T> syn::parse::Parse for Parenthesized<T>
where
  T: syn::parse::Parse,
{
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let content;
    syn::parenthesized!(content in input);
    Ok(Parenthesized(content.parse()?))
  }
}

impl<T> core::ops::Deref for Parenthesized<T>
where
  T: syn::parse::Parse,
{
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T> core::ops::DerefMut for Parenthesized<T>
where
  T: syn::parse::Parse,
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}
