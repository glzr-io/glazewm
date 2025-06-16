/// Used to add a new message to an existing error.
pub trait ErrorContext {
  /// Adds context to the error, typically a string describing the context.
  /// The original error is preserved, although some editors may hide it
  /// unless the error is expanded.
  ///
  /// # Example
  /// ```
  /// # fn example(stream: syn::parse::ParseStream) -> syn::Result<()> {
  /// stream.parse::<syn::Ident>().add_context("Expected ...")?;
  /// # }
  /// ```
  fn add_context<D: core::fmt::Display>(self, context: D) -> Self;

  /// Replaces the error message.
  ///
  /// # Example
  /// ```
  /// # fn example(stream: syn::parse::ParseStream) -> syn::Result<()> {
  /// stream.parse::<syn::Ident>().set_context("Expected ...")?;
  /// # }
  /// ```
  fn set_context<D: core::fmt::Display>(self, context: D) -> Self;
}

impl<T> ErrorContext for Result<T, syn::Error> {
  fn add_context<D: core::fmt::Display>(self, context: D) -> Self {
    self.map_err(|e| e.add_context(context))
  }

  fn set_context<D: core::fmt::Display>(self, context: D) -> Self {
    self.map_err(|e| e.set_context(context))
  }
}

impl ErrorContext for syn::Error {
  fn add_context<D: core::fmt::Display>(mut self, context: D) -> Self {
    self.combine(syn::Error::new(self.span(), context));
    self
  }

  fn set_context<D: core::fmt::Display>(self, context: D) -> Self {
    syn::Error::new(self.span(), context)
  }
}

pub trait ThenError<E>
where
  Self: Sized,
{
  /// Converts the `Result` into a `Result`, returning `Ok(self)` if false,
  /// or `Err(error)` if true.
  ///
  /// # Example
  /// ```
  /// # fn example(string: &str, string_span: syn::Span) -> syn::Result<()> {
  /// string.is_empty().then_error(string_span.error("Expected a non-empty string"))?;
  /// # }
  /// ```
  fn then_error(self, error: E) -> Result<Self, E>;
}

impl<E> ThenError<E> for bool {
  fn then_error(self, error: E) -> Result<Self, E> {
    if self { Err(error) } else { Ok(self) }
  }
}

/// Converts a type that can be tokenized into a `syn::Error`.
pub trait ToError {
  /// Converts the object into a `syn::Error` with the provided message.
  ///
  /// # Example
  /// ```
  /// # fn example(ident: syn::Ident) -> syn::Result<()> {
  /// return Err(ident.error("Didn't expect an identifier here"));
  /// # }
  /// ```
  fn error<D: core::fmt::Display>(&self, message: D) -> syn::Error;
}

impl<T> ToError for T
where
  T: quote::ToTokens,
{
  fn error<D: core::fmt::Display>(&self, message: D) -> syn::Error {
    syn::Error::new_spanned(self, message)
  }
}

/// Converts a type that has a span into a `syn::Error`.
#[allow(dead_code)]
pub trait ToSpanError {
  /// Converts the object's span into a `syn::Error` with the provided
  /// message,
  ///
  /// # Example
  /// ```
  /// # fn example(stream: syn::parse::ParseStream) -> syn::Result<()> {
  /// return Err(stream.span().error("Expected ..."));
  /// # }
  /// ```
  fn error<D: core::fmt::Display>(&self, message: D) -> syn::Error;
}

impl<T> ToSpanError for T
where
  T: syn::spanned::Spanned,
{
  fn error<D: core::fmt::Display>(&self, message: D) -> syn::Error {
    syn::Error::new(self.span(), message)
  }
}
