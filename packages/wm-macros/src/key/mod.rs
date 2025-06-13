use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Token, parse_macro_input};

mod from_str;
mod from_vk;
mod into_vk;

struct Key {
  ident: syn::Ident,
  str_values: Vec<String>,
  vk_value: syn::Ident,
}

fn variant_to_key(variant: &syn::Variant) -> syn::Result<Key> {
  let ident = variant.ident.clone();
  let attrs = &variant.attrs;

  let attr = match attrs
    .iter()
    .filter_map(|attr| attr.meta.require_list().ok())
    .find(|list| {
      if let Ok(ident) = list.path.require_ident() {
        *ident == "key"
      } else {
        false
      }
    }) {
    Some(attr) => attr,
    None => {
      if ident == "Custom" {
        // Custom variant is allowed to not have the key attribute
        return Ok(Key {
          ident: ident.clone(),
          str_values: Vec::new(),
          vk_value: syn::Ident::new("Custom", ident.span()),
        });
      } else {
        return Err(syn::Error::new_spanned(
          &variant.ident,
          "Missing `#[key(\"value\", VK_VAlue)]` attribute",
        ));
      }
    }
  };

  let get_string_variants = |string: String| -> syn::Result<Vec<String>> {
    if string.is_empty() {
      return Err(syn::Error::new_spanned(
        attr,
        "String value cannot be empty",
      ));
    }

    if string.contains('+') {
      return Err(syn::Error::new_spanned(
        attr,
        "String value cannot contain '+'",
      ));
    }

    if string.chars().last().unwrap() == ' ' {
      return Err(syn::Error::new_spanned(
        attr,
        "String value should not end with a space",
      ));
    }

    if !string.contains(' ') {
      return Ok(vec![string]);
    }

    let underscored = string.replace(' ', "_");

    let dashed = string.replace(' ', "-");

    let camel_cased = string.split(' ').fold(String::new(), |acc, el| {
      if acc.is_empty() {
        return el.to_string();
      }
      let mut chars = el.chars();
      let first_char = chars.next().unwrap();
      let rest = chars.as_str();

      let first_char = first_char.to_uppercase().to_string();

      let mut new_el = acc.to_string();
      new_el.push_str(&first_char);
      new_el.push_str(rest);

      new_el
    });

    let variants = vec![string, underscored, dashed, camel_cased];

    Ok(variants)
  };

  let parse_str =
    |input: syn::parse::ParseStream| -> syn::Result<Vec<String>> {
      let string = input.parse::<syn::LitStr>()?;
      get_string_variants(string.value())
    };

  let parse_strs =
    |input: syn::parse::ParseStream| -> syn::Result<Vec<String>> {
      let mut values = Vec::new();
      values.extend(parse_str(input)?);

      while input.peek(Token![|]) {
        input.parse::<Token![|]>()?;
        values.extend(parse_str(input)?);
      }

      Ok(values)
    };

  struct Conversions {
    str_values: Vec<String>,
    vk_value: syn::Ident,
  }

  let conversions: Conversions = attr.parse_args_with(
    |input: syn::parse::ParseStream| -> syn::Result<Conversions> {
      let str_values = parse_strs(input)?;

      // Error if no comma, but discard it if present
      _ = input.parse::<Token![,]>()?;

      let vk_value = input.parse::<syn::Ident>().map_err(|_| {
        syn::Error::new(
          input.span(),
          "Expected an identifier for VK value",
        )
      })?;

      Ok(Conversions {
        str_values,
        vk_value,
      })
    },
  )?;

  Ok(Key {
    ident,
    str_values: conversions.str_values,
    vk_value: conversions.vk_value,
  })
}

pub fn key_conversions(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  let name = &input.ident;

  let enum_data = match &input.data {
    syn::Data::Enum(data) => data,
    _ => {
      return quote! {
        compile_error!("KeyConversions can only be derived for enums");
      }
      .into();
    }
  };

  let (variants, errors): (Vec<_>, Vec<_>) = enum_data
    .variants
    .iter()
    .map(variant_to_key)
    .partition(|key| !key.is_err());

  let keys: Vec<_> = variants
    .into_iter()
    .map(|var| unsafe { var.unwrap_unchecked() })
    .collect();

  let errors: Vec<_> = errors
    .into_iter()
    .map(|err| unsafe { err.unwrap_err_unchecked() })
    .map(|err| err.into_compile_error())
    .collect();

  let from_str_impl = from_str::make_from_str_impl(&keys);
  let from_vk_impl = from_vk::make_from_vk_impl(&keys);
  let into_vk_impl = into_vk::make_into_vk_impl(&keys);

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
