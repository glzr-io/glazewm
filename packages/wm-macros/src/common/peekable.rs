//! Custom peek implementation to support generics and custom peek
//! functions.

// Syn has a `Peek` trait, but it works weirdly. There is `syn::Ident` the
// type which implements `Parse` and `syn::Ident` the function which
// implements `Peek`. This file is for implementing a custom method of
// peeking using the type itself rather than a function using the same name
// as a type. This allows peeking to work nicer with generics.

pub mod prelude {
  pub use super::{Peekable, TPeek};
}

/// Trait for any stream that can peek at the next token.
pub trait PeekableStream {
  fn is_empty(&self) -> bool;
  fn peek<T: syn::parse::Peek>(&self, token: T) -> bool;
}

impl PeekableStream for &syn::parse::ParseStream<'_> {
  fn peek<T: syn::parse::Peek>(&self, token: T) -> bool {
    (*self).peek(token)
  }
  fn is_empty(&self) -> bool {
    (*self).is_empty()
  }
}

impl PeekableStream for syn::parse::ParseStream<'_> {
  fn peek<T: syn::parse::Peek>(&self, token: T) -> bool {
    (*self).peek(token)
  }

  fn is_empty(&self) -> bool {
    (*self).is_empty()
  }
}
impl PeekableStream for syn::parse::ParseBuffer<'_> {
  fn peek<T: syn::parse::Peek>(&self, token: T) -> bool {
    self.peek(token)
  }

  fn is_empty(&self) -> bool {
    self.is_empty()
  }
}
impl PeekableStream for syn::parse::Lookahead1<'_> {
  fn peek<T: syn::parse::Peek>(&self, token: T) -> bool {
    self.peek(token)
  }

  fn is_empty(&self) -> bool {
    self.peek(syn::parse::End)
  }
}
impl PeekableStream for &syn::parse::Lookahead1<'_> {
  fn peek<T: syn::parse::Peek>(&self, token: T) -> bool {
    (*self).peek(token)
  }

  fn is_empty(&self) -> bool {
    (*self).peek(syn::parse::End)
  }
}

/// Custom trait for types that can be peeked.
pub trait Peekable {
  /// Gets the type's `Peek` implementation, since in syn the type
  /// implements `Parse` but there is a function with the same path that
  /// implements `Peek`. So this trait is used to get the function (`Peek`)
  /// from the type (`Parse`).
  ///
  /// # Examble
  /// ```
  /// fn peek_then_parse<T: Parse + Peekable>(stream: syn::parse::ParseStream) -> syn::Result<T> {
  ///   if stream.peek(T::peekable()) {
  ///     let value = stream.parse::<T>()?;
  ///   }
  /// }
  /// ```
  fn peek<T>(stream: T) -> bool
  where
    T: PeekableStream;
  // Useful for debugging and custom error messages.
  #[allow(dead_code)]
  fn display() -> &'static str;
}

/// Trait for the types in syn that have a corresponding function that
/// implements [syn::parse::Peek]. E.g. it implements a method for
/// [syn::Ident] the type that returns [syn::Ident] the function.
pub trait SynPeek {
  fn peekable() -> impl syn::parse::Peek;
  // To be forwarded to [Peekable]
  #[allow(dead_code)]
  fn display() -> &'static str;
}

/// Implement [Peekable] for [SynPeek]
impl<T: SynPeek> Peekable for T {
  fn peek<S>(stream: S) -> bool
  where
    S: PeekableStream,
  {
    stream.peek(T::peekable())
  }

  fn display() -> &'static str {
    T::display()
  }
}

/// Helper fucntion to get the display string for a peekable type.
// Used in a macro call
#[allow(dead_code)]
pub fn get_peek_display<T: syn::parse::Peek>(_peek: T) -> &'static str {
  use syn::token::Token;
  T::Token::display()
}

/// Extends the [PeekableStream] trait with a method to peek at a type
/// rather than a value.
///
/// # Example
/// ```
/// # fn example(stream: syn::parse::ParseStream) -> syn::Result<()> {
/// // Allows for
/// stream.tpeek::<syn::Ident>()?;
/// // Rather than
/// stream.peek(syn::Ident)?;
/// # }
pub trait TPeek<'a> {
  fn tpeek<T>(&'a self) -> bool
  where
    T: Peekable;
}

impl<'a, T> TPeek<'a> for T
where
  &'a T: PeekableStream + 'a,
{
  fn tpeek<U>(&'a self) -> bool
  where
    U: Peekable,
  {
    U::peek(self)
  }
}

/// Custom keyword macro to define a syn custom keyword that also
/// implements Peekable.
macro_rules! custom_keyword {
  ($name:ident) => {
    syn::custom_keyword!($name);

    impl $crate::common::peekable::SynPeek for $name {
      fn peekable() -> impl syn::parse::Peek {
        $name
      }

      fn display() -> &'static str {
        crate::common::peekable::get_peek_display(Self::peekable())
      }
    }
  };
}
pub(crate) use custom_keyword;

/// Macro for implementing [SynPeek] for a type that implements
/// [syn::parse::Peek].
macro_rules! impl_syn_peek {
  ($($name:tt)+) => {
    impl SynPeek for $($name)+ {
      fn peekable() -> impl syn::parse::Peek {
        $($name)+
      }

      fn display() -> &'static str {
        crate::common::peekable::get_peek_display(Self::peekable())
      }
    }
  };
}

impl_syn_peek!(syn::Ident);
impl_syn_peek!(syn::LitStr);
// TODO: Other syn types

// Copied from syn::Token!
impl_syn_peek!(syn::Token![abstract]);
impl_syn_peek!(syn::Token![as]);
impl_syn_peek!(syn::Token![async]);
impl_syn_peek!(syn::Token![auto]);
impl_syn_peek!(syn::Token![await]);
impl_syn_peek!(syn::Token![become]);
impl_syn_peek!(syn::Token![box]);
impl_syn_peek!(syn::Token![break]);
impl_syn_peek!(syn::Token![const]);
impl_syn_peek!(syn::Token![continue]);
impl_syn_peek!(syn::Token![crate]);
impl_syn_peek!(syn::Token![default]);
impl_syn_peek!(syn::Token![do]);
impl_syn_peek!(syn::Token![dyn]);
impl_syn_peek!(syn::Token![else]);
impl_syn_peek!(syn::Token![enum]);
impl_syn_peek!(syn::Token![extern]);
impl_syn_peek!(syn::Token![final]);
impl_syn_peek!(syn::Token![fn]);
impl_syn_peek!(syn::Token![for]);
impl_syn_peek!(syn::Token![if]);
impl_syn_peek!(syn::Token![impl]);
impl_syn_peek!(syn::Token![in]);
impl_syn_peek!(syn::Token![let]);
impl_syn_peek!(syn::Token![loop]);
impl_syn_peek!(syn::Token![macro]);
impl_syn_peek!(syn::Token![match]);
impl_syn_peek!(syn::Token![mod]);
impl_syn_peek!(syn::Token![move]);
impl_syn_peek!(syn::Token![mut]);
impl_syn_peek!(syn::Token![override]);
impl_syn_peek!(syn::Token![priv]);
impl_syn_peek!(syn::Token![pub]);
impl_syn_peek!(syn::Token![ref]);
impl_syn_peek!(syn::Token![return]);
impl_syn_peek!(syn::Token![Self]);
impl_syn_peek!(syn::Token![self]);
impl_syn_peek!(syn::Token![static]);
impl_syn_peek!(syn::Token![struct]);
impl_syn_peek!(syn::Token![super]);
impl_syn_peek!(syn::Token![trait]);
impl_syn_peek!(syn::Token![try]);
impl_syn_peek!(syn::Token![type]);
impl_syn_peek!(syn::Token![typeof]);
impl_syn_peek!(syn::Token![union]);
impl_syn_peek!(syn::Token![unsafe]);
impl_syn_peek!(syn::Token![unsized]);
impl_syn_peek!(syn::Token![use]);
impl_syn_peek!(syn::Token![virtual]);
impl_syn_peek!(syn::Token![where]);
impl_syn_peek!(syn::Token![while]);
impl_syn_peek!(syn::Token![yield]);
impl_syn_peek!(syn::Token![&]);
impl_syn_peek!(syn::Token![&&]);
impl_syn_peek!(syn::Token![&=]);
impl_syn_peek!(syn::Token![@]);
impl_syn_peek!(syn::Token![^]);
impl_syn_peek!(syn::Token![^=]);
impl_syn_peek!(syn::Token![:]);
impl_syn_peek!(syn::Token![,]);
impl_syn_peek!(syn::Token![$]);
impl_syn_peek!(syn::Token![.]);
impl_syn_peek!(syn::Token![..]);
impl_syn_peek!(syn::Token![...]);
impl_syn_peek!(syn::Token![..=]);
impl_syn_peek!(syn::Token![=]);
impl_syn_peek!(syn::Token![==]);
impl_syn_peek!(syn::Token![=>]);
impl_syn_peek!(syn::Token![>=]);
impl_syn_peek!(syn::Token![>]);
impl_syn_peek!(syn::Token![<-]);
impl_syn_peek!(syn::Token![<=]);
impl_syn_peek!(syn::Token![<]);
impl_syn_peek!(syn::Token![-]);
impl_syn_peek!(syn::Token![-=]);
impl_syn_peek!(syn::Token![!=]);
impl_syn_peek!(syn::Token![!]);
impl_syn_peek!(syn::Token![|]);
impl_syn_peek!(syn::Token![|=]);
impl_syn_peek!(syn::Token![||]);
impl_syn_peek!(syn::Token![::]);
impl_syn_peek!(syn::Token![%]);
impl_syn_peek!(syn::Token![%=]);
impl_syn_peek!(syn::Token![+]);
impl_syn_peek!(syn::Token![+=]);
impl_syn_peek!(syn::Token![#]);
impl_syn_peek!(syn::Token![?]);
impl_syn_peek!(syn::Token![->]);
impl_syn_peek!(syn::Token![;]);
impl_syn_peek!(syn::Token![<<]);
impl_syn_peek!(syn::Token![<<=]);
impl_syn_peek!(syn::Token![>>]);
impl_syn_peek!(syn::Token![>>=]);
impl_syn_peek!(syn::Token![/]);
impl_syn_peek!(syn::Token![/=]);
impl_syn_peek!(syn::Token![*]);
impl_syn_peek!(syn::Token![*=]);
impl_syn_peek!(syn::Token![~]);
impl_syn_peek!(syn::Token![_]);
