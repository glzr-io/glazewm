/// Trait for tuples where all items can be parsed from a
/// parse stream.
pub trait ParseableTuple
where
  Self: Sized,
{
  /// Parses all items in the tuple `T` from the stream in the order they
  /// appear in the tuple, and parses `Sep` in between each item. Returns
  /// all parsed items in a tuple, or the first error to occur.
  ///
  /// # Example
  /// Parses a `syn::Ident` and a `syn::LitStr`, which are seperated by a
  /// comma, from the stream. E.g. `some_name, "some string"`. If the
  /// order is reversed, it will fail to parse. If order is irrelevant,
  /// use `PeekableTuple` instead.
  /// ```
  /// # fn example(stream: syn::parse::ParseStream) -> syn::Result<()> {
  /// type T = (syn::Ident, syn::LitStr);
  /// T::parse_tuple::<syn::Token![,]>(stream)?;
  /// # }
  /// ```
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
  /// # Example
  /// Parses a `syn::Ident` and a `syn::LitStr`, which are seperated by a
  /// comma, from the stream in any order. Will successfully parse both:
  /// `some_name, "some string"` and `"some string", some_name`.
  /// ```
  /// # fn example(stream: syn::parse::ParseStream) -> syn::Result<()> {
  /// type T = (syn::Ident, syn::LitStr);
  /// let (ident, lit_str) = T::peek_parse_tuple::<syn::Token![,]>(stream)?;
  /// # }
  /// ```
  fn peek_parse_tuple<Sep>(
    stream: syn::parse::ParseStream,
  ) -> syn::Result<Self>
  where
    Sep: syn::parse::Parse + crate::common::peekable::Peekable;
}

// For token replacing, primarily for varaidic argument unpacking.
// Used to create the `None` value for each item in the tuple.
macro_rules! replace_expr {
  ($idc:expr, $sub:expr) => {
    $sub
  };
}

macro_rules! impl_for_tuple {
  // Might be a way to create the number list from the types, but too complex to be worth it.
  // Types are the generic type args, numbers are the indices for each type in the tuple.
  ($($types:tt),+ | $($numbers:tt),+) => {
    // Generic ensures that all types in the tuple implement `syn::parse::Parse`.
    impl<$($types),+> ParseableTuple for ($($types,)+) where $($types : syn::parse::Parse),+ {

      fn parse_tuple<Sep>(stream: syn::parse::ParseStream) -> syn::Result<Self> where Sep: syn::parse::Parse {
        // Return the output tuple with all items parsed from the stream.
        // Parsing happens in a block to allow the separator to be parsed inside of the tuple
        // constructor

        // Create a variable for the parsed version of each item in the tuple. Needs to be outside
        // of the tuple constructor to allow the seperator to be parsed before each item in the
        // correct order.
        $(
          // Parse the seperator before every item bar the first
          if $numbers != 0 {
            stream.parse::<Sep>()?;
          }
          // Parse the type from the stream
          #[allow(non_snake_case)]
          let $types = stream.parse::<$types>()?;
          // Parse the separator after the type
        )+

        // Pack the parsed items into a tuple
        Ok(($($types),+))
      }
    }

    // Generic ensures that all types in the tuple implement `syn::parse::Parse` and `Peekable`.
    impl<$($types),+> PeekableTuple for ($($types,)+) where $($types : syn::parse::Parse + crate::common::peekable::Peekable),+{
      // Set the output type to be the same as the tuple itself.

      fn peek_parse_tuple<Sep>(stream: syn::parse::ParseStream) -> syn::Result<Self>
        where Sep: syn::parse::Parse + crate::common::peekable::Peekable
      {
        // Creates a tuple with the same number of items as the tuple, but with each item being
        // `None`.
        let mut output: ($(Option<$types>,)+) = ($(replace_expr!($types, None),)+);

        // Iterate while all items in the tuple are `None`, meaning they have not been parsed yet.
        while $(output.$numbers.is_none())||+ {
          // Create a lookahaed for this iteration.
          let lookahead = stream.lookahead1();
          // For each type in the tuple, insert the following block.
          $(
            // Try to peek this tuple item.
            if output.$numbers.is_none() && lookahead.peek($types::peekable()) {
              // If so, parse it from the stream and set it in the output tuple.
              output.$numbers = Some(stream.parse::<$types>()?);
            }
            // Insert an else before the next item in the tuple, to create `else if` on subsequent
            // unpacking
            )else+
            else {
              // Else, if the item is not peeked, we will return an error.
              return Err(lookahead.error());
            }

          // If we havn't yet parsed all items in the tuple, parse the separator.
          if $(output.$numbers.is_none())||+ {
            stream.parse::<Sep>()?;
          }
        }

        // Return a tuple with all items parsed from the stream.
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
/// Parse `syn::Ident` and `syn::LitStr` from the stream, which are
/// separated by a comma. E.g. `some_name, "some string"`. If the order is
/// reversed, it will fail to parse.
/// ```
/// # fn example(stream: syn::parse::ParseStream) -> syn::Result<()> {
/// type T = (syn::Ident, syn::LitStr);
///
/// stream.parse::<Ordered<T, syn::Token![,]>>()?;
/// # }
/// ```
pub struct Ordered<T, Sep>(pub T, pub std::marker::PhantomData<Sep>)
where
  T: ParseableTuple,
  Sep: syn::parse::Parse;

impl<T, Sep> syn::parse::Parse for Ordered<T, Sep>
where
  T: ParseableTuple,
  Sep: syn::parse::Parse,
{
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let output = T::parse_tuple::<Sep>(input)? as T;
    Ok(Self(output, std::marker::PhantomData))
  }
}

impl<T, Sep> core::ops::Deref for Ordered<T, Sep>
where
  T: ParseableTuple,
  Sep: syn::parse::Parse,
{
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

/// Typpe wrapper to parse all items in tuple `T` in any order, using `Sep`
/// as the separator between items.
///
/// # Example
/// Parse `syn::Ident` and `syn::LitStr` from the stream in any order,
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
pub struct Unordered<T, Sep>(pub T, pub std::marker::PhantomData<Sep>)
where
  T: PeekableTuple,
  Sep: syn::parse::Parse + crate::common::peekable::Peekable;

impl<T, Sep> syn::parse::Parse for Unordered<T, Sep>
where
  T: PeekableTuple,
  Sep: syn::parse::Parse + crate::common::peekable::Peekable,
{
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let output = T::peek_parse_tuple::<Sep>(input)? as T;
    Ok(Self(output, std::marker::PhantomData))
  }
}

impl<T, Sep> core::ops::Deref for Unordered<T, Sep>
where
  T: PeekableTuple,
  Sep: syn::parse::Parse + crate::common::peekable::Peekable,
{
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

pub enum IfElse<If, Else>
where
  If: syn::parse::Parse + crate::common::peekable::Peekable,
  Else: syn::parse::Parse,
{
  If(If),
  Else(Else),
}

impl<If, Else> syn::parse::Parse for IfElse<If, Else>
where
  If: syn::parse::Parse + crate::common::peekable::Peekable,
  Else: syn::parse::Parse,
{
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    if input.peek(If::peekable()) {
      Ok(Self::If(input.parse()?))
    } else {
      Ok(Self::Else(input.parse()?))
    }
  }
}
