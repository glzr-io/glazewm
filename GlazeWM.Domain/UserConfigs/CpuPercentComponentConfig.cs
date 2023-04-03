using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.UserConfigs;

public class CpuPercentComponentConfig : BarComponentConfig
{
  /// <summary>
  /// Label/icon assigned to the CPU component.
  /// {0} is substituted by CPU percentage formatted using <see cref="NumberFormat"/>.
  /// </summary>
  public string StringFormat { get; set; } = "CPU {0}%";

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
  /// Multiplies the value returned from the counter.
  /// </summary>
  public float DivideBy { get; set; } = 1000f;

  /// <summary>
  /// How often this component refreshes in milliseconds.
  /// </summary>
  public int RefreshIntervalMs { get; set; } = 1000;

  /// <summary>
  /// The value to pull for RAM usage.
  /// </summary>
  public CpuMeasurement Counter { get; set; } = CpuMeasurement.CpuFrequency;
}
