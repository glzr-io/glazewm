namespace GlazeWM.Domain.UserConfigs
{
  public class RAMStatsComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Icon to represent RAM usage.
    /// </summary>
    public string LabelRAM { get; set; } = "";
    /// <summary>
    /// Sets default icon font if one isn't specified.
    /// </summary>
    public RAMStatsComponentConfig()
    {
      Padding = "0px 5px";
      FontFamily = "pack://application:,,,/Resources/#Font Awesome 6 Free Solid";
    }
  }
}
