// Enable proc macro diagnostics to allow emitting warnings and errors in
// line
#![feature(proc_macro_diagnostic)]

mod common;
mod enum_from_inner;
mod subenum;
use proc_macro::TokenStream;

mod prelude {
  pub use crate::common::{
    attributes::prelude::*, error_handling::prelude::*,
    peekable::prelude::*,
  };
}

/// Creates subenums from a main enum, and defines
/// * `impl From<SubEnum> for MainEnum`
/// * `impl TryFrom<MainEnum> for SubEnum`
/// * `impl TryFrom<SubEnumOne> for SubEnumTwo` where `SubEnumOne` and
///   `SubEnumTwo` share variant(s).
///
/// Accepts a defaults block of attributes to be added to every subenum
/// ```
/// #[subenum(defaults, {
///   /// Subenum of [X]
///   #[derive(Clone, Debug)]
/// })]
/// ```
///
/// And any number of subenum declarations, which are defined as
/// ```
/// #[subenum(SubenumName, {
///   /// Subset of [X] that can be checked for equality.
///   #[derive(PartialEq)] // Will also derive [Clone] and [Debug] from the defaults block
/// })]
/// ```
///
/// # Example
/// ```
/// /// Your main enum documentation
/// // Note that the defaults block does not apply to the main enum itself.
/// #[derive(Clone, Debug, wm_macros::SubEnum)]
/// #[subenum(defaults, {
///   /// Subenum of [MainEnum]
///   #[derive(Clone, Debug)]
/// })]
/// #[subenum(Similar, {
///   /// Subset of [MainEnum] that can be checked for equality.
///   #[derive(PartialEq)] // Will also derive [Clone] and [Debug] from the defaults block
/// })]
/// #[subenum(Hashable, {
///   /// Subset of [MainEnum] that can be hashed.
///   #[derive(Hash)] // Will also derive [Clone] and [Debug] from the defaults block
/// })]
/// pub enum MainEnum {
///   Path(PathBuf),
///   #[subenum(Similar, Hashable)]
///   Length(i32),
///   #[subenum(Hashable)]
///   Name(String)
/// }
///
/// let name = String::from("example");
/// let name_enum = MainEnum::from(name);
///
/// // Try to convert MainEnum to Hashable.
/// let hashable_name = Hashable::try_from(name_enum).unwrap(); // Will succeed, as `Name` is present in the `Hashable` subenum.
/// hashable_name.hash();
///
/// let similar_name = Similar::try_from(hashable_name.clone());
/// assert!(similar_name.is_err()); // Will fail, as `Name` is not present in the `Similar` subenum.
///
/// // And convert it back (infallible).
/// let name_enum: MainEnum = hashable_name.into();
///
/// let length = 42;
/// let length_enum: MainEnum = length.into();
///
/// let similar_length = Similar::try_from(length_enum).unwrap();
///
/// let other_length =  Hashable::Length(42);
/// let other_similar_length = Similar::try_from(other_length).unwrap(); // Convert between subenums that share variants.
///
/// assert_eq!(similar_length, other_similar_length);
/// ```
#[proc_macro_derive(SubEnum, attributes(subenum))]
pub fn sub_enum(input: TokenStream) -> TokenStream {
  subenum::sub_enum(input)
}

/// Creates `impl From<Inner> for Enum` and `impl TryFrom<Enum> for Inner`
///
/// # Example
/// ```
/// struct One;
/// struct Two;
///
/// #[derive(wm_macros::EnumFromInner)]
/// enum MyEnum {
///   One(One),
///   Two(Two),
/// }
///
/// let one = One;
/// let my_enum: MyEnum = one.into(); // Converts One into MyEnum::One(One)
///
/// let one = my_enum.try_into().unwrap(); // Attempts to convert MyEnum::One(One) into One
///
/// let two = Two;
/// let my_enum: MyEnum = two.into(); // Converts Two into MyEnum::Two(Two)
///
/// let one = my_enum.try_into(); // Will fail, as MyEnum::Two(Two) cannot be converted to One
/// assert!(one.is_err());
/// ```
#[proc_macro_derive(EnumFromInner)]
pub fn enum_from_inner(input: TokenStream) -> TokenStream {
  enum_from_inner::enum_from_inner(input)
}
