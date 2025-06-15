use attrs::{
  find_key_attr,
  variant::{VariantAttrs, VariantVkValue},
};
use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{DeriveInput, parse_macro_input};

mod attrs;
mod from_str;
mod from_vk;
mod into_vk;

/// Holds the Key enum variant information, including the identifier and
/// attributes parsed from the `#[key(...)]` attribute, such as the string
/// aliases and the assosicated platform-specific virtual key codes.
#[derive(Debug, Clone)]
struct Key {
  pub ident: syn::Ident,
  pub attrs: VariantAttrs,
}

/// Converts an enum variant into a Key struct. This collects the relevant
/// ident (`A`, `B`, etc.) and the string and virtual key (VK) values from
/// the `#[key("a", win = <key code>, macos = <key code>)]` attribute.
fn variant_to_key(variant: &syn::Variant) -> syn::Result<Key> {
  let ident = variant.ident.clone();
  let attrs = &variant.attrs;

  let attr = match attrs::find_key_attr(attrs) {
    Some(attr) => attr,
    None => {
      // Return an error if the variant does not have the attribute above
      // it
      return Err(syn::Error::new_spanned(
        &variant.ident,
        "Missing `#[key]` attribute for this variant. Key variants must be annotated with `#[key(\"string\" | \"list\", win = <key code>, macos = <key code>)]` and the wildcard variant must be annotated with `#[key(..)]`",
      ));
    }
  };

  // Parse the `#[key(...)]` attribute to extract the string values and the
  // vk_value Expects the format:
  // `("string", VK_VALUE)`
  // or
  // `("string" | "list", VK_VALUE)`
  let conversions: VariantAttrs = attr.parse_args()?;

  Ok(Key {
    ident,
    attrs: conversions,
  })
}

/// This macro derives the `KeyConversions` trait for an enum.
pub fn key_conversions(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  let name = &input.ident;

  // Find the `#[key]` attribute on the enum itself. This is required to
  // pass in the absolute paths for each platform's key codes.
  let enum_attr = match find_key_attr(&input.attrs) {
    Some(attr) => attr,
    None => {
      let error = syn::Error::new_spanned(
        name,
        "Missing `#[key]` attribute on the enum itself. It should be annotated like this:\n```rs\n#[key(win_prefix = <key::codes::prefix>, macos_prefix = <key::codes::prefix>)]\nenum // ...\n```",
      )
      .to_compile_error();
      let default_impls = default_fn_impls(name);
      return quote! {
        #error
        #default_impls
      }
      .into();
    }
  };

  // Parse the `#[key(...)]` attribute on the enum to extract the prefixes
  let enum_attrs = match enum_attr.parse_args::<attrs::enums::EnumAttr>() {
    Ok(attrs) => attrs,
    Err(e) => {
      // Forward the error to the outputed token stream as a compile error.
      // Include default function impls so that dependant code does
      // not error out.
      let error = e.into_compile_error();
      let default_impls = default_fn_impls(name);
      return quote! {
        #error
        #default_impls
      }
      .into();
    }
  };

  // Error out if the input is not an enum
  let enum_data = match &input.data {
    syn::Data::Enum(data) => data,
    _ => {
      let error = syn::Error::new_spanned(
        name,
        "This macro can only be used on enums. Please annotate an enum with `#[key(...)]` and derive the `KeyConversions` trait.",
      ).into_compile_error();
      let default_impls = default_fn_impls(name);
      return quote! {
        #error
        #default_impls
      }
      .into();
    }
  };

  // Iterate over the enum variants and convert them into `Key` structs.
  let (variants, errors): (Vec<_>, Vec<_>) = enum_data
    .variants
    .iter()
    .map(variant_to_key)
    // Partition the results into successful variants and errors.
    .partition(|key| !key.is_err());

  let keys: Vec<_> = variants
    .into_iter()
    // Saftey: Just partitioned the results, so this unwrap is safe.
    .map(|var| unsafe { var.unwrap_unchecked() })
    .collect();

  // Find any duplicate keys in the parsed keys.
  // Will give an error if a key is defined multiple times.
  let duplicate_errors = find_duplicate_keys(&keys);

  let errors: Vec<_> = errors
    .into_iter()
    // Saftey: Just partitioned the results, so this unwrap is safe.
    .map(|err| unsafe { err.unwrap_err_unchecked() })
    .chain(duplicate_errors)
    // Convert the errors into a token stream to include in the output.
    // This is what gives accuratly spanned error messages in the macro
    // input.
    .map(|err| err.into_compile_error())
    .collect();

  // Make each function impl
  let from_str_impl = from_str::make_from_str_impl(&keys);
  let from_vk_impl = from_vk::make_from_vk_impl(&keys, &enum_attrs);
  let into_vk_impl = into_vk::make_into_vk_impl(&keys, &enum_attrs);

  // Create the output token stream
  // Errors are unpacked into individual spanned compile errors
  let expanded = quote! {
      impl #name {
        #from_str_impl

        #from_vk_impl

        #into_vk_impl
      }

      #(#errors)*
  };

  TokenStream::from(expanded)
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
    if let VariantAttrs::Key(vk_value) = &key.attrs {
      if let VariantVkValue::Key(vk_value) = &vk_value.win_key {
        if !win_seen.insert(vk_value.clone()) {
          // If the key is already seen, we have a duplicate
          duplicates.push(syn::Error::new_spanned(
            vk_value,
            format!("Duplicate key value: {:?}. Wrap virtual keys with Vert, eg. The virtual key `Win` should use `Vert(VK_LWIN)` instead of just `VK_LWIN`", vk_value),
          ));
        }
      }

      if let VariantVkValue::Key(vk_value) = &vk_value.macos_key {
        if !macos_seen.insert(vk_value.clone()) {
          // If the key is already seen, we have a duplicate
          duplicates.push(syn::Error::new_spanned(
            vk_value,
            format!("Duplicate key value: {:?}. Wrap virtual keys with Vert, eg. The virtual key `Win` should use `Vert(VK_LWIN)` instead of just `VK_LWIN`", vk_value),
          ));
        }
      }
    }
  }

  duplicates
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
