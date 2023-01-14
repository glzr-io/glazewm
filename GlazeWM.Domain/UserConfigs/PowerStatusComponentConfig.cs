namespace GlazeWM.Domain.UserConfigs
{
  public class PowerStatusComponentConfig : BarComponentConfig
  {
    public string Draining { get; set; } = " {battery_level}% ";
    public string Low { get; set; } = " {battery_level}% (low) ";
    public string Charging { get; set; } = " {battery_level}% (on power) ";
  }
}
