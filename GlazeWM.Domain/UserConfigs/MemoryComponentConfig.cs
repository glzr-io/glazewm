using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.UserConfigs
{
  public class MemoryComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Label/icon assigned to the RAM component.
    /// {0} is substituted by RAM percentage formatted using <see cref="PercentFormat"/>.
    /// {1} is substituted by the current value (divided by <see cref="DivideBy"/>)
    /// {2} is substituted by the max value (divided by <see cref="DivideBy"/>)
    /// </summary>
    public string StringFormat { get; set; } = "RAM {1}/{2} GB";

    /// <summary>
    /// Numerical Format to use for the Percentage.
    /// </summary>
    public string PercentFormat { get; set; } = "00";

    /// <summary>
    /// Numerical Format to use for the Current Value.
    /// </summary>
    public string CurrentValueFormat { get; set; } = "00.00";

    /// <summary>
    /// Numerical Format to use for the Max Value.
    /// </summary>
    public string MaxValueFormat { get; set; } = "00.00";

    /// <summary>
    /// How often this component refreshes in milliseconds.
    /// </summary>
    public int RefreshIntervalMs { get; set; } = 1000;

    /// <summary>
    /// Multiplies the value returned from the counter.
    /// </summary>
    public float DivideBy { get; set; } = 1000f * 1000f * 1000f;

    /// <summary>
    /// The value to pull for memory usage.
    /// </summary>
    public RamMeasurement Counter { get; set; } = RamMeasurement.PhysicalMemory;
  }
}
