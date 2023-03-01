namespace GlazeWM.Domain.UserConfigs
{
  public class CPUStatsComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Icon to represent CPU usage.
    /// </summary>
    public string LabelCPU { get; set; } = "";
    /// Sets default icon font if one isn't specified.
    /// </summary>
    public CPUStatsComponentConfig()
    {
      Padding = "0px 5px";
      FontFamily = "pack://application:,,,/Resources/#Font Awesome 6 Free Solid";
    }
  }
}
