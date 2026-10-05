use std::path::{Path, PathBuf};

use windows::{
  core::{w, BSTR, HSTRING, PCWSTR},
  Win32::{
    Foundation::ERROR_FILE_NOT_FOUND,
    System::{
      Com::{CoCreateInstance, CLSCTX_INPROC_SERVER},
      Registry::{
        RegDeleteKeyValueW, RegGetValueW, HKEY_CURRENT_USER,
        RRF_RT_REG_BINARY, RRF_RT_REG_SZ,
      },
      TaskScheduler::{
        ITaskFolder, ITaskService, TaskScheduler, TASK_CREATE_OR_UPDATE,
        TASK_LOGON_INTERACTIVE_TOKEN,
      },
      Variant::VARIANT,
    },
  },
};

use super::com::COM_INIT;

/// Registry key that startup applications were conventionally listed
/// under, relative to `HKEY_CURRENT_USER`.
const RUN_KEY: PCWSTR =
  w!(r"Software\Microsoft\Windows\CurrentVersion\Run");

/// Registry key holding whether the user has disabled an entry of
/// [`RUN_KEY`] (e.g. through Task Manager).
const RUN_APPROVED_KEY: PCWSTR = w!(
  r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run"
);

/// Windows implementation of [`crate::Autostart`], backed by a scheduled
/// task that runs when the user signs in.
///
/// A scheduled task is used instead of the `Run` registry key because
/// Explorer only works through that key well after sign-in, and one entry
/// at a time. A task with a logon trigger is started by the Task Scheduler
/// service within seconds instead.
pub(crate) struct Autostart {
  /// Name the application registered under [`RUN_KEY`] with, before
  /// scheduled tasks were used.
  app_name: String,

  /// Name of the scheduled task. Includes the user name, since tasks of
  /// all users share one namespace.
  task_name: String,

  /// Account that the task is triggered by and runs as.
  user_id: String,

  /// Executable that the task starts.
  exe_path: PathBuf,
}

impl Autostart {
  /// Creates an instance for the current user.
  pub(crate) fn new(
    app_name: &str,
    exe_path: &Path,
  ) -> crate::Result<Self> {
    let user_name = std::env::var("USERNAME").map_err(|_| {
      crate::Error::Platform(
        "Unable to resolve the user name.".to_string(),
      )
    })?;

    // Local accounts are qualified by the computer name, which is what
    // `USERDOMAIN` holds for them.
    let user_id = match std::env::var("USERDOMAIN") {
      Ok(domain) if !domain.is_empty() => format!("{domain}\\{user_name}"),
      _ => user_name.clone(),
    };

    Ok(Self {
      app_name: app_name.to_string(),
      task_name: task_name(app_name, &user_name),
      user_id,
      exe_path: exe_path.to_path_buf(),
    })
  }

  /// Whether the scheduled task exists and is enabled.
  pub(crate) fn is_enabled(&self) -> crate::Result<bool> {
    let folder = root_task_folder()?;

    // SAFETY: `folder` is a valid COM interface pointer, and the `BSTR`
    // outlives the call.
    match unsafe { folder.GetTask(&BSTR::from(self.task_name.as_str())) } {
      // SAFETY: `task` is a valid COM interface pointer.
      Ok(task) => Ok(unsafe { task.Enabled() }?.as_bool()),
      Err(err) if err.code() == ERROR_FILE_NOT_FOUND.to_hresult() => {
        Ok(false)
      }
      Err(err) => Err(err.into()),
    }
  }

  /// Registers the scheduled task, replacing any existing registration.
  pub(crate) fn enable(&self) -> crate::Result<()> {
    let folder = root_task_folder()?;
    let xml = task_xml(&self.user_id, &self.exe_path.to_string_lossy());

    // SAFETY: `folder` is a valid COM interface pointer, and the `BSTR`s
    // outlive the call. Empty variants leave the credentials and security
    // descriptor to be derived from the task definition.
    unsafe {
      folder.RegisterTask(
        &BSTR::from(self.task_name.as_str()),
        &BSTR::from(xml.as_str()),
        TASK_CREATE_OR_UPDATE.0,
        VARIANT::default(),
        VARIANT::default(),
        TASK_LOGON_INTERACTIVE_TOKEN,
        VARIANT::default(),
      )
    }?;

    Ok(())
  }

  /// Removes the scheduled task. Succeeds if it doesn't exist.
  pub(crate) fn disable(&self) -> crate::Result<()> {
    let folder = root_task_folder()?;

    // SAFETY: `folder` is a valid COM interface pointer, and the `BSTR`
    // outlives the call.
    match unsafe {
      folder.DeleteTask(&BSTR::from(self.task_name.as_str()), 0)
    } {
      Err(err) if err.code() != ERROR_FILE_NOT_FOUND.to_hresult() => {
        Err(err.into())
      }
      _ => Ok(()),
    }
  }

  /// Replaces a registration under the `Run` registry key, as made by
  /// earlier versions, with the scheduled task.
  ///
  /// An entry that the user had disabled is removed without registering
  /// the task, so that their choice carries over.
  ///
  /// Returns whether a `Run` entry was found.
  pub(crate) fn migrate_from_run_key(&self) -> crate::Result<bool> {
    let value_name = HSTRING::from(self.app_name.as_str());

    // SAFETY: The key and value names are valid null-terminated strings.
    // No buffers are passed, so the call only reports whether a string
    // value of that name exists.
    let has_run_entry = unsafe {
      RegGetValueW(
        HKEY_CURRENT_USER,
        RUN_KEY,
        &value_name,
        RRF_RT_REG_SZ,
        None,
        None,
        None,
      )
    }
    .is_ok();

    if !has_run_entry {
      return Ok(false);
    }

    if !is_run_entry_disabled(&value_name) {
      self.enable()?;
    }

    // SAFETY: The key and value names are valid null-terminated strings.
    unsafe {
      RegDeleteKeyValueW(HKEY_CURRENT_USER, RUN_KEY, &value_name)
    }?;

    // SAFETY: As above. The value only exists once the user has toggled
    // the entry, so failing to delete it is expected.
    let _ = unsafe {
      RegDeleteKeyValueW(HKEY_CURRENT_USER, RUN_APPROVED_KEY, &value_name)
    };

    Ok(true)
  }
}

/// Whether the user has disabled the `Run` entry of the given name.
///
/// Explorer records this as a binary value whose first byte has its lowest
/// bit set when the entry is disabled. A missing value means the entry was
/// never toggled, and so is enabled.
fn is_run_entry_disabled(value_name: &HSTRING) -> bool {
  let mut data = [0_u8; 12];
  let mut size = u32::try_from(data.len()).unwrap_or(0);

  // SAFETY: The key and value names are valid null-terminated strings,
  // and `size` holds the capacity of `data`, which outlives the call.
  let is_read = unsafe {
    RegGetValueW(
      HKEY_CURRENT_USER,
      RUN_APPROVED_KEY,
      value_name,
      RRF_RT_REG_BINARY,
      None,
      Some(data.as_mut_ptr().cast()),
      Some(&raw mut size),
    )
  }
  .is_ok();

  is_read && size > 0 && data[0] & 1 == 1
}

/// Connects to the Task Scheduler service and gets its root folder.
fn root_task_folder() -> crate::Result<ITaskFolder> {
  // Ensure COM is initialized on the current thread.
  COM_INIT.with(|_| {});

  // SAFETY: COM is initialized on this thread, and `TaskScheduler` is the
  // class identifier that implements `ITaskService`.
  let service: ITaskService = unsafe {
    CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)
  }?;

  // SAFETY: `service` is a valid COM interface pointer. Empty variants
  // connect to the local machine as the current user.
  unsafe {
    service.Connect(
      VARIANT::default(),
      VARIANT::default(),
      VARIANT::default(),
      VARIANT::default(),
    )
  }?;

  // SAFETY: `service` is a valid, connected COM interface pointer, and the
  // `BSTR` outlives the call.
  Ok(unsafe { service.GetFolder(&BSTR::from("\\")) }?)
}

/// Name of the scheduled task for the given application and user.
fn task_name(app_name: &str, user_name: &str) -> String {
  format!("{app_name} autostart for {user_name}")
}

/// Builds the definition of the scheduled task.
///
/// The settings that deviate from the Task Scheduler defaults all exist to
/// keep the application from being held back or stopped:
///
/// - `Priority` 4 is normal priority. The default of 7 starts the process
///   at below-normal CPU priority and low I/O priority, which leaves it
///   starved for disk access during sign-in.
/// - `DisallowStartIfOnBatteries` and `StopIfGoingOnBatteries` default to
///   `true`, which would skip or kill the task on a laptop on battery.
/// - `ExecutionTimeLimit` defaults to three days, after which the process
///   would be killed.
fn task_xml(user_id: &str, exe_path: &str) -> String {
  let user_id = xml_escape(user_id);
  let exe_path = xml_escape(exe_path);

  format!(
    r#"<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <Triggers>
    <LogonTrigger>
      <Enabled>true</Enabled>
      <UserId>{user_id}</UserId>
    </LogonTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <UserId>{user_id}</UserId>
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>LeastPrivilege</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <AllowHardTerminate>false</AllowHardTerminate>
    <StartWhenAvailable>false</StartWhenAvailable>
    <AllowStartOnDemand>true</AllowStartOnDemand>
    <Enabled>true</Enabled>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <Priority>4</Priority>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>"{exe_path}"</Command>
    </Exec>
  </Actions>
</Task>"#
  )
}

/// Escapes a value for use as XML character data.
fn xml_escape(value: &str) -> String {
  value
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
  use std::path::Path;

  use super::{task_name, task_xml, xml_escape, Autostart};

  #[test]
  fn task_name_is_unique_per_user() {
    assert_eq!(
      task_name("GlazeWM", "alice"),
      "GlazeWM autostart for alice"
    );
    assert_ne!(task_name("GlazeWM", "alice"), task_name("GlazeWM", "bob"));
  }

  #[test]
  fn escapes_xml_special_characters() {
    assert_eq!(
      xml_escape(r#"R&D <"tools"> 'x'"#),
      "R&amp;D &lt;&quot;tools&quot;&gt; &apos;x&apos;"
    );
  }

  #[test]
  fn task_runs_at_normal_priority_and_on_battery() {
    let xml = task_xml(r"PC\alice", r"C:\Program Files\App\app.exe");

    assert!(xml.contains("<Priority>4</Priority>"));
    assert!(xml.contains(
      "<DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>"
    ));
    assert!(xml
      .contains("<StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>"));
    assert!(xml.contains("<ExecutionTimeLimit>PT0S</ExecutionTimeLimit>"));
  }

  #[test]
  fn task_is_triggered_by_and_runs_as_the_user() {
    let xml = task_xml(r"PC\alice", r"C:\Program Files\App\app.exe");

    assert_eq!(xml.matches(r"<UserId>PC\alice</UserId>").count(), 2);
    assert!(xml.contains("<LogonTrigger>"));
    assert!(xml.contains("<RunLevel>LeastPrivilege</RunLevel>"));
    assert!(
      xml.contains(r#"<Command>"C:\Program Files\App\app.exe"</Command>"#)
    );
  }

  #[test]
  fn task_definition_escapes_its_values() {
    let xml = task_xml(r"PC\R&D", r"C:\Tools & Co\app.exe");

    assert!(xml.contains(r"<UserId>PC\R&amp;D</UserId>"));
    assert!(xml.contains(r#""C:\Tools &amp; Co\app.exe""#));
  }

  /// Registers a real scheduled task under a name of its own, so that the
  /// round trip through the Task Scheduler service is covered.
  #[test]
  fn registers_and_removes_the_scheduled_task() {
    let autostart = Autostart::new(
      "GlazeWM platform test",
      Path::new(r"C:\Windows\System32\cmd.exe"),
    )
    .unwrap();

    // Start from a clean slate in case an earlier run was interrupted.
    autostart.disable().unwrap();
    assert!(!autostart.is_enabled().unwrap());

    autostart.enable().unwrap();
    let is_enabled = autostart.is_enabled();

    // Clean up before asserting, so a failure doesn't leave the task
    // behind.
    autostart.disable().unwrap();

    assert!(is_enabled.unwrap());
    assert!(!autostart.is_enabled().unwrap());

    // Removing a task that doesn't exist is not an error.
    autostart.disable().unwrap();
  }
}
