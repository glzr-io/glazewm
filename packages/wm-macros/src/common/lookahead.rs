pub trait PeekThenAdvance {
  /// Checks if the next token in the stream is a T, and if so parses
  /// it. Returns `Some(T)` if the token matches and is successfully
  /// parsed, None if the token does not match or parsing fails.
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
  ) -> Option<T>;
}

impl PeekThenAdvance for syn::parse::ParseStream<'_> {
  fn peek_then_advance<
    T: syn::parse::Parse + crate::common::peekable::Peekable,
  >(
    &self,
  ) -> Option<T> {
    if self.peek(T::peekable()) {
      self.parse::<T>().ok()
    } else {
      None
    }
  }
}

pub trait LookaheadPeekThenAdvance {
  /// Checks if the next token in the lookahead is a T, and if so parses
  /// it. Returns `Some(T)` if the token matches and is successfully
  /// parsed, None if the token does not match or parsing fails.
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
  ) -> Option<T>
  where
    T: syn::parse::Parse + crate::common::peekable::Peekable;
}

impl LookaheadPeekThenAdvance for syn::parse::Lookahead1<'_> {
  fn peek_then_advance<
    T: syn::parse::Parse + crate::common::peekable::Peekable,
  >(
    &self,
    stream: syn::parse::ParseStream,
  ) -> Option<T> {
    if self.peek(T::peekable()) {
      stream.parse::<T>().ok()
    } else {
      None
    }
  }
}
