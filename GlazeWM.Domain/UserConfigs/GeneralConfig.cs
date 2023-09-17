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
    /// If activated, by switching to the current workspace the previous focused workspace is activated.
    /// </summary>
    public bool ToggleWorkspaceOnRefocus { get; set; }
    /// <summary>
    /// Whether to enable window transition animations (on minimize, close, etc).
    /// </summary>
    public WindowAnimations WindowAnimations { get; set; } = WindowAnimations.Unchanged;
  }
}
