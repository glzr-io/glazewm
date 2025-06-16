use crate::Either;

// Can't implement for slices, as they are not sized and FnOnce cannot be
// in a slice ref (Can't move out of them, which means they can't be
// called).

// Marker trait for an iterable where all items are parser functions that
// also take a lookahead
pub trait ParseList<'a, T: 'a, F>: IntoIterator<Item = F>
where
  F: FnOnce(
    syn::parse::ParseStream<'_>,
    &syn::parse::Lookahead1<'_>,
  ) -> Option<syn::Result<T>>,
{
}

impl<'a, T, F, const N: usize> ParseList<'a, T, F> for [F; N]
where
  T: syn::parse::Parse + crate::common::peekable::Peekable + 'a,
  F: FnOnce(
    syn::parse::ParseStream<'_>,
    &syn::parse::Lookahead1<'_>,
  ) -> Option<syn::Result<T>>,
{
}

// Can probably remove this now, but keeping it to reference for now.
#[allow(dead_code)]
pub trait Alt {
  /// Parses either a `L` or `R` type from the stream, returning an
  /// `Either<L, R>`. Left will be attempted first, then right.
  /// Forwards parsing errors, and returns an error if neither type can be
  /// parsed.
  ///
  /// # Example
  /// Parse either an identifier or a string literal from the stream. This
  /// will error if any type other than `syn::Ident` or `syn::LitStr` is
  /// encountered. ```
  /// # fn example(stream: syn::parse::ParseStream) -> syn::Result<()> {
  /// match stream.alt::<syn::Ident, syn::LitStr>()? {
  ///   Either::Left(ident) => { // ... },
  ///   Either::Right(lit_str) => { // ... },
  /// }
  /// # }
  fn alt<L, R>(&self) -> syn::Result<Either<L, R>>
  where
    L: syn::parse::Parse + crate::common::peekable::Peekable,
    R: syn::parse::Parse + crate::common::peekable::Peekable;

  /// Parses either a `L` or `R` type from the stream, returning an
  /// `Either<L, R>`. Left will be attempted first, then right.
  /// A side will only be attempted if the corresponding boolean is
  /// `true`.
  ///
  /// # Example
  /// Parse an identifier or a string literal from the stream in any order.
  /// This will error if any type other than `syn::Ident` or `syn::LitStr`
  /// is encountered, or if there are more than one of each type in the
  /// stream.
  /// ```
  /// # fn example(stream: syn::parse::ParseStream) -> syn::Result<()> {
  /// let ident = None;
  /// let lit_str = None;
  ///
  /// while !stream.is_empty() {
  ///   match stream.alt_if::<syn::Ident, syn::LitStr>(ident.is_none(), lit_str.is_none())? {
  ///     Either::Left(i) => { ident = Some(i); },
  ///     Either::Right(s) => { lit_str = Some(s); },
  ///   }
  /// }
  /// # }
  /// ```
  fn alt_if<L, R>(
    &self,
    left: bool,
    right: bool,
  ) -> syn::Result<Either<L, R>>
  where
    L: syn::parse::Parse + crate::common::peekable::Peekable,
    R: syn::parse::Parse + crate::common::peekable::Peekable;

  /// Parses T from the stream, trying each parser in the list until one
  /// succeeds. Works like `alt_for`, but also passes the lookahead for
  /// better error handling.
  /// If all parsers fail, returns the lookahead error.
  fn alt_for<'a, T, L, F>(&self, list: L) -> syn::Result<T>
  where
    T: 'a,
    L: ParseList<'a, T, F>,
    F: FnOnce(
        syn::parse::ParseStream<'_>,
        &syn::parse::Lookahead1<'_>,
      ) -> Option<syn::Result<T>>
      + 'a;
}

impl Alt for syn::parse::ParseStream<'_> {
  fn alt<L, R>(&self) -> syn::Result<Either<L, R>>
  where
    L: syn::parse::Parse + crate::common::peekable::Peekable,
    R: syn::parse::Parse + crate::common::peekable::Peekable,
  {
    let lookahead = self.lookahead1();

    if lookahead.peek(L::peekable()) {
      Ok(Either::Left(self.parse::<L>()?))
    } else if lookahead.peek(R::peekable()) {
      Ok(Either::Right(self.parse::<R>()?))
    } else {
      Err(lookahead.error())
    }
  }

  fn alt_if<L, R>(
    &self,
    left: bool,
    right: bool,
  ) -> syn::Result<Either<L, R>>
  where
    L: syn::parse::Parse + crate::common::peekable::Peekable,
    R: syn::parse::Parse + crate::common::peekable::Peekable,
  {
    let lookahead = self.lookahead1();

    if left && lookahead.peek(L::peekable()) {
      Ok(Either::Left(self.parse::<L>()?))
    } else if right && lookahead.peek(R::peekable()) {
      Ok(Either::Right(self.parse::<R>()?))
    } else {
      Err(lookahead.error())
    }
  }

  fn alt_for<'a, T, L, F>(&self, list: L) -> syn::Result<T>
  where
    T: 'a,
    L: ParseList<'a, T, F>,
    F: FnOnce(
        syn::parse::ParseStream<'_>,
        &syn::parse::Lookahead1<'_>,
      ) -> Option<syn::Result<T>>
      + 'a,
  {
    let lookahead = self.lookahead1();

    for parser in list {
      if let Some(res) = parser(self, &lookahead) {
        return res;
      }
    }

    Err(lookahead.error())
  }
}

/// Trait for tuples where all items can be parsed from a
/// parse stream.
pub trait ParseableTuple {
  /// Output type of the tuple parsing. Should probably be the same type as
  /// the tuple itself.
  type Output;

  /// Parses all items in the tuple `T` from the stream in the order they
  /// appear in the tuple, and parses `Sep` in between each item. Returns
  /// all parsed items in a tuple, or the first error to occur.
  fn parse_tuple<Sep>(
    stream: syn::parse::ParseStream,
  ) -> syn::Result<Self::Output>
  where
    Sep: syn::parse::Parse;
}

/// Trait for tuples where all items can be peeked and parsed from a parse
/// stream.
pub trait PeekableTuple {
  /// Output type of the tuple parsing. Should probably be the same type as
  /// the tuple itself.
  type Output;

  /// Iterates until all items in the tuple `T` have been parsed, or an
  /// error occurs. Parsing is attempted in the order of the items in
  /// the tuple, although if an item is not found, it may be skipped and
  /// reattempted for the next item(s).
  fn peek_parse_tuple<Sep>(
    stream: syn::parse::ParseStream,
  ) -> syn::Result<Self::Output>
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
      // Set the output type to be the same as the tuple itself.
      type Output = ($($types,)+);

      fn parse_tuple<Sep>(stream: syn::parse::ParseStream) -> syn::Result<Self::Output> where Sep: syn::parse::Parse {
        // Return the output tuple with all items parsed from the stream.
        // Parsing happens in a block to allow the separator to be parsed inside of the tuple
        // constructor
        Ok((
            // Craete the block for each type in the tuple
          $(
            {
              // Parse the type from the stream
              let t = stream.parse::<$types>()?;
              // Parse the separator after the type
              stream.parse::<Sep>()?;
              // Return the parsed type to be included in the output tuple
              t
            },
            )+
        ))
      }
    }

    // Generic ensures that all types in the tuple implement `syn::parse::Parse` and `Peekable`.
    impl<$($types),+> PeekableTuple for ($($types,)+) where $($types : syn::parse::Parse + crate::common::peekable::Peekable),+ {
      // Set the output type to be the same as the tuple itself.
      type Output = ($($types,)+);

      fn peek_parse_tuple<Sep>(stream: syn::parse::ParseStream) -> syn::Result<Self::Output>
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
            if lookahead.peek($types::peekable()) {
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
impl_for_tuple!(T | 0);
impl_for_tuple!(T, U | 0, 1);
impl_for_tuple!(T, U, V | 0, 1, 2);
impl_for_tuple!(T, U, V, W | 0, 1, 2, 3);
impl_for_tuple!(T, U, V, W, X | 0, 1, 2, 3, 4);
impl_for_tuple!(T, U, V, W, X, Y | 0, 1, 2, 3, 4, 5);

/// Trait for parsing multiple items from a parse stream.
#[allow(dead_code)]
pub trait Combined {
  /// Parses all items in the tuple `T` from the stream in the order they
  /// are in the tuple, each item separated by `Sep`. Returns all parsed
  /// items in a tuple, or the first error to occur.
  fn parse_all<T, Sep>(&self) -> syn::Result<T::Output>
  where
    T: ParseableTuple,
    Sep: syn::parse::Parse;

  /// Parses all items in the tuple `T` from the stream in any order, each
  /// item separated by `Sep`. Returns all parsed items in a tuple, or the
  /// first error to occur.
  ///
  /// Parsing is attempted in the order of the items in the tuple, although
  /// if an item is not found, it may be skipped and reattempted for the
  /// next item(s).
  ///
  /// If none of the items are successfully peeked, an error is returned.
  ///
  /// If any item is not found, an error will be returned for the first
  /// missing item in the tuple.
  fn parse_all_unordered<T, Sep>(&self) -> syn::Result<T::Output>
  where
    T: PeekableTuple,
    Sep: syn::parse::Parse + crate::common::peekable::Peekable;
}

impl Combined for syn::parse::ParseStream<'_> {
  fn parse_all<T, Sep>(&self) -> syn::Result<T::Output>
  where
    T: ParseableTuple,
    Sep: syn::parse::Parse,
  {
    let output = T::parse_tuple::<Sep>(self)?;

    Ok(output)
  }

  fn parse_all_unordered<T, Sep>(&self) -> syn::Result<T::Output>
  where
    T: PeekableTuple,
    Sep: syn::parse::Parse + crate::common::peekable::Peekable,
  {
    let output = T::peek_parse_tuple::<Sep>(self)?;

    Ok(output)
  }
}
