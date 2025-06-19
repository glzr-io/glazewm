use syn::spanned::Spanned;

use super::{PathList, kw};
use crate::{
  common::{
    branch::{Optional, Ordered, Unordered},
    parenthesized::Parenthesized,
  },
  prelude::*,
};

// Name-value pairs for derives and delegates with format `name =
// value`
type DerivesNameValue =
  Ordered<(kw::derives, Parenthesized<PathList>), syn::Token![=]>;
type DelegatesNameValue =
  Ordered<(kw::delegates, Parenthesized<PathList>), syn::Token![=]>;

// Parse type for an unordered list of `derives = (<ident list>)` and
// `delegates = (<ident list>)`, which are separated by commas, and each
// item is optional.
#[derive(Debug, Clone, Default)]
struct DerivesAndDelegates {
  derives: Vec<syn::Path>,
  delegates: Vec<syn::Path>,
}

impl syn::parse::Parse for DerivesAndDelegates {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let Unordered {
      items: (derives, delegates),
      ..
    } = input.parse::<Unordered<
      (Optional<DerivesNameValue>, Optional<DelegatesNameValue>),
      syn::Token![,],
    >>()?;

    let derives = derives
      .to_opt()
      .map(|d| d.items.1.into_inner())
      .unwrap_or_default();

    let delegates = delegates
      .to_opt()
      .map_or_else(|| PathList(vec![]), |d| d.items.1.into_inner());

    Ok(DerivesAndDelegates {
      derives: derives.0,
      delegates: delegates.0,
    })
  }
}

struct SubEnumAttributeMeta {
  pub attr: SubEnumAttribute,
  pub span: proc_macro2::Span,
  pub docs: Vec<syn::LitStr>,
}

enum SubEnumAttribute {
  Defaults(DerivesAndDelegates),
  SubEnum(SubEnumDeclaration),
  Doc(syn::LitStr),
}

impl syn::parse::Parse for SubEnumAttribute {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    if input.peek(kw::defaults) {
      let defaults_meta = input.parse::<syn::MetaList>()?;
      let defaults = defaults_meta.parse_args::<DerivesAndDelegates>()?;
      Ok(SubEnumAttribute::Defaults(defaults))
    } else if input.peek(kw::doc) {
      input
        .parse::<Ordered<(kw::doc, syn::LitStr), syn::Token![=]>>()
        .map(
          |Ordered {
             items: (_, doc), ..
           }| SubEnumAttribute::Doc(doc),
        )
    } else {
      let sub_enum = input.parse::<SubEnumDeclaration>()?;
      Ok(SubEnumAttribute::SubEnum(sub_enum))
    }
  }
}

#[derive(Debug, Clone)]
pub struct SubEnumDeclaration {
  pub name: syn::Ident,
  pub derives: Vec<syn::Path>,
  pub delegates: Vec<syn::Path>,
  pub docs: Vec<syn::LitStr>,
}

impl syn::parse::Parse for SubEnumDeclaration {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    // Parse type for an idenfifier, followed by `NameValues`
    type Attr = Ordered<(syn::Ident, DerivesAndDelegates), syn::Token![,]>;

    let Ordered {
      items: (name, DerivesAndDelegates { derives, delegates }),
      ..
    } = input.parse::<Attr>()?;

    Ok(SubEnumDeclaration {
      name,
      derives,
      delegates,
      docs: vec![],
    })
  }
}

fn parse_enum_attrs<'a>(
  attrs: impl Iterator<Item = &'a syn::Meta>,
) -> syn::Result<Vec<SubEnumAttributeMeta>> {
  // Iterate over the enum attributes, storing all consecutive subenum doc
  // attributes for the next sub enum declaration.
  let mut docs = vec![];
  attrs
    .filter_map(|attr| match attr {
      syn::Meta::List(attr) => {
        if let Some(ident) = attr.path.get_ident() {
          if ident == crate::subenum::SUBENUM_ATTR_NAME {
            let parsed = attr.parse_args::<SubEnumAttribute>();
            let res = match parsed {
              Ok(parsed) => match &parsed {
                SubEnumAttribute::Doc(doc) => {
                  docs.push(doc.clone());
                  None
                }
                SubEnumAttribute::Defaults(_)
                | SubEnumAttribute::SubEnum(_) => {
                  let meta = Some(Ok(SubEnumAttributeMeta {
                    attr: parsed,
                    span: attr.span(),
                    docs: docs.clone(),
                  }));
                  docs.clear();
                  meta
                }
              },
              Err(e) => Some(Err(e)),
            };
            return res;
          }
        }
        docs.clear();
        None
      }
      _ => {
        docs.clear();
        None
      }
    })
    .collect()
}

pub fn collect_sub_enums<'a>(
  attrs: impl Iterator<Item = &'a syn::Meta>,
) -> syn::Result<Vec<SubEnumDeclaration>> {
  let mut parsed_attrs = parse_enum_attrs(attrs)?;

  let mut default_attr_spans = vec![];

  // Combine all Defaults attributes into one
  let defaults = parsed_attrs
    .iter_mut()
    .filter_map(|attr| match &mut attr.attr {
      SubEnumAttribute::Defaults(defaults) => {
        default_attr_spans.push(attr.span);
        Some(defaults)
      }
      _ => None,
    })
    .fold(DerivesAndDelegates::default(), |mut acc, el| {
      acc.derives.append(&mut el.derives);
      acc.delegates.append(&mut el.delegates);
      acc
    });

  if default_attr_spans.len() > 1 {
    let span = default_attr_spans
      .into_iter()
      // Saftey: a and b will always be from the same file
      .reduce(|a, b| a.join(b).unwrap())
      .unwrap();
    span.emit_note("Multiple `defaults` attributes found. Consider combining them into one.");
  }

  // Find derives and delegates that are in all enum declarations but not
  // in defaults

  let inner_enums = parsed_attrs
    .iter()
    .filter_map(|attr| match &attr.attr {
      SubEnumAttribute::SubEnum(sub_enum) => Some(sub_enum),
      _ => None,
    })
    .collect::<Vec<_>>();

  let mut all_derives = inner_enums
    .iter()
    .flat_map(|e| &e.derives)
    .collect::<Vec<_>>();
  all_derives.dedup();
  let mut all_delegates = inner_enums
    .iter()
    .flat_map(|e| &e.delegates)
    .collect::<Vec<_>>();
  all_delegates.dedup();

  for derive in &all_derives {
    if inner_enums.iter().all(|e| e.derives.contains(derive)) {
      // Get all the spans of the derive
      inner_enums.iter().filter_map(|e| {
        e.derives.iter().find(|d| d == derive).map(|d| d.span())
      }).for_each(|span| {
        let derive = derive.segments.iter().map(|seg| seg.ident.to_string()).reduce(|acc, el| format!("{acc}::{el}").to_string()).unwrap_or_else(|| "unknown".to_string());
        span.emit_help(format!("All sub-enums have the derive `{derive}` but it is not in the `defaults` attribute. Consider adding it to the `defaults` attribute."));
      });
    }
  }

  for delegate in &all_delegates {
    if inner_enums.iter().all(|e| e.delegates.contains(delegate)) {
      // Get all the spans of the delegate
      inner_enums.iter().filter_map(|e| {
        e.delegates
          .iter()
          .find(|d| d == delegate)
          .map(|d| d.span())
      }).for_each(|span| {
        let delegate = delegate.segments.iter().map(|seg| seg.ident.to_string()).reduce(|acc, el| format!("{acc}::{el}").to_string()).unwrap_or_else(|| "unknown".to_string());
        span.emit_help(format!("All sub-enums have the delegate `{delegate}` but it is not in the `defaults` attribute. Consider adding it to the `defaults` attribute."));
      });
    }
  }

  Ok(
    parsed_attrs
      .into_iter()
      .filter_map(|attr| match attr.attr {
        SubEnumAttribute::SubEnum(mut sub_enum) => {
          sub_enum.derives.extend(defaults.derives.iter().cloned());
          sub_enum
            .delegates
            .extend(defaults.delegates.iter().cloned());
          sub_enum.docs.extend(attr.docs);
          Some(sub_enum)
        }
        _ => None,
      })
      .collect::<Vec<_>>(),
  )
}
