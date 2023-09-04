namespace GlazeWM.Domain.UserConfigs
{
  public class SystemTrayComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Expands to show both pinned and unppined icons.
    /// </summary>
    public string LabelExpandText { get; set; } = "";

    /// <summary>
    /// Collapse to show only pinned icons.
    /// </summary>
    public string LabelCollapseText { get; set; } = "";

    /// <summary>
    /// Sets default icon font if one isn't specified.
    /// </summary>
    public SystemTrayComponentConfig()
    {
      FontFamily = "GlazeWM.App.Resources#Material Icons";
    }
  }
}
