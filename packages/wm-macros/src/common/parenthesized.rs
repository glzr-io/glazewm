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

#[allow(dead_code)]
impl<T> Parenthesized<T>
where
  T: syn::parse::Parse,
{
  pub fn inner(&self) -> &T {
    &self.0
  }

  pub fn into_inner(self) -> T {
    self.0
  }
}
