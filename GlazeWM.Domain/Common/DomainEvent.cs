namespace GlazeWM.Domain.Common
{
  public static class DomainEvent
  {
    public const string ApplicationExiting = "application_exiting";
    public const string BindingModeChanged = "binding_mode_changed";
    public const string FocusChanged = "focus_changed";
    public const string MonitorAdded = "monitor_added";
    public const string MonitorRemoved = "monitor_removed";
    public const string TilingDirectionChanged = "tiling_direction_changed";
    public const string UserConfigReloaded = "user_config_reloaded";
    public const string WorkspaceActivated = "workspace_activated";
    public const string WorkspaceDeactivated = "workspace_deactivated";
    public const string WorkingAreaResized = "working_area_resized";
  }
}
