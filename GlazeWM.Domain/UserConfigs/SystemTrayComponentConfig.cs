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
  }
}
