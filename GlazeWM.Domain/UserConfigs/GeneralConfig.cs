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
  }
}
