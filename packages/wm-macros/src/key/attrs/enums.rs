use syn::{
  parse::{ParseStream, discouraged::Speculative},
  spanned::Spanned,
};

/// Holds the attributes for an enum that contains platform-specific
/// prefixes
pub struct EnumAttr {
  pub win_prefix: syn::Path,
  pub macos_prefix: syn::Path,
}

/// Windows-specific prefix attribute. Used only to implement
/// syn::parse::Parse
struct WindowsPrefix {
  pub prefix: syn::Path,
}

impl syn::parse::Parse for WindowsPrefix {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    // Check for an ident.
    let ident = input.parse::<syn::Ident>()?;

    // Ensure that the ident is `win_prefix`
    if ident != "win_prefix" {
      return Err(syn::Error::new(
        ident.span(),
        "Expected `win_prefix` identifier",
      ));
    }

    // Check for the `=` token
    _ = input.parse::<syn::Token![=]>().map_err(|err| {
      syn::Error::new(err.span(), "Expected `=` after `win_prefix`")
    })?;

    // Parse the path that follows the `=`
    let prefix = input.parse::<syn::Path>().map_err(|err| {
      syn::Error::new(
        err.span(),
        "Expected a namespace path for the windows prefix",
      )
    })?;

    Ok(WindowsPrefix { prefix })
  }
}

/// MacOS-specific prefix attribute. Used only to implement
/// syn::parse::Parse
struct MacOSPrefix {
  pub prefix: syn::Path,
}

impl syn::parse::Parse for MacOSPrefix {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    // Check for an ident.
    let ident = input.parse::<syn::Ident>().map_err(|err| {
      syn::Error::new(err.span(), "Expected macos_prefix")
    })?;
    // Ensure that the ident is `macos_prefix`
    if ident != "macos_prefix" {
      return Err(syn::Error::new(
        ident.span(),
        "Expected `macos_prefix` identifier",
      ));
    }

    // Check for the `=` token
    _ = input.parse::<syn::Token![=]>().map_err(|err| {
      syn::Error::new(err.span(), "Expected `=` after `macos_prefix`")
    })?;

    // Get the path that follows the `=`
    let prefix = input.parse::<syn::Path>()?;
    Ok(MacOSPrefix { prefix })
  }
}

/// Enum that can hold either a Windows or MacOS prefix. Used to
/// implement syn::parse::Parse to parse either prefix type.
enum AnyPrefix {
  Windows(WindowsPrefix),
  MacOS(MacOSPrefix),
}

impl syn::parse::Parse for AnyPrefix {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    // Fork the input so that parsing either prefix does not advance the
    // main input. Advance the main input only after a successful parse of
    // the fork.
    let win_fork = input.fork();
    let mac_fork = input.fork();
    if let Ok(prefix) = win_fork.parse::<WindowsPrefix>() {
      input.advance_to(&win_fork);
      Ok(AnyPrefix::Windows(prefix))
    } else if let Ok(prefix) = mac_fork.parse::<MacOSPrefix>() {
      input.advance_to(&mac_fork);
      Ok(AnyPrefix::MacOS(prefix))
    } else {
      // Got neither, error out at the original input
      Err(syn::Error::new(
        input.span(),
        "Expected either `win_prefix` or `macos_prefix`",
      ))
    }
  }
}

/// Parse the `#[key(...)]` attribute on the enum to extract the prefixes.
impl syn::parse::Parse for EnumAttr {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let mut win_prefix = None;
    let mut macos_prefix = None;

    let mut is_first_loop = true;

    while !input.is_empty() {
      // If we have already parsed an item, we need to check for a comma
      if !is_first_loop {
        _ = input.parse::<syn::Token![,]>()?;
      } else {
        is_first_loop = false;
      }
      // Try to parse any prefix type
      let prefix = if let Ok(prefix) = input.parse::<AnyPrefix>() {
        prefix
      } else {
        // If we cannot parse a prefix, we break the loop
        break;
      };
      // Set the appropriate prefix based on its type
      // Also check for duplicates
      match prefix {
        AnyPrefix::Windows(wp) => {
          if win_prefix.is_some() {
            return Err(syn::Error::new(
              wp.prefix.span(),
              "Duplicate `win_prefix` attribute",
            ));
          }
          win_prefix = Some(wp.prefix);
        }
        AnyPrefix::MacOS(mp) => {
          if macos_prefix.is_some() {
            return Err(syn::Error::new(
              mp.prefix.span(),
              "Duplicate `macos_prefix` attribute",
            ));
          }
          macos_prefix = Some(mp.prefix);
        }
      }
    }

    // Ensure that both prefixes are present
    let win_prefix = win_prefix.ok_or_else(|| {
      syn::Error::new(input.span(), "Missing `win_prefix` attribute")
    })?;

    let macos_prefix = macos_prefix.ok_or_else(|| {
      syn::Error::new(input.span(), "Missing `macos_prefix` attribute")
    })?;

    Ok(EnumAttr {
      win_prefix,
      macos_prefix,
    })
  }
}
