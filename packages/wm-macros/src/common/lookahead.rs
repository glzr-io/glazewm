use crate::prelude::*;

pub trait PeekThenAdvance {
  /// Checks if the next token in the stream is a T, and if so parses
  /// it. Returns `Some(<parse result>)` if the peek is successful, and
  /// `None` if the token does not match.
  ///
  /// See `LookaheadPeekThenAdvance` for a version that uses a lookahead
  /// instead of a stream.
  ///
  /// # Example
  /// ```
  /// # fn example(stream: syn::parse::ParseStream) {
  /// if let Some(value) = stream.peek_then_advance::<syn::Ident>() {
  ///   // ...
  /// }
  /// # }
  /// ```
  fn peek_then_advance<
    T: syn::parse::Parse + crate::common::peekable::Peekable,
  >(
    &self,
  ) -> Option<syn::Result<T>>;
}

impl PeekThenAdvance for syn::parse::ParseStream<'_> {
  fn peek_then_advance<
    T: syn::parse::Parse + crate::common::peekable::Peekable,
  >(
    &self,
  ) -> Option<syn::Result<T>> {
    if self.tpeek::<T>() {
      Some(self.parse::<T>())
    } else {
      None
    }
  }
}

#[allow(dead_code)]
pub trait LookaheadPeekThenAdvance {
  /// Checks if the next token in the lookahead is a T, and if so parses
  /// it. Returns `Some(<parse result>)` if the peek is successful, and
  /// `None` if the token does not match.
  ///
  /// # Example
  /// ```
  /// # fn example(stream: syn::parse::ParseStream) {
  /// # let lookahead = stream.lookahead1();
  /// if let Some(value) = lookahead.peek_then_advance::<syn::Ident>(stream) {
  ///   // ...
  /// }
  /// # }
  /// ```
  fn peek_then_advance<T>(
    &self,
    stream: syn::parse::ParseStream,
  ) -> Option<syn::Result<T>>
  where
    T: syn::parse::Parse + crate::common::peekable::Peekable;
}

impl LookaheadPeekThenAdvance for syn::parse::Lookahead1<'_> {
  fn peek_then_advance<
    T: syn::parse::Parse + crate::common::peekable::Peekable,
  >(
    &self,
    stream: syn::parse::ParseStream,
  ) -> Option<syn::Result<T>> {
    if self.tpeek::<T>() {
      Some(stream.parse::<T>())
    } else {
      None
    }
  }
}
