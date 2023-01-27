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
    public bool CursorFollowsFocus { get; set; } = true;
  }
}
