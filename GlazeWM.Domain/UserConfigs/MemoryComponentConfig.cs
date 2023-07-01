namespace GlazeWM.Domain.UserConfigs
{
  public class MemoryComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Label assigned to the memory component.
    /// </summary>
    public string Label { get; set; } = "RAM: {percent_usage}%";

    /// <summary>
    /// How often this component refreshes in milliseconds.
    /// </summary>
    public int RefreshIntervalMs { get; set; } = 1000;
  }
}
