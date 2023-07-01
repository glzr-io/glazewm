namespace GlazeWM.Domain.UserConfigs
{
  public class GpuComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Label assigned to the GPU component.
    /// </summary>
    public string Label { get; set; } = "GPU: {percent_usage}%";

    /// <summary>
    /// How often this component refreshes in milliseconds.
    /// </summary>
    public int RefreshIntervalMs { get; set; } = 1000;
  }
}
