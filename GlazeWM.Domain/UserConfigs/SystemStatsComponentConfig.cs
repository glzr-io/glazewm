namespace GlazeWM.Domain.UserConfigs
{
  public class SystemStatsComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Icon to represent CPU usage.
    /// </summary>
    public string LabelCPU { get; set; } = "";
    /// <summary>
    /// Icon to represent GPU usage.
    /// </summary>
    public string LabelGPU { get; set; } = "";
    /// <summary>
    /// Icon to represent RAM usage.
    /// </summary>
    public string LabelRAM { get; set; } = "";
    /// <summary>
    /// Sets default icon font if one isn't specified.
    /// </summary>
    public SystemStatsComponentConfig()
    {
      FontFamily = "pack://application:,,,/Resources/#Font Awesome 6 Free Solid";
    }
  }
}
