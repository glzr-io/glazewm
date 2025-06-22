//! Types and traits for branching and combining parsing operations.
//! Implements functionality similar to `nom`s alternatives and
//! combinators, but for `syn::parse::ParseStream`.

use crate::prelude::*;

/// Trait for tuples where all items can be parsed from a
/// parse stream.
pub trait ParseableTuple
where
  Self: Sized,
{
  type FirstItem: syn::parse::Parse;

  /// Parses all items in the tuple `T` from the stream in the order they
  /// appear in the tuple, and parses `Sep` in between each item. Returns
  /// all parsed items in a tuple, or the first error to occur.
  ///
  /// Do not use this directly, use the [Ordered] type instead,
  fn parse_tuple<Sep>(
    stream: syn::parse::ParseStream,
  ) -> syn::Result<Self>
  where
    Sep: syn::parse::Parse;
}

/// Trait for tuples where all items can be peeked and parsed from a parse
/// stream.
pub trait PeekableTuple
where
  Self: Sized,
{
  /// Iterates until all items in the tuple `T` have been parsed, or an
  /// error occurs. Parsing is attempted in the order of the items in
  /// the tuple, although if an item is not found, it may be skipped and
  /// reattempted for the next item(s).
  ///
  /// Do not use this directly, use the [Unordered] type instead.
  fn peek_parse_tuple<Sep>(
    stream: syn::parse::ParseStream,
  ) -> syn::Result<Self>
  where
    Sep: syn::parse::Parse + crate::common::peekable::Peekable;
}

macro_rules! replace_expr {
  ($idc:expr, $sub:expr) => {
    $sub
  };
}

macro_rules! get_first_item {
  ($first:tt, $($types:tt),+) => {
    $first
  };
}

macro_rules! impl_for_tuple {
  ($($types:tt),+ | $($numbers:tt),+) => {
    // Generic ensures that all types in the tuple implement `syn::parse::Parse`.
    impl<$($types),+> ParseableTuple for ($($types,)+) where $($types : syn::parse::Parse),+ {
      type FirstItem = get_first_item!($($types),+);

      fn parse_tuple<Sep>(stream: syn::parse::ParseStream) -> syn::Result<Self> where Sep: syn::parse::Parse {
        $(
          // Also check if the stream is empty to allow for missing trailing Optional items
          if $numbers != 0 && !stream.is_empty() {
            stream.parse::<Sep>()?;
          }

          #[allow(non_snake_case)]
          let $types = stream.parse::<$types>()?;
        )+

        // Pack the parsed items into a tuple
        Ok(($($types),+))
      }
    }

    // Generic ensures that all types in the tuple implement `syn::parse::Parse` and `Peekable`.
    impl<$($types),+> PeekableTuple for ($($types,)+) where $($types : syn::parse::Parse + crate::common::peekable::Peekable),+{

      fn peek_parse_tuple<Sep>(stream: syn::parse::ParseStream) -> syn::Result<Self>
        where Sep: syn::parse::Parse + crate::common::peekable::Peekable
      {
        use crate::common::peekable::prelude::*;

        // Creates a tuple with the same number of items as the tuple, but with each item being
        // `None`.
        let mut output: ($(Option<$types>,)+) = ($(replace_expr!($types, None),)+);

        // Iterate while any of the items in the tuple are `None`.
        while $(output.$numbers.is_none())||+ {
          let lookahead = stream.lookahead1();

          $(
            if output.$numbers.is_none() && lookahead.tpeek::<$types>() {

              output.$numbers = Some(stream.parse::<$types>()?);
            }
            // Insert an else before the next item in the tuple, to create `else if` on subsequent
            // unpacking
            )else+
            else {
              return Err(lookahead.error());
            }

          // If we havn't yet parsed all items in the tuple, parse the separator.
          // Also check if the stream is empty, which allows missing Optional items to be skipped
          // TODO: Better handling of [Optional] items, so that we can handle them without needing
          // the stream to end.
          if !stream.is_empty() && $(output.$numbers.is_none())||+ {
            stream.parse::<Sep>()?;
          }
        }

        Ok(($(
        // Saftey, the output is guaranteed to have all items otherwise the loop would have errored
        // out.
          output.$numbers.unwrap(),
          )+))
      }
    }
  };
}

// Implement the `ParseableTuple` and `PeekableTuple` traits for tuples of
// different sizes.
impl_for_tuple!(T, U | 0, 1);
impl_for_tuple!(T, U, V | 0, 1, 2);
impl_for_tuple!(T, U, V, W | 0, 1, 2, 3);
impl_for_tuple!(T, U, V, W, X | 0, 1, 2, 3, 4);
impl_for_tuple!(T, U, V, W, X, Y | 0, 1, 2, 3, 4, 5);

/// Type wrapper to parse all items in tuple `T` in order, using
/// `Sep` as the separator between items.
///
/// # Example
/// Parse [syn::Ident] and [syn::LitStr] from the stream, which are
/// separated by a comma. E.g. `some_name, "some string"`. If the order is
/// reversed, it will fail to parse.
/// ```
/// fn example(stream: syn::parse::ParseStream) -> syn::Result<()> {
///   type T = (syn::Ident, syn::LitStr);
///
///   stream.parse::<Ordered<T, syn::Token![,]>>()?;
/// }
///
/// fn main() {
///   # use quote::quote;
///   let tokens = quote! { some_name, "some string" }.into();
///   let error = quote! { "some string", some_name }.into();
///
///   assert!(example(tokens).is_ok());
///   assert!(example(error).is_err());
/// }
/// ```
pub struct Ordered<T, Sep>
where
  T: ParseableTuple,
  Sep: syn::parse::Parse,
{
  pub items: T,
  sep: std::marker::PhantomData<Sep>,
}

impl<T, Sep> syn::parse::Parse for Ordered<T, Sep>
where
  T: ParseableTuple,
  Sep: syn::parse::Parse,
{
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let output = T::parse_tuple::<Sep>(input)? as T;
    Ok(Self {
      items: output,
      sep: std::marker::PhantomData,
    })
  }
}

/// Implement [Peekable] for [Ordered] if the first item in the
/// tuple is peekable, by forwarding the implementation to the first item.
impl<T, FirstItem, Sep> crate::common::peekable::Peekable
  for Ordered<T, Sep>
where
  T: ParseableTuple<FirstItem = FirstItem>,
  Sep: syn::parse::Parse,
  FirstItem: crate::common::peekable::Peekable,
{
  fn display() -> &'static str {
    FirstItem::display()
  }

  fn peek<S>(stream: S) -> bool
  where
    S: crate::common::peekable::PeekableStream,
  {
    FirstItem::peek(stream)
  }
}

impl<T, Sep> core::ops::Deref for Ordered<T, Sep>
where
  T: ParseableTuple,
  Sep: syn::parse::Parse,
{
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.items
  }
}

/// Type wrapper to parse all items in tuple `T` in any order, using `Sep`
/// as the separator between items.
///
/// # Example
/// Parse [syn::Ident] and [syn::LitStr] from the stream in any order,
/// which are separated by a comma. E.g. `some_name, "some string"` or
/// `"some string", some_name`.
/// ```
/// fn example(stream: proc_macro::TokenStream) -> syn::Result<(syn::Ident, syn::LitStr)> {
///   type T = (syn::Ident, syn::LitStr);
///
///   stream.parse2::<Unordered<T, syn::Token![,]>>().map(|Unordered(t, _)| t)
/// }
///
/// fn main() {
///   # use quote::quote;
///   let normal = quote! { some_name, "some string" }.into();
///   let reversed = quote! { "some string", some_name }.into();
///   let error = quote! {some_name, other_name}.into();
///
///   assert!(example(normal).is_ok());
///   assert!(example(reversed).is_ok());
///   assert!(example(error).is_err());
/// }
/// ```
pub struct Unordered<T, Sep>
where
  T: PeekableTuple,
  Sep: syn::parse::Parse + crate::common::peekable::Peekable,
{
  pub items: T,
  sep: std::marker::PhantomData<Sep>,
}

impl<T, Sep> syn::parse::Parse for Unordered<T, Sep>
where
  T: PeekableTuple,
  Sep: syn::parse::Parse + crate::common::peekable::Peekable,
{
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let output = T::peek_parse_tuple::<Sep>(input)? as T;
    Ok(Self {
      items: output,
      sep: std::marker::PhantomData,
    })
  }
}

impl<T, Sep> core::ops::Deref for Unordered<T, Sep>
where
  T: PeekableTuple,
  Sep: syn::parse::Parse + crate::common::peekable::Peekable,
{
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.items
  }
}

/// Type wrapper to parse `If` if it is present, otherwise parse `Else`.
/// `If` must be peekable so it can be checked if it is present before
/// parsing.
///
/// # Example
/// Parse [syn::Ident] if it is present, otherwise parse [syn::LitStr].
/// ```
/// type IfElseType = IfElse<syn::Ident, syn::LitStr>;
///
/// fn example(stream: syn::parse::ParseStream) -> syn::Result<IfElseType> {
///   stream.parse::<IfElseType>()
/// }
///
/// fn main() {
///   # use quote::quote;
///   let if_tokens = quote! { some_name }.into();
///   let else_tokens = quote! { "some string" }.into();
///
///   assert!(example(if_tokens).is_ok_and(|if_else| if_else.is_if()));
///   assert!(example(else_tokens).is_ok_and(|if_else| if_else.is_else()));
/// }
pub enum IfElse<If, Else>
where
  If: syn::parse::Parse + crate::common::peekable::Peekable,
  Else: syn::parse::Parse,
{
  If(If),
  Else(Else),
}

// Methods are used in doc tests, but linter doesn't pick that up.
#[allow(dead_code)]
impl<If, Else> IfElse<If, Else>
where
  If: syn::parse::Parse + crate::common::peekable::Peekable,
  Else: syn::parse::Parse,
{
  pub fn is_if(&self) -> bool {
    matches!(self, Self::If(_))
  }

  pub fn is_else(&self) -> bool {
    matches!(self, Self::Else(_))
  }
}

impl<If, Else> syn::parse::Parse for IfElse<If, Else>
where
  If: syn::parse::Parse + crate::common::peekable::Peekable,
  Else: syn::parse::Parse,
{
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    if input.tpeek::<If>() {
      Ok(Self::If(input.parse()?))
    } else {
      Ok(Self::Else(input.parse()?))
    }
  }
}

/// Type wrapper to parse `T` only if it is present, otherwise returns
/// None.
///
/// To be used in combination with [Ordered] then all optional items must
/// be last in the tuple, and the stream must end to indicate that the
/// optional items will not be present.
///
/// To be used in combination with [Unordered] the stream must end to
/// indicate that the [Optional]s will not be present.
///
/// # Example
/// Parse [syn::Ident] if it is present, otherwise return None.
/// ```
/// type OptionalType = Optional<syn::Ident>;
/// fn example(stream: syn::parse::ParseStream) -> syn::Result<OptionalType> {
///   stream.parse::<OptionalType>()
/// }
///
/// fn main() {
///   # use quote::quote;
///   let present = quote! { some_name }.into();
///   let not_present = quote! {}.into();
///   let other_present = quote! { "some_string" }.into();
///
///   assert!(example(present).is_ok_and(|opt| opt.is_some()));
///   assert!(example(not_present).is_ok_and(|opt| opt.is_none()));
///   assert!(example(other_present).is_ok_and(|opt| opt.is_none()));
/// }
/// ```
/// Used in combination with [Ordered] to parse a [syn::Ident] and
/// optionally a [syn::LitStr]:
/// ```
/// type OrderedOptionalType = Ordered<(syn::Ident, Optional<syn::LitStr>),
/// syn::Token![,]>;
///
/// fn example(stream: syn::parse::ParseStream) ->
/// syn::Result<OrderedOptionalType> {
///   stream.parse::<OrderedOptionalType>()
/// }
///
/// fn main() {
///   let present = quote! { some_name, "some string" }.into();
///   let missing = quote! { some_name }.into();
///   // Will error since the stream did not end after the Ordered to indicate no optionals.
///   let error = quote! { some_name, not_a_string }.into();
///
///   assert!(example(present).is_ok_and(|Ordered { items: (ident, string), ..}| string.is_some()));
///   assert!(example(missing).is_ok_and(|Ordered { items: (ident, string), ..}| string.is_none()));
///   assert!(example(error).is_err());
/// }
/// ```
/// Used in combination with [Unordered] it can be used to parse a
/// [syn::Ident] and optionally a [syn::LitStr] in any order:
/// ```
/// type UnorderedOptionalType = Unordered<(syn::Ident, Optional<syn::LitStr>), syn::Token![,]>;
///
/// fn example(stream: syn::parse::ParseStream) -> syn::Result<UnorderedOptionalType> {
///   stream.parse::<UnorderedOptionalType>()
/// }
///
/// fn main() {
///   let present = quote! { some_name, "some string" }.into();
///   let not_present = quote! { some_name }.into();
///   let backwards = quote! { "some string", some_name }.into();
///   // Will error since the stream did not end after the Unordered to indicate no optionals.
///   let error = quote! { some_name, not_a_string }.into();
///
///   assert!(example(present).is_ok_and(|Unordered { items: (ident, string), ..}| string.is_some()));
///   assert!(example(not_present).is_ok_and(|Unordered { items: (ident, string), ..}| string.is_none()));
///   assert!(example(backwards).is_ok_and(|Unordered { items: (ident, string), ..}| string.is_some()));
///   assert!(example(error).is_err());
/// }
/// ```
pub enum Optional<T>
where
  T: syn::parse::Parse + crate::common::peekable::Peekable,
{
  Some(T),
  None,
}

impl<T> syn::parse::Parse for Optional<T>
where
  T: syn::parse::Parse + crate::common::peekable::Peekable,
{
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    if input.is_empty() {
      return Ok(Self::None);
    }

    if input.tpeek::<T>() {
      Ok(Self::Some(input.parse()?))
    } else {
      Ok(Self::None)
    }
  }
}

impl<T> crate::common::peekable::Peekable for Optional<T>
where
  T: syn::parse::Parse + crate::common::peekable::Peekable,
{
  fn display() -> &'static str {
    T::display()
  }

  fn peek<S>(stream: S) -> bool
  where
    S: crate::common::peekable::PeekableStream,
  {
    if stream.is_empty() {
      return true;
    }

    T::peek(stream)
  }
}

// Methods are used in doc tests, but linter doesn't pick that up.
#[allow(dead_code)]
impl<T> Optional<T>
where
  T: syn::parse::Parse + crate::common::peekable::Peekable,
{
  pub fn is_some(&self) -> bool {
    matches!(self, Self::Some(_))
  }

  pub fn is_none(&self) -> bool {
    matches!(self, Self::None)
  }

  #[allow(clippy::wrong_self_convention)]
  pub fn to_opt(self) -> Option<T> {
    match self {
      Self::Some(value) => Some(value),
      Self::None => None,
    }
  }
}
