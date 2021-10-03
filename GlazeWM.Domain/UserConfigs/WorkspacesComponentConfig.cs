namespace GlazeWM.Domain.UserConfigs
{
  public class WorkspacesComponentConfig : BarComponentConfig
  {
    public string FocusedWorkspaceBorderWidth { get; set; } = "0";
    public string FocusedWorkspaceBorderColor { get; set; } = "blue";
    public string FocusedWorkspaceBackground { get; set; } = "#8192B3";

    /// <summary>
    /// Fallback to component foreground config if unset.
    /// </summary>
    public string FocusedWorkspaceForeground { get; set; } = null;

    public string DisplayedWorkspaceBorderWidth { get; set; } = "0";
    public string DisplayedWorkspaceBorderColor { get; set; } = "blue";
    public string DisplayedWorkspaceBackground { get; set; } = "#42403e";

    /// <summary>
    /// Fallback to component foreground config if unset.
    /// </summary>
    public string DisplayedWorkspaceForeground { get; set; } = null;

    public string DefaultWorkspaceBorderWidth { get; set; } = "0";
    public string DefaultWorkspaceBorderColor { get; set; } = "blue";

    /// <summary>
    /// Fallback to component background config if unset.
    /// </summary>
    public string DefaultWorkspaceBackground { get; set; } = null;

    /// <summary>
    /// Fallback to component foreground config if unset.
    /// </summary>
    public string DefaultWorkspaceForeground { get; set; } = null;
  }
}
