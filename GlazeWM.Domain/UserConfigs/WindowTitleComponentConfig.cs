namespace GlazeWM.Domain.UserConfigs
{
  public class WindowTitleComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Label assigned to the window title component.
    /// </summary>
    public string Label { get; set; } = "{window_title}";

    /// <summary>
    /// The maximum length after which the window title will be truncated; if set to -1, the title will not be truncated.
    /// </summary>
    public int MaxTitleLength { get; set; } = 60;
  }
}
