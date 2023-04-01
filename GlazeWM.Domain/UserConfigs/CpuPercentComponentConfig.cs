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
  public string NumberFormat { get; set; } = "00";
}
