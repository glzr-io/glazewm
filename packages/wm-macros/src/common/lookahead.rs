pub trait PeekThenAdvance {
  /// Checks if the next token in the stream matches the provided token
  /// type using the peek method, and if it does, it advances the stream
  /// and parses the next token as type `T`.
  /// Returns `Some(T)` if the token matches and is successfully parsed,
  /// None if the token does not match or parsing fails.
  ///
  /// Useful for scenarios such as:
  /// ```
  /// if stream.peek(T) {
  ///   stream.parse::<T>();
  /// }
  /// ```
  ///
  /// # Example
  /// ```
  /// if let Some(value) = stream.peek_then_advance::<syn::Ident, _>(syn::Ident) {
  ///   // ...
  /// }
  /// ```
  fn peek_then_advance<T: syn::parse::Parse, P: syn::parse::Peek>(
    &self,
    token: P,
  ) -> Option<T>;
}

impl PeekThenAdvance for syn::parse::ParseStream<'_> {
  fn peek_then_advance<T: syn::parse::Parse, P: syn::parse::Peek>(
    &self,
    token: P,
  ) -> Option<T> {
    if self.peek(token) {
      self.parse::<T>().ok()
    } else {
      None
    }
  }
}

pub trait LookaheadPeekThenAdvance {
  /// Checks if the next token in the lookahead matches the provided token
  /// type using the peek method, and if it does, it advances the given
  /// stream and parses the next token as type `T`.
  /// Returns `Some(T)` if the token matches and is successfully parsed,
  /// None if the token does not match or parsing fails.
  ///
  /// Useful for scenarios such as:
  /// ```
  /// if lookahead.peek(T) {
  ///   stream.parse::<T>();
  /// }
  /// ```
  ///
  /// # Example
  /// ```
  /// if let Some(value) = lookahead.peek_then_advance::<syn::Ident, _>(syn::Ident, stream) {
  ///   // ...
  /// }
  /// ```
  fn peek_then_advance<T, P>(
    &self,
    token: P,
    stream: syn::parse::ParseStream,
  ) -> Option<T>
  where
    T: syn::parse::Parse,
    P: syn::parse::Peek;
}

impl LookaheadPeekThenAdvance for syn::parse::Lookahead1<'_> {
  fn peek_then_advance<T: syn::parse::Parse, P: syn::parse::Peek>(
    &self,
    token: P,
    stream: syn::parse::ParseStream,
  ) -> Option<T> {
    if self.peek(token) {
      stream.parse::<T>().ok()
    } else {
      None
    }
  }
}
