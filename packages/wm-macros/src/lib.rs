// Enable proc macro diagnostics to allow emitting warnings and errors in
// line
#![feature(proc_macro_diagnostic)]

mod common;
mod enum_from_inner;
mod key;
mod subenum;
mod try_to_enum_discriminant;
use proc_macro::TokenStream;

mod prelude {
  pub use crate::common::{
    attributes::prelude::*, error_handling::prelude::*,
    peekable::prelude::*,
  };
}

enum Os {
  Windows,
  MacOS,
  Linux,
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

/// Generates conversions for an enum containing keys, using the `key`
/// attribute that specifies the name(s) of the key and the respective
/// native key variant for that key.
///
/// # Example
/// ```
/// enum WinKey {
///   A,
///   Ctrl,
///   PageUp
/// }
///
/// enum MacKey {
///   A,
///   Ctrl,
///   PageUp
/// }
///
/// enum LinuxKey {
///   Ctrl,
///   PageUp
/// }
///
/// #[derive(wm_macros::KeyConversions)]
/// #[key(win = WinKey, mac = MacKey, linux = LinuxKey)]
/// enum Key {
///   // Linux enum does not have an A key in this example, so marked as absent with `!`
///   #[key("a", win = A, mac = A, linux = !)]
///   A,
///   // Can use multiple names for the same key, separated by `|`.
///   #[key("ctrl" | "control", win = Ctrl, mac = Ctrl, linux = Ctrl)]
///   Ctrl,
///   // Using spaces in the name will generate variant names, such as in the following example
///   // "page up", "pageup" , "pageUp" "page_up" and "page-up" are all valid names generated from
///   // the single name.
///   #[key("page up", win = PageUp, mac = PageUp, linux = PageUp)]
///   PageUp,
///   // Wildcard for valid but unmatched keys
///   #[key(..)]
///   Custom(u16)
/// }
///
/// let name = "a";
/// let a_key = Key::from_str(name).unwrap(); // Will get None if the key does not have a match, and doesn't exist on the current keyboard layout.
///
/// let win_a = WinKey::A;
/// let a_key = Key::from_vk(win_a); // Infallible, will get Key::Custom if there isn't a match.
/// ```
#[proc_macro_derive(KeyConversions, attributes(key))]
pub fn key_conversions(input: TokenStream) -> TokenStream {
  key::key_conversions(input)
}

#[proc_macro_derive(TryToDiscriminant)]
pub fn try_to_discriminant(input: TokenStream) -> TokenStream {
  try_to_enum_discriminant::try_to_enum_discriminant(input)
}
