namespace GlazeWM.Domain.UserConfigs
{
  public class GPUStatsComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Icon to represent GPU usage.
    /// </summary>
    public string LabelGPU { get; set; } = "";
    /// <summary>
    /// Sets default icon font if one isn't specified.
    /// </summary>
    public GPUStatsComponentConfig()
    {
      Padding = "0px 5px";
      FontFamily = "pack://application:,,,/Resources/#Font Awesome 6 Free Solid";
    }
  }
}
