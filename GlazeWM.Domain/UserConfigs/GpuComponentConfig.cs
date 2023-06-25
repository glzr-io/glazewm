using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.UserConfigs
{
  public class GpuComponentConfig : BarComponentConfig
  {
    /// <summary>
    /// Label/icon assigned to the GPU component.
    /// {0} is substituted by GPU percentage formatted using <see cref="NumberFormat"/>.
    /// </summary>
    public string StringFormat { get; set; } = "GPU {0}%";

    /// <summary>
    /// Numerical Format to use for the Percentage.
    /// </summary>
    public string NumberFormat { get; set; } = "00";

    /// <summary>
    /// How often this component refreshes in milliseconds.
    /// </summary>
    public int RefreshIntervalMs { get; set; } = 1000;

    /// <summary>
    /// What individual subsystems to measure for GPU stats.
    /// </summary>
    public GpuPerformanceCategoryFlags Flags { get; set; } = GpuPerformanceCategoryFlags.Graphics;
  }
}
