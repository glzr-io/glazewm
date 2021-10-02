namespace GlazeWM.Domain.UserConfigs
{
  public class WorkspacesComponentConfig : BarComponentConfig
  {
    public string FocusedWorkspaceBorderWidth { get; set; } = "0";
    public string FocusedWorkspaceBorderColor { get; set; } = "blue";
    public string FocusedWorkspaceBackground { get; set; } = "#8192B3";
    public string FocusedWorkspaceForeground { get; set; } = "white";

    public string DisplayedWorkspaceBorderWidth { get; set; } = "0";
    public string DisplayedWorkspaceBorderColor { get; set; } = "blue";
    public string DisplayedWorkspaceBackground { get; set; } = "#42403e";
    public string DisplayedWorkspaceForeground { get; set; } = "white";

    public string DefaultWorkspaceBorderWidth { get; set; } = "0";
    public string DefaultWorkspaceBorderColor { get; set; } = "blue";
    public string DefaultWorkspaceBackground { get; set; } = "transparent";
    public string DefaultWorkspaceForeground { get; set; } = "white";
  }
}
