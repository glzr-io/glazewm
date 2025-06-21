//! Utilities for simplifying error handling and diagnostics.

pub mod prelude {
  #[allow(unused_imports)]
  pub use super::{
    EmitError, ErrorContext, ThenError, ToError, ToSpanError,
  };
}

/// Extension trait for [syn::Result] to add or replace a message of an
/// existing error.
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

/// Extends the `bool` type with a method that returns an error if the
/// value is `true`.
pub trait ThenError<E>
where
  Self: Sized,
{
  /// Returns `Err(error)` if the value is `true`, otherwise returns
  /// `Ok(self)`.
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

/// Extension trait for any type that can be tokenized to create a
/// [syn::Error] at its location.
pub trait ToError {
  /// Create a [syn::Error] at the span of this object with the given
  /// message.
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

/// Extension trait for any [syn::spanned::Spanned] type that creates a
/// [syn::Error] at its location.
#[allow(dead_code)]
pub trait ToSpanError {
  /// Creates a [syn::Error] at the location of this span with the given
  /// message.
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

#[allow(dead_code)]
pub trait EmitError {
  /// Directly emits an error message at the span of this object.
  /// Should only be used when you are sure that the error should be
  /// emitted, see [ToError] to convert to a [syn::Error] so allow the
  /// error to be propagated.
  fn emit_error<D: Into<String>>(&self, message: D);
  /// Directly emits a warning message at the span of this object.
  fn emit_warning<D: Into<String>>(&self, message: D);
  /// Emits a help message at the span of this object.
  fn emit_help<D: Into<String>>(&self, message: D);
  /// Emits a note message at the span of this object.
  fn emit_note<D: Into<String>>(&self, message: D);
}

impl<T> EmitError for T
where
  T: syn::spanned::Spanned,
{
  fn emit_error<D: Into<String>>(&self, message: D) {
    self.span().unwrap().error(message).emit();
  }

  fn emit_warning<D: Into<String>>(&self, message: D) {
    self.span().unwrap().warning(message).emit();
  }

  fn emit_help<D: Into<String>>(&self, message: D) {
    self.span().unwrap().help(message).emit();
  }

  fn emit_note<D: Into<String>>(&self, message: D) {
    self.span().unwrap().note(message).emit();
  }
}
