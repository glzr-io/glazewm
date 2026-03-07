/// Utilities for parsing attributes
pub mod attributes;
/// Utilities for parsing alternatives and combinators
pub mod branch;
/// Extension traits and utilities to make error handling more succinct
pub mod error_handling;
/// Utilities for parsing a named parameter (`name = value`)
pub mod named_parameter;
/// Type for a parsable type within parentheses
pub mod parenthesized;
/// Trait to get a `Peek` object from compatible syn types
pub mod peekable;
/// An owned version of a `syn::LitStr`
pub mod spanned_string;

pub(crate) use peekable::custom_keyword;
