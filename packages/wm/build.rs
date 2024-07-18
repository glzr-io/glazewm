fn main() {
  let mut res = tauri_winres::WindowsResource::new();

  // When the `ui_access` feature is enabled, the `uiAccess` attribute is
  // set to `true`. UIAccess is disabled by default because it requires the
  // application to be signed and installed in a secure location.
  let ui_access = {
    #[cfg(feature = "ui_access")]
    {
      "true"
    }
    #[cfg(not(feature = "ui_access"))]
    {
      "false"
    }
  };

  // Conditionally enable UIAccess, which grants privilege to set the
  // foreground window and to set the position of elevated windows.
  //
  // Ref: https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/security-policy-settings/user-account-control-only-elevate-uiaccess-applications-that-are-installed-in-secure-locations
  //
  // Additionally, declare support for per-monitor DPI awareness.
  let manifest_str = format!(
    r#"
<assembly
  xmlns="urn:schemas-microsoft-com:asm.v1"
  manifestVersion="1.0"
  xmlns:asmv3="urn:schemas-microsoft-com:asm.v3"
>
  <asmv3:trustInfo>
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="{}" />
      </requestedPrivileges>
    </security>
  </asmv3:trustInfo>

  <asmv3:application>
    <windowsSettings
      xmlns:ws2005="http://schemas.microsoft.com/SMI/2005/WindowsSettings"
      xmlns:ws2016="http://schemas.microsoft.com/SMI/2016/WindowsSettings"
    >
      <ws2005:dpiAware>true</ws2005:dpiAware>
      <ws2016:dpiAwareness>PerMonitorV2</ws2016:dpiAwareness>
    </windowsSettings>
  </asmv3:application>
</assembly>
"#,
    ui_access
  );

  res.set_manifest(&manifest_str);
  res.set_icon("../../resources/icon.ico");

  // Set language to English (US).
  res.set_language(0x0409);

  res.set("OriginalFilename", "glazewm.exe");
  res.set("ProductName", "GlazeWM");
  res.set("FileDescription", "GlazeWM");
  res.set("FileVersion", "0.0.0");
  res.set("ProductVersion", "0.0.0");

  res.compile().unwrap();
}
