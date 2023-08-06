namespace GlazeWM.Domain.UserConfigs
{
  public class GeneralConfig
  {
    /// <summary>
    /// Whether to show floating windows as always on top.
    /// </summary>
    public bool ShowFloatingOnTop { get; set; }
    /// <summary>
    /// Center the cursor in the middle of a newly focused window
    /// TODO: Not officially released because implementation is buggy. Use at own risk.
    /// </summary>
    public bool CursorFollowsFocus { get; set; }
    /// <summary>
    /// Focus the window directly under the cursor at all times
    /// </summary>
    public bool FocusFollowsCursor { get; set; }
    /// <summary>
    /// Amount by which to move floating windows
    /// </summary>
    public string FloatingWindowMoveAmount { get; set; } = "5%";
    /// <summary>
    /// Color for border drawn around a focused window.
    /// </summary>
    public string FocusBorderColor { get; set; } = "#FFFFFFFF";
    /// <summary>
    /// If activated, by switching to the current workspace the previous focused workspace is activated.
    /// </summary>
    public bool ToggleWorkspaceOnRefocus { get; set; }
  }
}
