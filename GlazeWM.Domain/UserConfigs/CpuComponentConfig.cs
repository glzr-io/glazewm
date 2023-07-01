namespace GlazeWM.Domain.UserConfigs
{
  public class CpuComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Label assigned to the CPU component.
    /// </summary>
    public string Label { get; set; } = "CPU {0}%";

    /// <summary>
    /// How often this component refreshes in milliseconds.
    /// </summary>
    public int RefreshIntervalMs { get; set; } = 1000;
  }
}
