namespace GlazeWM.Domain.UserConfigs
{
  public class GeneralConfig
  {
    /// <summary>
    /// Whether to show floating windows as always on top.
    /// </summary>
    public bool ShowFloatingOnTop { get; set; } = true;
    /// <summary>
    /// Center the cursor in the middle of a newly focused window
    /// </summary>
    public bool CursorFollowsFocus { get; set; } = false;
    /// <summary>
    /// Focus the window directly under the cursor at all times
    /// </summary>
    public bool FocusFollowsCursor { get; set; } = false;
  }
}
