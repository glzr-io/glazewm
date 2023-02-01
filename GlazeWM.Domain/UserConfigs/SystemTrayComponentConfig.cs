namespace GlazeWM.Domain.UserConfigs
{
  public class SystemTrayComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Text to display.
    /// </summary>
    public string ExpandText { get; set; } = "";

    /// <summary>
    /// 
    /// </summary>
    public string CollapseText { get; set; } = "";
  }
}