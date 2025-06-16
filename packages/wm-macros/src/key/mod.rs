use attrs::{
  enums::EnumAttr,
  find_key_attr,
  variant::{VariantAttr, VkValue},
};
use proc_macro::TokenStream;
use quote::{ToTokens as _, quote};
use syn::{DeriveInput, parse_macro_input};

use crate::common::error_handling::ToError as _;

mod attrs;
mod from_str;
mod from_vk;
mod into_vk;

const KEY_ATTR_NAME: &str = "key";

const MISSING_KEY_ATTR_ON_ENUM_ERR_MSG: &str = "Missing `#[key]` attribute on the enum itself. It should be annotated like this:\n```\n#[key(win_prefix = <key::codes::prefix>, macos_prefix = <key::codes::prefix>)]\nenum // ...\n```";
const MISSING_KEY_ATTR_ON_VARIANT_ERR_MSG: &str = "Missing `#[key]` attribute for this variant. Key variants must be annotated with `#[key(\"string\" | \"list\", win = <key code>, macos = <key code>)]` and the wildcard variant must be annotated with `#[key(..)]`";

/// Holds the Key enum variant information, including the identifier and
/// attributes parsed from the `#[key(...)]` attribute, such as the string
/// aliases and the assosicated platform-specific virtual key codes.
#[derive(Debug, Clone)]
struct Key {
  pub ident: syn::Ident,
  pub attrs: VariantAttr,
}

/// Converts an enum variant into a Key struct. This collects the ident and
/// parses the attribute parameters to construct a `Key` instance.
/// ```
/// enum Key {
///   #[key(<attribute parameters>)]
///   <ident>,
/// }
/// ```
fn variant_to_key(variant: &syn::Variant) -> syn::Result<Key> {
  let ident = variant.ident.clone();
  let attrs = &variant.attrs;

  let attr = attrs::find_key_attr(attrs)
    .ok_or(variant.ident.error(MISSING_KEY_ATTR_ON_VARIANT_ERR_MSG))?;

  let attrs: VariantAttr = attr.parse_args()?;

  Ok(Key { ident, attrs })
}

/// Parses the attribute parmeters for the enum itself from the enums
/// `#[key(...)]` attribute.
///
/// ```
/// #[key(<attribute parameters>)]
/// enum Key {
///   // ...
/// }
/// ```
///
/// Returns an `EnumAttr` struct containing the parsed parameters, or an
/// error if the attribute is missing or malformed.
fn get_enum_attr(input: &DeriveInput) -> syn::Result<EnumAttr> {
  let name = &input.ident;

  // Find the `#[key]` attribute on the enum itself. This is required to
  // pass in the absolute paths for each platform's key codes.
  let enum_attr = find_key_attr(&input.attrs)
    .ok_or(name.error(MISSING_KEY_ATTR_ON_ENUM_ERR_MSG))?;

  // Parse the enum attribute arguments into an `EnumAttr` struct.
  // This calls the `syn::parse::Parse` implementation for `EnumAttr` on
  // the arguments of the attribute.
  enum_attr.parse_args::<attrs::enums::EnumAttr>()
}

/// Get the enum data from the input.
/// Returns an error if the item being derived is not an enum.
fn get_enum_data(input: &DeriveInput) -> syn::Result<&syn::DataEnum> {
  let name = &input.ident;

  // Error out if the input is not an enum
  match &input.data {
    syn::Data::Enum(data) => Ok(data),
    _ => Err(name.error("This macro can only be used on enums")),
  }
}

/// This macro derives the `KeyConversions` trait for an enum.
pub fn key_conversions(input: TokenStream) -> TokenStream {
  // Syn has inbuilt parsing for derive macros, which returns the AST for
  // the derived item in a more friendly form.
  let input = parse_macro_input!(input as DeriveInput);

  let name = &input.ident;

  let enum_data = match get_enum_data(&input) {
    Ok(data) => data,
    Err(err) => {
      return error_output(name, &[err]).into();
    }
  };

  let enum_attrs = match get_enum_attr(&input) {
    Ok(attrs) => attrs,
    Err(err) => {
      return error_output(name, &[err]).into();
    }
  };

  // Iterate over the enum variants and convert them into `Key` structs,
  // then partition them into valid and error variants.
  let (variants, errors): (Vec<_>, Vec<_>) = enum_data
    .variants
    .iter()
    .map(variant_to_key)
    .partition(|key| !key.is_err());

  let keys: Vec<_> = variants
    .into_iter()
    // Saftey: Just partitioned the results, so this unwrap is safe.
    .map(|var| unsafe { var.unwrap_unchecked() })
    .collect();

  // Find any duplicate keys in the parsed keys.
  // Will give an error if a key is defined multiple times without being
  // explicitly marked as an alias
  let duplicate_errors = find_duplicate_keys(&keys);

  // Collect the errors from the partitioned results and the duplicate
  // errors. Converts the errors into compile errors to include in the
  // output, which is what gives accuratly spanned error messages.
  let errors: Vec<_> = errors
    .into_iter()
    // Saftey: Just partitioned the results, so this unwrap is safe.
    .map(|err| unsafe { err.unwrap_err_unchecked() })
    .chain(duplicate_errors)
    .map(|err| err.into_compile_error())
    .collect();

  // Make each function impl
  let from_str_impl = from_str::make_from_str_impl(&keys);
  let from_vk_impl = from_vk::make_from_vk_impl(&keys, &enum_attrs);
  let into_vk_impl = into_vk::make_into_vk_impl(&keys, &enum_attrs);

  // Create the output token stream
  // Uses `quote!` to convert normal Rust code into a token stream
  // Variable names can be interpolated using a #, although this cannot be
  // used with dot notation or the like - single identifiers only.
  // Errors are unpacked using `#(#var)<separator>*` syntax (separators are
  // optional).
  let expanded = quote! {
      impl #name {
        #from_str_impl

        #from_vk_impl

        #into_vk_impl
      }

      #(#errors)*
  };

  // quote uses proc_macro2::TokenStream, so we need to convert it back
  // into the compiler given proc_macro::TokenStream.
  expanded.into()
}

/// Find duplicate keys per platform in the provided keys slice.
/// Returns a vector of `syn::Error` instances for each duplicate key
/// found, containing a reminder to use the `Vert` wrapper for virtual
/// keys.
fn find_duplicate_keys(keys: &[Key]) -> Vec<syn::Error> {
  let mut win_seen = std::collections::HashSet::new();
  let mut macos_seen = std::collections::HashSet::new();
  let mut duplicates = Vec::new();

  for key in keys {
    if let VariantAttr::Key(vk_value) = &key.attrs {
      if let VkValue::Key(vk_value) = &vk_value.key_codes.win {
        if !win_seen.insert(vk_value.clone()) {
          // If the key is already seen, we have a duplicate
          duplicates.push(
            vk_value.error(format!("Duplicate key value: {}. Wrap virtual keys with `Virt(<key code>)`.", vk_value.to_token_stream())));
        }
      }

      if let VkValue::Key(vk_value) = &vk_value.key_codes.macos {
        if !macos_seen.insert(vk_value.clone()) {
          // If the key is already seen, we have a duplicate
          duplicates.push(
            vk_value.error(format!("Duplicate key value: {}. Wrap virtual keys with `Virt(<key code>)`.", vk_value.to_token_stream())));
        }
      }
    }
  }

  duplicates
}

/// Generate the error output for the macro when it encounters errors
/// Instead of panicking, it will return a token stream that contains
/// the errors as compile errors, along with some default implementations
/// so that the macro erroring does not cause every usage of the generated
/// functions to also show an error.
fn error_output(
  name: &syn::Ident,
  errors: &[syn::Error],
) -> proc_macro2::TokenStream {
  let errors = errors
    .iter()
    .map(|err| err.to_compile_error())
    .collect::<proc_macro2::TokenStream>();

  let default_impls = default_fn_impls(name);

  quote! {
    #default_impls

    #errors
  }
}

/// Generate some generic default implementations just so that if the macro
/// errors out we can spit out a fn impl so that every usage of the
/// generated fn impls does not also show an error.
fn default_fn_impls(name: &syn::Ident) -> proc_macro2::TokenStream {
  quote! {
    impl #name {
      pub fn from_str(key: &str) -> Option<Self> {
        panic!("`from_str` is not implemented for this enum. The macro likely panicked while parsing the input. Please check the input for errors.");
      }

      pub fn from_vk(vk: u16) -> Self {
        panic!("`from_vk` is not implemented for this enum. The macro likely panicked while parsing the input. Please check the input for errors.");
      }

      pub fn into_vk(self) -> u16 {
        panic!("`into_vk` is not implemented for this enum. The macro likely panicked while parsing the input. Please check the input for errors.");
      }
    }
  }
}
