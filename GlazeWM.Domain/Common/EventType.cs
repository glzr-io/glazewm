namespace GlazeWM.Domain.Common
{
  public enum EventType
  {
    BindingModeChanged,
    FocusChanged,
    MonitorAdded,
    MonitorRemoved,
    TilingDirectionChanged,
    UserConfigReloaded,
    WorkspaceActivated,
    WorkspaceDeactivated,
    ApplicationExiting,
  }

  public static class EventTypeExtensions
  {
    public static string ToString(this EventType eventType)
    {
      return;
    }
  }

  public static class EventType
  {
    public static string BindingModeChanged = "binding_mode_changed";
    public static string FocusChanged = "focus_changed";
    public static string MonitorAdded = "monitor_added";
    public static string MonitorRemoved = "monitor_removed";
    public static string TilingDirectionChanged = "tiling_direction_changed";
    public static string UserConfigReloaded = "user_config_reloaded";
    public static string WorkspaceActivated = "workspace_activated";
    public static string WorkspaceDeactivated = "workspace_deactivated";
    public static string ApplicationExiting = "application_exiting";
  }
}
