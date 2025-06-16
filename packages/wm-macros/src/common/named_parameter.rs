pub trait ParseNamedParameter {
  /// Parses a named parameter from the stream, expecting a name `N`
  /// followed by an `=` followed by the parameter `T`.
  ///
  /// If the next token does not match the expected name, the lookahead
  /// error is forwarded directly. If the name matdhes but then parsing
  /// fails, `Ok(Err(...))` is returned.
  ///
  /// # Example
  /// Attempts to parse `win = <syn::Path>` from the stream.
  /// ```
  /// # fn example(stream: syn::parse::ParseStream) -> syn::Result<()> {
  /// crate::common::custom_keyword!(win);
  ///
  /// stream.parse_named_parameter::<win, syn::Path>()??;
  /// # }
  fn parse_named_parameter<N, T>(
    &self,
  ) -> syn::Result<syn::Result<(N, T)>>
  where
    N: syn::parse::Parse + crate::common::peekable::Peekable,
    T: syn::parse::Parse;

  /// Parses a named parameter from the stream, using custom parsers.
  ///
  /// Parser errors are forwarded directly.
  ///
  /// # Examples
  /// Native syn parsers can be used by passing in `Type::parse`
  /// ```
  /// # fn example(stream: syn::parse::ParseStream) -> syn::Result<()> {
  /// stream.parse_named_parameter_with(syn::Ident::parse, syn::Path::parse)?;
  /// # }
  /// ```
  /// Closures can be used for more complex parsing logic, in this case
  /// usng in combination with alt_if to parse either a `win` or `macos`
  /// prefix. ```
  /// # fn example(stream: syn::parse::ParseStream) -> syn::Result<()> {
  /// crate::common::custom_keyword!(win_prefix);
  /// crate::common::custom_keyword!(macos_prefix);
  ///
  /// let mut win_prefix = None;
  /// let mut macos_prefix = None;
  ///
  /// while !stream.is_empty() {
  ///   let (os, path) = stream.parse_named_parameter_with(
  ///     |input| {
  ///       input.alt_if::<win_prefix, macos_prefix>(
  ///         win_prefix.is_none(),
  ///         macos_prefix.is_none()
  ///       ).map(|either| match either {
  ///         Either::Left(_) => Os::Windows,
  ///         Either::Right(_) => Os::MacOS,
  ///       }),
  ///     },
  ///     syn::Path::parse
  ///   )?;
  ///
  ///   match os {
  ///     Os::Windows => { win_prefix = Some(path); },
  ///     Os::MacOS => { macos_prefix = Some(path); },
  ///   }
  /// }
  /// # }
  /// ```
  fn parse_named_parameter_with<
    N,
    T,
    E,
    NF: FnOnce(syn::parse::ParseStream) -> Result<N, E>,
    PF: FnOnce(syn::parse::ParseStream) -> Result<T, E>,
  >(
    &self,
    name_fn: NF,
    parse_fn: PF,
  ) -> Result<(N, T), E>
  where
    E: From<syn::Error>;
}

impl ParseNamedParameter for syn::parse::ParseStream<'_> {
  fn parse_named_parameter<N, T>(&self) -> syn::Result<syn::Result<(N, T)>>
  where
    N: syn::parse::Parse + crate::common::peekable::Peekable,
    T: syn::parse::Parse,
  {
    let lookahead = self.lookahead1();
    if !lookahead.peek(N::peekable()) {
      return Err(lookahead.error());
    }
    Ok(self.parse_named_parameter_with(N::parse, T::parse))
  }

  fn parse_named_parameter_with<
    N,
    T,
    E,
    NF: FnOnce(syn::parse::ParseStream) -> Result<N, E>,
    PF: FnOnce(syn::parse::ParseStream) -> Result<T, E>,
  >(
    &self,
    name_fn: NF,
    parse_fn: PF,
  ) -> Result<(N, T), E>
  where
    E: From<syn::Error>,
  {
    let name = name_fn(self)?;
    self.parse::<syn::Token![=]>()?;
    let value = parse_fn(self)?;
    Ok((name, value))
  }
}
