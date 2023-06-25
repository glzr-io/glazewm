using System;
using System.Globalization;
using System.Management;
using Vostok.Sys.Metrics.PerfCounters;

namespace GlazeWM.Infrastructure.WindowsApi
{
  /// <summary>
  /// Provides access to current CPU statistics.
  /// </summary>
  public class CpuStatsService : IDisposable
  {
    private readonly IPerformanceCounter<double> _cpuCounter = PerformanceCounterFactory.Default.CreateCounter("Processor Information", "% Processor Utility", "_Total");
    private readonly IPerformanceCounter<double> _cpuFrequencyCurrent = PerformanceCounterFactory.Default.CreateCounter("Processor Information", "% Processor Performance", "_Total");
    private static int _baseFrequencyMhz = -1;
    private float _maxFrequencyMhz = -1;
    private float _maxPackagePower = -1;
    private float _maxCoreTemp = -1;

    public CpuStatsService()
    {
      GetMaxFrequency();
    }

    /// <inheritdoc />
    ~CpuStatsService() => Dispose();

    /// <inheritdoc />
    public void Dispose()
    {
      _cpuCounter.Dispose();
      GC.SuppressFinalize(this);
    }

    /// <summary>
    /// Returns the current CPU utilization as a percentage.
    /// </summary>
    /// <exception cref="ArgumentOutOfRangeException">Not a supported measurement.</exception>
    public CpuMeasurementResult GetMeasurement(CpuMeasurement measurement)
    {
      return measurement switch
      {
        CpuMeasurement.CpuUsage => new CpuMeasurementResult((float)_cpuCounter.Observe(), 100f),
        CpuMeasurement.CpuFrequency => GetResultWithMaxObservedValue((float)_cpuFrequencyCurrent.Observe() * _baseFrequencyMhz / 100f, ref _maxFrequencyMhz),
        CpuMeasurement.PackagePower => GetResultWithMaxObservedValue(LibreHardwareMonitorHelper.GetCpuPackagePower(), ref _maxPackagePower),
        CpuMeasurement.CoreTemp => GetResultWithMaxObservedValue(LibreHardwareMonitorHelper.GetCoreTemperature(), ref _maxCoreTemp),
        _ => throw new ArgumentOutOfRangeException(nameof(measurement), measurement, null)
      };
    }

    private static CpuMeasurementResult GetResultWithMaxObservedValue(float curValue, ref float maxValue)
    {
      if (curValue > maxValue)
        maxValue = curValue;

      return new CpuMeasurementResult(curValue, maxValue);
    }

    private static void GetMaxFrequency()
    {
      // WMI is slow but a necessary evil; we'll only init once; hopefully should be ok.
      if (_baseFrequencyMhz != -1)
        return;

      using var searcher = new ManagementObjectSearcher("SELECT * FROM Win32_Processor");
      foreach (var o in searcher.Get())
      {
        var obj = (ManagementObject)o;
        _baseFrequencyMhz = Convert.ToInt32(obj["MaxClockSpeed"], CultureInfo.InvariantCulture);
      }
    }

    /// <summary>
    /// Individual cpu counter measurement.
    /// </summary>
    /// <param name="CurrentValue">Current value for this counter.</param>
    /// <param name="MaxValue">Max value for this counter.</param>
    public record struct CpuMeasurementResult(float CurrentValue, float MaxValue)
    {
      /// <summary>
      /// Divides the items in the measurement by a specific value.
      /// </summary>
      /// <param name="divideBy">Number to divide by.</param>
      public void DivideBy(float divideBy)
      {
        CurrentValue /= divideBy;
        MaxValue /= divideBy;
      }
    }
  }

  /// <summary>
  /// The value to obtain measurements for.
  /// </summary>
  public enum CpuMeasurement
  {
    /// <summary>
    /// Current amount of physical RAM in use; i.e. working set.
    /// </summary>
    CpuUsage,

    /// <summary>
    /// Average frequency of the CPU.
    /// </summary>
    CpuFrequency,

    /// <summary>
    /// CPU Package/Full Die Power
    /// </summary>
    PackagePower,

    /// <summary>
    /// CPU Core Temperature
    /// </summary>
    CoreTemp,
  }
}
