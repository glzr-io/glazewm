/// A String with an associated source code span.
/// An owned version of a syn::LitStr.
#[derive(Debug, Clone)]
pub struct SpannedString {
  pub value: String,
  pub span: proc_macro2::Span,
}

impl SpannedString {
  pub fn new(value: String, span: proc_macro2::Span) -> Self {
    SpannedString { value, span }
  }

  pub fn from_lit_str(lit_str: syn::LitStr) -> Self {
    SpannedString {
      value: lit_str.value(),
      span: lit_str.span(),
    }
  }
}

impl From<SpannedString> for String {
  fn from(spanned_string: SpannedString) -> Self {
    spanned_string.value
  }
}

impl From<&SpannedString> for syn::LitStr {
  fn from(spanned_string: &SpannedString) -> Self {
    syn::LitStr::new(&spanned_string.value, spanned_string.span)
  }
}

impl From<syn::LitStr> for SpannedString {
  fn from(lit_str: syn::LitStr) -> Self {
    SpannedString::from_lit_str(lit_str)
  }
}

/// Parse a `SpannedString` from a `syn::LitStr`.
impl syn::parse::Parse for SpannedString {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let lit_str: syn::LitStr = input.parse()?;
    Ok(lit_str.into())
  }
}

/// Convert a `SpannedString` to a `syn::LitStr` for token generation.
impl quote::ToTokens for SpannedString {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    let lit_str: syn::LitStr = self.into();
    lit_str.to_tokens(tokens);
  }
}
