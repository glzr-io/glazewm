namespace GlazeWM.Domain.UserConfigs
{
  public class SystemTrayComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Expands to show both pinned and unppined icons.
    /// </summary>
    public string LabelExpandText { get; set; } = "<attr ff='pack://application:,,,/Resources/#Material Icons'></attr>";

    /// <summary>
    /// Collapse to show only pinned icons.
    /// </summary>
    public string LabelCollapseText { get; set; } = "<attr ff='pack://application:,,,/Resources/#Material Icons'></attr>";
  }
}
