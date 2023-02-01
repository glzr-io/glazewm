namespace GlazeWM.Domain.UserConfigs
{
  public class SystemTrayComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Expands to show both pinned and unppined icons.
    /// </summary>
    public string ExpandText { get; set; } = "";

    /// <summary>
    /// Collapse to show only pinned icons.
    /// </summary>
    public string CollapseText { get; set; } = "";

    /// <summary>
    /// Sets default icon font if one isn't specified.
    /// </summary>
    public SystemTrayComponentConfig()
    {
      FontFamily = "pack://application:,,,/Fonts/#Material Icons";
    }
  }
}
