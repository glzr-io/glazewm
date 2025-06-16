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
  fn peekable() -> impl syn::parse::Peek;
}

/// Custom keyword macro to define a syn custom keyword that also
/// implements Peekable.
macro_rules! custom_keyword {
  ($name:ident) => {
    syn::custom_keyword!($name);

    impl $crate::common::peekable::Peekable for $name {
      fn peekable() -> impl syn::parse::Peek {
        $name
      }
    }
  };
}
pub(crate) use custom_keyword;

macro_rules! impl_peekable {
  ($($name:tt)+) => {
    impl Peekable for $($name)+ {
      fn peekable() -> impl syn::parse::Peek {
        $($name)+
      }
    }
  };
}

impl_peekable!(syn::Ident);
impl_peekable!(syn::LitStr);
// TODO: Other syn types

// Copied from syn::Token!
impl_peekable!(syn::Token![abstract]);
impl_peekable!(syn::Token![as]);
impl_peekable!(syn::Token![async]);
impl_peekable!(syn::Token![auto]);
impl_peekable!(syn::Token![await]);
impl_peekable!(syn::Token![become]);
impl_peekable!(syn::Token![box]);
impl_peekable!(syn::Token![break]);
impl_peekable!(syn::Token![const]);
impl_peekable!(syn::Token![continue]);
impl_peekable!(syn::Token![crate]);
impl_peekable!(syn::Token![default]);
impl_peekable!(syn::Token![do]);
impl_peekable!(syn::Token![dyn]);
impl_peekable!(syn::Token![else]);
impl_peekable!(syn::Token![enum]);
impl_peekable!(syn::Token![extern]);
impl_peekable!(syn::Token![final]);
impl_peekable!(syn::Token![fn]);
impl_peekable!(syn::Token![for]);
impl_peekable!(syn::Token![if]);
impl_peekable!(syn::Token![impl]);
impl_peekable!(syn::Token![in]);
impl_peekable!(syn::Token![let]);
impl_peekable!(syn::Token![loop]);
impl_peekable!(syn::Token![macro]);
impl_peekable!(syn::Token![match]);
impl_peekable!(syn::Token![mod]);
impl_peekable!(syn::Token![move]);
impl_peekable!(syn::Token![mut]);
impl_peekable!(syn::Token![override]);
impl_peekable!(syn::Token![priv]);
impl_peekable!(syn::Token![pub]);
impl_peekable!(syn::Token![ref]);
impl_peekable!(syn::Token![return]);
impl_peekable!(syn::Token![Self]);
impl_peekable!(syn::Token![self]);
impl_peekable!(syn::Token![static]);
impl_peekable!(syn::Token![struct]);
impl_peekable!(syn::Token![super]);
impl_peekable!(syn::Token![trait]);
impl_peekable!(syn::Token![try]);
impl_peekable!(syn::Token![type]);
impl_peekable!(syn::Token![typeof]);
impl_peekable!(syn::Token![union]);
impl_peekable!(syn::Token![unsafe]);
impl_peekable!(syn::Token![unsized]);
impl_peekable!(syn::Token![use]);
impl_peekable!(syn::Token![virtual]);
impl_peekable!(syn::Token![where]);
impl_peekable!(syn::Token![while]);
impl_peekable!(syn::Token![yield]);
impl_peekable!(syn::Token![&]);
impl_peekable!(syn::Token![&&]);
impl_peekable!(syn::Token![&=]);
impl_peekable!(syn::Token![@]);
impl_peekable!(syn::Token![^]);
impl_peekable!(syn::Token![^=]);
impl_peekable!(syn::Token![:]);
impl_peekable!(syn::Token![,]);
impl_peekable!(syn::Token![$]);
impl_peekable!(syn::Token![.]);
impl_peekable!(syn::Token![..]);
impl_peekable!(syn::Token![...]);
impl_peekable!(syn::Token![..=]);
impl_peekable!(syn::Token![=]);
impl_peekable!(syn::Token![==]);
impl_peekable!(syn::Token![=>]);
impl_peekable!(syn::Token![>=]);
impl_peekable!(syn::Token![>]);
impl_peekable!(syn::Token![<-]);
impl_peekable!(syn::Token![<=]);
impl_peekable!(syn::Token![<]);
impl_peekable!(syn::Token![-]);
impl_peekable!(syn::Token![-=]);
impl_peekable!(syn::Token![!=]);
impl_peekable!(syn::Token![!]);
impl_peekable!(syn::Token![|]);
impl_peekable!(syn::Token![|=]);
impl_peekable!(syn::Token![||]);
impl_peekable!(syn::Token![::]);
impl_peekable!(syn::Token![%]);
impl_peekable!(syn::Token![%=]);
impl_peekable!(syn::Token![+]);
impl_peekable!(syn::Token![+=]);
impl_peekable!(syn::Token![#]);
impl_peekable!(syn::Token![?]);
impl_peekable!(syn::Token![->]);
impl_peekable!(syn::Token![;]);
impl_peekable!(syn::Token![<<]);
impl_peekable!(syn::Token![<<=]);
impl_peekable!(syn::Token![>>]);
impl_peekable!(syn::Token![>>=]);
impl_peekable!(syn::Token![/]);
impl_peekable!(syn::Token![/=]);
impl_peekable!(syn::Token![*]);
impl_peekable!(syn::Token![*=]);
impl_peekable!(syn::Token![~]);
impl_peekable!(syn::Token![_]);
