using System;
using System.Diagnostics;

namespace GlazeWM.Infrastructure.WindowsApi;

/// <summary>
/// Provides access to current CPU statistics.
/// </summary>
public class CpuStatsService : System.IDisposable
{
  private readonly PerformanceCounter _cpuCounter = new("Processor Information", "% Processor Utility", "_Total");

  /// <inheritdoc />
  ~CpuStatsService() => Dispose();

  /// <inheritdoc />
  public void Dispose()
  {
    _cpuCounter?.Dispose();
    GC.SuppressFinalize(this);
  }

  /// <summary>
  /// Returns the current CPU utilization as a percentage.
  /// </summary>
  public float GetCurrentLoadPercent() => _cpuCounter.NextValue();
}
