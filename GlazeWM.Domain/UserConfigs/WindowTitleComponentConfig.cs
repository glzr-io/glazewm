namespace GlazeWM.Domain.UserConfigs
{
  public class WindowTitleComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Label assigned to the window title component.
    /// </summary>
    public string Label { get; set; } = "{window_title}";
    public int MaxWindowTitleLength { get; set; } = 60;
  }
}
