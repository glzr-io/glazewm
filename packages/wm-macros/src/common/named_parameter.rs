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
  fn peekable() -> impl syn::parse::Peek {
    Name::peekable()
  }
}
