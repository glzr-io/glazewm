use crate::Either;

// Marker trait for an iterable where all items are parser functions
pub trait ParseList<'a, T: 'a, F>: IntoIterator<Item = F>
where
  F: FnOnce(syn::parse::ParseStream<'_>) -> syn::Result<T>,
{
}

// Implement for a genericly sized array of parser functions
impl<'a, T, F, const N: usize> ParseList<'a, T, F> for [F; N]
where
  T: syn::parse::Parse + crate::common::peekable::Peekable + 'a,
  F: FnOnce(syn::parse::ParseStream<'_>) -> syn::Result<T>,
{
}

// Implement for a Vec of parser functions
impl<'a, T, F> ParseList<'a, T, F> for Vec<F>
where
  T: syn::parse::Parse + crate::common::peekable::Peekable + 'a,
  F: FnOnce(syn::parse::ParseStream<'_>) -> syn::Result<T>,
{
}

// Can't implement for slices, as they are not sized and FnOnce cannot be
// in a slice ref (Can't move out of them, which means they can't be
// called).

// Marker trait for an iterable where all items are parser functions that
// also take a lookahead
pub trait ParsePeekList<'a, T: 'a, F>: IntoIterator<Item = F>
where
  F: FnOnce(
    syn::parse::ParseStream<'_>,
    &syn::parse::Lookahead1<'_>,
  ) -> syn::Result<T>,
{
}

impl<'a, T, F, const N: usize> ParsePeekList<'a, T, F> for [F; N]
where
  T: syn::parse::Parse + crate::common::peekable::Peekable + 'a,
  F: FnOnce(
    syn::parse::ParseStream<'_>,
    &syn::parse::Lookahead1<'_>,
  ) -> syn::Result<T>,
{
}

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
  /// succesds. If all parsers fail, returns an error with all errors
  /// combined.
  fn alt_for<'a, T, L, F>(&self, list: L) -> syn::Result<T>
  where
    T: 'a,
    L: ParseList<'a, T, F>,
    F: FnOnce(syn::parse::ParseStream<'_>) -> syn::Result<T> + 'a;

  /// Parses T from the stream, trying each parser in the list until one
  /// succeeds. Works like `alt_for`, but also passes the lookahead for
  /// better error handling.
  /// If all parsers fail, returns the lookahead error.
  fn alt_peek_for<'a, T, L, F>(&self, list: L) -> syn::Result<T>
  where
    T: 'a,
    L: ParsePeekList<'a, T, F>,
    F: FnOnce(
        syn::parse::ParseStream<'_>,
        &syn::parse::Lookahead1<'_>,
      ) -> syn::Result<T>
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
    F: FnOnce(syn::parse::ParseStream<'_>) -> syn::Result<T> + 'a,
  {
    let mut error = vec![];

    for parser in list {
      match parser(self) {
        Ok(value) => return Ok(value),
        Err(e) => {
          error.push(e);
        }
      }
    }

    let errors = error
      .into_iter()
      .reduce(|mut acc, el| {
        acc.combine(el);
        acc
      })
      .unwrap();

    Err(errors)
  }

  fn alt_peek_for<'a, T, L, F>(&self, list: L) -> syn::Result<T>
  where
    T: 'a,
    L: ParsePeekList<'a, T, F>,
    F: FnOnce(
        syn::parse::ParseStream<'_>,
        &syn::parse::Lookahead1<'_>,
      ) -> syn::Result<T>
      + 'a,
  {
    let lookahead = self.lookahead1();
    let mut error = vec![];

    for parser in list {
      match parser(self, &lookahead) {
        Ok(value) => return Ok(value),
        Err(e) => {
          error.push(e);
        }
      }
    }

    Err(lookahead.error())
  }
}
