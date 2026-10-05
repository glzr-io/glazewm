//! Self-update support backed by GitHub releases.

use std::{
  path::{Path, PathBuf},
  sync::atomic::{AtomicBool, Ordering},
  time::Duration,
};

use anyhow::Context;
use semver::Version;
use tokio::sync::mpsc;
use ureq::ResponseExt;
use wm_platform::Dispatcher;

/// GitHub repository (in `owner/repo` format) that releases are pulled
/// from.
///
/// Overridable at build time via the `UPDATE_REPO` environment variable.
const UPDATE_REPO: &str = match option_env!("UPDATE_REPO") {
  Some(repo) => repo,
  None => "hairbui76/glazewm",
};

/// Timeout for the requests that resolve the latest release.
const METADATA_TIMEOUT: Duration = Duration::from_secs(30);

/// Timeout for downloading the installer.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_mins(10);

/// Maximum time that the deferred installer waits for the WM process to
/// exit before giving up.
const EXIT_WAIT_SECS: u32 = 60;

/// Registry key under which the installer records its install directory.
const INSTALL_DIR_KEY: &str = r"HKLM:\SOFTWARE\glzr.io\GlazeWM";

/// Whether an update check is currently running.
///
/// Update checks are triggered from the system tray, which allows the menu
/// item to be clicked repeatedly. Only one check is allowed at a time.
static UPDATE_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// Latest release available on GitHub.
#[derive(Debug)]
struct LatestRelease {
  tag_name: String,
  version: Version,
  installer_name: String,
  installer_url: String,
}

/// Checks for a newer release in the background and, if the user agrees,
/// downloads and installs it.
///
/// Returns immediately. The check runs on a dedicated thread so that the
/// system tray remains responsive while the network request and download
/// are in-flight. Failures are surfaced to the user via an error dialog.
pub fn check_for_updates(
  dispatcher: &Dispatcher,
  exit_tx: &mpsc::UnboundedSender<()>,
) {
  if UPDATE_IN_PROGRESS
    .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
    .is_err()
  {
    tracing::info!("An update check is already in progress.");
    return;
  }

  let dispatcher = dispatcher.clone();
  let exit_tx = exit_tx.clone();

  std::thread::spawn(move || {
    if let Err(err) = run_update_flow(&dispatcher, &exit_tx) {
      tracing::error!("Update check failed: {:?}", err);
      dispatcher.show_error_dialog(
        "Unable to update GlazeWM",
        &format!("{err:#}"),
      );
    }

    UPDATE_IN_PROGRESS.store(false, Ordering::SeqCst);
  });
}

/// Runs the full update flow: resolve the latest release, prompt the user,
/// download the installer, and schedule it to run after exit.
fn run_update_flow(
  dispatcher: &Dispatcher,
  exit_tx: &mpsc::UnboundedSender<()>,
) -> anyhow::Result<()> {
  let current_version = current_version()?;
  let release = fetch_latest_release()?;

  tracing::info!(
    "Latest release is v{}, current version is v{current_version}.",
    release.version
  );

  if release.version <= current_version {
    dispatcher.show_info_dialog(
      "GlazeWM is up to date",
      &format!("You're running the latest version (v{current_version})."),
    );

    return Ok(());
  }

  ensure_installer_available(&release)?;

  let should_update = dispatcher.show_confirm_dialog(
    "Update available",
    &format!(
      "GlazeWM v{} is available. You're currently on v{current_version}.\n\n\
       Install it now? The installer is downloaded in the background, \
       which takes a moment with no window of its own. GlazeWM then exits, \
       the installer asks for administrator rights, and the new version is \
       started once it finishes.",
      release.version
    ),
  );

  if !should_update {
    tracing::info!("Update to v{} declined by user.", release.version);
    return Ok(());
  }

  let installer_path = download_installer(&release)?;
  run_installer_after_exit(&installer_path)?;

  tracing::info!("Exiting to install v{}.", release.version);
  exit_tx
    .send(())
    .context("Failed to signal exit for the pending update.")?;

  Ok(())
}

/// Parses the version that the application was built with.
fn current_version() -> anyhow::Result<Version> {
  Version::parse(env!("VERSION_NUMBER")).with_context(|| {
    format!("Invalid current version '{}'.", env!("VERSION_NUMBER"))
  })
}

/// Resolves the latest release from the redirect that GitHub serves for
/// a repository's `releases/latest` page.
///
/// The REST API is deliberately avoided. Unauthenticated API requests are
/// limited to 60 an hour per IP address, and addresses are routinely
/// shared between many people (by an ISP or an office network), so the
/// limit is often spent before the first request is made. The release
/// pages are not subject to it.
///
/// Draft and pre-releases are never the target of that redirect.
fn fetch_latest_release() -> anyhow::Result<LatestRelease> {
  let url = format!("https://github.com/{UPDATE_REPO}/releases/latest");

  let response = match http_agent(METADATA_TIMEOUT).head(&url).call() {
    Ok(response) => response,
    Err(ureq::Error::StatusCode(404)) => {
      anyhow::bail!("'{UPDATE_REPO}' has no published release.")
    }
    Err(err) => {
      return Err(err).with_context(|| format!("Failed to query '{url}'."))
    }
  };

  let final_url = response.get_uri().to_string();

  let tag_name = release_tag_from_url(&final_url).with_context(|| {
    format!("Unable to tell the latest release from '{final_url}'.")
  })?;

  let installer_name = installer_name(&tag_name);

  Ok(LatestRelease {
    version: parse_release_version(&tag_name)?,
    installer_url: format!(
      "https://github.com/{UPDATE_REPO}/releases/download/{tag_name}/{installer_name}"
    ),
    installer_name,
    tag_name,
  })
}

/// Checks that the release's installer can be downloaded.
///
/// Returns an error if it isn't attached to the release (yet), which
/// happens in the window between a release being published and the build
/// that produces its installers finishing.
fn ensure_installer_available(
  release: &LatestRelease,
) -> anyhow::Result<()> {
  match http_agent(METADATA_TIMEOUT)
    .head(&release.installer_url)
    .call()
  {
    Ok(_) => Ok(()),
    Err(ureq::Error::StatusCode(404)) => anyhow::bail!(
      "Release {} has no Windows installer attached yet. Try again in a \
       few minutes.",
      release.tag_name
    ),
    Err(err) => Err(err).with_context(|| {
      format!("Failed to query '{}'.", release.installer_url)
    }),
  }
}

/// Downloads the release's installer into a temporary directory.
///
/// The download is written to a `.part` file that is only renamed on
/// success, so that an interrupted download is never mistaken for a
/// complete installer.
///
/// Returns the path of the downloaded installer.
fn download_installer(release: &LatestRelease) -> anyhow::Result<PathBuf> {
  let download_dir = std::env::temp_dir().join("glazewm-update");
  std::fs::create_dir_all(&download_dir).with_context(|| {
    format!("Failed to create '{}'.", download_dir.display())
  })?;

  let installer_path = download_dir.join(&release.installer_name);
  let partial_path = installer_path.with_extension("part");

  tracing::info!(
    "Downloading '{}' to '{}'.",
    release.installer_url,
    installer_path.display()
  );

  let mut reader = http_agent(DOWNLOAD_TIMEOUT)
    .get(&release.installer_url)
    .call()
    .with_context(|| {
      format!("Failed to download '{}'.", release.installer_url)
    })?
    .into_body()
    .into_reader();

  {
    let mut file =
      std::fs::File::create(&partial_path).with_context(|| {
        format!("Failed to create '{}'.", partial_path.display())
      })?;

    std::io::copy(&mut reader, &mut file)
      .context("Failed to write the downloaded installer.")?;
  }

  std::fs::rename(&partial_path, &installer_path).with_context(|| {
    format!("Failed to finalize '{}'.", installer_path.display())
  })?;

  Ok(installer_path)
}

/// Schedules the installer to run once the current process has exited.
///
/// The installer overwrites `glazewm.exe`, so it cannot run while the WM
/// is still alive. A detached PowerShell process is spawned to poll for
/// the WM's exit, run the installer, and relaunch `GlazeWM` afterwards.
///
/// The spawned process is created without a console window so that it
/// stays invisible to the user.
fn run_installer_after_exit(installer_path: &Path) -> anyhow::Result<()> {
  use std::os::windows::process::CommandExt;

  /// Creation flag that prevents a console window from being allocated.
  ///
  /// Ref: <https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags>
  const CREATE_NO_WINDOW: u32 = 0x0800_0000;

  let exe_path = std::env::current_exe()
    .context("Failed to resolve the current executable path.")?;

  let script =
    deferred_install_script(std::process::id(), installer_path, &exe_path);

  std::process::Command::new("powershell")
    .args([
      "-NoProfile",
      "-NonInteractive",
      "-WindowStyle",
      "Hidden",
      "-Command",
      &script,
    ])
    .creation_flags(CREATE_NO_WINDOW)
    .spawn()
    .context("Failed to spawn the deferred installer process.")?;

  Ok(())
}

/// Builds the `PowerShell` script that waits for the WM process to exit,
/// runs the installer, and relaunches the WM.
///
/// The installer is run in passive mode so that the user sees its progress
/// without having to click through it. Exit code `3010` means the install
/// succeeded but wants a reboot, so it counts as success.
///
/// The newly installed executable is relaunched rather than the one that
/// was running. The two differ whenever the WM was started from outside
/// the install directory, such as from a local build, in which case
/// relaunching the running executable would leave the user on the old
/// version with no sign that anything happened. `exe_path` is only used as
/// a fallback for when the install directory cannot be read back.
fn deferred_install_script(
  pid: u32,
  installer_path: &Path,
  exe_path: &Path,
) -> String {
  let installer = escape_ps_literal(&installer_path.to_string_lossy());
  let fallback_exe = escape_ps_literal(&exe_path.to_string_lossy());

  // Built as separate statements rather than one long literal so that each
  // step of the script stays readable.
  [
    format!("$deadline = (Get-Date).AddSeconds({EXIT_WAIT_SECS});"),
    format!(
      "while ((Get-Process -Id {pid} -ErrorAction SilentlyContinue) -and ((Get-Date) -lt $deadline)) {{ Start-Sleep -Milliseconds 200 }};"
    ),
    format!(
      "if (Get-Process -Id {pid} -ErrorAction SilentlyContinue) {{ exit 1 }};"
    ),
    format!(
      "$proc = Start-Process -FilePath '{installer}' -ArgumentList '/passive','/norestart' -PassThru -Wait;"
    ),
    "if (($proc.ExitCode -ne 0) -and ($proc.ExitCode -ne 3010)) { exit $proc.ExitCode };".to_string(),
    format!("$exe = '{fallback_exe}';"),
    format!(
      "$dir = (Get-ItemProperty '{INSTALL_DIR_KEY}' -ErrorAction SilentlyContinue).InstallDir;"
    ),
    "if ($dir) { $installed = Join-Path $dir 'glazewm.exe'; if (Test-Path $installed) { $exe = $installed } };".to_string(),
    "Start-Process -FilePath $exe".to_string(),
  ]
  .join(" ")
}

/// Escapes a value for use within a single-quoted PowerShell string.
fn escape_ps_literal(value: &str) -> String {
  value.replace('\'', "''")
}

/// Creates an HTTP agent with the given global timeout.
fn http_agent(timeout: Duration) -> ureq::Agent {
  let config = ureq::Agent::config_builder()
    .user_agent(concat!("GlazeWM/", env!("VERSION_NUMBER")))
    .timeout_global(Some(timeout))
    .build();

  ureq::Agent::new_with_config(config)
}

/// Parses the semantic version out of a release tag (e.g. `v1.2.3`).
fn parse_release_version(tag_name: &str) -> anyhow::Result<Version> {
  Version::parse(tag_name.trim_start_matches('v'))
    .with_context(|| format!("Invalid release tag '{tag_name}'."))
}

/// Extracts the release tag from the URL of a release page, such as
/// `https://github.com/owner/repo/releases/tag/v1.2.3`.
///
/// Returns `None` for any other URL, which is where the `releases/latest`
/// redirect ends up when a repository has no release to point at.
fn release_tag_from_url(url: &str) -> Option<String> {
  let (_, tag_name) = url.split_once("/releases/tag/")?;

  // Drop anything trailing the tag, such as a query string.
  let tag_name = tag_name
    .split(['/', '?', '#'])
    .next()
    .filter(|tag_name| !tag_name.is_empty())?;

  Some(tag_name.to_string())
}

/// Name of the universal Windows installer attached to the release with
/// the given tag, as named by the release pipeline.
fn installer_name(tag_name: &str) -> String {
  format!("glazewm-{tag_name}.exe")
}

#[cfg(test)]
mod tests {
  use std::path::Path;

  use super::{
    deferred_install_script, escape_ps_literal, installer_name,
    parse_release_version, release_tag_from_url,
  };

  #[test]
  fn parses_release_version_with_and_without_prefix() {
    assert_eq!(
      parse_release_version("v3.11.0").unwrap().to_string(),
      "3.11.0"
    );
    assert_eq!(
      parse_release_version("3.11.0").unwrap().to_string(),
      "3.11.0"
    );
    assert!(parse_release_version("nightly").is_err());
  }

  #[test]
  fn reads_the_tag_from_a_release_page_url() {
    assert_eq!(
      release_tag_from_url(
        "https://github.com/hairbui76/glazewm/releases/tag/v3.15.1"
      ),
      Some("v3.15.1".to_string())
    );
  }

  #[test]
  fn ignores_anything_trailing_the_tag() {
    assert_eq!(
      release_tag_from_url(
        "https://github.com/owner/repo/releases/tag/v1.2.3?expanded=true"
      ),
      Some("v1.2.3".to_string())
    );
    assert_eq!(
      release_tag_from_url(
        "https://github.com/owner/repo/releases/tag/v1.2.3/"
      ),
      Some("v1.2.3".to_string())
    );
  }

  #[test]
  fn has_no_tag_for_a_repository_without_releases() {
    // Where `releases/latest` redirects to when there is nothing to show.
    assert_eq!(
      release_tag_from_url("https://github.com/owner/repo/releases"),
      None
    );
    assert_eq!(
      release_tag_from_url("https://github.com/owner/repo/releases/tag/"),
      None
    );
  }

  #[test]
  fn names_the_installer_after_the_release_tag() {
    assert_eq!(installer_name("v3.15.1"), "glazewm-v3.15.1.exe");
  }

  #[test]
  fn escapes_single_quotes_in_ps_literals() {
    assert_eq!(escape_ps_literal(r"C:\dev\it's"), r"C:\dev\it''s");
    assert_eq!(escape_ps_literal(r"C:\dev"), r"C:\dev");
  }

  #[test]
  fn deferred_script_quotes_paths_and_waits_on_pid() {
    let script = deferred_install_script(
      1234,
      Path::new(r"C:\temp\o'brien\glazewm-v3.11.0.exe"),
      Path::new(r"C:\Program Files\glzr.io\glazewm.exe"),
    );

    assert!(script.contains("-Id 1234"));
    assert!(
      script.contains(r"'C:\temp\o''brien\glazewm-v3.11.0.exe'"),
      "installer path should be single-quoted and escaped"
    );
    assert!(
      script.contains(r"'C:\Program Files\glzr.io\glazewm.exe'"),
      "fallback executable path should be single-quoted"
    );
  }

  #[test]
  fn deferred_script_relaunches_the_installed_executable() {
    let script = deferred_install_script(
      1234,
      Path::new(r"C:\temp\glazewm-v3.11.0.exe"),
      Path::new(r"D:\dev\glazewm\target\release\glazewm.exe"),
    );

    assert!(
      script
        .contains(r"(Get-ItemProperty 'HKLM:\SOFTWARE\glzr.io\GlazeWM'"),
      "install directory should be read back from the registry"
    );
    assert!(
      script.contains("Join-Path $dir 'glazewm.exe'"),
      "the installed executable should be preferred over the running one"
    );
    assert!(
      script.trim_end().ends_with("Start-Process -FilePath $exe"),
      "the resolved executable should be the one relaunched"
    );
  }
}
