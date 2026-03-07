//! Utilities for simplifying error handling and diagnostics.

pub mod prelude {
  // ToSpanError is not used yet, but will be used in the future.
  // TODO: Remove unused_imports allow
  #[allow(unused_imports)]
  pub use super::{EmitError, ThenError, ToError, ToSpanError};
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

// Very likely to be used in future.
// TODO: Remove dead code allow
#[allow(dead_code)]
/// Extension trait for any [syn::spanned::Spanned] type that creates a
/// [syn::Error] at its location. Use [ToError] where possible, as it
/// creates more accurately spanned errors.
pub trait ToSpanError {
  /// Creates a [syn::Error] at the location of this span with the given
  /// message.
  ///
  /// If the object can be tokenized, prefer using [ToError] instead, as it
  /// gives more accurately spanned errors.
  ///
  /// # Example
  /// ```
  /// # fn example(stream: syn::parse::ParseStream) -> syn::Result<()> {
  /// return Err(stream.span().serror("Expected ..."));
  /// # }
  /// ```
  fn serror<D: core::fmt::Display>(&self, message: D) -> syn::Error;
}

impl<T> ToSpanError for T
where
  T: syn::spanned::Spanned,
{
  fn serror<D: core::fmt::Display>(&self, message: D) -> syn::Error {
    syn::Error::new(self.span(), message)
  }
}

// Very likely to be used in future.
// TODO: Remove dead code allow
#[allow(dead_code)]
pub trait EmitError {
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
