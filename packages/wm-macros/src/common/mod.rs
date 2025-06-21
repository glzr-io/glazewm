/// Utilitirs for parsing attributes
pub mod attributes;
/// Utilities for parsing alternitives and combinators
pub mod branch;
/// Extensions for derive macros
pub mod derive;
/// Extension traits and utilities to make error handling more succinct
pub mod error_handling;
/// Extensions for syn ParseStreams and Lookaheads to combine common
/// operations
pub mod lookahead;
/// Utilitirs for parsing a named parameter (`name = vaue`)
pub mod named_parameter;
/// Type for a parsable type within parenthesis
pub mod parenthesized;
/// Trait to get a `Peek` object from compatible syn types
pub mod peekable;
/// An owned version of a `syn::LitStr`
pub mod spanned_string;

pub(crate) use peekable::custom_keyword;
