using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public class GPUStatsService : System.IDisposable
  {
    private readonly List<PerformanceCounter> _gpuCounters;

    public GPUStatsService()
    {
      var category = new PerformanceCounterCategory("GPU Engine");
      var counterNames = category.GetInstanceNames();

      _gpuCounters = counterNames
        .Where(counterName => counterName.EndsWith("engtype_3D"))
        .SelectMany(counterName => category.GetCounters(counterName))
        .Where(counter => counter.CounterName.Equals("Utilization Percentage"))
        .ToList();
    }

    public float GetCurrentUtilization()
    {
      return (float)_gpuCounters.Sum(x => x.NextValue());
    }

    public void Dispose()
    {
      _gpuCounters.ForEach(x => x.Dispose());
    }
  }
}
